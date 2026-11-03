//! Read-only discovery and parsing of Codex rollout transcripts.

use std::{
    fs,
    io::{self, BufRead},
    path::{Path, PathBuf},
};

use multi_link_model::{SkipReason, Skipped, Timestamp, jsonl::JsonlReader};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Turn {
    pub turn_id: String,
    /// The task_started record's top-level timestamp.
    pub started_at: Timestamp,
    /// The matching task_complete record's top-level timestamp.
    pub completed_at: Option<Timestamp>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rollout {
    pub id: String,
    pub transcript: String,
    pub cwd: String,
    pub branch: Option<String>,
    pub started_at: Timestamp,
    pub last_activity: Timestamp,
    /// In task_started file order, regardless of timestamp ordering.
    pub turns: Vec<Turn>,
    pub activity_timestamps: Vec<Timestamp>,
    pub asked: Option<String>,
    pub imported: bool,
    pub last_real_complete: Option<Timestamp>,
}

impl Rollout {
    /// None means no usable task_started; only the latest turn governs completion.
    pub fn last_turn_complete(&self) -> Option<bool> {
        self.turns.last().map(|turn| turn.completed_at.is_some())
    }
}

#[derive(Debug, Default)]
pub struct Discovery {
    /// Sorted by root-relative transcript path.
    pub rollouts: Vec<Rollout>,
    pub skipped: Vec<Skipped>,
    /// File open/read failures, separate from the schema's line-level skips.
    /// A partially read file contributes skips but never a partial rollout.
    pub unreadable: Vec<ReadFailure>,
}

#[derive(Debug)]
pub struct ReadFailure {
    pub file: String,
    pub error: io::Error,
}

/// Find rollout-*.jsonl recursively under root/.codex/sessions.
/// Missing sessions directories are empty. Directory traversal errors are returned;
/// individual file failures are collected in `unreadable` and scanning continues.
/// Symlinks are not followed, avoiding cycles and reads outside the supplied tree.
pub fn discover(root: &Path) -> io::Result<Discovery> {
    let mut paths = Vec::new();
    collect_paths(&root.join(".codex/sessions"), &mut paths)?;
    paths.sort();
    let mut result = Discovery::default();
    for path in paths {
        let transcript = relative_path(root, &path);
        let parsed = JsonlReader::from_path(&path, &transcript)
            .and_then(|reader| parse(reader, &transcript, &mut result.skipped));
        match parsed {
            Ok(Some(rollout)) => result.rollouts.push(rollout),
            Ok(None) => {}
            Err(error) => result.unreadable.push(ReadFailure {
                file: transcript,
                error,
            }),
        }
    }
    Ok(result)
}

fn collect_paths(directory: &Path, paths: &mut Vec<PathBuf>) -> io::Result<()> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    for entry in entries {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() {
            collect_paths(&entry.path(), paths)?;
        } else if kind.is_file()
            && entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.starts_with("rollout-") && name.ends_with(".jsonl"))
        {
            paths.push(entry.path());
        }
    }
    Ok(())
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|part| part.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn timestamp(value: &Value) -> Option<Timestamp> {
    value.as_str().and_then(|text| Timestamp::parse(text).ok())
}

