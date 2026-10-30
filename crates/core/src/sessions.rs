//! Session assembly without hand-off discovery or linking.

use std::{collections::BTreeMap, io, path::Path, time::Duration};

use multi_link_claude::SessionRecord;
use multi_link_codex::ReadFailure;
use multi_link_model::{Harness, Segment, Session, SessionsReport, StepSource, Timestamp};

use crate::{
    Liveness, ProcessProbe, RepoMode,
    state::{claude_state, codex_state, within_window},
};

pub struct Options<'a> {
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
    let claude = multi_link_claude::scan(root)?;
    let codex = multi_link_codex::discover(root)?;
    let mut records: BTreeMap<String, SessionRecord> = BTreeMap::new();
    for record in claude.records {
        let slot = records
            .entry(record.session_id.clone())
            .or_insert_with(|| record.clone());
        // Equal timestamps resolve deterministically to the ascending-first path.
        if record.updated_at > slot.updated_at {
            *slot = record;
        }
    }
    let mut sessions = Vec::new();
    let mut incomplete = Vec::new();
    for transcript in claude.transcripts {
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
        let (state, state_evidence) = record_state(options, record.as_ref());
        sessions.push(Session {
            id: transcript.id,
            harness: Harness::Claude,
            transcript: Some(transcript.transcript),
            record: record.map(|record| record.record),
            repo: repo(options, &cwd),
            cwd,
            branch: transcript.branch,
            started_at,
            last_activity,
            state,
            state_evidence,
            segments: segment(started_at, last_activity),
            subagents: transcript.subagents,
        });
    }
    for record in records.into_values() {
        let (state, state_evidence) = record_state(options, Some(&record));
        sessions.push(Session {
            id: record.session_id,
            harness: Harness::Claude,
            transcript: None,
            record: Some(record.record),
            repo: repo(options, &record.cwd),
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
            id: rollout.id,
            harness: Harness::Codex,
            transcript: Some(rollout.transcript),
            record: None,
            repo: repo(options, &rollout.cwd),
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
            || options
                .since
                .is_none_or(|window| within_window(options.now, session.last_activity, window))
    });
    sessions.sort_by(|a, b| (a.started_at, &a.id).cmp(&(b.started_at, &b.id)));
    let mut skipped = claude.skipped;
    skipped.extend(codex.skipped);
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

fn record_state(
    options: &Options<'_>,
    record: Option<&SessionRecord>,
) -> (multi_link_model::State, multi_link_model::StateEvidence) {
    let liveness = match record {
        Some(record) => options.probe.probe(record.pid, &record.proc_start),
        _ => Liveness::Unavailable,
    };
    claude_state(options.now, record, liveness)
}

fn repo(options: &Options<'_>, cwd: &str) -> String {
    options
        .repo_mode
        .map(Path::new(cwd))
        .repo
        .to_string_lossy()
        .into_owned()
}

fn segment(from: Timestamp, to: Timestamp) -> Vec<Segment> {
    vec![Segment {
        from,
        to,
        step: None,
        step_source: StepSource::Unknown,
    }]
}
