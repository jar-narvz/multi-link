//! Shared session rows, completion tracking and the menubar watch loop.
use std::{
    collections::{HashMap, HashSet},
    io::{self, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime},
};

use multi_link_core::{
    RepoMode, graph,
    open::{OpenAction, Resolver},
};
use multi_link_model::{Harness, Match, Session, SessionRow, State, Timestamp};
use multi_link_tui::{Refresh, view};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct Rows {
    sections: Vec<Section>,
}
#[derive(Debug, Serialize)]
struct Section {
    name: &'static str,
    rows: Vec<Row>,
}
#[derive(Debug, Serialize)]
struct Row {
    id: String,
    harness: Harness,
    label: String,
    step: Option<String>,
    state: State,
    phrase: String,
    asked: Option<String>,
    said: Option<String>,
    openable: bool,
    opens: Option<&'static str>,
    children: Vec<Child>,
    handoffs: Vec<Handoff>,
}
#[derive(Debug, Serialize)]
struct Child {
    id: String,
    harness: Harness,
    state: State,
    phrase: String,
    asked: Option<String>,
    said: Option<String>,
    openable: bool,
    opens: Option<&'static str>,
    r#match: &'static str,
}
#[derive(Debug, Serialize)]
struct Handoff {
    step: Option<String>,
    outcome: String,
    candidates: Vec<String>,
}

#[derive(Default)]
struct Details {
    said: Option<String>,
    opens: Option<&'static str>,
}

/// Read graph-style turn windows with original completion text. The graph's
/// reader normalizes whitespace and cuts text before the row/event preview cap.
fn raw_turns(path: &Path) -> io::Result<Vec<multi_link_model::Turn>> {
    let reader = multi_link_codex::open_rollout(path)?;
    let mut active: Option<(String, usize)> = None;
    let mut turns: Vec<multi_link_model::Turn> = Vec::new();
    for record in multi_link_model::jsonl::JsonlReader::new(reader, "rollout") {
        let Ok((_, value)) = record? else { continue };
        if value["type"] != "event_msg" {
            continue;
        }
        let payload = &value["payload"];
        let at = value["timestamp"]
            .as_str()
            .and_then(|s| Timestamp::parse(s).ok());
        if payload["type"] == "task_started" {
            active = None;
            if let (Some(id), Some(at)) = (payload["turn_id"].as_str(), at)
                && !id.starts_with("external-import")
            {
                active = Some((id.to_owned(), turns.len()));
                turns.push(multi_link_model::Turn {
                    start: at,
                    completed_at: None,
                    last_record_at: at,
                    prompt: None,
                    last_agent_message: None,
                });
            }
        } else if let Some((id, index)) = &active {
            let turn = &mut turns[*index];
            if let Some(at) = at {
                turn.last_record_at = turn.last_record_at.max(at);
            }
            if payload["type"] == "task_complete" && payload["turn_id"] == id.as_str() {
                turn.completed_at = at;
                turn.last_agent_message = payload["last_agent_message"].as_str().map(str::to_owned);
                active = None;
            }
        }
    }
    Ok(turns)
}

