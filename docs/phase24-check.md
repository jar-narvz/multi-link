# Phase 24 check: two setup fixes, Release 1.20, the first real brew upgrade

Step 24.4, 2026-11-17. Phase 24 fixes two setup bugs and ships them as Release 1.20 through the Phase 22 pipeline
unchanged (see PLAN.md Phase 24, decisions D1–D4, and [design](design.md)):

- **Step 24.1:** the setup window no longer keeps the orange "Notifications are not allowed for this application" line
  under a green Notifications row. The error has its own field and clears when the status reads authorized.
- **Step 24.2:** `hook check` / `setup` run from another copy of the CLI report hooks owned by an existing installed
  copy as `other copy` (exit 0, no repair command) instead of `outdated`; only `--take-over` moves them.
- **Step 24.3:** the app's **Turn on** / **Turn on everything** run `hook apply --take-over`, and a non-empty `owners`
  list shows amber "Hooks use another copy: <path>".

This file has two parts: what Claude checked without the owner, and the owner checklist for the push, the tag,
publishing and the first real `brew upgrade` (1.19 → 1.20), which Phase 22 could not cover.

## Checked by Claude

### Local Verify (Step 24.4)

Run on this Mac, 2026-11-17, with `apps/menubar/Info.plist` at `1.20` (both keys). Logs were kept in the session
scratchpad.

| Check | Command | Observed |
| --- | --- | --- |
| Rust tests | `TZ=UTC cargo test --workspace --locked` | exit 0; summed `test result` lines: 757 passed, 0 failed, 1 ignored. |
| Clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0. |
| Format | `cargo fmt --check` | exit 0. |
| Swift | `(cd apps/menubar && swift test --disable-sandbox)` | exit 0; "✔ Test run with 48 tests in 0 suites passed after 10.385 seconds." |
| App build | `(cd apps/menubar && ./build.sh)` | exit 0; "Built and verified build/multi-link.app (bundled CLI, app, icon)". |
| Workflows | `actionlint` | exit 0, no output. |
| Cask and tap tests | `sh tools/tests/cask.sh` | exit 0; 24 PASS, 0 FAIL. |
| Release tests | `sh tools/tests/release.sh` | exit 0; 11 PASS, 0 FAIL. |
| Local release build | `apps/menubar/release.sh` | exit 0; `hdiutil: verify: checksum … is VALID`; `multi-link-1.20-arm64.dmg` 2,942,622 bytes and its `.sha256` 92 bytes (`8ac36913…23e5caad10  multi-link-1.20-arm64.dmg`); `shasum -a 256 -c` from the build folder → `OK`; built app reads `1.20` for both keys. This local DMG is not the release asset: CI builds its own, so its size may differ. |

### The real settings, read-only (2026-11-17)

The coordinator ran this first; Claude re-ran it in this step with the same result. From the repo's cargo build
(`target/debug/multi-link`, which has the 24.2 code; `~/.cargo/bin/multi-link` is an older build from 2026-11-14 and
was not used), with input closed:

```
target/debug/multi-link hook check < /dev/null
```

| Observed | |
| --- | --- |
| File | `~/.claude/settings.json` |
| Events | `PostToolUse`, `PostToolUseFailure`, `PermissionRequest`, `Notification`, `SessionEnd`: all `other copy (owned by /Applications/multi-link.app/Contents/MacOS/multi-link)` |
| Closing line | names the owner and the optional `multi-link hook apply --take-over`; no repair command |
| Exit | 0 |
| Settings file | sha256 the same before and after (`cb012d65…`) |

Before 24.2 the same run would have called these hooks `outdated` and printed a command that repoints them to the
cargo copy. This is the bug from 2026-11-14, now reported correctly on real data.

### State before the owner check (2026-11-17)

