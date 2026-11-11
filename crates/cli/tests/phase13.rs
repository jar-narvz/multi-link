//! Phase 13 answer key (Step 13.0, Claude): literal `cd` launch folders and option-first resume ids.
use std::{fs, path::PathBuf, process::Command};

use serde_json::Value;

#[test]
fn phase13_links_match_answer_key() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let output = Command::new(env!("CARGO_BIN_EXE_multi-link"))
        .arg("links")
        .arg("--root")
        .arg(fixtures.join("sessions-phase13"))
        .args(["--now", "2026-11-10T08:00:00Z", "--since", "all", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    let actual: Value = serde_json::from_slice(&output.stdout).unwrap();
    let expected: Value =
        serde_json::from_slice(&fs::read(fixtures.join("expected/phase13/links.json")).unwrap())
            .unwrap();
    assert_eq!(actual, expected);
}