fn details(
    root: &Path,
    sessions: &[Session],
    resolver: &Resolver,
    rows: &Rows,
    turn_cache: &mut TurnCache,
) -> HashMap<String, Details> {
    let visible: HashSet<&str> = rows
        .sections
        .iter()
        .flat_map(|s| &s.rows)
        .flat_map(|r| {
            std::iter::once(r.id.as_str()).chain(r.children.iter().map(|c| c.id.as_str()))
        })
        .collect();
    sessions
        .iter()
        .filter(|s| visible.contains(s.id.as_str()))
        .map(|session| {
            let said = if session.harness == Harness::Codex {
                session
                    .transcript
                    .as_deref()
                    .and_then(|path| turn_cache.said(&root.join(path)))
                    .map(|text| {
                        let mut chars = text.chars();
                        let mut preview: String = chars.by_ref().take(120).collect();
                        if chars.next().is_some() {
                            preview.push('…');
                        }
                        preview
                    })
            } else {
                None
            };
            (
                session.id.clone(),
                Details {
                    said,
                    opens: match resolver.action(session, false) {
                        OpenAction::AppLink { .. } => Some("app"),
                        OpenAction::Jump { .. } => Some("terminal"),
                        _ => None,
                    },
                },
            )
        })
        .collect()
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

fn project(refresh: &Refresh, repo: &Path, mode: RepoMode) -> Rows {
    let repo = mode.map(repo).repo.to_string_lossy().into_owned();
    let now = refresh.overview.now;
    let offset = view::local_offset(now);
    let mut rows = Rows {
        sections: ["working", "waiting", "recent"]
            .map(|name| Section {
                name,
                rows: Vec::new(),
            })
            .into(),
    };
    let Some(group) = refresh.overview.repos.iter().find(|g| g.repo == repo) else {
        return rows;
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
    let mut top_level: Vec<_> = sessions
        .iter()
        .copied()
        .filter(|r| visible(r) && !nested(r))
        .collect();
    top_level.sort_by(newest_first);
    for row in top_level {
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
                asked: child.asked.clone(),
                said: None,
                openable: false,
                opens: None,
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
        let section = match row.state {
            State::Busy => 0,
            State::Waiting | State::Stale => 1,
            _ => 2,
        };
        rows.sections[section].rows.push(Row {
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
            asked: row.asked.clone(),
            said: None,
            openable: false,
            opens: None,
            children,
            handoffs,
        });
    }
    rows
}

/// Project and enrich the visible rows of one repository.
pub(crate) fn rows(
    refresh: &Refresh,
    repo: &Path,
    mode: RepoMode,
    root: &Path,
    sessions: &[Session],
    resolver: &Resolver,
    cache: &mut TurnCache,
) -> Result<serde_json::Value, WatchError> {
    let mut rows = project(refresh, repo, mode);
    enrich(&mut rows, root, sessions, resolver, cache);
    Ok(serde_json::to_value(rows)?)
}

fn enrich(
    rows: &mut Rows,
    root: &Path,
    sessions: &[Session],
    resolver: &Resolver,
    turn_cache: &mut TurnCache,
) {
    let details = details(root, sessions, resolver, rows, turn_cache);
    for row in rows.sections.iter_mut().flat_map(|s| &mut s.rows) {
        if let Some(detail) = details.get(&row.id) {
            row.said = detail.said.clone();
            row.openable = detail.opens.is_some();
            row.opens = detail.opens;
        }
        for child in &mut row.children {
            if let Some(detail) = details.get(&child.id) {
                child.said = detail.said.clone();
                child.openable = detail.opens.is_some();
                child.opens = detail.opens;
            }
        }
    }
}

pub(crate) type WatchError = Box<dyn std::error::Error>;
const TICK: Duration = Duration::from_secs(3);

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct DoneRun {
    run: String,
    id: String,
    step: Option<String>,
    said: Option<String>,
    done: bool,
}

pub(crate) trait MarkerStore {
    fn claim(&self, run: &str) -> io::Result<bool>;
}

pub(crate) struct FileMarkers(pub(crate) PathBuf);
impl MarkerStore for FileMarkers {
    fn claim(&self, run: &str) -> io::Result<bool> {
        // Invocation ids are filenames, never paths supplied by a transcript.
        if run.is_empty() || run == "." || run == ".." || run.contains(['/', '\\']) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid invocation id",
            ));
        }
        let directory = self.0.join("notified");
        std::fs::create_dir_all(&directory)?;
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join(run))
        {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(false),
            Err(error) => Err(error),
        }
    }
}

