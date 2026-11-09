use std::{fs, path::PathBuf, process::Command};

use serde_json::Value;

fn check(command: &str, key: &str, extra: &[&str]) {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let output = Command::new(env!("CARGO_BIN_EXE_multi-link"))
        .arg(command)
        .arg("--root")
        .arg(fixtures.join("sessions-phase9"))
        .args(["--now", "2026-11-09T12:00:00Z", "--json"])
        .args(extra)
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    let actual: Value = serde_json::from_slice(&output.stdout).unwrap();
    let expected: Value =
        serde_json::from_slice(&fs::read(fixtures.join("expected/phase9").join(key)).unwrap())
            .unwrap();
    assert_eq!(actual, expected, "{key}");
}

#[test]
fn phase9_links_match_answer_key() {
    check("links", "links.json", &["--since", "all"]);
}

#[test]
fn phase9_graph_fix_rounds_match_answer_key() {
    check(
        "graph",
        "graph-2.1.json",
        &["--repo", "/home/dev/repo-p9r", "--step", "2.1"],
    );
}
