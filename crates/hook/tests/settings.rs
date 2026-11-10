use multi_link_hook::settings::{check, snippet};
use serde_json::{Value, json};

const BIN: &str = "/new/multi-link";
fn inspect(value: &Value) -> multi_link_hook::settings::Check {
    check(&value.to_string(), BIN).unwrap()
}
fn status(value: &Value, event: &str) -> String {
    inspect(value)
        .findings
        .iter()
        .find(|f| f.event == event)
        .unwrap()
        .status
        .clone()
}

#[test]
fn missing_current_and_pre_session_end() {
    let empty = inspect(&json!({}));
    assert_eq!(
        empty
            .findings
            .iter()
            .map(|f| f.status.as_str())
            .collect::<Vec<_>>(),
        ["missing"; 5]
    );
    assert_eq!(empty.merged, snippet(BIN));
    let mut value = snippet(BIN);
    assert!(inspect(&value).current());
    assert!(!inspect(&value).needs_fix);
    value["hooks"].as_object_mut().unwrap().remove("SessionEnd");
    assert_eq!(status(&value, "SessionEnd"), "missing");
}

#[test]
fn outdated_paths_async_and_timeout_are_named() {
    let mut value = snippet("/old/multi-link");
    value["hooks"]["PostToolUse"][0]["hooks"][0]["async"] = json!(false);
    value["hooks"]["PostToolUse"][0]["hooks"][0]["timeout"] = json!(5);
    let result = inspect(&value);
    assert_eq!(result.findings[0].status, "outdated");
    let details = result.findings[0].details.join(" ");
    for expected in ["/old/multi-link", BIN, "async", "timeout"] {
        assert!(details.contains(expected), "{details}");
    }
    assert!(inspect(&result.merged).current());
}

#[test]
fn customizations_and_unknown_events_are_never_changed() {
    for matcher in ["", "*"] {
        let mut value = snippet(BIN);
        value["hooks"]["SessionEnd"][0]["matcher"] = json!(matcher);
        assert!(inspect(&value).current());
    }
    for key in ["if", "shell", "args"] {
        let mut value = snippet(BIN);
        value["hooks"]["PostToolUse"][0]["hooks"][0][key] = json!("intent");
        let result = inspect(&value);
        assert_eq!(status(&value, "PostToolUse"), "customized");
        assert_eq!(result.merged, value);
        assert!(!result.needs_fix);
    }
    let mut value = snippet(BIN);
    value["hooks"]["PostToolUse"][0]["matcher"] = json!("Bash|Edit");
    value["hooks"]["Stop"] =
        json!([{"hooks":[{"type":"command","command":"multi-link hook record"}]}]);
    value["disableAllHooks"] = json!(true);
    let result = inspect(&value);
    assert_eq!(status(&value, "PostToolUse"), "customized");
    assert!(
        result
            .findings
            .iter()
            .any(|f| f.status == "unknown event" && f.event == "Stop")
    );
    assert!(result.findings.iter().any(|f| f.event == "disableAllHooks"));
    assert_eq!(result.merged, value);
    assert!(!result.current());
}

#[test]
fn duplicate_merge_keeps_user_entries_order_and_keys() {
    let source = r#"{"z":7,"hooks":{"PostToolUse":[{"matcher":"Bash","hooks":[{"type":"command","command":"echo before"},{"type":"command","command":"multi-link hook record"},{"type":"command","command":"echo after"}]},{"matcher":"Bash","hooks":[{"type":"command","command":"'/old/multi-link' hook record"}]}],"Other":[]},"a":{"n":42}}"#;
    let result = check(source, BIN).unwrap();
    assert_eq!(result.findings[0].status, "duplicate");
    assert_eq!(
        result
            .merged
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["z", "hooks", "a"]
    );
    assert_eq!(
        result.merged["hooks"]["PostToolUse"][0],
        json!({"matcher":"Bash","hooks":[{"type":"command","command":"echo before"},{"type":"command","command":"echo after"}]})
    );
    assert_eq!(
        result.merged["hooks"]["PostToolUse"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(result.merged["a"], json!({"n":42}));
    assert_eq!(result.merged["hooks"]["Other"], json!([]));
}

#[test]
fn recognition_requires_exact_binary_and_suffix() {
    for command in [
        "echo multi-link hook record",
        "multi-link hook record extra",
        "other hook record",
        "multi-link hook records",
        "multi-link; hook record",
    ] {
        let value =
            json!({"hooks":{"SessionEnd":[{"hooks":[{"type":"command","command":command}]}]}});
        assert_eq!(status(&value, "SessionEnd"), "missing", "{command}");
        assert_eq!(
            inspect(&value).merged["hooks"]["SessionEnd"][0],
            value["hooks"]["SessionEnd"][0]
        );
    }
    let value = snippet("/a b/it's $(safe)`/multi-link");
    assert!(
        inspect(&value)
            .findings
            .iter()
            .all(|f| f.status == "outdated")
    );
}

#[test]
fn refuses_duplicate_keys_lossy_numbers_and_bad_shapes() {
    for source in [
        "",
        "{",
        "[]",
        "null",
        r#"{"a":1,"a":2}"#,
        r#"{"a":{"x":1,"\u0078":2}}"#,
        r#"{"x":1e400}"#,
        r#"{"x":18446744073709551616}"#,
        r#"{"x":0.123456789012345678901}"#,
        r#"{"hooks":null}"#,
        r#"{"hooks":[]}"#,
        r#"{"hooks":{"Stop":{}}}"#,
        r#"{"hooks":{"Stop":[null]}}"#,
        r#"{"hooks":{"Stop":[{}]}}"#,
        r#"{"hooks":{"Stop":[{"hooks":{}}]}}"#,
    ] {
        assert!(check(source, BIN).is_err(), "{source}");
    }
    for source in [
        r#"{"x":18446744073709551615}"#,
        r#"{"x":-9223372036854775808}"#,
        r#"{"x":1.25,"s":"123 \\\" 1e400"}"#,
    ] {
        assert!(check(source, BIN).is_ok(), "{source}");
    }
}

#[test]
fn merge_preserves_preexisting_empty_groups_and_null_matcher_is_custom() {
    let source = json!({"hooks":{"SessionEnd":[{"matcher":"mine","hooks":[]},{"hooks":[{"type":"command","command":"/old/multi-link hook record"}]}]}});
    let result = inspect(&source);
    assert_eq!(
        result.merged["hooks"]["SessionEnd"][0],
        json!({"matcher":"mine","hooks":[]})
    );
    assert_eq!(
        result.merged["hooks"]["SessionEnd"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let mut source = snippet(BIN);
    source["hooks"]["SessionEnd"][0]["matcher"] = Value::Null;
    assert_eq!(status(&source, "SessionEnd"), "customized");
    assert_eq!(inspect(&source).merged, source);
}

#[test]
fn bare_home_relative_binary_is_outdated_rather_than_missing() {
    let source = json!({"hooks":{"SessionEnd":[{"hooks":[{"type":"command","command":"~/.cargo/bin/multi-link hook record","async":true,"timeout":60}]}]}});
    let result = inspect(&source);
    let event = result
        .findings
        .iter()
        .find(|f| f.event == "SessionEnd")
        .unwrap();
    assert_eq!(event.status, "outdated");
    assert!(event.details[0].contains("~/.cargo/bin/multi-link"));
}
