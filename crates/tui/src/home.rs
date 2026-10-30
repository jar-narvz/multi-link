//! Home screen (design "Screen 6"): every repo in the overview, needs-you first, one block each.

use std::collections::HashSet;

use crate::timeline::TimeWindow;

use multi_link_model::{
    Convention, Harness, Link, Match, MergedRow, OverviewReport, RepoGroup, Row, SessionRow, State,
    Tier, Timestamp, VerifyResult,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

/// Keys 3.7 provides (PLAN Step 3.7, T11): `enter` expands a merged row, else opens the timeline.
const FOOTER: &str = "↑↓ select · enter expand/timeline · t timeline · o open · f fork · q quit";

/// Session states in the T5 order, for the per-repo counts.
const STATE_ORDER: [State; 6] = [
    State::Waiting,
    State::Busy,
    State::Stale,
    State::Idle,
    State::Unknown,
    State::Finished,
];

/// A selectable session, merged row, or collapsed repo summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target<'a> {
    Session(&'a SessionRow),
    Merged(&'a MergedRow),
    Repo(&'a RepoGroup),
}

/// Every row in screen order, ignoring repo collapse (also used by the timeline).
/// Merged members are not listed until expanded (T11).
pub fn selectable(report: &OverviewReport) -> Vec<(&str, Target<'_>)> {
    let mut targets = Vec::new();
    for group in &report.repos {
        for row in &group.rows {
            match row {
                Row::Session(session) => {
                    targets.push((group.repo.as_str(), Target::Session(session)));
                    for child in &session.children {
                        targets.push((group.repo.as_str(), Target::Session(child)));
                    }
                }
                Row::Merged(merged) => targets.push((group.repo.as_str(), Target::Merged(merged))),
            }
        }
    }
    targets
}

/// Presentation state for home; absent link data is shown as unavailable, never inferred.
#[derive(Default)]
pub struct HomeOptions<'a> {
    pub expanded_repos: HashSet<String>,
    pub links: Option<&'a multi_link_model::LinksReport>,
    pub hide_footer: bool,
}

fn collapsed(group: &RepoGroup, expanded: &HashSet<String>) -> bool {
    group.tier == Tier::Other
        && !expanded.contains(&group.repo)
        && !group_sessions(group).any(|row| row.state == State::Stale)
}

/// Visible targets only; a collapsed repo takes the place of all its rows.
pub fn selectable_home<'a>(
    report: &'a OverviewReport,
    expanded: &HashSet<String>,
) -> Vec<(&'a str, Target<'a>)> {
    let mut targets = Vec::new();
    for group in &report.repos {
        if collapsed(group, expanded) {
            targets.push((group.repo.as_str(), Target::Repo(group)));
        } else {
            for row in &group.rows {
                match row {
                    Row::Session(row) => {
                        targets.push((group.repo.as_str(), Target::Session(row)));
                        targets.extend(
                            row.children
                                .iter()
                                .map(|child| (group.repo.as_str(), Target::Session(child))),
                        );
                    }
                    Row::Merged(row) => targets.push((group.repo.as_str(), Target::Merged(row))),
                }
            }
        }
    }
    targets
}

/// Draw home with quiet repos collapsed. `selection` indexes [`selectable_home`] with an empty
/// expanded set; an index past the end highlights nothing. Content taller than the area is clipped;
/// the body scrolls only as
/// far as needed to keep the selected line visible. Never panics, whatever the size.
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
    options: &HomeOptions<'_>,
    area: Rect,
    buf: &mut Buffer,
) {
    let area = area.intersection(buf.area);
    if area.is_empty() {
        return;
    }
    let lines = body_lines(report, &options.expanded_repos);

    // Title on top and keys at the bottom when there is room for a body line between them.
    let (mut title_rows, mut footer_rows) = match area.height {
        1 => (0, 0),
        2 => (0, 1),
        _ => (1, 1),
    };
    if options.hide_footer {
        footer_rows = 0;
    }
    if title_rows == 1 {
        let title = summary_line(report, options.links, area.width);
        buf.set_line(area.x, area.y, &title, area.width);
    }
    if footer_rows == 1 {
        let footer = Line::from(FOOTER).dim();
        buf.set_line(area.x, area.bottom() - 1, &footer, area.width);
    }

    // A separate shared axis preserves U4's full summary at typical widths.
    let axis = MiniAxis::new(report, area.width).filter(|_| area.height >= 4);
    if let Some(axis) = &axis {
        axis.draw_header(area.x, area.y + 1, buf);
        title_rows += 1;
    }

    let body_top = area.y.saturating_add(title_rows);
    let body_height = usize::from(area.height - title_rows - footer_rows);
    let selected_line =
        selection.and_then(|index| lines.iter().position(|line| line.target == Some(index)));
    let window = scroll_window(lines.len(), body_height, selected_line);

    let visible = lines.iter().skip(window.offset).take(window.rows);
    for (row, line) in (0u16..).zip(visible) {
        // `area` lies inside the buffer, so these rows exist; checked math keeps a Rect at the
        // very bottom of the u16 range from overflowing.
        let Some(y) = body_top.checked_add(row) else {
            break;
        };
        let mut text = line.text.clone();
        if line.target.is_some() && line.target == selection {
            text = text.patch_style(Style::new().add_modifier(Modifier::REVERSED));
            // Fill the whole row so the highlight reads as a bar, not just the text.
            buf.set_style(Rect::new(area.x, y, area.width, 1), Modifier::REVERSED);
        }
        let text_width = if line.span.is_some() {
            axis.as_ref().map_or(area.width, |axis| axis.column - 1)
        } else {
            area.width
        };
        buf.set_line(area.x, y, &text, text_width);
        if let (Some(axis), Some((start, end))) = (&axis, line.span) {
            axis.draw_span(start, end, area.x, y, buf);
        }
    }
    if window.hidden_below > 0 {
        let more = Line::from(format!("  ↓ {} more lines", window.hidden_below)).dim();
        if let Some(y) = u16::try_from(window.rows)
            .ok()
            .and_then(|rows| body_top.checked_add(rows))
        {
            buf.set_line(area.x, y, &more, area.width);
        }
    }
}

