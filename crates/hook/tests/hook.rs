use multi_link_hook::{Clock, HookEnv, HookRecord, record, record_and_write, write};
use multi_link_model::Timestamp;
use serde_json::{Value, json};
use std::{
    cell::Cell,
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tempfile::TempDir;

const PARENT: &str = "11111111-1111-4111-8111-111111111111";
const CHILD: &str = "abcdefab-1234-4321-8123-abcdef123456";
fn at(ms: u64) -> Timestamp {
    Timestamp::from_epoch_ms(1793315559000 + ms as i64).unwrap()
}
fn env(home: &Path) -> HookEnv {
    HookEnv {
        home: home.into(),
        multi_link_home: None,
        now: at(0),
    }
}
fn payload() -> Value {
    json!({"session_id":PARENT,"cwd":"/synthetic/repo","hook_event_name":"PostToolUse","tool_name":"Bash","tool_use_id":"tool-1","tool_input":{"command":"codex exec 'synthetic'"},"tool_response":{"stdout":format!("session id: {}\n", CHILD.to_uppercase())}})
}
fn bytes(p: &Value) -> Vec<u8> {
    serde_json::to_vec(p).unwrap()
}
fn lines(home: &Path) -> Vec<Value> {
    fs::read_to_string(home.join(format!(".multi-link/hook/{PARENT}.jsonl")))
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect()
}

#[test]
fn foreground_shapes_and_exact_schema() {
    let e = env(Path::new("/synthetic/home"));
    let mut p = payload();
    let expected = json!({"v":1,"kind":"handoff","at":"2026-10-29T23:12:39.000Z","session_id":PARENT,"agent_id":null,"event":"PostToolUse","tool_use_id":"tool-1","cwd":"/synthetic/repo","child":CHILD,"child_evidence":"stdout","notification_type":null});
    assert_eq!(
        serde_json::to_value(record(&bytes(&p), &e).unwrap()).unwrap(),
        expected
    );
    p["tool_input"]["command"] = json!(format!("codex exec resume {CHILD}"));
    assert_eq!(
        record(&bytes(&p), &e).unwrap().child.as_deref(),
        Some(CHILD)
    );
    p["tool_response"]["stdout"] = json!("");
    assert_eq!(record(&bytes(&p), &e).unwrap().child, None);
    p["tool_input"]["command"] = json!("node /synthetic/codex-companion.mjs task 'synthetic'");
    p["agent_id"] = json!("synthetic-agent");
    p["tool_response"]["stdout"] =
        json!(format!("[codex] Thread ready ({})", CHILD.to_uppercase()));
    let r = record(&bytes(&p), &e).unwrap();
    assert_eq!(
        (
            r.child.as_deref(),
            r.child_evidence.as_deref(),
            r.agent_id.as_deref()
        ),
        (Some(CHILD), Some("thread_ready"), Some("synthetic-agent"))
    );
    p["hook_event_name"] = json!("PostToolUseFailure");
    p.as_object_mut().unwrap().remove("tool_response");
    p["error"] = json!(format!("synthetic failure\nsession id: {CHILD}"));
    p["is_interrupt"] = json!(false);
    p["duration_ms"] = json!(1);
    let r = record(&bytes(&p), &e).unwrap();
    assert_eq!(
        (
            r.event.as_str(),
            r.child.as_deref(),
            r.child_evidence.as_deref()
        ),
        ("PostToolUseFailure", Some(CHILD), Some("stdout"))
    );
}

#[test]
fn waiting_shapes() {
    let e = env(Path::new("/synthetic/home"));
    for event in ["PermissionRequest", "Notification"] {
        for kind in ["permission_prompt", "elicitation_dialog"] {
            let p = json!({"session_id":PARENT,"cwd":"/synthetic/repo","hook_event_name":event,"tool_name":"Read","notification_type":kind,"tool_use_id":"ignored"});
            let r = record(&bytes(&p), &e).unwrap();
            assert_eq!(r.kind, "waiting");
            assert_eq!(r.tool_use_id, None);
            assert_eq!(r.child, None);
            assert_eq!(r.child_evidence, None);
            assert_eq!(
                r.notification_type.as_deref(),
                if event == "Notification" {
                    Some(kind)
                } else {
                    None
                }
            );
        }
    }
}

#[test]
fn ignored_and_malformed_inputs_write_nothing() {
    let tmp = TempDir::new().unwrap();
    let e = env(tmp.path());
    let mut cases = vec![
        vec![],
        b"{".to_vec(),
        vec![b'x'; 10 * 1024 * 1024],
        b"null".to_vec(),
    ];
    for (key, value) in [
        ("session_id", "../x"),
        ("session_id", "ABCDEFAB-1234-4321-8123-ABCDEF123456"),
        ("hook_event_name", "Stop"),
        ("tool_name", "Read"),
    ] {
        let mut p = payload();
        p[key] = json!(value);
        cases.push(bytes(&p));
    }
    for command in ["echo synthetic", "codex --version"] {
        let mut p = payload();
        p["tool_input"]["command"] = json!(command);
        cases.push(bytes(&p));
    }
    for kind in ["idle_prompt", "unknown"] {
        let mut p = payload();
        p["hook_event_name"] = json!("Notification");
        p["notification_type"] = json!(kind);
        cases.push(bytes(&p));
    }
    for input in cases {
        assert_eq!(record(&input, &e), None);
        assert_eq!(
            record_and_write(&input, &e, &mut Fake::new(tmp.path())).unwrap(),
            0
        );
    }
    assert_eq!(fs::read_dir(tmp.path()).unwrap().count(), 0);
}

struct Fake {
    elapsed: Duration,
    home: PathBuf,
    task: Option<PathBuf>,
    first_checked: Cell<bool>,
}
impl Fake {
    fn new(home: &Path) -> Self {
        Self {
            elapsed: Duration::ZERO,
            home: home.into(),
            task: None,
            first_checked: Cell::new(false),
        }
    }
}
impl Clock for Fake {
    fn now(&self) -> Timestamp {
        at(self.elapsed.as_millis() as u64)
    }
    fn monotonic(&self) -> Duration {
        self.elapsed
    }
    fn sleep(&mut self, d: Duration) {
        assert!(d <= Duration::from_millis(250));
        let first = lines(&self.home);
        assert_eq!(first.len(), 1);
        assert_eq!(first[0]["child"], Value::Null);
        self.first_checked.set(true);
        self.elapsed += d;
        if self.elapsed == Duration::from_secs(3)
            && let Some(path) = &self.task
        {
            fs::write(path, format!("session id: {CHILD}\n")).unwrap();
        }
    }
}
fn background(home: &Path) -> (Value, PathBuf) {
    let mut p = payload();
    p["tool_response"] = json!({"stdout":"","stderr":"","backgroundTaskId":"abc123_-9","interrupted":false,"isImage":false,"noOutputExpected":false});
    p["tool_input"]["run_in_background"] = json!(true);
    p["scratchpad_dir"] = json!(home.join("scratchpad"));
    let tasks = home.join("tasks");
    fs::create_dir(&tasks).unwrap();
    (p, tasks.join("abc123_-9.output"))
}
#[test]
fn background_first_write_then_child_after_three_seconds() {
    let tmp = TempDir::new().unwrap();
    let (p, task) = background(tmp.path());
    let mut clock = Fake::new(tmp.path());
    clock.task = Some(task);
    assert_eq!(
        record_and_write(&bytes(&p), &env(tmp.path()), &mut clock).unwrap(),
        2
    );
    assert!(clock.first_checked.get());
    assert_eq!(clock.elapsed, Duration::from_secs(3));
    let result = lines(tmp.path());
    assert_eq!(result.len(), 2);
    assert_eq!(result[1]["child"], CHILD);
    assert_eq!(result[1]["child_evidence"], "task_output");
    assert_eq!(result[0]["at"], at(0).to_string());
    assert_eq!(result[1]["at"], at(3000).to_string());
}
#[test]
fn background_stops_at_twenty_five_seconds_before_watchdog() {
    let tmp = TempDir::new().unwrap();
    let (p, task) = background(tmp.path());
    // A valid id beyond the read cap must not be seen.
    let mut output = vec![b'x'; 64 * 1024];
    output.extend_from_slice(format!("\nsession id: {CHILD}\n").as_bytes());
    fs::write(task, output).unwrap();
    let mut clock = Fake::new(tmp.path());
    assert_eq!(
        record_and_write(&bytes(&p), &env(tmp.path()), &mut clock).unwrap(),
        1
    );
    assert_eq!(clock.elapsed, Duration::from_secs(25));
    assert_eq!(lines(tmp.path())[0]["child"], Value::Null);
}
#[test]
fn background_missing_fields_and_unsafe_task_ids_do_not_poll() {
    for field in [
        "scratchpad_dir",
        "backgroundTaskId",
        "slash",
        "dots",
        "space",
        "empty",
    ] {
        let tmp = TempDir::new().unwrap();
        let (mut p, _) = background(tmp.path());
        match field {
            "scratchpad_dir" => {
                p.as_object_mut().unwrap().remove(field);
            }
            "backgroundTaskId" => {
                p["tool_response"].as_object_mut().unwrap().remove(field);
            }
            other => {
                p["tool_response"]["backgroundTaskId"] = json!(match other {
                    "slash" => "../escape",
                    "dots" => "ab..cd",
                    "space" => "bad id",
                    _ => "",
                })
            }
        }
        let mut clock = Fake::new(tmp.path());
        assert_eq!(
            record_and_write(&bytes(&p), &env(tmp.path()), &mut clock).unwrap(),
            1
        );
        assert_eq!(clock.elapsed, Duration::ZERO);
    }
}

// Recursive listings retain names, types, bytes, and hashes for before/after checks.
fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn visit(base: &Path, path: &Path, entries: &mut Vec<(PathBuf, Vec<u8>)>) {
        let mut paths: Vec<_> = fs::read_dir(path)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        paths.sort();
        for p in paths {
            if p.is_dir() {
                entries.push((p.strip_prefix(base).unwrap().into(), vec![]));
                visit(base, &p, entries);
            } else {
                entries.push((p.strip_prefix(base).unwrap().into(), fs::read(p).unwrap()));
            }
        }
    }
    let mut result = vec![];
    visit(root, root, &mut result);
    result
}
#[test]
fn agent_directories_unchanged_and_only_expected_state_file_created() {
    let tmp = TempDir::new().unwrap();
    for dir in [".claude", ".codex"] {
        fs::create_dir(tmp.path().join(dir)).unwrap();
        fs::write(tmp.path().join(dir).join("synthetic"), b"synthetic bytes").unwrap();
    }
    let before: Vec<_> = [".claude", ".codex"]
        .map(|d| snapshot(&tmp.path().join(d)))
        .into();
    let e = env(tmp.path());
    write(&record(&bytes(&payload()), &e).unwrap(), &e).unwrap();
    let after: Vec<_> = [".claude", ".codex"]
        .map(|d| snapshot(&tmp.path().join(d)))
        .into();
    use std::hash::{DefaultHasher, Hash, Hasher};
    let hashes = |snapshots: &Vec<Vec<(PathBuf, Vec<u8>)>>| {
        snapshots
            .iter()
            .map(|entries| {
                entries
                    .iter()
                    .map(|(path, bytes)| {
                        let mut hasher = DefaultHasher::new();
                        bytes.hash(&mut hasher);
                        (path.clone(), hasher.finish())
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(hashes(&before), hashes(&after));
    assert_eq!(before, after);
    assert_eq!(
        snapshot(&tmp.path().join(".multi-link"))
            .iter()
            .map(|(p, _)| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        vec!["hook", &format!("hook/{PARENT}.jsonl")]
    );
}
#[test]
fn rejects_protected_relative_and_aliased_state_directories() {
    let tmp = TempDir::new().unwrap();
    for name in [".claude", ".codex"] {
        fs::create_dir(tmp.path().join(name)).unwrap();
    }
    symlink(tmp.path().join(".claude"), tmp.path().join("alias")).unwrap();
    for state in [
        tmp.path().join(".claude/x"),
        tmp.path().join(".codex"),
        PathBuf::from("relative"),
        tmp.path().join("alias/x"),
        tmp.path().join(".codex-sibling/../.codex/x"),
    ] {
        let mut e = env(tmp.path());
        e.multi_link_home = Some(state);
        let r = record(&bytes(&payload()), &e).unwrap();
        assert!(write(&r, &e).is_err());
        assert!(snapshot(&tmp.path().join(".claude")).is_empty());
        assert!(snapshot(&tmp.path().join(".codex")).is_empty());
    }
    assert!(!tmp.path().join(".multi-link").exists());
}
#[test]
fn rejects_symlink_hook_and_record_without_changing_targets() {
    for record_link in [false, true] {
        let tmp = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        fs::write(target.path().join("sentinel"), b"unchanged").unwrap();
        let before = snapshot(target.path());
        let state = tmp.path().join(".multi-link");
        fs::create_dir(&state).unwrap();
        if record_link {
            fs::create_dir(state.join("hook")).unwrap();
            symlink(
                target.path().join("sentinel"),
                state.join(format!("hook/{PARENT}.jsonl")),
            )
            .unwrap();
        } else {
            symlink(target.path(), state.join("hook")).unwrap();
        }
        let e = env(tmp.path());
        assert!(write(&record(&bytes(&payload()), &e).unwrap(), &e).is_err());
        assert_eq!(before, snapshot(target.path()));
    }
}
#[test]
fn unwritable_state_returns_error() {
    let tmp = TempDir::new().unwrap();
    let state = tmp.path().join(".multi-link");
    fs::create_dir(&state).unwrap();
    fs::set_permissions(&state, fs::Permissions::from_mode(0o500)).unwrap();
    let e = env(tmp.path());
    let result = write(&record(&bytes(&payload()), &e).unwrap(), &e);
    fs::set_permissions(&state, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(result.is_err());
}
#[test]
fn eight_threads_append_eight_hundred_valid_lines() {
    let tmp = TempDir::new().unwrap();
    let e = Arc::new(env(tmp.path()));
    let r = Arc::new(record(&bytes(&payload()), &e).unwrap());
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let e = e.clone();
            let r = r.clone();
            std::thread::spawn(move || {
                for _ in 0..100 {
                    write(&r, &e).unwrap();
                }
            })
        })
        .collect();
    let results: Vec<_> = threads.into_iter().map(|thread| thread.join()).collect();
    for result in results {
        result.unwrap();
    }
    let output =
        fs::read_to_string(tmp.path().join(format!(".multi-link/hook/{PARENT}.jsonl"))).unwrap();
    assert_eq!(output.lines().count(), 800);
    for line in output.lines() {
        assert!(line.len() + 1 < 4096);
        assert_eq!(serde_json::from_str::<HookRecord>(line).unwrap(), *r);
    }
}

#[test]
fn valid_state_override_and_component_sibling_are_allowed() {
    let tmp = TempDir::new().unwrap();
    let mut e = env(tmp.path());
    e.multi_link_home = Some(tmp.path().join(".claude-sibling"));
    let r = record(&bytes(&payload()), &e).unwrap();
    write(&r, &e).unwrap();
    assert!(!tmp.path().join(".multi-link").exists());
    let path = e
        .multi_link_home
        .unwrap()
        .join(format!("hook/{PARENT}.jsonl"));
    assert_eq!(fs::read_to_string(path).unwrap().lines().count(), 1);
}

#[test]
fn oversized_metadata_is_rejected_before_creating_state() {
    let tmp = TempDir::new().unwrap();
    let e = env(tmp.path());
    let mut p = payload();
    p["cwd"] = json!("x".repeat(4096));
    assert!(record(&bytes(&p), &e).is_none());
    let mut r = record(&bytes(&payload()), &e).unwrap();
    r.cwd = "x".repeat(4096);
    assert!(write(&r, &e).is_err());
    assert_eq!(fs::read_dir(tmp.path()).unwrap().count(), 0);
}

#[test]
fn child_parser_requires_uuid_and_session_line_and_prefers_stdout() {
    let e = env(Path::new("/synthetic/home"));
    let mut p = payload();
    for text in [
        "session id: ../bad",
        "mention session id: abcdefab-1234-4321-8123-abcdef123456",
        "session id: abcdefab-1234-4321-8123-abcdef123456 extra",
        "[codex] Thread ready (bad)",
    ] {
        p["tool_response"]["stdout"] = json!(text);
        let r = record(&bytes(&p), &e).unwrap();
        assert_eq!(r.child, None);
        assert_eq!(r.child_evidence, None);
    }
    p["tool_response"]["stdout"] = json!(format!(
        "[codex] Thread ready ({PARENT})\nsession id: {CHILD}"
    ));
    let r = record(&bytes(&p), &e).unwrap();
    assert_eq!(r.child.as_deref(), Some(CHILD));
    assert_eq!(r.child_evidence.as_deref(), Some("stdout"));
}

fn check_redirect_form(operator: &str) {
    for suffix in ["", " 2>&1"] {
        for space in ["", " "] {
            let tmp = TempDir::new().unwrap();
            let log = tmp.path().join("run.log");
            fs::write(&log, format!("session id: {CHILD}\n")).unwrap();
            let mut p = payload();
            p["cwd"] = json!(tmp.path());
            p["tool_response"]["stdout"] = json!("");
            p["tool_input"]["command"] = json!(format!(
                "codex exec 'test' {operator}{space}{}{suffix}",
                log.display()
            ));
            assert_eq!(
                record_and_write(&bytes(&p), &env(tmp.path()), &mut Fake::new(tmp.path())).unwrap(),
                2,
                "{operator}{space}{suffix}"
            );
            let records = lines(tmp.path());
            assert_eq!(records[0]["child"], Value::Null);
            assert_eq!(records[1]["child"], CHILD);
            assert_eq!(records[1]["child_evidence"], "redirect_file");
        }
    }
}

macro_rules! redirect_form_test {
    ($name:ident, $operator:literal) => {
        #[test]
        fn $name() {
            check_redirect_form($operator);
        }
    };
}
redirect_form_test!(redirect_truncate, ">");
redirect_form_test!(redirect_append, ">>");
redirect_form_test!(redirect_stdout_descriptor, "1>");
redirect_form_test!(redirect_both_streams, "&>");
redirect_form_test!(redirect_both_streams_append, "&>>");
redirect_form_test!(redirect_clobber, ">|");

#[test]
fn redirect_polling_refuses_fifo_home_escape_and_oversized_logs() {
    // This synthetic home lives outside system temp, under the checkout, so a
    // symlink from the allowed temp log really escapes both allowed roots.
    let home = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let tmp = TempDir::new().unwrap();
    let log = tmp.path().join("run.log");
    let mut p = payload();
    p["cwd"] = json!(tmp.path());
    p["tool_response"]["stdout"] = json!("");
    p["tool_input"]["command"] = json!(format!("codex exec x >> {}", log.display()));
    let run = || {
        let mut clock = Fake::new(home.path());
        let count = record_and_write(&bytes(&p), &env(home.path()), &mut clock).unwrap();
        assert_eq!(count, 1);
        let records = lines(home.path());
        assert!(records.iter().all(|r| r["child"].is_null()));
        fs::remove_file(home.path().join(format!(".multi-link/hook/{PARENT}.jsonl"))).unwrap();
    };
    fs::write(&log, format!("session id: {CHILD}\n{}", "x".repeat(65536))).unwrap();
    run();
    fs::write(&log, format!("session id: {CHILD}\nsession id: invalid\n")).unwrap();
    run();
    fs::remove_file(&log).unwrap();
    let private = home.path().join("private.log");
    fs::write(&private, format!("session id: {CHILD}\n")).unwrap();
    symlink(&private, &log).unwrap();
    run();
    fs::remove_file(&log).unwrap();
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&log)
            .status()
            .unwrap()
            .success()
    );
    let (send, receive) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut clock = Fake::new(home.path());
        let count = record_and_write(&bytes(&p), &env(home.path()), &mut clock).unwrap();
        send.send((count, lines(home.path()), clock.elapsed))
            .unwrap();
    });
    let (count, records, elapsed) = receive
        .recv_timeout(Duration::from_secs(1))
        .expect("FIFO must not block");
    assert_eq!(count, 1);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["child"], Value::Null);
    assert_eq!(
        elapsed,
        Duration::ZERO,
        "unsafe files must not enter the poll loop"
    );
}

