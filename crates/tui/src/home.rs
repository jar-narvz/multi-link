//! Board 7c home: words, attention cards, idle sentences and a quiet line.
use crate::{theme::Theme, view};
use multi_link_model::{
    Convention, Harness, LinksReport, MergedRow, OverviewReport, RepoGroup, Row, SessionRow, State,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use std::collections::HashSet;
use time::UtcOffset;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target<'a> {
    Session(&'a SessionRow),
    Merged(&'a MergedRow),
    Repo(&'a RepoGroup),
    Hidden(&'a RepoGroup),
}
pub fn selectable(report: &OverviewReport) -> Vec<(&str, Target<'_>)> {
    let mut out = Vec::new();
    for g in &report.repos {
        for r in &g.rows {
            match r {
                Row::Session(r) => {
                    out.push((g.repo.as_str(), Target::Session(r)));
                    out.extend(
                        r.children
                            .iter()
                            .map(|c| (g.repo.as_str(), Target::Session(c))),
                    );
                }
                Row::Merged(r) => out.push((g.repo.as_str(), Target::Merged(r))),
            }
        }
    }
    out
}
/// Deterministic home presentation state. The CLI supplies its theme and per-refresh offset.
pub struct HomeOptions<'a> {
    pub links: Option<&'a LinksReport>,
    pub hide_footer: bool,
    pub theme: Theme,
    pub offset: UtcOffset,
    /// Report window; omitted options retain the default two hours.
    pub window: Option<crate::TimeWindow>,
}
impl Default for HomeOptions<'_> {
    fn default() -> Self {
        Self {
            links: None,
            hide_footer: false,
            theme: Theme::from_env(Some("truecolor"), None),
            offset: UtcOffset::UTC,
            window: None,
        }
    }
}
/// Targets follow the rendered blocks: cards, idle sentences, then quiet repos.
pub fn home_targets<'a>(
    report: &'a OverviewReport,
    _: &HomeOptions<'_>,
) -> Vec<(&'a str, Target<'a>)> {
    let mut out = Vec::new();
    for group in ordered_groups(report) {
        let rows = view::attention(group);
        if rows.is_empty() {
            out.push((group.repo.as_str(), Target::Repo(group)));
        } else {
            out.extend(
                rows.into_iter()
                    .map(|r| (group.repo.as_str(), Target::Session(r))),
            );
            if view::sessions(group).any(|r| !r.imported && view::rank(r.state) >= 3) {
                out.push((group.repo.as_str(), Target::Hidden(group)));
            }
        }
    }
    out
}
pub fn selectable_home<'a>(
    report: &'a OverviewReport,
    _expanded: &HashSet<String>,
) -> Vec<(&'a str, Target<'a>)> {
    home_targets(report, &HomeOptions::default())
}
fn category(g: &RepoGroup) -> u8 {
    if !view::attention(g).is_empty() {
        0
    } else if view::sessions(g).any(|r| !r.imported && !r.activity.is_empty()) {
        1
    } else {
        2
    }
}
fn ordered_groups(report: &OverviewReport) -> Vec<&RepoGroup> {
    (0..3)
        .flat_map(|kind| report.repos.iter().filter(move |g| category(g) == kind))
        .collect()
}
/// Word-boundary truncation, measuring terminal cells rather than bytes.
pub fn fit(text: &str, width: usize) -> String {
    if Span::raw(text).width() <= width {
        return text.into();
    }
    if width == 0 {
        return String::new();
    }
    let mut end = 0;
    for (index, ch) in text.char_indices() {
        if ch.is_whitespace() && Span::raw(text[..index].trim_end()).width() < width {
            end = index;
        }
    }
    let mut prefix = text[..end].trim_end();
    while let Some(trimmed) = prefix
        .strip_suffix('·')
        .or_else(|| prefix.strip_suffix('→'))
    {
        prefix = trimmed.trim_end();
    }
    format!("{prefix}…")
}
pub(crate) fn plan(g: &RepoGroup) -> String {
    match g.plan.convention {
        Convention::Missing => "no PLAN.md".into(),
        Convention::Unsupported => "PLAN.md not in the step format".into(),
        Convention::Supported => g.plan.latest_step.as_ref().map_or_else(
            || "no step logged yet".into(),
            |s| {
                format!(
                    "step {}{}",
                    s.id,
                    s.title.as_ref().map_or(String::new(), |t| format!(" {t}"))
                )
            },
        ),
    }
}
fn glyph(s: State) -> &'static str {
    match s {
        State::Waiting => "◆",
        State::Busy => "●",
        State::Stale => "◌",
        State::Idle => "○",
        State::Unknown => "○",
        State::Finished => "✓",
    }
}
fn state_color(t: Theme, s: State) -> Color {
    match s {
        State::Waiting => t.needs_you,
        State::Busy => t.claude,
        State::Stale => t.warning,
        _ => t.secondary,
    }
}
fn segment(s: impl Into<String>, fg: Color, bg: Color, bold: bool) -> Span<'static> {
    Span::styled(
        s.into(),
        Style::default().fg(fg).bg(bg).add_modifier(if bold {
            Modifier::BOLD
        } else {
            Modifier::empty()
        }),
    )
}
fn write(buf: &mut Buffer, x: u16, y: u16, text: &str, width: u16, fg: Color, bg: Color) {
    buf.set_stringn(
        x,
        y,
        fit(text, usize::from(width)),
        usize::from(width),
        Style::default().fg(fg).bg(bg),
    );
}

