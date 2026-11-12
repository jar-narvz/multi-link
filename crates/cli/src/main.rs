mod hook;
mod pane;
mod tui;

use std::{
    collections::HashSet,
    io::{self, Write},
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clap::{Args, Parser, Subcommand};
use multi_link_core::{
    FixtureProcessProbe, Options, ProcessProbe, RealProcessProbe, RepoMode,
    open::{self, OpenAction},
};
use multi_link_model::{
    Convention, GraphReport, LinksReport, OverviewReport, Row, SessionRow, SessionsReport,
    StepsReport, Timestamp,
};
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
    /// Interactive dashboard.
    Tui(tui::TuiArgs),
    /// Repo sessions for the Claude Code pane.
    Pane(pane::PaneArgs),
    Sessions(CommonArgs),
    Links(CommonArgs),
    /// Home screen data: repo groups, rows, plan (one refresh pass).
    Overview(CommonArgs),
    /// Working, waiting and stale sessions with a hand-off feed.
    Live(CommonArgs),
    /// Resume a finished Claude session or open it in the app (FR18).
    Open(OpenArgs),
    /// One PLAN.md step across both harnesses.
    Graph(GraphArgs),
    Steps {
        #[command(flatten)]
        common: CommonArgs,
        /// Repositories to read (repeatable; relative to --root).
        #[arg(long)]
        repo: Vec<String>,
    },
}

#[derive(Args)]
struct GraphArgs {
    /// Home-shaped root (default: HOME); enables fixture mode.
    #[arg(long)]
    root: Option<PathBuf>,
    /// Override the current time with an RFC 3339 timestamp.
    #[arg(long)]
    now: Option<Timestamp>,
    /// The session repo, as `sessions` prints it.
    #[arg(long)]
    repo: String,
    /// The PLAN.md step id (a string: 1.10 is not 1.1).
    #[arg(long)]
    step: String,
    /// Print JSON.
    #[arg(long)]
    json: bool,
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

#[derive(Args)]
struct OpenArgs {
    /// Refuse any action other than opening an existing desktop app session.
    #[arg(long)]
    app_only: bool,
    /// Session id or a prefix of it (matched on full ids, history window off).
    id_prefix: String,
    /// Fork instead of resuming in place (`--fork-session`).
    #[arg(long)]
    fork: bool,
    /// Print the action as JSON instead of executing it.
    #[arg(long)]
    dry_run: bool,
    /// Home-shaped root (default: HOME); enables fixture mode.
    #[arg(long)]
    root: Option<PathBuf>,
    /// Override the current time with an RFC 3339 timestamp.
    #[arg(long)]
    now: Option<Timestamp>,
}

/// Exit codes of `open`: several matches, nothing runnable, no match.
const EXIT_AMBIGUOUS: i32 = 2;
const EXIT_UNAVAILABLE: i32 = 3;
const EXIT_NO_MATCH: i32 = 4;
/// macOS `open`, used for app links.
const OPEN_PROGRAM: &str = "/usr/bin/open";

/// The process an executable action runs: no shell, ids only as argv items.
/// `None` for `Unavailable`. Built separately so tests inspect it without spawning.
fn open_command(action: &OpenAction) -> Option<std::process::Command> {
    match action {
        OpenAction::Resume { program, args, cwd } => {
            let mut command = std::process::Command::new(program);
            command.args(args).current_dir(cwd);
            Some(command)
        }
        OpenAction::AppLink { url } => {
            // Absolute path: an app link never depends on a PATH lookup.
            let mut command = std::process::Command::new(OPEN_PROGRAM);
            command.arg(url);
            Some(command)
        }
        OpenAction::Unavailable { .. } | OpenAction::Jump { .. } => None,
    }
}

/// Returns the process exit code. Unavailable → reason on stderr, exit 3, in
/// both modes (`--dry-run` still prints the JSON first), so the exit code always
/// says whether the session can be opened.
fn open(
    args: &OpenArgs,
    root: &std::path::Path,
    options: &Options<'_>,
) -> Result<i32, Box<dyn std::error::Error>> {
    let mut scanner = multi_link_core::Scanner::default();
    let session = match open::find(&mut scanner, root, options, &args.id_prefix)? {
        open::Lookup::Found(session) => session,
        open::Lookup::NoMatch => {
            eprintln!("no session matches {}", args.id_prefix);
            return Ok(EXIT_NO_MATCH);
        }
        open::Lookup::Ambiguous(sessions) => {
            eprintln!("{} sessions match {}:", sessions.len(), args.id_prefix);
            for session in sessions {
                eprintln!(
                    "{:?}  {}  {}",
                    session.id,
                    label(session.harness),
                    label(session.state)
                );
            }
            return Ok(EXIT_AMBIGUOUS);
        }
    };
    let action = scanner.open_resolver(root)?.action(&session, args.fork);
    if args.dry_run {
        json(&mut io::stdout().lock(), &action)?;
    }
    if let OpenAction::Unavailable {
        reason,
        command,
        cwd,
        ..
    } = &action
    {
        match command {
            Some(command) => eprintln!(
                "cannot open {:?}: {}; run it yourself in {cwd:?}: {command}",
                session.id,
                reason.as_str()
            ),
            None => eprintln!("cannot open {:?}: {}", session.id, reason.as_str()),
        }
        return Ok(EXIT_UNAVAILABLE);
    }
    if args.app_only && !matches!(action, OpenAction::AppLink { .. }) {
        eprintln!("cannot open {:?}: no app link available", session.id);
        return Ok(EXIT_UNAVAILABLE);
    }
    if args.dry_run {
        return Ok(0);
    }
    if matches!(action, OpenAction::Jump { .. }) {
        eprintln!("cannot jump: terminal jump execution is not implemented yet");
        return Ok(EXIT_UNAVAILABLE);
    }
    let mut command = open_command(&action).expect("executable action has a command");
    let status = command
        .status()
        .map_err(|error| format!("cannot run {:?}: {error}", command.get_program()))?;
    let (code, message) = child_exit(command.get_program(), status);
    if let Some(message) = message {
        eprintln!("{message}");
    }
    Ok(code)
}

/// The child's own code is not passed through: it would collide with 2/3/4.
/// Success → 0; a non-zero exit or a signal → exit 1 with a message.
fn child_exit(
    program: &std::ffi::OsStr,
    status: std::process::ExitStatus,
) -> (i32, Option<String>) {
    if status.success() {
        (0, None)
    } else {
        (1, Some(format!("{program:?} exited with {status}")))
    }
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

fn home() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or("HOME is not set; supply --root")?)
}

