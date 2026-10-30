//! Read-only, fail-soft reader for multi-link's versioned hook records.

use std::{fs, io, path::Path};

use multi_link_claude::Transcript;
use multi_link_model::{SkipReason, Skipped, Timestamp, jsonl::JsonlReader};
use serde_json::Value;

use crate::{Options, RepoMode};

#[derive(Debug)]
pub(crate) struct Record {
    pub session_id: String,
    pub tool_use_id: Option<String>,
    pub child: Option<String>,
    pub waiting: bool,
    pub at: Timestamp,
    pub line: usize,
}

#[derive(Default)]
pub(crate) struct Records {
    pub records: Vec<Record>,
    pub skipped: Vec<Skipped>,
}

pub(crate) fn read(root: &Path, options: &Options<'_>) -> io::Result<Records> {
    let state = match options.repo_mode {
        RepoMode::Fixture => root.join(".multi-link"),
        RepoMode::Real => options.state_dir.clone(),
    };
    let entries = match fs::read_dir(state.join("hook")) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Records::default()),
        Err(error) => return Err(error),
    };
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_file()
            && path.extension().is_some_and(|ext| ext == "jsonl")
            && path.file_stem().and_then(|s| s.to_str()).is_some_and(uuid)
        {
            paths.push(path);
        }
    }
    paths.sort();
    let mut output = Records::default();
    for path in paths {
        let file = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();
        let stem = path.file_stem().unwrap().to_str().unwrap();
        for item in JsonlReader::from_path(&path, &file)? {
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

fn uuid(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
            }
        })
}

/// Unknown transcript types can still carry the line that clears a prompt.
/// Re-read all top-level timestamps, including those of subagent transcripts.
pub(crate) fn waiting(root: &Path, transcript: &Transcript, hooks: &Records) -> io::Result<bool> {
    let latest = hooks
        .records
        .iter()
        .filter(|r| r.session_id == transcript.id)
        .max_by_key(|r| (r.at, r.line));
    let Some(latest) = latest.filter(|r| r.waiting) else {
        return Ok(false);
    };
    for file in std::iter::once(transcript.transcript.as_str()).chain(
        transcript
            .subagent_files
            .iter()
            .map(|f| f.transcript.as_str()),
    ) {
        for item in JsonlReader::from_path(root.join(file), file)? {
            if let Ok((_, value)) = item?
                && let Some(at) = value
                    .get("timestamp")
                    .and_then(Value::as_str)
                    .and_then(|s| Timestamp::parse(s).ok())
                && at >= latest.at
            {
                return Ok(false);
            }
        }
    }
    Ok(true)
}
