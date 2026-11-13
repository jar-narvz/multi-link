use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

#[test]
fn menubar_matches_claudes_five_keys_byte_for_byte() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (name, fixture, now) in [
        ("multi-repo", "sessions-phase14", "2026-11-10T22:20:00Z"),
        ("waiting-quiet", "sessions-phase9", "2026-11-09T07:30:00Z"),
        ("open-actions", "sessions-phase18", "2026-11-12T09:00:00Z"),
        ("phase9-early", "sessions-phase9", "2026-11-09T06:00:00Z"),
        ("phase9-noon", "sessions-phase9", "2026-11-09T12:00:00Z"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_multi-link"))
            .current_dir(&root)
            .env("TZ", "UTC")
            .args([
                "menubar",
                "--json",
                "--root",
                &format!("fixtures/{fixture}"),
                "--now",
                now,
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            output.stdout,
            fs::read(root.join(format!("fixtures/expected/menubar/{name}.json"))).unwrap(),
            "{name}"
        );
    }
}

#[test]
fn menubar_reads_active_step_title_from_temporary_repo_plan() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    fs::create_dir(&repo).unwrap();
    fs::write(
        repo.join("PLAN.md"),
        "### Step 2.4 — Synthetic parser title\n",
    )
    .unwrap();
    let transcripts = temp.path().join(".claude/projects/synthetic");
    fs::create_dir_all(&transcripts).unwrap();
    let id = "aaaaaaaa-0000-4000-8000-000000000001";
    let record = json!({"type":"user", "sessionId":id, "cwd":repo, "timestamp":"2026-11-10T22:19:00Z", "uuid":"synthetic", "parentUuid":null, "message":{"role":"user", "content":"Read AGENTS.md, then do Step 2.4 in PLAN.md."}});
    let handoff = json!({"type":"assistant", "sessionId":id, "cwd":repo, "timestamp":"2026-11-10T22:19:01Z", "uuid":"handoff", "parentUuid":"synthetic", "message":{"role":"assistant", "content":[{"type":"tool_use", "id":"toolu_synthetic", "name":"Bash", "input":{"command":"codex exec 'Read AGENTS.md, then do Step 2.4 in PLAN.md.'"}}]}});
    fs::write(
        transcripts.join(format!("{id}.jsonl")),
        format!("{record}\n{handoff}\n"),
    )
    .unwrap();
    let records = temp.path().join(".claude/sessions");
    fs::create_dir_all(&records).unwrap();
    fs::write(records.join("12345.json"), json!({"pid":12345,"procStart":"synthetic start","sessionId":id,"cwd":repo,"startedAt":1794349140000_i64,"status":"waiting","updatedAt":1794349141000_i64,"statusUpdatedAt":1794349141000_i64}).to_string()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_multi-link"))
        .args(["menubar", "--json", "--root"])
        .arg(temp.path())
        .args(["--now", "2026-11-10T22:20:00Z"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["repos"][0]["step"],
        json!({"id":"2.4", "title":"Synthetic parser title", "more":0})
    );
}

#[cfg(unix)]
fn watcher_parent_test(orphan_at_start: bool) {
    use std::{
        fs,
        io::{BufRead, BufReader, Read, Write},
        process::Stdio,
        time::{Duration, Instant},
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let temp = tempfile::tempdir().unwrap();
    let output_path = temp.path().join("watch.jsonl");
    let script = if orphan_at_start {
        r#"import os, sys, time
pid = os.fork()
if pid:
    print(pid, flush=True)
    sys.exit(0)
while os.getppid() != 1:
    time.sleep(.001)
os.dup2(1, 2)
with open(sys.argv[1], 'w') as output:
    os.dup2(output.fileno(), 1)
os.execv(sys.argv[2], sys.argv[2:])
"#
    } else {
        r#"import subprocess, sys
with open(sys.argv[1], 'w') as output:
    child = subprocess.Popen(sys.argv[2:], stdout=output, stderr=sys.stdout)
print(child.pid, flush=True)
sys.stdin.readline()
"#
    };
    let mut parent = Command::new("python3")
        .args(["-c", script])
        .arg(&output_path)
        .arg(env!("CARGO_BIN_EXE_multi-link"))
        .current_dir(&root)
        .args([
            "menubar",
            "--json",
            "--watch",
            "--root",
            "fixtures/sessions-phase14",
            "--now",
            "2026-11-10T22:20:00Z",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let mut pid = String::new();
    let mut pipe = BufReader::new(parent.stdout.take().unwrap());
    pipe.read_line(&mut pid).unwrap();
    let (exited, exit_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut remaining = String::new();
        let result = pipe.read_to_string(&mut remaining);
        let _ = exited.send((result, remaining));
    });
    let pid = pid.trim();
    let started = Instant::now();
    if !orphan_at_start {
        loop {
            if fs::read_to_string(&output_path).is_ok_and(|s| s.ends_with('\n')) {
                break;
            }
            assert!(
                started.elapsed() < Duration::from_secs(5),
                "no first flushed line"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        let text = fs::read_to_string(&output_path).unwrap();
        let lines: Vec<serde_json::Value> = text
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0]["v"], 1);
        assert_eq!(lines[0]["repos"].as_array().unwrap().len(), 6);
        parent.stdin.take().unwrap().write_all(b"exit\n").unwrap();
    }
    assert!(parent.wait().unwrap().success());
    let limit = if orphan_at_start {
        Duration::from_secs(1)
    } else {
        Duration::from_millis(3250)
    };
    match exit_rx.recv_timeout(limit) {
        Ok((result, remaining)) => {
            result.unwrap();
            assert!(remaining.is_empty(), "watcher stderr: {remaining}");
        }
        Err(error) => {
            let _ = Command::new("python3")
                .args([
                    "-c",
                    "import os,signal,sys; os.kill(int(sys.argv[1]),signal.SIGKILL)",
                    pid,
                ])
                .status();
            panic!("watcher survived parent beyond {limit:?}: {error}");
        }
    }
    if orphan_at_start {
        assert!(
            fs::read_to_string(&output_path)
                .unwrap_or_default()
                .is_empty()
        );
    }
}

#[test]
#[cfg(unix)]
fn watcher_exits_within_one_tick_after_parent_exits() {
    watcher_parent_test(false);
}

#[test]
#[cfg(unix)]
fn watcher_started_with_parent_one_exits_immediately() {
    watcher_parent_test(true);
}

fn copy_tree(source: &Path, target: &Path) {
    std::fs::create_dir_all(target).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let to = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &to);
        } else {
            std::fs::copy(entry.path(), to).unwrap();
        }
    }
}

#[test]
fn app_only_accepts_app_link_and_refuses_resume_after_app_record_disappears() {
    let temp = tempfile::tempdir().unwrap();
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sessions-phase9");
    copy_tree(&fixtures, temp.path());
    let open = |id, dry_run| {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_multi-link"));
        cmd.args(["open", "--app-only", "--root"])
            .arg(temp.path())
            .args(["--now", "2026-11-09T06:00:00Z", id]);
        if dry_run {
            cmd.arg("--dry-run");
        }
        cmd.output().unwrap()
    };
    let output = open("92000002", true);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["action"], "app_link");
    std::fs::remove_file(temp.path().join(
        "Library/Application Support/Claude/claude-code-sessions/00000000-0000-4000-8000-0000000094a0/00000000-0000-4000-8000-0000000094b0/local_92f00002-0000-4000-8000-000000009202.json"
    )).unwrap();
    let output = open("92000002", true);
    assert_eq!(output.status.code(), Some(3), "{output:?}");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["action"], "unavailable");
    assert_eq!(value["reason"], "app_session");
    // A CLI session that resolves to Resume must also be refused, including
    // without dry-run: app-only must never launch a terminal resume command.
    let output = open("92000001", true);
    assert_eq!(output.status.code(), Some(3), "{output:?}");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["action"], "resume");
    assert!(String::from_utf8_lossy(&output.stderr).contains("no app link available"));
    let output = open("92000001", false);
    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("no app link available"));
}

