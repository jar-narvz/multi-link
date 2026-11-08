//! Step 8.4c: started-by links from the desktop app's session records (S1/S2).
//! Overview nesting is covered by later tasks; this file checks the data layer.

use std::{
    fs,
    path::{Path, PathBuf},
};

use multi_link_core::{FixtureProcessProbe, Options, RepoMode, Scanner, SessionsOutput};
use multi_link_model::Timestamp;
use serde_json::Value;

const DIR: &str = "Library/Application Support/Claude/claude-code-sessions";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn options(root: &Path) -> Options<'static> {
    Options {
        state_dir: root.join(".multi-link"),
        now: Timestamp::parse("2026-11-08T12:00:00Z").unwrap(),
        since: None,
        repo_mode: RepoMode::Fixture,
        probe: &FixtureProcessProbe,
    }
}

fn key(name: &str) -> Value {
    serde_json::from_slice(&fs::read(fixtures().join("expected/started").join(name)).unwrap())
        .unwrap()
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// A scratch copy of the fixture root (so tests can edit the app records).
fn scratch() -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("root");
    copy_tree(&fixtures().join("sessions-started"), &root);
    (temp, root)
}

fn started_by(output: &SessionsOutput) -> Vec<(String, Option<String>)> {
    output
        .report
        .sessions
        .iter()
        .map(|s| (s.id.clone(), s.started_by.clone()))
        .collect()
}

fn id(n: u32) -> String {
    format!("84c000{n:02}-0000-4000-8000-0000000084{n:02}")
}

fn first_record(root: &Path, n: u32) -> PathBuf {
    let name = format!("local_84cf00{n:02}-0000-4000-8000-0000000000{n:02}.json");
    walk(&root.join(DIR))
        .into_iter()
        .find(|p| p.file_name().unwrap().to_str().unwrap() == name)
        .unwrap()
}

fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else {
            out.push(path);
        }
    }
    out.sort();
    out
}

#[test]
fn sessions_match_the_key_on_cold_and_warm_passes() {
    let root = fixtures().join("sessions-started");
    let mut scanner = Scanner::default();
    let expected = key("sessions.json");
    for _ in 0..2 {
        let output = scanner.sessions(&root, &options(&root)).unwrap();
        assert_eq!(serde_json::to_value(&output.report).unwrap(), expected);
    }
}

#[test]
fn only_the_expected_sessions_have_started_by() {
    let root = fixtures().join("sessions-started");
    let output = Scanner::default().sessions(&root, &options(&root)).unwrap();
    let got: Vec<_> = started_by(&output)
        .into_iter()
        .filter_map(|(child, parent)| parent.map(|p| (child, p)))
        .collect();
    let mut want = vec![
        (id(2), id(1)),
        (id(3), id(1)),
        (id(4), id(2)),
        (id(8), id(9)),
        (id(10), id(1)),
    ];
    want.sort();
    let mut got = got;
    got.sort();
    assert_eq!(got, want);
}

#[test]
fn links_match_the_key_and_ignore_the_app_dir() {
    let root = fixtures().join("sessions-started");
    let mut scanner = Scanner::default();
    let links = scanner.links(&root, &options(&root)).unwrap();
    let actual = serde_json::to_value(&links.report).unwrap();
    assert_eq!(actual, key("links.json"));

    let (_temp, copy) = scratch();
    fs::remove_dir_all(copy.join(DIR)).unwrap();
    let without = Scanner::default().links(&copy, &options(&copy)).unwrap();
    assert_eq!(serde_json::to_value(&without.report).unwrap(), actual);
}

#[test]
fn bad_records_are_exactly_the_malformed_and_wrong_typed_files() {
    let root = fixtures().join("sessions-started");
    let output = Scanner::default().sessions(&root, &options(&root)).unwrap();
    let mut want = vec![
        walk(&root.join(DIR))
            .into_iter()
            .find(|p| p.to_str().unwrap().contains("local_84cf00e1"))
            .unwrap(),
        walk(&root.join(DIR))
            .into_iter()
            .find(|p| p.to_str().unwrap().contains("local_84cf0015"))
            .unwrap(),
    ]
    .into_iter()
    .map(|p| {
        p.strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .into_owned()
    })
    .collect::<Vec<_>>();
    want.sort();
    assert_eq!(output.bad_records, want);
    assert!(output.unreadable.is_empty());
}

