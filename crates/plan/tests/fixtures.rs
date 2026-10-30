use multi_link_model::{Convention, Entry, Step, Verify, VerifyResult, VerifySource};
use multi_link_plan::{parse_plan, parse_progress, steps_report};
use serde_json::Value;
use std::{fs, path::PathBuf};
use tempfile::tempdir;

fn step(id: &str, title: &str, command: Option<&str>) -> Step {
    Step {
        id: id.to_owned(),
        title: title.to_owned(),
        verify_command: command.map(str::to_owned),
        progress: Vec::new(),
    }
}

fn entry(agent: Option<&str>, command: Option<&str>, result: VerifyResult, text: &str) -> Entry {
    Entry {
        date: "2026-10-29".to_owned(),
        agent: agent.map(str::to_owned),
        verify: Verify {
            command: command.map(str::to_owned),
            result,
            result_text: text.to_owned(),
            source: VerifySource::Reported,
        },
    }
}

#[test]
fn fixture_report_matches_answer_key() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let report = steps_report(
        &fixtures.join("sessions"),
        &["repos/repo-a".to_owned(), "repos/repo-b".to_owned()],
    );
    let expected: Value =
        serde_json::from_slice(&fs::read(fixtures.join("expected/steps.json")).unwrap()).unwrap();
    assert_eq!(serde_json::to_value(report).unwrap(), expected);
}

#[test]
fn strict_ids_and_plan_order() {
    let input = "### Step 0.2ab — Invalid\n### Step 0.2A — Invalid\n\
                 ### Step 1.2.3 — Invalid\n### Step x — Invalid\n\
                 ### Step 1 - Wrong separator\n### Step 0.2a — Inserted\n\
                 ### Step 3b — Suffix\n### Step 0.10 — Ten\n### Step 0.1 — One\n";
    assert_eq!(
        parse_plan(input),
        Some(vec![
            step("0.2a", "Inserted", None),
            step("3b", "Suffix", None),
            step("0.10", "Ten", None),
            step("0.1", "One", None),
        ])
    );
    for id in ["", "a", "1.", ".1", "1..2", "1é", "١", "1 ", "-1"] {
        assert_eq!(parse_plan(&format!("### Step {id} — Invalid\n")), None);
    }
}

#[test]
fn fences_boundaries_crlf_and_first_verify() {
    let input = "```md\r\n### Step 8 — Fenced\r\n```\r\n\
                 ### Step 0.2a — Real\r\n```\r\n- Verify: `wrong`\r\n\
                 ### Step 9 — Also fenced\r\n```\r\n\
                   - **Verify:** `first` and `second`\r\n- Verify: `later`\r\n\
                 #### Detail\r\n### Step 0.10 — No command\r\n\
                 - Verify: no backticks\r\n- Verify: `ignored`\r\n\
                 ## Boundary\r\n- Verify: `outside`\r\n\
                 ### Step 0.1 — Ended\r\n# Boundary\r\n- Verify: `outside`\r\n";
    assert_eq!(
        parse_plan(input),
        Some(vec![
            step("0.2a", "Real", Some("first")),
            step("0.10", "No command", None),
            step("0.1", "Ended", None),
        ])
    );
    assert_eq!(parse_plan("```\n### Step 1 — Hidden\n"), None);
    assert_eq!(
        parse_plan("### Step 1 — Real\n### Step invalid\n- Verify: `outside`"),
        Some(vec![step("1", "Real", None)])
    );
}

#[test]
fn progress_results_are_reported_and_case_insensitive() {
    for (text, result) in [
        ("PASS 4 tests", VerifyResult::Pass),
        ("ok", VerifyResult::Pass),
        ("Ok: done", VerifyResult::Pass),
        ("FaIl: broken", VerifyResult::Fail),
        ("test result: ok", VerifyResult::Unknown),
        ("not run", VerifyResult::Unknown),
        ("", VerifyResult::Unknown),
    ] {
        let input = format!(
            "### 2026-10-29 — Step 0.2a: Title — Agent —  Codex  \n  - **Verify:** `cargo test` → {text}  \n- Verify: `ignored` → fail"
        );
        assert_eq!(
            parse_progress(&input),
            vec![(
                "0.2a".to_owned(),
                entry(Some("Codex"), Some("cargo test"), result, text)
            )]
        );
    }
}

#[test]
fn progress_missing_verify_and_invalid_headings() {
    let input = "### 2026-10-29 — Step 1: No verify\r\n\
                 ```\r\n- Verify: `hidden` → pass\r\n\
                 ### 2026-10-29 — Step 2: Hidden\r\n```\r\n\
                 ## Boundary\r\n- Verify: `outside` → pass\r\n\
                 ### 2026-10-29 — Step 3: No arrow\r\n- Verify: `check`\r\n\
                 ### 2026-10-29 — Step 4: No backticks\r\n- Verify: check → fail\r\n\
                 ### 2026-10-29 — Step 0.2ab: Invalid\r\n\
                 ### 2026-10-29 — Step 0.2A: Invalid\r\n\
                 ### 2026-10-29 — Step 1.2.3: Invalid\r\n\
                 ### 2026-1-29 — Step 5: Invalid date\r\n\
                 ### abcd-ef-gh — Step 5: Invalid date\r\n\
                 ### Notes\r\n- Verify: `outside` → pass\r\n";
    assert_eq!(
        parse_progress(input),
        vec![
            ("1".to_owned(), entry(None, None, VerifyResult::Unknown, "")),
            (
                "3".to_owned(),
                entry(None, Some("check"), VerifyResult::Unknown, "")
            ),
            (
                "4".to_owned(),
                entry(None, None, VerifyResult::Fail, "fail")
            ),
        ]
    );
}