#[test]
fn said_uses_last_completed_real_turn_and_cuts_at_120_unicode_characters() {
    use serde_json::json;
    let temp = tempfile::tempdir().unwrap();
    let sessions = temp.path().join(".codex/sessions");
    std::fs::create_dir_all(&sessions).unwrap();
    for length in [0, 119, 120, 121] {
        let message = "é".repeat(length);
        let records = [
            json!({"type":"session_meta","timestamp":"2026-11-10T22:00:00Z","payload":{"id":"aaaaaaaa-0000-4000-8000-000000000001","cwd":"/synthetic","timestamp":"2026-11-10T22:00:00Z"}}),
            json!({"type":"event_msg","timestamp":"2026-11-10T22:01:00Z","payload":{"type":"task_started","turn_id":"real"}}),
            json!({"type":"event_msg","timestamp":"2026-11-10T22:02:00Z","payload":{"type":"task_complete","turn_id":"real","last_agent_message":message}}),
            json!({"type":"event_msg","timestamp":"2026-11-10T22:02:10Z","payload":{"type":"task_started","turn_id":"external-import-1"}}),
            json!({"type":"event_msg","timestamp":"2026-11-10T22:02:20Z","payload":{"type":"task_complete","turn_id":"external-import-1","last_agent_message":"must be ignored"}}),
            json!({"type":"event_msg","timestamp":"2026-11-10T22:03:00Z","payload":{"type":"task_started","turn_id":"open"}}),
        ];
        let path = sessions.join("rollout-synthetic.jsonl");
        std::fs::write(
            &path,
            records.iter().map(|v| format!("{v}\n")).collect::<String>(),
        )
        .unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_multi-link"))
            .args(["menubar", "--json", "--root"])
            .arg(temp.path())
            .args(["--now", "2026-11-10T22:04:00Z"])
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            value["repos"][0]["rows"][0]["said"],
            if length > 120 {
                format!("{}…", "é".repeat(120))
            } else {
                message
            }
        );
    }
}

