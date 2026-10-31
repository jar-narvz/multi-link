//! Board 7 home renderer: explicit palette, local clock and shared lane model.
use crate::{
    theme::{ColorMode, Theme},
    view::{self, Lane},
};
use multi_link_model::{
    Convention, Harness, LinksReport, MergedRow, OverviewReport, RepoGroup, Row, SessionRow, State,
    Tier, VerifyResult,
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
    pub expanded_repos: HashSet<String>,
    pub split: HashSet<String>,
    pub show_all: HashSet<String>,
    pub links: Option<&'a LinksReport>,
    pub hide_footer: bool,
    pub theme: Theme,
    pub offset: UtcOffset,
}
impl Default for HomeOptions<'_> {
    fn default() -> Self {
        Self {
            expanded_repos: HashSet::new(),
            split: HashSet::new(),
            show_all: HashSet::new(),
            links: None,
            hide_footer: false,
            theme: Theme::from_env(Some("truecolor"), None),
            offset: UtcOffset::UTC,
        }
    }
}
/// Selectable targets in precisely the order used by the renderer.
pub fn home_targets<'a>(
    report: &'a OverviewReport,
    options: &HomeOptions<'_>,
) -> Vec<(&'a str, Target<'a>)> {
    let mut out = Vec::new();
    for g in &report.repos {
        if view::collapsed(g, &options.expanded_repos) {
            out.push((g.repo.as_str(), Target::Repo(g)));
        } else {
            let lanes = view::lanes(
                report,
                g,
                &options.split,
                options.show_all.contains(&g.repo),
            );
            out.extend(
                lanes
                    .visible
                    .iter()
                    .map(|l| (g.repo.as_str(), Target::Session(l.row))),
            );
            if lanes.hidden > 0 {
                out.push((g.repo.as_str(), Target::Hidden(g)));
            }
        }
    }
    out
}
pub fn selectable_home<'a>(
    report: &'a OverviewReport,
    expanded: &HashSet<String>,
) -> Vec<(&'a str, Target<'a>)> {
    home_targets(
        report,
        &HomeOptions {
            expanded_repos: expanded.clone(),
            ..HomeOptions::default()
        },
    )
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
fn plan(g: &RepoGroup) -> String {
    match g.plan.convention {
        Convention::Missing => "no PLAN.md".into(),
        Convention::Unsupported => "PLAN.md not in the step format".into(),
        Convention::Supported => g.plan.latest_step.as_ref().map_or_else(
            || "no step logged yet".into(),
            |s| {
                format!(
                    "latest step {}{}",
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
        State::Unknown => "?",
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
fn count(g: &RepoGroup, state: State) -> usize {
    view::sessions(g).filter(|r| r.state == state).count()
}
fn label(
    l: Lane<'_>,
    report: &OverviewReport,
    o: &HomeOptions<'_>,
    bg: Color,
) -> Vec<Span<'static>> {
    let t = o.theme;
    let state = l.state();
    let mut spans = vec![segment(
        format!("{} ", glyph(state)),
        state_color(t, state),
        bg,
        false,
    )];
    if let Some(step) = l.step() {
        spans.push(segment(format!("step {step}"), t.text, bg, false));
        spans.push(segment(" · ", t.secondary, bg, false));
    }
    let harness = if l.row.harness == Harness::Claude {
        "claude"
    } else {
        "codex"
    };
    spans.push(segment(
        harness,
        if harness == "claude" && view::rank(state) < 2 {
            t.claude
        } else {
            t.secondary
        },
        bg,
        false,
    ));
    if l.child.is_some() {
        spans.push(segment(
            format!(" → codex · {}", view::link_words(l.link.unwrap())),
            t.secondary,
            bg,
            false,
        ));
    } else if l.step().is_none() {
        if l.row.harness == Harness::Claude && view::rank(state) < 3 {
            if let Some(branch) = &l.row.branch {
                spans.push(segment(format!(" {branch}"), t.text, bg, false));
            }
            let words = match state {
                State::Busy => " · busy now".into(),
                State::Waiting => format!(
                    " · waiting since {}",
                    view::clock(l.row.last_activity, report.now, o.offset)
                ),
                _ => format!(
                    " · stale since {}",
                    view::clock(l.row.last_activity, report.now, o.offset)
                ),
            };
            spans.push(segment(
                words,
                if state == State::Waiting {
                    t.needs_you
                } else if state == State::Stale {
                    t.warning
                } else {
                    t.secondary
                },
                bg,
                false,
            ));
        } else {
            spans.push(segment(
                if harness == "claude" {
                    " · no hand-off"
                } else {
                    " · no parent found"
                },
                t.secondary,
                bg,
                false,
            ));
        }
    } else if let Some(link) = l.link {
        spans.push(segment(
            format!(" · {}", view::link_words(link)),
            t.secondary,
            bg,
            false,
        ));
    }
    spans
}
fn draw_spans(buf: &mut Buffer, x: u16, y: u16, spans: Vec<Span<'static>>, width: u16) {
    let full: String = spans.iter().map(|s| s.content.as_ref()).collect();
    let fitted = fit(&full, usize::from(width));
    if fitted == full {
        buf.set_line(x, y, &Line::from(spans), width);
        return;
    }
    let mut remaining = fitted.trim_end_matches('…').chars().count();
    let mut clipped = Vec::new();
    for mut span in spans {
        let count = span.content.chars().count();
        if remaining < count {
            span.content = format!(
                "{}…",
                span.content.chars().take(remaining).collect::<String>()
            )
            .into();
            clipped.push(span);
            break;
        }
        remaining -= count;
        clipped.push(span);
    }
    buf.set_line(x, y, &Line::from(clipped), width);
}
fn draw_bar(
    buf: &mut Buffer,
    y: u16,
    col: u16,
    cells: u16,
    l: Lane<'_>,
    report: &OverviewReport,
    t: Theme,
) {
    let window = crate::TimeWindow::default_for(report);
    let ms = |at| view::datetime(at).unix_timestamp_nanos() / 1_000_000;
    let (s, e) = (ms(window.start), ms(window.end));
    for x in col..col + cells {
        buf[(x, y)]
            .set_symbol(if t.mode == ColorMode::NoColor {
                "·"
            } else {
                " "
            })
            .set_bg(t.track)
            .set_fg(Color::Reset);
    }
    let bright = view::rank(l.state()) < 2;
    for r in std::iter::once(l.row).chain(l.child) {
        let a = ms(r.started_at);
        let b = ms(r.last_activity);
        let pos =
            |v: i128| (((v.clamp(s, e) - s) * i128::from(cells) / (e - s)) as u16).min(cells - 1);
        let (first, last) = (pos(a), pos(b.max(a)));
        let (symbol, color) = match r.harness {
            Harness::Claude => ("━", if bright { t.claude } else { t.claude_dim }),
            Harness::Codex => ("█", if bright { t.codex } else { t.codex_dim }),
        };
        for x in first..=last {
            buf[(col + x, y)].set_symbol(symbol).set_fg(color);
        }
        if a < s {
            buf[(col + first, y)].set_symbol("◂");
        }
        if b > e {
            buf[(col + last, y)].set_symbol("▸");
        }
    }
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
    let w = area.width;
    let h = area.height;
    let t = o.theme;
    // Lay out complete cards before scrolling so selected rows share the exact same model.
    let capacity = report
        .repos
        .iter()
        .map(|g| view::sessions(g).count() + g.concurrency.len() + 7)
        .sum::<usize>()
        + 1;
    let mut body = Buffer::empty(Rect::new(0, 0, w, capacity.min(65500) as u16));
    body.set_style(body.area, Style::default().bg(t.screen));
    let mut y = 0;
    let mut target = 0;
    let mut selected_y = None;
    let labels = if w >= 100 {
        46
    } else if w >= 80 {
        30
    } else {
        24
    };
    let col = labels + 5;
    let bars = w >= 60;
    let cells = w.saturating_sub(col + 3);
    if w >= 10 {
        for g in &report.repos {
            let quiet = view::collapsed(g, &o.expanded_repos);
            let lanes = view::lanes(report, g, &o.split, o.show_all.contains(&g.repo));
            let rows = if quiet {
                1
            } else {
                1 + lanes.visible.len()
                    + usize::from(lanes.hidden > 0)
                    + usize::from(g.latest_handoff.is_some())
                    + g.concurrency.len()
            };
            let bg = match g.tier {
                Tier::Waiting => t.waiting_card,
                Tier::Busy => t.busy_card,
                _ => t.screen,
            };
            let border = match g.tier {
                Tier::Waiting => t.needs_you,
                Tier::Busy => t.claude,
                _ => t.border,
            };
            for (dy, left, right) in [(0, "╭", "╮"), (rows as u16 + 1, "╰", "╯")] {
                body.set_string(
                    1,
                    y + dy,
                    format!("{left}{}{right}", "─".repeat(usize::from(w - 4))),
                    Style::default().fg(border).bg(t.screen),
                );
            }
            for dy in 1..=rows as u16 {
                body.set_style(Rect::new(2, y + dy, w - 4, 1), Style::default().bg(bg));
                body[(1, y + dy)].set_symbol("│").set_fg(border);
                body[(w - 2, y + dy)].set_symbol("│").set_fg(border);
            }
            y += 1;
            draw_card_title(&mut body, y, g, report, o, quiet, bg);
            if quiet {
                if selection == Some(target) {
                    selected_y = Some(y);
                }
                target += 1;
                y += 1;
            } else {
                y += 1;
                for l in lanes.visible {
                    if selection == Some(target) {
                        selected_y = Some(y);
                    }
                    target += 1;
                    draw_spans(
                        &mut body,
                        3,
                        y,
                        label(l, report, o, bg),
                        if bars { labels } else { w - 6 },
                    );
                    if bars && cells > 0 {
                        draw_bar(&mut body, y, col, cells, l, report, t);
                    }
                    y += 1;
                }
                if lanes.hidden > 0 {
                    if selection == Some(target) {
                        selected_y = Some(y);
                    }
                    target += 1;
                    write(
                        &mut body,
                        3,
                        y,
                        &format!(
                            "{} earlier session{} · enter shows all",
                            lanes.hidden,
                            if lanes.hidden == 1 { "" } else { "s" }
                        ),
                        w - 6,
                        t.secondary,
                        bg,
                    );
                    y += 1;
                }
                for warning in &g.concurrency {
                    write(
                        &mut body,
                        3,
                        y,
                        &format!("⚠ shared checkout: {}", basename(&warning.cwd)),
                        w - 6,
                        t.warning,
                        bg,
                    );
                    y += 1;
                }
                if let Some(l) = &g.latest_handoff {
                    let step = l.step.as_deref().unwrap_or("?");
                    let words = view::link_words(l);
                    let text = if w >= 100 {
                        format!(
                            "last: claude handed step {step} to codex at {} ({words})",
                            view::clock(l.at, report.now, o.offset)
                        )
                    } else {
                        format!("last: step {step} → codex · {words}")
                    };
                    write(&mut body, 3, y, &text, w - 6, t.secondary, bg);
                    y += 1;
                }
            }
            y += 2;
        }
    }
    let mut screen = Buffer::empty(Rect::new(0, 0, w, h));
    screen.set_style(screen.area, Style::default().bg(t.screen));
    draw_header(&mut screen, report, o);
    if h > 1 {
        screen.set_string(
            0,
            1,
            "─".repeat(usize::from(w)),
            Style::default().fg(t.border).bg(t.screen),
        );
    }
    if h > 2 && bars && cells > 0 {
        draw_axis(&mut screen, report, o, labels, col, cells);
    }
    let footer_rows = if o.hide_footer { 0 } else { 2 };
    let body_height = h.saturating_sub(3 + footer_rows);
    let offset = selected_y.map_or(0, |sy| sy.saturating_sub(body_height.saturating_sub(1)));
    for dy in 0..body_height {
        if offset + dy >= body.area.height {
            break;
        }
        for x in 0..w {
            screen[(x, dy + 3)] = body[(x, offset + dy)].clone();
            if selected_y == Some(offset + dy) {
                screen[(x, dy + 3)].set_bg(t.selected);
            }
        }
    }
    if !o.hide_footer && h >= 2 {
        draw_footer(&mut screen, o);
    }
    for dy in 0..h {
        for dx in 0..w {
            buf[(area.x + dx, area.y + dy)] = screen[(dx, dy)].clone();
        }
    }
}
pub(crate) fn short_id(id: &str) -> String {
    let chars: Vec<char> = id.chars().collect();
    if chars.len() <= 13 {
        return id.to_string();
    }
    let head: String = chars[..8].iter().collect();
    let tail: String = chars[chars.len() - 4..].iter().collect();
    format!("{head}…{tail}")
}

/// Last `/`-separated component of a recorded path.
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
    let all: Vec<_> = report.repos.iter().flat_map(view::sessions).collect();
    let n = |s| all.iter().filter(|r| r.state == s).count();
    let mut parts = Vec::new();
    let mut add = |text: String, fg, bold, group, drop_priority| {
        parts.push(HeaderPart {
            span: segment(text, fg, t.screen, bold),
            group,
            drop_priority,
        })
    };
    add("All repos · last 2h".into(), t.text, true, 0, 5);
    add(format!("● {} busy", n(State::Busy)), t.claude, false, 1, 4);
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
            6,
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

fn draw_axis(
    screen: &mut Buffer,
    report: &OverviewReport,
    o: &HomeOptions<'_>,
    labels: u16,
    col: u16,
    cells: u16,
) {
    let t = o.theme;
    let legend = vec![
        segment("━", t.claude, t.screen, false),
        segment(" claude  ", t.secondary, t.screen, false),
        segment("█", t.codex, t.screen, false),
        segment(
            " codex  bright busy · dim idle/done",
            t.secondary,
            t.screen,
            false,
        ),
    ];
    let full: String = legend.iter().map(|s| s.content.as_ref()).collect();
    if Span::raw(&full).width() <= usize::from(labels) {
        screen.set_line(3, 2, &Line::from(legend), labels);
    } else {
        write(
            screen,
            3,
            2,
            "━ claude  █ codex",
            labels,
            t.secondary,
            t.screen,
        );
    }
    let window = crate::TimeWindow::default_for(report);
    let mid = multi_link_model::Timestamp::from_epoch_ms(
        (view::datetime(window.start).unix_timestamp_nanos() / 1_000_000) as i64 + 3_600_000,
    )
    .unwrap();
    let mut start = view::clock(window.start, report.now, o.offset);
    if o.offset == UtcOffset::UTC {
        start.push_str(" UTC");
    }
    let end = format!("{} now", view::clock(window.end, report.now, o.offset));
    write(screen, col, 2, &start, cells, t.secondary, t.screen);
    if cells >= 30 {
        let mid = view::datetime(mid).to_offset(o.offset);
        let middle = format!("{:02}:{:02}", mid.hour(), mid.minute());
        write(
            screen,
            col + cells / 2 - 2,
            2,
            &middle,
            5,
            t.secondary,
            t.screen,
        );
    }
    if cells >= 16 {
        let ew = Span::raw(&end).width() as u16;
        write(screen, col + cells - ew, 2, &end, ew, t.secondary, t.screen);
    }
}

fn draw_footer(screen: &mut Buffer, o: &HomeOptions<'_>) {
    let w = screen.area.width;
    let h = screen.area.height;
    let t = o.theme;
    screen.set_string(
        0,
        h - 2,
        "─".repeat(usize::from(w)),
        Style::default().fg(t.border).bg(t.screen),
    );
    let mut keys = vec![
        ("↑↓", "move"),
        ("enter", "expand / open"),
        ("t", "timeline"),
        ("o", "open session"),
        ("f", "fork"),
        ("q", "quit"),
    ];
    let tz = format!("times in {}", view::offset_label(o.offset));
    let tw = Span::raw(&tz).width() as u16;
    let minimum = Span::raw("↑↓ move · q quit").width();
    let show_time = minimum + usize::from(tw) + 4 <= usize::from(w);
    let available = w.saturating_sub(if show_time { tw + 4 } else { 2 });
    let mut spans = footer_spans(&keys, t);
    while spans_width(&spans) > usize::from(available) && keys.len() > 2 {
        keys.remove(keys.len() - 2);
        spans = footer_spans(&keys, t);
    }
    // Extremely small surfaces can show only the complete quit item (or nothing).
    if spans_width(&spans) > usize::from(available) {
        spans = footer_spans(&[("q", "quit")], t);
    }
    if spans_width(&spans) <= usize::from(available) {
        screen.set_line(1.min(w - 1), h - 1, &Line::from(spans), available);
    }
    if show_time {
        screen.set_string(w - tw - 1, h - 1, tz, t.text_style(t.secondary, t.screen));
    }
}

fn footer_spans(keys: &[(&str, &str)], t: Theme) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for (index, (key, words)) in keys.iter().enumerate() {
        if index > 0 {
            spans.push(segment(" · ", t.secondary, t.screen, false));
        }
        spans.push(segment(*key, t.text, t.screen, false));
        spans.push(segment(format!(" {words}"), t.secondary, t.screen, false));
    }
    spans
}

fn draw_card_title(
    body: &mut Buffer,
    y: u16,
    g: &RepoGroup,
    report: &OverviewReport,
    o: &HomeOptions<'_>,
    quiet: bool,
    bg: Color,
) {
    let w = body.area.width;
    let t = o.theme;
    let name = basename(&g.repo);
    let mut right = Vec::new();
    if quiet {
        for harness in [Harness::Claude, Harness::Codex] {
            for (states, words) in [
                (&[State::Idle][..], "idle"),
                (&[State::Unknown, State::Finished][..], "done or unknown"),
            ] {
                let n = view::sessions(g)
                    .filter(|r| r.harness == harness && states.contains(&r.state))
                    .count();
                if n > 0 {
                    right.push(format!(
                        "{n} {} {words}",
                        if harness == Harness::Claude {
                            "claude"
                        } else {
                            "codex"
                        }
                    ));
                }
            }
        }
        right.push(format!(
            "last {}",
            view::clock(g.last_activity, report.now, o.offset)
        ));
    } else {
        for (state, word) in [
            (State::Waiting, "waiting"),
            (State::Busy, "busy"),
            (State::Stale, "stale"),
            (State::Idle, "idle"),
        ] {
            let n = count(g, state);
            if n > 0 {
                right.push(format!("{n} {word}"));
            }
        }
    }
    while right.len() > 1
        && Span::raw(right.join(" · ")).width() + Span::raw(name).width().min(18) + 8
            > usize::from(w - 3)
    {
        right.pop();
    }
    let right_text = right.join(" · ");
    let rw = Span::raw(&right_text).width() as u16;
    let rx = w.saturating_sub(3 + rw);
    let state = match g.tier {
        Tier::Waiting => State::Waiting,
        Tier::Busy => State::Busy,
        _ => State::Idle,
    };
    write(body, 3, y, glyph(state), 1, state_color(t, state), bg);
    body.set_line(
        5,
        y,
        &Line::from(segment(
            fit(
                name,
                if quiet {
                    18
                } else {
                    usize::from(rx.saturating_sub(8))
                },
            ),
            t.text,
            bg,
            true,
        )),
        w - 8,
    );
    let px = if quiet {
        24
    } else {
        8 + Span::raw(name).width() as u16
    };
    let mut plan_spans = vec![segment(
        plan(g),
        if quiet { t.secondary } else { t.text },
        bg,
        false,
    )];
    if !quiet && let Some(entry) = g.plan.latest_step.as_ref().and_then(|s| s.entry.as_ref()) {
        let (word, color) = match entry.verify.result {
            VerifyResult::Pass => ("✓", t.secondary),
            VerifyResult::Fail => ("✗", t.warning),
            _ => ("unclear", t.secondary),
        };
        plan_spans.push(segment(format!(" · Verify {word}"), color, bg, false));
    }
    let pw = rx.saturating_sub(px + 3);
    let wording = plan(g);
    let fitted = fit(&wording, usize::from(pw));
    let useful_plan =
        !fitted.ends_with('…') || fitted.trim_end_matches('…').split_whitespace().count() > 1;
    if px < w - 3 && useful_plan {
        draw_spans(body, px, y, plan_spans, pw);
    }
    if rw + 6 < w {
        let mut x = rx;
        for (i, part) in right.iter().enumerate() {
            if i > 0 {
                body.set_string(x, y, " · ", Style::default().fg(t.secondary).bg(bg));
                x += 3;
            }
            let color = if quiet {
                t.secondary
            } else if part.ends_with("waiting") {
                t.needs_you
            } else if part.ends_with("busy") {
                t.claude
            } else if part.ends_with("stale") {
                t.warning
            } else {
                t.secondary
            };
            body.set_string(x, y, part, Style::default().fg(color).bg(bg));
            x += Span::raw(part).width() as u16;
        }
    }
}
