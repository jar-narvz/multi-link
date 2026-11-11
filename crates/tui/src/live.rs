//! Pure live grid presentation (Phase 14).
use crate::{theme::Theme, view};
use multi_link_model::{
    Card, CardLink, Harness, InvocationForm, LiveOutcome, LiveReport, Match, State, Timestamp,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
};

use time::UtcOffset;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LiveOptions {
    pub selected: usize,
    /// First wide card row, or first flattened card below 120 columns.
    pub first_row: usize,
    pub theme: Theme,
    pub offset: UtcOffset,
}
impl Default for LiveOptions {
    fn default() -> Self {
        Self {
            selected: 0,
            first_row: 0,
            theme: Theme::from_env(None, None),
            offset: UtcOffset::UTC,
        }
    }
}

// A local canvas keeps every write inside the caller's area, including clipped cards.
struct Canvas<'a> {
    buf: &'a mut Buffer,
    area: Rect,
    theme: Theme,
}
impl Canvas<'_> {
    fn line(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        spans: Vec<Span<'_>>,
        bg: Color,
        bold: bool,
    ) {
        if y >= self.area.height as usize || x >= self.area.width as usize {
            return;
        }
        let width = width.min(self.area.width as usize - x);
        let mut line = Line::from(spans);
        if line.width() > width {
            let mut remaining = width.saturating_sub(1);
            let mut cut = Vec::new();
            let mut last = Style::default();
            for grapheme in line.styled_graphemes(Style::default()) {
                last = grapheme.style;
                let cells = Span::raw(grapheme.symbol).width();
                if cells > remaining {
                    break;
                }
                remaining -= cells;
                cut.push(Span::styled(grapheme.symbol.to_string(), grapheme.style));
            }
            if width > 0 {
                cut.push(Span::styled("…", last));
            }
            line = Line::from(cut);
        }
        let mut style = Style::default().bg(bg).remove_modifier(Modifier::BOLD);
        if bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        self.buf.set_line(
            self.area.x + x as u16,
            self.area.y + y as u16,
            &line.style(style),
            width as u16,
        );
    }
    fn text(&mut self, x: usize, y: usize, width: usize, text: impl Into<String>, fg: Color) {
        self.line(x, y, width, vec![span(text, fg)], self.theme.screen, false);
    }
    fn rule(&mut self, y: usize) {
        self.text(
            1,
            y,
            self.area.width.saturating_sub(2) as usize,
            "─".repeat(self.area.width.saturating_sub(2) as usize),
            self.theme.border,
        );
    }
}
fn span(text: impl Into<String>, color: Color) -> Span<'static> {
    Span::styled(
        text.into()
            .chars()
            .filter(|c| !c.is_control())
            .collect::<String>(),
        Style::default().fg(color),
    )
}
fn bold(text: impl Into<String>, color: Color) -> Span<'static> {
    span(text, color).add_modifier(Modifier::BOLD)
}
fn name(path: &str) -> &str {
    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(path)
}
fn id(id: &str) -> String {
    id.chars().take(8).collect()
}
fn harness(card: &Card) -> &str {
    if card.harness == Harness::Claude {
        "claude"
    } else {
        "codex"
    }
}
fn place(card: &Card) -> String {
    if card.checkout == card.repo {
        "main".into()
    } else {
        format!("{} worktree", name(&card.checkout))
    }
}
fn clock(at: Timestamp, o: &LiveOptions) -> String {
    view::clock_label(view::datetime(at).to_offset(o.offset), false)
}
fn period(r: &LiveReport) -> Option<String> {
    r.window_from.map(|from| {
        let m = (r.now.epoch_ms() - from.epoch_ms()) / 60_000;
        if m % 60 == 0 {
            format!("{}h", m / 60)
        } else {
            format!("{m}m")
        }
    })
}
fn confidence(m: Match, candidates: usize, t: Theme) -> Span<'static> {
    match m {
        Match::Exact => span("confirmed link", t.text),
        Match::Heuristic => span("matched by time", t.secondary),
        Match::Ambiguous => span(format!("{candidates} possible runs"), t.needs_you),
        _ => span("no run found", t.secondary),
    }
}
fn header(c: &mut Canvas<'_>, r: &LiveReport, o: &LiveOptions) {
    let t = c.theme;
    let w = c.area.width as usize;
    let mut left = vec![
        bold("multi-link", t.text),
        span(" › ", t.secondary),
        bold("live", t.text),
        span("  what every agent is doing now", t.secondary),
    ];
    let counts = |words: bool| {
        vec![
            span(
                format!(
                    "◆ {}{}",
                    r.counts.waiting,
                    if words { " waiting" } else { "" }
                ),
                if r.counts.waiting == 0 {
                    t.secondary
                } else {
                    t.needs_you
                },
            ),
            span(" · ", t.secondary),
            span(
                format!(
                    "● {}{}",
                    r.counts.busy,
                    if words { " working now" } else { "" }
                ),
                if r.counts.busy == 0 {
                    t.secondary
                } else {
                    t.claude
                },
            ),
            span(" · ", t.secondary),
            span(
                format!("○ {}{}", r.counts.stale, if words { " stale" } else { "" }),
                t.secondary,
            ),
        ]
    };
    let mut right = counts(true);
    right.push(span(format!("   {}", clock(r.now, o)), t.text));
    let fits = |a: &[Span<'_>], b: &[Span<'_>]| {
        a.iter().map(Span::width).sum::<usize>() + b.iter().map(Span::width).sum::<usize>() + 4 <= w
    };
    if !fits(&left, &right) {
        left.pop();
    }
    if !fits(&left, &right) {
        right = counts(true);
    }
    if !fits(&left, &right) {
        right = counts(false);
    }
    let rw = right.iter().map(Span::width).sum::<usize>();
    c.line(1, 0, w.saturating_sub(rw + 3), left, t.screen, false);
    c.line(w.saturating_sub(rw + 1), 0, rw, right, t.screen, false);
    c.rule(1);
}
fn feed(c: &mut Canvas<'_>, r: &LiveReport, o: &LiveOptions, k: usize) {
    if k == 0 {
        return;
    }
    let t = c.theme;
    let w = c.area.width as usize;
    c.line(
        1,
        2,
        w.saturating_sub(2),
        vec![
            bold("HAND-OFFS", t.secondary),
            span(" newest first · all repos", t.secondary),
        ],
        t.screen,
        false,
    );
    if r.feed.is_empty() {
        c.text(
            1,
            3,
            w.saturating_sub(2),
            period(r).map_or("no hand-offs".into(), |p| {
                format!("no hand-offs in the last {p}")
            }),
            t.secondary,
        );
        return;
    }
    let rows: Vec<_> = r
        .feed
        .iter()
        .take(k)
        .map(|row| {
            let work = row
                .step
                .as_ref()
                .map_or("work".into(), |s| format!("step {s}"));
            let mut sentence = vec![
                span(
                    if row.form == InvocationForm::Resume {
                        "claude resumed codex for "
                    } else {
                        "claude handed "
                    },
                    t.secondary,
                ),
                span(work, t.text),
            ];
            if row.form != InvocationForm::Resume {
                sentence.push(span(" to codex", t.secondary));
            }
            (row, sentence)
        })
        .collect();
    let cx = if w >= 120 {
        75
    } else {
        23 + rows
            .iter()
            .map(|(_, s)| s.iter().map(Span::width).sum::<usize>())
            .max()
            .unwrap_or(0)
            + 3
    };
    let ox = if w >= 120 {
        93
    } else {
        cx + rows
            .iter()
            .map(|(r, _)| confidence(r.r#match, r.candidates, t).width())
            .max()
            .unwrap_or(0)
            + 3
    };
    for (i, (row, sentence)) in rows.into_iter().enumerate() {
        let y = 3 + i;
        c.text(1, y, 7, clock(row.at, o), t.secondary);
        c.text(9, y, 14, name(&row.repo), t.text);
        c.line(23, y, w.saturating_sub(24), sentence, t.screen, false);
        c.line(
            cx,
            y,
            w.saturating_sub(cx + 1),
            vec![confidence(row.r#match, row.candidates, t)],
            t.screen,
            false,
        );
        match row.outcome {
            Some(LiveOutcome::Running) => {
                c.text(ox, y, w.saturating_sub(ox + 1), "codex running", t.codex)
            }
            Some(LiveOutcome::Done) => {
                if let Some(at) = row.outcome_at {
                    c.text(
                        ox,
                        y,
                        w.saturating_sub(ox + 1),
                        format!("codex done {}", clock(at, o)),
                        t.done,
                    )
                }
            }
            _ => {}
        }
    }
}
fn card(
    c: &mut Canvas<'_>,
    r: &LiveReport,
    o: &LiveOptions,
    card: &Card,
    rect: Rect,
    selected: bool,
    clip: usize,
) {
    let t = c.theme;
    let x = rect.x as usize;
    let y = rect.y as usize;
    let w = rect.width as usize;
    if w < 4 {
        return;
    }
    let color = match (card.harness, card.state) {
        (Harness::Claude, State::Stale) => t.claude_dim,
        (Harness::Codex, State::Stale) => t.codex_dim,
        (Harness::Claude, _) => t.claude,
        _ => t.codex,
    };
    let border = if card.state == State::Waiting {
        t.needs_you
    } else {
        color
    };
    let bg = if selected {
        t.selected
    } else {
        match card.state {
            State::Waiting => t.waiting_card,
            State::Stale => t.screen,
            _ => {
                if card.harness == Harness::Claude {
                    t.busy_card
                } else {
                    t.codex_box
                }
            }
        }
    };
    let (tl, tr, bl, br, h, v) = if selected {
        ("┏", "┓", "┗", "┛", "━", "┃")
    } else {
        ("╭", "╮", "╰", "╯", "─", "│")
    };
    for row in 0..10 {
        if y + row > clip {
            break;
        }
        for col in 0..w {
            c.line(x + col, y + row, 1, vec![span(" ", t.text)], bg, false);
        }
        if row == 0 || row == 9 {
            c.line(
                x,
                y + row,
                w,
                vec![span(
                    format!(
                        "{}{}{}",
                        if row == 0 { tl } else { bl },
                        h.repeat(w - 2),
                        if row == 0 { tr } else { br }
                    ),
                    border,
                )],
                bg,
                selected,
            );
        } else {
            c.line(x, y + row, 1, vec![span(v, border)], bg, selected);
            c.line(x + w - 1, y + row, 1, vec![span(v, border)], bg, selected);
        }
    }
    let glyph = match card.state {
        State::Waiting => "◆",
        State::Stale => "○",
        _ => "●",
    };
    let title = vec![
        span(
            glyph,
            if card.state == State::Stale {
                t.secondary
            } else {
                border
            },
        ),
        span(" ", color),
        bold(harness(card), color),
        bold(
            format!(
                " · {}{} ",
                name(&card.repo),
                card.step
                    .as_ref()
                    .map_or(String::new(), |s| format!(" · step {s}"))
            ),
            t.text,
        ),
    ];
    c.line(x + 2, y, w - 4, title, bg, false);
    let mut rows: Vec<Vec<Span<'static>>> = vec![vec![span(
        card.last_asked
            .as_ref()
            .map_or("no prompt".into(), |s| format!("“{s}”")),
        if card.last_asked.is_some() {
            t.text
        } else {
            t.secondary
        },
    )]];
    let mut status = Vec::new();
    match card.state {
        State::Busy => {
            status.push(span("working now", color));
            if let Some(since) = card.since {
                status.push(span(
                    if Some(since) == r.window_from {
                        format!(" · for over {}", period(r).unwrap_or_default())
                    } else {
                        format!(
                            " · for {} min",
                            view::minutes(r.now.epoch_ms() - since.epoch_ms())
                        )
                    },
                    t.secondary,
                ));
            }
        }
        State::Waiting | State::Stale => {
            let waiting = card.state == State::Waiting;
            status.push(bold(
                if waiting { "waiting on you" } else { "stale" },
                if waiting { t.needs_you } else { t.secondary },
            ));
            if let Some(at) = card.since {
                status.push(span(
                    format!(
                        " · {} {} ({} min)",
                        if waiting {
                            "since"
                        } else {
                            "no activity since"
                        },
                        clock(at, o),
                        view::minutes(r.now.epoch_ms() - at.epoch_ms())
                    ),
                    t.secondary,
                ));
            }
        }
        _ => {}
    }
    rows.push(status);
    let counts = [
        (card.counts.reads, "read"),
        (card.counts.edits, "edit"),
        (card.counts.runs, "run"),
    ]
    .into_iter()
    .filter(|(n, _)| *n > 0)
    .map(|(n, s)| format!("{n} {s}{}", if n == 1 { "" } else { "s" }))
    .collect::<Vec<_>>()
    .join(" · ");
    rows.push(vec![span(counts, t.text)]);
    rows.push(vec![span("┄".repeat(w - 4), t.border)]);
    for n in card.notable.iter().take(3) {
        let mut line = vec![span(format!("{}  ", clock(n.at, o)), t.secondary)];
        if let Some(rest) = n.label.strip_prefix('▸') {
            line.push(span("▸", color));
            line.push(span(rest, t.text));
        } else {
            line.push(span(n.label.clone(), t.text));
        }
        if n.ok == Some(false) {
            line.push(span(" ✗", t.warning));
        } else if n.ok == Some(true)
            && (n.label.starts_with("$ cargo ") || n.label == "$ pytest" || n.label == "$ npm test")
        {
            line.push(span(" ✓", t.done));
        }
        rows.push(line);
    }
    while rows.len() < 7 {
        rows.push(Vec::new());
    }
    let link = match &card.link {
        Some(CardLink::WaitingOn { child, at }) => vec![span(
            format!(
                "waiting on codex {} · {} min",
                id(child),
                view::minutes(r.now.epoch_ms() - at.epoch_ms())
            ),
            t.codex,
        )],
        Some(CardLink::Handoff {
            r#match,
            child,
            candidates,
            outcome,
            outcome_at,
            ..
        }) => {
            if *r#match == Match::Ambiguous {
                vec![span(format!("{candidates} possible runs"), t.needs_you)]
            } else if let Some(child) = child {
                vec![span(
                    format!(
                        "codex {}{}",
                        id(child),
                        if *outcome == Some(LiveOutcome::Done) {
                            outcome_at.map_or(String::new(), |at| format!(" done {}", clock(at, o)))
                        } else {
                            String::new()
                        }
                    ),
                    t.secondary,
                )]
            } else {
                vec![span("no run found", t.secondary)]
            }
        }
        Some(CardLink::From {
            parent, r#match, ..
        }) => vec![
            span(format!("└ from claude {} · ", id(parent)), t.secondary),
            span(
                if *r#match == Match::Exact {
                    "confirmed link"
                } else {
                    "matched by time"
                },
                t.text,
            ),
        ],
        Some(CardLink::None) => vec![span("not linked to any claude session", t.secondary)],
        _ => Vec::new(),
    };
    rows.push(link);
    for (i, line) in rows.into_iter().enumerate() {
        if y + i < clip {
            c.line(x + 2, y + i + 1, w - 4, line, bg, false);
        }
    }
    if y + 9 <= clip {
        c.line(
            x + 2,
            y + 9,
            w - 4,
            vec![span(
                format!(" {} · {} ", place(card), id(&card.id)),
                t.secondary,
            )],
            bg,
            false,
        );
    }
}

