# Phase 15 check: a fast first load with a school of fish

Step 15.4, 2026-11-11. Phase 15 makes the cold first load fast (the transcript parse runs on every core, F1) and shows
a loading screen while it runs: board 15 option E, two schools of ASCII fish in Claude orange and Codex blue, with a
progress bar and a real file counter (F4–F8); see
[design](design.md) → "Phase 15 decisions". The render answer keys are `fixtures/expected/tui/loading-*`.

## How the real check was run

Nothing real is committed (W10): reports, ids and terminal output stayed in scratch storage; this page gives counts
and timings only. All runs used the release build of `5a63110` (Step 15.3) on the owner's real `~/.claude` and
`~/.codex`, between 10:04 and 10:11 PST, while other Claude sessions were running on the same machine.

- **(a) Cold overview.** `/usr/bin/time -p target/release/multi-link overview --json`, 3 fresh processes.
- **(b) Worker first report, (d) warm refresh.** A scratch program (path dependencies on `multi-link-core`,
  `-plan`, `-model`; not in the repo) repeats the TUI worker's `refresh_with_live` without the graph: the overview over
  the 2 h window, the all-time pass, the step maps, then `Scanner::live` with `--live`. One fresh process per (b) run;
  for (d) one process runs 11 passes back to back on one `Scanner` and the last 10 are the warm passes.
- **(c) First home frame.** A scratch PTY harness starts the release `multi-link tui` at 76×22 and times, from the
  fork, when `ready ✓  opening home` and then the home footer (or the live grid header with `--live`) appear on its
  screen model. A reader thread drains the PTY the whole time, so the TUI never blocks on a full buffer.
- **PTY smoke.** The same harness, checks listed below. "Terminal restored" means the output's last
  `ESC[?1049h` (enter alternate screen) is followed by `ESC[?1049l` (leave) and `ESC[?25h` (show cursor), and after
  the exit the PTY's line discipline has echo, canonical mode and signals back on (raw mode undone).

