use multi_link_model::{OverviewReport, Row, State};
use multi_link_tui::view;

fn phase6() -> OverviewReport {
    serde_json::from_str(include_str!(
        "../../../fixtures/expected/phase6/overview.json"
    ))
    .unwrap()
}
#[test]
fn status_phrases_use_evidence_and_whole_minutes() {
    let r = phase6();
    let offset = time::UtcOffset::from_hms(-8, 0, 0).unwrap();
    let rows: Vec<_> = r.repos.iter().flat_map(view::sessions).collect();
    let row = |id: &str| *rows.iter().find(|r| r.id.ends_with(id)).unwrap();
    assert_eq!(
        view::status(row("6001"), r.now, offset),
        "waiting on you · since Nov 1"
    );
    assert_eq!(
        view::status(row("6002"), r.now, offset),
        "working now · for 5 min"
    );
    assert_eq!(view::status(row("6021"), r.now, offset), "done 9:31pm");
    assert_eq!(
        view::status(row("6011"), r.now, offset),
        "idle · last active 1 min ago"
    );
    let mut s = row("6002").clone();
    s.state = State::Stale;
    s.state_since = Some("2026-11-03T06:00:00Z".parse().unwrap());
    assert_eq!(view::status(&s, r.now, offset), "stale since 10:00pm");
    s.state = State::Busy;
    s.activity.clear();
    assert_eq!(view::status(&s, r.now, offset), "working now");
    s.state = State::Idle;
    s.last_activity = "2026-11-03T05:24:00Z".parse().unwrap();
    assert_eq!(view::status(&s, r.now, offset), "idle · last active 9:24pm");
    s.state = State::Unknown;
    assert_eq!(view::status(&s, r.now, offset), "state unknown");
    s.state = State::Finished;
    assert_eq!(view::status(&s, r.now, offset), "finished");
}
#[test]
fn clocks_use_elapsed_days_and_fractional_offsets() {
    let now = "2026-11-03T06:25:00Z".parse().unwrap();
    assert_eq!(view::clock(now, now, time::UtcOffset::UTC), "6:25am");
    assert_eq!(
        view::clock(now, now, time::UtcOffset::from_hms(5, 30, 0).unwrap()),
        "11:55am"
    );
    let yesterday = "2026-11-02T06:00:00Z".parse().unwrap();
    assert_eq!(
        view::clock(yesterday, now, time::UtcOffset::from_hms(-8, 0, 0).unwrap()),
        "Nov 1"
    );
    assert_eq!(
        view::offset_label(time::UtcOffset::from_hms(5, 30, 0).unwrap()),
        "UTC+5:30"
    );
    assert_eq!(
        view::last_active(
            "2026-11-03T05:26:00Z".parse().unwrap(),
            now,
            time::UtcOffset::UTC
        ),
        "59 min ago"
    );
    assert_eq!(
        view::last_active(
            "2026-11-03T05:25:00Z".parse().unwrap(),
            now,
            time::UtcOffset::UTC
        ),
        "5:25am"
    );
}
#[test]
fn labels_and_handoffs_never_invent_placeholders() {
    let r = phase6();
    let mut s = view::sessions(&r.repos[0]).next().unwrap().clone();
    assert_eq!(
        view::session_label(&s),
        "“Draft the release notes, then wait for my review before you publish.”"
    );
    s.last_asked = Some("Synthetic latest prompt".into());
    assert_eq!(view::session_label(&s), "“Synthetic latest prompt”");
    s.made = vec!["6.4".into()];
    assert_eq!(view::session_label(&s), "step 6.4");
    s.made.clear();
    s.last_asked = None;
    assert_eq!(
        view::session_label(&s),
        "“Draft the release notes, then wait for my review before you publish.”"
    );
    s.asked = Some(String::new());
    assert_eq!(view::session_label(&s), "claude · no prompt");
    s.harness = multi_link_model::Harness::Codex;
    assert_eq!(view::session_label(&s), "codex · no prompt");
    let mut link = r.repos[2].latest_handoff.clone().unwrap();
    assert_eq!(view::handed(&link), "handed work to codex");
    assert_eq!(view::outcome(&link, &r), "codex done");
    link.step = Some("6.3".into());
    assert_eq!(view::handed(&link), "handed step 6.3 to codex");
    link.r#match = multi_link_model::Match::Unlinked;
    assert_eq!(view::outcome(&link, &r), "no codex run found");
    link.r#match = multi_link_model::Match::Ambiguous;
    assert_eq!(view::outcome(&link, &r), "1 possible runs");
    link.r#match = multi_link_model::Match::Exact;
    link.child = Some("60310000-0000-4000-8000-000000006031".into());
    assert_eq!(view::outcome(&link, &r), "codex running");
    link.child = None;
    assert_eq!(view::outcome(&link, &r), "codex state unknown");
}

