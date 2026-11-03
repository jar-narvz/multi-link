//! The home screen's data (output-schema "overview", design T2, T5, T11).
//! Pure: built from one refresh's listed sessions and links plus a plan lookup.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::{Path, PathBuf},
};

use multi_link_model::{
    ConcurrencyWarning, Convention, Harness, LatestStep, Link, LinksReport, MergedRow,
    OverviewReport, Plan, RepoGroup, RepoSteps, Row, Session, SessionRow, SessionsReport, State,
    Tier,
};

use crate::RepoMode;

/// Per-harness threshold: 4 or more quiet top-level rows merge.
pub const MERGE_AT: usize = 4;

/// Where a group's PLAN.md / PROGRESS.md live. Real mode: the repo itself (the
/// main checkout; worktrees share it). Fixture mode: `<root>/repos/<basename>`,
/// basename = last `/`-separated component of the recorded repo string.
/// `None` (plan `missing`, lookup not called) when the fixture basename would
/// escape `<root>/repos`: no non-empty component (`/`, `""`), `.` or `..`.
pub fn plan_folder(root: &Path, repo_mode: RepoMode, repo: &str) -> Option<PathBuf> {
    match repo_mode {
        RepoMode::Real => Some(PathBuf::from(repo)),
        RepoMode::Fixture => {
            let basename = repo.rsplit('/').find(|part| !part.is_empty())?;
            if basename == "." || basename == ".." {
                return None;
            }
            Some(root.join("repos").join(basename))
        }
    }
}

