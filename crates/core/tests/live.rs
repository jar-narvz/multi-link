use multi_link_core::{FixtureProcessProbe, Options, RepoMode, Scanner, live};
use multi_link_model::*;
use std::{collections::HashMap, path::PathBuf};

fn t(s: &str) -> Timestamp {
    s.parse().unwrap()
}
fn input() -> (Vec<Session>, Vec<Link>) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sessions-phase14");
    let (s, l) = Scanner::default()
        .refresh(
            &root,
            &Options {
                now: t("2026-11-10T22:20:00Z"),
                since: None,
                repo_mode: RepoMode::Fixture,
                probe: &FixtureProcessProbe,
                state_dir: root.join("state"),
            },
        )
        .unwrap();
    (s.report.sessions, l.report.links)
}
fn report(s: &[Session], l: &[Link]) -> LiveReport {
    live::build(
        t("2026-11-10T22:20:00Z"),
        None,
        s,
        l,
        &HashMap::new(),
        &HashMap::new(),
    )
}
fn card<'a>(r: &'a LiveReport, tag: &str) -> &'a Card {
    r.groups
        .iter()
        .flat_map(|g| std::iter::once(&g.head).chain(&g.children))
        .find(|c| c.id.starts_with(tag))
        .unwrap()
}

#[test]
fn outcomes_no_rollout_resumed_and_feed_ties() {
    let (s, mut l) = input();
    l.retain(|l| l.at <= t("2026-11-10T21:32:00Z"));
    let r = report(&s, &l);
    assert_eq!(
        r.feed[0].parent,
        l.iter()
            .rev()
            .find(|l| l.at == t("2026-11-10T21:32:00Z"))
            .unwrap()
            .parent
    );
    assert!(r.feed[0].parent.starts_with("1416"));
    assert!(r.feed[1].parent.starts_with("1418"));
    let no_rollout = l
        .iter()
        .find(|l| l.child.as_deref().is_some_and(|id| id.starts_with("142c")))
        .unwrap()
        .clone();
    let r = report(&s, &[no_rollout]);
    assert_eq!(r.feed[0].outcome, None);
    assert_eq!(r.feed[0].outcome_at, None);
}

#[test]
fn codex_from_parent_without_card_and_parent_precedence() {
    let (mut s, l) = input();
    let r = report(&s, &l);
    assert!(
        matches!(&card(&r,"1421").link, Some(CardLink::From {parent,..}) if parent.starts_with("1411"))
    );
    s.iter_mut()
        .find(|s| s.id.starts_with("1411"))
        .unwrap()
        .state = State::Idle;
    let r = report(&s, &l);
    assert!(r.groups.iter().any(|g| g.head.id.starts_with("1421")));
    assert!(
        matches!(&card(&r,"1421").link, Some(CardLink::From {parent,..}) if parent.starts_with("1411"))
    );
    assert!(
        matches!(&card(&r,"1429").link, Some(CardLink::From {parent,..}) if parent.starts_with("1416"))
    );
    assert_eq!(card(&r, "1427").step, None);
    let g = r
        .groups
        .iter()
        .find(|g| g.head.id.starts_with("1415"))
        .unwrap();
    assert_eq!(
        g.children.iter().map(|c| &c.id[..4]).collect::<Vec<_>>(),
        ["1425", "1426", "1427"]
    );
}

#[test]
fn states_imports_and_sort_fallbacks() {
    let (s, _) = input();
    let mut template = s[0].clone();
    template.imported = false;
    template.activity.clear();
    template.state_since = None;
    let mut sessions = Vec::new();
    for (i, state) in [
        State::Unknown,
        State::Finished,
        State::Idle,
        State::Stale,
        State::Busy,
        State::Waiting,
    ]
    .into_iter()
    .enumerate()
    {
        let mut s = template.clone();
        s.id = i.to_string();
        s.state = state;
        sessions.push(s);
    }
    let mut imported = template.clone();
    imported.id = "imported".into();
    imported.state = State::Busy;
    imported.imported = true;
    sessions.push(imported);
    let r = report(&sessions, &[]);
    assert_eq!(
        r.groups
            .iter()
            .map(|g| g.head.id.as_str())
            .collect::<Vec<_>>(),
        ["5", "4", "3"]
    );
    assert_eq!(r.idle.count, 0);
    assert_eq!(
        r.counts,
        LiveCounts {
            waiting: 1,
            busy: 1,
            stale: 1
        }
    );
    assert_eq!(card(&r, "4").since, None);
    assert_eq!(card(&r, "3").since, Some(template.last_activity));
    // Codex lone heads use last_activity when no busy burst; ids break time ties.
    let mut a = template.clone();
    a.id = "a".into();
    a.harness = Harness::Codex;
    a.state = State::Busy;
    let mut b = a.clone();
    b.id = "b".into();
    let mut c = a.clone();
    c.id = "c".into();
    c.last_activity = t("2026-11-10T22:19:59Z");
    a.last_activity = t("2026-11-10T22:20:00Z");
    b.last_activity = a.last_activity;
    let r = report(&[b, a, c], &[]);
    assert_eq!(
        r.groups
            .iter()
            .map(|g| g.head.id.as_str())
            .collect::<Vec<_>>(),
        ["c", "a", "b"]
    );
}

