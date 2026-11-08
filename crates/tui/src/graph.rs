//! Board 9: a pure rendering of a step graph report.
use std::collections::HashSet;

use multi_link_model::{
    Counts, Edge, GraphReport, Harness, Match, Notable, Part, PartStatus, Timestamp,
};
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
    codex_lane: u16,
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
        lanes(report, options, &layout, &slots, first, &mut paint);
        panel(report, options, layout.panel, &mut paint);
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

fn visible_x(slot: usize, first: usize) -> Option<u16> {
    slot.checked_sub(first)
        .filter(|&slot| slot < 5)
        .map(|slot| 1 + 22 * slot as u16)
}

fn edge_x(slot: usize, first: usize, offset: u16, end: u16) -> Option<u16> {
    // The source box can be left of the window while its edge is still visible.
    let x = 1 + 22 * (slot as i128 - first as i128) + i128::from(offset);
    u16::try_from(x).ok().filter(|&x| x < end)
}

fn lanes(
    report: &GraphReport,
    options: &GraphOptions,
    layout: &Layout,
    slots: &[usize],
    first: usize,
    paint: &mut Paint<'_>,
) {
    for (i, part) in report.parts.iter().enumerate() {
        if let Some(x) = visible_x(slots[i], first) {
            let y = if part.harness == Harness::Claude {
                layout.claude_lane
            } else {
                layout.codex_lane
            };
            let selected = i == options.selected.min(report.parts.len() - 1);
            let card = part_box(part, selected, options);
            for dy in 0..9 {
                for dx in 0..30 {
                    if x + dx < paint.end() && y + dy < paint.buf.area.height {
                        paint.buf[(x + dx, y + dy)] = card[(dx, dy)].clone();
                    }
                }
            }
        }
        match &part.edge_in {
            Some(Edge::Handoff { .. }) => {
                if let Some(x) = edge_x(slots[i], first, 4, paint.end()) {
                    handoff(part, &report.step.id, x, options, paint);
                }
            }
            Some(Edge::Return { at }) => {
                if let Some(previous) = i.checked_sub(1)
                    && report.parts[previous].harness == Harness::Codex
                    && let Some(x) = edge_x(slots[previous], first, 26, paint.end())
                {
                    return_edge(x, *at, options, paint);
                }
            }
            None => {}
        }
    }
}

fn length(from: Timestamp, to: Timestamp) -> String {
    let seconds = ((to.epoch_ms() - from.epoch_ms()) / 1000).max(1);
    if seconds < 60 {
        format!("{seconds} s")
    } else {
        format!("{} min", seconds / 60)
    }
}

fn missing_run(part: &Part) -> String {
    match part.status {
        Some(PartStatus::Unlinked) => "no codex run found".to_owned(),
        Some(PartStatus::Ambiguous) => {
            let candidates = match part.edge_in {
                Some(Edge::Handoff { candidates, .. }) => candidates,
                _ => 0,
            };
            format!("{candidates} possible runs")
        }
        _ => "run unavailable".to_owned(),
    }
}

fn counts_label(counts: &Counts) -> String {
    [
        (counts.reads, "read"),
        (counts.edits, "edit"),
        (counts.runs, "run"),
    ]
    .into_iter()
    .filter(|(count, _)| *count > 0)
    .map(|(count, name)| format!("{count} {name}{}", if count == 1 { "" } else { "s" }))
    .collect::<Vec<_>>()
    .join(" · ")
}

