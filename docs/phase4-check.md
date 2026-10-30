# Phase 4 real check — 2026-10-30

**Step 4.6: measured before / after on real `~`. Phase 4 is NOT closed yet.** The owner still has
to look at the real home screen and confirm the Phase 4 need is met (stop-and-ask, below).
No code changed in this step. Claude ran every command below.

## Environment and privacy

- macOS, real `~` (no `--root`), real `ps`. Rust commands used the AGENTS.md PATH prefix.
- **After** = this checkout at `383537e` (Step 4.5), `cargo build --release -p multi-link`.
- **Before** = `9b1b374` (Release 1), built with `cargo build --release -p multi-link` in a detached
  `git worktree` under Claude's scratch directory, outside this checkout. This checkout's branch was
  never switched. The worktree was removed with `git worktree remove` after the check.
- Pinned `--now 2026-10-30T22:15:00Z` for every pinned run below.
- PTY driver: a 126-line Python 3 standard-library script (no extra packages) that opens a pty
  with `TERM=xterm-256color`, runs `multi-link tui --now <pinned> --since <window>`, keeps a
  minimal VT screen (cursor moves, erase, alternate screen; colours ignored), snapshots it after
  8 s, sends `q` and waits for exit. Captures and the script stayed in scratch storage.
  Emulator limits: every character counts as one column, scroll regions are ignored, and UTF-8 is
  decoded per read chunk. The captures contain no U+FFFD and their columns line up, so these limits
  did not show here.
- What is shown here: repo names, the header / axis / footer lines (counts only), line counts and
  row-type counts. Session rows, prompts, ids, step titles and paths are omitted.
- Row types were counted by a script: a repo line starts (after an optional state glyph) with a
  repo name from `overview --since all --json`; `steps` lines, `↳` hand-off lines, the axis line,
  `claude ×N` / `codex ×N` merged rows; everything else is a session row. That last rule also counts
  the `↓ N more lines` scroll hint, which appears only in the 40-row `--since all` captures (not used
  in the row-type table below; corrected by hand where quoted).

## Link counts (multi-link checkout)

Phase 4 baseline (PLAN.md, measured before 4.1): 43 Codex calls → **42 `unlinked` / 1 `heuristic` /
0 `ambiguous`**.

Re-run, unpinned (`target/release/multi-link links --since 24h --json`, 2026-10-30 22:14 UTC), counting
`links[]` whose `checkout` is exactly this checkout's path (not its `fixtures/` subfolders, not the sandbox repos):

```text
49 calls → heuristic 33, unlinked 12, ambiguous 4   (all 12 unlinked have 0 candidates)
```

This file came out byte-identical to the pinned after run below (the JSON has no `now` field and
nothing changed in that minute), so it adds no evidence of its own beyond confirming the pinned run.

Same instant for both builds (`links --since 24h --now 2026-10-30T22:15:00Z --json`, run back to back):

```text
before (9b1b374): 49 calls → unlinked 48, heuristic 1
after  (383537e): 49 calls → unlinked 12, heuristic 33, ambiguous 4
```

The 24h window moved since the baseline, so the call totals differ (43 → 49); the pinned pair is the
like-for-like comparison. Why the remaining 12 calls are unlinked was not investigated.

## Home screen captures

Runs, in order (each before / after pair back to back, all exit 0):

```text
22:16:21 before --since 2h 600x100    22:16:29 after --since 2h 600x100
22:16:37 before --since all 600x100   22:16:45 after --since all 600x100
22:16:53 before --since 2h 40x100     22:17:01 after --since 2h 40x100
22:17:09 before --since all 40x100    22:17:18 after --since all 40x100
```

The 600-row runs show the whole home list (nothing scrolls off). The 40-row runs show what one
ordinary terminal screen holds; both `--since all` 40-row captures end in a `↓ N more lines` hint
(before `↓ 224`, after `↓ 8`).

### Line counts (non-blank rows, footer excluded)

| Window | Before (9b1b374) | After (383537e) |
| --- | --- | --- |
| `--since 2h` (default) | 28 | 15 |
| `--since all` | 246 | 30 |

Each repo is followed by one blank separator row, so the list's real height (last non-blank row
before the footer, header included) is longer: 2h 31 → 18 rows, `--since all` 262 → 46 rows. At 46
rows the after `--since all` list still scrolls in a 40-row terminal.

Row types (600-row runs):

