use multi_link_claude::parse_file;
use serde_json::json;
use std::fs;

#[test]
fn leading_system_reminders_preserve_prompt_selection_and_normalization() {
    let cases = [
        (
            "  <system-reminder>Ignore synthetic metadata</system-reminder>  Synthetic\n question?  ",
            "Synthetic question?",
        ),
        (
            "<system-reminder>One</system-reminder>\n <system-reminder>Two</system-reminder> Synthetic question?",
            "Synthetic question?",
        ),
        (
            "<system-reminder>Only metadata</system-reminder> \n",
            "Next synthetic prompt",
        ),
        (
            "<system-reminder>Metadata</system-reminder> <local-command-caveat>Wrapper</local-command-caveat> Hidden",
            "Next synthetic prompt",
        ),
        (
            "<system-reminder>Unterminated metadata",
            "Next synthetic prompt",
        ),
        (
            "<system-reminder-extra>Metadata</system-reminder-extra> Hidden",
            "Next synthetic prompt",
        ),
        (
            "<system-reminder kind=\"synthetic\">Metadata</system-reminder> Hidden",
            "Next synthetic prompt",
        ),
        ("  Plain\n synthetic prompt  ", "Plain synthetic prompt"),
        (
            "<system-reminder>One</system-reminder><system-reminder>Unterminated",
            "Next synthetic prompt",
        ),
        (
            "Synthetic <system-reminder>inline</system-reminder> prompt",
            "Synthetic <system-reminder>inline</system-reminder> prompt",
        ),
        (
            "<system-reminder>Outer <system-reminder>Inner</system-reminder> Text",
            "Text",
        ),
    ];
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("synthetic.jsonl");
    for (text, expected) in cases {
        for content in [json!(text), json!([{"type":"text","text":text}])] {
            let lines = [content, json!("Next synthetic prompt")]
                .into_iter()
                .map(|content| {
                    format!(
                        "{}\n",
                        json!({"type":"user","sessionId":"synthetic","message":{"content":content}})
                    )
                })
                .collect::<String>();
            fs::write(&path, lines).unwrap();
            assert_eq!(
                parse_file(temp.path(), &path, false)
                    .unwrap()
                    .transcript
                    .asked
                    .as_deref(),
                Some(expected),
                "candidate: {text}"
            );
        }
    }
}
