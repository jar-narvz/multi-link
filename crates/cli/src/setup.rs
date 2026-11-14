//! Read-only installation report and explicit, prompted hook setup.
use crate::{
    hook::{self, Scope},
    settings_write::{self, Snapshot},
};
use clap::Args;
use serde_json::{Value, json};
use std::{
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
};

type Error = Box<dyn std::error::Error>;
#[derive(Args)]
pub struct SetupArgs {
    /// Print a read-only versioned report.
    #[arg(long, conflicts_with = "yes")]
    json: bool,
    /// Apply safe hook fixes without asking.
    #[arg(long)]
    yes: bool,
}

fn cli_state(binary: &Path) -> &'static str {
    let text = binary.to_string_lossy();
    if binary.starts_with("/Volumes") || text.contains("/AppTranslocation/") {
        "temporary"
    } else {
        "ok"
    }
}
fn app_item(binary: &Path, home: &Path) -> Value {
    let app = binary
        .parent()
        .filter(|p| p.file_name().is_some_and(|n| n == "MacOS"))
        .and_then(Path::parent)
        .filter(|p| p.file_name().is_some_and(|n| n == "Contents"))
        .and_then(Path::parent)
        .filter(|p| p.extension().is_some_and(|n| n == "app"))
        .filter(|_| binary.file_name().is_some_and(|n| n == "multi-link"));
    match app {
        Some(path) => {
            json!({"id":"app", "state":if path.parent() == Some(Path::new("/Applications")) || path.parent() == Some(home.join("Applications").as_path()) { "installed" } else { "elsewhere" }, "path":path})
        }
        None => json!({"id":"app","state":"none"}),
    }
}
fn hooks_item(
    file: &Path,
    exists: bool,
    check: Option<&multi_link_hook::settings::Check>,
) -> Value {
    match check {
        None => json!({"id":"hooks","state":"unreadable","file":file,"exists":exists,"events":[]}),
        Some(check) => {
            let state = if check.needs_fix {
                "fixable"
            } else if check
                .findings
                .iter()
                .any(|f| ["customized", "unknown event"].contains(&f.status.as_str()))
            {
                "attention"
            } else {
                "ok"
            };
            json!({"id":"hooks","state":state,"file":file,"exists":exists,"events":check.findings})
        }
    }
}
fn report(binary: &Path, home: &Path, hooks: Value) -> Value {
    json!({"v":1,"binary":binary,"items":[{"id":"cli","state":cli_state(binary)}, hooks, app_item(binary, home)]})
}
fn exit_code(report: &Value) -> i32 {
    i32::from(report["items"].as_array().unwrap().iter().any(|item| {
        matches!(
            item["state"].as_str(),
            Some("temporary" | "fixable" | "unreadable")
        )
    }))
}

struct Inspection {
    hooks: Value,
    source: Option<Snapshot>,
}
fn inspect(binary: &Path) -> Result<Inspection, Error> {
    let target = hook::settings_target(Scope::User)?;
    let absolute = if target.is_absolute() {
        target
    } else {
        std::env::current_dir()?.join(target)
    };
    hook::path_text(&absolute)?;
    let file = hook::resolved_settings(&absolute).unwrap_or(absolute);
    hook::path_text(&file)?;
    let exists = std::fs::symlink_metadata(&file).is_ok();
    let source = hook::read_settings(Scope::User, Some(file.clone())).ok();
    let check = source.as_ref().and_then(|s| hook::inspect(s, binary).ok());
    Ok(Inspection {
        hooks: hooks_item(&file, exists, check.as_ref()),
        source,
    })
}

