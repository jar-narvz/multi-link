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
    assert!(text(&normal()).contains("1 Codex Desktop import at 10:13pm hidden"));
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
    assert!(
        s.lines()
            .next()
            .unwrap()
            .ends_with("active 11:40pm–12:10am")
    );
    let labels: Vec<_> = s.lines().nth(3).unwrap().split_whitespace().collect();
    assert_eq!(
        labels,
        ["session", "11:40pm", "11:50pm", "12:00am", "12:13am", "now"]
    );
}

fn completion_time(minute: i64) -> Timestamp {
    Timestamp::from_epoch_ms(
        Timestamp::parse("2026-11-03T05:00:00Z").unwrap().epoch_ms() + minute * 60_000,
    )
    .unwrap()
}
fn completion_report(end: i64) -> OverviewReport {
    let mut r = report();
    r.now = completion_time(74);
    let group = r
        .repos
        .iter_mut()
        .find(|g| g.repo.ends_with("repo-r"))
        .unwrap();
    let Row::Session(parent) = &group.rows[0] else {
        panic!()
    };
    let mut row = parent.children[0].clone();
    row.handoffs.clear();
    row.activity = vec![multi_link_model::Activity {
        from: completion_time(0),
        to: completion_time(end),
    }];
    group.rows = vec![Row::Session(row)];
    r
}
fn completion_render(r: &OverviewReport, mut o: RepoOptions) -> Buffer {
    o.fitted = false;
    render(
        r,
        o,
        TimeWindow::new(completion_time(0), completion_time(74)),
        120,
    )
}
fn cells(b: &Buffer, y: u16, from: u16, to: u16) -> String {
    (from..to).map(|x| b[(x, y)].symbol()).collect()
}

#[test]
fn done_label_with_room_uses_done_colour_and_survives_no_color() {
    // The 74-minute window has exactly 74 bar cells, including an exact-fit case.
    for end in [10, 66] {
        let r = completion_report(end);
        let x = 44 + end as u16;
        let b = completion_render(&r, options());
        assert_eq!(cells(&b, 6, x, x + 8), "█ ✓ done");
        for col in x + 2..x + 8 {
            assert_eq!(b[(col, 6)].fg, options().theme.done);
        }
        let plain = completion_render(
            &r,
            RepoOptions {
                theme: multi_link_tui::theme::Theme::from_env(
                    None,
                    Some(std::ffi::OsStr::new("1")),
                ),
                ..options()
            },
        );
        assert_eq!(text(&plain), text(&b));
        assert!(
            plain
                .content
                .iter()
                .all(|c| c.fg == ratatui::style::Color::Reset)
        );
    }
}

#[test]
fn done_label_without_room_at_edge_keeps_adjacent_check() {
    for end in [67, 68, 72] {
        let r = completion_report(end);
        let b = completion_render(&r, options());
        let x = 44 + end as u16;
        assert_eq!(cells(&b, 6, x, x + 2), "█✓");
        assert_eq!(b[(x + 1, 6)].fg, options().theme.done);
        assert!(!cells(&b, 6, 44, 118).contains("done"));
    }
}

