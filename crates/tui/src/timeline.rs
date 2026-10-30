//! Repo timeline (design "Screen 2"): every session of one repo as a bar over a time window,
//! with hand-off markers on the parent's row.
//!
//! What a bar means: a **session span**, `started_at` → `last_activity`. It includes idle gaps;
//! the model has no busy intervals, so this screen never draws busy time (Codex P2 #10).
//!
//! Layout decisions:
//! - Rows follow the overview's order. A merged `claude ×N` row is expanded into its members
//!   (each with its Codex children): the timeline is the drill-in view of one repo, and a single
//!   merged bar could not show which session a hand-off came from.
//! - Codex children sit under their parent exactly as the overview nests them (T11: a child is
//!   listed once, under the parent of its latest hand-off). Unplaced Codex rows stay top-level.
//!   A child placed by an exact hand-off hangs from `└`; one placed by a non-exact (heuristic)
//!   hand-off from `└~`, so a guessed nesting never looks like an observed one.
//! - Labels name the state as a word (`busy`, `idle`, …), not home's glyphs: `●` `○` `?` are
//!   the hand-off markers here.
//! - Markers come from the `LinksReport`: every link whose `parent` is a session shown here is
//!   drawn at its `at` on that parent's row: exact `●`, heuristic `○`, ambiguous `?N` (N
//!   candidates; `?` alone when there are none), unlinked `×` (a launch whose child is unknown;
//!   drawn so a hand-off is never silently missing, and named in the legend).
//! - Collisions: when one cell holds several hand-offs, the least certain wins the glyph
//!   (ambiguous, heuristic, unlinked, exact), so a guess is never hidden behind `●`, and a `+`
//!   goes in the next free cell (else the previous free cell). Every marker glyph is placed
//!   before any digit or `+`, so neither ever covers another marker; `?N` keeps `?` at the
//!   link's column and draws its digits only when they all fit in free cells, else just `?`.
//! - Spans and markers are clipped to the window. A span that continues past an edge shows
//!   `◂` / `▸` in its edge cell; a link outside the window is not drawn; a zero-length span in
//!   the window still takes one cell.

use std::{collections::BTreeMap, time::Duration};

use multi_link_model::{
    Harness, Link, LinksReport, Match, OverviewReport, Row, SessionRow, State, Timestamp,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339, macros::format_description};

use crate::home::{basename, short_id};

/// Default window length: the last 2 hours before the report's `now`.
pub const DEFAULT_WINDOW: Duration = Duration::from_secs(2 * 60 * 60);

/// Widest label column: ` └~ stale codex  01a189e4…ec5d` plus one space.
const LABEL_WIDTH: u16 = 31;

const LEGEND_SPAN: &str = "━ session span: start → last activity, idle gaps included (not busy time) · ◂▸ past window edge";
const LEGEND_MARKERS: &str =
    "hand-off (parent's row): ● exact  ○ heuristic  ?N ambiguous, N candidates  × unlinked  + more";
const LEGEND_ROWS: &str = "child: └ placed by an exact hand-off  └~ by a heuristic one · state: unk unknown, done finished";

/// The time range a timeline shows, `start` to `end` inclusive. A window whose `end` is not
/// after its `start` is empty: rows are listed but no span or marker is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeWindow {
    pub start: Timestamp,
    pub end: Timestamp,
}

impl TimeWindow {
    pub fn new(start: Timestamp, end: Timestamp) -> Self {
        Self { start, end }
    }

    /// `length` ending at `end`. If the start would be out of the timestamp range, the window is
    /// empty (`start == end`) rather than a panic.
    pub fn ending_at(end: Timestamp, length: Duration) -> Self {
        let length_ms = i64::try_from(length.as_millis()).unwrap_or(i64::MAX);
        let start = epoch_ms(&end)
            .and_then(|end_ms| end_ms.checked_sub(length_ms))
            .and_then(|start_ms| Timestamp::from_epoch_ms(start_ms).ok())
            .unwrap_or(end);
        Self { start, end }
    }

    /// The default window: [`DEFAULT_WINDOW`] ending at the report's `now` (never the wall clock).
    pub fn default_for(report: &OverviewReport) -> Self {
        Self::ending_at(report.now, DEFAULT_WINDOW)
    }
}

