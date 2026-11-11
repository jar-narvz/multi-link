//! The step graph report (design G0-G11). `build` is pure; `Scanner::graph`
//! does the I/O and hands `build` everything it needs.

use std::{
    collections::{HashMap, HashSet},
    io,
    path::Path,
    process::Command,
};

use multi_link_model::{
    Activity, Call, CallKind, Calls, Counts, Edge, GraphCommit, GraphReport, GraphStep, Harness,
    Link, LinkEvidence, Match, Notable, Part, PartStatus, RepoSteps, Session, Timestamp, Turn,
};

use crate::{Options, RepoMode, Scanner, overview::plan_folder};

/// Presentation-only live part indices; the serialized report is unchanged.
/// Pick the last interval per session (ties by index), across all of its chains.
pub fn live_parts(report: &GraphReport, sessions: &[Session]) -> Vec<usize> {
    let mut last = HashMap::new();
    for (i, part) in report.parts.iter().enumerate() {
        if let Some(id) = part.session.as_deref() {
            // An unavailable resumed run has no interval, but its hand-off still
            // dates the window and supersedes the previous run of this child.
            let start = part.from.or(match part.edge_in {
                Some(Edge::Handoff { at, .. }) => Some(at),
                _ => None,
            });
            let key = (start, i);
            let entry = last
                .entry((part.harness == Harness::Codex, id))
                .or_insert(key);
            *entry = (*entry).max(key);
        }
    }
    let latest = |i: usize, part: &Part| {
        part.session.as_deref().is_some_and(|id| {
            last.get(&(part.harness == Harness::Codex, id))
                .is_some_and(|&(_, index)| index == i)
        })
    };
    let codex: Vec<usize> = report
        .parts
        .iter()
        .enumerate()
        .filter_map(|(i, part)| {
            (part.harness == Harness::Codex
                && latest(i, part)
                && part.status == Some(PartStatus::Working)
                && part.run_available == Some(true)
                && part.from.is_some()
                && part.to.is_some())
            .then_some(i)
        })
        .collect();
    report
        .parts
        .iter()
        .enumerate()
        .filter_map(|(i, part)| {
            let live = if part.harness == Harness::Codex {
                codex.contains(&i)
            } else {
                latest(i, part)
                    && sessions.iter().any(|s| {
                        s.harness == Harness::Claude
                            && Some(s.id.as_str()) == part.session.as_deref()
                            && s.state == multi_link_model::State::Busy
                    })
                    && part.to.is_some_and(|to| {
                        (0..=STALE_MS).contains(&(report.now.epoch_ms() - to.epoch_ms()))
                    })
                    && !codex
                        .iter()
                        .any(|&j| j > i && report.parts[j].chain == part.chain)
            };
            live.then_some(i)
        })
        .collect()
}

/// A Codex run read from its rollout.
#[derive(Debug, Clone, Default)]
pub struct CodexDetails {
    pub calls: Vec<Call>,
    pub turns: Vec<Turn>,
}

/// Everything `build` reads. Sessions and links come from a `--since all`
/// refresh. `claude_calls` is keyed by Claude session id (owned calls, G5).
/// `codex` is keyed by child session id; an absent key means the run is
/// unavailable (no session, or the rollout could not be read).
pub struct GraphInput<'a> {
    pub now: Timestamp,
    pub repo: &'a str,
    pub step: GraphStep,
    pub commit: Option<GraphCommit>,
    pub sessions: &'a [Session],
    pub links: &'a [Link],
    pub claude_calls: &'a HashMap<String, Vec<Call>>,
    pub codex: &'a HashMap<String, CodexDetails>,
}

/// True when `link` is an owned hand-off of a listed Claude session in `repo`
/// (design, pinned G1): its `invocation` is one of its parent's own calls.
/// A hand-off copied into a fork is listed again under the fork's id, but the
/// fork's calls do not hold its invocation, so it is not owned.
fn owned(
    link: &Link,
    repo: &str,
    sessions: &[Session],
    claude_calls: &HashMap<String, Vec<Call>>,
) -> bool {
    sessions
        .iter()
        .any(|s| s.id == link.parent && s.harness == Harness::Claude && s.repo == repo)
        && claude_calls
            .get(&link.parent)
            .is_some_and(|calls| calls.iter().any(|call| call.id == link.invocation))
}

