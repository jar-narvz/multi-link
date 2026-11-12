use std::{path::Path, process::Command};

#[test]
fn pane_matches_all_eleven_pretty_json_keys_byte_for_byte() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cases = [
        (
            "openable",
            "--root fixtures/sessions-phase9 --now 2026-11-09T06:00:00Z --cwd /home/dev/repo-p9c --session 92000002-0000-4000-8000-000092000002",
        ),
        (
            "busy",
            "--root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-p --session 14150000-0000-4000-8000-000000001415",
        ),
        (
            "waiting-stale",
            "--root fixtures/sessions-phase9 --now 2026-11-09T07:30:00Z --cwd /home/dev/repo-p9c --session 92000012-0000-4000-8000-00009200000c",
        ),
        (
            "done-2h",
            "--root fixtures/sessions-phase9 --now 2026-11-09T12:00:00Z --cwd /home/dev/repo-p9l --session 95000004-0000-4000-8000-000095000004",
        ),
        (
            "children-exact-likely",
            "--root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-m --session 14120000-0000-4000-8000-000000001412",
        ),
        (
            "unseen",
            "--root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-m --session aaaaaaaa-0000-4000-8000-000000000000",
        ),
        (
            "ambiguous",
            "--root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-r --session 141a0000-0000-4000-8000-00000000141a",
        ),
        (
            "child-other-repo",
            "--root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-q --session 14150000-0000-4000-8000-000000001415",
        ),
        (
            "idle-recent",
            "--root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-m --session 14180000-0000-4000-8000-000000001418",
        ),
        (
            "empty",
            "--root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-none --session aaaaaaaa-0000-4000-8000-000000000000",
        ),
        (
            "waiting-quiet",
            "--root fixtures/sessions-phase9 --now 2026-11-09T12:00:00Z --cwd /home/dev/repo-p9c --session 92000012-0000-4000-8000-00009200000c",
        ),
    ];
    assert_eq!(cases.len(), 11);
    for (name, args) in cases {
        let output = Command::new(env!("CARGO_BIN_EXE_multi-link"))
            .current_dir(&root)
            .env("TZ", "UTC")
            .args(["pane", "--json"])
            .args(args.split_whitespace())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let expected =
            std::fs::read(root.join(format!("fixtures/expected/pane/{name}.json"))).unwrap();
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(expected).unwrap(),
            "{name}"
        );
    }
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
            "pane",
            "--json",
            "--watch",
            "--root",
            "fixtures/sessions-phase14",
            "--now",
            "2026-11-10T22:20:00Z",
            "--cwd",
            "/home/dev/repo-m",
            "--session",
            "unseen",
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
        assert_eq!(lines[0]["seen"], false);
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
fn nested_current_session_has_no_strip() {
    let output = Command::new(env!("CARGO_BIN_EXE_multi-link"))
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .args([
            "pane",
            "--json",
            "--root",
            "fixtures/sessions-phase14",
            "--now",
            "2026-11-10T22:20:00Z",
            "--cwd",
            "/home/dev/repo-m",
            "--session",
            "14210000-0000-4000-8000-000000001421",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["seen"], true);
    assert_eq!(value["strip"], serde_json::Value::Null);
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
            .args(["pane", "--json", "--root"])
            .arg(temp.path())
            .args([
                "--now",
                "2026-11-10T22:04:00Z",
                "--cwd",
                "/synthetic",
                "--session",
                "aaaaaaaa-0000-4000-8000-000000000001",
            ])
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            value["sections"][0]["rows"][0]["said"],
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
            .args(["pane", "--json", "--watch", "--notify", "--root"])
            .arg(temp.path())
            .args([
                "--now",
                "2026-11-10T22:20:00Z",
                "--cwd",
                "/home/dev/repo-m",
                "--session",
                "unseen",
            ])
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
    assert_eq!(receive()["seen"], false);
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
