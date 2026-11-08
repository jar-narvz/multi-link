//! Board 9: a pure rendering of a step graph report.
use std::collections::HashSet;

use multi_link_model::{GraphReport, Harness, Timestamp};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use time::UtcOffset;

use crate::{home::basename, theme::Theme, view};

#[derive(Clone, Copy)]
pub struct GraphOptions {
    pub selected: usize,
    pub first_slot: usize,
    pub theme: Theme,
    pub offset: UtcOffset,
}

impl Default for GraphOptions {
    fn default() -> Self {
        Self {
            selected: 0,
            first_slot: 0,
            theme: Theme::from_env(Some("truecolor"), None),
            offset: UtcOffset::UTC,
        }
    }
}

/// First part at 0; advance one slot across harnesses, two within a harness.
pub fn graph_slots(report: &GraphReport) -> Vec<usize> {
    let mut slot = 0;
    report
        .parts
        .iter()
        .enumerate()
        .map(|(i, part)| {
            if i > 0 {
                slot += if part.harness == report.parts[i - 1].harness {
                    2
                } else {
                    1
                };
            }
            slot
        })
        .collect()
}

/// Move only far enough to show the selected part in the five-slot window.
pub fn graph_first_slot(report: &GraphReport, selected: usize, first_slot: usize) -> usize {
    let slots = graph_slots(report);
    let Some(&last) = slots.last() else { return 0 };
    let selected = slots[selected.min(slots.len() - 1)];
    first_slot
        .min(last.saturating_sub(4))
        .min(selected)
        .max(selected.saturating_sub(4))
}

/// Render in local coordinates, then copy into the caller's clipped area.
pub fn render_graph(report: &GraphReport, options: &GraphOptions, area: Rect, buf: &mut Buffer) {
    let area = area.intersection(buf.area);
    if area.is_empty() {
        return;
    }
    let mut screen = Buffer::empty(Rect::new(0, 0, area.width, area.height));
    screen.set_style(screen.area, Style::default().bg(options.theme.screen));
    let mut paint = Paint {
        buf: &mut screen,
        theme: options.theme,
    };
    header(report, options, &mut paint);
    paint.rule(2);
    if let Some(empty) = &report.empty {
        paint.write(2, 4, empty, options.theme.secondary);
    } else {
        let first = graph_first_slot(report, options.selected, options.first_slot);
        let slots = graph_slots(report);
        let earlier = slots.iter().filter(|&&slot| slot < first).count();
        let later = slots.iter().filter(|&&slot| slot >= first + 5).count();
        if earlier > 0 {
            paint.write(
                1,
                3,
                &format!("‹ {earlier} earlier"),
                options.theme.secondary,
            );
        }
        if later > 0 {
            paint.right(3, &format!("{later} later ›"), options.theme.secondary);
        }
        legend(&mut paint);
    }
    paint.rule(38);
    footer(options, &mut paint);
    options.theme.apply_no_color(&mut screen);
    for y in 0..area.height {
        for x in 0..area.width {
            buf[(area.x + x, area.y + y)] = screen[(x, y)].clone();
        }
    }
}

struct Paint<'a> {
    buf: &'a mut Buffer,
    theme: Theme,
}

impl Paint<'_> {
    // Exclusive right edge, preserving the screen's outer blank column.
    fn end(&self) -> u16 {
        self.buf.area.width.saturating_sub(1)
    }

    fn line(&mut self, x: u16, y: u16, line: Line<'_>, width: u16) {
        if y < self.buf.area.height && x < self.end() {
            self.buf.set_line(x, y, &line, width.min(self.end() - x));
        }
    }

    fn span(&self, text: impl Into<String>, color: Color) -> Span<'static> {
        Span::styled(text.into(), self.theme.text_style(color, self.theme.screen))
    }

    fn write(&mut self, x: u16, y: u16, text: &str, color: Color) {
        self.line(
            x,
            y,
            Line::from(self.span(text, color)),
            self.end().saturating_sub(x),
        );
    }

    fn right(&mut self, y: u16, text: &str, color: Color) {
        let text = cut(text, self.end().saturating_sub(1) as usize);
        let x = self.end().saturating_sub(Span::raw(&text).width() as u16);
        self.write(x, y, &text, color);
    }

    fn rule(&mut self, y: u16) {
        self.write(
            1,
            y,
            &"─".repeat(self.end().saturating_sub(1) as usize),
            self.theme.border,
        );
    }
}

/// Cut by display width, keeping an ellipsis even for a single long word.
fn cut(text: &str, width: usize) -> String {
    if Span::raw(text).width() <= width {
        return text.to_owned();
    }
    if width == 0 {
        return String::new();
    }
    let mut prefix = String::new();
    for ch in text.chars() {
        let end = prefix.len();
        prefix.push(ch);
        if Span::raw(&prefix).width() >= width {
            prefix.truncate(end);
            break;
        }
    }
    prefix.push('…');
    prefix
}

fn clock(at: Timestamp, offset: UtcOffset) -> String {
    let at = view::datetime(at).to_offset(offset);
    format!("{:02}:{:02}", at.hour(), at.minute())
}

