//! Settings inspection and merging with an injectable, read-only binary probe.
use std::os::unix::fs::PermissionsExt;
use std::{
    fmt,
    path::{Path, PathBuf},
};

use serde::{
    Deserialize, Deserializer,
    de::{self, MapAccess, SeqAccess, Visitor},
};
use serde_json::{Map, Value, json};

/// Resolve an existing executable regular file without ever executing it.
pub trait BinaryProbe {
    fn executable(&self, path: &Path) -> Option<PathBuf>;
}

pub struct FileSystemProbe;
impl BinaryProbe for FileSystemProbe {
    fn executable(&self, path: &Path) -> Option<PathBuf> {
        let canonical = std::fs::canonicalize(path).ok()?;
        let metadata = std::fs::metadata(&canonical).ok()?;
        (metadata.is_file() && metadata.permissions().mode() & 0o111 != 0).then_some(canonical)
    }
}

fn temporary(path: &Path) -> bool {
    path.starts_with("/Volumes")
        || path
            .components()
            .any(|c| c.as_os_str() == "AppTranslocation")
}

/// Quote one literal argument for a POSIX shell.
pub fn quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

/// The single definition used by install, uninstall and check.
pub fn snippet(binary: &str) -> Value {
    let entry = json!({"type":"command", "command":format!("{} hook record", quote(binary)), "async":true, "timeout":60});
    json!({"hooks": {
        "PostToolUse":[{"matcher":"Bash", "hooks":[entry.clone()]}],
        "PostToolUseFailure":[{"matcher":"Bash", "hooks":[entry.clone()]}],
        "PermissionRequest":[{"hooks":[entry.clone()]}],
        "Notification":[{"matcher":"permission_prompt|elicitation_dialog", "hooks":[entry.clone()]}],
        "SessionEnd":[{"hooks":[entry]}]
    }})
}

#[derive(Debug, serde::Serialize)]
pub struct Finding {
    pub event: String,
    pub status: String,
    pub details: Vec<String>,
}

#[derive(Debug)]
pub struct Check {
    pub findings: Vec<Finding>,
    pub merged: Value,
    pub needs_fix: bool,
    /// Sorted, unique canonical paths owning `other copy` events.
    pub owners: Vec<String>,
}
impl Check {
    pub fn current(&self) -> bool {
        self.findings
            .iter()
            .all(|f| ["ok", "other copy"].contains(&f.status.as_str()))
    }
}

