//! Terminal runtime. Only the worker touches disk; only this thread reads terminal input.
use multi_link_core::live::{codex_steps, started_steps};
use std::{
    collections::{HashMap, HashSet},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

use clap::Args;
use crossterm::{
    cursor::Show,
    event::{self, Event as TerminalEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use multi_link_core::{
    Options, Scanner,
    open::{self, OpenAction},
};
use multi_link_model::{Harness, Link, RepoSteps, Session, Timestamp};
use multi_link_tui::{App, Effect, Event, Refresh};
use ratatui::{Terminal, backend::CrosstermBackend};

#[derive(Args)]
pub struct TuiArgs {
    /// Home-shaped root; enables fixture mode.
    #[arg(long)]
    root: Option<PathBuf>,
    #[arg(long)]
    now: Option<Timestamp>,
    #[arg(long, default_value = "2h", value_parser = super::parse_since)]
    since: super::HistoryWindow,
}

enum Request {
    Open {
        id: String,
        fork: bool,
    },
    /// The step graph to build on every refresh from now on; None stops it.
    Graph(Option<(String, String)>),
}
enum Message {
    Refresh(Result<Box<Refresh>, String>),
    Open(Result<Option<OpenAction>, String>),
}

/// Terminal transitions are injectable so failure and unwind paths need no real terminal in tests.
trait Modes {
    fn enter(&self) -> io::Result<()>;
    fn leave(&self) -> io::Result<()>;
}
struct TerminalModes;
impl Modes for TerminalModes {
    fn enter(&self) -> io::Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)
    }
    fn leave(&self) -> io::Result<()> {
        let raw = disable_raw_mode();
        let screen = execute!(io::stdout(), LeaveAlternateScreen, Show);
        raw.and(screen)
    }
}

/// Restores normal mode on every return and unwind, including partial setup.
struct Screen<M: Modes>(M);
impl<M: Modes> Screen<M> {
    fn enter(modes: M) -> io::Result<Self> {
        let guard = Self(modes);
        guard.0.enter()?;
        Ok(guard)
    }
}
impl<M: Modes> Drop for Screen<M> {
    fn drop(&mut self) {
        let _ = self.0.leave();
    }
}

/// This guard restores the dashboard even if spawning/waiting panics.
struct Suspended<'a, M: Modes>(&'a Screen<M>);
impl<'a, M: Modes> Suspended<'a, M> {
    fn new(screen: &'a Screen<M>) -> io::Result<Self> {
        let guard = Self(screen);
        screen.0.leave()?;
        Ok(guard)
    }
    fn finish(self) -> io::Result<()> {
        let result = self.0.0.enter();
        if result.is_ok() {
            std::mem::forget(self);
        }
        result
    }
}
impl<M: Modes> Drop for Suspended<'_, M> {
    fn drop(&mut self) {
        let _ = self.0.0.enter();
    }
}

/// One refresh for the App: the overview, and the step graph it asked for (G11: rebuilt on every
/// refresh, so a stale turn or a new rollout shows without a key press). A failed graph build fails
/// the whole refresh, so the App keeps its last screen and shows the error.
fn refresh(
    scanner: &mut Scanner,
    root: &Path,
    options: &Options<'_>,
    graph: Option<&(String, String)>,
) -> io::Result<Box<Refresh>> {
    let mut plans = |folder: &Path| {
        multi_link_plan::steps_report(Path::new(""), &[folder.to_string_lossy().into_owned()])
            .repos
            .remove(0)
    };
    let output = scanner.overview(root, options, &mut plans)?;
    // Read before the graph's own refreshes replace the scanner's diagnostics.
    let diagnostics = [
        ("bad records", output.sessions.bad_records.len()),
        ("incomplete", output.sessions.incomplete.len()),
        ("unreadable", output.sessions.unreadable.len()),
        ("hook diagnostics", scanner.diagnostics.len()),
        ("hook conflicts", output.links.hook_conflicts),
    ]
    .into_iter()
    .filter(|(_, count)| *count > 0)
    .map(|(name, count)| format!("{name}: {count}"))
    .collect::<Vec<_>>()
    .join(" · ");
    // One all-time pass per refresh, whatever the TUI's window: started children that went
    // quiet long ago still count for `s` (G9 amendment), Codex incoming steps use all-time
    // same-repo parents (R6), and G2 step lists are all-time.
    let all = Options {
        state_dir: options.state_dir.clone(),
        since: None,
        ..*options
    };
    let (sessions, links) = scanner.refresh(root, &all)?;
    let sessions = &sessions.report.sessions;
    let links = &links.report.links;
    let started_steps = started_steps(sessions, links);
    let codex_steps = codex_steps(sessions, links);
    let (graph, graph_steps, graph_live) = match graph {
        None => (None, Vec::new(), Vec::new()),
        Some((repo, step)) => {
            let plan = multi_link_core::overview::plan_folder(root, options.repo_mode, repo)
                .map(|folder| plans(&folder));
            let steps = graph_steps(sessions, links, repo, plan.as_ref());
            let report = scanner.graph(root, options, repo, step, &mut plans)?;
            let live = multi_link_core::graph::live_parts(&report, sessions);
            (Some(report), steps, live)
        }
    };
    Ok(Box::new(Refresh {
        overview: output.report,
        links: output.links.report,
        diagnostics,
        graph,
        graph_steps,
        graph_live,
        codex_steps,
        started_steps,
    }))
}