fn parse<R: BufRead>(
    reader: JsonlReader<R>,
    transcript: &str,
    skipped: &mut Vec<Skipped>,
) -> io::Result<Option<Rollout>> {
    let mut metadata = None;
    let mut first_timestamp = None;
    let mut last_activity = None;
    let mut turns = Vec::new();
    let mut activity_timestamps = Vec::new();
    let mut asked = None;
    let mut response_prompt = None;
    let mut imported_turn: Option<String> = None;
    let mut starts = 0;
    let mut real_starts = 0;
    let mut last_real_complete = None;
    for record in reader {
        let (line, value) = match record? {
            Ok(record) => record,
            Err(skip) => {
                skipped.push(skip);
                continue;
            }
        };
        let at = timestamp(&value["timestamp"]);
        if let Some(at) = at {
            first_timestamp.get_or_insert(at);
            last_activity = Some(at);
        }
        let payload = &value["payload"];
        let event = value["type"] == "event_msg";
        if event && payload["type"] == "task_started" {
            starts += 1;
            imported_turn = payload["turn_id"]
                .as_str()
                .filter(|id| id.starts_with("external-import"))
                .map(str::to_owned);
            if imported_turn.is_none() {
                real_starts += 1;
            }
        }
        if imported_turn.is_none() {
            if value["type"] != "session_meta" {
                activity_timestamps.extend(at);
            }
            if event && payload["type"] == "user_message" && asked.is_none() {
                asked = payload["message"]
                    .as_str()
                    .map(multi_link_model::normalize_prompt);
            }
            if response_prompt.is_none()
                && value["type"] == "response_item"
                && payload["type"] == "message"
                && payload["role"] == "user"
            {
                response_prompt = payload["content"]
                    .as_array()
                    .and_then(|blocks| {
                        blocks
                            .iter()
                            .find(|block| block["type"] == "input_text")
                            .and_then(|block| block["text"].as_str())
                    })
                    .map(str::trim_start)
                    .filter(|text| {
                        !text.is_empty()
                            && !text.starts_with("# AGENTS.md instructions")
                            && !text.starts_with('<')
                    })
                    .map(multi_link_model::normalize_prompt);
            }
            if event && payload["type"] == "task_complete" {
                last_real_complete = at;
            }
        }
        if event
            && payload["type"] == "task_complete"
            && imported_turn
                .as_deref()
                .is_some_and(|id| payload["turn_id"] == id)
        {
            imported_turn = None;
        }
        match value["type"].as_str() {
            Some("session_meta") => {
                // Replayed parent metadata must never replace the file's first metadata.
                if metadata.is_none() {
                    metadata = Some(value["payload"].clone());
                }
            }
            Some("event_msg") => parse_turn(&value["payload"], at, &mut turns),
            Some("response_item" | "token_usage_record" | "turn_context" | "world_state") => {}
            _ => skipped.push(Skipped {
                file: transcript.to_owned(),
                line,
                reason: SkipReason::UnknownType,
            }),
        }
    }
    let Some(metadata) = metadata else {
        return Ok(None);
    };
    let (Some(id), Some(cwd), Some(started_at), Some(last_activity)) = (
        metadata["id"].as_str(),
        metadata["cwd"].as_str(),
        timestamp(&metadata["timestamp"]).or(first_timestamp),
        last_activity,
    ) else {
        return Ok(None);
    };
    Ok(Some(Rollout {
        id: id.to_owned(),
        transcript: transcript.to_owned(),
        cwd: cwd.to_owned(),
        branch: metadata["git"]["branch"].as_str().map(str::to_owned),
        started_at,
        last_activity,
        turns,
        activity_timestamps: if starts > 0 && real_starts == 0 {
            Vec::new()
        } else {
            activity_timestamps
        },
        asked: if starts > 0 && real_starts == 0 {
            None
        } else {
            asked.or(response_prompt)
        },
        imported: starts > 0 && real_starts == 0,
        last_real_complete: if starts > 0 && real_starts == 0 {
            None
        } else {
            last_real_complete
        },
    }))
}

fn parse_turn(payload: &Value, at: Option<Timestamp>, turns: &mut Vec<Turn>) {
    let (Some(turn_id), Some(at)) = (payload["turn_id"].as_str(), at) else {
        return;
    };
    match payload["type"].as_str() {
        Some("task_started") => turns.push(Turn {
            turn_id: turn_id.to_owned(),
            started_at: at,
            completed_at: None,
        }),
        Some("task_complete") => {
            if let Some(turn) = turns
                .iter_mut()
                .rev()
                .find(|turn| turn.turn_id == turn_id && turn.completed_at.is_none())
            {
                turn.completed_at = Some(at);
            }
        }
        _ => {}
    }
}

/// Parse a single rollout without traversing its directory.
pub fn parse_file(
    root: &Path,
    path: &Path,
    skipped: &mut Vec<Skipped>,
) -> io::Result<Option<Rollout>> {
    let transcript = relative_path(root, path);
    parse(
        JsonlReader::from_path(path, &transcript)?,
        &transcript,
        skipped,
    )
}
