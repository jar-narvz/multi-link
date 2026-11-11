use multi_link_core::{
    FixtureProcessProbe, LoadProgress, LoadSnapshot, Options, RepoMode, Scanner,
};
use multi_link_model::Timestamp;
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

fn options() -> Options<'static> {
    Options {
        state_dir: PathBuf::new(),
        now: Timestamp::parse("2026-10-29T23:15:00Z").unwrap(),
        since: None,
        repo_mode: RepoMode::Fixture,
        probe: &FixtureProcessProbe,
    }
}

fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let path = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &path);
        } else {
            fs::copy(entry.path(), path).unwrap();
        }
    }
}

fn write(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

#[test]
fn thread_counts_preserve_reports_diagnostics_and_stats() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    // The base fixture deliberately has no out-of-report read diagnostics.
    let baseline = Scanner::with_parse_threads(1)
        .sessions(&fixtures.join("sessions"), &options())
        .unwrap();
    assert!(baseline.bad_records.is_empty());
    assert!(baseline.incomplete.is_empty());
    assert!(baseline.unreadable.is_empty());

    let root = tempfile::tempdir().unwrap();
    copy_tree(&fixtures.join("sessions"), root.path());
    for hidden in [".multi-link", ".claude"] {
        copy_tree(
            &fixtures.join("hook").join(hidden),
            &root.path().join(hidden),
        );
    }
    fs::remove_file(root.path().join("task-output/be01y5dv6.output")).unwrap();
    write(root.path(), ".claude/sessions/bad.json", "not JSON\n");
    write(
        root.path(),
        ".claude/projects/test/incomplete.jsonl",
        "{\"type\":\"user\"}\n",
    );
    write(
        root.path(),
        ".codex/sessions/rollout-unreadable.jsonl.zst",
        "not a zstd stream",
    );
    let app_directory = "Library/Application Support/Claude/claude-code-sessions/a/b";
    for (name, text) in [
        ("local_good.json", r#"{"cliSessionId":"synthetic-session"}"#),
        ("local_ignored.json", r#"{"cliSessionId":null}"#),
        ("local_bad.json", "not JSON"),
    ] {
        write(root.path(), &format!("{app_directory}/{name}"), text);
    }
    let run = |threads| {
        let mut scanner = Scanner::with_parse_threads(threads);
        let result = scanner
            .overview(root.path(), &options(), &mut |folder| {
                multi_link_plan::steps_report(
                    Path::new(""),
                    &[folder.to_string_lossy().into_owned()],
                )
                .repos
                .remove(0)
            })
            .unwrap();
        let reports = (
            serde_json::to_vec(&result.report).unwrap(),
            serde_json::to_vec(&result.sessions.report).unwrap(),
            serde_json::to_vec(&result.links.report).unwrap(),
        );
        for output in [&result.sessions.bad_records, &result.links.bad_records] {
            assert_eq!(
                output,
                &[
                    ".claude/sessions/bad.json".to_owned(),
                    format!("{app_directory}/local_bad.json"),
                ]
            );
        }
        for output in [&result.sessions.incomplete, &result.links.incomplete] {
            assert_eq!(output, &[".claude/projects/test/incomplete.jsonl"]);
        }
        let unreadable = |failures: &[multi_link_codex::ReadFailure]| {
            assert_eq!(failures.len(), 1);
            assert_eq!(
                failures[0].file,
                ".codex/sessions/rollout-unreadable.jsonl.zst"
            );
            let errors: Vec<_> = failures
                .iter()
                .map(|failure| {
                    let text = failure.error.to_string();
                    assert!(!text.is_empty());
                    (failure.file.clone(), text)
                })
                .collect();
            errors
        };
        assert_eq!(result.links.hook_conflicts, 1);
        let diagnostics = serde_json::json!({
            "sessions": {
                "bad_records": result.sessions.bad_records,
                "incomplete": result.sessions.incomplete,
                "unreadable": unreadable(&result.sessions.unreadable),
            },
            "links": {
                "bad_records": result.links.bad_records,
                "incomplete": result.links.incomplete,
                "unreadable": unreadable(&result.links.unreadable),
                "hook_conflicts": result.links.hook_conflicts,
            },
            "scanner": &scanner.diagnostics,
        });
        (reports, diagnostics, scanner.stats)
    };
    assert_eq!(run(1), run(4));
}

#[test]
fn warm_app_record_refresh_parses_nothing() {
    let root = tempfile::tempdir().unwrap();
    let directory = "Library/Application Support/Claude/claude-code-sessions/a/b";
    for (name, text) in [
        ("local_good.json", r#"{"cliSessionId":"synthetic-session"}"#),
        ("local_ignored.json", r#"{"cliSessionId":null}"#),
        ("local_bad.json", "not JSON"),
    ] {
        write(root.path(), &format!("{directory}/{name}"), text);
    }
    let mut scanner = Scanner::with_parse_threads(4);
    let progress = LoadProgress::default();
    scanner.set_progress(progress.clone());
    let (first, first_links) = scanner.refresh(root.path(), &options()).unwrap();
    assert_eq!(scanner.stats.files, 3);
    assert_eq!(scanner.stats.bytes, 65);
    assert_eq!(progress.snapshot().files_total, 3);
    assert_eq!(progress.snapshot().files_done, 3);
    assert_eq!(first.bad_records, [format!("{directory}/local_bad.json")]);
    let (warm, warm_links) = scanner.refresh(root.path(), &options()).unwrap();
    assert_eq!(scanner.stats.files, 0);
    assert_eq!(scanner.stats.bytes, 0);
    assert_eq!(warm.bad_records, first.bad_records);
    assert_eq!(warm_links.bad_records, first_links.bad_records);
    assert_eq!(
        serde_json::to_vec(&warm.report).unwrap(),
        serde_json::to_vec(&first.report).unwrap()
    );
    assert_eq!(
        serde_json::to_vec(&warm_links.report).unwrap(),
        serde_json::to_vec(&first_links.report).unwrap()
    );
    assert_eq!(progress.snapshot().files_total, 3);
    assert_eq!(progress.snapshot().files_done, 3);
}

#[test]
fn malformed_claude_record_is_reported_once_with_identical_text() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        ".claude/projects/test/broken.jsonl",
        "{\"type\":\"user\",\"cwd\":\"/synthetic\",\"timestamp\":\"2026-10-29T23:15:00Z\"}\nnot JSON\n",
    );
    let run = |threads| {
        let mut scanner = Scanner::with_parse_threads(threads);
        let output = scanner.sessions(root.path(), &options()).unwrap();
        assert_eq!(output.report.sessions.len(), 1);
        assert_eq!(
            serde_json::to_value(&output.report.skipped).unwrap(),
            serde_json::json!([
                {"file": ".claude/projects/test/broken.jsonl", "line": 2, "reason": "invalid_json"}
            ])
        );
        assert!(output.bad_records.is_empty());
        assert!(output.incomplete.is_empty());
        assert!(output.unreadable.is_empty());
        serde_json::to_string(&output.report.skipped).unwrap()
    };
    assert_eq!(run(1), run(4));
}

#[test]
fn progress_survives_reset_and_all_time_and_warm_passes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sessions");
    let progress = LoadProgress::default();
    assert_eq!(progress.snapshot().phase, "listing");
    let mut scanner = Scanner::with_parse_threads(4);
    scanner.set_progress(progress.clone());
    let mut opts = options();
    opts.since = Some(Duration::from_secs(7200));
    scanner.refresh(&root, &opts).unwrap();
    let first: LoadSnapshot = progress.snapshot();
    assert_eq!(first.phase, "linking");
    assert!(first.files_total > 0);
    assert_eq!(first.files_done, first.files_total);
    scanner.refresh(&root, &options()).unwrap();
    assert_eq!(progress.snapshot(), first);
    assert_eq!(scanner.stats.files, 0);
    progress.finish();
    let done = progress.snapshot();
    assert_eq!(done.phase, "done");
    scanner.refresh(&root, &options()).unwrap();
    assert_eq!(progress.snapshot(), done);
}

#[test]
fn empty_load_goes_straight_to_linking() {
    let root = tempfile::tempdir().unwrap();
    let progress = LoadProgress::default();
    let mut scanner = Scanner::default();
    scanner.set_progress(progress.clone());
    scanner.refresh(root.path(), &options()).unwrap();
    let snapshot = progress.snapshot();
    assert_eq!(snapshot.phase, "linking");
    assert_eq!((snapshot.files_done, snapshot.files_total), (0, 0));
}

#[test]
fn observed_progress_is_monotonic_across_first_worker_load() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sessions");
    let progress = LoadProgress::default();
    let worker_progress = progress.clone();
    let worker = std::thread::spawn(move || {
        let mut scanner = Scanner::with_parse_threads(4);
        scanner.set_progress(worker_progress.clone());
        let mut window = options();
        window.since = Some(Duration::from_secs(7200));
        scanner.refresh(&root, &window).unwrap();
        scanner.refresh(&root, &options()).unwrap();
        worker_progress.finish();
        scanner.refresh(&root, &options()).unwrap();
    });
    let mut previous = progress.snapshot();
    while !worker.is_finished() {
        let next = progress.snapshot();
        let rank = |phase| match phase {
            "listing" => 0,
            "reading" => 1,
            "linking" => 2,
            "done" => 3,
            _ => panic!("unknown phase"),
        };
        assert!(rank(next.phase) >= rank(previous.phase));
        assert!(next.files_done >= previous.files_done);
        assert!(next.files_done <= next.files_total);
        if previous.files_total != 0 {
            assert_eq!(next.files_total, previous.files_total);
        }
        previous = next;
        std::thread::yield_now();
    }
    worker.join().unwrap();
    let last = progress.snapshot();
    assert_eq!(last.phase, "done");
    assert_eq!(last.files_done, last.files_total);
    assert!(last.files_total > 0);
}

#[test]
fn attaching_progress_to_warm_scanner_skips_reading() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sessions");
    let mut scanner = Scanner::default();
    scanner.refresh(&root, &options()).unwrap();
    let progress = LoadProgress::default();
    scanner.set_progress(progress.clone());
    scanner.refresh(&root, &options()).unwrap();
    assert_eq!(scanner.stats.files, 0);
    let snapshot = progress.snapshot();
    assert_eq!(snapshot.phase, "linking");
    assert_eq!((snapshot.files_done, snapshot.files_total), (0, 0));
}
