use std::{fs, path::Path};

use multi_link_core::{
    FakeProcessProbe, FixtureProcessProbe, Liveness, Options, RepoMode, Scanner,
};
use multi_link_model::{State, StateEvidence, Timestamp};
use serde_json::{Value, json};

const ID: &str = "aaaaaaaa-0000-4000-8000-000000000001";
const BEFORE: &str = "2026-11-09T08:00:00Z";
const END: &str = "2026-11-09T08:01:00Z";
const AFTER: &str = "2026-11-09T08:02:00Z";

fn write(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn transcript(root: &Path, suffix: &[Value]) {
    let first = json!({"type":"user", "sessionId":ID, "cwd":"/repo", "timestamp":BEFORE,
        "entrypoint":"cli", "message":{"content":"work"}});
    let text = std::iter::once(&first)
        .chain(suffix)
        .map(|v| format!("{v}\n"))
        .collect::<String>();
    write(root, &format!(".claude/projects/project/{ID}.jsonl"), &text);
}

fn hook(kind: &str, at: &str) -> Value {
    let mut value = json!({"v":1,"kind":kind,"at":at,"session_id":ID,"agent_id":null,
        "event":"PermissionRequest","tool_use_id":null,"cwd":"/repo",
        "child":null,"child_evidence":null,"notification_type":null});
    if kind == "ended" {
        value["event"] = json!("SessionEnd");
        value["reason"] = json!("prompt_input_exit");
    } else if kind == "handoff" {
        value["event"] = json!("PostToolUse");
        value["tool_use_id"] = json!("tool-1");
    }
    value
}

fn hooks(root: &Path, records: &[Value]) {
    write(
        root,
        &format!(".multi-link/hook/{ID}.jsonl"),
        &records.iter().map(|v| format!("{v}\n")).collect::<String>(),
    );
}

fn options(root: &Path) -> Options<'static> {
    Options {
        state_dir: root.join(".multi-link"),
        now: Timestamp::parse("2026-11-09T12:00:00Z").unwrap(),
        since: None,
        repo_mode: RepoMode::Fixture,
        probe: &FixtureProcessProbe,
    }
}

fn assert_state(
    scanner: &mut Scanner,
    root: &Path,
    options: &Options<'_>,
    expected: (State, StateEvidence),
) {
    let output = scanner.sessions(root, options).unwrap();
    assert!(
        output.report.skipped.is_empty(),
        "{:?}",
        output.report.skipped
    );
    let session = output.report.sessions.iter().find(|s| s.id == ID).unwrap();
    assert_eq!((session.state, session.state_evidence), expected);
    if matches!(expected.0, State::Finished | State::Unknown) {
        assert_eq!(session.state_since, None);
    }
}

#[test]
fn session_end_precedence_record_process_exit_end_transcript_exit_waiting() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let mut scanner = Scanner::default();
    let mut opts = options(root);
    transcript(root, &[json!({"type":"cost-state"})]);
    hooks(root, &[hook("waiting", BEFORE), hook("ended", END)]);
    assert_state(
        &mut scanner,
        root,
        &opts,
        (State::Finished, StateEvidence::SessionEnd),
    );
    let record_path = ".claude/sessions/123.json";
    write(root, record_path, &json!({"pid":123,"procStart":"synthetic-start","sessionId":ID,
        "cwd":"/repo","status":"idle","statusUpdatedAt":1794211200000_i64,"startedAt":1794211200000_i64,"updatedAt":1794211260000_i64}).to_string());
    assert_state(
        &mut scanner,
        root,
        &opts,
        (State::Idle, StateEvidence::SessionRecord),
    );
    let mut probe = FakeProcessProbe::default();
    probe
        .results
        .insert((123, "synthetic-start".into()), Liveness::Gone);
    opts.probe = &probe;
    assert_state(
        &mut scanner,
        root,
        &opts,
        (State::Finished, StateEvidence::ProcessExit),
    );
    fs::remove_file(root.join(record_path)).unwrap();
    hooks(root, &[hook("waiting", AFTER)]);
    assert_state(
        &mut scanner,
        root,
        &opts,
        (State::Finished, StateEvidence::TranscriptExit),
    );
    transcript(root, &[]);
    assert_state(
        &mut scanner,
        root,
        &opts,
        (State::Waiting, StateEvidence::Hook),
    );
    assert_eq!(
        scanner.sessions(root, &opts).unwrap().report.sessions[0].state_since,
        Some(Timestamp::parse(AFTER).unwrap())
    );
}

#[test]
fn late_async_ended_append_does_not_override_later_turn_or_hook() {
    for later_kind in ["waiting", "handoff", "ended", "turn"] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let mut scanner = Scanner::default();
        let opts = options(root);
        let suffix = if later_kind == "turn" {
            vec![json!({"type":"assistant","timestamp":AFTER,"message":{"content":[]}})]
        } else {
            vec![]
        };
        transcript(root, &suffix);
        let later = if later_kind == "turn" {
            vec![]
        } else {
            vec![hook(later_kind, AFTER)]
        };
        hooks(root, &later);
        // Warm the cache before the older async line lands last in append order.
        scanner.sessions(root, &opts).unwrap();
        hooks(root, &[later, vec![hook("ended", END)]].concat());
        let expected = match later_kind {
            "waiting" => (State::Waiting, StateEvidence::Hook),
            "ended" => (State::Finished, StateEvidence::SessionEnd),
            _ => (State::Unknown, StateEvidence::NoSignal),
        };
        assert_state(&mut scanner, root, &opts, expected);
    }
}

