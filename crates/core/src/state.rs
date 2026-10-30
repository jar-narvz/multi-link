//! Pure state decisions; callers supply both the clock and process evidence.

use std::time::Duration;

use multi_link_claude::SessionRecord;
use multi_link_codex::Rollout;
use multi_link_model::{State, StateEvidence, Timestamp};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::Liveness;

/// A missing record carries no exit evidence, even for an old transcript.
pub fn claude_state(
    now: Timestamp,
    record: Option<&SessionRecord>,
    liveness: Liveness,
) -> (State, StateEvidence) {
    let Some(record) = record else {
        return (State::Unknown, StateEvidence::NoSignal);
    };
    if liveness == Liveness::Gone {
        return (State::Finished, StateEvidence::ProcessExit);
    }
    let state = match record.status.as_str() {
        "busy" => activity_state(now, record.status_updated_at),
        "idle" => State::Idle,
        _ => State::Unknown,
    };
    (state, StateEvidence::SessionRecord)
}

pub fn codex_state(now: Timestamp, rollout: &Rollout) -> (State, StateEvidence) {
    match rollout.last_turn_complete() {
        None => (State::Unknown, StateEvidence::NoSignal),
        Some(true) => (State::Unknown, StateEvidence::TurnsComplete),
        Some(false) => (
            activity_state(now, rollout.last_activity),
            StateEvidence::OpenTurn,
        ),
    }
}

fn activity_state(now: Timestamp, activity: Timestamp) -> State {
    if within_window(now, activity, Duration::from_secs(300)) {
        State::Busy
    } else {
        State::Stale
    }
}

pub(crate) fn within_window(now: Timestamp, activity: Timestamp, window: Duration) -> bool {
    // Timestamp exposes ordering and formatting but not arithmetic. Its canonical
    // RFC3339 representation preserves all its millisecond precision.
    let instant = |stamp: Timestamp| {
        OffsetDateTime::parse(&stamp.to_string(), &Rfc3339)
            .expect("Timestamp always formats as RFC3339")
    };
    (instant(now) - instant(activity)).whole_nanoseconds() <= window.as_nanos() as i128
}
