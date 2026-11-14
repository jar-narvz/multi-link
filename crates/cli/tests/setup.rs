use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    process::{Command, Output},
};
const BIN: &str = env!("CARGO_BIN_EXE_multi-link");
fn command(home: &Path) -> Command {
    let mut c = Command::new(BIN);
    c.env("HOME", home)
        .env("CLAUDE_CONFIG_DIR", home.join("config"));
    c
}
fn hook(home: &Path, action: &str, file: &Path) -> Output {
    command(home)
        .args(["hook", action, "--settings"])
        .arg(file)
        .output()
        .unwrap()
}
fn names(dir: &Path) -> Vec<PathBuf> {
    let mut names: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    names.sort();
    names
}
fn home_snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn walk(root: &Path, dir: &Path, entries: &mut Vec<(PathBuf, Vec<u8>)>) {
        for path in names(dir) {
            let meta = fs::symlink_metadata(&path).unwrap();
            let bytes = if meta.is_symlink() {
                fs::read_link(&path)
                    .unwrap()
                    .as_os_str()
                    .as_encoded_bytes()
                    .to_vec()
            } else if meta.is_file() {
                fs::read(&path).unwrap()
            } else {
                vec![]
            };
            entries.push((path.strip_prefix(root).unwrap().to_owned(), bytes));
            if meta.is_dir() {
                walk(root, &path, entries);
            }
        }
    }
    let mut entries = vec![];
    walk(root, root, &mut entries);
    entries
}