/// Steps `[` `]` walk in a repo (G9): ids tagged on hand-offs whose parent is a listed Claude
/// session of `repo`, in PLAN.md order; tagged ids PLAN.md lacks follow, by first hand-off, then id.
fn graph_steps(
    sessions: &[Session],
    links: &[Link],
    repo: &str,
    plan: Option<&RepoSteps>,
) -> Vec<String> {
    let in_repo: HashSet<&str> = sessions
        .iter()
        .filter(|s| s.harness == Harness::Claude && s.repo == repo)
        .map(|s| s.id.as_str())
        .collect();
    let mut first: HashMap<&str, Timestamp> = HashMap::new();
    for link in links.iter().filter(|l| in_repo.contains(l.parent.as_str())) {
        if let Some(step) = link.step.as_deref() {
            let at = first.entry(step).or_insert(link.at);
            *at = (*at).min(link.at);
        }
    }
    let planned: Vec<&str> = plan
        .map(|plan| plan.steps.iter().map(|s| s.id.as_str()).collect())
        .unwrap_or_default();
    let mut rest: Vec<(Timestamp, &str)> = first
        .iter()
        .filter(|(id, _)| !planned.contains(id))
        .map(|(id, at)| (*at, *id))
        .collect();
    rest.sort();
    planned
        .iter()
        .filter(|id| first.contains_key(*id))
        .copied()
        .chain(rest.into_iter().map(|(_, id)| id))
        .map(String::from)
        .collect()
}

fn worker(args: TuiArgs, root: PathBuf, requests: Receiver<Request>, messages: Sender<Message>) {
    let fixture = multi_link_core::FixtureProcessProbe;
    let real = multi_link_core::RealProcessProbe::default();
    let mut scanner = Scanner::default();
    let mut next_refresh = Instant::now();
    let mut graph: Option<(String, String)> = None;
    loop {
        let now = match args.now.map(Ok).unwrap_or_else(super::clock_now) {
            Ok(now) => now,
            Err(error) => {
                let _ = messages.send(Message::Refresh(Err(error.to_string())));
                return;
            }
        };
        let options = Options {
            state_dir: super::state_dir(&root),
            now,
            since: args.since.0,
            repo_mode: if args.root.is_some() {
                multi_link_core::RepoMode::Fixture
            } else {
                multi_link_core::RepoMode::Real
            },
            probe: if args.root.is_some() { &fixture } else { &real },
        };
        if Instant::now() >= next_refresh {
            let started = Instant::now();
            let result = refresh(&mut scanner, &root, &options, graph.as_ref())
                .map_err(|error| error.to_string());
            if messages.send(Message::Refresh(result)).is_err() {
                return;
            }
            next_refresh = (started + Duration::from_secs(2)).max(Instant::now());
        }
        match requests.recv_timeout(next_refresh.saturating_duration_since(Instant::now())) {
            Ok(Request::Open { id, fork }) => {
                // Re-read liveness with the history window off, immediately before execution.
                let result = (|| {
                    let options = Options {
                        now: args
                            .now
                            .map(Ok)
                            .unwrap_or_else(super::clock_now)
                            .map_err(|e| e.to_string())?,
                        ..options
                    };
                    open::fresh_action(&mut scanner, &root, &options, &id, fork)
                        .map_err(|e| e.to_string())
                })();
                if messages.send(Message::Open(result)).is_err() {
                    return;
                }
            }
            Ok(Request::Graph(target)) => {
                // Build the newly asked step at once rather than at the next 2 s tick.
                graph = target;
                next_refresh = Instant::now();
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        }
    }
}

/// Handle a fresh worker result. Nothing executable is derived from the drawn report.
fn open_result(result: Result<Option<OpenAction>, String>) -> Result<OpenAction, String> {
    match result {
        Err(error) => Err(format!("open check failed: {error}")),
        Ok(None) => Err("session no longer found".into()),
        Ok(Some(OpenAction::Unavailable {
            reason,
            command,
            cwd,
        })) => Err(match command {
            Some(command) => format!("{}; in {cwd}: {command}", reason.as_str()),
            None => reason.as_str().into(),
        }),
        Ok(Some(action)) => Ok(action),
    }
}

/// Raw mode delivers Ctrl-C as input rather than SIGINT; use the ordinary quit path.
fn app_key(key: KeyEvent) -> KeyCode {
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        KeyCode::Char('q')
    } else {
        key.code
    }
}

