//! Resolve observed child ids before considering bounded heuristic candidates.

use std::{
    collections::{HashMap, HashSet},
    io,
    path::Path,
};

use multi_link_claude::Invocation;
use multi_link_codex::Rollout;
use multi_link_model::{Candidate, InvocationForm, Link, LinkEvidence, Match, Timestamp};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::RepoMode;

pub(crate) fn resolve(
    root: &Path,
    mode: RepoMode,
    invocations: &[Invocation],
    rollouts: &[Rollout],
    hooks: &crate::hook::Records,
    read_output: &mut impl FnMut(&Path) -> Option<String>,
) -> io::Result<(Vec<Link>, usize)> {
    let mut links = Vec::new();
    let mut hook_conflicts = 0;
    for invocation in invocations {
        let mut exact = exact_evidence(root, mode, invocation, read_output)?;
        let children: HashSet<_> = hooks
            .records
            .iter()
            .filter(|record| {
                record.session_id == invocation.parent
                    && record.tool_use_id.as_deref() == Some(invocation.invocation.as_str())
            })
            .filter_map(|record| record.child.as_ref())
            .collect();
        if children.len() > 1 {
            hook_conflicts += 1;
        } else if let Some(child) = children.into_iter().next() {
            if let Some((id, _)) = &exact {
                if id != child {
                    hook_conflicts += 1;
                }
            } else {
                exact = Some((child.clone(), LinkEvidence::Hook));
            }
        }
        links.push(Link {
            step: invocation.step.clone(),
            parent: invocation.parent.clone(),
            invocation: invocation.invocation.clone(),
            subagent: invocation.subagent.clone(),
            form: invocation.form,
            background: invocation.background,
            at: invocation.at,
            checkout: invocation.checkout.clone(),
            r#match: if exact.is_some() {
                Match::Exact
            } else {
                Match::Unlinked
            },
            child: exact.as_ref().map(|(id, _)| id.clone()),
            evidence: exact.map(|(_, evidence)| evidence),
            candidates: Vec::new(),
        });
    }
    let claimed: HashSet<_> = links
        .iter()
        .filter(|link| link.form != InvocationForm::Resume)
        .filter_map(|link| link.child.clone())
        .collect();
    for (link, invocation) in links.iter_mut().zip(invocations) {
        if link.r#match == Match::Exact || invocation.dir_moved {
            continue;
        }
        // An id-less resume is recognised only for exec resume --last. It
        // searches later real turns even in runs claimed by an exact launch.
        let resume_last =
            invocation.form == InvocationForm::Resume && invocation.resume_id.is_none();
        link.candidates = rollouts
            .iter()
            .filter(|child| child.cwd == invocation.checkout)
            .filter_map(|child| {
                let gap_ms = if resume_last {
                    child
                        .real_turns()
                        .skip(1)
                        .map(|turn| gap_ms(turn.started_at, invocation.at))
                        .filter(|gap| (0..=10_000).contains(gap))
                        .min()?
                } else {
                    if claimed.contains(&child.id) {
                        return None;
                    }
                    let gap = gap_ms(child.started_at, invocation.at);
                    if !(0..=60_000).contains(&gap) {
                        return None;
                    }
                    gap
                };
                Some(Candidate {
                    id: child.id.clone(),
                    gap_ms,
                })
            })
            .collect();
        link.candidates
            .sort_by(|a, b| (a.gap_ms, &a.id).cmp(&(b.gap_ms, &b.id)));
    }
    // K1 compares only original choices. Keep the smallest gap per checkout/run;
    // ties (including a hand-off's own gap) never remove a candidate.
    let mut choices = HashMap::new();
    for (link, invocation) in links.iter().zip(invocations) {
        if creation_competitor(link, invocation)
            && let Some(candidate) = nearest(&link.candidates)
        {
            choices
                .entry((link.checkout.clone(), candidate.id.clone()))
                .and_modify(|gap: &mut i64| *gap = (*gap).min(candidate.gap_ms))
                .or_insert(candidate.gap_ms);
        }
    }
    for (link, invocation) in links.iter_mut().zip(invocations) {
        if link.r#match == Match::Exact || invocation.dir_moved {
            continue;
        }
        let original_count = link.candidates.len();
        let creation = creation_competitor(link, invocation);
        if creation {
            link.candidates.retain(|candidate| {
                choices
                    .get(&(link.checkout.clone(), candidate.id.clone()))
                    .is_none_or(|gap| *gap >= candidate.gap_ms)
            });
        }
        let chosen = if creation {
            nearest(&link.candidates)
        } else {
            // R7 remains single-candidate only; K2 never resolves resume turns.
            match link.candidates.as_slice() {
                [candidate] => Some(candidate),
                _ => None,
            }
        };
        if let Some(candidate) = chosen {
            link.child = Some(candidate.id.clone());
            link.r#match = Match::Heuristic;
            link.evidence = if !creation {
                Some(LinkEvidence::ResumeTurn)
            } else if original_count > 1 {
                Some(LinkEvidence::NearestRun)
            } else {
                None
            };
        } else {
            link.r#match = if link.candidates.is_empty() {
                Match::Unlinked
            } else {
                Match::Ambiguous
            };
        }
    }
    links.sort_by(|a, b| (a.at, &a.invocation).cmp(&(b.at, &b.invocation)));
    Ok((links, hook_conflicts))
}

