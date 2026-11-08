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
    pub step: Option<String>,
    pub form: InvocationForm,
    pub resume_id: Option<String>,
    pub dir_moved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub step: Option<String>,
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
/// interpreter. Only the explicitly supported wrappers are removed; nothing expands.
pub fn recognise(command: &str) -> Option<Launch> {
    let mut dir_moved = false;
    for segment in segments(command) {
        let offset = wrapper_offset(&segment.tokens);
        let tokens = &segment.tokens[offset..];
        let words: Vec<_> = tokens.iter().map(|word| word.text.as_str()).collect();
        if help_only(tokens) {
            continue;
        }
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
        let changes_directory = words.first() == Some(&"codex") && changes_directory(tokens);
        return Some(Launch {
            step: multi_link_plan::step_tag(&command[segment.span]),
            form,
            resume_id,
            dir_moved: dir_moved || changes_directory,
        });
    }
    None
}

/// Help is an option only before the prompt/id, outside quoting and escaping.
fn help_only(words: &[Word]) -> bool {
    let start = match words {
        [codex, exec, resume, ..]
            if codex.text == "codex" && exec.text == "exec" && resume.text == "resume" =>
        {
            3
        }
        [codex, subcommand, ..]
            if codex.text == "codex" && matches!(subcommand.text.as_str(), "exec" | "resume") =>
        {
            2
        }
        _ => return false,
    };
    let mut options = words[start..].iter();
    while let Some(word) = options.next() {
        let text = word.text.as_str();
        if text == "--" || !text.starts_with('-') || text == "-" {
            break;
        }
        if !word.quoted_or_escaped && matches!(text, "--help" | "-h") {
            return true;
        }
        if matches!(
            text,
            "-c" | "-o"
                | "-C"
                | "-s"
                | "--add-dir"
                | "--config"
                | "--output-last-message"
                | "--cd"
                | "--sandbox"
        ) {
            options.next();
        }
    }
    false
}

/// Skip only the narrow wrapper grammar, including stacked wrappers.
fn wrapper_offset(words: &[Word]) -> usize {
    let mut offset = 0;
    loop {
        match words.get(offset).map(|word| word.text.as_str()) {
            Some("nohup") => offset += 1,
            Some("env") => {
                offset += 1;
                while words.get(offset).is_some_and(|word| assignment(&word.text)) {
                    offset += 1;
                }
            }
            Some("timeout" | "gtimeout")
                if words
                    .get(offset + 1)
                    .is_some_and(|word| duration(&word.text)) =>
            {
                offset += 2;
            }
            _ => return offset,
        }
    }
}

fn assignment(text: &str) -> bool {
    let Some((name, _)) = text.split_once('=') else {
        return false;
    };
    !name.is_empty()
        && name
            .bytes()
            .enumerate()
            .all(|(i, b)| b == b'_' || b.is_ascii_alphabetic() || (i > 0 && b.is_ascii_digit()))
}

fn duration(text: &str) -> bool {
    let number = text.strip_suffix(['s', 'm', 'h', 'd']).unwrap_or(text);
    !number.is_empty()
        && number
            .parse::<f64>()
            .is_ok_and(|n| n.is_finite() && n >= 0.0)
        && number.bytes().all(|b| b.is_ascii_digit() || b == b'.')
}

