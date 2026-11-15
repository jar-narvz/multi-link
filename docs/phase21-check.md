# Phase 21 check: one download, one setup window, a logo

Step 21.5, 2026-11-14. Phase 21 ships multi-link as `multi-link-1.18-arm64.dmg` (Release 1.18): the menu-bar app
with the CLI inside it, a setup window and icon C. See [design](design.md) → "Phase 21 decisions" and PLAN.md
Phase 21 (E1–E7). This file has two parts: what Claude checked on this Mac without the owner, and the owner checklist
for the parts that need the owner's password, clicks in System Settings, or a log out.

## Release version

The app said `1.0` (`CFBundleShortVersionString`) while the project's releases are numbered 1.x in the README and
PLAN.md (this phase is Release 1.18). The crate is `0.1.0` and the CLI has no `--version`, so the release label is
the only version users see. `apps/menubar/Info.plist` is now `1.18` (both `CFBundleShortVersionString` and
`CFBundleVersion`), so the DMG, the GitHub release and the README use one number. PLAN.md Review log has the Files
amendment.

## Checked by Claude (2026-11-14)

Check builds use the never-seen bundle id `dev.multi-link.check21` (it was not in `com.apple.ncprefs`, which held
`dev.multi-link.menubar`, `spike19`, `spike19b` and `spike21a`). `~/multi-link-sandbox/check21/check-dmg.sh <id>
<version> <dir>` (outside the repo, not shipped) copies `release.sh`'s app, sets the id and version, re-signs it
inside out like `build.sh`, and packages it like `release.sh`: v1 is `1.18`, v2 (the upgrade) is `1.18.1`.

| Check | Command | Observed |
| --- | --- | --- |
| Release DMG | `apps/menubar/release.sh` | `hdiutil: verify: checksum … is VALID`; prints `…/apps/menubar/build/multi-link-1.18-arm64.dmg`; 2,900,741 bytes, sha256 `0a5b1dce…b42e8`. Both executables `Mach-O 64-bit executable arm64`; app version `1.18`. |
| Check DMGs | `check-dmg.sh dev.multi-link.check21 1.18 …/v1`, then `1.18.1 …/v2` | Both: `designated => identifier "dev.multi-link.check21"`, id and version read back from the mounted copy; 2,900,761 and 2,900,766 bytes. |
| Local server | `python3 -m http.server 8765 --bind 127.0.0.1 --directory ~/multi-link-sandbox/check21`, then `curl -sI` each DMG | `HTTP/1.0 200 OK` with the right `Content-Length` for both; a `curl` download of v1 is byte-identical (`cmp`). Server stopped after. `curl` sets no quarantine, so the Safari download is the owner's (item 3). |
| Mounted copy says temporary | `hdiutil attach -readonly -nobrowse` (mounts at `/Volumes/multi-link 1.18`), then the bundled CLI's `setup --json` | exit 1; `cli: temporary`, `hooks: fixable`, `app: elsewhere`; `binary` is the `/Volumes/…` path. |
| Mounted copy refuses fixes | `CLAUDE_CONFIG_DIR=<scratch copy> …/multi-link setup --yes` | `move multi-link to Applications first`, exit 1; the scratch settings file unchanged (`cmp`). `setup --json --yes` exit 2 (usage error). |
| Gate window | `open -n "/Volumes/multi-link 1.18/multi-link.app"` | Window "Set up multi-link" with "Move multi-link to Applications first", icon C and Show in Finder (screenshot `~/multi-link-sandbox/check21/gate-from-dmg.png`). No child process, no `dev.multi-link.check21` defaults written, lock file untouched. Not quarantined (built here), so no Gatekeeper block in this run. |
| `hook apply` / `remove` | the app's bundled CLI with `--settings <scratch copy of ~/.claude/settings.json>` | apply: `updated … (backup: …)`, exit 0, then `hook check` → `… are current`; apply again: `no change: …`; remove: `updated …`, multi-link's entries gone (no other hooks were in the file, so `hooks` left the file), every other key kept. Backups mode `-rw-------`; the first is byte-identical to the original. |
| Rust | `TZ=UTC cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo fmt --check` | 747 passed / 0 failed / 1 ignored; clippy exit 0; fmt exit 0. |
| Swift | `swift test --disable-sandbox` in `apps/menubar` | `Test run with 36 tests in 0 suites passed`. |

### A mistake during the check, and what it shows

