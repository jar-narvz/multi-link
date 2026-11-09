//! Read-only discovery and parsing of Codex rollout transcripts.

use std::{
    fs,
    io::{self, BufRead, BufReader, Cursor, Read},
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
    pub last_asked: Option<String>,
    pub imported: bool,
    pub last_real_complete: Option<Timestamp>,
}

fn is_imported_turn(id: &str) -> bool {
    id.starts_with("external-import")
}

impl Rollout {
    /// Real turns in file order, using the same import rule as graph details.
    /// The first item is the initial work, never a resume candidate.
    pub fn real_turns(&self) -> impl Iterator<Item = &Turn> {
        self.turns
            .iter()
            .filter(|turn| !is_imported_turn(&turn.turn_id))
    }

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

/// Find plain and compressed rollouts. Missing directories are empty; symlinks
/// are not followed. A plain counterpart takes precedence before either is read.
pub fn rollout_paths(directory: &Path) -> io::Result<Vec<PathBuf>> {
    fn collect(directory: &Path, paths: &mut Vec<PathBuf>) -> io::Result<()> {
        let entries = match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        };
        for entry in entries {
            let entry = entry?;
            let kind = match entry.file_type() {
                Ok(kind) => kind,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error),
            };
            if kind.is_dir() {
                collect(&entry.path(), paths)?;
            } else if kind.is_file()
                && entry.file_name().to_str().is_some_and(|name| {
                    name.starts_with("rollout-")
                        && (name.ends_with(".jsonl") || name.ends_with(".jsonl.zst"))
                })
            {
                paths.push(entry.path());
            }
        }
        Ok(())
    }
    let mut paths = Vec::new();
    collect(directory, &mut paths)?;
    let plain: std::collections::HashSet<_> = paths
        .iter()
        .filter(|path| path.extension().is_some_and(|ext| ext == "jsonl"))
        .cloned()
        .collect();
    paths.retain(|path| !is_compressed(path) || !plain.contains(&path.with_extension("")));
    paths.sort();
    Ok(paths)
}

fn is_compressed(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "zst")
}

impl Discovery {
    /// Metadata ids also deduplicate copies with different filenames or folders.
    pub fn prefer_plain(&mut self) {
        let plain: std::collections::HashSet<_> = self
            .rollouts
            .iter()
            .filter(|r| !is_compressed(Path::new(&r.transcript)))
            .map(|r| r.id.clone())
            .collect();
        self.rollouts
            .retain(|r| !is_compressed(Path::new(&r.transcript)) || !plain.contains(&r.id));
    }
}

/// Open JSONL for every Codex reader. Only compressed input is bounded: decode
/// at most 64 MiB plus one detection byte, rejecting the entire file on failure.
/// Validate every frame (including checksums) before handing any records out.
pub fn open_rollout(path: &Path) -> io::Result<Box<dyn BufRead>> {
    const LIMIT: usize = 64 * 1024 * 1024;
    let mut source = BufReader::new(fs::File::open(path)?);
    if !is_compressed(path) {
        return Ok(Box::new(source));
    }
    let mut bytes = Vec::new();
    loop {
        let mut decoder = ruzstd::decoding::StreamingDecoder::new(&mut source)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let remaining = LIMIT + 1 - bytes.len();
        decoder
            .by_ref()
            .take(remaining as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > LIMIT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "rollout exceeds 64 MiB",
            ));
        }
        if let Some(expected) = decoder.decoder.get_checksum_from_data()
            && decoder.decoder.get_calculated_checksum() != Some(expected)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "rollout checksum mismatch",
            ));
        }
        if source.fill_buf()?.is_empty() {
            break;
        }
    }
    Ok(Box::new(Cursor::new(bytes)))
}

