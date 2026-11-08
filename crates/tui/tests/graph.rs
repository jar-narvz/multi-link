//! Board 9 contract tests. Immutable targets have no snapshot update path.
#[allow(dead_code)]
mod support;

use multi_link_model::{GraphCommit, GraphReport, Notable, PartStatus, Timestamp};
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
    assert_eq!(widths.iter().sum::<u16>(), 55);
    assert_eq!(x, 117, "strip must end at column 116");
    assert_eq!(buf[(22, 33)].bg, options.theme.screen);
    assert_colored_text(&buf, 23, 34, "08:05", options.theme.secondary);
    assert_colored_text(&buf, 112, 34, "08:12", options.theme.secondary);
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
