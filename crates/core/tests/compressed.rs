use std::{fs, path::PathBuf};

use multi_link_core::{FixtureProcessProbe, Options, RepoMode, Scanner};
use multi_link_model::Timestamp;
use serde_json::Value;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn options() -> Options<'static> {
    Options {
        state_dir: PathBuf::new(),
        now: Timestamp::parse("2026-11-09T12:00:00Z").unwrap(),
        since: None,
        repo_mode: RepoMode::Fixture,
        probe: &FixtureProcessProbe,
    }
}

#[test]
fn compressed_graph_matches_key() {
    let graph = Scanner::default()
        .graph(
            &fixtures().join("sessions-phase9"),
            &options(),
            "/home/dev/repo-p9a",
            "1.1",
            &mut |folder| {
                multi_link_plan::steps_report(folder, &[".".into()])
                    .repos
                    .remove(0)
            },
        )
        .unwrap();
    let key: Value = serde_json::from_slice(
        &fs::read(fixtures().join("expected/phase9/graph-1.1.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(serde_json::to_value(graph).unwrap(), key);
}

#[test]
fn scanner_caches_compressed_rollouts_and_invalidates_rewrites_and_plain_preference() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join(".codex/sessions");
    fs::create_dir_all(&dir).unwrap();
    let source =
        multi_link_codex::rollout_paths(&fixtures().join("sessions-phase9/.codex/sessions"))
            .unwrap()
            .into_iter()
            .find(|p| p.to_string_lossy().contains("91c00001"))
            .unwrap();
    let compressed = dir.join("rollout-child.jsonl.zst");
    fs::copy(source, &compressed).unwrap();
    let mut scanner = Scanner::default();
    let first = scanner.sessions(root.path(), &options()).unwrap();
    assert_eq!(first.report.sessions.len(), 1);
    assert!(first.unreadable.is_empty());
    assert_eq!(scanner.stats.files, 1);
    let warm = scanner.sessions(root.path(), &options()).unwrap();
    assert_eq!(first.report, warm.report);
    assert_eq!(scanner.stats.files, 0);
    fs::write(&compressed, b"corrupt").unwrap();
    let corrupt = scanner.sessions(root.path(), &options()).unwrap();
    assert_eq!(corrupt.report.sessions.len(), 0);
    assert_eq!(corrupt.unreadable.len(), 1);
    let plain = compressed.with_extension("");
    fs::write(&plain, "{\"type\":\"session_meta\",\"timestamp\":\"2026-11-09T09:00:45Z\",\"payload\":{\"id\":\"91c00001-0000-4000-8000-000091c00001\",\"cwd\":\"/plain\"}}\n").unwrap();
    let preferred = scanner.sessions(root.path(), &options()).unwrap();
    assert!(preferred.unreadable.is_empty());
    assert_eq!(preferred.report.sessions.len(), 1);
    assert_eq!(preferred.report.sessions[0].cwd, "/plain");
    fs::remove_file(plain).unwrap();
    let fallback = scanner.sessions(root.path(), &options()).unwrap();
    assert!(fallback.report.sessions.is_empty());
    assert_eq!(fallback.unreadable.len(), 1);
}
