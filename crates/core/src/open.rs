//! FR18 open: what `o` / `f` (and `multi-link open`) may do with one session.
//!
//! `action` is a pure function of a `Session` (output-schema "Open", design T7):
//! it only decides; the CLI and TUI execute. `Resume` needs positive exit
//! evidence (`finished`, which only a working process probe produces), so a
//! session that may still be running is never resumed.

use std::{io, path::Path};

use multi_link_model::{Harness, Session, State};
use serde::Serialize;

use crate::{Options, Scanner};

/// One open decision, serialized exactly as `open --dry-run` prints it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum OpenAction {
    /// Spawn `program args…` in `cwd`, inherited stdio, no shell.
    Resume {
        program: String,
        args: Vec<String>,
        cwd: String,
    },
    /// `open <url>` (macOS); undocumented desktop-app behaviour.
    AppLink { url: String },
    /// Nothing runs; `command` is offered for the user to run themselves.
    Unavailable {
        reason: Reason,
        command: Option<String>,
        cwd: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    UnsafeId,
    CodexNotSupported,
    NoTranscript,
    ForkNeedsFinished,
    RunningWithoutAppLink,
    NotKnownFinished,
}

impl Reason {
    pub fn as_str(self) -> &'static str {
        match self {
            Reason::UnsafeId => "unsafe_id",
            Reason::CodexNotSupported => "codex_not_supported",
            Reason::NoTranscript => "no_transcript",
            Reason::ForkNeedsFinished => "fork_needs_finished",
            Reason::RunningWithoutAppLink => "running_without_app_link",
            Reason::NotKnownFinished => "not_known_finished",
        }
    }
}

/// Lowercase uuid, 8-4-4-4-12 hex. Ids reach argv and text, and a file stem
/// could start with `-`, so anything else is refused.
pub fn is_safe_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    bytes.len() == 36
        && bytes.iter().enumerate().all(|(index, byte)| match index {
            8 | 13 | 18 | 23 => *byte == b'-',
            _ => byte.is_ascii_digit() || (b'a'..=b'f').contains(byte),
        })
}

/// `^local_[A-Za-z0-9-]+$`: the only host ids put into an app link.
pub fn is_app_host_id(host: &str) -> bool {
    host.strip_prefix("local_").is_some_and(|rest| {
        !rest.is_empty()
            && rest
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    })
}

/// The rule table of output-schema "Open", checked top to bottom.
pub fn action(session: &Session, fork: bool) -> OpenAction {
    let id = &session.id;
    let unavailable = |reason, command: Option<String>| OpenAction::Unavailable {
        reason,
        command,
        cwd: session.cwd.clone(),
    };
    if !is_safe_id(id) {
        return unavailable(Reason::UnsafeId, None);
    }
    if session.harness == Harness::Codex {
        let verb = if fork { "fork" } else { "resume" };
        return unavailable(
            Reason::CodexNotSupported,
            Some(format!("codex {verb} {id}")),
        );
    }
    let claude_command = || {
        let mut command = format!("claude --resume {id}");
        if fork {
            command.push_str(" --fork-session");
        }
        Some(command)
    };
    match session.state {
        State::Finished if session.transcript.is_some() => {
            let mut args = vec!["--resume".to_owned(), id.clone()];
            if fork {
                args.push("--fork-session".to_owned());
            }
            OpenAction::Resume {
                program: "claude".to_owned(),
                args,
                cwd: session.cwd.clone(),
            }
        }
        State::Finished => unavailable(Reason::NoTranscript, claude_command()),
        State::Busy | State::Idle | State::Waiting | State::Stale => {
            match session.host_session_id.as_deref() {
                Some(host) if is_app_host_id(host) && !fork => OpenAction::AppLink {
                    url: format!("claude://code/continue?session={host}"),
                },
                Some(host) if is_app_host_id(host) => {
                    unavailable(Reason::ForkNeedsFinished, claude_command())
                }
                _ => unavailable(Reason::RunningWithoutAppLink, claude_command()),
            }
        }
        State::Unknown => unavailable(Reason::NotKnownFinished, claude_command()),
    }
}

