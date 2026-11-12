use std::{fs, path::PathBuf, process::Command};

use serde_json::Value;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn check_keys(path: &str) {
    let keys: Value = serde_json::from_slice(&fs::read(fixtures().join(path)).unwrap()).unwrap();
    let root = keys["root"]
        .as_str()
        .unwrap()
        .strip_prefix("fixtures/")
        .unwrap();
    let now = keys["now"].as_str().unwrap();
    for mode in ["open", "fork"] {
        for (id, key) in keys[mode].as_object().unwrap() {
            let mut command = Command::new(env!("CARGO_BIN_EXE_multi-link"));
            command
                .args(["open", id, "--dry-run", "--root"])
                .arg(fixtures().join(root))
                .args(["--now", now]);
            if mode == "fork" {
                command.arg("--fork");
            }
            let output = command.output().unwrap();
            assert_eq!(
                output.status.code(),
                key["exit"].as_i64().map(|n| n as i32),
                "{mode} {id}"
            );
            let expected = format!(
                "{}\n",
                serde_json::to_string_pretty(&key["action"]).unwrap()
            );
            assert_eq!(
                String::from_utf8(output.stdout).unwrap(),
                expected,
                "{mode} {id}"
            );
        }
    }
}

#[test]
fn open_jump_keys_match_bytes_and_exit_codes() {
    check_keys("expected/open-jump/open.json");
}

#[test]
fn old_open_keys_match_bytes_and_exit_codes() {
    check_keys("expected/phase9/open.json");
}

fn fake_tools(body: &str) -> tempfile::TempDir {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    for (name, script) in [
        ("osascript", body.to_owned()),
        (
            "claude",
            format!(
                "#!/bin/sh\nprintf resumed > '{}'\n",
                dir.path().join("resumed").display()
            ),
        ),
    ] {
        let path = dir.path().join(name);
        fs::write(&path, script).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    dir
}

fn jump_cli(dir: &std::path::Path, id: &str, extra: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_multi-link"))
        .env("PATH", dir)
        .args(["open", id, "--root"])
        .arg(fixtures().join("sessions-phase18"))
        .args(["--now", "2026-11-12T09:00:00Z"])
        .args(extra)
        .output()
        .unwrap()
}

#[test]
fn jump_exact_argv_and_success_for_both_apps() {
    for (id, app, tty, script) in [
        ("18000001", "Terminal", "ttys004", "terminal"),
        ("18000002", "iTerm2", "ttys005", "iterm2"),
    ] {
        let dir = fake_tools(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$0.argv\"\nprintf 'window 2 tab 3\\n'\n",
        );
        let output = jump_cli(dir.path(), id, &["--no-resume"]);
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!("jumped to {app} · window 2 · tab 3 ({tty})\n")
        );
        assert!(output.stderr.is_empty());
        let source = fs::read_to_string(
            fixtures().join(format!("expected/open-jump/scripts/{script}.applescript")),
        )
        .unwrap();
        let mut expected = String::new();
        for line in source.lines() {
            expected.push_str(&format!("-e\n{line}\n"));
        }
        expected.push_str(&format!("/dev/{tty}\n"));
        assert_eq!(
            fs::read_to_string(dir.path().join("osascript.argv")).unwrap(),
            expected
        );
        assert!(!dir.path().join("resumed").exists());
    }
}

#[test]
fn jump_failures_never_resume() {
    for (body, expected) in [
        ("printf 'not found\\n'", "cannot jump: its tab is gone\n"),
        (
            "printf 'execution error: denied (-1743)\\n' >&2; exit 1",
            "cannot jump: blocked by macOS. Allow it in System Settings → Privacy & Security → Automation\n",
        ),
        (
            "printf 'execution error: broken (-42)\\n' >&2; exit 1",
            "cannot jump: osascript: execution error: broken (-42)\n",
        ),
        (
            "printf 'window two tab 3\\n'",
            "cannot jump: unexpected osascript output\n",
        ),
        (
            "printf 'window 2 tab 3 extra\\n'",
            "cannot jump: unexpected osascript output\n",
        ),
        ("exit 1", "cannot jump: osascript exited unsuccessfully\n"),
    ] {
        let dir = fake_tools(&format!("#!/bin/sh\n{body}\n"));
        let output = jump_cli(dir.path(), "18000001", &[]);
        assert_eq!(output.status.code(), Some(3));
        assert_eq!(String::from_utf8(output.stderr).unwrap(), expected);
        assert!(output.stdout.is_empty());
        assert!(!dir.path().join("resumed").exists());
    }
    let dir = fake_tools("#!/bin/sh\nexit 99\n");
    fs::remove_file(dir.path().join("osascript")).unwrap();
    let output = jump_cli(dir.path(), "18000001", &[]);
    assert_eq!(output.status.code(), Some(3));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .starts_with("cannot jump: cannot run osascript:")
    );
    assert!(!dir.path().join("resumed").exists());
}

#[test]
fn app_only_refuses_jump_without_running_script() {
    let dir = fake_tools("#!/bin/sh\nprintf called > \"$0.called\"\n");
    for flags in [vec!["--app-only"], vec!["--app-only", "--dry-run"]] {
        let output = jump_cli(dir.path(), "18000001", &flags);
        assert_eq!(output.status.code(), Some(3));
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            "cannot open \"18000001-0000-4000-8000-000018000001\": no app link available\n"
        );
        assert!(!dir.path().join("osascript.called").exists());
    }
}

#[test]
fn no_resume_accepts_app_link_and_refuses_resume() {
    let dir = fake_tools("#!/bin/sh\nexit 99\n");
    for (id, action) in [("92000001", "resume"), ("92000002", "app_link")] {
        for dry_run in [true, false] {
            if action == "app_link" && !dry_run {
                continue;
            }
            let mut command = Command::new(env!("CARGO_BIN_EXE_multi-link"));
            command
                .env("PATH", dir.path())
                .args(["open", id, "--no-resume", "--root"])
                .arg(fixtures().join("sessions-phase9"))
                .args(["--now", "2026-11-09T06:00:00Z"]);
            if dry_run {
                command.arg("--dry-run");
            }
            let output = command.output().unwrap();
            assert_eq!(
                output.status.code(),
                Some(if action == "resume" { 3 } else { 0 })
            );
            if action == "resume" {
                assert_eq!(
                    String::from_utf8(output.stderr).unwrap(),
                    "cannot open \"92000001-0000-4000-8000-000092000001\": resume disabled\n"
                );
            }
        }
    }
    assert!(!dir.path().join("resumed").exists());
}

#[test]
fn unavailable_message_includes_detail_and_preserves_absent_detail() {
    let dir = tempfile::tempdir().unwrap();
    for (prefix, id, reason) in [
        (
            "18000003",
            "18000003-0000-4000-8000-000018000003",
            "inside_multiplexer (tmux)",
        ),
        (
            "18000004",
            "18000004-0000-4000-8000-000018000004",
            "terminal_not_supported (Visual Studio Code)",
        ),
        ("18000005", "18000005-0000-4000-8000-000018000005", "no_tty"),
    ] {
        let output = jump_cli(dir.path(), prefix, &[]);
        assert_eq!(output.status.code(), Some(3));
        assert!(output.stdout.is_empty());
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!(
                "cannot open \"{id}\": {reason}; run it yourself in \"/home/dev/repo-p18\": claude --resume {id}\n"
            )
        );
    }
}