/// A local card keeps clipping independent of the screen's dimensions.
fn part_box(part: &Part, selected: bool, options: &GraphOptions) -> Buffer {
    let t = options.theme;
    let codex = part.harness == Harness::Codex;
    let missing = codex && (part.from.is_none() || part.to.is_none());
    let color = if missing {
        t.secondary
    } else if codex {
        t.codex
    } else {
        t.claude
    };
    let bg = if selected {
        t.graph_selected
    } else if codex && !missing {
        t.codex_box
    } else {
        t.screen
    };
    let style = |fg| t.text_style(fg, bg);
    let mut card = Buffer::empty(Rect::new(0, 0, 30, 9));
    card.set_style(card.area, Style::default().bg(bg));
    card.set_string(0, 0, format!("╭{}╮", "─".repeat(28)), style(color));
    card.set_string(0, 8, format!("╰{}╯", "─".repeat(28)), style(color));
    for y in 1..8 {
        card.set_string(0, y, "│", style(color));
        card.set_string(29, y, "│", style(color));
    }
    card.set_string(1, 3, "┄".repeat(28), style(t.border));
    let harness = if codex { "codex" } else { "claude" };
    let kind_style = if selected {
        style(if missing { t.secondary } else { t.text }).add_modifier(Modifier::BOLD)
    } else {
        style(if missing { t.secondary } else { t.text })
    };
    card.set_line(
        2,
        1,
        &Line::from(vec![
            Span::styled("● ", style(color)),
            Span::styled(harness, style(color).add_modifier(Modifier::BOLD)),
            Span::styled(
                cut(&format!(" · {}", part.kind), 27 - 2 - harness.len()),
                kind_style,
            ),
        ]),
        27,
    );
    box_subline(&mut card, part, options, bg, missing);
    if !missing {
        card.set_stringn(
            2,
            4,
            cut(&counts_label(&part.counts), 27),
            27,
            style(t.text),
        );
        let count = if part.waiting_reads > 0 { 2 } else { 3 };
        let start = part.notable.len().saturating_sub(count);
        for (row, notable) in part.notable[start..].iter().enumerate() {
            card.set_line(2, 5 + row as u16, &notable_line(notable, t, bg), 27);
        }
        if part.waiting_reads > 0 {
            card.set_stringn(
                2,
                7,
                cut(&format!("{} reads while codex ran", part.waiting_reads), 27),
                27,
                style(t.secondary),
            );
        }
    }
    card
}

fn box_subline(card: &mut Buffer, part: &Part, options: &GraphOptions, bg: Color, missing: bool) {
    let t = options.theme;
    let style = |fg| t.text_style(fg, bg);
    if missing {
        card.set_stringn(2, 2, cut(&missing_run(part), 27), 27, style(t.secondary));
        return;
    }
    let (Some(from), Some(to)) = (part.from, part.to) else {
        return;
    };
    let duration = length(from, to);
    let (label, color) = if part.harness == Harness::Claude {
        (clock(from, options.offset, false), t.secondary)
    } else {
        match part.status {
            Some(PartStatus::Done) => (
                format!("✓ done {}", clock(to, options.offset, false)),
                t.done,
            ),
            Some(PartStatus::Stale) => (
                format!("stale since {}", clock(from, options.offset, false)),
                t.warning,
            ),
            _ => ("working now".to_owned(), t.claude),
        }
    };
    let suffix = if part.status == Some(PartStatus::Stale) {
        String::new()
    } else {
        format!(" · {duration}")
    };
    card.set_line(
        2,
        2,
        &Line::from(vec![
            Span::styled(label, style(color)),
            Span::styled(suffix, style(t.secondary)),
        ]),
        27,
    );
}

fn notable_line(notable: &Notable, theme: Theme, bg: Color) -> Line<'static> {
    let mark = match notable.ok {
        Some(false) => Some((" ✗", theme.warning)),
        Some(true)
            if notable.label.starts_with("$ cargo ")
                || notable.label == "$ pytest"
                || notable.label == "$ npm test" =>
        {
            Some((" ✓", theme.done))
        }
        _ => None,
    };
    let label = cut(&notable.label, if mark.is_some() { 25 } else { 27 });
    let style = |fg| theme.text_style(fg, bg);
    let mut spans = if let Some(rest) = label.strip_prefix('▸') {
        vec![
            Span::styled("▸", style(theme.claude)),
            Span::styled(rest.to_owned(), style(theme.text)),
        ]
    } else {
        vec![Span::styled(label, style(theme.text))]
    };
    if let Some((mark, color)) = mark {
        spans.push(Span::styled(mark, style(color)));
    }
    Line::from(spans)
}