#[test]
fn absent_app_dir_adds_no_diagnostic_and_clears_every_started_by() {
    let (_temp, root) = scratch();
    let mut scanner = Scanner::default();
    let with = scanner.sessions(&root, &options(&root)).unwrap();
    assert!(started_by(&with).iter().any(|(_, p)| p.is_some()));
    assert_eq!(with.bad_records.len(), 2);

    fs::remove_dir_all(root.join(DIR)).unwrap();
    let without = scanner.sessions(&root, &options(&root)).unwrap();
    assert!(started_by(&without).iter().all(|(_, p)| p.is_none()));
    assert!(without.bad_records.is_empty());
    assert!(without.unreadable.is_empty());
    assert!(scanner.diagnostics.is_empty());
}

#[test]
fn one_scanner_sees_records_added_changed_and_deleted_and_reuses_unchanged_files() {
    let (_temp, root) = scratch();
    let mut scanner = Scanner::default();
    let first = scanner.sessions(&root, &options(&root)).unwrap();
    let cold_files = scanner.stats.files;
    assert!(cold_files > 0);

    // Unchanged: nothing is parsed again.
    let again = scanner.sessions(&root, &options(&root)).unwrap();
    assert_eq!(scanner.stats.files, 0);
    assert_eq!(started_by(&again), started_by(&first));

    // Changed: B (03) is rewritten to name A (02) as its parent.
    let b = first_record(&root, 3);
    let text = fs::read_to_string(&b).unwrap();
    let rewritten = text.replace(
        "local_84cf0001-0000-4000-8000-000000000001",
        "local_84cf0002-0000-4000-8000-000000000002",
    );
    assert_ne!(text, rewritten);
    fs::write(&b, rewritten).unwrap();
    let changed = scanner.sessions(&root, &options(&root)).unwrap();
    assert_eq!(scanner.stats.files, 1);
    let parent_of = |out: &SessionsOutput, n: u32| {
        started_by(out)
            .into_iter()
            .find(|(child, _)| *child == id(n))
            .unwrap()
            .1
    };
    assert_eq!(parent_of(&changed, 3), Some(id(2)));

    // Deleted: A2's (04) record goes, so A2 is no longer started by A.
    fs::remove_file(first_record(&root, 4)).unwrap();
    let deleted = scanner.sessions(&root, &options(&root)).unwrap();
    assert_eq!(parent_of(&deleted, 4), None);
    assert_eq!(parent_of(&deleted, 3), Some(id(2)));

    // Added: a record for Q1 (18) naming K (01) as its parent, parent app id unique.
    let dir = first_record(&root, 2).parent().unwrap().to_path_buf();
    fs::write(
        dir.join("local_84cf0099-0000-4000-8000-000000000099.json"),
        r#"{"sessionId":"local_84cf0099-0000-4000-8000-000000000099","cliSessionId":"84c00018-0000-4000-8000-000000008418","spawnedFrom":{"sessionId":"local_84cf0001-0000-4000-8000-000000000001"}}"#,
    )
    .unwrap();
    // Q1's cliSessionId already has a record (21 in b0) with no parent: the new
    // record is the only one naming a parent, so the link is made.
    let added = scanner.sessions(&root, &options(&root)).unwrap();
    assert_eq!(scanner.stats.files, 1);
    assert_eq!(parent_of(&added, 18), Some(id(1)));
}

#[cfg(unix)]
#[test]
fn unreadable_app_dir_is_an_unreadable_diagnostic_not_an_error() {
    use std::os::unix::fs::PermissionsExt;
    let (_temp, root) = scratch();
    let dir = root.join(DIR).join("00000000-0000-4000-8000-0000000084a0");
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o000)).unwrap();
    let output = Scanner::default().sessions(&root, &options(&root));
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
    if fs::read_dir(&dir).is_ok() && output.as_ref().is_ok_and(|o| o.unreadable.is_empty()) {
        return; // running as a user that ignores modes
    }
    let output = output.unwrap();
    assert_eq!(output.unreadable.len(), 1);
    assert!(output.unreadable[0].file.ends_with("84a0"));
    assert!(
        output
            .report
            .sessions
            .iter()
            .all(|s| s.started_by.is_none())
    );
}

fn overview(root: &Path) -> Value {
    let mut plans = |folder: &Path| {
        multi_link_plan::steps_report(root, &[folder.to_string_lossy().into_owned()])
            .repos
            .remove(0)
    };
    let output = Scanner::default()
        .overview(root, &options(root), &mut plans)
        .unwrap();
    serde_json::to_value(output.report).unwrap()
}

