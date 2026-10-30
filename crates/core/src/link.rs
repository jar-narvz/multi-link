//! Resolve observed child ids before considering bounded heuristic candidates.

use std::{collections::HashSet, fs, io, path::Path};

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
) -> io::Result<Vec<Link>> {
    let mut links = Vec::new();
    for invocation in invocations {
        let exact = exact_evidence(root, mode, invocation)?;
        links.push(Link {
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
        link.candidates = rollouts
            .iter()
            .filter(|child| child.cwd == invocation.checkout && !claimed.contains(&child.id))
            .filter_map(|child| {
                let gap_ms = gap_ms(child.started_at, invocation.at);
                (0..=60_000).contains(&gap_ms).then(|| Candidate {
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
                Match::Heuristic
            }
            _ => Match::Ambiguous,
        };
    }
    links.sort_by(|a, b| (a.at, &a.invocation).cmp(&(b.at, &b.invocation)));
    Ok(links)
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
        if !fs::metadata(&path).is_ok_and(|metadata| metadata.is_file()) {
            continue;
        }
        let output = match fs::read_to_string(path) {
            Ok(output) => output,
            Err(_) => continue,
        };
        if let Some(id) = session_id(&output) {
            return Ok(Some((id, LinkEvidence::TaskOutput)));
        }
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