fn bottom(c: &mut Canvas<'_>, r: &LiveReport, o: &LiveOptions, selected: Option<&Card>) {
    let t = c.theme;
    let w = c.area.width as usize;
    let h = c.area.height as usize;
    if h < 10 {
        return;
    }
    let idle = |repos: &[String]| {
        format!(
            "{} idle sessions ({}) · home shows them",
            r.idle.count,
            repos.join(" · ")
        )
    };
    if r.idle.count > 0 {
        let mut repos: Vec<_> = r.idle.repos.iter().map(|s| name(s).to_string()).collect();
        let mut text = idle(&repos);
        while Span::raw(&text).width() > w.saturating_sub(2) && !repos.is_empty() {
            repos.pop();
            let mut shortened = repos.clone();
            shortened.push("…".into());
            text = idle(&shortened);
        }
        c.text(1, h - 10, w.saturating_sub(2), text, t.secondary);
    }
    c.rule(h - 9);
    if let Some(card) = selected {
        c.text(2, h - 8, 8, "selected", t.secondary);
        let activity = if card.activity.is_empty() {
            String::new()
        } else {
            let ms: i64 = card
                .activity
                .iter()
                .map(|a| a.to.epoch_ms() - a.from.epoch_ms())
                .sum();
            format!(
                " · active about {} min in {} stretch{}",
                view::minutes(ms),
                card.activity.len(),
                if card.activity.len() == 1 { "" } else { "es" }
            )
        };
        let color = if card.harness == Harness::Claude {
            t.claude
        } else {
            t.codex
        };
        c.line(
            12,
            h - 8,
            w.saturating_sub(13),
            vec![
                bold(harness(card), color),
                span(
                    format!(
                        " · {} · {}{} · started {}{}",
                        id(&card.id),
                        name(&card.repo),
                        if card.checkout == card.repo {
                            String::new()
                        } else {
                            format!(", {}", place(card))
                        },
                        clock(card.started_at, o),
                        activity
                    ),
                    t.text,
                ),
            ],
            t.screen,
            false,
        );
        c.text(12, h - 7, 5, "asked", t.secondary);
        c.text(
            19,
            h - 7,
            w.saturating_sub(20),
            card.last_asked
                .as_ref()
                .map_or("no prompt".into(), |s| format!("“{s}”")),
            if card.last_asked.is_some() {
                t.text
            } else {
                t.secondary
            },
        );
    }
    let legend = vec![
        vec![span("●", t.claude), span(" working", t.secondary)],
        vec![span("◆", t.needs_you), span(" waiting on you", t.secondary)],
        vec![span("○ stale", t.secondary)],
        vec![span("│", t.text), span(" confirmed link", t.secondary)],
        vec![span("╎", t.text), span(" matched by time", t.secondary)],
        vec![span("✓", t.done), span(" exit 0", t.secondary)],
        vec![span("✗", t.warning), span(" exit non-zero", t.secondary)],
    ];
    items(c, h - 5, legend);
    c.rule(h - 3);
    let keys = [
        ("↑↓←→", "move"),
        ("enter", "repo view"),
        ("s", "step graph"),
        ("o", "open"),
        ("f", "fork"),
        ("esc", "back"),
        ("q", "quit"),
    ]
    .into_iter()
    .map(|(k, word)| vec![span(k, t.text), span(format!(" {word}"), t.secondary)])
    .collect();
    let (all, end) = items(c, h - 2, keys);
    let label = format!("times {}", view::offset_label(o.offset));
    let lw = Span::raw(&label).width();
    if all && w.saturating_sub(lw + 1) >= end + 3 {
        c.text(w - lw - 1, h - 2, lw, label, t.secondary);
    }
}
fn items(c: &mut Canvas<'_>, y: usize, items: Vec<Vec<Span<'static>>>) -> (bool, usize) {
    let mut x = 1;
    let right = c.area.width.saturating_sub(1) as usize;
    for item in items {
        let width = item.iter().map(Span::width).sum::<usize>();
        if x + width > right {
            return (false, x.saturating_sub(3));
        }
        c.line(x, y, width, item, c.theme.screen, false);
        x += width + 3;
    }
    (true, x.saturating_sub(3))
}

