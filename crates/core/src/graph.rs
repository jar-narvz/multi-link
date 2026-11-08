//! The step graph report (design G0-G11). `build` is pure; `Scanner::graph`
//! does the I/O and hands `build` everything it needs.

use std::{
    collections::{HashMap, HashSet},
    io,
    path::Path,
};

use multi_link_model::{
    Call, GraphCommit, GraphReport, GraphStep, Harness, Link, RepoSteps, Session, Timestamp, Turn,
};

use crate::{Options, RepoMode, Scanner, overview::plan_folder};

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

pub fn build(input: &GraphInput<'_>) -> GraphReport {
    let id = &input.step.id;
    let tagged = input
        .links
        .iter()
        .filter(|link| owned(link, input.repo, input.sessions, input.claude_calls))
        .filter(|link| link.step.as_deref() == Some(id.as_str()))
        .count();
    let empty = (tagged == 0).then(|| {
        format!("no hand-offs tagged step {id} (a Claude-only step, or its prompt did not name it)")
    });
    GraphReport {
        now: input.now,
        repo: input.repo.to_string(),
        step: input.step.clone(),
        commit: input.commit.clone(),
        parts: Vec::new(),
        empty,
    }
}

/// Newest commit whose subject holds exactly `(Step <step>)`. Stub until Task 6.
pub fn commit_for_step(_repo: &Path, _step: &str) -> Option<GraphCommit> {
    None
}

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
            let transcripts = multi_link_claude::scan(root)?.transcripts;
            for transcript in transcripts
                .iter()
                .filter(|t| parents.contains(t.id.as_str()))
            {
                // An unreadable transcript means that parent owns no calls.
                let calls = multi_link_claude::calls(root, transcript)
                    .map(|c| c.calls)
                    .unwrap_or_default();
                claude_calls.insert(transcript.id.clone(), calls);
            }
        }

        // Codex runs of every owned hand-off of a candidate parent, whatever its
        // step: the other steps' links are needed for the G2 caps.
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
            if let Ok((calls, turns)) = multi_link_codex::read_details(&root.join(path)) {
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