/// A fixed right-hand column: every lane uses the same two-hour window, including children.
/// Endpoint labels are UTC; the track shows the window, not activity or busy time.
struct MiniAxis {
    column: u16,
    cells: u16,
    start: i128,
    end: i128,
    labels: (String, String),
}

impl MiniAxis {
    fn new(report: &OverviewReport, width: u16) -> Option<Self> {
        if width < 60 {
            return None;
        }
        let window = TimeWindow::default_for(report);
        let parse = |at: Timestamp| OffsetDateTime::parse(&at.to_string(), &Rfc3339).ok();
        let start = parse(window.start)?;
        let end = parse(window.end)?;
        if start >= end {
            return None;
        }
        let cells = 24.min(width - 48);
        let clock = |at: OffsetDateTime| format!("{:02}:{:02}", at.hour(), at.minute());
        Some(Self {
            column: width - cells,
            cells,
            start: start.unix_timestamp_nanos() / 1_000_000,
            end: end.unix_timestamp_nanos() / 1_000_000,
            labels: (clock(start), clock(end)),
        })
    }

    fn draw_header(&self, x: u16, y: u16, buf: &mut Buffer) {
        buf.set_line(
            x,
            y,
            &Line::from("  session span · idle gaps included · UTC").dim(),
            self.column - 1,
        );
        let label = format!(
            "{:<width$}{}",
            self.labels.0,
            self.labels.1,
            width = usize::from(self.cells) - self.labels.1.len()
        );
        buf.set_line(x + self.column, y, &Line::from(label).dim(), self.cells);
    }

    fn draw_span(&self, start: Timestamp, end: Timestamp, x: u16, y: u16, buf: &mut Buffer) {
        let mut cells = vec!["·"; usize::from(self.cells)];
        let millis = |at: Timestamp| {
            OffsetDateTime::parse(&at.to_string(), &Rfc3339)
                .ok()
                .map(|at| at.unix_timestamp_nanos() / 1_000_000)
        };
        if let Some((from, to)) = millis(start).zip(millis(end)) {
            let (from, to) = (from.min(to), from.max(to));
            if to >= self.start && from <= self.end {
                let col = |at: i128| {
                    (((at.clamp(self.start, self.end) - self.start) * i128::from(self.cells)
                        / (self.end - self.start)) as usize)
                        .min(cells.len() - 1)
                };
                let (first, last) = (col(from), col(to));
                cells[first..=last].fill("━");
                if from < self.start {
                    cells[first] = "◂";
                }
                if to > self.end {
                    cells[last] = "▸";
                }
            }
        }
        // set_line patches the existing cell style, preserving the selected row's reversal.
        buf.set_line(
            x + self.column,
            y,
            &Line::from(cells.concat()).cyan(),
            self.cells,
        );
    }
}

