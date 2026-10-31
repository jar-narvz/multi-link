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

fn phase5() -> OverviewReport {
    load("fixtures/expected/phase5/overview.json")
}
fn phase5_options() -> HomeOptions<'static> {
    HomeOptions {
        offset: time::UtcOffset::from_hms(-7, 0, 0).unwrap(),
        ..HomeOptions::default()
    }
}
#[test]
fn phase5_matches_immutable_text_and_styles() {
    let links =
        serde_json::from_str(include_str!("../../../fixtures/expected/phase5/links.json")).unwrap();
    let options = HomeOptions {
        links: Some(&links),
        ..phase5_options()
    };
    let area = Rect::new(0, 0, 120, 40);
    let mut buf = Buffer::empty(area);
    render_home_with_options(&phase5(), None, &options, area, &mut buf);
    // Deliberately no update-snapshot path for the acceptance pair.
    assert_eq!(
        text(&buf),
        include_str!("../../../fixtures/expected/tui/home-phase5.txt")
    );
    assert_eq!(
        support::dump_styles(&buf),
        include_str!("../../../fixtures/expected/tui/home-phase5.styles.txt")
    );
}

#[test]
fn required_widths_scroll_selection_and_truncate_at_words() {
    let report = phase5();
    let expected = [
        (120, "○ step 4.3 · claude → codex · matched by time"),
        (100, "○ step 4.3 · claude → codex · matched by time"),
        (80, "○ step 4.3 · claude → codex…"),
        (60, "○ step 4.3 · claude…"),
        (40, "○ step 4.3 · claude → codex…"),
    ];
    for (width, label) in expected {
        let area = Rect::new(0, 0, width, 10);
        let mut buf = Buffer::empty(area);
        // Selected step 4.3 must remain visible at a height of ten rows.
        render_home_with_options(&report, Some(4), &phase5_options(), area, &mut buf);
        let screen = text(&buf);
        assert!(
            screen.lines().any(|l| l.contains(label)),
            "width {width}: {screen}"
        );
        assert!(
            screen
                .lines()
                .all(|l| Span::raw(l).width() <= usize::from(width))
        );
        let selected: Vec<_> = (0..10)
            .filter(|y| buf[(3, *y)].bg == phase5_options().theme.selected)
            .collect();
        assert_eq!(selected, [7]);
        if width < 60 {
            assert!(!screen.contains(['━', '█', '◂', '▸']));
        }
        if width < 80 {
            assert!(!screen.lines().next().unwrap().contains("hand-offs"));
        }
        assert!(
            buf.content
                .iter()
                .all(|c| !c.modifier.contains(ratatui::style::Modifier::DIM))
        );
    }
}

#[test]
fn sizes_offsets_and_word_boundaries_are_safe() {
    let mut report = phase5();
    report.repos[0].plan.latest_step.as_mut().unwrap().title = Some("ステップ".repeat(80));
    for (width, height) in [
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
        (60, 4),
        (120, 40),
    ] {
        let area = Rect::new(7, 3, width, height);
        let mut buf = Buffer::empty(Rect::new(0, 0, 140, 50));
        render_home_with_options(&report, Some(0), &phase5_options(), area, &mut buf);
        assert_eq!(buf[(0, 0)].symbol(), " ");
    }
    let area = Rect::new(0, 65533, 3, 2);
    let mut buf = Buffer::empty(area);
    render_home_with_options(&report, None, &phase5_options(), area, &mut buf);
}

#[test]
fn no_color_keeps_glyphs_and_dotted_tracks() {
    let options = HomeOptions {
        theme: multi_link_tui::theme::Theme::from_env(
            Some("truecolor"),
            Some(std::ffi::OsStr::new("1")),
        ),
        ..phase5_options()
    };
    let area = Rect::new(0, 0, 120, 40);
    let mut buf = Buffer::empty(area);
    render_home_with_options(&phase5(), None, &options, area, &mut buf);
    let screen = text(&buf);
    assert!(screen.contains("◆ claude main · waiting"));
    assert!(screen.contains("◂····"));
    assert!(screen.contains("██━"));
    assert!(
        buf.content
            .iter()
            .all(|c| c.fg == ratatui::style::Color::Reset
                && c.bg == ratatui::style::Color::Reset
                && !c.modifier.contains(ratatui::style::Modifier::DIM))
    );
}