/// Build the overview. `plans` maps a plan folder to its steps report (the
/// `steps` rules); the caller supplies it so core stays free of the plan parser.
pub fn build(
    root: &Path,
    repo_mode: RepoMode,
    sessions: &SessionsReport,
    links: &LinksReport,
    plans: &mut dyn FnMut(&Path) -> RepoSteps,
) -> OverviewReport {
    let by_id: HashMap<&str, &Session> = sessions
        .sessions
        .iter()
        .map(|session| (session.id.as_str(), session))
        .collect();
    let key = |link: &Link| (link.at, link.invocation.clone());

    // Codex placement (T11): the latest hand-off whose parent is a listed
    // Claude session in the child's own repo.
    let mut placed: HashMap<&str, &Link> = HashMap::new();
    for session in sessions
        .sessions
        .iter()
        .filter(|s| s.harness == Harness::Codex)
    {
        let placing = links
            .links
            .iter()
            .filter(|link| link.child.as_deref() == Some(session.id.as_str()))
            .filter(|link| {
                // The Claude check is stricter than the contract and harmless:
                // link parents are always Claude sessions (invocations come
                // from Claude transcripts).
                by_id.get(link.parent.as_str()).is_some_and(|parent| {
                    parent.harness == Harness::Claude && parent.repo == session.repo
                })
            })
            .max_by_key(|link| key(link));
        if let Some(link) = placing {
            placed.insert(session.id.as_str(), link);
        }
    }

    let session_row = |session: &Session, children: Vec<SessionRow>| {
        let mut handoffs: Vec<Link> = match session.harness {
            Harness::Codex => links
                .links
                .iter()
                .filter(|link| link.child.as_deref() == Some(session.id.as_str()))
                .cloned()
                .collect(),
            Harness::Claude => Vec::new(),
        };
        handoffs.sort_by_key(|link| key(link));
        SessionRow {
            id: session.id.clone(),
            harness: session.harness,
            state: session.state,
            state_evidence: session.state_evidence,
            checkout: session.checkout.clone(),
            branch: session.branch.clone(),
            started_at: session.started_at,
            last_activity: session.last_activity,
            handoffs,
            made: made(session, &links.links),
            activity: session.activity.clone(),
            asked: session.asked.clone(),
            last_asked: session.last_asked.clone(),
            imported: session.imported,
            state_since: session.state_since,
            children,
        }
    };

    // Group every listed session by repo; children share their parent's repo.
    let mut groups: BTreeMap<&str, Vec<&Session>> = BTreeMap::new();
    for session in &sessions.sessions {
        groups
            .entry(session.repo.as_str())
            .or_default()
            .push(session);
    }

    let mut repos: Vec<RepoGroup> = groups
        .into_iter()
        .map(|(repo, members)| {
            // Each placed child attaches once: to the first top-level member (in
            // member order) whose id is its placing parent. Two listed sessions
            // sharing an id must not both show the child (every session once).
            let mut owner: HashMap<&str, usize> = HashMap::new();
            for (index, parent) in members.iter().enumerate() {
                if placed.contains_key(parent.id.as_str()) {
                    continue;
                }
                for child in &members {
                    if let Some(link) = placed.get(child.id.as_str())
                        && link.parent == parent.id
                    {
                        owner.entry(child.id.as_str()).or_insert(index);
                    }
                }
            }
            let mut rows: Vec<SessionRow> = members
                .iter()
                .enumerate()
                .filter(|(_, session)| !placed.contains_key(session.id.as_str()))
                .map(|(index, session)| {
                    let mut children: Vec<(&Link, &Session)> = members
                        .iter()
                        .filter_map(|child| {
                            let link = placed.get(child.id.as_str())?;
                            (owner.get(child.id.as_str()) == Some(&index))
                                .then_some((*link, *child))
                        })
                        .collect();
                    children.sort_by_key(|(link, _)| key(link));
                    let children = children
                        .into_iter()
                        .map(|(_, child)| session_row(child, Vec::new()))
                        .collect();
                    session_row(session, children)
                })
                .collect();
            rows.sort_by(|a, b| {
                (
                    state_rank(a.state),
                    std::cmp::Reverse(a.last_activity),
                    &a.id,
                )
                    .cmp(&(
                        state_rank(b.state),
                        std::cmp::Reverse(b.last_activity),
                        &b.id,
                    ))
            });

            let tier = if members.iter().any(|s| s.state == State::Waiting) {
                Tier::Waiting
            } else if members.iter().any(|s| s.state == State::Busy) {
                Tier::Busy
            } else {
                Tier::Other
            };
            let last_activity = members
                .iter()
                .map(|s| s.last_activity)
                .max()
                .expect("a group has at least one session");

            // Hand-offs belong to the group that shows their parent.
            let group_links: Vec<&Link> = links
                .links
                .iter()
                .filter(|link| {
                    by_id
                        .get(link.parent.as_str())
                        .is_some_and(|parent| parent.repo == repo)
                })
                .collect();
            let active =
                |state: State| matches!(state, State::Waiting | State::Busy | State::Stale);
            let active_steps: BTreeSet<String> = group_links
                .iter()
                .filter(|link| {
                    active(by_id[link.parent.as_str()].state)
                        || link
                            .child
                            .as_deref()
                            .and_then(|child| by_id.get(child))
                            .is_some_and(|child| active(child.state))
                })
                .filter_map(|link| link.step.clone())
                .collect();
            let latest_handoff = group_links
                .iter()
                .max_by_key(|link| key(link))
                .map(|link| (*link).clone());

            let mut busy: BTreeMap<&str, Vec<String>> = BTreeMap::new();
            for session in members.iter().filter(|s| s.state == State::Busy) {
                busy.entry(session.cwd.as_str())
                    .or_default()
                    .push(session.id.clone());
            }
            let concurrency = busy
                .into_iter()
                .filter(|(_, ids)| ids.len() >= 2)
                .map(|(cwd, mut ids)| {
                    ids.sort();
                    ConcurrencyWarning {
                        cwd: cwd.to_owned(),
                        sessions: ids,
                    }
                })
                .collect();

            RepoGroup {
                repo: repo.to_owned(),
                tier,
                last_activity,
                plan: match plan_folder(root, repo_mode, repo) {
                    Some(folder) => plan(plans(&folder)),
                    None => Plan {
                        convention: Convention::Missing,
                        latest_step: None,
                    },
                },
                active_steps: active_steps.into_iter().collect(),
                concurrency,
                latest_handoff,
                rows: merge(rows),
            }
        })
        .collect();
    repos.sort_by(|a, b| {
        (
            tier_rank(a.tier),
            std::cmp::Reverse(a.last_activity),
            &a.repo,
        )
            .cmp(&(
                tier_rank(b.tier),
                std::cmp::Reverse(b.last_activity),
                &b.repo,
            ))
    });
    OverviewReport {
        now: sessions.now,
        hook: Default::default(),
        repos,
    }
}

