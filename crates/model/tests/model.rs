use std::{fs, io::Cursor, path::PathBuf};

use multi_link_model::{jsonl::JsonlReader, *};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

#[test]
fn session_preserves_all_thirteen_keys_and_nulls() {
    let instant = Timestamp::from_epoch_ms(0).unwrap();
    let session = Session {
        id: "session".into(),
        harness: Harness::Claude,
        transcript: None,
        record: None,
        cwd: "/repo".into(),
        repo: "/repo".into(),
        branch: None,
        started_at: instant,
        last_activity: instant,
        state: State::Unknown,
        state_evidence: StateEvidence::NoSignal,
        segments: vec![],
        subagents: vec![],
    };
    let value = serde_json::to_value(session).unwrap();
    assert_eq!(value.as_object().unwrap().len(), 13);
    assert_eq!(
        value,
        json!({
            "id": "session", "harness": "claude", "transcript": null,
            "record": null, "cwd": "/repo", "repo": "/repo", "branch": null,
            "started_at": "1970-01-01T00:00:00.000Z",
            "last_activity": "1970-01-01T00:00:00.000Z",
            "state": "unknown", "state_evidence": "no_signal",
            "segments": [], "subagents": []
        })
    );
    assert_eq!(
        serde_json::to_value(Segment {
            from: instant,
            to: instant,
            step: None,
            step_source: StepSource::Unknown,
        })
        .unwrap(),
        json!({"from": "1970-01-01T00:00:00.000Z",
        "to": "1970-01-01T00:00:00.000Z", "step": null, "step_source": "unknown"})
    );
}

#[test]
fn epoch_milliseconds_and_utc_format() {
    let instant = Timestamp::from_epoch_ms(1793315559714).unwrap();
    assert_eq!(instant.to_string(), "2026-10-29T23:12:39.714Z");
    assert_eq!(
        serde_json::to_value(instant).unwrap(),
        json!("2026-10-29T23:12:39.714Z")
    );
    assert_eq!(
        Timestamp::from_epoch_ms(-1).unwrap().to_string(),
        "1969-12-31T23:59:59.999Z"
    );
    assert!(Timestamp::from_epoch_ms(i64::MAX).is_err());
}

#[test]
fn timestamps_normalize_offsets_and_truncate_fractional_seconds() {
    for (input, expected) in [
        (
            "2026-10-30T01:12:39.714999999+02:00",
            "2026-10-29T23:12:39.714Z",
        ),
        ("2026-10-29T18:12:39-05:00", "2026-10-29T23:12:39.000Z"),
        ("2026-10-29T23:12:39.1Z", "2026-10-29T23:12:39.100Z"),
        (
            "2026-10-29T23:12:39.123456789123Z",
            "2026-10-29T23:12:39.123Z",
        ),
        ("1969-12-31T23:59:59.999999Z", "1969-12-31T23:59:59.999Z"),
    ] {
        assert_eq!(Timestamp::parse(input).unwrap().to_string(), expected);
        assert_eq!(
            Timestamp::parse(input).unwrap(),
            Timestamp::parse(expected).unwrap()
        );
    }
    let early: Timestamp = "2026-10-29T23:12:39.714001Z".parse().unwrap();
    let late: Timestamp = "2026-10-29T23:12:39.714002Z".parse().unwrap();
    assert_eq!(early, late);
    assert_eq!(early, Timestamp::from_epoch_ms(1793315559714).unwrap());
    assert_eq!(early.to_string(), late.to_string());
    assert_eq!(
        Timestamp::parse("2026-10-30T01:12:39+02:00").unwrap(),
        Timestamp::parse("2026-10-29T23:12:39Z").unwrap()
    );
    assert!(Timestamp::parse("not a timestamp").is_err());
}

fn round_trip<T: DeserializeOwned + Serialize>(path: &std::path::Path) {
    let expected: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let report: T = serde_json::from_value(expected.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(report).unwrap(),
        expected,
        "{}",
        path.display()
    );
}

#[test]
fn every_expected_report_round_trips() {
    let mut names = Vec::new();
    for entry in fs::read_dir(fixtures().join("expected")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }
        let name = path.file_name().unwrap().to_str().unwrap();
        match name {
            "sessions.json" => round_trip::<SessionsReport>(&path),
            "links.json" | "links-amb-hidden.json" => round_trip::<LinksReport>(&path),
            "steps.json" => round_trip::<StepsReport>(&path),
            _ => panic!("new expected report needs a type: {name}"),
        }
        names.push(name.to_owned());
    }
    names.sort();
    assert_eq!(
        names,
        [
            "links-amb-hidden.json",
            "links.json",
            "sessions.json",
            "steps.json"
        ]
    );
}