#[test]
fn local_axis_handles_midnight_and_fractional_offsets() {
    let mut report = phase5();
    let area = Rect::new(0, 0, 120, 40);
    for (hours, minutes, start, end, footer) in [
        (-7, 0, "14:20", "16:20 now", "UTC-7"),
        (0, 0, "21:20 UTC", "23:20 now", "UTC"),
        (5, 30, "02:50", "04:50 now", "UTC+5:30"),
    ] {
        let options = HomeOptions {
            offset: time::UtcOffset::from_hms(hours, minutes, 0).unwrap(),
            ..HomeOptions::default()
        };
        let mut buf = Buffer::empty(area);
        render_home_with_options(&report, None, &options, area, &mut buf);
        let screen = text(&buf);
        let lines: Vec<_> = screen.lines().collect();
        assert!(lines[2].contains(start), "{}", lines[2]);
        assert!(lines[2].ends_with(end));
        assert!(lines[39].ends_with(footer));
    }
    report.now = "2026-10-30T07:20:00Z".parse().unwrap();
    let mut buf = Buffer::empty(area);
    render_home_with_options(&report, None, &phase5_options(), area, &mut buf);
    assert!(text(&buf).lines().nth(2).unwrap().contains("Oct 29 22:20"));
    let middle: String = (82..87).map(|x| buf[(x, 2)].symbol()).collect();
    assert_eq!(middle, "23:20");
}

#[test]
fn header_counts_every_session_and_only_zero_waiting_is_partial() {
    let mut report = phase5();
    let links =
        serde_json::from_str(include_str!("../../../fixtures/expected/phase5/links.json")).unwrap();
    let options = HomeOptions {
        links: Some(&links),
        ..phase5_options()
    };
    let area = Rect::new(0, 0, 160, 40);
    let header = |report: &OverviewReport| {
        let mut buf = Buffer::empty(area);
        render_home_with_options(report, None, &options, area, &mut buf);
        text(&buf).lines().next().unwrap().to_owned()
    };
    assert!(header(&report).starts_with(
        " All repos · last 2h   ● 1 busy   ○ 7 idle · 10 done or unknown   3 repos · 4 hand-offs"
    ));
    assert!(header(&report).ends_with("◆ 1 need you"));
    let multi_link_model::Row::Session(waiting) = &mut report.repos[0].rows[0] else {
        panic!("session")
    };
    waiting.state = multi_link_model::State::Stale;
    let line = header(&report);
    assert!(line.contains("● 1 busy   ◌ 1 stale"));
    assert!(line.ends_with("◆ needs-you partial: no hook records seen"));
    report.hook.records = 1;
    assert!(!header(&report).contains("◆"));
}

#[test]
fn combined_bar_maps_child_independently_and_placing_step_wins() {
    use multi_link_model::{Row, State};
    let mut report = phase5();
    let mut parent = multi_link_tui::view::sessions(&report.repos[0])
        .find(|r| r.id.ends_with("5011"))
        .unwrap()
        .clone();
    parent.started_at = "2026-10-29T21:20:00Z".parse().unwrap();
    parent.last_activity = "2026-10-29T22:20:00Z".parse().unwrap();
    parent.children[0].started_at = parent.last_activity;
    parent.children[0].last_activity = "2026-10-29T23:30:00Z".parse().unwrap();
    parent.state = State::Busy;
    let mut other = parent.children[0].handoffs[0].clone();
    other.parent = "another-parent".into();
    other.step = Some("9.9".into());
    parent.children[0].handoffs.push(other);
    report.repos.truncate(1);
    report.repos[0].rows = vec![Row::Session(parent.clone())];
    let area = Rect::new(0, 0, 120, 10);
    let mut buf = Buffer::empty(area);
    render_home_with_options(&report, None, &phase5_options(), area, &mut buf);
    let bar: String = (51..117).map(|x| buf[(x, 5)].symbol()).collect();
    assert_eq!(bar, format!("{}{}▸", "━".repeat(33), "█".repeat(32)));
    assert!(!text(&buf).contains("9.9"));
    parent.made = vec!["4.2".into()];
    parent.children[0].last_activity = report.now;
    report.repos[0].rows = vec![Row::Session(parent)];
    render_home_with_options(&report, None, &phase5_options(), area, &mut buf);
    let screen = text(&buf);
    assert!(screen.lines().any(|l| l.contains("● step 4.2 · claude")));
    assert!(
        screen
            .lines()
            .any(|l| l.contains("? step 4.1 · codex · confirmed"))
    );
}

