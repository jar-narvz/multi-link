use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use multi_link_claude::SessionRecord;
use multi_link_codex::{Rollout, Turn};
use multi_link_core::{
    FakeProcessProbe, FixtureProcessProbe, Liveness, Options, ProcessProbe, RepoMode,
    sessions_report,
    state::{claude_state, codex_state},
};
use multi_link_model::{SessionsReport, State, StateEvidence, Timestamp};
use serde_json::{Value, json};

fn stamp(text: &str) -> Timestamp {
    Timestamp::parse(text).unwrap()
}
fn now() -> Timestamp {
    stamp("2026-10-29T23:15:00Z")
}
fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}
fn expected() -> Value {
    serde_json::from_slice(&fs::read(fixtures().join("expected/sessions.json")).unwrap()).unwrap()
}
fn options(since: Option<Duration>) -> Options<'static> {
    Options {
        now: now(),
        since,
        repo_mode: RepoMode::Fixture,
        probe: &FixtureProcessProbe,
    }
}
fn record(status: &str) -> SessionRecord {
    SessionRecord {
        pid: 42,
        proc_start: "start".into(),
        session_id: "merged".into(),
        cwd: "/record-cwd".into(),
        status: status.into(),
        status_updated_at: stamp("2026-10-29T23:10:00Z"),
        started_at: stamp("2026-10-29T20:00:00Z"),
        updated_at: now(),
        record: ".claude/sessions/42.json".into(),
    }
}

#[test]
fn fixture_report_equals_every_expected_field() {
    let output = sessions_report(&fixtures().join("sessions"), &options(None)).unwrap();
    assert_eq!(serde_json::to_value(&output.report).unwrap(), expected());
    assert!(output.bad_records.is_empty());
    assert!(output.incomplete.is_empty());
    assert!(output.unreadable.is_empty());
}

#[test]
fn since_two_hours_filters_only_transcript_history() {
    let output = sessions_report(
        &fixtures().join("sessions"),
        &options(Some(Duration::from_secs(7200))),
    )
    .unwrap();
    let mut expected: SessionsReport = serde_json::from_value(expected()).unwrap();
    let cutoff = stamp("2026-10-29T21:15:00Z");
    assert!(
        expected
            .sessions
            .iter()
            .any(|session| session.record.is_none() && session.last_activity < cutoff)
    );
    assert!(
        expected
            .sessions
            .iter()
            .any(|session| session.transcript.is_none() && session.last_activity < cutoff)
    );
    expected
        .sessions
        .retain(|session| session.record.is_some() || session.last_activity >= cutoff);
    assert_eq!(output.report, expected);
}

#[test]
fn claude_freshness_status_and_probe_precedence() {
    let mut record = record("busy");
    for liveness in [Liveness::Alive, Liveness::Unavailable] {
        assert_eq!(
            claude_state(now(), Some(&record), liveness),
            (State::Busy, StateEvidence::SessionRecord)
        );
        assert_eq!(
            claude_state(stamp("2026-10-29T23:15:01Z"), Some(&record), liveness),
            (State::Stale, StateEvidence::SessionRecord)
        );
    }
    record.status = "idle".into();
    record.status_updated_at = stamp("2026-10-29T20:15:00Z");
    assert_eq!(
        claude_state(now(), Some(&record), Liveness::Unavailable),
        (State::Idle, StateEvidence::SessionRecord)
    );
    record.status = "future-status".into();
    assert_eq!(
        claude_state(now(), Some(&record), Liveness::Alive),
        (State::Unknown, StateEvidence::SessionRecord)
    );
    assert_eq!(
        claude_state(now(), None, Liveness::Gone),
        (State::Unknown, StateEvidence::NoSignal)
    );
    let mut probe = FakeProcessProbe::default();
    for start in ["gone", "different start"] {
        probe.results.insert((42, start.into()), Liveness::Gone);
        record.proc_start = start.into();
        assert_eq!(
            claude_state(
                now(),
                Some(&record),
                probe.probe(record.pid, &record.proc_start)
            ),
            (State::Finished, StateEvidence::ProcessExit)
        );
    }
}