fn handoff(part: &Part, step: &str, x: u16, options: &GraphOptions, paint: &mut Paint<'_>) {
    let Some(Edge::Handoff {
        at,
        r#match: confidence,
        candidates,
        ..
    }) = &part.edge_in
    else {
        return;
    };
    let t = options.theme;
    let glyph = if *confidence == Match::Exact {
        "│"
    } else {
        "╎"
    };
    for (y, glyph) in [(13, glyph), (14, glyph), (15, "▼")] {
        paint.write(x, y, glyph, t.text, paint.end());
    }
    let label = if part.kind == "implement" {
        format!("handed step {step}")
    } else {
        "handed fix".to_owned()
    };
    paint.write(x + 2, 13, &label, t.text, paint.end());
    paint.write(
        x + 2,
        14,
        &clock(*at, options.offset, true),
        t.secondary,
        paint.end(),
    );
    let words = match_words(*confidence, *candidates);
    paint.write(x + 2, 15, &words, t.secondary, paint.end());
}

fn match_words(confidence: Match, candidates: usize) -> String {
    match confidence {
        Match::Exact => "confirmed".to_owned(),
        Match::Heuristic => "matched by time".to_owned(),
        Match::Ambiguous => format!("{candidates} possible runs"),
        Match::Unlinked => "no run found".to_owned(),
    }
}

fn panel(report: &GraphReport, options: &GraphOptions, y: u16, paint: &mut Paint<'_>) {
    let Some(selected) = report
        .parts
        .len()
        .checked_sub(1)
        .map(|last| options.selected.min(last))
    else {
        return;
    };
    let part = &report.parts[selected];
    let t = options.theme;
    let codex = part.harness == Harness::Codex;
    let available = !codex || part.run_available == Some(true);
    let harness = if codex { "codex" } else { "claude" };
    paint.rule(y.saturating_sub(1));
    paint.write(2, y, "selected", t.secondary, paint.end());
    let mut detail = format!(" · {}", part.kind);
    if !available {
        detail.push_str(&format!(" · {}", missing_run(part)));
    } else {
        if let Some(session) = &part.session {
            detail.push_str(&format!(
                " · {}",
                session.chars().take(8).collect::<String>()
            ));
        }
        if let (Some(from), Some(to)) = (part.from, part.to) {
            detail.push_str(&format!(
                " · {}–{}",
                clock(from, options.offset, true),
                clock(to, options.offset, true)
            ));
        }
        detail.push_str(&format!(
            " · active about {} min in {} stretch{}",
            view::minutes(part.active_ms),
            part.stretches,
            if part.stretches == 1 { "" } else { "es" }
        ));
    }
    paint.line(
        12,
        y,
        Line::from(vec![
            Span::styled(
                harness,
                t.text_style(if codex { t.codex } else { t.claude }, t.screen)
                    .add_modifier(Modifier::BOLD),
            ),
            paint.span(detail, t.text),
        ]),
        paint.end(),
    );
    if codex && available {
        for (dy, label, quote) in [(1, "asked", &part.asked), (3, "said", &part.said)] {
            if let Some(quote) = quote {
                paint.write(12, y + dy, label, t.secondary, paint.end());
                let until = paint.end().min(109);
                let quote = quoted(quote, until.saturating_sub(19) as usize);
                paint.write(19, y + dy, &quote, t.text, until);
            }
        }
    }
    if let Some(origin) = panel_origin(report, selected, options.offset) {
        paint.write(12, y + 2, "from", t.secondary, paint.end());
        paint.write(19, y + 2, &origin, t.text, paint.end());
    }
    if !codex && part.untagged_handoffs > 0 {
        paint.write(
            19,
            y + 3,
            &format!(
                "{} untagged hand-off{} here",
                part.untagged_handoffs,
                if part.untagged_handoffs == 1 { "" } else { "s" }
            ),
            t.secondary,
            paint.end(),
        );
    } else if part.status == Some(PartStatus::Unlinked) {
        paint.write(
            19,
            y + 3,
            "(the rollout may be gone)",
            t.secondary,
            paint.end(),
        );
    }
    if available && part.session.is_some() {
        paint.line(
            12,
            y + 4,
            Line::from(vec![
                paint.span("o", t.text),
                paint.span(
                    format!(" open in {}   ", if codex { "Codex" } else { "Claude" }),
                    t.secondary,
                ),
                paint.span("f", t.text),
                paint.span(" fork", t.secondary),
            ]),
            paint.end(),
        );
    }
}