#[test]
fn report_joins_exact_ids_and_keeps_latest_unmatched_entry() {
    let root = tempdir().unwrap();
    fs::write(
        root.path().join("PLAN.md"),
        "### Step 0.10 — Ten\n### Step 0.1 — One\n",
    )
    .unwrap();
    fs::write(
        root.path().join("PROGRESS.md"),
        "### 2026-10-29 — Step 99: Unmatched\n\
        ### 2026-10-29 — Step 0.1: One — Codex\n- Verify: `one` → pass\n\
        ### 2026-10-29 — Step 0.10: Ten\n- Verify: `ten` → fail\n\
        ### 2026-10-29 — Step 0.1: Earlier\n",
    )
    .unwrap();
    let report = steps_report(root.path(), &[".".to_owned()]);
    let repo = &report.repos[0];
    assert_eq!(repo.latest_logged_step.as_deref(), Some("99"));
    assert_eq!(
        repo.steps[0].progress,
        vec![entry(None, Some("ten"), VerifyResult::Fail, "fail")]
    );
    assert_eq!(
        repo.steps[1].progress,
        vec![
            entry(Some("Codex"), Some("one"), VerifyResult::Pass, "pass"),
            entry(None, None, VerifyResult::Unknown, ""),
        ]
    );
}

#[test]
fn absolute_repo_missing_progress_and_lossy_utf8() {
    let root = tempdir().unwrap();
    let repo = tempdir().unwrap();
    fs::write(
        repo.path().join("PLAN.md"),
        "### Step 1 — Title\n- Verify: `check`\n",
    )
    .unwrap();
    let path = repo.path().to_str().unwrap().to_owned();
    let report = steps_report(root.path(), std::slice::from_ref(&path));
    assert_eq!(report.repos[0].repo, path);
    assert_eq!(report.repos[0].convention, Convention::Supported);
    assert_eq!(report.repos[0].latest_logged_step, None);
    assert_eq!(
        report.repos[0].steps,
        vec![step("1", "Title", Some("check"))]
    );
    fs::write(
        repo.path().join("PLAN.md"),
        b"### Step 1 \xe2\x80\x94 Title\xff\r\n",
    )
    .unwrap();
    fs::write(
        repo.path().join("PROGRESS.md"),
        b"### 2026-10-29 \xe2\x80\x94 Step 1: Title\n- Verify: \xff \xe2\x86\x92 ok\xff",
    )
    .unwrap();
    let report = steps_report(root.path(), &[path]);
    assert_eq!(report.repos[0].steps[0].title, "Title\u{fffd}");
    assert_eq!(
        report.repos[0].steps[0].progress,
        vec![entry(None, None, VerifyResult::Pass, "ok\u{fffd}")]
    );
}

#[test]
fn missing_unsupported_and_unreadable_files() {
    let root = tempdir().unwrap();
    let repos = vec![
        "absent".to_owned(),
        "empty".to_owned(),
        "unsupported".to_owned(),
        "directory".to_owned(),
        "supported".to_owned(),
    ];
    for repo in &repos[1..] {
        fs::create_dir(root.path().join(repo)).unwrap();
    }
    fs::write(root.path().join("unsupported/PLAN.md"), "# Not a plan\n").unwrap();
    fs::write(
        root.path().join("unsupported/PROGRESS.md"),
        "### 2026-10-29 — Step 1: Unmatched\n",
    )
    .unwrap();
    fs::create_dir(root.path().join("directory/PLAN.md")).unwrap();
    fs::write(
        root.path().join("supported/PLAN.md"),
        "### Step 1 — Title\n",
    )
    .unwrap();
    fs::create_dir(root.path().join("supported/PROGRESS.md")).unwrap();
    let report = steps_report(root.path(), &repos);
    assert_eq!(
        report
            .repos
            .iter()
            .map(|repo| repo.repo.clone())
            .collect::<Vec<_>>(),
        repos
    );
    assert_eq!(
        report
            .repos
            .iter()
            .map(|repo| repo.convention)
            .collect::<Vec<_>>(),
        vec![
            Convention::Missing,
            Convention::Missing,
            Convention::Unsupported,
            Convention::Unsupported,
            Convention::Supported,
        ]
    );
    for repo in &report.repos {
        assert_eq!(repo.latest_logged_step, None);
        assert!(repo.steps.iter().all(|step| step.progress.is_empty()));
    }
    for repo in &report.repos[..4] {
        assert_eq!(repo.steps, Vec::new());
    }
}