/// Draw a LiveReport without reading files, environment, clock or application state.
/// `first_row` is a wide row index (pairs share a row), or a flattened card index
/// in the narrow form. Selection and scrolling are supplied by the caller.
pub fn render_live(report: &LiveReport, options: &LiveOptions, area: Rect, buf: &mut Buffer) {
    let area = area.intersection(buf.area);
    if area.is_empty() {
        return;
    }
    let t = options.theme;
    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            buf[(x, y)].reset();
            buf[(x, y)].set_style(Style::default().fg(t.text).bg(t.screen));
        }
    }
    let mut c = Canvas {
        buf,
        area,
        theme: t,
    };
    header(&mut c, report, options);
    let h = area.height as usize;
    let w = area.width as usize;
    let k = if h >= 40 {
        4
    } else if h >= 32 {
        2
    } else {
        0
    };
    feed(&mut c, report, options, k);
    let start = if k > 0 { 4 + k } else { 3 };
    let end = h.saturating_sub(12);
    let cards: Vec<_> = report
        .groups
        .iter()
        .flat_map(|g| std::iter::once(&g.head).chain(&g.children))
        .collect();
    let selected = cards.get(options.selected).copied();
    // Each row contains flattened card indices and their column/child position.
    let mut rows: Vec<Vec<(usize, bool, bool)>> = Vec::new();
    let mut index = 0;
    for group in &report.groups {
        if w >= 120 {
            let mut row = vec![(index, false, false)];
            index += 1;
            if !group.children.is_empty() {
                row.push((index, true, false));
                index += 1;
            }
            rows.push(row);
            for _ in group.children.iter().skip(1) {
                rows.push(vec![(index, true, true)]);
                index += 1;
            }
        } else {
            rows.push(vec![(index, false, false)]);
            index += 1;
            for _ in &group.children {
                rows.push(vec![(index, true, false)]);
                index += 1;
            }
        }
    }
    if rows.is_empty() && report.idle.count == 0 {
        let last = report.last_handoff.map_or(String::new(), |at| {
            format!(" · last hand-off {}", clock(at, options))
        });
        c.text(
            1,
            start,
            w.saturating_sub(2),
            format!("nothing working right now{last}"),
            t.secondary,
        );
    }
    let fits = end.saturating_add(1).saturating_sub(start) / 11
        + usize::from(end.saturating_add(1).saturating_sub(start) % 11 >= 10);
    let first = options.first_row.min(rows.len().saturating_sub(1));
    let mut after = cards.len();
    let mut drawn = 0;
    if fits == 0 {
        if let Some(card) = selected {
            let (_, child, extra) = rows
                .iter()
                .flatten()
                .find(|(idx, _, _)| *idx == options.selected)
                .copied()
                .unwrap();
            let (x, width) = card_position(w, child);
            self::card(
                &mut c,
                report,
                options,
                card,
                Rect::new(x as u16, start as u16, width as u16, 10),
                true,
                end,
            );
            if child && start + 4 <= end {
                connector(&mut c, w, extra, start + 4);
            }
            after = options.selected + 1;
        }
    } else {
        for (i, row) in rows.iter().skip(first).take(fits).enumerate() {
            let y = start + i * 11;
            for &(idx, child, extra) in row {
                let (x, width) = card_position(w, child);
                card(
                    &mut c,
                    report,
                    options,
                    cards[idx],
                    Rect::new(x as u16, y as u16, width as u16, 10),
                    idx == options.selected,
                    end,
                );
                if child {
                    connector(&mut c, w, extra, y + 4);
                }
                after = idx + 1;
            }
            drawn += 1;
        }
    }
    let more = cards.len().saturating_sub(after);
    if more > 0 {
        let y = if fits == 0 {
            h.saturating_sub(11)
        } else {
            start + drawn * 11 - 1
        };
        c.text(
            1,
            y,
            w.saturating_sub(2),
            format!("↓ {more} more"),
            t.secondary,
        );
    }
    bottom(&mut c, report, options, selected);
    if t.mode == crate::theme::ColorMode::NoColor {
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                let cell = &mut c.buf[(x, y)];
                cell.fg = Color::Reset;
                cell.bg = Color::Reset;
                cell.modifier.remove(Modifier::DIM);
            }
        }
    }
}
fn card_position(width: usize, child: bool) -> (usize, usize) {
    if width >= 120 {
        if child { (61, 58) } else { (1, 56) }
    } else if child {
        (5, width.saturating_sub(6))
    } else {
        (1, width.saturating_sub(2))
    }
}
fn connector(c: &mut Canvas<'_>, width: usize, extra: bool, y: usize) {
    let (x, text) = if width < 120 {
        (2, "└▸")
    } else if extra {
        (57, "└──▸")
    } else {
        (57, "──▸")
    };
    c.text(x, y, 4, text, c.theme.codex);
}
