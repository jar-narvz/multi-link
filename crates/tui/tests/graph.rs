//! Board 9 contract tests. Immutable targets have no snapshot update path.
#[allow(dead_code)]
mod support;

use multi_link_model::{Edge, GraphCommit, GraphReport, Notable, PartStatus, Timestamp};
use multi_link_tui::{GraphOptions, graph_first_slot, graph_slots, render_graph, theme::Theme};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier},
    text::Span,
};
use std::{fs, ops::RangeInclusive, path::PathBuf};
use time::UtcOffset;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn example_draws_graph_html() {
    let workspace = root();
    let temp = std::env::temp_dir().join(format!(
        "multi-link-graph-html-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&temp).unwrap();
    let out = temp.join("out.html");
    let output = std::process::Command::new(workspace.join("target/debug/examples/home_html"))
        .current_dir(&workspace)
        .arg("--graph")
        .arg("fixtures/expected/phase8/graph-1.1.json")
        .arg(&out)
        .args(["--size", "120x40", "--offset", "-8", "--selected", "1"])
        .output()
        .expect("run the home_html example built by cargo test");
    let html = fs::read_to_string(&out);
    fs::remove_dir_all(&temp).unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "home_html failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let html = html.expect("home_html writes out.html");
    let mut in_tag = false;
    let rendered_text: String = html
        .chars()
        .filter(|&ch| match ch {
            '<' => {
                in_tag = true;
                false
            }
            '>' => {
                in_tag = false;
                false
            }
            _ => !in_tag,
        })
        .collect();
    for expected in ["step 1.1", "implement", "where the time went"] {
        assert!(rendered_text.contains(expected), "missing text: {expected}");
    }
    assert!(html.contains("#1B2130"), "missing selected background");
}

fn report(id: &str) -> GraphReport {
    let path = root().join(format!("fixtures/expected/phase8/graph-{id}.json"));
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn golden_options() -> GraphOptions {
    GraphOptions {
        selected: 1,
        first_slot: 0,
        theme: Theme::from_env(Some("truecolor"), None),
        offset: UtcOffset::from_hms(-8, 0, 0).unwrap(),
    }
}

#[test]
fn panel_empty_quotes_are_absent() {
    let mut report = report("1.1");
    report.parts[1].asked = Some(String::new());
    report.parts[1].said = Some(String::new());
    let buf = render(&report, &golden_options(), 120, 40);
    assert_eq!(rendered_rows(&buf, 28..=28), "\n");
    assert_eq!(rendered_rows(&buf, 30..=30), "\n");
}

#[test]
fn panel_empty_session_is_rendered_like_no_session() {
    for selected in [0, 1] {
        let mut report = report("1.1");
        let options = GraphOptions {
            selected,
            ..golden_options()
        };
        report.parts[selected].session = None;
        let absent = render(&report, &options, 120, 40);
        report.parts[selected].session = Some(String::new());
        let empty = render(&report, &options, 120, 40);
        assert_eq!(
            rendered_rows(&empty, 27..=27),
            rendered_rows(&absent, 27..=27)
        );
        assert_eq!(rendered_rows(&empty, 31..=31), "\n");
    }
}

#[test]
fn quote_controls_are_removed_before_measuring_width() {
    let mut report = report("1.1");
    let quote = format!("\u{1b}[31m{} tail", "x".repeat(100));
    report.parts[1].asked = Some(quote.clone());
    report.parts[1].said = Some(quote);
    let buf = render(&report, &golden_options(), 120, 40);
    for (y, label) in [(28, "asked"), (30, "said ")] {
        let row = rendered_rows(&buf, y..=y);
        assert_eq!(
            row,
            format!("            {label}  “[31m{}…”\n", "x".repeat(83))
        );
        assert!(!row.contains('\u{1b}'));
        assert_eq!(buf[(108, y as u16)].symbol(), "”");
        assert_eq!(buf[(109, y as u16)].symbol(), " ");
    }
}

#[test]
fn title_controls_are_removed_before_width_cutting() {
    let mut report = report("1.1");
    report.step.title = Some("T".repeat(200));
    let plain = render(&report, &golden_options(), 120, 40);
    report.step.title = Some(format!("\u{1b}[31m{}", "T".repeat(200)));
    let controlled = render(&report, &golden_options(), 120, 40);
    report.step.title = Some(format!("[31m{}", "T".repeat(200)));
    let stripped = render(&report, &golden_options(), 120, 40);
    assert_ne!(
        rendered_rows(&plain, 0..=0),
        rendered_rows(&stripped, 0..=0)
    );
    assert_eq!(
        rendered_rows(&controlled, 0..=0),
        rendered_rows(&stripped, 0..=0)
    );
}

fn render(report: &GraphReport, options: &GraphOptions, w: u16, h: u16) -> Buffer {
    let mut buf = Buffer::empty(Rect::new(0, 0, w, h));
    render_graph(report, options, buf.area, &mut buf);
    buf
}

fn text(buf: &Buffer) -> String {
    let mut out = String::new();
    for y in 0..buf.area.height {
        let mut line = String::new();
        let mut x = 0;
        while x < buf.area.width {
            let s = buf[(x, y)].symbol();
            line.push_str(s);
            x += Span::raw(s).width().max(1) as u16;
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

fn text_rows(text: &str, rows: RangeInclusive<usize>) -> String {
    text.lines()
        .enumerate()
        .filter(|(row, _)| rows.contains(row))
        .map(|(_, line)| format!("{line}\n"))
        .collect()
}

fn style_rows(styles: &str, rows: RangeInclusive<usize>) -> String {
    styles
        .lines()
        .skip(1) // The style dump's header is not a cell row.
        .filter(|line| {
            let row: usize = line.split_whitespace().next().unwrap().parse().unwrap();
            rows.contains(&row)
        })
        .map(|line| format!("{line}\n"))
        .collect()
}

fn golden_rows(rows: RangeInclusive<usize>) -> String {
    text_rows(
        include_str!("../../../fixtures/expected/tui/step-phase8.txt"),
        rows,
    )
}

fn golden_styles(rows: RangeInclusive<usize>) -> String {
    style_rows(
        include_str!("../../../fixtures/expected/tui/step-phase8.styles.txt"),
        rows,
    )
}

fn rendered_rows(buf: &Buffer, rows: RangeInclusive<usize>) -> String {
    text_rows(&text(buf), rows)
}

fn rendered_styles(buf: &Buffer, rows: RangeInclusive<usize>) -> String {
    style_rows(&support::dump_styles(buf), rows)
}

#[test]
fn phase8_render_equals_golden() {
    let buf = render(&report("1.1"), &golden_options(), 120, 40);
    assert_eq!(
        text(&buf),
        include_str!("../../../fixtures/expected/tui/step-phase8.txt")
    );
    assert_eq!(
        support::dump_styles(&buf),
        include_str!("../../../fixtures/expected/tui/step-phase8.styles.txt")
    );
}

#[test]
fn strip_widths() {
    let options = golden_options();
    let buf = render(&report("1.1"), &options, 120, 40);
    for (first, last, label, background, foreground) in [
        (
            23,
            30,
            "plan",
            options.theme.claude_strip,
            options.theme.claude,
        ),
        (
            32,
            63,
            "implement",
            options.theme.graph_selected,
            options.theme.codex,
        ),
        (
            65,
            77,
            "review",
            options.theme.claude_strip,
            options.theme.claude,
        ),
        (
            79,
            88,
            "fix",
            options.theme.codex_strip,
            options.theme.codex,
        ),
        (
            90,
            116,
            "verify + commit",
            options.theme.claude_strip,
            options.theme.claude,
        ),
    ] {
        for x in first..=last {
            assert_eq!(buf[(x, 33)].bg, background, "strip fill at {x}");
        }
        assert_eq!(buf[(first - 1, 33)].bg, options.theme.screen);
        assert_eq!(buf[(last + 1, 33)].bg, options.theme.screen);
        assert_eq!(buf[(first - 1, 33)].symbol(), " ");
        assert_eq!(buf[(last + 1, 33)].symbol(), " ");
        assert_colored_text(&buf, first + 1, 33, label, foreground);
    }
    assert_eq!(rendered_rows(&buf, 33..=34), golden_rows(33..=34));
    assert_colored_text(&buf, 23, 34, "08:05", options.theme.secondary);
    assert_colored_text(&buf, 112, 34, "08:12", options.theme.secondary);
}

#[test]
fn strip_with_a_one_second_part() {
    let mut report = report("1.1");
    report.parts[0].to =
        Some(Timestamp::from_epoch_ms(report.parts[0].from.unwrap().epoch_ms() + 1_000).unwrap());
    let options = golden_options();
    let buf = render(&report, &options, 120, 40);
    // With 90 filled cells, the label minima use 47 and duration shares use 43.
    // The one-second plan receives no extra cell: its width is exactly 4 + 2.
    for (first, last, label, background, foreground) in [
        (
            23,
            28,
            "plan",
            options.theme.claude_strip,
            options.theme.claude,
        ),
        (
            30,
            62,
            "implement",
            options.theme.graph_selected,
            options.theme.codex,
        ),
        (
            64,
            77,
            "review",
            options.theme.claude_strip,
            options.theme.claude,
        ),
        (
            79,
            88,
            "fix",
            options.theme.codex_strip,
            options.theme.codex,
        ),
        (
            90,
            116,
            "verify + commit",
            options.theme.claude_strip,
            options.theme.claude,
        ),
    ] {
        for x in first..=last {
            assert_eq!(buf[(x, 33)].bg, background, "strip fill at {x}");
        }
        assert_eq!(buf[(first - 1, 33)].bg, options.theme.screen);
        assert_eq!(buf[(last + 1, 33)].bg, options.theme.screen);
        assert_colored_text(&buf, first + 1, 33, label, foreground);
    }
    assert_colored_text(&buf, 23, 34, "08:05", options.theme.secondary);
    assert_colored_text(&buf, 112, 34, "08:12", options.theme.secondary);
}

#[test]
fn strip_without_room_for_labels() {
    let mut report = report("1.1");
    report.parts = report.parts.iter().cloned().cycle().take(40).collect();
    let options = golden_options();
    let buf = render(&report, &options, 120, 40);
    let mut x = 23;
    let mut widths = Vec::new();
    for (index, part) in report.parts.iter().enumerate() {
        let background = if index == options.selected {
            options.theme.graph_selected
        } else if part.harness == report.parts[0].harness {
            options.theme.claude_strip
        } else {
            options.theme.codex_strip
        };
        let first = x;
        while x <= 116 && buf[(x, 33)].bg == background {
            assert_eq!(buf[(x, 33)].symbol(), " ", "no label at {x}");
            x += 1;
        }
        assert!(x > first, "part {index} must receive at least one cell");
        widths.push(x - first);
        assert_eq!(
            buf[(x, 33)].bg,
            options.theme.screen,
            "gap after part {index}"
        );
        assert_eq!(buf[(x, 33)].symbol(), " ");
        if index < 39 {
            x += 1;
        }
    }
    assert_eq!(widths.len(), 40);
    assert_eq!(
        widths,
        [
            1, 2, 1, 1, 2, 1, 2, 1, 1, 2, 1, 2, 1, 1, 2, 1, 2, 1, 1, 2, 1, 2, 1, 1, 2, 1, 2, 1, 1,
            2, 1, 2, 1, 1, 2, 1, 2, 1, 1, 1,
        ]
    );
    assert_eq!(widths.iter().sum::<u16>(), 55);
    assert_eq!(x, 117, "strip must end at column 116");
    assert_eq!(buf[(22, 33)].bg, options.theme.screen);
    assert_colored_text(&buf, 23, 34, "08:05", options.theme.secondary);
    assert_colored_text(&buf, 112, 34, "08:12", options.theme.secondary);
}

fn assert_strip_segments(buf: &Buffer, expected: &[(u16, Color)], screen: Color) {
    let mut x = 23;
    assert_eq!(buf[(22, 33)].bg, screen);
    for (width, background) in expected {
        for _ in 0..*width {
            assert_eq!(buf[(x, 33)].bg, *background, "strip fill at {x}");
            x += 1;
        }
        assert_eq!(buf[(x, 33)].bg, screen, "strip gap at {x}");
        assert_eq!(buf[(x, 33)].symbol(), " ");
        x += 1;
    }
    assert_eq!(x, 118, "last filled cell must be column 116");
}

#[test]
fn strip_with_too_many_parts_keeps_title_and_times_only() {
    let mut report = report("1.1");
    report.parts = report.parts.iter().cloned().cycle().take(60).collect();
    let options = golden_options();
    let buf = render(&report, &options, 120, 40);
    assert_colored_text(&buf, 2, 33, "where the time went", options.theme.secondary);
    for y in 33..=34 {
        for x in 23..120 {
            assert_eq!(buf[(x, y)].bg, options.theme.screen);
            if y == 33 || x > 116 {
                assert_eq!(buf[(x, y)].symbol(), " ", "cell {x},{y}");
            }
        }
    }
    assert_colored_text(&buf, 23, 34, "08:05", options.theme.secondary);
    assert_colored_text(&buf, 112, 34, "08:12", options.theme.secondary);
}

#[test]
fn strip_times_cover_all_parts_and_skip_missing_timestamps() {
    let mut report = report("1.1");
    report.parts[0].from = Some(Timestamp::parse("2026-11-06T16:00:00Z").unwrap());
    report.parts[0].to = Some(Timestamp::parse("2026-11-06T16:20:00Z").unwrap());
    report.parts[4].from = Some(Timestamp::parse("2026-11-06T16:05:00Z").unwrap());
    report.parts[4].to = Some(Timestamp::parse("2026-11-06T16:10:00Z").unwrap());
    let options = golden_options();
    let buf = render(&report, &options, 120, 40);
    assert_colored_text(&buf, 23, 34, "08:00", options.theme.secondary);
    assert_colored_text(&buf, 112, 34, "08:20", options.theme.secondary);

    report.parts[0].from = None;
    report.parts[4].to = None;
    let buf = render(&report, &options, 120, 40);
    assert_colored_text(&buf, 23, 34, "08:05", options.theme.secondary);
    assert_colored_text(&buf, 112, 34, "08:20", options.theme.secondary);
}

#[test]
fn strip_zero_length_parts_share_extra_cells_equally() {
    let mut report = report("1.1");
    for part in &mut report.parts {
        part.to = part.from;
    }
    let options = golden_options();
    let t = options.theme;
    let buf = render(&report, &options, 120, 40);
    // Label minima total 47; split the 43 extra cells as 9, 9, 9, 8, 8.
    assert_strip_segments(
        &buf,
        &[
            (15, t.claude_strip),
            (20, t.graph_selected),
            (17, t.claude_strip),
            (13, t.codex_strip),
            (25, t.claude_strip),
        ],
        t.screen,
    );
}

#[test]
fn strip_without_labels_shares_extra_cells_by_length() {
    let mut report = report("1.1");
    report.parts.truncate(3);
    for (index, part) in report.parts.iter_mut().enumerate() {
        part.kind = "long label ".repeat(4);
        part.to = Some(
            Timestamp::from_epoch_ms(part.from.unwrap().epoch_ms() + (index as i64 + 1) * 1_000)
                .unwrap(),
        );
    }
    let options = golden_options();
    let t = options.theme;
    let buf = render(&report, &options, 120, 40);
    // Three one-cell minima leave 89 cells in a 1:2:3 ratio. Remainders
    // award the last two cells to the first and second parts.
    assert_strip_segments(
        &buf,
        &[
            (16, t.claude_strip),
            (31, t.graph_selected),
            (45, t.claude_strip),
        ],
        t.screen,
    );
    for x in 23..=116 {
        assert_eq!(buf[(x, 33)].symbol(), " ", "no label at {x}");
    }
}

#[test]
fn strip_unlinked_part_keeps_label_minimum_and_selected_claude_colour() {
    let options = GraphOptions {
        selected: 0,
        ..golden_options()
    };
    let t = options.theme;
    let buf = render(&report("1.2"), &options, 120, 40);
    assert_strip_segments(
        &buf,
        &[
            (10, t.graph_selected),
            (11, t.codex_strip),
            (43, t.claude_strip),
            (5, t.codex_strip),
            (21, t.claude_strip),
        ],
        t.screen,
    );
    assert_colored_text(&buf, 24, 33, "plan", t.claude);
    assert_colored_text(&buf, 35, 33, "implement", t.codex);
}

#[test]
fn header_footer_and_legend_match_golden() {
    let buf = render(&report("1.1"), &golden_options(), 120, 40);
    for rows in [0..=3, 36..=39] {
        assert_eq!(rendered_rows(&buf, rows.clone()), golden_rows(rows.clone()));
        assert_eq!(rendered_styles(&buf, rows.clone()), golden_styles(rows));
    }
}

#[test]
fn thirty_nine_rows_drop_the_strip() {
    let report = report("1.1");
    let options = golden_options();
    let full = render(&report, &options, 120, 40);
    let short = render(&report, &options, 120, 39);

    assert!(!text(&short).contains("where the time went"));
    assert!(text(&short).contains(rendered_rows(&full, 36..=36).trim_end()));
    assert_eq!(
        rendered_rows(&short, 35..=35),
        rendered_rows(&full, 36..=36)
    );
    assert_eq!(rendered_rows(&short, 36..=36), "\n");
    assert_eq!(
        rendered_rows(&short, 38..=38),
        rendered_rows(&full, 39..=39)
    );
    assert_eq!(
        rendered_rows(&short, 37..=37),
        rendered_rows(&full, 38..=38)
    );
    assert_eq!(
        rendered_rows(&short, 27..=31),
        rendered_rows(&full, 27..=31)
    );
    assert_eq!(
        rendered_styles(&short, 27..=31),
        rendered_styles(&full, 27..=31)
    );
}

#[test]
fn thirty_rows_keep_the_panel_first_line() {
    let report = report("1.1");
    let options = golden_options();
    let full = render(&report, &options, 120, 40);
    let short = render(&report, &options, 120, 30);
    let short_text = text(&short);

    assert!(!short_text.contains("where the time went"));
    assert!(!short_text.contains("▭ claude part"));
    assert!(rendered_rows(&short, 27..=27).contains("selected"));
    assert_eq!(
        rendered_rows(&short, 27..=27),
        rendered_rows(&full, 27..=27)
    );
    for label in ["asked", "from", "said", "o open"] {
        assert!(
            !rendered_rows(&short, 27..=28).contains(label),
            "unexpected {label}"
        );
    }
    assert_eq!(
        rendered_rows(&short, 29..=29),
        rendered_rows(&full, 39..=39)
    );
    assert_eq!(
        rendered_rows(&short, 28..=28),
        rendered_rows(&full, 38..=38)
    );
    assert_eq!(rendered_rows(&short, 4..=24), rendered_rows(&full, 4..=24));
    assert_eq!(
        rendered_styles(&short, 4..=24),
        rendered_styles(&full, 4..=24)
    );
}

#[test]
fn no_color_keeps_glyphs_and_words() {
    let report = report("1.1");
    let options = golden_options();
    let full = render(&report, &options, 120, 40);
    let plain = render(
        &report,
        &GraphOptions {
            theme: Theme::from_env(None, Some(std::ffi::OsStr::new("1"))),
            ..options
        },
        120,
        40,
    );

    assert_eq!(text(&plain), text(&full));
    for y in 0..40 {
        for x in 0..120 {
            let cell = &plain[(x, y)];
            assert_eq!(cell.fg, Color::Reset, "foreground at ({x}, {y})");
            assert_eq!(cell.bg, Color::Reset, "background at ({x}, {y})");
            assert!(!cell.modifier.contains(Modifier::DIM), "DIM at ({x}, {y})");
        }
    }
}

#[test]
fn tiny_buffers_do_not_panic() {
    let report = report("1.1");
    let options = golden_options();
    for (width, height) in [(120, 10), (120, 5), (10, 40), (1, 1), (0, 0)] {
        render(&report, &options, width, height);
    }
}

#[test]
fn empty_step_shows_the_sentence() {
    let mut report = report("1.13");
    let options = golden_options();
    for verify in [Some("cargo test --workspace".to_owned()), None] {
        report.step.verify = verify;
        let buf = render(&report, &options, 120, 40);
        assert_eq!(
            rendered_rows(&buf, 0..=0),
            format!(
                " repo-g › step 1.13  {}\n",
                report.step.title.as_deref().unwrap()
            )
        );
        assert_eq!(rendered_rows(&buf, 2..=2), golden_rows(2..=2));
        assert_eq!(rendered_styles(&buf, 2..=2), golden_styles(2..=2));
        for x in 0..120 {
            if buf[(x, 1)].symbol() != " " {
                assert_eq!(buf[(x, 1)].fg, options.theme.secondary);
            }
        }
        assert_eq!(
            rendered_rows(&buf, 4..=4),
            format!("  {}\n", report.empty.as_deref().unwrap())
        );
        let sentence_width = Span::raw(report.empty.as_deref().unwrap()).width() as u16;
        for x in 2..2 + sentence_width {
            assert_eq!(buf[(x, 4)].fg, options.theme.secondary);
            assert_eq!(buf[(x, 4)].modifier, Modifier::empty());
        }
        let expected_verify = report
            .step
            .verify
            .as_ref()
            .map(|verify| format!("{:>119}\n", format!("Verify  {verify} (reported)")))
            .unwrap_or_else(|| "\n".to_owned());
        assert_eq!(rendered_rows(&buf, 1..=1), expected_verify);
        for y in 3..=37 {
            if y != 4 {
                assert_eq!(rendered_rows(&buf, y..=y), "\n", "row {y}");
            }
        }
        assert_eq!(rendered_rows(&buf, 38..=39), golden_rows(38..=39));
        assert_eq!(rendered_styles(&buf, 38..=39), golden_styles(38..=39));
        assert!(
            buf.content
                .iter()
                .all(|cell| cell.bg == options.theme.screen)
        );
    }
}

#[test]
fn commit_is_green_right_aligned_and_leaves_two_blanks_after_cut_title() {
    let mut report = report("1.1");
    report.commit = Some(GraphCommit {
        hash: "abc1234".to_owned(),
        at: Timestamp::parse("2026-11-06T16:20:00Z").unwrap(),
    });
    let options = golden_options();
    let original_title = report.step.title.clone().unwrap();
    for title in [original_title, "T".repeat(200)] {
        let long_title = title.len() == 200;
        report.step.title = Some(title);
        let buf = render(&report, &options, 120, 40);
        let row = rendered_rows(&buf, 0..=0);
        let row = row.trim_end_matches('\n');
        let commit = "✓ committed abc1234 · 08:20";
        assert!(row.ends_with(commit));
        assert_eq!(Span::raw(row).width(), 119);
        assert_eq!(buf[(119, 0)].symbol(), " ");
        let first = 119 - Span::raw(commit).width() as u16;
        for x in first..119 {
            assert_eq!(buf[(x, 0)].fg, options.theme.done, "column {x}");
        }
        if long_title {
            let left = row.strip_suffix(commit).unwrap();
            assert!(left.trim_end().ends_with('…'));
            assert_eq!(left.chars().rev().take_while(|c| *c == ' ').count(), 2);
            assert!(left.starts_with(" repo-g › step 1.1  T"));
        }
    }
}

#[test]
fn zero_length_range_shows_at_least_one_second() {
    let mut report = report("1.1");
    let instant = Timestamp::parse("2026-11-06T16:05:26Z").unwrap();
    for part in &mut report.parts {
        part.from = Some(instant);
        part.to = Some(instant);
    }
    let buf = render(&report, &golden_options(), 120, 40);
    assert!(rendered_rows(&buf, 1..=1).contains("· 1 s ·"));
}

#[test]
fn long_title_and_verify_are_cut_not_overlapped() {
    let mut report = report("1.1");
    report.step.title = Some("T".repeat(200));
    report.step.verify = Some("V".repeat(200));
    let buf = render(&report, &golden_options(), 120, 40);
    let rendered = text(&buf);
    let rows: Vec<_> = rendered.lines().collect();
    for (y, row) in rows.iter().enumerate().take(2) {
        assert!(Span::raw(*row).width() <= 119, "row {y}: {row}");
        assert_eq!(buf[(0, y as u16)].symbol(), " ");
        assert_eq!(buf[(119, y as u16)].symbol(), " ");
    }
    assert!(rows[0].starts_with(" repo-g › step 1.1  T"));
    assert!(rows[0].ends_with('…'));
    let (left, verify) = rows[1].split_once("Verify  ").unwrap();
    let golden = golden_rows(1..=1);
    let expected_left = golden.split_once("Verify  ").unwrap().0;
    assert_eq!(left.trim_end(), expected_left.trim_end());
    assert_eq!(left.chars().rev().take_while(|c| *c == ' ').count(), 2);
    assert!(verify.starts_with('V'));
    assert!(verify.ends_with("… (reported)"));
    assert_eq!(Span::raw(rows[1]).width(), 119);
}

#[test]
fn default_footer_uses_the_shared_utc_label() {
    let buf = render(&report("1.1"), &GraphOptions::default(), 120, 40);
    assert!(
        rendered_rows(&buf, 39..=39)
            .trim_end()
            .ends_with("times UTC")
    );
}

#[test]
fn long_repo_name_is_cut_before_the_step_label() {
    let mut report = report("1.1");
    report.repo = format!("/x/{}", "r".repeat(200));
    let buf = render(&report, &golden_options(), 120, 40);
    let row = rendered_rows(&buf, 0..=0);
    let (basename, rest) = row.split_once(" › step 1.1").unwrap();
    assert!(basename.starts_with(" r"));
    assert!(basename.ends_with('…'));
    assert!(rest.starts_with("  "));
    assert!(Span::raw(row.trim_end()).width() <= 119);
}

#[test]
fn verify_is_omitted_without_room_for_a_character_and_ellipsis() {
    let mut report = report("1.13");
    report.step.verify = Some("V".repeat(200));
    // Empty reports have no summary: widths 21/22/23 leave 19/20/21 cells
    // inside the margins, so the command gets zero/one/two cells.
    for width in [21, 22] {
        let buf = render(&report, &golden_options(), width, 40);
        assert_eq!(rendered_rows(&buf, 1..=1), "\n", "width {width}");
    }
    let buf = render(&report, &golden_options(), 23, 40);
    assert_eq!(rendered_rows(&buf, 1..=1), " Verify  V… (reported)\n");
}

#[test]
fn graph_theme_roles() {
    let truecolor = Theme::from_env(Some("truecolor"), None);
    assert_eq!(truecolor.graph_selected, Color::Rgb(0x1B, 0x21, 0x30));
    assert_eq!(truecolor.codex_box, Color::Rgb(0x12, 0x1A, 0x26));
    assert_eq!(truecolor.claude_strip, Color::Rgb(0x3A, 0x2A, 0x1F));
    assert_eq!(truecolor.codex_strip, Color::Rgb(0x1E, 0x2A, 0x3D));

    let indexed = Theme::from_env(None, None);
    assert_eq!(indexed.graph_selected, Color::Indexed(235));
    assert_eq!(indexed.codex_box, Color::Indexed(234));
    assert_eq!(indexed.claude_strip, Color::Indexed(236));
    assert_eq!(indexed.codex_strip, Color::Indexed(236));
}

#[test]
fn lanes_and_edges_match_golden() {
    let buf = render(&report("1.1"), &golden_options(), 120, 40);
    assert_eq!(rendered_rows(&buf, 3..=24), golden_rows(3..=24));
    assert_eq!(rendered_styles(&buf, 3..=24), golden_styles(3..=24));
}

#[test]
fn slot_rule_with_same_harness_neighbours() {
    for (id, expected) in [
        ("1.1", vec![0, 1, 2, 3, 4]),
        ("1.5", vec![0, 1, 3]),
        ("1.4", vec![0, 1, 2, 4, 5, 6]),
    ] {
        assert_eq!(graph_slots(&report(id)), expected, "report {id}");
    }
    let buf = render(&report("1.5"), &golden_options(), 120, 40);
    for x in [23, 67] {
        assert_eq!(buf[(x, 16)].symbol(), "╭", "Codex box at column {x}");
    }
}

#[test]
fn seven_parts_scroll_at_each_end() {
    let report = report("1.3");
    assert_eq!(graph_slots(&report), vec![0, 1, 2, 3, 4, 5, 6]);
    let mut options = GraphOptions {
        selected: 0,
        ..golden_options()
    };
    assert_eq!(graph_first_slot(&report, 0, 0), 0);
    let start = render(&report, &options, 120, 40);
    let row = rendered_rows(&start, 3..=3);
    assert!(row.trim_end().ends_with("2 later ›"));
    assert_eq!(Span::raw(row.trim_end()).width(), 119);
    assert_eq!(start[(119, 3)].symbol(), " ");
    assert!(!row.contains("earlier"));
    for y in 13..=15 {
        for x in 112..=118 {
            assert_eq!(start[(x, y)].symbol(), " ", "no orphan edge at {x},{y}");
        }
    }

    options.selected = 6;
    assert_eq!(graph_first_slot(&report, 6, 0), 2);
    let end = render(&report, &options, 120, 40);
    let row = rendered_rows(&end, 3..=3);
    assert!(row.starts_with(" ‹ 2 earlier"));
    assert!(!row.contains("later"));
    assert_eq!(graph_first_slot(&report, 3, 2), 2);

    for (name, buf) in [("start", &start), ("end", &end)] {
        // Exactly five visible boxes in the alternating lanes.
        for (x, top) in [(1, 4), (23, 16), (45, 4), (67, 16), (89, 4)] {
            assert_eq!(buf[(x, top)].symbol(), "╭", "{name}: box at {x},{top}");
        }
        for y in [4, 16] {
            let corners = (0..120).filter(|&x| buf[(x, y)].symbol() == "╭").count();
            assert_eq!(corners, if y == 4 { 3 } else { 2 }, "{name}: row {y}");
        }
    }
    assert!(rendered_rows(&end, 5..=5).ends_with("│ ● claude · verify + commit │\n"));
}

#[test]
fn unlinked_and_ambiguous_parts() {
    let options = GraphOptions {
        selected: 0,
        ..golden_options()
    };
    let buf = render(&report("1.2"), &options, 120, 40);
    for (x, kind, sub, edge_words) in [
        (23, "implement", "no codex run found", "no run found"),
        (67, "fix round 1", "2 possible runs", "2 possible runs"),
    ] {
        let title: String = (x + 2..x + 29).map(|col| buf[(col, 17)].symbol()).collect();
        assert_eq!(title.trim_end(), format!("● codex · {kind}"));
        let subline: String = (x + 2..x + 29).map(|col| buf[(col, 18)].symbol()).collect();
        assert_eq!(subline.trim_end(), sub);
        for y in 16..=24 {
            for col in x..x + 30 {
                let cell = &buf[(col, y)];
                assert_eq!(cell.bg, options.theme.screen, "background at {col},{y}");
                if y == 16
                    || y == 24
                    || col == x
                    || col == x + 29
                    || (y == 18 && cell.symbol() != " ")
                {
                    assert_eq!(cell.fg, options.theme.secondary, "grey at {col},{y}");
                }
            }
        }
        for y in 20..=23 {
            for col in x + 1..x + 29 {
                assert_eq!(
                    buf[(col, y)].symbol(),
                    " ",
                    "no counts/notables at {col},{y}"
                );
            }
        }
        for y in [13, 14] {
            assert_eq!(buf[(x + 4, y)].symbol(), "╎");
        }
        assert_eq!(buf[(x + 4, 15)].symbol(), "▼");
        let words: String = (x + 6..x + 6 + edge_words.len() as u16)
            .map(|col| buf[(col, 15)].symbol())
            .collect();
        assert_eq!(words, edge_words);
        // Neither following Claude part has a return edge.
        for y in 13..=15 {
            assert_eq!(buf[(x + 26, y)].symbol(), " ");
        }
    }
    assert!(!rendered_rows(&buf, 13..=15).contains("back to claude"));
}

#[test]
fn exact_edge_is_solid_and_heuristic_dashed() {
    let buf = render(&report("1.1"), &golden_options(), 120, 40);
    for (x, glyph, label) in [(27, "╎", "matched by time"), (71, "│", "confirmed")] {
        for y in [13, 14] {
            assert_eq!(buf[(x, y)].symbol(), glyph, "edge at {x},{y}");
        }
        assert_eq!(buf[(x, 15)].symbol(), "▼");
        let words: String = (x + 2..x + 2 + label.len() as u16)
            .map(|col| buf[(col, 15)].symbol())
            .collect();
        assert_eq!(words, label);
    }
}

#[test]
fn panel_matches_golden() {
    let buf = render(&report("1.1"), &golden_options(), 120, 40);
    assert_eq!(rendered_rows(&buf, 26..=31), golden_rows(26..=31));
    assert_eq!(rendered_styles(&buf, 26..=31), golden_styles(26..=31));
}

#[test]
fn panel_for_claude_part() {
    let options = GraphOptions {
        selected: 0,
        ..golden_options()
    };
    let buf = render(&report("1.1"), &options, 120, 40);
    assert!(
        rendered_rows(&buf, 27..=27)
            .starts_with("  selected  claude · plan · 81010000 · 08:05:26–08:05:43 · ")
    );
    assert_eq!(buf[(2, 27)].fg, options.theme.secondary);
    assert_eq!(buf[(12, 27)].fg, options.theme.claude);
    assert_eq!(buf[(12, 27)].modifier, Modifier::BOLD);
    assert!(!rendered_rows(&buf, 28..=28).contains("asked"));
    assert_eq!(
        rendered_rows(&buf, 29..=29),
        "            from   handed step 1.1 to codex at 08:05:43\n"
    );
    assert_eq!(
        rendered_rows(&buf, 31..=31),
        "            o open in Claude   f fork\n"
    );
}

#[test]
fn panel_for_part_without_run() {
    let report = report("1.2");
    for (selected, heading) in [
        (1, "  selected  codex · implement · no codex run found\n"),
        (3, "  selected  codex · fix round 1 · 2 possible runs\n"),
    ] {
        let options = GraphOptions {
            selected,
            ..golden_options()
        };
        let buf = render(&report, &options, 120, 40);
        assert_eq!(rendered_rows(&buf, 27..=27), heading);
        let panel = rendered_rows(&buf, 28..=30);
        assert!(!panel.contains("asked"));
        assert!(!panel.contains("said"));
        if selected == 1 {
            assert_eq!(
                rendered_rows(&buf, 30..=30),
                "                   (the rollout may be gone)\n"
            );
        }
        assert_eq!(rendered_rows(&buf, 31..=31), "\n");
    }
}

#[test]
fn untagged_handoffs_line() {
    let report = report("1.8");
    let selected = report
        .parts
        .iter()
        .position(|part| part.kind == "review" && part.untagged_handoffs == 1)
        .unwrap();
    let options = GraphOptions {
        selected,
        ..golden_options()
    };
    let buf = render(&report, &options, 120, 40);
    assert_eq!(
        rendered_rows(&buf, 30..=30),
        "                   1 untagged hand-off here\n"
    );
    assert_eq!(buf[(19, 30)].fg, options.theme.secondary);
}

#[test]
fn quote_cut_on_word_boundary_by_width() {
    let mut report = report("1.1");
    let asked = "日本 word ".repeat(40);
    report.parts[1].asked = Some(asked.clone());
    let buf = render(&report, &golden_options(), 120, 40);
    let row = rendered_rows(&buf, 28..=28);
    let row = row.trim_end_matches('\n');
    assert!(row.starts_with("            asked  “"), "row: {row:?}");
    assert!(row.ends_with("…”"), "row: {row:?}");
    let last_column = (0..120).rfind(|&x| buf[(x, 28)].symbol() != " ").unwrap();
    assert!(last_column <= 108, "last drawn column: {last_column}");
    assert!(Span::raw(row).width() <= 109, "row: {row:?}");
    let body = row
        .strip_prefix("            asked  “")
        .unwrap()
        .strip_suffix("…”")
        .unwrap()
        .trim_end();
    assert!(!body.is_empty());
    assert!(
        asked
            .strip_prefix(body)
            .is_some_and(|rest| rest.starts_with(' ')),
        "quote must end after a whole word: {body:?}"
    );
    assert_eq!(
        row,
        format!("            asked  “{}日本…”", "日本 word ".repeat(8))
    );
}

#[test]
fn panel_quote_keeps_text_when_the_first_word_does_not_fit() {
    let mut report = report("1.1");
    report.parts[1].asked = Some(format!("{} tail", "x".repeat(100)));
    let buf = render(&report, &golden_options(), 120, 40);
    assert_eq!(
        rendered_rows(&buf, 28..=28),
        format!("            asked  “{}…”\n", "x".repeat(87))
    );
    assert_eq!(buf[(108, 28)].symbol(), "”");
    assert_eq!(buf[(109, 28)].symbol(), " ");
}

#[test]
fn panel_missing_runs_keep_their_handoff_origin_without_a_gap() {
    let report = report("1.2");
    for (selected, expected) in [
        (
            1,
            "            from   claude 81020000 · plan · handed at 07:00:20 · no run found\n",
        ),
        (
            3,
            "            from   claude 81020000 · review · handed at 07:03:00 · 2 possible runs\n",
        ),
    ] {
        let buf = render(
            &report,
            &GraphOptions {
                selected,
                ..golden_options()
            },
            120,
            40,
        );
        assert_eq!(rendered_rows(&buf, 29..=29), expected);
        if selected == 1 {
            assert_colored_text(
                &buf,
                19,
                30,
                "(the rollout may be gone)",
                golden_options().theme.secondary,
            );
        }
    }
}

#[test]
fn panel_unavailable_run_has_no_open_action() {
    let buf = render(&report("1.11"), &golden_options(), 120, 40);
    assert_eq!(
        rendered_rows(&buf, 27..=27),
        "  selected  codex · implement · run unavailable\n"
    );
    assert_eq!(rendered_rows(&buf, 31..=31), "\n");
}

#[test]
fn panel_gap_uses_whole_minutes_and_at_least_one_second() {
    let mut report = report("1.1");
    for (gap, suffix) in [
        (125000, " · started 2 min after the call"),
        (59999, " · started 59 s after the call"),
        (0, " · started 1 s after the call"),
        (999, " · started 1 s after the call"),
        (-1, ""),
    ] {
        let Some(Edge::Handoff { gap_ms, .. }) = &mut report.parts[1].edge_in else {
            panic!("expected handoff")
        };
        *gap_ms = Some(gap);
        let buf = render(&report, &golden_options(), 120, 40);
        assert_eq!(
            rendered_rows(&buf, 29..=29),
            format!(
                "            from   claude 81010000 · plan · handed at 08:05:43 · matched by time{suffix}\n"
            ),
            "gap {gap}"
        );
    }
}

#[test]
fn panel_pluralizes_stretches_and_untagged_handoffs() {
    let buf = render(&report("1.3"), &golden_options(), 120, 40);
    assert!(rendered_rows(&buf, 27..=27).ends_with("active about 1 min in 2 stretches\n"));
    let mut report = report("1.8");
    report.parts[2].untagged_handoffs = 2;
    let buf = render(
        &report,
        &GraphOptions {
            selected: 2,
            ..golden_options()
        },
        120,
        40,
    );
    assert_eq!(
        rendered_rows(&buf, 30..=30),
        "                   2 untagged hand-offs here\n"
    );
}

#[test]
fn panel_working_run_without_a_reply_leaves_said_row_blank() {
    let buf = render(
        &report("1.5"),
        &GraphOptions {
            selected: 2,
            ..golden_options()
        },
        120,
        40,
    );
    assert_eq!(rendered_rows(&buf, 30..=30), "\n");
}

#[test]
fn panel_origin_without_parent_part_omits_the_kind() {
    let mut report = report("1.1");
    let Some(Edge::Handoff { parent_part, .. }) = &mut report.parts[1].edge_in else {
        panic!("expected handoff")
    };
    *parent_part = None;
    let buf = render(&report, &golden_options(), 120, 40);
    assert_eq!(
        rendered_rows(&buf, 29..=29),
        "            from   claude 81010000 · handed at 08:05:43 · matched by time · started 1 s after the call\n"
    );
}

#[test]
fn panel_claude_action_keys_are_text_and_words_are_grey() {
    let options = GraphOptions {
        selected: 0,
        ..golden_options()
    };
    let buf = render(&report("1.1"), &options, 120, 40);
    assert_colored_text(&buf, 12, 31, "o", options.theme.text);
    assert_colored_text(&buf, 14, 31, "open in Claude", options.theme.secondary);
    assert_colored_text(&buf, 31, 31, "f", options.theme.text);
    assert_colored_text(&buf, 33, 31, "fork", options.theme.secondary);
}

#[test]
fn selection_past_the_end_does_not_panic() {
    let options = GraphOptions {
        selected: 99,
        ..golden_options()
    };
    let buf = render(&report("1.1"), &options, 120, 40);
    assert!(
        rendered_rows(&buf, 27..=27)
            .starts_with("  selected  claude · verify + commit · 81010000 · 08:10:42–08:12:14 · ")
    );
    assert_eq!(
        rendered_rows(&buf, 29..=29),
        "            from   no hand-offs\n"
    );
    assert_eq!(
        rendered_rows(&buf, 31..=31),
        "            o open in Claude   f fork\n"
    );
}

fn assert_colored_text(buf: &Buffer, x: u16, y: u16, expected: &str, color: Color) {
    let mut column = x;
    for ch in expected.chars() {
        let symbol = ch.to_string();
        let cell = &buf[(column, y)];
        assert_eq!(cell.symbol(), symbol, "text at {},{y}", column);
        if ch != ' ' {
            assert_eq!(cell.fg, color, "colour at {},{y}", column);
        }
        column += Span::raw(symbol).width() as u16;
    }
}

#[test]
fn long_step_edge_labels_leave_room_before_the_next_edge() {
    let mut report = report("1.3");
    report.step.id = "12.345.678".into();
    let options = GraphOptions {
        selected: 0,
        ..golden_options()
    };
    let buf = render(&report, &options, 120, 40);
    assert_colored_text(&buf, 29, 13, "handed step 12.345…", options.theme.text);
    assert_eq!(buf[(48, 13)].symbol(), " ");
    assert_colored_text(&buf, 49, 13, "▲", options.theme.secondary);
}

#[test]
fn panel_gap_uses_nonnegative_lengths_with_a_one_second_minimum() {
    let options = golden_options();
    for (gap, expected) in [
        (-1, None),
        (0, Some("1 s")),
        (999, Some("1 s")),
        (59_999, Some("59 s")),
        (60_000, Some("1 min")),
        (119_999, Some("1 min")),
    ] {
        let mut report = report("1.1");
        if let Some(multi_link_model::Edge::Handoff { gap_ms, .. }) = &mut report.parts[1].edge_in {
            *gap_ms = Some(gap);
        }
        let buf = render(&report, &options, 120, 40);
        let row = rendered_rows(&buf, 29..=29);
        if let Some(expected) = expected {
            assert!(
                row.trim_end()
                    .ends_with(&format!("started {expected} after the call")),
                "gap {gap}: {row}"
            );
        } else {
            assert!(!row.contains("started"), "gap {gap}: {row}");
        }
    }
}

#[test]
fn incomplete_run_metadata_is_unavailable_in_both_box_and_panel() {
    let options = golden_options();
    for missing in ["availability", "from", "to"] {
        let mut report = report("1.1");
        let part = &mut report.parts[1];
        match missing {
            "availability" => part.run_available = Some(false),
            "from" => part.from = None,
            _ => part.to = None,
        }
        let buf = render(&report, &options, 120, 40);
        assert_colored_text(&buf, 25, 18, "run unavailable", options.theme.secondary);
        assert!(rendered_rows(&buf, 27..=27).contains("implement · run unavailable"));
        assert_eq!(rendered_rows(&buf, 28..=28), "\n");
        assert_eq!(rendered_rows(&buf, 31..=31), "\n");
    }
}

#[test]
fn return_edge_remains_visible_after_its_codex_box_scrolls_off() {
    let options = GraphOptions {
        selected: 7,
        ..golden_options()
    };
    let report = report("1.12");
    assert_eq!(graph_first_slot(&report, 7, 0), 4);
    let buf = render(&report, &options, 120, 40);
    assert_colored_text(&buf, 5, 13, "▲", options.theme.secondary);
    assert_colored_text(&buf, 7, 13, "back to claude", options.theme.secondary);
    assert_colored_text(&buf, 5, 14, "│", options.theme.secondary);
    assert_colored_text(&buf, 5, 15, "│", options.theme.secondary);
}

#[test]
fn edges_keep_their_columns_when_scrolled_one_slot() {
    let options = GraphOptions {
        selected: 3,
        first_slot: 1,
        ..golden_options()
    };
    let buf = render(&report("1.3"), &options, 120, 40);
    assert_colored_text(&buf, 27, 13, "▲", options.theme.secondary);
    assert_colored_text(&buf, 29, 13, "back to claude", options.theme.secondary);
    assert_colored_text(&buf, 5, 13, "│", options.theme.text);
}

#[test]
fn working_subline_splits_orange_status_and_grey_duration() {
    let options = golden_options();
    let buf = render(&report("1.5"), &options, 120, 40);
    assert_colored_text(&buf, 69, 18, "working now", options.theme.claude);
    assert_colored_text(&buf, 80, 18, " · 6 min", options.theme.secondary);
}

#[test]
fn stale_subline_names_the_run_start_in_red() {
    let mut report = report("1.1");
    report.parts[3].status = Some(PartStatus::Stale);
    let options = golden_options();
    let buf = render(&report, &options, 120, 40);
    assert_colored_text(&buf, 69, 18, "stale since 08:09", options.theme.warning);
    assert_eq!(buf[(86, 18)].symbol(), " ");
}

#[test]
fn failed_notables_are_red_and_unavailable_runs_keep_confirmed_edges() {
    let options = golden_options();
    let failed = render(&report("1.3"), &options, 120, 40);
    assert_colored_text(&failed, 38, 22, "✗", options.theme.warning);
    assert_colored_text(&failed, 82, 21, "✗", options.theme.warning);
    let unavailable = render(&report("1.11"), &options, 120, 40);
    assert_colored_text(
        &unavailable,
        25,
        18,
        "run unavailable",
        options.theme.secondary,
    );
    assert_colored_text(&unavailable, 27, 13, "│", options.theme.text);
    assert_colored_text(&unavailable, 27, 14, "│", options.theme.text);
    assert_colored_text(&unavailable, 29, 15, "confirmed", options.theme.secondary);
}

#[test]
fn waiting_reads_reserve_the_last_row_after_two_newest_notables() {
    let mut report = report("1.1");
    report.parts[0].notable = vec![
        Notable {
            label: "edit oldest.rs".into(),
            ok: None,
        },
        Notable {
            label: "$ git diff".into(),
            ok: Some(true),
        },
        Notable {
            label: "$ cargo test".into(),
            ok: Some(true),
        },
    ];
    let options = golden_options();
    let buf = render(&report, &options, 120, 40);
    assert_colored_text(&buf, 3, 9, "$ git diff", options.theme.text);
    assert_colored_text(&buf, 3, 10, "$ cargo test", options.theme.text);
    assert_colored_text(&buf, 16, 10, "✓", options.theme.done);
    assert_colored_text(
        &buf,
        3,
        11,
        "6 reads while codex ran",
        options.theme.secondary,
    );
    assert!(!rendered_rows(&buf, 4..=12).contains("oldest.rs"));
}

#[test]
fn second_chain_first_handoff_uses_step_wording() {
    let options = GraphOptions {
        selected: 4,
        ..golden_options()
    };
    let buf = render(&report("1.4"), &options, 120, 40);
    assert_colored_text(&buf, 95, 13, "handed step 1.4", options.theme.text);
    assert!(!rendered_rows(&buf, 13..=13).contains("handed fix"));
}

#[test]
fn long_notables_cut_to_box_width_and_reserve_the_success_mark() {
    let mut report = report("1.1");
    report.parts[1].notable = vec![
        Notable {
            label: format!("edit {}", "e".repeat(55)),
            ok: None,
        },
        Notable {
            label: format!("$ cargo test {}", "t".repeat(60)),
            ok: Some(true),
        },
    ];
    let options = golden_options();
    let buf = render(&report, &options, 120, 40);
    assert_colored_text(
        &buf,
        25,
        21,
        &format!("edit {}…", "e".repeat(21)),
        options.theme.text,
    );
    assert_colored_text(
        &buf,
        25,
        22,
        &format!("$ cargo test {}…", "t".repeat(11)),
        options.theme.text,
    );
    assert_colored_text(&buf, 51, 22, "✓", options.theme.done);
    for y in [21, 22] {
        assert_eq!(buf[(52, y)].symbol(), "│");
    }
}

#[test]
fn narrow_lists_every_part_in_order() {
    for (id, expected, edges) in [
        (
            "1.1",
            [
                ("claude · plan", "08:05 · 17 s", "1 read · 1 run"),
                (
                    "codex · implement",
                    "✓ done 08:09 · 3 min",
                    "6 reads · 3 edits · 6 runs",
                ),
                ("claude · review", "08:09 · 53 s", "6 reads"),
                (
                    "codex · fix round 1",
                    "✓ done 08:10 · 48 s",
                    "3 reads · 2 edits · 2 runs",
                ),
                (
                    "claude · verify + commit",
                    "08:10 · 1 min",
                    "4 reads · 1 edit · 3 runs",
                ),
            ],
            [
                &["handed step 1.1", "matched by time"][..],
                &["back to claude"][..],
                &["handed fix", "confirmed"][..],
                &["back to claude"][..],
            ],
        ),
        (
            "1.2",
            [
                ("claude · plan", "07:00 · 20 s", "1 read"),
                ("codex · implement", "no codex run found", ""),
                ("claude · review", "07:00 · 2 min", "1 read"),
                ("codex · fix round 1", "2 possible runs", ""),
                ("claude · review", "07:03 · 1 min", "1 run"),
            ],
            [
                &["handed step 1.2", "no run found"][..],
                &[][..],
                &["handed fix", "2 possible runs"][..],
                &[][..],
            ],
        ),
    ] {
        let report = report(id);
        for width in [119, 100, 80, 60] {
            let buf = render(&report, &golden_options(), width, 40);
            let rendered = text(&buf);
            let rows: Vec<_> = rendered.lines().collect();
            assert!(!rendered.contains('╭'), "{id}, width {width}: box corner");
            let mut cursor = 4;
            for (index, (title, subline, counts)) in expected.iter().enumerate() {
                let found = (cursor..rows.len()).find(|&y| rows[y].contains(title));
                assert!(found.is_some(), "{id}, width {width}: missing {title}");
                let y = found.unwrap();
                if index > 0 {
                    for words in edges[index - 1] {
                        assert!(
                            rows[cursor..y]
                                .iter()
                                .any(|row| row.starts_with("  ") && row.contains(words)),
                            "{id}, width {width}: edge {words:?} before part {index}"
                        );
                    }
                }
                assert!(
                    rows.get(y + 1)
                        .is_some_and(|row| { row.contains(subline) && row.contains(counts) }),
                    "{id}, width {width}: {title} must be followed by {subline:?} and {counts:?}"
                );
                cursor = y + 2;
            }
        }
    }
}

#[test]
fn narrow_footer_uses_up_down() {
    for id in ["1.1", "1.2"] {
        for width in [119, 100, 80, 60] {
            let buf = render(&report(id), &golden_options(), width, 40);
            assert!(
                rendered_rows(&buf, 39..=39).starts_with(" ↑↓ move along the step"),
                "{id}, width {width}: narrow footer"
            );
        }
    }
}

#[test]
fn narrow_cuts_no_words() {
    let whole_token = |token: &str| {
        let token = token.strip_prefix('“').unwrap_or(token);
        token
            .strip_suffix("…”")
            .or_else(|| token.strip_suffix('…'))
            .unwrap_or(token)
            .to_owned()
    };
    for id in ["1.1", "1.2"] {
        let report = report(id);
        for selected in 0..report.parts.len() {
            let options = GraphOptions {
                selected,
                ..golden_options()
            };
            let wide = text(&render(&report, &options, 120, 40));
            let tokens: Vec<_> = wide.split_whitespace().map(whole_token).collect();
            for width in [119, 100, 80, 60] {
                let narrow = text(&render(&report, &options, width, 40));
                for row in narrow.lines() {
                    let mut row_tokens = row.split_whitespace().peekable();
                    while let Some(token) = row_tokens.next() {
                        assert!(
                            token == "↑↓" || tokens.contains(&whole_token(token)),
                            "{id}, selected {selected}, width {width}: cut/new token {token:?} in {row:?}"
                        );
                        if token.ends_with('…') || token.ends_with("…”") {
                            assert!(
                                row_tokens.peek().is_none(),
                                "{id}, selected {selected}, width {width}: ellipsis before row end in {row:?}"
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn narrow_selected_part_is_highlighted() {
    for id in ["1.1", "1.2"] {
        let report = report(id);
        for selected in 0..report.parts.len() {
            let options = GraphOptions {
                selected,
                ..golden_options()
            };
            for width in [119, 100, 80, 60] {
                let buf = render(&report, &options, width, 40);
                let rendered = text(&buf);
                // List entries are the harness/kind rows before the selected panel.
                let title_rows: Vec<_> = rendered
                    .lines()
                    .enumerate()
                    .filter(|(_, row)| {
                        !row.contains("selected")
                            && (row.contains("claude · ") || row.contains("codex · "))
                    })
                    .map(|(y, _)| y as u16)
                    .collect();
                assert_eq!(
                    title_rows.len(),
                    report.parts.len(),
                    "{id}, selected {selected}, width {width}: one list title per part"
                );
                let y = title_rows[selected];
                for row in [y, y + 1] {
                    for x in 1..width - 1 {
                        assert_eq!(
                            buf[(x, row)].bg,
                            options.theme.graph_selected,
                            "{id}, selected {selected}, width {width}: background at {x},{row}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn short_screens_reserve_clean_rule_and_footer_rows() {
    let report = report("1.1");
    let options = golden_options();
    for width in [120, 119, 100, 80, 60] {
        let full = render(&report, &options, width, 40);
        for height in [26, 24] {
            let short = render(&report, &options, width, height);
            for (actual, expected) in [(height - 2, 38), (height - 1, 39)] {
                for x in 0..width {
                    assert_eq!(
                        short[(x, actual)],
                        full[(x, expected)],
                        "width {width}, height {height}, cell ({x}, {actual})"
                    );
                }
            }
        }
    }
}

#[test]
fn panel_first_line_moves_after_lanes_at_twenty_eight_and_twenty_nine_rows() {
    let report = report("1.1");
    let options = golden_options();
    for width in [120, 80] {
        let full = render(&report, &options, width, 40);
        for height in [28, 29] {
            let short = render(&report, &options, width, height);
            assert_eq!(
                rendered_rows(&short, 25..=25),
                rendered_rows(&full, 27..=27)
            );
            assert_eq!(
                rendered_rows(&short, 24..=24),
                rendered_rows(&full, 24..=24)
            );
            assert!(!rendered_rows(&short, 25..=usize::from(height - 3)).contains("asked"));
        }
    }
}

#[test]
fn empty_sentence_survives_at_five_and_six_rows() {
    let report = report("1.13");
    for width in [120, 80] {
        let full = render(&report, &golden_options(), width, 40);
        for height in [6, 5] {
            let buf = render(&report, &golden_options(), width, height);
            let row = usize::from(height - 3);
            assert_eq!(rendered_rows(&buf, row..=row), rendered_rows(&full, 4..=4));
        }
    }
}

#[test]
fn thirty_six_rows_keep_the_legend_after_the_full_panel() {
    let report = report("1.1");
    let options = golden_options();
    let full = render(&report, &options, 120, 40);
    let short = render(&report, &options, 120, 36);
    assert_eq!(
        rendered_rows(&short, 27..=31),
        rendered_rows(&full, 27..=31)
    );
    assert_eq!(rendered_rows(&short, 32..=32), "\n");
    assert_eq!(
        rendered_rows(&short, 33..=33),
        rendered_rows(&full, 36..=36)
    );
}

#[test]
fn taller_screens_keep_board_rows_and_move_only_rule_and_footer() {
    let report = report("1.1");
    let options = golden_options();
    let full = render(&report, &options, 120, 40);
    let tall = render(&report, &options, 120, 45);
    assert_eq!(rendered_rows(&tall, 0..=37), rendered_rows(&full, 0..=37));
    assert_eq!(
        rendered_styles(&tall, 0..=37),
        rendered_styles(&full, 0..=37)
    );
    assert_eq!(rendered_rows(&tall, 38..=42), "\n".repeat(5));
    assert_eq!(rendered_rows(&tall, 43..=44), rendered_rows(&full, 38..=39));
}

#[test]
fn shrinking_screens_drop_strip_then_legend_then_panel_lines() {
    let report = report("1.1");
    let options = golden_options();
    let full = render(&report, &options, 120, 40);
    let mut previous_count = 5;
    for height in (20..=41).rev() {
        let buf = render(&report, &options, 120, height);
        for (actual, expected) in [(height - 2, 38), (height - 1, 39)] {
            for x in 0..120 {
                assert_eq!(buf[(x, actual)], full[(x, expected)], "height {height}");
            }
        }
        let body = rendered_rows(&buf, 0..=usize::from(height - 3));
        let count = body
            .lines()
            .filter(|row| {
                ["selected", "asked", "from", "said", "o open"]
                    .iter()
                    .any(|label| row.trim_start().starts_with(label))
            })
            .count();
        let expected = match height {
            34.. => 5,
            33 => 4,
            32 => 3,
            31 => 2,
            28..=30 => 1,
            _ => 0,
        };
        assert_eq!(count, expected, "height {height}");
        assert!(count <= previous_count, "height {height}: panel grew");
        previous_count = count;
        assert_eq!(
            body.contains("where the time went"),
            height >= 40,
            "height {height}"
        );
        assert_eq!(
            body.contains("▭ claude part"),
            height >= 36,
            "height {height}"
        );
    }
}

#[test]
fn ten_rows_keep_rule_and_footer_on_the_last_two_rows() {
    let report = report("1.1");
    let options = golden_options();
    let full = render(&report, &options, 120, 40);
    let short = render(&report, &options, 120, 10);
    for x in 0..120 {
        assert_eq!(short[(x, 8)], full[(x, 38)], "rule cell {x}");
        assert_eq!(short[(x, 9)], full[(x, 39)], "footer cell {x}");
    }
}
