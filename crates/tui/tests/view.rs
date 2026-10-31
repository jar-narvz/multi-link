use multi_link_model::{OverviewReport, Row, State};
use multi_link_tui::view;
use std::collections::HashSet;

fn report() -> OverviewReport {
    serde_json::from_str(include_str!(
        "../../../fixtures/expected/phase5/overview.json"
    ))
    .unwrap()
}
#[test]
fn waiting_yesterday_and_quiet_chain_are_visible_without_core_merges() {
    let r = report();
    let g = &r.repos[0];
    let lanes = view::lanes(&r, g, &HashSet::new(), false);
    assert_eq!(
        lanes
            .visible
            .iter()
            .map(|l| &l.row.id[l.row.id.len() - 4..])
            .collect::<Vec<_>>(),
        [
            "5001", "5002", "5011", "5012", "5013", "5014", "5025", "5015", "5016"
        ]
    );
    assert_eq!(lanes.hidden, 1);
    assert_eq!(lanes.visible[0].state(), State::Waiting);
    assert_eq!(
        view::clock(
            lanes.visible[0].row.last_activity,
            r.now,
            time::UtcOffset::from_hms(-7, 0, 0).unwrap()
        ),
        "Oct 28 17:05"
    );
    assert_eq!(
        lanes.visible[2].child.unwrap().id,
        "00000000-0000-4000-8000-000000005021"
    );
    assert_eq!(view::lanes(&r, g, &HashSet::new(), true).visible.len(), 10);
}
#[test]
fn nine_quiet_rows_cap_at_eight_and_attention_is_outside_the_cap() {
    let mut r = report();
    let seed = view::sessions(&r.repos[0])
        .find(|s| s.id.ends_with("5012"))
        .unwrap()
        .clone();
    r.repos[0].rows = (0..9)
        .map(|i| {
            let mut s = seed.clone();
            s.id = format!("quiet-{i}");
            Row::Session(s)
        })
        .collect();
    let mut waiting = seed.clone();
    waiting.id = "waiting".into();
    waiting.state = State::Waiting;
    waiting.last_activity = "2026-10-28T00:00:00Z".parse().unwrap();
    r.repos[0].rows.push(Row::Session(waiting));
    let lanes = view::lanes(&r, &r.repos[0], &HashSet::new(), false);
    assert_eq!(
        lanes
            .visible
            .iter()
            .map(|l| l.row.id.as_str())
            .collect::<Vec<_>>(),
        [
            "waiting", "quiet-0", "quiet-1", "quiet-2", "quiet-3", "quiet-4", "quiet-5", "quiet-6",
            "quiet-7"
        ]
    );
    assert_eq!(lanes.hidden, 1);
}
#[test]
fn combined_child_has_its_own_span_and_window_membership() {
    let mut r = report();
    let mut parent = view::sessions(&r.repos[0])
        .find(|s| s.id.ends_with("5015"))
        .unwrap()
        .clone();
    parent.last_activity = "2026-10-29T20:00:00Z".parse().unwrap();
    parent.started_at = "2026-10-29T19:00:00Z".parse().unwrap();
    r.repos[0].rows = vec![Row::Session(parent)];
    let lanes = view::lanes(&r, &r.repos[0], &HashSet::new(), false);
    assert_eq!(lanes.visible.len(), 1);
    let l = lanes.visible[0];
    assert!(l.child.unwrap().last_activity > l.row.last_activity);
    assert_eq!(lanes.hidden, 0);
}
#[test]
fn different_steps_multiple_children_and_explicit_split_stay_separate() {
    let mut r = report();
    let mut parent = view::sessions(&r.repos[0])
        .find(|s| s.id.ends_with("5015"))
        .unwrap()
        .clone();
    parent.made = vec!["different".into()];
    r.repos[0].rows = vec![Row::Session(parent.clone())];
    let lanes = view::lanes(&r, &r.repos[0], &HashSet::new(), false);
    assert_eq!(lanes.visible.len(), 2);
    assert!(lanes.visible.iter().all(|l| l.child.is_none()));
    assert_eq!(lanes.visible[0].step(), Some("different"));
    assert_eq!(lanes.visible[1].step(), Some("4.5"));
    parent.made.clear();
    parent.children[0].handoffs[0].step = None;
    r.repos[0].rows = vec![Row::Session(parent.clone())];
    assert_eq!(
        view::lanes(&r, &r.repos[0], &HashSet::new(), false)
            .visible
            .len(),
        1
    );
    assert_eq!(
        view::lanes(&r, &r.repos[0], &HashSet::from([parent.id.clone()]), false)
            .visible
            .len(),
        2
    );
    let mut child = parent.children[0].clone();
    child.id = "second-child".into();
    parent.children.push(child);
    r.repos[0].rows = vec![Row::Session(parent)];
    assert_eq!(
        view::lanes(&r, &r.repos[0], &HashSet::new(), false)
            .visible
            .len(),
        3
    );
}
#[test]
fn child_attention_sets_lane_priority_and_shared_checkout_never_collapses() {
    let mut r = report();
    let mut parent = view::sessions(&r.repos[0])
        .find(|s| s.id.ends_with("5015"))
        .unwrap()
        .clone();
    parent.children[0].state = State::Waiting;
    parent.children[0].last_activity = "2026-10-28T00:00:00Z".parse().unwrap();
    parent.last_activity = parent.children[0].last_activity;
    r.repos[0].rows = vec![Row::Session(parent)];
    assert_eq!(
        view::lanes(&r, &r.repos[0], &HashSet::new(), false).visible[0].state(),
        State::Waiting
    );
    let g = &mut r.repos[1];
    assert!(view::collapsed(g, &HashSet::new()));
    g.concurrency.push(multi_link_model::ConcurrencyWarning {
        cwd: g.repo.clone(),
        sessions: vec!["a".into(), "b".into()],
    });
    assert!(!view::collapsed(g, &HashSet::new()));
}