The first mounted-copy run used `hdiutil attach -mountpoint <scratch folder>` instead of `/Volumes`. The `cli:
temporary` rule only knows `/Volumes/` and `AppTranslocation`, so the CLI said `cli: ok`, and `setup --yes` really
ran `hook apply` on the owner's `~/.claude/settings.json`, pointing its five multi-link hooks at the scratch mount.
Claude compared it with the backup `hook apply` had just made (only those five hook paths differed), wrote the backup's
bytes back (`cmp` identical, mode `644` kept), and `~/.cargo/bin/multi-link hook check` read `are current`, exit 0.
The file held the wrong paths for about a minute. The backup stays in `~/.claude/`
(`settings.json.multi-link-backup-1794647302429937000-…`); the owner may delete it.
What it shows: a DMG mounted somewhere other than `/Volumes` (only by hand, with `-mountpoint`) is not seen as
temporary. Finder and double-clicking always mount under `/Volumes`. README Limitations says so.

### Not checked by Claude

The real quarantine and Open Anyway, the setup window's rows on a real click, the notification prompt, `SMAppService`
register and a login launch, the upgrade keeping permissions, and a hand-off notification with the new icon. They need
the owner's password, System Settings or a log out, or would change the owner's real settings, so they are the owner
checklist below.

## Owner checklist

Each item is one command (copy and run it in Terminal) or one click. Nothing asks you to edit JSON. Your dev app and
`~/.cargo/bin/multi-link` stay installed; item 15 switches you to the real 1.18 app at the end.

1. **Quit the running dev app** so its single-instance lock doesn't hide the new one: click `ml` in the menu bar →
   **Quit**.
2. **Start the local download server** (leave this Terminal window open until item 12):
   ```bash
   python3 -m http.server 8765 --bind 127.0.0.1 --directory ~/multi-link-sandbox/check21
   ```
3. **Download v1 in Safari**: paste this into Safari's address bar. Look for: the file in Downloads.
   ```
   http://127.0.0.1:8765/v1/multi-link-1.18-arm64.dmg
   ```
4. **Confirm the real quarantine** (in a second Terminal window). Look for: a line that contains `Safari`.
   ```bash
   xattr -p com.apple.quarantine ~/Downloads/multi-link-1.18-arm64.dmg
   ```
5. **Open the DMG**: double-click it in Downloads. Look for: a window with multi-link and an Applications folder.
   Drag **multi-link** onto **Applications**.
6. **First open**:
   ```bash
   open /Applications/multi-link.app
   ```
   Look for: "multi-link Not Opened · Apple could not verify…". Click **Done**.
7. **Open Anyway**: this opens Privacy & Security; scroll to "multi-link was blocked…", click **Open Anyway**, enter
   your password, then **Open**. Look for: the "Set up multi-link" window with icon C and two rows, Claude hooks and
   Notifications.
   ```bash
   open "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension"
   ```
8. **Turn on everything** (click). Look for: Claude hooks turns green, naming `~/.claude/settings.json` and a backup;
   macOS asks to allow notifications (click **Allow**); the Notifications row turns green within a few seconds (if
   it stays amber, click **Open Settings** and turn on Allow Notifications; macOS may read "denied" for a moment
   right after the prompt, seen in spike (d)). Then click `ml` and check **Open at login** is ticked; if it shows
   **Open Login Items Settings**, click it and allow multi-link.
9. **Remove hooks, then Turn on again** (two clicks in the setup window, reopened with `Setup…`). Look for: the hooks
   row goes back to **Turn on**, then green again.
10. **Read the result from Terminal** (read-only). Look for: `cli: ok`, `hooks: ok`, `app: installed` with
    `/Applications/multi-link.app`, then `exit 0`.
    ```bash
    /Applications/multi-link.app/Contents/MacOS/multi-link setup < /dev/null; echo "exit $?"
    ```
11. **One real hand-off with the new icon** (a new Claude session loads the new hooks). Look for, within a minute: a
    banner `codex finished (…)` credited to multi-link with icon C.
    ```bash
    cd ~/multi-link-sandbox/repo-p17 && claude -p "Run this exact command and report its output: codex exec -s read-only 'Reply with just ok (Step 21.5 check)' < /dev/null" --allowedTools=Bash < /dev/null
    ```
12. **Upgrade over it**: click `ml` → **Quit**, then download v2 in Safari:
    ```
    http://127.0.0.1:8765/v2/multi-link-1.18.1-arm64.dmg
    ```
    Double-click it, drag **multi-link** onto **Applications**, click **Replace**. Stop the server (Ctrl-C in its
    window).
13. **Open the upgrade**: run this, expect "Not Opened" again (every new version needs Open Anyway once), then repeat
    item 7's command and Open Anyway. Look for: the app opens with no notification prompt, the Notifications row
    green, Claude hooks green, **Open at login** still ticked.
    ```bash
    open /Applications/multi-link.app
    ```