/// Draw the timeline of `repo` (a RepoGroup `repo` string from `report`) into `area`.
/// Pure: reads only its arguments. Rows that do not fit are cut and counted on the last body
/// row. An unknown repo shows a note instead of rows. Never panics, whatever the size.
pub fn render_timeline(
    report: &OverviewReport,
    links: &LinksReport,
    repo: &str,
    window: TimeWindow,
    area: Rect,
    buf: &mut Buffer,
) {
    let area = area.intersection(buf.area);
    if area.is_empty() {
        return;
    }
    let group = report.repos.iter().find(|group| group.repo == repo);
    let lanes = group.map(lanes).unwrap_or_default();

    let label_width = LABEL_WIDTH.min(area.width / 2);
    let lane_width = area.width - label_width;
    let lane_x = area.x + label_width;
    let scale = Scale::new(&window, lane_width);

    // Rows from the top: title, tick labels, axis; from the bottom: three legend lines. Small
    // areas drop the tick labels and the last two legend lines first, then the axis and legend.
    let height = area.height;
    let full = height >= 9;
    let (title, tick_row, axis_row) = match height {
        0 | 1 => (false, false, false),
        2 | 3 => (true, false, false),
        _ => (true, full, true),
    };
    let legend_rows: u16 = if full {
        3
    } else if height >= 4 {
        1
    } else {
        0
    };

    let mut y = area.y;
    let next_row = |y: &mut u16| {
        let row = *y;
        *y = y.saturating_add(1);
        row
    };
    if title {
        let row = next_row(&mut y);
        title_line(
            group.is_some(),
            repo,
            &window,
            scale.is_some(),
            area,
            row,
            buf,
        );
    }
    if tick_row {
        let row = next_row(&mut y);
        buf.set_stringn(
            area.x,
            row,
            "UTC",
            usize::from(label_width),
            Style::new().dim(),
        );
        if let Some(scale) = &scale {
            for (col, label) in scale.tick_labels() {
                buf.set_stringn(
                    lane_x + col,
                    row,
                    &label,
                    usize::from(lane_width - col),
                    Style::new().dim(),
                );
            }
        }
    }
    if axis_row {
        let row = next_row(&mut y);
        match &scale {
            Some(scale) => {
                let ticks: Vec<u16> = scale.ticks().into_iter().map(|(col, _)| col).collect();
                for col in 0..lane_width {
                    let symbol = if ticks.contains(&col) { "┬" } else { "─" };
                    set_cell(buf, lane_x + col, row, symbol, Style::new().dim());
                }
            }
            None => {
                buf.set_stringn(
                    lane_x,
                    row,
                    "empty window",
                    usize::from(lane_width),
                    Style::new().dim(),
                );
            }
        }
    }

    let body_top = y;
    let body_rows = (area.y + height)
        .saturating_sub(body_top)
        .saturating_sub(legend_rows);
    let total = lanes.len();
    // When lanes are cut, the last body row counts them (with one body row, it is the only row).
    let (shown, hidden) = if total > usize::from(body_rows) && body_rows >= 1 {
        (
            usize::from(body_rows - 1),
            total - usize::from(body_rows - 1),
        )
    } else {
        (usize::from(body_rows).min(total), 0)
    };
    let markers = group
        .map(|_| markers_by_lane(&lanes, links))
        .unwrap_or_default();
    for (index, lane) in lanes.iter().take(shown).enumerate() {
        let Some(row) = u16::try_from(index)
            .ok()
            .and_then(|i| body_top.checked_add(i))
        else {
            break;
        };
        buf.set_stringn(
            area.x,
            row,
            lane.label(),
            usize::from(label_width.saturating_sub(1)),
            Style::new(),
        );
        if let Some(scale) = &scale {
            draw_lane(scale, lane, &markers[index], lane_x, row, buf);
        }
    }
    if hidden > 0
        && let Some(row) = u16::try_from(shown)
            .ok()
            .and_then(|s| body_top.checked_add(s))
    {
        buf.set_stringn(
            area.x,
            row,
            format!("  ↓ {hidden} more rows"),
            usize::from(area.width),
            Style::new().dim(),
        );
    }
    if legend_rows >= 1 {
        let first = area.bottom() - legend_rows;
        buf.set_stringn(
            area.x,
            first,
            LEGEND_SPAN,
            usize::from(area.width),
            Style::new().dim(),
        );
        if legend_rows == 3 {
            for (offset, text) in [(1, LEGEND_MARKERS), (2, LEGEND_ROWS)] {
                buf.set_stringn(
                    area.x,
                    first + offset,
                    text,
                    usize::from(area.width),
                    Style::new().dim(),
                );
            }
        }
    }
}

