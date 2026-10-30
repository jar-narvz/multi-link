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
    pub cwd: String,
    pub repo: String,
    pub branch: Option<String>,
    pub started_at: Timestamp,
    pub last_activity: Timestamp,
    pub state: State,
    pub state_evidence: StateEvidence,
    pub segments: Vec<Segment>,
    pub subagents: Vec<String>,
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
