//! Shared output contract and read-only transcript utilities.

pub mod jsonl;
pub mod time;

use serde::{Deserialize, Serialize};
pub use time::Timestamp;

macro_rules! string_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($variant),+ }
    };
}

string_enum!(Harness { Claude, Codex });
string_enum!(State {
    Busy,
    Idle,
    Finished,
    Stale,
    Unknown,
    Waiting
});
string_enum!(StateEvidence {
    ProcessExit,
    TranscriptExit,
    SessionEnd,
    SessionRecord,
    NoSignal,
    OpenTurn,
    TurnsComplete,
    Hook
});
string_enum!(StepSource { Unknown, PromptTag });
string_enum!(SkipReason {
    InvalidJson,
    EmptyLine,
    UnknownType
});
string_enum!(InvocationForm {
    Exec,
    Resume,
    Plugin
});
string_enum!(Match {
    Exact,
    Heuristic,
    Ambiguous,
    Unlinked
});
string_enum!(LinkEvidence {
    Stdout,
    ResumeArg,
    ThreadReady,
    TaskOutput,
    RedirectFile,
    Hook
});
string_enum!(Convention {
    Supported,
    Missing,
    Unsupported
});
string_enum!(VerifyResult {
    Pass,
    Fail,
    Unknown
});
string_enum!(VerifySource { Reported, Observed });
string_enum!(Tier {
    Waiting,
    Busy,
    Other
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionsReport {
    pub now: Timestamp,
    pub sessions: Vec<Session>,
    pub skipped: Vec<Skipped>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub harness: Harness,
    pub transcript: Option<String>,
    pub record: Option<String>,
    /// Claude record `hostSessionId` (desktop app id); None without one and for Codex.
    pub host_session_id: Option<String>,
    pub cwd: String,
    pub repo: String,
    /// Git toplevel of `cwd` (the worktree or main checkout); `cwd` in fixture mode.
    pub checkout: String,
    pub branch: Option<String>,
    pub started_at: Timestamp,
    pub last_activity: Timestamp,
    pub state: State,
    pub state_evidence: StateEvidence,
    pub segments: Vec<Segment>,
    pub subagents: Vec<String>,
    pub activity: Vec<Activity>,
    pub asked: Option<String>,
    pub last_asked: Option<String>,
    pub title: Option<String>,
    /// Transcript id of the Claude session that started this one (8.4c S1); None otherwise.
    pub started_by: Option<String>,
    pub imported: bool,
    pub state_since: Option<Timestamp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Segment {
    pub from: Timestamp,
    pub to: Timestamp,
    pub step: Option<String>,
    pub step_source: StepSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Skipped {
    pub file: String,
    pub line: usize,
    pub reason: SkipReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinksReport {
    pub links: Vec<Link>,
    pub unparented: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    pub step: Option<String>,
    pub parent: String,
    pub invocation: String,
    pub subagent: Option<String>,
    pub form: InvocationForm,
    pub background: bool,
    pub at: Timestamp,
    pub checkout: String,
    pub r#match: Match,
    pub child: Option<String>,
    pub evidence: Option<LinkEvidence>,
    pub candidates: Vec<Candidate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Candidate {
    pub id: String,
    pub gap_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepsReport {
    pub repos: Vec<RepoSteps>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoSteps {
    pub repo: String,
    pub convention: Convention,
    pub latest_logged_step: Option<String>,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    pub id: String,
    pub title: String,
    pub verify_command: Option<String>,
    pub progress: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub date: String,
    pub agent: Option<String>,
    pub verify: Verify,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verify {
    pub command: Option<String>,
    pub result: VerifyResult,
    pub result_text: String,
    pub source: VerifySource,
}

/// `multi-link overview`: the home screen's data (output-schema "overview").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverviewReport {
    pub now: Timestamp,
    pub hook: HookSummary,
    pub repos: Vec<RepoGroup>,
}

/// Valid hook records observed in this refresh, regardless of the session window.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookSummary {
    pub records: usize,
    pub latest: Option<Timestamp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoGroup {
    /// Latest tagged hand-off for header wording; excluded from the JSON contract.
    #[serde(skip)]
    pub latest_tagged_handoff: Option<Link>,
    pub repo: String,
    pub tier: Tier,
    pub last_activity: Timestamp,
    pub plan: Plan,
    pub active_steps: Vec<String>,
    pub concurrency: Vec<ConcurrencyWarning>,
    pub latest_handoff: Option<Link>,
    pub rows: Vec<Row>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    pub convention: Convention,
    pub latest_step: Option<LatestStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatestStep {
    pub id: String,
    pub title: Option<String>,
    pub entry: Option<Entry>,
}

/// FR17: two or more `busy` sessions shown in one group with the same `cwd`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConcurrencyWarning {
    pub cwd: String,
    pub sessions: Vec<String>,
}

/// Each variant writes its own `"kind"` (a SessionRow carries `"kind": "session"`
/// everywhere, including as a child or a merged member).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// Keep rows inline: boxing would change the public row API for every consumer.
#[allow(clippy::large_enum_variant)]
#[serde(untagged)]
pub enum Row {
    Session(SessionRow),
    Merged(MergedRow),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename = "session")]
pub struct SessionRow {
    /// Display-only checkout place; never changes the recorded JSON branch.
    #[serde(skip)]
    pub place: Option<String>,
    pub id: String,
    pub harness: Harness,
    pub state: State,
    pub state_evidence: StateEvidence,
    pub checkout: String,
    pub branch: Option<String>,
    pub started_at: Timestamp,
    pub last_activity: Timestamp,
    pub handoffs: Vec<Link>,
    pub made: Vec<String>,
    pub activity: Vec<Activity>,
    pub asked: Option<String>,
    pub last_asked: Option<String>,
    pub title: Option<String>,
    /// Transcript id of the Claude session that started this one (8.4c S1); None otherwise.
    pub started_by: Option<String>,
    pub imported: bool,
    pub state_since: Option<Timestamp>,
    pub children: Vec<SessionRow>,
}

/// Four or more quiet top-level rows of one harness. Not actionable (T11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename = "merged")]
pub struct MergedRow {
    pub harness: Harness,
    pub count: usize,
    pub rows: Vec<SessionRow>,
}

/// One stretch of recorded activity, clipped to the report window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Activity {
    pub from: Timestamp,
    pub to: Timestamp,
}

/// Collapse prompt whitespace and truncate on Unicode character boundaries.
pub fn normalize_prompt(text: &str) -> String {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() > 120 {
        text.chars().take(119).chain(['…']).collect()
    } else {
        text
    }
}

string_enum!(CallKind {
    Read,
    Edit,
    Run,
    Handoff
});

/// On-demand transcript data; never embedded in a Session or scanner cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    pub id: String,
    pub at: Timestamp,
    pub kind: CallKind,
    pub label: Option<String>,
    pub ok: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Turn {
    pub start: Timestamp,
    pub completed_at: Option<Timestamp>,
    pub last_record_at: Timestamp,
    pub prompt: Option<String>,
    pub last_agent_message: Option<String>,
}

#[derive(Debug, Default)]
pub struct Calls {
    pub calls: Vec<Call>,
    pub skipped: Vec<Skipped>,
}

/// A deliberately small shell lexer, used for category labels only. No shell is run.
/// Quotes keep separators inside arguments; unsupported syntax remains a run.
pub fn shell_words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut escaped = false;
    for c in text.chars() {
        if escaped {
            word.push(c);
            escaped = false;
        } else if c == '\\' && quote != Some('\'') {
            escaped = true;
        } else if quote == Some(c) {
            quote = None;
        } else if quote.is_none() && matches!(c, '\'' | '"') {
            quote = Some(c);
        } else if quote.is_none() && (c.is_whitespace() || matches!(c, ';' | '&' | '|' | '\n')) {
            if !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
            if matches!(c, ';' | '&' | '|' | '\n') {
                words.push(if c == '\n' {
                    ";".to_owned()
                } else {
                    c.to_string()
                });
            }
        } else {
            word.push(c);
        }
    }
    if !word.is_empty() {
        words.push(word);
    }
    words
}

/// Fixed G5/G6 labels only: arguments and arbitrary command text never escape.
/// Launch recognition belongs to the Claude reader, which has the full grammar.
pub fn classify_bash(command: &str) -> (CallKind, Option<String>) {
    let words = shell_words(command);
    let head = skip_bash_prefixes(&words);
    let first = head.first().map(String::as_str).unwrap_or_default();
    let second = head.get(1).map(String::as_str).unwrap_or_default();
    let kind = if matches!(
        first,
        "cat" | "sed" | "grep" | "rg" | "head" | "tail" | "ls" | "awk" | "wc" | "find"
    ) || first == "git" && matches!(second, "diff" | "status" | "log" | "show")
    {
        CallKind::Read
    } else {
        CallKind::Run
    };
    let label_words = shell_words(&without_heredoc_bodies(command));
    let mut label = None;
    for segment in label_words.split(|word| matches!(word.as_str(), ";" | "&" | "|")) {
        let head = skip_bash_prefixes(segment);
        let first = head.first().map(String::as_str).unwrap_or_default();
        let second = head.get(1).map(String::as_str).unwrap_or_default();
        let next = match (first, second) {
            ("git", "diff" | "commit")
            | ("cargo", "test" | "clippy" | "fmt" | "build")
            | ("npm", "test") => Some(format!("$ {first} {second}")),
            ("pytest", _) => Some("$ pytest".to_owned()),
            _ => None,
        };
        if next.as_deref() == Some("$ git commit") {
            return (kind, next);
        }
        if label.is_none() {
            label = next;
        }
    }
    (kind, label)
}

fn skip_bash_prefixes(words: &[String]) -> &[String] {
    let mut i = 0;
    while i < words.len() {
        if matches!(words[i].as_str(), "export" | "cd") {
            let Some(end) = words[i..]
                .iter()
                .position(|w| matches!(w.as_str(), ";" | "&"))
            else {
                return &[];
            };
            i += end + 1;
        } else if matches!(words[i].as_str(), ";" | "&")
            || words[i].split_once('=').is_some_and(|(key, _)| {
                !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            })
        {
            i += 1;
        } else {
            break;
        }
    }
    &words[i..]
}

/// Keep the introducing line, but never classify text inside a heredoc body.
fn without_heredoc_bodies(command: &str) -> String {
    let mut output = String::new();
    let mut tags = std::collections::VecDeque::new();
    for line in command.lines() {
        if let Some(tag) = tags.front() {
            if line.trim() == tag {
                tags.pop_front();
            }
            continue;
        }
        output.push_str(line);
        output.push('\n');
        let mut rest = line;
        while let Some(index) = rest.find("<<") {
            rest = &rest[index + 2..];
            if rest.starts_with('<') {
                rest = rest.trim_start_matches('<');
                continue;
            }
            rest = rest.strip_prefix('-').unwrap_or(rest).trim_start();
            let tag = if let Some(quote) = rest.chars().next().filter(|c| matches!(c, '\'' | '"')) {
                rest[1..].find(quote).map(|end| &rest[1..end + 1])
            } else {
                let end = rest
                    .find(|c: char| c.is_whitespace() || matches!(c, ';' | '&' | '|' | ')' | '('))
                    .unwrap_or(rest.len());
                Some(&rest[..end])
            };
            if let Some(tag) = tag.filter(|tag| !tag.is_empty()) {
                tags.push_back(tag.to_owned());
            }
        }
    }
    output
}

pub fn edit_label(path: &str) -> Option<String> {
    let basename = std::path::Path::new(path.trim()).file_name()?.to_str()?;
    (!basename.is_empty()).then(|| format!("edit {basename}"))
}

/// Decode a quoted JS string without evaluating code. JSON double quotes and
/// simple single-quoted strings are assumptions of the undocumented exec format.
pub fn quoted_string(text: &str) -> Option<String> {
    let text = text.trim_start();
    let quote = text.chars().next()?;
    if !matches!(quote, '"' | '\'') {
        return None;
    }
    let mut escaped = false;
    for (i, c) in text.char_indices().skip(1) {
        if !escaped && c == quote {
            if quote == '"' {
                return serde_json::from_str(&text[..=i]).ok();
            }
            let inner = &text[1..i];
            return Some(
                inner
                    .replace("\\'", "'")
                    .replace("\\n", "\n")
                    .replace("\\\\", "\\"),
            );
        }
        escaped = !escaped && c == '\\';
    }
    None
}

string_enum!(PartStatus {
    Done,
    Working,
    Stale,
    Unlinked,
    Ambiguous,
    Unavailable
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphReport {
    pub now: Timestamp,
    pub repo: String,
    pub step: GraphStep,
    pub commit: Option<GraphCommit>,
    pub parts: Vec<Part>,
    pub empty: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphStep {
    pub id: String,
    pub title: Option<String>,
    pub verify: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphCommit {
    pub hash: String,
    pub at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Part {
    pub chain: usize,
    pub kind: String,
    pub harness: Harness,
    pub session: Option<String>,
    pub from: Option<Timestamp>,
    pub to: Option<Timestamp>,
    pub status: Option<PartStatus>,
    pub run_available: Option<bool>,
    pub active_ms: i64,
    pub stretches: usize,
    pub asked: Option<String>,
    pub said: Option<String>,
    pub counts: Counts,
    pub notable: Vec<Notable>,
    pub waiting_reads: usize,
    pub untagged_handoffs: usize,
    pub edge_in: Option<Edge>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counts {
    pub reads: usize,
    pub edits: usize,
    pub runs: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notable {
    pub label: String,
    pub ok: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Edge {
    Handoff {
        at: Timestamp,
        #[serde(rename = "match")]
        r#match: Match,
        parent: String,
        parent_part: Option<usize>,
        gap_ms: Option<i64>,
        candidates: usize,
    },
    Return {
        at: Timestamp,
    },
}
