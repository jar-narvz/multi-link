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
        });
    }
    for (record, liveness) in records.into_values() {
        let (state, state_evidence) = claude_state(options.now, Some(&record), liveness);
        sessions.push(Session {
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
