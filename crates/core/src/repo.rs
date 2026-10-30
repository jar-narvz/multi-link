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
