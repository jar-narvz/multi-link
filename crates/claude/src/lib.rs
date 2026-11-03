//! Read-only discovery of Claude transcripts and live-session records.

mod invocation;

pub use invocation::{Invocation, Launch, invocations, recognise};

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use multi_link_model::{SkipReason, Skipped, Timestamp, jsonl::JsonlReader};
use serde_json::Value;

/// Paths and skipped entries are sorted by root-relative path (then line).
#[derive(Debug, Default)]
pub struct ClaudeScan {
    pub transcripts: Vec<Transcript>,
    pub records: Vec<SessionRecord>,
    /// Omitted session-record files, as sorted root-relative paths (no line number).
    pub bad_records: Vec<String>,
    pub skipped: Vec<Skipped>,
}

/// Metadata comes only from known records in the main transcript, in file order.
/// Missing or wrongly typed metadata remains None; discovery is still exposed
/// so consumers can re-read even an empty or partially written transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transcript {
    pub id: String,
    pub transcript: String,
    pub cwd: Option<String>,
    pub branch: Option<String>,
    pub started_at: Option<Timestamp>,
    pub last_activity: Option<Timestamp>,
    pub subagents: Vec<String>,
    pub subagent_files: Vec<SubagentFile>,
    pub activity_timestamps: Vec<Timestamp>,
    pub asked: Option<String>,
    pub last_asked: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubagentFile {
    pub id: String,
    /// Relative to the scan root, just like Transcript::transcript.
    pub transcript: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRecord {
    pub pid: u32,
    pub proc_start: String,
    pub session_id: String,
    pub cwd: String,
    /// Preserved verbatim, including statuses introduced by future versions.
    pub status: String,
    pub status_updated_at: Timestamp,
    pub started_at: Timestamp,
    pub updated_at: Timestamp,
    pub record: String,
    /// `hostSessionId` (desktop app id, `local_…`) verbatim; None when absent or not a string.
    pub host_session_id: Option<String>,
}

/// Scan the home-shaped root. Missing discovery directories are empty.
/// Transcripts missing at open time are omitted; other transcript and directory
/// I/O errors propagate. Unreadable, invalid, or incomplete session-record JSON
/// files are omitted and listed in `bad_records` (they have no line number).
pub fn scan(root: &Path) -> io::Result<ClaudeScan> {
    let mut scan = ClaudeScan::default();
    for project in children(&root.join(".claude/projects"))? {
        if !project.is_dir() {
            continue;
        }
        for path in children(&project)? {
            if !path.is_file() || path.extension().is_none_or(|ext| ext != "jsonl") {
                continue;
            }
            let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            let parsed = match parse_file(root, &path, false) {
                Ok(parsed) => parsed,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error),
            };
            let mut transcript = parsed.transcript;
            scan.skipped.extend(parsed.skipped);
            for subagent_path in children(&project.join(id).join("subagents"))? {
                if !subagent_path.is_file()
                    || subagent_path.extension().is_none_or(|ext| ext != "jsonl")
                {
                    continue;
                }
                let Some(agent_id) = subagent_path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .and_then(|stem| stem.strip_prefix("agent-"))
                    .filter(|id| !id.is_empty())
                else {
                    continue;
                };
                let parsed = match parse_file(root, &subagent_path, true) {
                    Ok(parsed) => parsed,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                    Err(error) => return Err(error),
                };
                scan.skipped.extend(parsed.skipped);
                transcript
                    .activity_timestamps
                    .extend(parsed.transcript.activity_timestamps);
                transcript.subagents.push(agent_id.to_owned());
                transcript.subagent_files.push(SubagentFile {
                    id: agent_id.to_owned(),
                    transcript: parsed.transcript.transcript,
                });
            }
            transcript.subagents.sort();
            scan.transcripts.push(transcript);
        }
    }
    for path in children(&root.join(".claude/sessions"))? {
        if path.extension().is_some_and(|ext| ext == "json") {
            if let Some(record) = read_session_record(root, &path) {
                scan.records.push(record);
            } else {
                scan.bad_records.push(relative_path(root, &path));
            }
        }
    }
    scan.transcripts
        .sort_by(|a, b| a.transcript.cmp(&b.transcript));
    scan.records.sort_by(|a, b| a.record.cmp(&b.record));
    scan.bad_records.sort();
    scan.skipped
        .sort_by(|a, b| (&a.file, a.line).cmp(&(&b.file, b.line)));
    Ok(scan)
}

fn children(directory: &Path) -> io::Result<Vec<PathBuf>> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut paths = entries
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<Vec<_>>>()?;
    paths.sort();
    Ok(paths)
}