#[test]
fn latest_codex_turn_governs_and_open_turn_uses_last_activity() {
    let mut rollout = Rollout {
        id: "codex".into(),
        transcript: "rollout.jsonl".into(),
        cwd: "/repo".into(),
        branch: None,
        started_at: stamp("2026-10-29T20:00:00Z"),
        last_activity: stamp("2026-10-29T23:10:00Z"),
        turns: vec![],
    };
    assert_eq!(
        codex_state(now(), &rollout),
        (State::Unknown, StateEvidence::NoSignal)
    );
    rollout.turns.push(Turn {
        turn_id: "interrupted".into(),
        started_at: rollout.started_at,
        completed_at: None,
    });
    for clock in [stamp("2026-10-29T23:14:59Z"), now()] {
        assert_eq!(
            codex_state(clock, &rollout),
            (State::Busy, StateEvidence::OpenTurn)
        );
    }
    assert_eq!(
        codex_state(stamp("2026-10-29T23:15:01Z"), &rollout),
        (State::Stale, StateEvidence::OpenTurn)
    );
    rollout.turns.push(Turn {
        turn_id: "last".into(),
        started_at: rollout.last_activity,
        completed_at: Some(now()),
    });
    assert_eq!(
        codex_state(now(), &rollout),
        (State::Unknown, StateEvidence::TurnsComplete)
    );
}

fn write(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}
fn write_record(root: &Path, pid: u32, updated_at: i64, status: &str) {
    write(
        root,
        &format!(".claude/sessions/{pid}.json"),
        &json!({
            "pid": pid, "procStart": "start", "sessionId": "merged", "cwd": "/record-cwd",
            "status": status, "statusUpdatedAt": 1793315700000_i64,
            "startedAt": 1793304000000_i64, "updatedAt": updated_at,
        })
        .to_string(),
    );
}
fn write_transcript(root: &Path) {
    write(
        root,
        ".claude/projects/project/merged.jsonl",
        &format!(
            "{}\n{}\n",
            json!({"type":"user", "cwd":"/transcript-cwd", "gitBranch":"main", "timestamp":"2026-10-29T19:00:00Z"}),
            json!({"type":"assistant", "timestamp":"2026-10-29T20:00:00Z"})
        ),
    );
}

#[test]
fn merge_uses_transcript_fields_latest_record_and_bypasses_since() {
    let temp = tempfile::tempdir().unwrap();
    write_transcript(temp.path());
    write_record(temp.path(), 42, 1793315700000, "busy");
    write_record(temp.path(), 43, 1793315600000, "idle");
    let output = sessions_report(temp.path(), &options(Some(Duration::from_secs(7200)))).unwrap();
    assert_eq!(
        serde_json::to_value(output.report).unwrap(),
        json!({
            "now":"2026-10-29T23:15:00.000Z", "skipped":[], "sessions":[{
                "id":"merged", "harness":"claude", "transcript":".claude/projects/project/merged.jsonl",
                "record":".claude/sessions/42.json", "cwd":"/transcript-cwd", "repo":"/transcript-cwd", "branch":"main",
                "started_at":"2026-10-29T19:00:00.000Z", "last_activity":"2026-10-29T20:00:00.000Z",
                "state":"busy", "state_evidence":"session_record", "subagents":[],
                "segments":[{"from":"2026-10-29T19:00:00.000Z", "to":"2026-10-29T20:00:00.000Z", "step":null, "step_source":"unknown"}]
            }]
        })
    );
}

#[test]
fn report_uses_gone_probe_for_merged_and_record_only_sessions() {
    for merged in [false, true] {
        for start in ["start", "different start"] {
            let temp = tempfile::tempdir().unwrap();
            if merged {
                write_transcript(temp.path());
            }
            write_record(temp.path(), 42, 1793315700000, "busy");
            let path = temp.path().join(".claude/sessions/42.json");
            let mut record: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            record["procStart"] = json!(start);
            fs::write(path, record.to_string()).unwrap();
            let mut probe = FakeProcessProbe::default();
            probe.results.insert((42, start.into()), Liveness::Gone);
            let mut options = options(None);
            options.probe = &probe;
            let output = sessions_report(temp.path(), &options).unwrap();
            assert_eq!(output.report.sessions.len(), 1);
            let session = &output.report.sessions[0];
            assert_eq!(session.transcript.is_some(), merged);
            assert_eq!(session.record.as_deref(), Some(".claude/sessions/42.json"));
            assert_eq!(
                (session.state, session.state_evidence),
                (State::Finished, StateEvidence::ProcessExit)
            );
        }
    }
}

