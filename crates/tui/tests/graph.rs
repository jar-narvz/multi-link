//! Board 9 contract tests. Immutable targets have no snapshot update path.
#[allow(dead_code)]
mod support;

use multi_link_model::GraphReport;
use multi_link_tui::{GraphOptions, render_graph, theme::Theme};
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
