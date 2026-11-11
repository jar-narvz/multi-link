//! Live cards and hand-off feed (L0–L5). `build` is pure; `Scanner::live` reads G5 details.
use std::{
    collections::{BTreeSet, HashMap},
    io,
    path::Path,
};

use crate::{CodexDetails, Options, Scanner};
use multi_link_model::{
    Call, CallKind, Card, CardLink, Counts, FeedRow, Harness, Link, LiveCounts, LiveGroup,
    LiveIdle, LiveNotable, LiveOutcome, LiveReport, Match, Session, State, Timestamp,
};

/// L1 card predicate; callers supply the windowed sessions pass.
pub fn is_card(s: &Session) -> bool {
    !s.imported && matches!(s.state, State::Waiting | State::Busy | State::Stale)
}

fn parent_links(links: &[Link]) -> HashMap<&str, usize> {
    let mut parents = HashMap::new();
    for (i, l) in links.iter().enumerate() {
        if !matches!(l.r#match, Match::Exact | Match::Heuristic) {
            continue;
        }
        if let Some(child) = l.child.as_deref() {
            let entry = parents.entry(child).or_insert(i);
            if (l.at, i) > (links[*entry].at, *entry) {
                *entry = i;
            }
        }
    }
    parents
}

fn outcome(
    i: usize,
    links: &[Link],
    sessions: &[Session],
    codex: &HashMap<String, CodexDetails>,
    parents: &HashMap<&str, usize>,
) -> (Option<LiveOutcome>, Option<Timestamp>) {
    let l = &links[i];
    if !matches!(l.r#match, Match::Exact | Match::Heuristic) {
        return (None, None);
    }
    let Some(child) = l.child.as_deref() else {
        return (None, None);
    };
    let Some(s) = sessions.iter().find(|s| s.id == child) else {
        return (None, None);
    };
    if s.state == State::Busy && parents.get(child) == Some(&i) {
        return (Some(LiveOutcome::Running), None);
    }
    let end = codex
        .get(child)
        .and_then(|c| {
            c.turns
                .iter()
                .filter(|t| t.start >= l.at && t.completed_at.is_some())
                .min_by_key(|t| t.start)
        })
        .and_then(|t| t.completed_at);
    (end.map(|_| LiveOutcome::Done), end)
}

fn summarize_calls(
    s: &Session,
    calls: &HashMap<String, Vec<Call>>,
    links: &[Link],
) -> (Counts, Vec<LiveNotable>) {
    let owned = calls.get(&s.id).map(Vec::as_slice).unwrap_or_default();
    let labels: HashMap<&str, String> = owned
        .iter()
        .filter(|c| c.kind == CallKind::Handoff)
        .map(|c| {
            let step = links
                .iter()
                .find(|l| l.parent == s.id && l.invocation == c.id)
                .and_then(|l| l.step.as_deref());
            (
                c.id.as_str(),
                format!(
                    "▸ codex exec{}",
                    step.map(|s| format!(" · step {s}")).unwrap_or_default()
                ),
            )
        })
        .collect();
    let held = owned.iter().map(|c| (c.at, c)).collect::<Vec<_>>();
    crate::graph::summarize(&held, &labels)
}

