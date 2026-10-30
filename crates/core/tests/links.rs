use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use multi_link_core::{FixtureProcessProbe, Options, RepoMode, links_report};
use multi_link_model::{
    Candidate, InvocationForm, Link, LinkEvidence, LinksReport, Match, Timestamp,
};
use serde_json::{Value, json};

const A: &str = "00000000-0000-4000-8000-000000000001";
const B: &str = "00000000-0000-4000-8000-000000000002";
const C: &str = "00000000-0000-4000-8000-000000000003";
const RECENT: &str = "2026-10-29T23:00:00Z";
const OLD: &str = "2026-10-29T20:00:00Z";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}
fn expected(name: &str) -> LinksReport {
    serde_json::from_slice(&fs::read(fixtures().join("expected").join(name)).unwrap()).unwrap()
}
fn options(since: Option<Duration>) -> Options<'static> {
    Options {
        now: Timestamp::parse("2026-10-29T23:15:00Z").unwrap(),
        since,
        repo_mode: RepoMode::Fixture,
        probe: &FixtureProcessProbe,
    }
}
fn report(root: &Path) -> LinksReport {
    links_report(root, &options(None)).unwrap().report
}
fn write(root: &Path, name: &str, text: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
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
fn hide_session_lines(value: &mut Value) {
    match value {
        Value::String(text) => {
            *text = text
                .split('\n')
                .map(|line| {
                    if line.trim().starts_with("session id: ") {
                        "<stdout>"
                    } else {
                        line
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
        }
        Value::Array(values) => values.iter_mut().for_each(hide_session_lines),
        Value::Object(values) => values.values_mut().for_each(hide_session_lines),
        _ => {}
    }
}
fn hide_ambiguous_results(directory: &Path) {
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            hide_ambiguous_results(&entry.path());
        } else if entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.starts_with("11bf1b58-") || name.starts_with("f4d83303-"))
        {
            let text = fs::read_to_string(entry.path()).unwrap();
            let mut result = String::new();
            for line in text.lines() {
                let mut value: Value = serde_json::from_str(line).unwrap();
                hide_session_lines(&mut value);
                result.push_str(&value.to_string());
                result.push('\n');
            }
            fs::write(entry.path(), result).unwrap();
        }
    }
}

#[test]
fn fixture_report_equals_answer_key() {
    assert_eq!(report(&fixtures().join("sessions")), expected("links.json"));
}

#[test]
fn ambiguous_variant_equals_answer_key() {
    let temp = tempfile::tempdir().unwrap();
    copy_tree(&fixtures().join("sessions"), temp.path());
    hide_ambiguous_results(&temp.path().join(".claude/projects"));
    assert_eq!(report(temp.path()), expected("links-amb-hidden.json"));
}

#[test]
fn missing_background_output_falls_back_to_heuristic() {
    let temp = tempfile::tempdir().unwrap();
    copy_tree(&fixtures().join("sessions"), temp.path());
    fs::remove_file(temp.path().join("task-output/be01y5dv6.output")).unwrap();
    let mut expected = expected("links.json");
    let link = expected
        .links
        .iter_mut()
        .find(|link| link.parent.starts_with("07c80c2e"))
        .unwrap();
    link.r#match = Match::Heuristic;
    link.evidence = None;
    link.candidates = vec![Candidate {
        id: "01a189e3-f419-79a1-b801-38f3fecd7480".into(),
        gap_ms: 474,
    }];
    assert_eq!(report(temp.path()), expected);
}

fn call(root: &Path, parent: &str, at: &str, command: &str, result: &str) {
    let records = [
        json!({"type":"assistant", "sessionId":parent, "cwd":"/repo", "timestamp":at,
            "message":{"content":[{"type":"tool_use", "name":"Bash", "id":parent,
                "input":{"command":command}}]}}),
        json!({"type":"user", "sessionId":parent, "cwd":"/repo", "timestamp":at,
            "message":{"content":[{"type":"tool_result", "tool_use_id":parent, "content":result}]}}),
    ];
    write(
        root,
        &format!(".claude/projects/project/{parent}.jsonl"),
        &records
            .iter()
            .map(|value| format!("{value}\n"))
            .collect::<String>(),
    );
}
fn child(root: &Path, id: &str, at: &str, cwd: &str) {
    let record = json!({"type":"session_meta", "timestamp":at,
        "payload":{"id":id, "cwd":cwd, "timestamp":at}});
    write(
        root,
        &format!(".codex/sessions/rollout-{id}.jsonl"),
        &format!("{record}\n"),
    );
}
fn link(
    parent: &str,
    at: &str,
    form: InvocationForm,
    child: Option<&str>,
    evidence: Option<LinkEvidence>,
    candidates: Vec<Candidate>,
) -> Link {
    Link {
        parent: parent.into(),
        invocation: parent.into(),
        subagent: None,
        form,
        background: false,
        at: Timestamp::parse(at).unwrap(),
        checkout: "/repo".into(),
        r#match: if evidence.is_some() {
            Match::Exact
        } else {
            match candidates.len() {
                0 => Match::Unlinked,
                1 => Match::Heuristic,
                _ => Match::Ambiguous,
            }
        },
        child: child.map(str::to_owned),
        evidence,
        candidates,
    }
}
fn candidate(id: &str, gap_ms: i64) -> Candidate {
    Candidate {
        id: id.into(),
        gap_ms,
    }
}

