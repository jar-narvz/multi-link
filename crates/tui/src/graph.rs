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

const BOX_HEIGHT: u16 = 9;
const CLAUDE_LANE: u16 = 4;
const EDGE_ROWS: u16 = 3;
const CODEX_LANE: u16 = CLAUDE_LANE + BOX_HEIGHT + EDGE_ROWS;
const LANES_END: u16 = CODEX_LANE + BOX_HEIGHT;
const PANEL: u16 = LANES_END + 2;
const PANEL_ROWS: u16 = 5;
const STRIP: u16 = PANEL + PANEL_ROWS + 1;
const STRIP_ROWS: u16 = 2;
const LEGEND: u16 = STRIP + STRIP_ROWS + 1;
const FOOTER_ROWS: u16 = 2;
const FULL_HEIGHT: u16 = LEGEND + 2 + FOOTER_ROWS;
const MIN_PANEL_HEIGHT: u16 = PANEL + 1 + FOOTER_ROWS;
const MIN_LEGEND_HEIGHT: u16 = PANEL + PANEL_ROWS + 2 + FOOTER_ROWS;

#[derive(Clone, Copy, Debug, PartialEq)]
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
    first_visible_slot(
        &slots,
        selected.min(slots.len().saturating_sub(1)),
        first_slot,
    )
}

fn first_visible_slot(slots: &[usize], selected: usize, first_slot: usize) -> usize {
    let Some(&last) = slots.last() else { return 0 };
    let selected = slots[selected];
    first_slot
        .min(last.saturating_sub(4))
        .min(selected)
        .max(selected.saturating_sub(4))
}

/// Keep the lanes fixed, dropping the strip, legend, then panel detail as needed.
/// Body rows retain their coordinates and are clipped above the bottom rule.
struct Layout {
    header: [u16; 2],
    header_rule: u16,
    scroll: u16,
    claude_lane: u16,
    codex_lane: u16,
    edges: u16,
    panel: u16,
    panel_rows: u16,
    panel_rule: Option<u16>,
    strip: Option<u16>,
    legend: Option<u16>,
    rule: u16,
    footer: u16,
}

