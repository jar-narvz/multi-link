//! Hook I/O only; payload parsing and recording live in multi-link-hook.
use std::{
    io::{self, Read, Write},
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

use clap::{Subcommand, ValueEnum};
use multi_link_hook::{Clock, HookEnv};
use multi_link_model::Timestamp;
use serde_json::json;

#[derive(Subcommand)]
pub enum HookCommand {
    Record,
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
        HookCommand::Install { scope } => (scope, false),
        HookCommand::Uninstall { scope } => (scope, true),
    };
    let home = PathBuf::from(std::env::var_os("HOME").ok_or("HOME is not set")?);
    let target = match scope {
        Scope::User => home.join(".claude/settings.json"),
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
    };
    let exe = std::env::current_exe()?;
    let quoted = format!(
        "'{}'",
        exe.to_str()
            .ok_or("binary path is not UTF-8")?
            .replace('\'', "'\\''")
    );
    let entry = json!({"type":"command", "command":format!("{quoted} hook record"), "async":true, "timeout":60});
    let snippet = json!({"hooks": {
        "PostToolUse":[{"matcher":"Bash", "hooks":[entry.clone()]}],
        "PostToolUseFailure":[{"matcher":"Bash", "hooks":[entry.clone()]}],
        "PermissionRequest":[{"hooks":[entry.clone()]}],
        "Notification":[{"matcher":"permission_prompt|elicitation_dialog", "hooks":[entry]}]
    }});
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
    }
    super::json(&mut io::stdout().lock(), &snippet)?;
    Ok(())
}
