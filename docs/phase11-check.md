# Phase 11 check: where is now?

Step 11.3, 2026-11-10. Phase 11 adds a now line to the repo view (N0–N4) and live boxes to the step graph (L1–L7);
see [design](design.md#phase-11-decisions-where-is-now-steps-111112). The target is board 11 (boards 11a–11i on the
design canvas, approved by the owner on 2026-11-10).

## How the real check was run

Nothing real is committed (W10): frames, ids and paths stayed in scratch storage; this page only summarises them.

- **Real TUI, not a render.** A scratch Python harness ran the release `multi-link tui` in a 120×40 pseudo-terminal
  with `COLORTERM=truecolor`, decoded its output into a cell grid, and every 3 s saved the screen text plus every
  cell drawn in the now colour (`#E58AE5`). It opened the repo from home with ↓/enter, then pressed `s` for the step
  graph, and stepped back to the repo view every third frame.
- **A real hand-off cycle.** In a throwaway sandbox repo outside this one, a `claude -p` session with a synthetic
  one-step plan paused, ran `codex exec "Read AGENTS.md, then do Step 1.2 in PLAN.md."`, paused to review, ran
  `codex exec resume --last "Step 1.2 fix round 1: …"`, paused again and ended. The owner's user-level hooks
  recorded it as for any session.

## What the frames showed (step 1.2, 20 graph frames from 03:34:51 to 03:36:07)

| Phase of the cycle | Live box (heavy border, `● now`) | Matches |
|---|---|---|
| Codex implementing (6 frames) | `codex · implement`; `claude · plan` dimmed | 11d |
| Codex done, next Claude part not yet cut (1 frame) | `claude · plan` (its session's newest part, session working) | L1 |
| Claude reviewing (2 frames) | `claude · review`; plan and implement dimmed | L1, 11f's Claude case |
| Fix round via `resume --last` (6 frames) | `codex · fix round 1`; the earlier implement window of the same Codex session is not live | L1 (superseded window) |
| Claude's second review (3 frames) | the second `claude · review` | L1 |
| Session ended (2 frames) | none; no cell in the now colour; boxes as before Phase 11 | L3, N4 |

The now-colour cells of a live frame were exactly the five `● now` badge cells; every live border was heavy and in
its harness colour. A finished step (step 1.1 of an earlier run) showed no live box at all, and its repo view drew
the line with no bar reaching it, as board 11e.

**Repo view, real data.** Both the sandbox repo and this repo drew `▼` on the axis rule and `┃` from the grid row
under it down every lane row at the last axis cell (column 117 at 120 columns), with the `<clock> now` label in
magenta; bar rows kept their bar.

## Differences from the boards

- **L3, harness word kept bright.** Board 11d dims the harness word of finished parts. Phase 11 dims only borders
  and `●`, because the dim colours on the selected background are 3.06:1 (claude) and 3.61:1 (codex), under the
  4.5:1 text rule (Codex critique 9, PLAN.md Review log). The finished parts still recede clearly.
- **A working bar can stop one cell short of the line.** A bar ends at the session's last recorded activity; while
  Codex runs, that is a few seconds old, and at the fitted zoom one cell spans several seconds, so the bar can end
  in the cell before `┃` (seen at 03:35:11). Board 11b draws it touching. Not changed: the line marks now and the bar
  marks recorded work, and stretching the bar would claim work that was not recorded.
- **Off-screen live part (11g) and the 80-column list (11i)** did not occur on real data (five parts fit); they are
  covered by the fixture tests in `crates/tui/tests/graph.rs`.

## Images

`docs/img/repo-phase6-fixture.png` (re-shot: now line) and `docs/img/step-live-fixture.png` (new) are renders of
synthetic fixtures through the `home_html` example and local headless Chrome at 2× (2000×1360), as in
[phase6-check](phase6-check.md#how-the-images-were-made):

```sh
B=target/release/multi-link
F=(--root fixtures/sessions-phase6 --now 2026-11-03T06:25:00Z --since 2h --json)   # zsh: an array
$B overview $F > p6-overview.json
$B links    $F > p6-links.json
cargo run -p multi-link-tui --example home_html -- p6-overview.json p6-links.json p6-repo.html --offset -8 --repo repo-r
cargo run -p multi-link-tui --example home_html -- --graph fixtures/expected/phase8/graph-1.5.json step-live.html \
    --offset -8 --selected 2 --live 2
```

The text of both HTML pages equals `fixtures/expected/tui/repo-phase6.txt` and `step-live.txt` (40 of 40 lines each).
`--live 2` marks the part the `step_live_matches_snapshot` test marks; in the TUI the CLI worker computes it (L7).