fn changes_directory(words: &[Word]) -> bool {
    let mut words = words.iter();
    while let Some(word) = words.next() {
        let text = word.text.as_str();
        // A literal -- ends options; later tokens are prompt/positional text.
        if text == "--" {
            break;
        }
        let (value, expandable_dollar) = if matches!(text, "-C" | "--cd") {
            let Some(value) = words.next() else {
                return true;
            };
            (value.text.as_str(), value.expandable_dollar)
        } else if let Some(value) = text
            .strip_prefix("-C")
            .or_else(|| text.strip_prefix("--cd="))
        {
            (value, word.expandable_dollar)
        } else {
            continue;
        };
        if value != "." && !(expandable_dollar && matches!(value, "$PWD" | "${PWD}")) {
            return true;
        }
    }
    false
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
struct Segment {
    tokens: Vec<Word>,
    span: std::ops::Range<usize>,
}

#[derive(Default)]
struct Word {
    text: String,
    // Quoting/escaping is removed from text, but literal $PWD must stay distinct.
    expandable_dollar: bool,
    quoted_or_escaped: bool,
}

fn segments(command: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut words = Vec::new();
    let mut word = Word::default();
    let mut started = false;
    let mut quote = None;
    let mut chars = command.char_indices().peekable();
    let mut segment_start = 0;
    let mut heredocs = Vec::new();
    while let Some((index, ch)) = chars.next() {
        // Keep substitutions opaque even inside double quotes. Their quotes and
        // separators belong to the nested shell, not this command's segments.
        if ch == '$' && quote != Some('\'') && chars.peek().is_some_and(|(_, c)| *c == '(') {
            let end = substitution_end(&mut chars);
            word.text.push_str(&command[index..end]);
            word.expandable_dollar = true;
            started = true;
            continue;
        }
        if quote.is_none() && ch == '<' && chars.peek().is_some_and(|(_, c)| *c == '<') {
            chars.next();
            // Here-strings are not heredocs.
            if chars.peek().is_some_and(|(_, c)| *c == '<') {
                chars.next();
                continue;
            }
            let tabs = chars.peek().is_some_and(|(_, c)| *c == '-');
            if tabs {
                chars.next();
            }
            while chars.peek().is_some_and(|(_, c)| matches!(c, ' ' | '\t')) {
                chars.next();
            }
            let mut delimiter = String::new();
            let mut delimiter_quote = None;
            while let Some(&(_, c)) = chars.peek() {
                if delimiter_quote.is_none()
                    && (c.is_whitespace() || matches!(c, ';' | '&' | '|' | '(' | ')' | '<' | '>'))
                {
                    break;
                }
                chars.next();
                if delimiter_quote == Some(c) {
                    delimiter_quote = None;
                } else if delimiter_quote.is_none() && matches!(c, '\'' | '"') {
                    delimiter_quote = Some(c);
                } else if c == '\\' && delimiter_quote != Some('\'') {
                    if let Some((_, escaped)) = chars.next() {
                        delimiter.push(escaped);
                    }
                } else {
                    delimiter.push(c);
                }
            }
            heredocs.push((delimiter, tabs));
            continue;
        }
        if ch == '\\'
            && quote != Some('\'')
            && let Some(&(_, next)) = chars.peek()
            && (quote.is_none() || matches!(next, '$' | '`' | '"' | '\\' | '\n'))
        {
            chars.next();
            word.quoted_or_escaped = true;
            if next != '\n' {
                word.text.push(next);
                started = true;
            }
            continue;
        }
        if let Some(delimiter) = quote {
            if ch == delimiter {
                quote = None;
            } else {
                word.text.push(ch);
                word.expandable_dollar |= ch == '$' && delimiter == '"';
            }
        } else if matches!(ch, '\'' | '"') {
            quote = Some(ch);
            word.quoted_or_escaped = true;
            started = true;
        } else if ch.is_whitespace() || matches!(ch, ';' | '|' | '&' | '(' | ')') {
            if started {
                if word.quoted_or_escaped || !matches!(word.text.as_str(), "{" | "}") {
                    words.push(std::mem::take(&mut word));
                } else {
                    word = Word::default();
                }
                started = false;
            }
            // A single & is not a segment separator in the supported grammar.
            if ch == '&' && chars.peek().map(|(_, ch)| *ch) != Some('&') {
                word.text.push(ch);
                started = true;
            } else if matches!(ch, ';' | '|' | '&' | '(' | ')' | '\n') {
                if matches!(ch, '|' | '&') && chars.peek().map(|(_, next)| *next) == Some(ch) {
                    chars.next();
                }
                let segment_end = chars.peek().map_or(command.len(), |(index, _)| *index);
                segments.push(Segment {
                    tokens: std::mem::take(&mut words),
                    span: segment_start..index,
                });
                segment_start = segment_end;
                if ch == '\n' {
                    for (delimiter, tabs) in heredocs.drain(..) {
                        while let Some((start, _)) = chars.peek().copied() {
                            let mut end = command.len();
                            for (i, c) in chars.by_ref() {
                                if c == '\n' {
                                    end = i;
                                    break;
                                }
                            }
                            let line = &command[start..end];
                            if (if tabs {
                                line.trim_start_matches('\t')
                            } else {
                                line
                            }) == delimiter
                            {
                                break;
                            }
                        }
                    }
                    segment_start = chars.peek().map_or(command.len(), |(i, _)| *i);
                }
            }
        } else {
            word.text.push(ch);
            word.expandable_dollar |= ch == '$';
            started = true;
        }
    }
    if quote.is_none() {
        if started && (word.quoted_or_escaped || !matches!(word.text.as_str(), "{" | "}")) {
            words.push(word);
        }
        segments.push(Segment {
            tokens: words,
            span: segment_start..command.len(),
        });
    }
    segments
}

// The opening '(' is still in the iterator. Return the exclusive byte end.
fn substitution_end(chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>) -> usize {
    chars.next();
    let mut depth = 1;
    let mut quote = None;
    let mut end = 0;
    while let Some((index, ch)) = chars.next() {
        end = index + ch.len_utf8();
        if ch == '\\' && quote != Some('\'') {
            if let Some((i, c)) = chars.next() {
                end = i + c.len_utf8();
            }
        } else if quote == Some(ch) {
            quote = None;
        } else if quote.is_none() && matches!(ch, '\'' | '"') {
            quote = Some(ch);
        } else if quote.is_none() && ch == '(' {
            depth += 1;
        } else if quote.is_none() && ch == ')' {
            depth -= 1;
            if depth == 0 {
                break;
            }
        }
    }
    end
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
        step: launch.step,
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
