//! Step 8.1: the hand-drawn step graph target and answer keys are well formed. Step 8.3 compares the
//! report to the keys; Step 8.4 compares the render to the golden.
#[allow(dead_code)] // Only the styles header is needed here.
mod support;

use std::{collections::BTreeSet, fs, path::PathBuf};

use serde_json::Value;

const WIDTH: usize = 120;
const HEIGHT: usize = 40;
const GOLDEN: &str = "step-phase8";
const IDS: [&str; 13] = [
    "1.1", "1.2", "1.3", "1.4", "1.5", "1.6", "1.7", "1.8", "1.9", "1.10", "1.11", "1.12", "1.13",
];
const PART_FIELDS: [&str; 17] = [
    "chain",
    "kind",
    "harness",
    "session",
    "from",
    "to",
    "status",
    "run_available",
    "active_ms",
    "stretches",
    "asked",
    "said",
    "counts",
    "notable",
    "waiting_reads",
    "untagged_handoffs",
    "edge_in",
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &str) -> String {
    fs::read_to_string(root().join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
}

fn key(id: &str) -> Value {
    let path = format!("fixtures/expected/phase8/graph-{id}.json");
    serde_json::from_str(&read(&path)).unwrap_or_else(|error| panic!("{path}: {error}"))
}

fn text_rows() -> Vec<Vec<char>> {
    let text = read(&format!("fixtures/expected/tui/{GOLDEN}.txt"));
    assert!(!text.contains('\r'), "CRLF line ends");
    let rows: Vec<Vec<char>> = text.lines().map(|line| line.chars().collect()).collect();
    assert_eq!(rows.len(), HEIGHT, "rows");
    for (y, row) in rows.iter().enumerate() {
        assert!(row.len() <= WIDTH, "row {y} has {} cells", row.len());
        assert!(
            row.iter().all(|&c| c == ' ' || !c.is_control()),
            "row {y} has a control character"
        );
    }
    rows
}

fn cell(rows: &[Vec<char>], y: usize, x: usize) -> char {
    rows[y].get(x).copied().unwrap_or(' ')
}

/// One styles line: row, first, last, fg, bg, modifiers.
type Run<'a> = (usize, usize, usize, &'a str, &'a str, &'a str);

fn style_runs(styles: &str) -> Vec<Run<'_>> {
    let mut lines = styles.lines();
    assert_eq!(
        format!("{}\n", lines.next().unwrap()),
        support::HEADER,
        "header"
    );
    lines
        .map(|line| {
            let fields: Vec<&str> = line.split(' ').collect();
            assert_eq!(fields.len(), 5, "{line}");
            let (first, last) = fields[1].split_once('-').unwrap();
            (
                fields[0].parse().unwrap(),
                first.parse().unwrap(),
                last.parse().unwrap(),
                fields[2],
                fields[3],
                fields[4],
            )
        })
        .collect()
}

#[test]
fn golden_is_120_by_40_and_every_cell_is_styled_once() {
    let rows = text_rows();
    let styles = read(&format!("fixtures/expected/tui/{GOLDEN}.styles.txt"));
    let colour = |value: &str| {
        value == "-"
            || (value.len() == 7
                && value.starts_with('#')
                && value[1..].chars().all(|c| c.is_ascii_hexdigit()))
    };
    let (mut row, mut next) = (0usize, 0usize);
    for (y, first, last, fg, bg, modifiers) in style_runs(&styles) {
        if y != row {
            assert_eq!((y, next), (row + 1, WIDTH), "row {row} not covered");
            (row, next) = (y, 0);
        }
        assert_eq!(first, next, "gap or overlap at row {y} column {first}");
        assert!(first <= last && last < WIDTH, "row {y}: {first}-{last}");
        assert!(colour(fg) && colour(bg), "row {y}: {fg} {bg}");
        assert_ne!(bg, "-", "row {y}: every cell paints its background");
        let blanks = (first..=last).filter(|&x| cell(&rows, y, x) == ' ').count();
        if blanks > 0 {
            assert_eq!(
                blanks,
                last - first + 1,
                "row {y}: run mixes blank and drawn"
            );
            assert_eq!((fg, modifiers), ("-", "-"), "row {y}: blank cells");
        } else {
            assert_ne!(fg, "-", "row {y}: drawn cells have a colour");
        }
        assert!(
            modifiers == "-" || modifiers == "BOLD",
            "row {y}: V4 allows BOLD only"
        );
        next = last + 1;
    }
    assert_eq!((row, next), (HEIGHT - 1, WIDTH), "not every cell covered");
}

