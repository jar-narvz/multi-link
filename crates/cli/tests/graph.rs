use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

use serde_json::Value;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn graph(repo: &str, step: &str, json: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_multi-link"));
    command
        .arg("graph")
        .arg("--root")
        .arg(fixtures().join("sessions-phase8"))
        .args([
            "--now",
            "2026-11-06T18:00:00Z",
            "--repo",
            repo,
            "--step",
            step,
        ]);
    if json {
        command.arg("--json");
    }
    command.output().unwrap()
}

fn json(output: &Output) -> Value {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

const EMPTY_1_1: &str =
    "no hand-offs tagged step 1.1 (a Claude-only step, or its prompt did not name it)";

#[test]
fn empty_step_matches_key() {
    let key: Value = serde_json::from_slice(
        &fs::read(fixtures().join("expected/phase8/graph-1.13.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(json(&graph("/home/dev/repo-g", "1.13", true)), key);
}

#[test]
fn unknown_repo_gives_empty_sentence_and_nulls() {
    let report = json(&graph("/home/dev/nope", "1.1", true));
    assert_eq!(report["step"]["id"], "1.1");
    assert_eq!(report["step"]["title"], Value::Null);
    assert_eq!(report["step"]["verify"], Value::Null);
    assert_eq!(report["commit"], Value::Null);
    assert_eq!(report["parts"], Value::Array(vec![]));
    assert_eq!(report["empty"], EMPTY_1_1);
}

#[test]
fn text_mode_prints_header() {
    let output = graph("/home/dev/repo-g", "1.13", false);
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("› 1.13 Readme"), "{text}");
    assert!(text.contains("no hand-offs tagged step 1.13"), "{text}");
}