pub fn run(args: TuiArgs) -> Result<(), Box<dyn std::error::Error>> {
    let root = args.root.clone().map(Ok).unwrap_or_else(super::home)?;
    let colorterm = std::env::var_os("COLORTERM");
    let no_color = std::env::var_os("NO_COLOR");
    let theme = multi_link_tui::theme::Theme::from_env(
        colorterm.as_deref().and_then(|value| value.to_str()),
        no_color.as_deref(),
    );
    let screen = Screen::enter(TerminalModes)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    terminal.clear()?;
    let (requests, receiver) = mpsc::channel();
    let (sender, messages) = mpsc::channel();
    // Dropping the request sender stops the worker at its next boundary. Do not join a slow scan
    // on quit: the terminal must be restored immediately, even with an inaccessible filesystem.
    let history_window = args.since.0;
    thread::Builder::new()
        .name("multi-link-refresh".into())
        .spawn(move || worker(args, root, receiver, sender))?;
    let mut app = App::default();
    app.theme = Some(theme);
    app.history_window = history_window;
    app.history_all = history_window.is_none();
    loop {
        match messages.try_recv() {
            Ok(Message::Refresh(result)) => {
                for effect in app.update(Event::Refresh(result)) {
                    if effect == Effect::Bell {
                        io::stdout().write_all(b"\x07")?;
                        io::stdout().flush()?;
                    }
                }
            }
            Ok(Message::Open(result)) => {
                let message = match open_result(result) {
                    Err(message) => message,
                    Ok(action) => {
                        // No event reader runs while status() owns inherited terminal input.
                        let suspended = Suspended::new(&screen)?;
                        let mut command =
                            super::open_command(&action).expect("fresh executable action");
                        let status = command.status();
                        suspended.finish()?;
                        terminal.clear()?;
                        match status {
                            Ok(status) => super::child_exit(command.get_program(), status)
                                .1
                                .unwrap_or_else(|| "returned to dashboard".into()),
                            Err(error) => {
                                format!("cannot run {:?}: {error}", command.get_program())
                            }
                        }
                    }
                };
                app.update(Event::Opened(message));
            }
            Err(mpsc::TryRecvError::Disconnected) => return Err("refresh worker stopped".into()),
            Err(mpsc::TryRecvError::Empty) => {}
        }
        terminal.draw(|frame| {
            app.render(frame.area(), frame.buffer_mut());
            theme.apply_no_color(frame.buffer_mut());
        })?;
        if event::poll(Duration::from_millis(50))?
            && let TerminalEvent::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            for effect in app.update(Event::Key(app_key(key))) {
                match effect {
                    Effect::Quit => return Ok(()),
                    Effect::FreshOpen { id, fork } => requests.send(Request::Open { id, fork })?,
                    Effect::Bell => unreachable!(),
                    Effect::Graph(target) => requests.send(Request::Graph(target))?,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;
    use multi_link_tui::Effect;
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn control_c_uses_the_same_quit_effect_as_q() {
        let mut app = App::default();
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(app.update(Event::Key(app_key(key))), [Effect::Quit]);
        let plain = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        assert_eq!(app_key(plain), KeyCode::Char('c'));
        assert!(app.update(Event::Key(app_key(plain))).is_empty());
        let down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert_eq!(app_key(down), KeyCode::Down);
    }

    #[derive(Clone)]
    struct FakeModes {
        calls: Rc<RefCell<Vec<&'static str>>>,
        fail_enter: bool,
    }
    impl Modes for FakeModes {
        fn enter(&self) -> io::Result<()> {
            self.calls.borrow_mut().push("enter");
            if self.fail_enter {
                Err(io::Error::other("setup failed"))
            } else {
                Ok(())
            }
        }
        fn leave(&self) -> io::Result<()> {
            self.calls.borrow_mut().push("leave");
            Ok(())
        }
    }
    #[test]
    fn terminal_restores_after_success_spawn_failure_and_unwind() {
        for outcome in ["success", "spawn failure", "panic"] {
            let calls = Rc::new(RefCell::new(Vec::new()));
            let modes = FakeModes {
                calls: calls.clone(),
                fail_enter: false,
            };
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let screen = Screen::enter(modes).unwrap();
                let suspended = Suspended::new(&screen).unwrap();
                match outcome {
                    "panic" => panic!("child wait panicked"),
                    "spawn failure" => {
                        assert!(
                            std::process::Command::new("/nonexistent-multi-link-test-command")
                                .status()
                                .is_err()
                        );
                    }
                    _ => {}
                }
                suspended.finish().unwrap();
            }));
            assert_eq!(result.is_err(), outcome == "panic");
            assert_eq!(*calls.borrow(), ["enter", "leave", "enter", "leave"]);
        }
    }
    #[test]
    fn partial_setup_and_restore_failure_still_leave_normal_terminal() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        assert!(
            Screen::enter(FakeModes {
                calls: calls.clone(),
                fail_enter: true
            })
            .is_err()
        );
        assert_eq!(*calls.borrow(), ["enter", "leave"]);
        calls.borrow_mut().clear();
        let mut screen = Screen::enter(FakeModes {
            calls: calls.clone(),
            fail_enter: false,
        })
        .unwrap();
        screen.0.fail_enter = true;
        assert!(Suspended::new(&screen).unwrap().finish().is_err());
        drop(screen);
        assert_eq!(
            *calls.borrow(),
            ["enter", "leave", "enter", "enter", "leave"]
        );
    }
    fn options(probe: &dyn multi_link_core::ProcessProbe) -> Options<'_> {
        Options {
            state_dir: PathBuf::new(),
            now: "2026-10-29T23:15:00Z".parse().unwrap(),
            since: None,
            repo_mode: multi_link_core::RepoMode::Fixture,
            probe,
        }
    }
    fn request(app: &mut App) -> (String, bool) {
        let effects = app.update(Event::Key(KeyCode::Char('o')));
        let [Effect::FreshOpen { id, fork }] = effects.as_slice() else {
            panic!("{effects:?}")
        };
        (id.clone(), *fork)
    }
    fn fixtures() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
    }
    /// The worker's options for a fixture root, with the TUI's default 2 h window.
    fn fixture_options<'a>(probe: &'a dyn multi_link_core::ProcessProbe, now: &str) -> Options<'a> {
        Options {
            state_dir: PathBuf::new(),
            now: now.parse().unwrap(),
            since: Some(Duration::from_secs(2 * 3600)),
            repo_mode: multi_link_core::RepoMode::Fixture,
            probe,
        }
    }
    fn graph_target(repo: &str, step: &str) -> (String, String) {
        (repo.to_string(), step.to_string())
    }

    #[test]
    fn graph_steps_are_tagged_plan_steps_then_unplanned_by_first_handoff() {
        use multi_link_model::{Harness, RepoSteps, Step};
        let root = fixtures().join("sessions-phase8");
        let probe = multi_link_core::FixtureProcessProbe;
        let (sessions, links) = Scanner::default()
            .refresh(&root, &fixture_options(&probe, "2026-11-06T18:00:00Z"))
            .unwrap();
        let template = sessions
            .report
            .sessions
            .iter()
            .find(|s| s.harness == Harness::Claude)
            .unwrap()
            .clone();
        let session = |id: &str, repo: &str| {
            let mut s = template.clone();
            s.id = id.into();
            s.repo = repo.into();
            s
        };
        let sessions = vec![session("c1", "/r"), session("c2", "/other")];
        let template = links.report.links[0].clone();
        let link = |parent: &str, step: Option<&str>, at: &str| {
            let mut l = template.clone();
            l.parent = parent.into();
            l.step = step.map(Into::into);
            l.at = at.parse().unwrap();
            l
        };
        let links = vec![
            link("c1", Some("9.9"), "2026-11-06T10:05:00Z"),
            link("c1", Some("1.3"), "2026-11-06T10:04:00Z"),
            link("c1", Some("8.0"), "2026-11-06T10:01:00Z"),
            link("c1", Some("1.1"), "2026-11-06T10:03:00Z"),
            link("c1", Some("9.9"), "2026-11-06T09:00:00Z"),
            link("c1", None, "2026-11-06T10:00:00Z"),
            // Another repo's session and an unknown parent never count.
            link("c2", Some("1.2"), "2026-11-06T10:00:00Z"),
            link("nope", Some("1.2"), "2026-11-06T10:00:00Z"),
        ];
        let step = |id: &str| Step {
            id: id.into(),
            title: String::new(),
            verify_command: None,
            progress: Vec::new(),
        };
        let plan = RepoSteps {
            repo: "/r".into(),
            convention: multi_link_model::Convention::Missing,
            latest_logged_step: None,
            steps: vec![step("1.1"), step("1.2"), step("1.3")],
        };
        assert_eq!(
            graph_steps(&sessions, &links, "/r", Some(&plan)),
            ["1.1", "1.3", "9.9", "8.0"]
        );
        assert_eq!(
            graph_steps(&sessions, &links, "/r", None),
            ["9.9", "8.0", "1.1", "1.3"]
        );
        assert!(graph_steps(&sessions, &links, "/none", Some(&plan)).is_empty());
    }

    #[test]
    fn started_steps_include_children_outside_the_tui_window() {
        // sessions-started at 12:00 with the TUI's 2 h window: K (84c00001) and its child A
        // (84c00002, tagged 2.1, quiet since 09:30) are not in the overview, but K's entry is
        // still there; K's cross-repo child in repo-o is not.
        const K: &str = "84c00001-0000-4000-8000-000000008401";
        let root = fixtures().join("sessions-started");
        let probe = multi_link_core::FixtureProcessProbe;
        let options = fixture_options(&probe, "2026-11-08T12:00:00Z");
        let report = refresh(&mut Scanner::default(), &root, &options, None).unwrap();
        let listed = report.overview.repos.iter().any(|group| {
            multi_link_tui::view::sessions(group).any(|row| row.id.starts_with("84c00002"))
        });
        assert!(!listed, "A must be outside the window for this test");
        assert_eq!(
            report.started_steps.get(K).map(Vec::as_slice),
            Some(&[("/home/dev/repo-k".to_string(), "2.1".to_string())][..])
        );
    }

    #[test]
    fn graph_refresh_on_phase8_fixture() {
        let root = fixtures().join("sessions-phase8");
        let probe = multi_link_core::FixtureProcessProbe;
        let options = fixture_options(&probe, "2026-11-06T18:00:00Z");
        let mut scanner = Scanner::default();
        let plain = refresh(&mut scanner, &root, &options, None).unwrap();
        assert!(plain.graph.is_none() && plain.graph_steps.is_empty());
        let target = graph_target("/home/dev/repo-g", "1.5");
        let report = refresh(&mut scanner, &root, &options, Some(&target)).unwrap();
        let key: serde_json::Value = serde_json::from_slice(
            &std::fs::read(fixtures().join("expected/phase8/graph-1.5.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(report.graph.as_ref().unwrap()).unwrap(),
            key
        );
        assert!(
            report.graph_steps.iter().any(|id| id == "1.5"),
            "{:?}",
            report.graph_steps
        );
        // The overview is the same as without a graph: the 2 h window still applies to it.
        assert_eq!(report.overview, plain.overview);
        assert_eq!(report.diagnostics, plain.diagnostics);
    }

    #[test]
    fn live_indices_follow_worker_step_and_freshness() {
        let root = fixtures().join("sessions-phase8");
        let probe = multi_link_core::FixtureProcessProbe;
        let mut scanner = Scanner::default();
        for (step, now, expected) in [
            (Some("1.5"), "2026-11-06T18:00:00Z", vec![2]),
            (Some("1.1"), "2026-11-06T18:00:00Z", vec![]),
            (Some("1.5"), "2026-11-06T18:00:00Z", vec![2]),
            (Some("1.5"), "2026-11-06T18:06:00Z", vec![]),
            (None, "2026-11-06T18:00:00Z", vec![]),
        ] {
            let target = step.map(|s| graph_target("/home/dev/repo-g", s));
            let report = refresh(
                &mut scanner,
                &root,
                &fixture_options(&probe, now),
                target.as_ref(),
            )
            .unwrap();
            assert_eq!(report.graph_live, expected, "{step:?} at {now}");
        }
    }

    #[test]
    fn stale_threshold_changes_status_without_file_change() {
        use multi_link_model::PartStatus;
        // graph-1.5 part 2 is an open Codex turn whose last record is 17:58:30: working at
        // 18:00 (90 s quiet), stale at 18:06 (7.5 min > 5 min). Only `now` moves.
        let root = fixtures().join("sessions-phase8");
        let probe = multi_link_core::FixtureProcessProbe;
        let target = graph_target("/home/dev/repo-g", "1.5");
        let mut scanner = Scanner::default();
        let status = |scanner: &mut Scanner, now: &str| {
            let report =
                refresh(scanner, &root, &fixture_options(&probe, now), Some(&target)).unwrap();
            let part = &report.graph.as_ref().unwrap().parts[2];
            assert_eq!(
                part.session.as_deref(),
                Some("82520000-0000-4000-8000-000000008252")
            );
            part.status
        };
        assert_eq!(
            status(&mut scanner, "2026-11-06T18:00:00Z"),
            Some(PartStatus::Working)
        );
        assert_eq!(
            status(&mut scanner, "2026-11-06T18:06:00Z"),
            Some(PartStatus::Stale)
        );
    }

    fn phase9_codex_lane(id: &str) {
        let key: serde_json::Value =
            serde_json::from_str(include_str!("../../../fixtures/expected/phase9/lanes.json"))
                .unwrap();
        let lane = key["lanes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|lane| lane["codex"].as_str() == Some(id))
            .unwrap();
        let repo = lane["repo"].as_str().unwrap();
        let root = fixtures().join("sessions-phase9");
        let probe = multi_link_core::FixtureProcessProbe;
        let options = fixture_options(&probe, "2026-11-09T12:00:00Z");
        let report = refresh(&mut Scanner::default(), &root, &options, None).unwrap();
        let group = report
            .overview
            .repos
            .iter()
            .find(|g| g.repo == repo)
            .unwrap();
        let count = multi_link_tui::view::sessions(group).count();
        assert!(
            multi_link_tui::view::sessions(group).any(|row| row.id == id),
            "Codex lane {id} absent from the 2 h overview; do not widen the window"
        );
        let tagged_parent = match id {
            "95c00001-0000-4000-8000-000095c00001" => Some("95000001-0000-4000-8000-000095000001"),
            "95c00002-0000-4000-8000-000095c00002" => Some("95000002-0000-4000-8000-000095000002"),
            _ => None,
        };
        if let Some(parent) = tagged_parent {
            assert_eq!(
                report.codex_steps.get(id).map(String::as_str),
                lane["opens_step"].as_str()
            );
            // Both same-repo tagged parents went quiet before 10:00; window own_step cannot use them.
            assert!(
                !report
                    .overview
                    .repos
                    .iter()
                    .flat_map(multi_link_tui::view::sessions)
                    .any(|row| row.id == parent),
                "tagged parent {parent} unexpectedly present in the 2 h overview"
            );
        }
        let mut app = App::default();
        app.update(Event::Refresh(Ok(report)));
        app.selection = multi_link_tui::selectable_home(
            &app.report.as_ref().unwrap().overview,
            &Default::default(),
        )
        .iter()
        .position(|(r, _)| *r == repo)
        .unwrap();
        assert!(app.update(Event::Key(KeyCode::Enter)).is_empty());
        for _ in 0..count {
            app.update(Event::Key(KeyCode::Up));
        }
        for _ in 0..count {
            let effects = app.update(Event::Key(KeyCode::Char('o')));
            let [Effect::FreshOpen { id: selected, .. }] = effects.as_slice() else {
                panic!("no lane selected")
            };
            let found = selected == id;
            app.update(Event::Opened(String::new()));
            if found {
                break;
            }
            app.update(Event::Key(KeyCode::Down));
        }
        assert!(
            matches!(app.update(Event::Key(KeyCode::Char('o'))).as_slice(),
            [Effect::FreshOpen { id: selected, .. }] if selected == id)
        );
        app.update(Event::Opened(String::new()));
        let effects = app.update(Event::Key(KeyCode::Char('s')));
        if let Some(step) = lane["opens_step"].as_str() {
            assert_eq!(effects, [Effect::Graph(Some(graph_target(repo, step)))]);
        } else {
            assert!(effects.is_empty());
            let area = ratatui::layout::Rect::new(0, 0, 120, 40);
            let mut buf = ratatui::buffer::Buffer::empty(area);
            app.render(area, &mut buf);
            let footer = (0..120).map(|x| buf[(x, 39)].symbol()).collect::<String>();
            assert_eq!(footer.trim_end(), lane["footer"].as_str().unwrap());
        }
    }

    #[test]
    fn phase9_codex_lane_parent_outside_window() {
        phase9_codex_lane("95c00001-0000-4000-8000-000095c00001");
    }

    #[test]
    fn phase9_codex_lane_same_repo_beats_newer_cross_repo() {
        phase9_codex_lane("95c00002-0000-4000-8000-000095c00002");
    }

    #[test]
    fn phase9_codex_lane_untagged_has_no_step() {
        phase9_codex_lane("95c00004-0000-4000-8000-000095c00004");
    }

    #[test]
    fn coordinator_s_end_to_end() {
        // sessions-started: coordinator K (84c00001) started A (84c00002, hand-off tagged 2.1)
        // and B (84c00003, none); A started the grandchild A2. The committed fixture has no
        // other tagged hand-off, so the worker's `started_steps` for K is exactly A's step, and
        // the picker lists it. (K itself is quiet since 09:04, so the overview needs `--since
        // all` for its lane to be listed; the child entries are all-time either way.)
        const K: &str = "84c00001-0000-4000-8000-000000008401";
        let root = fixtures().join("sessions-started");
        let probe = multi_link_core::FixtureProcessProbe;
        let mut options = fixture_options(&probe, "2026-11-08T12:00:00Z");
        options.since = None;
        let mut scanner = Scanner::default();
        let mut app = App::default();
        app.update(Event::Refresh(Ok(refresh(
            &mut scanner,
            &root,
            &options,
            None,
        )
        .unwrap())));
        // K needs no attention, so home lists it only inside repo-k: select repo-k's first home
        // entry, open the repo view, go to the top lane, then walk ↓ until `o` names K (each `o` is
        // answered with `Opened` so the next one is not blocked).
        app.selection = multi_link_tui::selectable_home(
            &app.report.as_ref().unwrap().overview,
            &Default::default(),
        )
        .iter()
        .position(|(repo, _)| *repo == "/home/dev/repo-k")
        .expect("repo-k on home");
        assert!(app.update(Event::Key(KeyCode::Enter)).is_empty());
        for _ in 0..=40 {
            app.update(Event::Key(KeyCode::Up));
        }
        for _ in 0..=40 {
            let effects = app.update(Event::Key(KeyCode::Char('o')));
            let [Effect::FreshOpen { id, .. }] = effects.as_slice() else {
                panic!("no lane selected")
            };
            let found = id == K;
            app.update(Event::Opened(String::new()));
            if found {
                break;
            }
            app.update(Event::Key(KeyCode::Down));
        }
        assert!(
            matches!(app.update(Event::Key(KeyCode::Char('o'))).as_slice(), [Effect::FreshOpen { id, .. }] if id == K),
            "K's lane not reached"
        );
        app.update(Event::Opened(String::new()));
        let footer = |app: &App| {
            let area = ratatui::layout::Rect::new(0, 0, 120, 40);
            let mut buf = ratatui::buffer::Buffer::empty(area);
            app.render(area, &mut buf);
            (0..120)
                .map(|x| buf[(x, 39)].symbol())
                .collect::<String>()
                .trim_end()
                .to_string()
        };
        assert_eq!(
            app.report.as_ref().unwrap().started_steps.get(K),
            Some(&vec![graph_target("/home/dev/repo-k", "2.1")])
        );
        assert!(app.update(Event::Key(KeyCode::Char('s'))).is_empty());
        assert_eq!(
            footer(&app),
            "started: step 2.1 · ←→ choose · enter opens step 2.1 · esc cancel"
        );
        let target = graph_target("/home/dev/repo-k", "2.1");
        assert_eq!(
            app.update(Event::Key(KeyCode::Enter)),
            [Effect::Graph(Some(target.clone()))]
        );
        // The worker's next refresh carries the graph; the App shows it.
        let report = refresh(&mut scanner, &root, &options, Some(&target)).unwrap();
        let graph = report.graph.clone().unwrap();
        assert_eq!(
            (graph.repo.as_str(), graph.step.id.as_str()),
            ("/home/dev/repo-k", "2.1")
        );
        assert!(!graph.parts.is_empty());
        assert_eq!(report.graph_steps, ["2.1"]);
        app.update(Event::Refresh(Ok(report)));
        assert!(
            !footer(&app).starts_with("loading step"),
            "{}",
            footer(&app)
        );
    }

    #[test]
    fn app_open_running_fixture_uses_fresh_pass_and_never_resumes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sessions");
        let mut scanner = Scanner::default();
        let options = options(&multi_link_core::FixtureProcessProbe);
        let mut app = App::default();
        let report = refresh(&mut scanner, &root, &options, None).unwrap();
        app.update(Event::Refresh(Ok(report)));
        app.selection = multi_link_tui::selectable_home(&app.report.as_ref().unwrap().overview, &Default::default())
            .iter().position(|(_, target)| matches!(target, multi_link_tui::Target::Session(row) if row.id == "00000000-0000-4000-8000-000000000010")).unwrap();
        let (id, fork) = request(&mut app);
        assert_eq!(id, "00000000-0000-4000-8000-000000000010");
        let action = open::fresh_action(&mut scanner, &root, &options, &id, fork).unwrap();
        assert!(!matches!(action, Some(OpenAction::Resume { .. })));
        assert_eq!(
            open_result(Ok(action)).unwrap_err(),
            format!("running_without_app_link; in /home/dev/repo-c: claude --resume {id}")
        );
    }
    #[test]
    fn drawn_finished_but_fresh_running_never_resumes_and_errors_never_execute() {
        use multi_link_core::{FakeProcessProbe, Liveness};
        let root = tempfile::tempdir().unwrap();
        let id = "aaaaaaaa-0000-4000-8000-000000000001";
        let transcript = root.path().join(".claude/projects/project");
        let records = root.path().join(".claude/sessions");
        std::fs::create_dir_all(&transcript).unwrap();
        std::fs::create_dir_all(&records).unwrap();
        std::fs::write(
            transcript.join(format!("{id}.jsonl")),
            "{\"type\":\"user\",\"cwd\":\"/repo\",\"timestamp\":\"2026-10-29T23:00:00Z\"}\n",
        )
        .unwrap();
        std::fs::write(records.join("42.json"), serde_json::json!({"pid":42,"procStart":"start","sessionId":id,"cwd":"/repo","status":"idle","statusUpdatedAt":1793315700000_i64,"startedAt":1793314800000_i64,"updatedAt":1793315700000_i64}).to_string()).unwrap();
        let mut probe = FakeProcessProbe::default();
        probe.results.insert((42, "start".into()), Liveness::Gone);
        let mut scanner = Scanner::default();
        let mut app = App::default();
        app.update(Event::Refresh(Ok(refresh(
            &mut scanner,
            root.path(),
            &options(&probe),
            None,
        )
        .unwrap())));
        let multi_link_model::Row::Session(row) =
            &app.report.as_ref().unwrap().overview.repos[0].rows[0]
        else {
            panic!("session")
        };
        assert_eq!(row.state, multi_link_model::State::Finished);
        app.update(Event::Key(KeyCode::Enter));
        let (selected, fork) = request(&mut app);
        assert_eq!(selected, id);
        probe.results.insert((42, "start".into()), Liveness::Alive);
        let action =
            open::fresh_action(&mut scanner, root.path(), &options(&probe), &selected, fork)
                .unwrap();
        assert_eq!(
            open_result(Ok(action)).unwrap_err(),
            format!("running_without_app_link; in /repo: claude --resume {id}")
        );
        assert_eq!(
            open_result(Err("denied".into())).unwrap_err(),
            "open check failed: denied"
        );
        assert_eq!(
            open_result(Ok(None)).unwrap_err(),
            "session no longer found"
        );
        // ENOTDIR is deterministic even when tests run with elevated filesystem privileges.
        std::fs::write(transcript.join(id), "not a directory").unwrap();
        let result =
            refresh(&mut scanner, root.path(), &options(&probe), None).map_err(|e| e.to_string());
        assert!(result.is_err());
        app.update(Event::Opened(String::new()));
        app.update(Event::Refresh(result));
        assert!(app.update(Event::Key(KeyCode::Char('o'))).is_empty());
    }
}
