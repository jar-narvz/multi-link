//! Board 8c contract tests. Immutable targets have no snapshot update path.
#[allow(dead_code)]
mod support;
use multi_link_model::{LinksReport, OverviewReport, Row, Timestamp};
use multi_link_tui::{RepoOptions, TimeWindow, render_repo};
use ratatui::{buffer::Buffer, layout::Rect, text::Span};
use std::{fs, path::PathBuf};
use time::UtcOffset;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn report() -> OverviewReport {
    serde_json::from_str(include_str!(
        "../../../fixtures/expected/phase6/overview.json"
    ))
    .unwrap()
}
fn links(report: &OverviewReport) -> LinksReport {
    let mut links: Vec<_> = report
        .repos
        .iter()
        .flat_map(multi_link_tui::view::sessions)
        .flat_map(|r| r.handoffs.clone())
        .collect();
    links.sort_by_key(|l| (l.at, l.invocation.clone()));
    links.dedup_by_key(|l| l.invocation.clone());
    LinksReport {
        links,
        unparented: vec![],
    }
}
fn options() -> RepoOptions {
    RepoOptions {
        offset: UtcOffset::from_hms(-8, 0, 0).unwrap(),
        ..RepoOptions::default()
    }
}
fn render(r: &OverviewReport, o: RepoOptions, window: TimeWindow, width: u16) -> Buffer {
    let mut b = Buffer::empty(Rect::new(0, 0, width, 40));
    render_repo(
        r,
        &links(r),
        "/home/dev/repo-r",
        window,
        Some(0),
        &o,
        b.area,
        &mut b,
    );
    b
}
fn text(b: &Buffer) -> String {
    let mut out = String::new();
    for y in 0..b.area.height {
        let mut line = String::new();
        let mut x = 0;
        while x < b.area.width {
            let s = b[(x, y)].symbol();
            line.push_str(s);
            x += Span::raw(s).width().max(1) as u16;
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}
fn normal() -> Buffer {
    let r = report();
    render(&r, options(), TimeWindow::default_for(&r), 120)
}
#[test]
fn phase6_repo_matches_immutable_text_and_styles() {
    let b = normal();
    assert_eq!(
        text(&b),
        fs::read_to_string(root().join("fixtures/expected/tui/repo-phase6.txt")).unwrap()
    );
    assert_eq!(
        support::dump_styles(&b),
        fs::read_to_string(root().join("fixtures/expected/tui/repo-phase6.styles.txt")).unwrap()
    );
}
#[test]
fn fold_on_off() {
    let r = report();
    let w = TimeWindow::default_for(&r);
    assert!(text(&normal()).contains("42m"));
    let b = render(
        &r,
        RepoOptions {
            fold: false,
            ..options()
        },
        w,
        120,
    );
    assert!(!text(&b).contains("42m"));
    assert!(!text(&b).lines().nth(5).unwrap().contains('≈'));
}
#[test]
fn empty_window() {
    let mut r = report();
    let g = r
        .repos
        .iter_mut()
        .find(|g| g.repo.ends_with("repo-r"))
        .unwrap();
    g.rows.clear();
    let b = render(&r, options(), TimeWindow::default_for(&r), 120);
    assert_eq!(
        text(&b).lines().nth(6).unwrap(),
        "  no recorded activity in the last 2h"
    );
    let w = TimeWindow::new(r.now, r.now);
    let b = render(
        &r,
        RepoOptions {
            fitted: false,
            ..options()
        },
        w,
        120,
    );
    assert_eq!(text(&b).lines().nth(3).unwrap().trim(), "session");
    for y in 5..29 {
        for x in 44..118 {
            assert!(
                !matches!(b[(x, y)].symbol(), "█" | "◆" | "✓"),
                "bar at ({x}, {y})"
            );
        }
    }
}
#[test]
fn confirmed_vs_matched_link_lines() {
    let mut r = report();
    let b = normal();
    assert_eq!(b[(113, 7)].symbol(), "╎");
    let g = r
        .repos
        .iter_mut()
        .find(|g| g.repo.ends_with("repo-r"))
        .unwrap();
    let Row::Session(parent) = &mut g.rows[0] else {
        panic!()
    };
    parent.children[0].handoffs[0].r#match = multi_link_model::Match::Exact;
    let b = render(&r, options(), TimeWindow::default_for(&r), 120);
    assert_eq!(b[(113, 7)].symbol(), "│");
    assert!(text(&b).contains("confirmed · 12 s after the call"));
}
#[test]
fn ambiguous_child_top_level() {
    let mut r = report();
    let g = r
        .repos
        .iter_mut()
        .find(|g| g.repo.ends_with("repo-r"))
        .unwrap();
    let Row::Session(parent) = &mut g.rows[0] else {
        panic!()
    };
    let mut child = parent.children.remove(0);
    let mut link = child.handoffs[0].clone();
    link.r#match = multi_link_model::Match::Ambiguous;
    link.child = None;
    link.candidates = vec![multi_link_model::Candidate {
        id: child.id.clone(),
        gap_ms: 12000,
    }];
    child.handoffs.clear();
    parent.handoffs.push(link);
    g.rows.push(Row::Session(child));
    let b = render(&r, options(), TimeWindow::default_for(&r), 120);
    let s = text(&b);
    assert!(s.lines().nth(6).unwrap().starts_with("▌ codex"));
    assert!(s.lines().nth(7).unwrap().contains("1 possible runs"));
    assert!(!s.contains("└▸"));
}
#[test]
fn bar_clipped_at_window_start() {
    let r = report();
    let w = TimeWindow::new(Timestamp::parse("2026-11-03T06:20:00Z").unwrap(), r.now);
    let b = render(
        &r,
        RepoOptions {
            fitted: false,
            ..options()
        },
        w,
        120,
    );
    assert_eq!(b[(44, 6)].symbol(), "█");
    assert_eq!(b[(44, 12)].symbol(), "│");
}
#[test]
fn imports_on_off() {
    let r = report();
    let w = TimeWindow::default_for(&r);
    assert!(text(&normal()).contains("1 Codex Desktop import at 22:13 hidden"));
    let s = text(&render(
        &r,
        RepoOptions {
            imports: true,
            ..options()
        },
        w,
        120,
    ));
    assert!(!s.contains("hidden (copies"));
    assert!(s.contains("import ·"));
}
#[test]
fn below_80_columns_uses_list_form() {
    let r = report();
    let b = render(&r, options(), TimeWindow::default_for(&r), 79);
    let s = text(&b);
    assert!(s.contains("matched by time · 12 s after the call"));
    assert!(s.contains("Let's pick back up"));
    for y in 5..15 {
        assert!(!(0..79).any(|x| matches!(b[(x, y)].symbol(), "█" | "≈" | "│" | "╎")));
    }
}
#[test]
fn small_sizes_and_embedded_areas() {
    let r = report();
    let l = links(&r);
    for width in [0, 1, 3, 40, 79, 80, 120] {
        for height in [0, 1, 3, 8, 16, 40] {
            let mut b = Buffer::empty(Rect::new(7, 3, width, height));
            let a = b.area;
            render_repo(
                &r,
                &l,
                "/home/dev/repo-r",
                TimeWindow::default_for(&r),
                Some(2),
                &options(),
                a,
                &mut b,
            );
        }
    }
}

#[test]
fn no_color_keeps_repo_words_and_glyphs() {
    let r = report();
    let window = TimeWindow::default_for(&r);
    let color = render(&r, options(), window, 120);
    let plain = render(
        &r,
        RepoOptions {
            theme: multi_link_tui::theme::Theme::from_env(None, Some(std::ffi::OsStr::new("1"))),
            ..options()
        },
        window,
        120,
    );
    assert_eq!(text(&plain), text(&color));
    assert!(
        plain
            .content
            .iter()
            .all(|c| c.fg == ratatui::style::Color::Reset && c.bg == ratatui::style::Color::Reset)
    );
}

#[test]
fn waiting_mark_follows_the_last_recorded_burst() {
    let mut r = report();
    let group = r
        .repos
        .iter_mut()
        .find(|g| g.repo.ends_with("repo-r"))
        .unwrap();
    let Row::Session(parent) = &mut group.rows[0] else {
        panic!()
    };
    parent.state = multi_link_model::State::Waiting;
    let b = render(&r, options(), TimeWindow::default_for(&r), 120);
    assert_eq!(b[(116, 6)].symbol(), "◆");
    assert_eq!(b[(116, 6)].fg, options().theme.needs_you);
}

#[test]
fn empty_and_inverted_explicit_windows_have_no_bars() {
    let r = report();
    for window in [
        TimeWindow::new(r.now, r.now),
        TimeWindow::new(r.now, Timestamp::parse("2026-11-03T06:00:00Z").unwrap()),
    ] {
        let b = render(
            &r,
            RepoOptions {
                fitted: false,
                ..options()
            },
            window,
            120,
        );
        for y in 5..15 {
            assert!(!(44..118).any(|x| matches!(b[(x, y)].symbol(), "█" | "◆" | "✓")));
        }
    }
}

#[test]
fn selected_codex_opens_in_codex() {
    let r = report();
    let mut b = Buffer::empty(Rect::new(0, 0, 120, 40));
    render_repo(
        &r,
        &links(&r),
        "/home/dev/repo-r",
        TimeWindow::default_for(&r),
        Some(1),
        &options(),
        b.area,
        &mut b,
    );
    assert!(
        text(&b)
            .lines()
            .nth(30)
            .unwrap()
            .contains("selected  codex")
    );
    assert!(
        text(&b)
            .lines()
            .nth(31)
            .unwrap()
            .contains("o open in Codex")
    );
    assert!(
        text(&normal())
            .lines()
            .nth(31)
            .unwrap()
            .contains("o open in Claude")
    );
}

#[test]
fn fitted_range_crossing_local_midnight_uses_clock_labels() {
    let mut r = report();
    r.now = Timestamp::parse("2026-11-03T08:13:00Z").unwrap();
    let group = r
        .repos
        .iter_mut()
        .find(|g| g.repo.ends_with("repo-r"))
        .unwrap();
    let Row::Session(mut parent) = group.rows[0].clone() else {
        panic!()
    };
    parent.children.clear();
    parent.activity = vec![multi_link_model::Activity {
        from: Timestamp::parse("2026-11-03T07:40:00Z").unwrap(),
        to: Timestamp::parse("2026-11-03T08:10:00Z").unwrap(),
    }];
    group.rows = vec![Row::Session(parent)];
    let b = render(&r, options(), TimeWindow::default_for(&r), 120);
    let s = text(&b);
    assert!(s.lines().next().unwrap().ends_with("active 23:40–00:10"));
    let labels: Vec<_> = s.lines().nth(3).unwrap().split_whitespace().collect();
    assert_eq!(
        labels,
        ["session", "23:40", "23:50", "00:00", "00:13", "now"]
    );
}
