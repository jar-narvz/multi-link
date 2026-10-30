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
            let mut transcript = Transcript {
                id: id.to_owned(),
                transcript: relative_path(root, &path),
                cwd: None,
                branch: None,
                started_at: None,
                last_activity: None,
                subagents: Vec::new(),
                subagent_files: Vec::new(),
            };
            if !read_transcript(root, &path, &mut scan.skipped, |value| {
                if let Some(cwd) = value.get("cwd").and_then(Value::as_str) {
                    transcript.cwd = Some(cwd.to_owned());
                }
                if let Some(branch) = value.get("gitBranch").and_then(Value::as_str) {
                    transcript.branch = Some(branch.to_owned());
                }
                if let Some(timestamp) = value
                    .get("timestamp")
                    .and_then(Value::as_str)
                    .and_then(|text| Timestamp::parse(text).ok())
                {
                    transcript.started_at.get_or_insert(timestamp);
                    transcript.last_activity = Some(timestamp);
                }
            })? {
                continue;
            }
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
                if !read_transcript(root, &subagent_path, &mut scan.skipped, |_| {})? {
                    continue;
                }
                transcript.subagents.push(agent_id.to_owned());
                transcript.subagent_files.push(SubagentFile {
                    id: agent_id.to_owned(),
                    transcript: relative_path(root, &subagent_path),
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

/// Returns false only when the file disappeared before it could be opened.
fn read_transcript(
    root: &Path,
    path: &Path,
    skipped: &mut Vec<Skipped>,
    mut visit: impl FnMut(&Value),
) -> io::Result<bool> {
    let display_path = relative_path(root, path);
    let reader = match JsonlReader::from_path(path, &display_path) {
        Ok(reader) => reader,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    for record in reader {
        match record? {
            Err(skip) => skipped.push(skip),
            Ok((line, value)) => {
                if value
                    .get("type")
                    .and_then(Value::as_str)
                    .is_some_and(known_type)
                {
                    visit(&value);
                } else {
                    skipped.push(Skipped {
                        file: display_path.clone(),
                        line,
                        reason: SkipReason::UnknownType,
                    });
                }
            }
        }
    }
    Ok(true)
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

fn read_session_record(root: &Path, path: &Path) -> Option<SessionRecord> {
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transcript_missing_at_open_is_omitted_but_empty_file_is_read() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("x.jsonl");
        fs::write(&path, "").unwrap();
        assert!(path.is_file());
        // Deterministically model deletion after discovery, before open.
        fs::remove_file(&path).unwrap();
        let mut skipped = Vec::new();
        assert!(
            !read_transcript(root.path(), &path, &mut skipped, |_| {
                panic!("missing transcript must not yield records")
            })
            .unwrap()
        );
        assert!(skipped.is_empty());

        fs::write(&path, "").unwrap();
        assert!(read_transcript(root.path(), &path, &mut skipped, |_| {}).unwrap());
        assert!(skipped.is_empty());
    }
}
