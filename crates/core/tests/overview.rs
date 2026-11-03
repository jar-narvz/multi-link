use std::{fs, path::Path};

use multi_link_core::{FixtureProcessProbe, Options, RepoMode, Scanner};
use multi_link_model::Timestamp;
use serde_json::Value;

#[test]
fn phase4_overview_matches_answer_key() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let root = fixtures.join("sessions-phase4");
    let options = Options {
        state_dir: root.join(".multi-link"),
        now: Timestamp::parse("2026-10-29T23:15:00Z").unwrap(),
        since: None,
        repo_mode: RepoMode::Fixture,
        probe: &FixtureProcessProbe,
    };
    let mut plans = |folder: &Path| {
        multi_link_plan::steps_report(&root, &[folder.to_string_lossy().into_owned()])
            .repos
            .remove(0)
    };
    let output = Scanner::default()
        .overview(&root, &options, &mut plans)
        .unwrap();
    let expected: Value =
        serde_json::from_slice(&fs::read(fixtures.join("expected/phase4/overview.json")).unwrap())
            .unwrap();
    assert_eq!(serde_json::to_value(output.report).unwrap(), expected);
}

fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let path = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &path);
        } else {
            fs::copy(entry.path(), path).unwrap();
        }
    }
}

#[test]
fn every_overview_matches_its_key() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    for (source, key, now) in [
        ("sessions", "overview.json", "2026-10-29T23:15:00Z"),
        ("sessions", "hook/overview.json", "2026-10-29T23:15:00Z"),
        (
            "sessions-phase5",
            "phase5/overview.json",
            "2026-10-29T23:20:00Z",
        ),
    ] {
        let temp = tempfile::tempdir().unwrap();
        copy_tree(&fixtures.join(source), temp.path());
        if key.starts_with("hook/") {
            copy_tree(&fixtures.join("hook"), temp.path());
            fs::remove_file(temp.path().join("task-output/be01y5dv6.output")).unwrap();
        }
        let root = temp.path();
        let options = Options {
            state_dir: root.join(".multi-link"),
            now: Timestamp::parse(now).unwrap(),
            since: None,
            repo_mode: RepoMode::Fixture,
            probe: &FixtureProcessProbe,
        };
        let mut plans = |folder: &Path| {
            multi_link_plan::steps_report(root, &[folder.to_string_lossy().into_owned()])
                .repos
                .remove(0)
        };
        let mut scanner = Scanner::default();
        let expected: Value =
            serde_json::from_slice(&fs::read(fixtures.join("expected").join(key)).unwrap())
                .unwrap();
        for _ in 0..2 {
            let output = scanner.overview(root, &options, &mut plans).unwrap();
            assert_eq!(
                serde_json::to_value(output.report).unwrap(),
                expected,
                "{key}"
            );
        }
    }
}

#[test]
fn older_keys_only_add_made_and_hook() {
    fn remove_added(value: &mut Value) {
        match value {
            Value::Object(object) => {
                for field in ["activity", "asked", "last_asked", "imported", "state_since"] {
                    object.remove(field);
                }
                object.remove("hook");
                object.remove("made");
                for child in object.values_mut() {
                    remove_added(child);
                }
            }
            Value::Array(items) => items.iter_mut().for_each(remove_added),
            _ => {}
        }
    }
    // Captured from the committed keys before Step 5.2; no generated answers.
    let before: Value = serde_json::from_str(include_str!("overview-before-phase5.json")).unwrap();
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/expected");
    for (name, original) in before.as_object().unwrap() {
        let mut current: Value =
            serde_json::from_slice(&fs::read(fixtures.join(name)).unwrap()).unwrap();
        remove_added(&mut current);
        assert_eq!(&current, original, "{name}");
    }
}

#[test]
fn hook_summary_ignores_window_skips_invalid_and_resets_after_removal() {
    use serde_json::json;
    use std::time::Duration;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let dir = root.join(".multi-link/hook");
    let options = Options {
        state_dir: root.join(".multi-link"),
        now: Timestamp::parse("2026-10-29T23:15:00Z").unwrap(),
        since: Some(Duration::from_secs(1)),
        repo_mode: RepoMode::Fixture,
        probe: &FixtureProcessProbe,
    };
    let mut scanner = Scanner::default();
    let mut plans = |_: &Path| panic!("no sessions need a plan");
    let zero = json!({"records":0,"latest":null});
    let output = scanner.overview(root, &options, &mut plans).unwrap();
    assert_eq!(serde_json::to_value(output.report.hook).unwrap(), zero);
    assert!(scanner.diagnostics.is_empty());
    fs::create_dir_all(&dir).unwrap();
    let output = scanner.overview(root, &options, &mut plans).unwrap();
    assert_eq!(serde_json::to_value(output.report.hook).unwrap(), zero);
    let id = "00000000-0000-4000-8000-000000000001";
    let record = |at: &str| {
        json!({"v":1,"kind":"waiting","at":at,"session_id":id,
        "agent_id":null,"event":"PermissionRequest","tool_use_id":null,"cwd":"/repo",
        "child":null,"child_evidence":null,"notification_type":null})
    };
    let file = dir.join(format!("{id}.jsonl"));
    fs::write(
        &file,
        format!(
            "{}\n{}\n{{\"v\":2}}\ninvalid\n{}",
            record("2026-10-27T10:00:00Z"),
            record("2026-10-26T10:00:00Z"),
            record("2026-10-28T10:00:00Z")
        ),
    )
    .unwrap();
    // The final record has no newline and remains pending.
    let expected = json!({"records":2,"latest":"2026-10-27T10:00:00.000Z"});
    for _ in 0..2 {
        let output = scanner.overview(root, &options, &mut plans).unwrap();
        assert_eq!(serde_json::to_value(output.report.hook).unwrap(), expected);
        assert!(output.report.repos.is_empty());
    }
    fs::remove_file(file).unwrap();
    fs::remove_dir(&dir).unwrap();
    // ENOTDIR exercises a directory read failure even when tests run as root.
    fs::write(&dir, "not a directory").unwrap();
    let output = scanner.overview(root, &options, &mut plans).unwrap();
    assert_eq!(serde_json::to_value(output.report.hook).unwrap(), zero);
    assert_eq!(scanner.diagnostics.len(), 1);
    assert!(scanner.diagnostics[0].starts_with("hook directory: "));
    fs::remove_file(&dir).unwrap();
    let output = scanner.overview(root, &options, &mut plans).unwrap();
    assert_eq!(serde_json::to_value(output.report.hook).unwrap(), zero);
    assert!(scanner.diagnostics.is_empty());
}

#[test]
fn unreadable_hook_directory_is_zero_with_a_diagnostic() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let dir = root.join(".multi-link/hook");
    fs::create_dir_all(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o000)).unwrap();
    let options = Options {
        state_dir: root.join(".multi-link"),
        now: Timestamp::parse("2026-10-29T23:15:00Z").unwrap(),
        since: None,
        repo_mode: RepoMode::Fixture,
        probe: &FixtureProcessProbe,
    };
    let mut scanner = Scanner::default();
    let output = scanner.overview(root, &options, &mut |_: &Path| panic!("no sessions"));
    // Restore access before asserting, so failures also leave temp cleanup possible.
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(
        output.unwrap().report.hook,
        multi_link_model::HookSummary::default()
    );
    assert_eq!(scanner.diagnostics.len(), 1);
    assert!(scanner.diagnostics[0].starts_with("hook directory: "));
}
