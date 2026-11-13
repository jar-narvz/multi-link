# Phase 19 check: multi-link in the macOS menu bar

> **Phase 20 note (2026-11-12):** the pane is retired, so it no longer races the app for notifications; the pinned owner check below moved to `docs/phase20-check.md`.

Step 19.4, 2026-11-12. Phase 19 adds a SwiftUI menu-bar app (`apps/menubar/`) that shows every repo's sessions and
hand-offs, fed by `multi-link menubar --json --watch [--notify]` (`crates/cli/src/menubar.rs`). It sends the Codex-done
notifications itself while it can. The target look is board 19 (`docs/boards/board19-menu-bar.html`), options P2 By
repo and I1 Waiting count, picked by the owner. See [design](design.md) → "Phase 19 decisions".

## How the real check was run

- `apps/menubar/build.sh` built `apps/menubar/build/multi-link.app` from commit `8544152`: ad-hoc signed with the
  designated requirement `identifier "dev.multi-link.menubar"`; `codesign --verify --strict` passed.
- The installed `~/.cargo/bin/multi-link` is 1.14 and has no `menubar` subcommand, so the app's binary-path setting
  pointed at the release build: `defaults write dev.multi-link.menubar multiLinkBinary
  /Users/jarednarvaez/multi-link/target/release/multi-link`.
- Launched with `open apps/menubar/build/multi-link.app` against the real `~`, on macOS 15.7.3 (MacBook with a
  notch). The owner looked at the screen; Claude read window positions (`CGWindowListCopyWindowInfo`), the lock and
  the processes.

## Results

| Check | Result |
|---|---|
| App and watcher start | ✓ the app ran `multi-link menubar --json --watch` (no `--notify` while it couldn't deliver yet), then switched to `--watch --notify` and held `~/.multi-link/menubar.lock` |
| Icon (board 19 I1) | ✓ `ml` with no number (nothing waiting, something running), once there was room (next row) |
| Room in the menu bar | ✗ at first: the item got a slot at x 825–860, inside the notch, and was not drawn. After the owner removed Gemini it moved to x 863 and was still hidden. macOS showed nothing left of x ≈ 898. After the owner freed one more item it moved to x 908 and showed |
| Popover (board 19 P2) | ✓ owner: "Matches the board" (repo blocks with `step` in the header, row symbols, `└` child lines, `checked Ns ago · Open TUI · Quit`) |
| One notification per hand-off | ✓ a real `codex exec` hand-off from this Claude session made one new marker in `~/.multi-link/notified/` (28 → 29) |
| Who announced it | ✗ for the goal: owner: "From Script Editor". The pane's watcher, running the installed 1.14 binary that has no lock check, claimed the marker before the app's watcher. Expected with an old binary; the lock check is in 1.16 (`crates/cli/src/pane.rs` `PaneMarkers`) and covered by a test |
| Tick CPU of `menubar --watch` | 208 / 211 ms per 3 s tick on real `~` (Step 19.1, two 30 s samples; budget ≤ 250 ms) |

## Not checked live

- **A notification credited to multi-link.** This needs the pane to run a 1.16 binary (`cargo install`, then restart
  Claude Code), so the pane leaves the hand-off to the app. Pinned for the owner, with the Phase 18 check, which needs
  the same install.
- **The notification permission prompt.** The owner saw no prompt for `dev.multi-link.menubar`. The app's check (alert
  setting enabled) said it could deliver, and it took the lock. Whether macOS actually shows its banners is unproven
  until the check above. The 19.0 spikes showed macOS can report `authorizationStatus` denied while the alert setting
  is enabled, and then drop banners; if that happens, the app's rule needs to require `authorizationStatus ==
  .authorized` as well.
- **Quitting the app or denying notifications hands delivery back to the pane.** Covered by Swift tests (lock released
  on quit, on lost permission, on watcher failure) and the Rust lock test; not watched live.
- **Opening a row from the popover, and Open TUI.** The 19.0 spike ran the same `open --no-resume` jump and Open TUI
  command from an app bundle (owner allowed the Automation prompt); the shipped app's buttons were not pressed.

## Whole-phase Verify (Claude, 2026-11-12)

- Rust: no Rust change since Step 19.1, whose Verify Claude re-ran: `TZ=UTC cargo test --workspace` 736 passed /
  0 failed / 1 ignored, clippy `-D warnings` and `cargo fmt --check` clean.
- `fixtures/expected/{menubar,open-jump,phase12,phase13,phase14}/check.sh` and `phase9/check.sh 6`: all exit 0 (re-run
  for this step).
- `swift test` in `apps/menubar`: 18 passed (Step 19.3 re-review). `./build.sh`: built and verified.
- `claude plugin test mod/multi-link-pane`: 134 pass, 0 fail (re-run for this step; the mod is unchanged).

## Owner check

Pending (pinned 2026-11-12). Install the new binary, restart Claude Code, run one Codex hand-off, and confirm the
notification comes from multi-link, not Script Editor.