#[test]
fn report_trusts_stale_busy_record_when_probe_is_unavailable() {
    let temp = tempfile::tempdir().unwrap();
    write_record(temp.path(), 42, 1793315700000, "busy");
    let mut probe = FakeProcessProbe::default();
    probe
        .results
        .insert((42, "start".into()), Liveness::Unavailable);
    let mut options = options(None);
    options.now = stamp("2026-10-29T23:20:01Z");
    options.probe = &probe;
    let output = sessions_report(temp.path(), &options).unwrap();
    assert_eq!(output.report.sessions.len(), 1);
    let session = &output.report.sessions[0];
    assert_eq!(
        (session.state, session.state_evidence),
        (State::Stale, StateEvidence::SessionRecord)
    );
}

#[test]
fn transcript_only_and_incomplete_transcripts_have_no_invented_fields() {
    let temp = tempfile::tempdir().unwrap();
    write_transcript(temp.path());
    write(temp.path(), ".claude/projects/project/empty.jsonl", "");
    write(
        temp.path(),
        ".claude/projects/project/no-cwd.jsonl",
        &format!(
            "{}\n",
            json!({"type":"user", "timestamp":"2026-10-29T19:00:00Z"})
        ),
    );
    write(
        temp.path(),
        ".claude/projects/project/no-time.jsonl",
        &format!("{}\n", json!({"type":"user", "cwd":"/repo"})),
    );
    let output = sessions_report(temp.path(), &options(None)).unwrap();
    assert_eq!(
        output.incomplete,
        vec![
            ".claude/projects/project/empty.jsonl",
            ".claude/projects/project/no-cwd.jsonl",
            ".claude/projects/project/no-time.jsonl",
        ]
    );
    assert_eq!(output.report.sessions.len(), 1);
    assert_eq!(
        (
            output.report.sessions[0].state,
            output.report.sessions[0].state_evidence
        ),
        (State::Unknown, StateEvidence::NoSignal)
    );
    write(temp.path(), ".claude/projects/project/merged.jsonl", "");
    write_record(temp.path(), 42, 1793315700000, "idle");
    let output = sessions_report(temp.path(), &options(None)).unwrap();
    assert_eq!(output.report.sessions.len(), 1);
    assert_eq!(
        output.incomplete,
        vec![
            ".claude/projects/project/empty.jsonl",
            ".claude/projects/project/merged.jsonl",
            ".claude/projects/project/no-cwd.jsonl",
            ".claude/projects/project/no-time.jsonl",
        ]
    );
    assert_eq!(output.report.sessions[0].transcript, None);
    assert_eq!(output.report.sessions[0].segments, vec![]);
}

#[test]
fn file_diagnostics_are_separate_from_schema_skips() {
    let temp = tempfile::tempdir().unwrap();
    write(temp.path(), ".claude/sessions/bad.json", "bad json");
    write(
        temp.path(),
        ".codex/sessions/rollout-invalid.jsonl",
        "\u{fffd}",
    );
    fs::write(
        temp.path().join(".codex/sessions/rollout-invalid.jsonl"),
        [0xff, b'\n'],
    )
    .unwrap();
    let output = sessions_report(temp.path(), &options(None)).unwrap();
    assert_eq!(output.bad_records, vec![".claude/sessions/bad.json"]);
    assert!(output.unreadable.is_empty());
    assert_eq!(
        serde_json::to_value(output.report).unwrap(),
        json!({"now":"2026-10-29T23:15:00.000Z", "sessions":[], "skipped":[{
            "file":".codex/sessions/rollout-invalid.jsonl", "line":1, "reason":"invalid_json"
        }]})
    );
}

#[test]
fn since_boundary_is_inclusive_and_future_activity_is_retained() {
    let temp = tempfile::tempdir().unwrap();
    write_transcript(temp.path());
    let mut options = options(Some(Duration::from_secs(7200)));
    options.now = stamp("2026-10-29T22:00:00Z");
    assert_eq!(
        sessions_report(temp.path(), &options)
            .unwrap()
            .report
            .sessions
            .len(),
        1
    );
    options.now = stamp("2026-10-29T22:00:00.001Z");
    assert!(
        sessions_report(temp.path(), &options)
            .unwrap()
            .report
            .sessions
            .is_empty()
    );
    options.now = stamp("2026-10-29T19:00:00Z");
    assert_eq!(
        sessions_report(temp.path(), &options)
            .unwrap()
            .report
            .sessions
            .len(),
        1
    );
}
