use std::{fs, path::PathBuf, process::Command};

#[test]
fn live_matches_phase14_key() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let output = Command::new(env!("CARGO_BIN_EXE_multi-link"))
        .args(["live", "--root"])
        .arg(fixtures.join("sessions-phase14"))
        .args(["--now", "2026-11-10T22:20:00Z", "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let expected: serde_json::Value =
        serde_json::from_slice(&fs::read(fixtures.join("expected/phase14/live.json")).unwrap())
            .unwrap();
    assert_eq!(actual, expected);
}
