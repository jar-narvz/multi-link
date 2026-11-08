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
    first_visible_slot(&slots, selected, first_slot)
}

fn first_visible_slot(slots: &[usize], selected: usize, first_slot: usize) -> usize {
    let Some(&last) = slots.last() else { return 0 };
    let selected = slots[selected.min(slots.len() - 1)];
    first_slot
        .min(last.saturating_sub(4))
        .min(selected)
        .max(selected.saturating_sub(4))
}

/// Fixed board rows for now. An out-of-bounds row is the height sentinel,
/// which the painter clips; short-screen drop rules can be added here later.
struct Layout {
    header: [u16; 2],
    header_rule: u16,
    scroll: u16,
    claude_lane: u16,
    #[allow(dead_code)] // Reserved for the remaining graph rendering tasks.
    codex_lane: u16,
    #[allow(dead_code)]
    panel: u16,
    #[allow(dead_code)]
    strip: u16,
    legend: u16,
    rule: u16,
    footer: u16,
}

impl Layout {
    fn new(height: u16) -> Self {
        let row = |y: u16| y.min(height);
        Self {
            header: [row(0), row(1)],
            header_rule: row(2),
            scroll: row(3),
            claude_lane: row(4),
            codex_lane: row(16),
            panel: row(27),
            strip: row(33),
            legend: row(36),
            rule: row(38),
            footer: row(39),
        }
    }
}

/// Render in local coordinates, then copy into the caller's clipped area.
pub fn render_graph(report: &GraphReport, options: &GraphOptions, area: Rect, buf: &mut Buffer) {
    let area = area.intersection(buf.area);
    if area.is_empty() {
        return;
    }
    let mut screen = Buffer::empty(Rect::new(0, 0, area.width, area.height));
    screen.set_style(screen.area, Style::default().bg(options.theme.screen));
    let layout = Layout::new(area.height);
    let mut paint = Paint {
        buf: &mut screen,
        theme: options.theme,
    };
    header(report, options, &layout, &mut paint);
    paint.rule(layout.header_rule);
    if let Some(empty) = &report.empty {
        paint.write(
            2,
            layout.claude_lane,
            empty,
            options.theme.secondary,
            paint.end(),
        );
    } else {
        let slots = graph_slots(report);
        let first = first_visible_slot(&slots, options.selected, options.first_slot);
        let earlier = slots.iter().filter(|&&slot| slot < first).count();
        let later = slots.iter().filter(|&&slot| slot >= first + 5).count();
        if earlier > 0 {
            paint.write(
                1,
                layout.scroll,
                &format!("‹ {earlier} earlier"),
                options.theme.secondary,
                paint.end(),
            );
        }
        if later > 0 {
            paint.right(
                layout.scroll,
                &format!("{later} later ›"),
                options.theme.secondary,
            );
        }
        legend(layout.legend, &mut paint);
    }
    paint.rule(layout.rule);
    footer(options, layout.footer, &mut paint);
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

    fn line(&mut self, x: u16, y: u16, line: Line<'_>, until: u16) {
        let until = until.min(self.end());
        if y < self.buf.area.height && x < until {
            self.buf.set_line(x, y, &line, until - x);
        }
    }

    fn span(&self, text: impl Into<String>, color: Color) -> Span<'static> {
        Span::styled(text.into(), self.theme.text_style(color, self.theme.screen))
    }

    fn write(&mut self, x: u16, y: u16, text: &str, color: Color, until: u16) {
        self.line(x, y, Line::from(self.span(text, color)), until);
    }

    fn right(&mut self, y: u16, text: &str, color: Color) {
        let text = cut(text, self.end().saturating_sub(1) as usize);
        let x = self.end().saturating_sub(Span::raw(&text).width() as u16);
        self.write(x, y, &text, color, self.end());
    }

    fn rule(&mut self, y: u16) {
        self.write(
            1,
            y,
            &"─".repeat(self.end().saturating_sub(1) as usize),
            self.theme.border,
            self.end(),
        );
    }
}

/// Cut by character and display width, not at word boundaries. Quotes need
/// a separate word-boundary cutter. Keep an ellipsis even for a long word.
fn cut(text: &str, width: usize) -> String {
    if Span::raw(text).width() <= width {
        return text.to_owned();
    }
    if width == 0 {
        return String::new();
    }
    let mut prefix = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let mut bytes = [0; 4];
        let encoded: &str = ch.encode_utf8(&mut bytes);
        let cells = Span::raw(encoded).width();
        if used + cells >= width {
            break;
        }
        used += cells;
        prefix.push(ch);
    }
    prefix.push('…');
    prefix
}

