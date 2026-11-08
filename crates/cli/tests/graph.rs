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

const SENTINELS: [&str; 4] = [
    "sentinel-path-81",
    "sentinel-token-81",
    "sentinel-path-82",
    "sentinel-token-82",
];

/// `(step id, key file)` for every Phase 8 key.
fn phase8_keys() -> Vec<(String, PathBuf)> {
    let mut keys: Vec<(String, PathBuf)> = fs::read_dir(fixtures().join("expected/phase8"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .map(|path| {
            let name = path.file_stem().unwrap().to_str().unwrap().to_string();
            (name.strip_prefix("graph-").unwrap().to_string(), path)
        })
        .collect();
    keys.sort();
    keys
}

/// The repo each key was built for, read from the key itself.
fn key_repo(key: &Value) -> String {
    key["repo"].as_str().unwrap().to_string()
}

#[test]
fn every_phase8_key_matches() {
    let keys = phase8_keys();
    assert_eq!(keys.len(), 13);
    for (id, path) in keys {
        let key: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let actual = json(&graph(&key_repo(&key), &id, true));
        assert!(
            actual == key,
            "step {id} differs\nactual:\n{}\nkey:\n{}",
            serde_json::to_string_pretty(&actual).unwrap(),
            serde_json::to_string_pretty(&key).unwrap()
        );
    }
}

#[test]
fn sentinels_never_in_output() {
    for (id, path) in phase8_keys() {
        let key: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let repo = key_repo(&key);
        for json_mode in [true, false] {
            let output = graph(&repo, &id, json_mode);
            let text = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            for sentinel in SENTINELS {
                assert!(
                    !text.contains(sentinel),
                    "{sentinel} in step {id} (json {json_mode})"
                );
            }
        }
    }
}