#[test]
fn moved_launch_directory_disables_heuristic_but_preserves_exact() {
    for command in [
        "cd x && codex exec x",
        "codex exec -C x x",
        "codex exec --cd x x",
    ] {
        let temp = tempfile::tempdir().unwrap();
        call(temp.path(), "parent", RECENT, command, "");
        child(temp.path(), A, RECENT, "/repo");
        assert_eq!(
            report(temp.path()),
            LinksReport {
                links: vec![link(
                    "parent",
                    RECENT,
                    InvocationForm::Exec,
                    None,
                    None,
                    vec![]
                )],
                unparented: vec![A.into()],
            }
        );
        call(
            temp.path(),
            "parent",
            RECENT,
            command,
            &format!("session id: {A}"),
        );
        assert_eq!(
            report(temp.path()),
            LinksReport {
                links: vec![link(
                    "parent",
                    RECENT,
                    InvocationForm::Exec,
                    Some(A),
                    Some(LinkEvidence::Stdout),
                    vec![]
                )],
                unparented: vec![],
            }
        );
    }
}

#[test]
fn resume_does_not_claim_child_from_earlier_exec() {
    let temp = tempfile::tempdir().unwrap();
    call(temp.path(), "exec", RECENT, "codex exec x", "");
    call(
        temp.path(),
        "resume",
        "2026-10-29T23:01:00Z",
        &format!("codex resume {A}"),
        "",
    );
    child(temp.path(), A, "2026-10-29T23:00:01Z", "/repo");
    assert_eq!(
        report(temp.path()),
        LinksReport {
            links: vec![
                link(
                    "exec",
                    RECENT,
                    InvocationForm::Exec,
                    Some(A),
                    None,
                    vec![candidate(A, 1000)]
                ),
                link(
                    "resume",
                    "2026-10-29T23:01:00Z",
                    InvocationForm::Resume,
                    Some(A),
                    Some(LinkEvidence::ResumeArg),
                    vec![]
                ),
            ],
            unparented: vec![],
        }
    );
}

#[test]
fn heuristic_window_is_inclusive_and_checkout_is_an_exact_string() {
    let temp = tempfile::tempdir().unwrap();
    call(temp.path(), "parent", RECENT, "codex exec x", "");
    for (id, at, cwd) in [
        (A, RECENT, "/repo"),
        (B, "2026-10-29T23:01:00Z", "/repo"),
        (C, "2026-10-29T23:00:00Z", "/repo/"),
        ("before", "2026-10-29T22:59:59.999Z", "/repo"),
        ("after", "2026-10-29T23:01:00.001Z", "/repo"),
    ] {
        child(temp.path(), id, at, cwd);
    }
    assert_eq!(
        report(temp.path()),
        LinksReport {
            links: vec![link(
                "parent",
                RECENT,
                InvocationForm::Exec,
                None,
                None,
                vec![candidate(A, 0), candidate(B, 60_000)]
            )],
            unparented: vec![
                A.into(),
                B.into(),
                C.into(),
                "after".into(),
                "before".into()
            ],
        }
    );
}

