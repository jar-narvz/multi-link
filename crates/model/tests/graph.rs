use multi_link_model::{Edge, GraphReport, Match, Timestamp};
use serde_json::{Value, json};
use std::path::PathBuf;

#[test]
fn every_phase8_key_round_trips() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/expected/phase8");
    let mut read = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let original: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap())
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let report: GraphReport = serde_json::from_value(original.clone())
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert_eq!(
            serde_json::to_value(&report).unwrap(),
            original,
            "{}",
            path.display()
        );
        read += 1;
    }
    assert_eq!(read, 13);
}

#[test]
fn edge_kinds_serialize_as_schema() {
    let at: Timestamp = serde_json::from_value(json!("2026-11-06T18:00:00.000Z")).unwrap();
    let ret = serde_json::to_value(Edge::Return { at }).unwrap();
    assert_eq!(
        ret,
        json!({"kind": "return", "at": "2026-11-06T18:00:00.000Z"})
    );
    let handoff = serde_json::to_value(Edge::Handoff {
        at,
        r#match: Match::Exact,
        parent: "p".into(),
        parent_part: Some(0),
        gap_ms: None,
        candidates: 0,
        nearest_run: false,
    })
    .unwrap();
    assert!(handoff.get("nearest_run").is_none());
    assert_eq!(handoff["kind"], "handoff");
    assert_eq!(handoff["match"], "exact");
    assert!(handoff.get("gap_ms").unwrap().is_null());
}

#[test]
fn nearest_run_edges_round_trip_and_false_is_omitted() {
    let base = json!({"kind":"handoff", "at":"2026-11-06T18:00:00.000Z", "match":"heuristic",
        "parent":"p", "parent_part":0, "gap_ms":1000, "candidates":1});
    for nearest in [None, Some(false), Some(true)] {
        let mut input = base.clone();
        if let Some(nearest) = nearest {
            input["nearest_run"] = json!(nearest);
        }
        let edge: Edge = serde_json::from_value(input).unwrap();
        let mut expected = base.clone();
        if nearest == Some(true) {
            expected["nearest_run"] = json!(true);
        }
        assert_eq!(serde_json::to_value(edge).unwrap(), expected);
    }
}