pub fn run(args: SetupArgs) -> Result<i32, Error> {
    let binary = std::env::current_exe()?.canonicalize()?;
    hook::path_text(&binary)?;
    let home = PathBuf::from(std::env::var_os("HOME").ok_or("HOME is not set")?);
    // current_exe is canonical; compare ~/Applications in the same namespace.
    let home = home.canonicalize().unwrap_or(home);
    let inspection = inspect(&binary)?;
    let initial = report(&binary, &home, inspection.hooks);
    if args.json {
        super::json(&mut io::stdout().lock(), &initial)?;
        return Ok(exit_code(&initial));
    }
    println!(
        "cli: {} ({})",
        initial["items"][0]["state"].as_str().unwrap(),
        binary.display()
    );
    let hooks = &initial["items"][1];
    let fixable = hooks["state"] == "fixable";
    let temporary = cli_state(&binary) == "temporary";
    let backup = if fixable && !temporary {
        Some(settings_write::backup_path(
            &inspection
                .source
                .as_ref()
                .ok_or("missing settings snapshot")?
                .path,
        )?)
    } else {
        None
    };
    let backup_text = backup
        .as_ref()
        .map(|p| {
            if hooks["exists"] == true {
                format!("; backup: {}", p.display())
            } else {
                "; new file, no backup".into()
            }
        })
        .unwrap_or_default();
    println!(
        "hooks: {} ({}{})",
        hooks["state"].as_str().unwrap(),
        hooks["file"].as_str().unwrap(),
        backup_text
    );
    let app = &initial["items"][2];
    println!(
        "app: {}{}",
        app["state"].as_str().unwrap(),
        app["path"]
            .as_str()
            .map(|p| format!(" ({p})"))
            .unwrap_or_default()
    );
    if temporary {
        eprintln!("move multi-link to Applications first");
        return Ok(1);
    }
    if let Some(backup) = backup {
        let accepted = if args.yes {
            true
        } else if io::stdin().is_terminal() {
            print!("Apply Claude hooks? [y/N] ");
            io::stdout().flush()?;
            let mut answer = String::new();
            io::stdin().read_line(&mut answer)?;
            matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes")
        } else {
            false
        };
        if accepted {
            hook::apply(inspection.source.as_ref().unwrap(), &binary, &backup)?;
            println!("start a new Claude session to load them");
            return Ok(exit_code(&report(&binary, &home, inspect(&binary)?.hooks)));
        }
    }
    Ok(exit_code(&initial))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paths_states_and_exit_codes_match_e3() {
        let home = Path::new("/home/test");
        for (path, cli, app) in [
            (
                "/Applications/multi-link.app/Contents/MacOS/multi-link",
                "ok",
                "installed",
            ),
            (
                "/home/test/Applications/multi-link.app/Contents/MacOS/multi-link",
                "ok",
                "installed",
            ),
            (
                "/tmp/multi-link.app/Contents/MacOS/multi-link",
                "ok",
                "elsewhere",
            ),
            (
                "/Applications/nested/multi-link.app/Contents/MacOS/multi-link",
                "ok",
                "elsewhere",
            ),
            (
                "/Volumes/Install/multi-link.app/Contents/MacOS/multi-link",
                "temporary",
                "elsewhere",
            ),
            (
                "/private/var/folders/x/AppTranslocation/random/d/multi-link.app/Contents/MacOS/multi-link",
                "temporary",
                "elsewhere",
            ),
            ("/home/test/.cargo/bin/multi-link", "ok", "none"),
            ("/VolumesOther/multi-link", "ok", "none"),
        ] {
            let report = report(Path::new(path), home, json!({"id":"hooks","state":"ok"}));
            assert_eq!(report["items"][0]["state"], cli);
            assert_eq!(report["items"][2]["state"], app);
            assert_eq!(exit_code(&report), i32::from(cli == "temporary"));
        }
        for (statuses, expected) in [
            (vec!["ok"], "ok"),
            (vec!["true"], "ok"),
            (vec!["customized"], "attention"),
            (vec!["unknown event"], "attention"),
            (vec!["customized", "missing"], "fixable"),
            (vec!["duplicate"], "fixable"),
            (vec!["outdated"], "fixable"),
        ] {
            let check = multi_link_hook::settings::Check {
                findings: statuses
                    .iter()
                    .map(|s| multi_link_hook::settings::Finding {
                        event: "Test".into(),
                        status: (*s).into(),
                        details: vec![],
                    })
                    .collect(),
                merged: json!({}),
                needs_fix: statuses
                    .iter()
                    .any(|s| ["missing", "duplicate", "outdated"].contains(s)),
            };
            let hooks = hooks_item(Path::new("/tmp/settings.json"), true, Some(&check));
            assert_eq!(hooks["state"], expected);
            assert_eq!(
                exit_code(&report(Path::new("/bin/multi-link"), home, hooks)),
                i32::from(expected == "fixable")
            );
        }
        let hooks = hooks_item(Path::new("/tmp/settings.json"), true, None);
        assert_eq!(hooks["state"], "unreadable");
        assert_eq!(hooks["events"], json!([]));
        assert_eq!(
            exit_code(&report(Path::new("/bin/multi-link"), home, hooks)),
            1
        );
    }
}
