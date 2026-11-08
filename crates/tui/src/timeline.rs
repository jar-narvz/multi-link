//! Board 8c: recorded activity swimlanes with a fitted, optionally folded axis.
use std::time::Duration;

use crate::{
    home::{basename, fit, plan_fit},
    theme::Theme,
    view,
};
use multi_link_model::{
    Harness, LinksReport, Match, OverviewReport, RepoGroup, Row, SessionRow, State, StateEvidence,
    Timestamp,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
};
use time::UtcOffset;

pub const DEFAULT_WINDOW: Duration = Duration::from_secs(7200);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeWindow {
    pub start: Timestamp,
    pub end: Timestamp,
}
impl TimeWindow {
    pub fn new(start: Timestamp, end: Timestamp) -> Self {
        Self { start, end }
    }
    pub fn ending_at(end: Timestamp, length: Duration) -> Self {
        let start = i64::try_from(length.as_millis())
            .ok()
            .and_then(|n| end.epoch_ms().checked_sub(n))
            .and_then(|n| Timestamp::from_epoch_ms(n).ok())
            .unwrap_or(end);
        Self { start, end }
    }
    pub fn default_for(report: &OverviewReport) -> Self {
        Self::ending_at(report.now, DEFAULT_WINDOW)
    }
}

/// View state is independent of the reports and never changes their rows or links.
#[derive(Clone, Copy)]
pub struct RepoOptions {
    pub fitted: bool,
    pub fold: bool,
    pub imports: bool,
    pub theme: Theme,
    pub offset: UtcOffset,
}
impl Default for RepoOptions {
    fn default() -> Self {
        Self {
            fitted: true,
            fold: true,
            imports: false,
            theme: Theme::from_env(Some("truecolor"), None),
            offset: UtcOffset::UTC,
        }
    }
}

pub(crate) struct Lane<'a> {
    pub session: &'a SessionRow,
    pub parent: Option<&'a str>,
    /// 0 for a top-level lane, +1 per nesting level (S3: any depth).
    pub depth: u16,
}
fn visible(r: &SessionRow, imports: bool) -> bool {
    if r.imported {
        imports
    } else {
        !r.activity.is_empty()
    }
}
fn newest_first(a: &&SessionRow, b: &&SessionRow) -> std::cmp::Ordering {
    b.started_at
        .cmp(&a.started_at)
        .then_with(|| a.id.cmp(&b.id))
}
/// Rendering and keyboard selection must use this same ordering and visibility rule.
/// A visible row nests under its direct parent when that parent is visible, to any depth;
/// otherwise it is a top-level lane. Every level is newest first (W6), ties by id.
pub(crate) fn lanes(group: &RepoGroup, imports: bool) -> Vec<Lane<'_>> {
    fn walk<'a>(
        row: &'a SessionRow,
        parent_visible: bool,
        imports: bool,
        roots: &mut Vec<&'a SessionRow>,
    ) {
        let shown = visible(row, imports);
        if shown && !parent_visible {
            roots.push(row);
        }
        for child in &row.children {
            walk(child, shown, imports, roots);
        }
    }
    fn emit<'a>(
        row: &'a SessionRow,
        parent: Option<&'a str>,
        depth: u16,
        imports: bool,
        out: &mut Vec<Lane<'a>>,
    ) {
        out.push(Lane {
            session: row,
            parent,
            depth,
        });
        let mut children: Vec<_> = row
            .children
            .iter()
            .filter(|c| visible(c, imports))
            .collect();
        children.sort_by(newest_first);
        for child in children {
            emit(child, Some(&row.id), depth + 1, imports, out);
        }
    }
    let mut roots = Vec::new();
    for top in group.rows.iter().flat_map(|r| match r {
        Row::Session(r) => std::slice::from_ref(r),
        Row::Merged(r) => r.rows.as_slice(),
    }) {
        walk(top, false, imports, &mut roots);
    }
    roots.sort_by(newest_first);
    let mut out = Vec::new();
    for root in roots {
        emit(root, None, 0, imports, &mut out);
    }
    out
}

#[derive(Debug)]
struct Segment {
    from: i64,
    to: i64,
    x: u16,
    width: u16,
}
#[derive(Debug)]
struct Fold {
    from: i64,
    to: i64,
    x: u16,
}
#[derive(Debug)]
struct Axis {
    segments: Vec<Segment>,
    folds: Vec<Fold>,
    start: i64,
    end: i64,
    cells: u16,
}
/// Furthest column a nested lane's harness word may start at: `claude` plus its blank
/// then end at column 41, before the bar gap at 42.
const MAX_HARNESS_X: u16 = 35;

