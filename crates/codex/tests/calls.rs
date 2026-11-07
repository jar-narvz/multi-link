use multi_link_codex::{calls, codex_turns, read_details};
use multi_link_model::{CallKind, Timestamp, Turn};
use serde_json::{Value, json};
use std::{fs, path::Path};

fn t(second: u8) -> Timestamp {
    Timestamp::parse(&format!("2026-11-06T10:00:{second:02}Z")).unwrap()
}
fn record(kind: &str, payload: Value, second: u8) -> Value {
    json!({"type":kind,"timestamp":t(second).to_string(),"payload":payload})
}
fn event(kind: &str, id: &str, second: u8) -> Value {
    record("event_msg", json!({"type":kind,"turn_id":id}), second)
}
fn call(id: &str, input: Value, second: u8) -> Value {
    record(
        "response_item",
        json!({"type":"custom_tool_call","call_id":id,"name":"exec","input":input}),
        second,
    )
}
fn parse(records: Vec<Value>) -> (multi_link_model::Calls, Vec<Turn>) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic.jsonl");
    fs::write(
        &path,
        records.iter().map(|v| format!("{v}\n")).collect::<String>(),
    )
    .unwrap();
    let details = read_details(&path).unwrap();
    assert_eq!(calls(&path).unwrap().calls, details.0.calls);
    assert_eq!(codex_turns(&path).unwrap(), details.1);
    details
}

#[test]
fn nested_tools_one_call_per_record_and_results_by_id() {
    let (report, turns) = parse(vec![
        event("task_started", "real", 0),
        call(
            "read",
            json!(
                r#"text(await tools.exec_command({cmd:"git diff -- secret"})); text(await tools.apply_patch("ignored"));"#
            ),
            1,
        ),
        call(
            "edit",
            json!(
                r#"text(await tools.apply_patch("*** Begin Patch\n*** Update File: /synthetic/a.rs\n*** Add File: b.rs\n"));"#
            ),
            2,
        ),
        call("stdin", json!("await tools.write_stdin({session_id:1})"), 3),
        record(
            "response_item",
            json!({"type":"function_call","call_id":"js","name":"js","arguments":"{\"code\":\"script\"}"}),
            4,
        ),
        call("bad", json!({"unexpected":true}), 5),
        record(
            "response_item",
            json!({"type":"custom_tool_call_output","call_id":"read","output":[{"text":"{\"exit_code\":0}"},{"text":"{\"exit_code\":1}"}]}),
            6,
        ),
        record(
            "response_item",
            json!({"type":"function_call_output","call_id":"js","output":"{\"exit_code\":2}"}),
            7,
        ),
        record(
            "response_item",
            json!({"type":"custom_tool_call_output","call_id":"edit","output":"Success"}),
            8,
        ),
        call("read", json!("await tools.write_stdin({})"), 9),
        record("future_record", json!({}), 10),
    ]);
    let values: Vec<_> = report
        .calls
        .iter()
        .map(|c| (c.id.as_str(), c.kind, c.label.as_deref(), c.ok))
        .collect();
    assert_eq!(
        values,
        vec![
            ("read", CallKind::Read, Some("$ git diff"), Some(true)),
            ("edit", CallKind::Edit, Some("edit a.rs"), None),
            ("stdin", CallKind::Run, None, None),
            ("js", CallKind::Run, None, Some(false)),
            ("bad", CallKind::Run, None, None)
        ]
    );
    assert_eq!(report.skipped.len(), 1);
    assert_eq!(report.skipped[0].line, 11);
    assert_eq!(
        turns,
        vec![Turn {
            start: t(0),
            completed_at: None,
            last_record_at: t(10),
            prompt: None,
            last_agent_message: None
        }]
    );
}

#[test]
fn imported_turns_are_excluded_and_resumes_stay_separate() {
    let (report, turns) = parse(vec![
        event("task_started", "external-import-1", 0),
        call("import", json!("tools.exec_command({cmd:\"git diff\"})"), 1),
        event("task_complete", "wrong-id", 2),
        event("task_complete", "external-import-1", 3),
        event("task_started", "a", 4),
        record(
            "event_msg",
            json!({"type":"user_message","message":"  Synthetic   prompt "}),
            5,
        ),
        record(
            "event_msg",
            json!({"type":"task_complete","turn_id":"a","last_agent_message":" Synthetic done "}),
            6,
        ),
        event("task_started", "b", 7),
        call("real", json!("tools.exec_command({cmd:'cargo test'})"), 8),
    ]);
    assert_eq!(report.calls.len(), 1);
    assert_eq!(report.calls[0].label.as_deref(), Some("$ cargo test"));
    assert_eq!(
        turns,
        vec![
            Turn {
                start: t(4),
                completed_at: Some(t(6)),
                last_record_at: t(6),
                prompt: Some("Synthetic prompt".into()),
                last_agent_message: Some("Synthetic done".into())
            },
            Turn {
                start: t(7),
                completed_at: None,
                last_record_at: t(8),
                prompt: None,
                last_agent_message: None
            }
        ]
    );
    let (report, turns) = parse(vec![
        event("task_started", "external-import-2", 0),
        call("skip", json!("tools.write_stdin({})"), 1),
    ]);
    assert!(report.calls.is_empty());
    assert!(turns.is_empty());
}

