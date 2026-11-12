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

#[test]
fn jump_execution_is_temporarily_refused() {
    for (extra, message) in [
        (
            vec![],
            "cannot jump: terminal jump execution is not implemented yet\n",
        ),
        (
            vec!["--app-only"],
            "cannot open \"18000001-0000-4000-8000-000018000001\": no app link available\n",
        ),
        (
            vec!["--app-only", "--dry-run"],
            "cannot open \"18000001-0000-4000-8000-000018000001\": no app link available\n",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_multi-link"))
            .args(["open", "18000001", "--root"])
            .arg(fixtures().join("sessions-phase18"))
            .args(["--now", "2026-11-12T09:00:00Z"])
            .args(extra)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(3));
        assert_eq!(String::from_utf8(output.stderr).unwrap(), message);
    }
}