fn relative_path(root: &Path, path: &Path) -> String {
    // All callers discover paths by joining beneath root.
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

fn known_type(kind: &str) -> bool {
    matches!(
        kind,
        "agent-name"
            | "api-request"
            | "api-request-blob"
            | "api-request-shape"
            | "assistant"
            | "atis-latch"
            | "attachment"
            | "cost-state"
            | "custom-title"
            | "file-history-delta"
            | "file-history-snapshot"
            | "last-prompt"
            | "mode"
            | "queue-operation"
            | "system"
            | "user"
    )
}

pub fn read_session_record(root: &Path, path: &Path) -> Option<SessionRecord> {
    let value: Value = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
    let string = |key: &str| value.get(key)?.as_str().map(str::to_owned);
    let timestamp = |key: &str| Timestamp::from_epoch_ms(value.get(key)?.as_i64()?).ok();
    Some(SessionRecord {
        pid: value.get("pid")?.as_u64()?.try_into().ok()?,
        proc_start: string("procStart")?,
        session_id: string("sessionId")?,
        cwd: string("cwd")?,
        status: string("status")?,
        status_updated_at: timestamp("statusUpdatedAt")?,
        started_at: timestamp("startedAt")?,
        updated_at: timestamp("updatedAt")?,
        record: relative_path(root, path),
        host_session_id: string("hostSessionId"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_file_reports_missing_at_open_but_reads_empty_file() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("x.jsonl");
        fs::write(&path, "").unwrap();
        assert!(path.is_file());
        // Deterministically model deletion after discovery, before open.
        fs::remove_file(&path).unwrap();
        assert_eq!(
            parse_file(root.path(), &path, false).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );

        fs::write(&path, "").unwrap();
        let parsed = parse_file(root.path(), &path, false).unwrap();
        assert_eq!(parsed.transcript.id, "x");
        assert_eq!(parsed.transcript.cwd, None);
        assert_eq!(parsed.transcript.last_activity, None);
        assert!(parsed.skipped.is_empty());
        assert!(parsed.invocations.is_empty());
        assert_eq!(parsed.latest_timestamp, None);
    }
}

/// One independently cacheable transcript. Subagent discovery belongs to the caller.
#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub transcript: Transcript,
    pub skipped: Vec<Skipped>,
    pub invocations: Vec<Invocation>,
    /// Includes unknown record types, which can clear hook waiting.
    pub latest_timestamp: Option<Timestamp>,
}

pub fn parse_file(root: &Path, path: &Path, subagent: bool) -> io::Result<ParsedFile> {
    let id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let mut transcript = Transcript {
        id: id.to_owned(),
        transcript: relative_path(root, path),
        cwd: None,
        branch: None,
        started_at: None,
        last_activity: None,
        subagents: Vec::new(),
        subagent_files: Vec::new(),
        activity_timestamps: Vec::new(),
        asked: None,
        last_asked: None,
    };
    let activity_id = if subagent {
        path.parent()
            .and_then(Path::parent)
            .and_then(Path::file_name)
            .and_then(|s| s.to_str())
            .unwrap_or_default()
    } else {
        id
    };
    let mut skipped = Vec::new();
    let mut latest_timestamp = None;
    let mut call_owners = std::collections::HashMap::new();
    for item in JsonlReader::from_path(path, &transcript.transcript)? {
        let (line, value) = match item? {
            Ok(item) => item,
            Err(skip) => {
                skipped.push(skip);
                continue;
            }
        };
        if subagent
            && value["type"] == "assistant"
            && let Some(blocks) = value.pointer("/message/content").and_then(Value::as_array)
        {
            for block in blocks {
                if block["type"] == "tool_use"
                    && let Some(id) = block["id"].as_str()
                {
                    call_owners.entry(id.to_owned()).or_insert_with(|| {
                        (
                            value["sessionId"].as_str().map(str::to_owned),
                            value["agentId"].as_str().map(str::to_owned),
                        )
                    });
                }
            }
        }
        let at = value
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(|s| Timestamp::parse(s).ok());
        latest_timestamp = latest_timestamp.max(at);
        if value["sessionId"]
            .as_str()
            .is_none_or(|id| id == activity_id)
        {
            transcript.activity_timestamps.extend(at);
        }
        if !subagent
            && value["type"] == "user"
            && value["sessionId"].as_str() == Some(id)
            && let Some(prompt) = prompt(&value["message"]["content"])
        {
            transcript.asked.get_or_insert_with(|| prompt.clone());
            if value["isMeta"] != true
                && value["isCompactSummary"] != true
                && !prompt.starts_with("[Request interrupted by user")
            {
                transcript.last_asked = Some(prompt);
            }
        }
        if !value
            .get("type")
            .and_then(Value::as_str)
            .is_some_and(known_type)
        {
            skipped.push(Skipped {
                file: transcript.transcript.clone(),
                line,
                reason: SkipReason::UnknownType,
            });
            continue;
        }
        if let Some(cwd) = value.get("cwd").and_then(Value::as_str) {
            transcript.cwd = Some(cwd.to_owned());
        }
        if let Some(branch) = value.get("gitBranch").and_then(Value::as_str) {
            transcript.branch = Some(branch.to_owned());
        }
        if let Some(at) = at {
            transcript.started_at.get_or_insert(at);
            transcript.last_activity = Some(at);
        }
    }
    let scan = ClaudeScan {
        transcripts: vec![transcript.clone()],
        ..Default::default()
    };
    let mut calls = invocations(root, &scan)?;
    if subagent {
        calls.retain_mut(|call| {
            let Some((Some(parent), agent)) = call_owners.get(&call.invocation) else {
                return false;
            };
            call.parent = parent.clone();
            call.subagent = agent.clone();
            true
        });
    }
    Ok(ParsedFile {
        transcript,
        skipped,
        invocations: calls,
        latest_timestamp,
    })
}

fn prompt(content: &Value) -> Option<String> {
    let text = if let Some(blocks) = content.as_array() {
        if blocks.iter().any(|b| b["type"] == "tool_result") {
            return None;
        }
        blocks.iter().find(|b| b["type"] == "text")?["text"].as_str()?
    } else {
        content.as_str()?
    };
    let mut text = text.trim_start();
    while let Some(reminder) = text.strip_prefix("<system-reminder>") {
        let (_, rest) = reminder.split_once("</system-reminder>")?;
        text = rest.trim_start();
    }
    if text.is_empty() || text.starts_with('<') {
        return None;
    }
    Some(multi_link_model::normalize_prompt(text))
}