/// Time order of hand-offs: `(at, invocation)`, then parent so the order is
/// total across parents.
fn order(link: &Link) -> (Timestamp, &str, &str) {
    (link.at, link.invocation.as_str(), link.parent.as_str())
}

/// A Codex run: the child's turns that start in its hand-off's window (G4).
/// `turns` is empty exactly when the run has no interval (unlinked,
/// ambiguous or unavailable).
struct Run<'a> {
    status: PartStatus,
    session: Option<String>,
    turns: Vec<&'a Turn>,
}

/// One tagged hand-off of a chain and the run it launched.
struct Hop<'a> {
    link: &'a Link,
    run: Run<'a>,
}

/// One chain (G2): a parent's hand-offs tagged with the step, in time order,
/// with no hand-off tagged another step between them. Claude parts are cut
/// around these hops (`cut`), which flattens chains into parts.
struct Chain<'a> {
    hops: Vec<Hop<'a>>,
}

/// Chains of step `id`, ordered by first hand-off across parents. `owned` is
/// every owned hand-off of any step, sorted by `order`.
fn chains<'a>(input: &GraphInput<'a>, owned: &[&'a Link], id: &str) -> Vec<Chain<'a>> {
    let mut parents: Vec<&str> = owned.iter().map(|l| l.parent.as_str()).collect();
    parents.sort_unstable();
    parents.dedup();
    let mut groups: Vec<Vec<&'a Link>> = Vec::new();
    for parent in parents {
        let mut current: Option<Vec<&'a Link>> = None;
        for &link in owned.iter().filter(|l| l.parent == parent) {
            match link.step.as_deref() {
                Some(step) if step == id => current.get_or_insert_with(Vec::new).push(link),
                // A hand-off tagged another step ends the chain (pinned G2).
                Some(_) => groups.extend(current.take()),
                // An untagged hand-off never starts, ends or splits a chain.
                None => {}
            }
        }
        groups.extend(current);
    }
    groups.sort_by(|a, b| order(a[0]).cmp(&order(b[0])));
    groups
        .into_iter()
        .map(|links| Chain {
            hops: links
                .into_iter()
                .map(|link| Hop {
                    link,
                    run: run(input, owned, link),
                })
                .collect(),
        })
        .collect()
}

/// The run of one hand-off (G4, pinned G4 Status). Its window is from `at` to
/// the next owned hand-off with the same child, of any parent and any step,
/// half-open; unbounded when there is none.
fn run<'a>(input: &GraphInput<'a>, owned: &[&'a Link], link: &Link) -> Run<'a> {
    let no_run = |status, session: Option<&str>| Run {
        status,
        session: session.map(str::to_string),
        turns: Vec::new(),
    };
    match link.r#match {
        Match::Unlinked => return no_run(PartStatus::Unlinked, None),
        Match::Ambiguous => return no_run(PartStatus::Ambiguous, None),
        Match::Exact | Match::Heuristic => {}
    }
    let Some(child) = link.child.as_deref() else {
        return no_run(PartStatus::Unavailable, None);
    };
    let next = owned
        .iter()
        .filter(|l| l.child.as_deref() == Some(child) && order(l) > order(link))
        .map(|l| l.at)
        .min();
    let mut turns: Vec<&'a Turn> = input
        .codex
        .get(child)
        .map(|details| {
            details
                .turns
                .iter()
                .filter(|t| t.start >= link.at && next.is_none_or(|next| t.start < next))
                .collect()
        })
        .unwrap_or_default();
    turns.sort_by_key(|t| t.start);
    let Some(last) = turns.last() else {
        // No session, an unreadable rollout, or no turn in the window.
        return no_run(PartStatus::Unavailable, Some(child));
    };
    let status = if last.completed_at.is_some() {
        PartStatus::Done
    } else if input.now.epoch_ms() - last.last_record_at.epoch_ms() > STALE_MS {
        PartStatus::Stale
    } else {
        PartStatus::Working
    };
    Run {
        status,
        session: Some(child.to_string()),
        turns,
    }
}

/// The State rule's silence threshold (pinned G4 Status).
const STALE_MS: i64 = 300_000;

/// A turn's end: `task_complete`, or its last record while open.
fn turn_end(turn: &Turn) -> Timestamp {
    turn.completed_at.unwrap_or(turn.last_record_at)
}

/// One stretch per non-empty burst ∩ turn, single records included; returns
/// `(active_ms, stretches)`.
fn stretches(bursts: &[Activity], turns: &[&Turn]) -> (i64, usize) {
    let mut active_ms = 0;
    let mut count = 0;
    for burst in bursts {
        for turn in turns {
            let from = burst.from.max(turn.start);
            let to = burst.to.min(turn_end(turn));
            if from <= to {
                active_ms += to.epoch_ms() - from.epoch_ms();
                count += 1;
            }
        }
    }
    (active_ms, count)
}

/// The Codex part of a hop at `position` among its chain's Codex parts (G3).
fn codex_part(
    input: &GraphInput<'_>,
    chain: usize,
    position: usize,
    hop: &Hop<'_>,
    parent_part: Option<usize>,
) -> Part {
    let Hop { link, run } = hop;
    let first = run.turns.first();
    let last = run.turns.last();
    let from = first.map(|t| t.start);
    let bursts = run
        .session
        .as_deref()
        .and_then(|child| {
            input
                .sessions
                .iter()
                .find(|s| s.id == child && s.harness == Harness::Codex)
        })
        .map_or(&[][..], |s| s.activity.as_slice());
    let (active_ms, stretches) = stretches(bursts, &run.turns);
    // Codex calls belong to the run whose turn holds them (pinned G6).
    let held: Vec<(Timestamp, &Call)> = run
        .session
        .as_deref()
        .and_then(|child| input.codex.get(child))
        .map(|details| {
            details
                .calls
                .iter()
                .filter(|c| {
                    run.turns
                        .iter()
                        .any(|t| t.start <= c.at && c.at <= turn_end(t))
                })
                .map(|c| (c.at, c))
                .collect()
        })
        .unwrap_or_default();
    let (counts, notable) = tally(&held, &HashMap::new());
    Part {
        chain,
        kind: match position {
            0 => "implement".to_string(),
            n => format!("fix round {n}"),
        },
        harness: Harness::Codex,
        session: run.session.clone(),
        from,
        to: last.map(|t| turn_end(t)),
        status: Some(run.status),
        run_available: Some(!run.turns.is_empty()),
        active_ms,
        stretches,
        asked: first.and_then(|t| t.prompt.clone()),
        said: last
            .filter(|t| t.completed_at.is_some())
            .and_then(|t| t.last_agent_message.clone()),
        counts,
        notable,
        waiting_reads: 0,
        untagged_handoffs: 0,
        edge_in: Some(Edge::Handoff {
            at: link.at,
            r#match: link.r#match,
            parent: link.parent.clone(),
            parent_part,
            gap_ms: from.map(|from| from.epoch_ms() - link.at.epoch_ms()),
            candidates: link.candidates.len(),
            nearest_run: link.evidence == Some(LinkEvidence::NearestRun),
        }),
    }
}

/// Most notable calls a part keeps (G6).
const NOTABLE_MAX: usize = 3;

/// Counts and notable calls of the calls a part holds (G6, pinned G6 Merge).
/// `held` may be in any order; it is sorted by time, stably. Hand-offs are
/// never counted, and are notable only when `labels` names them (the chain's
/// own tagged hand-offs, keyed by invocation): their 8.2 label is not used.
/// A label seen twice keeps one entry at its latest position with the latest
/// `ok`; then the last 3 are kept.
fn tally(held: &[(Timestamp, &Call)], labels: &HashMap<&str, String>) -> (Counts, Vec<Notable>) {
    let (counts, notable) = summarize(held, labels);
    (
        counts,
        notable
            .into_iter()
            .map(|n| Notable {
                label: n.label,
                ok: n.ok,
            })
            .collect(),
    )
}

/// G6 summary shared by graphs and live cards. Latest occurrence supplies time and result.
pub fn summarize(
    held: &[(Timestamp, &Call)],
    labels: &HashMap<&str, String>,
) -> (Counts, Vec<multi_link_model::LiveNotable>) {
    let mut held = held.to_vec();
    held.sort_by_key(|(at, _)| *at);
    let mut counts = Counts::default();
    let mut notable: Vec<multi_link_model::LiveNotable> = Vec::new();
    for (_, call) in held {
        let count = match call.kind {
            CallKind::Read => &mut counts.reads,
            CallKind::Edit => &mut counts.edits,
            CallKind::Run => &mut counts.runs,
            CallKind::Handoff => {
                if let Some(label) = labels.get(call.id.as_str()) {
                    notable.retain(|n| &n.label != label);
                    notable.push(multi_link_model::LiveNotable {
                        at: call.at,
                        label: label.clone(),
                        ok: None,
                    });
                }
                continue;
            }
        };
        *count += 1;
        if let Some(label) = call.label.clone() {
            notable.retain(|n| n.label != label);
            notable.push(multi_link_model::LiveNotable {
                at: call.at,
                label,
                ok: call.ok,
            });
        }
    }
    let excess = notable.len().saturating_sub(NOTABLE_MAX);
    notable.drain(..excess);
    (counts, notable)
}

/// A run's interval `(from, to)`: first turn's start to last turn's end.
/// `None` for a run with no interval (unlinked, ambiguous, unavailable).
fn interval(run: &Run<'_>) -> Option<(Timestamp, Timestamp)> {
    Some((run.turns.first()?.start, turn_end(run.turns.last()?)))
}

/// True while the run's last turn has no `task_complete` (pinned G2 Open run).
fn is_open(run: &Run<'_>) -> bool {
    run.turns.last().is_some_and(|t| t.completed_at.is_none())
}

/// A listed Claude session's W1 bursts; none for an unknown id.
fn claude_bursts<'a>(input: &GraphInput<'a>, id: &str) -> &'a [Activity] {
    input
        .sessions
        .iter()
        .find(|s| s.id == id && s.harness == Harness::Claude)
        .map_or(&[][..], |s| s.activity.as_slice())
}

