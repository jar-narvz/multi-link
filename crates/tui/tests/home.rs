//! Home screen snapshots (design T4): render the overview answer keys and compare with reviewed
//! goldens in `fixtures/expected/tui/`. `UPDATE_SNAPSHOTS=1` rewrites the goldens instead.

mod support;

use std::{fs, path::PathBuf};

use multi_link_model::{Link, OverviewReport, Row, State};
use multi_link_tui::{HomeOptions, Target, render_home, render_home_with_options, selectable};
use ratatui::{buffer::Buffer, layout::Rect, style::Modifier, text::Span};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load(path: &str) -> OverviewReport {
    let text = fs::read_to_string(repo_root().join(path)).expect("read overview fixture");
    serde_json::from_str(&text).expect("overview fixture is an OverviewReport")
}

fn render(report: &OverviewReport, selection: Option<usize>, width: u16, height: u16) -> Buffer {
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    // Detailed lane tests explicitly expand repos; snapshots below exercise default collapse.
    let options = HomeOptions {
        expanded_repos: report
            .repos
            .iter()
            .map(|group| group.repo.clone())
            .collect(),
        ..HomeOptions::default()
    };
    render_home_with_options(report, selection, &options, area, &mut buf);
    buf
}

/// One line per buffer row, trailing spaces trimmed. A wide character's trailing cell is
/// skipped so the text reads as the terminal shows it.
fn text(buf: &Buffer) -> String {
    let mut out = String::new();
    for y in 0..buf.area.height {
        let mut line = String::new();
        let mut x = 0;
        while x < buf.area.width {
            let symbol = buf[(x, y)].symbol();
            line.push_str(symbol);
            x += Span::raw(symbol).width().max(1) as u16;
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

fn check_snapshot(fixture: &str, golden: &str, width: u16, height: u16) {
    let report = load(fixture);
    let links_path = if fixture.contains("hook") {
        "fixtures/expected/hook/links.json"
    } else {
        "fixtures/expected/links.json"
    };
    let links = if fixture.contains("phase4") {
        // The isolated Phase 4 fixture has no hand-offs (output-schema.md).
        multi_link_model::LinksReport {
            links: vec![],
            unparented: vec![],
        }
    } else {
        serde_json::from_str(&fs::read_to_string(repo_root().join(links_path)).unwrap()).unwrap()
    };
    let mut options = HomeOptions {
        links: Some(&links),
        ..HomeOptions::default()
    };
    if fixture.contains("phase4") {
        options.expanded_repos.insert("/home/dev/repo-i".into());
    }
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    render_home_with_options(&report, None, &options, area, &mut buf);
    support::check_styles(
        &buf,
        &repo_root()
            .join("fixtures/expected/tui")
            .join(golden.replace(".txt", ".styles.txt")),
    );
    let rendered = text(&buf);
    let path = repo_root().join("fixtures/expected/tui").join(golden);
    if std::env::var_os("UPDATE_SNAPSHOTS").is_some_and(|value| value == "1") {
        fs::write(&path, &rendered).expect("write golden");
        return;
    }
    let expected = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("missing golden {golden}; run with UPDATE_SNAPSHOTS=1"));
    assert_eq!(rendered, expected, "{golden} differs from the render");
}

#[test]
fn home_snapshot_100x30() {
    check_snapshot("fixtures/expected/overview.json", "home.txt", 100, 30);
}

#[test]
fn home_hook_snapshot_100x30() {
    check_snapshot(
        "fixtures/expected/hook/overview.json",
        "home-hook.txt",
        100,
        30,
    );
}

#[test]
fn home_phase4_snapshot_160x30() {
    check_snapshot(
        "fixtures/expected/phase4/overview.json",
        "home-phase4.txt",
        160,
        30,
    );
}

/// Tall enough for every group, so each overview row can be reviewed against the answer key.
#[test]
fn home_full_snapshot_100x80() {
    check_snapshot("fixtures/expected/overview.json", "home-full.txt", 100, 80);
}

#[test]
fn home_hook_full_snapshot_100x80() {
    check_snapshot(
        "fixtures/expected/hook/overview.json",
        "home-hook-full.txt",
        100,
        80,
    );
}

#[test]
fn small_sizes_do_not_panic() {
    for fixture in [
        "fixtures/expected/overview.json",
        "fixtures/expected/hook/overview.json",
    ] {
        let report = load(fixture);
        let count = selectable(&report).len();
        for (width, height) in [
            (0, 0),
            (1, 1),
            (0, 5),
            (5, 0),
            (20, 5),
            (40, 10),
            (2, 2),
            (3, 3),
            (60, 1),
            (60, 2),
            (60, 3),
            (60, 4),
            (100, 4),
        ] {
            for selection in [None, Some(0), count.checked_sub(1), Some(count + 5)] {
                render(&report, selection, width, height);
            }
        }
    }
}

#[test]
fn area_outside_the_buffer_is_ignored() {
    let report = load("fixtures/expected/overview.json");
    let mut buf = Buffer::empty(Rect::new(0, 0, 10, 4));
    render_home(&report, Some(0), Rect::new(5, 2, 50, 50), &mut buf);
    render_home(&report, Some(0), Rect::new(20, 20, 5, 5), &mut buf);
}

/// Rows whose every cell is reversed: the selection bar.
fn highlighted_rows(buf: &Buffer) -> Vec<u16> {
    (0..buf.area.height)
        .filter(|&y| (0..buf.area.width).all(|x| buf[(x, y)].modifier.contains(Modifier::REVERSED)))
        .collect()
}

fn row_text(buf: &Buffer, y: u16) -> String {
    text(buf)
        .lines()
        .nth(usize::from(y))
        .unwrap_or_default()
        .to_string()
}

#[test]
fn selection_highlights_the_selected_row() {
    let report = load("fixtures/expected/overview.json");
    let targets = selectable(&report);

    // Index 2 is repo-c's third row, 11c1f13c (rows come from the report as given).
    let Target::Session(third) = targets[2].1 else {
        panic!("expected a session row")
    };
    assert_eq!(&third.id[..8], "11c1f13c");
    let buf = render(&report, Some(2), 100, 30);
    let rows = highlighted_rows(&buf);
    assert_eq!(rows.len(), 1, "exactly one highlighted row");
    assert!(row_text(&buf, rows[0]).contains("11c1f13c"));

    // No selection, no highlight.
    assert!(highlighted_rows(&render(&report, None, 100, 30)).is_empty());
    // Past the end: no highlight, no panic.
    assert!(highlighted_rows(&render(&report, Some(targets.len()), 100, 30)).is_empty());
}

#[test]
fn selection_below_the_fold_scrolls_into_view() {
    let report = load("fixtures/expected/overview.json");
    let targets = selectable(&report);
    let last = targets.len() - 1;
    let Target::Session(session) = targets[last].1 else {
        panic!("expected a session row")
    };
    let buf = render(&report, Some(last), 100, 30);
    let rows = highlighted_rows(&buf);
    assert_eq!(rows.len(), 1);
    // The last body row (just above the footer) holds the selection.
    assert_eq!(rows[0], 28);
    assert!(row_text(&buf, rows[0]).contains(&session.id[..8]));
    // Title and footer stay in place.
    assert!(row_text(&buf, 0).starts_with("multi-link"));
    assert!(row_text(&buf, 29).contains("q quit"));
}

#[test]
fn selectable_lists_children_and_one_line_per_merged_row() {
    let report = load("fixtures/expected/overview.json");
    let targets = selectable(&report);
    let mut expected = 0;
    for group in &report.repos {
        for row in &group.rows {
            expected += match row {
                Row::Session(session) => 1 + session.children.len(),
                Row::Merged(_) => 1,
            };
        }
    }
    assert_eq!(targets.len(), expected);
    let merged = targets
        .iter()
        .filter(|(_, target)| matches!(target, Target::Merged(_)))
        .count();
    assert_eq!(merged, 1, "repo-b's claude ×5 is one line");
}

/// No fixture has a worktree checkout, a step tag, an active step or an ambiguous latest
/// hand-off, so pin those on a modified report.
#[test]
fn checkout_label_step_tag_and_ambiguous_handoff() {
    let mut report = load("fixtures/expected/overview.json");
    let group = report
        .repos
        .iter_mut()
        .find(|group| group.repo.ends_with("/repo-a"))
        .expect("repo-a group");
    group.active_steps = vec!["3.5".to_string()];
    let Row::Session(parent) = group
        .rows
        .iter_mut()
        .find(|row| matches!(row, Row::Session(s) if s.id.starts_with("ef72378c")))
        .expect("ef72378c row")
    else {
        unreachable!()
    };
    parent.checkout = "/home/dev/multi-link-sandbox/repo-a-wt2".to_string();
    let child = &mut parent.children[0];
    let link: &mut Link = child.handoffs.last_mut().expect("child hand-off");
    link.step = Some("3.5".to_string());
    let mut latest = link.clone();
    latest.child = None;
    latest.r#match = multi_link_model::Match::Ambiguous;
    latest.candidates = vec![
        multi_link_model::Candidate {
            id: "a".into(),
            gap_ms: 1,
        },
        multi_link_model::Candidate {
            id: "b".into(),
            gap_ms: 2,
        },
    ];
    group.latest_handoff = Some(latest);

    let screen = text(&render(&report, None, 160, 80));
    let line = |needle: &str| {
        screen
            .lines()
            .find(|line| line.contains(needle))
            .unwrap_or_else(|| panic!("no line with {needle}:\n{screen}"))
            .to_string()
    };
    assert!(line("ef72378c").contains("@repo-a-wt2"));
    assert!(line("01a189e2").contains("step 3.5"));
    assert!(line("Parse sample records").contains("active 3.5"));
    assert!(line("↳ hand-off ef72378c").contains("? (2 candidates)  ambiguous"));
    // A row whose checkout is the repo carries no label.
    assert!(!line("bb792d9c").contains('@'));
}

/// A long unicode title is cut at the area edge, on a character boundary, without panicking.
#[test]
fn long_unicode_title_is_truncated() {
    let mut report = load("fixtures/expected/overview.json");
    let group = report
        .repos
        .iter_mut()
        .find(|group| group.plan.latest_step.is_some())
        .expect("a group with a latest step");
    group.plan.latest_step.as_mut().unwrap().title = Some("ステップ".repeat(40));
    for width in [10, 21, 40, 99] {
        let screen = text(&render(&report, None, width, 80));
        for line in screen.lines() {
            assert!(
                Span::raw(line).width() <= usize::from(width),
                "{line:?} wider than {width}"
            );
        }
    }
}

#[test]
fn waiting_repo_gets_the_marker() {
    let report = load("fixtures/expected/hook/overview.json");
    let screen = text(&render(&report, None, 100, 80));
    let headers: Vec<&str> = screen
        .lines()
        .filter(|line| line.starts_with('◆'))
        .collect();
    assert_eq!(headers.len(), 2, "repo-a and repo-c wait:\n{screen}");
    assert!(headers[0].contains("1 waiting"));
    let waiting = report.repos[0]
        .rows
        .iter()
        .filter(|row| matches!(row, Row::Session(s) if s.state == State::Waiting))
        .count();
    assert_eq!(waiting, 1);
}

/// A Rect at the very end of the u16 range must not overflow the row arithmetic.
#[test]
fn area_at_the_bottom_of_the_u16_range_does_not_overflow() {
    let report = load("fixtures/expected/overview.json");
    for rect in [Rect::new(0, 65534, 1, 1), Rect::new(0, 65533, 3, 2)] {
        let mut buf = Buffer::empty(rect);
        for selection in [None, Some(0), Some(3)] {
            render_home(&report, selection, rect, &mut buf);
        }
    }
}

/// A child row shows its placing hand-off (T11: the latest link from its parent), not a later
/// link from another parent.
#[test]
fn child_row_shows_its_placing_handoff() {
    let mut report = load("fixtures/expected/overview.json");
    let group = report
        .repos
        .iter_mut()
        .find(|group| group.repo.ends_with("/repo-a"))
        .expect("repo-a group");
    let Row::Session(parent) = group
        .rows
        .iter_mut()
        .find(|row| matches!(row, Row::Session(s) if s.id.starts_with("ef72378c")))
        .expect("ef72378c row")
    else {
        unreachable!()
    };
    let parent_id = parent.id.clone();
    let child = &mut parent.children[0];
    let placing = child
        .handoffs
        .iter_mut()
        .rev()
        .find(|link| link.parent == parent_id)
        .expect("placing hand-off");
    placing.step = Some("3.5".to_string());
    let mut other = placing.clone();
    other.parent = "bb792d9c-7f69-410b-a571-b6d97d3aaffa".to_string();
    other.r#match = multi_link_model::Match::Heuristic;
    other.step = Some("9.9".to_string());
    child.handoffs.push(other);

    let screen = text(&render(&report, None, 160, 80));
    let line = screen
        .lines()
        .find(|line| line.contains("01a189e2"))
        .expect("child row");
    assert!(
        line.contains("step 3.5") && line.contains("exact"),
        "{line}"
    );
    assert!(
        !line.contains("heuristic") && !line.contains("9.9"),
        "{line}"
    );
    assert!(line.contains("3 hand-offs"), "{line}");
    // A top-level Codex row with no links says so without claiming there was no hand-off.
    let manual = screen
        .lines()
        .find(|line| line.contains("01a189e5"))
        .expect("manual run");
    assert!(manual.contains("no linked hand-off"), "{manual}");
}

/// Clipped content says how much is hidden below, and the selection stays visible above it.
#[test]
fn clipped_body_shows_lines_below() {
    let report = load("fixtures/expected/overview.json");
    let full = text(&render(&report, None, 100, 200));
    // Body lines = rows between the title and the footer, up to the last non-empty one.
    let rows: Vec<&str> = full.lines().collect();
    let body = &rows[2..rows.len() - 1];
    let body_total = body.iter().rposition(|line| !line.is_empty()).unwrap() + 1;
    let buf = render(&report, None, 100, 30);
    let marker = row_text(&buf, 28);
    assert_eq!(marker, format!("  ↓ {} more lines", body_total - 26));

    // Selecting the first row below the fold scrolls it just above the marker.
    let screen = text(&buf);
    let targets = selectable(&report);
    let hidden = targets
        .iter()
        .position(|(_, target)| match target {
            Target::Session(session) => !screen.contains(&session.id[..8]),
            Target::Merged(_) | Target::Repo(_) => false,
        })
        .expect("a row below the fold");
    let buf = render(&report, Some(hidden), 100, 30);
    assert_eq!(highlighted_rows(&buf), vec![27]);
    assert!(row_text(&buf, 28).contains("more lines"));
    // Nothing hidden: no marker.
    assert!(!text(&render(&report, None, 100, 80)).contains("more lines"));
}

#[test]
fn header_counts_hidden_members_children_and_the_full_link_report() {
    let mut report = load("fixtures/expected/hook/overview.json");
    // Latest-per-repo hand-offs cannot supply the full count.
    for group in &mut report.repos {
        group.latest_handoff = None;
    }
    let links = serde_json::from_str(
        &fs::read_to_string(repo_root().join("fixtures/expected/hook/links.json")).unwrap(),
    )
    .unwrap();
    let area = Rect::new(0, 0, 160, 80);
    let mut buf = Buffer::empty(area);
    let mut options = HomeOptions {
        links: Some(&links),
        ..HomeOptions::default()
    };
    render_home_with_options(&report, None, &options, area, &mut buf);
    let expected = "multi-link  2 waiting · 2 busy · 0 stale · 1 warning · 8 repos · 29 sessions · 13 hand-offs  now 2026-10-29T23:15:00.000Z";
    assert_eq!(row_text(&buf, 0), expected);
    options.expanded_repos = report
        .repos
        .iter()
        .map(|group| group.repo.clone())
        .collect();
    let mut buf = Buffer::empty(area);
    render_home_with_options(&report, None, &options, area, &mut buf);
    assert_eq!(row_text(&buf, 0), expected);
    assert!(row_text(&buf, 0).contains("  now "));
    let area = Rect::new(0, 0, 100, 30);
    let mut buf = Buffer::empty(area);
    render_home_with_options(&report, None, &options, area, &mut buf);
    assert_eq!(row_text(&buf, 0), expected.split("  now ").next().unwrap());
    assert!(row_text(&buf, 0).ends_with("hand-offs"));
}

#[test]
fn header_uses_singular_counts() {
    let mut report = load("fixtures/expected/phase4/overview.json");
    report.repos.retain(|group| group.repo.ends_with("/repo-j"));
    let mut links: multi_link_model::LinksReport = serde_json::from_str(
        &fs::read_to_string(repo_root().join("fixtures/expected/links.json")).unwrap(),
    )
    .unwrap();
    links.links.truncate(1);
    let options = HomeOptions {
        links: Some(&links),
        ..HomeOptions::default()
    };
    let area = Rect::new(0, 0, 100, 10);
    let mut buf = Buffer::empty(area);
    render_home_with_options(&report, None, &options, area, &mut buf);
    assert_eq!(
        row_text(&buf, 0),
        "multi-link  0 waiting · 0 busy · 0 stale · 0 warnings · 1 repo · 1 session · 1 hand-off"
    );
}

#[test]
fn default_collapse_highlights_only_repo_summaries_and_stale_lanes() {
    let report = load("fixtures/expected/phase4/overview.json");
    let targets = multi_link_tui::selectable_home(&report, &Default::default());
    assert_eq!(targets.len(), 8);
    let area = Rect::new(0, 0, 160, 30);
    for (index, (_, target)) in targets.iter().enumerate() {
        let mut buf = Buffer::empty(area);
        render_home(&report, Some(index), area, &mut buf);
        let rows = highlighted_rows(&buf);
        assert_eq!(rows.len(), 1);
        let expected = match target {
            Target::Session(row) => format!("00000000…{}", &row.id[row.id.len() - 4..]),
            Target::Repo(group) => format!("▸ {}", group.repo.rsplit('/').next().unwrap()),
            Target::Merged(_) => panic!("quiet merged rows should be hidden"),
        };
        assert!(row_text(&buf, rows[0]).contains(&expected));
    }
}

/// A single expanded repo with controlled spans on the default 21:15..23:15 UTC axis.
fn mini_bar_report() -> OverviewReport {
    let mut report = load("fixtures/expected/phase4/overview.json");
    report.repos.retain(|group| group.repo.ends_with("/repo-h"));
    let group = &mut report.repos[0];
    group.rows.truncate(1);
    let Row::Session(row) = &mut group.rows[0] else {
        panic!("session")
    };
    row.id = "parent".into();
    row.started_at = "2026-10-29T21:00:00Z".parse().unwrap();
    row.last_activity = "2026-10-29T21:25:00Z".parse().unwrap();
    row.children.clear();
    let mut child = row.clone();
    child.id = "child".into();
    child.checkout = "/home/dev/a-long-worktree-label".into();
    child.started_at = "2026-10-29T22:15:00Z".parse().unwrap();
    child.last_activity = child.started_at;
    row.children.push(child);
    report
}

fn cells(buf: &Buffer, y: u16, from: u16, to: u16) -> String {
    (from..to).map(|x| buf[(x, y)].symbol()).collect()
}

#[test]
fn mini_bars_clip_spans_and_draw_zero_length_children_on_the_shared_axis() {
    let mut report = mini_bar_report();
    let buf = render(&report, Some(1), 100, 10);
    assert_eq!(
        row_text(&buf, 0),
        "multi-link  0 waiting · 2 busy · 0 stale · 0 warnings · 1 repo · 2 sessions · ? hand-offs"
    );
    assert_eq!(cells(&buf, 1, 76, 100), "21:15              23:15");
    assert!(row_text(&buf, 1).contains("session span · idle gaps included · UTC"));
    assert_eq!(cells(&buf, 4, 76, 100), "◂━━·····················");
    assert_eq!(cells(&buf, 5, 76, 100), "············━···········");
    assert_eq!(highlighted_rows(&buf), vec![5]);

    let Row::Session(row) = &mut report.repos[0].rows[0] else {
        panic!("session")
    };
    row.started_at = "2026-10-29T22:15:00Z".parse().unwrap();
    row.last_activity = "2026-10-29T23:30:00Z".parse().unwrap();
    let buf = render(&report, None, 100, 10);
    assert_eq!(cells(&buf, 4, 76, 100), "············━━━━━━━━━━━▸");
    // Parent and child have different indentation and metadata lengths, but identical starts.
    assert_eq!(buf[(88, 4)].symbol(), "━");
    assert_eq!(buf[(88, 5)].symbol(), "━");
}

#[test]
fn mini_bars_show_empty_tracks_for_spans_outside_the_window() {
    let mut report = mini_bar_report();
    let Row::Session(row) = &mut report.repos[0].rows[0] else {
        panic!("session")
    };
    row.started_at = "2026-10-29T20:00:00Z".parse().unwrap();
    row.last_activity = "2026-10-29T21:00:00Z".parse().unwrap();
    row.children[0].started_at = "2026-10-30T00:00:00Z".parse().unwrap();
    row.children[0].last_activity = "2026-10-30T00:01:00Z".parse().unwrap();
    let buf = render(&report, None, 100, 10);
    assert_eq!(cells(&buf, 4, 76, 100), "························");
    assert_eq!(cells(&buf, 5, 76, 100), "························");
}

#[test]
fn mini_bars_start_at_sixty_columns_and_respect_offset_areas() {
    let report = mini_bar_report();
    let narrow = text(&render(&report, None, 59, 10));
    assert!(!narrow.contains("session span"));
    assert!(!narrow.contains(['━', '◂', '▸']));
    let buf = render(&report, None, 60, 10);
    assert_eq!(cells(&buf, 1, 48, 60), "21:15  23:15");
    assert_eq!(cells(&buf, 4, 48, 60), "◂━··········");
    let area = Rect::new(7, 3, 100, 10);
    let mut offset = Buffer::empty(Rect::new(0, 0, 120, 20));
    render_home(&report, None, area, &mut offset);
    assert_eq!(cells(&offset, 7, 83, 107), "◂━━·····················");
    assert_eq!(offset[(107, 7)].symbol(), " ");
}

#[test]
fn merged_and_collapsed_repo_rows_have_no_mini_bar() {
    let report = load("fixtures/expected/phase4/overview.json");
    let buf = render(&report, None, 160, 80);
    let screen = text(&buf);
    let merged: Vec<_> = screen
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("×4"))
        .collect();
    assert_eq!(merged.len(), 3);
    for (y, _) in merged {
        assert_eq!(cells(&buf, y as u16, 136, 160), "                        ");
    }
    let mut buf = Buffer::empty(Rect::new(0, 0, 160, 30));
    render_home(&report, None, buf.area, &mut buf);
    let screen = text(&buf);
    let collapsed: Vec<_> = screen
        .lines()
        .enumerate()
        .filter(|(_, line)| line.starts_with("▸ "))
        .collect();
    assert_eq!(collapsed.len(), 3);
    for (y, _) in collapsed {
        assert_eq!(cells(&buf, y as u16, 136, 160), "                        ");
    }
}

#[test]
fn mini_bars_include_both_window_endpoints_and_follow_report_now() {
    let mut report = mini_bar_report();
    let Row::Session(row) = &mut report.repos[0].rows[0] else {
        panic!("session")
    };
    row.started_at = "2026-10-29T21:15:00Z".parse().unwrap();
    row.last_activity = "2026-10-29T23:15:00Z".parse().unwrap();
    row.children[0].started_at = row.last_activity;
    row.children[0].last_activity = row.last_activity;
    let buf = render(&report, None, 100, 10);
    assert_eq!(cells(&buf, 4, 76, 100), "━━━━━━━━━━━━━━━━━━━━━━━━");
    assert_eq!(cells(&buf, 5, 76, 100), "·······················━");
    report.now = "2026-10-30T00:15:00Z".parse().unwrap();
    let buf = render(&report, None, 100, 10);
    assert_eq!(cells(&buf, 1, 76, 100), "22:15              00:15");
    assert_eq!(cells(&buf, 4, 76, 100), "◂━━━━━━━━━━━━···········");
    assert_eq!(cells(&buf, 5, 76, 100), "············━···········");
}
