use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

/// macOS `ps -p` rejects larger pids with a stderr error that would sink the
/// whole batch, so out-of-range pids are answered alone as Unavailable.
#[cfg(target_os = "macos")]
const MAX_PID: u32 = 99_999;
#[cfg(not(target_os = "macos"))]
const MAX_PID: u32 = i32::MAX as u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Liveness {
    Alive,
    Gone,
    Unavailable,
}

pub trait ProcessProbe {
    fn probe(&self, pid: u32, proc_start: &str) -> Liveness;

    /// Results have the same order and length as requests. Implementations with
    /// external probes override this to use at most one process per pass.
    fn batch(&self, requests: &[(u32, &str)]) -> Vec<Liveness> {
        requests
            .iter()
            .map(|(pid, start)| self.probe(*pid, start))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct RealProcessProbe {
    ps_binary: PathBuf,
}

impl Default for RealProcessProbe {
    fn default() -> Self {
        Self::new("/bin/ps")
    }
}

impl RealProcessProbe {
    pub fn new(ps_binary: impl Into<PathBuf>) -> Self {
        Self {
            ps_binary: ps_binary.into(),
        }
    }
}

impl ProcessProbe for RealProcessProbe {
    fn batch(&self, requests: &[(u32, &str)]) -> Vec<Liveness> {
        let mut pids: Vec<_> = requests
            .iter()
            .map(|(pid, _)| *pid)
            .filter(|p| *p != 0 && *p <= MAX_PID)
            .collect();
        pids.sort_unstable();
        pids.dedup();
        if pids.is_empty() {
            return vec![Liveness::Unavailable; requests.len()];
        }
        let list = pids
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let output = Command::new(&self.ps_binary)
            .args(["-o", "pid=,lstart=", "-p", &list])
            .env("TZ", "UTC")
            .output();
        let Ok(output) = output else {
            return vec![Liveness::Unavailable; requests.len()];
        };
        classify_batch(
            output.status.success(),
            &output.stdout,
            &output.stderr,
            requests,
        )
    }

    fn probe(&self, pid: u32, proc_start: &str) -> Liveness {
        // ps treats zero as a process-group selector, not an individual pid.
        if pid == 0 {
            return Liveness::Unavailable;
        }
        let Ok(output) = Command::new(&self.ps_binary)
            .args(["-p", &pid.to_string(), "-o", "lstart="])
            .env("TZ", "UTC")
            .output()
        else {
            return Liveness::Unavailable;
        };
        classify_output(
            output.status.success(),
            &output.stdout,
            &output.stderr,
            proc_start,
        )
    }
}

fn classify_batch(
    success: bool,
    stdout: &[u8],
    stderr: &[u8],
    requests: &[(u32, &str)],
) -> Vec<Liveness> {
    let unavailable = || vec![Liveness::Unavailable; requests.len()];
    if !stderr.iter().all(u8::is_ascii_whitespace) {
        return unavailable();
    }
    let Ok(text) = std::str::from_utf8(stdout) else {
        return unavailable();
    };
    if !success && !text.trim().is_empty() {
        return unavailable();
    }
    let mut starts = HashMap::new();
    for line in text.lines().filter(|s| !s.trim().is_empty()) {
        let line = line.trim();
        let Some(split) = line.find(char::is_whitespace) else {
            return unavailable();
        };
        let Ok(pid) = line[..split].parse::<u32>() else {
            return unavailable();
        };
        let start = line[split..].trim();
        if start.is_empty() || starts.insert(pid, start).is_some() {
            return unavailable();
        }
    }
    requests
        .iter()
        .map(|(pid, expected)| {
            if *pid == 0 || *pid > MAX_PID {
                Liveness::Unavailable
            } else {
                match starts.get(pid) {
                    None => Liveness::Gone,
                    Some(actual) if actual.split_whitespace().eq(expected.split_whitespace()) => {
                        Liveness::Alive
                    }
                    _ => Liveness::Gone,
                }
            }
        })
        .collect()
}

fn classify_output(success: bool, stdout: &[u8], stderr: &[u8], proc_start: &str) -> Liveness {
    // Permission failures (including sandbox denials) must never imply exit.
    // Treat other diagnostics conservatively too: an unreliable probe proves nothing.
    if !stderr.iter().all(u8::is_ascii_whitespace) {
        return Liveness::Unavailable;
    }
    let Ok(stdout) = std::str::from_utf8(stdout) else {
        return Liveness::Unavailable;
    };
    if stdout.trim().is_empty() {
        return Liveness::Gone;
    }
    if !success {
        return Liveness::Unavailable;
    }
    if stdout.split_whitespace().eq(proc_start.split_whitespace()) {
        Liveness::Alive
    } else {
        Liveness::Gone
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FixtureProcessProbe;

impl ProcessProbe for FixtureProcessProbe {
    fn probe(&self, _pid: u32, _proc_start: &str) -> Liveness {
        Liveness::Unavailable
    }
}

/// A deterministic probe keyed by both pid and recorded start time.
#[derive(Debug, Default, Clone)]
pub struct FakeProcessProbe {
    pub results: HashMap<(u32, String), Liveness>,
}

impl ProcessProbe for FakeProcessProbe {
    fn probe(&self, pid: u32, proc_start: &str) -> Liveness {
        self.results
            .get(&(pid, proc_start.to_owned()))
            .copied()
            .unwrap_or(Liveness::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::{Liveness, classify_output};

    #[test]
    fn classifies_ps_output_conservatively() {
        let start = "Thu Oct  1 19:40:25 2026";
        for (success, stdout, stderr, expected) in [
            (true, " Thu Oct  1 19:40:25 2026\n", "", Liveness::Alive),
            (true, "Thu Oct 1 19:40:25 2026", "", Liveness::Alive),
            (true, "different start", "", Liveness::Gone),
            (true, "", "", Liveness::Gone),
            (false, " \n", "", Liveness::Gone),
            (
                false,
                "",
                "ps: Operation not permitted",
                Liveness::Unavailable,
            ),
            (true, start, "ps: Permission denied", Liveness::Unavailable),
            (false, start, "", Liveness::Unavailable),
            (false, "", "ps: invalid option", Liveness::Unavailable),
        ] {
            assert_eq!(
                classify_output(success, stdout.as_bytes(), stderr.as_bytes(), start),
                expected,
                "success={success}, stdout={stdout:?}, stderr={stderr:?}"
            );
        }
        assert_eq!(
            classify_output(true, &[0xff], b"", start),
            Liveness::Unavailable
        );
    }
}

#[cfg(test)]
mod batch_tests {
    use super::*;

    #[test]
    fn mixed_batch_handles_missing_reused_and_zero_pids() {
        let start = "Fri Oct 30 00:08:22 2026";
        let requests = [
            (12, start),
            (12, "wrong"),
            (13, start),
            (0, start),
            (14, start),
            (MAX_PID + 1, start),
        ];
        let stdout = b" 12 Fri Oct 30 00:08:22 2026\n 14 Fri Oct 30 00:08:22 2026\n";
        assert_eq!(
            classify_batch(true, stdout, b"", &requests),
            vec![
                Liveness::Alive,
                Liveness::Gone,
                Liveness::Gone,
                Liveness::Unavailable,
                Liveness::Alive,
                Liveness::Unavailable
            ]
        );
        for (success, stdout, stderr) in [
            (false, &b""[..], &b"ps: Operation not permitted"[..]),
            (true, &b"garbage"[..], &b""[..]),
            (false, &stdout[..], &b""[..]),
            (true, &b"12\n"[..], &b""[..]),
            (true, &[0xff][..], &b""[..]),
        ] {
            assert_eq!(
                classify_batch(success, stdout, stderr, &requests),
                vec![Liveness::Unavailable; 6]
            );
        }
        assert_eq!(
            classify_batch(false, b"", b"", &requests),
            vec![
                Liveness::Gone,
                Liveness::Gone,
                Liveness::Gone,
                Liveness::Unavailable,
                Liveness::Gone,
                Liveness::Unavailable
            ]
        );
    }

    #[test]
    fn real_batch_spawns_once_with_deduplicated_pids_and_utc() {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        let binary = temp.path().join("ps");
        let count = temp.path().join("count");
        // The stub also checks the exact command, including the UTC environment.
        fs_write_stub(&binary, &count);
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
        let probe = RealProcessProbe::new(&binary);
        assert_eq!(
            probe.batch(&[
                (0, ""),
                (14, "wrong"),
                (12, "Fri Oct 30 00:08:22 2026"),
                (12, "old"),
                (MAX_PID + 1, "x")
            ]),
            vec![
                Liveness::Unavailable,
                Liveness::Gone,
                Liveness::Alive,
                Liveness::Gone,
                Liveness::Unavailable
            ]
        );
        assert_eq!(std::fs::read_to_string(&count).unwrap(), "call\n");
        assert!(probe.batch(&[]).is_empty());
        assert_eq!(probe.batch(&[(0, "")]), vec![Liveness::Unavailable]);
        assert_eq!(std::fs::read_to_string(&count).unwrap(), "call\n");
    }

    fn fs_write_stub(binary: &std::path::Path, count: &std::path::Path) {
        std::fs::write(binary, format!(
            "#!/bin/sh\necho call >> '{}'\n[ \"$TZ\" = UTC ] || exit 2\n[ \"$*\" = '-o pid=,lstart= -p 12,14' ] || exit 2\necho ' 12 Fri Oct 30 00:08:22 2026'\n",
            count.display())).unwrap();
    }
}
