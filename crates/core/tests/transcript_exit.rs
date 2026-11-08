use std::{
    fs,
    path::{Path, PathBuf},
};

use multi_link_core::{
    FakeProcessProbe, FixtureProcessProbe, Liveness, Options, RepoMode, Scanner, open,
};
use multi_link_model::{State, StateEvidence, Timestamp};
use serde_json::Value;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

#[test]
fn phase9_clean_exit_states_and_fresh_open_match_keys_on_cold_and_warm_passes() {
    let fixtures = fixtures();
    let root = fixtures.join("sessions-phase9");
    let options = Options {
        state_dir: root.join(".multi-link"),
        now: Timestamp::parse("2026-11-09T12:00:00Z").unwrap(),
        since: None,
        repo_mode: RepoMode::Fixture,
        probe: &FixtureProcessProbe,
    };
    let key: Value =
        serde_json::from_slice(&fs::read(fixtures.join("expected/phase9/sessions.json")).unwrap())
            .unwrap();
    let opens: Value =
        serde_json::from_slice(&fs::read(fixtures.join("expected/phase9/open.json")).unwrap())
            .unwrap();
    let mut scanner = Scanner::default();
    for _ in 0..2 {
        let output = scanner.sessions(&root, &options).unwrap();
        let actual = serde_json::to_value(output.report).unwrap();
        for expected in key["sessions"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|s| s["id"].as_str().unwrap().starts_with("92"))
        {
            let id = expected["id"].as_str().unwrap();
            assert_eq!(
                actual["sessions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|s| s["id"] == id)
                    .unwrap(),
                expected,
                "{id}"
            );
            for (mode, fork) in [("open", false), ("fork", true)] {
                let action = open::fresh_action(&mut scanner, &root, &options, id, fork)
                    .unwrap()
                    .unwrap();
                assert_eq!(
                    serde_json::to_value(action).unwrap(),
                    opens[mode][id]["action"],
                    "{mode} {id}"
                );
            }
        }
    }
}

#[test]
fn sigkill_record_still_uses_process_exit_and_resumes() {
    let root = fixtures().join("sessions-phase9");
    let mut probe = FakeProcessProbe::default();
    let record = multi_link_claude::scan(&root)
        .unwrap()
        .records
        .into_iter()
        .find(|r| r.session_id.starts_with("92000009"))
        .unwrap();
    probe
        .results
        .insert((record.pid, record.proc_start), Liveness::Gone);
    let options = Options {
        state_dir: root.join(".multi-link"),
        now: Timestamp::parse("2026-11-09T12:00:00Z").unwrap(),
        since: None,
        repo_mode: RepoMode::Fixture,
        probe: &probe,
    };
    let mut scanner = Scanner::default();
    let open::Lookup::Found(session) =
        open::find(&mut scanner, &root, &options, "92000009").unwrap()
    else {
        panic!("missing session")
    };
    assert_eq!(
        (session.state, session.state_evidence),
        (State::Finished, StateEvidence::ProcessExit)
    );
    assert_eq!(
        serde_json::to_value(
            open::fresh_action(&mut scanner, &root, &options, &session.id, false)
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        serde_json::json!({"action":"resume", "program":"claude", "args":["--resume", session.id], "cwd":"/home/dev/repo-p9c"})
    );
}

#[test]
fn fresh_open_refreshes_entrypoint_and_requires_one_safe_app_mapping() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let id = "aaaaaaaa-0000-4000-8000-000000000001";
    let transcript = root.join(format!(".claude/projects/project/{id}.jsonl"));
    fs::create_dir_all(transcript.parent().unwrap()).unwrap();
    let write_transcript = |entrypoint: &str| {
        let user = serde_json::json!({"type":"user", "sessionId":id, "cwd":"/repo", "timestamp":"2026-11-09T04:00:00Z", "entrypoint":entrypoint, "message":{"content":"work"}});
        fs::write(
            &transcript,
            format!("{user}\n{{\"type\":\"cost-state\"}}\n"),
        )
        .unwrap();
    };
    let apps = root.join("Library/Application Support/Claude/claude-code-sessions/a/b");
    fs::create_dir_all(&apps).unwrap();
    let map = |name: &str, host: &str| {
        let value = serde_json::json!({"cliSessionId":id,"sessionId":host});
        fs::write(apps.join(name), value.to_string()).unwrap();
    };
    let options = Options {
        state_dir: root.join(".multi-link"),
        now: Timestamp::parse("2026-11-09T12:00:00Z").unwrap(),
        since: None,
        repo_mode: RepoMode::Fixture,
        probe: &FixtureProcessProbe,
    };
    let mut scanner = Scanner::default();
    write_transcript("cli");
    assert!(matches!(
        open::fresh_action(&mut scanner, root, &options, id, false).unwrap(),
        Some(open::OpenAction::Resume { .. })
    ));
    write_transcript("claude-desktop");
    assert!(matches!(
        open::fresh_action(&mut scanner, root, &options, id, false).unwrap(),
        Some(open::OpenAction::Unavailable {
            reason: open::Reason::AppSession,
            ..
        })
    ));
    map("local_one.json", "local_valid");
    assert_eq!(
        open::fresh_action(&mut scanner, root, &options, id, false).unwrap(),
        Some(open::OpenAction::AppLink {
            url: "claude://code/continue?session=local_valid".into()
        })
    );
    // Even duplicate mappings to the same host are ambiguous (two records).
    map("local_two.json", "local_valid");
    assert!(matches!(
        open::fresh_action(&mut scanner, root, &options, id, false).unwrap(),
        Some(open::OpenAction::Unavailable {
            reason: open::Reason::AppSession,
            ..
        })
    ));
    fs::remove_file(apps.join("local_two.json")).unwrap();
    map("local_one.json", "local_unsafe?query");
    assert!(matches!(
        open::fresh_action(&mut scanner, root, &options, id, false).unwrap(),
        Some(open::OpenAction::Unavailable {
            reason: open::Reason::AppSession,
            ..
        })
    ));
    write_transcript("future-entrypoint");
    assert!(matches!(
        open::fresh_action(&mut scanner, root, &options, id, false).unwrap(),
        Some(open::OpenAction::Unavailable {
            reason: open::Reason::UnknownEntrypoint,
            ..
        })
    ));
}
