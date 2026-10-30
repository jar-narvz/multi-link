//! Links use all evidence; the history window filters only the assembled output.

use std::{collections::HashSet, io, path::Path};

use multi_link_codex::ReadFailure;
use multi_link_model::{Harness, LinksReport};

use crate::{Options, link, sessions_report};

#[derive(Debug)]
pub struct LinksOutput {
    /// Count of conflicting invocations, excluded from the JSON report.
    pub hook_conflicts: usize,
    pub report: LinksReport,
    pub bad_records: Vec<String>,
    pub incomplete: Vec<String>,
    pub unreadable: Vec<ReadFailure>,
}

/// Resolve all hand-offs, then retain links with a listed parent or child.
/// RepoMode::Fixture also selects task-output lookup by basename under root.
pub fn links_report(root: &Path, options: &Options<'_>) -> io::Result<LinksOutput> {
    let sessions = sessions_report(root, options)?;
    let claude = multi_link_claude::scan(root)?;
    let invocations = multi_link_claude::invocations(root, &claude)?;
    let codex = multi_link_codex::discover(root)?;
    let hooks = crate::hook::read(root, options)?;
    let (mut links, hook_conflicts) = link::resolve(
        root,
        options.repo_mode,
        &invocations,
        &codex.rollouts,
        &hooks,
    )?;
    let parented: HashSet<_> = links
        .iter()
        .filter_map(|link| link.child.as_deref())
        .collect();
    let mut unparented: Vec<_> = sessions
        .report
        .sessions
        .iter()
        .filter(|session| {
            session.harness == Harness::Codex && !parented.contains(session.id.as_str())
        })
        .map(|session| session.id.clone())
        .collect();
    unparented.sort();
    let listed: HashSet<_> = sessions
        .report
        .sessions
        .iter()
        .map(|session| session.id.as_str())
        .collect();
    links.retain(|link| {
        listed.contains(link.parent.as_str())
            || link
                .child
                .as_deref()
                .is_some_and(|child| listed.contains(child))
    });
    Ok(LinksOutput {
        hook_conflicts,
        report: LinksReport { links, unparented },
        bad_records: sessions.bad_records,
        incomplete: sessions.incomplete,
        unreadable: sessions.unreadable,
    })
}