/// Distinct steps in descending order of their newest listed launch.
fn made(session: &Session, links: &[Link]) -> Vec<String> {
    if session.harness == Harness::Codex {
        return Vec::new();
    }
    let mut launches: Vec<_> = links
        .iter()
        .filter(|link| link.parent == session.id && link.step.is_some())
        .collect();
    launches.sort_by(|a, b| (b.at, &b.invocation).cmp(&(a.at, &a.invocation)));
    let mut seen = BTreeSet::new();
    launches
        .into_iter()
        .filter_map(|link| link.step.as_ref())
        .filter(|step| seen.insert((*step).clone()))
        .cloned()
        .collect()
}

fn state_rank(state: State) -> u8 {
    match state {
        State::Waiting => 0,
        State::Busy => 1,
        State::Stale => 2,
        State::Idle => 3,
        State::Unknown => 4,
        State::Finished => 5,
    }
}

fn tier_rank(tier: Tier) -> u8 {
    match tier {
        Tier::Waiting => 0,
        Tier::Busy => 1,
        Tier::Other => 2,
    }
}

fn quiet_state(state: State) -> bool {
    matches!(state, State::Idle | State::Unknown | State::Finished)
}

/// Quiet top-level rows merge by harness; an active child keeps its parent visible.
fn quiet(row: &SessionRow) -> bool {
    quiet_state(row.state) && row.children.iter().all(|child| quiet_state(child.state))
}

/// Replace each harness's quiet rows at its first member's position, preserving
/// member order even when the two harnesses are interleaved.
fn merge(rows: Vec<SessionRow>) -> Vec<Row> {
    let mut out: Vec<Row> = rows.iter().cloned().map(Row::Session).collect();
    for harness in [Harness::Claude, Harness::Codex] {
        let merged: Vec<_> = rows
            .iter()
            .filter(|row| row.harness == harness && quiet(row))
            .cloned()
            .collect();
        if merged.len() < MERGE_AT {
            continue;
        }
        let slot = out
            .iter()
            .position(
                |row| matches!(row, Row::Session(row) if row.harness == harness && quiet(row)),
            )
            .expect("at least MERGE_AT quiet rows");
        out.retain(|row| !matches!(row, Row::Session(row) if row.harness == harness && quiet(row)));
        out.insert(
            slot,
            Row::Merged(MergedRow {
                harness,
                count: merged.len(),
                rows: merged,
            }),
        );
    }
    out
}

