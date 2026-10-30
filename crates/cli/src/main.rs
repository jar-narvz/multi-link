mod hook;

use std::{
    collections::HashSet,
    io::{self, Write},
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clap::{Args, Parser, Subcommand};
use multi_link_core::{FixtureProcessProbe, Options, ProcessProbe, RealProcessProbe, RepoMode};
use multi_link_model::{Convention, LinksReport, SessionsReport, StepsReport, Timestamp};
use serde::Serialize;

#[derive(Parser)]
#[command(name = "multi-link", about = "Inspect local Claude and Codex sessions")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Hook {
        #[command(subcommand)]
        command: hook::HookCommand,
    },
    Sessions(CommonArgs),
    Links(CommonArgs),
    Steps {
        #[command(flatten)]
        common: CommonArgs,
        /// Repositories to read (repeatable; relative to --root).
        #[arg(long)]
        repo: Vec<String>,
    },
}

#[derive(Args)]
struct CommonArgs {
    /// Home-shaped root (default: HOME); enables fixture mode.
    #[arg(long)]
    root: Option<PathBuf>,
    /// Override the current time with an RFC 3339 timestamp.
    #[arg(long)]
    now: Option<Timestamp>,
    /// History duration (number + s|m|h|d), or 'all'.
    #[arg(long, default_value = "2h", value_parser = parse_since)]
    since: HistoryWindow,
    /// Print JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Clone)]
struct HistoryWindow(Option<Duration>);

fn parse_since(input: &str) -> Result<HistoryWindow, String> {
    if input == "all" {
        return Ok(HistoryWindow(None));
    }
    let invalid = || "expected 'all' or a duration such as 30m, 2h, or 1d".to_owned();
    let unit = input.as_bytes().last().ok_or_else(invalid)?;
    let multiplier = match unit {
        b's' => 1,
        b'm' => 60,
        b'h' => 3600,
        b'd' => 86400,
        _ => return Err(invalid()),
    };
    let number = &input[..input.len() - 1];
    if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid());
    }
    let seconds = number
        .parse::<u64>()
        .ok()
        .and_then(|number| number.checked_mul(multiplier))
        .ok_or_else(invalid)?;
    Ok(HistoryWindow(Some(Duration::from_secs(seconds))))
}

fn clock_now() -> Result<Timestamp, Box<dyn std::error::Error>> {
    let milliseconds = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(elapsed) => i64::try_from(elapsed.as_millis())?,
        Err(error) => -i64::try_from(error.duration().as_millis())?,
    };
    Ok(Timestamp::from_epoch_ms(milliseconds)?)
}

fn hook_conflicts(count: usize) {
    if count > 0 {
        eprintln!("hook_conflicts: {count}");
    }
}

fn diagnostic<T>(label: &str, entries: &[T]) {
    if !entries.is_empty() {
        eprintln!("{label}: {}", entries.len());
    }
}

fn json(writer: &mut impl Write, report: &impl Serialize) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut *writer, report)?;
    writeln!(writer)
}

fn label(value: impl Serialize) -> String {
    serde_json::to_value(value)
        .expect("display enum must serialize")
        .as_str()
        .expect("display enum must serialize as a string")
        .to_owned()
}

fn table<const N: usize>(
    writer: &mut impl Write,
    header: [&str; N],
    rows: Vec<[String; N]>,
) -> io::Result<()> {
    let mut widths = header.map(|column| column.chars().count());
    for row in &rows {
        for (width, column) in widths.iter_mut().zip(row) {
            *width = (*width).max(column.chars().count());
        }
    }
    for row in std::iter::once(header.map(str::to_owned)).chain(rows) {
        let mut line = String::new();
        for (index, column) in row.iter().enumerate() {
            line.push_str(column);
            if index + 1 < N {
                line.push_str(&" ".repeat(widths[index] - column.chars().count() + 2));
            }
        }
        writeln!(writer, "{}", line.trim_end())?;
    }
    Ok(())
}

fn prefix(id: &str) -> String {
    id.chars().take(8).collect()
}

fn sessions_text(writer: &mut impl Write, report: &SessionsReport) -> io::Result<()> {
    let rows = report
        .sessions
        .iter()
        .map(|session| {
            [
                prefix(&session.id),
                label(session.harness),
                label(session.state),
                session.last_activity.to_string(),
                session.repo.clone(),
            ]
        })
        .collect();
    table(
        writer,
        ["ID", "HARNESS", "STATE", "LAST_ACTIVITY", "REPO"],
        rows,
    )?;
    writeln!(
        writer,
        "{} sessions, {} skipped lines",
        report.sessions.len(),
        report.skipped.len()
    )?;
    Ok(())
}