#[derive(Default)]
pub(crate) struct TurnCache {
    entries: HashMap<PathBuf, (u64, SystemTime, Vec<multi_link_model::Turn>)>,
}
impl TurnCache {
    fn said(&mut self, path: &Path) -> Option<String> {
        graph::said(
            self.get(path)?
                .iter()
                .rev()
                .find(|turn| turn.completed_at.is_some()),
        )
    }

    fn get(&mut self, path: &Path) -> Option<&[multi_link_model::Turn]> {
        self.get_with(path, raw_turns)
    }

    fn get_with(
        &mut self,
        path: &Path,
        read: impl FnOnce(&Path) -> io::Result<Vec<multi_link_model::Turn>>,
    ) -> Option<&[multi_link_model::Turn]> {
        let stamp = std::fs::metadata(path).and_then(|m| Ok((m.len(), m.modified()?)));
        let Ok((len, modified)) = stamp else {
            self.entries.remove(path);
            return None;
        };
        if !self
            .entries
            .get(path)
            .is_some_and(|e| (e.0, e.1) == (len, modified))
        {
            self.entries.remove(path);
            let turns = read(path).ok()?;
            self.entries.insert(path.to_owned(), (len, modified, turns));
        }
        self.entries.get(path).map(|entry| entry.2.as_slice())
    }
}