fn state_dir(root: &std::path::Path) -> PathBuf {
    std::env::var_os("MULTI_LINK_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join(".multi-link"))
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

/// A row and every session nested under it, at any depth (8.4c S3).
fn count_sessions(row: &SessionRow) -> usize {
    1 + row.children.iter().map(count_sessions).sum::<usize>()
}

/// One line per repo: tier, session count, last activity, latest step, repo.
fn overview_text(writer: &mut impl Write, report: &OverviewReport) -> io::Result<()> {
    let rows = report
        .repos
        .iter()
        .map(|group| {
            let sessions: usize = group
                .rows
                .iter()
                .map(|row| match row {
                    Row::Session(row) => count_sessions(row),
                    Row::Merged(merged) => merged.rows.iter().map(count_sessions).sum(),
                })
                .sum();
            let step = match &group.plan.latest_step {
                Some(step) => step.id.clone(),
                None => label(group.plan.convention),
            };
            let warning = if group.concurrency.is_empty() {
                String::new()
            } else {
                format!(
                    " (!{} busy in one cwd)",
                    group.concurrency[0].sessions.len()
                )
            };
            [
                label(group.tier),
                sessions.to_string(),
                group.last_activity.to_string(),
                step,
                format!("{}{warning}", group.repo),
            ]
        })
        .collect();
    table(
        writer,
        ["TIER", "SESSIONS", "LAST_ACTIVITY", "STEP", "REPO"],
        rows,
    )?;
    writeln!(writer, "{} repos", report.repos.len())
}

fn clock(hms: &Timestamp) -> String {
    // RFC 3339 UTC: `2026-11-06T10:00:00.000Z`; keep `HH:MM:SS`.
    hms.to_string().chars().skip(11).take(8).collect()
}

fn graph_text(writer: &mut impl Write, report: &GraphReport) -> io::Result<()> {
    writeln!(
        writer,
        "{} › {} {}",
        report.repo,
        report.step.id,
        report.step.title.as_deref().unwrap_or("")
    )?;
    if let Some(commit) = &report.commit {
        // `HH:MM` of the commit time (G7).
        let at: String = clock(&commit.at).chars().take(5).collect();
        writeln!(writer, "✓ committed {} · {at}", commit.hash)?;
    }
    if let Some(verify) = &report.step.verify {
        writeln!(writer, "Verify {verify} (reported)")?;
    }
    if let Some(empty) = &report.empty {
        return writeln!(writer, "{empty}");
    }
    for part in &report.parts {
        let range = |time: &Option<Timestamp>| time.as_ref().map_or("-".to_string(), clock);
        writeln!(
            writer,
            "{} {} · {} {} {}–{} {} {} reads · {} edits · {} runs",
            part.chain,
            part.kind,
            label(part.harness),
            // First 8 chars, not bytes: a byte cut can split a char.
            part.session
                .as_deref()
                .map_or("-".to_string(), |id| id.chars().take(8).collect()),
            range(&part.from),
            range(&part.to),
            part.status.map_or(String::new(), label),
            part.counts.reads,
            part.counts.edits,
            part.counts.runs
        )?;
        // Notable calls are fixed labels only (G6); `ok` is the exit status.
        for notable in &part.notable {
            let mark = match notable.ok {
                Some(true) => " ok",
                Some(false) => " failed",
                None => "",
            };
            writeln!(writer, "    {}{mark}", notable.label)?;
        }
        if part.waiting_reads > 0 {
            writeln!(writer, "    {} reads while codex ran", part.waiting_reads)?;
        }
        if part.untagged_handoffs > 0 {
            let plural = if part.untagged_handoffs == 1 { "" } else { "s" };
            writeln!(
                writer,
                "    {} untagged hand-off{plural} here",
                part.untagged_handoffs
            )?;
        }
        if let Some(asked) = &part.asked {
            writeln!(writer, "    asked {asked}")?;
        }
        if let Some(said) = &part.said {
            writeln!(writer, "    said {said}")?;
        }
    }
    Ok(())
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let cli = match cli.command {
        Command::Hook { command } => return hook::run(command),
        Command::Tui(args) => return tui::run(args),
        Command::Pane(args) => return pane::run(args),
        command => Cli { command },
    };
    if let Command::Open(args) = &cli.command {
        let fixture_probe = FixtureProcessProbe;
        let real_probe = RealProcessProbe::default();
        let (root, repo_mode, probe): (_, _, &dyn ProcessProbe) = match &args.root {
            Some(root) => (root.clone(), RepoMode::Fixture, &fixture_probe),
            None => (home()?, RepoMode::Real, &real_probe),
        };
        let options = Options {
            state_dir: state_dir(&root),
            now: match args.now {
                Some(now) => now,
                None => clock_now()?,
            },
            since: None,
            repo_mode,
            probe,
        };
        let code = open(args, &root, &options)?;
        if code != 0 {
            std::process::exit(code);
        }
        return Ok(());
    }
    if let Command::Graph(args) = &cli.command {
        let fixture_probe = FixtureProcessProbe;
        let real_probe = RealProcessProbe::default();
        let (root, repo_mode, probe): (_, _, &dyn ProcessProbe) = match &args.root {
            Some(root) => (root.clone(), RepoMode::Fixture, &fixture_probe),
            None => (home()?, RepoMode::Real, &real_probe),
        };
        let options = Options {
            state_dir: state_dir(&root),
            now: match args.now {
                Some(now) => now,
                None => clock_now()?,
            },
            since: None,
            repo_mode,
            probe,
        };
        let mut plans = |folder: &std::path::Path| {
            multi_link_plan::steps_report(
                std::path::Path::new(""),
                &[folder.to_string_lossy().into_owned()],
            )
            .repos
            .remove(0)
        };
        let report = multi_link_core::Scanner::default()
            .graph(&root, &options, &args.repo, &args.step, &mut plans)?;
        let mut stdout = io::stdout().lock();
        if args.json {
            json(&mut stdout, &report)?;
        } else {
            graph_text(&mut stdout, &report)?;
        }
        return Ok(());
    }
    let common = match &cli.command {
        Command::Hook { .. }
        | Command::Tui(_)
        | Command::Pane(_)
        | Command::Open(_)
        | Command::Graph(_) => {
            unreachable!()
        }
        Command::Sessions(common)
        | Command::Links(common)
        | Command::Live(common)
        | Command::Overview(common)
        | Command::Steps { common, .. } => common,
    };
    let fixture_probe = FixtureProcessProbe;
    let real_probe = RealProcessProbe::default();
    let (root, repo_mode, probe): (_, _, &dyn ProcessProbe) = match &common.root {
        Some(root) => (root.clone(), RepoMode::Fixture, &fixture_probe),
        None => (home()?, RepoMode::Real, &real_probe),
    };
    let options = Options {
        state_dir: state_dir(&root),
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
        Command::Hook { .. }
        | Command::Tui(_)
        | Command::Pane(_)
        | Command::Open(_)
        | Command::Graph(_) => {
            unreachable!()
        }
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
        Command::Live(_) => {
            let report = multi_link_core::Scanner::default().live(&root, &options)?;
            if common.json {
                json(&mut stdout, &report)?;
            } else {
                for group in &report.groups {
                    for card in std::iter::once(&group.head).chain(&group.children) {
                        writeln!(
                            stdout,
                            "{} {} {} {}",
                            label(card.harness),
                            card.id,
                            label(card.state),
                            card.repo
                        )?;
                    }
                }
                for row in &report.feed {
                    writeln!(
                        stdout,
                        "{} {} {} {}",
                        row.at,
                        row.repo,
                        row.invocation,
                        label(row.r#match)
                    )?;
                }
            }
        }
        Command::Overview(_) => {
            let mut plans = |folder: &std::path::Path| {
                multi_link_plan::steps_report(
                    std::path::Path::new(""),
                    &[folder.to_string_lossy().into_owned()],
                )
                .repos
                .remove(0)
            };
            let output =
                multi_link_core::Scanner::default().overview(&root, &options, &mut plans)?;
            hook_conflicts(output.links.hook_conflicts);
            diagnostic("bad_records", &output.sessions.bad_records);
            diagnostic("incomplete", &output.sessions.incomplete);
            diagnostic("unreadable", &output.sessions.unreadable);
            if common.json {
                json(&mut stdout, &output.report)?;
            } else {
                overview_text(&mut stdout, &output.report)?;
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

    fn argv(command: &std::process::Command) -> Vec<String> {
        std::iter::once(command.get_program())
            .chain(command.get_args())
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    /// Builds, never spawns: tests must not start a harness or open a URL.
    #[test]
    fn open_commands_run_without_a_shell_in_cwd() {
        let id = "aaaaaaaa-0000-4000-8000-000000000001";
        let resume = OpenAction::Resume {
            program: "claude".into(),
            args: vec!["--resume".into(), id.into(), "--fork-session".into()],
            cwd: "/some/repo".into(),
        };
        let command = open_command(&resume).unwrap();
        assert_eq!(argv(&command), ["claude", "--resume", id, "--fork-session"]);
        assert_eq!(
            command.get_current_dir(),
            Some(std::path::Path::new("/some/repo"))
        );
        let url = "claude://code/continue?session=local_abc";
        let command = open_command(&OpenAction::AppLink { url: url.into() }).unwrap();
        assert_eq!(argv(&command), ["/usr/bin/open", url]);
        assert_eq!(command.get_current_dir(), None);
        assert!(
            open_command(&OpenAction::Unavailable {
                detail: None,
                reason: open::Reason::NotKnownFinished,
                command: Some(format!("claude --resume {id}")),
                cwd: "/some/repo".into(),
            })
            .is_none()
        );
    }

    #[test]
    fn child_exit_never_passes_the_child_code_through() {
        use std::os::unix::process::ExitStatusExt;
        let program = std::ffi::OsStr::new("claude");
        assert_eq!(child_exit(program, ExitStatusExt::from_raw(0)), (0, None));
        // Exit codes 2, 3, 4 (wait status = code << 8) and SIGKILL (raw 9) all become 1.
        for raw in [2 << 8, 3 << 8, 4 << 8, 9] {
            let (code, message) = child_exit(program, ExitStatusExt::from_raw(raw));
            assert_eq!(code, 1, "{raw}");
            let message = message.unwrap();
            assert!(message.starts_with("\"claude\" exited with "), "{message}");
        }
    }

    #[test]
    fn graph_text_cuts_non_ascii_session_ids_by_chars() {
        use multi_link_model::{Counts, GraphStep, Harness, Part};
        let at = Timestamp::parse("2026-11-06T10:00:00Z").unwrap();
        let report = GraphReport {
            now: at,
            repo: "/repo".into(),
            step: GraphStep {
                id: "1.1".into(),
                title: None,
                verify: None,
            },
            commit: None,
            parts: vec![Part {
                chain: 0,
                kind: "plan".into(),
                harness: Harness::Claude,
                session: Some("aéééééééé-1".into()),
                from: Some(at),
                to: Some(at),
                status: None,
                run_available: None,
                active_ms: 0,
                stretches: 0,
                asked: None,
                said: None,
                counts: Counts::default(),
                notable: Vec::new(),
                waiting_reads: 0,
                untagged_handoffs: 0,
                edge_in: None,
            }],
            empty: None,
        };
        let mut out = Vec::new();
        graph_text(&mut out, &report).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains(" aééééééé 10:00:00"), "{text}");
    }

    #[test]
    fn labels_use_serde_names_for_multiword_variants() {
        assert_eq!(
            label(multi_link_model::StateEvidence::ProcessExit),
            "process_exit"
        );
    }
}
