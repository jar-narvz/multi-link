use std::fs;

use multi_link_claude::parse_file;
use multi_link_model::Timestamp;
use serde_json::{Value, json};

#[test]
fn last_turn_timestamp_is_maximum_own_main_turn_timestamp() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("main.jsonl");
    let early = "2026-11-09T08:00:00Z";
    let later = "2026-11-09T08:02:00Z";
    let latest = "2026-11-09T08:03:00Z";
    let mut records = Vec::new();
    for (mut record, expected) in [
        (
            json!({"type":"user","isMeta":true,"timestamp":latest}),
            None,
        ),
        (json!({"type":"assistant","timestamp":later}), Some(later)),
        (json!({"type":"assistant","timestamp":early}), Some(later)),
        (
            json!({"type":"assistant","timestamp":latest,"sessionId":"inherited"}),
            Some(later),
        ),
        (
            json!({"type":"user","timestamp":latest,"message":{"content":"<command-name>/exit</command-name>"}}),
            Some(later),
        ),
        (
            json!({"type":"user","timestamp":latest,"message":{"content":"<local-command-stdout>bye</local-command-stdout>"}}),
            Some(later),
        ),
        (json!({"type":"cost-state","timestamp":latest}), Some(later)),
        (
            json!({"type":"user","timestamp":latest,"message":{"content":[{"type":"tool_result","tool_use_id":"t","content":"ok"}]}}),
            Some(latest),
        ),
        (
            json!({"type":"assistant","timestamp":"invalid"}),
            Some(latest),
        ),
        (json!({"type":"assistant"}), Some(latest)),
    ] {
        if record.get("sessionId").is_none() {
            record["sessionId"] = json!("main");
        }
        records.push(record);
        let text = records
            .iter()
            .map(|v: &Value| format!("{v}\n"))
            .collect::<String>();
        fs::write(&path, text).unwrap();
        assert_eq!(
            parse_file(root.path(), &path, false)
                .unwrap()
                .transcript
                .last_turn_at,
            expected.map(|s| Timestamp::parse(s).unwrap())
        );
        assert_eq!(
            parse_file(root.path(), &path, true)
                .unwrap()
                .transcript
                .last_turn_at,
            None
        );
    }
}
