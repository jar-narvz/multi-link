//! Metadata-only Claude hook recorder.
//!
//! `record` parses the first/only record without I/O, using `HookEnv::now`.
//! `record_and_write` writes that record before polling, then appends at most one
//! child update. Its injected `Clock` supplies wall time, monotonic time and sleep.
//! `write` takes the same injected environment so even direct writes enforce the
//! state-directory policy without consulting process environment variables.

use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::OpenOptionsExt,
    path::{Component, Path, PathBuf},
    time::Duration,
};

use multi_link_model::Timestamp;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MAX_INPUT_BYTES: usize = 1024 * 1024;
const POLL_LIMIT: Duration = Duration::from_secs(25);
const POLL_INTERVAL: Duration = Duration::from_millis(250);
#[cfg(target_os = "macos")]
const O_NOFOLLOW: i32 = 0x100;
#[cfg(not(target_os = "macos"))]
const O_NOFOLLOW: i32 = 0x20000;

#[derive(Debug, Clone)]
pub struct HookEnv {
    pub home: PathBuf,
    pub multi_link_home: Option<PathBuf>,
    pub now: Timestamp,
}

/// Both time sources must advance consistently; monotonic time must not go back.
pub trait Clock {
    fn now(&self) -> Timestamp;
    fn monotonic(&self) -> Duration;
    fn sleep(&mut self, duration: Duration);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookRecord {
    pub v: u8,
    pub kind: String,
    pub at: Timestamp,
    pub session_id: String,
    pub agent_id: Option<String>,
    pub event: String,
    pub tool_use_id: Option<String>,
    pub cwd: String,
    pub child: Option<String>,
    pub child_evidence: Option<String>,
    pub notification_type: Option<String>,
}

fn uuid(input: &str) -> Option<String> {
    (input.len() == 36
        && input.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        }))
    .then(|| input.to_ascii_lowercase())
}

fn parse(payload: &[u8]) -> Option<Value> {
    if payload.len() > MAX_INPUT_BYTES {
        return None;
    }
    serde_json::from_slice(payload).ok()
}

fn child(text: &str, threads: bool) -> Option<(String, String)> {
    for line in text.lines() {
        if let Some(id) = line
            .trim()
            .strip_prefix("session id:")
            .and_then(|s| uuid(s.trim()))
        {
            return Some((id, "stdout".into()));
        }
    }
    if threads {
        for line in text.lines() {
            if let Some((_, rest)) = line.split_once("[codex] Thread ready (")
                && let Some((id, _)) = rest.split_once(')')
                && let Some(id) = uuid(id)
            {
                return Some((id, "thread_ready".into()));
            }
        }
    }
    None
}

pub fn record(payload: &[u8], env: &HookEnv) -> Option<HookRecord> {
    record_value(&parse(payload)?, env)
}

fn record_value(p: &Value, env: &HookEnv) -> Option<HookRecord> {
    let session_id = p.get("session_id")?.as_str()?;
    if uuid(session_id)?.as_str() != session_id {
        return None;
    }
    let event = p.get("hook_event_name")?.as_str()?;
    let mut r = HookRecord {
        v: 1,
        kind: String::new(),
        at: env.now,
        session_id: session_id.into(),
        agent_id: p.get("agent_id").and_then(Value::as_str).map(str::to_owned),
        event: event.into(),
        tool_use_id: None,
        cwd: p.get("cwd")?.as_str()?.into(),
        child: None,
        child_evidence: None,
        notification_type: None,
    };
    match event {
        "PostToolUse" | "PostToolUseFailure" => {
            if p.get("tool_name")?.as_str()? != "Bash" {
                return None;
            }
            multi_link_claude::recognise(p.get("tool_input")?.get("command")?.as_str()?)?;
            r.kind = "handoff".into();
            r.tool_use_id = Some(p.get("tool_use_id")?.as_str()?.into());
            let output = if event == "PostToolUseFailure" {
                p.get("error")
            } else {
                p.get("tool_response").and_then(|v| v.get("stdout"))
            };
            if let Some((id, evidence)) =
                output.and_then(Value::as_str).and_then(|s| child(s, true))
            {
                r.child = Some(id);
                r.child_evidence = Some(evidence);
            }
        }
        "PermissionRequest" => r.kind = "waiting".into(),
        "Notification" => {
            let kind = p.get("notification_type")?.as_str()?;
            if !["permission_prompt", "elicitation_dialog"].contains(&kind) {
                return None;
            }
            r.kind = "waiting".into();
            r.notification_type = Some(kind.into());
        }
        _ => return None,
    }
    // Reject oversized metadata before any directory is created.
    encoded(&r).ok()?;
    Some(r)
}

fn task_path(p: &Value) -> Option<PathBuf> {
    if !p.get("tool_input")?.get("run_in_background")?.as_bool()? {
        return None;
    }
    let id = p.get("tool_response")?.get("backgroundTaskId")?.as_str()?;
    if id.is_empty()
        || id.contains("..")
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return None;
    }
    let scratch = Path::new(p.get("scratchpad_dir")?.as_str()?);
    if !scratch.is_absolute() {
        return None;
    }
    Some(scratch.parent()?.join("tasks").join(format!("{id}.output")))
}

