use multi_link_model::{CardLink, LiveReport};
use serde_json::{Value, json};

#[test]
fn live_key_round_trips_with_all_nulls_and_timestamps() {
    let key: Value =
        serde_json::from_str(include_str!("../../../fixtures/expected/phase14/live.json")).unwrap();
    let report: LiveReport = serde_json::from_value(key.clone()).unwrap();
    assert_eq!(serde_json::to_value(report).unwrap(), key);
}

#[test]
fn card_link_variants_preserve_exact_shape_and_nulls() {
    for value in [
        json!({"kind":"none"}),
        json!({"kind":"from","parent":"p","match":"heuristic","at":"2026-11-10T21:32:00.000Z"}),
        json!({"kind":"handoff","at":"2026-11-10T21:32:00.000Z","match":"unlinked","child":null,"candidates":0,"outcome":null,"outcome_at":null}),
    ] {
        let link: CardLink = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(link).unwrap(), value);
    }
}