| Window, build | repo lines (expanded / collapsed) | steps | session rows | merged rows | hand-off lines | axis |
| --- | --- | --- | --- | --- | --- | --- |
| 2h, before | 4 / 0 | 4 | 16 | 1 | 2 | 0 |
| 2h, after | 2 / 2 | 2 | 3 | 2 | 2 | 1 |
| all, before | 17 / 0 | 17 | 200 | 7 | 4 | 0 |
| all, after | 2 / 15 | 2 | 3 | 4 | 2 | 1 |

### Header lines

```text
before 2h:  multi-link  4 repos  now <ts>
after  2h:  multi-link  0 waiting · 2 busy · 0 stale · 0 warnings · 4 repos · 25 sessions · 9 hand-offs
            session span · idle gaps included · UTC   …   20:15   22:15
before all: multi-link  17 repos  now <ts>
after  all: multi-link  0 waiting · 2 busy · 0 stale · 0 warnings · 17 repos · 388 sessions · 108 hand-offs
```

Footer (both builds, unchanged):
`↑↓ select · enter expand/timeline · t timeline · esc collapse · o open · f fork · q quit`.

### Repos on screen (in order; `(c)` = collapsed to one line)

```text
2h  before, 40 rows: multi-link route-impact jarednarvaez-app posting-triage
2h  after,  40 rows: multi-link route-impact jarednarvaez-app(c) posting-triage(c)
all before, 40 rows: multi-link
all after,  40 rows: multi-link route-impact jarednarvaez-app(c) repo-a(c) posting-triage(c)
                     physaflow(c) scratch-2026-10-29-338fd3(c) repo-b(c) cxtest(c) review(c)
                     osm-validate(c) scratch-2026-10-27-b43bd1(c) Tailored Resumes(c)
all after, 600 rows: the 13 above, then jarednarvaez(c) assets(c) notes(c) scratch-2026-10-14-640c90(c)
```

With `--since all`, the before screen's first 40 rows hold one repo (34 of its session rows); the
after screen's first 40 rows hold 13 of the 17 repos.

## Observations

- **Busy repos first, quiet repos one line each (U3).** In both windows the two busy repos stay
  expanded; every tier-`other` repo is collapsed (none had a `stale` session).
- **`codex ×N` (U2) plus links (U1).** Before, the 2h screen listed 12 single `? codex … unknown` rows
  (6 per busy repo). After: route-impact's 6 are one `codex ×6` row; in multi-link, `claude ×9` now
  reports 5 codex children and 1 unparented Codex row is left as a single row (under the merge
  threshold of 4). That the 5 children are exactly 5 of the 6 former rows was not checked by id.
- **Linker fix visible on screen (U1).** The multi-link hand-off line reads `→ -  unlinked` before and
  `heuristic` with a child id after; the `claude ×9` row says `0 codex children` before and
  `5 codex children` after.
- **Header (U4)** answers waiting / busy / stale / warnings before scrolling; before it showed only the
  repo count.
- **Time bars (U5)** are drawn on 3 session rows in each after capture at 100 columns; merged rows and
  collapsed repos have none, as designed.
- **Lane text clipping (4.5 review item 2):** at 100 columns, no hand-off or session text was cut in
  these captures. Longer hand-off metadata may still clip; not seen here.
- **Trade-off to judge:** a collapsed repo hides its latest hand-off line. `--since all` showed 4 hand-off
  lines before and 2 after (the other 2 belong to repos that are now collapsed); `enter` expands them.
- **Pinned-`now` artefact:** sessions still running after 22:15 show `… ahead` and a `▸` clipped bar
  end, because the pinned `now` is earlier than their last activity. Not a defect of either build.
- **Live data:** the two builds of a pair ran 8 s apart on live files, so a pair is not a frozen input,
  and the absolute counts move with what is running (a repo that turns busy expands). An earlier trial
  run was not kept and is not used here.

## Whole-phase Verify (Claude, 2026-10-30)

```text
cargo test --workspace                              → exit 0, 29 `test result: ok`, 0 failed
cargo clippy --workspace --all-targets -- -D warnings → exit 0, 0 warning/error lines
                                                      (re-run after touching crate roots, not cached)
```

## Owner check (stop-and-ask) — pending

Need: "from the home screen I can tell in a few seconds which repo needs me and what is running, the way
mockup screen 6 promises." The captures above show the screen got shorter and the header now carries
the counts; whether that meets the need is the owner's call. To check:

```sh
cargo build --release -p multi-link
target/release/multi-link tui
target/release/multi-link tui --since all
```

Owner's answer: _not yet recorded._
