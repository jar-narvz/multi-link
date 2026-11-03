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

#[test]
fn latest_prompt_skips_injected_records_and_updates_after_append() {
    for blocks in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("synthetic.jsonl");
        let user = |text: &str| {
            let content = if blocks {
                json!([{"type":"text","text":text}])
            } else {
                json!(text)
            };
            json!({"type":"user","sessionId":"synthetic","message":{"content":content}})
        };
        let first = user("Synthetic first prompt");
        let second = user("<system-reminder>Metadata</system-reminder> Synthetic\n second prompt");
        fs::write(&path, format!("{first}\n")).unwrap();
        assert_eq!(
            parse_file(temp.path(), &path, false)
                .unwrap()
                .transcript
                .last_asked
                .as_deref(),
            Some("Synthetic first prompt")
        );
        let mut meta = user("Synthetic injected text");
        meta["isMeta"] = json!(true);
        let mut compact = user("Synthetic summary");
        compact["isCompactSummary"] = json!(true);
        let mut inherited = user("Synthetic inherited text");
        inherited["sessionId"] = json!("other");
        let records = [
            first,
            second,
            meta,
            compact,
            user("[Request interrupted by user for tool use]"),
            user("<system-reminder>Metadata only</system-reminder>"),
            json!({"type":"user","sessionId":"synthetic","message":{"content":[{"type":"tool_result","content":"Synthetic result"},{"type":"text","text":"Synthetic result text"}]}}),
            inherited,
        ];
        fs::write(
            &path,
            records.iter().map(|r| format!("{r}\n")).collect::<String>(),
        )
        .unwrap();
        let parsed = parse_file(temp.path(), &path, false).unwrap().transcript;
        assert_eq!(parsed.asked.as_deref(), Some("Synthetic first prompt"));
        assert_eq!(
            parsed.last_asked.as_deref(),
            Some("Synthetic second prompt")
        );
        assert_eq!(
            parse_file(temp.path(), &path, true)
                .unwrap()
                .transcript
                .last_asked,
            None
        );
    }
}

#[test]
fn latest_exclusions_preserve_first_prompt_contract() {
    for (flag, text) in [
        ("isMeta", "Synthetic injected text"),
        ("isCompactSummary", "Synthetic summary"),
        ("unused", "[Request interrupted by user]"),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("synthetic.jsonl");
        let mut record = json!({"type":"user","sessionId":"synthetic","message":{"content":text}});
        record[flag] = json!(true);
        fs::write(&path, format!("{record}\n")).unwrap();
        let parsed = parse_file(temp.path(), &path, false).unwrap().transcript;
        assert_eq!(parsed.asked.as_deref(), Some(text));
        assert_eq!(parsed.last_asked, None);
    }
}
