#!/usr/bin/env bash
# Phase 9 answer-key check (Step 9.0; N=6 added in Step 9.6). Usage, from the repo root: fixtures/expected/phase9/check.sh <N>
# N = 1..6: compares the cases of steps 9.1..9.N (ids starting 91..9N) with the keys in this folder,
# using `cargo run` (never the installed binary) and the fixed --now. Keys are Claude's; do not edit them.
set -euo pipefail
N=${1:?usage: check.sh <1-6>}
[[ $N =~ ^[1-6]$ ]] || { echo "N must be 1-6" >&2; exit 2; }
K=fixtures/expected/phase9
ROOT=fixtures/sessions-phase9
NOW=2026-11-09T12:00:00Z
ML=(cargo run -q -p multi-link --)
RE="^9[1-$N]"
fail=0
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
check() { # name, key-side jq filter, live-side command... (the command must also exit 0)
  local name=$1 filter=$2 rc; shift 2
  set +e; "$@" >"$TMP/out" 2>/dev/null; rc=$?; set -e
  if (( rc != 0 )); then echo "FAIL $name ($RE): command exited $rc"; fail=1; return; fi
  if diff <(jq -S "$filter" "$K/$name") <(jq -S "$filter" "$TMP/out") >"$TMP/diff" 2>&1; then
    echo "ok   $name ($RE)"
  else
    echo "FAIL $name ($RE)"; head -40 "$TMP/diff"; fail=1
  fi
}
S_F="{now, sessions: [.sessions[] | select(.id | test(\"$RE\"))]}"
L_F="{links: [.links[] | select(.parent | test(\"$RE\"))], unparented}"
check sessions.json "$S_F" "${ML[@]}" sessions --root $ROOT --now $NOW --since all --json
check links.json "$L_F" "${ML[@]}" links --root $ROOT --now $NOW --since all --json
check graph-1.1.json . "${ML[@]}" graph --root $ROOT --now $NOW --repo /home/dev/repo-p9a --step 1.1 --json
# Step 9.6 (R7): implement, then fix rounds 1 and 2 from `codex exec resume --last`, each after a Claude review part.
(( N >= 6 )) && check graph-2.1.json . "${ML[@]}" graph --root $ROOT --now $NOW --repo /home/dev/repo-p9r --step 2.1 --json
if (( N >= 4 )); then
  check sessions.json . "${ML[@]}" sessions --root $ROOT --now $NOW --since all --json
  check links.json . "${ML[@]}" links --root $ROOT --now $NOW --since all --json
fi
# stderr diagnostics of the sessions run: the corrupt .zst is the only unreadable file (from 9.1 on).
set +e; err=$("${ML[@]}" sessions --root $ROOT --now $NOW --since all --json 2>&1 >/dev/null); rc=$?; set -e
if [[ $rc == 0 && $err == "unreadable: 1" ]]; then echo "ok   stderr unreadable: 1"; else echo "FAIL stderr (exit $rc): $err"; fail=1; fi
if (( N >= 2 )); then
  for mode in open fork; do
    for id in $(jq -r --arg m $mode --arg re "$RE" '.[$m] | keys[] | select(test($re))' $K/open.json); do
      args=(open "$id" --dry-run --root $ROOT --now $NOW); [[ $mode == fork ]] && args+=(--fork)
      set +e; out=$("${ML[@]}" "${args[@]}" 2>/dev/null); code=$?; set -e
      want=$(jq -cS --arg m $mode --arg id "$id" '.[$m][$id]' $K/open.json)
      got=$(jq -cS --argjson c $code '{exit: $c, action: .}' <<<"$out")
      if [[ $want == "$got" ]]; then echo "ok   open $mode ${id:0:8}"; else echo "FAIL open $mode ${id:0:8}: want $want got $got"; fail=1; fi
    done
  done
fi
(( N >= 5 )) && echo "note lanes.json is checked by the 9.5 worker-to-App test, not here"
exit $fail
