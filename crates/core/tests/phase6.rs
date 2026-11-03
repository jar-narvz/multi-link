use multi_link_core::{FixtureProcessProbe, Options, RepoMode, Scanner};
use multi_link_model::{State, StateEvidence, Timestamp};
use serde_json::{Value, json};
use std::{fs, path::Path, time::Duration};

fn stamp(s: &str) -> Timestamp {
    Timestamp::parse(s).unwrap()
}
fn options(root: &Path) -> Options<'static> {
    Options {
        state_dir: root.join(".multi-link"),
        now: stamp("2026-11-03T06:25:00Z"),
        since: Some(Duration::from_secs(7200)),
        repo_mode: RepoMode::Fixture,
        probe: &FixtureProcessProbe,
    }
}
fn strip(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for field in ["activity", "asked", "last_asked", "imported", "state_since"] {
                map.remove(field);
            }
            map.values_mut().for_each(strip);
        }
        Value::Array(items) => items.iter_mut().for_each(strip),
        _ => {}
    }
}
#[test]
fn every_existing_json_key_preserves_committed_values() {
    let before: Value = serde_json::from_str(include_str!("keys-before-phase6.json")).unwrap();
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/expected");
    for (name, original) in before.as_object().unwrap() {
        let mut current: Value =
            serde_json::from_slice(&fs::read(fixtures.join(name)).unwrap()).unwrap();
        let mut original = original.clone();
        strip(&mut current);
        strip(&mut original);
        assert_eq!(current, original, "{name}");
    }
}
#[test]
fn phase6_sessions_and_overview_match_keys_on_cold_and_warm_passes() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let root = fixtures.join("sessions-phase6");
    let mut scanner = Scanner::default();
    let mut plans = |folder: &Path| {
        multi_link_plan::steps_report(&root, &[folder.to_string_lossy().into_owned()])
            .repos
            .remove(0)
    };
    for _ in 0..2 {
        let output = scanner
            .overview(&root, &options(&root), &mut plans)
            .unwrap();
        for (name, actual) in [
            (
                "sessions",
                serde_json::to_value(output.sessions.report).unwrap(),
            ),
            ("overview", serde_json::to_value(output.report).unwrap()),
        ] {
            let expected: Value = serde_json::from_slice(
                &fs::read(fixtures.join(format!("expected/phase6/{name}.json"))).unwrap(),
            )
            .unwrap();
            assert_eq!(actual, expected, "{name}");
        }
    }
}
fn write_lines(path: &Path, lines: &[Value]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        lines.iter().map(|v| format!("{v}\n")).collect::<String>(),
    )
    .unwrap();
}
fn event(kind: &str, id: &str, at: &str) -> Value {
    json!({"type":"event_msg","timestamp":at,"payload":{"type":kind,"turn_id":id,"message":"Synthetic real turn"}})
}
#[test]
fn cached_import_gains_real_work_on_second_scanner_pass() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let path = root.join(".codex/sessions/rollout-cache.jsonl");
    let mut lines = vec![
        json!({"type":"session_meta","timestamp":"2026-11-03T06:00:00Z","payload":{"id":"cache","cwd":"/synthetic","timestamp":"2026-11-03T06:00:00Z"}}),
        event("task_started", "external-import-a", "2026-11-03T06:01:00Z"),
        event("user_message", "external-import-a", "2026-11-03T06:01:01Z"),
        event("task_complete", "external-import-a", "2026-11-03T06:01:02Z"),
        event("thread_settings_applied", "", "2026-11-03T06:02:00Z"),
    ];
    write_lines(&path, &lines);
    let mut scanner = Scanner::default();
    let first = scanner
        .sessions(root, &options(root))
        .unwrap()
        .report
        .sessions
        .remove(0);
    assert!(first.imported);
    assert_eq!(first.activity, vec![]);
    assert_eq!(first.asked, None);
    assert_eq!(first.state_since, None);
    lines.extend([
        event("task_started", "real", "2026-11-03T06:20:00Z"),
        event("user_message", "real", "2026-11-03T06:20:01Z"),
        event("task_complete", "real", "2026-11-03T06:21:00Z"),
    ]);
    write_lines(&path, &lines);
    let next = scanner
        .sessions(root, &options(root))
        .unwrap()
        .report
        .sessions
        .remove(0);
    assert!(!next.imported);
    assert_eq!(next.asked.as_deref(), Some("Synthetic real turn"));
    assert_eq!(next.state_since, Some(stamp("2026-11-03T06:21:00Z")));
    assert_eq!(next.activity.len(), 2);
    assert_eq!(next.activity[0].from, stamp("2026-11-03T06:02:00Z"));
    assert_eq!(next.activity[1].to, stamp("2026-11-03T06:21:00Z"));
}
#[test]
fn subagent_only_activity_ignores_inherited_history_and_updates_warm_cache() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let path = root.join(".claude/projects/test/parent.jsonl");
    write_lines(
        &path,
        &[
            json!({"type":"user","sessionId":"parent","cwd":"/synthetic","timestamp":"2026-11-02T01:00:00Z","message":{"content":"Synthetic parent"}}),
            json!({"type":"user","sessionId":"inherited","timestamp":"2026-11-03T06:23:00Z","message":{"content":"Synthetic inherited"}}),
        ],
    );
    let sub = root.join(".claude/projects/test/parent/subagents/agent-child.jsonl");
    write_lines(
        &sub,
        &[json!({"type":"assistant","sessionId":"parent","timestamp":"2026-11-03T06:20:00Z"})],
    );
    let mut scanner = Scanner::default();
    for end in ["2026-11-03T06:20:00Z", "2026-11-03T06:21:00Z"] {
        write_lines(
            &sub,
            &[json!({"type":"assistant","sessionId":"parent","timestamp":end})],
        );
        let session = scanner
            .sessions(root, &options(root))
            .unwrap()
            .report
            .sessions
            .remove(0);
        assert_eq!(
            session.activity,
            vec![multi_link_model::Activity {
                from: stamp(end),
                to: stamp(end)
            }]
        );
        assert_eq!(session.last_activity, stamp("2026-11-03T06:23:00Z"));
    }
}
#[test]
fn state_since_follows_winning_record_hook_and_codex_evidence() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let record = root.join(".claude/sessions/1.json");
    let at = stamp("2026-11-03T06:00:00Z");
    fs::create_dir_all(record.parent().unwrap()).unwrap();
    let mut scanner = Scanner::default();
    for (status, expected) in [("waiting", Some(at)), ("busy", Some(at)), ("idle", None)] {
        fs::write(&record, json!({"pid":1,"procStart":"synthetic","sessionId":"record","cwd":"/synthetic","status":status,
            "statusUpdatedAt":at.epoch_ms(),"startedAt":at.epoch_ms(),"updatedAt":at.epoch_ms()}).to_string()).unwrap();
        let s = scanner
            .sessions(root, &options(root))
            .unwrap()
            .report
            .sessions
            .remove(0);
        assert_eq!(s.state_since, expected);
        assert_eq!(
            s.state,
            match status {
                "waiting" => State::Waiting,
                "busy" => State::Stale,
                _ => State::Idle,
            }
        );
    }
    fs::remove_file(record).unwrap();
    let id = "00000000-0000-4000-8000-000000006200";
    write_lines(
        &root.join(format!(".claude/projects/test/{id}.jsonl")),
        &[
            json!({"type":"user","sessionId":id,"cwd":"/synthetic","timestamp":"2026-11-03T06:00:00Z","message":{"content":"Synthetic waiting"}}),
        ],
    );
    write_lines(
        &root.join(format!(".multi-link/hook/{id}.jsonl")),
        &[
            json!({"v":1,"kind":"waiting","at":"2026-11-03T06:10:00Z","session_id":id,"agent_id":null,"event":"PermissionRequest","tool_use_id":null,"cwd":"/synthetic","child":null,"child_evidence":null,"notification_type":null}),
        ],
    );
    let s = scanner
        .sessions(root, &options(root))
        .unwrap()
        .report
        .sessions
        .remove(0);
    assert_eq!(
        (s.state, s.state_evidence, s.state_since),
        (
            State::Waiting,
            StateEvidence::Hook,
            Some(stamp("2026-11-03T06:10:00Z"))
        )
    );
}

