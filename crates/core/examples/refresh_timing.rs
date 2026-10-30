//! Read real home data without modifying it. Keep one Scanner between passes.
use multi_link_core::{Options, RealProcessProbe, RepoMode, Scanner};
use multi_link_model::Timestamp;
use std::{
    env, io,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let passes: usize = env::args().nth(1).unwrap_or_else(|| "3".into()).parse()?;
    let root = PathBuf::from(env::var_os("HOME").ok_or_else(|| io::Error::other("HOME missing"))?);
    let state_dir = env::var_os("MULTI_LINK_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join(".multi-link"));
    let probe = RealProcessProbe::default();
    let mut scanner = Scanner::default();
    for pass in 1..=passes {
        let start = Instant::now();
        let now = Timestamp::from_epoch_ms(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_millis()
                .try_into()?,
        )?;
        let options = Options {
            state_dir: state_dir.clone(),
            now,
            since: Some(Duration::from_secs(7200)),
            repo_mode: RepoMode::Real,
            probe: &probe,
        };
        let (sessions, _) = scanner.refresh(&root, &options)?;
        println!(
            "pass {pass}: {} ms, {} sessions, {} files re-parsed, {} bytes re-parsed",
            start.elapsed().as_millis(),
            sessions.report.sessions.len(),
            scanner.stats.files,
            scanner.stats.bytes
        );
        for diagnostic in &scanner.diagnostics {
            eprintln!("{diagnostic}");
        }
        if pass < passes {
            std::thread::sleep(Duration::from_secs(2));
        }
    }
    Ok(())
}