fn label_fit(text: &str, width: usize) -> String {
    if let Some(prompt) = text.strip_prefix('“').and_then(|s| s.strip_suffix('”')) {
        if width < 3 {
            return fit(text, width);
        }
        format!("“{}”", fit(prompt, width - 2))
    } else {
        fit(text, width)
    }
}
fn draw_spans(buf: &mut Buffer, x: u16, y: u16, spans: Vec<Span<'static>>, width: u16) {
    buf.set_line(x, y, &Line::from(spans), width);
}

pub fn render_home(
    report: &OverviewReport,
    selection: Option<usize>,
    area: Rect,
    buf: &mut Buffer,
) {
    render_home_with_options(report, selection, &HomeOptions::default(), area, buf);
}
pub fn render_home_with_options(
    report: &OverviewReport,
    selection: Option<usize>,
    o: &HomeOptions<'_>,
    area: Rect,
    buf: &mut Buffer,
) {
    let area = area.intersection(buf.area);
    if area.is_empty() {
        return;
    }
    let (w, h, t) = (area.width, area.height, o.theme);
    let capacity = report
        .repos
        .iter()
        .map(|g| view::sessions(g).count() + 10)
        .sum::<usize>()
        + 1;
    let mut body = Buffer::empty(Rect::new(0, 0, w, capacity.min(65500) as u16));
    body.set_style(body.area, Style::default().bg(t.screen));
    let mut y = 0;
    let mut cursor = SelectionCursor {
        requested: selection,
        ..SelectionCursor::default()
    };
    if w >= 10 {
        let groups = ordered_groups(report);
        for g in groups.iter().copied().filter(|g| category(g) < 2) {
            y = draw_group(&mut body, y, g, report, o, &mut cursor);
        }

        let quiet: Vec<_> = groups.into_iter().filter(|g| category(g) == 2).collect();
        if !quiet.is_empty() {
            write(&mut body, 3, y, "quiet", 5, t.secondary, t.screen);
            let mut x = 9;
            for (i, g) in quiet.iter().enumerate() {
                let words = if view::sessions(g).all(|r| r.imported) {
                    format!("{} (only Codex Desktop imports)", basename(&g.repo))
                } else {
                    format!(
                        "{} (last active {})",
                        basename(&g.repo),
                        view::clock(g.last_activity, report.now, o.offset)
                    )
                };
                let separator = if i > 0 { " · " } else { "" };
                let words = if x > 9
                    && usize::from(x) + Span::raw(separator).width() + Span::raw(&words).width()
                        > usize::from(w.saturating_sub(2))
                {
                    y += 1;
                    x = 9;
                    words
                } else {
                    format!("{separator}{words}")
                };
                let first = x.min(w - 1);
                let length =
                    Span::raw(fit(&words, usize::from(w.saturating_sub(x + 2)))).width() as u16;
                if cursor.requested == Some(cursor.index) {
                    cursor.columns = Some((first, first + length));
                }
                cursor.mark(y);
                write(
                    &mut body,
                    first,
                    y,
                    &words,
                    w.saturating_sub(x + 2),
                    t.secondary,
                    t.screen,
                );
                x = x.saturating_add(Span::raw(words).width() as u16);
            }
        }
    }
    let mut screen = Buffer::empty(Rect::new(0, 0, w, h));
    screen.set_style(screen.area, Style::default().bg(t.screen));
    draw_header(&mut screen, report, o);
    if h > 1 {
        rule(&mut screen, 1, t);
    }
    let footer_rows = if o.hide_footer { 0 } else { 2 };
    let body_height = h.saturating_sub(3 + footer_rows);
    let offset = cursor.line.map_or(0, |sy: u16| {
        sy.saturating_sub(body_height.saturating_sub(1))
    });
    for dy in 0..body_height {
        if offset + dy >= body.area.height {
            break;
        }
        for x in 0..w {
            screen[(x, dy + 3)] = body[(x, offset + dy)].clone();
            if cursor.line == Some(offset + dy)
                && cursor
                    .columns
                    .is_none_or(|(first, end)| x >= first && x < end)
            {
                screen[(x, dy + 3)].set_bg(t.selected);
            }
        }
    }
    if !o.hide_footer && h >= 2 {
        draw_footer(&mut screen, report, o);
    }
    for dy in 0..h {
        for dx in 0..w {
            buf[(area.x + dx, area.y + dy)] = screen[(dx, dy)].clone();
        }
    }
}
#[derive(Default)]
struct SelectionCursor {
    requested: Option<usize>,
    index: usize,
    line: Option<u16>,
    columns: Option<(u16, u16)>,
}
impl SelectionCursor {
    fn mark(&mut self, y: u16) {
        if self.requested == Some(self.index) {
            self.line = Some(y);
        }
        self.index += 1;
    }
}
fn draw_group(
    body: &mut Buffer,
    mut y: u16,
    g: &RepoGroup,
    report: &OverviewReport,
    o: &HomeOptions<'_>,
    cursor: &mut SelectionCursor,
) -> u16 {
    let (w, t) = (body.area.width, o.theme);
    let rows = view::attention(g);
    let card = !rows.is_empty();
    let state = rows.first().map_or(State::Idle, |r| r.state);
    let bg = match state {
        State::Waiting => t.waiting_card,
        State::Busy => t.busy_card,
        _ => t.screen,
    };
    let border = if card {
        state_color(t, state)
    } else {
        t.border
    };
    let inside = if card {
        3 + rows.len()
            + usize::from(g.latest_handoff.is_some())
            + usize::from(!g.concurrency.is_empty())
    } else {
        1
    };
    box_rows(body, y, inside as u16, border, bg);
    y += 1;
    let name = basename(&g.repo);
    body.set_string(3, y, glyph(state), t.text_style(state_color(t, state), bg));
    let name = fit(name, usize::from(w.saturating_sub(12)));
    body.set_string(
        5,
        y,
        &name,
        t.text_style(t.text, bg).add_modifier(Modifier::BOLD),
    );
    let px = 7 + Span::raw(&name).width() as u16;
    if card {
        draw_counts_and_plan(body, y, g, &rows, o, bg, px);
        y += 2;
        let sx = status_column(w, &rows, report, o);
        for row in &rows {
            cursor.mark(y);
            draw_session(body, y, (row, &view::home_label(g, row)), report, o, bg, sx);
            y += 1;
        }
        if !g.concurrency.is_empty() {
            let names = g
                .concurrency
                .iter()
                .map(|warning| basename(&warning.cwd))
                .collect::<Vec<_>>()
                .join(", ");
            write(
                body,
                5,
                y,
                &format!("⚠ shared checkout: {names}"),
                w - 9,
                t.warning,
                bg,
            );
            y += 1;
        }
        let rest: Vec<_> = view::sessions(g)
            .filter(|r| !r.imported && view::rank(r.state) >= 3)
            .collect();
        if !rest.is_empty() {
            cursor.mark(y);
        }
        let mut summary = activity_summary(&rows, &rest, period(report, o));
        if g.latest_handoff.is_none() {
            summary.push_str(" · no hand-offs");
        }
        write(body, 5, y, &summary, w - 9, t.secondary, bg);
        y += 1;
        if let Some(link) = &g.latest_handoff {
            let words = format!(
                "last hand-off  {} · claude {} · {}",
                view::clock(link.at, report.now, o.offset),
                view::handed(link),
                view::outcome(link, report)
            );
            write(body, 5, y, &words, w - 9, t.secondary, bg);
            y += 1;
        }
    } else {
        cursor.mark(y);
        let last = view::sessions(g)
            .filter(|r| !r.imported)
            .flat_map(|r| &r.activity)
            .map(|b| b.to)
            .max()
            .unwrap();
        let handoff = g.latest_handoff.as_ref().map_or_else(
            || "no hand-offs".into(),
            |l| {
                format!(
                    "claude {} at {}, {}",
                    view::handed(l),
                    view::clock(l.at, report.now, o.offset),
                    view::outcome(l, report)
                )
            },
        );
        write(
            body,
            px,
            y,
            &format!(
                "idle · last active {} · {handoff}",
                view::last_active(last, report.now, o.offset)
            ),
            w.saturating_sub(px + 4),
            t.secondary,
            bg,
        );
        y += 1;
    }
    y += 2;
    y
}
fn draw_counts_and_plan(
    body: &mut Buffer,
    y: u16,
    g: &RepoGroup,
    rows: &[&SessionRow],
    o: &HomeOptions<'_>,
    bg: Color,
    px: u16,
) {
    let (w, t) = (body.area.width, o.theme);
    let mut counts = Vec::new();
    for (state, words) in [
        (State::Waiting, "waiting"),
        (State::Busy, "working now"),
        (State::Stale, "stale"),
    ] {
        let n = rows.iter().filter(|r| r.state == state).count();
        if n > 0 {
            if !counts.is_empty() {
                counts.push(segment(" · ", t.secondary, bg, false));
            }
            counts.push(segment(
                format!("{n} {words}"),
                state_color(t, state),
                bg,
                false,
            ));
        }
    }
    let rw = spans_width(&counts) as u16;
    let rx = w.saturating_sub(4 + rw);
    let wording = plan(g);
    if rx > px + 3 {
        let fg = if g.plan.latest_step.is_some() {
            t.text
        } else {
            t.secondary
        };
        write(body, px, y, &wording, rx - px - 2, fg, bg);
    }
    if rx > px {
        draw_spans(body, rx, y, counts, rw);
    }
}
fn period(report: &OverviewReport, o: &HomeOptions<'_>) -> String {
    crate::timeline::period(
        o.window
            .unwrap_or_else(|| crate::TimeWindow::default_for(report)),
    )
}

