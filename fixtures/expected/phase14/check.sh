#!/usr/bin/env bash
# Phase 14 answer-key check (Step 14.0). Usage, from the repo root: fixtures/expected/phase14/check.sh
# Compares `live` on fixtures/sessions-phase14 with live.json here, using `cargo run` (never the installed
# binary), the fixed --now and the default 2h window (the TUI's). Keys are Claude's; do not edit them.
set -euo pipefail
K=fixtures/expected/phase14
ROOT=fixtures/sessions-phase14
NOW=2026-11-10T22:20:00Z
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
if ! cargo run -q -p multi-link -- live --root $ROOT --now $NOW --json >"$TMP/out" 2>"$TMP/err"; then
  echo "FAIL live.json: command failed"; head -5 "$TMP/err"; exit 1
fi
if diff <(jq -S . "$K/live.json") <(jq -S . "$TMP/out") >"$TMP/diff"; then
  echo "ok   live.json"
else
  echo "FAIL live.json"; head -80 "$TMP/diff"; exit 1
fi
