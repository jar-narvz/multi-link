//! Home-only lanes. Core's report and timeline ordering remain unchanged.
use multi_link_model::{
    Harness, Link, Match, OverviewReport, RepoGroup, Row, SessionRow, State, Tier, Timestamp,
};
use std::collections::HashSet;
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
    if at.date() == now.date() {
        time
    } else {
        let month = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ][at.month() as usize - 1];
        format!("{month} {} {time}", at.day())
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
pub fn collapsed(group: &RepoGroup, expanded: &HashSet<String>) -> bool {
    group.tier == Tier::Other
        && group.concurrency.is_empty()
        && !expanded.contains(&group.repo)
        && !sessions(group).any(|r| rank(r.state) < 3)
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
#[derive(Clone, Copy, Debug)]
pub struct Lane<'a> {
    pub row: &'a SessionRow,
    pub child: Option<&'a SessionRow>,
    pub link: Option<&'a Link>,
}
impl Lane<'_> {
    pub fn state(&self) -> State {
        self.child.map_or(self.row.state, |c| {
            if rank(c.state) < rank(self.row.state) {
                c.state
            } else {
                self.row.state
            }
        })
    }
    pub fn step(&self) -> Option<&str> {
        if self.row.harness == Harness::Claude {
            self.row
                .made
                .first()
                .map(String::as_str)
                .or_else(|| self.link.and_then(|l| l.step.as_deref()))
        } else {
            self.link.and_then(|l| l.step.as_deref())
        }
    }
}
pub struct Lanes<'a> {
    pub visible: Vec<Lane<'a>>,
    pub hidden: usize,
}
pub fn lanes<'a>(
    report: &OverviewReport,
    group: &'a RepoGroup,
    split: &HashSet<String>,
    show_all: bool,
) -> Lanes<'a> {
    let mut all = Vec::new();
    for row in &group.rows {
        let rows = match row {
            Row::Session(r) => std::slice::from_ref(r),
            Row::Merged(r) => r.rows.as_slice(),
        };
        for r in rows {
            let child = r.children.first();
            let link = child.and_then(|c| placing(c, Some(&r.id)));
            let combine = r.harness == Harness::Claude
                && r.children.len() == 1
                && child.is_some_and(|c| c.harness == Harness::Codex)
                && !split.contains(&r.id)
                && link.is_some_and(|l| {
                    r.made
                        .first()
                        .zip(l.step.as_ref())
                        .is_none_or(|(a, b)| a == b)
                });
            all.push(Lane {
                row: r,
                child: if combine { child } else { None },
                link: if combine { link } else { placing(r, None) },
            });
            if !combine {
                all.extend(r.children.iter().map(|c| Lane {
                    row: c,
                    child: None,
                    link: placing(c, Some(&r.id)),
                }));
            }
        }
    }
    all.sort_by(|a, b| {
        let priority = |l: &Lane<'_>| {
            if rank(l.state()) < 3 {
                rank(l.state())
            } else {
                3
            }
        };
        (priority(a), a.row.started_at, &a.row.id).cmp(&(priority(b), b.row.started_at, &b.row.id))
    });
    let start = crate::TimeWindow::default_for(report).start;
    let mut quiet = 0;
    let total = all.len();
    all.retain(|l| {
        if show_all || rank(l.state()) < 3 {
            return true;
        }
        let in_window = |r: &SessionRow| r.last_activity >= start && r.last_activity <= report.now;
        if in_window(l.row) || l.child.is_some_and(in_window) {
            quiet += 1;
            quiet <= 8
        } else {
            false
        }
    });
    Lanes {
        hidden: total - all.len(),
        visible: all,
    }
}
