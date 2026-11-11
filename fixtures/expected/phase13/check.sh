#!/usr/bin/env bash
# Phase 13 answer-key check (Step 13.0). Usage, from the repo root: fixtures/expected/phase13/check.sh
# Compares `links` on fixtures/sessions-phase13 with links.json here, using `cargo run` (never the installed
# binary) and the fixed --now. Keys are Claude's; do not edit them.
set -euo pipefail
K=fixtures/expected/phase13
ROOT=fixtures/sessions-phase13
NOW=2026-11-10T08:00:00Z
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
if ! cargo run -q -p multi-link -- links --root $ROOT --now $NOW --since all --json >"$TMP/out" 2>/dev/null; then
  echo "FAIL links.json: command failed"; exit 1
fi
if diff <(jq -S . "$K/links.json") <(jq -S . "$TMP/out") >"$TMP/diff"; then
  echo "ok   links.json"
else
  echo "FAIL links.json"; head -80 "$TMP/diff"; exit 1
fi
