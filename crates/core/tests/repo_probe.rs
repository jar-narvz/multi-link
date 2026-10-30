use std::path::Path;
use std::process::Command;

use multi_link_core::{
    FakeProcessProbe, FixtureProcessProbe, Liveness, ProcessProbe, RealProcessProbe, RepoMode,
    RepoPaths,
};

fn git(cwd: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_COMMON_DIR")
        .output()
        .expect("git must be installed for repo mapper tests");
    assert!(output.status.success(), "git {args:?}: {output:?}");
}

#[test]
fn nested_main_and_worktree_map_to_shared_repo() {
    let temp = tempfile::tempdir().unwrap();
    let main = temp.path().join("main");
    let worktree = temp.path().join("worktree");
    std::fs::create_dir(&main).unwrap();
    git(&main, &["init"]);
    git(&main, &["config", "user.name", "Test User"]);
    git(&main, &["config", "user.email", "test@example.invalid"]);
    git(
        &main,
        &[
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--allow-empty",
            "-m",
            "Initial",
        ],
    );
    git(
        &main,
        &[
            "worktree",
            "add",
            "-b",
            "test-worktree",
            worktree.to_str().unwrap(),
        ],
    );
    let main = main.canonicalize().unwrap();
    let worktree = worktree.canonicalize().unwrap();
    for checkout in [&main, &worktree] {
        let nested = checkout.join("nested/subfolder");
        std::fs::create_dir_all(&nested).unwrap();
        assert_eq!(
            RepoMode::Real.map(&nested),
            RepoPaths {
                repo: main.clone(),
                checkout: checkout.clone(),
            }
        );
        assert_eq!(
            RepoMode::Fixture.map(&nested),
            RepoPaths {
                repo: nested.clone(),
                checkout: nested,
            }
        );
    }
}

#[test]
fn fixture_mode_preserves_even_nonexistent_relative_paths() {
    let cwd = Path::new("nonexistent/fixture/../checkout");
    assert_eq!(
        RepoMode::Fixture.map(cwd),
        RepoPaths {
            repo: cwd.to_owned(),
            checkout: cwd.to_owned(),
        }
    );
}

#[test]
fn real_mapper_falls_back_outside_git_or_for_missing_directory() {
    let temp = tempfile::tempdir().unwrap();
    for cwd in [temp.path().to_owned(), temp.path().join("missing")] {
        assert_eq!(
            RepoMode::Real.map(&cwd),
            RepoPaths {
                repo: cwd.clone(),
                checkout: cwd,
            }
        );
    }
}

#[test]
fn mapper_ignores_inherited_git_environment() {
    const TEST_ROOT: &str = "MULTI_LINK_TEST_REPO_ROOT";
    if let Some(root) = std::env::var_os(TEST_ROOT) {
        let root = std::path::PathBuf::from(root);
        assert_eq!(
            RepoMode::Real.map(&root.join("nested")),
            RepoPaths {
                repo: root.clone(),
                checkout: root,
            }
        );
        return;
    }

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    git(&root, &["init"]);
    std::fs::create_dir(root.join("nested")).unwrap();
    // Isolate environment changes in a child test process so parallel tests
    // cannot observe or race with them.
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "mapper_ignores_inherited_git_environment"])
        .env(TEST_ROOT, &root)
        .env("GIT_DIR", root.join("missing-git-dir"))
        .env("GIT_WORK_TREE", root.join("wrong-worktree"))
        .env("GIT_COMMON_DIR", root.join("missing-common-dir"))
        .env("GIT_INDEX_FILE", root.join("missing-index"))
        .env("GIT_CONFIG_PARAMETERS", "invalid config parameters")
        .env("GIT_CEILING_DIRECTORIES", &root)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
}

#[test]
fn fake_and_fixture_probes_are_deterministic() {
    let mut fake = FakeProcessProbe::default();
    fake.results.insert((12, "start".into()), Liveness::Alive);
    fake.results
        .insert((12, "old start".into()), Liveness::Gone);
    assert_eq!(fake.probe(12, "start"), Liveness::Alive);
    assert_eq!(fake.probe(12, "old start"), Liveness::Gone);
    assert_eq!(fake.probe(13, "start"), Liveness::Unavailable);
    assert_eq!(
        FixtureProcessProbe.probe(12, "start"),
        Liveness::Unavailable
    );
}

#[test]
fn missing_ps_is_unavailable() {
    let temp = tempfile::tempdir().unwrap();
    assert_eq!(
        RealProcessProbe::new(temp.path().join("missing-ps")).probe(std::process::id(), "start"),
        Liveness::Unavailable
    );
    assert_eq!(
        RealProcessProbe::default().probe(0, "start"),
        Liveness::Unavailable
    );
}

#[test]
#[cfg(unix)]
fn probe_uses_utc_start_time_instead_of_local_rendering() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let ps = temp.path().join("ps");
    std::fs::write(
        &ps,
        "#!/bin/sh\nif [ \"$TZ\" = UTC ]; then\n  echo 'Fri Oct 30 00:08:22 2026'\nelse\n  echo 'Thu Oct 29 17:08:22 2026'\nfi\n",
    )
    .unwrap();
    std::fs::set_permissions(&ps, std::fs::Permissions::from_mode(0o700)).unwrap();
    let probe = RealProcessProbe::new(ps);
    assert_eq!(probe.probe(12, "Fri Oct 30 00:08:22 2026"), Liveness::Alive);
    assert_eq!(probe.probe(12, "Thu Oct 29 17:08:22 2026"), Liveness::Gone);
}

#[test]
fn own_pid_matches_start_time_and_rejects_reused_identity() {
    let probe = RealProcessProbe::default();
    let pid = std::process::id();
    if probe.probe(pid, "wrong start") == Liveness::Unavailable {
        eprintln!("SKIP: real ps cannot probe the test's own pid in this environment");
        return;
    }
    let output = Command::new("/bin/ps")
        .args(["-p", &pid.to_string(), "-o", "lstart="])
        .env("TZ", "UTC")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let start = String::from_utf8(output.stdout).unwrap();
    assert!(!start.trim().is_empty());
    assert_eq!(probe.probe(pid, &start), Liveness::Alive);
    assert_eq!(probe.probe(pid, "wrong start"), Liveness::Gone);
}

#[test]
fn reaped_child_is_gone() {
    let probe = RealProcessProbe::default();
    if probe.probe(std::process::id(), "wrong start") == Liveness::Unavailable {
        eprintln!("SKIP: real ps cannot probe the test's own pid; reaped-child check unavailable");
        return;
    }
    let mut child = Command::new("/usr/bin/true").spawn().unwrap();
    let pid = child.id();
    assert!(child.wait().unwrap().success());
    assert_eq!(probe.probe(pid, "any start"), Liveness::Gone);
}
