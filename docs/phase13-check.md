# Phase 13 check: known folders and a key that is never lost

Step 13.3, 2026-11-10. Phase 13 reads a literal `cd` before a launch as the run's folder (C1–C2), reads explicit
resume ids that come after options (C3), and reads keys with crossterm `use-dev-tty` (D1); see
[design](design.md#phase-13-decisions-known-folders-and-a-key-that-is-never-lost-step-130). The fixture keys are in `fixtures/expected/phase13/` and the amended Phase 12
key; their results are in [output-schema](output-schema.md#phase-13-fixture-results-step-130).

## How the real check was run

Nothing real is committed (W10): link files, ids, paths and commands stayed in scratch storage; this page gives
counts only.

- **Before and after.** The release build of `8f643fd` (Step 13.2, before C1–C3) and the release build of `46b4a36`
  (Step 13.1) each wrote `multi-link links --since all --json` over the owner's real `~/.claude` and `~/.codex`, eight
  minutes apart. Links were paired by (parent session, time, tool call).
- **Masked check.** For every exact launch link of the new build, a scratch copy of its checkout's transcripts was made
  with the observed child id removed (stdout and redirect logs), and the resolver was run on the copy. The answer is
  right when the time match picks the child the id named.
- **Resize check.** `python3 -I tools/resize-key-check.py target/release/multi-link 5` (committed; synthetic fixture).
- **Real TUI smoke.** A scratch harness ran the release `multi-link tui` on real data in a 120×40 pseudo-terminal,
  pressed 5 keys and resized, then sent one `q`.

## Real links, before → after

240 links before, 241 after (1 new hand-off was recorded between the runs: an exact `resume` from a redirect log).

| Change | Count |
|---|---:|
| `exec` unlinked → heuristic, no `launch_dir` (the `cd` named the session's own folder) | 32 |
| `exec` unlinked → heuristic with `launch_dir` (a folder other than the session's) | 1 |
| `exec` unlinked → heuristic, `nearest_run` | 1 |
| `resume --last` unlinked → heuristic, `resume_turn` | 3 |
| ambiguous → heuristic, `nearest_run` (same hand-off) | 1 |
| exact `exec` / `stdout` → exact `resume` / `resume_arg`, same child | 1 |
| unchanged | 201 |

Totals: unlinked 49 → 12, exact 58 → 59 (the new hand-off), heuristic 132 → 170, ambiguous 1 → 0. No link was lost
and no child changed. Every one of the 37 newly linked hand-offs has a `cd` before its codex call in the real command;
the 3 resumes are `codex exec resume --last` after a `cd`.

- **The last ambiguous hand-off** had runs 0.9 s and 7.9 s after it. The 7.9 s run is the child (1.6 s gap) of a
  hand-off that C1 newly linked, so K1 gave it to that hand-off and the 0.9 s run became the nearest run.
- **The exact link that changed form** is `codex exec --skip-git-repo-check resume <id>`: before C3 it was read as a
  launch whose child was seen on stdout; now it is a resume of the id it names. Same child either way.

## Masked check (32 exact launch links, id hidden)

| Outcome | Count |
|---|---:|
| right, heuristic | 26 |
| right, heuristic `nearest_run` | 3 |
| wrong | 0 |
| no answer (unlinked) | 1 |
| still exact (other evidence kept) | 2 |

The unlinked one is the only exact launch link with a `cd` before it; its child started before the hand-off, so no
time rule can pick it. So `cd` resolution itself has no hidden-id test on real data. What supports it: each folder
it resolved had exactly one run in the window. The 2 links that stayed unlinked at the Phase 12 check (another exact
launch had observed the same child) are now right: that other call was the `--skip-git-repo-check resume <id>` form
above, which is now a resume and no longer claims the child.

## Resize and keys

- **Resize check:** 5 of 5 rounds pass on the final release build (the TUI exits on a single `q` sent with a resize).
  The `8f643fd^` build (crossterm without `use-dev-tty`) failed 5 of 5 at Step 13.2, each time exiting only on a
  second `q`.
- **Real TUI smoke:** first frame 2975 bytes, redraw after 5 keys and a resize 10709 bytes, exit status 0 on one `q`.

## Differences from the plan

- **More hand-offs linked than predicted.** The plan predicted 29 `exec` and 2 `resume` hand-offs would link; 34 and
  3 did. The planning scan matched `cd` only at the start of a command or after `&&`/`;`, and missed `cd` after a
  newline or inside a heredoc script; the data also grew between the scan and the check.
- **Phase 12 key amended.** Phase 12 case (k) runs `cd /home/dev/repo-p12k/sub && codex exec …`; C1 resolves it, so its
  link gains `launch_dir` (still unlinked: no run in that folder). The 13.0 claim that no other committed key had a
  `cd` before a codex call was wrong.
- **Fix round 1 of 13.1** made a single `&` a word boundary but not a C1 operator, made the builtin deny-list apply
  only after a C1 `cd` (so `command -v codex && codex exec` still links), and stopped reading interactive
  `codex resume --last` as R7.
- **Counts at 13.1.** The 13.1 PROGRESS entry said 31 and unlinked 48; recounting the saved link files gives 32 and
  49. The figures on this page are from those files.
