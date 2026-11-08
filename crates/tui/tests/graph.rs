//! Board 9 contract tests. Immutable targets have no snapshot update path.
#[allow(dead_code)]
mod support;

use multi_link_model::{GraphCommit, GraphReport, Timestamp};
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