| Check | Command | Observed |
| --- | --- | --- |
| Local branch | `git status -sb`, `git log origin/main..HEAD` | `main` **ahead 5, behind 1**: Phase 24's plan, Steps 24.1–24.3 and this step are not pushed; `origin/main` has the owner's GitHub edit `b8ded8f` "Update README.md" (removes "Who it's for"). |
| Merge | `git merge-tree --write-tree HEAD origin/main` | clean. This step's README edits are in other hunks. Item 1 therefore pulls with a merge commit (no rebase, no history rewrite) before pushing. |

## Owner checklist

Each item is one command (copy and run it in Terminal) or one click, with what to look for. Nothing asks you to edit
JSON. Stop and tell Claude at the first item that doesn't look as described. Claude never pushes, tags or publishes.
After each item Claude checks it read-only (the "Claude checks" line).

1. **Bring in your GitHub README edit, then push `main`.**
   ```bash
   git -C ~/multi-link pull --no-rebase --no-edit origin main
   ```
   Look for: `Merge made by the 'ort' strategy.` and `README.md | 8 --------`. Then:
   ```bash
   git -C ~/multi-link push origin main
   ```
   Look for: `main -> main`.
   Claude checks: `git status -sb` → `main...origin/main` with no ahead/behind; `git show HEAD:apps/menubar/Info.plist`
   → `1.20`.
2. **Wait for CI to pass** (about 5 minutes). This waits and prints the result.
   ```bash
   cd ~/multi-link && gh run watch "$(gh run list --workflow ci.yml --commit "$(git rev-parse HEAD)" --limit 1 --json databaseId --jq '.[0].databaseId')" --exit-status
   ```
   Look for: every step with a green tick and the command ending without an error. If it says it needs a run id, the
   run hasn't registered yet: wait 10 seconds and run it again. If CI fails, stop here.
   Claude checks: `gh run view <id> --json conclusion,headSha` → `success` on the pushed commit, and the Swift step's
   test count (48).
3. **Tag the release and push the tag.** This starts the release workflow (check → CI → build → draft).
   ```bash
   cd ~/multi-link && git tag v1.20 && git push origin v1.20
   ```
   Look for: `* [new tag] v1.20 -> v1.20`. Then wait for the run (about 10 minutes):
   ```bash
   cd ~/multi-link && gh run watch "$(gh run list --workflow release.yml --commit "$(git rev-parse 'v1.20^{commit}')" --limit 1 --json databaseId --jq '.[0].databaseId')" --exit-status
   ```
   Look for: jobs `check`, `ci` and `release` all green (same "wait 10 seconds" note as item 2).
   Claude checks: the run's three jobs `success`, and `v1.20` points at the commit CI passed in item 2.
4. **Check the draft.** Open it in the browser:
   ```bash
   gh release view v1.20 --repo jar-narvz/multi-link --web
   ```
   Look for: title "multi-link 1.20", marked **Draft**, and exactly two files under Assets:
   `multi-link-1.20-arm64.dmg` and `multi-link-1.20-arm64.dmg.sha256`. Nothing else.
   Claude checks: `gh release view v1.20 --json name,isDraft,assets` → `isDraft` true, those two names only, their
   exact byte sizes.
5. **Publish it**: on that page click **Edit** (pencil) → **Publish release**. Look for: the release shows as
   **Latest**, no longer Draft.
   Claude checks: `isDraft` false, `publishedAt` set, URL `…/releases/tag/v1.20`.