fn repo_rows<'a>(overview: &'a Value, repo: &str) -> &'a Vec<Value> {
    overview["repos"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["repo"] == repo)
        .unwrap()["rows"]
        .as_array()
        .unwrap()
}

/// Every session row in `rows`, at any depth and inside merged rows.
fn all_rows(rows: &[Value], out: &mut Vec<Value>) {
    for row in rows {
        if row["kind"] == "merged" {
            all_rows(row["rows"].as_array().unwrap(), out);
        } else {
            out.push(row.clone());
            all_rows(row["children"].as_array().unwrap(), out);
        }
    }
}

#[test]
fn overview_matches_the_key_on_cold_and_warm_passes() {
    let root = fixtures().join("sessions-started");
    let mut plans = |folder: &Path| {
        multi_link_plan::steps_report(&root, &[folder.to_string_lossy().into_owned()])
            .repos
            .remove(0)
    };
    let mut scanner = Scanner::default();
    let expected = key("overview.json");
    for _ in 0..2 {
        let output = scanner
            .overview(&root, &options(&root), &mut plans)
            .unwrap();
        assert_eq!(serde_json::to_value(&output.report).unwrap(), expected);
    }
}

#[test]
fn nesting_is_k_a_codex_a2_and_k_b_and_each_session_appears_once() {
    let root = fixtures().join("sessions-started");
    let overview = overview(&root);
    let rows = repo_rows(&overview, "/home/dev/repo-k");
    let k = rows
        .iter()
        .find(|row| row["id"] == id(1).as_str())
        .expect("K is a visible top-level row (A2 is busy)");
    let kids: Vec<_> = k["children"].as_array().unwrap().iter().collect();
    assert_eq!(
        kids.iter().map(|c| c["id"].clone()).collect::<Vec<_>>(),
        [id(2), id(3)]
    );
    let a: Vec<_> = kids[0]["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(
        a,
        ["84c000c2-0000-4000-8000-0000000084c2".to_owned(), id(4)]
    );

    let mut seen = Vec::new();
    for group in overview["repos"].as_array().unwrap() {
        let mut flat = Vec::new();
        all_rows(group["rows"].as_array().unwrap(), &mut flat);
        seen.extend(flat.iter().map(|r| r["id"].as_str().unwrap().to_owned()));
    }
    let listed: Vec<_> = Scanner::default()
        .sessions(&root, &options(&root))
        .unwrap()
        .report
        .sessions
        .iter()
        .map(|s| s.id.clone())
        .collect();
    seen.sort();
    let mut listed = listed;
    listed.sort();
    assert_eq!(seen, listed);
}

#[test]
fn k_is_not_quiet_merged_while_a2_is_busy_and_hand_off_counts_ignore_the_app_dir() {
    let root = fixtures().join("sessions-started");
    let with = overview(&root);
    let find = |overview: &Value, n: &str| {
        let mut flat = Vec::new();
        all_rows(repo_rows(overview, "/home/dev/repo-k"), &mut flat);
        flat.into_iter().find(|r| r["id"] == n).unwrap()
    };
    let top_ids: Vec<_> = repo_rows(&with, "/home/dev/repo-k")
        .iter()
        .filter(|r| r["kind"] == "session")
        .map(|r| r["id"].clone())
        .collect();
    assert!(top_ids.contains(&Value::String(id(1))));

    let (_temp, copy) = scratch();
    fs::remove_dir_all(copy.join(DIR)).unwrap();
    let without = overview(&copy);
    // A (02) hands off to a Codex child: its `made` and the child's `handoffs` are
    // non-empty, so a started-by edge leaking into either would show here.
    let child = "84c000c2-0000-4000-8000-0000000084c2";
    let a = id(2);
    assert!(
        !find(&with, &a)["made"].as_array().unwrap().is_empty(),
        "A made at least one hand-off"
    );
    assert!(
        !find(&with, child)["handoffs"]
            .as_array()
            .unwrap()
            .is_empty(),
        "the Codex child has a hand-off"
    );
    assert_eq!(find(&with, &a)["made"], find(&without, &a)["made"]);
    assert_eq!(
        find(&with, child)["handoffs"],
        find(&without, child)["handoffs"]
    );
    assert_eq!(
        with["repos"][0]["latest_handoff"],
        without["repos"][0]["latest_handoff"]
    );
    let links = |root: &Path| {
        let out = Scanner::default().links(root, &options(root)).unwrap();
        serde_json::to_value(&out.report).unwrap()
    };
    assert_eq!(links(&root), links(&copy));
}

