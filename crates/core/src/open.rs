//! FR18 open: what `o` / `f` (and `multi-link open`) may do with one session.
//!
//! `action` uses a `Session` and internal reopening metadata (schema "Open", T7):
//! it only decides; the CLI and TUI execute. `Resume` needs positive exit
//! evidence (a gone process or a clean transcript exit), so a
//! session that may still be running is never resumed.

use std::{io, path::Path};

use multi_link_model::{Harness, Session, State, StateEvidence};
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
    /// Identify the existing terminal tab; execution is added in Step 18.2.
    Jump {
        app: TerminalApp,
        tty: String,
        pid: u32,
        cwd: String,
    },
    /// Nothing runs; `command` is offered for the user to run themselves.
    Unavailable {
        reason: Reason,
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
        command: Option<String>,
        cwd: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalApp {
    Terminal,
    Iterm2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    UnsafeId,
    CodexNotSupported,
    NoTranscript,
    ForkNeedsFinished,
    RunningWithoutAppLink,
    InsideMultiplexer,
    TerminalNotSupported,
    NoTty,
    ProcessChanged,
    AmbiguousProcess,
    ProcessUnknown,
    NotKnownFinished,
    AppSession,
    UnknownEntrypoint,
}

impl Reason {
    pub fn as_str(self) -> &'static str {
        match self {
            Reason::UnsafeId => "unsafe_id",
            Reason::CodexNotSupported => "codex_not_supported",
            Reason::NoTranscript => "no_transcript",
            Reason::ForkNeedsFinished => "fork_needs_finished",
            Reason::RunningWithoutAppLink => "running_without_app_link",
            Reason::InsideMultiplexer => "inside_multiplexer",
            Reason::TerminalNotSupported => "terminal_not_supported",
            Reason::NoTty => "no_tty",
            Reason::ProcessChanged => "process_changed",
            Reason::AmbiguousProcess => "ambiguous_process",
            Reason::ProcessUnknown => "process_unknown",
            Reason::NotKnownFinished => "not_known_finished",
            Reason::AppSession => "app_session",
            Reason::UnknownEntrypoint => "unknown_entrypoint",
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

/// Internal reopening metadata; deliberately kept out of the Session JSON schema.
#[derive(Debug, Default)]
pub struct Resolver {
    records: Vec<multi_link_claude::SessionRecord>,
    snapshot: crate::probe::ProcessSnapshot,
    entrypoints: std::collections::HashMap<String, String>,
    app_ids: std::collections::HashMap<String, Vec<String>>,
}

impl Resolver {
    pub(crate) fn new(scan: multi_link_claude::ClaudeScan) -> Self {
        Self {
            records: scan.records,
            snapshot: Default::default(),
            entrypoints: scan
                .transcripts
                .into_iter()
                .filter_map(|t| t.entrypoint.map(|entrypoint| (t.id, entrypoint)))
                .collect(),
            app_ids: scan.app_records.into_iter().fold(
                std::collections::HashMap::new(),
                |mut ids, record| {
                    ids.entry(record.cli_session_id)
                        .or_default()
                        .push(record.app_id);
                    ids
                },
            ),
        }
    }

    pub(crate) fn set_snapshot(&mut self, snapshot: crate::probe::ProcessSnapshot) {
        self.snapshot = snapshot;
    }

    pub(crate) fn needs_snapshot(session: &Session) -> bool {
        session.harness == Harness::Claude
            && matches!(
                session.state,
                State::Busy | State::Idle | State::Waiting | State::Stale
            )
            && !session
                .host_session_id
                .as_deref()
                .is_some_and(is_app_host_id)
    }

    fn jump(&self, session: &Session) -> OpenAction {
        use crate::probe::{ProcessSnapshot, process_start};
        let unavailable = |reason, detail| OpenAction::Unavailable {
            reason,
            detail,
            command: Some(format!("claude --resume {}", session.id)),
            cwd: session.cwd.clone(),
        };
        let rows = match &self.snapshot {
            ProcessSnapshot::Absent => return unavailable(Reason::RunningWithoutAppLink, None),
            ProcessSnapshot::Unknown => return unavailable(Reason::ProcessUnknown, None),
            ProcessSnapshot::Known(rows) => rows,
        };
        let live: Vec<_> = self
            .records
            .iter()
            .filter(|record| record.session_id == session.id)
            .filter_map(|record| {
                rows.get(&record.pid)
                    .filter(|row| process_start(&record.proc_start) == Some(row.started_at))
            })
            .collect();
        let own = match live.as_slice() {
            [] => return unavailable(Reason::ProcessChanged, None),
            [row] => *row,
            _ => return unavailable(Reason::AmbiguousProcess, None),
        };
        if matches!(own.tty.as_str(), "??" | "-") {
            return unavailable(Reason::NoTty, None);
        }
        let mut pid = own.pid;
        let mut seen = std::collections::HashSet::new();
        for _ in 0..64 {
            if pid <= 1 || !seen.insert(pid) {
                break;
            }
            let Some(row) = rows.get(&pid) else {
                break;
            };
            let basename = Path::new(&row.comm).file_name().and_then(|s| s.to_str());
            if let Some(name @ ("tmux" | "screen" | "zellij")) = basename {
                return unavailable(Reason::InsideMultiplexer, Some(name.to_owned()));
            }
            let app = if row.comm.contains("/Terminal.app/Contents/MacOS/Terminal") {
                Some(TerminalApp::Terminal)
            } else if row.comm.contains("/iTerm.app/Contents/MacOS/iTerm2") {
                Some(TerminalApp::Iterm2)
            } else {
                None
            };
            if let Some(app) = app {
                let tty = if own.tty.starts_with("/dev/") {
                    own.tty.clone()
                } else {
                    format!("/dev/{}", own.tty)
                };
                return OpenAction::Jump {
                    app,
                    tty,
                    pid: own.pid,
                    cwd: session.cwd.clone(),
                };
            }
            if let Some((prefix, _)) = row.comm.split_once(".app/") {
                let name = prefix.rsplit('/').next().unwrap_or(prefix);
                return unavailable(Reason::TerminalNotSupported, Some(name.to_owned()));
            }
            pid = row.ppid;
        }
        unavailable(Reason::TerminalNotSupported, None)
    }

    pub fn action(&self, session: &Session, fork: bool) -> OpenAction {
        resolved_action(session, fork, self)
    }
}

/// The rule table of output-schema "Open", checked top to bottom.
pub fn action(session: &Session, fork: bool) -> OpenAction {
    Resolver::default().action(session, fork)
}

fn resolved_action(session: &Session, fork: bool, resolver: &Resolver) -> OpenAction {
    let id = &session.id;
    let unavailable = |reason, command: Option<String>| OpenAction::Unavailable {
        reason,
        detail: None,
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
            if matches!(
                session.state_evidence,
                StateEvidence::TranscriptExit | StateEvidence::SessionEnd
            ) {
                match resolver.entrypoints.get(id).map(String::as_str) {
                    Some("cli" | "sdk-cli") => {}
                    Some("claude-desktop") => {
                        if !fork
                            && let Some(hosts) = resolver.app_ids.get(id)
                            && let [host] = hosts.as_slice()
                            && is_app_host_id(host)
                        {
                            return OpenAction::AppLink {
                                url: format!("claude://code/continue?session={host}"),
                            };
                        }
                        return unavailable(Reason::AppSession, claude_command());
                    }
                    _ => return unavailable(Reason::UnknownEntrypoint, claude_command()),
                }
            }
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
                _ if !fork => resolver.jump(session),
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
        Lookup::Found(session) if session.id == id => {
            Some(scanner.open_resolver(root)?.action(&session, fork))
        }
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
                    detail: None,
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
                        detail: None,
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
                    detail: None,
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
                    detail: None,
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
                    detail: None,
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
        const HOOK_WAITING: [&str; 1] = ["d1f3b188"];
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
            let resolver = Scanner::default().open_resolver(&root).unwrap();
            let sessions = output.report.sessions;
            assert_eq!(sessions.len(), if hook { 29 } else { 28 });
            let mut counts: BTreeMap<(bool, &str), usize> = BTreeMap::new();
            for session in &sessions {
                let short = &session.id[..8];
                // Transcript exit is positive evidence even without a process probe.
                if session.state == State::Finished {
                    assert_eq!(session.state_evidence, StateEvidence::TranscriptExit);
                }
                for fork in [false, true] {
                    let action = resolver.action(session, fork);
                    assert!(
                        !matches!(action, OpenAction::Resume { .. }),
                        "{} fork={fork}",
                        session.id
                    );
                    let expected = if session.harness == Harness::Codex {
                        Some(Reason::CodexNotSupported)
                    } else if session.state == State::Finished {
                        Some(Reason::AppSession)
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
            let waiting_rows = if hook { 1 } else { 0 };
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
                assert_eq!(counts[&(fork, "not_known_finished")], 1);
                assert_eq!(counts[&(fork, "app_session")], 12);
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

#[cfg(test)]
mod jump_tests {
    use super::*;
    use crate::probe::ProcessSnapshot;

    const START: &str = "Mon Nov 09 05:30:00 2026";
    const ID: &str = "18000001-0000-4000-8000-000018000001";
    const HEADER: &str = "PID PPID STARTED TTY COMM\n";
    const OWN: &str = "18001 500 Mon Nov  9 05:30:00 2026 ttys004 claude\n";
    const TERMINAL: &str = "500 1 Mon Nov  9 05:30:00 2026 ?? /System/Applications/Utilities/Terminal.app/Contents/MacOS/Terminal\n";

    fn setup() -> (Resolver, Session) {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sessions-phase18");
        let mut scan = multi_link_claude::scan(&root).unwrap();
        scan.records.retain(|r| r.session_id == ID);
        scan.records[0].proc_start = START.into();
        let resolver = Resolver::new(scan);
        let session: Session = serde_json::from_value(serde_json::json!({
            "id":ID,"harness":"claude","repo":"/repo","checkout":"/repo","cwd":"/repo",
            "branch":null,"started_at":"2026-11-09T05:30:00.000Z",
            "last_activity":"2026-11-12T09:00:00.000Z","state":"idle",
            "state_evidence":"session_record","transcript":null,"pid":18001,"host_session_id":null,
            "segments":[],"imported":false,
            "subagents":[],"activity":[],"state_since":null,"title":null
        }))
        .unwrap();
        (resolver, session)
    }

    #[test]
    fn every_whole_snapshot_failure_is_process_unknown_never_resume() {
        let valid = format!("{HEADER}{OWN}{TERMINAL}");
        let duplicate = format!("{valid}{OWN}");
        let malformed = format!("{valid}malformed\n");
        let truncated = format!("{valid}17 1 Mon Nov 9\n");
        for (success, stdout, stderr) in [
            (false, valid.as_bytes(), &b""[..]),
            (false, &b""[..], &b""[..]),
            (true, valid.as_bytes(), &b"ps: denied"[..]),
            (true, malformed.as_bytes(), &b""[..]),
            (true, truncated.as_bytes(), &b""[..]),
            (true, duplicate.as_bytes(), &b""[..]),
            (true, &b""[..], &b""[..]),
            (true, HEADER.as_bytes(), &b""[..]),
            (true, &b"\xff"[..], &b""[..]),
            (
                true,
                &b"PID PPID STARTED TTY COMM\n1 0 invalid date here now xx ?? comm\n"[..],
                &b""[..],
            ),
        ] {
            let (mut resolver, session) = setup();
            resolver.set_snapshot(ProcessSnapshot::from_output(success, stdout, stderr));
            assert_eq!(
                resolver.action(&session, false),
                OpenAction::Unavailable {
                    reason: Reason::ProcessUnknown,
                    detail: None,
                    command: Some(format!("claude --resume {ID}")),
                    cwd: "/repo".into(),
                },
                "{success} {stdout:?} {stderr:?}"
            );
        }
    }

    #[test]
    fn ancestor_failures_no_tty_and_multiplexers_never_resume() {
        for (own, ancestors, reason, detail) in [
            (OWN.to_owned(), "".to_owned(), Reason::TerminalNotSupported, None),
            (OWN.to_owned(), "500 18001 Mon Nov 9 05:30:00 2026 ?? shell\n".into(), Reason::TerminalNotSupported, None),
            (OWN.replace("18001 500", "18001 0"), "".into(), Reason::TerminalNotSupported, None),
            (OWN.replace("18001 500", "18001 1"), TERMINAL.into(), Reason::TerminalNotSupported, None),
            (OWN.replace("ttys004", "-"), TERMINAL.into(), Reason::NoTty, None),
            (OWN.replace("ttys004", "??"), TERMINAL.into(), Reason::NoTty, None),
            (OWN.into(), "500 1 Mon Nov 9 05:30:00 2026 ?? /usr/bin/screen\n".into(), Reason::InsideMultiplexer, Some("screen")),
            (OWN.into(), "500 1 Mon Nov 9 05:30:00 2026 ?? /usr/bin/zellij\n".into(), Reason::InsideMultiplexer, Some("zellij")),
            (OWN.into(), "500 1 Mon Nov 9 05:30:00 2026 ?? /Applications/Other.app/Contents/Helper.app/Contents/MacOS/x\n".into(), Reason::TerminalNotSupported, Some("Other")),
        ] {
            let (mut resolver, session) = setup();
            resolver.set_snapshot(ProcessSnapshot::from_output(true, format!("{HEADER}{own}{ancestors}").as_bytes(), b""));
            assert_eq!(resolver.action(&session, false), OpenAction::Unavailable {
                reason, detail: detail.map(str::to_owned),
                command: Some(format!("claude --resume {ID}")), cwd: "/repo".into(),
            });
        }
    }

    #[test]
    fn walk_stops_at_64_rows_and_uses_only_own_tty() {
        let (mut resolver, session) = setup();
        for (links, jump) in [(62, true), (63, false)] {
            let mut text = format!("{HEADER}{OWN}");
            for index in 0..links {
                text.push_str(&format!(
                    "{} {} Mon Nov 9 05:30:00 2026 ?? shell\n",
                    500 + index,
                    501 + index
                ));
            }
            text.push_str(&TERMINAL.replace("500 1", &format!("{} 1", 500 + links)));
            resolver.set_snapshot(ProcessSnapshot::from_output(true, text.as_bytes(), b""));
            if jump {
                assert_eq!(
                    resolver.action(&session, false),
                    OpenAction::Jump {
                        app: TerminalApp::Terminal,
                        tty: "/dev/ttys004".into(),
                        pid: 18001,
                        cwd: "/repo".into(),
                    }
                );
            } else {
                assert!(matches!(
                    resolver.action(&session, false),
                    OpenAction::Unavailable {
                        reason: Reason::TerminalNotSupported,
                        ..
                    }
                ));
            }
        }
    }

    #[test]
    fn zero_padded_record_day_matches_and_prefixed_tty_is_canonical() {
        let (mut resolver, session) = setup();
        resolver.set_snapshot(ProcessSnapshot::from_output(
            true,
            format!(
                "{HEADER}{}{TERMINAL}",
                OWN.replace("ttys004", "/dev/ttys004")
            )
            .as_bytes(),
            b"",
        ));
        assert_eq!(
            resolver.action(&session, false),
            OpenAction::Jump {
                app: TerminalApp::Terminal,
                tty: "/dev/ttys004".into(),
                pid: 18001,
                cwd: "/repo".into(),
            }
        );
    }
}