#[test]
fn malformed_records_fail_soft_and_string_mentions_are_not_edits() {
    let (report, _) = parse(vec![
        record(
            "response_item",
            json!({"type":"custom_tool_call","input":"missing id"}),
            0,
        ),
        call(
            "valid",
            json!(r#"tools.exec_command({cmd:"echo tools.apply_patch"})"#),
            1,
        ),
        record("response_item", json!({"type":"future_tool"}), 2),
    ]);
    assert_eq!(report.calls.len(), 1);
    assert_eq!(report.calls[0].kind, CallKind::Run);
    assert_eq!(report.calls[0].label, None);
    assert_eq!(report.skipped.len(), 2);
}

#[test]
fn first_command_property_and_completion_without_message() {
    let (report, turns) = parse(vec![
        event("task_started", "real", 0),
        call(
            "quoted",
            json!(r#"tools.exec_command({workdir:"/synthetic/cmd:noise", "cmd": "git diff"})"#),
            1,
        ),
        call(
            "variable",
            json!(r#"tools.exec_command({cmd: variable}); tools.exec_command({cmd:"cargo test"})"#),
            2,
        ),
        event("task_complete", "real", 3),
        record("turn_context", json!({}), 4),
    ]);
    assert_eq!(report.calls[0].label.as_deref(), Some("$ git diff"));
    assert_eq!(
        (report.calls[1].kind, report.calls[1].label.as_deref()),
        (CallKind::Run, None)
    );
    assert_eq!(
        turns,
        vec![Turn {
            start: t(0),
            completed_at: Some(t(3)),
            last_record_at: t(3),
            prompt: None,
            last_agent_message: None
        }]
    );
}

#[test]
fn phase8_real_shapes_are_supported() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sessions-phase8");
    let discovered = multi_link_codex::discover(&root).unwrap();
    let mut count = 0;
    for rollout in discovered.rollouts {
        let (report, turns) = read_details(&root.join(rollout.transcript)).unwrap();
        assert!(report.skipped.is_empty());
        assert_eq!(turns.len(), rollout.turns.len());
        count += report.calls.len();
    }
    assert_eq!(count, 50);
}

/// Explicit opt-in; never part of network-free, synthetic-only normal tests.
/// Prints counts only. Reads the home-shaped root without copying any real text.
#[test]
#[ignore]
fn real_counts() {
    let root = std::env::var_os("MULTI_LINK_COUNTS_ROOT").expect("set counts root explicitly");
    let root = Path::new(&root);
    let discovery = multi_link_codex::discover(root).unwrap();
    let mut kinds = [0usize; 4];
    let mut null = 0;
    let mut skipped = 0;
    let mut rollouts = 0;
    let mut unreadable = discovery.unreadable.len();
    for rollout in discovery.rollouts.into_iter().filter(|r| !r.imported) {
        match calls(&root.join(rollout.transcript)) {
            Ok(report) => {
                rollouts += 1;
                skipped += report.skipped.len();
                for call in report.calls {
                    kinds[match call.kind {
                        CallKind::Read => 0,
                        CallKind::Edit => 1,
                        CallKind::Run => 2,
                        CallKind::Handoff => 3,
                    }] += 1;
                    null += usize::from(call.ok.is_none());
                }
            }
            Err(_) => unreadable += 1,
        }
    }
    let total: usize = kinds.iter().sum();
    println!(
        "rollouts={rollouts} reads={} edits={} runs={} handoffs={} total={total} ok_null={null} ok_null_share={:.6} records_skipped={skipped} unreadable={unreadable}",
        kinds[0],
        kinds[1],
        kinds[2],
        kinds[3],
        if total == 0 {
            0.0
        } else {
            null as f64 / total as f64
        }
    );
}

fn user_response(text: &str, second: u8) -> Value {
    record(
        "response_item",
        json!({"type":"message","role":"user","content":[{"type":"input_text","text":text}]}),
        second,
    )
}

#[test]
fn turn_prompts_use_filtered_response_fallback_and_first_event_priority() {
    for event_first in [false, true] {
        let event_prompt = record(
            "event_msg",
            json!({"type":"user_message","message":" First   event "}),
            6,
        );
        let mut records = vec![event("task_started", "a", 0)];
        if event_first {
            records.push(event_prompt.clone());
        }
        records.extend([
            user_response("   # AGENTS.md instructions synthetic", 1),
            user_response("  <environment_context>synthetic", 2),
            user_response("   ", 3),
            user_response("  First   response ", 4),
            user_response("Later response", 5),
        ]);
        if !event_first {
            records.push(event_prompt);
        }
        records.extend([
            record(
                "event_msg",
                json!({"type":"user_message","message":"Later event"}),
                7,
            ),
            event("task_complete", "a", 8),
            event("task_started", "b", 9),
            user_response("   # AGENTS.md instructions synthetic", 10),
            user_response(" <wrapper>", 11),
            user_response(" ", 12),
            user_response(" First   fallback ", 13),
            user_response("Later fallback", 14),
            event("task_complete", "b", 15),
            event("task_started", "c", 16),
            user_response(" <wrapper>", 17),
        ]);
        let (report, turns) = parse(records);
        assert!(report.skipped.is_empty());
        assert_eq!(
            turns
                .iter()
                .map(|t| t.prompt.as_deref())
                .collect::<Vec<_>>(),
            vec![Some("First event"), Some("First fallback"), None]
        );
    }
}

#[test]
fn additional_tools_is_a_known_non_call_record() {
    let (report, turns) = parse(vec![record(
        "response_item",
        json!({"type":"additional_tools"}),
        0,
    )]);
    assert!(report.calls.is_empty());
    assert!(report.skipped.is_empty());
    assert!(turns.is_empty());
}