/// True when `link` is tagged with a step other than `id` (pinned G2
/// Previous and next: an untagged hand-off is neither).
fn other_step(link: &Link, id: &str) -> bool {
    link.step.as_deref().is_some_and(|step| step != id)
}

/// Chain start and end (G2, pinned G2 Previous and next).
/// Start: the parent burst holding the first hand-off, not before the end of
/// the previous run tagged another step (only when that run has an interval).
/// End: the parent's last burst that overlaps the time after the last
/// hand-off and starts before the next hand-off tagged another step, capped
/// at that hand-off. Either falls back to the hand-off itself. Also returns
/// that cap (the next hand-off tagged another step), when there is one.
fn bounds(
    input: &GraphInput<'_>,
    owned: &[&Link],
    id: &str,
    chain: &Chain<'_>,
) -> (Timestamp, Timestamp, Option<Timestamp>) {
    let first = chain.hops[0].link;
    let last = chain.hops[chain.hops.len() - 1].link;
    let bursts = claude_bursts(input, &first.parent);
    let same_parent_other = |l: &&&Link| l.parent == first.parent && other_step(l, id);

    let mut start = bursts
        .iter()
        .find(|b| b.from <= first.at && first.at <= b.to)
        .map_or(first.at, |b| b.from);
    let previous = owned
        .iter()
        .filter(same_parent_other)
        .rfind(|l| order(l) < order(first));
    if let Some(previous) = previous
        && let Some((_, to)) = interval(&run(input, owned, previous))
    {
        start = start.max(to);
    }

    let next = owned
        .iter()
        .filter(same_parent_other)
        .find(|l| order(l) > order(last))
        .map(|l| l.at);
    let end = bursts
        .iter()
        .filter(|b| b.to > last.at && next.is_none_or(|next| b.from < next))
        .map(|b| b.to)
        .next_back()
        .unwrap_or(last.at);
    (start, next.map_or(end, |next| end.min(next)), next)
}