fn group_cards(
    cards: &[Card],
    sessions: &[Session],
    links: &[Link],
    parents: &HashMap<&str, usize>,
) -> Vec<LiveGroup> {
    let mut groups: Vec<LiveGroup> = Vec::new();
    for c in cards {
        let grouped = c.harness == Harness::Codex
            && parents.get(c.id.as_str()).is_some_and(|&i| {
                cards
                    .iter()
                    .any(|p| p.id == links[i].parent && p.harness == Harness::Claude)
            });
        if !grouped {
            groups.push(LiveGroup {
                head: c.clone(),
                children: Vec::new(),
            });
        }
    }
    for c in cards.iter().filter(|c| c.harness == Harness::Codex) {
        if let Some(&i) = parents.get(c.id.as_str())
            && let Some(g) = groups
                .iter_mut()
                .find(|g| g.head.harness == Harness::Claude && g.head.id == links[i].parent)
        {
            g.children.push(c.clone());
        }
    }

    for g in &mut groups {
        g.children
            .sort_by_key(|c| (links[parents[c.id.as_str()]].at, c.id.clone()));
        if let Some(c) = g.children.last() {
            g.head.link = Some(CardLink::WaitingOn {
                child: c.id.clone(),
                at: links[parents[c.id.as_str()]].at,
            });
        }
    }
    let last_activity: HashMap<&str, Timestamp> = sessions
        .iter()
        .map(|s| (s.id.as_str(), s.last_activity))
        .collect();
    groups.sort_by_key(|g| {
        (
            match g.head.state {
                State::Waiting => 0,
                State::Busy => 1,
                _ => 2,
            },
            g.head
                .since
                .or_else(|| last_activity.get(g.head.id.as_str()).copied()),
            g.head.id.clone(),
        )
    });
    groups
}

