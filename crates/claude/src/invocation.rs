//! Narrow launch recognition and safe redirect evidence reads.
//! Transcript result text is preserved for the linker.

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
    pub launch_dir: Option<String>,
    pub stdout_redirect: Option<String>,
    pub stdout_append: bool,
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
    pub launch_dir: Option<String>,
    pub result_text: Option<String>,
}

/// Recognise the first launch segment using the Phase 1 grammar, not a shell
/// interpreter. Only the explicitly supported wrappers are removed; nothing expands.
pub fn recognise(command: &str) -> Option<Launch> {
    recognise_in(command, ".")
}

// The transcript cwd is the starting folder; the public recogniser needs only
// directory certainty and redirect metadata, so it uses a relative base.
fn recognise_in(command: &str, checkout: &str) -> Option<Launch> {
    let mut folder = Some(checkout.to_owned());
    let mut cd_depth = None;
    let mut redirect_dir_moved = false;
    for segment in segments(command) {
        let offset = wrapper_offset(&segment.tokens);
        let tokens = &segment.tokens[offset..];
        let segment_moves = matches!(
            tokens.first().map(|word| word.text.as_str()),
            Some("cd" | "pushd")
        ) || segment
            .tokens
            .iter()
            .any(|word| !word.quoted_or_escaped && matches!(word.text.as_str(), "cd" | "pushd"));
        // A single & is tracked for C1 boundaries, but does not introduce a
        // supported launch segment in the existing recognition grammar.
        let words: Vec<_> = if segment.before == "&" {
            Vec::new()
        } else {
            tokens.iter().map(|word| word.text.as_str()).collect()
        };
        if help_only(tokens) {
            redirect_dir_moved |= segment_moves;
            continue;
        }
        // R7's id-less --last form belongs only to exec resume. Interactive
        // resume still needs an explicit UUID, including after C3 options.
        let selected_resume = resume_options(tokens)
            .and_then(resume_selection)
            .filter(|id| id.is_some() || words.get(1) == Some(&"exec"));
        let (form, resume_id) = match words.as_slice() {
            ["codex", "exec", "resume", id, ..] | ["codex", "resume", id, ..]
                if uuid_shaped(id) =>
            {
                (InvocationForm::Resume, Some((*id).to_owned()))
            }
            ["codex", "exec", ..] | ["codex", "resume", ..] if selected_resume.is_some() => {
                (InvocationForm::Resume, selected_resume.flatten())
            }
            ["codex", "exec", ..] => (InvocationForm::Exec, None),
            ["node", path, "task", ..] if path.ends_with("codex-companion.mjs") => {
                (InvocationForm::Plugin, None)
            }
            _ => {
                if let Some(target) = literal_cd(&segment) {
                    folder = if target.starts_with('/') {
                        Some(normalise_folder(target))
                    } else {
                        folder.map(|base| normalise_folder(&format!("{base}/{target}")))
                    };
                    cd_depth = Some(segment.depth);
                } else if tokens.first().is_some_and(|word| {
                    matches!(word.text.as_str(), "cd" | "pushd" | "popd")
                        || (cd_depth.is_some()
                            && matches!(
                                word.text.as_str(),
                                "eval" | "source" | "." | "exec" | "builtin" | "command"
                            ))
                }) {
                    folder = None;
                }
                if segment.after == ")" && cd_depth.is_some_and(|depth| segment.depth <= depth) {
                    folder = None;
                }
                redirect_dir_moved |= segment_moves;
                continue;
            }
        };
        redirect_dir_moved |= segment.after_subshell;
        let changes_directory = words.first() == Some(&"codex") && changes_directory(tokens);
        return Some(Launch {
            step: multi_link_plan::step_tag(&command[segment.span]),
            form,
            resume_id,
            dir_moved: folder.is_none() || changes_directory,
            launch_dir: folder.filter(|folder| !changes_directory && folder != checkout),
            stdout_redirect: redirect_target(tokens, redirect_dir_moved),
            stdout_append: tokens
                .iter()
                .any(|word| word.redirect && matches!(word.text.as_str(), ">>" | "&>>")),
        });
    }
    None
}