#[test]
fn summaries_merge_latest_time_and_handoff_phrases() {
    let at = t("2026-11-10T22:00:00Z");
    let later = t("2026-11-10T22:01:00Z");
    let calls = [
        Call {
            id: "a".into(),
            at,
            kind: CallKind::Run,
            label: Some("$ cargo test".into()),
            ok: Some(false),
        },
        Call {
            id: "b".into(),
            at: later,
            kind: CallKind::Run,
            label: Some("$ cargo test".into()),
            ok: Some(true),
        },
        Call {
            id: "c".into(),
            at: later,
            kind: CallKind::Handoff,
            label: None,
            ok: Some(true),
        },
    ];
    let held = calls.iter().map(|c| (c.at, c)).collect::<Vec<_>>();
    let (counts, n) =
        multi_link_core::graph::summarize(&held, &HashMap::from([("c", "▸ codex exec".into())]));
    assert_eq!(
        counts,
        Counts {
            reads: 0,
            edits: 0,
            runs: 2
        }
    );
    assert_eq!(
        n,
        vec![
            LiveNotable {
                at: later,
                label: "$ cargo test".into(),
                ok: Some(true)
            },
            LiveNotable {
                at: later,
                label: "▸ codex exec".into(),
                ok: None
            }
        ]
    );
}

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}
fn fixture_options<'a>(probe: &'a FixtureProcessProbe, now: &str) -> Options<'a> {
    Options {
        now: t(now),
        since: None,
        repo_mode: RepoMode::Fixture,
        probe,
        state_dir: PathBuf::new(),
    }
}
use multi_link_core::live::{codex_steps, started_steps};
#[test]
fn started_steps_are_direct_same_repo_children_by_start() {
    use multi_link_model::Harness;
    let root = fixtures().join("sessions-phase8");
    let probe = multi_link_core::FixtureProcessProbe;
    let (sessions, links) = Scanner::default()
        .refresh(&root, &fixture_options(&probe, "2026-11-06T18:00:00Z"))
        .unwrap();
    let template = sessions
        .report
        .sessions
        .iter()
        .find(|s| s.harness == Harness::Claude)
        .unwrap()
        .clone();
    let session = |id: &str, repo: &str, by: Option<&str>, started: &str| {
        let mut s = template.clone();
        s.id = id.into();
        s.repo = repo.into();
        s.started_by = by.map(Into::into);
        s.started_at = started.parse().unwrap();
        s.last_activity = s.started_at;
        s
    };
    let mut codex = session("x1", "/r", Some("k"), "2026-11-06T09:01:00Z");
    codex.harness = Harness::Codex;
    let sessions = vec![
        session("k", "/r", None, "2026-11-06T09:00:00Z"),
        // Started after b: listed second. Quiet for days: no window applies here.
        session("a", "/r", Some("k"), "2026-11-06T09:10:00Z"),
        session("b", "/r", Some("k"), "2026-11-06T09:05:00Z"),
        // Grandchild of k (started by a).
        session("a2", "/r", Some("a"), "2026-11-06T09:20:00Z"),
        // Another repo's child is not a child (8.4c S3).
        session("o", "/other", Some("k"), "2026-11-06T09:02:00Z"),
        // No tagged hand-off: skipped.
        session("d", "/r", Some("k"), "2026-11-06T09:03:00Z"),
        // Same step as b: deduped, b (earlier start) kept.
        session("e", "/r", Some("k"), "2026-11-06T09:30:00Z"),
        // Same start as e, later id.
        session("f", "/r", Some("k"), "2026-11-06T09:30:00Z"),
        // Parent not listed.
        session("g", "/r", Some("gone"), "2026-11-06T09:00:00Z"),
        codex,
    ];
    let template = links.report.links[0].clone();
    let link = |parent: &str, step: Option<&str>, at: &str| {
        let mut l = template.clone();
        l.parent = parent.into();
        l.step = step.map(Into::into);
        l.at = at.parse().unwrap();
        l
    };
    let links = vec![
        link("a", Some("2.0"), "2026-11-06T09:15:00Z"),
        link("a", Some("2.1"), "2026-11-06T09:16:00Z"),
        link("a", None, "2026-11-06T09:17:00Z"),
        link("b", Some("3.1"), "2026-11-06T09:06:00Z"),
        link("a2", Some("9.9"), "2026-11-06T09:21:00Z"),
        link("o", Some("7.7"), "2026-11-06T09:03:00Z"),
        link("d", None, "2026-11-06T09:04:00Z"),
        link("e", Some("3.1"), "2026-11-06T09:31:00Z"),
        link("f", Some("4.0"), "2026-11-06T09:31:00Z"),
        link("g", Some("5.0"), "2026-11-06T09:01:00Z"),
        link("k", Some("1.0"), "2026-11-06T09:01:00Z"),
    ];
    let entry = |step: &str| ("/r".to_string(), step.to_string());
    let map = started_steps(&sessions, &links);
    assert_eq!(map["k"], [entry("3.1"), entry("2.1"), entry("4.0")]);
    assert_eq!(map["a"], [entry("9.9")]);
    assert_eq!(map.len(), 2, "{map:?}");
}
#[test]
fn codex_steps_use_latest_tagged_same_repo_incoming_by_time_and_position() {
    let root = fixtures().join("sessions-phase9");
    let probe = multi_link_core::FixtureProcessProbe;
    let (sessions, links) = Scanner::default()
        .refresh(&root, &fixture_options(&probe, "2026-11-09T12:00:00Z"))
        .unwrap();
    let template = &sessions.report.sessions[0];
    let session = |id: &str, repo: &str, harness: Harness| {
        let mut s = template.clone();
        s.id = id.into();
        s.repo = repo.into();
        s.harness = harness;
        s
    };
    let sessions = vec![
        session("p", "/r", Harness::Claude),
        session("other", "/other", Harness::Claude),
        session("x", "/r", Harness::Codex),
        session("untagged", "/r", Harness::Codex),
    ];
    let template = &links.report.links[0];
    let link = |parent: &str, child: &str, step: Option<&str>, at: &str| {
        let mut l = template.clone();
        l.parent = parent.into();
        l.child = Some(child.into());
        l.step = step.map(Into::into);
        l.at = at.parse().unwrap();
        l
    };
    let links = vec![
        link("p", "x", Some("2.0"), "2026-11-09T09:00:00Z"),
        link("p", "x", Some("2.1"), "2026-11-09T09:00:00Z"),
        link("p", "x", Some("1.0"), "2026-11-09T08:00:00Z"),
        link("other", "x", Some("7.1"), "2026-11-09T11:00:00Z"),
        link("missing", "x", Some("8.1"), "2026-11-09T11:00:00Z"),
        link("p", "x", None, "2026-11-09T12:00:00Z"),
        link("p", "untagged", None, "2026-11-09T12:00:00Z"),
        link("p", "p", Some("9.9"), "2026-11-09T12:00:00Z"),
    ];
    assert_eq!(
        codex_steps(&sessions, &links),
        HashMap::from([("x".into(), "2.1".into())])
    );
}
#[test]
fn resumed_outcomes_running_precedes_done_and_older_link_uses_first_completion() {
    let (mut s, l) = input();
    let links: Vec<_> = l
        .into_iter()
        .filter(|l| l.child.as_deref().is_some_and(|c| c.starts_with("1421")))
        .collect();
    let child = s
        .iter()
        .find(|s| s.id.starts_with("1421"))
        .unwrap()
        .id
        .clone();
    let turn = |start, end| Turn {
        start: t(start),
        completed_at: Some(t(end)),
        last_record_at: t(end),
        prompt: None,
        last_agent_message: None,
    };
    let codex = HashMap::from([(
        child.clone(),
        multi_link_core::CodexDetails {
            calls: vec![],
            turns: vec![
                turn("2026-11-10T21:40:01Z", "2026-11-10T21:48:00Z"),
                turn("2026-11-10T22:10:01Z", "2026-11-10T22:15:00Z"),
                turn("2026-11-10T22:16:00Z", "2026-11-10T22:17:00Z"),
            ],
        },
    )]);
    let r = live::build(
        t("2026-11-10T22:20:00Z"),
        None,
        &s,
        &links,
        &HashMap::new(),
        &codex,
    );
    assert_eq!(r.feed[0].outcome, Some(LiveOutcome::Running));
    assert_eq!(r.feed[0].outcome_at, None);
    assert_eq!(r.feed[1].outcome, Some(LiveOutcome::Done));
    assert_eq!(r.feed[1].outcome_at, Some(t("2026-11-10T21:48:00Z")));
    s.iter_mut().find(|s| s.id == child).unwrap().state = State::Idle;
    let r = live::build(
        t("2026-11-10T22:20:00Z"),
        None,
        &s,
        &links,
        &HashMap::new(),
        &codex,
    );
    assert_eq!(r.feed[0].outcome, Some(LiveOutcome::Done));
    assert_eq!(r.feed[0].outcome_at, Some(t("2026-11-10T22:15:00Z")));
}

