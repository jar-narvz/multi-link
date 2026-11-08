use std::path::{Path, PathBuf};
use std::process::Command;

/// Paths identifying the shared main repository and the current checkout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoPaths {
    pub repo: PathBuf,
    pub checkout: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoMode {
    Real,
    Fixture,
}

impl RepoMode {
    /// Fixture paths and failed git lookups retain the supplied cwd verbatim.
    pub fn map(self, cwd: &Path) -> RepoPaths {
        let fallback = || RepoPaths {
            repo: cwd.to_path_buf(),
            checkout: cwd.to_path_buf(),
        };
        if self == Self::Fixture {
            return fallback();
        }
        let mut command = Command::new("git");
        for (name, _) in std::env::vars_os() {
            if name.as_encoded_bytes().starts_with(b"GIT_") {
                command.env_remove(name);
            }
        }
        let Ok(output) = command
            .arg("-C")
            .arg(cwd)
            .args([
                "rev-parse",
                "--path-format=absolute",
                "--show-toplevel",
                "--git-common-dir",
            ])
            .output()
        else {
            return fallback();
        };
        if !output.status.success() {
            return fallback();
        }
        let Ok(stdout) = std::str::from_utf8(&output.stdout) else {
            return fallback();
        };
        let mut lines = stdout.lines();
        let (Some(checkout), Some(common_dir)) = (lines.next(), lines.next()) else {
            return fallback();
        };
        let checkout = Path::new(checkout);
        let common_dir = Path::new(common_dir);
        if !checkout.is_absolute() || !common_dir.is_absolute() || lines.next().is_some() {
            return fallback();
        }
        RepoPaths {
            repo: repo_root(checkout, common_dir).to_path_buf(),
            checkout: checkout.to_path_buf(),
        }
    }
}

fn repo_root<'a>(checkout: &'a Path, common_dir: &'a Path) -> &'a Path {
    if common_dir.file_name().is_some_and(|name| name == ".git") {
        common_dir.parent().unwrap_or(checkout)
    } else {
        checkout
    }
}

#[cfg(test)]
mod tests {
    use super::repo_root;
    use std::path::Path;

    #[test]
    fn groups_only_common_dirs_named_dot_git() {
        let checkout = Path::new("/repos/checkout");
        for (common_dir, expected) in [
            ("/repos/main/.git", "/repos/main"),
            ("/repos/super/.git/modules/x", "/repos/checkout"),
            ("/repos/repo.git", "/repos/checkout"),
        ] {
            assert_eq!(
                repo_root(checkout, Path::new(common_dir)),
                Path::new(expected)
            );
        }
    }
}

/// Presentation metadata is deliberately excluded from Session/SessionRow JSON.
/// Like the scanner's repo mapping, branch lookups are memoised per checkout.
pub(crate) fn place(mode: RepoMode, session: &multi_link_model::Session) -> Option<String> {
    use std::{
        collections::HashMap,
        sync::{Mutex, OnceLock},
    };
    if mode == RepoMode::Fixture {
        return None;
    }
    if session.checkout != session.repo {
        return Path::new(&session.checkout)
            .file_name()
            .map(|name| format!("worktree {}", name.to_string_lossy()));
    }
    // A removed Claude worktree maps back to its own cwd; its path still names it.
    let parts: Vec<_> = Path::new(&session.checkout).components().collect();
    if let Some(name) = parts.windows(3).find_map(|w| {
        (w[0].as_os_str() == ".claude" && w[1].as_os_str() == "worktrees")
            .then(|| w[2].as_os_str().to_string_lossy().into_owned())
    }) {
        return Some(format!("worktree {name}"));
    }
    #[derive(Default)]
    struct Branch {
        head: Option<String>,
        name: Option<String>,
    }
    static BRANCHES: OnceLock<Mutex<HashMap<String, Branch>>> = OnceLock::new();
    let mut branches = BRANCHES
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let head = std::fs::read_to_string(Path::new(&session.checkout).join(".git/HEAD")).ok();
    let refresh = branches
        .get(&session.checkout)
        .is_none_or(|cached| cached.head != head);
    let cached = branches.entry(session.checkout.clone()).or_default();
    if refresh {
        cached.name = (|| {
            let mut command = Command::new("git");
            for (name, _) in std::env::vars_os() {
                if name.as_encoded_bytes().starts_with(b"GIT_") {
                    command.env_remove(name);
                }
            }
            let output = command
                .arg("-C")
                .arg(&session.checkout)
                .args(["symbolic-ref", "--quiet", "--short", "HEAD"])
                .output()
                .ok()?;
            if !output.status.success() {
                return None;
            }
            let branch = String::from_utf8(output.stdout).ok()?.trim().to_owned();
            (!branch.is_empty()).then_some(branch)
        })();
        cached.head = head;
    }
    cached.name.clone().or_else(|| session.branch.clone())
}

#[cfg(test)]
mod place_tests {
    use super::*;
    use multi_link_model::{Harness, SessionsReport};

    fn git(path: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success(), "git {args:?}: {output:?}");
    }

    #[test]
    fn current_main_branch_and_worktree_folder_ignore_recorded_branch() {
        let temp = tempfile::tempdir().unwrap();
        let main = temp.path().join("main");
        let tree = temp.path().join("step-8.4b");
        std::fs::create_dir(&main).unwrap();
        git(&main, &["init", "-b", "current-branch"]);
        git(
            &main,
            &[
                "-c",
                "user.name=Synthetic",
                "-c",
                "user.email=test@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "--allow-empty",
                "-m",
                "Synthetic",
            ],
        );
        git(
            &main,
            &[
                "worktree",
                "add",
                "-b",
                "worktree-branch",
                tree.to_str().unwrap(),
            ],
        );
        let report: SessionsReport = serde_json::from_str(include_str!(
            "../../../fixtures/expected/phase6/sessions.json"
        ))
        .unwrap();
        let mut session = report.sessions[0].clone();
        session.branch = Some("main".into());
        for harness in [Harness::Claude, Harness::Codex] {
            session.harness = harness;
            let paths = RepoMode::Real.map(&main);
            session.repo = paths.repo.to_string_lossy().into_owned();
            session.checkout = paths.checkout.to_string_lossy().into_owned();
            assert_eq!(
                place(RepoMode::Real, &session).as_deref(),
                Some("current-branch")
            );
            let paths = RepoMode::Real.map(&tree);
            session.checkout = paths.checkout.to_string_lossy().into_owned();
            assert_eq!(
                place(RepoMode::Real, &session).as_deref(),
                Some("worktree step-8.4b")
            );
            assert_eq!(session.branch.as_deref(), Some("main"));
            assert_eq!(place(RepoMode::Fixture, &session), None);
        }
        session.checkout = session.repo.clone();
        git(&main, &["branch", "-m", "renamed-branch"]);
        assert_eq!(
            place(RepoMode::Real, &session).as_deref(),
            Some("renamed-branch")
        );
        let missing = temp.path().join("missing").to_string_lossy().into_owned();
        session.repo = missing.clone();
        session.checkout = missing;
        assert_eq!(place(RepoMode::Real, &session).as_deref(), Some("main"));
        let removed = temp
            .path()
            .join("main/.claude/worktrees/step-8.4/crates")
            .to_string_lossy()
            .into_owned();
        session.repo = removed.clone();
        session.checkout = removed;
        assert_eq!(
            place(RepoMode::Real, &session).as_deref(),
            Some("worktree step-8.4")
        );
    }
}