/// Full-refresh links, before row visibility, nesting and time window.
pub(crate) fn done_runs(
    root: &Path,
    repo: &str,
    sessions: &[Session],
    links: &[multi_link_model::Link],
    cache: &mut TurnCache,
) -> Vec<DoneRun> {
    let order =
        |link: &multi_link_model::Link| (link.at, link.invocation.clone(), link.parent.clone());
    let mut owned: Vec<_> = links
        .iter()
        .filter(|link| {
            matches!(link.r#match, Match::Exact | Match::Heuristic)
                && link.child.is_some()
                && sessions
                    .iter()
                    .any(|s| s.id == link.parent && s.harness == Harness::Claude && s.repo == repo)
        })
        .collect();
    owned.sort_by_key(|link| order(link));
    owned
        .iter()
        .filter_map(|link| {
            let child = link.child.as_deref()?;
            let path = sessions
                .iter()
                .find(|s| s.id == child && s.harness == Harness::Codex)
                .and_then(|s| s.transcript.as_deref());
            let next = owned
                .iter()
                .filter(|other| other.child.as_deref() == Some(child) && order(other) > order(link))
                .map(|other| other.at)
                .min();
            // The graph's G4 window is half-open, selected by turn start, and only
            // task_complete (not session state or silence) supplies completion.
            let last = path
                .and_then(|path| cache.get(&root.join(path)))
                .into_iter()
                .flatten()
                .filter(|turn| turn.start >= link.at && next.is_none_or(|end| turn.start < end))
                .max_by_key(|turn| turn.start);
            Some(DoneRun {
                run: link.invocation.clone(),
                id: child.into(),
                step: link.step.clone(),
                said: graph::said(last),
                done: last.is_some_and(|turn| turn.completed_at.is_some()),
            })
        })
        .collect()
}

pub(crate) trait WatchRuntime {
    fn parent(&self) -> u32;
    fn elapsed(&self) -> Duration;
    fn now(&self) -> Result<Timestamp, WatchError>;
    fn wait(&mut self, duration: Duration);
}

pub(crate) struct Runtime {
    pub(crate) start: Instant,
    pub(crate) fixed_now: Option<Timestamp>,
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

#[cfg(test)]
fn watch(
    out: &mut impl Write,
    runtime: &mut impl WatchRuntime,
    parent: u32,
    mut pass: impl FnMut(Timestamp) -> Result<serde_json::Value, WatchError>,
) -> Result<(), WatchError> {
    watch_passes(
        out,
        runtime,
        parent,
        false,
        &FileMarkers(PathBuf::new()),
        |now| Ok((pass(now)?, Vec::new())),
    )
}

fn write_line(out: &mut impl Write, value: &impl Serialize) -> Result<(), WatchError> {
    serde_json::to_writer(&mut *out, value)?;
    out.write_all(b"\n")?;
    out.flush()?;
    Ok(())
}

pub(crate) fn watch_passes(
    out: &mut impl Write,
    runtime: &mut impl WatchRuntime,
    parent: u32,
    notify: bool,
    markers: &impl MarkerStore,
    mut pass: impl FnMut(Timestamp) -> Result<(serde_json::Value, Vec<DoneRun>), WatchError>,
) -> Result<(), WatchError> {
    let mut previous = None;
    let mut known = HashMap::new();
    while parent != 1 && runtime.parent() == parent {
        let tick_start = runtime.elapsed();
        let result = runtime
            .now()
            .and_then(|now| pass(now).map(|value| (now, value)));
        if runtime.parent() != parent {
            break;
        }
        match result {
            Ok((now, (value, runs))) => {
                let mut claim_error = None;
                if notify {
                    for run in runs {
                        let before = known.insert((run.run.clone(), run.id.clone()), run.done);
                        if before != Some(false) || !run.done {
                            continue;
                        }
                        match markers.claim(&run.run) {
                            Ok(true) => {
                                #[derive(Serialize)]
                                struct Event<'a> {
                                    kind: &'static str,
                                    run: &'a str,
                                    id: &'a str,
                                    step: &'a Option<String>,
                                    said: Option<String>,
                                }
                                #[derive(Serialize)]
                                struct Line<'a> {
                                    v: u8,
                                    event: Event<'a>,
                                }
                                write_line(
                                    out,
                                    &Line {
                                        v: 1,
                                        event: Event {
                                            kind: "codex_done",
                                            run: &run.run,
                                            id: &run.id,
                                            step: &run.step,
                                            said: run
                                                .said
                                                .map(|text| text.chars().take(120).collect()),
                                        },
                                    },
                                )?;
                            }
                            Ok(false) => {}
                            Err(error) => {
                                claim_error = Some(error);
                            }
                        }
                    }
                }
                if let Some(error) = claim_error {
                    write_line(out, &serde_json::json!({"v":1,"error":error.to_string()}))?;
                    previous = None;
                } else {
                    let mut key = value.clone();
                    without_times(&mut key);
                    if previous.as_ref() == Some(&key) {
                        write_line(out, &serde_json::json!({"v":1,"tick":now}))?;
                    } else {
                        write_line(out, &value)?;
                    }
                    previous = Some(key);
                }
            }
            Err(error) => {
                write_line(out, &serde_json::json!({"v":1,"error":error.to_string()}))?;
                previous = None;
            }
        }
        runtime.wait(TICK.saturating_sub(runtime.elapsed() - tick_start));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use multi_link_core::{FixtureProcessProbe, Options, Scanner};
    use multi_link_model::Row;
    use std::fs;

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

    fn projected(refresh: &Refresh) -> Rows {
        project(refresh, Path::new(REPO), RepoMode::Fixture)
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
        let rows = projected(&refresh);
        assert_eq!(rows.sections[0].rows.len(), 1);
        assert_eq!(rows.sections[0].rows[0].id, OWNER);
        assert!(rows.sections[0].rows[0].children.is_empty());
        assert_eq!(rows.sections[1].rows.len(), 1);
        assert_eq!(rows.sections[1].rows[0].id, "started-claude-child");
        assert!(rows.sections[2].rows.is_empty());
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
        let rows = projected(&refresh);
        let row = &rows.sections[0].rows[0];
        assert!(row.children.is_empty());
        assert_eq!(row.handoffs.len(), 1);
        assert_eq!(row.handoffs[0].step.as_deref(), Some("16.1"));
        assert_eq!(row.handoffs[0].outcome, "no codex run found");
        assert!(row.handoffs[0].candidates.is_empty());
    }

    #[test]
    fn flat_newest_first_then_id_and_no_imports() {
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
        let rows = projected(&refresh);
        assert_eq!(
            rows.sections[0]
                .rows
                .iter()
                .map(|r| r.id.as_str())
                .collect::<Vec<_>>(),
            ["a-tied", "b-newer", OWNER]
        );
    }

    #[test]
    fn hidden_parent_promotes_codex_but_preserves_placing_step_and_label() {
        let mut refresh = fixture();
        let mut parent = sample(&refresh);
        let child_id = parent.children[0].id.clone();
        parent.imported = true;
        set_rows(&mut refresh, vec![parent]);
        let rows = projected(&refresh);
        let rows: Vec<_> = rows.sections.iter().flat_map(|s| &s.rows).collect();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, child_id);
        assert_eq!(rows[0].step.as_deref(), Some("2.3"));
        assert_eq!(rows[0].label, "step 2.3");
        assert!(rows[0].children.is_empty());
    }

    #[test]
    fn rows_labels_and_phrases_equal_public_repo_view_functions() {
        let refresh = fixture();
        for group in &refresh.overview.repos {
            let rows = project(&refresh, Path::new(&group.repo), RepoMode::Fixture);
            let expected: Vec<_> = view::sessions(group).filter(|r| visible(r)).collect();
            let actual_count: usize = rows
                .sections
                .iter()
                .flat_map(|s| &s.rows)
                .map(|r| 1 + r.children.len())
                .sum();
            assert_eq!(actual_count, expected.len(), "{}", group.repo);
            for row in rows.sections.iter().flat_map(|s| &s.rows) {
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

    #[test]
    fn turn_cache_reads_original_said_from_compressed_rollouts() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../fixtures/sessions-phase9/.codex/sessions/2026/11/09/rollout-2026-11-09T09-00-45-91c00001-0000-4000-8000-000091c00001.jsonl.zst"
        );
        assert_eq!(TurnCache::default().said(&path).as_deref(), Some("<text>"));
    }

    #[test]
    fn turn_cache_shares_parses_and_reads_growth_and_new_mtime() {
        use std::cell::Cell;
        use std::fs::{File, FileTimes, OpenOptions};

        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("rollout-synthetic.jsonl");
        fs::write(&path, "first").unwrap();
        let reads = Cell::new(0);
        let at = Timestamp::parse("2026-11-10T22:20:00Z").unwrap();
        let read = |path: &Path| {
            reads.set(reads.get() + 1);
            Ok(vec![multi_link_model::Turn {
                start: at,
                completed_at: Some(at),
                last_record_at: at,
                prompt: None,
                last_agent_message: Some(fs::read_to_string(path)?),
            }])
        };
        let mut cache = TurnCache::default();
        assert_eq!(
            cache.get_with(&path, read).unwrap()[0]
                .last_agent_message
                .as_deref(),
            Some("first")
        );
        assert_eq!(reads.get(), 1);
        assert_eq!(cache.said(&path).as_deref(), Some("first"));
        assert_eq!(
            graph::said(cache.get(&path).unwrap().last()).as_deref(),
            Some("first")
        );
        assert_eq!(
            cache.get_with(&path, read).unwrap()[0]
                .last_agent_message
                .as_deref(),
            Some("first")
        );
        assert_eq!(reads.get(), 1);

        OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(b" second")
            .unwrap();
        assert_eq!(
            cache.get_with(&path, read).unwrap()[0]
                .last_agent_message
                .as_deref(),
            Some("first second")
        );
        assert_eq!(reads.get(), 2);
        let modified = fs::metadata(&path).unwrap().modified().unwrap();
        fs::write(&path, "third fourth").unwrap();
        File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_times(FileTimes::new().set_modified(modified + Duration::from_secs(1)))
            .unwrap();
        assert_eq!(
            cache.get_with(&path, read).unwrap()[0]
                .last_agent_message
                .as_deref(),
            Some("third fourth")
        );
        assert_eq!(reads.get(), 3);
        assert_eq!(
            cache.get_with(&path, read).unwrap()[0]
                .last_agent_message
                .as_deref(),
            Some("third fourth")
        );
        assert_eq!(reads.get(), 3);
    }
}

#[cfg(test)]
mod watch_tests {
    use super::*;
    use multi_link_core::{FixtureProcessProbe, Options, Scanner};
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
            "../../../fixtures/expected/menubar/multi-repo.json"
        ))
        .unwrap()
    }
    #[test]
    fn watch_first_change_ticks_and_all_phrases_are_filtered_and_flushed() {
        let mut output = Output::default();
        let mut runtime = FakeRuntime {
            tick: 0,
            end: 13,
            initial_parent: 42,
        };
        let mut passes = 0;
        let flushed = output.flushed.clone();
        watch(&mut output, &mut runtime, 42, |now| {
            assert_eq!(flushed.get(), passes);
            let mut value = object();
            value["now"] = json!(now);
            for repo in value["repos"].as_array_mut().unwrap() {
                for row in repo["rows"].as_array_mut().unwrap() {
                    row["phrase"] = json!(format!("row {passes}"));
                    for child in row["children"].as_array_mut().unwrap() {
                        child["phrase"] = json!(format!("child {passes}"));
                    }
                }
            }
            if passes >= 1 {
                value["repos"][1]["rows"][1]["children"][0]["state"] = json!("waiting");
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
        // Immediate cold pass and actual change at 3 s; all other ticks carry a tick line.
        assert_eq!(
            lines
                .iter()
                .filter_map(|v| v["now"].as_str())
                .collect::<Vec<_>>(),
            ["2026-11-10T22:20:00.000Z", "2026-11-10T22:20:03.000Z"]
        );
        assert_eq!(
            lines[0]["repos"][1]["rows"][1]["children"][0]["state"],
            "busy"
        );
        assert_eq!(
            lines[1]["repos"][1]["rows"][1]["children"][0]["state"],
            "waiting"
        );
        assert_eq!(lines.len(), 13);
        assert_eq!(lines[2], json!({"v":1,"tick":"2026-11-10T22:20:06.000Z"}));
        assert_eq!(lines[12], json!({"v":1,"tick":"2026-11-10T22:20:36.000Z"}));
        for line in &lines {
            assert_eq!(line["v"], 1);
        }
        assert_eq!(output.flushes, 13);
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
    fn replay(notify: bool, store: &impl MarkerStore) -> String {
        let script: Value = serde_json::from_str(include_str!(
            "../../../fixtures/expected/menubar/watch-script.json"
        ))
        .unwrap();
        let ticks = script["ticks"].as_array().unwrap();
        let mut runtime = FakeRuntime {
            tick: 0,
            end: ticks.len() as u64,
            initial_parent: 42,
        };
        let mut output = Output::default();
        let mut index = 0;
        watch_passes(&mut output, &mut runtime, 42, notify, store, |now| {
            let tick = &ticks[index];
            index += 1;
            assert_eq!(
                Timestamp::parse(tick["now"].as_str().unwrap()).unwrap(),
                now
            );
            if let Some(error) = tick["error"].as_str() {
                return Err(io::Error::other(error).into());
            }
            let report = json!({"v":1,"now":now,"waiting":0,"running":false,"repos":[]});
            Ok((
                report,
                serde_json::from_value(tick["runs"].clone()).unwrap(),
            ))
        })
        .unwrap();
        String::from_utf8(output.bytes).unwrap()
    }

    #[test]
    fn watch_scripts_match_byte_for_byte_with_and_without_notify() {
        let temp = tempfile::tempdir().unwrap();
        let store = FileMarkers(temp.path().to_owned());
        assert_eq!(
            replay(false, &store),
            include_str!("../../../fixtures/expected/menubar/watch-tick.jsonl")
        );
        assert!(!temp.path().join("notified").exists());
        assert_eq!(
            replay(true, &store),
            include_str!("../../../fixtures/expected/menubar/watch-codex-done.jsonl")
        );
        assert_eq!(
            std::fs::read_dir(temp.path().join("notified"))
                .unwrap()
                .count(),
            2
        );
        // Permanent markers suppress a second watcher even after its own baseline.
        assert_eq!(
            replay(true, &store),
            include_str!("../../../fixtures/expected/menubar/watch-tick.jsonl")
        );
    }

    fn transition(store: &impl MarkerStore, out: &mut impl Write) -> Result<(), WatchError> {
        let mut runtime = FakeRuntime {
            tick: 0,
            end: 7,
            initial_parent: 42,
        };
        let mut tick = 0;
        watch_passes(out, &mut runtime, 42, true, store, |_| {
            let current = tick;
            tick += 1;
            if current == 2 {
                return Err(io::Error::other("failed scan").into());
            }
            let runs = if current == 1 || current == 3 {
                vec![]
            } else {
                vec![DoneRun {
                    run: "invocation".into(),
                    id: "child".into(),
                    step: None,
                    said: Some("é".repeat(121)),
                    done: current >= 4,
                }]
            };
            Ok((object(), runs))
        })
    }

    #[test]
    fn disappeared_run_and_failed_scan_keep_state_and_body_is_capped() {
        let temp = tempfile::tempdir().unwrap();
        let mut output = Output::default();
        transition(&FileMarkers(temp.path().into()), &mut output).unwrap();
        let lines: Vec<Value> = std::str::from_utf8(&output.bytes)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let events: Vec<_> = lines.iter().filter(|v| v.get("event").is_some()).collect();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0]["event"],
            json!({"kind":"codex_done","run":"invocation","id":"child","step":null,"said":"é".repeat(120)})
        );
    }

    #[test]
    fn simultaneous_watchers_only_one_claims() {
        struct RacingStore {
            files: FileMarkers,
            barrier: std::sync::Arc<std::sync::Barrier>,
        }
        impl MarkerStore for RacingStore {
            fn claim(&self, run: &str) -> io::Result<bool> {
                self.barrier.wait();
                self.files.claim(run)
            }
        }
        let temp = tempfile::tempdir().unwrap();
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let handles: Vec<_> = (0..2)
            .map(|_| {
                let store = RacingStore {
                    files: FileMarkers(temp.path().into()),
                    barrier: barrier.clone(),
                };
                std::thread::spawn(move || {
                    let mut output = Output::default();
                    transition(&store, &mut output).unwrap();
                    String::from_utf8(output.bytes)
                        .unwrap()
                        .lines()
                        .filter(|line| line.contains("codex_done"))
                        .count()
                })
            })
            .collect();
        assert_eq!(
            handles
                .into_iter()
                .map(|h| h.join().unwrap())
                .sum::<usize>(),
            1
        );
    }

    #[test]
    fn broken_pipe_after_claim_leaves_tombstone() {
        struct BrokenEvent;
        impl Write for BrokenEvent {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                // serde writes the field name separately; fail inside the event.
                if bytes == b"codex_done" {
                    return Err(io::ErrorKind::BrokenPipe.into());
                }
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let temp = tempfile::tempdir().unwrap();
        let store = FileMarkers(temp.path().into());
        assert!(transition(&store, &mut BrokenEvent).is_err());
        assert!(temp.path().join("notified/invocation").is_file());
        let mut retry = Output::default();
        transition(&store, &mut retry).unwrap();
        assert!(
            !String::from_utf8(retry.bytes)
                .unwrap()
                .contains("codex_done")
        );
    }

    #[test]
    fn full_runs_include_hidden_old_imports_and_use_each_handoff_window() {
        let temp = tempfile::tempdir().unwrap();
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sessions-phase14");
        let now = Timestamp::parse("2026-11-10T22:20:00Z").unwrap();
        let options = Options {
            state_dir: temp.path().into(),
            now,
            since: None,
            repo_mode: RepoMode::Fixture,
            probe: &FixtureProcessProbe,
        };
        let source = Scanner::default()
            .sessions(&fixture, &options)
            .unwrap()
            .report
            .sessions;
        let mut parent = source
            .iter()
            .find(|s| s.harness == Harness::Claude)
            .unwrap()
            .clone();
        parent.id = "parent".into();
        parent.repo = "/repo".into();
        parent.imported = true;
        let mut child = source
            .iter()
            .find(|s| s.harness == Harness::Codex)
            .unwrap()
            .clone();
        child.id = "child".into();
        child.imported = true;
        child.transcript = Some("missing.jsonl".into());
        let sessions = vec![parent, child];
        let make_link = |run, at, matching| {
            serde_json::from_value::<multi_link_model::Link>(json!({
                "step":null,"parent":"parent","invocation":run,"subagent":null,"form":"exec",
                "background":false,"at":at,"checkout":"/repo","match":matching,
                "child":"child","evidence":null,"candidates":[]
            }))
            .unwrap()
        };
        let links = vec![
            make_link("first", "2026-11-09T10:00:00Z", "exact"),
            make_link("second", "2026-11-09T11:00:00Z", "heuristic"),
            make_link("unowned", "2026-11-09T10:30:00Z", "ambiguous"),
        ];
        let mut cache = TurnCache::default();
        let runs = done_runs(temp.path(), "/repo", &sessions, &links, &mut cache);
        assert_eq!(
            runs.iter()
                .map(|r| (r.run.as_str(), r.done))
                .collect::<Vec<_>>(),
            [("first", false), ("second", false)]
        );
        let records = [
            json!({"type":"event_msg","timestamp":"2026-11-09T10:00:00Z","payload":{"type":"task_started","turn_id":"a"}}),
            json!({"type":"event_msg","timestamp":"2026-11-09T10:01:00Z","payload":{"type":"task_complete","turn_id":"a","last_agent_message":"first answer"}}),
            // This later open turn belongs to the first handoff, so it is not done.
            json!({"type":"event_msg","timestamp":"2026-11-09T10:40:00Z","payload":{"type":"task_started","turn_id":"b"}}),
            json!({"type":"event_msg","timestamp":"2026-11-09T11:00:00Z","payload":{"type":"task_started","turn_id":"c"}}),
            json!({"type":"event_msg","timestamp":"2026-11-09T11:01:00Z","payload":{"type":"task_complete","turn_id":"c","last_agent_message":"second\nanswer"}}),
            json!({"type":"event_msg","timestamp":"2026-11-09T11:02:00Z","payload":{"type":"task_started","turn_id":"external-import-x"}}),
        ];
        std::fs::write(
            temp.path().join("missing.jsonl"),
            records.iter().map(|r| format!("{r}\n")).collect::<String>(),
        )
        .unwrap();
        let runs = done_runs(temp.path(), "/repo", &sessions, &links, &mut cache);
        assert_eq!(
            runs.iter()
                .map(|r| (r.run.as_str(), r.done, r.said.as_deref()))
                .collect::<Vec<_>>(),
            [
                ("first", false, None),
                ("second", true, Some("second\nanswer"))
            ]
        );
        assert!(done_runs(temp.path(), "/other", &sessions, &links, &mut cache).is_empty());
        // Changed rollout invalidates the turn cache, including when the child resumes.
        use std::io::Write;
        writeln!(std::fs::OpenOptions::new().append(true).open(temp.path().join("missing.jsonl")).unwrap(), "{}", json!({"type":"event_msg","timestamp":"2026-11-09T11:03:00Z","payload":{"type":"task_started","turn_id":"d"}})).unwrap();
        let runs = done_runs(temp.path(), "/repo", &sessions, &links, &mut cache);
        assert!(!runs[1].done);
        assert_eq!(runs[1].said, None);
    }
}
