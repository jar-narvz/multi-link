use std::{
    fs,
    path::{Path, PathBuf},
};

use multi_link_claude::scan;
use multi_link_model::{Harness, SessionsReport, SkipReason, Skipped, Timestamp};
use serde_json::{Value, json};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

#[test]
fn transcript_fields_and_skips_match_answer_key() {
    let expected: SessionsReport =
        serde_json::from_slice(&fs::read(fixtures().join("expected/sessions.json")).unwrap())
            .unwrap();
    let root = fixtures().join("sessions");
    let actual = scan(&root).unwrap();
    let sessions: Vec<_> = expected
        .sessions
        .iter()
        .filter(|session| session.harness == Harness::Claude && session.transcript.is_some())
        .collect();
    assert_eq!(sessions.len(), 13);
    assert_eq!(actual.transcripts.len(), 13);
    for expected in sessions {
        let transcript = actual
            .transcripts
            .iter()
            .find(|t| t.id == expected.id)
            .unwrap();
        assert_eq!(
            transcript.transcript,
            *expected.transcript.as_ref().unwrap()
        );
        assert_eq!(transcript.cwd.as_deref(), Some(expected.cwd.as_str()));
        assert_eq!(transcript.branch, expected.branch);
        assert_eq!(transcript.started_at, Some(expected.started_at));
        assert_eq!(transcript.last_activity, Some(expected.last_activity));
        assert_eq!(transcript.subagents, expected.subagents);
        assert_eq!(transcript.subagent_files.len(), expected.subagents.len());
        for file in &transcript.subagent_files {
            assert!(expected.subagents.contains(&file.id));
            let parent = Path::new(&transcript.transcript).with_extension("");
            assert_eq!(
                Path::new(&file.transcript),
                parent
                    .join("subagents")
                    .join(format!("agent-{}.jsonl", file.id))
            );
            assert!(root.join(&file.transcript).is_file());
        }
    }
    let expected_skips: Vec<_> = expected
        .skipped
        .into_iter()
        .filter(|skip| skip.file.starts_with(".claude/"))
        .collect();
    assert_eq!(expected_skips.len(), 3);
    assert_eq!(actual.skipped, expected_skips);
    assert!(
        actual
            .transcripts
            .windows(2)
            .all(|pair| pair[0].transcript < pair[1].transcript)
    );
}

#[test]
fn all_session_record_fields_match_source_files() {
    let root = fixtures().join("sessions");
    let actual = scan(&root).unwrap();
    assert_eq!(actual.records.len(), 5);
    assert_eq!(
        actual.records.iter().filter(|r| r.status == "busy").count(),
        1
    );
    assert_eq!(
        actual.records.iter().filter(|r| r.status == "idle").count(),
        4
    );
    assert_eq!(
        actual
            .records
            .iter()
            .map(|r| r.record.as_str())
            .collect::<Vec<_>>(),
        vec![
            ".claude/sessions/60243.json",
            ".claude/sessions/64044.json",
            ".claude/sessions/65362.json",
            ".claude/sessions/65681.json",
            ".claude/sessions/69401.json",
        ]
    );
    for record in actual.records {
        let source: Value =
            serde_json::from_slice(&fs::read(root.join(&record.record)).unwrap()).unwrap();
        assert_eq!(json!(record.pid), source["pid"]);
        assert_eq!(json!(record.proc_start), source["procStart"]);
        assert_eq!(json!(record.session_id), source["sessionId"]);
        assert_eq!(json!(record.cwd), source["cwd"]);
        assert_eq!(json!(record.status), source["status"]);
        for (actual, field) in [
            (record.status_updated_at, "statusUpdatedAt"),
            (record.started_at, "startedAt"),
            (record.updated_at, "updatedAt"),
        ] {
            assert_eq!(
                actual,
                Timestamp::from_epoch_ms(source[field].as_i64().unwrap()).unwrap()
            );
        }
    }
}

#[test]
fn missing_directories_are_empty() {
    let root = tempfile::tempdir().unwrap();
    for relative in [None, Some(".claude/projects"), Some(".claude/sessions")] {
        if let Some(relative) = relative {
            fs::create_dir_all(root.path().join(relative)).unwrap();
        }
        let actual = scan(root.path()).unwrap();
        assert!(actual.transcripts.is_empty());
        assert!(actual.records.is_empty());
        assert!(actual.bad_records.is_empty());
        assert!(actual.skipped.is_empty());
    }
}