/// A Claude part of `parent` over `[from, to]`: bursts ∩ part, one stretch per
/// burst that meets it, single records included (pinned Stretches).
fn claude_part(
    input: &GraphInput<'_>,
    chain: usize,
    kind: &str,
    parent: &str,
    (from, to): (Timestamp, Timestamp),
    edge_in: Option<Edge>,
) -> Part {
    let mut active_ms = 0;
    let mut stretches = 0;
    for burst in claude_bursts(input, parent) {
        let (a, b) = (burst.from.max(from), burst.to.min(to));
        if a <= b {
            active_ms += b.epoch_ms() - a.epoch_ms();
            stretches += 1;
        }
    }
    Part {
        chain,
        kind: kind.to_string(),
        harness: Harness::Claude,
        session: Some(parent.to_string()),
        from: Some(from),
        to: Some(to),
        status: None,
        run_available: None,
        active_ms,
        stretches,
        asked: None,
        said: None,
        counts: Counts::default(),
        notable: Vec::new(),
        waiting_reads: 0,
        untagged_handoffs: 0,
        edge_in,
    }
}

/// What `cut` tells `place` about one chain.
struct ChainCut {
    /// The time inside the chain `(start, end)` for calls in no part (pinned
    /// G6).
    inside: (Timestamp, Timestamp),
    /// Index in `parts` of the chain's last part, if it has one. It is named
    /// `review` here; `place` renames it `verify + commit` when it holds a
    /// `$ git commit` call (G3), with the same ownership as every call.
    last: Option<usize>,
}

