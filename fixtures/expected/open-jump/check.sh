#!/usr/bin/env bash
# Phase 18 answer-key check (Step 18.0). Usage, from the repo root: fixtures/expected/open-jump/check.sh
# Compares `open --dry-run [--fork]` for every session in fixtures/sessions-phase18 with open.json, using
# `cargo run` (never the installed binary) and the fixed --now. The fixture process table is
# fixtures/sessions-phase18/fixture-ps.txt (fixture mode only). Keys are Claude's; do not edit them.
set -euo pipefail
K=fixtures/expected/open-jump
ROOT=fixtures/sessions-phase18
NOW=2026-11-12T09:00:00Z
ML=(cargo run -q -p multi-link --)
fail=0
for mode in open fork; do
  for id in $(jq -r --arg m $mode '.[$m] | keys[]' $K/open.json); do
    args=(open "$id" --dry-run --root $ROOT --now $NOW); [[ $mode == fork ]] && args+=(--fork)
    set +e; out=$("${ML[@]}" "${args[@]}" 2>/dev/null); code=$?; set -e
    want=$(jq -cS --arg m $mode --arg id "$id" '.[$m][$id]' $K/open.json)
    got=$(jq -cS --argjson c $code '{exit: $c, action: .}' <<<"$out")
    if [[ $want == "$got" ]]; then echo "ok   open $mode ${id:0:8}"; else echo "FAIL open $mode ${id:0:8}: want $want got $got"; fail=1; fi
  done
done
exit $fail