#[test]
fn redirect_poll_observes_later_id_after_first_null_record() {
    let tmp = TempDir::new().unwrap();
    let log = tmp.path().join("run.log");
    let mut p = payload();
    p["cwd"] = json!(tmp.path());
    p["tool_response"]["stdout"] = json!("");
    p["tool_input"]["command"] = json!(format!("codex exec x > {}", log.display()));
    let mut clock = Fake::new(tmp.path());
    clock.task = Some(log);
    assert_eq!(
        record_and_write(&bytes(&p), &env(tmp.path()), &mut clock).unwrap(),
        2
    );
    assert_eq!(clock.elapsed, Duration::from_secs(3));
    assert!(clock.first_checked.get());
    assert_eq!(lines(tmp.path())[1]["child_evidence"], "redirect_file");
}

#[test]
fn oversized_redirect_truncating_forms_observe_prefix_append_forms_refuse() {
    for (operator, expected) in [
        (">", 2),
        ("1>", 2),
        ("&>", 2),
        (">|", 2),
        (">>", 1),
        ("&>>", 1),
    ] {
        let tmp = TempDir::new().unwrap();
        let log = tmp.path().join("run.log");
        fs::write(
            &log,
            format!(
                "session id: {CHILD}\n{}\nsession id: {PARENT}\n",
                "x".repeat(65536)
            ),
        )
        .unwrap();
        let mut p = payload();
        p["cwd"] = json!(tmp.path());
        p["tool_response"]["stdout"] = json!("");
        p["tool_input"]["command"] = json!(format!("codex exec x {operator} {}", log.display()));
        let mut clock = Fake::new(tmp.path());
        assert_eq!(
            record_and_write(&bytes(&p), &env(tmp.path()), &mut clock).unwrap(),
            expected,
            "{operator}"
        );
        let records = lines(tmp.path());
        if expected == 2 {
            assert_eq!(records[1]["child"], CHILD);
            assert_eq!(records[1]["child_evidence"], "redirect_file");
        } else {
            assert_eq!(records[0]["child"], Value::Null);
            assert_eq!(clock.elapsed, Duration::ZERO);
        }
    }
}