#[test]
fn done_label_with_following_bar_or_link_within_six_cells_keeps_check() {
    let mut r = completion_report(10);
    let group = r
        .repos
        .iter_mut()
        .find(|g| g.repo.ends_with("repo-r"))
        .unwrap();
    let Row::Session(row) = &mut group.rows[0] else {
        panic!()
    };
    // Preserve a later bar even if the bursts arrive out of order.
    row.activity.insert(
        0,
        multi_link_model::Activity {
            from: completion_time(14),
            to: completion_time(15),
        },
    );
    let b = completion_render(&r, options());
    assert_eq!(cells(&b, 6, 54, 60), "█✓  ██");

    // A link to a second child crosses the first child's completion label.
    for kind in [
        multi_link_model::Match::Exact,
        multi_link_model::Match::Heuristic,
    ] {
        let mut r = completion_report(10);
        let group = r
            .repos
            .iter_mut()
            .find(|g| g.repo.ends_with("repo-r"))
            .unwrap();
        let Row::Session(mut parent) = group.rows[0].clone() else {
            panic!()
        };
        let mut first = parent.clone();
        first.id = "first-child".into();
        first.started_at = completion_time(20);
        let mut second = first.clone();
        second.id = "second-child".into();
        second.started_at = completion_time(19);
        second.activity[0].from = completion_time(20);
        second.activity[0].to = completion_time(25);
        let template = report();
        let link = links(&template)
            .links
            .into_iter()
            .find(|l| l.child.is_some())
            .unwrap();
        let mut link = link;
        link.parent = parent.id.clone();
        link.child = Some(second.id.clone());
        link.at = completion_time(14);
        link.r#match = kind;
        second.handoffs = vec![link];
        parent.activity[0].to = completion_time(2);
        parent.children = vec![first, second];
        group.rows = vec![Row::Session(parent)];
        for o in [
            options(),
            RepoOptions {
                theme: multi_link_tui::theme::Theme::from_env(
                    None,
                    Some(std::ffi::OsStr::new("1")),
                ),
                ..options()
            },
        ] {
            let b = completion_render(&r, o);
            assert_eq!(
                cells(&b, 9, 54, 60),
                if kind == multi_link_model::Match::Exact {
                    "█✓  ││"
                } else {
                    "█✓  ╎│"
                }
            );
        }
    }
}

