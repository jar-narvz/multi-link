use std::{path::Path, process::Command};

#[test]
fn pane_matches_all_ten_pretty_json_keys_byte_for_byte() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cases = [
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
    assert_eq!(cases.len(), 10);
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
