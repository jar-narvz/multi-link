//! The pane projects the worker's repo view, using its labels, placement and status words.
use std::{
    collections::{HashMap, HashSet, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    io::{self, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime},
};

use clap::Args;
use multi_link_core::{
    FixtureProcessProbe, Options, ProcessProbe, RealProcessProbe, RepoMode, Scanner, graph,
    open::{OpenAction, Resolver},
};
use multi_link_model::{
    GraphReport, Harness, Match, PartStatus, Session, SessionRow, State, Timestamp,
};
use multi_link_tui::{Refresh, view};
use serde::{Deserialize, Serialize};

#[derive(Args)]
pub struct PaneArgs {
    /// Print the pane object as pretty JSON.
    #[arg(long, required = true)]
    json: bool,
    /// Stream compact JSON changes and a checked tick every 3 seconds.
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
    strip: Option<Strip>,
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
    asked: Option<String>,
    said: Option<String>,
    openable: bool,
    opens: Option<&'static str>,
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

#[derive(Clone, Debug, Serialize)]
struct Strip {
    step: String,
    title: Option<String>,
    parts: Vec<StripPart>,
}
#[derive(Clone, Debug, Serialize)]
struct StripPart {
    harness: Harness,
    kind: String,
    status: Option<&'static str>,
    active_min: i64,
    is_this: bool,
}

/// Hash only graph invalidation inputs, without serializing the refresh. Sorting
/// record hashes makes scanner ordering irrelevant to the session/link sets.
/// Activity timestamps advance while work continues; the 30 s cap updates ages.
fn strip_fingerprint(sessions: &[Session], links: &[multi_link_model::Link]) -> u64 {
    let mut session_keys: Vec<_> = sessions
        .iter()
        .map(|session| {
            let mut hash = DefaultHasher::new();
            (&session.id, std::mem::discriminant(&session.state)).hash(&mut hash);
            hash.finish()
        })
        .collect();
    let mut link_keys: Vec<_> = links
        .iter()
        .map(|link| {
            let mut hash = DefaultHasher::new();
            (
                &link.step,
                &link.parent,
                &link.invocation,
                &link.subagent,
                std::mem::discriminant(&link.form),
                link.background,
                link.at,
                &link.checkout,
                &link.launch_dir,
            )
                .hash(&mut hash);
            (
                std::mem::discriminant(&link.r#match),
                &link.child,
                link.evidence.map(|value| std::mem::discriminant(&value)),
            )
                .hash(&mut hash);
            link.candidates.len().hash(&mut hash);
            for candidate in &link.candidates {
                (&candidate.id, candidate.gap_ms).hash(&mut hash);
            }
            hash.finish()
        })
        .collect();
    session_keys.sort_unstable();
    link_keys.sort_unstable();
    let mut hash = DefaultHasher::new();
    (session_keys, link_keys).hash(&mut hash);
    hash.finish()
}

struct CachedStrip {
    fingerprint: u64,
    rebuilt_at: Duration,
    strip: Strip,
}

#[derive(Default)]
struct StripCache {
    entry: Option<CachedStrip>,
}
impl StripCache {
    fn get(
        &mut self,
        step: Option<&str>,
        sessions: &[Session],
        links: &[multi_link_model::Link],
        elapsed: Duration,
        rebuild: impl FnOnce(&str) -> Result<Strip, WatchError>,
    ) -> Result<Option<Strip>, WatchError> {
        let Some(step) = step else {
            self.entry = None;
            return Ok(None);
        };
        let fingerprint = strip_fingerprint(sessions, links);
        if let Some(entry) = &self.entry
            && entry.strip.step == step
            && entry.fingerprint == fingerprint
            && elapsed.saturating_sub(entry.rebuilt_at) < Duration::from_secs(30)
        {
            return Ok(Some(entry.strip.clone()));
        }
        // Refresh even unchanged inputs after 30 s for staleness and active minutes.
        let strip = rebuild(step)?;
        self.entry = Some(CachedStrip {
            fingerprint,
            rebuilt_at: elapsed,
            strip: strip.clone(),
        });
        Ok(Some(strip))
    }
}

fn strip(report: &GraphReport, sessions: &[Session], this: &str) -> Strip {
    let live = graph::live_parts(report, sessions);
    let parts = report
        .parts
        .iter()
        .enumerate()
        .map(|(i, part)| {
            let latest = !report.parts.iter().enumerate().any(|(j, p)| {
                p.harness == part.harness
                    && p.session == part.session
                    && (p.from, j) > (part.from, i)
            });
            let status = match part.harness {
                Harness::Codex => match part.status {
                    Some(PartStatus::Done) => Some("done"),
                    Some(PartStatus::Stale) => Some("stale"),
                    Some(PartStatus::Working) if live.contains(&i) => Some("running"),
                    _ => None,
                },
                Harness::Claude if live.contains(&i) => Some("running"),
                Harness::Claude
                    if latest
                        && sessions.iter().any(|s| {
                            Some(s.id.as_str()) == part.session.as_deref()
                                && s.harness == Harness::Claude
                                && s.state == State::Waiting
                        }) =>
                {
                    Some("waiting")
                }
                Harness::Claude
                    if report
                        .parts
                        .iter()
                        .skip(i + 1)
                        .any(|p| p.chain == part.chain) =>
                {
                    Some("done")
                }
                Harness::Claude => None,
            };
            StripPart {
                harness: part.harness,
                kind: part.kind.clone(),
                status,
                active_min: (part.active_ms / 60_000).max(1),
                is_this: part.session.as_deref() == Some(this),
            }
        })
        .collect();
    Strip {
        step: report.step.id.clone(),
        title: report.step.title.clone(),
        parts,
    }
}

#[derive(Default)]
struct Details {
    said: Option<String>,
    opens: Option<&'static str>,
}

/// Read graph-style turn windows with original completion text. The graph's
/// reader normalizes whitespace and cuts text before the pane's H1/H4 cap.
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
    pane: &Pane,
    turn_cache: &mut TurnCache,
) -> HashMap<String, Details> {
    let visible: HashSet<&str> = pane
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
        strip: None,
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
            asked: row.asked.clone(),
            said: None,
            openable: false,
            opens: None,
            is_this: row.id == this,
            children,
            handoffs,
        });
    }
    pane
}

