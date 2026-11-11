//! The pane projects the worker's repo view, using its labels, placement and status words.
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use clap::Args;
use multi_link_core::{
    FixtureProcessProbe, Options, ProcessProbe, RealProcessProbe, RepoMode, Scanner,
};
use multi_link_model::{Harness, Match, SessionRow, State, Timestamp};
use multi_link_tui::{Refresh, view};
use serde::Serialize;

#[derive(Args)]
pub struct PaneArgs {
    /// Print the pane object as pretty JSON.
    #[arg(long, required = true)]
    json: bool,
    /// Stream compact JSON changes and a heartbeat every 30 seconds.
    #[arg(long)]
    watch: bool,
    /// Home-shaped root; enables fixture mode.
    #[arg(long)]
    root: Option<PathBuf>,
    #[arg(long)]
    now: Option<Timestamp>,
    /// Checkout directory to map to a repo.
    #[arg(long)]
    cwd: PathBuf,
    /// Current Claude Code session id.
    #[arg(long)]
    session: String,
}

#[derive(Debug, Serialize)]
struct Pane {
    v: u8,
    repo: String,
    now: Timestamp,
    #[serde(rename = "this")]
    this_session: String,
    seen: bool,
    sections: Vec<Section>,
}
#[derive(Debug, Serialize)]
struct Section {
    name: &'static str,
    rows: Vec<PaneRow>,
}
#[derive(Debug, Serialize)]
struct PaneRow {
    id: String,
    harness: Harness,
    label: String,
    step: Option<String>,
    state: State,
    phrase: String,
    is_this: bool,
    children: Vec<Child>,
    handoffs: Vec<Handoff>,
}
#[derive(Debug, Serialize)]
struct Child {
    id: String,
    harness: Harness,
    state: State,
    phrase: String,
    r#match: &'static str,
}
#[derive(Debug, Serialize)]
struct Handoff {
    step: Option<String>,
    outcome: String,
    candidates: Vec<String>,
}

fn visible(row: &SessionRow) -> bool {
    !row.imported
        && (!row.activity.is_empty() || matches!(row.state, State::Waiting | State::Stale))
}

fn newest_first(a: &&SessionRow, b: &&SessionRow) -> std::cmp::Ordering {
    b.started_at
        .cmp(&a.started_at)
        .then_with(|| a.id.cmp(&b.id))
}