/// `repo-b  timeline  2026-10-29 21:15:00 → 23:15:00 UTC (2h)  /home/dev/…/repo-b`.
fn title_line(
    known: bool,
    repo: &str,
    window: &TimeWindow,
    drawable: bool,
    area: Rect,
    row: u16,
    buf: &mut Buffer,
) {
    let width = usize::from(area.width);
    if !known {
        buf.set_stringn(
            area.x,
            row,
            format!("{repo}  not in this overview"),
            width,
            Style::new().bold(),
        );
        return;
    }
    let name = basename(repo);
    let range = if drawable {
        window_text(window)
    } else {
        format!("{}  (empty window)", window_text(window))
    };
    let (x, _) = buf.set_stringn(area.x, row, name, width, Style::new().bold());
    let used = usize::from(x - area.x);
    let rest = format!("  timeline  {range}  ");
    let (x, _) = buf.set_stringn(x, row, &rest, width.saturating_sub(used), Style::new());
    let used = usize::from(x - area.x);
    buf.set_stringn(x, row, repo, width.saturating_sub(used), Style::new().dim());
}

/// One session row of the timeline.
struct Lane<'a> {
    session: &'a SessionRow,
    /// Tree branch for a Codex child (` ├ ` / ` └ `, `~` instead of the last space when its
    /// placing hand-off is not exact); `None` for a top-level row.
    branch: Option<String>,
}

impl Lane<'_> {
    fn label(&self) -> String {
        let indent = match &self.branch {
            Some(branch) => format!("{branch} "),
            None => String::new(),
        };
        let harness = match self.session.harness {
            Harness::Claude => "claude",
            Harness::Codex => "codex",
        };
        format!(
            "{indent}{:<5} {harness:<6} {}",
            state_word(self.session.state),
            short_id(&self.session.id)
        )
    }
}

/// Short, fixed-width-friendly state words (at most 5 characters).
fn state_word(state: State) -> &'static str {
    match state {
        State::Waiting => "wait",
        State::Busy => "busy",
        State::Stale => "stale",
        State::Idle => "idle",
        State::Unknown => "unk",
        State::Finished => "done",
    }
}

/// Branch for a child row: ` └ ` when its placing hand-off (the latest from this parent, T11;
/// `handoffs` is ordered by `at`) is exact, ` └~` otherwise.
fn branch(child: &SessionRow, parent: &str, last: bool) -> String {
    let placing = child
        .handoffs
        .iter()
        .rev()
        .find(|link| link.parent == parent)
        .or_else(|| child.handoffs.last());
    let exact = placing.is_some_and(|link| link.r#match == Match::Exact);
    let tree = if last { '└' } else { '├' };
    format!(" {tree}{}", if exact { ' ' } else { '~' })
}

/// The group's sessions in overview order, merged rows expanded, children under their parent.
fn lanes(group: &multi_link_model::RepoGroup) -> Vec<Lane<'_>> {
    let mut out = Vec::new();
    for row in &group.rows {
        let sessions: &[SessionRow] = match row {
            Row::Session(session) => std::slice::from_ref(session),
            Row::Merged(merged) => &merged.rows,
        };
        for session in sessions {
            out.push(Lane {
                session,
                branch: None,
            });
            let last = session.children.len().saturating_sub(1);
            for (position, child) in session.children.iter().enumerate() {
                out.push(Lane {
                    session: child,
                    branch: Some(branch(child, &session.id, position == last)),
                });
            }
        }
    }
    out
}

