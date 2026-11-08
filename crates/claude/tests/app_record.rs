//! Desktop-app session records (8.4c S1): parsing rules and directory listing.
//! The record format is undocumented app behaviour; every rule here is an assumption.

use std::{fs, path::Path};

use multi_link_claude::{
    AppRecord, AppRecordRead, app_record_paths, read_app_record, scan_app_records,
};

const DIR: &str = "Library/Application Support/Claude/claude-code-sessions";

fn write(root: &Path, relative: &str, text: &str) -> std::path::PathBuf {
    let path = root.join(DIR).join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, text).unwrap();
    path
}

fn parse(text: &str) -> AppRecordRead {
    let root = tempfile::tempdir().unwrap();
    let path = write(root.path(), "a/b/local_x.json", text);
    read_app_record(&path)
}

fn record(read: AppRecordRead) -> AppRecord {
    match read {
        AppRecordRead::Record(record) => record,
        other => panic!("expected a record, got {other:?}"),
    }
}

#[test]
fn plain_record_has_no_parent_and_no_flags() {
    let got = record(parse(r#"{"sessionId":"local_a","cliSessionId":"cli-a"}"#));
    assert_eq!(
        got,
        AppRecord {
            app_id: "local_a".into(),
            cli_session_id: "cli-a".into(),
            parent_app_id: None,
            user_fork: false,
            btw: false,
        }
    );
}

#[test]
fn spawned_from_gives_parent_and_flags() {
    let got = record(parse(
        r#"{"sessionId":"local_a","cliSessionId":"c","spawnedFrom":{"sessionId":"local_p","userFork":true,"btw":false}}"#,
    ));
    assert_eq!(got.parent_app_id.as_deref(), Some("local_p"));
    assert!(got.user_fork);
    assert!(!got.btw);
    let got = record(parse(
        r#"{"sessionId":"local_a","cliSessionId":"c","spawnedFrom":{"sessionId":"local_p","btw":true}}"#,
    ));
    assert!(got.btw && !got.user_fork);
}

#[test]
fn spawned_from_without_session_id_means_no_parent() {
    let got = record(parse(
        r#"{"sessionId":"local_a","cliSessionId":"c","spawnedFrom":{"name":"n"}}"#,
    ));
    assert_eq!(got.parent_app_id, None);
}

#[test]
fn missing_or_null_cli_session_id_is_ignored_even_if_other_fields_are_odd() {
    assert!(matches!(
        parse(r#"{"sessionId":"local_a"}"#),
        AppRecordRead::Ignored
    ));
    assert!(matches!(
        parse(r#"{"sessionId":"local_a","cliSessionId":null,"spawnedFrom":"x"}"#),
        AppRecordRead::Ignored
    ));
}

#[test]
fn malformed_json_and_wrong_types_are_bad() {
    for text in [
        r#"{"sessionId": "local_a", "cliSessionId": "#,
        "[]",
        r#"{"sessionId":"local_a","cliSessionId":7}"#,
        r#"{"sessionId":"local_a","cliSessionId":"c","spawnedFrom":"local_p"}"#,
        r#"{"sessionId":"local_a","cliSessionId":"c","spawnedFrom":{"sessionId":3}}"#,
        r#"{"sessionId":"local_a","cliSessionId":"c","spawnedFrom":{"sessionId":"p","userFork":"yes"}}"#,
        r#"{"sessionId":"local_a","cliSessionId":"c","spawnedFrom":{"sessionId":"p","btw":1}}"#,
        r#"{"sessionId":5,"cliSessionId":"c"}"#,
    ] {
        assert!(
            matches!(parse(text), AppRecordRead::Bad),
            "should be bad: {text}"
        );
    }
}

#[test]
fn absent_session_id_falls_back_to_the_file_stem() {
    let root = tempfile::tempdir().unwrap();
    let path = write(
        root.path(),
        "a/b/local_stem.json",
        r#"{"cliSessionId":"c"}"#,
    );
    assert_eq!(record(read_app_record(&path)).app_id, "local_stem");
}

#[test]
fn listing_takes_only_local_json_two_levels_down_sorted() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "a/b/local_2.json", "{}");
    write(root.path(), "a/b/local_1.json", "{}");
    write(root.path(), "a/c/local_0.json", "{}");
    write(root.path(), "a/b/other.json", "{}");
    write(root.path(), "a/b/local_3.txt", "{}");
    write(root.path(), "a/local_9.json", "{}");
    write(root.path(), "top.json", "{}");
    let (paths, failures) = app_record_paths(root.path());
    assert!(failures.is_empty());
    let names: Vec<String> = paths
        .iter()
        .map(|p| {
            p.strip_prefix(root.path().join(DIR))
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(
        names,
        ["a/b/local_1.json", "a/b/local_2.json", "a/c/local_0.json"]
    );
}

#[test]
fn absent_directory_is_empty_with_no_failure() {
    let root = tempfile::tempdir().unwrap();
    let scan = scan_app_records(root.path());
    assert!(scan.records.is_empty() && scan.bad.is_empty() && scan.unreadable.is_empty());
}

#[test]
fn scan_collects_records_and_bad_relative_paths() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "a/b/local_ok.json",
        r#"{"sessionId":"local_ok","cliSessionId":"c"}"#,
    );
    write(root.path(), "a/b/local_bad.json", "{");
    write(
        root.path(),
        "a/b/local_none.json",
        r#"{"sessionId":"local_none"}"#,
    );
    let scan = scan_app_records(root.path());
    assert_eq!(scan.records.len(), 1);
    assert_eq!(scan.records[0].app_id, "local_ok");
    assert_eq!(scan.bad, [format!("{DIR}/a/b/local_bad.json")]);
}

#[cfg(unix)]
#[test]
fn unreadable_directory_is_reported_not_fatal() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "a/b/local_ok.json", r#"{"cliSessionId":"c"}"#);
    let dir = root.path().join(DIR).join("a");
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o000)).unwrap();
    let (paths, failures) = app_record_paths(root.path());
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
    if fs::read_dir(&dir).is_ok() && failures.is_empty() {
        return; // running as a user that ignores modes
    }
    assert!(paths.is_empty());
    assert_eq!(failures.len(), 1);
    assert!(failures[0].0.ends_with("claude-code-sessions/a"));
}
