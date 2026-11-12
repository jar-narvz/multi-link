# Phase 17 check: step strip, row actions and Codex-done notifications

Step 17.4, 2026-11-11. Phase 17 adds a step strip, row actions and a macOS notification when a Codex hand-off
finishes to the Claude Code pane (`mod/multi-link-pane/`), from new `multi-link pane` fields and watcher lines
(`crates/cli/src/pane.rs`). The target look is board 17 (`docs/boards/board17-pane-strip.html`), options S1 Chain,
A1 Expand in place and N2 What it said, picked by the owner; see [design](design.md) → "Phase 17 decisions".

## How the real check was run

- Release build of the Step 17.3 tree plus this step's mod fix (below), real `~`.
- A sandbox repo `~/multi-link-sandbox/repo-p17` with one PLAN.md step, `Step 1.1 — Say ok (Codex)`.
- A scratch PTY harness (never in the repo) started **two** interactive `claude --plugin-dir …/mod/multi-link-pane`
  sessions in that repo at 150 × 44 (`CLAUDE_CODE_*` cleared, session persistence forced), `target/release` first on
  `PATH`, with a logging shim for `osascript` ahead of the real one. The shim records the time, the argument count,
  the title and the body's length only (no message text), then runs `/usr/bin/osascript` with the same arguments, so
  the notification really showed.
- Pane A was told to do Step 1.1 by running one synthetic `codex exec 'Do Step 1.1 in PLAN.md: reply with just the
  word ok.'`. Pane B only watched. The harness timed both panes, counted `osascript` calls and new files in
  `~/.multi-link/notified/`, measured one watcher's CPU over 30 s with `ps`, then sent `/exit` to both.
- Claude Code 2.1.322, terminal surface.

## Results (run 4, the final one)

| Check | Result |
|---|---|
| Both watchers started with `--notify` (macOS, option on) | ✓ 2 of 2 |
| Strip, board 17 S1 | ✓ `step 1.1 Say ok (Codex)`, then `▌claude plan ✓ 1m`, ` codex implement ✓ 1m`, `▌claude review · 1m`; this session's parts marked `▌` |
| One notification per hand-off, two panes open | ✓ exactly **1** `osascript` call, **1** new marker file |
| Notification argv (H4) | ✓ 4 arguments: `-e <fixed script> <title> <body>`; title `codex finished (1.1)`; body 2 characters (`ok`) |
| Pane data first seen, from `claude` launch | A 3.64 s, B 4.27 s (run 3: 4.61 / 3.77 s); two sessions started at once |
| Warm tick CPU, one watcher, 30 s idle | **195 ms** per 3 s tick (run 3: 197 ms; 17.2's 60 s single-watcher run: 219 ms). NFR ≤ 250 ms: pass |
| Long rows cut to the pane | ✓ `✓ “Do Step 1.1 in PLAN.md. Run exactly this Bash command, then r…` ends at the pane edge |
| Empty sections | ✓ no `waiting ─` heading when nothing waits |
| Footer | ✓ `checked 0s ago` (tick lines reset it) |
| No orphan 5 s after `/exit` | ✓ 0 watchers for the sandbox repo |

Pane A after the hand-off (the README capture):

```text
multi-link · repo-p17
step 1.1 Say ok (Codex)
▌claude plan       ✓ 1m
 codex  implement  ✓ 1m
▌claude review     · 1m
recent (2h) ─
▌● this session  1.1
▌ └ codex 01a1cdad ✓
 ✓ “Do Step 1.1 in PLAN.md. Run exactly this Bash command, then r…
  └ codex 01a1cdaa ✓
checked 0s ago
```

The `“Do Step 1.1…”` row is run 3's session (same repo, same prompt). Pane B showed `this session · not seen yet`
(no prompt was sent there, so Claude Code had not saved a transcript yet), which is right.

## What the real check found and fixed

1. **The pane measured the wrong box (since Phase 16).** The mod took its width and height from `e.viewport`, which is
   the whole screen (150 columns), not the pane body (about 65). Long rows wrapped onto a second line, and the
   overflow rules never applied on a real surface. Phase 16's rows were short enough to hide it; Phase 17's labels and
   `asked` / `said` lines are not. The fix reads `e.props.bodyColumns` and `e.props.scroll.bodyRows`. The tests now
   mount with matching pane props, and a new test (screen 150 wide, pane 8 wide) fails without the fix.
2. **An old binary looped instead of saying "too old".** A 1.13 `multi-link` answers `--notify` with `unexpected
   argument`, which the mod read as a crash and retried forever. The check now matches that too (new test, fails
   without the fix). This mattered for the owner, whose installed binary was 1.13.

Claude made both edits; Codex reviewed them read-only ("Clean — no findings"). `claude plugin test`: 90 pass.

## Earlier runs

- Run 1 sent Enter at the trust prompt too early (the default answer is "No, exit"); no pane started.
- Run 2: pane B started (data 12.8 s from launch, including the prompt) with `--notify`; pane A exited at the trust
  prompt because the scripted arrow key was read as Esc. The folder is trusted from then on.
- Run 3: everything worked, but the Codex prompt named no step, so the hand-off had none: title `codex finished`, no
  strip. One `osascript` call, one marker. Long rows wrapped (finding 1 above).

## Not checked by the harness

- **Pressing a row.** Scripted keys did not reach the pane (17.0 found the same). Presses, one open row at a time,
  `Copy id` and `Open in app` are covered by the mod's test kit; the owner checks them in the desktop app below.
- **The notification as macOS draws it**, and which app it is credited to: the owner sees it.
- A narrow pane on a real surface (covered by tests now that the size is right).

## Owner check (stop and ask)

Pending.