fn plan(steps: RepoSteps) -> Plan {
    let latest_step = match (&steps.convention, steps.latest_logged_step) {
        (Convention::Supported, Some(id)) => {
            let step = steps.steps.iter().find(|step| step.id == id);
            Some(LatestStep {
                title: step.map(|step| step.title.clone()),
                entry: step.and_then(|step| step.progress.first().cloned()),
                id,
            })
        }
        _ => None,
    };
    Plan {
        convention: steps.convention,
        latest_step,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use multi_link_model::{
        Entry, InvocationForm, Match, StateEvidence, Step, Timestamp, Verify, VerifyResult,
        VerifySource,
    };

    fn at(minute: u32) -> Timestamp {
        Timestamp::parse(&format!("2026-10-29T23:{minute:02}:00Z")).unwrap()
    }

    fn session(id: &str, harness: Harness, repo: &str, state: State, minute: u32) -> Session {
        Session {
            id: id.into(),
            harness,
            transcript: None,
            record: None,
            host_session_id: None,
            cwd: repo.into(),
            repo: repo.into(),
            checkout: repo.into(),
            branch: None,
            started_at: at(0),
            last_activity: at(minute),
            state,
            state_evidence: StateEvidence::NoSignal,
            segments: Vec::new(),
            subagents: Vec::new(),
            activity: Vec::new(),
            asked: None,
            last_asked: None,
            imported: false,
            state_since: None,
        }
    }

    fn claude(id: &str, repo: &str, state: State, minute: u32) -> Session {
        session(id, Harness::Claude, repo, state, minute)
    }

    fn codex(id: &str, repo: &str, state: State, minute: u32) -> Session {
        session(id, Harness::Codex, repo, state, minute)
    }

    fn link(parent: &str, child: &str, minute: u32) -> Link {
        Link {
            step: None,
            parent: parent.into(),
            invocation: format!("toolu_{parent}_{child}"),
            subagent: None,
            form: InvocationForm::Exec,
            background: false,
            at: at(minute),
            checkout: String::new(),
            r#match: Match::Exact,
            child: Some(child.into()),
            evidence: None,
            candidates: Vec::new(),
        }
    }

    fn missing(folder: &Path) -> RepoSteps {
        RepoSteps {
            repo: folder.to_string_lossy().into_owned(),
            convention: Convention::Missing,
            latest_logged_step: None,
            steps: Vec::new(),
        }
    }

    fn run(sessions: Vec<Session>, links: Vec<Link>) -> OverviewReport {
        build(
            Path::new("/root"),
            RepoMode::Fixture,
            &SessionsReport {
                now: at(30),
                sessions,
                skipped: Vec::new(),
            },
            &LinksReport {
                links,
                unparented: Vec::new(),
            },
            &mut missing,
        )
    }

    fn ids(rows: &[Row]) -> Vec<String> {
        rows.iter()
            .map(|row| match row {
                Row::Session(row) => row.id.clone(),
                Row::Merged(merged) => format!("merged{}", merged.count),
            })
            .collect()
    }

    fn only_session(row: &Row) -> &SessionRow {
        match row {
            Row::Session(row) => row,
            Row::Merged(_) => panic!("expected a session row"),
        }
    }

    #[test]
    fn made_handles_empty_untagged_and_single_launch() {
        let parent = claude("p", "/r", State::Idle, 1);
        assert_eq!(made(&parent, &[]), Vec::<String>::new());
        let mut launch = link("p", "c", 1);
        assert_eq!(made(&parent, &[launch.clone()]), Vec::<String>::new());
        launch.step = Some("5.2".into());
        assert_eq!(made(&parent, &[launch.clone()]), ["5.2"]);
        assert_eq!(
            made(&codex("p", "/r", State::Idle, 1), &[launch]),
            Vec::<String>::new()
        );
    }

    #[test]
    fn made_deduplicates_by_newest_time_and_invocation_for_any_match() {
        let parent = claude("p", "/r", State::Idle, 1);
        let launches = [
            ("old", "1", 1),
            ("a", "2", 2),
            ("z", "1", 2),
            ("other-parent", "3", 9),
            ("middle", "4", 2),
        ]
        .into_iter()
        .map(|(invocation, step, minute)| {
            let mut launch = link("p", "c", minute);
            launch.invocation = invocation.into();
            launch.step = Some(step.into());
            launch.r#match = Match::Unlinked;
            launch.child = None;
            if invocation == "other-parent" {
                launch.parent = "elsewhere".into();
            }
            launch
        })
        .collect::<Vec<_>>();
        assert_eq!(made(&parent, &launches), ["1", "4", "2"]);
    }

    #[test]
    fn cross_repo_link_keeps_child_top_level_in_its_own_repo() {
        let report = run(
            vec![
                claude("parent", "/a", State::Idle, 1),
                codex("child", "/b", State::Unknown, 2),
            ],
            vec![link("parent", "child", 1)],
        );
        let groups: Vec<_> = report.repos.iter().map(|g| g.repo.as_str()).collect();
        assert_eq!(groups, ["/b", "/a"]);
        let b = &report.repos[0];
        assert_eq!(ids(&b.rows), ["child"]);
        assert_eq!(only_session(&b.rows[0]).handoffs.len(), 1);
        let a = &report.repos[1];
        assert!(only_session(&a.rows[0]).children.is_empty());
        // The hand-off belongs to the group that shows its parent.
        assert_eq!(a.latest_handoff.as_ref().unwrap().parent, "parent");
        assert!(b.latest_handoff.is_none());
    }

    #[test]
    fn merge_needs_exactly_four_quiet_rows() {
        let three = run(
            (0..3)
                .map(|i| claude(&format!("q{i}"), "/r", State::Idle, i))
                .chain([claude("busy", "/r", State::Busy, 0)])
                .collect(),
            vec![],
        );
        assert_eq!(ids(&three.repos[0].rows), ["busy", "q2", "q1", "q0"]);

        let four = run(
            vec![
                claude("q0", "/r", State::Idle, 4),
                claude("q1", "/r", State::Unknown, 9),
                claude("q2", "/r", State::Finished, 8),
                claude("q3", "/r", State::Idle, 1),
                claude("busy", "/r", State::Busy, 0),
                codex("solo", "/r", State::Unknown, 7),
            ],
            vec![],
        );
        let rows = &four.repos[0].rows;
        // Merged row takes the first quiet row's place; one Codex row stays separate.
        assert_eq!(ids(rows), ["busy", "merged4", "solo"]);
        let Row::Merged(merged) = &rows[1] else {
            panic!("expected merged")
        };
        let members: Vec<_> = merged.rows.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(members, ["q0", "q3", "q1", "q2"]);
    }

    #[test]
    fn codex_merge_threshold_excludes_active_rows() {
        for count in [3, 4] {
            let sessions = (0..count)
                .map(|i| codex(&format!("q{i}"), "/r", State::Unknown, i * 2))
                .chain([
                    codex("busy", "/r", State::Busy, 3),
                    codex("waiting", "/r", State::Waiting, 1),
                    codex("stale", "/r", State::Stale, 5),
                ])
                .collect();
            let report = run(sessions, vec![]);
            let rows = &report.repos[0].rows;
            if count == 3 {
                assert_eq!(ids(rows), ["waiting", "busy", "stale", "q2", "q1", "q0"]);
            } else {
                assert_eq!(ids(rows), ["waiting", "busy", "stale", "merged4"]);
                let Row::Merged(merged) = &rows[3] else {
                    panic!("expected merged")
                };
                assert_eq!(merged.harness, Harness::Codex);
                assert_eq!(
                    merged
                        .rows
                        .iter()
                        .map(|r| r.id.as_str())
                        .collect::<Vec<_>>(),
                    ["q3", "q2", "q1", "q0"]
                );
            }
        }
    }

    #[test]
    fn interleaved_harnesses_merge_independently_at_their_first_member() {
        for first in [Harness::Codex, Harness::Claude] {
            let second = if first == Harness::Codex {
                Harness::Claude
            } else {
                Harness::Codex
            };
            let sessions = (0..4)
                .flat_map(|i| {
                    [
                        session(&format!("first{i}"), first, "/r", State::Idle, i * 2 + 1),
                        session(&format!("second{i}"), second, "/r", State::Idle, i * 2),
                    ]
                })
                .collect();
            let report = run(sessions, vec![]);
            let rows = &report.repos[0].rows;
            assert_eq!(rows.len(), 2);
            for (row, harness, prefix) in [(&rows[0], first, "first"), (&rows[1], second, "second")]
            {
                let Row::Merged(merged) = row else {
                    panic!("expected merged")
                };
                assert_eq!(merged.harness, harness);
                assert_eq!(merged.count, 4);
                assert_eq!(
                    merged.rows.iter().map(|r| r.id.clone()).collect::<Vec<_>>(),
                    [
                        format!("{prefix}3"),
                        format!("{prefix}2"),
                        format!("{prefix}1"),
                        format!("{prefix}0")
                    ]
                );
            }
        }
    }

    #[test]
    fn busy_codex_child_keeps_unknown_parent_out_of_the_merge() {
        let mut sessions: Vec<_> = (0..4)
            .map(|i| claude(&format!("q{i}"), "/r", State::Unknown, i))
            .collect();
        sessions.push(codex("child", "/r", State::Busy, 20));
        let report = run(sessions, vec![link("q3", "child", 3)]);
        let group = &report.repos[0];
        assert_eq!(group.tier, Tier::Busy);
        // Only 3 quiet rows remain, so no merge; q3 keeps its own place.
        assert_eq!(ids(&group.rows), ["q3", "q2", "q1", "q0"]);
        assert_eq!(only_session(&group.rows[0]).children[0].id, "child");

        // Same with 5 parents: the 4 quiet ones merge, q4 stays visible.
        let mut sessions: Vec<_> = (0..5)
            .map(|i| claude(&format!("q{i}"), "/r", State::Unknown, i))
            .collect();
        sessions.push(codex("child", "/r", State::Stale, 20));
        let report = run(sessions, vec![link("q4", "child", 3)]);
        assert_eq!(ids(&report.repos[0].rows), ["q4", "merged4"]);
    }

    #[test]
    fn stale_sits_between_busy_and_idle_and_finished_is_last() {
        let report = run(
            vec![
                claude("finished", "/r", State::Finished, 9),
                claude("idle", "/r", State::Idle, 1),
                claude("stale", "/r", State::Stale, 2),
                claude("busy", "/r", State::Busy, 3),
                claude("waiting", "/r", State::Waiting, 0),
                claude("unknown", "/r", State::Unknown, 8),
            ],
            vec![],
        );
        assert_eq!(
            ids(&report.repos[0].rows),
            ["waiting", "busy", "stale", "idle", "unknown", "finished"]
        );
        assert_eq!(report.repos[0].tier, Tier::Waiting);
        // A stale-only group is `other`.
        let stale = run(vec![claude("s", "/r", State::Stale, 0)], vec![]);
        assert_eq!(stale.repos[0].tier, Tier::Other);
    }

    #[test]
    fn equal_tier_and_activity_order_repos_by_name() {
        let report = run(
            vec![
                claude("z", "/zeta", State::Idle, 5),
                claude("a", "/alpha", State::Unknown, 5),
                claude("m", "/mid", State::Idle, 6),
                claude("b", "/busy", State::Busy, 1),
            ],
            vec![],
        );
        let groups: Vec<_> = report.repos.iter().map(|g| g.repo.as_str()).collect();
        assert_eq!(groups, ["/busy", "/mid", "/alpha", "/zeta"]);
    }

    #[test]
    fn two_repos_with_one_basename_read_the_same_plan_folder() {
        let mut folders = Vec::new();
        let mut plans = |folder: &Path| {
            folders.push(folder.to_owned());
            RepoSteps {
                repo: String::new(),
                convention: Convention::Supported,
                latest_logged_step: Some("2".into()),
                steps: vec![Step {
                    id: "2".into(),
                    title: "Two".into(),
                    verify_command: None,
                    progress: vec![Entry {
                        date: "2026-10-29".into(),
                        agent: None,
                        verify: Verify {
                            command: None,
                            result: VerifyResult::Pass,
                            result_text: "pass".into(),
                            source: VerifySource::Reported,
                        },
                    }],
                }],
            }
        };
        let report = build(
            Path::new("/root"),
            RepoMode::Fixture,
            &SessionsReport {
                now: at(30),
                sessions: vec![
                    claude("one", "/home/a/app", State::Idle, 1),
                    claude("two", "/home/b/app", State::Idle, 2),
                ],
                skipped: Vec::new(),
            },
            &LinksReport {
                links: Vec::new(),
                unparented: Vec::new(),
            },
            &mut plans,
        );
        assert_eq!(folders, vec![PathBuf::from("/root/repos/app"); 2]);
        assert_eq!(report.repos[0].repo, "/home/b/app");
        assert_eq!(report.repos[1].repo, "/home/a/app");
        assert_eq!(report.repos[0].plan, report.repos[1].plan);
        let latest = report.repos[0].plan.latest_step.as_ref().unwrap();
        assert_eq!(latest.title.as_deref(), Some("Two"));
        assert_eq!(
            latest.entry.as_ref().unwrap().verify.result,
            VerifyResult::Pass
        );
        assert_eq!(
            plan_folder(Path::new("/root"), RepoMode::Real, "/home/a/app"),
            Some(PathBuf::from("/home/a/app"))
        );
    }

    #[test]
    fn fixture_basename_that_escapes_repos_is_missing_without_lookup() {
        for repo in ["/", "", "///", "/home/..", "/home/.", "..", "/a/../"] {
            assert_eq!(
                plan_folder(Path::new("/root"), RepoMode::Fixture, repo),
                None,
                "{repo:?}"
            );
        }
        assert_eq!(
            plan_folder(Path::new("/root"), RepoMode::Fixture, "/home/app/"),
            Some(PathBuf::from("/root/repos/app"))
        );
        let mut calls = 0;
        let mut plans = |folder: &Path| {
            calls += 1;
            missing(folder)
        };
        let report = build(
            Path::new("/root"),
            RepoMode::Fixture,
            &SessionsReport {
                now: at(30),
                sessions: vec![
                    claude("root", "/", State::Idle, 1),
                    claude("up", "/home/..", State::Idle, 2),
                ],
                skipped: Vec::new(),
            },
            &LinksReport {
                links: Vec::new(),
                unparented: Vec::new(),
            },
            &mut plans,
        );
        assert_eq!(calls, 0);
        for group in &report.repos {
            assert_eq!(
                group.plan,
                Plan {
                    convention: Convention::Missing,
                    latest_step: None,
                }
            );
        }
    }

    fn all_row_ids(rows: &[Row]) -> Vec<String> {
        fn session(row: &SessionRow, out: &mut Vec<String>) {
            out.push(row.id.clone());
            out.extend(row.children.iter().map(|child| child.id.clone()));
        }
        let mut out = Vec::new();
        for row in rows {
            match row {
                Row::Session(row) => session(row, &mut out),
                Row::Merged(merged) => merged.rows.iter().for_each(|row| session(row, &mut out)),
            }
        }
        out
    }

    #[test]
    fn duplicate_parent_ids_attach_a_child_once() {
        let mut second = claude("dup", "/r", State::Idle, 5);
        second.cwd = "/r/other".into();
        let sessions = vec![
            claude("dup", "/r", State::Idle, 1),
            second,
            codex("child", "/r", State::Unknown, 2),
        ];
        let listed = sessions.len();
        let report = run(sessions, vec![link("dup", "child", 1)]);
        let rows = &report.repos[0].rows;
        // Every listed session exactly once.
        assert_eq!(all_row_ids(rows).len(), listed);
        // The first parent in member order (minute 1) takes the child.
        let owners: Vec<_> = rows
            .iter()
            .map(only_session)
            .map(|row| (row.last_activity, row.children.len()))
            .collect();
        assert_eq!(owners, [(at(5), 0), (at(1), 1)]);
    }

    #[test]
    fn logged_step_missing_from_plan_has_null_title_and_entry() {
        let latest = plan(RepoSteps {
            repo: String::new(),
            convention: Convention::Supported,
            latest_logged_step: Some("9".into()),
            steps: Vec::new(),
        })
        .latest_step
        .unwrap();
        assert_eq!(
            (latest.id.as_str(), latest.title, latest.entry),
            ("9", None, None)
        );
        assert!(plan(missing(Path::new("/x"))).latest_step.is_none());
    }

    #[test]
    fn child_placed_under_latest_same_repo_parent_with_every_handoff() {
        let report = run(
            vec![
                claude("early", "/r", State::Idle, 1),
                claude("late", "/r", State::Idle, 2),
                codex("child", "/r", State::Unknown, 3),
                codex("other", "/r", State::Unknown, 3),
            ],
            vec![
                link("late", "child", 2),
                link("early", "child", 1),
                link("late", "other", 1),
            ],
        );
        let rows = &report.repos[0].rows;
        assert_eq!(ids(rows), ["late", "early"]);
        let late = only_session(&rows[0]);
        // Children ordered by their placing hand-off: other (minute 1) before child (2).
        let children: Vec<_> = late.children.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(children, ["other", "child"]);
        let handoffs: Vec<_> = late.children[1]
            .handoffs
            .iter()
            .map(|l| l.parent.as_str())
            .collect();
        assert_eq!(handoffs, ["early", "late"]);
        assert!(only_session(&rows[1]).children.is_empty());
    }

    #[test]
    fn concurrency_and_active_steps() {
        let mut stepped = link("p", "c", 1);
        stepped.step = Some("3.3".into());
        let mut quiet_step = link("q", "d", 1);
        quiet_step.step = Some("9".into());
        let mut busy_b = claude("b2", "/r", State::Busy, 2);
        busy_b.cwd = "/r/sub".into();
        let report = run(
            vec![
                claude("p", "/r", State::Idle, 1),
                codex("c", "/r", State::Busy, 5),
                claude("q", "/r", State::Idle, 1),
                codex("d", "/r", State::Unknown, 1),
                claude("b1", "/r", State::Busy, 3),
                busy_b,
                claude("s", "/r", State::Stale, 4),
            ],
            vec![stepped, quiet_step],
        );
        let group = &report.repos[0];
        // Active via the busy child; "9" has neither side active.
        assert_eq!(group.active_steps, ["3.3"]);
        // c and b1 share cwd /r; b2 is alone in /r/sub; stale s does not count.
        assert_eq!(
            group.concurrency,
            [ConcurrencyWarning {
                cwd: "/r".into(),
                sessions: vec!["b1".into(), "c".into()],
            }]
        );
    }
}