#[test]
fn reserved_enum_values_serialize_as_schema_strings() {
    macro_rules! check {
        ($($value:expr => $text:literal),+ $(,)?) => {
            $(assert_eq!(serde_json::to_value($value).unwrap(), json!($text));)+
        };
    }
    check!(State::Busy => "busy", State::Finished => "finished", State::Stale => "stale",
        State::Waiting => "waiting", StateEvidence::ProcessExit => "process_exit",
        StateEvidence::OpenTurn => "open_turn", StateEvidence::TurnsComplete => "turns_complete",
        StepSource::PromptTag => "prompt_tag", StepSource::Hook => "hook",
        SkipReason::UnknownType => "unknown_type", Convention::Missing => "missing",
        Convention::Unsupported => "unsupported", VerifyResult::Pass => "pass",
        VerifyResult::Fail => "fail", VerifySource::Observed => "observed");
}

#[test]
fn malformed_fixture_skips_bad_lines_and_continues() {
    let directory = fixtures().join("sessions/.claude/projects/-home-dev-repo-e");
    let path = fs::read_dir(directory)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert!(
        path.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .ends_with("0e.jsonl")
    );
    let display_path = path
        .strip_prefix(fixtures().join("sessions"))
        .unwrap()
        .to_str()
        .unwrap();
    let records = JsonlReader::from_path(&path, display_path)
        .unwrap()
        .collect::<std::io::Result<Vec<_>>>()
        .unwrap();
    let skipped: Vec<_> = records
        .iter()
        .filter_map(|record| record.as_ref().err())
        .cloned()
        .collect();
    assert_eq!(
        skipped,
        vec![
            Skipped {
                file: display_path.into(),
                line: 3,
                reason: SkipReason::InvalidJson
            },
            Skipped {
                file: display_path.into(),
                line: 4,
                reason: SkipReason::EmptyLine
            },
        ]
    );
    let yielded: Vec<_> = records
        .iter()
        .filter_map(|record| record.as_ref().ok().map(|(line, _)| *line))
        .collect();
    assert_eq!(yielded, vec![1, 2, 5, 6]);
}

#[test]
fn unterminated_last_line_is_pending_even_when_valid_json() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tail.jsonl");
    for tail in ["{\"pending\":", "{\"complete\":true}"] {
        fs::write(&path, format!("{{\"first\":1}}\n{tail}")).unwrap();
        let mut reader = JsonlReader::from_path(&path, "relative/tail.jsonl").unwrap();
        let records = reader
            .by_ref()
            .collect::<std::io::Result<Vec<_>>>()
            .unwrap();
        assert_eq!(records, vec![Ok((1, json!({"first": 1})))]);
        assert_eq!(reader.pending(), tail.as_bytes());
        assert_eq!(reader.pending_line(), 2);
        assert!(reader.next().is_none());
    }
}

#[test]
fn crlf_whitespace_and_invalid_utf8_keep_line_numbers_and_display_path() {
    let input = b"{\"ok\":1}\r\n \t\r\n\r\nnope\r\n\xff\nnull\n\xe2\x82";
    let mut reader = JsonlReader::new(Cursor::new(input), "caller/path.jsonl");
    let records = reader
        .by_ref()
        .collect::<std::io::Result<Vec<_>>>()
        .unwrap();
    assert_eq!(
        records,
        vec![
            Ok((1, json!({"ok": 1}))),
            Err(Skipped {
                file: "caller/path.jsonl".into(),
                line: 2,
                reason: SkipReason::EmptyLine
            }),
            Err(Skipped {
                file: "caller/path.jsonl".into(),
                line: 3,
                reason: SkipReason::EmptyLine
            }),
            Err(Skipped {
                file: "caller/path.jsonl".into(),
                line: 4,
                reason: SkipReason::InvalidJson
            }),
            Err(Skipped {
                file: "caller/path.jsonl".into(),
                line: 5,
                reason: SkipReason::InvalidJson
            }),
            Ok((6, Value::Null)),
        ]
    );
    assert_eq!(reader.pending(), b"\xe2\x82");
    assert_eq!(reader.pending_line(), 7);
}

#[test]
fn empty_reader_and_io_errors_are_distinct_from_skips() {
    let mut empty = JsonlReader::new(Cursor::new(b""), "empty");
    assert!(empty.next().is_none());
    assert_eq!(empty.pending(), b"");
    assert_eq!(empty.pending_line(), 1);
    let mut failing = JsonlReader::new(std::io::BufReader::new(FailingRead), "unreadable");
    assert_eq!(
        failing.next().unwrap().unwrap_err().kind(),
        std::io::ErrorKind::PermissionDenied
    );
    assert!(failing.next().is_none());
}

struct FailingRead;
impl std::io::Read for FailingRead {
    fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "test read failure",
        ))
    }
}