/// C1 deliberately accepts a small literal path alphabet, without expansion.
fn literal_cd(segment: &Segment) -> Option<&str> {
    let [cd, target] = segment.tokens.as_slice() else {
        return None;
    };
    let path = target.text.as_str();
    (cd.text == "cd"
        && matches!(segment.before, "" | ";" | "\n" | "&&" | "(")
        && matches!(segment.after, ";" | "\n" | "&&")
        && !target.redirect
        && (path.starts_with('/')
            || path.starts_with("./")
            || path.starts_with("../")
            || matches!(path, "." | ".."))
        && path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._/+@%:,=- ".contains(&b)))
    .then_some(path)
}

/// Lexical only: never consult the filesystem or resolve symlinks.
fn normalise_folder(path: &str) -> String {
    let absolute = path.starts_with('/');
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." if parts.last().is_some_and(|part| *part != "..") => {
                parts.pop();
            }
            ".." if absolute => {}
            _ => parts.push(part),
        }
    }
    let joined = parts.join("/");
    if absolute {
        format!("/{joined}")
    } else if joined.is_empty() {
        ".".into()
    } else {
        joined
    }
}

/// Only literal, unambiguous stdout targets can supply evidence. Descriptor
/// duplication (2>&1) does not redirect stdout; all other stdout operators count.
fn redirect_target(tokens: &[Word], dir_moved: bool) -> Option<String> {
    let redirects: Vec<_> = tokens
        .iter()
        .enumerate()
        .filter(|(_, w)| w.redirect && !w.text.starts_with('2'))
        .collect();
    let [(index, operator)] = redirects.as_slice() else {
        return None;
    };
    if !matches!(
        operator.text.as_str(),
        ">" | ">>" | "1>" | "&>" | "&>>" | ">|"
    ) {
        return None;
    }
    let target = tokens.get(index + 1)?;
    if target.redirect
        || target.text.is_empty()
        || target
            .text
            .contains(['$', '`', '*', '?', '[', ']', '~', '<', '>'])
        || (dir_moved && !Path::new(&target.text).is_absolute())
    {
        return None;
    }
    Some(target.text.clone())
}

impl Launch {
    /// Read redirect evidence directly each time (never through a path cache).
    /// Fixture roots replace absolute paths and do not consult the environment.
    /// NotFound may be retried; other errors refuse the file without polling.
    pub fn redirect_child(
        &self,
        cwd: &Path,
        fixture_root: Option<&Path>,
    ) -> io::Result<Option<String>> {
        let Some(target) = self.stdout_redirect.as_deref() else {
            return Ok(None);
        };
        let target = Path::new(target);
        let (cwd, path, mut allowed) = if let Some(root) = fixture_root {
            let cwd = root.join(cwd.strip_prefix("/").unwrap_or(cwd));
            let path = if target.is_absolute() {
                root.join(target.strip_prefix("/").map_err(io::Error::other)?)
            } else {
                cwd.join(target)
            };
            (cwd, path, vec![root.join("tmp"), root.join("private/tmp")])
        } else {
            let path = if target.is_absolute() {
                target.to_owned()
            } else {
                cwd.join(target)
            };
            let mut allowed = vec![
                std::path::PathBuf::from("/tmp"),
                std::path::PathBuf::from("/private/tmp"),
            ];
            if let Some(temp) = std::env::var_os("TMPDIR") {
                let temp = std::path::PathBuf::from(temp);
                if temp.is_absolute() {
                    allowed.push(temp);
                }
            }
            (cwd.to_owned(), path, allowed)
        };
        allowed.push(cwd);
        let text = read_redirect(&path, &allowed, self.stdout_append)?;
        let mut lines = text
            .lines()
            .filter_map(|line| line.trim().strip_prefix("session id:"));
        let Some(id) = lines.next() else {
            return Ok(None);
        };
        let id = id.trim();
        if lines.next().is_some() || !uuid_shaped(id) {
            return Ok(None);
        }
        Ok(Some(id.to_ascii_lowercase()))
    }
}