fn summary(report: &GraphReport, offset: UtcOffset) -> String {
    let Some(from) = report.parts.iter().filter_map(|p| p.from).min() else {
        return String::new();
    };
    let Some(to) = report.parts.iter().filter_map(|p| p.to).max() else {
        return String::new();
    };
    let date = view::datetime(from).to_offset(offset);
    let seconds = (to.epoch_ms() - from.epoch_ms()).max(0) / 1000;
    let length = if seconds < 60 {
        format!("{seconds} s")
    } else {
        format!("{} min", seconds / 60)
    };
    let sessions = report
        .parts
        .iter()
        .filter(|p| p.harness == Harness::Claude)
        .filter_map(|p| p.session.as_deref())
        .collect::<HashSet<_>>()
        .len();
    let runs = report
        .parts
        .iter()
        .filter(|p| p.run_available == Some(true))
        .count();
    let fixes = report
        .parts
        .iter()
        .filter(|p| p.kind.starts_with("fix round "))
        .count();
    let plural = |n| if n == 1 { "" } else { "s" };
    format!(
        "{} {} · {}–{} · {length} · {sessions} claude session{} · {runs} codex run{} · {fixes} fix round{}",
        &date.month().to_string()[..3],
        date.day(),
        clock(from, offset),
        clock(to, offset),
        plural(sessions),
        plural(runs),
        plural(fixes)
    )
}

fn header(report: &GraphReport, options: &GraphOptions, paint: &mut Paint<'_>) {
    let t = options.theme;
    let commit = report.commit.as_ref().map(|commit| {
        format!(
            "✓ committed {} · {}",
            commit.hash,
            clock(commit.at, options.offset)
        )
    });
    let mut title_end = paint.end();
    if let Some(commit) = commit {
        title_end = title_end.saturating_sub(Span::raw(&commit).width() as u16 + 2);
        paint.right(0, &commit, t.done);
    }
    let bold = t.text_style(t.text, t.screen).add_modifier(Modifier::BOLD);
    let prefix = Line::from(vec![
        Span::styled(basename(&report.repo).to_owned(), bold),
        paint.span(" › ", t.secondary),
        Span::styled(format!("step {}", report.step.id), bold),
        paint.span("  ", t.text),
    ]);
    let title_x = 1usize.saturating_add(prefix.width());
    paint.line(1, 0, prefix, title_end.saturating_sub(1));
    if title_x < usize::from(title_end) {
        let title = cut(
            report.step.title.as_deref().unwrap_or(""),
            usize::from(title_end) - title_x,
        );
        paint.write(title_x as u16, 0, &title, t.text);
    }
    let left = if report.parts.is_empty() {
        String::new()
    } else {
        summary(report, options.offset)
    };
    let left = cut(&left, paint.end().saturating_sub(1) as usize);
    paint.write(1, 1, &left, t.secondary);
    if let Some(verify) = &report.step.verify {
        let start = if left.is_empty() {
            1
        } else {
            1 + Span::raw(&left).width() as u16 + 2
        };
        let available = paint.end().saturating_sub(start) as usize;
        let fixed = Span::raw("Verify   (reported)").width();
        if available >= fixed {
            let command = cut(verify, available - fixed);
            paint.right(1, &format!("Verify  {command} (reported)"), t.secondary);
        }
    }
}

fn legend(paint: &mut Paint<'_>) {
    let t = paint.theme;
    let mut spans = Vec::new();
    for (i, (glyph, words, color)) in [
        ("▭", " claude part", t.claude),
        ("▭", " codex run", t.codex),
        ("╎", " matched by time", t.text),
        ("│", " confirmed link", t.text),
        ("✓", " done, or exit 0", t.done),
        ("✗", " exit non-zero", t.warning),
    ]
    .into_iter()
    .enumerate()
    {
        if i > 0 {
            spans.push(paint.span("   ", t.secondary));
        }
        spans.push(paint.span(glyph, color));
        spans.push(paint.span(words, t.secondary));
    }
    paint.line(1, 36, Line::from(spans), paint.end().saturating_sub(1));
}

fn footer(options: &GraphOptions, paint: &mut Paint<'_>) {
    let t = paint.theme;
    let mut spans = Vec::new();
    for (i, (key, words)) in [
        ("←→", " move along the step"),
        ("[ ]", " prev / next step"),
        ("o", " open"),
        ("f", " fork"),
        ("esc", " repo view"),
        ("q", " quit"),
    ]
    .into_iter()
    .enumerate()
    {
        if i > 0 {
            spans.push(paint.span("   ", t.secondary));
        }
        spans.push(paint.span(key, t.text));
        spans.push(paint.span(words, t.secondary));
    }
    let seconds = options.offset.whole_seconds();
    let sign = if seconds < 0 { "-" } else { "+" };
    let seconds = seconds.unsigned_abs();
    let zone = if seconds.is_multiple_of(3600) {
        format!("times UTC{sign}{}", seconds / 3600)
    } else {
        format!("times UTC{sign}{}:{:02}", seconds / 3600, seconds / 60 % 60)
    };
    let width = paint
        .end()
        .saturating_sub(Span::raw(&zone).width() as u16 + 3);
    paint.line(1, 39, Line::from(spans), width.saturating_sub(1));
    paint.right(39, &zone, t.secondary);
}
