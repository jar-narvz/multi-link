//! Session assembly without hand-off discovery or linking.

use std::{
    collections::BTreeMap,
    io,
    path::{Path, PathBuf},
    time::Duration,
};

use multi_link_claude::SessionRecord;
use multi_link_codex::ReadFailure;
use multi_link_model::{
    Harness, Segment, Session, SessionsReport, State, StateEvidence, StepSource, Timestamp,
};

use crate::{
    Liveness, ProcessProbe, RepoMode,
    state::{claude_state, codex_state, within_window},
};

pub struct Options<'a> {
    /// Real-mode state directory; fixture mode always uses root/.multi-link.
    pub state_dir: PathBuf,
    pub now: Timestamp,
    /// None lists all history; record-backed sessions are always visible.
    pub since: Option<Duration>,
    /// Controls git lookups only; independent of the process probe.
    pub repo_mode: RepoMode,
    /// Callers using a fixture root must pass FixtureProcessProbe to disable ps.
    /// Step 1.9 selects both this probe and RepoMode::Fixture from --root.
    pub probe: &'a dyn ProcessProbe,
}

#[derive(Debug)]
pub struct SessionsOutput {
    pub report: SessionsReport,
    pub bad_records: Vec<String>,
    /// Sorted root-relative paths of Claude transcripts missing required metadata.
    pub incomplete: Vec<String>,
    pub unreadable: Vec<ReadFailure>,
}

/// Read every transcript before applying the output-only history window.
pub fn sessions_report(root: &Path, options: &Options<'_>) -> io::Result<SessionsOutput> {
    crate::Scanner::default().sessions(root, options)
}

pub(crate) fn assemble(
    options: &Options<'_>,
    claude: multi_link_claude::ClaudeScan,
    codex: multi_link_codex::Discovery,
    hooks: crate::hook::Records,
    waiting: &std::collections::HashSet<String>,
) -> io::Result<SessionsOutput> {
    // Several records can share one sessionId (a crashed record left behind plus
    // a live resume). Pick one per id: see `choose_record`.
    let mut grouped: BTreeMap<String, Vec<SessionRecord>> = BTreeMap::new();
    for record in claude.records {
        grouped
            .entry(record.session_id.clone())
            .or_default()
            .push(record);
    }
    let mut records: BTreeMap<String, (SessionRecord, Liveness)> = grouped
        .into_iter()
        .map(|(id, group)| (id, choose_record(options, group)))
        .collect();
    let mut sessions = Vec::new();
    let mut incomplete = Vec::new();
    for transcript in claude.transcripts {
        let hook_waiting =
            !records.contains_key(&transcript.id) && waiting.contains(&transcript.transcript);
        let (Some(cwd), Some(started_at), Some(last_activity)) = (
            transcript.cwd,
            transcript.started_at,
            transcript.last_activity,
        ) else {
            // Required schema fields cannot be invented for partial transcripts.
            // Leave any matching record available as a record-only session.
            incomplete.push(transcript.transcript);
            continue;
        };
        let record = records.remove(&transcript.id);
        let (state, state_evidence) = if hook_waiting {
            (State::Waiting, StateEvidence::Hook)
        } else {
            match &record {
                Some((record, liveness)) => claude_state(options.now, Some(record), *liveness),
                None => claude_state(options.now, None, Liveness::Unavailable),
            }
        };
        let state_since = if hook_waiting {
            hooks
                .records
                .iter()
                .filter(|r| r.session_id == transcript.id)
                .max_by_key(|r| (r.at, r.line))
                .map(|r| r.at)
        } else {
            record
                .as_ref()
                .and_then(|(r, _)| record_state_since(state, state_evidence, r))
        };
        let (record, host_session_id) = match record {
            Some((record, _)) => (Some(record.record), record.host_session_id),
            None => (None, None),
        };
        sessions.push(Session {
            id: transcript.id,
            harness: Harness::Claude,
            transcript: Some(transcript.transcript),
            record,
            host_session_id,
            repo: cwd.clone(),
            checkout: cwd.clone(),
            cwd,
            branch: transcript.branch,
            started_at,
            last_activity,
            state,
            state_evidence,
            segments: segment(started_at, last_activity),
            subagents: transcript.subagents,
            activity: activity(transcript.activity_timestamps, options.now, options.since),
            asked: transcript.asked,
            last_asked: transcript.last_asked,
            title: transcript.title,
            imported: false,
            state_since,
        });
    }
    for (record, liveness) in records.into_values() {
        let (state, state_evidence) = claude_state(options.now, Some(&record), liveness);
        sessions.push(Session {
            state_since: record_state_since(state, state_evidence, &record),
            activity: Vec::new(),
            asked: None,
            last_asked: None,
            title: None,
            imported: false,
            id: record.session_id,
            harness: Harness::Claude,
            transcript: None,
            record: Some(record.record),
            host_session_id: record.host_session_id,
            repo: record.cwd.clone(),
            checkout: record.cwd.clone(),
            cwd: record.cwd,
            branch: None,
            started_at: record.started_at,
            last_activity: record.updated_at,
            state,
            state_evidence,
            segments: Vec::new(),
            subagents: Vec::new(),
        });
    }
    for rollout in codex.rollouts {
        let (state, state_evidence) = codex_state(options.now, &rollout);
        sessions.push(Session {
            activity: activity(rollout.activity_timestamps, options.now, options.since),
            asked: rollout.asked,
            last_asked: rollout.last_asked,
            title: None,
            imported: rollout.imported,
            state_since: if state == State::Unknown
                && state_evidence == StateEvidence::TurnsComplete
            {
                rollout.last_real_complete
            } else {
                None
            },
            id: rollout.id,
            harness: Harness::Codex,
            transcript: Some(rollout.transcript),
            record: None,
            host_session_id: None,
            repo: rollout.cwd.clone(),
            checkout: rollout.cwd.clone(),
            cwd: rollout.cwd,
            branch: rollout.branch,
            started_at: rollout.started_at,
            last_activity: rollout.last_activity,
            state,
            state_evidence,
            segments: segment(rollout.started_at, rollout.last_activity),
            subagents: Vec::new(),
        });
    }
    sessions.retain(|session| {
        session.record.is_some()
            || session.state == State::Waiting
            || options
                .since
                .is_none_or(|window| within_window(options.now, session.last_activity, window))
    });
    sessions.sort_by(|a, b| (a.started_at, &a.id).cmp(&(b.started_at, &b.id)));
    let mut skipped = claude.skipped;
    skipped.extend(codex.skipped);
    skipped.extend(hooks.skipped);
    skipped.sort_by(|a, b| (&a.file, a.line).cmp(&(&b.file, b.line)));
    incomplete.sort();
    Ok(SessionsOutput {
        report: SessionsReport {
            now: options.now,
            sessions,
            skipped,
        },
        bad_records: claude.bad_records,
        incomplete,
        unreadable: codex.unreadable,
    })
}