/// Result of an id-prefix lookup.
#[derive(Debug)]
pub enum Lookup {
    Found(Box<Session>),
    NoMatch,
    /// Every matching session, in report order.
    Ambiguous(Vec<Session>),
}

/// Sessions whose full id starts with `prefix`; an empty prefix matches nothing.
pub fn lookup(sessions: Vec<Session>, prefix: &str) -> Lookup {
    if prefix.is_empty() {
        return Lookup::NoMatch;
    }
    let mut matches: Vec<Session> = sessions
        .into_iter()
        .filter(|session| session.id.starts_with(prefix))
        .collect();
    match matches.len() {
        0 => Lookup::NoMatch,
        1 => Lookup::Found(Box::new(matches.remove(0))),
        _ => Lookup::Ambiguous(matches),
    }
}

/// A fresh pass with the `--since` window off (an old finished session is
/// exactly the one to resume), then a prefix lookup on full ids.
pub fn find(
    scanner: &mut Scanner,
    root: &Path,
    options: &Options<'_>,
    prefix: &str,
) -> io::Result<Lookup> {
    let unwindowed = Options {
        state_dir: options.state_dir.clone(),
        now: options.now,
        since: None,
        repo_mode: options.repo_mode,
        probe: options.probe,
    };
    let output = scanner.sessions(root, &unwindowed)?;
    Ok(lookup(output.report.sessions, prefix))
}

