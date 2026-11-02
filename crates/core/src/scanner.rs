//! Stateful, read-only refreshes. Only parsed data is cached; state and windows
//! are recomputed on every pass.
use std::{
    collections::{HashMap, HashSet},
    fs, io,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    time::SystemTime,
};

use multi_link_claude::{ClaudeScan, ParsedFile, SessionRecord, SubagentFile};
use multi_link_codex::{Discovery, ReadFailure, Rollout};
use multi_link_model::{HookSummary, OverviewReport, RepoSteps, Skipped, Timestamp};

use crate::{LinksOutput, Liveness, Options, ProcessProbe, RepoMode, SessionsOutput};

/// Reuse this refresh's batch results during session assembly.
struct BatchProcessProbe {
    results: HashMap<(u32, String), Liveness>,
}

impl ProcessProbe for BatchProcessProbe {
    fn probe(&self, pid: u32, proc_start: &str) -> Liveness {
        self.results
            .get(&(pid, proc_start.to_owned()))
            .copied()
            .unwrap_or(Liveness::Unavailable)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ParseStats {
    pub files: usize,
    pub bytes: u64,
}

/// Assumes fine-grained mtime (APFS: nanoseconds). On a 1 s mtime filesystem a
/// same-length, same-inode rewrite within one second would be missed.
#[derive(PartialEq, Eq)]
struct Stamp {
    len: u64,
    modified: SystemTime,
    inode: u64,
}

struct Cache<T>(HashMap<PathBuf, (Stamp, T)>);

impl<T> Default for Cache<T> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<T: Clone> Cache<T> {
    fn read(
        &mut self,
        path: &Path,
        stats: &mut ParseStats,
        parse: impl FnOnce() -> io::Result<T>,
    ) -> io::Result<T> {
        let meta = match fs::metadata(path) {
            Ok(meta) => meta,
            Err(error) => {
                self.0.remove(path);
                return Err(error);
            }
        };
        let stamp = Stamp {
            len: meta.len(),
            modified: meta.modified()?,
            inode: meta.ino(),
        };
        if let Some((old, value)) = self.0.get(path)
            && old == &stamp
        {
            return Ok(value.clone());
        }
        // Invalidate first: a failed parse must never leave a reusable old value.
        self.0.remove(path);
        stats.files += 1;
        stats.bytes += stamp.len;
        let value = parse()?;
        self.0.insert(path.to_owned(), (stamp, value.clone()));
        Ok(value)
    }

    fn prune(&mut self) {
        self.0.retain(|path, _| path.exists());
    }
}

/// One refresh's full result: the overview plus the sessions and links it was
/// built from (their diagnostics and reports, for the CLI and the timeline).
#[derive(Debug)]
pub struct OverviewOutput {
    pub report: OverviewReport,
    pub sessions: SessionsOutput,
    pub links: LinksOutput,
}

/// Own one scanner per refresh worker. Changing roots clears all caches.
/// Statistics describe the most recent sessions/links call. Diagnostics are
/// separate from the stable JSON reports (including hook-directory failures).
#[derive(Default)]
pub struct Scanner {
    root: Option<PathBuf>,
    claude: Cache<ParsedFile>,
    records: Cache<Option<SessionRecord>>,
    codex: Cache<(Option<Rollout>, Vec<Skipped>)>,
    hooks: Cache<crate::hook::Records>,
    hook_summary: HookSummary,
    outputs: Cache<String>,
    repos: HashMap<String, crate::RepoPaths>,
    pub stats: ParseStats,
    pub diagnostics: Vec<String>,
}

type ClaudeInputs = (
    ClaudeScan,
    Vec<multi_link_claude::Invocation>,
    HashMap<String, Option<Timestamp>>,
);

struct Inputs {
    claude: ClaudeScan,
    codex: Discovery,
    hooks: crate::hook::Records,
    waiting: HashSet<String>,
    invocations: Vec<multi_link_claude::Invocation>,
}

impl Scanner {
    fn read(&mut self, root: &Path, options: &Options<'_>) -> io::Result<Inputs> {
        if self.root.as_deref() != Some(root) {
            *self = Self {
                root: Some(root.to_owned()),
                ..Default::default()
            };
        }
        self.stats = ParseStats::default();
        self.diagnostics.clear();
        self.claude.prune();
        self.records.prune();
        self.codex.prune();
        self.hooks.prune();
        self.outputs.prune();
        let (claude, invocations, latest) = self.read_claude(root)?;
        let mut codex = self.read_codex(root)?;
        let hooks = self.read_hooks(root, options, &mut codex);
        self.hook_summary = hooks.summary();
        let waiting = claude
            .transcripts
            .iter()
            .filter_map(|transcript| {
                let timestamp = latest.get(&transcript.transcript).copied().flatten();
                let hook = hooks
                    .records
                    .iter()
                    .filter(|r| r.session_id == transcript.id)
                    .max_by_key(|r| (r.at, r.line))?;
                (hook.waiting && timestamp.is_none_or(|at| at < hook.at))
                    .then(|| transcript.transcript.clone())
            })
            .collect();
        Ok(Inputs {
            claude,
            codex,
            hooks,
            waiting,
            invocations,
        })
    }

    fn read_claude(&mut self, root: &Path) -> io::Result<ClaudeInputs> {
        let mut claude = ClaudeScan::default();
        let mut invocations = Vec::new();
        let mut latest: HashMap<String, Option<Timestamp>> = HashMap::new();
        for project in children(&root.join(".claude/projects"))? {
            if !project.is_dir() {
                continue;
            }
            for path in children(&project)? {
                if !path.is_file() || !extension(&path, "jsonl") {
                    continue;
                }
                let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };
                let parsed = match self.claude.read(&path, &mut self.stats, || {
                    multi_link_claude::parse_file(root, &path, false)
                }) {
                    Ok(parsed) => parsed,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                    Err(error) => return Err(error),
                };
                let mut transcript = parsed.transcript;
                let mut timestamp = parsed.latest_timestamp;
                claude.skipped.extend(parsed.skipped);
                invocations.extend(parsed.invocations);
                for subpath in children(&project.join(id).join("subagents"))? {
                    if !subpath.is_file() || !extension(&subpath, "jsonl") {
                        continue;
                    }
                    let Some(agent) = subpath
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .and_then(|s| s.strip_prefix("agent-"))
                        .filter(|s| !s.is_empty())
                    else {
                        continue;
                    };
                    let parsed = match self.claude.read(&subpath, &mut self.stats, || {
                        multi_link_claude::parse_file(root, &subpath, true)
                    }) {
                        Ok(parsed) => parsed,
                        Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                        Err(error) => return Err(error),
                    };
                    transcript
                        .activity_timestamps
                        .extend(parsed.transcript.activity_timestamps);
                    transcript.subagents.push(agent.to_owned());
                    transcript.subagent_files.push(SubagentFile {
                        id: agent.to_owned(),
                        transcript: parsed.transcript.transcript,
                    });
                    timestamp = timestamp.max(parsed.latest_timestamp);
                    claude.skipped.extend(parsed.skipped);
                    invocations.extend(parsed.invocations);
                }
                transcript.subagents.sort();
                latest.insert(transcript.transcript.clone(), timestamp);
                claude.transcripts.push(transcript);
            }
        }
        for path in children(&root.join(".claude/sessions"))? {
            if !extension(&path, "json") {
                continue;
            }
            match self.records.read(&path, &mut self.stats, || {
                Ok(multi_link_claude::read_session_record(root, &path))
            }) {
                Ok(Some(record)) => claude.records.push(record),
                _ => claude.bad_records.push(relative(root, &path)),
            }
        }
        claude
            .transcripts
            .sort_by(|a, b| a.transcript.cmp(&b.transcript));
        claude.records.sort_by(|a, b| a.record.cmp(&b.record));
        claude.bad_records.sort();
        invocations.sort_by(|a, b| {
            (a.at, &a.parent, &a.invocation).cmp(&(b.at, &b.parent, &b.invocation))
        });
        Ok((claude, invocations, latest))
    }

    fn read_codex(&mut self, root: &Path) -> io::Result<Discovery> {
        let mut paths = Vec::new();
        rollout_paths(&root.join(".codex/sessions"), &mut paths)?;
        paths.sort();
        let mut codex = Discovery::default();
        for path in paths {
            // Preserve partial skips even when a rollout read fails.
            let mut skipped = Vec::new();
            let parsed = self.codex.read(&path, &mut self.stats, || {
                let rollout = multi_link_codex::parse_file(root, &path, &mut skipped)?;
                Ok((rollout, skipped.clone()))
            });
            match parsed {
                Ok((rollout, skips)) => {
                    codex.rollouts.extend(rollout);
                    codex.skipped.extend(skips);
                }
                Err(error) => {
                    codex.skipped.extend(skipped);
                    codex.unreadable.push(ReadFailure {
                        file: relative(root, &path),
                        error,
                    });
                }
            }
        }
        Ok(codex)
    }

    fn read_hooks(
        &mut self,
        root: &Path,
        options: &Options<'_>,
        codex: &mut Discovery,
    ) -> crate::hook::Records {
        let state = match options.repo_mode {
            RepoMode::Fixture => root.join(".multi-link"),
            RepoMode::Real => options.state_dir.clone(),
        };
        let mut hooks = crate::hook::Records::default();
        match children(&state.join("hook")) {
            Ok(paths) => {
                for path in paths {
                    // Like the old reader: regular files only, symlinks skipped.
                    if !fs::symlink_metadata(&path).is_ok_and(|m| m.is_file())
                        || !extension(&path, "jsonl")
                        || !path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .is_some_and(crate::hook::uuid)
                    {
                        continue;
                    }
                    match self.hooks.read(&path, &mut self.stats, || {
                        crate::hook::parse_file(root, &path)
                    }) {
                        Ok(parsed) => {
                            hooks.records.extend(parsed.records);
                            hooks.skipped.extend(parsed.skipped);
                        }
                        Err(error) => {
                            self.diagnostics.push(format!("hook read: {error}"));
                            codex.unreadable.push(ReadFailure {
                                file: relative(root, &path),
                                error,
                            });
                        }
                    }
                }
            }
            Err(error) => {
                self.diagnostics.push(format!("hook directory: {error}"));
                codex.unreadable.push(ReadFailure {
                    file: relative(root, &state.join("hook")),
                    error,
                });
            }
        }
        hooks
    }

    fn assemble(&mut self, options: &Options<'_>, inputs: Inputs) -> io::Result<SessionsOutput> {
        let requests: Vec<_> = inputs
            .claude
            .records
            .iter()
            .map(|r| (r.pid, r.proc_start.as_str()))
            .collect();
        let results = options.probe.batch(&requests);
        let probe = BatchProcessProbe {
            results: requests
                .iter()
                .zip(results)
                .map(|((pid, start), live)| ((*pid, (*start).to_owned()), live))
                .collect(),
        };
        let batched = Options {
            state_dir: options.state_dir.clone(),
            now: options.now,
            since: options.since,
            repo_mode: options.repo_mode,
            probe: &probe,
        };
        let mut output = crate::sessions::assemble(
            &batched,
            inputs.claude,
            inputs.codex,
            inputs.hooks,
            &inputs.waiting,
        )?;
        for session in &mut output.report.sessions {
            // Fixture mode keeps repo = checkout = cwd (set by assembly).
            if options.repo_mode == RepoMode::Real {
                let paths = self
                    .repos
                    .entry(session.cwd.clone())
                    .or_insert_with(|| options.repo_mode.map(Path::new(&session.cwd)));
                session.repo = paths.repo.to_string_lossy().into_owned();
                session.checkout = paths.checkout.to_string_lossy().into_owned();
            }
        }
        Ok(output)
    }

    pub fn sessions(&mut self, root: &Path, options: &Options<'_>) -> io::Result<SessionsOutput> {
        let inputs = self.read(root, options)?;
        self.assemble(options, inputs)
    }

    pub fn links(&mut self, root: &Path, options: &Options<'_>) -> io::Result<LinksOutput> {
        self.refresh(root, options).map(|(_, links)| links)
    }

    /// One full refresh: one discovery pass and one batch liveness probe.
    pub fn refresh(
        &mut self,
        root: &Path,
        options: &Options<'_>,
    ) -> io::Result<(SessionsOutput, LinksOutput)> {
        let inputs = self.read(root, options)?;
        let (links, conflicts) = crate::link::resolve(
            root,
            options.repo_mode,
            &inputs.invocations,
            &inputs.codex.rollouts,
            &inputs.hooks,
            &mut |path| {
                self.outputs
                    .read(path, &mut self.stats, || fs::read_to_string(path))
                    .ok()
            },
        )?;
        let sessions = self.assemble(options, inputs)?;
        let links = crate::links::assemble(&sessions, links, conflicts)?;
        Ok((sessions, links))
    }

    /// One `refresh` plus the overview built from it. `plans` reads a plan
    /// folder with the `steps` rules (the CLI passes the plan crate's parser).
    pub fn overview(
        &mut self,
        root: &Path,
        options: &Options<'_>,
        plans: &mut dyn FnMut(&Path) -> RepoSteps,
    ) -> io::Result<OverviewOutput> {
        let (sessions, links) = self.refresh(root, options)?;
        let mut report = crate::overview::build(
            root,
            options.repo_mode,
            &sessions.report,
            &links.report,
            plans,
        );
        report.hook = self.hook_summary.clone();
        Ok(OverviewOutput {
            report,
            sessions,
            links,
        })
    }
}

fn extension(path: &Path, ext: &str) -> bool {
    path.extension().is_some_and(|s| s == ext)
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

fn children(path: &Path) -> io::Result<Vec<PathBuf>> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut paths = entries
        .map(|e| e.map(|e| e.path()))
        .collect::<io::Result<Vec<_>>>()?;
    paths.sort();
    Ok(paths)
}

fn rollout_paths(path: &Path, paths: &mut Vec<PathBuf>) -> io::Result<()> {
    for child in children(path)? {
        // A rollout moved or archived after read_dir must not fail the pass.
        let kind = match fs::symlink_metadata(&child) {
            Ok(meta) => meta.file_type(),
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        if kind.is_dir() {
            rollout_paths(&child, paths)?;
        } else if kind.is_file()
            && child
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.starts_with("rollout-") && s.ends_with(".jsonl"))
        {
            paths.push(child);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FixtureProcessProbe, Liveness, ProcessProbe};
    use std::{cell::Cell, io::Write, time::Duration};

    fn options(probe: &dyn ProcessProbe) -> Options<'_> {
        Options {
            state_dir: PathBuf::new(),
            now: Timestamp::parse("2026-10-29T23:15:00Z").unwrap(),
            since: None,
            repo_mode: RepoMode::Fixture,
            probe,
        }
    }

    fn copy_tree(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), target).unwrap();
            }
        }
    }

    #[test]
    fn cache_detects_append_truncate_mtime_replacement_and_deletion() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        fs::write(&path, "one").unwrap();
        let mut cache = Cache::<String>::default();
        let mut stats = ParseStats::default();
        let mut read = |stats: &mut ParseStats| {
            cache
                .read(&path, stats, || fs::read_to_string(&path))
                .unwrap()
        };
        assert_eq!(read(&mut stats), "one");
        assert_eq!(read(&mut stats), "one");
        assert_eq!(stats, ParseStats { files: 1, bytes: 3 });
        fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(b"two")
            .unwrap();
        assert_eq!(read(&mut stats), "onetwo");
        assert_eq!(stats, ParseStats { files: 2, bytes: 9 });
        fs::write(&path, "x").unwrap();
        assert_eq!(read(&mut stats), "x");
        assert_eq!(
            stats,
            ParseStats {
                files: 3,
                bytes: 10
            }
        );
        fs::write(&path, "y").unwrap();
        fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(42))
            .unwrap();
        assert_eq!(read(&mut stats), "y");
        assert_eq!(
            stats,
            ParseStats {
                files: 4,
                bytes: 11
            }
        );
        // Equal length AND mtime: only the new inode can invalidate this entry.
        let replacement = temp.path().join("replacement");
        fs::write(&replacement, "z").unwrap();
        fs::File::options()
            .write(true)
            .open(&replacement)
            .unwrap()
            .set_modified(fs::metadata(&path).unwrap().modified().unwrap())
            .unwrap();
        assert_ne!(
            fs::metadata(&path).unwrap().ino(),
            fs::metadata(&replacement).unwrap().ino()
        );
        fs::rename(replacement, &path).unwrap();
        assert_eq!(read(&mut stats), "z");
        assert_eq!(
            stats,
            ParseStats {
                files: 5,
                bytes: 12
            }
        );
        fs::remove_file(&path).unwrap();
        assert_eq!(
            cache
                .read(&path, &mut stats, || unreachable!())
                .unwrap_err()
                .kind(),
            io::ErrorKind::NotFound
        );
        assert!(cache.0.is_empty());
    }

    #[test]
    fn warm_refresh_caches_every_file_kind_and_recomputes_state() {
        struct Batch(Cell<usize>);
        impl ProcessProbe for Batch {
            fn probe(&self, _: u32, _: &str) -> Liveness {
                panic!("must batch");
            }
            fn batch(&self, requests: &[(u32, &str)]) -> Vec<Liveness> {
                self.0.set(self.0.get() + 1);
                vec![Liveness::Unavailable; requests.len()]
            }
        }
        let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
        let root = tempfile::tempdir().unwrap();
        copy_tree(&fixtures.join("sessions"), root.path());
        copy_tree(&fixtures.join("hook"), root.path());
        fs::remove_file(root.path().join("task-output/be01y5dv6.output")).unwrap();
        let probe = Batch(Cell::new(0));
        let mut scanner = Scanner::default();
        let options = options(&probe);
        for pass in 1..=2 {
            let (sessions, links) = scanner.refresh(root.path(), &options).unwrap();
            for (name, actual) in [
                ("sessions", serde_json::to_value(&sessions.report).unwrap()),
                ("links", serde_json::to_value(&links.report).unwrap()),
            ] {
                let expected: serde_json::Value = serde_json::from_slice(
                    &fs::read(fixtures.join(format!("expected/hook/{name}.json"))).unwrap(),
                )
                .unwrap();
                assert_eq!(actual, expected);
            }
            assert_eq!(probe.0.get(), pass);
            if pass == 1 {
                assert!(scanner.stats.files > 0);
                assert!(!scanner.claude.0.is_empty());
                assert!(!scanner.records.0.is_empty());
                assert!(!scanner.codex.0.is_empty());
                assert!(!scanner.hooks.0.is_empty());
            } else {
                assert_eq!(scanner.stats, ParseStats::default());
            }
        }
        // A single changed transcript invalidates only its entry, including
        // unknown timestamps used for waiting, without re-parsing the others.
        let path = scanner.claude.0.keys().min().unwrap().clone();
        fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(b"{\"type\":\"future\",\"timestamp\":\"2026-10-30T23:15:00Z\"}\n")
            .unwrap();
        scanner.refresh(root.path(), &options).unwrap();
        assert_eq!(scanner.stats.files, 1);
        assert_eq!(probe.0.get(), 3);

        let mut later = options;
        later.now = Timestamp::parse("2026-10-30T23:15:00Z").unwrap();
        later.since = Some(Duration::from_secs(1));
        let (sessions, _) = scanner.refresh(root.path(), &later).unwrap();
        assert_eq!(scanner.stats.files, 0);
        assert!(sessions.report.sessions.len() < 27);
        assert_eq!(probe.0.get(), 4);
    }

    #[test]
    fn task_output_is_cached_but_changes_and_removal_update_links() {
        let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sessions");
        let root = tempfile::tempdir().unwrap();
        copy_tree(&fixtures, root.path());
        let mut scanner = Scanner::default();
        let options = options(&FixtureProcessProbe);
        let (_, first) = scanner.refresh(root.path(), &options).unwrap();
        assert_eq!(scanner.outputs.0.len(), 1);
        let path = root.path().join("task-output/be01y5dv6.output");
        scanner.refresh(root.path(), &options).unwrap();
        assert_eq!(scanner.stats.files, 0);
        fs::write(&path, "session id: aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa\n").unwrap();
        let (_, changed) = scanner.refresh(root.path(), &options).unwrap();
        assert_eq!(scanner.stats.files, 1);
        let find_bg = |report: LinksOutput| {
            report
                .report
                .links
                .into_iter()
                .find(|l| l.parent.starts_with("07c80c2e"))
                .unwrap()
        };
        assert_eq!(
            find_bg(changed).child.as_deref(),
            Some("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa")
        );
        fs::remove_file(&path).unwrap();
        let (_, removed) = scanner.refresh(root.path(), &options).unwrap();
        assert!(scanner.outputs.0.is_empty());
        assert_ne!(find_bg(first).evidence, find_bg(removed).evidence);
    }

    #[test]
    fn hook_directory_errors_are_diagnostics_and_can_recover() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join(".multi-link")).unwrap();
        let hooks = root.path().join(".multi-link/hook");
        fs::write(&hooks, "not a directory").unwrap();
        let mut scanner = Scanner::default();
        let options = options(&FixtureProcessProbe);
        let (sessions, links) = scanner.refresh(root.path(), &options).unwrap();
        assert!(sessions.report.sessions.is_empty());
        assert!(links.report.links.is_empty());
        assert_eq!(sessions.unreadable.len(), 1);
        assert_eq!(sessions.unreadable[0].file, ".multi-link/hook");
        assert_eq!(links.unreadable.len(), 1);
        assert_eq!(scanner.diagnostics.len(), 1);
        assert!(scanner.diagnostics[0].starts_with("hook directory: "));
        fs::remove_file(&hooks).unwrap();
        fs::create_dir(&hooks).unwrap();
        scanner.refresh(root.path(), &options).unwrap();
        assert!(scanner.diagnostics.is_empty());
    }

    #[test]
    fn symlinked_hook_files_are_skipped() {
        let root = tempfile::tempdir().unwrap();
        let hooks = root.path().join(".multi-link/hook");
        fs::create_dir_all(&hooks).unwrap();
        let target = root.path().join("elsewhere.jsonl");
        fs::write(&target, "not json\n").unwrap();
        std::os::unix::fs::symlink(
            &target,
            hooks.join("01a18ba2-f020-7000-8000-000000000000.jsonl"),
        )
        .unwrap();
        let mut scanner = Scanner::default();
        scanner
            .refresh(root.path(), &options(&FixtureProcessProbe))
            .unwrap();
        assert!(scanner.hooks.0.is_empty());
        assert_eq!(scanner.stats.files, 0);
    }

    #[test]
    fn git_mapping_only_sees_listed_cwds_and_is_memoised() {
        let root = tempfile::tempdir().unwrap();
        let project = root.path().join(".claude/projects/test");
        fs::create_dir_all(&project).unwrap();
        for (id, cwd, at) in [
            ("old", "/synthetic/old", "2026-10-28T00:00:00Z"),
            ("new", "/synthetic/new", "2026-10-29T23:15:00Z"),
        ] {
            fs::write(
                project.join(format!("{id}.jsonl")),
                format!("{{\"type\":\"user\",\"cwd\":\"{cwd}\",\"timestamp\":\"{at}\"}}\n"),
            )
            .unwrap();
        }
        let mut options = options(&FixtureProcessProbe);
        options.repo_mode = RepoMode::Real;
        options.since = Some(Duration::from_secs(1));
        let mut scanner = Scanner::default();
        scanner.sessions(root.path(), &options).unwrap();
        assert_eq!(scanner.repos.len(), 1);
        assert!(scanner.repos.contains_key("/synthetic/new"));
        // A memoised result is reused even if git cannot reproduce it.
        scanner.repos.get_mut("/synthetic/new").unwrap().repo = PathBuf::from("/memoised");
        let sessions = scanner.sessions(root.path(), &options).unwrap();
        assert_eq!(sessions.report.sessions[0].repo, "/memoised");
        assert_eq!(scanner.stats.files, 0);
    }
}