const STEPS: [i64; 11] = [
    60, 120, 300, 600, 900, 1800, 3600, 7200, 21600, 43200, 86400,
];
impl Axis {
    fn new(start: i64, end: i64, cells: u16, bursts: &[(i64, i64)], fold: bool) -> Self {
        let mut spans: Vec<_> = bursts
            .iter()
            .map(|&(a, b)| (a.max(start), b.min(end)))
            .filter(|&(a, b)| a <= b)
            .collect();
        spans.sort();
        let mut cursor = start;
        let mut gaps = Vec::new();
        for (a, b) in spans {
            if a - cursor > 900_000 {
                gaps.push((cursor, a));
            }
            cursor = cursor.max(b);
        }
        if end - cursor > 900_000 {
            gaps.push((cursor, end));
        }
        gaps.sort_by_key(|&(a, b)| (std::cmp::Reverse(b - a), a));
        gaps.truncate(if fold {
            3.min(usize::from(cells / 6))
        } else {
            0
        });
        gaps.sort();
        let mut ranges = Vec::new();
        cursor = start;
        for &(a, b) in &gaps {
            ranges.push((cursor, a));
            cursor = b;
        }
        ranges.push((cursor, end));
        let available = cells.saturating_sub(gaps.len() as u16 * 5);
        let total: i128 = ranges
            .iter()
            .map(|&(a, b)| i128::from((b - a).max(0)))
            .sum();
        let mut widths: Vec<u16> = ranges
            .iter()
            .map(|&(a, b)| {
                if total > 0 {
                    (i128::from((b - a).max(0)) * i128::from(available) / total) as u16
                } else {
                    0
                }
            })
            .collect();
        let mut order: Vec<_> = (0..ranges.len()).collect();
        order.sort_by_key(|&i| {
            (
                std::cmp::Reverse(if total > 0 {
                    i128::from((ranges[i].1 - ranges[i].0).max(0)) * i128::from(available) % total
                } else {
                    0
                }),
                i,
            )
        });
        let remaining = available - widths.iter().sum::<u16>();
        for i in 0..usize::from(remaining) {
            widths[order[i % order.len()]] += 1;
        }
        let mut x = 0;
        let mut segments = Vec::new();
        let mut folds = Vec::new();
        for (i, ((from, to), width)) in ranges.into_iter().zip(widths).enumerate() {
            segments.push(Segment { from, to, x, width });
            x += width;
            if let Some(&(from, to)) = gaps.get(i) {
                folds.push(Fold { from, to, x });
                x += 5;
            }
        }
        Self {
            segments,
            folds,
            start,
            end,
            cells,
        }
    }
    fn col(&self, at: i64) -> Option<u16> {
        self.segments
            .iter()
            .find(|s| s.width > 0 && at >= s.from && at <= s.to)
            .map(|s| {
                s.x + if s.to == s.from {
                    0
                } else {
                    ((i128::from(at - s.from) * i128::from(s.width) / i128::from(s.to - s.from))
                        as u16)
                        .min(s.width - 1)
                }
            })
    }
    /// Stop visiting as soon as spacing fails, rather than allocating every minute
    /// in a long history window for each candidate step.
    fn visit_ticks(
        &self,
        step: i64,
        offset: UtcOffset,
        mut visit: impl FnMut(usize, u16, i64) -> bool,
    ) -> bool {
        let step = step * 1000;
        let offset = i64::from(offset.whole_seconds()) * 1000;
        for (i, s) in self
            .segments
            .iter()
            .enumerate()
            .filter(|(_, s)| s.width > 0)
        {
            let mut at = (s.from + offset).div_euclid(step) * step - offset;
            if at < s.from {
                at += step;
            }
            while at <= s.to && at < self.end {
                if let Some(col) = self.col(at)
                    && !visit(i, col, at)
                {
                    return false;
                }
                at += step;
            }
        }
        true
    }
    fn ticks(&self, step: i64, offset: UtcOffset) -> Vec<(usize, u16, i64)> {
        let mut out = Vec::new();
        self.visit_ticks(step, offset, |i, col, at| {
            out.push((i, col, at));
            true
        });
        out
    }
    fn step(&self, offset: UtcOffset) -> i64 {
        STEPS
            .into_iter()
            .find(|&step| {
                let mut previous: Option<(usize, u16)> = None;
                self.visit_ticks(step, offset, |i, col, _| {
                    let fits = previous.is_none_or(|(j, x)| j != i || col.saturating_sub(x) >= 13);
                    previous = Some((i, col));
                    fits
                })
            })
            .unwrap_or(86400)
    }
}
fn fold_label(ms: i64) -> String {
    let m = ms / 60000;
    if m < 60 {
        format!("{m}m")
    } else if m < 600 {
        format!("{}h{:02}", m / 60, m % 60)
    } else if m < 6000 {
        format!("{}h", m / 60)
    } else {
        format!("{}d", m / 1440)
    }
}
/// The vertical mark from a nested lane up to its parent: a Codex child's placing hand-off
/// (`│` confirmed, `╎` otherwise), or a started child's ancestry mark, a solid `│` at the
/// child's `started_at` (S3; not a hand-off edge).
fn connector(lane: &Lane<'_>) -> Option<(i64, &'static str)> {
    let parent = lane.parent?;
    if view::started_child(lane.session, Some(parent)) {
        return Some((lane.session.started_at.epoch_ms(), "│"));
    }
    let link = view::placing(lane.session, Some(parent))?;
    Some((
        link.at.epoch_ms(),
        if link.r#match == Match::Exact {
            "│"
        } else {
            "╎"
        },
    ))
}
fn done(r: &SessionRow) -> bool {
    r.harness == Harness::Codex
        && r.state == State::Unknown
        && r.state_evidence == StateEvidence::TurnsComplete
}