#[test]
fn ended_reader_validates_fields_and_ignores_invalid_later_records() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    transcript(root, &[]);
    let valid = hook("ended", END);
    let mut invalid = Vec::new();
    for field in valid.as_object().unwrap().keys() {
        let mut missing = valid.clone();
        missing.as_object_mut().unwrap().remove(field);
        invalid.push(missing);
        let mut wrong = valid.clone();
        wrong[field] = json!([]);
        invalid.push(wrong);
    }
    for (field, value) in [
        ("agent_id", json!("agent")),
        ("tool_use_id", json!("tool")),
        ("child", json!(ID)),
        ("child_evidence", json!("stdout")),
        ("notification_type", json!("permission_prompt")),
        ("event", json!("Notification")),
        ("reason", Value::Null),
    ] {
        let mut wrong = valid.clone();
        wrong[field] = value;
        invalid.push(wrong);
    }
    let mut future = hook("waiting", AFTER);
    future["v"] = json!(2);
    invalid.push(future);
    hooks(root, &[vec![valid], invalid.clone()].concat());
    let output = Scanner::default().sessions(root, &options(root)).unwrap();
    assert_eq!(
        output.report.sessions[0].state_evidence,
        StateEvidence::SessionEnd
    );
    assert_eq!(
        serde_json::to_value(output.report.skipped).unwrap(),
        json!(
            (2..=invalid.len() + 1)
                .map(|line| json!({"file":format!(".multi-link/hook/{ID}.jsonl"),
            "line":line,"reason":"unknown_type"}))
                .collect::<Vec<_>>()
        )
    );
}

#[test]
fn phase9_session_end_and_clear_match_claude_keys_on_cold_and_warm_passes() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let root = fixtures.join("sessions-phase9");
    let key: Value =
        serde_json::from_slice(&fs::read(fixtures.join("expected/phase9/sessions.json")).unwrap())
            .unwrap();
    let mut scanner = Scanner::default();
    for _ in 0..2 {
        let output = scanner.sessions(&root, &options(&root)).unwrap();
        assert!(output.report.skipped.is_empty());
        for expected in key["sessions"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|s| s["id"].as_str().unwrap().starts_with("94"))
        {
            let actual = output
                .report
                .sessions
                .iter()
                .find(|s| s.id == expected["id"].as_str().unwrap())
                .unwrap();
            assert_eq!(serde_json::to_value(actual).unwrap(), *expected);
        }
    }
}

#[test]
fn non_turn_activity_does_not_clear_ended_but_every_turn_kind_does() {
    for (record, clears) in [
        (json!({"type":"assistant","message":{"content":[]}}), true),
        (json!({"type":"user","message":{"content":"resume"}}), true),
        (
            json!({"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t","content":"done"}]}}),
            true,
        ),
        (
            json!({"type":"user","isMeta":true,"message":{"content":"meta"}}),
            false,
        ),
        (
            json!({"type":"user","message":{"content":"<local-command-stdout>bye</local-command-stdout>"}}),
            false,
        ),
        (
            json!({"type":"user","message":{"content":"<command-name>/exit</command-name>"}}),
            false,
        ),
        (
            json!({"type":"assistant","sessionId":"other","message":{"content":[]}}),
            false,
        ),
        (json!({"type":"cost-state"}), false),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let mut record = record;
        record["timestamp"] = json!(AFTER);
        transcript(root, &[record]);
        // Subagent turns never clear a main-session ended record.
        write(
            root,
            &format!(".claude/projects/project/{ID}/subagents/agent-test.jsonl"),
            &format!(
                "{}\n",
                json!({"type":"assistant","sessionId":ID,"timestamp":AFTER,"message":{"content":[]}})
            ),
        );
        hooks(root, &[hook("ended", END)]);
        let expected = if clears {
            (State::Unknown, StateEvidence::NoSignal)
        } else {
            (State::Finished, StateEvidence::SessionEnd)
        };
        assert_state(&mut Scanner::default(), root, &options(root), expected);
    }
}

#[test]
fn ended_equal_timestamps_count_and_latest_ended_time_wins() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    transcript(
        root,
        &[json!({"type":"assistant","timestamp":END,"message":{"content":[]}})],
    );
    let mut arbitrary = hook("ended", END);
    arbitrary["reason"] = json!("future-reason");
    // Only strictly later times clear ended, even if a same-time waiting line is last.
    hooks(root, &[arbitrary, hook("waiting", END)]);
    assert_state(
        &mut Scanner::default(),
        root,
        &options(root),
        (State::Finished, StateEvidence::SessionEnd),
    );
    hooks(root, &[hook("ended", AFTER), hook("ended", BEFORE)]);
    assert_state(
        &mut Scanner::default(),
        root,
        &options(root),
        (State::Finished, StateEvidence::SessionEnd),
    );
}

#[test]
fn warm_scan_clears_finished_end_after_resume_turn() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let opts = options(root);
    let mut scanner = Scanner::default();
    transcript(root, &[]);
    hooks(root, &[hook("ended", END)]);
    assert_state(
        &mut scanner,
        root,
        &opts,
        (State::Finished, StateEvidence::SessionEnd),
    );
    assert!(matches!(
        multi_link_core::open::fresh_action(&mut scanner, root, &opts, ID, false).unwrap(),
        Some(multi_link_core::open::OpenAction::Resume { .. })
    ));
    transcript(
        root,
        &[json!({"type":"user","timestamp":AFTER,"message":{"content":"continue"}})],
    );
    assert_state(
        &mut scanner,
        root,
        &opts,
        (State::Unknown, StateEvidence::NoSignal),
    );
    assert!(matches!(
        multi_link_core::open::fresh_action(&mut scanner, root, &opts, ID, false).unwrap(),
        Some(multi_link_core::open::OpenAction::Unavailable {
            reason: multi_link_core::open::Reason::NotKnownFinished,
            ..
        })
    ));
}