/// For each lane, the links drawn on it: every link whose `parent` is that lane's session
/// (the first lane with that id, if two rows share one).
fn markers_by_lane<'a>(lanes: &[Lane<'_>], links: &'a LinksReport) -> Vec<Vec<&'a Link>> {
    let mut out: Vec<Vec<&Link>> = lanes.iter().map(|_| Vec::new()).collect();
    for link in &links.links {
        if let Some(index) = lanes.iter().position(|lane| lane.session.id == link.parent) {
            out[index].push(link);
        }
    }
    out
}

/// Draw one lane: the clipped session span, then hand-off markers on top.
fn draw_lane(scale: &Scale, lane: &Lane<'_>, links: &[&Link], x: u16, y: u16, buf: &mut Buffer) {
    let bar = match lane.session.harness {
        Harness::Claude => Style::new().fg(Color::Cyan),
        Harness::Codex => Style::new().fg(Color::Magenta),
    };
    if let (Some(from), Some(to)) = (
        epoch_ms(&lane.session.started_at),
        epoch_ms(&lane.session.last_activity),
    ) && let Some(span) = scale.span(from, to)
    {
        for col in span.first..=span.last {
            set_cell(buf, x + col, y, "━", bar);
        }
        if span.cut_before {
            set_cell(buf, x + span.first, y, "◂", bar);
        }
        if span.cut_after {
            set_cell(buf, x + span.last, y, "▸", bar);
        }
    }

    draw_markers(scale, links, x, y, buf);
}

/// Hand-off markers of one lane (rules in the module docs): first every cell's winning glyph,
/// then `?N` digits and `+` only into cells no marker holds.
fn draw_markers(scale: &Scale, links: &[&Link], x: u16, y: u16, buf: &mut Buffer) {
    let mut by_col: BTreeMap<u16, Vec<(i64, &Link)>> = BTreeMap::new();
    for link in links {
        if let Some(at) = epoch_ms(&link.at)
            && let Some(col) = scale.col(at)
        {
            by_col.entry(col).or_default().push((at, *link));
        }
    }
    let winners: Vec<(u16, &Link, usize)> = by_col
        .iter()
        .filter_map(|(col, group)| {
            let winner = group
                .iter()
                .max_by_key(|(at, link)| (rank(link.r#match), *at, link.invocation.as_str()))?;
            Some((*col, winner.1, group.len()))
        })
        .collect();
    let mut used = vec![false; usize::from(scale.cells)];
    for (col, link, _) in &winners {
        let (glyph, style) = marker(link);
        set_cell(buf, x + col, y, glyph, style);
        used[usize::from(*col)] = true;
    }
    let free = |used: &[bool], col: u16| usize::from(col) < used.len() && !used[usize::from(col)];
    for (col, link, count) in winners {
        let (_, style) = marker(link);
        let mut next = col.saturating_add(1);
        if link.r#match == Match::Ambiguous && !link.candidates.is_empty() {
            let digits = link.candidates.len().to_string();
            let len = u16::try_from(digits.len()).unwrap_or(u16::MAX);
            if (next..next.saturating_add(len)).all(|cell| free(&used, cell)) {
                for (cell, digit) in (next..).zip(digits.chars()) {
                    set_cell(buf, x + cell, y, &digit.to_string(), style);
                    used[usize::from(cell)] = true;
                }
                next = next.saturating_add(len);
            }
        }
        if count > 1 {
            let spot = if free(&used, next) {
                Some(next)
            } else {
                col.checked_sub(1).filter(|cell| free(&used, *cell))
            };
            if let Some(cell) = spot {
                set_cell(
                    buf,
                    x + cell,
                    y,
                    "+",
                    Style::new().add_modifier(Modifier::BOLD),
                );
                used[usize::from(cell)] = true;
            }
        }
    }
}

fn rank(r#match: Match) -> u8 {
    match r#match {
        Match::Exact => 0,
        Match::Unlinked => 1,
        Match::Heuristic => 2,
        Match::Ambiguous => 3,
    }
}

/// The one-cell glyph of a marker; an ambiguous link's candidate count is drawn after it.
fn marker(link: &Link) -> (&'static str, Style) {
    let bold = Style::new().add_modifier(Modifier::BOLD);
    match link.r#match {
        Match::Exact => ("●", bold.fg(Color::Green)),
        Match::Heuristic => ("○", bold.fg(Color::Yellow)),
        Match::Ambiguous => ("?", bold.fg(Color::Yellow)),
        Match::Unlinked => ("×", bold.fg(Color::DarkGray)),
    }
}