#[test]
fn window_hides_old_stale_and_imports_from_cards_and_idle() {
    let root = fixtures().join("sessions-phase14");
    let options = Options {
        since: Some(std::time::Duration::from_secs(7200)),
        ..fixture_options(&FixtureProcessProbe, "2026-11-10T22:20:00Z")
    };
    let r = Scanner::default().live(&root, &options).unwrap();
    let ids: Vec<_> = r
        .groups
        .iter()
        .flat_map(|g| std::iter::once(&g.head).chain(&g.children))
        .map(|c| &c.id[..4])
        .collect();
    assert_eq!(
        ids,
        [
            "1413", "1411", "1421", "1412", "1422", "1415", "1425", "1426", "1427", "1416", "1428",
            "1429", "1424"
        ]
    );
    assert_eq!(r.idle.count, 7);
    assert_eq!(
        r.idle.repos,
        [
            "/home/dev/repo-m",
            "/home/dev/repo-n",
            "/home/dev/repo-r",
            "/home/dev/repo-s"
        ]
    );
    assert_eq!(card(&r, "1425").since, r.window_from);
    assert_eq!(card(&r, "1413").since, Some(t("2026-11-10T22:12:00Z")));
    assert_eq!(card(&r, "1411").since, Some(t("2026-11-10T21:45:00Z")));
}

