# Phase 16 check: a multi-link pane inside Claude Code

Step 16.4, 2026-11-11. Phase 16 adds `multi-link pane --json [--watch]` (`crates/cli/src/pane.rs`) and a Claude Code
mod that draws its output beside the chat (`mod/multi-link-pane/`). The target look is board 16 option C, picked by the
owner; see [design](design.md) → "Phase 16 decisions". The answer keys are `fixtures/expected/pane/*.json`.

## How the real check was run

- Release build of `93194cc` (`cargo build --release --bin multi-link`), real `~`, this repo.
- A scratch PTY harness (never in the repo) started `claude --plugin-dir /Users/jarednarvaez/multi-link/mod/multi-link-pane`
  at 150 × 44 in this repo, with `target/release` first on `PATH`. A reader thread kept draining output into a small
  screen model (cursor moves, erase, scroll). It timed the pane's first text and first data, ran `ps` for
  `pane --json --watch` after start and 5 s after `/exit`, and saved the screen.
- Claude Code 2.1.322, terminal surface. Run 1 inherited this coordinator's `CLAUDE_CODE_CHILD_SESSION` marker, so
  Claude Code saved no transcript and the pane rightly showed `this session · not seen yet`. Runs 2–4 cleared the
  `CLAUDE_CODE_*` variables and set `CLAUDE_CODE_FORCE_SESSION_PERSISTENCE=1`.

## Results

| Check | Result |
|---|---|
| Pane opens beside the chat, board 16 C layout | ✓ docked right; header `multi-link · multi-link`, `working` / `waiting` / `recent (2h)` rules, this session `▌`, Codex runs nested with `└` |
| This session found (run 2, after one prompt) | ✓ `▌● this session` under `recent (2h)` (idle after its reply) |
| Codex hand-offs nested | ✓ the coordinator session's four Phase 16 Codex runs under it, all `✓` (exact links, so no `likely`) |
| Pane text first seen, from `claude` launch | 0.94 / 0.83 / 0.84 s (`reading sessions…`) |
| Pane data first seen, from `claude` launch | 2.81 / 2.90 / 2.53 s, median **2.81 s** |
| Pane data, from the pane opening | 1.87 / 2.07 / 1.69 s, median **1.87 s** (NFR ≤ 2.5 s after the session starts: pass) |
| Watcher alone, first line (3 fresh processes) | 2012 / 1569 / 1640 ms, median 1640 ms |
| Warm refresh cost | 1.40 s CPU over 10 ticks = **140 ms CPU per tick** (NFR ≤ 250 ms: pass); cold pass 10.4 s CPU over all cores |
| One watcher per session | ✓ one `multi-link pane --json --watch --cwd … --session <id>`, parent `claude` |
| No orphan 5 s after `/exit` | ✓ `ps` → none, in all five runs |
| Old `multi-link` on `PATH` (installed 1.12, no `pane`) | ✓ `This multi-link is too old for the pane. Update it.`, no watcher left running |

How the NFRs were read: the 2.5 s budget counts from the session starting. Claude Code itself takes about 0.85 s to
start before the mod's `session.start` runs and opens the pane, so the honest figures are both above: 2.81 s from
launching `claude`, 1.87 s from the pane opening. The 140 ms is CPU time per 3 s tick (a 30 s window after the first
line), not wall time per refresh.

## Desktop app (owner, 2026-11-11)

The owner installed the 1.13 binary (`cargo install --path crates/cli --locked`), set `CLAUDE_CODE_PLUGIN_DIRS` to the
mod folder in `~/.claude/settings.json`, restarted the Claude desktop app and opened this repo's coordinator session.
Their screenshot (3:44 PM) shows the pane docked to the right of the chat titled `multi-link`, drawing:

```text
multi-link · multi-link
working ─
waiting ─
recent (2h) ─
▌● this session  16.3
▌ └ codex 01a1cc93 ✓
 ✓ "Reply with just the word ok."
updated 12s ago
```

So G5 (a) and (b) hold in the desktop app too: the desktop session's id matched its transcript (this session is
found and marked), the app draws a mod pane, and the app's `PATH` reaches `~/.cargo/bin/multi-link` (no
`isn't on PATH`). The `"Reply with just the word ok."` row is one of this check's own PTY test sessions. The
coordinator shows under `recent` because it was idle, waiting for the owner, when the screenshot was taken.

## Not checked

- Narrow panes on a real surface: overflow is covered by `claude plugin test` (20×6, 30×12, working-only), not seen
  live.
- A Codex link matched by time (`likely`) and an ambiguous hand-off live: covered by the answer keys and mod tests;
  none were in this repo's window.

## Observations

- Empty sections keep their rule (`waiting ─` with nothing under it). Board 16 C showed every section with rows, so
  this was not specified; it reads as "nothing waiting". Left as is.
- The `updated Ns ago` footer counts from the last line the watcher printed, which is only on a change or every 30 s,
  so it can read up to about 30 s on a quiet repo.

## Owner check (stop and ask)

Pending.