fn set_cell(buf: &mut Buffer, x: u16, y: u16, symbol: &str, style: Style) {
    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_symbol(symbol).set_style(style);
    }
}

/// Maps instants in a non-empty window onto `cells` lane columns.
struct Scale {
    start: i64,
    end: i64,
    cells: u16,
}

/// A span's visible columns and whether it continues past either window edge.
#[derive(Debug, PartialEq, Eq)]
struct SpanCols {
    first: u16,
    last: u16,
    cut_before: bool,
    cut_after: bool,
}

/// Tick steps in seconds, smallest first; days double after the last one.
const STEPS: [i64; 18] = [
    1, 2, 5, 10, 15, 30, 60, 120, 300, 600, 900, 1_800, 3_600, 7_200, 10_800, 21_600, 43_200,
    86_400,
];

impl Scale {
    /// `None` for an empty or inverted window, an unparsable bound, or no lane cells.
    fn new(window: &TimeWindow, cells: u16) -> Option<Self> {
        let start = epoch_ms(&window.start)?;
        let end = epoch_ms(&window.end)?;
        (end > start && cells > 0).then_some(Self { start, end, cells })
    }

    /// Column of instant `at`, or `None` outside the window. Cell `i` covers
    /// `[start + i·d/cells, start + (i+1)·d/cells)`; `end` itself is the last cell.
    fn col(&self, at: i64) -> Option<u16> {
        if at < self.start || at > self.end {
            return None;
        }
        let offset = i128::from(at) - i128::from(self.start);
        let length = i128::from(self.end) - i128::from(self.start);
        let col = offset * i128::from(self.cells) / length;
        Some(u16::try_from(col).unwrap_or(u16::MAX).min(self.cells - 1))
    }

    /// The span `from..=to` clipped to the window; `None` when it lies wholly outside.
    fn span(&self, from: i64, to: i64) -> Option<SpanCols> {
        let (from, to) = (from.min(to), from.max(to));
        if to < self.start || from > self.end {
            return None;
        }
        Some(SpanCols {
            first: self.col(from.max(self.start))?,
            last: self.col(to.min(self.end))?,
            cut_before: from < self.start,
            cut_after: to > self.end,
        })
    }

    /// The smallest step whose ticks are at least a label plus 3 cells apart.
    fn step_ms(&self) -> i64 {
        let length = i128::from(self.end) - i128::from(self.start);
        let fits = |step_s: i64| {
            let gap = i128::from(step_s) * 1_000 * i128::from(self.cells) / length;
            gap >= i128::from(label_width(step_s) + 3)
        };
        if let Some(step) = STEPS.iter().copied().find(|step| fits(*step)) {
            return step * 1_000;
        }
        let mut step = 2 * 86_400;
        while !fits(step) && step < i64::MAX / 4_000 {
            step *= 2;
        }
        step * 1_000
    }

    /// Tick columns and instants: every multiple of the step inside the window.
    fn ticks(&self) -> Vec<(u16, i64)> {
        let step = self.step_ms();
        let mut at = self.start.div_euclid(step) * step;
        if at < self.start {
            at = at.saturating_add(step);
        }
        let mut out = Vec::new();
        while at <= self.end {
            if let Some(col) = self.col(at) {
                out.push((col, at));
            }
            match at.checked_add(step) {
                Some(next) => at = next,
                None => break,
            }
        }
        out
    }

    /// Tick labels that fit whole in the lane without touching the previous one.
    fn tick_labels(&self) -> Vec<(u16, String)> {
        let step_s = self.step_ms() / 1_000;
        let mut out = Vec::new();
        let mut free_from = 0u16;
        for (col, at) in self.ticks() {
            let Some(label) = clock(at, step_s) else {
                continue;
            };
            let len = u16::try_from(label.chars().count()).unwrap_or(u16::MAX);
            if col >= free_from && col.saturating_add(len) <= self.cells {
                free_from = col.saturating_add(len + 1);
                out.push((col, label));
            }
        }
        out
    }
}