fn panel_origin(report: &GraphReport, selected: usize, offset: UtcOffset) -> Option<String> {
    let part = &report.parts[selected];
    if part.harness == Harness::Claude {
        let last = report
            .parts
            .iter()
            .filter_map(|child| {
                if let Some(Edge::Handoff {
                    at,
                    parent_part: Some(parent),
                    ..
                }) = &child.edge_in
                    && *parent == selected
                {
                    Some((child, *at))
                } else {
                    None
                }
            })
            .max_by_key(|(_, at)| *at);
        return Some(last.map_or_else(
            || "no hand-offs".to_owned(),
            |(child, at)| {
                let handed = if child.kind == "implement" {
                    format!("handed step {}", report.step.id)
                } else {
                    "handed fix".to_owned()
                };
                format!("{handed} to codex at {}", clock(at, offset, true))
            },
        ));
    }
    let Some(Edge::Handoff {
        at,
        parent,
        parent_part,
        r#match: confidence,
        gap_ms,
        candidates,
    }) = &part.edge_in
    else {
        return None;
    };
    let mut origin = format!("claude {}", parent.chars().take(8).collect::<String>());
    if let Some(parent) = parent_part.and_then(|index| report.parts.get(index)) {
        origin.push_str(&format!(" · {}", parent.kind));
    }
    origin.push_str(&format!(
        " · handed at {} · {}",
        clock(*at, offset, true),
        match_words(*confidence, *candidates)
    ));
    if let Some(gap) = gap_ms {
        let seconds = gap / 1000;
        let duration = if seconds < 60 {
            format!("{seconds} s")
        } else {
            format!("{} min", seconds / 60)
        };
        origin.push_str(&format!(" · started {duration} after the call"));
    }
    Some(origin)
}

/// Keep whole words within the display width, including quotes and ellipsis.
fn quoted(text: &str, width: usize) -> String {
    if width < 2 {
        return String::new();
    }
    if Span::raw(text).width() + 2 <= width {
        return format!("“{text}”");
    }
    if width < 3 {
        return "“”".to_owned();
    }
    let mut end = 0;
    for (index, ch) in text.char_indices() {
        if ch.is_whitespace() && Span::raw(&text[..index]).width() <= width - 3 {
            end = index;
        }
        if Span::raw(&text[..index + ch.len_utf8()]).width() > width - 3 {
            break;
        }
    }
    format!("“{}…”", text[..end].trim_end())
}

fn return_edge(x: u16, at: Timestamp, options: &GraphOptions, paint: &mut Paint<'_>) {
    let color = options.theme.secondary;
    for (y, glyph) in [(13, "▲"), (14, "│"), (15, "│")] {
        paint.write(x, y, glyph, color, paint.end());
    }
    paint.write(x + 2, 13, "back to claude", color, paint.end());
    paint.write(
        x + 2,
        14,
        &clock(at, options.offset, true),
        color,
        paint.end(),
    );
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
