# Phase 22 check: CI, a Homebrew tap, multi-link in Terminal

Step 22.6, 2026-11-16. Phase 22 makes Release 1.19 the first release built by GitHub Actions: CI on every push to
`main` (`.github/workflows/ci.yml`), a draft release on a `v*` tag (`release.yml`), a cask pushed to
`jar-narvz/homebrew-tap` when the owner publishes (`cask.yml`), and "Use in Terminal" in the setup window. See
[design](design.md) → "Phase 22 decisions" and PLAN.md Phase 22 (R1–R7). This file has two parts: what Claude checked
without the owner, and the owner checklist for the push, the tag, publishing and the switch of the real app to brew.

## Checked by Claude

### Remote CI (Step 22.5, 2026-11-15)

| Run | Commit | Observed |
| --- | --- | --- |
| 43678448297 | `6b0a20a` (Step 22.4) | Toolchain, cask tests, Rust tests, clippy and fmt passed; `swift test` then hung with no output for 28 minutes and the owner cancelled the run (29m59s, `cancelled`). |
| 43771809152 | `74993ca` (the fix) | `success`, 2026-11-15 22:53:48–22:58:19Z (4m31s). Log: image macos-15-arm64, `arm64`, `Xcode 26.3`, `Apple Swift version 6.2.4`, `rustc 1.99.1`; Swift step "✔ Test run with 41 tests in 0 suites passed after 10.512 seconds."; build step "Built and verified build/multi-link.app". |

**The hang and the fix.** The app's `Watcher` (two pipe readers per child process) and `MenubarActions.run` made
blocking calls (`read()`, `readDataToEndOfFile`, `waitUntilExit`) inside `Task.detached`, which runs on Swift's
cooperative thread pool: one thread per core, and the macos-15 runner has 3. The parallel tests blocked every pool
thread, so the tasks that would quit the children never ran. It reproduced on this Mac with
`LIBDISPATCH_COOPERATIVE_POOL_STRICT=1`, and it was a real app bug, not only a test artefact. `74993ca` moves the
blocking calls to dedicated threads (`MenubarCore/BlockingIO.swift`) and adds the regression test
`moreWatchersThanCoresAllReportAndQuit`. CI's Swift step now has `timeout-minutes: 10` and runs under a pty
(`script -q /dev/null`), so a future hang shows the last "Test … started" lines instead of nothing.

### State before the owner check (read with `gh`, 2026-11-16)

| Check | Command | Observed |
| --- | --- | --- |
| Tap repo | `gh repo view jar-narvz/homebrew-tap --json name,isEmpty,defaultBranchRef,visibility` | Public, **empty** (no default branch yet). The first `cask.yml` run creates `main` and `Casks/multi-link.rb`. |
| Tap token | `gh secret list --repo jar-narvz/multi-link` | `TAP_TOKEN`, set 2026-11-15T04:56:02Z. |
| Releases | `gh release list --repo jar-narvz/multi-link --limit 3` | Only `v1.18` (Latest). |
| Local branch | `git status -sb` | `main` ahead of `origin/main`: `4e857c0` (Step 22.5 record) and this step's commit are not pushed. |
| Real app | `/Applications/multi-link.app/Contents/MacOS/multi-link setup < /dev/null` | `cli: ok`, `hooks: ok`, `app: installed (/Applications/multi-link.app)`, exit 0. This is the 1.18 DMG install. |
| `PATH` on this Mac | `zsh -ic 'whence -a multi-link'` | Only `~/.cargo/bin/multi-link` (a source build from 2026-11-14), and `~/.cargo/bin` comes before `/opt/homebrew/bin` in the login shell's `PATH`. So after the brew install, a bare `multi-link` here still runs the cargo copy. Item 8 therefore calls the brew link by its full path; this is the README's "an earlier `PATH` entry wins" limit, seen for real. |

### Local Verify (Step 22.6)

Run on this Mac, 2026-11-16, with Info.plist at `1.19`. Logs were kept in the session scratchpad.

