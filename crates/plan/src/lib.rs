//! Read-only PLAN.md / PROGRESS.md parsing and steps reports.

mod markdown;
mod plan;
mod progress;
mod tag;

pub use tag::step_tag;

pub use plan::parse_plan;
pub use progress::parse_progress;

use multi_link_model::{Convention, RepoSteps, StepsReport};
use std::{fs, io, path::Path};

/// Report repositories in caller order, preserving each supplied repository string.
pub fn steps_report(root: &Path, repos: &[String]) -> StepsReport {
    StepsReport {
        repos: repos.iter().map(|repo| repo_steps(root, repo)).collect(),
    }
}

fn repo_steps(root: &Path, repo: &str) -> RepoSteps {
    // Path::join preserves an absolute right-hand path.
    let folder = root.join(repo);
    let mut report = RepoSteps {
        repo: repo.to_owned(),
        convention: Convention::Unsupported,
        latest_logged_step: None,
        steps: Vec::new(),
    };
    let bytes = match fs::read(folder.join("PLAN.md")) {
        Ok(bytes) => bytes,
        Err(error) => {
            if error.kind() == io::ErrorKind::NotFound {
                report.convention = Convention::Missing;
            }
            return report;
        }
    };
    let Some(steps) = parse_plan(&String::from_utf8_lossy(&bytes)) else {
        return report;
    };
    report.convention = Convention::Supported;
    report.steps = steps;
    if let Ok(bytes) = fs::read(folder.join("PROGRESS.md")) {
        let entries = parse_progress(&String::from_utf8_lossy(&bytes));
        report.latest_logged_step = entries.first().map(|(id, _)| id.clone());
        for step in &mut report.steps {
            step.progress = entries
                .iter()
                .filter(|(id, _)| id == &step.id)
                .map(|(_, entry)| entry.clone())
                .collect();
        }
    }
    report
}
