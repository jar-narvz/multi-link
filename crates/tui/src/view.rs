//! Presentation rules derived from reports; core grouping and values stay unchanged.
use multi_link_model::{
    Harness, Link, Match, OverviewReport, RepoGroup, Row, SessionRow, State, Timestamp,
};
use time::{OffsetDateTime, UtcOffset, format_description::well_known::Rfc3339};

pub fn datetime(at: Timestamp) -> OffsetDateTime {
    OffsetDateTime::parse(&at.to_string(), &Rfc3339).expect("model timestamp")
}
pub fn local_offset(at: Timestamp) -> UtcOffset {
    UtcOffset::local_offset_at(datetime(at)).unwrap_or(UtcOffset::UTC)
}
pub fn clock(at: Timestamp, now: Timestamp, offset: UtcOffset) -> String {
    let at = datetime(at).to_offset(offset);
    let now = datetime(now).to_offset(offset);
    let time = format!("{:02}:{:02}", at.hour(), at.minute());
    let age = now - at;
    if (age >= time::Duration::ZERO && age < time::Duration::hours(24))
        || (age < time::Duration::ZERO && at.date() == now.date())
    {
        time
    } else {
        let month = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ][at.month() as usize - 1];
        format!("{month} {}", at.day())
    }
}
pub fn offset_label(offset: UtcOffset) -> String {
    let seconds = offset.whole_seconds();
    if seconds == 0 {
        return "UTC".into();
    }
    let minutes = seconds.abs() / 60;
    format!(
        "UTC{}{}{}",
        if seconds < 0 { "-" } else { "+" },
        minutes / 60,
        if minutes % 60 == 0 {
            String::new()
        } else {
            format!(":{:02}", minutes % 60)
        }
    )
}
pub fn rank(state: State) -> u8 {
    match state {
        State::Waiting => 0,
        State::Busy => 1,
        State::Stale => 2,
        State::Idle => 3,
        State::Unknown => 4,
        State::Finished => 5,
    }
}
pub fn sessions(group: &RepoGroup) -> impl Iterator<Item = &SessionRow> {
    group
        .rows
        .iter()
        .flat_map(|r| match r {
            Row::Session(r) => std::slice::from_ref(r),
            Row::Merged(r) => r.rows.as_slice(),
        })
        .flat_map(|r| std::iter::once(r).chain(&r.children))
}
pub fn placing<'a>(child: &'a SessionRow, parent: Option<&str>) -> Option<&'a Link> {
    child
        .handoffs
        .iter()
        .rev()
        .find(|l| parent.is_none_or(|p| p == l.parent))
}
pub fn link_words(link: &Link) -> String {
    match link.r#match {
        Match::Exact => "confirmed".into(),
        Match::Heuristic => "matched by time".into(),
        Match::Ambiguous => format!("{} possible runs", link.candidates.len()),
        Match::Unlinked => "no run found".into(),
    }
}
/// Home attention rows are individual sessions; imports never affect presentation counts.
pub fn attention(group: &RepoGroup) -> Vec<&SessionRow> {
    let mut rows: Vec<_> = sessions(group)
        .filter(|r| !r.imported && rank(r.state) < 3)
        .collect();
    rows.sort_by_key(|r| (rank(r.state), r.started_at, &r.id));
    rows
}

pub fn minutes(milliseconds: i64) -> i64 {
    (milliseconds / 60_000).max(1)
}

pub fn last_active(at: Timestamp, now: Timestamp, offset: UtcOffset) -> String {
    let age = now.epoch_ms() - at.epoch_ms();
    if age < 3_600_000 {
        format!("{} min ago", minutes(age))
    } else {
        clock(at, now, offset)
    }
}

pub fn status(row: &SessionRow, now: Timestamp, offset: UtcOffset) -> String {
    let since = || clock(row.state_since.unwrap_or(row.last_activity), now, offset);
    match row.state {
        State::Busy => row.activity.last().map_or_else(
            || "working now".into(),
            |burst| {
                format!(
                    "working now · for {} min",
                    minutes(now.epoch_ms() - burst.from.epoch_ms())
                )
            },
        ),
        State::Waiting => format!("waiting on you · since {}", since()),
        State::Stale => format!("stale since {}", since()),
        State::Unknown
            if row.harness == Harness::Codex
                && row.state_evidence == multi_link_model::StateEvidence::TurnsComplete =>
        {
            format!("done {}", since())
        }
        State::Idle => format!(
            "idle · last active {}",
            last_active(
                row.activity.last().map_or(row.last_activity, |b| b.to),
                now,
                offset
            )
        ),
        State::Finished => "finished".into(),
        State::Unknown => "state unknown".into(),
    }
}

pub fn session_label(row: &SessionRow) -> String {
    session_label_for(row, None)
}

/// A nested run uses its placing parent's link even when it has later hand-offs elsewhere.
pub fn home_label(group: &RepoGroup, row: &SessionRow) -> String {
    let parent = sessions(group).find(|r| r.children.iter().any(|c| c.id == row.id));
    session_label_for(row, parent.map(|r| r.id.as_str()))
}

fn session_label_for(row: &SessionRow, parent: Option<&str>) -> String {
    if let Some(title) = &row.title {
        return title.clone();
    }
    let step = if row.harness == Harness::Claude {
        row.made.first().map(String::as_str)
    } else {
        placing(row, parent).and_then(|l| l.step.as_deref())
    };
    if let Some(step) = step {
        format!("step {step}")
    } else if let Some(prompt) = row
        .last_asked
        .as_deref()
        .filter(|s| prompt_label(s))
        .or_else(|| row.asked.as_deref().filter(|s| prompt_label(s)))
    {
        format!("“{prompt}”")
    } else {
        format!(
            "{} · no prompt",
            if row.harness == Harness::Claude {
                "claude"
            } else {
                "codex"
            }
        )
    }
}

pub fn handed(link: &Link) -> String {
    link.step.as_ref().map_or_else(
        || "handed work to codex".into(),
        |step| format!("handed step {step} to codex"),
    )
}

pub fn outcome(link: &Link, report: &OverviewReport) -> String {
    match link.r#match {
        Match::Unlinked => "no codex run found".into(),
        Match::Ambiguous => format!("{} possible runs", link.candidates.len()),
        _ => {
            let child = report
                .repos
                .iter()
                .flat_map(sessions)
                .find(|r| Some(&r.id) == link.child.as_ref());
            match child {
                Some(r)
                    if r.state == State::Unknown
                        && r.state_evidence == multi_link_model::StateEvidence::TurnsComplete =>
                {
                    "codex done"
                }
                Some(r) if matches!(r.state, State::Busy | State::Stale) => "codex running",
                _ => "codex state unknown",
            }
            .into()
        }
    }
}

fn prompt_label(s: &str) -> bool {
    !s.trim().is_empty()
        && !s
            .trim_start()
            .starts_with("Background (from the session that started this one):")
}

pub fn place(row: &SessionRow) -> &str {
    row.place.as_deref().or(row.branch.as_deref()).unwrap_or("")
}