fn axis_report(now: i64, bursts: &[(i64, i64)]) -> OverviewReport {
    let mut r = completion_report(0);
    r.now = completion_time(now);
    let group = r
        .repos
        .iter_mut()
        .find(|g| g.repo.ends_with("repo-r"))
        .unwrap();
    let Row::Session(row) = &mut group.rows[0] else {
        panic!()
    };
    row.activity = bursts
        .iter()
        .map(|&(from, to)| multi_link_model::Activity {
            from: completion_time(from),
            to: completion_time(to),
        })
        .collect();
    r
}
fn axis_words(b: &Buffer) -> Vec<String> {
    cells(b, 3, 44, b.area.width - 2)
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

#[test]
fn fold_near_end_yields_to_full_now_label() {
    let r = axis_report(104, &[(0, 60), (100, 104)]);
    let b = render(&r, options(), TimeWindow::default_for(&r), 120);
    assert_eq!(cells(&b, 3, 110, 118), "44pm now");
    assert_eq!(
        axis_words(&b),
        ["9:00pm", "9:15pm", "9:30pm", "9:45pm", "10:44pm", "now"]
    );
}

#[test]
fn trailing_fold_with_zero_width_end_keeps_now_label() {
    let r = axis_report(102, &[(0, 60)]);
    let b = render(&r, options(), TimeWindow::default_for(&r), 120);
    assert_eq!(cells(&b, 3, 107, 118), "10:42pm now");
    assert_eq!(b[(117, 4)].symbol(), "▼");
    assert_eq!(b[(117, 7)].symbol(), "┃");
    assert_eq!(
        axis_words(&b),
        ["9:00pm", "9:15pm", "9:30pm", "9:45pm", "10:42pm", "now"]
    );
}

#[test]
fn ticks_in_end_minute_are_suppressed_in_fitted_and_explicit_ranges() {
    let mut r = axis_report(0, &[(0, 0)]);
    r.now = Timestamp::from_epoch_ms(completion_time(0).epoch_ms() + 30_000).unwrap();
    let w = TimeWindow::new(completion_time(0), r.now);
    for fitted in [true, false] {
        let b = render(
            &r,
            RepoOptions {
                fitted,
                ..options()
            },
            w,
            120,
        );
        assert_eq!(axis_words(&b), ["9:00pm", "now"]);
    }
}

#[test]
fn explicit_interior_now_precedes_ticks_and_end_clock() {
    // Now reserves its full label first, shifting left near the end.
    for (now, expected) in [
        (
            29,
            vec!["9:00pm", "9:15pm", "9:29pm", "now", "9:45pm", "10:14pm"],
        ),
        (
            30,
            vec!["9:00pm", "9:15pm", "9:30pm", "now", "9:45pm", "10:14pm"],
        ),
        (
            63,
            vec!["9:00pm", "9:15pm", "9:30pm", "9:45pm", "10:03pm", "now"],
        ),
        (
            65,
            vec!["9:00pm", "9:15pm", "9:30pm", "9:45pm", "10:05pm", "now"],
        ),
        (
            68,
            vec!["9:00pm", "9:15pm", "9:30pm", "9:45pm", "10:08pm", "now"],
        ),
    ] {
        let r = axis_report(now, &[(0, 74)]);
        let b = render(
            &r,
            RepoOptions {
                fitted: false,
                ..options()
            },
            TimeWindow::new(completion_time(0), completion_time(74)),
            120,
        );
        assert_eq!(axis_words(&b), expected, "now at minute {now}");
    }
}

#[test]
fn explicit_range_before_now_uses_plain_end_clock() {
    // The wider end clock suppresses the overlapping 10:00pm tick (P4).
    let r = axis_report(90, &[(0, 74)]);
    let b = render(
        &r,
        RepoOptions {
            fitted: false,
            ..options()
        },
        TimeWindow::new(completion_time(0), completion_time(74)),
        120,
    );
    assert_eq!(
        axis_words(&b),
        ["9:00pm", "9:15pm", "9:30pm", "9:45pm", "10:14pm"]
    );
}

// Phase 11: these tests catch incorrect presentation mapping and paint order.
#[test]
fn fitted_now_line_keeps_background_and_avoids_chrome() {
    let b = normal();
    let t = options().theme;
    assert_eq!(b[(117, 4)].symbol(), "▼");
    assert_eq!(b[(117, 4)].fg, ratatui::style::Color::Rgb(0xE5, 0x8A, 0xE5));
    assert_eq!(b[(117, 9)].symbol(), "✓");
    assert_eq!(b[(117, 9)].fg, t.done);
    for y in (5..15).filter(|&y| y != 9) {
        assert_eq!(b[(117, y)].symbol(), "┃", "row {y}");
        assert_eq!(b[(117, y)].fg, ratatui::style::Color::Rgb(0xE5, 0x8A, 0xE5));
    }
    assert_eq!(b[(117, 6)].bg, t.selected);
    assert_eq!(b[(117, 10)].bg, t.child);
    for y in [0, 1, 2, 3, 15, 29, 35, 39] {
        assert_ne!(b[(117, y)].symbol(), "┃");
    }
}

#[test]
fn interior_now_overwrites_link_but_bar_and_done_label_win() {
    let mut r = report();
    r.now = Timestamp::parse("2026-11-03T06:23:00Z").unwrap();
    let b = render(&r, options(), TimeWindow::default_for(&r), 120);
    assert_eq!(b[(117, 7)].symbol(), "┃");
    assert_eq!(b[(117, 6)].symbol(), "█");
    assert_eq!(b[(117, 6)].fg, options().theme.claude);

    let r = axis_report(14, &[(0, 10)]);
    let b = render(
        &r,
        RepoOptions {
            fitted: false,
            ..options()
        },
        TimeWindow::new(completion_time(0), completion_time(74)),
        120,
    );
    assert_eq!(b[(58, 4)].symbol(), "▼");
    assert_eq!(cells(&b, 6, 54, 62), "█ ✓ done");
    assert_eq!(b[(58, 6)].fg, options().theme.done);
    assert_eq!(b[(58, 7)].symbol(), "┃");
}

#[test]
fn now_line_modes_and_outside_windows() {
    use multi_link_tui::theme::Theme;
    use ratatui::style::Color;
    for theme in [
        Theme::from_env(None, None),
        Theme::from_env(None, Some(std::ffi::OsStr::new("1"))),
    ] {
        let r = report();
        let b = render(
            &r,
            RepoOptions { theme, ..options() },
            TimeWindow::default_for(&r),
            120,
        );
        assert_eq!(b[(117, 4)].symbol(), "▼");
        assert_eq!(b[(117, 7)].symbol(), "┃");
        assert_eq!(
            b[(117, 7)].fg,
            if theme.mode == multi_link_tui::theme::ColorMode::Indexed {
                Color::Indexed(176)
            } else {
                Color::Reset
            }
        );
    }
    let r = axis_report(90, &[(0, 60)]);
    for fitted in [false, true] {
        let b = render(
            &r,
            RepoOptions {
                fitted,
                ..options()
            },
            TimeWindow::new(completion_time(0), completion_time(74)),
            120,
        );
        assert!(!text(&b).contains('┃'));
        assert!(!text(&b).contains('▼'));
    }
    let r = report();
    let b = render(&r, options(), TimeWindow::default_for(&r), 79);
    assert!(!text(&b).contains('┃'));
    assert!(!text(&b).contains('▼'));
}

#[test]
fn short_scrolled_screen_limits_now_to_visible_lanes() {
    let r = report();
    let mut b = Buffer::empty(Rect::new(0, 0, 120, 12));
    render_repo(
        &r,
        &links(&r),
        "/home/dev/repo-r",
        TimeWindow::default_for(&r),
        Some(2),
        &options(),
        b.area,
        &mut b,
    );
    assert_eq!(b[(117, 4)].symbol(), "▼");
    for y in 7..10 {
        assert_eq!(b[(117, y)].symbol(), "┃");
    }
    assert_eq!(b[(117, 5)].symbol(), "┃");
    assert_ne!(b[(117, 10)].symbol(), "┃");
    assert_ne!(b[(117, 11)].symbol(), "┃");
    assert!(cells(&b, 6, 0, 44).contains("codex"));
}

#[test]
fn fixture_lane_reaching_now_matches_snapshot() {
    let mut r = report();
    r.now = Timestamp::parse("2026-11-03T06:24:00Z").unwrap();
    let b = render(&r, options(), TimeWindow::default_for(&r), 120);
    assert_eq!(b[(117, 9)].symbol(), "█");
    assert_eq!(b[(117, 9)].fg, options().theme.codex);
    assert_eq!(b[(117, 10)].symbol(), "┃");
    let dir = root().join("fixtures/expected/tui");
    assert_eq!(
        text(&b),
        fs::read_to_string(dir.join("repo-now.txt")).unwrap()
    );
    assert_eq!(
        support::dump_styles(&b),
        fs::read_to_string(dir.join("repo-now.styles.txt")).unwrap()
    );
}

#[test]
fn nearest_run_words_survive_lane_truncation_at_120_and_80() {
    use multi_link_model::{LinkEvidence, Match};
    for (confidence, evidence, words) in [
        (
            Match::Heuristic,
            Some(LinkEvidence::NearestRun),
            "matched by time · nearest run",
        ),
        (Match::Heuristic, None, "matched by time"),
        (
            Match::Heuristic,
            Some(LinkEvidence::ResumeTurn),
            "matched by time",
        ),
        (Match::Exact, Some(LinkEvidence::Stdout), "confirmed"),
    ] {
        let mut r = report();
        let g = r
            .repos
            .iter_mut()
            .find(|g| g.repo.ends_with("repo-r"))
            .unwrap();
        let Row::Session(parent) = &mut g.rows[0] else {
            panic!()
        };
        let link = &mut parent.children[0].handoffs[0];
        link.r#match = confidence;
        link.evidence = evidence;
        assert_eq!(multi_link_tui::view::link_words(link), words);
        for width in [120, 80] {
            let screen = text(&render(&r, options(), TimeWindow::default_for(&r), width));
            assert!(
                screen.contains(&format!("{words} · 12 s after the call")),
                "{width}:\n{screen}"
            );
            assert_eq!(
                screen.contains("nearest run"),
                evidence == Some(LinkEvidence::NearestRun)
            );
        }
    }
}