/// Build from listed sessions/links and owned, non-imported G5 calls and turns.
pub fn build(
    now: Timestamp,
    window_from: Option<Timestamp>,
    sessions: &[Session],
    links: &[Link],
    calls: &HashMap<String, Vec<Call>>,
    codex: &HashMap<String, CodexDetails>,
) -> LiveReport {
    let parents = parent_links(links);
    let steps = codex_steps(sessions, links);
    let in_window = |l: &&Link| l.at <= now && window_from.is_none_or(|from| l.at >= from);
    let latest = |id: &str, tagged: bool, windowed: bool| {
        links
            .iter()
            .enumerate()
            .filter(|(_, l)| {
                l.parent == id && (!tagged || l.step.is_some()) && (!windowed || in_window(l))
            })
            .max_by_key(|(i, l)| (l.at, *i))
    };
    let make_card = |s: &Session| {
        let link = if s.harness == Harness::Codex {
            Some(
                parents
                    .get(s.id.as_str())
                    .map(|&i| {
                        let l = &links[i];
                        CardLink::From {
                            parent: l.parent.clone(),
                            r#match: l.r#match,
                            at: l.at,
                        }
                    })
                    .unwrap_or(CardLink::None),
            )
        } else {
            latest(&s.id, false, true).map(|(i, l)| {
                let (outcome, outcome_at) = outcome(i, links, sessions, codex, &parents);
                CardLink::Handoff {
                    at: l.at,
                    r#match: l.r#match,
                    child: l.child.clone(),
                    candidates: l.candidates.len(),
                    outcome,
                    outcome_at,
                }
            })
        };
        let (counts, notable) = summarize_calls(s, calls, links);
        Card {
            id: s.id.clone(),
            harness: s.harness,
            state: s.state,
            repo: s.repo.clone(),
            checkout: s.checkout.clone(),
            started_at: s.started_at,
            activity: s.activity.clone(),
            last_asked: s.last_asked.clone(),
            step: if s.harness == Harness::Codex {
                steps.get(&s.id).cloned()
            } else {
                latest(&s.id, true, false).and_then(|(_, l)| l.step.clone())
            },
            since: if s.state == State::Busy {
                s.activity.last().map(|a| a.from)
            } else {
                s.state_since.or(Some(s.last_activity))
            },
            counts,
            notable,
            link,
        }
    };
    let cards: Vec<_> = sessions
        .iter()
        .filter(|s| is_card(s))
        .map(make_card)
        .collect();
    let groups = group_cards(&cards, sessions, links, &parents);
    let idle: Vec<_> = sessions
        .iter()
        .filter(|s| !s.imported && !is_card(s) && !s.activity.is_empty())
        .collect();
    let mut feed: Vec<_> = links
        .iter()
        .enumerate()
        .filter(|(_, l)| in_window(l))
        .collect();
    feed.sort_by_key(|(i, l)| std::cmp::Reverse((l.at, *i)));
    let feed = feed
        .into_iter()
        .take(4)
        .map(|(i, l)| {
            let (outcome, outcome_at) = outcome(i, links, sessions, codex, &parents);
            FeedRow {
                at: l.at,
                parent: l.parent.clone(),
                invocation: l.invocation.clone(),
                form: l.form,
                step: l.step.clone(),
                r#match: l.r#match,
                child: l.child.clone(),
                repo: sessions
                    .iter()
                    .find(|s| s.id == l.parent)
                    .map(|s| s.repo.clone())
                    .unwrap_or_else(|| l.checkout.clone()),
                candidates: l.candidates.len(),
                outcome,
                outcome_at,
            }
        })
        .collect();
    LiveReport {
        now,
        window_from,
        counts: LiveCounts {
            waiting: cards.iter().filter(|c| c.state == State::Waiting).count(),
            busy: cards.iter().filter(|c| c.state == State::Busy).count(),
            stale: cards.iter().filter(|c| c.state == State::Stale).count(),
        },
        groups,
        idle: LiveIdle {
            count: idle.len(),
            repos: idle
                .iter()
                .map(|s| s.repo.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
        },
        last_handoff: links.iter().map(|l| l.at).max(),
        feed,
    }
}

fn turn_children(links: &[Link], window_from: Option<Timestamp>, now: Timestamp) -> BTreeSet<&str> {
    links
        .iter()
        .filter(|l| window_from.is_none_or(|from| l.at >= from && l.at <= now))
        .filter_map(|l| l.child.as_deref())
        .collect()
}

impl Scanner {
    /// Read calls for cards, plus turns for windowed linked children. Readers exclude imports.
    pub fn live(&mut self, root: &Path, options: &Options<'_>) -> io::Result<LiveReport> {
        let (sessions, links) = self.refresh(root, options)?;
        let sessions = &sessions.report.sessions;
        let links = &links.report.links;
        let window_from = options
            .since
            .map(|d| {
                Timestamp::from_epoch_ms(
                    options
                        .now
                        .epoch_ms()
                        .saturating_sub(i64::try_from(d.as_millis()).unwrap_or(i64::MAX)),
                )
            })
            .transpose()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        let turn_children = turn_children(links, window_from, options.now);
        let mut calls = HashMap::new();
        let mut codex = HashMap::new();
        let transcripts = if sessions
            .iter()
            .any(|s| is_card(s) && s.harness == Harness::Claude)
        {
            self.claude_transcripts(root).unwrap_or_default()
        } else {
            Vec::new()
        };
        for s in sessions {
            if s.harness == Harness::Claude && is_card(s) {
                if let Some(t) = transcripts.iter().find(|t| t.id == s.id) {
                    calls.insert(
                        s.id.clone(),
                        multi_link_claude::calls(root, t)
                            .map(|c| c.calls)
                            .unwrap_or_default(),
                    );
                }
            } else if s.harness == Harness::Codex
                && let Some(path) = &s.transcript
            {
                let turns = if is_card(s) {
                    multi_link_codex::read_details(&root.join(path)).map(|(c, turns)| {
                        calls.insert(s.id.clone(), c.calls);
                        turns
                    })
                } else if turn_children.contains(s.id.as_str()) {
                    multi_link_codex::codex_turns(&root.join(path))
                } else {
                    continue;
                };
                if let Ok(turns) = turns {
                    codex.insert(
                        s.id.clone(),
                        CodexDetails {
                            calls: Vec::new(),
                            turns,
                        },
                    );
                }
            }
        }
        Ok(build(
            options.now,
            window_from,
            sessions,
            links,
            &calls,
            &codex,
        ))
    }
}
/// Latest tagged incoming hand-off per Codex session (R6), using all-time sessions to
/// require a parent in the child's repo. Max `at` wins, ties by later link position.
pub fn codex_steps(sessions: &[Session], links: &[Link]) -> HashMap<String, String> {
    let by_id: HashMap<&str, &Session> = sessions.iter().map(|s| (s.id.as_str(), s)).collect();
    let mut latest: HashMap<&str, (Timestamp, usize, &str)> = HashMap::new();
    for (i, link) in links.iter().enumerate() {
        let (Some(child), Some(step)) = (link.child.as_deref(), link.step.as_deref()) else {
            continue;
        };
        let (Some(child_session), Some(parent)) =
            (by_id.get(child), by_id.get(link.parent.as_str()))
        else {
            continue;
        };
        if child_session.harness != Harness::Codex || parent.repo != child_session.repo {
            continue;
        }
        let entry = latest.entry(child).or_insert((link.at, i, step));
        if (link.at, i) > (entry.0, entry.1) {
            *entry = (link.at, i, step);
        }
    }
    latest
        .into_iter()
        .map(|(id, (_, _, step))| (id.to_string(), step.to_string()))
        .collect()
}

/// Steps a Claude session's direct started children reached (G9 amendment), keyed by the
/// parent's id. A child is a listed Claude session whose `started_by` is a listed Claude session
/// of the same repo (8.4c S3: cross-repo children are not children). Each child gives its own
/// latest tagged hand-off (max `at`, ties by later position) as (child repo, step); children in
/// `started_at` then id order; one entry per (repo, step), the first kept; untagged children
/// skipped. Direct children only.
pub fn started_steps(
    sessions: &[Session],
    links: &[Link],
) -> HashMap<String, Vec<(String, String)>> {
    let claude: HashMap<&str, &Session> = sessions
        .iter()
        .filter(|s| s.harness == Harness::Claude)
        .map(|s| (s.id.as_str(), s))
        .collect();
    let mut latest: HashMap<&str, (Timestamp, usize, &str)> = HashMap::new();
    for (i, link) in links.iter().enumerate() {
        if let Some(step) = link.step.as_deref() {
            let entry = latest
                .entry(link.parent.as_str())
                .or_insert((link.at, i, step));
            if (link.at, i) > (entry.0, entry.1) {
                *entry = (link.at, i, step);
            }
        }
    }
    let mut children: Vec<(&str, &Session)> = claude
        .values()
        .filter_map(|child| {
            let parent = claude.get(child.started_by.as_deref()?)?;
            (parent.repo == child.repo).then_some((parent.id.as_str(), *child))
        })
        .collect();
    children.sort_by(|a, b| (a.1.started_at, &a.1.id).cmp(&(b.1.started_at, &b.1.id)));
    let mut map: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for (parent, child) in children {
        let Some((_, _, step)) = latest.get(child.id.as_str()) else {
            continue;
        };
        let entry = (child.repo.clone(), step.to_string());
        let entries = map.entry(parent.to_string()).or_default();
        if !entries.contains(&entry) {
            entries.push(entry);
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turn_children_include_window_boundaries_and_all_time_links() {
        let link = |child: &str, at: &str| Link {
            parent: "p".into(),
            invocation: child.into(),
            at: at.parse().unwrap(),
            child: Some(child.into()),
            r#match: Match::Exact,
            form: multi_link_model::InvocationForm::Exec,
            background: false,
            checkout: "repo".into(),
            launch_dir: None,
            subagent: None,
            step: None,
            evidence: None,
            candidates: Vec::new(),
        };
        let links = [
            link("old", "2026-11-10T19:59:59Z"),
            link("start", "2026-11-10T20:00:00Z"),
            link("end", "2026-11-10T22:00:00Z"),
            link("future", "2026-11-10T22:00:01Z"),
        ];
        let now = "2026-11-10T22:00:00Z".parse().unwrap();
        let from = "2026-11-10T20:00:00Z".parse().unwrap();
        assert_eq!(
            turn_children(&links, Some(from), now),
            BTreeSet::from(["start", "end"])
        );
        assert_eq!(
            turn_children(&links, None, now),
            BTreeSet::from(["old", "start", "end", "future"])
        );
    }
}
