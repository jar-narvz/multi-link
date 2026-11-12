use std::{
    cell::Cell,
    fs,
    path::{Path, PathBuf},
};

use multi_link_core::{
    FixtureSnapshotProbe, Liveness, Options, ProcessProbe, ProcessSnapshot, RealProcessProbe,
    RepoMode, Scanner, SnapshotProbe,
    open::{OpenAction, Reason},
};
use multi_link_model::{Harness, State, Timestamp};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
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

struct Batch {
    calls: Cell<usize>,
    result: Liveness,
}
impl ProcessProbe for Batch {
    fn probe(&self, _: u32, _: &str) -> Liveness {
        panic!("must use the liveness batch")
    }
    fn batch(&self, requests: &[(u32, &str)]) -> Vec<Liveness> {
        self.calls.set(self.calls.get() + 1);
        vec![self.result; requests.len()]
    }
}
struct Snapshot {
    calls: Cell<usize>,
    result: ProcessSnapshot,
}
impl SnapshotProbe for Snapshot {
    fn snapshot(&self) -> ProcessSnapshot {
        self.calls.set(self.calls.get() + 1);
        self.result.clone()
    }
}
fn options(probe: &dyn ProcessProbe) -> Options<'_> {
    Options {
        state_dir: "state".into(),
        now: Timestamp::parse("2026-11-12T09:00:00Z").unwrap(),
        since: None,
        repo_mode: RepoMode::Fixture,
        probe,
    }
}

#[test]
fn one_batch_and_one_snapshot_per_resolver_build_not_per_action() {
    let batch = Batch {
        calls: Cell::new(0),
        result: Liveness::Unavailable,
    };
    let root = fixtures().join("sessions-phase18");
    let snapshot = Snapshot {
        calls: Cell::new(0),
        result: FixtureSnapshotProbe { root: root.clone() }.snapshot(),
    };
    let mut scanner = Scanner::default();
    let sessions = scanner
        .sessions(&root, &options(&batch))
        .unwrap()
        .report
        .sessions;
    for count in 1..=2 {
        let resolver = scanner
            .open_resolver_with_snapshot(&root, &snapshot)
            .unwrap();
        for session in &sessions {
            for fork in [false, true] {
                assert!(!matches!(
                    resolver.action(session, fork),
                    OpenAction::Resume { .. }
                ));
            }
        }
        let mut session = sessions
            .iter()
            .find(|s| s.id.starts_with("18000001"))
            .unwrap()
            .clone();
        for state in [State::Busy, State::Idle, State::Waiting, State::Stale] {
            session.state = state;
            assert!(matches!(
                resolver.action(&session, false),
                OpenAction::Jump { pid: 18001, .. }
            ));
        }
        session.host_session_id = Some("local_app".into());
        assert_eq!(
            resolver.action(&session, false),
            OpenAction::AppLink {
                url: "claude://code/continue?session=local_app".into(),
            }
        );
        session.harness = Harness::Codex;
        assert!(matches!(
            resolver.action(&session, false),
            OpenAction::Unavailable {
                reason: Reason::CodexNotSupported,
                ..
            }
        ));
        assert_eq!(snapshot.calls.get(), count);
        assert_eq!(batch.calls.get(), 1);
    }
}