/// Output-schema Session `record`: the session is `finished` only when every
/// record is confirmed gone. Otherwise the record not confirmed gone with the
/// greatest `updatedAt` wins; when all are gone, the greatest `updatedAt`.
/// Ties go to the ascending-first file name.
fn choose_record(options: &Options<'_>, group: Vec<SessionRecord>) -> (SessionRecord, Liveness) {
    let mut probed: Vec<_> = group
        .into_iter()
        .map(|record| {
            let liveness = options.probe.probe(record.pid, &record.proc_start);
            (record, liveness)
        })
        .collect();
    let all_gone = probed.iter().all(|(_, live)| *live == Liveness::Gone);
    if !all_gone {
        probed.retain(|(_, live)| *live != Liveness::Gone);
    }
    probed
        .into_iter()
        .reduce(|best, next| {
            let later = next.0.updated_at > best.0.updated_at
                || (next.0.updated_at == best.0.updated_at && next.0.record < best.0.record);
            if later { next } else { best }
        })
        .expect("a record group is never empty")
}

fn segment(from: Timestamp, to: Timestamp) -> Vec<Segment> {
    vec![Segment {
        from,
        to,
        step: None,
        step_source: StepSource::Unknown,
    }]
}

fn record_state_since(
    state: State,
    evidence: StateEvidence,
    record: &SessionRecord,
) -> Option<Timestamp> {
    (matches!(state, State::Waiting | State::Stale) && evidence == StateEvidence::SessionRecord)
        .then_some(record.status_updated_at)
}

fn activity(
    mut timestamps: Vec<Timestamp>,
    now: Timestamp,
    since: Option<Duration>,
) -> Vec<multi_link_model::Activity> {
    timestamps.sort();
    timestamps.dedup();
    let mut bursts: Vec<multi_link_model::Activity> = Vec::new();
    for at in timestamps {
        if let Some(last) = bursts.last_mut()
            && at.epoch_ms() - last.to.epoch_ms() <= 300_000
        {
            last.to = at;
        } else {
            bursts.push(multi_link_model::Activity { from: at, to: at });
        }
    }
    let start = since.and_then(|d| {
        Timestamp::from_epoch_ms(
            now.epoch_ms()
                .saturating_sub(d.as_millis().min(i64::MAX as u128) as i64),
        )
        .ok()
    });
    bursts
        .into_iter()
        .filter_map(|mut burst| {
            if burst.from > now || start.is_some_and(|start| burst.to < start) {
                return None;
            }
            burst.to = burst.to.min(now);
            if let Some(start) = start {
                burst.from = burst.from.max(start);
            }
            Some(burst)
        })
        .collect()
}

#[cfg(test)]
mod phase6_tests {
    use super::*;
    use multi_link_model::Activity;

    fn t(seconds: i64) -> Timestamp {
        Timestamp::from_epoch_ms(seconds * 1000).unwrap()
    }

    #[test]
    fn burst_boundaries_sort_deduplicate_and_keep_singletons() {
        for gap in [299, 300, 301] {
            let result = activity(vec![t(gap), t(0), t(gap)], t(1000), None);
            let expected = if gap <= 300 {
                vec![Activity {
                    from: t(0),
                    to: t(gap),
                }]
            } else {
                vec![
                    Activity {
                        from: t(0),
                        to: t(0),
                    },
                    Activity {
                        from: t(gap),
                        to: t(gap),
                    },
                ]
            };
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn bursts_clip_both_edges_after_grouping_and_drop_outside() {
        let stamps = vec![t(0), t(400), t(600), t(800), t(1000), t(1400)];
        assert_eq!(
            activity(stamps, t(900), Some(Duration::from_secs(400))),
            vec![Activity {
                from: t(500),
                to: t(900)
            }]
        );
        assert_eq!(
            activity(vec![t(900), t(1000)], t(900), None),
            vec![Activity {
                from: t(900),
                to: t(900)
            }]
        );
    }
}
