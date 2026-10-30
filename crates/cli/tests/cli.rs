use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

use serde_json::{Value, json};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn run(command: &str, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_multi-link"))
        .arg(command)
        .arg("--root")
        .arg(fixtures().join("sessions"))
        .args(extra)
        .output()
        .unwrap()
}

fn expected(command: &str) -> Value {
    serde_json::from_slice(&fs::read(fixtures().join(format!("expected/{command}.json"))).unwrap())
        .unwrap()
}

fn success(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn args(command: &str) -> Vec<&'static str> {
    if command == "steps" {
        vec!["--repo", "repos/repo-a", "--repo", "repos/repo-b"]
    } else {
        vec!["--now", "2026-10-29T23:15:00Z", "--since", "all"]
    }
}

#[test]
fn phase_one_json_matches_answer_keys() {
    for command in ["sessions", "links", "steps"] {
        let mut args = args(command);
        args.push("--json");
        let output = run(command, &args);
        success(&output);
        assert!(output.stderr.is_empty());
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap(),
            expected(command),
            "{command}"
        );
    }
}

#[test]
fn text_has_one_row_per_item_and_a_footer() {
    for (command, header, count, footer) in [
        (
            "sessions",
            "ID        HARNESS  STATE    LAST_ACTIVITY             REPO",
            27,
            "27 sessions, 4 skipped lines",
        ),
        (
            "links",
            "PARENT    FORM    MATCH     CHILD     AT",
            13,
            "13 links, 2 unparented",
        ),
        (
            "steps",
            "REPO          ID   VERIFY   TITLE",
            4,
            "4 steps in 2 repos",
        ),
    ] {
        let output = run(command, &args(command));
        success(&output);
        let text = String::from_utf8(output.stdout).unwrap();
        let lines: Vec<_> = text.lines().collect();
        assert!(lines.iter().all(|line| *line == line.trim_end()));
        assert_eq!(lines.len(), count + 2, "{command}");
        assert_eq!(lines[0], header);
        assert_eq!(lines.last(), Some(&footer));
        if command == "sessions" {
            for (line, session) in lines[1..=count]
                .iter()
                .zip(expected(command)["sessions"].as_array().unwrap())
            {
                assert_eq!(
                    *line,
                    format!(
                        "{:<8}  {:<7}  {:<7}  {:<24}  {}",
                        &session["id"].as_str().unwrap()[..8],
                        session["harness"].as_str().unwrap(),
                        session["state"].as_str().unwrap(),
                        session["last_activity"].as_str().unwrap(),
                        session["repo"].as_str().unwrap()
                    )
                );
            }
        } else if command == "links" {
            for (line, link) in lines[1..=count]
                .iter()
                .zip(expected(command)["links"].as_array().unwrap())
            {
                assert_eq!(
                    *line,
                    format!(
                        "{:<8}  {:<6}  {:<8}  {:<8}  {}",
                        &link["parent"].as_str().unwrap()[..8],
                        link["form"].as_str().unwrap(),
                        link["match"].as_str().unwrap(),
                        link["child"].as_str().map(|id| &id[..8]).unwrap_or("-"),
                        link["at"].as_str().unwrap()
                    )
                );
            }
        } else {
            assert_eq!(
                &lines[1..5],
                &[
                    "repos/repo-a  0.4  unknown  Parse sample records",
                    "repos/repo-a  0.5  -        Display records",
                    "repos/repo-b  3    unknown  Parse sample records",
                    "repos/repo-b  4    -        Display records",
                ]
            );
        }
    }
}

#[test]
fn invalid_flags_are_clap_errors() {
    for command in ["sessions", "links", "steps"] {
        for (flag, value) in [
            ("--since", ""),
            ("--since", "30"),
            ("--since", "-2h"),
            ("--since", "1.5h"),
            ("--since", "+2h"),
            ("--since", "2w"),
            ("--since", "18446744073709551615d"),
            ("--since", "é"),
            ("--now", "yesterday"),
            ("--now", "2026-10-29"),
        ] {
            let argument = format!("{flag}={value}");
            let output = run(command, &[&argument, "--json"]);
            assert_eq!(output.status.code(), Some(2), "{command} {flag} {value}");
            assert!(output.stdout.is_empty());
            let stderr = String::from_utf8(output.stderr).unwrap();
            assert!(stderr.starts_with("error:"), "{stderr}");
            assert!(stderr.contains(flag), "{stderr}");
        }
    }
}

