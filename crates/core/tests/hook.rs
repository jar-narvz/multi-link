use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use multi_link_core::{FixtureProcessProbe, Options, RepoMode, links_report, sessions_report};
use multi_link_model::{State, StateEvidence, Timestamp};
use serde_json::{Value, json};

const ID: &str = "00000000-0000-4000-8000-000000000001";
const CODEX: &str = "00000000-0000-4000-8000-000000000002";
const EARLY: &str = "2026-10-29T19:00:00.000Z";
const WAIT: &str = "2026-10-29T20:00:00.000Z";
const LATE: &str = "2026-10-29T21:00:00.000Z";

fn options() -> Options<'static> {
    Options {
        state_dir: PathBuf::from("ignored-in-fixture-mode"),
        now: Timestamp::parse("2026-10-29T23:15:00Z").unwrap(),
        since: None,
        repo_mode: RepoMode::Fixture,
        probe: &FixtureProcessProbe,
    }
}
fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
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
fn write(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}
fn hook(id: &str, at: &str) -> Value {
    json!({"v":1,"kind":"waiting","at":at,"session_id":id,"agent_id":null,
        "event":"PermissionRequest","tool_use_id":null,"cwd":"/repo",
        "child":null,"child_evidence":null,"notification_type":null})
}
fn write_hooks(root: &Path, id: &str, records: &[Value]) {
    write(
        root,
        &format!(".multi-link/hook/{id}.jsonl"),
        &records.iter().map(|v| format!("{v}\n")).collect::<String>(),
    );
}
fn transcript(root: &Path, at: &str) {
    write(
        root,
        &format!(".claude/projects/project/{ID}.jsonl"),
        &format!(
            "{}\n",
            json!({"type":"user", "cwd":"/repo", "timestamp":at})
        ),
    );
}
fn state(root: &Path) -> (State, StateEvidence) {
    let output = sessions_report(root, &options()).unwrap();
    let session = output.report.sessions.iter().find(|s| s.id == ID).unwrap();
    (session.state, session.state_evidence)
}

#[test]
fn hook_variant_equals_all_three_answer_keys() {
    let temp = tempfile::tempdir().unwrap();
    copy_tree(&fixtures().join("sessions"), temp.path());
    // Copy hidden overlay directories explicitly; globbing hook/* misses both.
    for hidden in [".multi-link", ".claude"] {
        copy_tree(
            &fixtures().join("hook").join(hidden),
            &temp.path().join(hidden),
        );
    }
    fs::remove_file(temp.path().join("task-output/be01y5dv6.output")).unwrap();
    let expected = |name: &str| -> Value {
        serde_json::from_slice(&fs::read(fixtures().join("expected/hook").join(name)).unwrap())
            .unwrap()
    };
    let links = links_report(temp.path(), &options()).unwrap();
    assert_eq!(
        serde_json::to_value(links.report).unwrap(),
        expected("links.json")
    );
    assert_eq!(links.hook_conflicts, 1);
    for (since, name) in [
        (None, "sessions.json"),
        (Some(Duration::from_secs(7200)), "sessions-2h.json"),
    ] {
        let mut opts = options();
        opts.since = since;
        let output = sessions_report(temp.path(), &opts).unwrap();
        assert_eq!(serde_json::to_value(output.report).unwrap(), expected(name));
    }
}

#[test]
fn waiting_cleared_by_main_or_subagent_top_level_timestamp_including_unknown_types() {
    for subagent in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        transcript(temp.path(), EARLY);
        write_hooks(temp.path(), ID, &[hook(ID, WAIT)]);
        assert_eq!(state(temp.path()), (State::Waiting, StateEvidence::Hook));
        let path = if subagent {
            format!(".claude/projects/project/{ID}/subagents/agent-test.jsonl")
        } else {
            format!(".claude/projects/project/{ID}.jsonl")
        };
        let text = if subagent {
            String::new()
        } else {
            fs::read_to_string(temp.path().join(&path)).unwrap()
        };
        for timestamp in [WAIT, LATE] {
            write(
                temp.path(),
                &path,
                &format!(
                    "{text}{}\n",
                    json!({"type":"future-type", "timestamp":timestamp})
                ),
            );
            assert_eq!(
                state(temp.path()),
                (State::Unknown, StateEvidence::NoSignal)
            );
        }
    }
}