fn creation_competitor(link: &Link, invocation: &Invocation) -> bool {
    link.r#match != Match::Exact
        && !invocation.dir_moved
        && invocation.resume_id.is_none()
        && invocation.form != InvocationForm::Resume
}

/// Candidates are sorted by gap, then id. K2's margins are inclusive.
fn nearest(candidates: &[Candidate]) -> Option<&Candidate> {
    match candidates {
        [candidate] => Some(candidate),
        [first, rest @ ..] if first.gap_ms <= 5_000 && rest.iter().all(|c| c.gap_ms >= 10_000) => {
            Some(first)
        }
        _ => None,
    }
}

fn gap_ms(later: Timestamp, earlier: Timestamp) -> i64 {
    let instant = |stamp: Timestamp| {
        OffsetDateTime::parse(&stamp.to_string(), &Rfc3339)
            .expect("Timestamp always formats as RFC3339")
    };
    (instant(later) - instant(earlier)).whole_milliseconds() as i64
}

fn exact_evidence(
    root: &Path,
    mode: RepoMode,
    invocation: &Invocation,
    read_output: &mut impl FnMut(&Path) -> Option<String>,
) -> io::Result<Option<(String, LinkEvidence)>> {
    if let Some(id) = &invocation.resume_id {
        return Ok(Some((id.clone(), LinkEvidence::ResumeArg)));
    }
    let text = invocation.result_text.as_deref().unwrap_or_default();
    if let Some(id) = session_id(text) {
        return Ok(Some((id, LinkEvidence::Stdout)));
    }
    for line in text.lines() {
        if let Some(rest) = line.trim().strip_prefix("[codex] Thread ready (")
            && let Some((id, _)) = rest.split_once(')')
            && uuid_shaped(id)
        {
            return Ok(Some((id.to_owned(), LinkEvidence::ThreadReady)));
        }
    }
    for line in text.lines() {
        let Some(named) = line.trim().strip_prefix("Output is being written to: ") else {
            continue;
        };
        let named = Path::new(named.trim());
        if named
            .extension()
            .is_none_or(|extension| extension != "output")
        {
            continue;
        }
        let path = match mode {
            RepoMode::Real => named.to_path_buf(),
            RepoMode::Fixture => {
                let Some(name) = named.file_name() else {
                    continue;
                };
                root.join("task-output").join(name)
            }
        };
        let Some(output) = read_output(&path) else {
            continue;
        };
        if let Some(id) = session_id(&output) {
            return Ok(Some((id, LinkEvidence::TaskOutput)));
        }
    }
    if let Some(launch) = multi_link_claude::recognise(&invocation.command)
        && let Ok(Some(id)) = launch.redirect_child(
            Path::new(&invocation.checkout),
            (mode == RepoMode::Fixture).then_some(root),
        )
    {
        return Ok(Some((id, LinkEvidence::RedirectFile)));
    }
    Ok(None)
}