#[test]
fn a_session_with_only_a_record_has_no_started_by_and_does_not_nest() {
    let (_temp, root) = scratch();
    let a2 = walk(&root.join(".claude/projects"))
        .into_iter()
        .find(|p| p.to_str().unwrap().contains("84c00004"))
        .unwrap();
    fs::remove_file(a2).unwrap();
    let output = Scanner::default().sessions(&root, &options(&root)).unwrap();
    let a2_row = output
        .report
        .sessions
        .iter()
        .find(|s| s.id == id(4))
        .expect("A2 is still listed from its live record");
    assert!(a2_row.transcript.is_none());
    assert_eq!(a2_row.started_by, None);
    let overview = overview(&root);
    let mut flat = Vec::new();
    all_rows(repo_rows(&overview, "/home/dev/repo-k"), &mut flat);
    let a = flat.iter().find(|r| r["id"] == id(2).as_str()).unwrap();
    let kids: Vec<_> = a["children"].as_array().unwrap().iter().collect();
    assert!(kids.iter().all(|c| c["id"] != id(4).as_str()));
    // A2 is no longer anyone's child.
    let k = flat.iter().find(|r| r["id"] == id(1).as_str()).unwrap();
    for child in k["children"].as_array().unwrap() {
        assert!(
            child["children"]
                .as_array()
                .unwrap()
                .iter()
                .all(|c| c["id"] != id(4).as_str())
        );
    }
}

#[test]
fn a_cycle_with_a_member_outside_the_window_keeps_every_member_top_level() {
    let (_temp, root) = scratch();
    let app_id = |n: u32| format!("local_84cf00{n:02}-0000-4000-8000-0000000000{n:02}");
    let reparent = |n: u32, from: u32, to: u32| {
        let path = first_record(&root, n);
        let text = fs::read_to_string(&path).unwrap();
        // Only the parent named in `spawnedFrom`, not the record's own `sessionId`.
        let rewritten = text.replace(
            &format!("\"sessionId\": \"{}\",\n    \"taskId\"", app_id(from)),
            &format!("\"sessionId\": \"{}\",\n    \"taskId\"", app_id(to)),
        );
        assert_ne!(text, rewritten);
        fs::write(path, rewritten).unwrap();
    };
    // A 3-cycle C1 (12) -> C2 (13) -> S (14) -> C1, where each names the next as parent.
    reparent(13, 12, 14);
    reparent(14, 14, 12);
    // Push S outside a 1 day window; C1 and C2 stay inside it.
    let s = walk(&root.join(".claude/projects"))
        .into_iter()
        .find(|p| p.to_str().unwrap().contains("84c00014"))
        .unwrap();
    let text = fs::read_to_string(&s).unwrap();
    fs::write(&s, text.replace("2026-11-08T", "2026-10-01T")).unwrap();
    let started = |since| {
        let mut opts = options(&root);
        opts.since = since;
        let output = Scanner::default().sessions(&root, &opts).unwrap();
        started_by(&output)
            .into_iter()
            .filter(|(who, _)| [id(12), id(13), id(14)].contains(who))
            .collect::<Vec<_>>()
    };
    let all = started(None);
    assert_eq!(all.len(), 3);
    assert!(all.iter().all(|(_, parent)| parent.is_none()));
    let windowed = started(Some(std::time::Duration::from_secs(24 * 3600)));
    assert_eq!(windowed.len(), 2, "S is outside the window");
    assert!(windowed.iter().all(|(_, parent)| parent.is_none()));
}

#[test]
fn removing_the_app_dir_puts_every_row_back_at_top_level() {
    let (_temp, root) = scratch();
    fs::remove_dir_all(root.join(DIR)).unwrap();
    let overview = overview(&root);
    let mut flat = Vec::new();
    for group in overview["repos"].as_array().unwrap() {
        all_rows(group["rows"].as_array().unwrap(), &mut flat);
    }
    for row in &flat {
        assert_eq!(row["started_by"], Value::Null);
        for child in row["children"].as_array().unwrap() {
            assert_eq!(child["harness"], "codex", "only Codex children remain");
        }
    }
    let mut rows = Vec::new();
    all_rows(repo_rows(&overview, "/home/dev/repo-k"), &mut rows);
    let a = rows.iter().find(|r| r["id"] == id(2).as_str()).unwrap();
    assert_eq!(a["children"].as_array().unwrap().len(), 1);
}
