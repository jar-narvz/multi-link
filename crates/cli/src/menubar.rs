//! All-repository session reports and completion notifications.
use crate::rows::{
    self, DoneRun, FileMarkers, MarkerStore, Runtime, TurnCache, WatchError, WatchRuntime,
};
use clap::Args;
use multi_link_core::{
    FixtureProcessProbe, Options, ProcessProbe, RealProcessProbe, RepoMode, Scanner,
};
use multi_link_model::{Session, State, Timestamp};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeSet, HashSet},
    io,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Args)]
pub struct MenubarArgs {
    /// Print the menu-bar report as JSON.
    #[arg(long, required = true)]
    json: bool,
    /// Stream changes and checked ticks every 3 seconds.
    #[arg(long)]
    watch: bool,
    /// Emit at-most-once Codex completion events (watch mode only).
    #[arg(long, requires = "watch")]
    notify: bool,
    /// Home-shaped root; enables fixture mode.
    #[arg(long)]
    root: Option<PathBuf>,
    #[arg(long)]
    now: Option<Timestamp>,
}

#[derive(Serialize)]
struct Report {
    v: u8,
    now: Timestamp,
    waiting: usize,
    running: bool,
    repos: Vec<Repo>,
}
#[derive(Serialize)]
struct Repo {
    repo: String,
    name: String,
    tier: &'static str,
    last_activity: Timestamp,
    step: Option<Step>,
    rows: Vec<Value>,
}
#[derive(Serialize)]
struct Step {
    id: String,
    title: Option<String>,
    more: usize,
}

fn project(
    refresh: &multi_link_tui::Refresh,
    sessions: &[Session],
    root: &Path,
    mode: RepoMode,
    resolver: &multi_link_core::open::Resolver,
    cache: &mut TurnCache,
) -> Result<Report, WatchError> {
    project_with_rows(refresh.overview.now, sessions, |repo| {
        rows::rows(
            refresh,
            Path::new(repo),
            mode,
            root,
            sessions,
            resolver,
            cache,
        )
    })
}

fn project_with_rows(
    now: Timestamp,
    sessions: &[Session],
    mut rows_for: impl FnMut(&str) -> Result<Value, WatchError>,
) -> Result<Report, WatchError> {
    let mut report = Report {
        v: 1,
        now,
        waiting: 0,
        running: false,
        repos: Vec::new(),
    };
    // Avoid mapping repos and enriching rows that the row projection cannot show. Quiet
    // waiting/stale sessions remain candidates even outside the activity window.
    let repos: BTreeSet<_> = sessions
        .iter()
        .filter(|s| {
            !s.imported && (in_window(s, now) || matches!(s.state, State::Waiting | State::Stale))
        })
        .map(|s| s.repo.as_str())
        .collect();
    for repo in repos {
        let projected = rows_for(repo)?;
        let mut rows = Vec::new();
        let mut steps = Vec::new();
        let mut shown = HashSet::new();
        let mut waiting = false;
        let mut busy = false;
        for section in projected["sections"].as_array().unwrap() {
            for row in section["rows"].as_array().unwrap() {
                let mut flattened = json!({"section": section["name"]});
                for (key, value) in row.as_object().unwrap() {
                    flattened[key] = value.clone();
                }
                if matches!(row["state"].as_str(), Some("busy" | "waiting" | "stale"))
                    && let Some(step) = row["step"].as_str()
                    && !steps.iter().any(|s| s == step)
                {
                    steps.push(step.to_owned());
                }
                for item in std::iter::once(row).chain(row["children"].as_array().unwrap()) {
                    shown.insert(item["id"].as_str().unwrap());
                    waiting |= item["state"] == "waiting";
                    busy |= item["state"] == "busy";
                    report.running |= item["state"] == "busy";
                    if item["harness"] == "claude"
                        && matches!(item["state"].as_str(), Some("waiting" | "stale"))
                    {
                        report.waiting += 1;
                    }
                }
                rows.push(flattened);
            }
        }
        if rows.is_empty() {
            continue;
        }
        let last_activity = sessions
            .iter()
            .filter(|s| shown.contains(s.id.as_str()))
            .map(|s| s.last_activity)
            .max()
            .unwrap();
        let step = steps.first().map(|id| {
            let title = std::fs::read_to_string(Path::new(repo).join("PLAN.md"))
                .ok()
                .and_then(|text| multi_link_plan::parse_plan(&text))
                .and_then(|steps| steps.into_iter().find(|s| &s.id == id))
                .map(|s| s.title);
            Step {
                id: id.clone(),
                title,
                more: steps.len() - 1,
            }
        });
        report.repos.push(Repo {
            repo: repo.into(),
            name: Path::new(repo)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            tier: if waiting {
                "waiting"
            } else if busy {
                "busy"
            } else {
                "other"
            },
            last_activity,
            step,
            rows,
        });
    }
    let rank = |tier| match tier {
        "waiting" => 0,
        "busy" => 1,
        _ => 2,
    };
    report.repos.sort_by(|a, b| {
        rank(a.tier)
            .cmp(&rank(b.tier))
            .then_with(|| b.last_activity.cmp(&a.last_activity))
            .then_with(|| a.repo.cmp(&b.repo))
    });
    Ok(report)
}