fn project(refresh: &Refresh, cwd: &Path, mode: RepoMode, this: &str) -> Pane {
    let repo = mode.map(cwd).repo.to_string_lossy().into_owned();
    let now = refresh.overview.now;
    let offset = view::local_offset(now);
    let mut pane = Pane {
        v: 1,
        repo,
        now,
        this_session: this.into(),
        seen: false,
        sections: ["working", "waiting", "recent"]
            .map(|name| Section {
                name,
                rows: Vec::new(),
            })
            .into(),
    };
    let Some(group) = refresh.overview.repos.iter().find(|g| g.repo == pane.repo) else {
        return pane;
    };
    let sessions: Vec<_> = view::sessions(group).collect();
    // Overview already chose the placing parent (T11). Preserve it even when hidden,
    // for the Codex label/step; only visible Claude parents receive nested children.
    let parent = |row: &SessionRow| {
        sessions
            .iter()
            .copied()
            .find(|p| p.children.iter().any(|c| c.id == row.id))
    };
    let nested = |row: &SessionRow| {
        row.harness == Harness::Codex
            && parent(row).is_some_and(|p| p.harness == Harness::Claude && visible(p))
    };
    let mut rows: Vec<_> = sessions
        .iter()
        .copied()
        .filter(|r| visible(r) && !nested(r))
        .collect();
    rows.sort_by(|a, b| {
        (b.id == this)
            .cmp(&(a.id == this))
            .then_with(|| newest_first(a, b))
    });
    for row in rows {
        let mut children: Vec<_> = row
            .children
            .iter()
            .filter(|c| visible(c) && c.harness == Harness::Codex)
            .collect();
        children.sort_by(newest_first);
        let children = children
            .into_iter()
            .map(|child| Child {
                id: child.id.clone(),
                harness: child.harness,
                state: child.state,
                phrase: view::status(child, now, offset),
                r#match: match view::placing(child, Some(&row.id)).map(|l| l.r#match) {
                    Some(Match::Exact) => "exact",
                    _ => "likely",
                },
            })
            .collect::<Vec<_>>();
        let mut handoffs: Vec<_> = refresh
            .links
            .links
            .iter()
            .filter(|l| {
                row.harness == Harness::Claude
                    && l.parent == row.id
                    && matches!(l.r#match, Match::Ambiguous | Match::Unlinked)
            })
            .collect();
        handoffs.sort_by(|a, b| (a.at, &a.invocation).cmp(&(b.at, &b.invocation)));
        let handoffs = handoffs
            .into_iter()
            .map(|link| Handoff {
                step: link.step.clone(),
                outcome: view::outcome(link, &refresh.overview),
                candidates: link.candidates.iter().map(|c| c.id.clone()).collect(),
            })
            .collect();
        pane.seen |= row.id == this || children.iter().any(|c| c.id == this);
        let section = match row.state {
            State::Busy => 0,
            State::Waiting | State::Stale => 1,
            _ => 2,
        };
        pane.sections[section].rows.push(PaneRow {
            id: row.id.clone(),
            harness: row.harness,
            label: view::home_label(group, row),
            step: if row.harness == Harness::Claude {
                row.made.first().cloned()
            } else {
                view::placing(row, parent(row).map(|p| p.id.as_str())).and_then(|l| l.step.clone())
            },
            state: row.state,
            phrase: view::status(row, now, offset),
            is_this: row.id == this,
            children,
            handoffs,
        });
    }
    pane
}

type WatchError = Box<dyn std::error::Error>;
const TICK: Duration = Duration::from_secs(3);
const HEARTBEAT: Duration = Duration::from_secs(30);

trait WatchRuntime {
    fn parent(&self) -> u32;
    fn elapsed(&self) -> Duration;
    fn now(&self) -> Result<Timestamp, WatchError>;
    fn wait(&mut self, duration: Duration);
}