| Check | Command | Observed |
| --- | --- | --- |
| Rust tests | `TZ=UTC cargo test --workspace --locked` | exit 0; summed `test result` lines: 748 passed, 0 failed, 1 ignored. |
| Clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0. |
| Format | `cargo fmt --check` | exit 0. |
| Swift | `swift test --disable-sandbox` in `apps/menubar` | exit 0; "✔ Test run with 41 tests in 0 suites passed after 11.208 seconds." |
| Workflows | `actionlint` | exit 0, no output. |
| Cask and tap tests | `sh tools/tests/cask.sh` | exit 0; 24 PASS, 0 FAIL. |
| Release tests | `sh tools/tests/release.sh` | exit 0; 11 PASS, 0 FAIL. |
| Local release build | `apps/menubar/release.sh` | exit 0; `hdiutil: verify: checksum … is VALID`; `multi-link-1.19-arm64.dmg` 2,929,088 bytes and its `.sha256` 92 bytes (`f088bde0…686373a6  multi-link-1.19-arm64.dmg`); `shasum -a 256 -c` from the build folder → `OK`; built app reads `1.19`. This local DMG is not the release asset: CI builds its own, so its size may differ. |

Review: a read-only Codex review of the README and this checklist found 5 points, all accepted: the release and cask
run watchers now pick the run for this tag or event, the cask run's result is checked before the tap, no estimated
DMG size, the README unticks Open at login before quitting, and the real `brew upgrade` is recorded as not covered.

## Owner checklist

Each item is one command (copy and run it in Terminal) or one click, with what to look for. Nothing asks you to edit
JSON. Stop and tell Claude at the first item that doesn't look as described. Claude never pushes, tags or publishes.

