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
