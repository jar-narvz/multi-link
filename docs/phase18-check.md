# Phase 18 check: jump to a running terminal session's tab

> **Phase 20 note (2026-11-12):** the pane's `Jump to terminal` button is gone with the pane, so its pinned owner check is dropped; the jump itself was proven from the CLI and the TUI below.

Step 18.4, 2026-11-12. Phase 18 makes `multi-link open` bring a running Claude session's own Terminal.app (or iTerm2)
tab to the front, instead of saying `unavailable` and printing a `claude --resume` that would start a second copy. The
same rule runs from the CLI, the TUI (`o`) and the Claude Code pane (`Jump to terminal`). The target look is board 18
(`docs/boards/board18-terminal-jump.html`), option J1 Jump straight, picked by the owner. See
[design](design.md) → "Phase 18 decisions".

## How the real check was run

- A release build of the Step 18.3 tree, using the real `~`. macOS 15 (Darwin 24.6.0), Terminal.app.
- A sandbox repo, `~/multi-link-sandbox/repo-p17`. `claude "Reply with just the word ok."` was started there in a new
  Terminal window, so the session has a transcript. A session that was never sent a prompt has no transcript, and no
  list shows it (see Limitations).
- Before each jump, a different Terminal window was moved to the front by AppleScript. After the jump, AppleScript read
  `{tty of selected tab of front window, frontmost}`.
- The TUI was driven by a scratch PTY harness (never in the repo) at 150 × 44, from the sandbox repo.

## Results

| Check | Command | Result |
|---|---|---|
| Resolution on real processes | `multi-link open a29299c0-… --dry-run` | ✓ `{"action":"jump","app":"terminal","tty":"/dev/ttys006","pid":92942,"cwd":"…/repo-p17"}`, exit 0 |
| Pane row | `multi-link pane --json --cwd ~/multi-link-sandbox/repo-p17 --session x` | ✓ the row has `openable: true`, `opens: "terminal"` |
| CLI jump (first sandbox session, 88044ae2-…) | `multi-link open 88044ae2-…` | ✓ `jumped to Terminal · window 2 · tab 1 (ttys006)`, exit 0; front window's selected tab `/dev/ttys006`, Terminal frontmost |
| TUI `o` | home → repo-p17 card → Enter → `o` on the session lane | ✓ footer `jumped to Terminal · window 2 · tab 1 (ttys006)`; front tab `/dev/ttys006`, Terminal frontmost; the TUI kept running (no suspend) |
| `--app-only` refuses a jump | `multi-link open a29299c0 --app-only` | ✓ `no app link available`, exit 3 |
| `--no-resume` accepts a jump | `multi-link open a29299c0 --no-resume --dry-run` | ✓ the same `jump` action |
| Tab gone: the real script | the committed `terminal.applescript` run by `osascript` with `/dev/ttys999` | ✓ prints `not found`, exit 0. The CLI maps that answer to `cannot jump: its tab is gone` (unit test with a fake runner) |
| Watcher tick with a running Terminal session present (one `ps` per tick) | 30 s of `pane --watch` CPU from `ps` | 226 ms per 3 s tick (budget ≤ 250 ms); first line after 2.70 s |

## Not checked live

- **The Automation prompt and -1743.** The caller was already allowed to control Terminal, so macOS showed no prompt.
  A denied jump (error -1743) was not seen. Its message, `cannot jump: blocked by macOS. Allow it in System Settings →
  Privacy & Security → Automation`, is covered by a fake-runner test only.
- **A real tab-gone process.** No process was found that is alive on a tty with no Terminal tab. Only the script's
  `not found` answer was checked live (above).
- **tmux / screen / zellij, Ghostty, VS Code.** None was used for a session here (tmux is not installed). Those answers
  (`inside_multiplexer`, `terminal_not_supported` with the app name) come from `fixtures/sessions-phase18` and its
  `fixture-ps.txt` (`fixtures/expected/open-jump/check.sh`).
- **iTerm2.** It is not installed. Its script (`application id "com.googlecode.iterm2"`) is covered by tests only.
- **The pane's button in the desktop app.** It is covered by the mod's test kit (`claude plugin test`, 134 pass). The
  owner check is below.

## Whole-phase Verify (Claude, 2026-11-12)

- `TZ=UTC cargo test --workspace`: 730 passed, 0 failed, 1 ignored (an existing ignored test).
- `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check`: clean.
- `fixtures/expected/{open-jump,phase12,phase13,phase14}/check.sh` and `phase9/check.sh 6`: all exit 0.
- `claude plugin validate mod/multi-link-pane`: `✔ Validation passed`, plus the existing non-fatal catalog notice
  (no license or icon). `claude plugin test mod/multi-link-pane`: 134 pass, 0 fail.

## Owner check

Pending: from the desktop app's pane, press `Jump to terminal` on the repo-p17 Terminal session's row.