1. **Push `main`** (two commits: Step 22.5's record and this step).
   ```bash
   git -C ~/multi-link push origin main
   ```
   Look for: `main -> main`.
2. **Wait for CI to pass** (about 5 minutes). This waits and prints the result.
   ```bash
   cd ~/multi-link && gh run watch "$(gh run list --workflow ci.yml --commit "$(git rev-parse HEAD)" --limit 1 --json databaseId --jq '.[0].databaseId')" --exit-status
   ```
   Look for: every step with a green tick and the command ending without an error. If it says it needs a run id, the
   run hasn't registered yet: wait 10 seconds and run it again. If CI fails, stop here.
3. **Tag the release and push the tag.** This starts the release workflow (check → CI → build → draft).
   ```bash
   cd ~/multi-link && git tag v1.19 && git push origin v1.19
   ```
   Look for: `* [new tag] v1.19 -> v1.19`. Then wait for the run (about 10 minutes):
   ```bash
   cd ~/multi-link && gh run watch "$(gh run list --workflow release.yml --commit "$(git rev-parse 'v1.19^{commit}')" --limit 1 --json databaseId --jq '.[0].databaseId')" --exit-status
   ```
   Look for: jobs `check`, `ci` and `release` all green (same "wait 10 seconds" note as item 2).
4. **Check the draft.** Open it in the browser:
   ```bash
   gh release view v1.19 --repo jar-narvz/multi-link --web
   ```
   Look for: title "multi-link 1.19", marked **Draft**, and exactly two files under Assets:
   `multi-link-1.19-arm64.dmg` and `multi-link-1.19-arm64.dmg.sha256`. Nothing else. Note the two sizes shown for
   Claude (Claude reads the exact bytes with `gh release view v1.19` afterwards).
5. **Publish it**: on that page click **Edit** (pencil) → **Publish release**. Look for: the release shows as
   **Latest**, no longer Draft.
6. **See the cask land in the tap.** Publishing started `cask.yml`; wait for it:
   ```bash
   cd ~/multi-link && gh run watch "$(gh run list --workflow cask.yml --event release --limit 1 --json databaseId --jq '.[0].databaseId')" --exit-status
   ```
   Look for: the run green (same "wait 10 seconds" note as item 2). Then read the cask:
   ```bash
   gh api repos/jar-narvz/homebrew-tap/contents/Casks/multi-link.rb --jq .content | base64 -d | head -3
   ```
   Look for: `cask "multi-link" do` and `version "1.19"`. If the run failed or this says "Not Found", stop and tell
   Claude.
7. **Switch the real app to brew.** Do these in order.
   1. Click `ml` in the menu bar and untick **Open at login**.
   2. Click `ml` → **Quit**.
   3. Move the DMG copy to the Trash (recoverable from the Trash). This opens Finder with it selected; press ⌘⌫
      (a Terminal `mv` into `~/.Trash` fails with "Operation not permitted" without extra privacy permissions):
      ```bash
      open -R /Applications/multi-link.app
      ```
   4. Trust the tap:
      ```bash
      brew trust --cask jar-narvz/tap/multi-link
      ```
   5. Install:
      ```bash
      brew install --cask jar-narvz/tap/multi-link
      ```
      Look for: "Moving App 'multi-link.app' to '/Applications/multi-link.app'", "Linking Binary 'multi-link' to
      '/opt/homebrew/bin/multi-link'", the caveat about Open Anyway, and "multi-link was successfully installed!".
   6. Open it:
      ```bash
      open /Applications/multi-link.app
      ```
      Look for: "multi-link Not Opened". Click **Done**.
   7. Open System Settings → Privacy & Security, scroll to "multi-link was blocked…", click **Open Anyway**, enter your
      password, then click **Open Anyway** again in the dialog. Look for: the setup window.
   8. In the setup window click **Turn on everything** (hooks should already be green). Look for: both rows green, and
      under them "Use in Terminal" with "multi-link is linked at /opt/homebrew/bin/multi-link."
   9. Click `ml` and tick **Open at login** if Turn on everything didn't.
8. **Check the CLI from a new Terminal window** (⌘N), through the brew link:
   ```bash
   /opt/homebrew/bin/multi-link setup < /dev/null; echo "exit $?"
   ```
   Look for: `cli: ok (/Applications/multi-link.app/Contents/MacOS/multi-link)`, `hooks: ok (…/.claude/settings.json)`,
   `app: installed (/Applications/multi-link.app)`, `exit 0`. (A bare `multi-link` on this Mac still finds
   `~/.cargo/bin/multi-link` first; see "PATH on this Mac" above.)
9. **One real hand-off banner** (a new Claude session, so it loads the hooks):
   ```bash
   cd ~/multi-link-sandbox/repo-p17 && claude -p "Run this exact command and report its output: codex exec -s read-only 'Reply with just ok (Step 22.6 check)' < /dev/null" --allowedTools=Bash < /dev/null
   ```
   Look for, within a minute: a banner `codex finished (…)` credited to multi-link.
10. **Tell Claude the result**: "22.6 check: all pass", or the item number and what you saw instead.

## Not covered by this check

- **A real `brew upgrade`** (fresh install → Open Anyway → upgrade with no new prompt). It needs a newer release than
  1.19, so it is deferred to the next release; until then it rests on Homebrew's source and the isolated
  `dev.multi-link.check22` test (design "Phase 22 decisions"). The README says so.
- A fresh macOS account or VM: the owner's Mac already approved earlier multi-link builds' notifications.

## Owner answer

2026-11-17: **all pass.** The owner ran items 1–9 with Claude; Claude checked each item read-only with `gh`.

| Item | Result |
|---|---|
| 1–2 push + CI | `main` at `620a0bc` pushed; CI run 43952924468 `success` (every step green) |
| 3 tag | `v1.19` pushed; release run 43956976978 `success`: `check`, `ci / test`, `release` all `success` |
| 4 draft | `gh release view v1.19` → name `multi-link 1.19`, `isDraft` true, exactly `multi-link-1.19-arm64.dmg` 2,929,184 bytes and `multi-link-1.19-arm64.dmg.sha256` 92 bytes (CI's build; the local one was 2,929,088) |
| 5 publish | owner published; `isDraft` false, `publishedAt` 2026-11-17T22:45:07Z, URL `…/releases/tag/v1.19` (a draft's URL reads `untagged-…` until published) |
| 6 tap | `cask.yml` run 44193075230 triggered by `release`, job `cask` ✓ in 12s; tap `Casks/multi-link.rb` → `version "1.19"`, `sha256 "30a20284…82b1c734"`, equal to the release's `.sha256` file |
| 7 brew switch | Terminal `mv … ~/.Trash/` failed: "Operation not permitted" (macOS privacy protection), so the owner trashed it in Finder (step 3 now says so). `brew install --cask jar-narvz/tap/multi-link` downloaded 2.9 MB, moved the app to `/Applications`, linked `/opt/homebrew/bin/multi-link`, printed the caveats. `/opt/homebrew/bin/multi-link` → `/Applications/multi-link.app/Contents/MacOS/multi-link`; quarantine `03c1;…` (approved via Open Anyway); `brew list --cask` shows `multi-link` |
| 8 CLI via brew link | `cli: ok (/Applications/multi-link.app/Contents/MacOS/multi-link)`, `hooks: ok (~/.claude/settings.json)`, `app: installed (/Applications/multi-link.app)`, `exit 0`: the canonical path (R6), not the brew link |
| 9 banner | owner saw the `codex finished (…)` banner from multi-link for the sandbox hand-off |

Still not covered: a real `brew upgrade` (see above), deferred to the next release.