#[test]
fn no_snapshot_for_finished_unknown_app_link_or_empty_reports() {
    for (status, host, live) in [
        ("idle", Some("local_app"), Liveness::Unavailable),
        ("new_status", None, Liveness::Unavailable),
        ("idle", None, Liveness::Gone),
    ] {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join(".claude/sessions");
        fs::create_dir_all(&dir).unwrap();
        let mut record: serde_json::Value = serde_json::from_slice(
            &fs::read(fixtures().join("sessions-phase18/.claude/sessions/18001.json")).unwrap(),
        )
        .unwrap();
        record["status"] = status.into();
        if let Some(host) = host {
            record["hostSessionId"] = host.into();
        }
        fs::write(dir.join("18001.json"), serde_json::to_vec(&record).unwrap()).unwrap();
        let batch = Batch {
            calls: Cell::new(0),
            result: live,
        };
        let snapshot = Snapshot {
            calls: Cell::new(0),
            result: ProcessSnapshot::Unknown,
        };
        let mut scanner = Scanner::default();
        let sessions = scanner
            .sessions(root.path(), &options(&batch))
            .unwrap()
            .report
            .sessions;
        assert_eq!(sessions.len(), 1);
        scanner
            .open_resolver_with_snapshot(root.path(), &snapshot)
            .unwrap();
        assert_eq!(snapshot.calls.get(), 0, "{status} {host:?} {live:?}");
        fs::remove_file(dir.join("18001.json")).unwrap();
        assert!(
            scanner
                .sessions(root.path(), &options(&batch))
                .unwrap()
                .report
                .sessions
                .is_empty()
        );
        scanner
            .open_resolver_with_snapshot(root.path(), &snapshot)
            .unwrap();
        assert_eq!(snapshot.calls.get(), 0);
    }
}

#[test]
fn missing_fixture_table_preserves_old_answer_and_read_errors_are_unknown() {
    let root = tempfile::tempdir().unwrap();
    copy_tree(&fixtures().join("sessions-phase18"), root.path());
    fs::remove_file(root.path().join("fixture-ps.txt")).unwrap();
    let batch = Batch {
        calls: Cell::new(0),
        result: Liveness::Unavailable,
    };
    let mut scanner = Scanner::default();
    let sessions = scanner
        .sessions(root.path(), &options(&batch))
        .unwrap()
        .report
        .sessions;
    let resolver = scanner.open_resolver(root.path()).unwrap();
    for session in &sessions {
        assert!(matches!(
            resolver.action(session, false),
            OpenAction::Unavailable {
                reason: Reason::RunningWithoutAppLink,
                ..
            }
        ));
    }
    fs::create_dir(root.path().join("fixture-ps.txt")).unwrap();
    let resolver = scanner.open_resolver(root.path()).unwrap();
    for session in &sessions {
        assert!(matches!(
            resolver.action(session, false),
            OpenAction::Unavailable {
                reason: Reason::ProcessUnknown,
                ..
            }
        ));
    }
    // Reusing a scanner on another root cannot borrow its last open context.
    let empty = tempfile::tempdir().unwrap();
    let snapshot = Snapshot {
        calls: Cell::new(0),
        result: ProcessSnapshot::Unknown,
    };
    scanner
        .open_resolver_with_snapshot(empty.path(), &snapshot)
        .unwrap();
    assert_eq!(snapshot.calls.get(), 0);
}

#[test]
fn real_snapshot_uses_exact_argv_c_locale_utc_and_one_process() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    let binary = root.path().join("ps");
    let calls = root.path().join("calls");
    fs::write(&binary, format!("#!/bin/sh\necho call >> '{}'\n[ \"$*\" = '-axo pid,ppid,lstart,tty,comm' ] || exit 2\n[ \"$LC_ALL\" = C ] || exit 2\n[ \"$TZ\" = UTC ] || exit 2\n[ -z \"${{LANG+x}}${{LC_TIME+x}}\" ] || exit 2\nprintf '%s\\n' 'PID PPID STARTED TTY COMM' '12 1 Mon Nov  9 05:30:00 2026 ttys004 /Applications/Visual Studio Code.app/Contents/MacOS/Code'\n", calls.display())).unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
    let ProcessSnapshot::Known(rows) = RealProcessProbe::new(binary).snapshot() else {
        panic!("snapshot unknown")
    };
    assert_eq!(
        rows[&12].comm,
        "/Applications/Visual Studio Code.app/Contents/MacOS/Code"
    );
    assert_eq!(rows[&12].started_at.to_string(), "2026-11-09 5:30:00.0");
    assert_eq!(fs::read_to_string(calls).unwrap(), "call\n");
    assert_eq!(
        RealProcessProbe::new(root.path().join("missing")).snapshot(),
        ProcessSnapshot::Unknown
    );
}
