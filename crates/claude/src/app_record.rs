//! Claude desktop-app session records (8.4c S1).
//!
//! The app keeps one JSON record per app session under
//! `<home>/Library/Application Support/Claude/claude-code-sessions/<a>/<b>/local_<id>.json`.
//! This is undocumented app behaviour; every field below is an assumption that the
//! synthetic fixture and the real-data check confirm. Fields we do not read are ignored.
//! A JSON `null` counts as absent for every field we read.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde_json::Value;

const DIRECTORY: &str = "Library/Application Support/Claude/claude-code-sessions";

/// The fields S1 needs from one record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppRecord {
    /// The record's `sessionId` (`local_<id>`); the file stem when the field is absent.
    pub app_id: String,
    /// Transcript id of the Claude Code session this record belongs to.
    pub cli_session_id: String,
    /// `spawnedFrom.sessionId`: the app id of the record that started this one.
    pub parent_app_id: Option<String>,
    /// `spawnedFrom.userFork`: the user forked the session (not a session's act).
    pub user_fork: bool,
    /// `spawnedFrom.btw`: a side question (not a session's act).
    pub btw: bool,
}

/// Result of parsing one record file. Cheap to clone so the scanner can cache it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppRecordRead {
    Record(AppRecord),
    /// No `cliSessionId`: nothing to link, and not a diagnostic.
    Ignored,
    /// Unreadable file, malformed JSON, or a present field of the wrong type.
    Bad,
}

/// Every record found, with the files that could not be used.
#[derive(Debug, Default)]
pub struct AppRecordScan {
    pub records: Vec<AppRecord>,
    /// Sorted root-relative paths of bad record files.
    pub bad: Vec<String>,
    /// Directories that exist but could not be read (root-relative path, error).
    pub unreadable: Vec<(String, io::Error)>,
}

/// Record files in sorted path order. An absent directory is empty and silent; a
/// directory that exists but cannot be listed is returned as a failure and skipped.
pub fn app_record_paths(root: &Path) -> (Vec<PathBuf>, Vec<(String, io::Error)>) {
    let mut paths = Vec::new();
    let mut failures = Vec::new();
    let top = root.join(DIRECTORY);
    for first in list(root, &top, &mut failures) {
        if !first.is_dir() {
            continue;
        }
        for second in list(root, &first, &mut failures) {
            if !second.is_dir() {
                continue;
            }
            for path in list(root, &second, &mut failures) {
                let is_record = path.extension().is_some_and(|ext| ext == "json")
                    && path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with("local_"));
                if is_record && path.is_file() {
                    paths.push(path);
                }
            }
        }
    }
    paths.sort();
    (paths, failures)
}

fn list(root: &Path, directory: &Path, failures: &mut Vec<(String, io::Error)>) -> Vec<PathBuf> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Vec::new(),
        Err(error) => {
            failures.push((relative(root, directory), error));
            return Vec::new();
        }
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    paths.sort();
    paths
}

/// Parse one record file.
pub fn read_app_record(path: &Path) -> AppRecordRead {
    let Ok(bytes) = fs::read(path) else {
        return AppRecordRead::Bad;
    };
    let Ok(value) = serde_json::from_slice::<Value>(&bytes) else {
        return AppRecordRead::Bad;
    };
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    parse(&value, stem)
}

fn parse(value: &Value, stem: &str) -> AppRecordRead {
    let Some(object) = value.as_object() else {
        return AppRecordRead::Bad;
    };
    let cli_session_id = match object.get("cliSessionId") {
        None | Some(Value::Null) => return AppRecordRead::Ignored,
        Some(Value::String(id)) => id.clone(),
        Some(_) => return AppRecordRead::Bad,
    };
    let Ok(app_id) = string_field(object.get("sessionId")) else {
        return AppRecordRead::Bad;
    };
    let mut record = AppRecord {
        app_id: app_id.unwrap_or_else(|| stem.to_owned()),
        cli_session_id,
        parent_app_id: None,
        user_fork: false,
        btw: false,
    };
    match object.get("spawnedFrom") {
        None | Some(Value::Null) => {}
        Some(Value::Object(spawned)) => {
            let (Ok(parent), Ok(user_fork), Ok(btw)) = (
                string_field(spawned.get("sessionId")),
                bool_field(spawned.get("userFork")),
                bool_field(spawned.get("btw")),
            ) else {
                return AppRecordRead::Bad;
            };
            record.parent_app_id = parent;
            record.user_fork = user_fork;
            record.btw = btw;
        }
        Some(_) => return AppRecordRead::Bad,
    }
    AppRecordRead::Record(record)
}

fn string_field(value: Option<&Value>) -> Result<Option<String>, ()> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(_) => Err(()),
    }
}

fn bool_field(value: Option<&Value>) -> Result<bool, ()> {
    match value {
        None | Some(Value::Null) => Ok(false),
        Some(Value::Bool(flag)) => Ok(*flag),
        Some(_) => Err(()),
    }
}

/// List and parse every record under `root` (uncached; the scanner caches per file).
pub fn scan_app_records(root: &Path) -> AppRecordScan {
    let (paths, unreadable) = app_record_paths(root);
    let mut scan = AppRecordScan {
        unreadable,
        ..Default::default()
    };
    for path in paths {
        match read_app_record(&path) {
            AppRecordRead::Record(record) => scan.records.push(record),
            AppRecordRead::Ignored => {}
            AppRecordRead::Bad => scan.bad.push(relative(root, &path)),
        }
    }
    scan.bad.sort();
    scan
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}