#[test]
fn background_non_missing_task_open_error_is_returned_after_first_record() {
    let tmp = TempDir::new().unwrap();
    let (p, task) = background(tmp.path());
    // A symlink loop gives ELOOP on open on both macOS and Linux, even as root.
    symlink(&task, &task).unwrap();
    let mut clock = Fake::new(tmp.path());
    assert!(record_and_write(&bytes(&p), &env(tmp.path()), &mut clock).is_err());
    assert_eq!(clock.elapsed, Duration::ZERO);
    let records = lines(tmp.path());
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["child"], Value::Null);
}

#[test]
fn session_end_writes_one_metadata_record_without_polling() {
    for reason in [
        "prompt_input_exit",
        "clear",
        "other",
        "logout",
        "future-reason",
        "",
    ] {
        let tmp = TempDir::new().unwrap();
        let mut p = payload();
        p["hook_event_name"] = json!("SessionEnd");
        p["reason"] = json!(reason);
        p["agent_id"] = json!("ignored-agent");
        let mut clock = Fake::new(tmp.path());
        assert_eq!(
            record_and_write(&bytes(&p), &env(tmp.path()), &mut clock).unwrap(),
            1
        );
        assert_eq!(clock.elapsed, Duration::ZERO);
        assert_eq!(
            lines(tmp.path()),
            vec![json!({"v":1,"kind":"ended","at":"2026-10-29T23:12:39.000Z",
            "session_id":PARENT,"agent_id":null,"event":"SessionEnd","tool_use_id":null,
            "cwd":"/synthetic/repo","child":null,"child_evidence":null,"notification_type":null,"reason":reason})]
        );
    }
}

#[test]
fn session_end_requires_string_reason() {
    let tmp = TempDir::new().unwrap();
    for reason in [None, Some(Value::Null), Some(json!(1)), Some(json!([]))] {
        let mut p = payload();
        p["hook_event_name"] = json!("SessionEnd");
        if let Some(reason) = reason {
            p["reason"] = reason;
        }
        let mut clock = Fake::new(tmp.path());
        assert_eq!(
            record_and_write(&bytes(&p), &env(tmp.path()), &mut clock).unwrap(),
            0
        );
    }
    assert!(!tmp.path().join(".multi-link").exists());
}