/// Discover rollouts, collecting individual read/decode failures without a
/// partial rollout. Directory traversal errors are returned to the caller.
pub fn discover(root: &Path) -> io::Result<Discovery> {
    let mut result = Discovery::default();
    for path in rollout_paths(&root.join(".codex/sessions"))? {
        match parse_file(root, &path, &mut result.skipped) {
            Ok(Some(rollout)) => result.rollouts.push(rollout),
            Ok(None) => {}
            Err(error) => result.unreadable.push(ReadFailure {
                file: relative_path(root, &path),
                error,
            }),
        }
    }
    result.prefer_plain();
    Ok(result)
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
    let mut last_asked = None;
    let mut last_response_prompt = None;
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
                .filter(|id| is_imported_turn(id))
                .map(str::to_owned);
            if imported_turn.is_none() {
                real_starts += 1;
            }
        }
        if imported_turn.is_none() {
            if value["type"] != "session_meta" {
                activity_timestamps.extend(at);
            }
            if event
                && payload["type"] == "user_message"
                && let Some(prompt) = payload["message"]
                    .as_str()
                    .map(multi_link_model::normalize_prompt)
            {
                asked.get_or_insert_with(|| prompt.clone());
                last_asked = Some(prompt);
            }
            if value["type"] == "response_item"
                && payload["type"] == "message"
                && payload["role"] == "user"
            {
                let prompt = response_user_prompt(payload);
                if let Some(prompt) = prompt {
                    response_prompt.get_or_insert_with(|| prompt.clone());
                    last_response_prompt = Some(prompt);
                }
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
        last_asked: if starts > 0 && real_starts == 0 {
            None
        } else {
            last_asked.or(last_response_prompt)
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
        JsonlReader::new(open_rollout(path)?, &transcript),
        &transcript,
        skipped,
    )
}

/// Read calls on demand. Undocumented payloads are assumptions, never required
/// for the scanner to work. Missing files remain I/O errors for graph availability.
pub fn calls(path: &Path) -> io::Result<multi_link_model::Calls> {
    Ok(read_details(path)?.0)
}

pub fn codex_turns(path: &Path) -> io::Result<Vec<multi_link_model::Turn>> {
    Ok(read_details(path)?.1)
}

fn response_user_prompt(payload: &Value) -> Option<String> {
    if payload["type"] != "message" || payload["role"] != "user" {
        return None;
    }
    payload["content"]
        .as_array()?
        .iter()
        .find(|block| block["type"] == "input_text")?["text"]
        .as_str()
        .map(str::trim_start)
        .filter(|text| {
            !text.is_empty()
                && !text.starts_with("# AGENTS.md instructions")
                && !text.starts_with('<')
        })
        .map(multi_link_model::normalize_prompt)
}

/// Combined form avoids opening a rollout twice when both are needed.
pub fn read_details(
    path: &Path,
) -> io::Result<(multi_link_model::Calls, Vec<multi_link_model::Turn>)> {
    use multi_link_model::{Call, Calls, Turn as GraphTurn};
    let file = path.to_string_lossy();
    let mut report = Calls::default();
    let mut turns: Vec<GraphTurn> = Vec::new();
    let mut active: Option<(String, usize)> = None;
    let mut imported: Option<String> = None;
    let mut event_prompts = std::collections::HashSet::new();
    let mut outputs = std::collections::HashMap::new();
    let mut seen = std::collections::HashSet::new();
    for record in JsonlReader::new(open_rollout(path)?, file.as_ref()) {
        let (line, value) = match record? {
            Ok(record) => record,
            Err(skip) => {
                report.skipped.push(skip);
                continue;
            }
        };
        let payload = &value["payload"];
        let at = timestamp(&value["timestamp"]);
        let event = value["type"] == "event_msg";
        let subtype = payload["type"].as_str().unwrap_or_default();
        let mut unknown = false;
        if event && subtype == "task_started" {
            active = None;
            imported = None;
            if let (Some(id), Some(at)) = (payload["turn_id"].as_str(), at) {
                if is_imported_turn(id) {
                    imported = Some(id.to_owned());
                } else {
                    active = Some((id.to_owned(), turns.len()));
                    turns.push(GraphTurn {
                        start: at,
                        completed_at: None,
                        last_record_at: at,
                        prompt: None,
                        last_agent_message: None,
                    });
                }
            } else {
                unknown = true;
            }
        }
        if let Some(id) = imported.as_deref() {
            if event && subtype == "task_complete" && payload["turn_id"] == id {
                imported = None;
            }
            continue;
        }
        if let Some((id, index)) = &active {
            let turn = &mut turns[*index];
            if let Some(at) = at {
                turn.last_record_at = turn.last_record_at.max(at);
            }
            if event && subtype == "user_message" {
                if let Some(prompt) = payload["message"].as_str()
                    && event_prompts.insert(*index)
                {
                    turn.prompt = Some(multi_link_model::normalize_prompt(prompt));
                }
            } else if value["type"] == "response_item" && turn.prompt.is_none() {
                turn.prompt = response_user_prompt(payload);
            }
            if event && subtype == "task_complete" && payload["turn_id"] == id.as_str() {
                turn.completed_at = at;
                turn.last_agent_message = payload["last_agent_message"]
                    .as_str()
                    .map(multi_link_model::normalize_prompt);
                active = None;
            }
        }
        match value["type"].as_str() {
            Some("response_item") => match subtype {
                "custom_tool_call" | "function_call" => {
                    if let (Some(id), Some(at)) = (payload["call_id"].as_str(), at) {
                        if seen.insert(id.to_owned()) {
                            let (kind, label) = classify_call(payload);
                            report.calls.push(Call {
                                id: id.to_owned(),
                                at,
                                kind,
                                label,
                                ok: None,
                            });
                        }
                    } else {
                        unknown = true;
                    }
                }
                "custom_tool_call_output" | "function_call_output" => {
                    if let Some(id) = payload["call_id"].as_str() {
                        outputs
                            .entry(id.to_owned())
                            .or_insert_with(|| output_exit(&payload["output"]));
                    } else {
                        unknown = true;
                    }
                }
                "message" | "reasoning" | "web_search_call" | "compaction" | "additional_tools" => {
                }
                _ => unknown = true,
            },
            Some(
                "event_msg" | "session_meta" | "turn_context" | "token_usage_record"
                | "world_state",
            ) => {}
            _ => unknown = true,
        }
        if unknown {
            report.skipped.push(Skipped {
                file: file.to_string(),
                line,
                reason: SkipReason::UnknownType,
            });
        }
    }
    for call in &mut report.calls {
        call.ok = outputs.get(&call.id).copied().flatten();
    }
    Ok((report, turns))
}

fn classify_call(payload: &Value) -> (multi_link_model::CallKind, Option<String>) {
    use multi_link_model::{CallKind, classify_bash, quoted_string};
    let name = payload["name"].as_str().unwrap_or_default();
    let input = payload["input"]
        .as_str()
        .or_else(|| payload["arguments"].as_str())
        .unwrap_or_default();
    if name == "apply_patch" {
        return (CallKind::Edit, patch_label(input));
    }
    if name == "exec_command" {
        return serde_json::from_str::<Value>(input)
            .ok()
            .and_then(|v| v["cmd"].as_str().map(classify_bash))
            .unwrap_or((CallKind::Run, None));
    }
    if name == "write_stdin" {
        return (CallKind::Run, None);
    }
    // Look only outside string literals. A command mentioning tools.apply_patch
    // must not become an edit; the first nested tool defines this record's call.
    let mut rest = input;
    while !rest.is_empty() {
        let c = rest.chars().next().unwrap();
        if matches!(c, '"' | '\'' | '`') {
            let mut escaped = false;
            let end = rest.char_indices().skip(1).find_map(|(i, ch)| {
                if !escaped && ch == c {
                    Some(i + ch.len_utf8())
                } else {
                    escaped = !escaped && ch == '\\';
                    None
                }
            });
            rest = end.map(|i| &rest[i..]).unwrap_or_default();
            continue;
        }
        if let Some(tail) = rest.strip_prefix("tools.apply_patch(") {
            return (
                CallKind::Edit,
                quoted_string(tail).and_then(|patch| patch_label(&patch)),
            );
        }
        if rest.starts_with("tools.write_stdin(") {
            return (CallKind::Run, None);
        }
        if let Some(tail) = rest.strip_prefix("tools.exec_command(") {
            let cmd = command_input(tail);
            return cmd
                .as_deref()
                .map(classify_bash)
                .unwrap_or((CallKind::Run, None));
        }
        rest = &rest[c.len_utf8()..];
    }
    (CallKind::Run, None)
}

/// Inspect property names outside string values, without evaluating JavaScript.
fn command_input(mut text: &str) -> Option<String> {
    while !text.is_empty() {
        text = text.trim_start();
        let c = text.chars().next()?;
        if matches!(c, ')' | '}') {
            return None;
        }
        if matches!(c, '\'' | '"') {
            let mut escaped = false;
            let end = text.char_indices().skip(1).find_map(|(i, ch)| {
                if !escaped && ch == c {
                    Some(i + ch.len_utf8())
                } else {
                    escaped = !escaped && ch == '\\';
                    None
                }
            })?;
            let key = multi_link_model::quoted_string(&text[..end]);
            text = &text[end..];
            if key.as_deref() == Some("cmd")
                && let Some(value) = text.trim_start().strip_prefix(':')
            {
                return multi_link_model::quoted_string(value);
            }
        } else if c.is_ascii_alphabetic() || c == '_' {
            let end = text
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(text.len());
            let key = &text[..end];
            text = &text[end..];
            if key == "cmd"
                && let Some(value) = text.trim_start().strip_prefix(':')
            {
                return multi_link_model::quoted_string(value);
            }
        } else {
            text = &text[c.len_utf8()..];
        }
    }
    None
}

fn patch_label(patch: &str) -> Option<String> {
    patch
        .lines()
        .find_map(|line| {
            line.strip_prefix("*** Update File:")
                .or_else(|| line.strip_prefix("*** Add File:"))
        })
        .and_then(multi_link_model::edit_label)
}

/// First exit_code, in output block order. Do not infer success from prose.
fn output_exit(value: &Value) -> Option<bool> {
    match value {
        Value::Array(blocks) => blocks.iter().find_map(output_exit),
        Value::Object(object) => {
            if let Some(code) = object.get("exit_code").and_then(Value::as_i64) {
                return Some(code == 0);
            }
            object.get("text").and_then(output_exit)
        }
        Value::String(text) => {
            if let Ok(parsed) = serde_json::from_str::<Value>(text) {
                return output_exit(&parsed);
            }
            let (_, tail) = text.split_once("\"exit_code\"")?;
            let tail = tail.trim_start().strip_prefix(':')?.trim_start();
            let number: String = tail
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '-')
                .collect();
            number.parse::<i64>().ok().map(|code| code == 0)
        }
        _ => None,
    }
}