fn session_id(text: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let id = line.trim().strip_prefix("session id: ")?.trim();
        uuid_shaped(id).then(|| id.to_owned())
    })
}

fn uuid_shaped(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(|(index, byte)| {
            if [8, 13, 18, 23].contains(&index) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn stamp(ms: i64) -> Timestamp {
        Timestamp::from_epoch_ms(ms).unwrap()
    }

    fn launch(id: &str, ms: i64) -> Invocation {
        Invocation {
            step: None,
            parent: id.into(),
            invocation: id.into(),
            subagent: None,
            form: InvocationForm::Exec,
            background: false,
            at: stamp(ms),
            checkout: "/repo".into(),
            command: "codex exec x".into(),
            resume_id: None,
            dir_moved: false,
            result_text: None,
        }
    }

    fn run(id: &str, ms: i64) -> Rollout {
        Rollout {
            id: id.into(),
            transcript: String::new(),
            cwd: "/repo".into(),
            branch: None,
            started_at: stamp(ms),
            last_activity: stamp(ms),
            turns: vec![],
            activity_timestamps: vec![],
            asked: None,
            last_asked: None,
            imported: false,
            last_real_complete: None,
        }
    }

    fn resolve_calls(calls: &[Invocation], runs: &[Rollout]) -> Vec<Link> {
        resolve(
            Path::new("/unused"),
            RepoMode::Fixture,
            calls,
            runs,
            &crate::hook::Records::default(),
            &mut |_| None,
        )
        .unwrap()
        .0
    }

    #[test]
    fn nearest_run_boundaries_and_original_candidate_evidence() {
        for (first, second, chosen) in [
            (4999, 10000, true),
            (5000, 10000, true),
            (5001, 10001, false),
            (1000, 9999, false),
            (1000, 10001, true),
            (1000, 7000, false),
            (6000, 30000, false),
            (0, 0, false),
        ] {
            let links = resolve_calls(&[launch("a", 0)], &[run("r", first), run("s", second)]);
            let link = &links[0];
            assert_eq!(
                link.candidates,
                vec![
                    Candidate {
                        id: "r".into(),
                        gap_ms: first
                    },
                    Candidate {
                        id: "s".into(),
                        gap_ms: second
                    }
                ]
            );
            assert_eq!(
                link.r#match,
                if chosen {
                    Match::Heuristic
                } else {
                    Match::Ambiguous
                }
            );
            assert_eq!(link.child.as_deref(), chosen.then_some("r"));
            assert_eq!(
                serde_json::to_value(link.evidence).unwrap(),
                if chosen {
                    json!("nearest_run")
                } else {
                    json!(null)
                }
            );
        }
        let links = resolve_calls(&[launch("a", 0)], &[run("r", 1000)]);
        assert_eq!(links[0].r#match, Match::Heuristic);
        assert_eq!(links[0].evidence, None);
        let links = resolve_calls(&[launch("a", 0)], &[]);
        assert_eq!(links[0].r#match, Match::Unlinked);
        assert_eq!(links[0].evidence, None);
    }

    #[test]
    fn competitors_use_original_choices_once_independent_of_order() {
        // B originally chooses nothing, C chooses t. B then chooses s, but
        // that new choice must not remove s from A through a second pass.
        let calls = [launch("a", 0), launch("b", 20000), launch("c", 25000)];
        let runs = [run("r", 1000), run("s", 21000), run("t", 26000)];
        let links = resolve_calls(&calls, &runs);
        assert_eq!(
            links[0]
                .candidates
                .iter()
                .map(|c| c.id.as_str())
                .collect::<Vec<_>>(),
            vec!["r", "s"]
        );
        assert_eq!(links[0].child.as_deref(), Some("r"));
        assert_eq!(links[1].child.as_deref(), Some("s"));
        assert_eq!(links[2].child.as_deref(), Some("t"));
        assert_eq!(
            resolve_calls(
                &[calls[2].clone(), calls[0].clone(), calls[1].clone()],
                &runs
            ),
            links
        );
        // Without C, ambiguous B chooses nothing and removes neither s nor t.
        let links = resolve_calls(&calls[..2], &runs);
        assert_eq!(links[0].candidates.len(), 3);
        assert_eq!(links[1].r#match, Match::Ambiguous);
        assert_eq!(links[1].child, None);
        // B chooses a different run: even a smaller gap to t cannot remove t.
        let links = resolve_calls(
            &calls[..2],
            &[run("r", 1000), run("s", 21000), run("t", 50000)],
        );
        assert_eq!(
            links[0]
                .candidates
                .iter()
                .map(|c| c.id.as_str())
                .collect::<Vec<_>>(),
            vec!["r", "t"]
        );
    }

    #[test]
    fn competitors_keep_equal_gaps_and_ignore_other_checkouts_moves_and_resumes() {
        for runs in [vec![run("r", 1000)], vec![run("r", 1000), run("s", 30000)]] {
            let links = resolve_calls(&[launch("a", 0), launch("b", 0)], &runs);
            assert_eq!(links[0].child.as_deref(), Some("r"));
            assert_eq!(links[1].child.as_deref(), Some("r"));
            assert_eq!(links[0].candidates, links[1].candidates);
        }
        for kind in ["checkout", "move", "resume"] {
            let mut other = launch("b", 20000);
            match kind {
                "checkout" => other.checkout = "/other".into(),
                "move" => other.dir_moved = true,
                _ => other.form = InvocationForm::Resume,
            }
            let links = resolve_calls(&[launch("a", 0), other], &[run("r", 21000)]);
            assert_eq!(links[0].child.as_deref(), Some("r"), "{kind}");
            assert_eq!(links[0].evidence, None);
        }
        let links = resolve_calls(&[launch("a", 0), launch("b", 20000)], &[run("r", 21000)]);
        assert_eq!(links[0].r#match, Match::Unlinked);
        assert!(links[0].candidates.is_empty());
        assert_eq!(links[0].child, None);
        assert_eq!(links[0].evidence, None);
        assert_eq!(links[1].child.as_deref(), Some("r"));
        assert_eq!(links[1].evidence, None);
    }
    #[test]
    fn resume_turns_do_not_use_nearest_run_margins_or_creation_competition() {
        let mut resume = launch("resume", 0);
        resume.form = InvocationForm::Resume;
        resume.command = "codex exec resume --last x".into();
        let mut runs = [run("r", 1000), run("s", 30000)];
        for (run, gap) in runs.iter_mut().zip([1000, 10000]) {
            run.turns = vec![
                multi_link_codex::Turn {
                    turn_id: "first".into(),
                    started_at: stamp(-60000),
                    completed_at: None,
                },
                multi_link_codex::Turn {
                    turn_id: "later".into(),
                    started_at: stamp(gap),
                    completed_at: None,
                },
            ];
        }
        let links = resolve_calls(&[resume, launch("launch", 500)], &runs);
        let resumed = &links[0];
        assert_eq!(resumed.r#match, Match::Ambiguous);
        assert_eq!(resumed.child, None);
        assert_eq!(resumed.evidence, None);
        assert_eq!(
            resumed
                .candidates
                .iter()
                .map(|c| c.gap_ms)
                .collect::<Vec<_>>(),
            vec![1000, 10000]
        );
        assert_eq!(links[1].child.as_deref(), Some("r"));
    }
}