fn clock(at: Timestamp, offset: UtcOffset, with_seconds: bool) -> String {
    let at = view::datetime(at).to_offset(offset);
    let mut label = format!("{:02}:{:02}", at.hour(), at.minute());
    if with_seconds {
        label.push_str(&format!(":{:02}", at.second()));
    }
    label
}

fn summary(report: &GraphReport, offset: UtcOffset) -> String {
    let Some(from) = report.parts.iter().filter_map(|p| p.from).min() else {
        return String::new();
    };
    let Some(to) = report.parts.iter().filter_map(|p| p.to).max() else {
        return String::new();
    };
    let date = view::datetime(from).to_offset(offset);
    let seconds = ((to.epoch_ms() - from.epoch_ms()) / 1000).max(1);
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
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    format!(
        "{} {} · {}–{} · {length} · {sessions} claude session{} · {runs} codex run{} · {fixes} fix round{}",
        MONTHS[date.month() as usize - 1],
        date.day(),
        clock(from, offset, false),
        clock(to, offset, false),
        plural(sessions),
        plural(runs),
        plural(fixes)
    )
}

fn header(report: &GraphReport, options: &GraphOptions, layout: &Layout, paint: &mut Paint<'_>) {
    let t = options.theme;
    let commit = report.commit.as_ref().map(|commit| {
        format!(
            "✓ committed {} · {}",
            commit.hash,
            clock(commit.at, options.offset, false)
        )
    });
    let mut title_end = paint.end();
    if let Some(commit) = commit {
        title_end = title_end.saturating_sub(Span::raw(&commit).width() as u16 + 2);
        paint.right(layout.header[0], &commit, t.done);
    }
    let bold = t.text_style(t.text, t.screen).add_modifier(Modifier::BOLD);
    let step = format!("step {}", report.step.id);
    let suffix_width = Span::raw(format!(" › {step}  ")).width();
    // Keep one cell for the title (or its ellipsis) when it is present.
    let title_min = usize::from(
        report
            .step
            .title
            .as_ref()
            .is_some_and(|title| !title.is_empty()),
    );
    let repo_width =
        usize::from(title_end.saturating_sub(1)).saturating_sub(suffix_width + title_min);
    let prefix = Line::from(vec![
        Span::styled(cut(basename(&report.repo), repo_width), bold),
        paint.span(" › ", t.secondary),
        Span::styled(step, bold),
        paint.span("  ", t.text),
    ]);
    let title_x = 1usize.saturating_add(prefix.width());
    paint.line(1, layout.header[0], prefix, title_end);
    if title_x < usize::from(title_end) {
        let title = cut(
            report.step.title.as_deref().unwrap_or(""),
            usize::from(title_end) - title_x,
        );
        paint.write(title_x as u16, layout.header[0], &title, t.text, title_end);
    }
    let left = if report.parts.is_empty() {
        String::new()
    } else {
        summary(report, options.offset)
    };
    let left = cut(&left, paint.end().saturating_sub(1) as usize);
    paint.write(1, layout.header[1], &left, t.secondary, paint.end());
    if let Some(verify) = &report.step.verify {
        let start = if left.is_empty() {
            1
        } else {
            1 + Span::raw(&left).width() as u16 + 2
        };
        let available = paint.end().saturating_sub(start) as usize;
        let prefix = "Verify  ";
        let suffix = " (reported)";
        let fixed = Span::raw(prefix).width() + Span::raw(suffix).width();
        if available >= fixed + 2 {
            let command = cut(verify, available - fixed);
            paint.right(
                layout.header[1],
                &format!("{prefix}{command}{suffix}"),
                t.secondary,
            );
        }
    }
}

fn legend(y: u16, paint: &mut Paint<'_>) {
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
    paint.line(1, y, Line::from(spans), paint.end());
}

fn footer(options: &GraphOptions, y: u16, paint: &mut Paint<'_>) {
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
    let zone = format!("times {}", view::offset_label(options.offset));
    let until = paint
        .end()
        .saturating_sub(Span::raw(&zone).width() as u16 + 3);
    paint.line(1, y, Line::from(spans), until);
    paint.right(y, &zone, t.secondary);
}
