//! Resolve observed child ids before considering bounded heuristic candidates.

use std::{collections::HashSet, io, path::Path};

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
        link.r#match = match link.candidates.as_slice() {
            [] => Match::Unlinked,
            [candidate] => {
                link.child = Some(candidate.id.clone());
                if resume_last {
                    link.evidence = Some(LinkEvidence::ResumeTurn);
                }
                Match::Heuristic
            }
            _ => Match::Ambiguous,
        };
    }
    links.sort_by(|a, b| (a.at, &a.invocation).cmp(&(b.at, &b.invocation)));
    Ok((links, hook_conflicts))
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