#[test]
fn link_variants_and_untagged_notable() {
    let (mut s, mut l) = input();
    let parent = s.iter().find(|s| s.id.starts_with("1413")).unwrap().clone();
    let mut link = l.iter().find(|l| l.parent == parent.id).unwrap().clone();
    // No child: the Claude card must retain ambiguous and unlinked hand-off information.
    link.child = None;
    link.r#match = Match::Ambiguous;
    let r = report(std::slice::from_ref(&parent), std::slice::from_ref(&link));
    assert!(matches!(
        &r.groups[0].head.link,
        Some(CardLink::Handoff {
            r#match: Match::Ambiguous,
            outcome: None,
            ..
        })
    ));
    link.r#match = Match::Unlinked;
    link.candidates.clear();
    let r = report(std::slice::from_ref(&parent), std::slice::from_ref(&link));
    assert!(matches!(
        &r.groups[0].head.link,
        Some(CardLink::Handoff {
            r#match: Match::Unlinked,
            candidates: 0,
            outcome: None,
            ..
        })
    ));
    assert_eq!(
        report(std::slice::from_ref(&parent), &[]).groups[0]
            .head
            .link,
        None
    );
    let r = report(&s, &l);
    assert_eq!(card(&r, "1424").link, Some(CardLink::None));
    assert!(
        matches!(&card(&r,"1411").link,Some(CardLink::WaitingOn{child,at}) if child.starts_with("1421") && *at==t("2026-11-10T22:10:00Z"))
    );
    let id = parent.id.clone();
    let calls = HashMap::from([(
        id.clone(),
        vec![Call {
            id: "untagged".into(),
            at: parent.last_activity,
            kind: CallKind::Handoff,
            label: None,
            ok: None,
        }],
    )]);
    l.clear();
    s = vec![parent];
    let r = live::build(
        t("2026-11-10T22:20:00Z"),
        None,
        &s,
        &l,
        &calls,
        &HashMap::new(),
    );
    assert_eq!(
        r.groups[0].head.notable,
        vec![LiveNotable {
            at: s[0].last_activity,
            label: "▸ codex exec".into(),
            ok: None
        }]
    );
}