fn label_width(step_s: i64) -> u16 {
    if step_s < 60 { 8 } else { 5 }
}

/// Tick label: `MM-DD` for day steps, `HH:MM` for minute steps, `HH:MM:SS` below a minute.
fn clock(at_ms: i64, step_s: i64) -> Option<String> {
    let at = instant(at_ms)?;
    let text = if step_s >= 86_400 {
        at.format(format_description!("[month]-[day]"))
    } else if step_s >= 60 {
        at.format(format_description!("[hour]:[minute]"))
    } else {
        at.format(format_description!("[hour]:[minute]:[second]"))
    };
    text.ok()
}

/// `2026-10-29 21:15:00 → 23:15:00 UTC (2h)`; the end repeats the date when it differs.
fn window_text(window: &TimeWindow) -> String {
    let parts = |at: &Timestamp| {
        let at = epoch_ms(at).and_then(instant)?;
        let date = at
            .format(format_description!("[year]-[month]-[day]"))
            .ok()?;
        let time = at
            .format(format_description!("[hour]:[minute]:[second]"))
            .ok()?;
        Some((date, time))
    };
    let (Some((start_date, start_time)), Some((end_date, end_time))) =
        (parts(&window.start), parts(&window.end))
    else {
        return format!("{} → {}", window.start, window.end);
    };
    let end = if end_date == start_date {
        end_time
    } else {
        format!("{end_date} {end_time}")
    };
    let length = epoch_ms(&window.end)
        .zip(epoch_ms(&window.start))
        .map(|(end, start)| duration_text(end.saturating_sub(start)))
        .unwrap_or_default();
    format!("{start_date} {start_time} → {end} UTC ({length})")
}

/// `2h`, `1h 30m`, `4m`, `45s`, `3d 2h`; whole seconds, largest two units; negative reads `-…`.
fn duration_text(ms: i64) -> String {
    let sign = if ms < 0 { "-" } else { "" };
    let mut seconds = ms.unsigned_abs() / 1_000;
    let mut parts = Vec::new();
    for (unit, size) in [("d", 86_400), ("h", 3_600), ("m", 60), ("s", 1)] {
        if seconds >= size {
            parts.push(format!("{}{unit}", seconds / size));
            seconds %= size;
        }
    }
    if parts.is_empty() {
        return "0s".to_string();
    }
    parts.truncate(2);
    format!("{sign}{}", parts.join(" "))
}

fn instant(ms: i64) -> Option<OffsetDateTime> {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(ms) * 1_000_000).ok()
}

