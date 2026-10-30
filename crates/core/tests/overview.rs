use std::{fs, path::Path};

use multi_link_core::{FixtureProcessProbe, Options, RepoMode, Scanner};
use multi_link_model::Timestamp;
use serde_json::Value;

#[test]
fn phase4_overview_matches_answer_key() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let root = fixtures.join("sessions-phase4");
    let options = Options {
        state_dir: root.join(".multi-link"),
        now: Timestamp::parse("2026-10-29T23:15:00Z").unwrap(),
        since: None,
        repo_mode: RepoMode::Fixture,
        probe: &FixtureProcessProbe,
    };
    let mut plans = |folder: &Path| {
        multi_link_plan::steps_report(&root, &[folder.to_string_lossy().into_owned()])
            .repos
            .remove(0)
    };
    let output = Scanner::default()
        .overview(&root, &options, &mut plans)
        .unwrap();
    let expected: Value =
        serde_json::from_slice(&fs::read(fixtures.join("expected/phase4/overview.json")).unwrap())
            .unwrap();
    assert_eq!(serde_json::to_value(output.report).unwrap(), expected);
}