pub fn render_timeline(
    report: &OverviewReport,
    links: &LinksReport,
    repo: &str,
    window: TimeWindow,
    area: Rect,
    buf: &mut Buffer,
) {
    render_repo(
        report,
        links,
        repo,
        window,
        None,
        &RepoOptions::default(),
        area,
        buf,
    );
}

/// Draw with explicit presentation state. The compatibility name above uses a fitted axis.
#[allow(clippy::too_many_arguments)]
pub fn render_repo(
    report: &OverviewReport,
    links: &LinksReport,
    repo: &str,
    window: TimeWindow,
    selection: Option<usize>,
    options: &RepoOptions,
    area: Rect,
    buf: &mut Buffer,
) {
    let area = area.intersection(buf.area);
    if area.is_empty() {
        return;
    }
    // Local coordinates make clipping safe for embedded buffers too.
    let mut screen = Buffer::empty(Rect::new(0, 0, area.width, area.height));
    draw_repo(report, links, repo, window, selection, options, &mut screen);
    for y in 0..area.height {
        for x in 0..area.width {
            buf[(area.x + x, area.y + y)] = screen[(x, y)].clone();
        }
    }
}

/// Painter clips every piece of text to its region and resets styles on repainted rows.
struct Paint<'a> {
    buf: &'a mut Buffer,
    theme: Theme,
}
impl Paint<'_> {
    fn text(&mut self, x: u16, y: u16, text: &str, width: u16, fg: Color, bg: Color) {
        if y < self.buf.area.height && x < self.buf.area.width {
            self.buf.set_stringn(
                x,
                y,
                text,
                usize::from(width.min(self.buf.area.width - x)),
                self.theme.text_style(fg, bg),
            );
        }
    }
    fn put(&mut self, x: u16, y: u16, text: &str, fg: Color, bg: Color) {
        self.text(x, y, text, self.buf.area.width.saturating_sub(x), fg, bg);
    }
    fn rule(&mut self, y: u16) {
        self.put(
            1,
            y,
            &"─".repeat(usize::from(self.buf.area.width.saturating_sub(2))),
            self.theme.border,
            self.theme.screen,
        );
    }
    fn background(&mut self, y: u16, bg: Color) {
        if y < self.buf.area.height {
            for x in 0..self.buf.area.width {
                self.buf[(x, y)].reset();
                self.buf[(x, y)].set_style(Style::default().bg(bg));
            }
        }
    }
    fn keys(&mut self, y: u16, x: u16, keys: &[(&str, &str)], limit: u16) {
        let mut x = x;
        for &(key, words) in keys {
            let n = Span::raw(format!("{key} {words}")).width() as u16;
            if x + n > limit {
                break;
            }
            self.put(x, y, key, self.theme.text, self.theme.screen);
            self.put(
                x + key.chars().count() as u16 + 1,
                y,
                words,
                self.theme.secondary,
                self.theme.screen,
            );
            x += n + 3;
        }
    }
}