/// Which body lines are drawn: `rows` lines from `offset`, then (when `hidden_below > 0`) one
/// `↓ N more lines` row in the last body row.
#[derive(Debug, PartialEq, Eq)]
struct Window {
    offset: usize,
    rows: usize,
    hidden_below: usize,
}

/// Scroll just far enough to keep the selected line visible. When lines remain below the
/// window, the last body row says how many, so the selection must fit in the rows above it.
/// A one-row body has no room for the marker and shows only the (selected) line.
fn scroll_window(total: usize, height: usize, selected: Option<usize>) -> Window {
    let offset_for = |rows: usize| selected.map_or(0, |line| (line + 1).saturating_sub(rows));
    let offset = offset_for(height);
    if total - offset.min(total) <= height || height <= 1 {
        return Window {
            offset,
            rows: height,
            hidden_below: 0,
        };
    }
    let rows = height - 1;
    let offset = offset_for(rows);
    Window {
        offset,
        rows,
        hidden_below: total - offset - rows,
    }
}

struct BodyLine {
    text: Line<'static>,
    /// Index into `selectable` when this line is a selectable row.
    target: Option<usize>,
    span: Option<(Timestamp, Timestamp)>,
}

/// The body in screen order; target indices match `selectable_home(report, expanded)`.
fn body_lines(report: &OverviewReport, expanded: &HashSet<String>) -> Vec<BodyLine> {
    let now = epoch_seconds(&report.now);
    let mut lines = Vec::new();
    let mut next_target = 0;
    let mut push = |lines: &mut Vec<BodyLine>, text: Line<'static>, selectable: bool, span| {
        let target = selectable.then(|| {
            next_target += 1;
            next_target - 1
        });
        lines.push(BodyLine { text, target, span });
    };

    for (index, group) in report.repos.iter().enumerate() {
        if index > 0 {
            push(&mut lines, Line::default(), false, None);
        }
        if collapsed(group, expanded) {
            let mut line = header_line(group);
            line.spans[0] = Span::from("▸ ").bold();
            // Put the step before the full path so Verify remains visible at typical widths.
            let path = line.spans.pop().unwrap();
            let plan = plan_line(group);
            line.spans
                .push(Span::from(plan.to_string().trim_start().to_string()));
            line.spans.push(Span::from(" · "));
            line.spans.push(path);
            push(&mut lines, line, true, None);
            continue;
        }
        push(&mut lines, header_line(group), false, None);
        push(&mut lines, plan_line(group), false, None);
        for row in &group.rows {
            match row {
                Row::Session(session) => {
                    push(
                        &mut lines,
                        session_line(session, group, now, None),
                        true,
                        Some((session.started_at, session.last_activity)),
                    );
                    let last = session.children.len().saturating_sub(1);
                    for (position, child) in session.children.iter().enumerate() {
                        let branch = if position == last { "└ " } else { "├ " };
                        push(
                            &mut lines,
                            session_line(child, group, now, Some((branch, &session.id))),
                            true,
                            Some((child.started_at, child.last_activity)),
                        );
                    }
                }
                Row::Merged(merged) => push(&mut lines, merged_line(merged, now), true, None),
            }
        }
        for warning in &group.concurrency {
            let ids: Vec<String> = warning.sessions.iter().map(|id| short_id(id)).collect();
            let text = format!(
                "  ! {} busy sessions in one folder {}: {}",
                warning.sessions.len(),
                warning.cwd,
                ids.join(", ")
            );
            push(&mut lines, Line::from(text).yellow(), false, None);
        }
        if let Some(link) = &group.latest_handoff {
            push(&mut lines, handoff_line(link, now), false, None);
        }
    }
    lines
}

