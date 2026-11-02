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