pub(crate) fn period(window: TimeWindow) -> String {
    let ms = window.end.epoch_ms() - window.start.epoch_ms();
    if ms == 7_200_000 {
        "2h".into()
    } else {
        fold_label(ms)
    }
}
#[allow(clippy::too_many_arguments)]
fn draw_repo(
    report: &OverviewReport,
    links: &LinksReport,
    repo: &str,
    window: TimeWindow,
    selection: Option<usize>,
    o: &RepoOptions,
    buf: &mut Buffer,
) {
    let t = o.theme;
    let w = buf.area.width;
    let h = buf.area.height;
    let mut p = Paint { buf, theme: t };
    for y in 0..h {
        p.background(y, t.screen);
    }
    let Some(group) = report.repos.iter().find(|g| g.repo == repo) else {
        p.put(1, 0, "repo not in this overview", t.secondary, t.screen);
        return;
    };
    let lanes = lanes(group, o.imports);
    let bursts: Vec<_> = lanes
        .iter()
        .flat_map(|l| &l.session.activity)
        .map(|b| (b.from.epoch_ms(), b.to.epoch_ms()))
        .collect();
    let first = bursts.iter().map(|b| b.0).min();
    let last = bursts.iter().map(|b| b.1).max();
    let start = if o.fitted {
        first
            .unwrap_or(window.start.epoch_ms())
            .max(window.start.epoch_ms())
    } else {
        window.start.epoch_ms()
    };
    let end = if o.fitted {
        report.now.epoch_ms().min(window.end.epoch_ms())
    } else {
        window.end.epoch_ms()
    };
    let axis = Axis::new(
        start,
        end,
        w.saturating_sub(46),
        &bursts,
        o.fitted && o.fold,
    );
    let bars = w >= 80 && (end > start || (o.fitted && end == start)) && axis.cells > 0;
    let ticks = if bars {
        axis.ticks(axis.step(o.offset), o.offset)
    } else {
        vec![]
    };
    let scene = RepoScene {
        report,
        links,
        group,
        options: o,
        lanes,
        axis,
        bars,
        ticks,
        selection,
    };
    scene.draw_header(&mut p, window, first, last);
    scene.draw_axis(&mut p);
    let bottom = h.saturating_sub(11);
    let body_end = if h >= 18 { bottom } else { h.saturating_sub(1) };
    let body_start = if h >= 9 {
        6
    } else if h >= 4 {
        2.min(body_end)
    } else {
        0
    };
    let capacity = body_end.saturating_sub(body_start);
    let first_lane = selection.map_or(0, |s| {
        s.saturating_sub(usize::from(capacity.saturating_sub(2) / 3))
    });
    let mut positions = Vec::new();
    let mut y = body_start;
    for (i, lane) in scene.lanes.iter().enumerate().skip(first_lane) {
        if y >= body_end {
            break;
        }
        let bg = if selection == Some(i) {
            t.selected
        } else if lane.parent.is_some() {
            t.child
        } else {
            t.screen
        };
        p.background(y, bg);
        if y + 1 < body_end {
            p.background(y + 1, bg);
        }
        positions.push((i, y, bg));
        y = y.saturating_add(3);
    }
    let grid_end = y.saturating_sub(1).min(body_end.saturating_sub(1));
    scene.draw_grid(&mut p, body_start.saturating_sub(1), grid_end);
    scene.draw_links(&mut p, &positions);
    for &(i, y, bg) in &positions {
        scene.draw_lane(&mut p, i, y, bg, body_end);
    }
    if bars {
        for &(i, y, bg) in &positions {
            scene.draw_done(&mut p, scene.lanes[i].session, y, bg, &positions);
        }
    }
    if scene.lanes.is_empty() {
        p.put(
            2,
            body_start,
            &format!("no recorded activity in the last {}", period(window)),
            t.secondary,
            t.screen,
        );
    }
    scene.draw_imports(&mut p, y, body_end);
    if h >= 18 {
        scene.draw_selected(&mut p, bottom);
        scene.draw_legend(&mut p);
    }
    scene.draw_footer(&mut p);
}