fn links_text(writer: &mut impl Write, report: &LinksReport) -> io::Result<()> {
    let rows = report
        .links
        .iter()
        .map(|link| {
            [
                prefix(&link.parent),
                label(link.form),
                label(link.r#match),
                link.child
                    .as_deref()
                    .map(prefix)
                    .unwrap_or_else(|| "-".into()),
                link.at.to_string(),
            ]
        })
        .collect();
    table(writer, ["PARENT", "FORM", "MATCH", "CHILD", "AT"], rows)?;
    writeln!(
        writer,
        "{} links, {} unparented",
        report.links.len(),
        report.unparented.len()
    )
}

fn steps_text(writer: &mut impl Write, report: &StepsReport) -> io::Result<()> {
    let mut rows = Vec::new();
    let mut count = 0;
    for repo in &report.repos {
        if repo.convention != Convention::Supported {
            rows.push([
                repo.repo.clone(),
                "-".into(),
                label(repo.convention),
                String::new(),
            ]);
        }
        for step in &repo.steps {
            let result = step
                .progress
                .first()
                .map(|entry| label(entry.verify.result))
                .unwrap_or_else(|| "-".into());
            rows.push([
                repo.repo.clone(),
                step.id.clone(),
                result,
                step.title.clone(),
            ]);
            count += 1;
        }
    }
    table(writer, ["REPO", "ID", "VERIFY", "TITLE"], rows)?;
    writeln!(writer, "{count} steps in {} repos", report.repos.len())
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let cli = match cli.command {
        Command::Hook { command } => return hook::run(command),
        command => Cli { command },
    };
    let common = match &cli.command {
        Command::Hook { .. } => unreachable!(),
        Command::Sessions(common) | Command::Links(common) | Command::Steps { common, .. } => {
            common
        }
    };
    let fixture_probe = FixtureProcessProbe;
    let real_probe = RealProcessProbe::default();
    let (root, repo_mode, probe): (_, _, &dyn ProcessProbe) = match &common.root {
        Some(root) => (root.clone(), RepoMode::Fixture, &fixture_probe),
        None => (
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .ok_or("HOME is not set; supply --root")?,
            RepoMode::Real,
            &real_probe,
        ),
    };
    let options = Options {
        state_dir: std::env::var_os("MULTI_LINK_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join(".multi-link")),
        now: match common.now {
            Some(now) => now,
            None => clock_now()?,
        },
        since: common.since.0,
        repo_mode,
        probe,
    };
    let mut stdout = io::stdout().lock();
    match &cli.command {
        Command::Hook { .. } => unreachable!(),
        Command::Sessions(_) => {
            let output = multi_link_core::sessions_report(&root, &options)?;
            hook_conflicts(multi_link_core::links_report(&root, &options)?.hook_conflicts);
            diagnostic("bad_records", &output.bad_records);
            diagnostic("incomplete", &output.incomplete);
            diagnostic("unreadable", &output.unreadable);
            if common.json {
                json(&mut stdout, &output.report)?;
            } else {
                sessions_text(&mut stdout, &output.report)?;
            }
        }
        Command::Links(_) => {
            let output = multi_link_core::links_report(&root, &options)?;
            hook_conflicts(output.hook_conflicts);
            diagnostic("bad_records", &output.bad_records);
            diagnostic("incomplete", &output.incomplete);
            diagnostic("unreadable", &output.unreadable);
            if common.json {
                json(&mut stdout, &output.report)?;
            } else {
                links_text(&mut stdout, &output.report)?;
            }
        }
        Command::Steps { repo, .. } => {
            let mut repos = repo.clone();
            if repos.is_empty() {
                let output = multi_link_core::sessions_report(&root, &options)?;
                diagnostic("bad_records", &output.bad_records);
                diagnostic("incomplete", &output.incomplete);
                diagnostic("unreadable", &output.unreadable);
                let mut seen = HashSet::new();
                for session in output.report.sessions {
                    if seen.insert(session.repo.clone()) {
                        repos.push(session.repo);
                    }
                }
            }
            let report = multi_link_plan::steps_report(&root, &repos);
            if common.json {
                json(&mut stdout, &report)?;
            } else {
                steps_text(&mut stdout, &report)?;
            }
        }
    }
    Ok(())
}

fn main() {
    // Enter the fail-silent boundary before argument parsing for hook invocations.
    if std::env::args_os().skip(1).take(2).eq(["hook", "record"]) {
        hook::record();
        std::process::exit(0);
    }
    if let Err(error) = run(Cli::parse()) {
        eprintln!("multi-link: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_use_serde_names_for_multiword_variants() {
        assert_eq!(
            label(multi_link_model::StateEvidence::ProcessExit),
            "process_exit"
        );
    }
}
