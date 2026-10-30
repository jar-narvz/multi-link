//! Recognition only: result text is preserved for the linker, never interpreted.

use std::{
    collections::{HashMap, HashSet},
    io,
    path::Path,
};

use multi_link_model::{InvocationForm, Timestamp, jsonl::JsonlReader};
use serde_json::Value;

use crate::ClaudeScan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Launch {
    pub form: InvocationForm,
    pub resume_id: Option<String>,
    pub dir_moved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub parent: String,
    pub invocation: String,
    pub subagent: Option<String>,
    pub form: InvocationForm,
    pub background: bool,
    pub at: Timestamp,
    pub checkout: String,
    pub command: String,
    pub resume_id: Option<String>,
    pub dir_moved: bool,
    pub result_text: Option<String>,
}

/// Recognise the first launch segment using the Phase 1 grammar, not a shell
/// interpreter. No expansion, environment prefixes, or wrapper commands.
pub fn recognise(command: &str) -> Option<Launch> {
    let mut dir_moved = false;
    for tokens in segments(command) {
        let words: Vec<_> = tokens.iter().map(String::as_str).collect();
        let (form, resume_id) = match words.as_slice() {
            ["codex", "exec", "resume", id, ..] | ["codex", "resume", id, ..]
                if uuid_shaped(id) =>
            {
                (InvocationForm::Resume, Some((*id).to_owned()))
            }
            ["codex", "exec", ..] => (InvocationForm::Exec, None),
            ["node", path, "task", ..] if path.ends_with("codex-companion.mjs") => {
                (InvocationForm::Plugin, None)
            }
            _ => {
                dir_moved |= words.first() == Some(&"cd");
                continue;
            }
        };
        // A literal -- ends options; later tokens are prompt/positional text.
        let changes_directory = words.first() == Some(&"codex")
            && words.iter().take_while(|word| **word != "--").any(|word| {
                *word == "-C"
                    || word.starts_with("-C")
                    || *word == "--cd"
                    || word.starts_with("--cd=")
            });
        return Some(Launch {
            form,
            resume_id,
            dir_moved: dir_moved || changes_directory,
        });
    }
    None
}

fn uuid_shaped(text: &str) -> bool {
    text.len() == 36
        && text.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
}

/// Tokenise words and segments together so quoted separators stay inside words.
/// Backslash escapes outside single quotes; double quotes preserve backslashes
/// except before shell-escapable characters. Unclosed quotes fail soft.
fn segments(command: &str) -> Vec<Vec<String>> {
    let mut segments = Vec::new();
    let mut words = Vec::new();
    let mut word = String::new();
    let mut started = false;
    let mut quote = None;
    let mut chars = command.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\'
            && quote != Some('\'')
            && let Some(&next) = chars.peek()
            && (quote.is_none() || matches!(next, '$' | '`' | '"' | '\\' | '\n'))
        {
            chars.next();
            if next != '\n' {
                word.push(next);
                started = true;
            }
            continue;
        }
        if let Some(delimiter) = quote {
            if ch == delimiter {
                quote = None;
            } else {
                word.push(ch);
            }
        } else if matches!(ch, '\'' | '"') {
            quote = Some(ch);
            started = true;
        } else if ch.is_whitespace() || matches!(ch, ';' | '|' | '&') {
            if started {
                words.push(std::mem::take(&mut word));
                started = false;
            }
            // A single & is not a segment separator in the supported grammar.
            if ch == '&' && chars.peek() != Some(&'&') {
                word.push(ch);
                started = true;
            } else if matches!(ch, ';' | '|' | '&' | '\n') {
                if matches!(ch, '|' | '&') && chars.peek() == Some(&ch) {
                    chars.next();
                }
                segments.push(std::mem::take(&mut words));
            }
        } else {
            word.push(ch);
            started = true;
        }
    }
    if quote.is_none() {
        if started {
            words.push(word);
        }
        segments.push(words);
    }
    segments
}

