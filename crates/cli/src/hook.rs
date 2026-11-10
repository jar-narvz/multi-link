//! Hook I/O only; payload parsing and recording live in multi-link-hook.
use std::{
    fs,
    io::{self, Read, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

use clap::{Subcommand, ValueEnum};
use multi_link_hook::settings::{self, quote};
use multi_link_hook::{Clock, HookEnv};
use multi_link_model::Timestamp;

#[derive(Subcommand)]
pub enum HookCommand {
    Record,
    /// Inspect one settings file; optionally print its merged JSON without writing.
    Check {
        #[arg(long, value_enum, default_value = "user")]
        scope: Scope,
        #[arg(long)]
        settings: Option<PathBuf>,
        #[arg(long)]
        merged: bool,
    },
    /// Print JSON on stdout and paste instructions on stderr; never edit settings.
    Install {
        #[arg(long, value_enum, default_value = "user")]
        scope: Scope,
    },
    /// Print JSON entries to remove on stdout and cleanup instructions on stderr.
    Uninstall {
        #[arg(long, value_enum, default_value = "user")]
        scope: Scope,
    },
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Scope {
    User,
    Local,
}

struct RealClock(Instant);
impl Clock for RealClock {
    fn now(&self) -> Timestamp {
        super::clock_now().expect("system time must fit a timestamp")
    }
    fn monotonic(&self) -> Duration {
        self.0.elapsed()
    }
    fn sleep(&mut self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

pub fn record() {
    std::panic::set_hook(Box::new(|_| {}));
    let _ = std::panic::catch_unwind(|| {
        let deadline = Duration::from_secs(30);
        #[cfg(debug_assertions)]
        let deadline = std::env::var("MULTI_LINK_HOOK_TEST")
            .ok()
            .and_then(|test| {
                test.strip_prefix("deadline_ms=")
                    .and_then(|s| s.parse().ok())
            })
            .map(Duration::from_millis)
            .unwrap_or(deadline);
        // Spawn can fail without panicking; then return silently before any blocking I/O.
        if std::thread::Builder::new()
            .spawn(move || {
                std::thread::sleep(deadline);
                std::process::exit(0);
            })
            .is_err()
        {
            return;
        }
        #[cfg(debug_assertions)]
        if std::env::var("MULTI_LINK_HOOK_TEST").as_deref() == Ok("panic") {
            panic!("injected hook panic");
        }
        let Some(home) = std::env::var_os("HOME") else {
            return;
        };
        let mut clock = RealClock(Instant::now());
        let env = HookEnv {
            home: home.into(),
            multi_link_home: std::env::var_os("MULTI_LINK_HOME").map(PathBuf::from),
            now: clock.now(),
        };
        let mut stdin = io::stdin().lock();
        let mut payload = Vec::new();
        if stdin
            .by_ref()
            .take(multi_link_hook::MAX_INPUT_BYTES as u64)
            .read_to_end(&mut payload)
            .is_err()
        {
            return;
        }
        // Drain without retaining excess bytes. Reject oversized input as a whole.
        let Ok(excess) = io::copy(&mut stdin, &mut io::sink()) else {
            return;
        };
        if excess == 0 {
            let _ = multi_link_hook::record_and_write(&payload, &env, &mut clock);
        }
    });
}

pub fn run(command: HookCommand) -> Result<(), Box<dyn std::error::Error>> {
    let (scope, uninstall) = match command {
        HookCommand::Record => {
            record();
            return Ok(());
        }
        HookCommand::Check {
            scope,
            settings,
            merged,
        } => {
            let code = match check_settings(scope, settings, merged) {
                Ok(code) => code,
                Err(error) => {
                    eprintln!("multi-link: {error}");
                    2
                }
            };
            std::process::exit(code);
        }
        HookCommand::Install { scope } => (scope, false),
        HookCommand::Uninstall { scope } => (scope, true),
    };
    let home = PathBuf::from(std::env::var_os("HOME").ok_or("HOME is not set")?);
    let target = settings_target(scope)?;
    let exe = std::env::current_exe()?;
    let snippet = settings::snippet(exe.to_str().ok_or("binary path is not UTF-8")?);
    let mut stderr = io::stderr().lock();
    if uninstall {
        writeln!(
            stderr,
            "Remove the entries shown on stdout from {}",
            target.display()
        )?;
        let state = std::env::var_os("MULTI_LINK_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".multi-link"));
        writeln!(
            stderr,
            "You may delete the state folder: {}",
            state.display()
        )?;
    } else {
        writeln!(stderr, "Merge the JSON on stdout into {}", target.display())?;
        writeln!(
            stderr,
            "Run multi-link hook check to check and update your settings."
        )?;
    }
    super::json(&mut io::stdout().lock(), &snippet)?;
    Ok(())
}

fn settings_target(scope: Scope) -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(match scope {
        Scope::User => match std::env::var_os("CLAUDE_CONFIG_DIR") {
            Some(config) => PathBuf::from(config).join("settings.json"),
            None => PathBuf::from(std::env::var_os("HOME").ok_or("HOME is not set")?)
                .join(".claude/settings.json"),
        },
        Scope::Local => {
            let cwd = std::env::current_dir()?;
            let root = Command::new("git")
                .args(["rev-parse", "--show-toplevel"])
                .env("GIT_OPTIONAL_LOCKS", "0")
                .output()
                .ok()
                .filter(|output| output.status.success())
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .map(|root| PathBuf::from(root.trim_end_matches('\n')))
                .unwrap_or(cwd);
            root.join(".claude/settings.local.json")
        }
    })
}

// Follow existing symlinks, including ancestors of an absent target. A dangling
// symlink is an error, rather than a missing file to replace.
fn resolved_settings(path: &Path) -> io::Result<PathBuf> {
    match fs::symlink_metadata(path) {
        Ok(_) => fs::canonicalize(path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let parent = path
                .parent()
                .ok_or_else(|| io::Error::other("no existing ancestor"))?;
            Ok(resolved_settings(parent)?.join(
                path.file_name()
                    .ok_or_else(|| io::Error::other("missing file name"))?,
            ))
        }
        Err(error) => Err(error),
    }
}

fn path_text(path: &Path) -> Result<&str, Box<dyn std::error::Error>> {
    let text = path.to_str().ok_or("path is not UTF-8")?;
    if text.contains(['\n', '\r']) {
        return Err("path contains a newline".into());
    }
    Ok(text)
}

fn merge_command(
    target: &Path,
    exe: &Path,
    mode: Option<u32>,
) -> Result<String, Box<dyn std::error::Error>> {
    let file = path_text(target)?;
    let r = quote(file);
    let e = quote(path_text(exe)?);
    Ok(if let Some(mode) = mode {
        let now = super::clock_now()?.to_string();
        let timestamp = format!("{}Z", now[..19].replace(['-', ':'], ""));
        let temp = quote(&format!("{file}.multi-link-XXXXXX"));
        let backup = quote(&format!("{file}.multi-link-backup-{timestamp}-XXXXXX"));
        format!(
            "tmp=$(mktemp {temp}) && cp -p {r} \"$(mktemp {backup})\" && {e} hook check --settings {r} --merged > \"$tmp\" && chmod {mode:o} \"$tmp\" && mv \"$tmp\" {r} || {{ rm -f \"$tmp\"; false; }}"
        )
    } else {
        let parent = path_text(target.parent().ok_or("settings has no parent")?)?;
        let d = quote(parent);
        let temp = quote(&format!("{parent}/settings.json.multi-link-XXXXXX"));
        format!(
            "mkdir -p {d} && tmp=$(mktemp {temp}) && {e} hook check --settings {r} --merged > \"$tmp\" && [ ! -e {r} ] && mv \"$tmp\" {r} || {{ rm -f \"$tmp\"; false; }}"
        )
    })
}

fn check_settings(
    scope: Scope,
    explicit: Option<PathBuf>,
    merged: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    let target = match explicit {
        Some(path) => path,
        None => settings_target(scope)?,
    };
    path_text(&target)?;
    let absolute = if target.is_absolute() {
        target
    } else {
        std::env::current_dir()?.join(target)
    };
    let target =
        resolved_settings(&absolute).map_err(|e| format!("{}: {e}", absolute.display()))?;
    path_text(&target)?;
    let (text, mode) = match fs::metadata(&target) {
        Ok(metadata) => {
            if !metadata.is_file() {
                return Err(
                    format!("{}: settings must be a regular file", target.display()).into(),
                );
            }
            (
                fs::read_to_string(&target).map_err(|e| format!("{}: {e}", target.display()))?,
                Some(metadata.permissions().mode() & 0o7777),
            )
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => ("{}".into(), None),
        Err(error) => return Err(format!("{}: {error}", target.display()).into()),
    };
    let exe = std::env::current_exe()?;
    let result = settings::check(&text, path_text(&exe)?)
        .map_err(|e| format!("{}: {e}", target.display()))?;
    if merged {
        super::json(&mut io::stdout().lock(), &result.merged)?;
        return Ok(0);
    }
    println!("File: {}", target.display());
    for finding in &result.findings {
        let details = if finding.details.is_empty() {
            String::new()
        } else {
            format!(" ({})", finding.details.join("; "))
        };
        println!("{}: {}{}", finding.event, finding.status, details);
    }
    if result.current() {
        println!("multi-link hooks in {} are current", target.display());
        Ok(0)
    } else {
        if result.needs_fix {
            println!("Run this command to back up existing settings and merge generated hooks:");
            println!("{}", merge_command(&target, &exe, mode)?);
        }
        Ok(1)
    }
}