#[test]
fn latest_prompt_reaches_overview_after_cached_transcript_changes() {
    for harness in ["claude", "codex"] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let path = root.join(if harness == "claude" {
            ".claude/projects/test/synthetic.jsonl"
        } else {
            ".codex/sessions/rollout-synthetic.jsonl"
        });
        let mut lines = if harness == "claude" {
            Vec::new()
        } else {
            vec![
                json!({"type":"session_meta","timestamp":"2026-11-03T06:00:00Z","payload":{"id":"synthetic","cwd":"/synthetic","timestamp":"2026-11-03T06:00:00Z"}}),
            ]
        };
        let mut scanner = Scanner::default();
        for text in ["Synthetic first prompt", "Synthetic second prompt"] {
            lines.push(if harness == "claude" {
                json!({"type":"user","sessionId":"synthetic","cwd":"/synthetic","timestamp":"2026-11-03T06:01:00Z","message":{"content":text}})
            } else {
                json!({"type":"event_msg","timestamp":"2026-11-03T06:01:00Z","payload":{"type":"user_message","message":text}})
            });
            write_lines(&path, &lines);
            let mut plans = |folder: &Path| {
                multi_link_plan::steps_report(root, &[folder.to_string_lossy().into_owned()])
                    .repos
                    .remove(0)
            };
            // Both a changed-file refresh and a warm cache pass retain the latest prompt.
            for _ in 0..2 {
                let output = scanner.overview(root, &options(root), &mut plans).unwrap();
                let session = &output.sessions.report.sessions[0];
                assert_eq!(session.asked.as_deref(), Some("Synthetic first prompt"));
                assert_eq!(session.last_asked.as_deref(), Some(text));
                let row = match &output.report.repos[0].rows[0] {
                    multi_link_model::Row::Session(row) => row,
                    _ => panic!("expected one session"),
                };
                assert_eq!(row.asked, session.asked);
                assert_eq!(row.last_asked.as_deref(), Some(text));
            }
        }
    }
}
