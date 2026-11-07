use multi_link_model::CallKind;
use serde_json::{Value, json};
use std::{fs, path::Path};

fn record(kind: &str, owner: &str, blocks: Vec<Value>) -> Value {
    json!({"type":kind,"sessionId":owner,"timestamp":"2026-11-06T10:00:00Z","message":{"content":blocks}})
}
fn tool(id: &str, name: &str, input: Value) -> Value {
    json!({"type":"tool_use","id":id,"name":name,"input":input})
}
fn write(path: &Path, records: Vec<Value>) {
    fs::write(
        path,
        records.iter().map(|r| format!("{r}\n")).collect::<String>(),
    )
    .unwrap();
}

#[test]
fn every_claude_tool_results_ownership_and_deduplication() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("parent.jsonl");
    let mut blocks = Vec::new();
    for name in [
        "Read",
        "Grep",
        "Glob",
        "Edit",
        "Write",
        "NotebookEdit",
        "Agent",
        "Bash",
        "mcp__ccd_read",
    ] {
        blocks.push(tool(name, name, json!({"file_path":"/synthetic/a.rs","notebook_path":"/synthetic/a.ipynb","command":"git diff -- private"})));
    }
    write(
        &path,
        vec![
            record(
                "assistant",
                "other",
                vec![tool(
                    "copied",
                    "Bash",
                    json!({"command":"codex exec 'Step 1.1'"}),
                )],
            ),
            record(
                "api-request-blob",
                "parent",
                vec![tool("blob", "Read", json!({}))],
            ),
            record("assistant", "parent", blocks),
            record("assistant", "parent", vec![tool("Read", "Read", json!({}))]),
            record(
                "user",
                "other",
                vec![json!({"type":"tool_result","tool_use_id":"Edit","is_error":false})],
            ),
            record(
                "user",
                "parent",
                vec![
                    json!({"type":"tool_result","tool_use_id":"Read","is_error":false}),
                    json!({"type":"tool_result","tool_use_id":"Edit","is_error":true}),
                ],
            ),
        ],
    );
    let mut transcript = multi_link_claude::parse_file(dir.path(), &path, false)
        .unwrap()
        .transcript;
    let sub = dir.path().join("agent-sub.jsonl");
    write(
        &sub,
        vec![
            record("assistant", "parent", vec![tool("sub", "Read", json!({}))]),
            record(
                "assistant",
                "other",
                vec![tool("foreign-sub", "Read", json!({}))],
            ),
        ],
    );
    transcript
        .subagent_files
        .push(multi_link_claude::SubagentFile {
            id: "sub".into(),
            transcript: "agent-sub.jsonl".into(),
        });
    let report = multi_link_claude::calls(dir.path(), &transcript).unwrap();
    let actual: Vec<_> = report
        .calls
        .iter()
        .map(|c| (c.id.as_str(), c.kind, c.label.as_deref(), c.ok))
        .collect();
    assert_eq!(
        actual,
        vec![
            ("Read", CallKind::Read, None, Some(true)),
            ("Grep", CallKind::Read, None, None),
            ("Glob", CallKind::Read, None, None),
            ("Edit", CallKind::Edit, Some("edit a.rs"), Some(false)),
            ("Write", CallKind::Edit, Some("edit a.rs"), None),
            ("NotebookEdit", CallKind::Edit, Some("edit a.rs"), None),
            ("Agent", CallKind::Run, None, None),
            ("Bash", CallKind::Read, Some("$ git diff"), None),
            ("sub", CallKind::Read, None, None),
        ]
    );
    assert!(report.skipped.is_empty());
}