fn epoch_ms(at: &Timestamp) -> Option<i64> {
    OffsetDateTime::parse(&at.to_string(), &Rfc3339)
        .ok()
        .and_then(|at| i64::try_from(at.unix_timestamp_nanos() / 1_000_000).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> i64 {
        epoch_ms(&Timestamp::parse(text).unwrap()).unwrap()
    }

    fn scale(start: &str, end: &str, cells: u16) -> Scale {
        let window = TimeWindow::new(
            Timestamp::parse(start).unwrap(),
            Timestamp::parse(end).unwrap(),
        );
        Scale::new(&window, cells).expect("non-empty window")
    }

    #[test]
    fn columns_cover_the_window_and_end_is_the_last_cell() {
        let scale = scale("2026-10-29T22:00:00Z", "2026-10-29T22:10:00Z", 10);
        assert_eq!(scale.col(at("2026-10-29T22:00:00Z")), Some(0));
        assert_eq!(scale.col(at("2026-10-29T22:00:59.999Z")), Some(0));
        assert_eq!(scale.col(at("2026-10-29T22:01:00Z")), Some(1));
        assert_eq!(scale.col(at("2026-10-29T22:10:00Z")), Some(9));
        assert_eq!(scale.col(at("2026-10-29T21:59:59.999Z")), None);
        assert_eq!(scale.col(at("2026-10-29T22:10:00.001Z")), None);
    }

    #[test]
    fn spans_are_clipped_and_flag_cut_edges() {
        let scale = scale("2026-10-29T22:00:00Z", "2026-10-29T22:10:00Z", 10);
        let span = |from: &str, to: &str| scale.span(at(from), at(to));
        assert_eq!(
            span("2026-10-29T21:00:00Z", "2026-10-29T22:03:00Z"),
            Some(SpanCols {
                first: 0,
                last: 3,
                cut_before: true,
                cut_after: false
            })
        );
        assert_eq!(
            span("2026-10-29T22:05:00Z", "2026-10-29T23:00:00Z"),
            Some(SpanCols {
                first: 5,
                last: 9,
                cut_before: false,
                cut_after: true
            })
        );
        // Zero length: one cell, even on the window's end.
        assert_eq!(
            span("2026-10-29T22:10:00Z", "2026-10-29T22:10:00Z"),
            Some(SpanCols {
                first: 9,
                last: 9,
                cut_before: false,
                cut_after: false
            })
        );
        assert_eq!(span("2026-10-29T21:00:00Z", "2026-10-29T21:59:00Z"), None);
        assert_eq!(span("2026-10-29T22:11:00Z", "2026-10-29T22:12:00Z"), None);
    }

    #[test]
    fn empty_or_inverted_windows_have_no_scale() {
        let a = Timestamp::parse("2026-10-29T22:00:00Z").unwrap();
        let b = Timestamp::parse("2026-10-29T22:10:00Z").unwrap();
        assert!(Scale::new(&TimeWindow::new(a, a), 10).is_none());
        assert!(Scale::new(&TimeWindow::new(b, a), 10).is_none());
        assert!(Scale::new(&TimeWindow::new(a, b), 0).is_none());
    }

    #[test]
    fn tick_steps_leave_room_for_labels() {
        // 2 h over 73 cells: 10 min ticks would be 6 cells apart (< 5 + 3), so 15 min.
        let two_hours = scale("2026-10-29T21:15:00Z", "2026-10-29T23:15:00Z", 73);
        assert_eq!(two_hours.step_ms(), 900_000);
        let labels: Vec<String> = two_hours
            .tick_labels()
            .into_iter()
            .map(|(_, l)| l)
            .collect();
        assert_eq!(labels.first().map(String::as_str), Some("21:15"));
        // 23:15 sits in the last cell and has no room for its label.
        assert_eq!(labels.last().map(String::as_str), Some("23:00"));
        // Below a minute the labels carry seconds.
        let short = scale("2026-10-29T22:00:00Z", "2026-10-29T22:01:00Z", 73);
        assert!(short.step_ms() < 60_000);
        assert!(short.tick_labels()[0].1.len() == 8);
        // A decade still terminates with day-multiple steps.
        let long = scale("2016-10-29T22:00:00Z", "2026-10-29T22:00:00Z", 20);
        assert!(long.step_ms() >= 86_400_000);
        assert!(long.ticks().len() <= 20);
    }

    #[test]
    fn default_window_is_two_hours_before_now() {
        let now = Timestamp::parse("2026-10-29T23:15:00Z").unwrap();
        let window = TimeWindow::ending_at(now, DEFAULT_WINDOW);
        assert_eq!(
            window.start,
            Timestamp::parse("2026-10-29T21:15:00Z").unwrap()
        );
        assert_eq!(window.end, now);
        // An out-of-range start gives an empty window, not a panic.
        let huge = TimeWindow::ending_at(now, Duration::from_secs(u64::MAX));
        assert_eq!(huge.start, huge.end);
    }

    #[test]
    fn duration_and_window_text() {
        assert_eq!(duration_text(7_200_000), "2h");
        assert_eq!(duration_text(5_400_000), "1h 30m");
        assert_eq!(duration_text(240_000), "4m");
        assert_eq!(duration_text(45_500), "45s");
        assert_eq!(duration_text(0), "0s");
        assert_eq!(duration_text(-60_000), "-1m");
        assert_eq!(duration_text(90_061_000), "1d 1h");
        let window = TimeWindow::new(
            Timestamp::parse("2026-10-29T23:00:00Z").unwrap(),
            Timestamp::parse("2026-10-30T01:00:00Z").unwrap(),
        );
        assert_eq!(
            window_text(&window),
            "2026-10-29 23:00:00 → 2026-10-30 01:00:00 UTC (2h)"
        );
    }
}
