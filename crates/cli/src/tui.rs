//! Terminal runtime. Only the worker touches disk; only this thread reads terminal input.
use std::{
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
use multi_link_model::Timestamp;
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
    Open { id: String, fork: bool },
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

fn refresh(scanner: &mut Scanner, root: &Path, options: &Options<'_>) -> io::Result<Box<Refresh>> {
    let mut plans = |folder: &Path| {
        multi_link_plan::steps_report(Path::new(""), &[folder.to_string_lossy().into_owned()])
            .repos
            .remove(0)
    };
    let output = scanner.overview(root, options, &mut plans)?;
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
    Ok(Box::new(Refresh {
        overview: output.report,
        links: output.links.report,
        diagnostics,
    }))
}

fn worker(args: TuiArgs, root: PathBuf, requests: Receiver<Request>, messages: Sender<Message>) {
    let fixture = multi_link_core::FixtureProcessProbe;
    let real = multi_link_core::RealProcessProbe::default();
    let mut scanner = Scanner::default();
    let mut next_refresh = Instant::now();
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
            let result = refresh(&mut scanner, &root, &options).map_err(|error| error.to_string());
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
    thread::Builder::new()
        .name("multi-link-refresh".into())
        .spawn(move || worker(args, root, receiver, sender))?;
    let mut app = App::default();
    app.theme = Some(theme);
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
    #[test]
    fn app_open_running_fixture_uses_fresh_pass_and_never_resumes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sessions");
        let mut scanner = Scanner::default();
        let options = options(&multi_link_core::FixtureProcessProbe);
        let mut app = App::default();
        let report = refresh(&mut scanner, &root, &options).unwrap();
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
            refresh(&mut scanner, root.path(), &options(&probe)).map_err(|e| e.to_string());
        assert!(result.is_err());
        app.update(Event::Opened(String::new()));
        app.update(Event::Refresh(result));
        assert!(app.update(Event::Key(KeyCode::Char('o'))).is_empty());
    }
}