impl Layout {
    fn new(height: u16) -> Self {
        let rule = height.saturating_sub(FOOTER_ROWS);
        let compact = height < MIN_PANEL_HEIGHT;
        let panel = if compact {
            if rule > CLAUDE_LANE - 1 {
                LANES_END.min(rule - 1)
            } else {
                rule
            }
        } else {
            PANEL
        };
        let panel_rows = rule
            .saturating_sub(panel)
            .min(if compact { 1 } else { PANEL_ROWS });
        let legend = (height >= MIN_LEGEND_HEIGHT).then(|| {
            // Keep a blank before the rule whenever the full panel leaves room.
            let blank = u16::from(height > MIN_LEGEND_HEIGHT);
            LEGEND.min(height - FOOTER_ROWS - 1 - blank)
        });
        Self {
            header: [0, 1],
            header_rule: CLAUDE_LANE - 2,
            scroll: CLAUDE_LANE - 1,
            claude_lane: CLAUDE_LANE,
            codex_lane: CODEX_LANE,
            edges: CODEX_LANE - EDGE_ROWS,
            panel,
            panel_rows,
            panel_rule: (panel_rows > 0 && !compact).then(|| panel - 1),
            strip: (height >= FULL_HEIGHT).then_some(STRIP),
            legend,
            rule,
            footer: height.saturating_sub(1),
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
    let options = GraphOptions {
        selected: options.selected.min(report.parts.len().saturating_sub(1)),
        ..*options
    };
    let options = &options;
    let layout = Layout::new(area.height);
    let mut paint = Paint {
        buf: &mut screen,
        theme: options.theme,
        bottom: layout.rule,
    };
    header(report, options, &layout, &mut paint);
    paint.rule(layout.header_rule);
    if report.empty.is_none() && layout.panel_rows > 0 {
        paint.bottom = layout.panel_rule.unwrap_or(layout.panel);
    }
    if let Some(empty) = &report.empty {
        let y = layout.claude_lane.min(layout.rule.saturating_sub(1));
        paint.clear_row(y);
        paint.write(2, y, empty, options.theme.secondary, paint.end());
    } else if area.width < 120 {
        list(report, options, &layout, &mut paint);
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
    }
    if report.empty.is_none() {
        paint.bottom = layout.rule;
        panel(report, options, &layout, &mut paint);
        if let Some(y) = layout.strip {
            strip(report, options, y, &mut paint);
        }
        if let Some(y) = layout.legend {
            legend(y, &mut paint);
        }
    }
    let mut paint = Paint {
        buf: &mut screen,
        theme: options.theme,
        bottom: area.height,
    };
    paint.clear_row(layout.rule);
    paint.clear_row(layout.footer);
    paint.rule(layout.rule);
    footer(options, layout.footer, &mut paint);
    options.theme.apply_no_color(&mut screen);
    for y in 0..area.height {
        for x in 0..area.width {
            buf[(area.x + x, area.y + y)] = screen[(x, y)].clone();
        }
    }
}

/// Two rows per part, with its incoming edge on the preceding indented row.
fn list(report: &GraphReport, options: &GraphOptions, layout: &Layout, paint: &mut Paint<'_>) {
    let t = options.theme;
    let mut rows = Vec::new();
    let mut starts = Vec::new();
    for part in &report.parts {
        if let Some(edge) = &part.edge_in {
            let line = match edge {
                Edge::Handoff {
                    at,
                    r#match,
                    candidates,
                    ..
                } => {
                    let handed = if part.kind == "implement" {
                        format!("handed step {}", report.step.id)
                    } else {
                        "handed fix".to_owned()
                    };
                    let glyph = if *r#match == Match::Exact {
                        "│"
                    } else {
                        "╎"
                    };
                    Line::from(vec![
                        paint.span(format!("{glyph} {handed}"), t.text),
                        paint.span(
                            format!(
                                " · {} · {}",
                                clock(*at, options.offset, true),
                                match_words(*r#match, *candidates)
                            ),
                            t.secondary,
                        ),
                    ])
                }
                Edge::Return { at } => Line::from(paint.span(
                    format!("▲ back to claude · {}", clock(*at, options.offset, true)),
                    t.secondary,
                )),
            };
            rows.push((3, line, None));
        }
        let index = starts.len();
        starts.push(rows.len());
        let missing = part.harness == Harness::Codex && !has_run(part);
        let color = if missing {
            t.secondary
        } else if part.harness == Harness::Codex {
            t.codex
        } else {
            t.claude
        };
        let bg = if index == options.selected {
            t.graph_selected
        } else {
            t.screen
        };
        let style = |fg| t.text_style(fg, bg);
        rows.push((
            1,
            Line::from(vec![
                Span::styled("● ", style(color)),
                Span::styled(
                    if part.harness == Harness::Codex {
                        "codex"
                    } else {
                        "claude"
                    },
                    style(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" · {}", part.kind),
                    if index == options.selected {
                        style(if missing { t.secondary } else { t.text })
                            .add_modifier(Modifier::BOLD)
                    } else {
                        style(if missing { t.secondary } else { t.text })
                    },
                ),
            ]),
            Some(bg),
        ));
        let mut spans = Vec::new();
        if let Some((label, role, suffix)) = subline(part, options.offset) {
            spans.push(Span::styled(label, style(role.color(t))));
            spans.push(Span::styled(suffix, style(t.secondary)));
        }
        let counts = counts_label(&part.counts);
        if !missing && !counts.is_empty() {
            if !spans.is_empty() {
                spans.push(Span::styled(" · ", style(t.secondary)));
            }
            spans.push(Span::styled(counts, style(t.text)));
        }
        rows.push((3, Line::from(spans), Some(bg)));
    }
    let end = layout.panel_rule.unwrap_or(layout.panel).min(layout.rule);
    let room = usize::from(end.saturating_sub(layout.claude_lane));
    let selected = starts.get(options.selected).copied().unwrap_or(0);
    let first = (selected + 2)
        .saturating_sub(room)
        .min(rows.len().saturating_sub(room));
    for (dy, (x, line, bg)) in rows.into_iter().skip(first).take(room).enumerate() {
        let y = layout.claude_lane + dy as u16;
        if let Some(bg) = bg {
            paint.fill(Rect::new(1, y, paint.end().saturating_sub(1), 1), bg);
        }
        paint.line(x, y, line, paint.end());
    }
}

fn slot_column(slot: usize, first: usize) -> i64 {
    1 + 22 * (slot as i64 - first as i64)
}

fn visible_x(slot: usize, first: usize) -> Option<u16> {
    slot.checked_sub(first)
        .filter(|&slot| slot < 5)
        .map(|_| slot_column(slot, first) as u16)
}

fn edge_x(slot: usize, first: usize, offset: u16, end: u16) -> Option<u16> {
    // Only return edges can remain visible when their source box is left of the window.
    let x = slot_column(slot, first) + i64::from(offset);
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
            let selected = i == options.selected;
            let card = part_box(part, selected, options);
            for dy in 0..BOX_HEIGHT {
                for dx in 0..30 {
                    if x + dx < paint.end() && y + dy < paint.bottom {
                        paint.buf[(x + dx, y + dy)] = card[(dx, dy)].clone();
                    }
                }
            }
        }
        if visible_x(slots[i], first).is_none() {
            continue;
        }
        match &part.edge_in {
            Some(Edge::Handoff { .. }) => {
                if let Some(x) = edge_x(slots[i], first, 4, paint.end()) {
                    handoff(part, &report.step.id, x, layout.edges, options, paint);
                }
            }
            Some(Edge::Return { at }) => {
                if let Some(previous) = i.checked_sub(1)
                    && report.parts[previous].harness == Harness::Codex
                    && let Some(x) = edge_x(slots[previous], first, 26, paint.end())
                {
                    return_edge(x, layout.edges, *at, options, paint);
                }
            }
            None => {}
        }
    }
}

