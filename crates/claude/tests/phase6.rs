use multi_link_claude::parse_file;
use multi_link_model::Timestamp;
use serde_json::json;
use std::fs;
#[test]
fn prompt_skips_inherited_tool_result_blank_and_wrappers_then_uses_first_text_block() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("synthetic.jsonl");
    let contents = [
        ("inherited", json!("Synthetic inherited")),
        (
            "synthetic",
            json!([{"type":"tool_result","content":"Synthetic tool"},{"type":"text","text":"Ignore tool record"}]),
        ),
        ("synthetic", json!(" \n\t ")),
        (
            "synthetic",
            json!("  <local-command-caveat>Synthetic wrapper"),
        ),
        (
            "synthetic",
            json!([{"type":"image"},{"type":"text","text":"  Synthetic\n text question?  "},{"type":"text","text":"Later text"}]),
        ),
        ("synthetic", json!("Later prompt")),
    ];
    let lines = contents.into_iter().map(|(id, content)| format!("{}\n",json!({"type":"user","sessionId":id,"timestamp":"2026-11-03T06:00:00Z","message":{"content":content}}))).collect::<String>();
    fs::write(&path, lines).unwrap();
    let parsed = parse_file(temp.path(), &path, false).unwrap().transcript;
    assert_eq!(parsed.asked.as_deref(), Some("Synthetic text question?"));
    assert_eq!(
        parsed.activity_timestamps,
        vec![Timestamp::parse("2026-11-03T06:00:00Z").unwrap(); 5]
    );
}
#[test]
fn multibyte_prompt_truncates_to_119_characters_plus_ellipsis() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("synthetic.jsonl");
    for (length, expected) in [
        (120, "界".repeat(120)),
        (121, format!("{}…", "界".repeat(119))),
    ] {
        fs::write(&path, format!("{}\n",json!({"type":"user","sessionId":"synthetic","message":{"content":"界".repeat(length)}}))).unwrap();
        assert_eq!(
            parse_file(temp.path(), &path, false)
                .unwrap()
                .transcript
                .asked,
            Some(expected)
        );
    }
}