fn activity_summary(rows: &[&SessionRow], rest: &[&SessionRow], period: String) -> String {
    if rest.is_empty() {
        let bursts: Vec<_> = rows.iter().flat_map(|r| &r.activity).collect();
        let ms = bursts
            .iter()
            .map(|b| b.to.epoch_ms() - b.from.epoch_ms())
            .sum();
        format!(
            "active about {} min in the last {period}, in {} stretch{}",
            view::minutes(ms),
            bursts.len(),
            if bursts.len() == 1 { "" } else { "es" }
        )
    } else {
        let active = rest.iter().filter(|r| !r.activity.is_empty()).count();
        let n = rest.len();
        let plural = if n == 1 { "" } else { "s" };
        if active == 0 {
            format!("{n} older session{plural}, none active in the last {period} · enter shows all")
        } else {
            format!(
                "{n} other session{plural}, {active} active in the last {period} · enter shows all"
            )
        }
    }
}
fn box_rows(buf: &mut Buffer, y: u16, inside: u16, border: Color, bg: Color) {
    let w = buf.area.width;
    for (dy, left, right) in [(0, "╭", "╮"), (inside + 1, "╰", "╯")] {
        buf.set_string(
            1,
            y + dy,
            format!("{left}{}{right}", "─".repeat(usize::from(w - 4))),
            Style::default().fg(border),
        );
    }
    for dy in 1..=inside {
        buf.set_style(Rect::new(2, y + dy, w - 4, 1), Style::default().bg(bg));
        for x in [1, w - 2] {
            buf[(x, y + dy)].set_symbol("│").set_fg(border);
        }
    }
}
fn session_branch(row: &SessionRow, w: u16, phrase_width: u16) -> String {
    let branch = if w >= 80 {
        row.branch.as_deref().unwrap_or("")
    } else {
        ""
    };
    // Three label cells retain a quoted ellipsis; only then may the branch shrink.
    let branch_budget = w.saturating_sub(4 + 20 + phrase_width + 2);
    fit(branch, usize::from((w / 6).min(branch_budget)))
}
fn status_column(
    w: u16,
    rows: &[&SessionRow],
    report: &OverviewReport,
    o: &HomeOptions<'_>,
) -> u16 {
    let preferred = if w >= 120 {
        66
    } else if w >= 80 {
        w / 2 + 6
    } else {
        w / 2
    };
    // Reserve every row's phrase and branch before assigning one label budget to the card.
    rows.iter().fold(preferred, |sx, row| {
        let phrase_width = Span::raw(view::status(row, report.now, o.offset)).width() as u16;
        let branch = session_branch(row, w, phrase_width);
        let end = w.saturating_sub(
            4 + Span::raw(&branch).width() as u16 + u16::from(!branch.is_empty()) * 2,
        );
        sx.min(end.saturating_sub(phrase_width).max(20.min(end)))
    })
}
fn draw_session(
    buf: &mut Buffer,
    y: u16,
    (row, label): (&SessionRow, &str),
    report: &OverviewReport,
    o: &HomeOptions<'_>,
    bg: Color,
    sx: u16,
) {
    let (w, t) = (buf.area.width, o.theme);
    let fg = state_color(t, row.state);
    write(buf, 3, y, glyph(row.state), 1, fg, bg);
    let harness = if row.harness == Harness::Claude {
        "claude"
    } else {
        "codex"
    };
    write(
        buf,
        5,
        y,
        harness,
        6,
        if row.harness == Harness::Claude {
            t.claude
        } else {
            t.codex
        },
        bg,
    );
    let phrase = view::status(row, report.now, o.offset);
    let branch = session_branch(row, w, Span::raw(&phrase).width() as u16);
    let bw = Span::raw(&branch).width() as u16;
    let bx = w.saturating_sub(4 + bw);
    let label = label_fit(label, usize::from(sx.saturating_sub(17)));
    write(buf, 15, y, &label, sx.saturating_sub(17), t.text, bg);
    let available = bx.saturating_sub(sx + u16::from(!branch.is_empty()) * 2);
    let phrase = fit(&phrase, usize::from(available));
    buf.set_stringn(
        sx,
        y,
        phrase,
        usize::from(available),
        t.text_style(fg, bg).add_modifier(Modifier::BOLD),
    );
    if !branch.is_empty() {
        write(buf, bx, y, &branch, bw, t.secondary, bg);
    }
}
fn rule(buf: &mut Buffer, y: u16, t: Theme) {
    let w = buf.area.width;
    if w > 2 {
        buf.set_string(
            1,
            y,
            "─".repeat(usize::from(w - 2)),
            t.text_style(t.border, t.screen),
        );
    }
}
pub(crate) fn basename(path: &str) -> &str {
    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(path)
}
/// A header part is indivisible; related counts share a separator when both survive.
struct HeaderPart {
    span: Span<'static>,
    group: u8,
    drop_priority: u8,
}
fn header_spans(parts: &[HeaderPart], theme: Theme) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            spans.push(segment(
                if parts[index - 1].group == part.group {
                    " · "
                } else {
                    "   "
                },
                theme.secondary,
                theme.screen,
                false,
            ));
        }
        spans.push(part.span.clone());
    }
    spans
}
fn spans_width(spans: &[Span<'_>]) -> usize {
    spans.iter().map(Span::width).sum()
}
fn draw_header(screen: &mut Buffer, report: &OverviewReport, o: &HomeOptions<'_>) {
    let w = screen.area.width;
    let t = o.theme;
    let all: Vec<_> = report
        .repos
        .iter()
        .flat_map(view::sessions)
        .filter(|r| !r.imported)
        .collect();
    let n = |s| all.iter().filter(|r| r.state == s).count();
    let mut parts = Vec::new();
    let mut add = |text: String, fg, bold, group, drop_priority| {
        parts.push(HeaderPart {
            span: segment(text, fg, t.screen, bold),
            group,
            drop_priority,
        })
    };
    add(
        format!("All repos · last {}", period(report, o)),
        t.text,
        true,
        0,
        6,
    );
    add(
        format!("● {} working now", n(State::Busy)),
        t.claude,
        false,
        1,
        5,
    );
    if n(State::Stale) > 0 {
        add(
            format!("◌ {} stale", n(State::Stale)),
            t.warning,
            false,
            2,
            7,
        );
    }
    let warnings: usize = report.repos.iter().map(|g| g.concurrency.len()).sum();
    if warnings > 0 {
        add(
            format!("⚠ {warnings} shared checkouts"),
            t.warning,
            false,
            3,
            4,
        );
    }
    add(
        format!("○ {} idle", n(State::Idle)),
        t.secondary,
        false,
        4,
        3,
    );
    add(
        format!("{} done or unknown", n(State::Finished) + n(State::Unknown)),
        t.secondary,
        false,
        4,
        2,
    );
    add(
        format!("{} repos", report.repos.len()),
        t.secondary,
        false,
        5,
        1,
    );
    if w >= 80 {
        add(
            format!("{} hand-offs", o.links.map_or(0, |l| l.links.len())),
            t.secondary,
            false,
            5,
            0,
        );
    }
    let needs = if n(State::Waiting) > 0 {
        format!("◆ {} need you", n(State::Waiting))
    } else if report.hook.records == 0 {
        "◆ needs-you partial: no hook records seen".into()
    } else {
        String::new()
    };
    // Compact wording keeps the evidence note readable even when the full sentence cannot fit.
    let needs = if Span::raw(&needs).width() > usize::from(w.saturating_sub(2)) {
        [
            "◆ partial: no hook records",
            "◆ needs-you partial",
            "◆ partial",
            "◆",
        ]
        .into_iter()
        .find(|text| Span::raw(*text).width() <= usize::from(w))
        .unwrap_or("")
        .to_string()
    } else {
        needs
    };
    let nw = Span::raw(&needs).width() as u16;
    let hx = w.saturating_sub(nw + 1);
    let available = if needs.is_empty() {
        w.saturating_sub(2)
    } else {
        hx.saturating_sub(3)
    };
    let mut spans = header_spans(&parts, t);
    while spans_width(&spans) > usize::from(available) && !parts.is_empty() {
        let index = parts
            .iter()
            .enumerate()
            .min_by_key(|(_, p)| p.drop_priority)
            .unwrap()
            .0;
        parts.remove(index);
        spans = header_spans(&parts, t);
    }
    screen.set_line(1.min(w - 1), 0, &Line::from(spans), available);
    if nw > 0 {
        screen.set_stringn(
            hx,
            0,
            needs,
            usize::from(nw),
            t.text_style(t.needs_you, t.screen),
        );
    }
}

fn draw_footer(buf: &mut Buffer, report: &OverviewReport, o: &HomeOptions<'_>) {
    let (w, h, t) = (buf.area.width, buf.area.height, o.theme);
    rule(buf, h - 2, t);
    let imports = report
        .repos
        .iter()
        .flat_map(view::sessions)
        .filter(|r| r.imported)
        .count();
    let tz = format!("times {}", view::offset_label(o.offset));
    let mut note = if imports > 0 {
        format!(
            "{imports} Codex Desktop import{} hidden · {tz}",
            if imports == 1 { "" } else { "s" }
        )
    } else {
        tz.clone()
    };
    let mut keys = vec![
        ("↑↓", "move"),
        ("enter", "repo view"),
        ("o", "open session"),
        ("q", "quit"),
    ];
    if Span::raw(&note).width() + spans_width(&footer_spans(&keys, t)) + 5 > usize::from(w) {
        note = tz;
    }
    let minimum_keys = footer_spans(&[("↑↓", "move"), ("q", "quit")], t);
    if Span::raw(&note).width() + spans_width(&minimum_keys) + 5 > usize::from(w) {
        note.clear();
    }
    let nw = Span::raw(&note).width() as u16;
    let available = w.saturating_sub(if note.is_empty() { 2 } else { nw + 5 });
    let mut spans = footer_spans(&keys, t);
    while spans_width(&spans) > usize::from(available) && keys.len() > 2 {
        keys.remove(keys.len() - 2);
        spans = footer_spans(&keys, t);
    }
    if spans_width(&spans) <= usize::from(available) {
        buf.set_line(1, h - 1, &Line::from(spans), available);
    }
    if nw + 2 <= w {
        buf.set_string(w - nw - 1, h - 1, note, t.text_style(t.secondary, t.screen));
    }
}
fn footer_spans(keys: &[(&str, &str)], t: Theme) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for (i, (key, words)) in keys.iter().enumerate() {
        if i > 0 {
            spans.push(segment("   ", t.secondary, t.screen, false));
        }
        spans.push(segment(*key, t.text, t.screen, false));
        spans.push(segment(format!(" {words}"), t.secondary, t.screen, false));
    }
    spans
}