fn length(milliseconds: i64) -> String {
    let seconds = (milliseconds / 1000).max(1);
    if seconds < 60 {
        format!("{seconds} s")
    } else {
        format!("{} min", seconds / 60)
    }
}

pub(crate) fn missing_run(part: &Part) -> String {
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
    .map(|(count, name)| plural(count, name))
    .collect::<Vec<_>>()
    .join(" · ")
}

fn short_id(id: &str) -> String {
    id.chars().take(8).collect()
}

fn plural(n: usize, word: &str) -> String {
    let suffix = if n == 1 {
        ""
    } else if word.ends_with("ch") {
        "es"
    } else {
        "s"
    };
    format!("{n} {word}{suffix}")
}

/// A local card keeps clipping independent of the screen's dimensions.
fn part_box(part: &Part, selected: bool, options: &GraphOptions) -> Buffer {
    let t = options.theme;
    let codex = part.harness == Harness::Codex;
    let missing = codex && !has_run(part);
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
    let mut card = Buffer::empty(Rect::new(0, 0, 30, BOX_HEIGHT));
    card.set_style(card.area, Style::default().bg(bg));
    card.set_string(0, 0, format!("╭{}╮", "─".repeat(28)), style(color));
    card.set_string(
        0,
        BOX_HEIGHT - 1,
        format!("╰{}╯", "─".repeat(28)),
        style(color),
    );
    for y in 1..BOX_HEIGHT - 1 {
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
    box_subline(&mut card, part, options, bg);
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

pub(crate) fn has_run(part: &Part) -> bool {
    part.run_available == Some(true) && part.from.is_some() && part.to.is_some()
}

#[derive(Clone, Copy)]
enum Role {
    Secondary,
    Done,
    Warning,
    Claude,
}

impl Role {
    fn color(self, theme: Theme) -> Color {
        match self {
            Self::Secondary => theme.secondary,
            Self::Done => theme.done,
            Self::Warning => theme.warning,
            Self::Claude => theme.claude,
        }
    }
}

/// Status text and its role, followed by a secondary-colour duration suffix.
fn subline(part: &Part, offset: UtcOffset) -> Option<(String, Role, String)> {
    if part.harness == Harness::Codex && !has_run(part) {
        return Some((missing_run(part), Role::Secondary, String::new()));
    }
    let (from, to) = (part.from?, part.to?);
    let (label, role) = if part.harness == Harness::Claude {
        (clock(from, offset, false), Role::Secondary)
    } else {
        match part.status {
            Some(PartStatus::Done) => (format!("✓ done {}", clock(to, offset, false)), Role::Done),
            Some(PartStatus::Stale) => (
                format!("stale since {}", clock(to, offset, false)),
                Role::Warning,
            ),
            _ => ("working now".to_owned(), Role::Claude),
        }
    };
    let suffix = if part.status == Some(PartStatus::Stale) {
        String::new()
    } else {
        format!(" · {}", length(to.epoch_ms() - from.epoch_ms()))
    };
    Some((label, role, suffix))
}

fn box_subline(card: &mut Buffer, part: &Part, options: &GraphOptions, bg: Color) {
    let Some((label, role, suffix)) = subline(part, options.offset) else {
        return;
    };
    let t = options.theme;
    card.set_line(
        2,
        2,
        &Line::from(vec![
            Span::styled(cut(&label, 27), t.text_style(role.color(t), bg)),
            Span::styled(suffix, t.text_style(t.secondary, bg)),
        ]),
        27,
    );
}

struct Mark {
    text: &'static str,
    role: Role,
}

fn notable_mark(notable: &Notable) -> Option<Mark> {
    match notable.ok {
        Some(false) => Some(Mark {
            text: " ✗",
            role: Role::Warning,
        }),
        Some(true)
            if notable.label.starts_with("$ cargo ")
                || notable.label == "$ pytest"
                || notable.label == "$ npm test" =>
        {
            Some(Mark {
                text: " ✓",
                role: Role::Done,
            })
        }
        _ => None,
    }
}

fn notable_line(notable: &Notable, theme: Theme, bg: Color) -> Line<'static> {
    let mark = notable_mark(notable);
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
    if let Some(mark) = mark {
        spans.push(Span::styled(mark.text, style(mark.role.color(theme))));
    }
    Line::from(spans)
}

fn handoff(part: &Part, step: &str, x: u16, y: u16, options: &GraphOptions, paint: &mut Paint<'_>) {
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
    for (y, glyph) in [(y, glyph), (y + 1, glyph), (y + 2, "▼")] {
        paint.write(x, y, glyph, t.text, paint.end());
    }
    let label = if part.kind == "implement" {
        format!("handed step {step}")
    } else {
        "handed fix".to_owned()
    };
    paint.write(x + 2, y, &cut(&label, 19), t.text, paint.end());
    paint.write(
        x + 2,
        y + 1,
        &cut(&clock(*at, options.offset, true), 19),
        t.secondary,
        paint.end(),
    );
    let words = match_words(*confidence, *candidates);
    paint.write(x + 2, y + 2, &cut(&words, 19), t.secondary, paint.end());
}

fn match_words(confidence: Match, candidates: usize) -> String {
    match confidence {
        Match::Exact => "confirmed".to_owned(),
        Match::Heuristic => "matched by time".to_owned(),
        Match::Ambiguous => format!("{candidates} possible runs"),
        Match::Unlinked => "no run found".to_owned(),
    }
}

fn panel(report: &GraphReport, options: &GraphOptions, layout: &Layout, paint: &mut Paint<'_>) {
    let y = layout.panel;
    let rows = layout.panel_rows;
    let visible = |dy| dy < rows;
    if !visible(0) {
        return;
    }
    let selected = options.selected;
    let Some(part) = report.parts.get(selected) else {
        return;
    };
    let t = options.theme;
    let codex = part.harness == Harness::Codex;
    let available = !codex || has_run(part);
    let session = part.session.as_deref().filter(|id| !id.is_empty());
    let harness = if codex { "codex" } else { "claude" };
    paint.clear_row(y);
    if let Some(y) = layout.panel_rule {
        paint.rule(y);
    }
    paint.write(2, y, "selected", t.secondary, paint.end());
    let mut detail = format!(" · {}", part.kind);
    if !available {
        detail.push_str(&format!(" · {}", missing_run(part)));
    } else {
        if let Some(session) = session {
            detail.push_str(&format!(" · {}", short_id(session)));
        }
        if let (Some(from), Some(to)) = (part.from, part.to) {
            detail.push_str(&format!(
                " · {}–{}",
                clock(from, options.offset, true),
                clock(to, options.offset, true)
            ));
        }
        detail.push_str(&format!(
            " · active about {} min in {}",
            view::minutes(part.active_ms),
            plural(part.stretches, "stretch")
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
            if visible(dy)
                && let Some(quote) = quote.as_deref().filter(|text| !text.is_empty())
            {
                paint.write(12, y + dy, label, t.secondary, paint.end());
                let until = paint.end().min(109);
                let quote = quoted(quote, until.saturating_sub(19) as usize);
                paint.write(19, y + dy, &quote, t.text, until);
            }
        }
    }
    if visible(2)
        && let Some(origin) = panel_origin(report, part, selected, options.offset)
    {
        paint.write(12, y + 2, "from", t.secondary, paint.end());
        paint.write(19, y + 2, &origin, t.text, paint.end());
    }
    if visible(3) && !codex && part.untagged_handoffs > 0 {
        paint.write(
            19,
            y + 3,
            &format!(
                "{} here",
                plural(part.untagged_handoffs, "untagged hand-off")
            ),
            t.secondary,
            paint.end(),
        );
    } else if visible(3) && part.status == Some(PartStatus::Unlinked) {
        paint.write(
            19,
            y + 3,
            "(the rollout may be gone)",
            t.secondary,
            paint.end(),
        );
    }
    if visible(4) && available && session.is_some() {
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

fn panel_origin(
    report: &GraphReport,
    part: &Part,
    selected: usize,
    offset: UtcOffset,
) -> Option<String> {
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
    let mut origin = format!("claude {}", short_id(parent));
    if let Some(parent) = parent_part.and_then(|index| report.parts.get(index)) {
        origin.push_str(&format!(" · {}", parent.kind));
    }
    origin.push_str(&format!(
        " · handed at {} · {}",
        clock(*at, offset, true),
        match_words(*confidence, *candidates)
    ));
    if let Some(gap) = gap_ms.filter(|gap| *gap >= 0) {
        origin.push_str(&format!(" · started {} after the call", length(gap)));
    }
    Some(origin)
}

fn quoted(text: &str, width: usize) -> String {
    if width < 2 {
        return String::new();
    }
    format!("“{}”", cut_words(text, width - 2))
}

/// Keep the latest whole-word boundary that fits, or cut an over-long word.
fn cut_words(text: &str, width: usize) -> String {
    let text = without_controls(text);
    if Span::raw(&text).width() <= width {
        return text;
    }
    if width == 0 {
        return String::new();
    }
    let mut end = 0;
    let mut used = 0;
    for (index, ch) in text.char_indices() {
        if ch.is_whitespace() {
            end = index;
        }
        let mut bytes = [0; 4];
        let encoded: &str = ch.encode_utf8(&mut bytes);
        used += Span::raw(encoded).width();
        if used >= width {
            break;
        }
    }
    if end == 0 {
        return cut(&text, width);
    }
    let prefix = text[..end].trim_end().trim_end_matches('·').trim_end();
    format!("{prefix}…")
}

fn without_controls(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_control()).collect()
}

fn return_edge(x: u16, y: u16, at: Timestamp, options: &GraphOptions, paint: &mut Paint<'_>) {
    let color = options.theme.secondary;
    for (y, glyph) in [(y, "▲"), (y + 1, "│"), (y + 2, "│")] {
        paint.write(x, y, glyph, color, paint.end());
    }
    paint.write(x + 2, y, &cut("back to claude", 19), color, paint.end());
    paint.write(
        x + 2,
        y + 1,
        &cut(&clock(at, options.offset, true), 19),
        color,
        paint.end(),
    );
}

struct Paint<'a> {
    buf: &'a mut Buffer,
    theme: Theme,
    bottom: u16,
}

impl Paint<'_> {
    fn fill(&mut self, area: Rect, bg: Color) {
        let clip = Rect::new(0, 0, self.buf.area.width, self.bottom);
        self.buf
            .set_style(area.intersection(clip), Style::default().bg(bg));
    }

    // Exclusive right edge, preserving the screen's outer blank column.
    fn end(&self) -> u16 {
        self.buf.area.width.saturating_sub(1)
    }

    fn line(&mut self, x: u16, y: u16, line: Line<'_>, until: u16) {
        let until = until.min(self.end());
        if y < self.bottom && x < until {
            let line = if self.buf.area.width < 120 {
                word_line(line, usize::from(until - x))
            } else {
                line
            };
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

    fn clear_row(&mut self, y: u16) {
        if y < self.bottom {
            for x in 0..self.buf.area.width {
                self.buf[(x, y)].reset();
                self.buf[(x, y)].set_bg(self.theme.screen);
            }
        }
    }
}

/// Preserve span colours while applying the same word cut as panel quotes.
fn word_line(line: Line<'_>, width: usize) -> Line<'_> {
    let text = line.to_string();
    let shortened = cut_words(&text, width);
    if shortened == text {
        return line;
    }
    let mut remaining = shortened
        .strip_suffix('…')
        .unwrap_or(&shortened)
        .chars()
        .count();
    let mut spans = Vec::new();
    let mut style = Style::default();
    for span in line.spans {
        style = span.style;
        let prefix: String = span.content.chars().take(remaining).collect();
        remaining -= prefix.chars().count();
        spans.push(Span::styled(prefix, style));
        if remaining == 0 {
            break;
        }
    }
    if shortened.ends_with('…') {
        spans.push(Span::styled("…", style));
    }
    Line::from(spans)
}

/// Cut by character and display width, keeping an ellipsis for a long word.
fn cut(text: &str, width: usize) -> String {
    let text = without_controls(text);
    if Span::raw(&text).width() <= width {
        return text;
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
    let length = length(to.epoch_ms() - from.epoch_ms());
    let sessions = report
        .parts
        .iter()
        .filter(|p| p.harness == Harness::Claude)
        .filter_map(|p| p.session.as_deref())
        .filter(|id| !id.is_empty())
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
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    format!(
        "{} {} · {}–{} · {length} · {} · {} · {}",
        MONTHS[date.month() as usize - 1],
        date.day(),
        clock(from, offset, false),
        clock(to, offset, false),
        plural(sessions, "claude session"),
        plural(runs, "codex run"),
        plural(fixes, "fix round")
    )
}

fn header(report: &GraphReport, options: &GraphOptions, layout: &Layout, paint: &mut Paint<'_>) {
    let t = options.theme;
    let narrow = paint.buf.area.width < 120;
    let fit = if narrow { cut_words } else { cut };
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
    let prefix = if narrow {
        Line::from(vec![
            Span::styled(basename(&report.repo).to_owned(), bold),
            paint.span(" › ", t.secondary),
            Span::styled(step, bold),
        ])
    } else {
        Line::from(vec![
            Span::styled(cut(basename(&report.repo), repo_width), bold),
            paint.span(" › ", t.secondary),
            Span::styled(step, bold),
            paint.span("  ", t.text),
        ])
    };
    let title_x = 1usize.saturating_add(prefix.width()) + if narrow { 2 } else { 0 };
    paint.line(1, layout.header[0], prefix, title_end);
    if title_x < usize::from(title_end) {
        let title = fit(
            report.step.title.as_deref().unwrap_or(""),
            usize::from(title_end) - title_x,
        );
        if !narrow || title != "…" {
            paint.write(title_x as u16, layout.header[0], &title, t.text, title_end);
        }
    }
    let left = if report.parts.is_empty() {
        String::new()
    } else {
        summary(report, options.offset)
    };
    let left = fit(&left, paint.end().saturating_sub(1) as usize);
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
            // cut_words permits a fallback for a single overlong word. A
            // command with arguments must retain at least its whole first word.
            if narrow
                && verify.split_whitespace().nth(1).is_some()
                && verify
                    .split_whitespace()
                    .next()
                    .is_some_and(|word| Span::raw(word).width() > available - fixed)
            {
                return;
            }
            let command = fit(verify, available - fixed);
            paint.right(
                layout.header[1],
                &format!("{prefix}{command}{suffix}"),
                t.secondary,
            );
        }
    }
}

/// Reserve label minima, then apportion the remaining cells by elapsed time.
/// Integer remainders keep equal shares deterministic, with earlier parts first.
fn strip_widths(parts: &[Part], labels: &[&str], span: usize) -> Option<(Vec<usize>, bool)> {
    let available = span.checked_sub(parts.len().saturating_sub(1))?;
    if parts.is_empty() || available < parts.len() {
        return None;
    }
    let mut widths: Vec<_> = labels
        .iter()
        .map(|label| Span::raw(*label).width() + 2)
        .collect();
    let show_labels = widths.iter().sum::<usize>() <= available;
    if !show_labels {
        widths.fill(1);
    }
    let extra = available - widths.iter().sum::<usize>();
    let mut lengths: Vec<u128> = parts
        .iter()
        .map(|part| match (part.from, part.to) {
            (Some(from), Some(to)) => {
                (i128::from(to.epoch_ms()) - i128::from(from.epoch_ms())).max(0) as u128
            }
            _ => 0,
        })
        .collect();
    // With no elapsed time to distinguish parts, share the space equally.
    if lengths.iter().all(|&length| length == 0) {
        lengths.fill(1);
    }
    let total: u128 = lengths.iter().sum();
    let mut remainders = Vec::with_capacity(parts.len());
    let mut left = extra;
    for (index, length) in lengths.into_iter().enumerate() {
        let share = length * extra as u128;
        let whole = (share / total) as usize;
        widths[index] += whole;
        left -= whole;
        remainders.push((index, share % total));
    }
    remainders.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    for &(index, _) in remainders.iter().take(left) {
        widths[index] += 1;
    }
    Some((widths, show_labels))
}

fn strip(report: &GraphReport, options: &GraphOptions, y: u16, paint: &mut Paint<'_>) {
    let t = options.theme;
    let end = if paint.buf.area.width < 120 {
        paint.end().saturating_sub(2)
    } else {
        117
    };
    paint.write(2, y, "where the time went", t.secondary, paint.end());
    // The time range describes the whole report, even when parts overlap or
    // there are too many parts to draw with a cell and a gap each.
    if let Some(from) = report.parts.iter().filter_map(|part| part.from).min() {
        paint.write(
            23,
            y + 1,
            &clock(from, options.offset, false),
            t.secondary,
            end,
        );
    }
    if let Some(to) = report.parts.iter().filter_map(|part| part.to).max() {
        paint.write(
            end.saturating_sub(5),
            y + 1,
            &clock(to, options.offset, false),
            t.secondary,
            end,
        );
    }
    let labels: Vec<_> = report
        .parts
        .iter()
        .map(|part| {
            if part.kind.starts_with("fix round ") {
                "fix"
            } else {
                part.kind.as_str()
            }
        })
        .collect();
    let Some((widths, show_labels)) =
        strip_widths(&report.parts, &labels, usize::from(end.saturating_sub(23)))
    else {
        return;
    };
    let selected = options.selected;
    let mut x = 23;
    for (index, ((part, label), width)) in report.parts.iter().zip(labels).zip(widths).enumerate() {
        let codex = part.harness == Harness::Codex;
        let bg = if index == selected {
            t.graph_selected
        } else if codex {
            t.codex_strip
        } else {
            t.claude_strip
        };
        let area = Rect::new(x, y, width as u16, 1).intersection(paint.buf.area);
        paint.buf.set_style(area, Style::default().bg(bg));
        if show_labels {
            paint.line(
                x + 1,
                y,
                Line::from(Span::styled(
                    label,
                    t.text_style(if codex { t.codex } else { t.claude }, bg),
                )),
                x + width as u16,
            );
        }
        x += width as u16 + 1;
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
    let zone = format!("times {}", view::offset_label(options.offset));
    let until = paint
        .end()
        .saturating_sub(Span::raw(&zone).width() as u16 + 3);
    let narrow = paint.buf.area.width < 120;
    let mut spans = Vec::new();
    let mut hints = [
        (
            if paint.buf.area.width < 120 {
                "↑↓"
            } else {
                "←→"
            },
            " move along the step",
        ),
        ("[ ]", " prev / next step"),
        ("o", " open"),
        ("f", " fork"),
        ("esc", " repo view"),
        ("q", " quit"),
    ];
    let hints_width: usize = hints
        .iter()
        .map(|(key, words)| Span::raw(*key).width() + Span::raw(*words).width())
        .sum::<usize>()
        + 3 * (hints.len() - 1);
    if narrow && hints_width > usize::from(until.saturating_sub(1)) {
        hints[1..].rotate_right(2);
    }
    for (i, (key, words)) in hints.into_iter().enumerate() {
        if narrow {
            let used: usize = spans.iter().map(Span::width).sum();
            let gap = if i > 0 { 3 } else { 0 };
            if used + gap + Span::raw(key).width() + Span::raw(words).width()
                > usize::from(until.saturating_sub(1))
            {
                break;
            }
        }
        if i > 0 {
            spans.push(paint.span("   ", t.secondary));
        }
        spans.push(paint.span(key, t.text));
        spans.push(paint.span(words, t.secondary));
    }
    paint.line(1, y, Line::from(spans), until);
    paint.right(y, &zone, t.secondary);
}
