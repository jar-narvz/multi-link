#!/usr/bin/env bash
# Phase 12 answer-key check (Step 12.0). Usage, from the repo root: fixtures/expected/phase12/check.sh
# Compares links, the overview projection below and the step graph of repo-p12a with the keys in this folder,
# using `cargo run` (never the installed binary) and the fixed --now. Keys are Claude's; do not edit them.
set -euo pipefail
K=fixtures/expected/phase12
ROOT=fixtures/sessions-phase12
NOW=2026-11-10T12:00:00Z
ML=(cargo run -q -p multi-link --)
fail=0
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
check() { # name, live-side jq filter, command... (the command must also exit 0); the key is compared as is
  local name=$1 filter=$2 rc; shift 2
  set +e; "$@" >"$TMP/out" 2>/dev/null; rc=$?; set -e
  if (( rc != 0 )); then echo "FAIL $name: command exited $rc"; fail=1; return; fi
  if diff <(jq -S . "$K/$name") <(jq -S "$filter" "$TMP/out") >"$TMP/diff" 2>&1; then
    echo "ok   $name"
  else
    echo "FAIL $name"; head -60 "$TMP/diff"; fail=1
  fi
}
# Overview projection: the row tree (ids, children, hand-off links) of the repos K1/K2 change.
OV='def h: {invocation, match, child, evidence};
def t: {id, children: ([.children[]? | t] | sort_by(.id)), handoffs: ([.handoffs[] | h] | sort_by(.invocation))};
{repos: [.repos[] | select(.repo | test("/repo-p12(a|e|f2|i)$")) | {repo, latest: (.latest_handoff | h),
  rows: ([.rows[] | t] | sort_by(.id))}] | sort_by(.repo)}'
check links.json . "${ML[@]}" links --root $ROOT --now $NOW --since all --json
check overview.json "$OV" "${ML[@]}" overview --root $ROOT --now $NOW --since all --json
check graph-1.1.json . "${ML[@]}" graph --root $ROOT --now $NOW --repo /home/dev/repo-p12a --step 1.1 --json
exit $fail
