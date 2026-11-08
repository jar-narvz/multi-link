//! Commit lookup for the graph report, against a real temporary git repo.

use std::{path::Path, process::Command};

use multi_link_core::commit_for_step;

fn git(dir: &Path, date: &str, args: &[&str]) {
    let mut command = Command::new("git");
    for (name, _) in std::env::vars_os() {
        if name.as_encoded_bytes().starts_with(b"GIT_") {
            command.env_remove(name);
        }
    }
    let status = command
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "T")
        .env("GIT_AUTHOR_EMAIL", "t@example.com")
        .env("GIT_COMMITTER_NAME", "T")
        .env("GIT_COMMITTER_EMAIL", "t@example.com")
        .env("GIT_AUTHOR_DATE", date)
        .env("GIT_COMMITTER_DATE", date)
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?}");
}

fn head(dir: &Path) -> String {
    let out = Command::new("git")
        .current_dir(dir)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

#[test]
fn newest_exact_step_commit() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    git(repo, "2026-11-01T10:00:00Z", &["init", "-q"]);
    let mut hashes = Vec::new();
    for (subject, date) in [
        ("A (Step 6.7)", "2026-11-01T10:00:00Z"),
        ("B (Step 6.70)", "2026-11-02T10:00:00Z"),
        ("C (Step 6.7)", "2026-11-03T10:00:00Z"),
    ] {
        git(
            repo,
            date,
            &["commit", "-q", "--allow-empty", "-m", subject],
        );
        hashes.push(head(repo));
    }

    let c = commit_for_step(repo, "6.7").unwrap();
    assert_eq!(c.hash.len(), 7);
    assert_eq!(c.hash, hashes[2][..7]);
    assert_eq!(
        c.at,
        multi_link_model::Timestamp::parse("2026-11-03T10:00:00Z").unwrap()
    );

    let b = commit_for_step(repo, "6.70").unwrap();
    assert_eq!(b.hash, hashes[1][..7]);

    assert_eq!(commit_for_step(repo, "6.8"), None);
    // `6.7` never matches `(Step 6.70)` alone, and `6` matches neither.
    assert_eq!(commit_for_step(repo, "6"), None);

    let plain = tempfile::tempdir().unwrap();
    assert_eq!(commit_for_step(plain.path(), "6.7"), None);
}
