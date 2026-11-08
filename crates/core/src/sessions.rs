//! Session assembly without hand-off discovery or linking.

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    io,
    path::{Path, PathBuf},
    time::Duration,
};

use multi_link_claude::{AppRecord, SessionRecord};
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
        let hook_ended = !records.contains_key(&transcript.id)
            && hooks.ended(&transcript.id, transcript.last_turn_at);
        let hook_waiting = !hook_ended
            && !records.contains_key(&transcript.id)
            && !transcript.clean_exit
            && waiting.contains(&transcript.transcript);
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
        let (state, state_evidence) = if hook_ended {
            (State::Finished, StateEvidence::SessionEnd)
        } else if hook_waiting {
            (State::Waiting, StateEvidence::Hook)
        } else {
            match &record {
                Some((record, liveness)) => claude_state(options.now, Some(record), *liveness),
                None if transcript.clean_exit => (State::Finished, StateEvidence::TranscriptExit),
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
            started_by: None,
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
            started_by: None,
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
            started_by: None,
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
    // S1: only transcript-backed Claude sessions can be either end of a started-by link,
    // judged before the window so a cycle with a member outside it still counts.
    let eligible: HashSet<String> = sessions
        .iter()
        .filter(|session| session.harness == Harness::Claude && session.transcript.is_some())
        .map(|session| session.id.clone())
        .collect();
    sessions.retain(|session| {
        session.record.is_some()
            || session.state == State::Waiting
            || options
                .since
                .is_none_or(|window| within_window(options.now, session.last_activity, window))
    });
    sessions.sort_by(|a, b| (a.started_at, &a.id).cmp(&(b.started_at, &b.id)));
    let listed: HashSet<&str> = sessions
        .iter()
        .filter(|session| session.harness == Harness::Claude && session.transcript.is_some())
        .map(|session| session.id.as_str())
        .collect();
    let eligible: HashSet<&str> = eligible.iter().map(String::as_str).collect();
    let started = started_by_links(&claude.app_records, &eligible, &listed);
    for session in &mut sessions {
        if session.harness == Harness::Claude {
            session.started_by = started.get(session.id.as_str()).cloned();
        }
    }
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

/// S1: which listed Claude session started which. Returns child transcript id to parent
/// transcript id. Uses only the desktop app's records; a link needs no guessing, so it is
/// always exact. Rules, in order:
/// - a record is eligible when it names a parent (`spawnedFrom.sessionId`) and is neither a
///   user fork nor a `btw` side question (those are the user's acts);
/// - one app id on two records makes the parent unresolvable, so children naming it get no
///   link;
/// - two eligible records of one child naming different parents give that child no link;
/// - both ends must be transcript-backed Claude sessions (`eligible`, whatever the window
///   shows; a record-only session is neither end) and differ (a self-link is dropped);
/// - every session on a cycle loses its edge (sessions pointing into a cycle keep theirs);
///   cycles are found on all eligible edges, then only links between `listed` sessions
///   (the window) are kept.
pub(crate) fn started_by_links(
    app_records: &[AppRecord],
    eligible: &HashSet<&str>,
    listed: &HashSet<&str>,
) -> HashMap<String, String> {
    let mut by_app_id: HashMap<&str, Vec<&AppRecord>> = HashMap::new();
    for record in app_records {
        by_app_id.entry(&record.app_id).or_default().push(record);
    }
    // The parent a record names, as a comparable key: the parent's transcript id when the
    // parent record is unique, else the raw app id (unresolvable).
    let resolve = |app_id: &str| -> Result<&str, ()> {
        match by_app_id.get(app_id).map(Vec::as_slice) {
            Some([only]) => Ok(only.cli_session_id.as_str()),
            _ => Err(()),
        }
    };
    let mut named: BTreeMap<&str, Vec<Result<&str, &str>>> = BTreeMap::new();
    for record in app_records {
        if record.user_fork || record.btw {
            continue;
        }
        let Some(parent) = record.parent_app_id.as_deref() else {
            continue;
        };
        named
            .entry(&record.cli_session_id)
            .or_default()
            .push(resolve(parent).map_err(|()| parent));
    }
    let mut edges: HashMap<&str, &str> = HashMap::new();
    for (child, mut parents) in named {
        parents.sort();
        parents.dedup();
        if let [Ok(parent)] = parents.as_slice()
            && parent != &child
            && eligible.contains(child)
            && eligible.contains(parent)
        {
            edges.insert(child, parent);
        }
    }
    // Each child has at most one parent, so a node is on a cycle exactly when walking up
    // from it returns to it.
    let on_cycle: HashSet<&str> = edges
        .keys()
        .copied()
        .filter(|start| {
            let mut at = *start;
            for _ in 0..edges.len() {
                match edges.get(at) {
                    Some(next) if next == start => return true,
                    Some(next) => at = next,
                    None => return false,
                }
            }
            false
        })
        .collect();
    // The window only decides which links are shown, never which edges form a cycle.
    edges
        .into_iter()
        .filter(|(child, parent)| {
            !on_cycle.contains(child) && listed.contains(child) && listed.contains(parent)
        })
        .map(|(child, parent)| (child.to_owned(), parent.to_owned()))
        .collect()
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

#[cfg(test)]
mod started_by_tests {
    use super::*;

    fn rec(app: &str, cli: &str, parent: Option<&str>) -> AppRecord {
        AppRecord {
            app_id: app.into(),
            cli_session_id: cli.into(),
            parent_app_id: parent.map(str::to_owned),
            user_fork: false,
            btw: false,
        }
    }

    fn links(records: &[AppRecord], listed: &[&str]) -> Vec<(String, String)> {
        links_in(records, listed, listed)
    }

    fn links_in(
        records: &[AppRecord],
        eligible: &[&str],
        listed: &[&str],
    ) -> Vec<(String, String)> {
        let eligible: HashSet<&str> = eligible.iter().copied().collect();
        let listed: HashSet<&str> = listed.iter().copied().collect();
        let mut out: Vec<_> = started_by_links(records, &eligible, &listed)
            .into_iter()
            .collect();
        out.sort();
        out
    }

    fn pair(a: &str, b: &str) -> (String, String) {
        (a.to_owned(), b.to_owned())
    }

    #[test]
    fn a_chain_links_every_edge() {
        let records = [
            rec("la", "a", None),
            rec("lb", "b", Some("la")),
            rec("lc", "c", Some("lb")),
        ];
        assert_eq!(
            links(&records, &["a", "b", "c"]),
            [pair("b", "a"), pair("c", "b")]
        );
    }

    #[test]
    fn unlisted_ends_self_links_and_flags_give_no_link() {
        let mut fork = rec("lf", "f", Some("la"));
        fork.user_fork = true;
        let mut side = rec("ls", "s", Some("la"));
        side.btw = true;
        let records = [
            rec("la", "a", None),
            rec("lb", "b", Some("la")),
            rec("lx", "x", Some("la")),
            rec("lself", "self", Some("lself")),
            fork,
            side,
        ];
        // `b` is not listed; `x` is listed but its parent is not.
        assert_eq!(
            links(&records, &["a", "x", "self", "f", "s"]),
            [pair("x", "a")]
        );
        assert_eq!(links(&records, &["x", "self", "f", "s"]), []);
    }

    #[test]
    fn conflicts_and_shared_app_ids_give_no_link() {
        let records = [
            rec("la", "a", None),
            rec("lb", "b", None),
            rec("ld", "d", Some("la")),
            rec("ld2", "d", Some("lb")),
            // Same app id on two records: a child naming it has no resolvable parent.
            rec("dup", "p", None),
            rec("dup", "q", None),
            rec("lv", "v", Some("dup")),
        ];
        assert_eq!(links(&records, &["a", "b", "d", "p", "q", "v"]), []);
    }

    #[test]
    fn two_records_naming_the_same_parent_agree() {
        let records = [
            rec("la", "a", None),
            rec("ld", "d", Some("la")),
            rec("ld2", "d", Some("la")),
        ];
        assert_eq!(links(&records, &["a", "d"]), [pair("d", "a")]);
    }

    #[test]
    fn a_cycle_loses_every_edge_but_a_tail_into_it_keeps_its_own() {
        let records = [
            rec("l1", "c1", Some("l2")),
            rec("l2", "c2", Some("l1")),
            rec("lt", "t", Some("l1")),
        ];
        assert_eq!(links(&records, &["c1", "c2", "t"]), [pair("t", "c1")]);
    }

    #[test]
    fn a_cycle_is_found_before_the_window_filter() {
        // c3 is outside the window but still a transcript-backed session: the 3-cycle
        // keeps c1 and c2 top level, as it does with every session listed.
        let records = [
            rec("l1", "c1", Some("l2")),
            rec("l2", "c2", Some("l3")),
            rec("l3", "c3", Some("l1")),
        ];
        let all = ["c1", "c2", "c3"];
        assert_eq!(links_in(&records, &all, &["c1", "c2"]), []);
        assert_eq!(links_in(&records, &all, &all), []);
    }

    #[test]
    fn a_session_with_no_transcript_is_neither_end_of_a_link() {
        let records = [rec("la", "a", None), rec("lb", "b", Some("la"))];
        // `b` is only a record: not eligible, so not linked even if it is listed.
        assert_eq!(links_in(&records, &["a"], &["a", "b"]), []);
        // `a` is only a record: a child of it gets no link.
        assert_eq!(links_in(&records, &["b"], &["a", "b"]), []);
    }
}