fn read_redirect(path: &Path, allowed: &[std::path::PathBuf], append: bool) -> io::Result<String> {
    use std::{
        fs::OpenOptions,
        io::Read,
        os::unix::fs::{MetadataExt, OpenOptionsExt},
    };
    let allowed: Vec<_> = allowed
        .iter()
        .filter_map(|base| base.canonicalize().ok())
        .collect();
    // Refuse known outside paths before open: opening a device can itself have
    // side effects. This precheck does not replace descriptor validation below.
    let canonical = path.canonicalize()?;
    if !allowed.iter().any(|base| canonical.starts_with(base)) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "redirect outside allowed roots",
        ));
    }
    #[cfg(target_os = "macos")]
    const O_NONBLOCK: i32 = 4;
    #[cfg(not(target_os = "macos"))]
    const O_NONBLOCK: i32 = 0x800;
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(O_NONBLOCK)
        .open(path)?;
    // metadata() is fstat on this opened descriptor, not a path-based stat.
    // Append logs may hide another run beyond the cap; truncating redirects
    // hold one run, so their first 64 KiB can still supply evidence.
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.nlink() != 1 || (append && metadata.len() > 64 * 1024) {
        return Err(io::Error::other("unsafe redirect file"));
    }
    let canonical = opened_path(&file).ok_or_else(|| io::Error::other("no opened file path"))?;
    if !allowed.iter().any(|base| canonical.starts_with(base)) {
        return Err(io::Error::other("unsafe redirect file"));
    }
    let mut bytes = Vec::new();
    (&file).take(64 * 1024).read_to_end(&mut bytes)?;
    let metadata = file.metadata()?;
    if metadata.nlink() != 1 || (append && metadata.len() > 64 * 1024) {
        return Err(io::Error::other("unsafe redirect file"));
    }
    String::from_utf8(bytes).map_err(io::Error::other)
}

// Resolve the opened object, so replacing a pathname cannot redirect validation
// to a different object from the one whose contents we read.
fn opened_path(file: &std::fs::File) -> Option<std::path::PathBuf> {
    use std::os::fd::AsRawFd;
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::ffi::OsStrExt;
        unsafe extern "C" {
            fn fcntl(fd: i32, cmd: i32, ...) -> i32;
        }
        let mut buffer = [0u8; 1024];
        // SAFETY: F_GETPATH (50) writes at most MAXPATHLEN (1024) bytes into
        // this live buffer; the descriptor is owned by file for the call.
        if unsafe { fcntl(file.as_raw_fd(), 50, buffer.as_mut_ptr()) } == -1 {
            return None;
        }
        let end = buffer.iter().position(|b| *b == 0)?;
        Some(Path::new(std::ffi::OsStr::from_bytes(&buffer[..end])).to_owned())
    }
    #[cfg(not(target_os = "macos"))]
    {
        std::fs::read_link(format!("/proc/self/fd/{}", file.as_raw_fd())).ok()
    }
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
    help_options(&words[start..]) || exec_resume_options(words).is_some_and(help_options)
}

fn help_options(words: &[Word]) -> bool {
    let mut options = words.iter();
    while let Some(word) = options.next() {
        let text = word.text.as_str();
        if text == "--" || !text.starts_with('-') || text == "-" {
            break;
        }
        if !word.quoted_or_escaped && matches!(text, "--help" | "-h") {
            return true;
        }
        if option_takes_value(text) {
            options.next();
        }
    }
    false
}

/// Find the resume subcommand after exec options, stopping at the first
/// positional word. Preserve the narrower, existing UUID-resume grammar above.
fn exec_resume_options(words: &[Word]) -> Option<&[Word]> {
    if words.first()?.text != "codex" || words.get(1)?.text != "exec" {
        return None;
    }
    let mut index = 2;
    while let Some(word) = words.get(index) {
        let text = word.text.as_str();
        if word.redirect || text == "--" || text == "-" {
            return None;
        }
        if !text.starts_with('-') {
            return (text == "resume" && !word.quoted_or_escaped).then_some(&words[index + 1..]);
        }
        index += if option_takes_value(text) { 2 } else { 1 };
    }
    None
}

fn resume_options(words: &[Word]) -> Option<&[Word]> {
    if words.first()?.text == "codex" && words.get(1)?.text == "resume" {
        Some(&words[2..])
    } else {
        exec_resume_options(words)
    }
}

/// C3: skip only options whose arity is known before accepting a positional id.
fn resume_selection(words: &[Word]) -> Option<Option<String>> {
    let mut options = words.iter();
    let mut last = false;
    while let Some(word) = options.next() {
        let text = word.text.as_str();
        if word.redirect || matches!(text, "--" | "-") {
            return None;
        }
        if !text.starts_with('-') {
            return if last {
                Some(None)
            } else {
                uuid_shaped(text).then(|| Some(text.to_owned()))
            };
        }
        if text == "--last" && !word.quoted_or_escaped {
            last = true;
            continue;
        }
        if option_takes_value(text) {
            let value = options.next()?;
            if value.redirect {
                return None;
            }
        } else if !(text.starts_with("--") && text.contains('=')
            || text
                .split_once('=')
                .is_some_and(|(name, _)| option_takes_value(name)))
            && !matches!(
                text,
                "--json"
                    | "--skip-git-repo-check"
                    | "--full-auto"
                    | "--dangerously-bypass-approvals-and-sandbox"
                    | "--ephemeral"
                    | "--all"
                    | "--oss"
            )
        {
            return None;
        }
    }
    last.then_some(None)
}