#[test]
fn home_attention_includes_children_and_merged_members_but_hides_imports() {
    let mut r = phase6();
    let g = &mut r.repos[0];
    let Row::Session(parent) = &mut g.rows[1] else {
        panic!("session")
    };
    parent.children[0].state = State::Stale;
    let mut later = parent.children[0].handoffs[0].clone();
    later.parent = "another-parent".into();
    later.step = Some("9.9".into());
    parent.children[0].handoffs.push(later);
    assert_eq!(view::home_label(g, view::attention(g)[2]), "step 6.3");
    assert_eq!(
        view::attention(g)
            .iter()
            .map(|r| &r.id[r.id.len() - 4..])
            .collect::<Vec<_>>(),
        ["6001", "6002", "6021"]
    );
    let Row::Session(parent) = &mut g.rows[1] else {
        panic!("session")
    };
    parent.children[0].imported = true;
    assert_eq!(view::attention(g).len(), 2);
    let members = g
        .rows
        .drain(2..4)
        .map(|r| match r {
            Row::Session(s) => s,
            _ => panic!("session"),
        })
        .collect();
    g.rows.push(Row::Merged(multi_link_model::MergedRow {
        harness: multi_link_model::Harness::Claude,
        count: 2,
        rows: members,
    }));
    assert_eq!(view::sessions(g).count(), 6);
    assert_eq!(view::attention(g).len(), 2);
}

#[test]
fn clock_keeps_previous_day_until_exactly_24_hours_old() {
    let now = "2026-11-03T06:25:00Z".parse().unwrap();
    let recent = "2026-11-02T06:26:00Z".parse().unwrap();
    let boundary = "2026-11-02T06:25:00Z".parse().unwrap();
    for (offset, expected_clock, expected_date) in [
        (time::UtcOffset::UTC, "6:26am", "Nov 2"),
        (
            time::UtcOffset::from_hms(-8, 0, 0).unwrap(),
            "10:26pm",
            "Nov 1",
        ),
        (
            time::UtcOffset::from_hms(5, 30, 0).unwrap(),
            "11:56am",
            "Nov 2",
        ),
    ] {
        assert_eq!(view::clock(recent, now, offset), expected_clock);
        assert_eq!(view::clock(boundary, now, offset), expected_date);
    }
    // Local midnight has passed, but this timestamp is only 90 minutes old.
    let offset = time::UtcOffset::from_hms(-8, 0, 0).unwrap();
    let now = "2026-11-03T08:30:00Z".parse().unwrap();
    assert_eq!(
        view::clock("2026-11-03T07:00:00Z".parse().unwrap(), now, offset),
        "11:00pm"
    );
}

#[test]
fn future_clocks_keep_the_local_date_rule() {
    let now = "2026-10-29T23:15:00Z".parse().unwrap();
    let future = "2026-10-30T06:08:00Z".parse().unwrap();
    assert_eq!(view::clock(future, now, time::UtcOffset::UTC), "Oct 30");
    assert_eq!(
        view::clock(future, now, time::UtcOffset::from_hms(-8, 0, 0).unwrap()),
        "10:08pm"
    );
}

#[test]
fn title_precedes_step_and_each_background_candidate_is_rejected() {
    let report = phase6();
    let mut row = view::sessions(&report.repos[0]).next().unwrap().clone();
    row.title = Some("Synthetic release review".into());
    row.made = vec!["8.4b".into()];
    assert_eq!(view::session_label(&row), "Synthetic release review");
    row.title = None;
    assert_eq!(view::session_label(&row), "step 8.4b");
    row.made.clear();
    row.last_asked =
        Some("Background (from the session that started this one): synthetic context".into());
    row.asked = Some("Synthetic first prompt".into());
    assert_eq!(view::session_label(&row), "“Synthetic first prompt”");
    row.asked = row.last_asked.clone();
    assert_eq!(view::session_label(&row), "claude · no prompt");
    row.last_asked = Some("Synthetic latest prompt".into());
    assert_eq!(view::session_label(&row), "“Synthetic latest prompt”");
    row.place = Some("worktree step-8.4b".into());
    row.branch = Some("main".into());
    assert_eq!(view::place(&row), "worktree step-8.4b");
    assert_eq!(serde_json::to_value(&row).unwrap()["branch"], "main");
    assert!(serde_json::to_value(&row).unwrap().get("place").is_none());
}

#[test]
fn clocks_use_compact_twelve_hour_times_across_an_offset() {
    let offset = time::UtcOffset::from_hms(5, 30, 0).unwrap();
    for (local, expected) in [
        ("00:00:00", "12:00am"),
        ("00:30:00", "12:30am"),
        ("09:05:00", "9:05am"),
        ("12:00:00", "12:00pm"),
        ("13:07:00", "1:07pm"),
        ("23:59:00", "11:59pm"),
    ] {
        let at = format!("2026-11-08T{local}+05:30").parse().unwrap();
        assert_eq!(view::clock(at, at, offset), expected);
    }
}

#[test]
fn clock_seconds_use_the_local_hour_and_lowercase_suffix() {
    let at = view::datetime("2026-11-08T02:35:43Z".parse().unwrap())
        .to_offset(time::UtcOffset::from_hms(5, 30, 0).unwrap());
    assert_eq!(view::clock_label(at, true), "8:05:43am");
    assert_eq!(view::clock_label(at, false), "8:05am");
}
