#!/usr/bin/env bash
# Phase 19 answer-key check (Step 19.0). Usage, from the repo root: fixtures/expected/menubar/check.sh
# Compares `multi-link menubar --json` for five fixture scenarios with the keys in this folder, using
# `cargo run` (never the installed binary) and a fixed --now. watch-codex-done.jsonl is checked by 19.1's
# watcher test (it replays fixtures/expected/pane/watch-script.json), not here. Keys are Claude's; do not edit them.
set -euo pipefail
export TZ=UTC
K=fixtures/expected/menubar
ML=(cargo run -q -p multi-link --)
if ! "${ML[@]}" menubar --help >/dev/null 2>&1; then echo "menubar subcommand missing"; exit 2; fi
fail=0
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
check() { # key name, root, now
  local name=$1 root=$2 now=$3 rc
  set +e; "${ML[@]}" menubar --json --root "$root" --now "$now" >"$TMP/out" 2>/dev/null; rc=$?; set -e
  if (( rc != 0 )); then echo "FAIL $name: exit $rc"; fail=1; return; fi
  if diff <(jq -S . "$K/$name.json") <(jq -S . "$TMP/out") >"$TMP/diff"; then echo "ok   $name"
  else echo "FAIL $name"; head -40 "$TMP/diff"; fail=1; fi
}
check multi-repo fixtures/sessions-phase14 2026-11-10T22:20:00Z
check waiting-quiet fixtures/sessions-phase9 2026-11-09T07:30:00Z
check open-actions fixtures/sessions-phase18 2026-11-12T09:00:00Z
# Step 20.0: the pane keys' other roots and times, captured from `menubar --json` before the pane was removed.
check phase9-early fixtures/sessions-phase9 2026-11-09T06:00:00Z
check phase9-noon fixtures/sessions-phase9 2026-11-09T12:00:00Z
exit $fail