#[test]
fn since_lists_links_when_either_endpoint_is_visible_and_only_visible_unparented() {
    for (parent_at, child_at) in [(OLD, RECENT), (RECENT, OLD)] {
        let temp = tempfile::tempdir().unwrap();
        call(
            temp.path(),
            "parent",
            parent_at,
            "codex exec x",
            &format!("session id: {A}"),
        );
        child(temp.path(), A, child_at, "/repo");
        child(temp.path(), B, OLD, "/repo");
        child(temp.path(), C, RECENT, "/repo");
        assert_eq!(
            links_report(temp.path(), &options(Some(Duration::from_secs(3600))))
                .unwrap()
                .report,
            LinksReport {
                links: vec![link(
                    "parent",
                    parent_at,
                    InvocationForm::Exec,
                    Some(A),
                    Some(LinkEvidence::Stdout),
                    vec![]
                )],
                unparented: vec![C.into()],
            }
        );
    }
}

#[test]
fn old_exact_link_excludes_child_from_recent_heuristic_even_when_hidden() {
    let temp = tempfile::tempdir().unwrap();
    call(
        temp.path(),
        "old",
        OLD,
        "codex exec x",
        &format!("session id: {A}"),
    );
    call(temp.path(), "recent", RECENT, "codex exec x", "");
    // A recent start can have an old last_activity: the output window uses activity,
    // while matching uses the session_meta start. Keep both endpoints of the exact link hidden.
    child(temp.path(), A, RECENT, "/repo");
    let path = temp
        .path()
        .join(format!(".codex/sessions/rollout-{A}.jsonl"));
    let mut text = fs::read_to_string(&path).unwrap();
    text.push_str(&format!(
        "{}\n",
        json!({"type":"event_msg", "timestamp":OLD, "payload":{}})
    ));
    fs::write(path, text).unwrap();
    assert_eq!(
        links_report(temp.path(), &options(Some(Duration::from_secs(3600))))
            .unwrap()
            .report,
        LinksReport {
            links: vec![link(
                "recent",
                RECENT,
                InvocationForm::Exec,
                None,
                None,
                vec![]
            )],
            unparented: vec![]
        }
    );
}

#[test]
fn directory_task_output_falls_back_like_missing_output_in_real_mode() {
    let temp = tempfile::tempdir().unwrap();
    let output_path = temp.path().join("named.output");
    call(
        temp.path(),
        "parent",
        RECENT,
        "codex exec x",
        &format!("Output is being written to: {}", output_path.display()),
    );
    child(temp.path(), A, "2026-10-29T23:00:01Z", "/repo");
    let mut opts = options(None);
    opts.repo_mode = RepoMode::Real;
    let missing = links_report(temp.path(), &opts).unwrap().report;
    fs::create_dir(&output_path).unwrap();
    let directory = links_report(temp.path(), &opts).unwrap().report;
    assert_eq!(directory, missing);
    assert_eq!(
        directory,
        LinksReport {
            links: vec![link(
                "parent",
                RECENT,
                InvocationForm::Exec,
                Some(A),
                None,
                vec![candidate(A, 1000)]
            )],
            unparented: vec![],
        }
    );
}

#[test]
fn exact_evidence_precedence_and_real_task_path() {
    let temp = tempfile::tempdir().unwrap();
    let output_path = temp.path().join("named.output");
    fs::write(&output_path, format!("session id: {C}\n")).unwrap();
    let task = format!("Output is being written to: {}", output_path.display());
    let ready = format!("[codex] Turn ready ({A})\n[codex] Thread ready ({B})\n{task}");
    for (command, result, id, evidence, form) in [
        (
            format!("codex resume {A}"),
            format!("session id: {C}\n{ready}"),
            A,
            LinkEvidence::ResumeArg,
            InvocationForm::Resume,
        ),
        (
            "codex exec x".into(),
            format!("session id: {A}\n{ready}"),
            A,
            LinkEvidence::Stdout,
            InvocationForm::Exec,
        ),
        (
            "codex exec x".into(),
            ready,
            B,
            LinkEvidence::ThreadReady,
            InvocationForm::Exec,
        ),
        (
            "codex exec x".into(),
            task,
            C,
            LinkEvidence::TaskOutput,
            InvocationForm::Exec,
        ),
    ] {
        call(temp.path(), "parent", RECENT, &command, &result);
        let mut opts = options(None);
        opts.repo_mode = RepoMode::Real;
        assert_eq!(
            links_report(temp.path(), &opts).unwrap().report,
            LinksReport {
                links: vec![link(
                    "parent",
                    RECENT,
                    form,
                    Some(id),
                    Some(evidence),
                    vec![]
                )],
                unparented: vec![],
            }
        );
    }
}