14. **Real login launch** (after the upgrade, so it also shows the login item survived it): Apple menu → **Log Out**, then log back in. Look for: `ml` in the menu bar without opening
    it, and this prints `/Applications/multi-link.app/…/multi-link-menubar`:
    ```bash
    pgrep -fl multi-link-menubar
    ```
15. **Switch to the real 1.18 app** (the check copy uses a test id). In the popover untick **Open at login**, click
    **Quit**, then open the real DMG and drag multi-link onto Applications, **Replace**. It is built on this Mac, so
    it opens without a block; click **Turn on everything** once (your notification permission for multi-link is
    already on).
    ```bash
    open /Users/jarednarvaez/multi-link/apps/menubar/build/multi-link-1.18-arm64.dmg
    ```
16. **Tell Claude the result**: which items matched and which didn't (a screenshot of anything odd helps). Phase 21
    closes on that answer.
17. **Publish (when you are happy)**: push, then create the GitHub release with the DMG. Run after the Step 21.5
    commit:
    ```bash
    git -C /Users/jarednarvaez/multi-link push
    ```
    ```bash
    cd /Users/jarednarvaez/multi-link && gh release create v1.18 apps/menubar/build/multi-link-1.18-arm64.dmg --title "multi-link 1.18" --notes "Mac app for Apple silicon (macOS 15+). Unsigned: after dragging to Applications, open it once, then System Settings → Privacy & Security → Open Anyway. See README → Install."
    ```

Afterwards, optional tidying: eject the mounted DMGs (Finder), delete `~/Downloads/multi-link-1.18*-arm64.dmg`, and
remove the test entry "multi-link" for `dev.multi-link.check21` in System Settings → Notifications if it shows twice.

## Owner answer

In progress (owner with Claude, 2026-11-14, 18:14–18:23 local).
- 1: skipped at first; Claude quit the dev app (`osascript … quit`) before Open Anyway.
- 2: server started by Claude; log shows `GET /v1/multi-link-1.18-arm64.dmg` 200 (18:14:47) and
  `GET /v2/multi-link-1.18.1-arm64.dmg` 200 (18:22:12).
- 3–4: downloaded with **Chrome**, not Safari: `0281;6af91597;Chrome;DF99A9C4-…`. Real quarantine either way.
- 5–7: pass. "Not Opened", then Open Anyway; the setup window showed icon C and two amber rows; icon C in the Dock.
- 8: pass. Both rows green, "Everything is on", "Login Item Added". **Bug:** under the green Notifications row an
  orange "Notifications are not allowed for this application" stayed (the transient denial right after the prompt,
  spike (d)); the window does not clear the error once the status is authorized. Gone after a relaunch (item 13).
- 9: pass. Backups show apply (…139, from `~/.cargo/bin`), remove (…179), apply (…182).
- 10: pass (run by Claude): `cli: ok (/Applications/multi-link.app/…)`, `hooks: ok`, `app: installed`, `exit 0`.
- 11: pass. Owner saw the `codex finished` banner from multi-link with icon C; the popover linked
  `claude · step 21.5` → `codex · exact`. **Checklist bug:** the command as first written failed (`--allowedTools Bash
  "…"` swallowed the prompt: "Input must be provided…"); fixed above.
- 12–13: pass. 1.18.1 installed with Chrome quarantine, "Not Opened" again, Open Anyway, no notification prompt, both
  rows green, Open at login still ticked.
- 14: pass after a real log out. A first `pgrep` showed the same PID 16692 (started 18:22:35, no log out yet). After
  the log out: `last` shows a console login at 18:25, `loginwindow` started 18:24:56, Dock 18:25:05, and
  `19601 /Applications/multi-link.app/Contents/MacOS/multi-link-menubar` started 18:25:30 with nobody opening it.
- 15: pass. Real app installed: `dev.multi-link.menubar` 1.18, no quarantine (built here), PID 21199; `setup` →
  `cli: ok`, `hooks: ok`, `app: installed`, `exit 0`; window "Everything is on" with both rows green, popover
  **Open at login** ticked (owner screenshot, 18:27).
- 16: owner confirmed item by item in chat, 2026-11-14. **Phase 21 closed.** 17 (push, `gh release create v1.18`)
  stays with the owner.
- Found: (1) a stale "Notifications are not allowed for this application" line under a green Notifications row
  until relaunch; (2) the checklist's item 11 command (fixed). Both are small; (1) goes to the backlog.