The loading screen counted **1069** files to read on this machine (1049 in Step 15.3's run earlier the same day).

## Timings (budgets from F3)

Two series were run. The first ran while the machine's 1-minute load average was 12–17 (other sessions building);
the second at 4.7–6.2. The second is the recorded result; the first is shown because one of its runs missed (b).

| Budget | Series 2 (recorded) | Series 1 (load 12–17) | Step 15.1 | Baseline (15.0) |
|---|---|---|---|---|
| (a) cold `overview --json` ≤ 2.0 s | 1.27 / 1.16 / 1.16 s → **1.16 s** | 1.54 / 1.18 / 1.13 s → 1.18 s | 0.97 s | 7.99 s |
| (b) first report, no `--live` ≤ 2.0 s | 1446 / 1598 / 1525 ms → **1525 ms** | 1531 / 1641 / 1657 ms → 1641 ms | 1397 ms | 8624 ms |
| (b) first report, `--live` ≤ 2.0 s | 1613 / 1581 / 1629 ms → **1613 ms** | 1716 / 1853 / 2212 ms → 1853 ms | 1408 ms | 8514 ms |
| (d) warm, median of 10, no `--live` | **130 ms** (126–132) | 130 ms (124–308) | 110 ms | — |
| (d) warm, median of 10, `--live` | **194 ms** (187–216) | 199 ms (192–229) | 169 ms | — |

- (a) user time 9.25–9.53 s, sys 1.18–1.26 s in series 2: the parse runs on all cores (wall 1.16 s).
- (b) parts in series 2 (overview + all-time + live): 1006+440, 1157+441, 1082+443 ms without `--live`;
  1098+439+75, 1058+444+79, 1113+440+77 ms with it. The all-time pass (~440 ms) is mostly serial `git rev-parse` per
  folder (Step 15.1's note), not parsing.
- (d) 14.4's warm refresh with the grid open was 191–212 ms, so the 20 % bound is 229–254 ms; the `--live` median
  of 194 ms is inside it.
- **All four budgets pass in series 2.** In series 1 one `--live` run took 2212 ms (over 2.0 s) at load 17; its
  median (1853 ms) still passes. Both series are 0.1–0.2 s slower than Step 15.1's numbers (load ≈ 3.7 then); the
  file count also grew, and the cause of the difference was not isolated.

### (c) First home frame ≤ 2.25 s (real TUI in a PTY, 3 fresh processes each)

| Start | `ready ✓` on screen | First home frame |
|---|---|---|
| `multi-link tui` | 1.546 / 1.680 / 1.720 s | 1.821 / 1.952 / 1.989 s → **1.952 s** |
| `multi-link tui --live` | 1.702 / 1.811 / 1.667 s | 1.969 / 2.086 / 1.939 s → **1.969 s** |

These include process start, terminal setup, the 250 ms fade and the next 50 ms frame. F3's formula
(max(first report, 1.0 s) + 0.25 s) on series 2's (b) medians gives 1.775 / 1.863 s; the real TUI adds about 0.1–0.2 s
on top of the scratch program (channel, graph, drawing). An earlier run of the same harness (without `--live`) gave
1.905 / 1.963 / 1.940 s.

## PTY smoke (76×22 start, real `~`)

| Check | Result |
|---|---|
| loading screen shows (title and hint on screen) | ok, 0.007 s after start |
| counter rises, total fixed | ok, `36 / 1069` → `176 / 1069` in the first 0.25 s (5 distinct values) |
| resize to 60×16: art and title stay, title on row H−7 = 9 | ok |
| resize to 30×6: no art or title, counter line and hint | ok |
| resize back to 76×22, home drawn | ok |
| `q` from home: exit 0, terminal restored | ok, 6 ms |
| `q` during loading (0.07 s, counter on screen): exit 0, restored, home never drawn | ok, 3 ms |
| Ctrl-C during loading: exit 0, restored, home never drawn | ok, 2 ms |
| `q` during the fade (sent as soon as `ready ✓` showed, 1.64 s): exit 0, restored, home never drawn | ok, 4 ms |
| Ctrl-C during the fade (1.58 s): exit 0, restored, home never drawn | ok, 4 ms |
| space at 0.16 s skips to `loading sessions… · q quit`, then home | ok, footer 8 ms later, home at 1.62 s |
| `q` after the skip: exit 0, restored | ok |
| `tui --live`: loading screen, then the live grid | ok, grid at 1.95 s |
| `q` from the live grid: exit 0, restored | ok |

14 of 14 ok (two full runs; the table is the second). "During the fade" is inferred from timing: `ready ✓` appears
when the worker has sent its first report, which here is after 1.0 s, so the 250 ms fade starts then (F5); the
harness did not read the fade's colours.

## Styled render vs board 15 E

`docs/img/loading-phase15-fixture.png` is the fixture frame (`drawC(4.0)`, counter `reading transcripts  612 / 871`)
drawn by `crates/tui/examples/home_html.rs --loading 4.0 … --size 76x22` and screenshot by headless Chrome at 2x.
It was compared by eye with a scratch copy of `docs/boards/board15-loading.html` paused at the same frame (option E
only): the same fish in the same cells, the same six pale `*` meets, the same speck positions, the title on the
same row. Differences, all from the rules: the board's title and bar are continuous orange→blue gradients, the render
uses `claude` for `m u l t i`, `codex` for `l i n k`, and the bar's two halves with to-do cells in `border` (F6);
the board's counter is simulated (`ready ✓  opening home`), the render's is the fixture's. The glyph-for-glyph check
(225 of 225 art cells) and the colour-family check were done in Steps 15.0 and 15.2 and the goldens still pass.

## Not checked on real data

- The 256-colour and `NO_COLOR` screens, and the fade's two colour steps, are checked against the goldens and in
  app tests only; the PTY run used truecolor and read text, not colours.
- Sizes other than 76×22, 60×16 and 30×6 (including below 30×6) are covered by the goldens and bounded-buffer
  tests only.

## Owner check (stop and ask)

Pending.