/// Shared geometry keeps axis, grid, links and bars on the same columns.
struct RepoScene<'a> {
    report: &'a OverviewReport,
    links: &'a LinksReport,
    group: &'a RepoGroup,
    options: &'a RepoOptions,
    lanes: Vec<Lane<'a>>,
    axis: Axis,
    bars: bool,
    ticks: Vec<(usize, u16, i64)>,
    selection: Option<usize>,
}
impl RepoScene<'_> {
    fn clock(&self, ms: i64) -> String {
        let at = view::datetime(Timestamp::from_epoch_ms(ms).unwrap_or(self.report.now))
            .to_offset(self.options.offset);
        format!("{:02}:{:02}", at.hour(), at.minute())
    }
    fn tick_label(&self, ms: i64) -> String {
        let at = view::datetime(Timestamp::from_epoch_ms(ms).unwrap_or(self.report.now))
            .to_offset(self.options.offset);
        // Day labels distinguish midnight ticks on multi-day axes. Short ranges,
        // including those crossing midnight, always retain clock labels.
        if self.axis.end - self.axis.start > 86_400_000 && at.time() == time::Time::MIDNIGHT {
            format!("{} {}", &at.month().to_string()[..3], at.day())
        } else {
            self.clock(ms)
        }
    }

    fn draw_header(
        &self,
        p: &mut Paint<'_>,
        window: TimeWindow,
        first: Option<i64>,
        last: Option<i64>,
    ) {
        let report = self.report;
        let group = self.group;
        let o = self.options;
        let w = p.buf.area.width;
        let h = p.buf.area.height;
        let t = self.options.theme;
        let clock = |ms| self.clock(ms);
        let title = format!("Sessions in {}", basename(self.group.repo.as_str()));
        p.put(1, 0, &title, t.text, t.screen);
        if h > 0 {
            for x in 1..(1 + Span::raw(&title).width() as u16).min(w) {
                p.buf[(x, 0)].modifier.insert(Modifier::BOLD);
            }
        }
        let period = period(window);
        let range = if o.fitted {
            match (first, last) {
                (Some(a), Some(b)) => format!("last {period} · active {}–{}", clock(a), clock(b)),
                _ => format!("last {period}"),
            }
        } else {
            format!(
                "{}–{}{}",
                clock(window.start.epoch_ms()),
                clock(window.end.epoch_ms()),
                if window.start <= report.now && report.now <= window.end {
                    " · now in range"
                } else {
                    ""
                }
            )
        };
        let rx = w.saturating_sub(range.chars().count() as u16 + 1);
        p.text(
            Span::raw(&title).width() as u16 + 3,
            0,
            &plan_fit(
                group,
                usize::from(rx.saturating_sub(Span::raw(&title).width() as u16 + 5)),
            ),
            rx.saturating_sub(Span::raw(&title).width() as u16 + 3),
            t.secondary,
            t.screen,
        );
        p.put(rx, 0, &range, t.secondary, t.screen);
        p.rule(1);
        p.rule(4);
    }
    fn draw_axis(&self, p: &mut Paint<'_>) {
        let report = self.report;
        let axis = &self.axis;
        let bars = self.bars;
        let ticks = &self.ticks;
        let t = self.options.theme;
        let clock = |ms| self.clock(ms);
        let end = self.axis.end;
        p.put(2, 3, "session", t.secondary, t.screen);
        if !bars {
            return;
        }
        // Reserve actual label cells in priority order. Every later label must fit
        // entirely in the axis, with a blank cell between labels to keep words distinct.
        let mut occupied = Vec::new();
        let mut place = |x: u16, label: &str| {
            let right = x.saturating_add(label.chars().count() as u16);
            if right > axis.cells || occupied.iter().any(|&(a, b)| x <= b && a <= right) {
                return false;
            }
            occupied.push((x, right));
            p.put(44 + x, 3, label, t.secondary, t.screen);
            true
        };
        for f in &axis.folds {
            let label = fold_label(f.to - f.from);
            place(f.x + (5u16.saturating_sub(label.len() as u16)) / 2, &label);
        }
        let end_clock = clock(end);
        let now = report.now.epoch_ms();
        let end_label = if end == now {
            format!("{end_clock} now")
        } else {
            end_clock.clone()
        };
        for label in [end_label.as_str(), if end == now { "now" } else { "" }] {
            if !label.is_empty()
                && let Some(x) = axis.cells.checked_sub(label.len() as u16)
                && place(x, label)
            {
                break;
            }
        }
        let now_clock = clock(now);
        let interior_now = !self.options.fitted && axis.start <= now && now < end;
        if interior_now && let Some(x) = axis.col(now) {
            for label in [format!("{now_clock} now"), "now".into()] {
                if place(x, &label) {
                    break;
                }
            }
        }
        for &(i, x, at) in ticks {
            // Compare clock strings, including when the priority label shortened
            // to `now` or had to be dropped altogether.
            let tick_clock = clock(at);
            if tick_clock == end_clock || (interior_now && tick_clock == now_clock) {
                continue;
            }
            let label = self.tick_label(at);
            let s = &axis.segments[i];
            if x + label.len() as u16 <= s.x + s.width {
                place(x, &label);
            }
        }
    }

    fn draw_links(&self, p: &mut Paint<'_>, positions: &[(usize, u16, Color)]) {
        let lanes = &self.lanes;
        let axis = &self.axis;
        let bars = self.bars;
        let t = self.options.theme;
        for &(i, cy, _) in positions {
            let lane = &lanes[i];
            if let Some(parent) = lane.parent
                && let Some((at, glyph)) = connector(lane)
                && let Some((_, py, _)) = positions
                    .iter()
                    .find(|&&(j, _, _)| lanes[j].session.id == parent)
                && bars
                && let Some(x) = axis.col(at)
            {
                for y in py + 1..cy {
                    let bg = p.buf[(0, y)].bg;
                    p.put(44 + x, y, glyph, t.secondary, bg);
                }
            }
        }
    }
    fn draw_lane(&self, p: &mut Paint<'_>, i: usize, y: u16, bg: Color, body_end: u16) {
        let group = self.group;
        let lanes = &self.lanes;
        let selection = self.selection;
        let bars = self.bars;
        let w = p.buf.area.width;
        let t = self.options.theme;
        let lane = &lanes[i];
        let r = lane.session;
        let child = lane.parent.is_some();
        let color = if r.imported {
            t.secondary
        } else if r.harness == Harness::Claude {
            t.claude
        } else {
            t.codex
        };
        let harness = if r.harness == Harness::Claude {
            "claude"
        } else {
            "codex"
        };
        if selection == Some(i) {
            p.put(0, y, "▌", color, bg);
            if y + 1 < body_end {
                p.put(0, y + 1, "▌", color, bg);
            }
        }
        // Each nesting level indents 2 columns (S3); depth 1 keeps the Phase 6 columns.
        // The harness word and its blank must end before column 42 (the bar gap), so a
        // very deep chain stops indenting there instead of painting into the bars.
        let wanted = 6 + 2 * lane.depth.saturating_sub(1);
        let hx = if child { wanted.min(MAX_HARNESS_X) } else { 2 };
        if child {
            p.put(hx - 3, y, "└▸", t.secondary, bg);
        }
        p.put(hx, y, harness, color, bg);
        let label = if r.imported {
            format!("import · {}", view::home_label(group, r))
        } else {
            view::home_label(group, r)
        };
        // One blank after the harness word: `codex` children keep column 12 at depth 1.
        let lx = if child {
            hx + harness.len() as u16 + 1
        } else {
            9
        };
        let label_width = if bars {
            42u16.saturating_sub(lx)
        } else {
            w.saturating_sub(lx + 1)
        };
        let label = label
            .strip_prefix('“')
            .and_then(|s| s.strip_suffix('”'))
            .map_or_else(
                || fit(&label, usize::from(label_width)),
                |s| format!("“{}”", fit(s, usize::from(label_width.saturating_sub(2)))),
            );
        p.text(
            lx,
            y,
            &label,
            label_width,
            if r.imported { t.secondary } else { t.text },
            bg,
        );
        let sub = self.subline(lane);
        if y + 1 < body_end {
            let sx = if child { hx } else { 4 };
            // Depth 1 keeps its Phase 6 widths; deeper sub lines stop before the bar gap too.
            let room = if bars && lane.depth >= 2 {
                42u16.saturating_sub(sx)
            } else {
                w.saturating_sub(sx + 1)
            };
            p.text(sx, y + 1, &sub, room, t.secondary, bg);
        }
        if self.bars {
            self.draw_bars(p, r, y, color, bg);
        }
    }
    fn subline(&self, lane: &Lane<'_>) -> String {
        let report = self.report;
        let links = self.links;
        let o = self.options;
        let r = lane.session;
        if view::started_child(r, lane.parent) {
            format!(
                "started by this session · {}",
                self.clock(r.started_at.epoch_ms())
            )
        } else if lane.parent.is_some() {
            view::placing(r, lane.parent)
                .map(|l| {
                    let seconds = (r.started_at.epoch_ms() - l.at.epoch_ms()).max(0) / 1000;
                    format!(
                        "{} · {} after the call",
                        view::link_words(l),
                        if seconds < 60 {
                            format!("{seconds} s")
                        } else {
                            format!("{} min", seconds / 60)
                        }
                    )
                })
                .unwrap_or_default()
        } else {
            let place = view::place(r);
            let mut sub = if place.is_empty() {
                String::new()
            } else {
                format!("{place} · ")
            };
            sub += &view::status(r, report.now, o.offset);
            if let Some(l) = links.links.iter().find(|l| {
                l.r#match == Match::Ambiguous && l.candidates.iter().any(|c| c.id == r.id)
            }) {
                sub += &format!(" · {}", view::link_words(l));
            }
            sub
        }
    }
    fn draw_bars(&self, p: &mut Paint<'_>, r: &SessionRow, y: u16, color: Color, bg: Color) {
        let axis = &self.axis;
        let t = self.options.theme;

        for burst in &r.activity {
            let a = burst.from.epoch_ms().max(axis.start);
            let b = burst.to.epoch_ms().min(axis.end);
            if a <= b
                && let (Some(a), Some(b)) = (axis.col(a), axis.col(b))
            {
                for x in a..=b {
                    p.put(44 + x, y, "█", color, bg);
                }
            }
        }
        if let Some(b) = r.activity.last()
            && let Some(x) = axis.col(b.to.epoch_ms())
            && x + 1 < axis.cells
            && r.state == State::Waiting
        {
            p.put(45 + x, y, "◆", t.needs_you, bg);
        }
    }
    fn draw_done(
        &self,
        p: &mut Paint<'_>,
        r: &SessionRow,
        y: u16,
        bg: Color,
        positions: &[(usize, u16, Color)],
    ) {
        let axis = &self.axis;
        let Some(x) = r
            .activity
            .last()
            .filter(|_| done(r))
            .and_then(|b| axis.col(b.to.epoch_ms()))
            .filter(|x| x + 1 < axis.cells)
        else {
            return;
        };
        // Include the blank before the six-cell label in the clearance check.
        let space = x + 1..=x + 7;
        let blocked = x + 7 >= axis.cells
            || space
                .clone()
                .any(|col| p.buf[(44 + col, y)].symbol() == "█")
            || axis
                .folds
                .iter()
                .any(|f| space.start() < &(f.x + 5) && space.end() >= &f.x)
            || positions.iter().any(|&(i, cy, _)| {
                let lane = &self.lanes[i];
                lane.parent
                    .and_then(|parent| {
                        let (at, _) = connector(lane)?;
                        let (_, py, _) = positions
                            .iter()
                            .find(|&&(j, _, _)| self.lanes[j].session.id == parent)?;
                        let col = axis.col(at)?;
                        Some(*py < y && y < cy && space.contains(&col))
                    })
                    .unwrap_or(false)
            });
        if blocked {
            p.put(45 + x, y, "✓", self.options.theme.done, bg);
        } else {
            p.put(45 + x, y, " ✓ done", self.options.theme.done, bg);
        }
    }
    fn draw_imports(&self, p: &mut Paint<'_>, y: u16, body_end: u16) {
        let report = self.report;
        let group = self.group;
        let o = self.options;
        let t = self.options.theme;
        let imports: Vec<_> = view::sessions(group).filter(|r| r.imported).collect();
        if !o.imports && !imports.is_empty() && y < body_end {
            let a = imports.iter().map(|r| r.last_activity).min().unwrap();
            let b = imports.iter().map(|r| r.last_activity).max().unwrap();
            let times = if a == b {
                format!("at {}", view::clock(a, report.now, o.offset))
            } else {
                format!(
                    "between {} and {}",
                    view::clock(a, report.now, o.offset),
                    view::clock(b, report.now, o.offset)
                )
            };
            p.put(
                2,
                y,
                &format!(
                    "▸ {} Codex Desktop import{} {times} hidden (copies of Claude chats, not runs)",
                    imports.len(),
                    if imports.len() == 1 { "" } else { "s" }
                ),
                t.secondary,
                t.screen,
            );
        }
    }
    fn draw_selected(&self, p: &mut Paint<'_>, bottom: u16) {
        let report = self.report;
        let links = self.links;
        let lanes = &self.lanes;
        let o = self.options;
        let selection = self.selection;
        let w = p.buf.area.width;
        let t = self.options.theme;
        p.rule(bottom);
        if let Some(lane) = selection.and_then(|i| lanes.get(i)) {
            let r = lane.session;
            let total = r
                .activity
                .iter()
                .map(|b| b.to.epoch_ms() - b.from.epoch_ms())
                .sum();
            let handoff = r
                .handoffs
                .iter()
                .rev()
                .find(|l| l.parent == r.id)
                .or_else(|| links.links.iter().rev().find(|l| l.parent == r.id));
            let handoff = handoff.map_or_else(
                || "no hand-offs".into(),
                |l| {
                    format!(
                        "{} at {}",
                        view::handed(l),
                        view::clock(l.at, report.now, o.offset)
                    )
                },
            );
            p.put(2, bottom + 1, "selected", t.secondary, t.screen);
            p.put(
                12,
                bottom + 1,
                &format!(
                    "{} {} · started {} · active about {} min in {} stretch{} · {handoff}",
                    if r.harness == Harness::Claude {
                        "claude"
                    } else {
                        "codex"
                    },
                    r.id.chars().take(8).collect::<String>(),
                    view::clock(r.started_at, report.now, o.offset),
                    view::minutes(total),
                    r.activity.len(),
                    if r.activity.len() == 1 { "" } else { "es" }
                ),
                t.text,
                t.screen,
            );
            let open = match r.harness {
                Harness::Claude => "open in Claude",
                Harness::Codex => "open in Codex",
            };
            p.keys(bottom + 2, 12, &[("o", open), ("f", "fork")], w);
        }
    }
    fn draw_legend(&self, p: &mut Paint<'_>) {
        let w = p.buf.area.width;
        let h = p.buf.area.height;
        let t = self.options.theme;
        let mut x = 1;
        for (glyph, words, color) in [
            ("█", "claude active", t.claude),
            ("█", "codex active", t.codex),
            ("│", "confirmed link", t.secondary),
            ("╎", "matched by time", t.secondary),
            ("◆", "waiting on you", t.needs_you),
            ("≈", "idle folded", t.secondary),
            ("✓", "done", t.done),
        ] {
            let n = words.len() as u16 + 2;
            if x + n > w {
                break;
            }
            p.put(x, h - 4, glyph, color, t.screen);
            p.put(
                x + 2,
                h - 4,
                words,
                if glyph == "◆" { color } else { t.secondary },
                t.screen,
            );
            x += n + 3;
        }
    }
    fn draw_footer(&self, p: &mut Paint<'_>) {
        let o = self.options;
        let w = p.buf.area.width;
        let h = p.buf.area.height;
        let t = self.options.theme;
        if h >= 2 {
            p.rule(h - 2);
        }
        let right = format!("times {}", view::offset_label(o.offset));
        let rx = w.saturating_sub(right.len() as u16 + 1);
        p.keys(
            h - 1,
            1,
            &[
                ("↑↓", "select"),
                ("o", "open"),
                ("f", "fork"),
                ("s", "step"),
                ("+/-", "zoom"),
                ("z", if o.fold { "unfold idle" } else { "fold idle" }),
                ("i", "imports"),
                ("esc", "home"),
                ("q", "quit"),
            ],
            rx.saturating_sub(2),
        );
        p.put(rx, h - 1, &right, t.secondary, t.screen);
    }
    fn draw_grid(&self, p: &mut Paint<'_>, start: u16, end: u16) {
        let axis = &self.axis;
        let bars = self.bars;
        let ticks = &self.ticks;
        let t = self.options.theme;
        if bars {
            for y in start..=end {
                let bg = p.buf[(0, y)].bg;
                for &(_, x, _) in ticks {
                    p.put(44 + x, y, "│", t.track, bg);
                }
                for f in &axis.folds {
                    p.put(44 + f.x, y, "≈ ≈ ≈", t.border, bg);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const MINUTE: i64 = 60_000;
    #[test]
    fn gaps_of_14_vs_16_minutes() {
        let axis = Axis::new(
            0,
            40 * MINUTE,
            74,
            &[
                (0, 5 * MINUTE),
                (19 * MINUTE, 20 * MINUTE),
                (36 * MINUTE, 40 * MINUTE),
            ],
            true,
        );
        assert_eq!(
            axis.folds
                .iter()
                .map(|f| (f.from / MINUTE, f.to / MINUTE))
                .collect::<Vec<_>>(),
            [(20, 36)]
        );
        let unfolded = Axis::new(
            0,
            40 * MINUTE,
            74,
            &[
                (0, 5 * MINUTE),
                (19 * MINUTE, 20 * MINUTE),
                (36 * MINUTE, 40 * MINUTE),
            ],
            false,
        );
        assert!(unfolded.folds.is_empty());
    }
    #[test]
    fn more_than_three_gaps_keep_longest_then_earliest() {
        let bursts = [
            (0, MINUTE),
            (21 * MINUTE, 22 * MINUTE),
            (42 * MINUTE, 43 * MINUTE),
            (73 * MINUTE, 74 * MINUTE),
            (94 * MINUTE, 95 * MINUTE),
        ];
        let axis = Axis::new(0, 95 * MINUTE, 90, &bursts, true);
        assert_eq!(
            axis.folds
                .iter()
                .map(|f| (f.from / MINUTE, f.to / MINUTE))
                .collect::<Vec<_>>(),
            [(1, 21), (22, 42), (43, 73)]
        );
        assert_eq!(axis.segments.iter().map(|s| s.width).sum::<u16>(), 75);
    }
    #[test]
    fn tick_choice_at_40_bar_columns() {
        let axis = Axis::new(0, 120 * MINUTE, 40, &[], false);
        assert_eq!(axis.step(UtcOffset::UTC), 3600);
    }
    #[test]
    fn tick_choice_at_90_bar_columns() {
        let axis = Axis::new(0, 120 * MINUTE, 90, &[], false);
        assert_eq!(axis.step(UtcOffset::UTC), 1800);
    }
    #[test]
    fn tick_choice_at_seven_day_window() {
        let axis = Axis::new(0, 7 * 24 * 60 * MINUTE, 90, &[], false);
        assert_eq!(axis.step(UtcOffset::UTC), 86400);
    }
    #[test]
    fn segment_allocation_and_window_edges_are_exact() {
        let axis = Axis::new(
            0,
            60 * MINUTE,
            74,
            &[(0, 10 * MINUTE), (50 * MINUTE, 60 * MINUTE)],
            true,
        );
        assert_eq!(
            axis.segments.iter().map(|s| s.width).collect::<Vec<_>>(),
            [35, 34]
        );
        assert_eq!(axis.col(0), Some(0));
        assert_eq!(axis.col(60 * MINUTE), Some(73));
        assert_eq!(axis.col(30 * MINUTE), None);
        assert_eq!(axis.col(-1), None);
    }
    #[test]
    fn ticks_align_to_local_midnight() {
        let axis = Axis::new(0, 120 * MINUTE, 90, &[], false);
        let offset = UtcOffset::from_hms(5, 45, 0).unwrap();
        assert_eq!(axis.ticks(1800, offset)[0].2, 15 * MINUTE);
    }
    #[test]
    fn long_history_uses_the_largest_tick_step() {
        let axis = Axis::new(0, 10 * 365 * 24 * 60 * MINUTE, 90, &[], false);
        assert_eq!(axis.step(UtcOffset::UTC), 86400);
    }
}