/// Cut one chain into parts (G2 cuts, G3 names) and append them to `parts`:
/// `plan`, then per hop its Codex part and the Claude part after it. A Claude
/// part never starts before the chain start and is kept only when
/// `from < to`. The time inside the chain runs from chain start to chain end,
/// or, when there is no last part, to the latest end of the chain's runs if
/// that is later (Codex is still at work for the chain until then), but never
/// past the next hand-off tagged another step.
fn cut(
    input: &GraphInput<'_>,
    owned: &[&Link],
    id: &str,
    index: usize,
    chain: &Chain<'_>,
    parts: &mut Vec<Part>,
) -> ChainCut {
    let parent = chain.hops[0].link.parent.as_str();
    let (start, end, cap) = bounds(input, owned, id, chain);
    // Index in `parts` of the chain's latest Claude part so far: it holds or
    // precedes the next hand-off.
    let mut last_claude = None;
    let first_at = chain.hops[0].link.at;
    if start < first_at {
        last_claude = Some(parts.len());
        parts.push(claude_part(
            input,
            index,
            "plan",
            parent,
            (start, first_at),
            None,
        ));
    }
    let mut last = None;
    for (position, hop) in chain.hops.iter().enumerate() {
        parts.push(codex_part(input, index, position, hop, last_claude));
        let next = chain.hops.get(position + 1);
        let to = next.map_or(end, |h| h.link.at);
        let (from, edge_in) = match interval(&hop.run) {
            // No interval: Claude's time runs through the hand-off, but no
            // Claude part covers an earlier run of the chain still going at
            // it (G2), open runs included (their `to` is the last record).
            None => {
                let at = hop.link.at;
                match chain.hops[..position]
                    .iter()
                    .filter_map(|h| interval(&h.run))
                    .map(|(_, earlier_to)| earlier_to)
                    .filter(|&earlier_to| earlier_to > at)
                    .max()
                {
                    Some(run_to) => (run_to, Some(Edge::Return { at: run_to })),
                    None => (at, None),
                }
            }
            Some(_) if next.is_none() && is_open(&hop.run) => continue,
            // Same max rule as no interval: the part starts when the latest
            // earlier run of the chain still going at this run's end ends
            // (an open run's `to` is its last record). If such a run is
            // open, the chain still has no last part.
            Some((_, run_to)) => {
                let still_going: Vec<(Timestamp, bool)> = chain.hops[..position]
                    .iter()
                    .filter_map(|h| interval(&h.run).map(|(_, to)| (to, is_open(&h.run))))
                    .filter(|&(earlier_to, _)| earlier_to > run_to)
                    .collect();
                if next.is_none() && still_going.iter().any(|&(_, open)| open) {
                    continue;
                }
                let from = still_going
                    .iter()
                    .map(|&(earlier_to, _)| earlier_to)
                    .fold(run_to, Timestamp::max);
                (from, Some(Edge::Return { at: from }))
            }
        };
        // No Claude part starts before the chain start. When that moves
        // `from`, it lands on the end of another step's run, which is not
        // this chain's: no return edge, unless a run of this chain also ends
        // exactly there.
        let (from, edge_in) = if from < start {
            let returns = chain.hops[..=position]
                .iter()
                .filter_map(|h| interval(&h.run))
                .any(|(_, run_to)| run_to == start);
            (start, returns.then_some(Edge::Return { at: start }))
        } else {
            (from, edge_in)
        };
        if from >= to {
            continue;
        }
        last_claude = Some(parts.len());
        if next.is_none() {
            last = Some(parts.len());
        }
        parts.push(claude_part(
            input,
            index,
            "review",
            parent,
            (from, to),
            edge_in,
        ));
    }
    let inside_to = if last.is_some() {
        end
    } else {
        let latest = chain
            .hops
            .iter()
            .filter_map(|h| interval(&h.run))
            .map(|(_, run_to)| run_to)
            .fold(end, Timestamp::max);
        cap.map_or(latest, |cap| latest.min(cap))
    };
    ChainCut {
        inside: (start, inside_to),
        last,
    }
}

