use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Liveness {
    Alive,
    Gone,
    Unavailable,
}

pub trait ProcessProbe {
    fn probe(&self, pid: u32, proc_start: &str) -> Liveness;
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
    fn probe(&self, pid: u32, proc_start: &str) -> Liveness {
        // ps treats zero as a process-group selector, not an individual pid.
        if pid == 0 {
            return Liveness::Unavailable;
        }
        let Ok(output) = Command::new(&self.ps_binary)
            .args(["-p", &pid.to_string(), "-o", "lstart="])
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