#[test]
fn golden_boxes_sit_on_the_slot_grid_with_the_selection_on_implement() {
    let rows = text_rows();
    let styles = read(&format!("fixtures/expected/tui/{GOLDEN}.styles.txt"));
    let runs = style_runs(&styles);
    let bg_at = |y: usize, x: usize| {
        runs.iter()
            .find(|run| run.0 == y && run.1 <= x && x <= run.2)
            .unwrap()
            .4
    };
    for (top, slots) in [(4, [0, 2, 4].as_slice()), (16, [1, 3].as_slice())] {
        for slot in slots {
            let x = 1 + 22 * slot;
            assert_eq!(cell(&rows, top, x), '╭', "slot {slot}");
            assert_eq!(cell(&rows, top, x + 29), '╮', "slot {slot}");
            assert_eq!(cell(&rows, top + 8, x), '╰', "slot {slot}");
            assert_eq!(cell(&rows, top + 8, x + 29), '╯', "slot {slot}");
        }
    }
    let selected = "#1B2130";
    for y in 16..=24 {
        assert_eq!(bg_at(y, 23), selected, "implement row {y}");
        assert_eq!(bg_at(y, 52), selected, "implement row {y}");
        assert_eq!(bg_at(y, 67), "#121A26", "fix round row {y}");
    }
    assert_eq!(bg_at(33, 32), selected, "strip follows the selection");
    assert_eq!(bg_at(33, 79), "#1E2A3D", "unselected Codex strip");
    assert_eq!(bg_at(33, 23), "#3A2A1F", "Claude strip");
}

#[test]
fn every_step_in_the_fixture_plan_has_a_key_and_no_extra_keys_exist() {
    let plan = read("fixtures/sessions-phase8/repos/repo-g/PLAN.md");
    let planned: BTreeSet<String> = plan
        .lines()
        .filter_map(|line| line.strip_prefix("### Step "))
        .map(|rest| rest.split(' ').next().unwrap().to_owned())
        .collect();
    let expected: BTreeSet<String> = IDS.iter().map(|id| (*id).to_owned()).collect();
    assert_eq!(planned, expected);
    let on_disk: BTreeSet<String> = fs::read_dir(root().join("fixtures/expected/phase8"))
        .unwrap()
        .map(|entry| {
            let name = entry.unwrap().file_name().into_string().unwrap();
            name.strip_prefix("graph-")
                .and_then(|rest| rest.strip_suffix(".json"))
                .unwrap_or_else(|| panic!("unexpected file {name}"))
                .to_owned()
        })
        .collect();
    assert_eq!(on_disk, expected);
}

#[test]
fn keys_have_every_field_and_consistent_parts() {
    for id in IDS {
        let report = key(id);
        for field in ["now", "repo", "step", "commit", "parts", "empty"] {
            assert!(report.get(field).is_some(), "{id}: {field}");
        }
        assert_eq!(report["step"]["id"], id);
        assert_eq!(report["now"], "2026-11-06T18:00:00.000Z");
        assert!(report["commit"].is_null(), "{id}: fixture mode runs no git");
        let parts = report["parts"].as_array().unwrap();
        assert_eq!(parts.is_empty(), report["empty"].is_string(), "{id}: empty");
        for (index, part) in parts.iter().enumerate() {
            let at = format!("{id} part {index}");
            for field in PART_FIELDS {
                assert!(part.get(field).is_some(), "{at}: {field}");
            }
            assert!(part["notable"].as_array().unwrap().len() <= 3, "{at}");
            match part["harness"].as_str().unwrap() {
                "claude" => {
                    assert!(part["from"].as_str().unwrap() < part["to"].as_str().unwrap());
                    assert!(part["status"].is_null(), "{at}");
                    assert!(part["run_available"].is_null(), "{at}");
                    assert!(part["asked"].is_null() && part["said"].is_null(), "{at}");
                }
                "codex" => {
                    assert_eq!(part["edge_in"]["kind"], "handoff", "{at}");
                    let available = part["run_available"].as_bool().unwrap();
                    assert_eq!(part["from"].is_string(), available, "{at}");
                    assert_eq!(part["edge_in"]["gap_ms"].is_i64(), available, "{at}");
                    let status = part["status"].as_str().unwrap();
                    assert_eq!(
                        ["done", "working", "stale"].contains(&status),
                        available,
                        "{at}: {status}"
                    );
                }
                other => panic!("{at}: harness {other}"),
            }
        }
    }
}

#[test]
fn keys_hold_only_fixed_labels() {
    for id in IDS {
        let report = key(id);
        for part in report["parts"].as_array().unwrap() {
            for call in part["notable"].as_array().unwrap() {
                let label = call["label"].as_str().unwrap();
                let fixed = label.starts_with("▸ codex exec · step ")
                    || label.starts_with("▸ codex exec · continue ")
                    || label.starts_with("edit ")
                    || [
                        "$ git diff",
                        "$ git commit",
                        "$ cargo test",
                        "$ cargo clippy",
                        "$ cargo fmt",
                        "$ cargo build",
                        "$ pytest",
                        "$ npm test",
                    ]
                    .contains(&label);
                assert!(fixed, "{id}: {label}");
            }
        }
        assert!(
            !report.to_string().contains("sentinel"),
            "{id}: command text reached the key"
        );
    }
}
