use std::{fs, path::PathBuf};

use multi_link_codex::discover;
use multi_link_model::{Harness, SessionsReport, SkipReason, Skipped, Timestamp};
use serde_json::{Value, json};
use tempfile::TempDir;

const T0: &str = "2026-10-29T00:00:00Z";
const T1: &str = "2026-10-29T00:00:01Z";
const T2: &str = "2026-10-29T00:00:02Z";

fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(relative)
}

fn meta() -> Value {
    json!({"type": "session_meta", "timestamp": T0,
        "payload": {"id": "child", "cwd": "/repo", "timestamp": T0}})
}

fn event(kind: &str, turn: &str, at: &str) -> Value {
    json!({"type": "event_msg", "timestamp": at,
        "payload": {"type": kind, "turn_id": turn, "started_at": 0}})
}

fn write_rollout(root: &TempDir, relative: &str, records: &[Value]) {
    let path = root.path().join(".codex/sessions").join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let text: String = records.iter().map(|value| format!("{value}\n")).collect();
    fs::write(path, text).unwrap();
}

#[test]
fn fixture_sessions_turns_and_skips_match_answer_key() {
    let expected: SessionsReport =
        serde_json::from_slice(&fs::read(fixture_path("expected/sessions.json")).unwrap()).unwrap();
    let actual = discover(&fixture_path("sessions")).unwrap();
    assert!(actual.unreadable.is_empty());
    let expected_sessions: Vec<_> = expected
        .sessions
        .iter()
        .filter(|session| session.harness == Harness::Codex)
        .collect();
    assert_eq!(expected_sessions.len(), 9);
    assert_eq!(actual.rollouts.len(), expected_sessions.len());
    for expected in expected_sessions {
        let actual = actual
            .rollouts
            .iter()
            .find(|r| r.id == expected.id)
            .unwrap();
        assert_eq!(actual.id, expected.id);
        assert_eq!(Some(&actual.transcript), expected.transcript.as_ref());
        assert_eq!(actual.cwd, expected.cwd);
        assert_eq!(actual.branch, expected.branch);
        assert_eq!(actual.started_at, expected.started_at);
        assert_eq!(actual.last_activity, expected.last_activity);
    }
    let resumed = actual
        .rollouts
        .iter()
        .find(|r| r.id == "01a189e2-d57f-7a91-b74c-ae85abd1a6a1")
        .unwrap();
    assert_eq!(resumed.turns.len(), 2);
    assert_eq!(resumed.last_turn_complete(), Some(true));
    assert_eq!(
        resumed
            .turns
            .iter()
            .map(|turn| (turn.started_at, turn.completed_at))
            .collect::<Vec<_>>(),
        vec![
            (
                Timestamp::parse("2026-10-29T22:58:27.430Z").unwrap(),
                Some(Timestamp::parse("2026-10-29T22:58:36.167Z").unwrap())
            ),
            (
                Timestamp::parse("2026-10-29T22:58:54.438Z").unwrap(),
                Some(Timestamp::parse("2026-10-29T22:58:58.954Z").unwrap())
            ),
        ]
    );
    let expected_skipped: Vec<_> = expected
        .skipped
        .into_iter()
        .filter(|skip| skip.file.starts_with(".codex/"))
        .collect();
    assert_eq!(actual.skipped, expected_skipped);
}

#[test]
fn unknown_types_are_counted_and_later_lines_are_read() {
    let root = TempDir::new().unwrap();
    write_rollout(
        &root,
        "rollout-a.jsonl",
        &[
            meta(),
            json!({"type": "future"}),
            json!({}),
            json!({"type": 1}),
            json!(null),
            event("task_started", "t", T1),
            event("task_complete", "t", T2),
        ],
    );
    let actual = discover(root.path()).unwrap();
    assert_eq!(
        actual.rollouts[0].last_activity,
        Timestamp::parse(T2).unwrap()
    );
    assert_eq!(actual.rollouts[0].last_turn_complete(), Some(true));
    assert_eq!(
        actual.skipped,
        (2..=5)
            .map(|line| Skipped {
                file: ".codex/sessions/rollout-a.jsonl".into(),
                line,
                reason: SkipReason::UnknownType,
            })
            .collect::<Vec<_>>()
    );
}

#[test]
fn open_last_turn_is_not_complete() {
    let root = TempDir::new().unwrap();
    write_rollout(
        &root,
        "rollout-a.jsonl",
        &[
            meta(),
            event("task_started", "t", T1),
            event("task_complete", "other", T2),
        ],
    );
    let actual = discover(root.path()).unwrap();
    assert_eq!(actual.rollouts[0].last_turn_complete(), Some(false));
    assert_eq!(actual.rollouts[0].turns[0].completed_at, None);
}