#[test]
fn full_height_widths_keep_whole_fields_and_quit_visible() {
    let report = phase5();
    let links =
        serde_json::from_str(include_str!("../../../fixtures/expected/phase5/links.json")).unwrap();
    let options = HomeOptions {
        links: Some(&links),
        ..phase5_options()
    };
    for width in [120, 100, 80, 60, 40] {
        let area = Rect::new(0, 0, width, 40);
        let mut buf = Buffer::empty(area);
        render_home_with_options(&report, None, &options, area, &mut buf);
        let screen = text(&buf);
        let lines: Vec<_> = screen.lines().collect();
        assert!(
            !screen.contains("·…") && !screen.contains("→…"),
            "width {width}: {screen}"
        );
        assert!(!lines[0].contains('…'), "width {width}: {}", lines[0]);
        assert!(!lines[39].contains('…'), "width {width}: {}", lines[39]);
        assert!(lines[0].ends_with("◆ 1 need you"));
        assert!(lines[39].contains("q quit"));
        for line in &lines {
            for (index, _) in line.match_indices('…') {
                let before = &line[..index];
                assert!(
                    !before.ends_with("latest")
                        && !before.ends_with("no")
                        && !before.ends_with("PLAN.md"),
                    "single-word plan at {width}: {line}"
                );
            }
        }
        let repo_m = lines.iter().find(|line| line.contains("repo-m")).unwrap();
        if width >= 80 {
            assert!(repo_m.contains("last 16:19"), "width {width}: {repo_m}");
        }
        let expected_header = match width {
            120 => {
                "All repos · last 2h   ● 1 busy   ○ 7 idle · 10 done or unknown   3 repos · 4 hand-offs"
            }
            100 => "All repos · last 2h   ● 1 busy   ○ 7 idle · 10 done or unknown   3 repos",
            80 => "All repos · last 2h   ● 1 busy   ○ 7 idle · 10 done or unknown",
            60 => "All repos · last 2h   ● 1 busy   ○ 7 idle",
            40 => "All repos · last 2h",
            _ => unreachable!(),
        };
        assert_eq!(lines[0].split("◆").next().unwrap().trim(), expected_header);
        let expected_keys = match width {
            120 | 100 => {
                "↑↓ move · enter expand / open · t timeline · o open session · f fork · q quit"
            }
            80 => "↑↓ move · enter expand / open · t timeline · q quit",
            60 => "↑↓ move · enter expand / open · q quit",
            40 => "↑↓ move · q quit",
            _ => unreachable!(),
        };
        assert_eq!(
            lines[39].split("times in").next().unwrap().trim(),
            expected_keys
        );
    }
}

#[test]
fn footer_drops_time_only_when_minimum_keys_and_note_cannot_fit() {
    let report = phase5();
    let area = Rect::new(0, 0, 30, 40);
    let mut buf = Buffer::empty(area);
    render_home_with_options(&report, None, &phase5_options(), area, &mut buf);
    assert_eq!(
        text(&buf).lines().nth(39).unwrap().trim(),
        "↑↓ move · q quit"
    );
}
#[test]
fn partial_header_note_survives_narrow_layout_without_ellipsis() {
    let mut report = phase5();
    let multi_link_model::Row::Session(row) = &mut report.repos[0].rows[0] else {
        panic!("session")
    };
    row.state = multi_link_model::State::Idle;
    for width in [120, 100, 80, 60, 40] {
        let area = Rect::new(0, 0, width, 40);
        let mut buf = Buffer::empty(area);
        render_home_with_options(&report, None, &phase5_options(), area, &mut buf);
        let screen = text(&buf);
        let header = screen.lines().next().unwrap();
        assert!(!header.contains('…'));
        assert!(header.ends_with(if width == 40 {
            "◆ partial: no hook records"
        } else {
            "◆ needs-you partial: no hook records seen"
        }));
    }
}