#[test]
fn child_ties_cross_repo_and_no_started_by_fallback() {
    let (s, l) = input();
    let mut p = s.iter().find(|s| s.id.starts_with("1411")).unwrap().clone();
    p.id = "parent".into();
    let mut x = s.iter().find(|s| s.id.starts_with("1421")).unwrap().clone();
    x.id = "x".into();
    x.repo = "/other".into();
    let mut y = x.clone();
    y.id = "y".into();
    let mut lone = x.clone();
    lone.id = "lone".into();
    lone.started_by = Some(p.id.clone());
    let mut lx = l[0].clone();
    lx.parent = p.id.clone();
    lx.child = Some(x.id.clone());
    lx.at = t("2026-11-10T22:00:00Z");
    lx.r#match = Match::Heuristic;
    let mut ly = lx.clone();
    ly.child = Some(y.id.clone());
    let mut ambiguous = lx.clone();
    ambiguous.child = Some(lone.id.clone());
    ambiguous.r#match = Match::Ambiguous;
    let r = report(&[p, x, y, lone], &[ly, lx, ambiguous]);
    let g = r.groups.iter().find(|g| g.head.id == "parent").unwrap();
    assert_eq!(
        g.children.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
        ["x", "y"]
    );
    assert_eq!(g.children[0].step, None);
    assert_eq!(
        g.head.link,
        Some(CardLink::WaitingOn {
            child: "y".into(),
            at: t("2026-11-10T22:00:00Z")
        })
    );
    assert!(r.groups.iter().any(|g| g.head.id == "lone"));
    assert_eq!(card(&r, "lone").link, Some(CardLink::None));
}

#[test]
fn head_sort_uses_state_since_burst_and_full_id() {
    let (s, _) = input();
    let mut template = s[0].clone();
    template.imported = false;
    template.activity.clear();
    let make = |id: &str, state, since: &str| {
        let mut s = template.clone();
        s.id = id.into();
        s.state = state;
        s.state_since = Some(t(since));
        s
    };
    let mut busy_a = make("busy-a", State::Busy, "2026-11-10T22:19:00Z");
    busy_a.activity = vec![Activity {
        from: t("2026-11-10T22:00:00Z"),
        to: t("2026-11-10T22:19:00Z"),
    }];
    let mut busy_b = busy_a.clone();
    busy_b.id = "busy-b".into();
    let r = report(
        &[
            make("wait-b", State::Waiting, "2026-11-10T22:05:00Z"),
            make("stale-b", State::Stale, "2026-11-10T21:00:00Z"),
            busy_b,
            make("wait-a", State::Waiting, "2026-11-10T22:00:00Z"),
            busy_a,
            make("stale-a", State::Stale, "2026-11-10T20:00:00Z"),
        ],
        &[],
    );
    assert_eq!(
        r.groups
            .iter()
            .map(|g| g.head.id.as_str())
            .collect::<Vec<_>>(),
        ["wait-a", "wait-b", "busy-a", "busy-b", "stale-a", "stale-b"]
    );
}

#[test]
fn empty_report_window_boundaries_and_missing_parent_repo() {
    let now = t("2026-11-10T22:20:00Z");
    let from = t("2026-11-10T20:20:00Z");
    let empty = live::build(now, Some(from), &[], &[], &HashMap::new(), &HashMap::new());
    assert_eq!(empty.groups, vec![]);
    assert_eq!(empty.feed, vec![]);
    assert_eq!(empty.last_handoff, None);
    assert_eq!(
        empty.idle,
        LiveIdle {
            count: 0,
            repos: vec![]
        }
    );
    let (_, l) = input();
    let mut old = l[0].clone();
    old.at = t("2026-11-10T20:19:59Z");
    old.invocation = "old".into();
    let mut first = old.clone();
    first.at = from;
    first.invocation = "first".into();
    let mut last = old.clone();
    last.at = now;
    last.invocation = "last".into();
    let mut future = old.clone();
    future.at = t("2026-11-10T22:20:01Z");
    future.invocation = "future".into();
    let links = [old, first, last, future];
    let r = live::build(
        now,
        Some(from),
        &[],
        &links,
        &HashMap::new(),
        &HashMap::new(),
    );
    assert_eq!(
        r.feed
            .iter()
            .map(|r| r.invocation.as_str())
            .collect::<Vec<_>>(),
        ["last", "first"]
    );
    assert_eq!(r.feed[0].repo, links[2].checkout);
    assert_eq!(r.last_handoff, Some(t("2026-11-10T22:20:01Z")));
}