#[test]
fn completed_last_turn_governs_over_earlier_unmatched_turn() {
    let root = TempDir::new().unwrap();
    write_rollout(
        &root,
        "rollout-a.jsonl",
        &[
            meta(),
            event("task_started", "interrupted", T2),
            event("task_started", "latest", T0),
            event("task_complete", "latest", T1),
        ],
    );
    let actual = discover(root.path()).unwrap();
    let rollout = &actual.rollouts[0];
    assert_eq!(
        rollout
            .turns
            .iter()
            .map(|t| t.turn_id.as_str())
            .collect::<Vec<_>>(),
        vec!["interrupted", "latest"]
    );
    assert_eq!(rollout.turns[0].completed_at, None);
    assert_eq!(
        rollout.turns[1].completed_at,
        Some(Timestamp::parse(T1).unwrap())
    );
    assert_eq!(rollout.last_turn_complete(), Some(true));
    assert_eq!(rollout.last_activity, Timestamp::parse(T1).unwrap());
}

#[test]
fn missing_sessions_directory_is_empty() {
    let root = TempDir::new().unwrap();
    let actual = discover(root.path()).unwrap();
    assert!(actual.rollouts.is_empty());
    assert!(actual.skipped.is_empty());
    assert!(actual.unreadable.is_empty());
}

#[test]
fn first_metadata_wins_and_bad_timestamps_are_ignored() {
    let root = TempDir::new().unwrap();
    let mut first = meta();
    first["payload"]["timestamp"] = json!("invalid");
    let parent = json!({"type": "session_meta", "timestamp": T2,
        "payload": {"id": "parent", "cwd": "/parent", "git": {"branch": "parent"}}});
    write_rollout(
        &root,
        "rollout-a.jsonl",
        &[first, parent, event("task_started", "bad", "invalid")],
    );
    let actual = discover(root.path()).unwrap();
    let rollout = &actual.rollouts[0];
    assert_eq!(rollout.id, "child");
    assert_eq!(rollout.cwd, "/repo");
    assert_eq!(rollout.branch, None);
    assert_eq!(rollout.started_at, Timestamp::parse(T0).unwrap());
    assert_eq!(rollout.last_activity, Timestamp::parse(T2).unwrap());
    assert_eq!(rollout.last_turn_complete(), None);
}

#[test]
fn unusable_first_metadata_does_not_hide_skips_or_use_replayed_metadata() {
    let root = TempDir::new().unwrap();
    for (name, first) in [
        (
            "rollout-a.jsonl",
            json!({"type": "session_meta", "payload": {"cwd": "/repo"}}),
        ),
        (
            "rollout-b.jsonl",
            json!({"type": "session_meta", "payload": {"id": "child"}}),
        ),
        (
            "rollout-c.jsonl",
            json!({"type": "session_meta", "payload": null}),
        ),
    ] {
        write_rollout(&root, name, &[first, json!({"type": "future"}), meta()]);
    }
    let actual = discover(root.path()).unwrap();
    assert!(actual.rollouts.is_empty());
    assert_eq!(
        actual.skipped,
        ["a", "b", "c"].map(|name| Skipped {
            file: format!(".codex/sessions/rollout-{name}.jsonl"),
            line: 2,
            reason: SkipReason::UnknownType,
        })
    );
}

#[test]
fn discovery_is_recursive_sorted_and_filters_filenames() {
    let root = TempDir::new().unwrap();
    for name in [
        "z/rollout-b.jsonl",
        "a/nested/rollout-a.jsonl",
        "ignore.jsonl",
        "rollout-x.txt",
    ] {
        write_rollout(&root, name, &[meta()]);
    }
    let actual = discover(root.path()).unwrap();
    assert_eq!(
        actual
            .rollouts
            .iter()
            .map(|r| r.transcript.as_str())
            .collect::<Vec<_>>(),
        vec![
            ".codex/sessions/a/nested/rollout-a.jsonl",
            ".codex/sessions/z/rollout-b.jsonl",
        ]
    );
}

#[test]
fn blank_and_invalid_lines_are_reported_but_partial_tail_is_buffered() {
    let root = TempDir::new().unwrap();
    write_rollout(&root, "rollout-a.jsonl", &[meta()]);
    let path = root.path().join(".codex/sessions/rollout-a.jsonl");
    fs::write(
        &path,
        format!(
            "{}\n\ninvalid\n{}\n{{partial",
            meta(),
            event("task_started", "t", T1)
        ),
    )
    .unwrap();
    let actual = discover(root.path()).unwrap();
    assert_eq!(actual.rollouts[0].last_turn_complete(), Some(false));
    assert_eq!(
        actual.skipped,
        vec![
            Skipped {
                file: ".codex/sessions/rollout-a.jsonl".into(),
                line: 2,
                reason: SkipReason::EmptyLine
            },
            Skipped {
                file: ".codex/sessions/rollout-a.jsonl".into(),
                line: 3,
                reason: SkipReason::InvalidJson
            },
        ]
    );
}