struct Runtime {
    start: Instant,
    fixed_now: Option<Timestamp>,
}
impl WatchRuntime for Runtime {
    fn parent(&self) -> u32 {
        #[cfg(unix)]
        {
            std::os::unix::process::parent_id()
        }
        #[cfg(not(unix))]
        {
            // Parent checking is Unix-only; teardown owns the watcher elsewhere.
            0
        }
    }
    fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
    fn now(&self) -> Result<Timestamp, WatchError> {
        self.fixed_now.map_or_else(super::clock_now, Ok)
    }
    fn wait(&mut self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

/// Compare data rather than its clock or human-readable age, including nested children.
fn without_times(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(fields) => {
            fields.remove("now");
            fields.remove("phrase");
            for field in fields.values_mut() {
                without_times(field);
            }
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(without_times),
        _ => {}
    }
}

fn watch(
    out: &mut impl Write,
    runtime: &mut impl WatchRuntime,
    parent: u32,
    mut pass: impl FnMut(Timestamp) -> Result<serde_json::Value, WatchError>,
) -> Result<(), WatchError> {
    let mut previous = None;
    let mut printed_at = Duration::ZERO;
    while parent != 1 && runtime.parent() == parent {
        let tick_start = runtime.elapsed();
        let result = runtime.now().and_then(&mut pass);
        // A parent lost during the refresh must not leave an orphan printing data.
        if runtime.parent() != parent {
            break;
        }
        let (value, key) = match result {
            Ok(value) => {
                let mut key = value.clone();
                without_times(&mut key);
                if previous.as_ref() == Some(&key)
                    && runtime.elapsed().saturating_sub(printed_at) < HEARTBEAT
                {
                    runtime.wait(TICK.saturating_sub(runtime.elapsed() - tick_start));
                    continue;
                }
                (value, Some(key))
            }
            Err(error) => (
                serde_json::json!({"v": 1, "error": error.to_string()}),
                None,
            ),
        };
        serde_json::to_writer(&mut *out, &value)?;
        out.write_all(b"\n")?;
        out.flush()?;
        previous = key;
        printed_at = runtime.elapsed();
        runtime.wait(TICK.saturating_sub(runtime.elapsed() - tick_start));
    }
    Ok(())
}

pub fn run(args: PaneArgs) -> Result<(), WatchError> {
    let mut runtime = Runtime {
        start: Instant::now(),
        fixed_now: args.now,
    };
    // Capture the parent before even resolving the home or running a cold pass.
    let parent = runtime.parent();
    if args.watch && parent == 1 {
        return Ok(());
    }
    let fixture_probe = FixtureProcessProbe;
    let real_probe = RealProcessProbe::default();
    let (root, repo_mode, probe): (_, _, &dyn ProcessProbe) = match args.root {
        Some(root) => (root, RepoMode::Fixture, &fixture_probe),
        None => (super::home()?, RepoMode::Real, &real_probe),
    };
    let mut scanner = Scanner::default();
    let mut pass = |now| -> Result<Pane, WatchError> {
        let options = Options {
            state_dir: super::state_dir(&root),
            now,
            since: Some(Duration::from_secs(2 * 60 * 60)),
            repo_mode,
            probe,
        };
        let refresh = super::tui::refresh_with_live(&mut scanner, &root, &options, None, false)?;
        Ok(project(&refresh, &args.cwd, repo_mode, &args.session))
    };
    let mut out = io::stdout().lock();
    if args.watch {
        // If setup lost the original parent, do not adopt the new one.
        if runtime.parent() != parent {
            return Ok(());
        }
        watch(&mut out, &mut runtime, parent, |now| {
            Ok(serde_json::to_value(pass(now)?)?)
        })
    } else {
        super::json(&mut out, &pass(runtime.now()?)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use multi_link_model::Row;
    use std::{fs, process::Command};

    const REPO: &str = "/home/dev/repo-m";
    const OWNER: &str = "14110000-0000-4000-8000-000000001411";

    fn fixture() -> Box<Refresh> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sessions-phase14");
        let options = Options {
            state_dir: super::super::state_dir(&root),
            now: Timestamp::parse("2026-11-10T22:20:00Z").unwrap(),
            since: Some(Duration::from_secs(7200)),
            repo_mode: RepoMode::Fixture,
            probe: &FixtureProcessProbe,
        };
        super::super::tui::refresh_with_live(&mut Scanner::default(), &root, &options, None, false)
            .unwrap()
    }

    fn pane(refresh: &Refresh, this: &str) -> Pane {
        project(refresh, Path::new(REPO), RepoMode::Fixture, this)
    }

    fn sample(refresh: &Refresh) -> SessionRow {
        view::sessions(
            refresh
                .overview
                .repos
                .iter()
                .find(|g| g.repo == REPO)
                .unwrap(),
        )
        .find(|r| r.id == OWNER)
        .unwrap()
        .clone()
    }

    fn set_rows(refresh: &mut Refresh, rows: Vec<SessionRow>) {
        refresh
            .overview
            .repos
            .iter_mut()
            .find(|g| g.repo == REPO)
            .unwrap()
            .rows = rows.into_iter().map(Row::Session).collect();
    }

    #[test]
    fn started_claude_child_is_its_own_row_in_its_state_section() {
        let mut refresh = fixture();
        let mut parent = sample(&refresh);
        let mut child = parent.clone();
        child.id = "started-claude-child".into();
        child.started_by = Some(parent.id.clone());
        child.state = State::Waiting;
        child.children.clear();
        parent.children = vec![child];
        set_rows(&mut refresh, vec![parent]);
        let pane = pane(&refresh, "started-claude-child");
        assert_eq!(pane.sections[0].rows.len(), 1);
        assert_eq!(pane.sections[0].rows[0].id, OWNER);
        assert!(pane.sections[0].rows[0].children.is_empty());
        assert_eq!(pane.sections[1].rows.len(), 1);
        assert_eq!(pane.sections[1].rows[0].id, "started-claude-child");
        assert!(pane.sections[1].rows[0].is_this);
        assert!(pane.seen);
        assert!(pane.sections[2].rows.is_empty());
    }

    #[test]
    fn unlinked_handoff_has_words_and_no_child() {
        let mut refresh = fixture();
        let mut parent = sample(&refresh);
        let mut link = refresh
            .links
            .links
            .iter()
            .find(|l| l.parent == OWNER)
            .unwrap()
            .clone();
        link.r#match = Match::Unlinked;
        link.child = None;
        link.candidates.clear();
        link.step = Some("16.1".into());
        refresh.links.links = vec![link];
        parent.children.clear();
        set_rows(&mut refresh, vec![parent]);
        let pane = pane(&refresh, OWNER);
        let row = &pane.sections[0].rows[0];
        assert!(row.children.is_empty());
        assert_eq!(row.handoffs.len(), 1);
        assert_eq!(row.handoffs[0].step.as_deref(), Some("16.1"));
        assert_eq!(row.handoffs[0].outcome, "no codex run found");
        assert!(row.handoffs[0].candidates.is_empty());
    }

    #[test]
    fn this_first_then_flat_newest_first_then_id_and_no_imports() {
        let mut refresh = fixture();
        let mut owner = sample(&refresh);
        owner.children.clear();
        let mut newer = owner.clone();
        newer.id = "b-newer".into();
        newer.started_at = refresh.overview.now;
        let mut tied = newer.clone();
        tied.id = "a-tied".into();
        tied.started_by = Some(owner.id.clone());
        newer.children = vec![tied];
        let mut imported = owner.clone();
        imported.id = "imported".into();
        imported.imported = true;
        set_rows(&mut refresh, vec![owner, newer, imported]);
        let pane = pane(&refresh, OWNER);
        assert_eq!(
            pane.sections[0]
                .rows
                .iter()
                .map(|r| r.id.as_str())
                .collect::<Vec<_>>(),
            [OWNER, "a-tied", "b-newer"]
        );
        assert!(!self::pane(&refresh, "imported").seen);
    }

    #[test]
    fn hidden_parent_promotes_codex_but_preserves_placing_step_and_label() {
        let mut refresh = fixture();
        let mut parent = sample(&refresh);
        let child_id = parent.children[0].id.clone();
        parent.imported = true;
        set_rows(&mut refresh, vec![parent]);
        let pane = pane(&refresh, &child_id);
        let rows: Vec<_> = pane.sections.iter().flat_map(|s| &s.rows).collect();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, child_id);
        assert_eq!(rows[0].step.as_deref(), Some("2.3"));
        assert_eq!(rows[0].label, "step 2.3");
        assert!(rows[0].children.is_empty());
        assert!(pane.seen);
    }

    #[test]
    fn rows_labels_and_phrases_equal_public_repo_view_functions() {
        let refresh = fixture();
        for group in &refresh.overview.repos {
            let pane = project(
                &refresh,
                Path::new(&group.repo),
                RepoMode::Fixture,
                "unknown",
            );
            assert!(!pane.seen);
            let expected: Vec<_> = view::sessions(group).filter(|r| visible(r)).collect();
            let actual_count: usize = pane
                .sections
                .iter()
                .flat_map(|s| &s.rows)
                .map(|r| 1 + r.children.len())
                .sum();
            assert_eq!(actual_count, expected.len(), "{}", group.repo);
            for row in pane.sections.iter().flat_map(|s| &s.rows) {
                let source = expected.iter().find(|r| r.id == row.id).unwrap();
                assert_eq!(row.label, view::home_label(group, source));
                assert_eq!(
                    row.phrase,
                    view::status(
                        source,
                        refresh.overview.now,
                        view::local_offset(refresh.overview.now)
                    )
                );
                for child in &row.children {
                    let source = expected.iter().find(|r| r.id == child.id).unwrap();
                    assert_eq!(
                        child.phrase,
                        view::status(
                            source,
                            refresh.overview.now,
                            view::local_offset(refresh.overview.now)
                        )
                    );
                }
            }
        }
    }

    fn git(path: &Path, args: &[&str]) {
        let mut command = Command::new("git");
        for (name, _) in std::env::vars_os() {
            if name.as_encoded_bytes().starts_with(b"GIT_") {
                command.env_remove(name);
            }
        }
        let output = command.arg("-C").arg(path).args(args).output().unwrap();
        assert!(output.status.success(), "git {args:?}: {output:?}");
    }

    #[test]
    fn temporary_git_worktree_cwds_share_one_pane_from_real_refresh() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let main = root.join("main");
        let tree = root.join("tree");
        fs::create_dir(&main).unwrap();
        git(&main, &["init", "-b", "main"]);
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
            &["worktree", "add", "-b", "step", tree.to_str().unwrap()],
        );
        let transcripts = root.join(".claude/projects/synthetic");
        fs::create_dir_all(&transcripts).unwrap();
        let mut cwds = Vec::new();
        for (index, checkout) in [&main, &tree].iter().enumerate() {
            let cwd = checkout.join("nested");
            fs::create_dir(&cwd).unwrap();
            let id = format!("aaaaaaaa-0000-4000-8000-{index:012}");
            let record = serde_json::json!({
                "type": "user", "sessionId": id, "cwd": cwd, "timestamp": "2026-11-10T22:19:00Z",
                "uuid": format!("synthetic-{index}"), "parentUuid": null,
                "message": {"role": "user", "content": "synthetic worktree test"}
            });
            fs::write(
                transcripts.join(format!("{id}.jsonl")),
                format!("{record}\n"),
            )
            .unwrap();
            cwds.push(cwd);
        }
        let options = Options {
            state_dir: super::super::state_dir(&root),
            now: Timestamp::parse("2026-11-10T22:20:00Z").unwrap(),
            since: Some(Duration::from_secs(7200)),
            repo_mode: RepoMode::Real,
            probe: &FixtureProcessProbe,
        };
        let refresh = super::super::tui::refresh_with_live(
            &mut Scanner::default(),
            &root,
            &options,
            None,
            false,
        )
        .unwrap();
        assert_eq!(refresh.overview.repos.len(), 1);
        let expected = [
            "aaaaaaaa-0000-4000-8000-000000000000",
            "aaaaaaaa-0000-4000-8000-000000000001",
        ];
        for cwd in cwds {
            let pane = project(&refresh, &cwd, RepoMode::Real, expected[0]);
            assert_eq!(pane.repo, main.to_str().unwrap());
            let ids: Vec<_> = pane
                .sections
                .iter()
                .flat_map(|s| &s.rows)
                .map(|r| r.id.as_str())
                .collect();
            assert_eq!(ids, expected);
            assert!(pane.seen);
        }
        let empty = project(&refresh, &root.join("missing"), RepoMode::Real, expected[0]);
        assert_eq!(empty.repo, root.join("missing").to_str().unwrap());
        assert!(!empty.seen);
    }
}

