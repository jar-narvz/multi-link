//! Home screen (design "Screen 6"): every repo in the overview, needs-you first, one block each.

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

/// A selectable line of the home screen: one top-level row or one Codex child row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target<'a> {
    Session(&'a SessionRow),
    Merged(&'a MergedRow),
}

/// Every selectable line in screen order; `render_home`'s `selection` indexes this list.
/// Merged members are not listed: a `claude ×N` row is one line until expanded (T11).
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

/// Draw the home screen into `area`. `selection` is an index into [`selectable`]; an index past
/// the end highlights nothing. Content taller than the area is clipped; the body scrolls only as
/// far as needed to keep the selected line visible. Never panics, whatever the size.
pub fn render_home(
    report: &OverviewReport,
    selection: Option<usize>,
    area: Rect,
    buf: &mut Buffer,
) {
    render_home_with_footer(report, selection, true, area, buf);
}

pub(crate) fn render_home_with_footer(
    report: &OverviewReport,
    selection: Option<usize>,
    keys: bool,
    area: Rect,
    buf: &mut Buffer,
) {
    let area = area.intersection(buf.area);
    if area.is_empty() {
        return;
    }
    let lines = body_lines(report);

    // Title on top and keys at the bottom when there is room for a body line between them.
    let (title_rows, mut footer_rows) = match area.height {
        1 => (0, 0),
        2 => (0, 1),
        _ => (1, 1),
    };
    if !keys {
        footer_rows = 0;
    }
    if title_rows == 1 {
        let title = Line::from(vec![
            Span::from("multi-link").bold(),
            Span::from(format!(
                "  {}  now {}",
                plural(report.repos.len(), "repo", "repos"),
                report.now
            )),
        ]);
        buf.set_line(area.x, area.y, &title, area.width);
    }
    if footer_rows == 1 {
        let footer = Line::from(FOOTER).dim();
        buf.set_line(area.x, area.bottom() - 1, &footer, area.width);
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
        buf.set_line(area.x, y, &text, area.width);
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
}

/// The body in screen order. Walks the report exactly as `selectable` does, so the n-th
/// selectable line here is `selectable(report)[n]`.
fn body_lines(report: &OverviewReport) -> Vec<BodyLine> {
    let now = epoch_seconds(&report.now);
    let mut lines = Vec::new();
    let mut next_target = 0;
    let mut push = |lines: &mut Vec<BodyLine>, text: Line<'static>, selectable: bool| {
        let target = selectable.then(|| {
            next_target += 1;
            next_target - 1
        });
        lines.push(BodyLine { text, target });
    };

    for (index, group) in report.repos.iter().enumerate() {
        if index > 0 {
            push(&mut lines, Line::default(), false);
        }
        push(&mut lines, header_line(group), false);
        push(&mut lines, plan_line(group), false);
        for row in &group.rows {
            match row {
                Row::Session(session) => {
                    push(&mut lines, session_line(session, group, now, None), true);
                    let last = session.children.len().saturating_sub(1);
                    for (position, child) in session.children.iter().enumerate() {
                        let branch = if position == last { "└ " } else { "├ " };
                        push(
                            &mut lines,
                            session_line(child, group, now, Some((branch, &session.id))),
                            true,
                        );
                    }
                }
                Row::Merged(merged) => push(&mut lines, merged_line(merged, now), true),
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
            push(&mut lines, Line::from(text).yellow(), false);
        }
        if let Some(link) = &group.latest_handoff {
            push(&mut lines, handoff_line(link, now), false);
        }
    }
    lines
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

/// `claude ×N`: core already merged the quiet rows; show what they hold.
fn merged_line(merged: &MergedRow, now: Option<i64>) -> Line<'static> {
    let children: usize = merged.rows.iter().map(|row| row.children.len()).sum();
    let newest = merged.rows.iter().map(|row| row.last_activity).max();
    let newest = newest.map_or_else(
        || "-".to_string(),
        |at| age(now, &at).trim_start().to_string(),
    );
    Line::from(format!(
        "  ▸ claude ×{}  {} · newest {newest} · enter expands",
        merged.count,
        plural(children, "codex child", "codex children")
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