6. **See the cask land in the tap.** Publishing started `cask.yml`; wait for it:
   ```bash
   cd ~/multi-link && gh run watch "$(gh run list --workflow cask.yml --event release --commit "$(git rev-parse 'v1.20^{commit}')" --limit 1 --json databaseId --jq '.[0].databaseId')" --exit-status
   ```
   Look for: the run green (same "wait 10 seconds" note as item 2; the filter on v1.20's commit keeps it from picking
   1.19's run). Then read the cask:
   ```bash
   gh api repos/jar-narvz/homebrew-tap/contents/Casks/multi-link.rb --jq .content | base64 -d | head -3
   ```
   Look for: `cask "multi-link" do` and `version "1.20"`. If the run failed or this says "Not Found", stop and tell
   Claude.
   Claude checks: the cask's `sha256` equals the release's `.sha256` file.
7. **Upgrade with Homebrew** (the first real `brew upgrade`). First click `ml` → **Quit** (leave **Open at login**
   as it is): a running 1.19 would keep running from memory, and `open` below would only bring the old one forward.
   ```bash
   brew update && brew upgrade --cask multi-link
   ```
   Look for: the download of `multi-link-1.20-arm64.dmg`, "Moving App 'multi-link.app' to
   '/Applications/multi-link.app'", and "multi-link 1.19 -> 1.20" (or similar) with no error. Then open it:
   ```bash
   open /Applications/multi-link.app
   ```
   Look for: **no** "multi-link Not Opened" dialog and no Open Anyway in Privacy & Security; `ml` comes back in the
   menu bar. If macOS blocks it, stop and tell Claude (do Open Anyway only after that, so the result is recorded).
   Claude checks: `brew list --cask --versions multi-link` → `multi-link 1.20`; the installed app's
   `CFBundleShortVersionString` → `1.20`; `xattr -p com.apple.quarantine /Applications/multi-link.app` (the flag stays;
   multi-link never removes it); `/opt/homebrew/bin/multi-link` still links to the app's bundled CLI.
8. **Check the setup window.** Click `ml` → **Setup…**.
   Look for: every row green, no amber "Hooks use another copy" line, and no orange "Notifications are not allowed"
   line. (If the Hooks row is amber, click **Turn on**: it moves the hooks to this app, with a backup.)
   Claude checks: nothing more to read here; your answer is the result.
9. **Check the CLI from a new Terminal window** (⌘N), through the brew link:
   ```bash
   /opt/homebrew/bin/multi-link setup < /dev/null; echo "exit $?"
   ```
   Look for: `cli: ok (/Applications/multi-link.app/Contents/MacOS/multi-link)`, `hooks: ok (…/.claude/settings.json)`,
   `app: installed (/Applications/multi-link.app)`, `exit 0`, and no line about another copy. (A bare `multi-link` on
   this Mac still finds `~/.cargo/bin/multi-link` first; see `docs/phase22-check.md` → "PATH on this Mac".)
   Claude checks: the same command read-only, plus `hook check` from the repo's cargo build now reading the hooks as
   `other copy` owned by the upgraded app's path (the path is unchanged by the upgrade, so the hooks did not need to
   move).
10. **One real hand-off banner** (a new Claude session, so it loads the hooks):
    ```bash
    cd ~/multi-link-sandbox/repo-p17 && claude -p "Run this exact command and report its output: codex exec -s read-only 'Reply with just ok (Step 24.4 check)' < /dev/null" --allowedTools=Bash < /dev/null
    ```
    Look for, within a minute: a banner `codex finished (…)` credited to multi-link.
    Claude checks: the sandbox session's hand-off appears in `multi-link` (exact link when the hook saw the Codex id).
11. **Tell Claude the result**: "24.4 check: all pass", or the item number and what you saw instead.

## Not covered by this check

- **The stale Notifications line on a real Mac.** Notifications are already authorized on this Mac, so the upgrade
  cannot reproduce the denied-then-allowed sequence of the Phase 21 check, item 8. The fix rests on Step 24.1's tests
  (a fake notifier that throws, reads denied, then authorized on a later `pollTick()` → the error clears; one that
  stays denied → kept; a hook error survives). Item 8 shows only that the line is absent when nothing is wrong.
- **Turn on taking over real hooks.** The owner's hooks already belong to the brew app, so item 8 should show no amber
  line. The takeover rests on Step 24.3's tests and its real run with a temp `CLAUDE_CONFIG_DIR`.
- A fresh macOS account or VM (as in Phase 22).

## Owner answer

Pending.