#[cfg(test)]
mod watch_tests {
    use super::*;
    use serde_json::{Value, json};

    struct FakeRuntime {
        tick: u64,
        end: u64,
        initial_parent: u32,
    }
    impl WatchRuntime for FakeRuntime {
        fn parent(&self) -> u32 {
            if self.tick < self.end {
                self.initial_parent
            } else {
                99
            }
        }
        fn elapsed(&self) -> Duration {
            Duration::from_secs(self.tick * 3)
        }
        fn now(&self) -> Result<Timestamp, Box<dyn std::error::Error>> {
            Ok(Timestamp::from_epoch_ms(
                Timestamp::parse("2026-11-10T22:20:00Z").unwrap().epoch_ms()
                    + self.tick as i64 * 3000,
            )
            .unwrap())
        }
        fn wait(&mut self, duration: Duration) {
            assert_eq!(duration, Duration::from_secs(3));
            self.tick += 1;
        }
    }
    #[derive(Default)]
    struct Output {
        bytes: Vec<u8>,
        flushes: usize,
        flushed: std::rc::Rc<std::cell::Cell<usize>>,
    }
    impl io::Write for Output {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            self.flushed.set(self.flushes);
            Ok(())
        }
    }
    fn object() -> Value {
        serde_json::from_str(include_str!(
            "../../../fixtures/expected/pane/children-exact-likely.json"
        ))
        .unwrap()
    }
    #[test]
    fn watch_first_change_heartbeat_and_all_phrases_are_filtered_and_flushed() {
        let mut output = Output::default();
        let mut runtime = FakeRuntime {
            tick: 0,
            end: 13,
            initial_parent: 42,
        };
        let mut passes = 0;
        let flushed = output.flushed.clone();
        watch(&mut output, &mut runtime, 42, |now| {
            assert_eq!(
                flushed.get(),
                match passes {
                    0 => 0,
                    1 => 1,
                    2..=11 => 2,
                    _ => 3,
                }
            );
            let mut value = object();
            value["now"] = json!(now);
            for section in value["sections"].as_array_mut().unwrap() {
                for row in section["rows"].as_array_mut().unwrap() {
                    row["phrase"] = json!(format!("row {passes}"));
                    for child in row["children"].as_array_mut().unwrap() {
                        child["phrase"] = json!(format!("child {passes}"));
                    }
                }
            }
            if passes >= 1 {
                value["sections"][0]["rows"][0]["children"][0]["state"] = json!("waiting");
            }
            passes += 1;
            Ok(value)
        })
        .unwrap();
        let lines: Vec<Value> = String::from_utf8(output.bytes)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        // Immediate cold pass, actual change at 3 s, heartbeat at 33 s.
        assert_eq!(
            lines
                .iter()
                .map(|v| v["now"].as_str().unwrap())
                .collect::<Vec<_>>(),
            [
                "2026-11-10T22:20:00.000Z",
                "2026-11-10T22:20:03.000Z",
                "2026-11-10T22:20:33.000Z"
            ]
        );
        assert_eq!(
            lines[0]["sections"][0]["rows"][0]["children"][0]["state"],
            "busy"
        );
        assert_eq!(
            lines[1]["sections"][0]["rows"][0]["children"][0]["state"],
            "waiting"
        );
        for line in &lines {
            assert_eq!(line.as_object().unwrap().len(), 6);
            assert_eq!(line["v"], 1);
        }
        assert_eq!(output.flushes, 3);
        assert_eq!(passes, 13);
    }
    #[test]
    fn watch_errors_continue_and_unchanged_success_restores_data() {
        let mut output = Output::default();
        let mut runtime = FakeRuntime {
            tick: 0,
            end: 4,
            initial_parent: 42,
        };
        let mut passes = 0;
        watch(&mut output, &mut runtime, 42, |_| {
            passes += 1;
            if passes == 1 || passes == 3 {
                Err(io::Error::other("broken \"pass\"\nretry").into())
            } else {
                Ok(object())
            }
        })
        .unwrap();
        let lines: Vec<Value> = String::from_utf8(output.bytes)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0], json!({"v": 1, "error": "broken \"pass\"\nretry"}));
        assert_eq!(lines[1], object());
        assert_eq!(lines[2], lines[0]);
        assert_eq!(lines[3], object());
        assert_eq!(output.flushes, 4);
    }
    #[test]
    fn watch_parent_one_exits_before_clock_or_pass() {
        let mut output = Output::default();
        let mut runtime = FakeRuntime {
            tick: 0,
            end: 5,
            initial_parent: 1,
        };
        watch(&mut output, &mut runtime, 1, |_| {
            panic!("orphan must not refresh")
        })
        .unwrap();
        assert!(output.bytes.is_empty());
        assert_eq!(runtime.tick, 0);
    }
}