#[test]
fn handoff_forms_share_label_and_unknown_records_are_counted() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("parent.jsonl");
    write(
        &path,
        vec![
            record(
                "assistant",
                "parent",
                vec![
                    tool("exec", "Bash", json!({"command":"codex exec 'Step 1.1'"})),
                    tool(
                        "resume",
                        "Bash",
                        json!({"command":"codex resume 00000000-0000-4000-8000-000000000001"}),
                    ),
                    tool(
                        "plugin",
                        "Bash",
                        json!({"command":"node /synthetic/codex-companion.mjs task 'Step 1.1'"}),
                    ),
                    tool("commit", "Bash", json!({"command":"git commit -m private"})),
                ],
            ),
            record("future", "parent", vec![]),
        ],
    );
    let transcript = multi_link_claude::parse_file(dir.path(), &path, false)
        .unwrap()
        .transcript;
    let report = multi_link_claude::calls(dir.path(), &transcript).unwrap();
    assert_eq!(report.calls.len(), 4);
    for call in &report.calls[..3] {
        assert_eq!(
            (call.kind, call.label.as_deref()),
            (CallKind::Handoff, Some("▸ codex exec"))
        );
    }
    assert_eq!(
        (report.calls[3].kind, report.calls[3].label.as_deref()),
        (CallKind::Run, Some("$ git commit"))
    );
    assert_eq!(report.skipped.len(), 1);
    assert_eq!(report.skipped[0].line, 2);
}

#[test]
fn phase8_fork_does_not_own_copied_handoff() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sessions-phase8");
    let scan = multi_link_claude::scan(&root).unwrap();
    let parent = scan
        .transcripts
        .iter()
        .find(|t| t.id.starts_with("8112"))
        .unwrap();
    let fork = scan
        .transcripts
        .iter()
        .find(|t| t.id.starts_with("8113"))
        .unwrap();
    let parent_calls = multi_link_claude::calls(&root, parent).unwrap();
    let fork_calls = multi_link_claude::calls(&root, fork).unwrap();
    assert_eq!(
        parent_calls
            .calls
            .iter()
            .filter(|c| c.kind == CallKind::Handoff)
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>(),
        vec!["toolu_p8_8112s_4", "toolu_p8_8112_6"]
    );
    assert_eq!(
        fork_calls
            .calls
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>(),
        vec!["toolu_p8_8113_102", "toolu_p8_8113_104"]
    );
}

#[test]
fn missing_subagent_is_skipped_but_missing_main_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("parent.jsonl");
    write(
        &path,
        vec![record(
            "assistant",
            "parent",
            vec![tool("main", "Read", json!({}))],
        )],
    );
    let mut transcript = multi_link_claude::parse_file(dir.path(), &path, false)
        .unwrap()
        .transcript;
    transcript
        .subagent_files
        .push(multi_link_claude::SubagentFile {
            id: "gone".into(),
            transcript: "agent-gone.jsonl".into(),
        });
    let report = multi_link_claude::calls(dir.path(), &transcript).unwrap();
    assert_eq!(
        report
            .calls
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>(),
        vec!["main"]
    );
    assert!(report.skipped.is_empty());
    fs::remove_file(path).unwrap();
    assert_eq!(
        multi_link_claude::calls(dir.path(), &transcript)
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::NotFound
    );
}

#[test]
fn result_without_is_error_keeps_ok_unknown() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("parent.jsonl");
    write(
        &path,
        vec![
            record(
                "assistant",
                "parent",
                vec![
                    tool("read", "Read", json!({})),
                    tool("bash", "Bash", json!({"command":"cargo test"})),
                ],
            ),
            record(
                "user",
                "parent",
                vec![
                    json!({"type":"tool_result","tool_use_id":"read","content":"synthetic success"}),
                    json!({"type":"tool_result","tool_use_id":"bash","content":"synthetic success"}),
                ],
            ),
        ],
    );
    let transcript = multi_link_claude::parse_file(dir.path(), &path, false)
        .unwrap()
        .transcript;
    let report = multi_link_claude::calls(dir.path(), &transcript).unwrap();
    assert_eq!(
        report
            .calls
            .iter()
            .map(|c| (c.id.as_str(), c.ok))
            .collect::<Vec<_>>(),
        vec![("read", None), ("bash", None)]
    );
}