/// For the TUI right before it executes: recompute the action for one full id
/// from a fresh pass. `None` when the id is no longer found.
pub fn fresh_action(
    scanner: &mut Scanner,
    root: &Path,
    options: &Options<'_>,
    id: &str,
    fork: bool,
) -> io::Result<Option<OpenAction>> {
    Ok(match find(scanner, root, options, id)? {
        Lookup::Found(session) if session.id == id => Some(action(&session, fork)),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs,
        path::{Path, PathBuf},
        time::Duration,
    };

    use multi_link_model::{Harness, Session, State, StateEvidence, Timestamp};
    use serde_json::json;

    use super::*;
    use crate::{FakeProcessProbe, FixtureProcessProbe, Liveness, ProcessProbe, RepoMode};

    const ID: &str = "aaaaaaaa-0000-4000-8000-000000000001";
    const HOST: &str = "local_0e64648e-30ae-48f8-af8d-d3a5d67a5e5c";

    fn stamp(text: &str) -> Timestamp {
        Timestamp::parse(text).unwrap()
    }
    fn fixtures() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
    }
    fn options(probe: &dyn ProcessProbe) -> Options<'_> {
        Options {
            state_dir: PathBuf::new(),
            now: stamp("2026-10-29T23:15:00Z"),
            since: None,
            repo_mode: RepoMode::Fixture,
            probe,
        }
    }
    fn session(harness: Harness, state: State) -> Session {
        Session {
            id: ID.into(),
            harness,
            transcript: Some("t.jsonl".into()),
            record: None,
            host_session_id: None,
            cwd: "/repo".into(),
            repo: "/repo".into(),
            checkout: "/repo".into(),
            branch: None,
            started_at: stamp("2026-10-29T20:00:00Z"),
            last_activity: stamp("2026-10-29T20:00:00Z"),
            state,
            state_evidence: StateEvidence::NoSignal,
            segments: vec![],
            subagents: vec![],
            activity: Vec::new(),
            asked: None,
            last_asked: None,
            title: None,
            started_by: None,
            imported: false,
            state_since: None,
        }
    }
    fn reason(action: &OpenAction) -> Option<Reason> {
        match action {
            OpenAction::Unavailable { reason, .. } => Some(*reason),
            _ => None,
        }
    }
    fn write(root: &Path, path: &str, text: &str) {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    fn write_record(root: &Path, pid: u32, status: &str, updated_at: i64, host: Option<&str>) {
        let mut record = json!({
            "pid": pid, "procStart": "start", "sessionId": ID, "cwd": "/record-cwd",
            "status": status, "statusUpdatedAt": 1793315700000_i64,
            "startedAt": 1793304000000_i64, "updatedAt": updated_at,
        });
        if let Some(host) = host {
            record["hostSessionId"] = json!(host);
        }
        write(
            root,
            &format!(".claude/sessions/{pid}.json"),
            &record.to_string(),
        );
    }
    fn write_transcript(root: &Path) {
        write(
            root,
            &format!(".claude/projects/project/{ID}.jsonl"),
            &format!(
                "{}\n",
                json!({"type":"user", "cwd":"/transcript-cwd", "timestamp":"2026-10-29T19:00:00Z"})
            ),
        );
    }
    /// Action for ID in `root` with liveness per pid (missing pid → Unavailable).
    fn decide(root: &Path, liveness: &[(u32, Liveness)], fork: bool) -> (Session, OpenAction) {
        let mut probe = FakeProcessProbe::default();
        for (pid, live) in liveness {
            probe.results.insert((*pid, "start".into()), *live);
        }
        let session = match find(&mut Scanner::default(), root, &options(&probe), ID).unwrap() {
            Lookup::Found(session) => *session,
            other => panic!("{other:?}"),
        };
        let action = action(&session, fork);
        (session, action)
    }

    #[test]
    fn record_transcript_and_gone_process_resume_in_cwd() {
        let temp = tempfile::tempdir().unwrap();
        write_transcript(temp.path());
        write_record(temp.path(), 42, "busy", 1793315700000, Some(HOST));
        for (fork, args) in [
            (false, vec!["--resume", ID]),
            (true, vec!["--resume", ID, "--fork-session"]),
        ] {
            let (session, action) = decide(temp.path(), &[(42, Liveness::Gone)], fork);
            assert_eq!(session.state, State::Finished);
            assert_eq!(
                action,
                OpenAction::Resume {
                    program: "claude".into(),
                    args: args.into_iter().map(String::from).collect(),
                    cwd: "/transcript-cwd".into(),
                }
            );
        }
        // The fresh-pass helper the TUI uses agrees, and needs the full id.
        let mut probe = FakeProcessProbe::default();
        probe.results.insert((42, "start".into()), Liveness::Gone);
        let opts = options(&probe);
        let mut scanner = Scanner::default();
        assert!(matches!(
            fresh_action(&mut scanner, temp.path(), &opts, ID, false).unwrap(),
            Some(OpenAction::Resume { .. })
        ));
        assert_eq!(
            fresh_action(&mut scanner, temp.path(), &opts, &ID[..8], false).unwrap(),
            None
        );
    }

    #[test]
    fn record_only_gone_is_no_transcript() {
        let temp = tempfile::tempdir().unwrap();
        write_record(temp.path(), 42, "idle", 1793315700000, Some(HOST));
        for fork in [false, true] {
            let (session, action) = decide(temp.path(), &[(42, Liveness::Gone)], fork);
            assert_eq!((session.state, session.transcript), (State::Finished, None));
            let suffix = if fork { " --fork-session" } else { "" };
            assert_eq!(
                action,
                OpenAction::Unavailable {
                    reason: Reason::NoTranscript,
                    command: Some(format!("claude --resume {ID}{suffix}")),
                    cwd: "/record-cwd".into(),
                }
            );
        }
    }

    #[test]
    fn one_gone_and_one_alive_record_never_resumes() {
        let temp = tempfile::tempdir().unwrap();
        write_transcript(temp.path());
        // The gone record is newer; the alive one must still win.
        write_record(temp.path(), 42, "busy", 1793315700000, None);
        write_record(temp.path(), 43, "idle", 1793315600000, Some(HOST));
        for fork in [false, true] {
            let (session, action) = decide(
                temp.path(),
                &[(42, Liveness::Gone), (43, Liveness::Alive)],
                fork,
            );
            assert_ne!(session.state, State::Finished);
            assert!(!matches!(action, OpenAction::Resume { .. }), "{action:?}");
            if fork {
                assert_eq!(reason(&action), Some(Reason::ForkNeedsFinished));
            } else {
                assert_eq!(
                    action,
                    OpenAction::AppLink {
                        url: format!("claude://code/continue?session={HOST}")
                    }
                );
            }
        }
    }

    #[test]
    fn alive_record_with_and_without_host_id() {
        for host in [Some(HOST), None] {
            let temp = tempfile::tempdir().unwrap();
            write_transcript(temp.path());
            write_record(temp.path(), 42, "busy", 1793315700000, host);
            let (session, action) = decide(temp.path(), &[(42, Liveness::Alive)], false);
            assert_eq!(session.state, State::Busy);
            match host {
                Some(host) => assert_eq!(
                    action,
                    OpenAction::AppLink {
                        url: format!("claude://code/continue?session={host}")
                    }
                ),
                None => assert_eq!(
                    action,
                    OpenAction::Unavailable {
                        reason: Reason::RunningWithoutAppLink,
                        command: Some(format!("claude --resume {ID}")),
                        cwd: "/transcript-cwd".into(),
                    }
                ),
            }
        }
    }

    #[test]
    fn unavailable_probe_follows_record_status_and_never_resumes() {
        for (status, host, fork, expected) in [
            ("busy", Some(HOST), false, None),
            ("idle", Some(HOST), true, Some(Reason::ForkNeedsFinished)),
            ("waiting", None, false, Some(Reason::RunningWithoutAppLink)),
            ("other", Some(HOST), false, Some(Reason::NotKnownFinished)),
        ] {
            let temp = tempfile::tempdir().unwrap();
            write_transcript(temp.path());
            write_record(temp.path(), 42, status, 1793315700000, host);
            let (session, action) = decide(temp.path(), &[(42, Liveness::Unavailable)], fork);
            assert_ne!(session.state, State::Finished);
            assert!(!matches!(action, OpenAction::Resume { .. }), "{status}");
            assert_eq!(reason(&action), expected, "{status}");
        }
    }

    #[test]
    fn transcript_only_claude_and_completed_codex_offer_commands() {
        let temp = tempfile::tempdir().unwrap();
        write_transcript(temp.path());
        for fork in [false, true] {
            let (session, action) = decide(temp.path(), &[], fork);
            assert_eq!(session.state, State::Unknown);
            let suffix = if fork { " --fork-session" } else { "" };
            assert_eq!(
                action,
                OpenAction::Unavailable {
                    reason: Reason::NotKnownFinished,
                    command: Some(format!("claude --resume {ID}{suffix}")),
                    cwd: "/transcript-cwd".into(),
                }
            );
        }
        // Completed-turn Codex rollout from the fixtures, under a fake probe.
        let probe = FakeProcessProbe::default();
        let codex = "01a189e2-d57f-7a91-b74c-ae85abd1a6a1";
        let session = match find(
            &mut Scanner::default(),
            &fixtures().join("sessions"),
            &options(&probe),
            codex,
        )
        .unwrap()
        {
            Lookup::Found(session) => *session,
            other => panic!("{other:?}"),
        };
        assert_eq!(
            (session.state, session.state_evidence),
            (State::Unknown, StateEvidence::TurnsComplete)
        );
        for (fork, verb) in [(false, "resume"), (true, "fork")] {
            assert_eq!(
                action(&session, fork),
                OpenAction::Unavailable {
                    reason: Reason::CodexNotSupported,
                    command: Some(format!("codex {verb} {codex}")),
                    cwd: session.cwd.clone(),
                }
            );
        }
    }

    #[test]
    fn pure_rules_cover_unsafe_ids_codex_states_and_bad_host_ids() {
        // Unsafe ids: checked first, even for a finished session with a transcript.
        for id in [
            "-rf",
            "AAAAAAAA-0000-4000-8000-000000000001",
            "aaaaaaaa-0000-4000-8000-00000000000",
            "aaaaaaaa-0000-4000-8000-0000000000012",
            "aaaaaaaa_0000-4000-8000-000000000001",
            "gaaaaaaa-0000-4000-8000-000000000001",
            "aaaaaaaa-0000-4000-8000-00000000000 ",
            "",
        ] {
            let mut finished = session(Harness::Claude, State::Finished);
            finished.id = id.into();
            assert_eq!(
                action(&finished, false),
                OpenAction::Unavailable {
                    reason: Reason::UnsafeId,
                    command: None,
                    cwd: "/repo".into()
                },
                "{id:?}"
            );
        }
        // Codex in any state (even an impossible `finished`) never resumes.
        for state in [State::Busy, State::Stale, State::Finished, State::Unknown] {
            assert_eq!(
                reason(&action(&session(Harness::Codex, state), false)),
                Some(Reason::CodexNotSupported)
            );
        }
        // Stale and waiting records behave as running.
        for state in [State::Stale, State::Waiting, State::Idle, State::Busy] {
            let mut running = session(Harness::Claude, state);
            running.host_session_id = Some(HOST.into());
            assert!(matches!(
                action(&running, false),
                OpenAction::AppLink { .. }
            ));
            for bad in [
                "local_",
                "local_a b",
                "local_a;open",
                "LOCAL_abc",
                "abc",
                "local_é",
            ] {
                running.host_session_id = Some(bad.into());
                assert_eq!(
                    reason(&action(&running, false)),
                    Some(Reason::RunningWithoutAppLink),
                    "{bad:?}"
                );
            }
            running.host_session_id = Some(HOST.into());
        }
        assert!(is_safe_id(ID));
        assert!(is_app_host_id("local_Ab-9"));
    }

    #[test]
    fn json_shape_matches_the_schema() {
        let mut finished = session(Harness::Claude, State::Finished);
        assert_eq!(
            serde_json::to_value(action(&finished, true)).unwrap(),
            json!({"action":"resume", "program":"claude",
                   "args":["--resume", ID, "--fork-session"], "cwd":"/repo"})
        );
        finished.id = "-x".into();
        assert_eq!(
            serde_json::to_value(action(&finished, false)).unwrap(),
            json!({"action":"unavailable", "reason":"unsafe_id", "command":null, "cwd":"/repo"})
        );
        let mut running = session(Harness::Claude, State::Idle);
        running.host_session_id = Some(HOST.into());
        assert_eq!(
            serde_json::to_value(action(&running, false)).unwrap(),
            json!({"action":"app_link", "url": format!("claude://code/continue?session={HOST}")})
        );
        for reason in [
            Reason::UnsafeId,
            Reason::CodexNotSupported,
            Reason::NoTranscript,
            Reason::ForkNeedsFinished,
            Reason::RunningWithoutAppLink,
            Reason::NotKnownFinished,
        ] {
            assert_eq!(
                serde_json::to_value(reason).unwrap(),
                json!(reason.as_str())
            );
        }
    }

    fn copy_tree(source: &Path, target: &Path) {
        fs::create_dir_all(target).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            let to = target.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &to);
            } else {
                fs::copy(entry.path(), to).unwrap();
            }
        }
    }

    /// Output-schema "Open" → Fixture coverage, over every fixture session × {open, fork}.
    #[test]
    fn fixture_table_never_resumes_and_matches_schema_coverage() {
        const DESKTOP: [&str; 5] = ["6358ec7e", "1f47e9b3", "f8017fcd", "9d02a552", "bed52539"];
        const SYNTHETIC_BUSY: &str = "00000000-0000-4000-8000-000000000010";
        const HOOK_WAITING: [&str; 2] = ["d1f3b188", "11c1f13c"];
        let hook_root = tempfile::tempdir().unwrap();
        copy_tree(&fixtures().join("sessions"), hook_root.path());
        copy_tree(&fixtures().join("hook"), hook_root.path());
        fs::remove_file(hook_root.path().join("task-output/be01y5dv6.output")).unwrap();
        for (root, hook) in [
            (fixtures().join("sessions"), false),
            (hook_root.path().to_path_buf(), true),
        ] {
            let output = Scanner::default()
                .sessions(&root, &options(&FixtureProcessProbe))
                .unwrap();
            let sessions = output.report.sessions;
            assert_eq!(sessions.len(), if hook { 29 } else { 28 });
            let mut counts: BTreeMap<(bool, &str), usize> = BTreeMap::new();
            for session in &sessions {
                let short = &session.id[..8];
                // Fixture mode has no probe: upstream drift to `finished` must fail here.
                assert_ne!(session.state, State::Finished, "{}", session.id);
                for fork in [false, true] {
                    let action = action(session, fork);
                    assert!(
                        !matches!(action, OpenAction::Resume { .. }),
                        "{} fork={fork}",
                        session.id
                    );
                    let expected = if session.harness == Harness::Codex {
                        Some(Reason::CodexNotSupported)
                    } else if DESKTOP.contains(&short) {
                        fork.then_some(Reason::ForkNeedsFinished)
                    } else if session.id == SYNTHETIC_BUSY
                        || (hook && HOOK_WAITING.contains(&short))
                    {
                        Some(Reason::RunningWithoutAppLink)
                    } else {
                        Some(Reason::NotKnownFinished)
                    };
                    assert_eq!(reason(&action), expected, "{} fork={fork}", session.id);
                    if expected.is_none() {
                        assert_eq!(
                            action,
                            OpenAction::AppLink {
                                url: format!(
                                    "claude://code/continue?session={}",
                                    session.host_session_id.as_deref().unwrap()
                                )
                            }
                        );
                    }
                    let label = match &action {
                        OpenAction::Unavailable { reason, .. } => reason.as_str(),
                        _ => "app_link",
                    };
                    *counts.entry((fork, label)).or_default() += 1;
                }
            }
            // Every named fixture is present, so the table exercised each row.
            let waiting_rows = if hook { 2 } else { 0 };
            for fork in [false, true] {
                let (desktop_open, desktop_fork) = if fork { (0, 5) } else { (5, 0) };
                assert_eq!(
                    counts.get(&(fork, "app_link")).copied().unwrap_or(0),
                    desktop_open
                );
                assert_eq!(
                    counts
                        .get(&(fork, "fork_needs_finished"))
                        .copied()
                        .unwrap_or(0),
                    desktop_fork
                );
                assert_eq!(
                    counts[&(fork, "running_without_app_link")],
                    1 + waiting_rows
                );
                assert_eq!(counts[&(fork, "codex_not_supported")], 9);
                assert_eq!(counts[&(fork, "not_known_finished")], 13 - waiting_rows / 2);
            }
        }
    }

    #[test]
    fn lookup_finds_old_ids_and_reports_none_or_ambiguous() {
        let root = fixtures().join("sessions");
        let mut windowed = options(&FixtureProcessProbe);
        windowed.since = Some(Duration::from_secs(7200));
        // 0d7faed4 last wrote on 2026-10-27, far outside a 2 h window.
        let listed = Scanner::default().sessions(&root, &windowed).unwrap();
        assert!(
            !listed
                .report
                .sessions
                .iter()
                .any(|s| s.id.starts_with("0d7faed4"))
        );
        let mut scanner = Scanner::default();
        match find(&mut scanner, &root, &windowed, "0d7faed4").unwrap() {
            Lookup::Found(session) => {
                assert_eq!(session.id, "0d7faed4-bfd7-40bc-9d56-c3652efeb876")
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(
            find(&mut scanner, &root, &windowed, "ffffffff").unwrap(),
            Lookup::NoMatch
        ));
        assert!(matches!(
            find(&mut scanner, &root, &windowed, "").unwrap(),
            Lookup::NoMatch
        ));
        match find(&mut scanner, &root, &windowed, "01a189e4").unwrap() {
            Lookup::Ambiguous(sessions) => assert_eq!(sessions.len(), 4),
            other => panic!("{other:?}"),
        }
    }
}