// Deserialize objects with duplicate detection before Value can discard a key.
struct Unique(Value);
impl<'de> Deserialize<'de> for Unique {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct UniqueVisitor;
        impl<'de> Visitor<'de> for UniqueVisitor {
            type Value = Unique;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("JSON with unique object keys")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Unique, A::Error> {
                let mut map = Map::new();
                while let Some(key) = access.next_key::<String>()? {
                    if map.contains_key(&key) {
                        return Err(de::Error::custom("duplicate object key"));
                    }
                    map.insert(key, access.next_value::<Unique>()?.0);
                }
                Ok(Unique(Value::Object(map)))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut access: A) -> Result<Unique, A::Error> {
                let mut array = Vec::new();
                while let Some(value) = access.next_element::<Unique>()? {
                    array.push(value.0);
                }
                Ok(Unique(Value::Array(array)))
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Unique, E> {
                Ok(Unique(json!(v)))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Unique, E> {
                Ok(Unique(v.into()))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Unique, E> {
                Ok(Unique(Value::Null))
            }
        }
        deserializer.deserialize_any(UniqueVisitor)
    }
}

fn parse(text: &str) -> Result<Value, String> {
    let value = serde_json::from_str::<Unique>(text)
        .map_err(|e| e.to_string())?
        .0;
    // JSON has already been validated. Skip quoted strings (including escaped
    // quotes) and compare every original number token with its serialized form.
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                i += 1;
                while bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                i += 1;
            }
            b'-' | b'0'..=b'9' => {
                let start = i;
                while i < bytes.len()
                    && matches!(bytes[i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
                {
                    i += 1;
                }
                let token = &text[start..i];
                let number: serde_json::Number =
                    serde_json::from_str(token).map_err(|e| e.to_string())?;
                if number.to_string() != token {
                    return Err("number would change when serialized".into());
                }
            }
            _ => i += 1,
        }
    }
    Ok(value)
}

/// Recognize only a literal bare or install-style single-quoted binary argument.
fn binary(entry: &Value) -> Option<String> {
    if entry.get("type")?.as_str()? != "command" {
        return None;
    }
    let argument = entry
        .get("command")?
        .as_str()?
        .strip_suffix(" hook record")?;
    let path = if let Some(quoted) = argument
        .strip_prefix('\'')
        .and_then(|s| s.strip_suffix('\''))
    {
        // Install escapes apostrophes by closing the quote, escaping, reopening.
        let path = quoted.replace("'\\''", "'");
        if quote(&path) != argument {
            return None;
        }
        path
    } else {
        if argument.is_empty()
            || argument
                .chars()
                .any(|c| c.is_whitespace() || "'\"\\;$`|&<>()*?[]{}".contains(c))
        {
            return None;
        }
        argument.to_owned()
    };
    (path.rsplit('/').next() == Some("multi-link")).then_some(path)
}

fn validate(value: &Value) -> Result<(), String> {
    let root = value.as_object().ok_or("settings must be an object")?;
    if let Some(hooks) = root.get("hooks") {
        for (event, groups) in hooks.as_object().ok_or("hooks must be an object")? {
            for group in groups
                .as_array()
                .ok_or_else(|| format!("{event} must be an array"))?
            {
                let group = group
                    .as_object()
                    .ok_or_else(|| format!("{event} group must be an object"))?;
                group
                    .get("hooks")
                    .and_then(Value::as_array)
                    .ok_or_else(|| format!("{event} group hooks must be an array"))?;
            }
        }
    }
    Ok(())
}

/// Refuse lossy source text, then inspect and merge without mutating the input.
pub fn check(text: &str, binary_path: &str) -> Result<Check, String> {
    check_with_probe(text, binary_path, false, &FileSystemProbe)
}

/// Check a previously parsed value. Source-text refusals belong to `check`.
pub fn check_value(value: &Value, binary_path: &str) -> Result<Check, String> {
    check_value_with_probe(value, binary_path, false, &FileSystemProbe)
}

/// Inspect with an explicit ownership policy and filesystem probe.
pub fn check_with_probe(
    text: &str,
    binary_path: &str,
    take_over: bool,
    probe: &dyn BinaryProbe,
) -> Result<Check, String> {
    check_value_with_probe(&parse(text)?, binary_path, take_over, probe)
}

fn check_value_with_probe(
    value: &Value,
    binary_path: &str,
    take_over: bool,
    probe: &dyn BinaryProbe,
) -> Result<Check, String> {
    validate(value)?;
    let running = probe
        .executable(Path::new(binary_path))
        .unwrap_or_else(|| PathBuf::from(binary_path));
    let binary_path = running.to_str().ok_or("binary path is not UTF-8")?;
    let expected = snippet(binary_path);
    let mut merged = value.clone();
    let mut findings = Vec::new();
    let mut needs_fix = false;
    let mut owners = std::collections::BTreeSet::new();
    for (event, template) in expected["hooks"].as_object().unwrap() {
        let groups = value
            .get("hooks")
            .and_then(|h| h.get(event))
            .and_then(Value::as_array);
        let mut entries = Vec::new();
        for group in groups.into_iter().flatten() {
            for entry in group["hooks"].as_array().unwrap() {
                if let Some(path) = binary(entry) {
                    entries.push((group, entry, path));
                }
            }
        }
        let matcher = &template[0]["matcher"];
        let customized = entries.iter().any(|(group, entry, _)| {
            let actual = &group["matcher"];
            let matches = if matcher.is_null() {
                group.get("matcher").is_none() || actual == "" || actual == "*"
            } else {
                actual == matcher
            };
            !matches
                || entry
                    .as_object()
                    .unwrap()
                    .keys()
                    .any(|key| !["type", "command", "async", "timeout"].contains(&key.as_str()))
        });
        let mut details = Vec::new();
        let mut event_owners = std::collections::BTreeSet::new();
        for (_, entry, path) in &entries {
            let stored = Path::new(path);
            let resolved = stored
                .is_absolute()
                .then(|| probe.executable(stored))
                .flatten();
            if path != binary_path && resolved.as_deref() != Some(running.as_path()) {
                if let Some(owner) =
                    resolved.filter(|p| !temporary(Path::new(path)) && !temporary(p))
                {
                    let owner = owner.to_str().ok_or("owner path is not UTF-8")?.to_owned();
                    details.push(format!("owned by {owner}"));
                    event_owners.insert(owner);
                } else {
                    details.push(format!("binary: {} -> {}", path, binary_path));
                }
            }
            for key in ["async", "timeout"] {
                if entry.get(key) != template[0]["hooks"][0].get(key) {
                    details.push(format!(
                        "{key}: {} -> {}",
                        entry[key], template[0]["hooks"][0][key]
                    ));
                }
            }
        }
        let status = if customized {
            details.push(
                "edit multi-link entries and matcher by hand; custom fields or matcher are kept"
                    .into(),
            );
            "customized"
        } else if !event_owners.is_empty() && !take_over {
            owners.extend(event_owners);
            if entries.len() > 1 {
                details.push("duplicate entries kept".into());
            }
            "other copy"
        } else if entries.is_empty() {
            "missing"
        } else if entries.len() > 1 {
            "duplicate"
        } else if details.is_empty() {
            "ok"
        } else {
            "outdated"
        };
        if ["missing", "outdated", "duplicate"].contains(&status) {
            needs_fix = true;
            let root = merged.as_object_mut().unwrap();
            let hooks = root
                .entry("hooks")
                .or_insert_with(|| json!({}))
                .as_object_mut()
                .unwrap();
            let groups = hooks
                .entry(event)
                .or_insert_with(|| json!([]))
                .as_array_mut()
                .unwrap();
            groups.retain_mut(|group| {
                let hooks = group["hooks"].as_array_mut().unwrap();
                let before = hooks.len();
                hooks.retain(|entry| binary(entry).is_none());
                before == hooks.len() || !hooks.is_empty()
            });
            groups.push(template[0].clone());
        }
        findings.push(Finding {
            event: event.clone(),
            status: status.into(),
            details,
        });
    }
    if let Some(hooks) = value.get("hooks").and_then(Value::as_object) {
        for (event, groups) in hooks {
            if expected["hooks"].get(event).is_none()
                && groups.as_array().unwrap().iter().any(|g| {
                    g["hooks"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|e| binary(e).is_some())
                })
            {
                findings.push(Finding {
                    event: event.clone(),
                    status: "unknown event".into(),
                    details: vec!["edit multi-link entries by hand; kept unchanged".into()],
                });
            }
        }
    }
    if value.get("disableAllHooks") == Some(&Value::Bool(true)) {
        findings.push(Finding {
            event: "disableAllHooks".into(),
            status: "true".into(),
            details: vec!["hooks are disabled; edit by hand".into()],
        });
    }
    Ok(Check {
        findings,
        merged,
        needs_fix,
        owners: owners.into_iter().collect(),
    })
}

/// Remove recognized multi-link entries at every event, preserving other data.
/// Return None for a semantic no-op, so callers retain the original bytes.
pub fn remove(text: &str) -> Result<Option<Value>, String> {
    let mut value = parse(text)?;
    validate(&value)?;
    let mut changed = false;
    if let Some(events) = value.get_mut("hooks").and_then(Value::as_object_mut) {
        events.retain(|_, groups| {
            let groups = groups.as_array_mut().unwrap();
            let before = groups.len();
            groups.retain_mut(|group| {
                let hooks = group["hooks"].as_array_mut().unwrap();
                let before = hooks.len();
                hooks.retain(|entry| binary(entry).is_none());
                let removed = before != hooks.len();
                changed |= removed;
                !removed || !hooks.is_empty()
            });
            groups.len() == before || !groups.is_empty()
        });
        if changed && events.is_empty() {
            value.as_object_mut().unwrap().remove("hooks");
        }
    }
    Ok(changed.then_some(value))
}
