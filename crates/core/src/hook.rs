//! Read-only, fail-soft reader for multi-link's versioned hook records.

use std::{io, path::Path};

use multi_link_model::{SkipReason, Skipped, Timestamp, jsonl::JsonlReader};
use serde_json::Value;

#[derive(Debug, Clone)]
pub(crate) struct Record {
    pub session_id: String,
    pub tool_use_id: Option<String>,
    pub child: Option<String>,
    pub waiting: bool,
    pub at: Timestamp,
    pub line: usize,
}

#[derive(Default, Clone)]
pub(crate) struct Records {
    pub records: Vec<Record>,
    pub skipped: Vec<Skipped>,
}

pub(crate) fn parse_file(root: &Path, path: &Path) -> io::Result<Records> {
    let file = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned();
    let stem = path.file_stem().unwrap().to_str().unwrap();
    let mut output = Records::default();
    for item in JsonlReader::from_path(path, &file)? {
        match item? {
            Err(skipped) => output.skipped.push(skipped),
            Ok((line, value)) => {
                if let Some(record) = parse(&value, stem, line) {
                    output.records.push(record);
                } else {
                    output.skipped.push(Skipped {
                        file: file.clone(),
                        line,
                        reason: if value.is_object() {
                            SkipReason::UnknownType
                        } else {
                            SkipReason::InvalidJson
                        },
                    });
                }
            }
        }
    }
    Ok(output)
}

fn nullable_string(value: &Value, field: &str) -> bool {
    value
        .get(field)
        .is_some_and(|v| v.is_null() || v.is_string())
}

fn parse(value: &Value, stem: &str, line: usize) -> Option<Record> {
    if value.get("v")?.as_u64()? != 1
        || value.get("session_id")?.as_str()? != stem
        || !value.get("cwd")?.is_string()
        || ![
            "agent_id",
            "tool_use_id",
            "child",
            "child_evidence",
            "notification_type",
        ]
        .iter()
        .all(|field| nullable_string(value, field))
    {
        return None;
    }
    let at = Timestamp::parse(value.get("at")?.as_str()?).ok()?;
    let kind = value.get("kind")?.as_str()?;
    let event = value.get("event")?.as_str()?;
    let tool = value["tool_use_id"].as_str();
    let child = value["child"].as_str();
    let evidence = value["child_evidence"].as_str();
    let notification = value["notification_type"].as_str();
    let valid = match kind {
        "handoff" => {
            matches!(event, "PostToolUse" | "PostToolUseFailure")
                && tool.is_some()
                && notification.is_none()
                && match (child, evidence) {
                    (None, None) => true,
                    (Some(id), Some("stdout" | "thread_ready" | "task_output")) => uuid(id),
                    _ => false,
                }
        }
        "waiting" => {
            tool.is_none()
                && child.is_none()
                && evidence.is_none()
                && match event {
                    "PermissionRequest" => notification.is_none(),
                    "Notification" => matches!(
                        notification,
                        Some("permission_prompt" | "elicitation_dialog")
                    ),
                    _ => false,
                }
        }
        _ => false,
    };
    valid.then(|| Record {
        session_id: stem.to_owned(),
        tool_use_id: tool.map(str::to_owned),
        child: child.map(str::to_owned),
        waiting: kind == "waiting",
        at,
        line,
    })
}

pub(crate) fn uuid(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
            }
        })
}