fn summary_line(
    report: &OverviewReport,
    links: Option<&multi_link_model::LinksReport>,
    width: u16,
) -> Line<'static> {
    let mut sessions = 0;
    let mut counts = [0usize; 3];
    for row in report.repos.iter().flat_map(group_sessions) {
        sessions += 1;
        match row.state {
            State::Waiting => counts[0] += 1,
            State::Busy => counts[1] += 1,
            State::Stale => counts[2] += 1,
            _ => {}
        }
    }
    let warnings: usize = report
        .repos
        .iter()
        .map(|group| group.concurrency.len())
        .sum();
    let handoffs = links.map_or_else(
        || "? hand-offs".to_string(),
        |links| plural(links.links.len(), "hand-off", "hand-offs"),
    );
    let text = format!(
        "multi-link  {} waiting · {} busy · {} stale · {} · {} · {} · {handoffs}",
        counts[0],
        counts[1],
        counts[2],
        plural(warnings, "warning", "warnings"),
        plural(report.repos.len(), "repo", "repos"),
        plural(sessions, "session", "sessions"),
    );
    let with_now = Line::from(format!("{text}  now {}", report.now));
    if with_now.width() <= usize::from(width) {
        with_now
    } else {
        Line::from(text)
    }
}

/// `◆` waiting / `●` busy marker, repo name, session counts by state, full repo path.
fn header_line(group: &RepoGroup) -> Line<'static> {
    let marker = match group.tier {
        Tier::Waiting => "◆ ",
        Tier::Busy => "● ",
        Tier::Other => "  ",
    };
    let mut counts = [0usize; STATE_ORDER.len()];
    for session in group_sessions(group) {
        if let Some(slot) = STATE_ORDER.iter().position(|state| *state == session.state) {
            counts[slot] += 1;
        }
    }
    let counts: Vec<String> = STATE_ORDER
        .iter()
        .zip(counts)
        .filter(|(_, count)| *count > 0)
        .map(|(state, count)| format!("{count} {}", state_name(*state)))
        .collect();
    Line::from(vec![
        Span::from(marker).bold(),
        Span::from(basename(&group.repo).to_string()).bold(),
        Span::from(format!("  {}  ", counts.join(" · "))),
        Span::from(group.repo.clone()).dim(),
    ])
}

/// Latest logged PLAN.md step with its reported Verify, plus active hand-off steps.
fn plan_line(group: &RepoGroup) -> Line<'static> {
    let mut text = String::from("  steps  ");
    match (group.plan.convention, &group.plan.latest_step) {
        (Convention::Missing, _) => text.push_str("no PLAN.md"),
        (Convention::Unsupported, _) => text.push_str("PLAN.md in an unsupported format"),
        (Convention::Supported, None) => text.push_str("no step logged yet"),
        (Convention::Supported, Some(step)) => {
            text.push_str(&format!("latest {}", step.id));
            if let Some(title) = &step.title {
                text.push_str(&format!(" {title}"));
            }
            if let Some(entry) = &step.entry {
                let result = match entry.verify.result {
                    VerifyResult::Pass => "✓",
                    VerifyResult::Fail => "✗",
                    VerifyResult::Unknown => "?",
                };
                text.push_str(&format!(" · Verify {result} ({})", entry.date));
            }
        }
    }
    if !group.active_steps.is_empty() {
        text.push_str(&format!(" · active {}", group.active_steps.join(", ")));
    }
    Line::from(text)
}

