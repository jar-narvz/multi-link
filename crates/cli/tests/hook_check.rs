use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::Path,
    process::{Command, Output},
};

const BIN: &str = env!("CARGO_BIN_EXE_multi-link");
fn run(path: &Path, merged: bool) -> Output {
    let mut command = Command::new(BIN);
    command.args(["hook", "check", "--settings"]).arg(path);
    if merged {
        command.arg("--merged");
    }
    command.output().unwrap()
}
fn printed(output: &Output) -> String {
    String::from_utf8(output.stdout.clone())
        .unwrap()
        .lines()
        .last()
        .unwrap()
        .into()
}
fn shell(command: &str) -> Output {
    Command::new("sh").args(["-c", command]).output().unwrap()
}

#[test]
fn check_merge_command_preserves_values_order_modes_and_backup() {
    for mode in [0o644, 0o600] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("- settings ' $(never) `never`.json");
        let source = "{\"z\":7,\"other\":{\"enabled\":false},\"a\":42}\n";
        fs::write(&path, source).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
        let output = run(&path, false);
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        assert_eq!(fs::read_to_string(&path).unwrap(), source);
        let merged = run(&path, true);
        assert_eq!(merged.status.code(), Some(0));
        assert!(merged.stderr.is_empty());
        let value: Value = serde_json::from_slice(&merged.stdout).unwrap();
        assert_eq!(value["other"], json!({"enabled":false}));
        let applied = shell(&printed(&output));
        assert!(applied.status.success(), "{applied:?}");
        assert_eq!(run(&path, false).status.code(), Some(0));
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            mode
        );
        let backups: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.file_name().unwrap().to_string_lossy().contains("backup-"))
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(&backups[0]).unwrap(), source.as_bytes());
        let saved: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(saved, value);
        assert_eq!(
            saved
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["z", "other", "a", "hooks"]
        );
        assert_eq!(run(&path, true).status.code(), Some(0));
    }
}

#[test]
fn absent_and_symlinked_files_and_reread() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("new/settings.json");
    let output = run(&target, false);
    assert!(!target.exists());
    assert!(shell(&printed(&output)).status.success());
    assert_eq!(
        fs::metadata(&target).unwrap().permissions().mode() & 0o777,
        0o600
    );
    fs::write(&target, "{}").unwrap();
    let link = dir.path().join("link.json");
    symlink(&target, &link).unwrap();
    let output = run(&link, false);
    fs::write(&target, r#"{"edited":true}"#).unwrap();
    assert!(shell(&printed(&output)).status.success());
    assert!(fs::symlink_metadata(&link).unwrap().is_symlink());
    let value: Value = serde_json::from_slice(&fs::read(&target).unwrap()).unwrap();
    assert_eq!(value["edited"], true);
    assert_eq!(run(&link, false).status.code(), Some(0));
}

#[test]
fn scope_and_explicit_override_use_temp_config() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("settings.json"), "{}").unwrap();
    let output = Command::new(BIN)
        .args(["hook", "check", "--scope", "user"])
        .env("CLAUDE_CONFIG_DIR", dir.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains(dir.path().to_str().unwrap())
    );
    let explicit = dir.path().join("explicit.json");
    let output = Command::new(BIN)
        .args(["hook", "check", "--scope", "local", "--settings"])
        .arg(&explicit)
        .env("CLAUDE_CONFIG_DIR", dir.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains(explicit.to_str().unwrap())
    );
}

#[test]
fn errors_have_no_command_and_do_not_touch_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.json");
    for source in ["", "{", "[]", r#"{"a":1,"a":2}"#] {
        fs::write(&path, source).unwrap();
        for merged in [false, true] {
            let output = run(&path, merged);
            assert_eq!(output.status.code(), Some(2), "{output:?}");
            assert!(output.stdout.is_empty());
            assert!(!output.stderr.is_empty());
            assert_eq!(fs::read_to_string(&path).unwrap(), source);
        }
    }
    fs::set_permissions(&path, fs::Permissions::from_mode(0o0)).unwrap();
    assert_eq!(run(&path, false).status.code(), Some(2));
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let dangling = dir.path().join("dangling");
    symlink(dir.path().join("missing"), &dangling).unwrap();
    let fifo = dir.path().join("fifo");
    assert!(
        Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success()
    );
    for path in [&dangling, dir.path(), &fifo, &dir.path().join("new\nline")] {
        let output = run(path, false);
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn failing_merge_command_leaves_original_and_cleans_temp() {
    let dir = tempfile::tempdir().unwrap();
    let exe = dir.path().join("multi-link");
    fs::copy(BIN, &exe).unwrap();
    let path = dir.path().join("settings.json");
    fs::write(&path, "{}").unwrap();
    let output = Command::new(&exe)
        .args(["hook", "check", "--settings"])
        .arg(&path)
        .output()
        .unwrap();
    let stub = dir.path().join("stub");
    fs::write(&stub, "#!/bin/sh\nexit 2\n").unwrap();
    fs::set_permissions(&stub, fs::Permissions::from_mode(0o700)).unwrap();
    fs::rename(&stub, &exe).unwrap();
    assert!(!shell(&printed(&output)).status.success());
    assert_eq!(fs::read_to_string(&path).unwrap(), "{}");
    let names: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names.len(), 3, "{names:?}"); // original, binary, backup only
    assert_eq!(names.iter().filter(|n| n.contains("backup-")).count(), 1);
}

#[test]
fn local_scope_and_special_binary_path_use_absolute_quoted_paths() {
    let dir = tempfile::tempdir().unwrap();
    assert!(
        Command::new("git")
            .arg("init")
            .arg(dir.path())
            .output()
            .unwrap()
            .status
            .success()
    );
    let nested = dir.path().join("- space ' $(never) `never`");
    fs::create_dir(&nested).unwrap();
    let binary = nested.join("multi-link");
    fs::copy(BIN, &binary).unwrap();
    let output = Command::new(&binary)
        .args(["hook", "check", "--scope", "local"])
        .current_dir(&nested)
        .env("HOME", dir.path())
        .env("CLAUDE_CONFIG_DIR", dir.path().join("unused"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let target = dir.path().join(".claude/settings.local.json");
    assert!(String::from_utf8_lossy(&output.stdout).contains(target.to_str().unwrap()));
    assert!(!target.exists());
    let applied = shell(&printed(&output));
    assert!(applied.status.success(), "{applied:?}");
    let checked = Command::new(&binary)
        .args(["hook", "check", "--settings"])
        .arg(&target)
        .output()
        .unwrap();
    assert_eq!(checked.status.code(), Some(0), "{checked:?}");
}
