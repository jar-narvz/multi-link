//! Home screen snapshots (design T4): render the overview answer keys and compare with reviewed
//! goldens in `fixtures/expected/tui/`. `UPDATE_SNAPSHOTS=1` rewrites the goldens instead.

mod support;

use std::{fs, path::PathBuf};

use multi_link_model::OverviewReport;
use multi_link_tui::{HomeOptions, render_home_with_options};
use ratatui::{buffer::Buffer, layout::Rect, text::Span};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load(path: &str) -> OverviewReport {
    let text = fs::read_to_string(repo_root().join(path)).expect("read overview fixture");
    serde_json::from_str(&text).expect("overview fixture is an OverviewReport")
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
    let options = HomeOptions {
        links: Some(&links),
        ..HomeOptions::default()
    };
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

fn phase6() -> OverviewReport {
    load("fixtures/expected/phase6/overview.json")
}
fn options() -> HomeOptions<'static> {
    HomeOptions {
        offset: time::UtcOffset::from_hms(-8, 0, 0).unwrap(),
        ..HomeOptions::default()
    }
}
fn render(report: &OverviewReport, width: u16, height: u16, selection: Option<usize>) -> Buffer {
    let links = multi_link_model::LinksReport {
        links: report
            .repos
            .iter()
            .filter_map(|g| g.latest_handoff.clone())
            .collect(),
        unparented: vec![],
    };
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    render_home_with_options(
        report,
        selection,
        &HomeOptions {
            links: Some(&links),
            ..options()
        },
        area,
        &mut buf,
    );
    buf
}
#[test]
fn phase6_matches_immutable_text_and_styles() {
    let buf = render(&phase6(), 120, 40, None);
    assert_eq!(
        text(&buf),
        include_str!("../../../fixtures/expected/tui/home-phase6.txt")
    );
    assert_eq!(
        support::dump_styles(&buf),
        include_str!("../../../fixtures/expected/tui/home-phase6.styles.txt")
    );
}
#[test]
fn widths_keep_words_quotes_and_scroll_selected_rows() {
    let report = phase6();
    for width in [120, 100, 80, 60] {
        let buf = render(&report, width, 40, None);
        let screen = text(&buf);
        assert!(screen.contains("q quit"));
        assert!(!screen.contains("·…"));
        assert!(!screen.contains(['█', '━']));
        assert!(!screen.contains("step ?"));
        for line in screen.lines() {
            assert!(Span::raw(line).width() <= usize::from(width));
            assert_eq!(
                line.matches('“').count(),
                line.matches('”').count(),
                "{line}"
            );
        }
        let scrolled = render(&report, width, 10, Some(3));
        assert!(text(&scrolled).contains("claude"));
        assert!((3..8).any(|y| scrolled[(3, y)].bg == options().theme.selected));
    }
}
#[test]
fn imports_yesterday_waiting_and_rest_counts() {
    let screen = text(&render(&phase6(), 120, 40, None));
    assert!(screen.contains("waiting on you · since Nov 1"));
    assert!(screen.contains("4 other sessions, 2 active in the last 2h"));
    assert!(screen.contains("repo-t (only Codex Desktop imports)"));
    assert!(screen.contains("2 Codex Desktop imports hidden"));
    assert!(screen.contains("○ 5 idle · 3 done or unknown"));
}
#[test]
fn no_color_preserves_every_word_and_glyph() {
    let report = phase6();
    let area = Rect::new(0, 0, 120, 40);
    let mut buf = Buffer::empty(area);
    let color = render(&report, 120, 40, None);
    let links = multi_link_model::LinksReport {
        links: report
            .repos
            .iter()
            .filter_map(|g| g.latest_handoff.clone())
            .collect(),
        unparented: vec![],
    };
    render_home_with_options(
        &report,
        None,
        &HomeOptions {
            links: Some(&links),
            theme: multi_link_tui::theme::Theme::from_env(
                Some("truecolor"),
                Some(std::ffi::OsStr::new("1")),
            ),
            ..options()
        },
        area,
        &mut buf,
    );
    assert_eq!(text(&buf), text(&color));
    assert!(
        buf.content
            .iter()
            .all(|c| c.fg == ratatui::style::Color::Reset && c.bg == ratatui::style::Color::Reset)
    );
}
#[test]
fn small_and_offset_areas_are_safe() {
    for (w, h) in [
        (0, 0),
        (0, 5),
        (1, 1),
        (2, 2),
        (3, 3),
        (5, 10),
        (7, 3),
        (10, 1),
        (10, 10),
        (40, 10),
        (60, 3),
    ] {
        let area = Rect::new(7, 3, w, h);
        let mut buf = Buffer::empty(Rect::new(0, 0, 140, 50));
        render_home_with_options(&phase6(), Some(0), &options(), area, &mut buf);
        assert_eq!(buf[(0, 0)].symbol(), " ");
    }
}

