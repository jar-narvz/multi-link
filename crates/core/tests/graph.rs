//! Graph report tests. The small builders below are reused by later graph tasks:
//! times are RFC 3339 strings, bursts are `(from, to)` pairs.

use std::collections::HashMap;

use multi_link_core::{CodexDetails, GraphInput, build_graph};
use multi_link_model::{
    Activity, Call, CallKind, Candidate, Edge, GraphReport, GraphStep, Harness, InvocationForm,
    Link, Match, Part, PartStatus, Session, State, StateEvidence, Timestamp, Turn,
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
        self.build_at(repo, step, "2026-11-06T18:00:00Z")
    }

    pub fn build_at(&self, repo: &str, step: &str, now: &str) -> GraphReport {
        build_graph(&GraphInput {
            now: t(now),
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

impl World {
    /// An owned hand-off: the link plus the parent's own hand-off call.
    pub fn hand_off(
        &mut self,
        parent: &str,
        invocation: &str,
        step: Option<&str>,
        at: &str,
        r#match: Match,
        child: Option<&str>,
    ) {
        self.links
            .push(link(parent, invocation, step, at, r#match, child));
        self.claude_calls
            .entry(parent.to_string())
            .or_default()
            .push(handoff(invocation, at));
    }

    /// A readable rollout for `child` with these turns.
    pub fn rollout(&mut self, child: &str, turns: Vec<Turn>) {
        self.codex.insert(
            child.to_string(),
            CodexDetails {
                calls: Vec::new(),
                turns,
            },
        );
    }
}

/// A completed turn that said `said`.
pub fn done_turn(start: &str, end: &str, prompt: &str, said: &str) -> Turn {
    Turn {
        last_agent_message: Some(said.to_string()),
        ..turn(start, Some(end), end, Some(prompt))
    }
}

pub fn codex_parts(report: &GraphReport) -> Vec<&Part> {
    report
        .parts
        .iter()
        .filter(|p| p.harness == Harness::Codex)
        .collect()
}

/// `edge_in` of a Codex part without `parent_part` (Task 4 fills it):
/// `(at, match, parent, gap_ms, candidates)`.
pub fn handoff_edge(part: &Part) -> (Timestamp, Match, String, Option<i64>, usize) {
    match part.edge_in.clone() {
        Some(Edge::Handoff {
            at,
            r#match,
            parent,
            gap_ms,
            candidates,
            ..
        }) => (at, r#match, parent, gap_ms, candidates),
        other => panic!("expected a handoff edge, got {other:?}"),
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

/// Parent `p` with one long burst, so chains are never cut by Claude bursts.
fn parent_world(p: &str) -> World {
    let mut w = World::new();
    w.sessions.push(claude_session(
        p,
        "/repo",
        &[("2026-11-06T08:00:00Z", "2026-11-06T16:00:00Z")],
    ));
    w
}

fn kinds(parts: &[&Part]) -> Vec<(usize, String)> {
    parts.iter().map(|p| (p.chain, p.kind.clone())).collect()
}

#[test]
fn two_resumes_in_one_burst_are_two_runs() {
    let mut w = parent_world("p1");
    w.sessions.push(codex_session(
        "c1",
        &[("2026-11-06T09:00:21Z", "2026-11-06T09:03:00Z")],
    ));
    w.hand_off(
        "p1",
        "h1",
        Some("1.9"),
        "2026-11-06T09:00:20Z",
        Match::Heuristic,
        Some("c1"),
    );
    w.hand_off(
        "p1",
        "h2",
        Some("1.9"),
        "2026-11-06T09:01:20Z",
        Match::Exact,
        Some("c1"),
    );
    w.hand_off(
        "p1",
        "h3",
        Some("1.10"),
        "2026-11-06T09:02:20Z",
        Match::Exact,
        Some("c1"),
    );
    w.links[0].candidates = vec![Candidate {
        id: "c1".into(),
        gap_ms: 1000,
    }];
    w.rollout(
        "c1",
        vec![
            done_turn(
                "2026-11-06T09:00:21Z",
                "2026-11-06T09:01:00Z",
                "do 1.9",
                "Retry added.",
            ),
            done_turn(
                "2026-11-06T09:01:21Z",
                "2026-11-06T09:02:00Z",
                "fix delay",
                "Delay fixed.",
            ),
            done_turn(
                "2026-11-06T09:02:21Z",
                "2026-11-06T09:03:00Z",
                "do 1.10",
                "Backoff added.",
            ),
        ],
    );

    let report = w.build("/repo", "1.9");
    let parts = codex_parts(&report);
    assert_eq!(
        kinds(&parts),
        vec![(0, "implement".into()), (0, "fix round 1".into())]
    );
    let first = parts[0];
    assert_eq!(first.session.as_deref(), Some("c1"));
    assert_eq!(first.status, Some(PartStatus::Done));
    assert_eq!(first.run_available, Some(true));
    assert_eq!(first.from, Some(t("2026-11-06T09:00:21Z")));
    assert_eq!(first.to, Some(t("2026-11-06T09:01:00Z")));
    assert_eq!(first.asked.as_deref(), Some("do 1.9"));
    assert_eq!(first.said.as_deref(), Some("Retry added."));
    assert_eq!((first.active_ms, first.stretches), (39000, 1));
    assert_eq!(
        handoff_edge(first),
        (
            t("2026-11-06T09:00:20Z"),
            Match::Heuristic,
            "p1".to_string(),
            Some(1000),
            1
        )
    );
    assert_eq!(first.counts, Default::default());
    assert_eq!((first.waiting_reads, first.untagged_handoffs), (0, 0));
    let second = parts[1];
    assert_eq!(second.from, Some(t("2026-11-06T09:01:21Z")));
    assert_eq!(second.to, Some(t("2026-11-06T09:02:00Z")));
    assert_eq!(second.asked.as_deref(), Some("fix delay"));
    assert_eq!(second.said.as_deref(), Some("Delay fixed."));
    assert_eq!(handoff_edge(second).4, 0);

    let later = w.build("/repo", "1.10");
    let parts = codex_parts(&later);
    assert_eq!(kinds(&parts), vec![(0, "implement".into())]);
    assert_eq!(parts[0].from, Some(t("2026-11-06T09:02:21Z")));
    assert_eq!(parts[0].said.as_deref(), Some("Backoff added."));
}

#[test]
fn run_window_is_half_open() {
    // A turn starting exactly at the hand-off is in its run; one starting
    // exactly at the next hand-off of the same child is not.
    let mut w = parent_world("p1");
    w.sessions.push(codex_session(
        "c1",
        &[("2026-11-06T09:00:00Z", "2026-11-06T09:02:00Z")],
    ));
    w.hand_off(
        "p1",
        "h1",
        Some("1.1"),
        "2026-11-06T09:00:00Z",
        Match::Exact,
        Some("c1"),
    );
    w.hand_off(
        "p1",
        "h2",
        Some("1.1"),
        "2026-11-06T09:01:00Z",
        Match::Exact,
        Some("c1"),
    );
    w.rollout(
        "c1",
        vec![
            done_turn("2026-11-06T09:00:00Z", "2026-11-06T09:00:30Z", "one", "a"),
            done_turn("2026-11-06T09:01:00Z", "2026-11-06T09:01:30Z", "two", "b"),
        ],
    );
    let report = w.build("/repo", "1.1");
    let parts = codex_parts(&report);
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].to, Some(t("2026-11-06T09:00:30Z")));
    assert_eq!(handoff_edge(parts[0]).3, Some(0));
    assert_eq!(parts[1].from, Some(t("2026-11-06T09:01:00Z")));
}

#[test]
fn a_b_a_is_two_chains() {
    let mut w = parent_world("p1");
    w.hand_off(
        "p1",
        "h1",
        Some("1.6"),
        "2026-11-06T11:00:20Z",
        Match::Unlinked,
        None,
    );
    w.hand_off(
        "p1",
        "h2",
        Some("1.7"),
        "2026-11-06T11:03:00Z",
        Match::Unlinked,
        None,
    );
    w.hand_off(
        "p1",
        "h3",
        Some("1.6"),
        "2026-11-06T11:05:00Z",
        Match::Unlinked,
        None,
    );
    let report = w.build("/repo", "1.6");
    let parts = codex_parts(&report);
    assert_eq!(
        kinds(&parts),
        vec![(0, "implement".into()), (1, "implement".into())]
    );
    assert_eq!(handoff_edge(parts[1]).0, t("2026-11-06T11:05:00Z"));
    let b = w.build("/repo", "1.7");
    assert_eq!(kinds(&codex_parts(&b)), vec![(0, "implement".into())]);
}

#[test]
fn untagged_handoff_does_not_split_a_chain() {
    let mut w = parent_world("p1");
    w.hand_off(
        "p1",
        "h1",
        Some("1.8"),
        "2026-11-06T11:00:00Z",
        Match::Unlinked,
        None,
    );
    w.hand_off(
        "p1",
        "h2",
        None,
        "2026-11-06T11:02:00Z",
        Match::Unlinked,
        None,
    );
    w.hand_off(
        "p1",
        "h3",
        Some("1.8"),
        "2026-11-06T11:04:00Z",
        Match::Unlinked,
        None,
    );
    let report = w.build("/repo", "1.8");
    assert_eq!(
        kinds(&codex_parts(&report)),
        vec![(0, "implement".into()), (0, "fix round 1".into())]
    );
}

#[test]
fn chains_ordered_across_parents() {
    let mut w = parent_world("p1");
    w.sessions.push(claude_session(
        "p2",
        "/repo",
        &[("2026-11-06T12:00:00Z", "2026-11-06T14:00:00Z")],
    ));
    // p1's hand-off is listed first but is later in time.
    w.hand_off(
        "p1",
        "h1",
        Some("1.4"),
        "2026-11-06T13:20:20Z",
        Match::Unlinked,
        None,
    );
    w.hand_off(
        "p2",
        "h2",
        Some("1.4"),
        "2026-11-06T13:00:20Z",
        Match::Unlinked,
        None,
    );
    let report = w.build("/repo", "1.4");
    let parts = codex_parts(&report);
    assert_eq!(
        kinds(&parts),
        vec![(0, "implement".into()), (1, "implement".into())]
    );
    assert_eq!(handoff_edge(parts[0]).2, "p2");
    assert_eq!(handoff_edge(parts[1]).2, "p1");
}

fn open_world() -> World {
    let mut w = parent_world("p1");
    w.sessions.push(codex_session(
        "c1",
        &[("2026-11-06T17:52:01Z", "2026-11-06T17:58:30Z")],
    ));
    w.hand_off(
        "p1",
        "h1",
        Some("1.5"),
        "2026-11-06T17:52:00Z",
        Match::Heuristic,
        Some("c1"),
    );
    w.rollout(
        "c1",
        vec![turn(
            "2026-11-06T17:52:01Z",
            None,
            "2026-11-06T17:58:30Z",
            Some("second half"),
        )],
    );
    w
}

#[test]
fn open_turn_working_then_stale() {
    let w = open_world();
    let working = w.build_at("/repo", "1.5", "2026-11-06T18:03:29Z");
    let part = codex_parts(&working)[0];
    assert_eq!(part.status, Some(PartStatus::Working));
    assert_eq!(part.run_available, Some(true));
    assert_eq!(part.to, Some(t("2026-11-06T17:58:30Z")));
    assert_eq!(part.asked.as_deref(), Some("second half"));
    assert_eq!(part.said, None);
    assert_eq!((part.active_ms, part.stretches), (389000, 1));

    let edge = w.build_at("/repo", "1.5", "2026-11-06T18:03:30Z");
    assert_eq!(codex_parts(&edge)[0].status, Some(PartStatus::Working));

    let stale = w.build_at("/repo", "1.5", "2026-11-06T18:03:31Z");
    let part = codex_parts(&stale)[0];
    assert_eq!(part.status, Some(PartStatus::Stale));
    assert_eq!(part.to, Some(t("2026-11-06T17:58:30Z")));
}

fn assert_no_run(part: &Part, status: PartStatus, session: Option<&str>) {
    assert_eq!(part.status, Some(status));
    assert_eq!(part.session.as_deref(), session);
    assert_eq!(part.run_available, Some(false));
    assert_eq!((part.from, part.to), (None, None));
    assert_eq!((part.active_ms, part.stretches), (0, 0));
    assert_eq!((part.asked.as_deref(), part.said.as_deref()), (None, None));
    assert_eq!(handoff_edge(part).3, None);
}

#[test]
fn unlinked_and_ambiguous_have_no_interval() {
    let mut w = parent_world("p1");
    w.hand_off(
        "p1",
        "h1",
        Some("1.2"),
        "2026-11-06T15:00:20Z",
        Match::Unlinked,
        None,
    );
    w.hand_off(
        "p1",
        "h2",
        Some("1.2"),
        "2026-11-06T15:03:00Z",
        Match::Ambiguous,
        None,
    );
    w.links[1].candidates = vec![
        Candidate {
            id: "c1".into(),
            gap_ms: 1000,
        },
        Candidate {
            id: "c2".into(),
            gap_ms: 2000,
        },
    ];
    let report = w.build("/repo", "1.2");
    let parts = codex_parts(&report);
    assert_eq!(
        kinds(&parts),
        vec![(0, "implement".into()), (0, "fix round 1".into())]
    );
    assert_no_run(parts[0], PartStatus::Unlinked, None);
    assert_eq!(
        handoff_edge(parts[0]),
        (
            t("2026-11-06T15:00:20Z"),
            Match::Unlinked,
            "p1".to_string(),
            None,
            0
        )
    );
    assert_no_run(parts[1], PartStatus::Ambiguous, None);
    assert_eq!(handoff_edge(parts[1]).1, Match::Ambiguous);
    assert_eq!(handoff_edge(parts[1]).4, 2);
}

#[test]
fn child_without_rollout_is_unavailable() {
    // Exact link, but the child has no Session (or its rollout vanished), so
    // `codex` has no entry for it.
    let mut w = parent_world("p1");
    w.hand_off(
        "p1",
        "h1",
        Some("1.11"),
        "2026-11-06T08:00:20Z",
        Match::Exact,
        Some("c9"),
    );
    let report = w.build("/repo", "1.11");
    let parts = codex_parts(&report);
    assert_eq!(kinds(&parts), vec![(0, "implement".into())]);
    assert_no_run(parts[0], PartStatus::Unavailable, Some("c9"));
    assert_eq!(handoff_edge(parts[0]).1, Match::Exact);
}

#[test]
fn rollout_with_no_turn_in_window_is_unavailable() {
    let mut w = parent_world("p1");
    w.sessions.push(codex_session(
        "c1",
        &[("2026-11-06T09:00:00Z", "2026-11-06T10:10:00Z")],
    ));
    // Window of h1 is [10:00, 10:05): the only turns are before it and at
    // the next hand-off of the same child (another step).
    w.hand_off(
        "p1",
        "h1",
        Some("2.1"),
        "2026-11-06T10:00:00Z",
        Match::Exact,
        Some("c1"),
    );
    w.hand_off(
        "p1",
        "h2",
        Some("2.2"),
        "2026-11-06T10:05:00Z",
        Match::Exact,
        Some("c1"),
    );
    w.rollout(
        "c1",
        vec![
            done_turn("2026-11-06T09:00:00Z", "2026-11-06T09:01:00Z", "old", "x"),
            done_turn("2026-11-06T10:05:00Z", "2026-11-06T10:06:00Z", "next", "y"),
        ],
    );
    let report = w.build("/repo", "2.1");
    assert_no_run(codex_parts(&report)[0], PartStatus::Unavailable, Some("c1"));

    // A readable rollout with no turns at all.
    w.rollout("c1", Vec::new());
    let report = w.build("/repo", "2.2");
    assert_no_run(codex_parts(&report)[0], PartStatus::Unavailable, Some("c1"));
}

#[test]
fn silent_turn_two_stretches() {
    let mut w = parent_world("p1");
    // A 5 min 19 s silence splits the child's activity into two bursts.
    w.sessions.push(codex_session(
        "c1",
        &[
            ("2026-11-06T14:00:21Z", "2026-11-06T14:00:51Z"),
            ("2026-11-06T14:06:10Z", "2026-11-06T14:06:40Z"),
        ],
    ));
    w.hand_off(
        "p1",
        "h1",
        Some("1.3"),
        "2026-11-06T14:00:20Z",
        Match::Exact,
        Some("c1"),
    );
    w.rollout(
        "c1",
        vec![done_turn(
            "2026-11-06T14:00:21Z",
            "2026-11-06T14:06:40Z",
            "go",
            "done",
        )],
    );
    let report = w.build("/repo", "1.3");
    let part = codex_parts(&report)[0];
    assert_eq!((part.active_ms, part.stretches), (60000, 2));
}

#[test]
fn single_record_burst_touching_a_turn_is_a_stretch() {
    let mut w = parent_world("p1");
    w.sessions.push(codex_session(
        "c1",
        &[
            ("2026-11-06T14:00:00Z", "2026-11-06T14:00:10Z"),
            ("2026-11-06T14:01:00Z", "2026-11-06T14:01:00Z"),
            ("2026-11-06T14:02:00Z", "2026-11-06T14:03:00Z"),
        ],
    ));
    w.hand_off(
        "p1",
        "h1",
        Some("1.3"),
        "2026-11-06T13:59:00Z",
        Match::Exact,
        Some("c1"),
    );
    // The turn ends exactly where the single-record burst sits; the third
    // burst is outside the turn.
    w.rollout(
        "c1",
        vec![done_turn(
            "2026-11-06T14:00:05Z",
            "2026-11-06T14:01:00Z",
            "go",
            "done",
        )],
    );
    let report = w.build("/repo", "1.3");
    let part = codex_parts(&report)[0];
    assert_eq!((part.active_ms, part.stretches), (5000, 2));
}

#[test]
fn fix_round_numbering_restarts_per_chain() {
    let mut w = parent_world("p1");
    w.hand_off(
        "p1",
        "h1",
        Some("3.1"),
        "2026-11-06T10:00:00Z",
        Match::Unlinked,
        None,
    );
    w.hand_off(
        "p1",
        "h2",
        Some("3.1"),
        "2026-11-06T10:10:00Z",
        Match::Unlinked,
        None,
    );
    w.hand_off(
        "p1",
        "h3",
        Some("3.1"),
        "2026-11-06T10:20:00Z",
        Match::Unlinked,
        None,
    );
    w.hand_off(
        "p1",
        "h4",
        Some("3.2"),
        "2026-11-06T10:30:00Z",
        Match::Unlinked,
        None,
    );
    w.hand_off(
        "p1",
        "h5",
        Some("3.1"),
        "2026-11-06T10:40:00Z",
        Match::Unlinked,
        None,
    );
    w.hand_off(
        "p1",
        "h6",
        Some("3.1"),
        "2026-11-06T10:50:00Z",
        Match::Unlinked,
        None,
    );
    let report = w.build("/repo", "3.1");
    assert_eq!(
        kinds(&codex_parts(&report)),
        vec![
            (0, "implement".into()),
            (0, "fix round 1".into()),
            (0, "fix round 2".into()),
            (1, "implement".into()),
            (1, "fix round 1".into()),
        ]
    );
}
