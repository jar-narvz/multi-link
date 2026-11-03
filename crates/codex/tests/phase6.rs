use multi_link_codex::{Rollout, parse_file};
use multi_link_model::Timestamp;
use serde_json::{Value, json};
use std::{fs, path::Path};
fn parse(events: Vec<Value>) -> Rollout {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("rollout-synthetic.jsonl");
    let header = json!({"type":"session_meta","timestamp":"2026-11-03T06:00:00Z","payload":{"id":"synthetic","cwd":"/synthetic","timestamp":"2026-11-03T06:00:00Z"}});
    fs::write(
        &path,
        std::iter::once(header)
            .chain(events)
            .map(|v| format!("{v}\n"))
            .collect::<String>(),
    )
    .unwrap();
    parse_file(temp.path(), Path::new(&path), &mut Vec::new())
        .unwrap()
        .unwrap()
}
fn event(kind: &str, id: &str, second: u8) -> Value {
    json!({"type":"event_msg","timestamp":format!("2026-11-03T06:00:{second:02}Z"),"payload":{"type":kind,"turn_id":id,"message":"Synthetic prompt"}})
}
fn t(second: u8) -> Timestamp {
    Timestamp::parse(&format!("2026-11-03T06:00:{second:02}Z")).unwrap()
}
#[test]
fn all_import_even_with_records_outside_turns_has_no_activity_prompt_or_completion() {
    let rollout = parse(vec![
        event("thread_settings_applied", "", 1),
        event("task_started", "external-import-a", 2),
        event("user_message", "", 3),
        event("task_complete", "external-import-a", 4),
        event("thread_settings_applied", "", 5),
    ]);
    assert!(rollout.imported);
    assert_eq!(rollout.activity_timestamps, vec![]);
    assert_eq!(rollout.asked, None);
    assert_eq!(rollout.last_real_complete, None);
    assert_eq!(rollout.last_activity, t(5));
    assert_eq!(rollout.last_turn_complete(), Some(true));
}
#[test]
fn unmatched_complete_does_not_close_import_and_real_start_interrupts_it() {
    let rollout = parse(vec![
        event("task_started", "external-import-a", 1),
        event("task_complete", "unmatched", 2),
        event("user_message", "", 3),
        event("task_started", "real", 4),
        event("user_message", "", 5),
        event("task_complete", "real", 6),
    ]);
    assert!(!rollout.imported);
    assert_eq!(rollout.activity_timestamps, vec![t(4), t(5), t(6)]);
    assert_eq!(rollout.asked.as_deref(), Some("Synthetic prompt"));
    assert_eq!(rollout.last_real_complete, Some(t(6)));
    assert_eq!(rollout.turns[0].completed_at, None);
}
#[test]
fn mixed_import_closes_inclusively_and_next_import_stays_excluded_through_eof() {
    let rollout = parse(vec![
        event("task_started", "external-import-a", 1),
        event("task_complete", "external-import-a", 2),
        event("user_message", "", 3),
        event("task_started", "real", 4),
        event("task_complete", "real", 5),
        event("task_started", "external-import-b", 6),
        event("user_message", "", 7),
    ]);
    assert!(!rollout.imported);
    assert_eq!(rollout.activity_timestamps, vec![t(3), t(4), t(5)]);
    assert_eq!(rollout.last_real_complete, Some(t(5)));
    assert_eq!(rollout.last_turn_complete(), Some(false));
}
#[test]
fn no_import_or_no_turns_preserve_prompt_and_skip_metadata_activity() {
    for events in [
        vec![event("user_message", "", 1)],
        vec![
            event("task_started", "real", 1),
            event("user_message", "", 2),
            event("task_complete", "real", 3),
        ],
    ] {
        let expected: Vec<_> = (1..=events.len() as u8).map(t).collect();
        let rollout = parse(events);
        assert!(!rollout.imported);
        assert_eq!(rollout.activity_timestamps, expected);
        assert_eq!(rollout.asked.as_deref(), Some("Synthetic prompt"));
    }
}

fn response(role: &str, texts: &[&str]) -> Value {
    json!({"type":"response_item","timestamp":"2026-11-03T06:00:02Z",
        "payload":{"type":"message","role":role,"content":texts.iter().map(|text|
            json!({"type":"input_text","text":text})).collect::<Vec<_>>()}})
}

#[test]
fn response_prompt_fallback_skips_instruction_wrappers_and_uses_first_input_text() {
    let rollout = parse(vec![
        response("user", &["  # AGENTS.md instructions for /synthetic"]),
        response(
            "user",
            &[" <environment_context>synthetic</environment_context>"],
        ),
        response("user", &["  "]),
        response("assistant", &["Synthetic assistant text"]),
        response(
            "user",
            &["\n Build  a synthetic\nwidget. ", "Ignored second block"],
        ),
        response("user", &["Ignored later prompt"]),
    ]);
    assert_eq!(rollout.asked.as_deref(), Some("Build a synthetic widget."));
    assert_eq!(
        parse(vec![response(
            "user",
            &["<wrapper>", "Do not select this block"]
        )])
        .asked,
        None
    );
}

#[test]
fn user_message_has_priority_even_when_later_or_blank() {
    for prompt in ["Synthetic event prompt", ""] {
        let mut message = event("user_message", "", 3);
        message["payload"]["message"] = json!(prompt);
        for records in [
            vec![response("user", &["Synthetic fallback"]), message.clone()],
            vec![message, response("user", &["Synthetic fallback"])],
        ] {
            assert_eq!(parse(records).asked.as_deref(), Some(prompt));
        }
    }
}

#[test]
fn response_prompts_in_imports_stay_excluded() {
    let records = vec![
        event("task_started", "external-import-a", 1),
        response("user", &["Synthetic imported prompt"]),
        event("task_complete", "external-import-a", 3),
    ];
    assert_eq!(parse(records.clone()).asked, None);
    let mut mixed = records;
    mixed.extend([
        event("task_started", "real", 4),
        response("user", &["Synthetic real prompt"]),
    ]);
    assert_eq!(parse(mixed).asked.as_deref(), Some("Synthetic real prompt"));
}