/// One mini lane: state glyph, harness, id prefix, state, age, then checkout label, step tag
/// and (Codex) how it is linked. `child_of` is a child row's tree prefix and parent id.
fn session_line(
    session: &SessionRow,
    group: &RepoGroup,
    now: Option<i64>,
    child_of: Option<(&str, &str)>,
) -> Line<'static> {
    let indent = match child_of {
        Some((branch, _)) => format!("    {branch}"),
        None => "  ".to_string(),
    };
    let harness = match session.harness {
        Harness::Claude => "claude",
        Harness::Codex => "codex",
    };
    // The link this lane describes: under a parent, the placing hand-off (T11: the latest link
    // from that parent; `handoffs` is ordered by `at`, `invocation`); top-level, the latest one.
    let shown = match child_of {
        Some((_, parent)) => session
            .handoffs
            .iter()
            .rev()
            .find(|link| link.parent == parent),
        None => session.handoffs.last(),
    };
    let mut extras = Vec::new();
    if session.checkout != group.repo {
        extras.push(format!("@{}", basename(&session.checkout)));
    }
    if let Some(step) = shown.and_then(|link| link.step.as_ref()) {
        extras.push(format!("step {step}"));
    }
    if session.harness == Harness::Codex {
        match shown {
            // Not "no hand-off": an ambiguous candidate has hand-offs that name no child.
            None => extras.push("no linked hand-off".to_string()),
            Some(link) => {
                extras.push(match_name(link.r#match).to_string());
                if session.handoffs.len() > 1 {
                    extras.push(format!("{} hand-offs", session.handoffs.len()));
                }
            }
        }
    }
    let state = Span::from(format!("{:<8}", state_name(session.state)));
    let state = match session.state {
        State::Waiting => state.yellow().bold(),
        State::Busy => state.green(),
        _ => state,
    };
    Line::from(vec![
        Span::from(format!(
            "{indent}{} {harness:<6} {}  ",
            glyph(session.state),
            short_id(&session.id)
        )),
        state,
        Span::from(format!(" {:<10}", age(now, &session.last_activity))),
        Span::from(format!("  {}", extras.join(" · "))).dim(),
    ])
}

/// Core already merged the quiet rows by harness; show what they hold.
fn merged_line(merged: &MergedRow, now: Option<i64>) -> Line<'static> {
    let children: usize = merged.rows.iter().map(|row| row.children.len()).sum();
    let newest = merged.rows.iter().map(|row| row.last_activity).max();
    let newest = newest.map_or_else(
        || "-".to_string(),
        |at| age(now, &at).trim_start().to_string(),
    );
    let harness = match merged.harness {
        Harness::Claude => "claude",
        Harness::Codex => "codex",
    };
    let children = if merged.harness == Harness::Claude {
        format!("{} · ", plural(children, "codex child", "codex children"))
    } else {
        String::new()
    };
    Line::from(format!(
        "  ▸ {harness} ×{}  {children}newest {newest} · enter expands",
        merged.count,
    ))
}

/// Latest hand-off: parent → child (or why there is none), match, form, step, age.
fn handoff_line(link: &Link, now: Option<i64>) -> Line<'static> {
    let child = match (&link.child, link.r#match) {
        (Some(child), _) => short_id(child),
        (None, Match::Ambiguous) => format!("? ({} candidates)", link.candidates.len()),
        (None, _) => "-".to_string(),
    };
    let form = match link.form {
        multi_link_model::InvocationForm::Exec => "exec",
        multi_link_model::InvocationForm::Resume => "resume",
        multi_link_model::InvocationForm::Plugin => "plugin",
    };
    let mut text = format!(
        "  ↳ hand-off {} → {child}  {} {form}",
        short_id(&link.parent),
        match_name(link.r#match)
    );
    if link.background {
        text.push_str(" bg");
    }
    if let Some(step) = &link.step {
        text.push_str(&format!(" step {step}"));
    }
    text.push_str(&format!(" · {}", age(now, &link.at).trim_start()));
    Line::from(text).dim()
}

/// Every session shown in a group: rows, children, merged members and their children.
fn group_sessions(group: &RepoGroup) -> impl Iterator<Item = &SessionRow> {
    group
        .rows
        .iter()
        .flat_map(|row| match row {
            Row::Session(session) => std::slice::from_ref(session),
            Row::Merged(merged) => merged.rows.as_slice(),
        })
        .flat_map(|session| std::iter::once(session).chain(session.children.iter()))
}

fn glyph(state: State) -> &'static str {
    match state {
        State::Waiting => "◆",
        State::Busy => "●",
        State::Stale => "◌",
        State::Idle => "○",
        State::Unknown => "?",
        State::Finished => "✓",
    }
}

fn state_name(state: State) -> &'static str {
    match state {
        State::Waiting => "waiting",
        State::Busy => "busy",
        State::Stale => "stale",
        State::Idle => "idle",
        State::Unknown => "unknown",
        State::Finished => "finished",
    }
}

fn match_name(r#match: Match) -> &'static str {
    match r#match {
        Match::Exact => "exact",
        Match::Heuristic => "heuristic",
        Match::Ambiguous => "ambiguous",
        Match::Unlinked => "unlinked",
    }
}