#[test]
fn notify_flag_uses_full_refresh_and_writes_permanent_invocation_marker() {
    use serde_json::{Value, json};
    use std::{
        io::{BufRead, BufReader, Write},
        process::Stdio,
        sync::mpsc,
        time::Duration,
    };
    struct Watcher(std::process::Child);
    impl Drop for Watcher {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sessions-phase14");
    let temp = tempfile::tempdir().unwrap();
    copy_tree(&fixture, temp.path());
    let markers = temp.path().join("watch-state");
    let mut watcher = Watcher(
        Command::new(env!("CARGO_BIN_EXE_multi-link"))
            .args(["menubar", "--json", "--watch", "--notify", "--root"])
            .arg(temp.path())
            .args(["--now", "2026-11-10T22:20:00Z"])
            .env("MULTI_LINK_HOME", &markers)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    let (sender, receiver) = mpsc::channel();
    let stdout = watcher.0.stdout.take().unwrap();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if sender.send(line.unwrap()).is_err() {
                break;
            }
        }
    });
    let receive = || -> Value {
        serde_json::from_str(&receiver.recv_timeout(Duration::from_secs(5)).unwrap()).unwrap()
    };
    assert_eq!(receive()["repos"].as_array().unwrap().len(), 6);
    assert!(
        !markers.join("notified").exists(),
        "baseline must not claim"
    );
    let rollout = temp.path().join(".codex/sessions/2026/11/10/rollout-2026-11-10T21-40-04-14210000-0000-4000-8000-000000001421.jsonl");
    writeln!(
        std::fs::OpenOptions::new()
            .append(true)
            .open(rollout)
            .unwrap(),
        "{}",
        json!({"type":"event_msg","timestamp":"2026-11-10T22:19:59Z","payload":{
            "type":"task_complete","turn_id":"14210002-0000-4000-8000-000000021421",
            "last_agent_message":"finished synthetic"
        }})
    )
    .unwrap();
    assert_eq!(
        receive(),
        json!({"v":1,"event":{
            "kind":"codex_done","run":"toolu_p14_1411_18",
            "id":"14210000-0000-4000-8000-000000001421","step":"2.3","said":"finished synthetic"
        }})
    );
    assert_eq!(receive()["v"], 1); // Changed data follows the event in the same tick.
    assert_eq!(receive(), json!({"v":1,"tick":"2026-11-10T22:20:00.000Z"}));
    assert!(markers.join("notified/toolu_p14_1411_18").is_file());
    assert_eq!(
        std::fs::read_dir(markers.join("notified")).unwrap().count(),
        1
    );
}

#[test]
fn menubar_opens_distinguishes_terminal_app_and_unavailable() {
    use serde_json::{Value, json};
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (fixture, now, expected) in [
        (
            "sessions-phase18",
            "2026-11-12T09:00:00Z",
            vec![
                ("18000001", json!("terminal")),
                ("18000002", json!("terminal")),
                ("18000003", Value::Null),
                ("18000004", Value::Null),
                ("18000005", Value::Null),
                ("18000006", Value::Null),
                ("18000007", Value::Null),
                ("18000008", Value::Null),
                ("18000009", json!("terminal")),
                ("18000010", Value::Null),
                ("18000011", Value::Null),
            ],
        ),
        (
            "sessions-phase14",
            "2026-11-10T22:20:00Z",
            vec![("14120000", Value::Null)],
        ),
        (
            "sessions-phase9",
            "2026-11-09T06:00:00Z",
            vec![("92000002", json!("app")), ("92000001", Value::Null)],
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_multi-link"))
            .current_dir(&root)
            .env("TZ", "UTC")
            .args([
                "menubar",
                "--json",
                "--root",
                &format!("fixtures/{fixture}"),
                "--now",
                now,
            ])
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        let rows: Vec<_> = report["repos"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|s| s["rows"].as_array().unwrap())
            .collect();
        for (prefix, opens) in expected {
            let row = rows
                .iter()
                .find(|r| r["id"].as_str().unwrap().starts_with(prefix))
                .unwrap();
            assert!(row.get("opens").is_some(), "missing opens: {prefix}");
            assert_eq!(row["opens"], opens, "{prefix}");
            assert_eq!(row["openable"], opens != Value::Null, "{prefix}");
        }
        for row in rows {
            for child in row["children"].as_array().unwrap() {
                assert!(child.get("opens").is_some());
                assert_eq!(child["opens"], Value::Null);
                assert_eq!(child["openable"], false);
            }
        }
    }
}

#[test]
fn retired_pane_command_is_rejected() {
    let output = Command::new(env!("CARGO_BIN_EXE_multi-link"))
        .args(["pane", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unrecognized subcommand 'pane'"));
}