#[test]
fn unknown_types_contribute_nothing_and_later_lines_are_read() {
    let root = tempfile::tempdir().unwrap();
    let path = ".claude/projects/project/main.jsonl";
    write(
        root.path(),
        path,
        concat!(
            "{\"type\":\"future\",\"cwd\":\"/ignored\",\"gitBranch\":\"ignored\",\"timestamp\":\"2020-01-01T00:00:00Z\"}\n",
            "{\"type\":\"user\",\"cwd\":\"/first\",\"gitBranch\":\"first\",\"sessionId\":\"inherited\",\"timestamp\":\"2026-10-29T02:00:00Z\"}\n",
            "{\"type\":\"assistant\",\"cwd\":\"/last\",\"gitBranch\":\"last\",\"timestamp\":\"2026-10-29T01:00:00Z\"}\n",
            "{\"type\":\"future\",\"cwd\":\"/ignored\",\"gitBranch\":\"ignored\",\"timestamp\":\"2030-01-01T00:00:00Z\"}\n",
        ),
    );
    let actual = scan(root.path()).unwrap();
    let transcript = &actual.transcripts[0];
    assert_eq!(transcript.id, "main");
    assert_eq!(transcript.cwd.as_deref(), Some("/last"));
    assert_eq!(transcript.branch.as_deref(), Some("last"));
    assert_eq!(
        transcript.started_at,
        Some(Timestamp::parse("2026-10-29T02:00:00Z").unwrap())
    );
    assert_eq!(
        transcript.last_activity,
        Some(Timestamp::parse("2026-10-29T01:00:00Z").unwrap())
    );
    assert_eq!(
        actual.skipped,
        vec![
            Skipped {
                file: path.into(),
                line: 1,
                reason: SkipReason::UnknownType
            },
            Skipped {
                file: path.into(),
                line: 4,
                reason: SkipReason::UnknownType
            },
        ]
    );
}

#[test]
fn subagents_are_sorted_scanned_and_do_not_change_parent_metadata() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        ".claude/projects/p/main.jsonl",
        "{\"type\":\"user\",\"cwd\":\"/main\",\"gitBranch\":\"main\",\"timestamp\":\"2026-10-29T00:00:00Z\"}\n",
    );
    let directory = ".claude/projects/p/main/subagents";
    write(
        root.path(),
        &format!("{directory}/agent-z.jsonl"),
        "{\"type\":\"future\"}\n\n{bad}\n",
    );
    write(
        root.path(),
        &format!("{directory}/agent-a.jsonl"),
        "{\"type\":\"assistant\",\"cwd\":\"/sub\",\"gitBranch\":\"sub\",\"timestamp\":\"2030-01-01T00:00:00Z\"}\n",
    );
    // Neither recursive main discovery nor arbitrary files in subagents are allowed.
    write(
        root.path(),
        ".claude/projects/p/nested/extra.jsonl",
        "{\"type\":\"future\"}\n",
    );
    write(root.path(), &format!("{directory}/other.jsonl"), "{bad}\n");
    let actual = scan(root.path()).unwrap();
    assert_eq!(actual.transcripts.len(), 1);
    let transcript = &actual.transcripts[0];
    assert_eq!(transcript.subagents, ["a", "z"]);
    assert_eq!(
        transcript
            .subagent_files
            .iter()
            .map(|file| file.id.as_str())
            .collect::<Vec<_>>(),
        ["a", "z"]
    );
    assert_eq!(transcript.cwd.as_deref(), Some("/main"));
    assert_eq!(transcript.branch.as_deref(), Some("main"));
    assert_eq!(
        transcript.started_at,
        Some(Timestamp::parse("2026-10-29T00:00:00Z").unwrap())
    );
    assert_eq!(transcript.last_activity, transcript.started_at);
    assert_eq!(
        actual.skipped,
        vec![
            Skipped {
                file: format!("{directory}/agent-z.jsonl"),
                line: 1,
                reason: SkipReason::UnknownType
            },
            Skipped {
                file: format!("{directory}/agent-z.jsonl"),
                line: 2,
                reason: SkipReason::EmptyLine
            },
            Skipped {
                file: format!("{directory}/agent-z.jsonl"),
                line: 3,
                reason: SkipReason::InvalidJson
            },
        ]
    );
}