/// Short id: the first 8 and last 4 characters (`01a189e4…3eb3`). Codex ids are UUIDv7, whose
/// first 8 hex digits are a timestamp shared by sessions started close together (four fixture
/// ids start `01a189e4`), so a plain 8-character prefix would show different sessions alike.
/// Char-safe; ids are ASCII in practice.
pub(crate) fn short_id(id: &str) -> String {
    let chars: Vec<char> = id.chars().collect();
    if chars.len() <= 13 {
        return id.to_string();
    }
    let head: String = chars[..8].iter().collect();
    let tail: String = chars[chars.len() - 4..].iter().collect();
    format!("{head}…{tail}")
}

fn plural(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

/// Last `/`-separated component of a recorded path.
pub(crate) fn basename(path: &str) -> &str {
    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(path)
}

fn epoch_seconds(at: &Timestamp) -> Option<i64> {
    OffsetDateTime::parse(&at.to_string(), &Rfc3339)
        .ok()
        .map(OffsetDateTime::unix_timestamp)
}

/// Compact age of `at` relative to the report's `now`: `45s ago`, `12m ago`, `5h ago`, `3d ago`.
/// A time after `now` (clock skew, or a record written after the fixed fixture clock) reads
/// `… ahead` rather than pretending it is `0s ago`.
fn age(now: Option<i64>, at: &Timestamp) -> String {
    let (Some(now), Some(at)) = (now, epoch_seconds(at)) else {
        return "? ago".to_string();
    };
    let seconds = now - at;
    let (amount, direction) = if seconds < 0 {
        (-seconds, "ahead")
    } else {
        (seconds, "ago")
    };
    let amount = match amount {
        0..60 => format!("{amount}s"),
        60..3_600 => format!("{}m", amount / 60),
        3_600..172_800 => format!("{}h", amount / 3_600),
        _ => format!("{}d", amount / 86_400),
    };
    format!("{amount:>4} {direction}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_id_and_basename_are_char_safe() {
        assert_eq!(short_id("ab"), "ab");
        assert_eq!(short_id("0123456789abc"), "0123456789abc");
        assert_eq!(
            short_id("01a189e4-d953-7b63-b9b5-088a72542eb3"),
            "01a189e4…2eb3"
        );
        assert_eq!(short_id("éééééééé-ü-ççççç"), "éééééééé…çççç");
        assert_eq!(basename("/home/dev/repo-a"), "repo-a");
        assert_eq!(basename("/home/dev/repo-a/"), "repo-a");
        assert_eq!(basename("repo"), "repo");
    }

    #[test]
    fn scroll_window_keeps_selection_above_the_marker() {
        let window = |offset, rows, hidden_below| Window {
            offset,
            rows,
            hidden_below,
        };
        // Everything fits: no marker.
        assert_eq!(scroll_window(5, 5, None), window(0, 5, 0));
        assert_eq!(scroll_window(5, 5, Some(4)), window(0, 5, 0));
        // Clipped: the last row is the marker; selection 3 fits in the 4 rows above it.
        assert_eq!(scroll_window(10, 5, None), window(0, 4, 6));
        assert_eq!(scroll_window(10, 5, Some(3)), window(0, 4, 6));
        assert_eq!(scroll_window(10, 5, Some(4)), window(1, 4, 5));
        // Selecting the last line shows the tail without a marker.
        assert_eq!(scroll_window(10, 5, Some(9)), window(5, 5, 0));
        // One-row and empty bodies never draw the marker.
        assert_eq!(scroll_window(10, 1, Some(7)), window(7, 1, 0));
        assert_eq!(scroll_window(10, 0, Some(7)), window(8, 0, 0));
        assert_eq!(scroll_window(0, 3, None), window(0, 3, 0));
    }

    #[test]
    fn age_buckets() {
        let now = Timestamp::parse("2026-10-29T23:15:00.000Z").unwrap();
        let at = |text: &str| Timestamp::parse(text).unwrap();
        let now_s = epoch_seconds(&now);
        assert_eq!(age(now_s, &at("2026-10-29T23:13:30.000Z")), "  1m ago");
        assert_eq!(age(now_s, &at("2026-10-29T23:14:15.000Z")), " 45s ago");
        assert_eq!(age(now_s, &at("2026-10-29T18:15:00.000Z")), "  5h ago");
        assert_eq!(age(now_s, &at("2026-10-26T23:15:00.000Z")), "  3d ago");
        // A time after `now` is shown as such, never as a negative or zero age.
        assert_eq!(age(now_s, &at("2026-10-30T06:08:00.000Z")), "  6h ahead");
    }
}