#[test]
fn apply_matches_check_bytes_preserves_mode_and_private_backup_and_is_idempotent() {
    let old = multi_link_hook::settings::snippet("/old/multi-link");
    let mut duplicate = old.clone();
    duplicate["hooks"]["SessionEnd"]
        .as_array_mut()
        .unwrap()
        .push(old["hooks"]["SessionEnd"][0].clone());
    let mut custom = old.clone();
    custom["hooks"]["PostToolUse"][0]["matcher"] = json!("Custom");
    custom["hooks"]["Future"] =
        json!([{"hooks":[{"type":"command","command":"multi-link hook record"}]}]);
    for source in [
        None,
        Some(json!({"keep": [3, 1], "disableAllHooks":true})),
        Some(old),
        Some(duplicate),
        Some(custom),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("nested/settings.json");
        let bytes = source.map(|v| serde_json::to_vec(&v).unwrap());
        if let Some(bytes) = &bytes {
            fs::create_dir(file.parent().unwrap()).unwrap();
            fs::write(&file, bytes).unwrap();
            fs::set_permissions(&file, fs::Permissions::from_mode(0o640)).unwrap();
        }
        let expected = command(dir.path())
            .args(["hook", "check", "--settings"])
            .arg(&file)
            .arg("--merged")
            .output()
            .unwrap();
        assert_eq!(expected.status.code(), Some(0));
        let applied = hook(dir.path(), "apply", &file);
        assert_eq!(applied.status.code(), Some(0), "{applied:?}");
        assert_eq!(fs::read(&file).unwrap(), expected.stdout);
        let before = names(file.parent().unwrap());
        let backups: Vec<_> = before
            .iter()
            .filter(|p| p.to_string_lossy().contains("backup-"))
            .collect();
        assert_eq!(backups.len(), usize::from(bytes.is_some()));
        if let Some(original) = bytes {
            assert_eq!(
                String::from_utf8(applied.stdout).unwrap(),
                format!(
                    "updated {} (backup: {})\n",
                    fs::canonicalize(&file).unwrap().display(),
                    fs::canonicalize(backups[0]).unwrap().display()
                )
            );
            assert_eq!(fs::read(backups[0]).unwrap(), original);
            assert_eq!(
                fs::metadata(backups[0]).unwrap().permissions().mode() & 0o777,
                0o600
            );
            assert_eq!(
                fs::metadata(&file).unwrap().permissions().mode() & 0o777,
                0o640
            );
        } else {
            assert_eq!(
                String::from_utf8(applied.stdout).unwrap(),
                format!("created {}\n", fs::canonicalize(&file).unwrap().display())
            );
            assert_eq!(
                fs::metadata(&file).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        let repeated = hook(dir.path(), "apply", &file);
        assert!(repeated.status.success());
        assert_eq!(
            String::from_utf8(repeated.stdout).unwrap(),
            format!(
                "no change: {}\n",
                fs::canonicalize(&file).unwrap().display()
            )
        );
        assert_eq!(names(file.parent().unwrap()), before);
        assert_eq!(fs::read(&file).unwrap(), expected.stdout);
    }
}
#[test]
fn remove_keeps_other_hooks_events_keys_and_symlinks() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target.json");
    let link = dir.path().join("link.json");
    let source = json!({"keep":7,"hooks":{"Future":[{"matcher":"x","keep":9,"hooks":[{"type":"command","command":"'/old/multi-link' hook record","extra":true},{"type":"command","command":"other"}]}],"Stop":[]}});
    fs::write(&target, source.to_string()).unwrap();
    symlink(&target, &link).unwrap();
    assert!(hook(dir.path(), "apply", &link).status.success());
    let before_remove = names(dir.path());
    let removed = hook(dir.path(), "remove", &link);
    assert!(removed.status.success());
    let backups: Vec<_> = names(dir.path())
        .into_iter()
        .filter(|p| !before_remove.contains(p))
        .collect();
    assert_eq!(backups.len(), 1);
    assert_eq!(
        String::from_utf8(removed.stdout).unwrap(),
        format!(
            "updated {} (backup: {})\n",
            fs::canonicalize(&target).unwrap().display(),
            fs::canonicalize(&backups[0]).unwrap().display()
        )
    );
    assert!(fs::symlink_metadata(&link).unwrap().is_symlink());
    let actual: Value = serde_json::from_slice(&fs::read(&target).unwrap()).unwrap();
    assert_eq!(actual["keep"], 7);
    assert_eq!(
        actual["hooks"]["Future"],
        json!([{"matcher":"x","keep":9,"hooks":[{"type":"command","command":"other"}]}])
    );
    assert_eq!(actual["hooks"]["Stop"], json!([]));
    assert_eq!(
        actual,
        json!({"keep":7,"hooks":{"Future":[{"matcher":"x","keep":9,"hooks":[{"type":"command","command":"other"}]}],"Stop":[]}})
    );
    let before = names(dir.path());
    let bytes = fs::read(&target).unwrap();
    let repeated = hook(dir.path(), "remove", &link);
    assert!(repeated.status.success());
    assert_eq!(
        String::from_utf8(repeated.stdout).unwrap(),
        format!(
            "no change: {}\n",
            fs::canonicalize(&target).unwrap().display()
        )
    );
    assert_eq!(names(dir.path()), before);
    assert_eq!(fs::read(&target).unwrap(), bytes);
}
#[test]
fn invalid_unreadable_and_nonregular_files_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("settings.json");
    for bytes in ["{", "[]", "{\"x\":1,\"x\":2}", "{\"hooks\":false}"] {
        fs::write(&file, bytes).unwrap();
        for action in ["apply", "remove"] {
            assert!(!hook(dir.path(), action, &file).status.success());
            assert_eq!(fs::read_to_string(&file).unwrap(), bytes);
            assert_eq!(names(dir.path()), std::slice::from_ref(&file));
        }
    }
    fs::set_permissions(&file, fs::Permissions::from_mode(0o0)).unwrap();
    for action in ["apply", "remove"] {
        assert!(!hook(dir.path(), action, &file).status.success());
    }
    fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
    for action in ["apply", "remove"] {
        assert!(!hook(dir.path(), action, dir.path()).status.success());
    }
}
#[test]
fn setup_json_contract_is_read_only_and_no_tty_does_not_apply() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config");
    fs::create_dir(&config).unwrap();
    let file = config.join("settings.json");
    fs::write(&file, "{}").unwrap();
    let before = home_snapshot(dir.path());
    let output = command(dir.path())
        .args(["setup", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["v"], 1);
    assert_eq!(
        value["binary"],
        fs::canonicalize(BIN).unwrap().to_str().unwrap()
    );
    assert_eq!(value["items"][0], json!({"id":"cli","state":"ok"}));
    assert_eq!(value["items"][1]["id"], "hooks");
    assert_eq!(value["items"][1]["state"], "fixable");
    assert_eq!(
        value["items"][1]["file"],
        fs::canonicalize(&file).unwrap().to_str().unwrap()
    );
    assert_eq!(value["items"][1]["exists"], true);
    assert_eq!(value["items"][1]["events"].as_array().unwrap().len(), 5);
    assert_eq!(value["items"][2], json!({"id":"app","state":"none"}));
    assert_eq!(home_snapshot(dir.path()), before);
    let text = command(dir.path()).arg("setup").output().unwrap();
    assert_eq!(text.status.code(), Some(1));
    assert!(!String::from_utf8_lossy(&text.stdout).contains("[y/N]"));
    assert_eq!(fs::read_to_string(&file).unwrap(), "{}");
    assert_eq!(names(&config), std::slice::from_ref(&file));
    assert_eq!(home_snapshot(dir.path()), before);
    let conflict = command(dir.path())
        .args(["setup", "--json", "--yes"])
        .output()
        .unwrap();
    assert_eq!(conflict.status.code(), Some(2));
    assert!(conflict.stdout.is_empty());
    assert!(
        command(dir.path())
            .args(["setup", "--yes"])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert_eq!(
        command(dir.path())
            .args(["setup", "--json"])
            .output()
            .unwrap()
            .status
            .code(),
        Some(0)
    );
}

#[test]
fn setup_attention_unreadable_absent_symlink_and_failure_contract() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config");
    let inspect = || {
        let output = command(dir.path())
            .args(["setup", "--json"])
            .output()
            .unwrap();
        assert!(output.stderr.is_empty(), "{output:?}");
        (
            output.status.code().unwrap(),
            serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        )
    };
    let (code, value) = inspect();
    assert_eq!(code, 1);
    assert_eq!(value["items"][1]["exists"], false);
    assert!(!config.exists());
    fs::create_dir(&config).unwrap();
    let file = config.join("settings.json");
    fs::write(&file, "{").unwrap();
    let (code, value) = inspect();
    assert_eq!(code, 1);
    assert_eq!(value["items"][1]["state"], "unreadable");
    assert_eq!(value["items"][1]["events"], json!([]));
    assert_eq!(fs::read(&file).unwrap(), b"{");
    assert_eq!(names(&config), std::slice::from_ref(&file));
    let mut custom =
        multi_link_hook::settings::snippet(fs::canonicalize(BIN).unwrap().to_str().unwrap());
    custom["hooks"]["PostToolUse"][0]["hooks"][0]["custom"] = json!(true);
    let original = custom.to_string();
    fs::write(&file, &original).unwrap();
    let (code, value) = inspect();
    assert_eq!(code, 0);
    assert_eq!(value["items"][1]["state"], "attention");
    assert!(
        command(dir.path())
            .args(["setup", "--yes"])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert_eq!(fs::read_to_string(&file).unwrap(), original);
    assert_eq!(names(&config), std::slice::from_ref(&file));
    let target = dir.path().join("resolved.json");
    fs::rename(&file, &target).unwrap();
    symlink(&target, &file).unwrap();
    let (code, value) = inspect();
    assert_eq!(code, 0);
    assert_eq!(
        value["items"][1]["file"],
        fs::canonicalize(&target).unwrap().to_str().unwrap()
    );
    let output = command(dir.path())
        .env_remove("HOME")
        .args(["setup", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}
#[test]
fn temporary_setup_refuses_yes_and_keeps_home_identical() {
    let dir = tempfile::tempdir().unwrap();
    let folder = dir
        .path()
        .join("AppTranslocation/random/multi-link.app/Contents/MacOS");
    fs::create_dir_all(&folder).unwrap();
    let binary = folder.join("multi-link");
    fs::copy(BIN, &binary).unwrap();
    let config = dir.path().join("config");
    fs::create_dir(&config).unwrap();
    let file = config.join("settings.json");
    fs::write(&file, "{}").unwrap();
    for args in [["setup", "--json"], ["setup", "--yes"]] {
        let output = Command::new(&binary)
            .env("HOME", dir.path())
            .env("CLAUDE_CONFIG_DIR", &config)
            .args(args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        if args[1] == "--json" {
            let value: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(value["items"][0]["state"], "temporary");
            assert_eq!(value["items"][2]["state"], "elsewhere");
        } else {
            assert!(
                String::from_utf8_lossy(&output.stderr)
                    .contains("move multi-link to Applications first")
            );
        }
        assert_eq!(fs::read(&file).unwrap(), b"{}");
        assert_eq!(names(&config), std::slice::from_ref(&file));
    }
}
#[test]
fn apply_and_remove_use_same_user_and_local_selection_as_check() {
    let dir = tempfile::tempdir().unwrap();
    let config_file = dir.path().join("config/settings.json");
    assert!(
        command(dir.path())
            .args(["hook", "apply"])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(config_file.exists());
    assert!(
        command(dir.path())
            .args(["hook", "remove"])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(&config_file).unwrap()).unwrap(),
        json!({})
    );
    assert!(
        Command::new("git")
            .arg("init")
            .arg(dir.path())
            .output()
            .unwrap()
            .status
            .success()
    );
    let nested = dir.path().join("nested");
    fs::create_dir(&nested).unwrap();
    let local = dir.path().join(".claude/settings.local.json");
    for action in ["apply", "remove"] {
        assert!(
            command(dir.path())
                .current_dir(&nested)
                .args(["hook", action, "--scope", "local"])
                .output()
                .unwrap()
                .status
                .success()
        );
        assert!(local.exists());
    }
    let absent = dir.path().join("absent/settings.json");
    let removed = hook(dir.path(), "remove", &absent);
    assert!(removed.status.success());
    let resolved_absent = fs::canonicalize(dir.path())
        .unwrap()
        .join("absent/settings.json");
    assert_eq!(
        String::from_utf8(removed.stdout).unwrap(),
        format!("no change: {}\n", resolved_absent.display())
    );
    assert!(!absent.parent().unwrap().exists());
}
#[test]
fn tty_setup_answered_no_keeps_original_without_backup() {
    // A PTY exercises the real is_terminal/prompt boundary using Python's stdlib.
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config");
    fs::create_dir(&config).unwrap();
    let file = config.join("settings.json");
    fs::write(&file, "{}").unwrap();
    let output = Command::new("python3")
        .env("HOME", dir.path())
        .env("CLAUDE_CONFIG_DIR", &config)
        .args([
            "-c",
            r#"
import os, pty, select, sys, time
pid, fd = pty.fork()
if pid == 0:
    os.execv(sys.argv[1], [sys.argv[1], 'setup'])
output = b''
answered = False
deadline = time.monotonic() + 15
while time.monotonic() < deadline:
    if not select.select([fd], [], [], 0.1)[0]:
        continue
    try:
        chunk = os.read(fd, 65536)
    except OSError:
        break
    if not chunk:
        break
    output += chunk
    if b'[y/N]' in output and not answered:
        os.write(fd, b'n\n')
        answered = True
else:
    os.kill(pid, 9)
    raise RuntimeError('setup did not finish')
_, status = os.waitpid(pid, 0)
sys.stdout.buffer.write(output)
sys.exit(os.waitstatus_to_exitcode(status))
"#,
            BIN,
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stdout).contains("[y/N]"));
    assert_eq!(fs::read(&file).unwrap(), b"{}");
    assert_eq!(names(&config), [file]);
}

#[test]
fn home_app_is_installed_even_when_home_is_a_symlink() {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    let alias = dir.path().join("alias");
    let folder = home.join("Applications/multi-link.app/Contents/MacOS");
    fs::create_dir_all(&folder).unwrap();
    symlink(&home, &alias).unwrap();
    let binary = folder.join("multi-link");
    fs::copy(BIN, &binary).unwrap();
    let output = Command::new(&binary)
        .env("HOME", &alias)
        .env("CLAUDE_CONFIG_DIR", home.join("config"))
        .args(["setup", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        value["items"][2],
        json!({"id":"app","state":"installed","path":fs::canonicalize(home.join("Applications/multi-link.app")).unwrap()})
    );
}

#[test]
fn current_compact_settings_are_not_reformatted_or_backed_up() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("settings.json");
    let value =
        multi_link_hook::settings::snippet(fs::canonicalize(BIN).unwrap().to_str().unwrap());
    let source = value.to_string();
    fs::write(&file, &source).unwrap();
    assert!(hook(dir.path(), "apply", &file).status.success());
    assert_eq!(fs::read_to_string(&file).unwrap(), source);
    assert_eq!(names(dir.path()), [file]);
}

#[test]
fn apply_then_remove_restores_unrelated_keys() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("settings.json");
    fs::write(&file, r#"{"keep":7}"#).unwrap();
    assert!(hook(dir.path(), "apply", &file).status.success());
    assert!(hook(dir.path(), "remove", &file).status.success());
    let value: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    assert_eq!(value, json!({"keep":7}));
}

#[test]
fn removal_keeps_matcher_and_user_hook_in_shared_group() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("settings.json");
    fs::write(&file, r#"{"keep":7,"hooks":{"Future":[{"matcher":"mine","extra":9,"hooks":[{"type":"command","command":"/old/multi-link hook record"},{"type":"command","command":"user-hook"}]}]}}"#).unwrap();
    assert!(hook(dir.path(), "remove", &file).status.success());
    let value: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    assert_eq!(
        value,
        json!({"keep":7,"hooks":{"Future":[{"matcher":"mine","extra":9,"hooks":[{"type":"command","command":"user-hook"}]}]}})
    );
}

#[test]
fn removal_preserves_preexisting_empty_groups_events_and_hooks_object() {
    for source in [
        json!({"keep":7,"hooks":{}}),
        json!({"keep":7,"hooks":{"Stop":[],"Future":[{"matcher":"mine","hooks":[]}]}}),
        json!({"keep":7,"hooks":{"Stop":[],"Future":[{"matcher":"mine","hooks":[]},{"hooks":[{"type":"command","command":"/old/multi-link hook record"}]}],"SessionEnd":[{"hooks":[{"type":"command","command":"/old/multi-link hook record"}]}]}}),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("settings.json");
        let bytes = source.to_string();
        fs::write(&file, &bytes).unwrap();
        assert!(hook(dir.path(), "remove", &file).status.success());
        let actual = fs::read(&file).unwrap();
        if source["hooks"].get("SessionEnd").is_some() {
            assert_eq!(
                serde_json::from_slice::<Value>(&actual).unwrap(),
                json!({"keep":7,"hooks":{"Stop":[],"Future":[{"matcher":"mine","hooks":[]}]}})
            );
        } else {
            assert_eq!(actual, bytes.as_bytes());
            assert_eq!(names(dir.path()), [file]);
        }
    }
}