/// Re-read discovered main and subagent files. Missing files are omitted; other
/// I/O errors propagate. Malformed/unknown records were counted by `scan` and
/// are ignored here. Calls lacking required metadata are omitted. Output order
/// is timestamp, parent, invocation, independent of discovery order.
pub fn invocations(root: &Path, scan: &ClaudeScan) -> io::Result<Vec<Invocation>> {
    let mut calls = Vec::new();
    for transcript in &scan.transcripts {
        read_calls(
            root,
            &transcript.transcript,
            &transcript.id,
            false,
            &mut calls,
        )?;
        for file in &transcript.subagent_files {
            read_calls(root, &file.transcript, &transcript.id, true, &mut calls)?;
        }
    }
    calls.sort_by(|a, b| (&a.at, &a.parent, &a.invocation).cmp(&(&b.at, &b.parent, &b.invocation)));
    Ok(calls)
}

fn read_calls(
    root: &Path,
    file: &str,
    main_id: &str,
    subagent: bool,
    calls: &mut Vec<Invocation>,
) -> io::Result<()> {
    let reader = match JsonlReader::from_path(root.join(file), file) {
        Ok(reader) => reader,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    let start = calls.len();
    let mut seen = HashSet::new();
    let mut results: HashMap<String, Vec<String>> = HashMap::new();
    for record in reader {
        let Ok((_, value)) = record? else { continue };
        let kind = value.get("type").and_then(Value::as_str);
        let Some(blocks) = value.pointer("/message/content").and_then(Value::as_array) else {
            continue;
        };
        let single_result = blocks
            .iter()
            .filter(|block| block["type"] == "tool_result")
            .count()
            == 1;
        for block in blocks {
            if kind == Some("assistant") && block["type"] == "tool_use" {
                let Some(id) = block.get("id").and_then(Value::as_str) else {
                    continue;
                };
                if !seen.insert(id.to_owned()) {
                    continue;
                }
                if block["name"] == "Bash"
                    && let Some(call) = parse_call(&value, block, main_id, subagent)
                {
                    calls.push(call);
                }
            } else if kind == Some("user") && block["type"] == "tool_result" {
                let Some(id) = block.get("tool_use_id").and_then(Value::as_str) else {
                    continue;
                };
                let text = results.entry(id.to_owned()).or_default();
                collect_text(&block["content"], text);
                if let Some(stdout) = value
                    .pointer("/toolUseResult/stdout")
                    .and_then(Value::as_str)
                    .filter(|_| single_result)
                {
                    text.push(stdout.to_owned());
                }
            }
        }
    }
    for call in &mut calls[start..] {
        call.result_text = results.remove(&call.invocation).map(|text| text.join("\n"));
    }
    Ok(())
}

fn parse_call(value: &Value, block: &Value, main_id: &str, subagent: bool) -> Option<Invocation> {
    let command = block.pointer("/input/command")?.as_str()?;
    let launch = recognise(command)?;
    Some(Invocation {
        parent: if subagent {
            value.get("sessionId")?.as_str()?
        } else {
            main_id
        }
        .to_owned(),
        invocation: block.get("id")?.as_str()?.to_owned(),
        subagent: if subagent {
            value
                .get("agentId")
                .and_then(Value::as_str)
                .map(str::to_owned)
        } else {
            None
        },
        form: launch.form,
        background: block
            .pointer("/input/run_in_background")
            .and_then(Value::as_bool)
            == Some(true),
        at: Timestamp::parse(value.get("timestamp")?.as_str()?).ok()?,
        checkout: value.get("cwd")?.as_str()?.to_owned(),
        command: command.to_owned(),
        resume_id: launch.resume_id,
        dir_moved: launch.dir_moved,
        result_text: None,
    })
}

fn collect_text(content: &Value, output: &mut Vec<String>) {
    if let Some(text) = content.as_str() {
        output.push(text.to_owned());
    } else if let Some(blocks) = content.as_array() {
        for block in blocks {
            if block["type"] == "text"
                && let Some(text) = block.get("text").and_then(Value::as_str)
            {
                output.push(text.to_owned());
            }
        }
    }
}