/// Returns the number of lines appended (0, 1, or 2). Missing task files are
/// retried; other I/O errors are returned. Polling reads at most 64 KiB per try.
pub fn record_and_write(
    payload: &[u8],
    env: &HookEnv,
    clock: &mut impl Clock,
) -> io::Result<usize> {
    let start = clock.monotonic();
    let Some(p) = parse(payload) else {
        return Ok(0);
    };
    let mut received = env.clone();
    received.now = clock.now();
    let Some(mut r) = record_value(&p, &received) else {
        return Ok(0);
    };
    write(&r, env)?;
    if r.kind != "handoff" || r.child.is_some() {
        return Ok(1);
    }
    let Some(path) = task_path(&p) else {
        return Ok(1);
    };
    while clock.monotonic().saturating_sub(start) < POLL_LIMIT {
        match File::open(&path) {
            Ok(file) => {
                let mut bytes = Vec::new();
                file.take(64 * 1024).read_to_end(&mut bytes)?;
                if let Some((id, _)) = child(&String::from_utf8_lossy(&bytes), false) {
                    r.child = Some(id);
                    r.child_evidence = Some("task_output".into());
                    r.at = clock.now();
                    write(&r, env)?;
                    return Ok(2);
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        let remaining = POLL_LIMIT.saturating_sub(clock.monotonic().saturating_sub(start));
        clock.sleep(POLL_INTERVAL.min(remaining));
    }
    Ok(1)
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

fn encoded(r: &HookRecord) -> io::Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec(r)?;
    bytes.push(b'\n');
    if bytes.len() >= 4096 {
        return Err(invalid("hook record exceeds line limit"));
    }
    Ok(bytes)
}

fn normalized(path: &Path) -> io::Result<PathBuf> {
    if !path.is_absolute() {
        return Err(invalid("state and home must be absolute"));
    }
    let mut result = PathBuf::new();
    for c in path.components() {
        match c {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            _ => result.push(c),
        }
    }
    Ok(result)
}

// Resolve the deepest existing ancestor, including symlinks, before creating anything.
fn resolved(path: &Path) -> io::Result<PathBuf> {
    let mut ancestor = path;
    let mut suffix = Vec::new();
    loop {
        match fs::canonicalize(ancestor) {
            Ok(mut base) => {
                for part in suffix.iter().rev() {
                    base.push(part);
                }
                return Ok(base);
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                suffix.push(
                    ancestor
                        .file_name()
                        .ok_or_else(|| invalid("no existing ancestor"))?,
                );
                ancestor = ancestor.parent().ok_or_else(|| invalid("no parent"))?;
            }
            Err(e) => return Err(e),
        }
    }
}

fn state_dir(env: &HookEnv) -> io::Result<PathBuf> {
    let home = normalized(&env.home)?;
    let state = normalized(
        &env.multi_link_home
            .clone()
            .unwrap_or_else(|| home.join(".multi-link")),
    )?;
    let real_state = resolved(&state)?;
    for protected in [home.join(".claude"), home.join(".codex")] {
        if state.starts_with(&protected) || real_state.starts_with(resolved(&protected)?) {
            return Err(invalid("state directory overlaps agent data"));
        }
    }
    Ok(state)
}

fn directory(path: &Path) -> io::Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(O_NOFOLLOW)
        .open(path)?;
    if !file.metadata()?.is_dir() {
        return Err(invalid("expected directory"));
    }
    Ok(file)
}

fn mkdir(path: &Path) -> io::Result<()> {
    match fs::create_dir(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(target_os = "macos")]
const O_NONBLOCK: i32 = 4;
#[cfg(not(target_os = "macos"))]
const O_NONBLOCK: i32 = 0x800;

/// Append one bounded JSON line with one write on an O_APPEND descriptor.
/// Uses std OpenOptionsExt with O_NOFOLLOW for both directories and the record
/// file, plus symlink_metadata checks before creation. No libc dependency or FFI.
/// The state directory's parent must already exist; no ancestor is created.
/// Directory checks assume the user's state tree is not concurrently renamed
/// by an adversary; O_NOFOLLOW atomically protects the final file component.
pub fn write(r: &HookRecord, env: &HookEnv) -> io::Result<()> {
    if uuid(&r.session_id).as_deref() != Some(r.session_id.as_str()) {
        return Err(invalid("invalid session id"));
    }
    let bytes = encoded(r)?;
    let state = state_dir(env)?;
    mkdir(&state)?;
    let _state = directory(&state)?;
    let hook = state.join("hook");
    // symlink_metadata never follows an existing hook symlink, including a
    // dangling one. Recheck via O_NOFOLLOW after mkdir (other hooks may race).
    match fs::symlink_metadata(&hook) {
        Ok(meta) if !meta.is_dir() => {
            return Err(invalid("hook must be a directory, not a symlink"));
        }
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => mkdir(&hook)?,
        Err(e) => return Err(e),
    }
    let _hook = directory(&hook)?;
    let path = hook.join(format!("{}.jsonl", r.session_id));
    match fs::symlink_metadata(&path) {
        Ok(meta) if !meta.is_file() => return Err(invalid("record must be a regular file")),
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }
    let mut options = OpenOptions::new();
    options
        .append(true)
        .create(true)
        .mode(0o600)
        .custom_flags(O_NOFOLLOW | O_NONBLOCK);
    let opened = options.open(&path);
    // macOS can return ENOENT when another appender wins a create lookup.
    // One retry retains the same no-follow and append flags.
    let mut file = match opened {
        Err(e) if e.kind() == io::ErrorKind::NotFound => options.open(&path)?,
        result => result?,
    };
    if !file.metadata()?.is_file() {
        return Err(invalid("expected regular record file"));
    }
    if file.write(&bytes)? != bytes.len() {
        return Err(io::Error::new(
            io::ErrorKind::WriteZero,
            "partial hook record write",
        ));
    }
    Ok(())
}