/// Exact option names only: an equals-form already contains its value.
fn option_takes_value(text: &str) -> bool {
    matches!(
        text,
        "-c" | "-o"
            | "-C"
            | "-s"
            | "--add-dir"
            | "--config"
            | "--output-last-message"
            | "--cd"
            | "--sandbox"
            | "-m"
            | "--model"
            | "-i"
            | "--image"
            | "-p"
            | "--profile"
            | "--enable"
            | "--disable"
            | "--output-schema"
            | "--thread-source"
            | "--cyber-access-program"
            | "--local-provider"
            | "--color"
    )
}

/// Skip the branch introducer and narrow wrapper grammar, including stacked wrappers.
fn wrapper_offset(words: &[Word]) -> usize {
    let mut offset = 0;
    loop {
        match words.get(offset).map(|word| word.text.as_str()) {
            Some("then") if !words[offset].quoted_or_escaped => offset += 1,
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
    after_subshell: bool,
    before: &'static str,
    after: &'static str,
    depth: usize,
}

#[derive(Default)]
struct Word {
    text: String,
    // Quoting/escaping is removed from text, but literal $PWD must stay distinct.
    expandable_dollar: bool,
    quoted_or_escaped: bool,
    redirect: bool,
}

fn segments(command: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut words = Vec::new();
    let mut word = Word::default();
    let mut started = false;
    let mut quote = None;
    let mut chars = command.char_indices().peekable();
    let mut segment_start = 0;
    let mut after_subshell = false;
    let mut before = "";
    let mut depth: usize = 0;
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
        // Preserve shell redirection operators separately from words, including
        // glued targets and descriptor duplication. Quoted operators stay text.
        if quote.is_none()
            && (ch == '>' || (ch == '&' && chars.peek().is_some_and(|(_, c)| *c == '>')))
        {
            let descriptor = if started
                && !word.quoted_or_escaped
                && word.text.bytes().all(|b| b.is_ascii_digit())
            {
                std::mem::take(&mut word.text)
            } else {
                if started {
                    words.push(std::mem::take(&mut word));
                }
                String::new()
            };
            word = Word::default();
            started = false;
            let mut operator = descriptor;
            operator.push(ch);
            if ch == '&' {
                chars.next();
                operator.push('>');
            }
            if let Some(&(_, next)) = chars.peek()
                && matches!(next, '>' | '|' | '&')
            {
                chars.next();
                operator.push(next);
                if next == '&' {
                    while let Some(&(_, digit)) = chars.peek() {
                        if !digit.is_ascii_digit() && digit != '-' {
                            break;
                        }
                        chars.next();
                        operator.push(digit);
                    }
                }
            }
            words.push(Word {
                text: operator,
                redirect: true,
                ..Word::default()
            });
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
            if matches!(ch, ';' | '|' | '&' | '(' | ')' | '\n') {
                let doubled =
                    matches!(ch, '|' | '&') && chars.peek().map(|(_, next)| *next) == Some(ch);
                if doubled {
                    chars.next();
                }
                let after = match (ch, doubled) {
                    ('&', true) => "&&",
                    ('&', false) => "&",
                    ('|', true) => "||",
                    ('|', false) => "|",
                    (';', _) => ";",
                    ('(', _) => "(",
                    (')', _) => ")",
                    _ => "\n",
                };
                let segment_end = chars.peek().map_or(command.len(), |(index, _)| *index);
                segments.push(Segment {
                    tokens: std::mem::take(&mut words),
                    span: segment_start..index,
                    after_subshell,
                    before,
                    after,
                    depth,
                });
                before = after;
                if ch == '(' {
                    depth += 1;
                }
                if ch == ')' {
                    depth = depth.saturating_sub(1);
                }
                after_subshell |= ch == '(';
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
            after_subshell,
            before,
            after: "",
            depth,
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
    let checkout = value.get("cwd")?.as_str()?;
    let launch = recognise_in(command, checkout)?;
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
        launch_dir: launch.launch_dir,
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