#[test]
fn unexpected_transcript_shapes_do_not_panic_or_fabricate_metadata() {
    let root = tempfile::tempdir().unwrap();
    let path = ".claude/projects/p/main.jsonl";
    write(
        root.path(),
        path,
        concat!(
            "null\n[]\n{\"type\":5}\n{}\n",
            "{\"type\":\"user\",\"cwd\":[],\"gitBranch\":false,\"timestamp\":17}\n",
            "{\"type\":\"assistant\",\"timestamp\":\"invalid\"}\n",
            "{\"type\":\"user\",\"cwd\":\"/pending\"}",
        ),
    );
    let actual = scan(root.path()).unwrap();
    let transcript = &actual.transcripts[0];
    assert_eq!(transcript.cwd, None);
    assert_eq!(transcript.branch, None);
    assert_eq!(transcript.started_at, None);
    assert_eq!(transcript.last_activity, None);
    assert_eq!(
        actual.skipped,
        (1..=4)
            .map(|line| Skipped {
                file: path.into(),
                line,
                reason: SkipReason::UnknownType,
            })
            .collect::<Vec<_>>()
    );
}

#[test]
fn malformed_session_records_are_omitted_without_stopping_discovery() {
    let root = tempfile::tempdir().unwrap();
    let valid = json!({"pid": 7, "procStart": "start", "sessionId": "session",
        "cwd": "/repo", "status": "future-status", "startedAt": 0,
        "updatedAt": 2, "statusUpdatedAt": 1});
    write(root.path(), ".claude/sessions/invalid.json", "{bad");
    write(root.path(), ".claude/sessions/missing.json", "{}");
    write(root.path(), ".claude/sessions/null.json", "null");
    for (index, (field, value)) in [
        ("pid", json!(-1)),
        ("pid", json!(4294967296_u64)),
        ("procStart", json!(false)),
        ("sessionId", json!([])),
        ("cwd", Value::Null),
        ("status", json!(3)),
        ("statusUpdatedAt", json!("bad")),
        ("startedAt", json!(i64::MAX)),
        ("updatedAt", json!(1.5)),
    ]
    .into_iter()
    .enumerate()
    {
        let mut invalid = valid.clone();
        invalid[field] = value;
        write(
            root.path(),
            &format!(".claude/sessions/invalid-{index}.json"),
            &invalid.to_string(),
        );
    }
    write(
        root.path(),
        ".claude/sessions/valid.json",
        &valid.to_string(),
    );
    let actual = scan(root.path()).unwrap();
    assert!(actual.transcripts.is_empty());
    assert!(actual.skipped.is_empty());
    assert_eq!(
        actual.bad_records,
        [
            ".claude/sessions/invalid-0.json",
            ".claude/sessions/invalid-1.json",
            ".claude/sessions/invalid-2.json",
            ".claude/sessions/invalid-3.json",
            ".claude/sessions/invalid-4.json",
            ".claude/sessions/invalid-5.json",
            ".claude/sessions/invalid-6.json",
            ".claude/sessions/invalid-7.json",
            ".claude/sessions/invalid-8.json",
            ".claude/sessions/invalid.json",
            ".claude/sessions/missing.json",
            ".claude/sessions/null.json",
        ]
    );
    assert_eq!(actual.records.len(), 1);
    let record = &actual.records[0];
    assert_eq!(record.record, ".claude/sessions/valid.json");
    assert_eq!(record.pid, 7);
    assert_eq!(record.session_id, "session");
    assert_eq!(record.status, "future-status");
    assert_eq!(record.started_at.to_string(), "1970-01-01T00:00:00.000Z");
    assert_eq!(
        record.status_updated_at.to_string(),
        "1970-01-01T00:00:00.001Z"
    );
    assert_eq!(record.updated_at.to_string(), "1970-01-01T00:00:00.002Z");
}

#[test]
fn unreadable_session_record_is_counted() {
    let root = tempfile::tempdir().unwrap();
    // Reading a directory as JSON fails even when tests run with elevated privileges.
    fs::create_dir_all(root.path().join(".claude/sessions/unreadable.json")).unwrap();
    let actual = scan(root.path()).unwrap();
    assert_eq!(actual.bad_records, [".claude/sessions/unreadable.json"]);
    assert!(actual.records.is_empty());
    assert!(actual.skipped.is_empty());
}
