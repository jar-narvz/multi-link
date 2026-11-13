# Phase 20 check: the Claude Code pane is retired

Step 20.3, 2026-11-12 to 2026-11-13. Phase 20 removes the Claude Code pane and `multi-link pane` (Release 1.17). The
menu-bar app (Phase 19) is now the only thing that shows sessions outside the terminal and the only notifier.
See [design](design.md) → "Phase 20 decisions" and PLAN.md Phase 20 (R1–R6).

## How the real check was run

- R6, the pane turned off: the owner removed `CLAUDE_CODE_PLUGIN_DIRS` from `~/.claude/settings.json` with the
  command Claude gave (backup `~/.claude/settings.json.bak-pane`).
- The owner installed 1.17 (`cargo install --path /Users/jarednarvaez/multi-link/crates/cli --locked`).
  `~/.cargo/bin/multi-link pane` now answers `error: unrecognized subcommand 'pane'`.
- The owner did not restart Claude Code. Two `multi-link pane --json --watch … --notify` processes started before the
  setting was removed were still running the old 1.16 binary from memory (one for this repo, one for another repo).
  Claude stopped them (`pkill -f 'multi-link pane --json'`); none came back, and a restarted one could not run,
  because the installed binary has no `pane`.
- The running menu-bar app was a pre-20.2 build. Asked to quit, it did not (its main thread was stuck, the bug 20.2
  fixes), so Claude ended its process. `apps/menubar/build.sh` had built the bundle from the tree committed as
  `c5b1e5e`; the binary-path setting was removed (`defaults delete dev.multi-link.menubar multiLinkBinary`) and the
  app opened with `open apps/menubar/build/multi-link.app`. It started
  `~/.cargo/bin/multi-link menubar --json --watch --notify` and was the only multi-link watcher left (`pgrep -fl
  multi-link`).
- Hand-offs: `codex exec -s read-only` from this Claude session in this repo, each prompt naming Step 20.3 and asking
  only for a fixed reply. The owner watched the screen.

## Results

| Check | Result |
|---|---|
| No pane process (R6) | ✓ after Claude stopped the two old watchers; `pgrep` showed only the app and its watcher for the rest of the check |
| App on 1.17 with the default lookup | ✓ `~/.cargo/bin/multi-link menubar --json --watch --notify`, no saved path |
| Hand-off 1 | ✗ one new marker in `~/.multi-link/notified/` (37 → 38), so the app's watcher claimed it, but the owner saw no banner |
| Hand-off 2 | ✗ one new marker (38 → 39); owner: "i didnt see anything" |
| Why | macOS's saved setting for `dev.multi-link.menubar` in `~/Library/Preferences/com.apple.ncprefs.plist` was flags `0x1080200e`; Script Editor's, whose banners show, is `0x0280200e`. The app's lacked bit 25 (`0x2000000`). The owner turned on System Settings → Notifications → multi-link → Allow Notifications, and the flags became `0x1280200e` (bit 25 set). The flag meanings are undocumented; this is read from the before/after values. The app's rule (alert setting enabled) said it could deliver the whole time |
| Hand-off 3 | ✓ one new marker (39 → 40); owner: "Yes, from multi-link". **A notification credited to multi-link**, closing the pinned Phase 19 check |
| Single-instance guard, stuck-watcher fix, one-line header | Swift tests only (Step 20.2); not exercised live beyond the app and its watcher staying up and claiming each hand-off for the length of the check |

Two hand-offs were announced to nobody: the app claimed their markers while macOS dropped the banners without an error.
This is the case the 19.0 spikes predicted (Phase 19 check, "Not checked live"). Step 20.4 makes the app's rule
require `authorizationStatus == .authorized` too, so a watcher that cannot really deliver never claims a marker.

## Whole-phase Verify (Claude, 2026-11-12)

- `TZ=UTC cargo test --workspace`: 731 passed / 0 failed / 1 ignored. `cargo clippy --workspace --all-targets -- -D
  warnings` and `cargo fmt --check`: clean.
- `fixtures/expected/{menubar,open-jump,phase12,phase13,phase14}/check.sh` and `phase9/check.sh 6`: all exit 0.
- `apps/menubar`: `swift build -c release` complete, `swift test` 22 passed, `./build.sh` "Built and verified
  build/multi-link.app".

## Owner check

Done (2026-11-13): after Allow Notifications was turned on, a real Codex hand-off showed a banner credited to
multi-link. The pinned Phase 18 pane check is dropped: its button is gone, and the jump was proven from the CLI and
the TUI (`docs/phase18-check.md`).
