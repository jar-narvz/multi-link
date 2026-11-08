//! Graph report tests. The small builders below are reused by later graph tasks:
//! times are RFC 3339 strings, bursts are `(from, to)` pairs.

use std::collections::HashMap;

use multi_link_core::{CodexDetails, GraphInput, build_graph};
use multi_link_model::{
    Activity, Call, CallKind, GraphReport, GraphStep, Harness, InvocationForm, Link, Match,
    Session, State, StateEvidence, Timestamp, Turn,
};

pub fn t(text: &str) -> Timestamp {
    Timestamp::parse(text).unwrap()
}

fn session(id: &str, harness: Harness, repo: &str, bursts: &[(&str, &str)]) -> Session {
    let activity: Vec<Activity> = bursts
        .iter()
        .map(|(from, to)| Activity {
            from: t(from),
            to: t(to),
        })
        .collect();
    Session {
        id: id.to_string(),
        harness,
        transcript: Some(format!("transcripts/{id}.jsonl")),
        record: None,
        host_session_id: None,
        cwd: repo.to_string(),
        repo: repo.to_string(),
        checkout: repo.to_string(),
        branch: None,
        started_at: activity
            .first()
            .map_or(t("2026-11-06T00:00:00Z"), |a| a.from),
        last_activity: activity.last().map_or(t("2026-11-06T00:00:00Z"), |a| a.to),
        state: State::Idle,
        state_evidence: StateEvidence::NoSignal,
        segments: Vec::new(),
        subagents: Vec::new(),
        activity,
        asked: None,
        last_asked: None,
        imported: false,
        state_since: None,
    }
}

pub fn claude_session(id: &str, repo: &str, bursts: &[(&str, &str)]) -> Session {
    session(id, Harness::Claude, repo, bursts)
}

pub fn codex_session(id: &str, bursts: &[(&str, &str)]) -> Session {
    session(id, Harness::Codex, "/repo", bursts)
}

/// A hand-off link. `child` of `None` is an unlinked or ambiguous hand-off.
pub fn link(
    parent: &str,
    invocation: &str,
    step: Option<&str>,
    at: &str,
    r#match: Match,
    child: Option<&str>,
) -> Link {
    Link {
        step: step.map(str::to_string),
        parent: parent.to_string(),
        invocation: invocation.to_string(),
        subagent: None,
        form: InvocationForm::Exec,
        background: false,
        at: t(at),
        checkout: "/repo".to_string(),
        r#match,
        child: child.map(str::to_string),
        evidence: None,
        candidates: Vec::new(),
    }
}

pub fn call(id: &str, at: &str, kind: CallKind, label: Option<&str>, ok: Option<bool>) -> Call {
    Call {
        id: id.to_string(),
        at: t(at),
        kind,
        label: label.map(str::to_string),
        ok,
    }
}

pub fn turn(start: &str, completed: Option<&str>, last: &str, prompt: Option<&str>) -> Turn {
    Turn {
        start: t(start),
        completed_at: completed.map(t),
        last_record_at: t(last),
        prompt: prompt.map(str::to_string),
        last_agent_message: None,
    }
}

/// A parent's hand-off call, as `claude::calls` would report it.
pub fn handoff(id: &str, at: &str) -> Call {
    call(id, at, CallKind::Handoff, Some("▸ codex exec"), None)
}

#[derive(Default)]
pub struct World {
    pub sessions: Vec<Session>,
    pub links: Vec<Link>,
    pub claude_calls: HashMap<String, Vec<Call>>,
    pub codex: HashMap<String, CodexDetails>,
}

impl World {
    pub fn new() -> Self {
        World::default()
    }

    pub fn build(&self, repo: &str, step: &str) -> GraphReport {
        build_graph(&GraphInput {
            now: t("2026-11-06T18:00:00Z"),
            repo,
            step: GraphStep {
                id: step.to_string(),
                title: None,
                verify: None,
            },
            commit: None,
            sessions: &self.sessions,
            links: &self.links,
            claude_calls: &self.claude_calls,
            codex: &self.codex,
        })
    }
}

fn empty_sentence(step: &str) -> String {
    format!("no hand-offs tagged step {step} (a Claude-only step, or its prompt did not name it)")
}

/// One owned, tagged hand-off from `p1` in `/repo`.
fn owned_world() -> World {
    let mut w = World::new();
    w.sessions.push(claude_session(
        "p1",
        "/repo",
        &[("2026-11-06T10:00:00Z", "2026-11-06T10:10:00Z")],
    ));
    w.links.push(link(
        "p1",
        "h1",
        Some("1.1"),
        "2026-11-06T10:05:00Z",
        Match::Unlinked,
        None,
    ));
    w.claude_calls
        .insert("p1".into(), vec![handoff("h1", "2026-11-06T10:05:00Z")]);
    w
}

#[test]
fn owned_tagged_handoff_is_not_empty() {
    let report = owned_world().build("/repo", "1.1");
    assert_eq!(report.empty, None);
    assert_eq!(report.repo, "/repo");
}

#[test]
fn fork_copy_is_not_owned() {
    // The fork's file copies the parent's records, so `links` lists the same
    // invocation under the fork's id; the fork's own calls never hold it.
    let mut w = owned_world();
    w.sessions.push(claude_session(
        "fork",
        "/repo",
        &[("2026-11-06T10:20:00Z", "2026-11-06T10:30:00Z")],
    ));
    w.links.clear();
    w.links.push(link(
        "fork",
        "h1",
        Some("1.1"),
        "2026-11-06T10:05:00Z",
        Match::Unlinked,
        None,
    ));
    w.claude_calls.insert("fork".into(), Vec::new());
    let report = w.build("/repo", "1.1");
    assert_eq!(report.empty, Some(empty_sentence("1.1")));
    assert!(report.parts.is_empty());
}

#[test]
fn other_repo_parent_is_ignored() {
    let w = owned_world();
    let report = w.build("/other", "1.1");
    assert_eq!(report.empty, Some(empty_sentence("1.1")));
}

#[test]
fn unknown_parent_is_ignored() {
    let mut w = owned_world();
    w.sessions.clear();
    let report = w.build("/repo", "1.1");
    assert_eq!(report.empty, Some(empty_sentence("1.1")));
}

#[test]
fn step_1_1_ignores_1_10_tags() {
    let mut w = owned_world();
    w.links[0].step = Some("1.10".into());
    assert_eq!(w.build("/repo", "1.1").empty, Some(empty_sentence("1.1")));
    assert_eq!(w.build("/repo", "1.10").empty, None);
}

#[test]
fn untagged_handoff_alone_is_empty() {
    let mut w = owned_world();
    w.links[0].step = None;
    assert_eq!(w.build("/repo", "1.1").empty, Some(empty_sentence("1.1")));
}
