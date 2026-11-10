//! Phase 12 answer keys (Step 12.0, Claude): nearest-run links and what they change downstream.
use std::{fs, path::PathBuf, process::Command};

use serde_json::{Value, json};

fn run(command: &str, extra: &[&str]) -> Value {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let output = Command::new(env!("CARGO_BIN_EXE_multi-link"))
        .arg(command)
        .arg("--root")
        .arg(fixtures.join("sessions-phase12"))
        .args(["--now", "2026-11-10T12:00:00Z", "--json"])
        .args(extra)
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    serde_json::from_slice(&output.stdout).unwrap()
}

fn key(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/expected/phase12")
        .join(name);
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

/// Same projection as `OV` in `fixtures/expected/phase12/check.sh`.
fn handoff(link: &Value) -> Value {
    json!({
        "invocation": link["invocation"],
        "match": link["match"],
        "child": link["child"],
        "evidence": link["evidence"],
    })
}

fn by(field: &'static str) -> impl FnMut(&Value, &Value) -> std::cmp::Ordering {
    move |a, b| a[field].as_str().cmp(&b[field].as_str())
}

fn tree(row: &Value) -> Value {
    let mut children: Vec<_> = row["children"]
        .as_array()
        .map(|rows| rows.iter().map(tree).collect())
        .unwrap_or_default();
    children.sort_by(by("id"));
    let mut handoffs: Vec<_> = row["handoffs"]
        .as_array()
        .unwrap()
        .iter()
        .map(handoff)
        .collect();
    handoffs.sort_by(by("invocation"));
    json!({"id": row["id"], "children": children, "handoffs": handoffs})
}

fn overview_projection(overview: &Value) -> Value {
    let wanted = ["a", "e", "f2", "i"].map(|case| format!("/home/dev/repo-p12{case}"));
    let mut repos: Vec<_> = overview["repos"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|repo| wanted.iter().any(|name| repo["repo"] == name.as_str()))
        .map(|repo| {
            let mut rows: Vec<_> = repo["rows"].as_array().unwrap().iter().map(tree).collect();
            rows.sort_by(by("id"));
            json!({"repo": repo["repo"], "latest": handoff(&repo["latest_handoff"]), "rows": rows})
        })
        .collect();
    repos.sort_by(by("repo"));
    json!({ "repos": repos })
}

#[test]
fn phase12_links_match_answer_key() {
    assert_eq!(run("links", &["--since", "all"]), key("links.json"));
}

#[test]
fn phase12_overview_reparents_nearest_runs() {
    let overview = run("overview", &["--since", "all"]);
    assert_eq!(overview_projection(&overview), key("overview.json"));
}

#[test]
fn phase12_graph_uses_the_nearest_run() {
    assert_eq!(
        run("graph", &["--repo", "/home/dev/repo-p12a", "--step", "1.1"]),
        key("graph-1.1.json")
    );
}