#[test]
fn latest_valid_hook_uses_time_then_line_and_handoff_clears_waiting() {
    let temp = tempfile::tempdir().unwrap();
    transcript(temp.path(), EARLY);
    let waiting = hook(ID, WAIT);
    let mut handoff = waiting.clone();
    handoff["kind"] = json!("handoff");
    handoff["event"] = json!("PostToolUse");
    handoff["tool_use_id"] = json!("tool");
    for (records, expected) in [
        (vec![handoff.clone(), waiting.clone()], State::Waiting),
        (vec![waiting.clone(), handoff.clone()], State::Unknown),
    ] {
        write_hooks(temp.path(), ID, &records);
        assert_eq!(state(temp.path()).0, expected);
    }
    handoff["at"] = json!(LATE);
    write_hooks(temp.path(), ID, &[handoff.clone(), waiting.clone()]);
    assert_eq!(state(temp.path()).0, State::Unknown);
    handoff["v"] = json!(2);
    write_hooks(temp.path(), ID, &[waiting, handoff]);
    assert_eq!(state(temp.path()).0, State::Waiting);
}

#[test]
fn reader_rejects_invalid_records_and_buffers_final_line() {
    let temp = tempfile::tempdir().unwrap();
    transcript(temp.path(), EARLY);
    let valid = hook(ID, WAIT);
    let mut invalid = Vec::new();
    for field in valid.as_object().unwrap().keys() {
        let mut missing = valid.clone();
        missing.as_object_mut().unwrap().remove(field);
        invalid.push(missing);
        let mut wrong_type = valid.clone();
        wrong_type[field] = json!([]);
        invalid.push(wrong_type);
    }
    for (field, value) in [
        ("v", json!(2)),
        ("kind", json!("future")),
        ("event", json!("PostToolUse")),
        ("notification_type", json!("idle_prompt")),
        ("child", json!(CODEX)),
        ("child_evidence", json!("stdout")),
        ("session_id", json!(CODEX)),
        ("tool_use_id", json!("tool")),
        ("at", json!("bad")),
    ] {
        let mut record = valid.clone();
        record[field] = value;
        invalid.push(record);
    }
    let text = format!(
        "\nnot json\n[]\n{}{}\n{}",
        invalid.iter().map(|v| format!("{v}\n")).collect::<String>(),
        valid,
        valid
    );
    let file = format!(".multi-link/hook/{ID}.jsonl");
    write(temp.path(), &file, &text);
    // These names must never be read, even when they contain malformed lines.
    for name in [
        "README.jsonl",
        "00000000-0000-4000-8000-00000000000A.jsonl",
        "extra.json",
        "00000000-0000-4000-8000-000000000003.jsonl.bak",
    ] {
        write(temp.path(), &format!(".multi-link/hook/{name}"), "bad\n");
    }
    let output = sessions_report(temp.path(), &options()).unwrap();
    let expected: Vec<_> = (1..=invalid.len() + 3).map(|line| json!({
        "file":file, "line":line, "reason": match line { 1 => "empty_line", 2 | 3 => "invalid_json", _ => "unknown_type" }
    })).collect();
    assert_eq!(
        serde_json::to_value(output.report.skipped).unwrap(),
        json!(expected)
    );
    assert_eq!(state(temp.path()), (State::Waiting, StateEvidence::Hook));
}

#[test]
fn codex_and_unknown_hook_ids_never_create_or_change_sessions() {
    let temp = tempfile::tempdir().unwrap();
    write_hooks(temp.path(), ID, &[hook(ID, WAIT)]);
    assert!(
        sessions_report(temp.path(), &options())
            .unwrap()
            .report
            .sessions
            .is_empty()
    );
    write(
        temp.path(),
        &format!(".codex/sessions/rollout-{CODEX}.jsonl"),
        &format!(
            "{}\n",
            json!({"type":"session_meta", "timestamp":EARLY,
            "payload":{"id":CODEX, "cwd":"/repo", "timestamp":EARLY}})
        ),
    );
    write_hooks(temp.path(), CODEX, &[hook(CODEX, WAIT)]);
    let output = sessions_report(temp.path(), &options()).unwrap();
    assert_eq!(output.report.sessions.len(), 1);
    assert_eq!(output.report.sessions[0].id, CODEX);
    assert_eq!(output.report.sessions[0].state, State::Unknown);
    let mut opts = options();
    opts.since = Some(Duration::from_secs(7200));
    assert!(
        sessions_report(temp.path(), &opts)
            .unwrap()
            .report
            .sessions
            .is_empty()
    );
}

#[test]
fn real_mode_uses_caller_state_directory_and_absolute_skipped_path() {
    let root = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    transcript(root.path(), EARLY);
    write(
        state_dir.path(),
        &format!("hook/{ID}.jsonl"),
        &format!("\n{}\n", hook(ID, WAIT)),
    );
    let mut opts = options();
    opts.repo_mode = RepoMode::Real;
    opts.state_dir = state_dir.path().to_owned();
    let output = sessions_report(root.path(), &opts).unwrap();
    assert_eq!(output.report.sessions[0].state, State::Waiting);
    assert_eq!(
        output.report.skipped[0].file,
        state_dir
            .path()
            .join(format!("hook/{ID}.jsonl"))
            .to_string_lossy()
    );
}