#[test]
fn duration_units_and_offsets_use_the_same_options() {
    for since in ["3600s", "60m", "1h"] {
        let output = run(
            "sessions",
            &[
                "--since",
                since,
                "--now",
                "2026-10-30T00:15:00+01:00",
                "--json",
            ],
        );
        success(&output);
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        let mut answer = expected("sessions");
        answer["sessions"]
            .as_array_mut()
            .unwrap()
            .retain(|session| {
                !session["record"].is_null()
                    || session["last_activity"].as_str().unwrap() >= "2026-10-29T22:15:00.000Z"
            });
        assert_eq!(report, answer);
    }
    let output = run(
        "sessions",
        &["--since", "1d", "--now", "2026-10-29T23:15:00Z", "--json"],
    );
    success(&output);
    let mut answer = expected("sessions");
    answer["sessions"]
        .as_array_mut()
        .unwrap()
        .retain(|session| {
            !session["record"].is_null()
                || session["last_activity"].as_str().unwrap() >= "2026-10-28T23:15:00.000Z"
        });
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        answer
    );
}

#[test]
fn default_steps_repos_preserve_first_seen_absolute_strings() {
    let output = run(
        "steps",
        &["--since", "all", "--now", "2026-10-29T23:15:00Z", "--json"],
    );
    success(&output);
    let mut repos = Vec::new();
    for session in expected("sessions")["sessions"].as_array().unwrap() {
        if !repos.contains(&session["repo"]) {
            repos.push(session["repo"].clone());
        }
    }
    let actual: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        actual["repos"]
            .as_array()
            .unwrap()
            .iter()
            .map(|repo| repo["repo"].clone())
            .collect::<Vec<_>>(),
        repos
    );

    let output = run(
        "steps",
        &[
            "--repo",
            "repos/repo-b",
            "--repo",
            "repos/repo-a",
            "--repo",
            "repos/repo-b",
            "--json",
        ],
    );
    success(&output);
    let answer = expected("steps");
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        json!({"repos": [answer["repos"][1], answer["repos"][0], answer["repos"][1]]})
    );
}

#[test]
fn report_io_errors_exit_one_without_stdout() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join(".claude"), "not a directory").unwrap();
    for command in ["sessions", "links", "steps"] {
        let output = Command::new(env!("CARGO_BIN_EXE_multi-link"))
            .args([command, "--json", "--root"])
            .arg(root.path())
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .starts_with("multi-link: ")
        );
    }
}

#[test]
fn diagnostics_stay_on_stderr_and_do_not_fail_reports() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    let records = root.path().join(".claude/sessions");
    fs::create_dir_all(&records).unwrap();
    fs::write(records.join("bad.json"), "{}").unwrap();
    let projects = root.path().join(".claude/projects/project");
    fs::create_dir_all(&projects).unwrap();
    fs::write(
        projects.join("incomplete.jsonl"),
        "{\"type\":\"user\"}\ninvalid\n",
    )
    .unwrap();
    let codex = root.path().join(".codex/sessions");
    fs::create_dir_all(&codex).unwrap();
    let unreadable = codex.join("rollout-unreadable.jsonl");
    fs::write(&unreadable, "").unwrap();
    fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o000)).unwrap();
    for command in ["sessions", "links", "steps"] {
        let output = Command::new(env!("CARGO_BIN_EXE_multi-link"))
            .args([command, "--json", "--root"])
            .arg(root.path())
            .output()
            .unwrap();
        success(&output);
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            "bad_records: 1\nincomplete: 1\nunreadable: 1\n"
        );
        if command == "sessions" {
            assert_eq!(
                report["skipped"],
                json!([{"file": ".claude/projects/project/incomplete.jsonl", "line": 2, "reason": "invalid_json"}])
            );
        }
    }
}

#[test]
fn steps_text_shows_missing_and_unsupported_repos() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("unsupported")).unwrap();
    fs::write(root.path().join("unsupported/PLAN.md"), "# No steps\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_multi-link"))
        .args(["steps", "--root"])
        .arg(root.path())
        .args(["--repo", "missing", "--repo", "unsupported"])
        .output()
        .unwrap();
    success(&output);
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "REPO         ID  VERIFY       TITLE\nmissing      -   missing\nunsupported  -   unsupported\n0 steps in 2 repos\n"
    );
}

const HOOK_PARENT: &str = "11111111-1111-4111-8111-111111111111";
const HOOK_CHILD: &str = "abcdefab-1234-4321-8123-abcdef123456";