#[test]
fn every_target_scrolls_into_view_and_quiet_selection_identifies_one_repo() {
    let report = phase6();
    let targets = multi_link_tui::selectable_home(&report, &Default::default());
    assert_eq!(targets.len(), 7);
    for width in [120, 100, 80, 60] {
        for i in 0..targets.len() {
            let buf = render(&report, width, 10, Some(i));
            assert!(
                (3..8).any(|y| (0..width).any(|x| buf[(x, y)].bg == options().theme.selected)),
                "width {width}, target {i}"
            );
        }
    }
    let imports = render(&report, 120, 40, Some(5));
    assert_eq!(imports[(9, 23)].bg, options().theme.selected);
    assert_ne!(imports[(51, 23)].bg, options().theme.selected);
    let quiet = render(&report, 120, 40, Some(6));
    assert_ne!(quiet[(9, 23)].bg, options().theme.selected);
    assert_eq!(quiet[(51, 23)].bg, options().theme.selected);
}
#[test]
fn unknown_labels_and_untagged_handoffs_do_not_generate_question_marks() {
    let mut report = phase6();
    for group in &mut report.repos {
        for row in &mut group.rows {
            let rows = match row {
                multi_link_model::Row::Session(s) => std::slice::from_mut(s),
                multi_link_model::Row::Merged(m) => m.rows.as_mut_slice(),
            };
            for row in rows {
                row.asked = None;
                row.last_asked = None;
                row.made.clear();
                row.handoffs.clear();
                for child in &mut row.children {
                    child.asked = None;
                    child.last_asked = None;
                    child.handoffs.clear();
                }
            }
        }
        if let Some(link) = &mut group.latest_handoff {
            link.step = None;
        }
    }
    for width in [120, 100, 80, 60] {
        assert!(!text(&render(&report, width, 40, None)).contains('?'));
    }
}

#[test]
fn attention_status_phrases_survive_narrow_widths() {
    let report = phase6();
    for width in [120, 100, 80, 60] {
        let screen = text(&render(&report, width, 80, None));
        let mut lines: Vec<_> = screen
            .lines()
            .filter(|line| line.contains("claude") || line.contains("codex"))
            .collect();
        for row in report
            .repos
            .iter()
            .flat_map(multi_link_tui::view::attention)
        {
            let phrase = multi_link_tui::view::status(row, report.now, options().offset);
            let index = lines
                .iter()
                .position(|line| line.contains(&phrase))
                .unwrap_or_else(|| panic!("width {width}: missing {phrase}"));
            lines.remove(index);
        }
    }
}

#[test]
fn shared_checkout_warnings_have_a_red_header_and_card_row() {
    let report = load("fixtures/expected/overview.json");
    let buf = render(&report, 160, 80, None);
    let screen = text(&buf);
    assert!(
        screen
            .lines()
            .next()
            .unwrap()
            .contains("⚠ 1 shared checkouts")
    );
    let warning_line = screen
        .lines()
        .position(|line| line.contains("⚠ shared checkout: repo-c"))
        .unwrap();
    assert!(
        screen
            .lines()
            .nth(warning_line - 1)
            .unwrap()
            .contains("claude")
    );
    for (y, words) in [
        (0, "⚠ 1 shared checkouts"),
        (warning_line, "⚠ shared checkout: repo-c"),
    ] {
        let line = screen.lines().nth(y).unwrap();
        let byte = line.find(words).unwrap();
        let x = Span::raw(&line[..byte]).width() as u16;
        assert_eq!(buf[(x, y as u16)].fg, options().theme.warning);
    }
    let clean = text(&render(&phase6(), 160, 80, None));
    assert!(!clean.contains("shared checkout"));
}

