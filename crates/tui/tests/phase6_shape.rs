//! Step 6.1: the hand-drawn Phase 6 targets are well formed. Steps 6.3 and 6.4 compare real renders to them.
#[allow(dead_code)] // Only the styles header is needed here.
mod support;

use std::{fs, path::PathBuf};

const WIDTH: usize = 120;
const HEIGHT: usize = 40;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &str) -> String {
    fs::read_to_string(root().join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// Text golden: 40 rows, at most 120 cells each (every glyph used is one cell wide).
fn text_rows(name: &str) -> Vec<Vec<char>> {
    let text = read(&format!("fixtures/expected/tui/{name}.txt"));
    assert!(!text.contains('\r'), "{name}: CRLF line ends");
    let rows: Vec<Vec<char>> = text.lines().map(|line| line.chars().collect()).collect();
    assert_eq!(rows.len(), HEIGHT, "{name}: rows");
    for (y, row) in rows.iter().enumerate() {
        assert!(
            row.len() <= WIDTH,
            "{name}: row {y} has {} cells",
            row.len()
        );
        assert!(
            row.iter().all(|&c| c == ' ' || !c.is_control()),
            "{name}: row {y} has a control character"
        );
    }
    rows
}

/// Styles golden: the Phase 5 header, then runs that cover every cell exactly once, in order.
fn check_styles(name: &str, rows: &[Vec<char>]) {
    let styles = read(&format!("fixtures/expected/tui/{name}.styles.txt"));
    let mut lines = styles.lines();
    assert_eq!(
        format!("{}\n", lines.next().unwrap()),
        support::HEADER,
        "{name}: header"
    );
    let (mut row, mut next) = (0usize, 0usize);
    for line in lines {
        let fields: Vec<&str> = line.split(' ').collect();
        assert_eq!(fields.len(), 5, "{name}: {line}");
        let y: usize = fields[0].parse().unwrap();
        let (first, last) = fields[1].split_once('-').unwrap();
        let (first, last): (usize, usize) = (first.parse().unwrap(), last.parse().unwrap());
        if y != row {
            assert_eq!(
                (y, next),
                (row + 1, WIDTH),
                "{name}: row {row} not covered: {line}"
            );
            (row, next) = (y, 0);
        }
        assert_eq!(first, next, "{name}: gap or overlap at {line}");
        assert!(first <= last && last < WIDTH, "{name}: {line}");
        let colour = |value: &str| {
            value == "-"
                || (value.len() == 7
                    && value.starts_with('#')
                    && value[1..].chars().all(|c| c.is_ascii_hexdigit()))
        };
        assert!(colour(fields[2]) && colour(fields[3]), "{name}: {line}");
        assert_ne!(
            fields[3], "-",
            "{name}: every cell paints its background: {line}"
        );
        let blank = (first..=last).all(|x| rows[y].get(x).is_none_or(|&c| c == ' '));
        let any_blank = (first..=last).any(|x| rows[y].get(x).is_none_or(|&c| c == ' '));
        if any_blank {
            assert!(blank, "{name}: run mixes blank and drawn cells: {line}");
            assert_eq!(fields[2], "-", "{name}: blank cells have fg -: {line}");
            assert!(
                !["BOLD", "DIM", "ITALIC"]
                    .iter()
                    .any(|m| fields[4].split('+').any(|f| f == *m)),
                "{name}: {line}"
            );
        } else {
            assert_ne!(fields[2], "-", "{name}: drawn cells have a colour: {line}");
        }
        assert!(
            fields[4] == "-" || fields[4].split('+').all(|m| m == "BOLD"),
            "{name}: Phase 5 V4 allows BOLD only: {line}"
        );
        next = last + 1;
    }
    assert_eq!(
        (row, next),
        (HEIGHT - 1, WIDTH),
        "{name}: not every cell covered"
    );
}

#[test]
fn phase6_goldens_are_120_by_40_and_fully_styled() {
    for name in ["home-phase6", "repo-phase6"] {
        let rows = text_rows(name);
        check_styles(name, &rows);
    }
}

#[test]
fn phase6_keys_parse_as_json() {
    for name in ["sessions", "overview"] {
        let path = format!("fixtures/expected/phase6/{name}.json");
        let value: serde_json::Value =
            serde_json::from_str(&read(&path)).unwrap_or_else(|error| panic!("{path}: {error}"));
        assert!(value.is_object(), "{path}");
    }
}

#[test]
fn phase6_keys_carry_the_added_fields_on_every_session() {
    fn check(row: &serde_json::Value, path: &str) {
        for field in ["activity", "asked", "last_asked", "imported", "state_since"] {
            assert!(
                row.get(field).is_some(),
                "{path}: {} lacks {field}",
                row["id"]
            );
        }
        for child in row["children"].as_array().into_iter().flatten() {
            check(child, path);
        }
    }
    let path = "fixtures/expected/phase6/sessions.json";
    let sessions: serde_json::Value = serde_json::from_str(&read(path)).unwrap();
    for session in sessions["sessions"].as_array().unwrap() {
        check(session, path);
    }
    let path = "fixtures/expected/phase6/overview.json";
    let overview: serde_json::Value = serde_json::from_str(&read(path)).unwrap();
    for group in overview["repos"].as_array().unwrap() {
        for row in group["rows"].as_array().unwrap() {
            match row["kind"].as_str() {
                Some("merged") => row["rows"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .for_each(|r| check(r, path)),
                _ => check(row, path),
            }
        }
    }
}