const WINDOW: Duration = Duration::from_secs(2 * 60 * 60);

fn in_window(session: &Session, now: Timestamp) -> bool {
    session.last_activity.epoch_ms() >= now.epoch_ms() - WINDOW.as_millis() as i64
}

fn completion_repos(sessions: &[Session], now: Timestamp) -> BTreeSet<&str> {
    // A recently active session may still complete a hand-off. Keep imported
    // sessions here: notification tracking is independent of row visibility.
    sessions
        .iter()
        .filter(|s| in_window(s, now))
        .map(|s| s.repo.as_str())
        .collect()
}

/// Run the shared watcher with permanent completion markers.
fn watch(
    out: &mut impl io::Write,
    runtime: &mut impl WatchRuntime,
    parent: u32,
    notify: bool,
    markers: &impl MarkerStore,
    pass: impl FnMut(Timestamp) -> Result<(Value, Vec<DoneRun>), WatchError>,
) -> Result<(), WatchError> {
    rows::watch_passes(out, runtime, parent, notify, markers, pass)
}

pub fn run(args: MenubarArgs) -> Result<(), WatchError> {
    let mut runtime = Runtime {
        start: Instant::now(),
        fixed_now: args.now,
    };
    let parent = runtime.parent();
    if args.watch && parent == 1 {
        return Ok(());
    }
    let fixture_probe = FixtureProcessProbe;
    let real_probe = RealProcessProbe::default();
    let (root, mode, probe): (_, _, &dyn ProcessProbe) = match args.root {
        Some(root) => (root, RepoMode::Fixture, &fixture_probe),
        None => (super::home()?, RepoMode::Real, &real_probe),
    };
    let mut scanner = Scanner::default();
    let mut cache = TurnCache::default();
    let markers = FileMarkers(super::state_dir(&root));
    let mut pass = |now| {
        let options = Options {
            state_dir: super::state_dir(&root),
            now,
            since: Some(WINDOW),
            repo_mode: mode,
            probe,
        };
        let refresh = super::tui::refresh_with_live(&mut scanner, &root, &options, None, false)?;
        let all = Options {
            since: None,
            state_dir: options.state_dir.clone(),
            ..options
        };
        let (sessions, links) = scanner.refresh(&root, &all)?;
        let sessions = sessions.report.sessions;
        let resolver = scanner.open_resolver(&root)?;
        let report = project(&refresh, &sessions, &root, mode, &resolver, &mut cache)?;
        let mut runs = Vec::new();
        if args.notify {
            for repo in completion_repos(&sessions, now) {
                runs.extend(rows::done_runs(
                    &root,
                    repo,
                    &sessions,
                    &links.report.links,
                    &mut cache,
                ));
            }
        }
        Ok((serde_json::to_value(report)?, runs))
    };
    let mut out = io::stdout().lock();
    if args.watch {
        watch(&mut out, &mut runtime, parent, args.notify, &markers, pass)
    } else {
        super::json(&mut out, &pass(runtime.now()?)?.0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate_sessions() -> Vec<Session> {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sessions-phase14");
        let options = Options {
            state_dir: PathBuf::new(),
            now: Timestamp::parse("2026-11-10T22:20:00Z").unwrap(),
            since: None,
            repo_mode: RepoMode::Fixture,
            probe: &FixtureProcessProbe,
        };
        let template = Scanner::default()
            .sessions(&fixture, &options)
            .unwrap()
            .report
            .sessions
            .remove(0);
        [
            (
                "/old-finished",
                "2026-11-10T20:19:59.999Z",
                State::Finished,
                false,
            ),
            ("/old-busy", "2026-11-10T20:19:59.999Z", State::Busy, false),
            ("/boundary", "2026-11-10T20:20:00Z", State::Finished, false),
            ("/waiting", "2026-11-09T22:20:00Z", State::Waiting, false),
            ("/stale", "2026-11-09T22:20:00Z", State::Stale, false),
            ("/imported", "2026-11-10T22:19:00Z", State::Busy, true),
            ("/recent", "2026-11-10T22:19:00Z", State::Busy, false),
            ("/recent", "2026-11-09T22:20:00Z", State::Finished, false),
        ]
        .into_iter()
        .map(|(repo, at, state, imported)| {
            let mut session = template.clone();
            session.repo = repo.into();
            session.last_activity = Timestamp::parse(at).unwrap();
            session.state = state;
            session.imported = imported;
            session
        })
        .collect()
    }

    #[test]
    fn projection_never_builds_rows_for_old_finished_or_imported_only_repos() {
        let sessions = candidate_sessions();
        let mut calls = Vec::new();
        let report = project_with_rows(
            Timestamp::parse("2026-11-10T22:20:00Z").unwrap(),
            &sessions,
            |repo| {
                calls.push(repo.to_owned());
                Ok(json!({"sections": []}))
            },
        )
        .unwrap();
        assert_eq!(calls, ["/boundary", "/recent", "/stale", "/waiting"]);
        assert!(report.repos.is_empty());
    }

    #[test]
    fn completion_scans_only_repos_with_activity_in_the_window() {
        let sessions = candidate_sessions();
        assert_eq!(
            completion_repos(&sessions, Timestamp::parse("2026-11-10T22:20:00Z").unwrap())
                .into_iter()
                .collect::<Vec<_>>(),
            ["/boundary", "/imported", "/recent"]
        );
    }

    struct ScriptClock {
        tick: usize,
        end: usize,
    }
    impl WatchRuntime for ScriptClock {
        fn parent(&self) -> u32 {
            if self.tick < self.end { 42 } else { 99 }
        }
        fn elapsed(&self) -> Duration {
            Duration::from_secs(self.tick as u64 * 3)
        }
        fn now(&self) -> Result<Timestamp, WatchError> {
            Ok(Timestamp::from_epoch_ms(
                Timestamp::parse("2026-11-10T22:20:00Z")?.epoch_ms() + self.tick as i64 * 3000,
            )?)
        }
        fn wait(&mut self, duration: Duration) {
            assert_eq!(duration, Duration::from_secs(3));
            self.tick += 1;
        }
    }

    fn replay(markers: &impl MarkerStore) -> String {
        let script: Value = serde_json::from_str(include_str!(
            "../../../fixtures/expected/menubar/watch-script.json"
        ))
        .unwrap();
        let ticks = script["ticks"].as_array().unwrap();
        let mut runtime = ScriptClock {
            tick: 0,
            end: ticks.len(),
        };
        let mut index = 0;
        let mut output = Vec::new();
        let pass = |now| {
            let tick = &ticks[index];
            index += 1;
            assert_eq!(
                Timestamp::parse(tick["now"].as_str().unwrap()).unwrap(),
                now
            );
            if let Some(error) = tick["error"].as_str() {
                return Err(io::Error::other(error).into());
            }
            Ok((
                json!({"v":1,"now":now,"waiting":0,"running":false,"repos":[]}),
                serde_json::from_value(tick["runs"].clone()).unwrap(),
            ))
        };
        watch(&mut output, &mut runtime, 42, true, markers, pass).unwrap();
        String::from_utf8(output).unwrap()
    }

    #[test]
    fn menubar_watcher_replays_claudes_script_byte_for_byte() {
        let temp = tempfile::tempdir().unwrap();
        let markers = FileMarkers(temp.path().into());
        assert_eq!(
            replay(&markers),
            include_str!("../../../fixtures/expected/menubar/watch-codex-done.jsonl")
        );
        assert_eq!(
            std::fs::read_dir(temp.path().join("notified"))
                .unwrap()
                .count(),
            2
        );
    }
}