#[test]
fn footer_at_80_keeps_navigation_before_import_note() {
    let buf = render(&phase6(), 80, 40, None);
    let screen = text(&buf);
    let footer = screen.lines().last().unwrap();
    for words in [
        "↑↓ move",
        "enter repo view",
        "o open session",
        "q quit",
        "times UTC-8",
    ] {
        assert!(footer.contains(words), "missing {words}: {footer}");
    }
    assert!(!footer.contains("imports hidden"));
}

#[test]
fn header_at_100_keeps_working_now_before_shared_checkouts() {
    let report = load("fixtures/expected/overview.json");
    let screen = text(&render(&report, 100, 30, None));
    let header = screen.lines().next().unwrap();
    assert!(header.contains("● 2 working now"), "{header}");
    assert!(!header.contains("shared checkouts"), "{header}");
}

#[test]
fn attention_status_columns_align_within_each_card() {
    let report = phase6();
    for width in [120, 80, 60] {
        let screen = text(&render(&report, width, 80, None));
        for group in &report.repos {
            let rows = multi_link_tui::view::attention(group);
            if rows.is_empty() {
                continue;
            }
            let name = group.repo.rsplit('/').next().unwrap();
            let mut card = screen
                .lines()
                .skip_while(|line| !line.contains(name))
                .take_while(|line| !line.contains('╰'));
            let mut columns = Vec::new();
            for row in rows {
                let phrase = multi_link_tui::view::status(row, report.now, options().offset);
                let line = card
                    .find(|line| line.contains(&phrase))
                    .unwrap_or_else(|| panic!("width {width}, {name}: missing {phrase}"));
                let byte = line.find(&phrase).unwrap();
                columns.push(Span::raw(&line[..byte]).width());
            }
            assert!(
                columns.iter().all(|x| *x == columns[0]),
                "width {width}, {name}: {columns:?}"
            );
            if width == 120 {
                assert_eq!(columns[0], 66, "{name}");
            }
        }
    }
}

#[test]
fn home_names_the_window_in_header_counts_and_activity_summary() {
    let mut report = phase6();
    for hours in [2, 16] {
        let options = HomeOptions {
            window: Some(multi_link_tui::TimeWindow::ending_at(
                report.now,
                std::time::Duration::from_secs(hours * 3600),
            )),
            ..options()
        };
        let area = Rect::new(0, 0, 160, 80);
        let mut buf = Buffer::empty(area);
        render_home_with_options(&report, None, &options, area, &mut buf);
        let screen = text(&buf);
        assert!(screen.contains(&format!("All repos · last {hours}h")));
        assert!(
            screen.contains(&format!(
                "4 other sessions, 2 active in the last {hours}h · enter shows all"
            )),
            "{screen}"
        );
        assert!(
            screen.contains(&format!(
                "active about 13 min in the last {hours}h, in 3 stretches"
            )),
            "{screen}"
        );
        if hours == 16 {
            assert!(!screen.contains("last 2h"));
        }
    }
    // The count line also names the window when none of the hidden rows was active.
    for row in &mut report.repos[0].rows {
        if let multi_link_model::Row::Session(row) = row {
            row.activity.clear();
            for child in &mut row.children {
                child.activity.clear();
            }
        }
    }
    let area = Rect::new(0, 0, 160, 80);
    let mut buf = Buffer::empty(area);
    render_home_with_options(
        &report,
        None,
        &HomeOptions {
            window: Some(multi_link_tui::TimeWindow::ending_at(
                report.now,
                std::time::Duration::from_secs(16 * 3600),
            )),
            ..options()
        },
        area,
        &mut buf,
    );
    assert!(text(&buf).contains("4 older sessions, none active in the last 16h · enter shows all"));
}
