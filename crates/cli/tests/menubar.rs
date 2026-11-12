use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

#[test]
fn menubar_matches_claudes_three_keys_byte_for_byte() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (name, fixture, now) in [
        ("multi-repo", "sessions-phase14", "2026-11-10T22:20:00Z"),
        ("waiting-quiet", "sessions-phase9", "2026-11-09T07:30:00Z"),
        ("open-actions", "sessions-phase18", "2026-11-12T09:00:00Z"),
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