/// The hand-off label of a chain's hop at `position` (G6, pinned G3/G6 Per
/// chain): the chain's first hand-off is `step`, later ones `continue`.
fn handoff_label(id: &str, position: usize) -> String {
    match position {
        0 => format!("▸ codex exec · step {id}"),
        _ => format!("▸ codex exec · continue {id}"),
    }
}

/// True when the Claude part `part` holds `call` (pinned G6 Which part holds
/// a call): `from ≤ at ≤ to`, but a hand-off belongs to the part it ends
/// (`from < at ≤ to`).
fn holds(part: &Part, call: &Call) -> bool {
    let (Some(from), Some(to)) = (part.from, part.to) else {
        return false;
    };
    let after_from = match call.kind {
        CallKind::Handoff => from < call.at,
        _ => from <= call.at,
    };
    after_from && call.at <= to
}

/// Place the parents' owned calls in the Claude parts (G6): counts, notable
/// calls, `waiting_reads`, `untagged_handoffs` and the `verify + commit`
/// name of a last part (G3). `cuts` is each chain's result from `cut`. A call
/// held by two parts that meet at one instant goes to the earlier one.
fn place(
    input: &GraphInput<'_>,
    owned: &[&Link],
    id: &str,
    chains: &[Chain<'_>],
    cuts: &[ChainCut],
    parts: &mut [Part],
) {
    let claude: Vec<usize> = (0..parts.len())
        .filter(|&i| parts[i].harness == Harness::Claude)
        .collect();
    let mut held: Vec<Vec<(Timestamp, &Call)>> = vec![Vec::new(); parts.len()];
    let mut waiting = vec![0; parts.len()];
    // Unheld calls already placed, by `(parent, call id)`: two chains of one
    // parent whose times inside meet place a call once, in the earlier chain.
    let mut placed: HashSet<(&str, &str)> = HashSet::new();
    // Per chain: its parent's calls held by no part.
    for (index, chain) in chains.iter().enumerate() {
        let parent = chain.hops[0].link.parent.as_str();
        let (start, end) = cuts[index].inside;
        // Only the chain's own tagged hand-offs are notable.
        let labels: HashMap<&str, String> = chain
            .hops
            .iter()
            .enumerate()
            .map(|(position, hop)| (hop.link.invocation.as_str(), handoff_label(id, position)))
            .collect();
        let mine: Vec<usize> = claude
            .iter()
            .copied()
            .filter(|&i| parts[i].chain == index)
            .collect();
        for call in input.claude_calls.get(parent).into_iter().flatten() {
            let holder = claude
                .iter()
                .copied()
                .find(|&i| parts[i].session.as_deref() == Some(parent) && holds(&parts[i], call));
            if let Some(i) = holder {
                // Held calls are gathered once, from the holder's chain.
                if parts[i].chain == index {
                    held[i].push((call.at, call));
                }
                continue;
            }
            if call.at < start || call.at > end || !placed.insert((parent, call.id.as_str())) {
                continue;
            }
            // The last Claude part of this chain before the call.
            let Some(&before) = mine
                .iter()
                .rev()
                .find(|&&i| parts[i].to.is_some_and(|to| to <= call.at))
            else {
                continue;
            };
            match call.kind {
                CallKind::Read => waiting[before] += 1,
                CallKind::Handoff if labels.contains_key(call.id.as_str()) => {
                    held[before].push((call.at, call));
                }
                _ => {}
            }
        }
        for &i in &mine {
            let (counts, notable) = tally(&held[i], &labels);
            let commits = held[i]
                .iter()
                .any(|(_, c)| c.label.as_deref() == Some("$ git commit"));
            let part = &mut parts[i];
            part.counts = counts;
            part.notable = notable;
            part.waiting_reads = waiting[i];
            // Owned untagged hand-offs with `from < at ≤ to` (schema); one
            // held by no part counts nowhere.
            part.untagged_handoffs = owned
                .iter()
                .filter(|l| l.parent == parent && l.step.is_none())
                .filter(|l| part.from.is_some_and(|from| from < l.at))
                .filter(|l| part.to.is_some_and(|to| l.at <= to))
                .count();
            if cuts[index].last == Some(i) && commits {
                part.kind = "verify + commit".to_string();
            }
        }
    }
}

pub fn build(input: &GraphInput<'_>) -> GraphReport {
    let id = &input.step.id;
    let mut owned_links: Vec<&Link> = input
        .links
        .iter()
        .filter(|link| owned(link, input.repo, input.sessions, input.claude_calls))
        .collect();
    owned_links.sort_by(|a, b| order(a).cmp(&order(b)));
    let chains = chains(input, &owned_links, id);
    let mut parts = Vec::new();
    let cuts: Vec<ChainCut> = chains
        .iter()
        .enumerate()
        .map(|(index, chain)| cut(input, &owned_links, id, index, chain, &mut parts))
        .collect();
    place(input, &owned_links, id, &chains, &cuts, &mut parts);
    // A chain exists exactly when an owned hand-off has this step (G1).
    let empty = chains.is_empty().then(|| {
        format!("no hand-offs tagged step {id} (a Claude-only step, or its prompt did not name it)")
    });
    GraphReport {
        now: input.now,
        repo: input.repo.to_string(),
        step: input.step.clone(),
        commit: input.commit.clone(),
        parts,
        empty,
    }
}

/// Newest commit whose subject holds exactly `(Step <step>)` (a substring, so
/// `(Step 1.10)` never matches `1.1`). Runs `git` with `GIT_*` removed, as
/// `RepoMode::map` does; any failure is `None`.
pub fn commit_for_step(repo: &Path, step: &str) -> Option<GraphCommit> {
    let mut command = Command::new("git");
    for (name, _) in std::env::vars_os() {
        if name.as_encoded_bytes().starts_with(b"GIT_") {
            command.env_remove(name);
        }
    }
    let output = command
        .arg("-C")
        .arg(repo)
        .args(["log", "--abbrev=7", "--format=%h%x1f%cI%x1f%s"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = std::str::from_utf8(&output.stdout).ok()?;
    let marker = format!("(Step {step})");
    // The newest line whose subject matches decides; if its date does not
    // parse, the answer is `None`, never an older commit.
    let line = text.lines().find(|line| {
        line.splitn(3, '\u{1f}')
            .nth(2)
            .is_some_and(|subject| subject.contains(&marker))
    })?;
    let mut fields = line.splitn(3, '\u{1f}');
    let (hash, at) = (fields.next()?, fields.next()?);
    Some(GraphCommit {
        hash: hash.to_string(),
        at: Timestamp::parse(at).ok()?,
    })
}

/// Reads one Codex rollout's calls and turns, like `codex::read_details`.
#[doc(hidden)]
pub type ReadDetails<'a> = dyn FnMut(&Path) -> io::Result<(Calls, Vec<Turn>)> + 'a;

impl Scanner {
    /// Build the graph of one PLAN.md step. `plans` maps a plan folder to its
    /// steps report; the caller supplies it so core stays free of the plan parser.
    pub fn graph(
        &mut self,
        root: &Path,
        options: &Options<'_>,
        repo: &str,
        step: &str,
        plans: &mut dyn FnMut(&Path) -> RepoSteps,
    ) -> io::Result<GraphReport> {
        self.graph_with_reader(
            root,
            options,
            repo,
            step,
            plans,
            &mut multi_link_codex::read_details,
        )
    }

    /// `graph` with the Codex rollout reader injected, so a test can make a
    /// rollout vanish between the refresh and the read. Not public API.
    #[doc(hidden)]
    pub fn graph_with_reader(
        &mut self,
        root: &Path,
        options: &Options<'_>,
        repo: &str,
        step: &str,
        plans: &mut dyn FnMut(&Path) -> RepoSteps,
        read_details: &mut ReadDetails<'_>,
    ) -> io::Result<GraphReport> {
        // G2: report windows are `--since all`, whatever the caller's window.
        let all = Options {
            state_dir: options.state_dir.clone(),
            now: options.now,
            since: None,
            repo_mode: options.repo_mode,
            probe: options.probe,
        };
        let (sessions, links) = self.refresh(root, &all)?;
        let sessions = &sessions.report.sessions;
        let links = &links.report.links;

        let found = plan_folder(root, options.repo_mode, repo)
            .map(|folder| plans(&folder))
            .and_then(|steps| steps.steps.into_iter().find(|s| s.id == step));
        let header = GraphStep {
            id: step.to_string(),
            title: found.as_ref().map(|s| s.title.clone()),
            verify: found.and_then(|s| s.verify_command),
        };
        let commit = match options.repo_mode {
            RepoMode::Fixture => None,
            RepoMode::Real => commit_for_step(Path::new(repo), step),
        };

        // Candidate parents: listed Claude sessions in this repo that launched
        // a hand-off tagged with this step.
        let in_repo: HashMap<&str, &Session> = sessions
            .iter()
            .filter(|s| s.harness == Harness::Claude && s.repo == repo)
            .map(|s| (s.id.as_str(), s))
            .collect();
        let parents: HashSet<&str> = links
            .iter()
            .filter(|l| l.step.as_deref() == Some(step))
            .filter(|l| in_repo.contains_key(l.parent.as_str()))
            .map(|l| l.parent.as_str())
            .collect();

        let mut claude_calls: HashMap<String, Vec<Call>> = HashMap::new();
        if !parents.is_empty() {
            // Fail soft: a scan error means no transcripts, so no owned calls.
            // G11: reuse the scanner's file-key cache of parsed transcripts.
            let transcripts = self.claude_transcripts(root).unwrap_or_default();
            let load = |wanted: &HashSet<&str>, into: &mut HashMap<String, Vec<Call>>| {
                for transcript in transcripts
                    .iter()
                    .filter(|t| wanted.contains(t.id.as_str()))
                {
                    // An unreadable transcript means that parent owns no calls.
                    let calls = multi_link_claude::calls(root, transcript)
                        .map(|c| c.calls)
                        .unwrap_or_default();
                    into.insert(transcript.id.clone(), calls);
                }
            };
            load(&parents, &mut claude_calls);
            // G4: a run window ends at the next owned hand-off of the same
            // child from any parent, so also read the other in-repo parents
            // that launched one of these children (and only those).
            let children: HashSet<&str> = links
                .iter()
                .filter(|l| parents.contains(l.parent.as_str()))
                .filter(|l| owned(l, repo, sessions, &claude_calls))
                .filter_map(|l| l.child.as_deref())
                .collect();
            let others: HashSet<&str> = links
                .iter()
                .filter(|l| l.child.as_deref().is_some_and(|c| children.contains(c)))
                .map(|l| l.parent.as_str())
                .filter(|p| in_repo.contains_key(p) && !parents.contains(p))
                .collect();
            if !others.is_empty() {
                load(&others, &mut claude_calls);
            }
        }

        // Codex runs of every owned hand-off of a candidate parent, whatever its
        // step: the other steps' links are needed for the G2 caps. Other
        // parents' hand-offs share these children, so they need no reads.
        let by_id: HashMap<&str, &Session> = sessions
            .iter()
            .filter(|s| s.harness == Harness::Codex)
            .map(|s| (s.id.as_str(), s))
            .collect();
        let mut codex: HashMap<String, CodexDetails> = HashMap::new();
        for link in links {
            if !parents.contains(link.parent.as_str())
                || !owned(link, repo, sessions, &claude_calls)
            {
                continue;
            }
            let Some(child) = link.child.as_deref() else {
                continue;
            };
            let Some(path) = by_id.get(child).and_then(|s| s.transcript.as_deref()) else {
                continue;
            };
            if let Ok((calls, turns)) = read_details(&root.join(path)) {
                codex.insert(
                    child.to_string(),
                    CodexDetails {
                        calls: calls.calls,
                        turns,
                    },
                );
            }
        }

        Ok(build(&GraphInput {
            now: options.now,
            repo,
            step: header,
            commit,
            sessions,
            links,
            claude_calls: &claude_calls,
            codex: &codex,
        }))
    }
}
