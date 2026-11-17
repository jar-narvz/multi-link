# Installing multi-link

The [README](../README.md#install) has the short version. This page has every install path, the Terminal link,
switching from the DMG to Homebrew, building from source, uninstalling and the hook details.

## Contents

- [Install](#install)
- [Homebrew (recommended)](#homebrew-recommended)
- [Download the Mac app](#download-the-mac-app)
- [Use in Terminal](#use-in-terminal)
- [Build from source](#build-from-source)
- [Uninstall](#uninstall)
- [Hook setup](#hook-setup)

## Install

For Apple silicon Macs (arm64) on macOS 15 or later. No Rust or Xcode needed.

### Homebrew (recommended)

```bash
brew trust --cask jar-narvz/tap/multi-link
```

```bash
brew install --cask jar-narvz/tap/multi-link
```

The first command tells Homebrew to load casks from multi-link's own tap (Homebrew 7 refuses casks from a tap you
have not trusted); the second downloads the release, moves `multi-link.app` to `/Applications` and links its bundled
CLI as `multi-link` in Homebrew's `bin` folder (`/opt/homebrew/bin`). Then:

1. Open multi-link from Applications once. macOS blocks it, because the app is not notarized ([limits](reference.md#limitations-and-validation)):
   click **Done**, then open System Settings → Privacy & Security, scroll to "multi-link was blocked…", click **Open
   Anyway** and enter your Mac's password. Until you do, `multi-link` in Terminal is stopped with no message.
2. The setup window opens: click **Turn on everything** (step 4 of [Download the Mac app](#download-the-mac-app) says what it turns on).

Upgrade with `brew upgrade --cask multi-link`. An upgrade normally keeps the Open Anyway approval, because every
version is signed with the same identifier (seen in Homebrew's source and an isolated test, [design](design.md)
"Phase 22 decisions"). Not yet seen on a real install: the first real `brew upgrade` comes with the next release.

**Already installed from the DMG?** Homebrew refuses to install over an app it did not put there. Untick **Open at
login** in the popover, quit multi-link, move `/Applications/multi-link.app` to the Trash, then run the two commands above. The hooks
keep working, because Homebrew puts the app at the same path. Don't use `--force` for this: it deletes the existing app
without a backup.

### Download the Mac app

No terminal needed.

1. Download `multi-link-1.19-arm64.dmg` from the [releases page](https://github.com/jar-narvz/multi-link/releases).
2. Open it and drag **multi-link** onto **Applications** (`~/Applications` works too) before opening it. A copy
   opened inside the download is blocked the same way (step 3) and then shows only "Move multi-link to Applications
   first", starting nothing.
3. Open multi-link from Applications. macOS blocks it the first time, because the app is not notarized ([limits](reference.md#limitations-and-validation)): click
   **Done**, then open System Settings → Privacy & Security, scroll to "multi-link was blocked…", click **Open
   Anyway** and enter your Mac's password. This is needed once per copy and once per new version.
4. The setup window opens. Click **Turn on everything**: it adds multi-link's Claude hooks (after saving a backup of
   your settings file beside it), asks macOS to allow notifications, and turns on **Open at login**. Rows turn green
   on their own once each part is on; if macOS says notifications are off, **Open Settings** goes straight there.
   Start a new Claude session so it loads the hooks.

The menu-bar item `ml` is the app; `Setup…` in its popover reopens the window. The app carries its own copy of the
command-line tool, so it needs nothing else installed.

### Use in Terminal

A Homebrew install already puts `multi-link` on your `PATH`. A DMG install does not: the setup window's **Use in
Terminal** line shows whether `/usr/local/bin/multi-link` or `/opt/homebrew/bin/multi-link` points at this app's CLI.
If neither exists, it shows one command and a **Copy command** button; paste it into Terminal (it asks for your
password):

```bash
sudo mkdir -p /usr/local/bin && sudo ln -s '/Applications/multi-link.app/Contents/MacOS/multi-link' /usr/local/bin/multi-link
```

(The window fills in the app's real path.) `ln` has no `-f`, so it never replaces a file already at that path, and the
app never runs `sudo` itself. If a path holds something else, the window shows it and offers nothing over it. Check
with `command -v multi-link`: an earlier `PATH` entry wins (for example `~/.cargo/bin` from a source build). Through
any of these links, `multi-link setup` and `hook apply` still write the app's own path into the hooks.


### Build from source

Requires macOS, Rust/Cargo and git. From the repo:

```sh
cargo install --path crates/cli --locked
multi-link tui
```

This puts the binary at `~/.cargo/bin/multi-link`. To try it without installing, use
`cargo build --release -p multi-link` and run `target/release/multi-link` instead. `multi-link setup` checks and
fixes the hooks from there. The app and the DMG are built with `apps/menubar/build.sh` and `apps/menubar/release.sh`
(Swift 6.2 tools; [menu-bar app](reference.md#menu-bar-app-macos)). To build the app yourself (Xcode or the Swift 6.2
command-line tools; macOS 15+) and open it, from the repo root:

```bash
apps/menubar/build.sh
```

```bash
open apps/menubar/build/multi-link.app
```

### Uninstall

1. In the app, `Setup…` → **Remove hooks** (or run
   `/Applications/multi-link.app/Contents/MacOS/multi-link hook remove`; `multi-link hook remove` for a cargo install). Only
   multi-link's entries leave the settings file; a backup is saved beside it first.
2. Uncheck **Open at login** in the popover.
3. Remove the app:
   - Homebrew: `brew uninstall --cask multi-link` (quits the app, removes it and the `/opt/homebrew/bin` link;
     `--zap` also removes its preferences).
   - DMG: quit the app and move it to the Trash. If you made the Terminal link, remove it with
     `sudo rm /usr/local/bin/multi-link`.
   - Source build: `cargo uninstall multi-link`.

Neither Homebrew nor the app removes the hooks or the login item for you, so do steps 1 and 2 first. multi-link's own
state (`~/.multi-link`) stays until you delete it.

### Hook setup

The screens work without hooks. The Claude Code hook adds exact links for hand-offs whose Codex id only reaches the
hook, step tags as they happen, "waiting on you" when the app has no session record, and `session_end` when a
session exits. multi-link changes your Claude settings only when you ask it to: the setup window's **Turn on**,
`multi-link setup` (it asks first), or `multi-link hook apply`. `hook check` only reads them.

```sh
multi-link hook check
```

It reads one settings file: `~/.claude/settings.json` (or `$CLAUDE_CONFIG_DIR/settings.json` when that is set),
`--scope local` for the repo's `.claude/settings.local.json`, or `--settings <path>`. It prints the file it read,
one line per hook event and, when something can be fixed, one command:

- `ok`: exactly one multi-link entry, equal to what this binary would generate. All `ok` → `multi-link hooks in
  <file> are current`, exit 0.
- `other copy`: a non-customized event using an existing executable regular file owned by another copy, with
  `owned by <canonical path>` and any `async` / `timeout` drift shown. The entire event is kept, including duplicates. All events `ok` / `other copy`
  → exit 0, owner names and an optional `multi-link hook apply --take-over` instruction; no repair command.
- `missing`, `outdated` (a missing, dangling, non-executable, non-regular, `/Volumes/` or AppTranslocation binary
  path, or this copy's `async` / `timeout` drift) or `duplicate` without another owner: fixed by the printed command.
  Exit 1. Only absolute stored binary paths are probed; bare or home-relative paths stay `outdated`.
  A stored symlink resolving to this binary counts as the same copy.
- `customized` (a multi-link entry you changed, such as an `if` key or a narrower matcher), `unknown event`, or
  `disableAllHooks: true`: reported for you to edit by hand, never changed. Exit 1. Customization takes precedence
  over another owner: its path is shown in details, but the event stays `customized`, including with `--take-over`,
  and is excluded from `owners`.
- A file it cannot safely read or merge (invalid JSON, wrong types, a directory, a dangling symlink, values that would
  not survive a rewrite, below): the reason, no command, exit 2.

Copy the printed command and run it. It is plain `sh`: it copies an existing file to a backup beside it
(`settings.json.multi-link-backup-<UTC time>-XXXXXX`; an absent file is created instead), writes `multi-link hook check --merged` to a private temp file
in the same folder, gives it the file's mode and moves it into place in one step, so the file is never half-written;
if any part fails it removes the temp file and the original stays as it was. Only multi-link's own entries for
`missing`, `outdated` and `duplicate` events without another owner change; your other hooks, keys and their order stay. Run
`multi-link hook check` again to see exit 0, then start a new Claude session so it loads the hooks.

`multi-link hook apply` does the same in one step (same `--scope` / `--settings`), and also re-reads the file just
before the final rename, refusing if it changed since it was read. It prints `updated <file> (backup: <path>)`,
`created <file>` or `no change: <file>`. `multi-link hook remove` takes multi-link's entries out the same way, with a
backup.

To move hooks from another copy, use `multi-link hook apply --take-over` (or preview with
`multi-link hook check --merged --take-over`). It writes the canonical running binary path and saves the usual
backup. `multi-link setup --take-over` says the owners will be moved if applied, then asks first; add `--yes`
to answer yes. Without the flag, setup suggests the optional takeover command. Plain `setup --yes` leaves other
owners alone. `setup --json` always stays read-only and rejects `--take-over` and `--yes`; its hooks item includes
`owners`, a sorted unique array of canonical paths from `other copy` events only, empty when none. The other binary
is never executed.

Run the check with the binary you want the hooks to call (normally the installed one): the hooks are compared with
that binary's path. `hook install` still prints the bare snippet, and `hook uninstall` prints what to remove.
