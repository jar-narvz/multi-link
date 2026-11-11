# Phase 14 check: the live grid

Step 14.4, 2026-11-10. Phase 14 adds `multi-link live` and the live grid screen (`l`, `tui --live`): one card per
working, waiting or stale session, a Claude card paired with the Codex runs it started, under a feed of hand-offs;
see [design](design.md#phase-14-decisions-live-grid-what-is-each-agent-doing-now-step-140). The fixture key is
`fixtures/expected/phase14/live.json`; the hand-drawn render goldens are `fixtures/expected/tui/live-phase14*`.

## How the real check was run

Nothing real is committed (W10): reports, renders, ids and terminal output stayed in scratch storage; this page gives
counts and timings only. All runs used the release build of `4cd6db1` (Step 14.3) on the owner's real `~/.claude` and
`~/.codex`, between 18:16 and 18:31 PST.

- **Timing.** A scratch program linked against `multi-link-core` and `multi-link-plan` repeats the TUI worker's
  refresh with the grid open (`refresh_with_live` in `crates/cli/src/tui.rs` with no graph): overview over the 2 h
  window, the all-time pass, the step maps, then `Scanner::live`. One `Scanner` is kept across passes, as in the
  worker. Pass 1 is cold; 10 warm passes follow 2 s apart (the TUI's cadence); then the program waits until a card
  session's main transcript grows and runs one more pass. Each pass is timed as a whole and by part.
- **Counts** come from the same passes and from `multi-link live --json`.
- **PTY smoke.** A scratch harness ran the release `multi-link tui` in a 120×40 pseudo-terminal: wait for the first
  real frame, `l`, `↓ → ↑ ←`, `esc`, `l`, resize to 100×40, 60×20 and back to 120×40, `q`; then `tui --live`: `↓`,
  `esc`, `q`. Screen checks read the output with escape codes removed. A second run resized after each move to force a
  full frame and read the `selected` line.

## Refresh time with the grid open (threshold 2 s per pass)

Two runs of the timing program; the second split each pass into parts.

| Pass | Run 1 | Run 2 | Run 2 parts: overview · all-time · live |
|---|---:|---:|---|
| cold (first pass) | 9354 ms | 9156 ms | 8607 · 447 · 102 ms |
| warm, median of 10 | 191 ms (174–218) | 212 ms (187–248) | 62–101 · 55–63 · 64–87 ms |
| after a card's transcript grew | 197 ms | 205 ms | 73 · 60 · 72 ms |

- **Warm and changed-file passes meet the threshold** by a wide margin. In both runs the grown file was a busy card's
  main transcript (+493 and +6638 bytes); L9's cache re-reads that one session, and the pass stayed at the warm level.
- **The cold pass does not meet it.** 8.6 s of the 9.2 s is the overview's first full parse of every transcript in
  the window, which predates Phase 14 (6.4 s at Step 8.5, on less data). The grid's own share of the cold pass is
  102 ms. The TUI shows `loading sessions…` until that first pass ends; the grid adds no visible wait.
- The step maps (`started_steps`, `codex_steps`) take 0–1 ms.

## Cards on real data

| When | Cards | Waiting | Working now | Stale | Codex cards | Groups with a child | Idle sessions (repos) | Feed rows |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| timing run 1, passes 1–5 | 2 | 0 | 2 | 0 | 0 | 0 | 14 (2) | 4 |
| timing run 1, passes 6–11 | 1 | 0 | 1 | 0 | 0 | 0 | 15 (3) | 4 |
| timing run 2, all passes | 2 | 0 | 2 | 0 | 0 | 0 | 14 (2) | 4 |
| `multi-link live --json`, 18:30 | 2 | 0 | 2 | 0 | 0 | 0 | 14 (2) | 4 |

Both cards were Claude sessions (this step's own session and one other), each a lone group. One of them moved
between working and idle during run 1 as its last burst aged out. No Codex run was working, waiting or stale during
the check, so pairing, `──▸` rows, Codex cards and waiting/stale cards were not seen on real data; they are covered by
the fixture key and goldens only.

## PTY smoke

| Check | Result |
|---|---|
| home footer shows `l live` | ok |
| `l` opens the grid (`multi-link › live` header) | ok |
| `↓ → ↑ ←` keep the grid running | ok |
| `esc` returns to home | ok |
| `l` opens it again | ok |
| resize 100×40, 60×20, 120×40: each redraws | ok |
| `q` exits, status 0 | ok |
| `tui --live` starts on the grid | ok |
| `esc` from `--live` goes home | ok |
| `q` exits, status 0 | ok |

The forced-full-frame run had one card on screen (the other session was idle by then), so `↓`/`↑` kept the
selection on it, as L2 says. Moving between groups and across a two-child row is covered by Step 14.3's app tests on
the fixture, not by real data.

## Image

`docs/img/live-phase14-fixture.png`: the fixture report (`fixtures/expected/phase14/live.json`) drawn at 120×40 by
`crates/tui/examples/home_html.rs --live-grid … --offset -8 --size 120x40`, screenshot by headless Chrome at 2x.

## Owner check (stop and ask)

Done 2026-11-11: the owner installed the build (`cargo install --path …/crates/cli --locked`), opened `multi-link tui`,
pressed `l` and answered "look good". The cold first pass (9.2–9.4 s, above) was disclosed before the check.