/// Shared row projection and enrichment for the pane and the menu bar.
pub(crate) fn rows(
    refresh: &Refresh,
    repo: &Path,
    mode: RepoMode,
    root: &Path,
    sessions: &[Session],
    resolver: &Resolver,
    cache: &mut TurnCache,
) -> Result<serde_json::Value, WatchError> {
    let mut pane = project(refresh, repo, mode, "");
    enrich(&mut pane, root, sessions, resolver, cache);
    Ok(serde_json::to_value(pane)?)
}

fn enrich(
    pane: &mut Pane,
    root: &Path,
    sessions: &[Session],
    resolver: &Resolver,
    turn_cache: &mut TurnCache,
) {
    let details = details(root, sessions, resolver, pane, turn_cache);
    for row in pane.sections.iter_mut().flat_map(|s| &mut s.rows) {
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

/// Probe the app's advisory lock immediately before the pane claims a marker.
/// Dropping the probe releases the lock before the atomic marker creation.
pub(crate) struct PaneMarkers {
    pub(crate) files: FileMarkers,
    pub(crate) lock: PathBuf,
}
impl MarkerStore for PaneMarkers {
    fn claim(&self, run: &str) -> io::Result<bool> {
        if let Some(parent) = self.lock.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&self.lock)?;
        match file.try_lock() {
            Ok(()) => drop(file),
            Err(std::fs::TryLockError::WouldBlock) => return Ok(false),
            Err(std::fs::TryLockError::Error(error)) => return Err(error),
        }
        self.files.claim(run)
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

/// Full-refresh links, before the pane's visibility, nesting and time window.
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
    let mut strip_cache = StripCache::default();
    let strip_clock = Instant::now();
    let mut turn_cache = TurnCache::default();
    let markers = PaneMarkers {
        files: FileMarkers(super::state_dir(&root)),
        lock: root.join(".multi-link/menubar.lock"),
    };
    let mut pass = |now| -> Result<(Pane, Vec<DoneRun>), WatchError> {
        let options = Options {
            state_dir: super::state_dir(&root),
            now,
            since: Some(Duration::from_secs(2 * 60 * 60)),
            repo_mode,
            probe,
        };
        let refresh = super::tui::refresh_with_live(&mut scanner, &root, &options, None, false)?;
        let mut pane = project(&refresh, &args.cwd, repo_mode, &args.session);
        let all = Options {
            since: None,
            state_dir: options.state_dir.clone(),
            ..options
        };
        let (all_sessions, all_links) = scanner.refresh(&root, &all)?;
        let sessions = all_sessions.report.sessions;
        let runs = if args.notify {
            done_runs(
                &root,
                &pane.repo,
                &sessions,
                &all_links.report.links,
                &mut turn_cache,
            )
        } else {
            Vec::new()
        };
        let resolver = scanner.open_resolver(&root)?;
        enrich(&mut pane, &root, &sessions, &resolver, &mut turn_cache);
        let step = pane
            .sections
            .iter()
            .flat_map(|s| &s.rows)
            .find(|r| r.is_this)
            .and_then(|r| r.step.as_deref());
        pane.strip = strip_cache.get(
            step,
            &sessions,
            &all_links.report.links,
            strip_clock.elapsed(),
            |step| {
                let mut plans = |folder: &Path| {
                    multi_link_plan::steps_report(
                        Path::new(""),
                        &[folder.to_string_lossy().into_owned()],
                    )
                    .repos
                    .remove(0)
                };
                let report = scanner.graph(&root, &options, &pane.repo, step, &mut plans)?;
                Ok(strip(&report, &sessions, &args.session))
            },
        )?;
        Ok((pane, runs))
    };
    let mut out = io::stdout().lock();
    if args.watch {
        // If setup lost the original parent, do not adopt the new one.
        if runtime.parent() != parent {
            return Ok(());
        }
        watch_passes(
            &mut out,
            &mut runtime,
            parent,
            args.notify,
            &markers,
            |now| {
                let (pane, runs) = pass(now)?;
                Ok((serde_json::to_value(pane)?, runs))
            },
        )
    } else {
        super::json(&mut out, &pass(runtime.now()?)?.0)?;
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

    #[test]
    fn strip_statuses_use_current_parts_and_preserve_graph_order() {
        let mut report: GraphReport = serde_json::from_str(include_str!(
            "../../../fixtures/expected/phase12/graph-1.1.json"
        ))
        .unwrap();
        let mut session: Session = serde_json::from_value(
            serde_json::from_str::<serde_json::Value>(include_str!(
                "../../../fixtures/expected/sessions.json"
            ))
            .unwrap()["sessions"][0]
                .clone(),
        )
        .unwrap();
        session.id = OWNER.into();
        session.harness = Harness::Claude;
        session.state = State::Busy;
        let mut old = report.parts[0].clone();
        old.session = Some(OWNER.into());
        old.from = Some(Timestamp::from_epoch_ms(report.now.epoch_ms() - 60_000).unwrap());
        old.to = old.from;
        old.active_ms = 59_999;
        let mut done = old.clone();
        done.harness = Harness::Codex;
        done.session = Some("done".into());
        done.kind = "implement".into();
        done.status = Some(PartStatus::Done);
        let mut current = old.clone();
        current.kind = "review".into();
        current.from = Some(report.now);
        current.to = Some(report.now);
        current.active_ms = 120_999;
        let mut stale = done.clone();
        stale.chain = 1;
        stale.status = Some(PartStatus::Stale);
        let mut unknown = done.clone();
        unknown.chain = 2;
        unknown.status = Some(PartStatus::Unavailable);
        let mut tail = current.clone();
        tail.chain = 3;
        tail.session = Some("other".into());
        report.parts = vec![old, done, current, stale, unknown, tail];
        let actual = strip(&report, &[session.clone()], OWNER);
        assert_eq!(
            actual.parts.iter().map(|p| p.status).collect::<Vec<_>>(),
            [
                Some("done"),
                Some("done"),
                Some("running"),
                Some("stale"),
                None,
                None
            ]
        );
        assert_eq!(
            actual
                .parts
                .iter()
                .map(|p| p.kind.as_str())
                .collect::<Vec<_>>(),
            [
                "plan",
                "implement",
                "review",
                "implement",
                "implement",
                "review"
            ]
        );
        assert_eq!(
            actual.parts.iter().map(|p| p.is_this).collect::<Vec<_>>(),
            [true, false, true, false, false, false]
        );
        assert_eq!(actual.parts[0].active_min, 1);
        assert_eq!(actual.parts[2].active_min, 2);
        session.state = State::Waiting;
        let waiting = strip(&report, &[session], OWNER);
        assert_eq!(waiting.parts[0].status, Some("done"));
        assert_eq!(waiting.parts[2].status, Some("waiting"));
        let mut working = report.parts[1].clone();
        working.session = Some("working".into());
        working.status = Some(PartStatus::Working);
        working.run_available = Some(true);
        working.chain = 4;
        let mut resumed = working.clone();
        resumed.from = Some(report.now);
        resumed.to = Some(report.now);
        report.parts.extend([working, resumed]);
        let codex = strip(&report, &[], OWNER);
        assert_eq!(codex.parts[6].status, None);
        assert_eq!(codex.parts[7].status, Some("running"));
    }

    #[test]
    fn strip_cache_rebuilds_only_on_input_changes_or_thirty_seconds() {
        use std::cell::Cell;
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sessions-phase14");
        let options = Options {
            state_dir: root.join(".multi-link"),
            now: Timestamp::parse("2026-11-10T22:20:00Z").unwrap(),
            since: None,
            repo_mode: RepoMode::Fixture,
            probe: &FixtureProcessProbe,
        };
        let (all_sessions, all_links) = Scanner::default().refresh(&root, &options).unwrap();
        let mut sessions = all_sessions.report.sessions;
        let mut links = all_links.report.links;
        let builds = Cell::new(0);
        let build = |step: &str| -> Result<Strip, WatchError> {
            builds.set(builds.get() + 1);
            Ok(Strip {
                step: step.into(),
                title: Some(format!("build {}", builds.get())),
                parts: vec![],
            })
        };
        let mut cache = StripCache::default();
        let mut check =
            |step, sessions: &[Session], links: &[multi_link_model::Link], elapsed, expected| {
                let result = cache
                    .get(step, sessions, links, Duration::from_secs(elapsed), build)
                    .unwrap();
                assert_eq!(builds.get(), expected);
                assert_eq!(
                    result.as_ref().map(|s| s.title.as_deref().unwrap()),
                    step.map(|_| format!("build {expected}")).as_deref()
                );
            };
        check(Some("17.2"), &sessions, &links, 0, 1);
        check(Some("17.2"), &sessions, &links, 29, 1);
        check(Some("17.2"), &sessions, &links, 30, 2);
        check(Some("17.2"), &sessions, &links, 31, 2);
        sessions.reverse();
        links.reverse();
        sessions[0].asked = Some("unrelated prompt metadata".into());
        check(Some("17.2"), &sessions, &links, 31, 2);
        sessions[0].state = if sessions[0].state == State::Busy {
            State::Waiting
        } else {
            State::Busy
        };
        check(Some("17.2"), &sessions, &links, 32, 3);
        links.push(links[0].clone());
        links.last_mut().unwrap().invocation = "new-invocation".into();
        check(Some("17.2"), &sessions, &links, 33, 4);
        check(Some("17.3"), &sessions, &links, 34, 5);
        sessions[0].last_activity = Timestamp::parse("2026-11-10T22:20:01Z").unwrap();
        check(Some("17.3"), &sessions, &links, 35, 5);
        sessions[0].id = "new-session".into();
        check(Some("17.3"), &sessions, &links, 36, 6);
        links[0].step = Some("other-step".into());
        check(Some("17.3"), &sessions, &links, 37, 7);
        sessions[0].last_activity = Timestamp::parse("2026-11-10T22:20:02Z").unwrap();
        check(Some("17.3"), &sessions, &links, 66, 7);
        check(Some("17.3"), &sessions, &links, 67, 8);
        check(None, &sessions, &links, 68, 8);
        check(Some("17.3"), &sessions, &links, 69, 9);
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
        // Immediate cold pass and actual change at 3 s; all other ticks carry a tick line.
        assert_eq!(
            lines
                .iter()
                .filter_map(|v| v["now"].as_str())
                .collect::<Vec<_>>(),
            ["2026-11-10T22:20:00.000Z", "2026-11-10T22:20:03.000Z"]
        );
        assert_eq!(
            lines[0]["sections"][0]["rows"][0]["children"][0]["state"],
            "busy"
        );
        assert_eq!(
            lines[1]["sections"][0]["rows"][0]["children"][0]["state"],
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
            "../../../fixtures/expected/pane/watch-script.json"
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
            let mut pane = script["pane"].clone();
            pane["now"] = json!(now);
            Ok((pane, serde_json::from_value(tick["runs"].clone()).unwrap()))
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
            include_str!("../../../fixtures/expected/pane/watch-tick.jsonl")
        );
        assert!(!temp.path().join("notified").exists());
        assert_eq!(
            replay(true, &store),
            include_str!("../../../fixtures/expected/pane/watch-codex-done.jsonl")
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
            include_str!("../../../fixtures/expected/pane/watch-tick.jsonl")
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