fn hook_payload() -> Value {
    json!({"session_id":HOOK_PARENT,"cwd":"/synthetic/repo","hook_event_name":"PostToolUse","tool_name":"Bash","tool_use_id":"tool-1","tool_input":{"command":"codex exec 'synthetic'"},"tool_response":{"stdout":format!("session id: {HOOK_CHILD}\n")}})
}

fn hook_command(home: &std::path::Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_multi-link"));
    command
        .env("HOME", home)
        .env_remove("MULTI_LINK_HOME")
        .env_remove("MULTI_LINK_HOOK_TEST");
    command
}

fn record_input(command: &mut Command, input: &[u8]) -> Output {
    use std::{io::Write, process::Stdio};
    let mut child = command
        .args(["hook", "record"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    let output = child.wait_with_output().unwrap();
    silent_success(&output);
    output
}

fn silent_success(output: &Output) {
    success(output);
    assert!(output.stdout.is_empty(), "{:?}", output.stdout);
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);
}

fn hook_lines(state: &std::path::Path) -> Vec<Value> {
    fs::read_to_string(state.join(format!("hook/{HOOK_PARENT}.jsonl")))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn hook_payload_shapes_write_exact_metadata_silently() {
    let mut cases = Vec::new();
    cases.push((
        hook_payload(),
        "handoff",
        Some(HOOK_CHILD),
        Some("stdout"),
        None,
    ));
    let mut resume = hook_payload();
    resume["tool_input"]["command"] = json!(format!("codex exec resume {HOOK_CHILD}"));
    cases.push((resume, "handoff", Some(HOOK_CHILD), Some("stdout"), None));
    let mut plugin = hook_payload();
    plugin["tool_input"]["command"] = json!("node /synthetic/codex-companion.mjs task 'synthetic'");
    plugin["agent_id"] = json!("synthetic-agent");
    plugin["tool_response"]["stdout"] = json!(format!("[codex] Thread ready ({HOOK_CHILD})"));
    cases.push((
        plugin,
        "handoff",
        Some(HOOK_CHILD),
        Some("thread_ready"),
        None,
    ));
    let mut failure = hook_payload();
    failure["hook_event_name"] = json!("PostToolUseFailure");
    failure.as_object_mut().unwrap().remove("tool_response");
    failure["error"] = json!(format!("synthetic failure\nsession id: {HOOK_CHILD}"));
    cases.push((failure, "handoff", Some(HOOK_CHILD), Some("stdout"), None));
    for event in ["PermissionRequest", "Notification"] {
        for notification in ["permission_prompt", "elicitation_dialog"] {
            let p = json!({"session_id":HOOK_PARENT,"cwd":"/synthetic/repo","hook_event_name":event,"tool_name":"Read","notification_type":notification});
            cases.push((
                p,
                "waiting",
                None,
                None,
                if event == "Notification" {
                    Some(notification)
                } else {
                    None
                },
            ));
        }
    }
    let mut no_child = hook_payload();
    no_child["tool_response"]["stdout"] = json!("");
    cases.push((no_child, "handoff", None, None, None));
    for (p, kind, child, evidence, notification) in cases {
        let home = tempfile::tempdir().unwrap();
        let before = std::time::SystemTime::now();
        record_input(
            &mut hook_command(home.path()),
            &serde_json::to_vec(&p).unwrap(),
        );
        let records = hook_lines(&home.path().join(".multi-link"));
        assert_eq!(records.len(), 1);
        let timestamp: multi_link_model::Timestamp =
            serde_json::from_value(records[0]["at"].clone()).unwrap();
        let millis = |time: std::time::SystemTime| {
            time.duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64
        };
        assert!(timestamp >= multi_link_model::Timestamp::from_epoch_ms(millis(before)).unwrap());
        assert!(
            timestamp
                <= multi_link_model::Timestamp::from_epoch_ms(millis(std::time::SystemTime::now()))
                    .unwrap()
        );
        assert_eq!(
            records[0],
            json!({"v":1,"kind":kind,"at":timestamp,"session_id":HOOK_PARENT,"agent_id":p.get("agent_id").cloned().unwrap_or(Value::Null),"event":p["hook_event_name"],"tool_use_id":if kind == "handoff" { json!("tool-1") } else { Value::Null },"cwd":"/synthetic/repo","child":child,"child_evidence":evidence,"notification_type":notification})
        );
    }
}

#[test]
fn background_record_writes_initial_and_task_output_to_override_state() {
    let home = tempfile::tempdir().unwrap();
    let tasks = home.path().join("tasks");
    fs::create_dir(&tasks).unwrap();
    fs::write(
        tasks.join("abc123.output"),
        format!("session id: {HOOK_CHILD}\n"),
    )
    .unwrap();
    let mut p = hook_payload();
    p["tool_input"]["run_in_background"] = json!(true);
    p["tool_response"] = json!({"stdout":"", "backgroundTaskId":"abc123"});
    p["scratchpad_dir"] = json!(home.path().join("scratchpad"));
    let state = home.path().join("custom-state");
    record_input(
        hook_command(home.path()).env("MULTI_LINK_HOME", &state),
        &serde_json::to_vec(&p).unwrap(),
    );
    let records = hook_lines(&state);
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["child"], Value::Null);
    let mut expected = records[0].clone();
    expected["child"] = json!(HOOK_CHILD);
    expected["child_evidence"] = json!("task_output");
    expected["at"] = records[1]["at"].clone();
    assert_eq!(records[1], expected);
    assert!(!home.path().join(".multi-link").exists());
}

#[test]
fn hook_record_errors_and_ignored_inputs_are_silent() {
    let home = tempfile::tempdir().unwrap();
    let mut non_codex = hook_payload();
    non_codex["tool_input"]["command"] = json!("echo synthetic");
    let mut oversized = serde_json::to_vec(&hook_payload()).unwrap();
    oversized.resize(1024 * 1024 + 1, b' ');
    for input in [
        b"{".to_vec(),
        vec![],
        vec![0xff],
        serde_json::to_vec(&non_codex).unwrap(),
        oversized,
    ] {
        record_input(&mut hook_command(home.path()), &input);
        assert_eq!(fs::read_dir(home.path()).unwrap().count(), 0);
    }
    record_input(
        hook_command(home.path()).env_remove("HOME"),
        &serde_json::to_vec(&hook_payload()).unwrap(),
    );
    assert_eq!(fs::read_dir(home.path()).unwrap().count(), 0);
    use std::os::unix::fs::PermissionsExt;
    let state = home.path().join("unwritable");
    fs::create_dir(&state).unwrap();
    fs::set_permissions(&state, fs::Permissions::from_mode(0o500)).unwrap();
    record_input(
        hook_command(home.path()).env("MULTI_LINK_HOME", &state),
        &serde_json::to_vec(&hook_payload()).unwrap(),
    );
    assert_eq!(fs::read_dir(&state).unwrap().count(), 0);
    fs::set_permissions(&state, fs::Permissions::from_mode(0o700)).unwrap();
}

#[test]
fn hook_panic_and_blocked_stdin_exit_zero_silently() {
    use std::{
        process::Stdio,
        time::{Duration, Instant},
    };
    let home = tempfile::tempdir().unwrap();
    let output = hook_command(home.path())
        .env("MULTI_LINK_HOOK_TEST", "panic")
        .args(["hook", "record"])
        .output()
        .unwrap();
    silent_success(&output);
    let start = Instant::now();
    let mut child = hook_command(home.path())
        .env("MULTI_LINK_HOOK_TEST", "deadline_ms=500")
        .args(["hook", "record"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stdin = child.stdin.take().unwrap(); // Keep open without writing.
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if start.elapsed() >= Duration::from_secs(2) {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("hook watchdog did not exit within 2 s");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    silent_success(&child.wait_with_output().unwrap());
    assert!(start.elapsed() >= Duration::from_millis(450));
    drop(stdin);
    assert_eq!(fs::read_dir(home.path()).unwrap().count(), 0);
}

fn snapshot(path: &std::path::Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut result = Vec::new();
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            result.push((path.clone(), Vec::new()));
            result.extend(snapshot(&path));
        } else {
            result.push((path.clone(), fs::read(path).unwrap()));
        }
    }
    result.sort();
    result
}

#[test]
fn install_uninstall_print_parseable_entries_without_changing_files() {
    let home = tempfile::tempdir().unwrap();
    let repo = home.path().join("repo");
    fs::create_dir(&repo).unwrap();
    assert!(
        Command::new("git")
            .arg("init")
            .arg(&repo)
            .output()
            .unwrap()
            .status
            .success()
    );
    let nested = repo.join("nested");
    fs::create_dir(&nested).unwrap();
    fs::create_dir(home.path().join(".claude")).unwrap();
    fs::write(
        home.path().join(".claude/settings.json"),
        "synthetic sentinel",
    )
    .unwrap();
    let before = snapshot(home.path());
    for action in ["install", "uninstall"] {
        for (scope, cwd, target) in [
            (None, &nested, home.path().join(".claude/settings.json")),
            (
                Some("user"),
                &nested,
                home.path().join(".claude/settings.json"),
            ),
            (
                Some("local"),
                &nested,
                repo.join(".claude/settings.local.json"),
            ),
            (
                Some("local"),
                &home.path().to_path_buf(),
                home.path().join(".claude/settings.local.json"),
            ),
        ] {
            let mut command = hook_command(home.path());
            command.current_dir(cwd).args(["hook", action]);
            if let Some(scope) = scope {
                command.args(["--scope", scope]);
            }
            let output = command.output().unwrap();
            success(&output);
            let entry = json!({"type":"command", "command":format!("'{}' hook record", env!("CARGO_BIN_EXE_multi-link").replace('\'', "'\\''")),"async":true,"timeout":60});
            assert_eq!(
                serde_json::from_slice::<Value>(&output.stdout).unwrap(),
                json!({"hooks":{
                    "PostToolUse":[{"matcher":"Bash","hooks":[entry.clone()]}],
                    "PostToolUseFailure":[{"matcher":"Bash","hooks":[entry.clone()]}],
                    "PermissionRequest":[{"hooks":[entry.clone()]}],
                    "Notification":[{"matcher":"permission_prompt|elicitation_dialog","hooks":[entry]}]
                }})
            );
            let stderr = String::from_utf8(output.stderr).unwrap();
            assert!(stderr.contains(target.to_str().unwrap()), "{stderr}");
            if action == "uninstall" {
                assert!(stderr.contains(home.path().join(".multi-link").to_str().unwrap()));
            }
            assert_eq!(snapshot(home.path()), before);
        }
    }
    let output = hook_command(home.path())
        .env("MULTI_LINK_HOME", home.path().join("custom"))
        .args(["hook", "uninstall"])
        .output()
        .unwrap();
    success(&output);
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains(home.path().join("custom").to_str().unwrap())
    );
    assert_eq!(snapshot(home.path()), before);
}

#[test]
fn setup_quotes_binary_paths_with_spaces_and_apostrophes() {
    let home = tempfile::tempdir().unwrap();
    let binary = home.path().join("multi link's binary");
    fs::copy(env!("CARGO_BIN_EXE_multi-link"), &binary).unwrap();
    let output = Command::new(&binary)
        .env("HOME", home.path())
        .args(["hook", "install"])
        .output()
        .unwrap();
    success(&output);
    let snippet: Value = serde_json::from_slice(&output.stdout).unwrap();
    let command = snippet["hooks"]["PermissionRequest"][0]["hooks"][0]["command"]
        .as_str()
        .unwrap();
    assert_eq!(
        command,
        format!(
            "'{}' hook record",
            binary.to_str().unwrap().replace('\'', "'\\''")
        )
    );
    let child = Command::new("sh")
        .args(["-c", command])
        .env("HOME", home.path())
        .env_remove("MULTI_LINK_HOME")
        .env_remove("MULTI_LINK_HOOK_TEST")
        .output()
        .unwrap();
    silent_success(&child);
}

#[test]
fn hook_conflicts_are_stderr_only_for_sessions_and_links() {
    fn copy_tree(source: &std::path::Path, target: &std::path::Path) {
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
    let root = tempfile::tempdir().unwrap();
    copy_tree(&fixtures().join("sessions"), root.path());
    copy_tree(&fixtures().join("hook"), root.path());
    fs::remove_file(root.path().join("task-output/be01y5dv6.output")).unwrap();
    for command in ["sessions", "links"] {
        let output = Command::new(env!("CARGO_BIN_EXE_multi-link"))
            .arg(command)
            .arg("--root")
            .arg(root.path())
            .args(["--now", "2026-10-29T23:15:00Z", "--since", "all", "--json"])
            .output()
            .unwrap();
        success(&output);
        assert_eq!(output.stderr, b"hook_conflicts: 1\n");
        let actual: Value = serde_json::from_slice(&output.stdout).unwrap();
        let answer: Value = serde_json::from_slice(
            &fs::read(fixtures().join(format!("expected/hook/{command}.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(actual, answer);
    }
}
