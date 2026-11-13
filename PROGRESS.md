# PROGRESS.md — multi-link

## Current state
**Phase 20 is in progress (retire the Claude Code pane; owner, 2026-11-12). Step 20.0 is committed (Claude).
Next: Step 20.1 (remove the `pane` command and the mod; Codex).**
- Keys: `fixtures/expected/menubar/` now has five reports plus `watch-script.json`, `watch-tick.jsonl` and
  `watch-codex-done.jsonl`; every pane key's rows are proven equal to them (design "Phase 20 decisions").
- The owner turns the pane off with the command Claude gave (removes `CLAUDE_CODE_PLUGIN_DIRS`) and restarts Claude
  Code before the 20.3 check. The pinned Phase 18 pane check is dropped; the pinned Phase 19 notification check
  moves to 20.3.

**Phase 19: Step 19.4 is done; the owner check is pinned for later (Claude, 2026-11-12).** The SwiftUI menu-bar app
(`apps/menubar/`, built by `build.sh`) shows every repo's sessions (board 19 P2 + I1) from `multi-link menubar --watch`.
- Checked live: the popover matches the board (owner); the icon showed only after the owner freed two menu-bar items
  (notch); one real hand-off made one notification, but from Script Editor, because the pane's 1.14 watcher has no
  lock check (`docs/phase19-check.md`).
- **Pinned owner check (shared with Phase 18):** `cargo install --path /Users/jarednarvaez/multi-link/crates/cli
  --locked`, restart Claude Code, then (a) Phase 18's `Jump to terminal` from the pane and (b) one Codex hand-off with
  the menu-bar app open: the notification should come from multi-link. If (b) shows nothing, require
  `authorizationStatus == .authorized` in the app's permission check (design M3 permission).
- The app's binary-path setting points at `target/release/multi-link` for the check; remove it after the install
  with `defaults delete dev.multi-link.menubar multiLinkBinary`.
- Next after the owner checks: the web browser app (backlog), reading the same `menubar` contract.

**Phase 18: Step 18.4 is done; the owner check is pending (Claude, 2026-11-12).** `multi-link open`, the TUI's `o`
and the pane's `Jump to terminal` bring a running Claude session's Terminal.app / iTerm2 tab to the front (board 18 J1).
- Checked live with Terminal.app: the CLI and the TUI `o` jumped to the right tab from another front window; the real
  script answers `not found` for a missing tab. iTerm2, tmux, other terminals, a denied Automation prompt (-1743) and a
  real tab-gone process are covered by fixtures and tests only (`docs/phase18-check.md`, README Limitations).
- A Claude session never sent a prompt has no transcript, so no list (TUI, pane) shows it, though `open <id>` resolves it.
- **Pinned (owner, 2026-11-12): the owner check is deferred; the owner tests it later.** Steps: run `cargo install
  --path /Users/jarednarvaez/multi-link/crates/cli --locked`, restart the app, open a session in
  `~/multi-link-sandbox/repo-p17`, bring another window to the front, then press `Jump to terminal` on the Terminal
  session's row. If that sandbox session has ended, start one with a prompt first (`claude "Reply with just the word
  ok."` in that folder, in Terminal). Phase 18 closes when the answer is recorded; later phases do not wait for it.

**Phase 17 is closed (2026-11-11): owner confirmed in the desktop app ("Saw both").** The pane shows the step strip,
opens rows with `Open in app` / `Copy id`, and macOS shows `codex finished (<step>)` once per hand-off. macOS needs
Script Editor allowed once for `osascript` notifications (README). Nothing is planned next; backlog in PLAN.md.

**Phase 16 is closed (2026-11-11): owner confirmed the Claude Code pane ("Looks good").** `multi-link pane --json
[--watch]` and the mod in `mod/multi-link-pane/` are committed; checked in the terminal and in the desktop app
(`docs/phase16-check.md`). The owner's `~/.claude/settings.json` sets `CLAUDE_CODE_PLUGIN_DIRS` to the mod folder, so
every Claude Code session on this machine loads the pane. **Next:** the backlog (PLAN.md): tmux / Herdr jump
(optional), desktop app with notifications, pane step strip and actions (board 16 B). Owner's polish notes, not
asked for yet: empty section rules stay drawn; `updated Ns ago` counts from the last printed line.

**Step 16.2 is committed (Codex loop, 2026-11-11): `multi-link pane --watch --json`.** One long-lived `Scanner`;
compact, flushed NDJSON on a data change (ignoring `now` and every `phrase`) or a 30 s heartbeat, 3 s ticks;
`{ "v": 1, "error": "…" }` lines keep going; exits when its parent pid changes (or is 1 at start) and on a broken
stdout pipe. Real `~` first line median 1640 ms (Claude, outside the sandbox). **For 16.3:** spawn
`multi-link pane --json --watch --cwd <session cwd> --session <$.session.id()>`; buffer chunks to newlines; an
`error` line keeps the last data; the process ends with the spawn loop (G5 (d)).

**Phase 16 (Claude Code pane) is in progress. Step 16.0 is committed (Claude step, 2026-11-11): G1–G6 and the pins are
in `docs/design.md` → "Phase 16 decisions"; ten answer keys in `fixtures/expected/pane/`.**
- **G5 spike passed in the terminal** (Claude Code 2.1.322): `$.session.id()` = transcript file id (G1 keeps
  `--session`); pane docks right and redraws from an atom on a timer; `/command` registers; `$.process.spawn` streams
  raw text chunks (buffer to newlines) and unload / `/exit` kill the child (its ppid is `claude`); `claude plugin test`
  mounts on all four surfaces. Desktop-app half of (a)/(b) pending for 16.4.
- **For 16.1:** each key's exact command is in the Step 16.0 log entry below (`TZ=UTC`, `--root`, `--now`, `--cwd`,
  `--session`); print pretty JSON byte for byte. Rules that differ from the plain repo view (all pinned in design.md):
  quiet `waiting`/`stale` rows stay visible; every Claude lane (also a started child) is a row, only Codex lanes nest;
  flat `started_at` sort per section with `this` first; placement is the overview's T11 rule (a cross-repo child is a
  top-level row in its own repo); label = `view::home_label`; `handoffs` from the 2 h links report (ambiguous /
  unlinked only); `repo` = mapped cwd even with no group. No key covers a started child or an unlinked hand-off:
  unit-test both.

**Phase 15 is closed (2026-11-11): owner confirmed the loading screen ("Looks good"). Step 15.4 is committed
(Claude step): Phase 15 real check and README.** All F3 budgets pass on real `~` (release `5a63110`, series 2, load 4.7–6.2): (a) cold `overview --json`
median **1.16 s**; (b) worker first report **1525 ms** without `--live`, **1613 ms** with; (c) first home frame in
the real TUI (PTY) **1.952 s** (`--live` 1.969 s); (d) warm **130 / 194 ms**. Under load 12–17 one `--live` first report
took 2212 ms (median 1853 ms). PTY smoke 14/14 ok (resize 60×16 / 30×6, skip, `--live`, `q` and Ctrl-C during
loading and the fade restore the terminal). README has a "Loading screen" section with
`docs/img/loading-phase15-fixture.png`; details in `docs/phase15-check.md`. **Next:** the backlog (PLAN.md). Scratch tools (not in the repo): PTY harness with a draining reader thread and screen model, and
the (b)/(d) timing crate, in this step's session scratchpad.

Step 15.2 is committed: `multi_link_tui::{render_loading, LoadingOptions, LoadPhase, Fade}` is the pure renderer;
`time` is seconds since loading began (no board `+1`). `home_html --loading SECONDS OUT.html --size WxH` draws the
fixture frame (reading 612 / 871) for the 15.4 README image.

**Step 15.1 is committed (2026-11-11, Codex loop): parallel cold parse, F2 measurement (no `Arc`), progress
handle.** F3 passed after review round 2 (Claude, release, real
`~`, 3 fresh processes): (a) cold `overview --json` median **0.97 s** (was 7.99 s); (b) worker first report median
**1397 ms** without `--live`, **1408 ms** with (was 8624 / 8514 ms); (d) warm median 110 / 169 ms. PLAN.md F1 was
amended in review to prefetch Claude desktop-app records too (Review log). No disk cache (15.1a) is needed.
- **Core progress handle for 15.3:** `multi_link_core::{LoadProgress, LoadSnapshot}`. The worker creates a
  `LoadProgress`, calls `scanner.set_progress(p.clone())` before its first pass (survives the root reset), sends a
  clone to the UI thread, and calls `p.finish()` after sending its first report (after the all-time pass). The UI reads
  `p.snapshot()` → `LoadSnapshot { phase: &'static str ("listing" | "reading" | "linking" | "done"), files_total,
  files_done }`; totals are hidden (0 / 0) while phase is listing; zero misses go straight to linking; later passes
  never touch it. In 15.3 the CLI maps the phase string and counts into TUI-owned `LoadingOptions`.
- **Where the cold time still goes (if it ever matters again):** overview ≈ 0.93–1.04 s; the all-time pass ≈ 390 ms
  cold, mostly serial `git rev-parse` per new cwd (`RepoMode::map`), not parsing — the next lever, not a disk cache.
- **Baseline to beat (release `c29c516`, real `~`, 3 fresh processes):** F3 (a) cold `overview --json` wall 9.02 /
  7.99 / 7.95 s (median 7.99 s; user ≈ 7.25 s, sys 0.67–0.80 s); F3 (b) first report without `--live` 9344 / 8624 /
  8446 ms (median 8624; overview ≈ 8.0–8.8 s, all-time pass 435–524 ms), with `--live` 8510 / 8514 / 8550 ms (median
  8514; live part 63–65 ms). Budget: (a) and (b) ≤ 2.0 s each. 15.1 measurements are in the newest log entry below.
- **Timing harness for (b)** (scratch only, never in the repo): a crate with path deps on `crates/{core,plan,model}`
  whose `main` repeats `refresh_with_live` from `crates/cli/src/tui.rs` without the graph — `Scanner::overview` with
  `since` 2 h and `RealProcessProbe`, then `Scanner::refresh` with `since: None`, `live::started_steps` /
  `codex_steps`, then `Scanner::live` when `--live` — one fresh process per run, timed with `Instant`.
- **For 15.2:** `NO_COLOR` must drop BOLD on the loading screen too (`Theme::apply_no_color` removes only DIM; don't
  change it); round with `floor(v + 0.5)`; the meet `*` uses a per-frame head map (design.md, pinned rules).
- **Loading goldens** (`fixtures/expected/tui/`, read-only answer keys tested by 15.2):
  `loading-{120x40,76x22,60x16,59x16,50x12,29x6,28x5}.{txt,styles.txt}` (truecolor), `loading-76x22-256.styles.txt`,
  `loading-76x22-nocolor.{txt,styles.txt}`, `loading-76x22-{ready,linking,zero,fade1}.{txt,styles.txt}`; all at
  `drawC(4.0)`, counter `reading transcripts  612 / 871` unless named. The 76×22 art equals the board's frame glyph for
  glyph; `-zero` is phase reading `0 / 0`.

**Phase 14 is closed (2026-11-11): Step 14.4 is committed and the owner confirmed it ("look good").** The live grid
(`l` from home or the repo view, `multi-link tui --live`) shows a card per working, waiting or stale session with its
last prompt, state, counts and last 3 notable calls, a Claude card paired with its live Codex card, and a hand-off feed;
data from `core::live` / `multi-link live`. Real check in `docs/phase14-check.md`: warm refresh ~0.2 s; **cold first
pass 9.2–9.4 s, over the 2 s NFR**, almost all the overview's first parse (predates Phase 14). Next is the backlog
(PLAN.md); a faster first load is a candidate. The owner pushes `main`.

**Phase 13 is closed (2026-11-10): Step 13.3 is committed and the owner confirmed it ("Looks good").** C1 resolves a
literal `cd` into Link `launch_dir` (omitted when equal to `checkout`); C3 reads option-first explicit resume ids; D1
reads keys with crossterm `use-dev-tty`. Real check in `docs/phase13-check.md`: unlinked 49 → 12, ambiguous 1 → 0,
masked 0 wrong, resize check 5/5. Next is the backlog (PLAN.md). The owner pushes `main`.

**Phase 12 is closed (2026-11-10): Step 12.2 is committed and the owner confirmed it ("Looks good").** Launch
hand-offs with several runs in the window get the nearest run when the evidence is clear (K1–K3, `nearest_run`,
never exact); real check in `docs/phase12-check.md`. Next is the backlog (PLAN.md).

**Phase 11 is closed (2026-11-10): Step 11.3 is committed and the owner confirmed it ("Looks good").** The repo view
draws the now line; the step graph marks live parts (L1, computed by the CLI worker). Real check in
`docs/phase11-check.md`. Next is the backlog (PLAN.md): `cd`-then-`resume --last`, ambiguous tie-break, lost key on
resize, live grid. The owner pushes `main`.

**Step 11.2 is committed (2026-11-10, Codex loop): step graph live box. Step 11.3 (real check, images, README, owner
confirm) is next.** `multi_link_core::graph::live_parts` gives live part indices; the CLI worker sends them as
`Refresh::graph_live`; `GraphOptions::live` drives the heavy border, `● now` badge, dimmed marks and off-screen
labels. New snapshot `step-live`; `home_html.rs` has a live variant for screenshots.
**Step 11.1 is committed (cd5c0b1): repo view now line.**
The `now` colour is `Theme::now`; the column resolver is `Axis::now_col` (presentation only); the line is drawn by
`RepoScene::draw_now` after links, before lanes, from the grid row under the rule. New snapshot `repo-now`.
**Phase 11 (Where is now?, Release 1.8)**: The owner approved
board 11 (canvas R5Bh9iX6bsn3jSu8F8D58C, boards 11a–11i) on 2026-11-10. Codex critiqued the plan (10 points, 8
accepted, 2 in part; PLAN.md Review log). Key rules: one `now` colour (#E58AE5 / 176) used only for now; repo view
draws `▼` + `┃` at the now column (paint order: after links, before lanes; now label first); step graph marks live
parts with a heavy border and `● now`, finished parts' borders and `●` dim (text keeps its colour, unlike 11d).
A Claude part is live only if it is its session's last part, the session is `Busy` and its last activity is within
5 min; a Codex part only if `Working` and its child's last part. The CLI worker computes liveness (`Refresh::graph_live`).

**Phase 10 is closed (2026-11-10): Step 10.2 is committed and the owner confirmed it ("done").** The owner reinstalled
`~/.cargo/bin/multi-link`, ran the one command its `hook check` printed, then started and `/exit`ed one Claude session.
Real settings now: all five events `ok` from the installed binary (exit 0), every hook on `~/.cargo/bin/multi-link`,
backup `settings.json.multi-link-backup-20261110T104151Z-…` beside them; the post-merge session shows `finished` /
`session_end`. README "Install" / "Hook setup" use `hook check`; `docs/design.md` has "Phase 10 decisions" (H0–H7).
Next is the backlog (PLAN.md): `cd`-then-`resume --last`, ambiguous tie-break, lost key on resize, board 11, live grid.
Owner-facing manual steps must be one ready-to-run command, never hand-edited JSON. The owner pushes `main`.

**Step 10.1 (2026-11-09, Codex, mode Codex loop) is committed: `multi-link hook check` and `--merged`. Step 10.2 is
next (Claude): README "Install" switches to `hook check` + its printed command; limitations get H4's whitespace and
strict-number note, H6's millisecond race and H7's "one file" meaning; `docs/design.md` gets H0–H7 as "Phase 10
decisions"; then stop and ask the owner to run the command printed by the installed binary
(`cargo install --path crates/cli` first, then `~/.cargo/bin/multi-link hook check`), start and `/exit` one Claude
session, and confirm `hook check` exits 0, a backup sits beside the settings, and that session shows `finished` /
`session_end`.** The snippet now lives once in `multi_link_hook::settings::snippet`; check/merge/refusals are pure
(`settings::check`), file I/O and the command are in `crates/cli/src/hook.rs`. serde_json `preserve_order` is on
workspace-wide (keys unchanged). Real read-only check with `target/release/multi-link`: four events `ok`,
`SessionEnd` `outdated` (`~/.cargo/bin/multi-link` vs `target/release`); run from `~/.cargo/bin` the other four will
show `outdated` instead, which is the merge 10.2 wants. Board 11 remains in the backlog awaiting approval.
Owner-facing manual steps must be one ready-to-run command, never hand-edited JSON. The owner pushes `main`.

**Phase 9 is closed (2026-11-09): the owner confirmed it on real data** ("looks good") after reinstalling and adding
`SessionEnd` to `~/.claude/settings.json`.

**Step 9.7 (2026-11-08, Claude step, no Codex) is committed: Phase 9 real check, README, re-shot images. Phase 9 waited
on the owner's check (done 2026-11-09, above)** (stop-and-ask in `docs/phase9-check.md`): reinstall, re-merge the hook snippet with `SessionEnd`,
confirm the need. Real counts at one clock (`--now 2026-11-09T00:34:07Z`, 6a2c306 → HEAD): Codex sessions 223 → 425;
Claude `unknown` 232 → 3, `finished` 0 → 229 (all `transcript_exit`; `session_end` 0 until the snippet is merged);
unlinked hand-offs 124 → 47 of 222 → 221; exact 28 → 44; ambiguous 1 → 9 (all 8 new from `.zst` runs on 2026-10-30,
two runs in one checkout within 60 s). R7's 13 real `resume --last` hand-offs had their turn 0.8–2.71 s later.
Follow-ups noticed, not done: ambiguous tie-break (nearest run far ahead of the rest, or a run already matched
elsewhere); `cd`-then-`resume --last` stays unlinked (3 real).

**Step 9.6 (2026-11-08, Codex, mode Codex loop) is committed: fix rounds from `codex exec resume --last` (R7).
Step 9.7 is next** (real check, README, owner confirm). Claude wrote the fixtures and keys first (`aea1002`), reviewed
one round (2 findings, both fixed) and re-ran Verify: 580 passed / 0 failed / 1 ignored; clippy, fmt clean; `check.sh 6`
47/47 `ok`; no key edited by Codex.
- **9.6 / R7:** `recognise` classifies `codex exec [opts] resume [opts] --last` as `Resume` with no id (exec options may
  precede `resume`; a quoted `--last`/`resume`, or `--last` after the prompt, is not a flag). `Rollout::real_turns`
  (shared import predicate with `read_details`). `link::resolve`: an id-less resume takes candidates from runs in the same
  checkout whose real turn, not the first, starts 0–10 s after the hand-off (exact-launch claims ignored, no 60 s fallback,
  `dir_moved` suppresses); one → `heuristic` / `resume_turn`. Option-first `resume <uuid>` stays `Exec` as before.
- **Fixtures (ids `96…`, times 03:00–03:36Z, outside the 2 h lanes window):** repo-p9r (graph `graph-2.1.json`), p9s (late),
  p9t (ambiguous, `96c00005` manual run), p9u (imported then real). `check.sh 6` adds them. No existing key changed.
- **Real data (counts only, `--now 2026-11-09T00:26:54Z`):** unlinked 53 → 47 (5 now `resume_turn`, 1 was this session's
  own `codex exec … resume --help` probe, now help-only). Step 8.7's three Codex parts were already `exact` /
  `redirect_file` before 9.6 (from 9.3); implement 21:29:59–21:32:05. 9.7 should count the 3 `--last` hand-offs that stay
  unlinked.
Quota check for 9.6: `primary 75%, secondary 30%, CODEX: OK` (82% before the fix round).

**Step 9.5 (2026-11-08, Codex, mode Codex loop) is committed: Codex lane's step from all-time hand-offs (R6).**
Claude reviewed one round (3 findings, all
fixed) and re-ran Verify: 570 passed / 0 failed / 1 ignored; clippy, fmt clean; `check.sh 5` 46/46 `ok`; keys unchanged.
- **9.5 / R6:** the TUI worker's existing all-time pass fills `Refresh::codex_steps` (Codex id → step of its latest
  tagged incoming hand-off from a same-repo parent, max `(at, position)`). `s` on a Codex lane uses that entry first,
  then window `own_step`, then `no step on this session`. Claude lanes unchanged. Worker-to-App tests in
  `crates/cli/src/tui.rs` (`phase9_codex_lane_*`) cover the three `lanes.json` lanes.
Quota check for 9.5: `primary 68%, secondary 29%, CODEX: OK`.

**Step 9.4 (2026-11-08, Codex, mode Codex loop) is committed: `SessionEnd` hook record (R5).**
Claude reviewed the diff (no findings, no fix round),
re-ran Verify (565 passed / 0 failed / 1 ignored; clippy, fmt clean; `check.sh 4` 46/46 `ok`, keys unchanged) and ran
the sandbox PTY `/exit` check (one `ended` line, `prompt_input_exit`; log entry below).
- **9.4 / R5:** `hook record` turns `SessionEnd` into one `{"kind":"ended","reason":…}` line (no polling). The core
  reader accepts `ended`; `Records::ended(id, last_turn_at)` takes the latest `ended` by `(at, line)` and drops it when
  a later own main-transcript turn or any later valid hook record of that id exists (late async lines too).
  `sessions::assemble`: session record (and so `process_exit`) → `session_end` → `transcript_exit` → hook `waiting`.
  New `Transcript::last_turn_at` (scope amendment in PLAN.md's Review log) uses the 9.0 turn definition.
- **Snippet:** install/uninstall print `SessionEnd` with `async: true`, `timeout: 60`. The owner re-merges it into
  `~/.claude/settings.json` at 9.7; until then real sessions get no `ended` records.
- Sandbox `repo-a/.claude/settings.local.json` now holds the 9.4 snippet pointing at `target/debug/multi-link`
  (git-ignored in the sandbox; harmless if the binary moves).
Quota check for 9.4: `primary 60%, secondary 28%, CODEX: OK` (63% before the second Codex round).

**Step 9.3 (2026-11-08, Codex, mode Codex loop) is committed: hand-off child from the redirect file (R4).** Claude reviewed one round
(5 findings, all fixed) and re-ran Verify: `check.sh 3` 32 `ok`, workspace 554 passed / 0 failed / 1 ignored.
Quota check supplied for 9.3: `primary 44%, secondary 25%, CODEX: OK`.
- **9.3 / R4:** Shared launch-segment redirect parsing and descriptor-based safe reads serve both hook
  and core. Non-blocking open, regular-file fstat, opened-object path containment, 64 KiB cap, exactly
  one `session id:` line in the bounded read. Round 1 adds pre-open containment and nlink == 1;
  truncating forms read a 64 KiB prefix, append forms refuse oversized logs before/after reading.
  Relative targets refuse earlier cd/pushd/subshell; expansions and multiple
  stdout redirects are refused. Hook writes null first, polls up to 25 s, appends at most one child;
  unsafe files stop immediately. Persisted `redirect_file` records remain usable after log removal.
- **9.3 verification (round 1):** 554 passed, 0 failed, 1 ignored; Phase 9 `check.sh 3`: all 32 lines `ok`;
  clippy/fmt clean. Real release exact links: 28 at `6a2c306` → 38 (10 `redirect_file`), counts only
  from scratch, no agent-data writes. No answer-key edits, staging or commits. Full commands below.
Run `python3 -I tools/codex-quota.py` before any later `codex exec`.
- **9.2 / R2–R3:** Own main-transcript `cost-state` after every turn gives `finished` / `transcript_exit`
  when no session record exists, before hook waiting. Records and process-exit rules retain precedence.
  Last own string `entrypoint` and app-record mappings reach CLI and TUI reopening through
  `Scanner::open_resolver` / `open::Resolver`; no Session JSON key was added. CLI/sdk exits resume;
  desktop exits need exactly one safe app mapping and cannot fork; missing/other entrypoints are unavailable.
- **9.2 verification:** Phase 9 `check.sh 2`: all 32 lines `ok`; `cargo test --workspace`: 537 passed,
  0 failed, 1 ignored. All 45 formerly failing tests and the five owned test updates pass. Clippy/fmt clean.
  Real release counts at the same fixed clock: 241 Claude sessions in both runs; unknown 231 → 3,
  finished 0 → 228; open actions resume 0 → 30, app_link 10 → 133, unavailable 231 → 78.
  Full entrypoint/action splits, commands and actual output are in the 9.2 log below. Scratch archive baseline
  is `531aeeb`; no agent-data writes, answer-key edits, staging or commits.
- **9.1 / R1:** `multi_link_codex::open_rollout` is the common reader for discovery/scan and graph details;
  `rollout_paths` supplies both discovery paths. Pure-Rust `ruzstd` decodes `.jsonl.zst` with a 64 MiB
  decompressed limit; plain reads stay uncapped. Plain counterparts win, with metadata-id deduplication
  for differently named copies. Failures count as `unreadable`; scanner file-key caching is retained.
- **9.1 verification:** Phase 9 `check.sh 1`: all four lines `ok`. Workspace: 488 passed, exactly the 45
  documented failures below, 1 ignored; newly failing tests: none. Excluding those 45: 488 passed, 0 failed,
  1 ignored, 45 filtered out. Fmt and clippy clean. Real release run: 419 Codex sessions; 212 hand-offs =
  54 unlinked, 121 heuristic, 28 exact, 9 ambiguous; both stderr empty. See the log for commands and baseline.
What later steps need:
- **Fixed clock and root:** `--root fixtures/sessions-phase9 --now 2026-11-09T12:00:00Z` (`--since all`; lanes use the
  TUI's 2 h window). Keys: `fixtures/expected/phase9/{sessions,links,open,graph-1.1,lanes}.json`. They are the *final*
  Phase 9 output. Every id's first two hex digits name its step (`91…`–`95…`).
- **How each step checks its keys:** `fixtures/expected/phase9/check.sh <N>` (from the repo root) compares the cases of
  steps 9.1–9.N with `cargo run -q -p multi-link --` and the fixed `--now`: sessions and links filtered by id prefix
  `^9[1-N]` (links by `parent`), `graph-1.1.json` whole, stderr exactly `unreadable: 1`, and from N=2 every `open` /
  `--fork` dry-run in `open.json` (exit code and action). From N=4 it also compares the whole sessions and links keys.
  `lanes.json` is for 9.5's worker-to-App test. At 9.0 (no code) every Phase 9 case fails and every unchanged case
  passes (`check.sh 5`: links and open for the unchanged ids `ok`). Keys are Claude's: never edit them.
- **Fixture files** (`fixtures/sessions-phase9/`, all generated synthetic): Claude transcripts in
  `.claude/projects/-home-dev-{repo-p9a,repo-p9c,repo-p9e,repo-p9l,repo-p9m,repo-r1…repo-r14}/<id>.jsonl`; records
  `.claude/sessions/{92009,92010,94003,94006}.json`; Codex rollouts in `.codex/sessions/2026/11/09/` (`91c00001` only
  as `.jsonl.zst`; `91c00002` as both; `91c00003` a corrupt `.jsonl.zst`); hook files `.multi-link/hook/{9300000b,
  92000011,92000012,94000001,94000002,94000004,94000006,94000007}-….jsonl`; app records under `Library/Application Support/
  Claude/claude-code-sessions/00000000-…94a0/` (`local_92f00002`, `local_92f00004`, `local_92f000a4`); redirect logs
  under `tmp/dev/p9/`, `home/dev/elsewhere/`, `home/dev/repo-r6/`, `home/dev/repo-r8/logs/`; plans
  `repos/{repo-p9a,repo-p9l,repo-p9m}/PLAN.md`. The case table is in `docs/output-schema.md` → Phase 9 fixture results.
- **Contract choices 9.0 made (now binding):** open reasons `app_session` (desktop, no/two mappings, or fork) and
  `unknown_entrypoint`; `process_exit` keeps resume regardless of entrypoint; `state_since` `null` for both new
  `finished` rows; a *turn* = `assistant`, or non-meta `user` that is not a local-command echo (`tool_result` counts);
  `cost-state` position is file order; hook `waiting` comes after `transcript_exit` and `session_end`; `ended` has
  a `reason` key that other kinds do not; fixture-mode redirect reads are root-relative with containment in
  `<root>/<cwd>`, `<root>/tmp`, `<root>/private/tmp` (`$TMPDIR` ignored).
- **Existing keys changed under R2 (12 `fixtures/sessions` transcripts end with a `cost-state` after their last turn;
  all `claude-desktop`, none with a record):** 07c80c2e, 0d7faed4, 11bf1b58, 11c1f13c, 80cce3e4, 85a715ba, 9d9a0ed8,
  bb792d9c, c86d374f, df403bfa, ef72378c, f4d83303. Values (each `unknown`/`no_signal` → `finished`/`transcript_exit`
  unless noted): `sessions.json` 12; `overview.json` 12; `hook/sessions.json` 12 (11c1f13c was `waiting`/`hook`,
  `state_since` 2026-10-28T10:03:11.004Z → `null`); `hook/sessions-2h.json` 9 changed and 11c1f13c removed (no longer
  `waiting`, last line before the window); `hook/overview.json` 12, repo-c `tier` `waiting` → `busy`, repo-c rows now
  `…0010`, 6358ec7e, 11c1f13c, 85a715ba. TUI goldens `home-hook{,-full}{,.styles}.txt` follow (`◆ 1 need you`, repo-c
  `2 working now`, no waiting card row; render of the key, deterministic). `home-full` is unchanged.
- **Historical 9.1 failures, all resolved by 9.2 (45; at 9.1: 488 pass, 1 ignored):** 42 only because `transcript_exit`
  is not yet a `StateEvidence` value or the state differs from the code (they pass once 9.2 implements R2 and R3):
  cli `overview_json_matches_answer_key_and_text_has_one_line_per_repo`, `duration_units_and_offsets_use_the_same_options`,
  `phase_one_json_matches_answer_keys`, `hook_conflicts_are_stderr_only_for_sessions_links_and_overview`; claude
  `fixtures::transcript_fields_and_skips_match_answer_key`; codex `fixtures::fixture_sessions_turns_and_skips_match_answer_key`;
  core `scanner::tests::warm_refresh_caches_every_file_kind_and_recomputes_state`, `hook::hook_variant_equals_all_three_answer_keys`,
  `overview::every_overview_matches_its_key`, `sessions::since_two_hours_filters_only_transcript_history`,
  `sessions::fixture_report_equals_every_expected_field`; model `model::every_expected_report_round_trips`; tui lib
  `app::graph_tests::*` (16) and `app::tests::*` (7: bell_only_on_entry…, checking_message…, each_new_waiting_id…,
  failed_refresh…, footer_prioritizes…, open_results_survive…, refresh_preserves_selected_full_id…); tui
  `home::{header_at_100_keeps_working_now_before_shared_checkouts, home_full_snapshot_100x80, home_hook_full_snapshot_100x80,
  home_snapshot_100x30, home_hook_snapshot_100x30, shared_checkout_warnings_have_a_red_header_and_card_row}`, `theme::no_color_preserves_home_and_timeline_glyphs_and_visible_tracks`.
  **5 owned test updates applied in 9.2 (all inside 9.2's Files; identified in 9.0):** 3 of the 45,
  cli `text_has_one_row_per_item_and_a_footer` (the `STATE` column widens to `finished`) and the two key-snapshot
  guards below, plus 2 that pass today and 9.2's code will break unless edited: cli
  `open_dry_run_transcript_only_claude_and_codex_are_unavailable_exit_three` (0d7faed4 → `app_session`, same command and
  cwd, exit 3) and core `open::tests::fixture_table_never_resumes_and_matches_schema_coverage` (the 12 are `finished`
  and `app_session`; the root still never resumes). The guards in `crates/core/tests/`:
  `overview::older_keys_only_add_made_and_hook` (`overview-before-phase5.json`) and
  `phase6::every_existing_json_key_preserves_committed_values` (`keys-before-phase6.json`). Those snapshots copy old
  key values; 9.2 may change in them **only the values listed above** (Claude reviews that the diff equals this list),
  because 9.0's Files did not include them. `cli::open_lookup_no_match_ambiguous_and_old_ids` keeps passing under R3.
- **9.1's "tests clean"** therefore means: every test outside the 45 passes. The hook variant no longer has a
  `waiting`/`hook` row; the Phase 9 root carries it (92000012) plus the precedence case (92000011).

**Step 9.6a (2026-11-08, Codex) is merged into main (merge commit 226f5e3, which also carries Step 9.0's R7 docs).**
R8 sets home from the first known record with cwd belonging to the session (sessionId absent or equal to
activity_id). Before home exists, inherited records retain the previous metadata behavior; afterward,
outside cwd/branch metadata is ignored. Session `85a715ba…` keeps repo-c as its unchanged answer key requires.
Workspace tests: 525 passed, 0 failed, 1 ignored; clippy/fmt clean. The worktree release real run passes both
requested checks (log below). All answer keys and PLAN.md remain untouched by Codex.

**Phase 8 is closed (2026-11-08): the owner confirmed the step graph on real data** ("it looks good", then "Yes, close
Phase 8"), after reinstalling with Step 8.7. Nothing after Phase 8 is planned in PLAN.md yet. Candidates, in the
coordinator's recommended order (owner to choose): fixes (clean-exit resume from `.claude/handoff/backlog-findings.md`,
T10 redirect-file child, the 8.5 Codex-lane window limit, reading `.zst` rollouts, re-shooting `docs/img/**` for the
12-hour clock); then the live grid (`.claude/handoff/proposal-live-grid-and-pane-jump.md`, needs board 10 and Q3/Q4/Q6);
then the pane jump only if the owner runs `claude` in tmux or Herdr (Q1). `main` is not pushed (the owner pushes).

**Step 8.7 (2026-11-08, Codex loop) is committed: every TUI clock is 12-hour (`9:05pm`, `8:05:43am`, `12:00am`).**
One helper, `view::clock_label(at, with_seconds)`, feeds `view::clock`, `RepoScene::clock` (axis ticks, header ranges)
and `graph::clock`. JSON and CLI text are unchanged (UTC). All 8 TUI golden pairs were regenerated at this step's commit;
later steps pin them there. Wider labels drop some axis ticks under P4 and cut two long lines (see the 8.7 log entry).
`docs/img/**` predates 8.7 (README says so).

**Step 8.6 (2026-11-08). Step 8.6 is committed; the owner check it asked for is now done (Phase 8 closed, above).** 8.6 ran in
the main checkout as a Claude step (no Codex, no worktree). The owner must reinstall (`cargo install --path crates/cli
--force`; the installed binary predates the graph), run `multi-link tui`, press `s` on a repo-view lane and confirm
the step graph meets need 4. Record the answer in `docs/phase8-check.md` → "Owner check" and here. A "no" with notes
becomes amendment steps, as 8.4a–c were.
- **Finding for the next planner (not fixed, Phase 8 non-goal):** most "lost" Codex rollouts look compressed, not deleted.
  `~/.codex/sessions` held 202 `rollout-*.jsonl.zst` (folders 2026-10-16 to 10-30) next to 211 plain `.jsonl` on
  2026-11-08; multi-link reads only `.jsonl`. 81 of 116 unlinked hand-offs have a `.zst` rollout whose file-name time
  is 0–10 s later (one confirmed by content, step 3.7); 17 have a plain one (likely link misses), 18 neither. Reading `.zst` (a decompress dependency; who compresses was not investigated) is a candidate
  backlog step. PLAN.md's "Codex rollouts disappear" line is out of date.
- Also seen: a `codex exec resume --last` fix hand-off is unlinked and its turn lands in the resumed run's part (step
  6.6), while the panel says `(the rollout may be gone)`; candidate wording/link fix for a later step.
- Real step 6.7 matches board 9's layout; differences (G5/G6 counts, W5 minutes, wording) are in `docs/phase8-check.md`.

**Step 8.5 (2026-11-08).** **Step 8.5 is committed (Claude-only mode, merged ff-only into `main`; branch
`step/8.5` and worktree `.claude/worktrees/step-8.5` left in place); Step 8.6 closes Phase 8.** Run
`python3 -I tools/codex-quota.py` first (at 8.5's start: 95% of the 5-hour window, `CODEX: LOW`, resets Sun 01:48;
weekly 18%). What 8.6 needs from 8.5:
- **How to reach a graph:** `multi-link tui` → `enter` on a repo → select a lane → `s`. Own tagged hand-off: its latest
  step opens. A Codex lane uses its latest tagged incoming hand-off whose parent is in the same repo group. A coordinator
  with none shows `started: step X, step Y · ←→ choose · enter opens step X · esc cancel` (direct started children,
  all-time, from the worker's `Refresh.started_steps`). Otherwise footer `no step on this session`. In the graph: `←→`
  (`↑↓` below 120 columns) move, `[ ]` walk `Refresh.graph_steps` (PLAN.md steps with a tagged hand-off from a Claude
  session in the repo, then tagged ids not in PLAN.md by first hand-off), `o`/`f` open the part's session (footer words
  `no codex run found` / `N possible runs` / `run unavailable` otherwise), `esc` back to the repo view.
- For real renders without the TUI, `multi-link graph --repo R --step ID --json` plus the `home_html --graph` example
  (Step 8.4 notes below) still work and give the same report the app draws.
- **G11 (real data, release build, warm scanner):** the whole refresh with step 8.4's graph open (the largest real step:
  25 tagged hand-offs, 51 parts) takes 229–236 ms (overview 32–33 ms + graph 160–164 ms + all-time pass 34–39 ms). The
  first, cold refresh after start takes 6.4 s, almost all of it the overview's first parse (which predates Phase 8).
  Before 8.5's fix the warm graph alone took 4.7–4.9 s.
- **Known limits for the README (8.6):** a Codex lane whose Claude parent is outside the TUI window (default 2 h) says
  `no step on this session` (review round 2, parked: needs a background hand-off whose parent goes quiet first);
  rapid `[ ]` builds each intermediate graph (~230 ms each); with a graph open a refresh runs the all-time pass twice
  (once inside `Scanner::graph`); the repo footer's `q quit` no longer draws at 101–109 columns; the `o`/`f` rule is
  restated in `App::open_part` (same expression as the panel in graph.rs).
- 519 tests pass, 1 ignored; clippy and fmt clean.

**Step 8.4c (2026-11-08).** **Step 8.4c is committed (Claude-only mode, merged ff-only into `main`; branch
`step/8.4c` and worktree `.claude/worktrees/step-8.4c` left in place); Step 8.5 is next.** Order: 8.5 → 8.6. Run
`python3 -I tools/codex-quota.py` before every `codex exec` (at 8.4c's start: 95% of the 5-hour window, `CODEX: LOW`,
resets Sun 01:48; weekly 18%). What 8.5 needs from 8.4c (its G9 amendment uses `started_by`):
- `Session.started_by` / `SessionRow.started_by: Option<String>` (parent's transcript id), serialised right after
  `title`. Set only between transcript-backed Claude sessions that are both listed; cross-repo children keep the value
  but stay top level. Source: `multi_link_claude::{AppRecord, read_app_record, scan_app_records}` reading
  `<root>/Library/Application Support/Claude/claude-code-sessions/*/*/local_*.json`; the core scanner caches them by
  file key and fills `started_by` in `core::sessions`. Cycles are found on the full record graph before the `--since`
  filter. A record with no (or null) `cliSessionId` is ignored; malformed / wrong-typed → `bad_records`.
- Overview `children` is now a tree of any depth mixing Codex children and started Claude children in one ascending
  list (Codex key = placing hand-off `at`, started key = `started_at`, ties by id). A child is a started child iff its
  harness is claude. Quiet merge eligibility is recursive. **G9 for 8.5: "direct started children" = a lane's
  `children` with harness claude.**
- TUI: `view::subtree` walks every depth (bell, selection, `o`/`f`, home attention, selection kept by id across a
  reparent; a vanished row falls back to its nearest listed ancestor). The repo view keeps W6's newest-first order at
  every level, so it draws children in the reverse of the JSON order (ruled; PLAN.md Review log). Indent 2 per level,
  clamped so text ends before the bar column (deep chains past ~15 levels share one column; deep sub lines truncate).
- Fixture root `fixtures/sessions-started` (`--now 2026-11-08T12:00:00Z --since all`), keys `fixtures/expected/started/
  {sessions,links,overview}.json`. Coordinator K = `84c00001-…`; its direct started children A (`…02`, has an exact
  Codex child and started A2) and B (`…03`). 8.5's `s` test on K can build on it (A's hand-off is tagged step 2.1;
  the root has no PLAN.md, so G1 graphs need one added or a test-made copy).
- Known limits: `home::selectable` (exported, no callers) still stops at depth 1 (parked, outside Files); the plain
  `overview` text count is now recursive (`crates/cli/src/main.rs`, added to the Files line in review).
- 490 tests pass, 1 ignored; clippy and fmt clean.

**Step 8.4b (2026-11-08).** **Step 8.4b is committed (started in the Codex loop; Codex quota went LOW
mid-step, so the review fixes and both reviews ran Claude-only); Step 8.4c is next.** Order: 8.4c → 8.5 → 8.6. Run
`python3 -I tools/codex-quota.py` before every `codex exec` (at 8.4b's end: 95% of the 5-hour window, `CODEX: LOW`,
resets Sun 01:48; weekly 18%). If still LOW, 8.4c runs in Claude-only mode. What 8.4c needs from 8.4b:
- `title: Option<String>` sits on `Transcript` (claude crate), `Session` and `SessionRow` (model), between `last_asked`
  and `imported`. 8.4c's `started_by` is a second additive key; its key check is `git diff <8.4b commit> --
  fixtures/expected ':!fixtures/expected/started' …` and should also exclude `fixtures/expected/tui` if a home golden
  changes (8.4b's amendment, PLAN.md Review log; 8.4c's goldens rule says every home golden unchanged, so it may not).
- Display-only fields, `#[serde(skip)]`: `SessionRow.place` (U6, from `core::repo::place`, memoised git per checkout,
  refreshed when `.git/HEAD` changes; `None` in fixture mode) and `RepoGroup.latest_tagged_handoff` (U5). A report
  loaded back from JSON loses both; the TUI's `view::place` then falls back to `branch`.
- TUI helpers in home.rs: `plan_fit(group, width)` (cuts the step title before ` · latest hand-off step <id>`),
  `place_fit(place, width)` (`worktree x` › `wt x` › `wt x…`). Row label: `view::session_label_for` (title first).
- Real data at `--now 2026-11-08T23:00:00Z --since all`, 411 overview rows: labels before (8.4a) 212 prompt / 91 step /
  108 no-prompt; after 179 title / 40 step / 92 prompt / 100 no-prompt; 61 rows have a preamble prompt; 11 rows run in
  a linked worktree.
- Known limits (review nits, not fixed): at header widths ~30–38 `plan_fit` can leave `… · latest hand-off step X`
  (step id lost); the suffix also shows when no step is logged (`no PLAN.md · latest hand-off step X`, untested);
  the removed-worktree fallback is path-based (`.claude/worktrees/<name>` anywhere in the cwd); control characters in
  a title are removed, not replaced by a space (U1 literal); `.git` as a file never refreshes the memoised branch.
- 447 tests pass, 1 ignored; clippy and fmt clean.

**Step 8.4a (2026-11-08).** **Step 8.4a is committed (Codex loop, review clean, no fix round); Step 8.4b
is next.** Order: 8.4b → 8.4c → 8.5 → 8.6 (owner check closes Phase 8). Mode: the Codex loop is the default; before
every `codex exec` run `python3 -I tools/codex-quota.py` (CLAUDE.md, "Mode switch"; at 8.4a hand-off: 81% / 16%,
`CODEX: OK`). What 8.4b needs from 8.4a:
- U3 lives in `claude_state` (crates/core/src/state.rs): `busy` + `Liveness::Alive` → `busy` at any age; Codex's open-turn
  rule still uses the 300 s window (U3 is Claude only). U4 is `help_only` in crates/claude/src/invocation.rs; its
  value-option list is the step's five plus long forms `--config`, `--output-last-message`, `--cd`, `--sandbox`.
- Known limits of U4 as specified (not bugs, not fixed): `codex exec resume <uuid> --help` is still linked (the id is
  the first non-option word); value options outside the list (`-m`, `-p`, `-i`) are not skipped, so `codex exec -m X
  --help` is still linked. Not counted in real data.
- Real data at `--now 2026-11-08T12:00:00Z`: 213 → 202 links, all 11 dropped are help-only; unparented 112 → 114.
  8.4b's own before/after should be taken on the 8.4a build.
- 440 tests pass, 1 ignored; clippy clean; `git diff --exit-code fixtures/expected` exit 0.
Earlier notes that still apply:
- **Why the amendment:** the owner watched 8.4 run in `multi-link tui` and could not tell rows apart (shared preamble
  labels, a busy session shown `stale`, `codex exec resume --help` linked, header on the wrong step). The phase intro
  in PLAN.md now amends G0 and the whole-phase Verify: from 8.4a on, the pinned `git diff` is replaced by each step's
  own "only these lines changed" check.
- **8.4c's source is the desktop app's per-session records** (`~/Library/Application Support/Claude/
  claude-code-sessions/*/*/local_*.json`: `cliSessionId`, `spawnedFrom`; undocumented, they survive exit). Found while
  planning; counts in the step. In 8.4c the step session (Claude) writes the fixture and answer keys before the Codex
  hand-off.
- 8.5 gained a G9 amendment (`s` on a coordinator lane lists its direct started children's steps) and now depends on 8.4c.
- Facts from 2026-11-07, not yet used by a step: Claude transcripts carry `custom-title` (47 of 60; 8.4b uses it) and
  keep `gitBranch` at the start branch inside a worktree (8.4b's U6). Codex rollouts carry `rate_limits` (read by
  `tools/codex-quota.py`). `codex exec resume --last` filters by cwd (`--all` disables it). Step 8.4's own graph: 16
  parts, Codex calls misnamed `fix round 1–7`, 6 of 8 runs unlinked (wrapper script and a `cd`); a known limit for
  8.6's README. The owner's installed `multi-link` binary predates the graph command.
- Waiting on the owner, not blocking: `.claude/handoff/proposal-live-grid-and-pane-jump.md` (board 10, Q1–Q7, a Phase
  9), `.claude/handoff/backlog-findings.md` (clean-exit resume, T10, resize key; draft steps, fixes untested), mockups
  canvas https://claude.ai/artifact/FvyrvdUYqSnUdpYG7t9Uaj.

**Step 8.4 (2026-11-07).** **Step 8.4 is committed (hybrid mode, stopped after Task 5; see its entry).** What 8.5
needs from it:
- Render API (crates/tui/src/graph.rs, re-exported from lib.rs): `render_graph(&GraphReport, &GraphOptions, Rect, &mut
  Buffer)`; `GraphOptions { selected, first_slot, theme, offset }` (Clone, Copy, Debug, PartialEq; Default = 0, 0,
  truecolor, UTC); `graph_slots(&report)`; `graph_first_slot(&report, selected, first_slot)`. For `←→` and a refresh that
  keeps the selection, store the slot `graph_first_slot` returns back into `first_slot`. The narrow list (< 120 columns)
  recomputes its scroll from `selected` on each render, so `↑↓` only changes `selected`.
- Theme gains `graph_selected`, `codex_box`, `claude_strip`, `codex_strip` (G8 palette additions, truecolor and 256).
- The footer already reads `←→ move along the step   [ ] prev / next step   o open   f fork   esc repo view   q quit`
  (`↑↓` below 120 columns; when hints must be cut, `esc` and `q` are kept before `[ ]`, `o`, `f`). `o`/`f` are drawn only
  for a part with a run and a non-empty session; 8.5 must apply the same rule to the keys.
- **Render readings decided in review (beyond PLAN.md/design.md wording; those files were outside the step):** edges draw
  only into a visible box (a return may start left of the window); strip row 34 uses the earliest `from` / latest `to`,
  like row 1; too many parts for 1 cell + gap each: the strip keeps its title and times, no cells; short screens: the last
  two rows are always rule + footer, the panel's first line sits just above the rule while a row is free below the header,
  the legend stays while it fits, above 40 rows the 40-row layout stays; narrow: keeps ` (reported)`, edge glyphs `╎ │ ▲`,
  rules and a right-aligned `times UTC±H`; empty `asked`/`said`/session count as absent; a negative gap drops `started …
  after the call`; `stale since HH:MM` shows the run's `to` (last record), as the repo view does. A planner should copy
  these into G8/G10 at the next PLAN.md edit.
- `example_draws_graph_html` runs the example binary next to the test binary; a full `cargo test` builds it, a filtered
  `cargo test --test graph` may run a stale one (said in the test).
- `cargo run -p multi-link-tui --example home_html -- --graph REPORT.json OUT.html [--size WxH] [--offset …]
  [--selected N]` renders a graph report to HTML (for 8.6's images).
- 438 tests pass, 1 ignored (`real_counts`, opt-in), clippy and `cargo fmt --check -p multi-link-tui` clean, pinned
  `git diff` exit 0, golden unchanged.

**Before Step 8.4 (2026-11-07).** **Step 8.3 is committed (Superpowers trial, done); Step 8.4 is next** (**hybrid
mode**, CLAUDE.md, owner 2026-11-07: Claude splits tasks and reviews, Codex writes each task test-first behind a
red/green gate, Claude commits on `step/8.4`; Superpowers is installed for Claude Code and Codex). What 8.4 needs:
- The report: `multi-link graph --repo R --step ID [--json]`; types `model::{GraphReport, GraphStep, GraphCommit, Part,
  PartStatus, Counts, Notable, Edge}` (crates/model). `Edge` is serde-tagged `kind: handoff|return`. Build it in tests with
  the CLI command in output-schema "Phase 8 fixture results"; all 13 keys match (`crates/cli/tests/graph.rs`).
- Core API: `multi_link_core::graph::{build, GraphInput, CodexDetails, commit_for_step}` and `Scanner::graph(root,
  options, repo, step, plans)` (in `crates/core/src/graph.rs`, not scanner.rs). The TUI depends on model only, so 8.4
  renders a `GraphReport` value; 8.5 wires `Scanner::graph` into the refresh.
- `Scanner::graph` calls `multi_link_claude::scan(root)` once per build for the parents' `Transcript`s (a full parse, no
  cache). 8.5 must measure it against G11's 2 s; a cached or per-file path needs the claude crate.
- Text mode (no `--json`) is plain lines with no golden; it prints the same facts as the JSON.
- **Readings added in review (not in PLAN.md's or design.md's wording; those files were outside the step):** see the 8.3
  entry's "Readings added in review". The one that changes the picture: **no Claude part covers a run of its chain that is
  still going**; a Claude part after a hand-off starts at the latest end of earlier still-going runs (with a return edge
  there). A planner should copy all five into G2/G6 at the next PLAN.md edit.
- Golden `step-phase8.txt` / `.styles.txt` are 8.4's target; do not edit them. A mismatch blamed on a key or the golden is
  reported, not fixed.
- 358 tests pass, 1 ignored (`real_counts`, opt-in), clippy clean, pinned `git diff` exit 0.

**Before Step 8.3 (2026-11-06).** **Step 8.2 is committed; Step 8.3 is next** (Superpowers trial, rules in CLAUDE.md;
stop and ask if Superpowers is not installed). Its baseline measures are in the 8.2 entry. What 8.3 needs:
- Readers, on demand only: `claude::calls(root, transcript)`, `codex::calls(path)`, `codex::codex_turns(path)`, and
  `codex::read_details(path)` for both in one read. Types `model::{Call, CallKind, Calls, Turn}`; `Calls.skipped` holds
  skipped records. A missing Codex or main Claude file is an I/O error, which is how G4's `run unavailable` is seen; a
  missing subagent file is skipped.
- `Call.label` is the fixed category only. `▸ codex exec` is the bare hand-off label; 8.3 adds `· step` / `· continue`.
- **Reading added in 8.2's review (not in PLAN.md's or design.md's wording; those files were outside the step):** `kind`
  is by the first command word, but `label` looks at every simple command, `$ git commit` wins, else the first labelled
  one, and heredoc bodies are ignored. Real commits are all `git add … && git commit`, so G3's `verify + commit` needs this.
  A planner should copy this into G5 at the next PLAN.md edit.
- **`Turn.prompt`** is the first `user_message` event, else the first filtered `response_item` user message. Real turns
  use only the second; the phase 8 fixture uses only the first.
- `ok` on non-Bash Claude calls is usually null: those tools omit `is_error` on success (real counts in the entry).
- `links` lists the fork's copied hand-off twice (same `invocation`, parent 8112 and 8113). 8.3 drops it by the pinned G1
  rule, joining `link.invocation` to the parent's `Call.id`. On real data 1 of 180 invocations is such a copy.
- Keys: `fixtures/expected/phase8/graph-<id>.json`, 13 ids, 59 parts. Golden `step-phase8.txt` / `.styles.txt` are 8.4's
  target; do not edit them. A mismatch blamed on a key or the golden is reported, not fixed.
- Two PLAN.md readings were pinned in design.md rather than PLAN.md: G2's "runs through it" and the part that holds a
  hand-off. If the planner disagrees, the keys for 1.2, 1.5 and 1.11 change.
- 296 tests pass, 1 ignored (`real_counts`, opt-in), clippy clean, pinned `git diff` exit 0.

**Workflow trial (2026-11-06).** The owner wants to try Superpowers: Step 8.3 only, rules in CLAUDE.md. Steps 8.1 and
8.2 use the normal loop, and 8.2 records the trial's measures as the baseline. Superpowers was not installed when
this was written; the owner installs it before 8.3. Implementers commit on `trial/step-8.3`, never on `main`.

**Before Step 8.1 (2026-11-06).** **Phase 8 (step graph) is planned.** The owner approved board 9
on the mockup canvas. Plan in PLAN.md (G0–G11, Steps 8.1–8.6; Codex critique logged, 9 accepted, 1 in part). The owner
kept the defaults: the need line, `s` from the repo view, no replay. Facts measured 2026-11-06 are
in the phase. Watch out: Codex rollouts on disk fell to 159 (295 sessions counted 2026-11-03), and unlinked hand-offs
rose 32 → 108; cause not investigated. Phase 8 is the number: the planning session's "Phase 7" went to the polish phase.

**Earlier (2026-11-06).** **Phase 7 is closed: the need is met.** The owner saw real renders of home
at 2h and 16h and the repo view at 4d and answered "Yes, close Phase 7". Verdict recorded in
`docs/phase7-check.md`. Phase 7 commits: ed043e0 (plan) through 9d58365 (7.5), all local; the owner pushes. No
next phase is planned. Board 9 (step graph target, mockup canvas) was reported as waiting on the owner's approval
in the plan-step-graph session; nothing from it is in PLAN.md, and if approved it becomes Phase 8 (that session
calls it Phase 7, a number now taken).

**Resume here (2026-11-03, latest): Step 7.5 is committed; Phase 7 is still NOT closed.** It waits on the
owner's stop-and-ask, which now covers Steps 7.1–7.5: the owner runs `target/release/multi-link tui` (and
`--since 16h`) and confirms, including that labels now quote each session's latest prompt. The coordinator
records the answer in `docs/phase7-check.md` → "Owner check" and here. What 7.5 changed:
- Session/SessionRow carry additive `last_asked` (P5); `asked` is unchanged. Labels: latest step › `last_asked`
  › `asked` › `· no prompt`. The selected line and `o` are unchanged.
- Real counts (counts only): Claude 186 sessions, `asked` null 11, `last_asked` null 14 (3 extra: only `isMeta`
  eligible text, so they fall back to `asked`), 89 differ; Codex 295, both null 154, 6 differ.
- 276 tests pass, clippy clean, the four 6.1 goldens unchanged vs 26af6f2, JSON equals 081cdcc minus `last_asked`.
- `docs/phase7-check.md` does not mention P5 yet (7.4 wrote it before 7.5); add a line when recording the answer.

**Before Step 7.5:**
**Resume here (2026-11-02, latest): Step 7.4 is committed except its last Verify item (the owner's answer);
Phase 7 is NOT closed.** It waits on the owner's
stop-and-ask: the owner runs `target/release/multi-link tui` (and `--since 16h`) and confirms the Phase 7 need.
The coordinator records the answer in `docs/phase7-check.md` → "Owner check" and here. What 7.4 found:
- Whole-phase Verify passes; real Claude count `[185,11]` (baseline `[181,43]`); the 11 are the plan's non-goal.
- Real 16 h multi-link repo view: three folds all show their length (`1h05`, `54m`, `2h56`) beside `21:19 now`.
  The two adjacent folds read `54m 2h56`; left as is (possible owner point).
- P3 is not visible on that day's real screens (every shown time was on the same local date); tests cover it.
- Real renders stayed in scratch (W10).

**Before Step 7.4: Phase 7 in progress; Step 7.3 is committed.** Next: Step 7.4 (Claude, real
check and owner confirm; ends with a stop-and-ask). What 7.4 needs:
- Repo axis (`draw_axis`, `crates/tui/src/timeline.rs`) reserves label cells in P4 order: fold lengths, end label
  (`HH:MM now` → `now` → nothing; plain `HH:MM` → nothing when the range ends before now), explicit-range interior
  now (`HH:MM now` → `now` → nothing), then ticks. Labels keep one blank cell between them (stricter than
  "covers nothing"; it stops words joining). Ticks whose clock equals the end or interior-now clock are skipped
  even when that label was shortened or dropped.
- Explicit ranges never fold, so on screen the fold/end collision appears only in fitted mode.
- No TUI golden changed in 7.3; the four 6.1 `*-phase6*` goldens are unchanged vs 26af6f2.
- For 7.4's write-up: P1 real count baseline `[181,43]` (7.1 has the after count), P2 home `last 16h`, P3 clock
  for the last 24 h, P4 as above. Real renders from scratch only (W10).

**Before Phase 7:**
**Resume here (2026-11-02, latest).** **Phase 6 is closed: the need is met.** The owner checked the real
screen and approved it with one change (Step 6.6 linked launches in subshells / after heredocs and fixed Codex
prompt labels; Step 6.7 added the `✓ done` label). Verdict recorded in `docs/phase6-check.md`. Phase 6 commits:
d4d862a (plan) through 0190090 (6.7), all local; the owner pushes. No next phase is planned; candidates are in
PLAN.md's Backlog (hook install for needs-you, step graph from board 3, live grid).

**Resume here (2026-11-02, latest): Step 6.7 is committed; Phase 6 is still NOT closed** (the coordinator records
the owner's verdict).
- Repo view: a finished Codex bar gets green `✓ done` after one blank cell when all seven cells (blank + label) fit
  inside the bar area without covering a bar, a link line or a fold band; otherwise the adjacent `✓`, as before.
  The label may cover a grid tick. Drawn in `draw_done` (`crates/tui/src/timeline.rs`) after all lanes.
- Real route-impact view (scratch only): `✓ done` shows after the 16:02 PST Codex run.
- The 6.1 `repo-phase6` pair is unchanged (its Codex bar ends at the right edge, so it keeps `✓`).

**Before Step 6.7:**
**Step 6.6 is committed; Phase 6 is still NOT closed** (owner check below).
- Recogniser (`crates/claude/src/invocation.rs`) now skips heredoc bodies (X1), splits on `(` `)` and drops bare
  `{` `}` (X2), and skips `nohup` / `env NAME=v…` / `timeout|gtimeout DUR` (X3). Real launches 124 → 149
  (unlinked 26 → 32: the newly recognised launches without a matching Codex run).
- Codex `asked` falls back to the first eligible `response_item` user message (X4); real 2h Codex rows with `asked`
  null 1 → 0. The route-impact 16:02 run is nested under its Claude parent with a link line (scratch render only).
- Still open from 6.5's list: Claude `no prompt` when the first record starts `<system-reminder>` (41 of 175 real
  main transcripts have no prompt: 37 wrapper-only, 4 no eligible text; rule unchanged by decision); home says
  `last 2h` whatever `--since`; fold label / duplicate `now` tick; times before midnight show only the date.

**Earlier (before Step 6.6):**
**Step 6.5 is committed; Phase 6 is NOT closed.** It waits on the owner's
stop-and-ask: the owner looks at the real screen (`target/release/multi-link tui`, and `--since 16h` for last night's
runs) and confirms. Record the answer in `docs/phase6-check.md` → "Owner check" and here. Real renders were shown from
scratch only (W10). Open points found on real data (docs/phase6-check.md, "Differences from the boards"), each a
candidate follow-up step, not fixed in 6.5 (outside its files):
- Codex `asked` is null on every real run: 2026-11-02 `codex exec` rollouts have no `user_message`; the prompt is in a
  `response_item` `message` with role `user` (crates/codex parser, W3). Lanes read `codex codex · no prompt`.
- Sessions started by another session have a first user record starting `<system-reminder>`, so `claude · no prompt`.
- Home says `last 2h` whatever `--since` is (home.rs:503/513/515/681); the repo view follows the window.
- The two newest multi-link Codex runs (done 00:11, 00:17 on Nov 2) have no parent link.
- Fold length label not drawn when it would touch `HH:MM now`; a tick equal to the `now` minute prints twice
  (`14:07  14:07 now`); times before local midnight show only the date.
- `home_html` now takes `--repo NAME` and `--since`; the repo view needs the same `--since` as the JSON.

**Earlier (2026-11-02): Step 6.4 is committed.** Next: Step 6.5 (Claude, visual check and README; ends with a
stop-and-ask to the owner). What 6.5 needs:
- Repo view is board 8c in `crates/tui/src/timeline.rs`: `render_repo(report, links, repo, window, selection, &RepoOptions, area,
  buf)` (`RepoOptions { fitted, fold, imports, theme, offset }`, all `pub` from `multi_link_tui`); `render_timeline` is a fitted
  wrapper. The phase6 render equals the 6.1 `repo-phase6.txt` + `.styles.txt` (test `phase6_repo_matches_immutable_text_and_styles`).
  6.5 extends `crates/tui/examples/home_html.rs` with a repo mode calling `render_repo` (a scratch copy that does this worked in 6.4:
  same HTML function, `TimeWindow::default_for(&report)`, `Some(0)` selection, `offset = view::local_offset(report.now)`).
- Keys: `enter`/`t` open, `esc` home, `z` folds, `i` imports, `+`/`-`/`←`/`→` explicit range clamped to the report window, `0` fitted.
  Old timeline goldens are retired (listed in the 6.4 log).
- Readability points to show the owner (from 6.4's real-data look, scratch only): Codex children read `codex codex · no prompt`
  (W3 fallback repeats the harness; real `codex exec` rollouts get `asked` null — check whether core misses their prompt); W5 times
  after local midnight show only the date (`started Nov 1`); today's real data had no folds; plus 6.3's home points below.

**Earlier (2026-11-01): Step 6.3 is committed.** Next: Step 6.4 (Codex, repo view to board 8c). What 6.4 needs:
- Home is board 7c (`crates/tui/src/home.rs`); the phase6 120x40 render equals the 6.1 `home-phase6.txt` + `.styles.txt`
  (test `phase6_matches_immutable_text_and_styles`, no update path). Phase 5 home goldens and their immutability test are gone.
- `enter` and `t` on any home target (card, card row, idle sentence, quiet entry) still open the OLD timeline
  (`crates/tui/src/timeline.rs`); 6.4 replaces that screen and its goldens. Home selection keeps full session ids or the
  repo across refreshes and across opening/closing the repo screen (tests in `app.rs`).
- Presentation helpers 6.4 can reuse are in `crates/tui/src/view.rs`: `sessions(group)` (rows, merged members, children),
  `attention`, `status` (W5 phrases), `clock` / `last_active` / `offset_label`, `home_label` / `session_label` (W3 order),
  `handed` / `outcome`, `link_words` (`confirmed` / `matched by time` / `N possible runs`), `minutes`.
- Narrow widths (home): status phrases are never cut; within a card all phrases share one column; below 80 the branch drops
  first, then labels shrink on a word boundary. Header drop order: hand-offs/repos, done-or-unknown, idle, shared checkouts,
  working now. Footer drops the imports note before keys. The V8 `⚠ N shared checkouts` header part and the card's
  `⚠ shared checkout: <name>` line are kept (FR17).
- Real-data look (scratch only, W10): matches board 7c's structure and colours. Open readability points for 6.5 / the owner:
  at 120 the header drops `N repos · N hand-offs` when the long `needs-you partial` note shows; `N other sessions, K active
  in the last 2h` can read as K sessions working when they are idle now; the `enter shows all` wording (known from 6.1).

**Earlier (2026-11-01): Step 6.2 is committed.** What 6.3 needed:
- Data: `Session` and `SessionRow` (children and merged members too) carry `activity: Vec<multi_link_model::Activity>`
  (`{from, to}`, oldest first, already clipped to the window), `asked: Option<String>` (already normalized: whitespace
  collapsed, max 119 chars + `…`), `imported: bool` and `state_since: Option<Timestamp>`. Imported rows are still in
  every report: hiding and counting them (`N Codex Desktop imports hidden`, `repo-a (only Codex Desktop imports)`) is TUI work.
- W5 inputs: `state_since` is set only for waiting/stale (record or hook) and Codex `unknown` + `turns_complete`
  (= `done HH:MM`, last real `task_complete`); busy, idle, finished and Codex stale/open_turn are null, so `idle · last
  active` uses the last burst's `to`, else `last_activity`. `working now · for N min` uses the burst containing `now`.
- Core is frozen for 6.3: `crates/core/tests/phase6.rs` pins every key (old values via
  `crates/core/tests/keys-before-phase6.json`); a TUI step must not need core changes. A blank Codex `user_message` gives
  `asked` `""` (not null); treat an empty `asked` like null in the label fallback.
- `multi_link_model::normalize_prompt` exists if the view needs it; `Timestamp::epoch_ms()` was added in the model.
- Target: `fixtures/expected/tui/home-phase6.txt` + `.styles.txt` (from 6.1) cell for cell; see 6.1's notes below for the
  column rules and the known `· enter shows all` wording issue.

**Earlier (2026-11-01): Step 6.1 is committed.** What 6.2 needed (done):
- Contract: `docs/output-schema.md` → Session → "Phase 6 fields" (exact rules for `activity`, `asked`,
  `imported`, `state_since`) and "Phase 6 fixture results". Rules added in 6.1 beyond PLAN's first wording (all in
  PLAN W1 / the Review log): the Codex `session_meta` record is not activity; Claude records whose `sessionId`
  differs from the session's are not activity; an `imported` rollout has `activity` `[]` and `asked` `null`.
- Test target: `multi-link sessions|overview --root fixtures/sessions-phase6 --now 2026-11-03T06:25:00Z --since 2h
  --json` == `fixtures/expected/phase6/{sessions,overview}.json` (parsed). With the four fields removed, both keys
  equal today's CLI output (checked in 6.1), so 6.2 should only need to add them. Field place: Session after
  `subagents`; SessionRow after `made` (order does not matter to the parsed compare).
- Existing keys gain the four fields; `asked` there is NOT always null: `fixtures/sessions` Claude prompts like
  `CASE-exec: <prompt>` give that string; `<prompt>` (phase4/5) and Codex without `user_message` give null.
- The fixture has a real W1 case: `6002` is silent 6.5 min while its Codex run works, so it has 4 bursts.
- `crates/tui/tests/phase6_shape.rs` pins the goldens' shape and that the keys carry the four fields.
- For 6.3/6.4: design.md → "Phase 6 decisions" → "Target screens" has every column rule; the goldens
  `fixtures/expected/tui/{home,repo}-phase6.txt` + `.styles.txt` were drawn by a scratch script from those rules (a
  copy is in the coordinator's notes folder, `step-6.1/`). New theme roles 6.4 must add: done green `#7CCB8E` / 114,
  child row background `#12151B` / 234. Known wording issue kept for the owner: the card's `· enter shows all` opens
  the repo view, which lists only sessions active in the window.

**Earlier (2026-11-01).** The owner ran Phase 5 on the real screen: the bars on home and the
timeline screen are "really hard to read … random blocks and timestamps on top". Phase 5 closes as not
meeting the need for its bars (6.5 records this in `docs/phase5-check.md`). The owner approved **board 7c**
(home in words, no timeline) and **board 8c** (repo view: board 2's swimlanes on real data) on the mockup
canvas. **Phase 6 is planned** in PLAN.md (W0–W10, Steps 6.1–6.5; Codex critique logged, 10/10 accepted).
Next: Step 6.1, then 6.2 → 6.5 in order; 6.5 ends with a stop-and-ask. Facts measured 2026-11-01 for 6.1/6.2:
140 of 260 local Codex rollouts are Codex Desktop imports (every `task_started` id starts `external-import`,
originator `Codex Desktop`; one more rollout mixes imported and real turns); a 5-min burst gap splits 2 of
117 closed turns in the 12 newest real Claude transcripts. Claude turn markers seen: `user` with string
content opens, `assistant` with `stop_reason` `end_turn` closes. Privacy (W10): screens now show prompts;
never commit real renders or the boards as images. The older notes below are history.

**Earlier (2026-10-30).** Phase 4 is built (4.1–4.6 committed) but the owner judged the real
screen still hard to read, nothing like the mockup, so Phase 4 closes as not meeting the need. **Phase 5**
(home redesign to the owner-approved **board 7**; priority UX/UI, readability, colour): **5.1 is committed.
5.2 is committed** (Claude-reviewed, 0 fix rounds). **5.3 is committed** (Claude-reviewed, 0 fix rounds).
**5.4 is committed** (Claude-reviewed, 1 Codex fix round: narrow-width truncation). **5.5 is committed
but Phase 5 is NOT closed: waiting on the owner.** Next: the owner runs `target/release/multi-link tui` on
the real screen and says whether the need is met; Claude records the answer in `docs/phase5-check.md`
("Owner check") and here. If not met, the owner's notes become new Phase 5 steps. The older notes below are history.

- **What the owner check / any follow-up step needs from 5.5:** `docs/phase5-check.md` has the real-`~`,
  fixture and board 7 images (`docs/img/`) and 18 numbered differences. Found issues for a possible next
  step (none fixed in 5.5, which may not touch `home.rs`): (1) in a terminal the lane tracks touch, so a
  card's bar area reads as one dark slab, unlike board 7's separated lanes — the biggest visual gap;
  (13) `last: claude handed step ? …` when the newest hand-off has no step (V6 has no wording for it);
  (14) the header hand-off count is `links.len()` over the whole scan (14 on real `~`, 5 inside the
  window) and is dropped first at 120 columns when the long needs-you note shows; (15) a combined lane
  whose rows both lack a step shows no step. `cargo run -p multi-link-tui --example home_html --
  OVERVIEW.json LINKS.json OUT.html [--size WxH] [--offset H[:MM]|local]` renders any home to HTML.
  The owner's Phase 4 verdict is quoted two ways in the repo (phase4-check.md vs PLAN.md / design.md);
  confirm the exact words with the owner.
- **What 5.5 needs from 5.4:** home draws via `render_home_with_options(report, selection, &HomeOptions { links, theme, offset, expanded_repos, split, show_all, .. }, area, buf)`; `HomeOptions::default()` is UTC and truecolor. The lane model is `multi_link_tui::view::lanes(report, group, split, show_all)`; `view::local_offset(now)` gives the offset (UTC fallback). The CLI passes the env theme into `App.theme` and still runs `theme.apply_no_color` after each draw. Phase5 120×40 render == the 5.1 pair (test `phase5_matches_immutable_text_and_styles`, no update path). Narrow widths drop whole header parts (hand-offs, repos, done/unknown, idle, busy) and footer keys (keeps `↑↓ move` + `q quit`), and `fit` never leaves `·…` / `→…`. Known rough edges for 5.5 to judge on real data: at 80 the waiting label loses its time (`waiting since…`); at 100 the title can end `Verify…`; Esc on home now clears expanded/split/show-all; below the screen height there is no "more below" hint.
- **What 5.4 needs from 5.3:** `multi_link_tui::theme::Theme::from_env(colorterm, no_color)` gives every
  palette role as a `Color` (`Rgb` truecolor, `Indexed` 256, `Reset` under nonempty `NO_COLOR`) plus
  `text_style(fg, bg)`. The CLI builds it once in `crates/cli/src/tui.rs::run` but today only calls
  `theme.apply_no_color(buf)` after each draw (strips colour + DIM from the legacy renders); 5.4 must pass the
  theme into `App` / `render_home` and draw with it, and can then drop or keep that post-pass (under NO_COLOR
  the new home must still draw the `·` track itself, V4). Styled goldens: `crates/tui/tests/support/mod.rs`
  `dump_styles` / `check_styles` (`UPDATE_STYLE_SNAPSHOTS=1` rewrites generated `.styles.txt`; text goldens
  still use `UPDATE_SNAPSHOTS=1`). `home.rs::check_snapshot` checks `<golden>.styles.txt` next to each text
  golden, so 5.4's new home renders regenerate the legacy home `.styles.txt` along with the text. The
  phase5 pair is checked only by the round-trip test in `tests/theme.rs`; 5.4 adds the real render check
  against it (never regenerate it). Legacy timeline styles keep named ANSI colours (written as `Cyan` etc.).
- **What 5.3 / 5.4 need from 5.2:** `OverviewReport.hook: HookSummary { records: usize, latest:
  Option<Timestamp> }` (model crate; zero / `None` when no hook records, also for an unreadable dir, which
  adds a `hook directory: …` diagnostic). `SessionRow.made: Vec<String>`, newest step first; a Claude row's
  label is `made[0]`, a Codex row's step is its placing link's `step` in `handoffs`. Phase 5 key values:
  `hook` zero, `made` `5011` `["4.1"]`, `5013` `["4.3"]`, `5014` `["4.4"]`, `5015` `["4.5"]`. Core still
  emits MergedRow `claude ×7` in repo-l; the TUI view flattens it (V3). Test
  `crates/core/tests/overview.rs::every_overview_matches_its_key` pins the phase5 key.

- **What 5.2 needs from 5.1:** the field rules are in `docs/output-schema.md`: SessionRow `made` (after
  `handoffs`, before `children`; distinct non-null link steps where `parent` = this id, newest first, any
  `match`, `[]` on Codex rows) and the report's top-level `hook` `{records, latest}` (valid v1 records in all
  hook files, `--since` ignored, unreadable dir → zeros plus a stderr diagnostic). Test target:
  `overview --root fixtures/sessions-phase5 --now 2026-10-29T23:20:00Z --since all --json` ==
  `fixtures/expected/phase5/overview.json`; `links` with the same flags == `phase5/links.json` (that one
  already passes on today's code). The phase5 key is the CLI output at 7d399ba plus `made` / `hook`, so 5.2
  should only need to add the two fields. The existing keys get `made` / `hook` and nothing else.
- **What 5.3 / 5.4 need from 5.1:** design.md → "Phase 5 decisions" → "Target screen" has the layout,
  bar mapping, palette (truecolor + 256), contrast table and the styled-golden format (blank cells: fg `-`,
  no BOLD/DIM/ITALIC). The target is `fixtures/expected/tui/home-phase5.txt` + `.styles.txt` (120×40, UTC−7,
  truecolor, nothing selected, bar window 2 h ending at `now` whatever `--since` is). The golden was drawn
  by a scratch Python script (not committed); a copy for reference is in the coordinator's notes folder
  (`step-5.1/draw_target.py`, plus `to_html.py`, which renders a golden pair to HTML for an eye check).

Phases 0–3 / **Release 1 are done**, including Step **3.8**.
Whole-phase Verify passes with documented limitations; Step 3.8 reviewed by Claude and committed locally (not pushed).
Next: **Phase 4 (home screen cleanup, Release 1.1)**, planned 2026-10-30 after the owner found the real
home screen far noisier than mockup screen 6. **4.1 and 4.2 are committed** (coordinator workspace gate
2026-10-30: `cargo test --workspace` 28 `test result: ok`, 0 failed; clippy `-D warnings` clean).
`-C .` / `-C "$PWD"` no longer count as a directory move (real-`~` counts in the 4.1 entry).
**4.3 is committed** (Claude-reviewed; `cargo test --workspace` 29 `test result: ok`, clippy clean).
**4.4 is committed** (Claude-reviewed, 1 Codex fix round; Claude re-ran `cargo test --workspace`
→ 29 `test result: ok`, 0 failed; clippy `-D warnings` clean).
**4.5 is committed** (Claude-reviewed, 0 fix rounds; Claude re-ran `cargo test --workspace` → 29
`test result: ok`, 0 failed; clippy `-D warnings` clean).
**4.6 is committed but Phase 4 is NOT closed: waiting on the owner.** Real-`~` before/after is in
`docs/phase4-check.md` (home non-blank rows 28 → 15 default, 246 → 30 `--since all`; multi-link links at one
pinned time 48 → 12 unlinked). Next: the owner runs `target/release/multi-link tui` (and `--since all`), says
whether the need is met, and Claude records the answer in `docs/phase4-check.md` ("Owner check") and here.
If not met, the owner's notes become new Phase 4 steps. Open trade-off: collapsed repos hide their latest
hand-off line. Codex-off mode is off.
Claude's host checks confirm warm timing with changed files, draining resize, Ctrl-C,
stub/missing-program child handling and terminal restoration. The child checks used a synthetic
record in a scratch HOME because real `~` had 0 finished sessions. The crossterm 0.29 lost-key
item remains backlog: a key arriving with resize can be delayed until the next key. A cleanly
exited terminal Claude session deletes its record, leaving `unknown` / `Unavailable` with a
resume command; Resume requires a record that outlives the process. See `docs/phase3-check.md`.
Codex-off mode is off: Codex implements, Claude plans and reviews.

- **What 4.6 needs from 4.5:** home lanes (top-level and child session rows) end in a fixed bar column on the
  right: 24 cells at width ≥ 72 (12–23 at 60–71, none below 60 or below height 4), window
  `TimeWindow::default_for(report)` = 2 h ending at the report's `now`, `━` span, `·` empty track, `◂` / `▸`
  clipped ends. The shared axis (`session span · idle gaps included · UTC` + `HH:MM … HH:MM`) is its own line
  under the U4 summary (deviation from "on the header line", accepted in review: the summary already fills
  ~92 of 100 columns), so the body is one row shorter. Lane text is cut one cell before the bar column, so at
  100 columns long hand-off metadata clips; the 4.6 pty captures should show whether that hurts the need.
- **What 4.5 needs from 4.4:** home draws through `render_home_with_options(report, selection,
  &HomeOptions { expanded_repos, links, hide_footer }, area, buf)`; `render_home` = default options (every
  quiet repo collapsed, `? hand-offs`). `selectable_home(report, expanded)` lists visible targets (adds
  `Target::Repo` for a collapsed line); `selectable` (all rows) is still what the timeline uses. Session lane
  lines come from `body_lines` in `crates/tui/src/home.rs`; collapsed repo lines and merged rows get no bar.
  The U4 header is line 0 (`summary_line`, takes the width; `now <ts>` only when the whole line fits), so
  4.5's shared axis labels must not push the counts past 100 columns (at 100 cols `now` is already dropped).
  Golden helper `check_snapshot` in `crates/tui/tests/home.rs` expands only repo-i for phase4 and passes the
  fixture links; `render()` there expands every repo for lane tests.
- **What 4.4 needs from 4.3:** `MergedRow.harness` exists; core emits `codex ×N` and `claude ×N` (at most
  one per harness per group). Step 4.4 now omits `0 codex children` on Codex merged rows. App refresh keeps
  a merged row selected by (repo, harness); merged-row `enter` expansion stays keyed by repo (expands both).
  Core test `crates/core/tests/overview.rs` already pins the phase4 key; render the 4.4 golden from that key.
- **What 4.3 / 4.4 need from 4.2:** contract in `docs/output-schema.md` → Overview → Rows (Merge) and
  "Phase 4 fixture results"; design U1–U5. Test target: `overview --root fixtures/sessions-phase4 --now
  2026-10-29T23:15:00Z --since all --json` == `fixtures/expected/phase4/overview.json`. U3 keys: separate
  per-repo expanded set; `enter` on a collapsed repo expands it, merged rows stay merged; `esc` clears both sets.
- **ps locale P1: fixed in 3.6a** (`LC_ALL=C` + `TZ=UTC` on every `ps` call); 3.7 is no longer blocked by it.
- **What 3.7 needs from 3.6:** `render_timeline(&OverviewReport, &LinksReport, repo, TimeWindow, area, buf)` and
  `TimeWindow::default_for(report)`; 3.7 adds selected-lane scrolling, +/- zoom, left/right pan and 0 reset.

- **What 3.7 needs from 3.4:** before `o` / `f`, call `open::fresh_action(&mut scanner, root, &options, full_id, fork)`
  (fresh pass, window off; `None` → footer "session no longer found"). Execute `Resume` as
  `Command::new(program).args(args).current_dir(cwd).status()` and `AppLink` as `/usr/bin/open <url>`, wrapped in T9's
  terminal suspend/restore; show `Unavailable` with its reason and command. The CLI's `open_command` builds these.
- **What 3.7 needs from 3.5:** `selection` indexes `selectable(report)`; a merged row is one line until expanded, and
  3.7 expands it in App presentation state before selecting individual members. crossterm is a tui dependency.

- **What 3.4 needs from 3.3:** `Session.host_session_id` and `Session.record` already follow the
  several-records-per-id rule (`core/src/sessions.rs::choose_record`: `finished` only when every record is
  `Gone`). `open::action` can work on a `Session` alone. For the lookup by id prefix with the window off, call
  `Scanner::sessions` / `refresh` with `since: None`.
- **What 3.5 needs from 3.3:** the types are `multi_link_model::{OverviewReport, RepoGroup, Tier, Plan,
  LatestStep, ConcurrencyWarning, Row, SessionRow, MergedRow}`. `Row` is `#[serde(untagged)]`, and each struct
  writes its own `"kind"`. Render from `fixtures/expected/overview.json` and `hook/overview.json`, which
  deserialize into `OverviewReport` (model round-trip test).
- **Plan lookup is injected:** `Scanner::overview(root, &options, &mut plans)`, where `plans: FnMut(&Path) ->
  RepoSteps`. Core does not depend on `multi-link-plan` at runtime; it is a dev-dependency for the example only.
  The CLI and `refresh_timing` pass `multi_link_plan::steps_report`. 3.7's refresh worker must pass the same
  closure. `core::overview::plan_folder` decides the folder.
- **3.3 timing (Claude, real `~`, warm release build, now includes overview + plan parse):** `refresh_timing --
  3` → pass 1 3460 ms, pass 2 25 ms (k=0), pass 3 45 ms (k=2, 2.1 MB). Warm refresh stays far under 2000 ms.
- **Backlog nit:** the CLI plan lookup turns the folder into a string with `to_string_lossy`, so a non-UTF-8 repo
  path shows `missing` (review round 1, rejected for 3.3).
- **3.1 result (Claude, real `~`, real `ps`, warm release build):** `refresh_timing -- 3` → pass 1 3058 ms,
  pass 2 29 ms (k=1), pass 3 31 ms (k=2). Warm refresh ≤ 2000 ms with k ≥ 1, so **offset tailing is not needed**
  before 3.7 (T1 holds as written). Passes with k=0 also occur (live sessions write in bursts).
- **What 3.2+ need to know from 3.1:** keep ONE `core::Scanner` per refresh worker and call
  `Scanner::refresh(root, &options)` → `(SessionsOutput, LinksOutput)` (one discovery pass, one batched `ps`).
  `Scanner::stats` (files/bytes re-parsed) and `Scanner::diagnostics` (hook dir/file errors, also listed in
  `unreadable`) describe the last pass. `sessions_report` / `links_report` are fresh-Scanner wrappers; the CLI
  `sessions` command still calls both (≈ 5.9 s warm, was 16.4 s), so 3.3's `overview` CLI should use one
  `refresh`. Repo memo (`Scanner::repos`, per cwd) never expires and memoises the cwd fallback when git fails.
- **Resolved in 3.7 (T9):** retain whole-pass failure on a non-NotFound Claude transcript/subagent directory
  error (EACCES, ENOTDIR). The TUI keeps the last good report, shows the refresh error, and blocks open/fork until
  a successful refresh. Partial-report conversion would require core/parser files outside 3.7's scope. A real
  ENOTDIR subagent-path regression test covers the retained behavior; no core changes.
- **Baseline for 3.1 (measured 2026-10-30, warm release build, real `~`):** `sessions --json` 16.4–16.9 s
  (10.4 s user, 4.5 s sys); `--root ~` (no ps/git) 6.0 s; `links` 9.8 s; `steps` 7.0 s. Cause: one `git
  rev-parse` per session before the window filter, one `ps` per record, and a full re-parse of every file.
- **What exists:** Core `Scanner::{sessions,links,refresh,overview}` caches per-file parsed results and git mappings,
  with one batch process probe per refresh. Rust workspace `crates/{model,claude,codex,plan,core,hook,cli}`. `multi-link sessions | links |
  steps | overview` (JSON or text; `--root` fixture mode, `--now`, `--since`) and `multi-link hook record | install |
  uninstall` (install only prints). `cargo test --workspace` → 24 binaries ok; clippy `-D warnings` clean.
- **Answer keys:** `fixtures/expected/{sessions,links,steps}.json` (Phase 1), `links-tagged.json`,
  `links-amb-hidden.json`, `fixtures/expected/hook/*` (Hook variant). Verify commands are in PLAN.md 1.9 / 2.x.
- **Key decisions to carry into Phase 3:** exact links only with an observed child id; waiting comes from the
  Claude session record first (H11), hook second; freshness limit applies to `busy` only; liveness = pid +
  start time in UTC (1.10); step ids `N`, `N.M` (letter-suffix decision in design.md); multi-link never writes
  to `~/.claude/` or `~/.codex/`.
- **Settled in Phase 3 planning:** background hook child `null` in clean-env `-p` → deferred to backlog (T10);
  hook dir read errors → diagnostics (T9, in 3.1's Scanner work); hook files read twice → gone with the cache.
- **Parallel-step rule:** parallel sessions don't commit; the coordinator commits each step separately
  (learned in 1.2/1.5).
- **Leftovers outside the repo:** `~/multi-link-sandbox/` (sandbox repos, deps clones, `state/hook/` from 2.8)
  and `~/.multi-link/spike/` (Step 2.1 capture). Safe to keep; delete by hand if unwanted.

## Log (newest first)

### 2026-11-12 — Step 20.0: Pins and keys before the deletion — Claude
- Did: Captured `menubar --json` for the pane keys' two uncovered root/time pairs, copied and reshaped the watch
  fixtures kept tests read, cross-checked all 12 pane keys' rows against the menubar keys, pinned R2–R4.
- Changed: `docs/design.md`, `fixtures/expected/menubar/{phase9-early,phase9-noon}.json`,
  `fixtures/expected/menubar/{watch-script.json,watch-tick.jsonl}`, `fixtures/expected/menubar/check.sh`, `PROGRESS.md`.
- Verify: `fixtures/expected/menubar/check.sh` → ok ×5, exit 0. Cross-check: 12 keys, 55 rows, 0 mismatches.
- Review: Claude's own step; the Codex plan critique (8 findings, all accepted) is in the PLAN Review log.
- Gotchas: pane rows are keyed by id for the cross-check, since the pane orders rows around "this session".

### 2026-11-12 — Step 19.4: Real check, README (owner check pinned) — Claude
- Did: Ran `apps/menubar/build/multi-link.app` against real `~` with the app's binary path set to the release build.
  Watched the watcher switch to `--notify` and take the lock; the owner confirmed the popover matches board 19. The
  notch hid the icon until the owner freed two menu-bar items (window x 825 → 863 → 908). A real `codex exec` hand-off
  made one marker (28 → 29), announced by the pane through Script Editor (installed 1.14 binary, no lock check). The
  owner declined installing the new binary now, so the multi-link-credited notification is pinned with Phase 18's check.
  Wrote `docs/phase19-check.md` and the README menu-bar section and limitations.
- Changed: `docs/phase19-check.md`, `README.md`, `PROGRESS.md`.
- Verify: all six `check.sh` exit 0 and `claude plugin test` 134 pass (re-run); Rust unchanged since 19.1 (736 / 0 /
  1); `swift test` 18 passed and `build.sh` verified at 19.3. Owner's answers: "Matches the board", "From Script
  Editor"; owner check pinned.
- Review: Claude's own step (docs only).
- Gotchas: on a notched MacBook a full menu bar hides new status items; macOS showed nothing left of x ≈ 898 even
  though the notch ends at 848. The owner saw no notification prompt for the app, yet its alert setting read enabled.

### 2026-11-12 — Step 19.3: Review round 1 — Codex
- Did: Fixed all three Claude findings. Startup resolves the binary and starts the watcher before awaiting
  notification authorization, so reports arrive while the prompt remains open; it then calls
  `watcher.refreshPermission()` after the request returns, retaining the once-per-launch guard and using
  settings rather than the authorization result. Injected startup effects let tests suspend authorization
  without contacting the real notification center or using the real delivery lock. Headers now include the
  optional step title before `+N`; absent titles keep the prior form. Open TUI activates Terminal inside the
  AppleScript run handler after opening the window, preserving shell quoting.
- Changed: `apps/menubar/Sources/MenubarCore/{AppController,Presentation,Actions}.swift`,
  `apps/menubar/Tests/MenubarCoreTests/PresentationTests.swift`, `PROGRESS.md`.
- Verify: Regression tests for both title forms (with/without `+N`) and the exact AppleScript argv failed before
  the fixes (4 assertion issues across 3 tests). Temporarily restoring authorization-before-watcher ordering
  made the startup regression fail with `reportBeforeAnswer?.repos.isEmpty → nil`; restored the fixed ordering.
  Final `swift test --disable-sandbox --package-path apps/menubar` → exit 0, actual final test output:
  ```text
✔ Test symbolsActionsAndColors() passed after 0.001 seconds.
✔ Test completionRequestAndForegroundDelegate() passed after 0.001 seconds.
✔ Test commandsAndResults() passed after 0.001 seconds.
✔ Test headersWithAndWithoutStepTitle() passed after 0.001 seconds.
✔ Test deliveryLockRequiresPermissionAndHealthAndRetries() passed after 0.201 seconds.
✔ Test lookupOrderAndNotFound() passed after 0.463 seconds.
✔ Test fixtureContractAndByteFraming() passed after 0.463 seconds.
✔ Test presentationContract() passed after 0.567 seconds.
✔ Test modelKeepsLastReportOnErrors() passed after 0.567 seconds.
✔ Test fixtureBlocksAndChildLines() passed after 0.567 seconds.
✔ Test actionProcessUsesExactArgvAndDrainsBothPipes() passed after 1.127 seconds.
✔ Test injectedPermissionControlsDeliveryAndQuitReleasesLock() passed after 1.386 seconds.
✔ Test unexpectedExitReleasesDeliveryDuringBackoff() passed after 1.387 seconds.
✔ Test watcherRecognizesOldBinaryAndUnknownVersion() passed after 31.263 seconds.
✔ Test transientScanOutputKeepsDeliveryAndChild(_:) with 2 test cases passed after 31.309 seconds.
✔ Test startupWatchesWhileAuthorizationPendingAndRefreshesAfterward() passed after 31.310 seconds.
✔ Test watcherDrainsStderrRestartsWithBackoffAndQuits() passed after 31.738 seconds.
✔ Test secondInstanceCannotClaimAndTakesOverAfterQuit() passed after 31.914 seconds.
✔ Test run with 18 tests in 0 suites passed after 31.914 seconds.
  ```
  In `apps/menubar`, `swift build -c release --disable-sandbox` → exit 0:
  ```text
[0/1] Planning build
Building for production...
[0/3] Write sources
[1/3] Write swift-version--58304C5D6DBC2206.txt
[3/4] Compiling MenubarCore Actions.swift
[4/6] Compiling MenubarApp App.swift
[4/6] Write Objects.LinkFileList
[5/6] Linking multi-link-menubar
Build complete! (4.02s)
  ```
  `git diff --check` → exit 0, no output.
- Review (Claude, 2026-11-12): re-review clean. Re-ran `swift test` (18 passed) and `./build.sh` (built and
  verified; `Signature=adhoc`, designated requirement `identifier "dev.multi-link.menubar"`).
- Gotchas: No app launch or live macOS prompt/banner/Terminal activation check; those remain Step 19.4.
  Startup regression checks report arrival before answering, one authorization request across two starts,
  disabled delivery before approval, and the permission banner clearing immediately after the settings refresh.

### 2026-11-12 — Step 19.3: Popover, icon, actions, notifications, app bundle — Codex
- Did: Implemented board 19 P2 / I1 with thin SwiftUI views. MenubarCore preserves report block/row order, builds
  step headers with `+N`, harness labels and `└` child lines, state symbols and light/dark board palettes, icon
  text, actions from `opens`, exact open argv, checked age and error footer text (including M4's retry hint),
  quoted Open TUI commands, and notification requests. AppController handles startup, permission reporting,
  clipboard/session/TUI actions and shutdown; permission is requested once at launch and settings checks drive
  delivery through the existing watcher. The retained notification delegate returns banner + sound; requests
  use `run`, pane title/body text and default sound. Notifications-off banner dismisses once per launch and
  opens Notifications settings. Build script creates the release bundle in the repo, copies Info.plist and
  executable, signs with the bundle-id designated requirement, verifies, and never launches.
- Changed: `apps/menubar/Sources/MenubarApp/App.swift`, `apps/menubar/Sources/MenubarCore/{Model,Presentation,
  Actions,Notifications,AppController}.swift`, `apps/menubar/Tests/MenubarCoreTests/PresentationTests.swift`,
  `apps/menubar/build.sh`, `apps/menubar/Info.plist`, `PROGRESS.md`.
- Verify: New presentation tests first failed because the presentation/action/notification APIs were absent;
  the fixture-label test then failed on the missing harness prefix before adding it. Final commands and actual
  output below (Swift commands use `--disable-sandbox` per M3 and the handoff). From `apps/menubar`,
  `swift test --disable-sandbox` (invoked from root with `--package-path apps/menubar`) → exit 0, final test output:
  ```text
✔ Test symbolsActionsAndColors() passed after 0.001 seconds.
✔ Test commandsAndResults() passed after 0.001 seconds.
✔ Test completionRequestAndForegroundDelegate() passed after 0.001 seconds.
✔ Test modelKeepsLastReportOnErrors() passed after 0.001 seconds.
✔ Test deliveryLockRequiresPermissionAndHealthAndRetries() passed after 0.006 seconds.
✔ Test fixtureContractAndByteFraming() passed after 0.716 seconds.
✔ Test lookupOrderAndNotFound() passed after 0.716 seconds.
✔ Test fixtureBlocksAndChildLines() passed after 0.717 seconds.
✔ Test presentationContract() passed after 0.717 seconds.
✔ Test injectedPermissionControlsDeliveryAndQuitReleasesLock() passed after 0.841 seconds.
✔ Test actionProcessUsesExactArgvAndDrainsBothPipes() passed after 0.842 seconds.
✔ Test unexpectedExitReleasesDeliveryDuringBackoff() passed after 0.842 seconds.
✔ Test watcherRecognizesOldBinaryAndUnknownVersion() passed after 1.285 seconds.
✔ Test watcherDrainsStderrRestartsWithBackoffAndQuits() passed after 1.406 seconds.
✔ Test transientScanOutputKeepsDeliveryAndChild(_:) with 2 test cases passed after 1.481 seconds.
✔ Test secondInstanceCannotClaimAndTakesOverAfterQuit() passed after 2.957 seconds.
✔ Test run with 16 tests in 0 suites passed after 2.958 seconds.
  ```
  `./build.sh` → exit 0:
  ```text
[0/1] Planning build
Building for production...
[0/4] Write sources
[2/4] Write swift-version--58304C5D6DBC2206.txt
[4/5] Compiling MenubarCore Actions.swift
[5/7] Compiling MenubarApp App.swift
[5/7] Write Objects.LinkFileList
[6/7] Linking multi-link-menubar
Build complete! (3.97s)
build/multi-link.app: replacing existing signature
Built and verified build/multi-link.app
  ```
  `codesign --verify --strict build/multi-link.app` → exit 0, no output (also enforced by `build.sh` with `set -e`).
  `codesign -dv build/multi-link.app 2>&1 | rg 'Identifier=|Format=|Signature=|Info.plist'` → exit 0:
  ```text
Identifier=dev.multi-link.menubar
Format=app bundle with Mach-O thin (arm64)
Signature=adhoc
Info.plist entries=10
TeamIdentifier=not set
  ```
  `codesign -d -r- build/multi-link.app 2>&1 | rg '^designated'` → exit 0:
  ```text
designated => identifier "dev.multi-link.menubar"
  ```
  `plutil -p build/multi-link.app/Contents/Info.plist` → exit 0:
  ```text
{
  "CFBundleDisplayName" => "multi-link"
  "CFBundleExecutable" => "multi-link-menubar"
  "CFBundleIdentifier" => "dev.multi-link.menubar"
  "CFBundleName" => "multi-link"
  "CFBundlePackageType" => "APPL"
  "CFBundleShortVersionString" => "1.0"
  "CFBundleVersion" => "1"
  "LSMinimumSystemVersion" => "15.0"
  "LSUIElement" => 1
  "NSAppleEventsUsageDescription" => "multi-link opens the TUI and focuses your agent's terminal tab."
}
  ```
  `git diff --check` → exit 0, no output.
- Review (Claude, 2026-11-12): three findings, fixed in Review round 1: the watcher waited for the first-launch
  notification prompt; the repo header dropped the step title; Open TUI didn't bring Terminal forward.
- Gotchas: No app launch, notification permission prompt, actual banner or screenshot verification in this step;
  those are Step 19.4. Block ordering deliberately follows the CLI report rather than re-sorting in Swift.
  The notification test decodes a wire event and injects a request sink; it never calls the system notification
  center. The action process test uses a local fake executable and drains 262144 stderr bytes. Existing unrelated
  `.claude/` and `mod/multi-link-pane/tsconfig.json` untracked files were left untouched.

### 2026-11-12 — Step 19.2: Review round 1 — Codex
- Did: Fixed Claude's health finding: decodable lines, including scan errors, refresh watcher health;
  malformed lines neither refresh nor clear it. Too-old output, child exit and 10 s of silence still lose
  health. Errors still reach the model. Added a parameterized regression using a live fake child, checking
  error/malformed dispatch, uninterrupted delivery ownership, a competing lock probe, and unchanged child-start
  records after each transient line (only the initial startup rotation is allowed).
- Changed: `apps/menubar/Sources/MenubarCore/Watcher.swift`,
  `apps/menubar/Tests/MenubarCoreTests/CoreTests.swift`, `PROGRESS.md`.
- Verify: Before the fix, `swift test --disable-sandbox --package-path apps/menubar
  --filter transientScanOutputKeepsDeliveryAndChild` → exit 1, actual decisive output:
  ```text
  ✘ Test transientScanOutputKeepsDeliveryAndChild(_:) with 2 test cases failed after 0.813 seconds with 4 issues.
  ✘ Test run with 1 test in 0 suites failed after 0.813 seconds with 4 issues.
  ```
  Both error and malformed cases failed delivery-history and child-start assertions.
  After the fix, the same command → exit 0, actual decisive output:
  ```text
  ✔ Test transientScanOutputKeepsDeliveryAndChild(_:) with 2 test cases passed after 0.636 seconds.
  ✔ Test run with 1 test in 0 suites passed after 0.637 seconds.
  ```
  In `apps/menubar`, `swift build -c release --disable-sandbox` → exit 0, actual output:
  ```text
  [0/1] Planning build
  Building for production...
  [0/3] Write sources
  [1/3] Write swift-version--58304C5D6DBC2206.txt
  [3/4] Compiling MenubarCore BinaryLookup.swift
  [4/5] Compiling MenubarApp App.swift
  [4/6] Write Objects.LinkFileList
  [5/6] Linking multi-link-menubar
  Build complete! (2.41s)
  ```
  `swift test --disable-sandbox` → exit 0, actual final output:
  ```text
  ✔ Test modelKeepsLastReportOnErrors() passed after 0.001 seconds.
  ✔ Test deliveryLockRequiresPermissionAndHealthAndRetries() passed after 0.003 seconds.
  ✔ Test lookupOrderAndNotFound() passed after 0.431 seconds.
  ✔ Test fixtureContractAndByteFraming() passed after 0.431 seconds.
  ✔ Test injectedPermissionControlsDeliveryAndQuitReleasesLock() passed after 0.772 seconds.
  ✔ Test unexpectedExitReleasesDeliveryDuringBackoff() passed after 0.978 seconds.
  ✔ Test transientScanOutputKeepsDeliveryAndChild(_:) with 2 test cases passed after 1.111 seconds.
  ✔ Test watcherRecognizesOldBinaryAndUnknownVersion() passed after 1.179 seconds.
  ✔ Test watcherDrainsStderrRestartsWithBackoffAndQuits() passed after 1.409 seconds.
  ✔ Test secondInstanceCannotClaimAndTakesOverAfterQuit() passed after 3.021 seconds.
  ✔ Test run with 10 tests in 0 suites passed after 3.021 seconds.
  ```
- Review (Claude, 2026-11-12): re-review clean. Re-ran `swift build -c release` and `swift test` in `apps/menubar`
  (outside the sandbox): build complete, 10 tests passed.
- Gotchas: Accepted gap: after the lock is taken, the --notify child's first pass is a baseline, so a completion
  in those few seconds at startup or after a permission change is not announced.

### 2026-11-12 — Step 19.2: The Swift package: model, watcher, binary lookup, lock — Codex
- Did: Added a dependency-free Swift 6.2 package targeting macOS 15, a minimal executable, and a core library.
  Decodes the shared report/row/child/handoff/event contract; buffers raw bytes to newlines and dispatches reports,
  ticks, errors and completion events; unknown versions show too old, malformed lines fail soft. Observable UI
  state is main-actor isolated and retains the last report on errors. Lookup tries the saved path, cargo, Homebrew
  and local bin in order and returns the complete attempted list on failure. Actor-owned Process supervision runs
  off the main actor, continuously drains both pipes, bounds retained stderr, retries exits with exponential
  1–30 s backoff, recognizes old-subcommand diagnostics and never restarts after Quit.
  Permission is injectable; the real adapter reads alertSetting == .enabled at launch, explicit refresh (for the
  future popover), and every 60 s. Delivery requires permission plus a line within 10 s. The lock uses
  O_CREAT|O_RDWR|O_CLOEXEC and LOCK_EX|LOCK_NB, retries contention every 2 s, and releases on permission/health
  loss, unexpected exit/backoff, incompatible output or Quit. Only the owning instance runs with --notify, so
  denied/second instances do not consume permanent notification markers. Delivery changes rotate the watcher.
- Changed: `.gitignore`, `apps/menubar/Package.swift`, `apps/menubar/Sources/MenubarApp/App.swift`,
  `apps/menubar/Sources/MenubarCore/{Model,BinaryLookup,DeliveryLock,Watcher}.swift`,
  `apps/menubar/Tests/MenubarCoreTests/CoreTests.swift`, `PROGRESS.md`.
- Verify: In `apps/menubar`, `swift build -c release --disable-sandbox` → exit 0, actual output:
  ```text
  Building for production...
  [0/5] Write sources
  [2/5] Write swift-version--58304C5D6DBC2206.txt
  [4/6] Compiling MenubarCore BinaryLookup.swift
  [5/7] Compiling MenubarApp App.swift
  [5/7] Write Objects.LinkFileList
  [6/7] Linking multi-link-menubar
  Build complete! (4.28s)
  ```
  `swift test --disable-sandbox` → exit 0, actual final output (Swift Testing cases, after the empty XCTest wrapper):
  ```text
  ✔ Test modelKeepsLastReportOnErrors() passed after 0.001 seconds.
  ✔ Test deliveryLockRequiresPermissionAndHealthAndRetries() passed after 0.003 seconds.
  ✔ Test lookupOrderAndNotFound() passed after 0.321 seconds.
  ✔ Test fixtureContractAndByteFraming() passed after 0.321 seconds.
  ✔ Test injectedPermissionControlsDeliveryAndQuitReleasesLock() passed after 0.352 seconds.
  ✔ Test unexpectedExitReleasesDeliveryDuringBackoff() passed after 0.433 seconds.
  ✔ Test watcherDrainsStderrRestartsWithBackoffAndQuits() passed after 1.143 seconds.
  ✔ Test watcherRecognizesOldBinaryAndUnknownVersion() passed after 1.875 seconds.
  ✔ Test secondInstanceCannotClaimAndTakesOverAfterQuit() passed after 2.834 seconds.
  ✔ Test run with 9 tests in 0 suites passed after 2.835 seconds.
  ```
  `git diff --check` → no output, exit 0. `git check-ignore apps/menubar/.build/x apps/menubar/build/x` →
  ```text
  apps/menubar/.build/x
  apps/menubar/build/x
  ```
- Review (Claude, 2026-11-12): one finding, an `error` line cleared health, so one transient scan error dropped the
  lock and cold-restarted the child twice; fixed in Review round 1. Accepted: the `--notify` child's first pass after
  the lock is taken is a baseline (a few seconds without announcements). PLAN.md and unrelated files untouched.
- Gotchas: Initial test compilation failed on the absent implementation; subsequent runs exposed a fixture-path
  error, Swift Testing's mutating-call macro limitation, and Foundation read(upToCount:) waiting to fill pipe
  reads (4 test issues). Corrected the path/assertions and used POSIX reads; final suite is green. Tests use temp
  scripts/files and fake permission only, with no live agent data or real notification center. Deliberate watcher
  rotations when delivery changes establish a new notification baseline, so the existing restart loss limitation
  applies to those transitions too. No views, notification posting, app bundle, signing or real app launch in this
  step; the minimal executable exits immediately. Those remain for 19.3.

### 2026-11-12 — Step 19.1: Review round 1 — Codex
- Did: Addressed Claude's blocking performance finding by filtering the all-time sessions before expensive
  pane projection/enrichment: only non-imported sessions active within the two-hour window or waiting/stale
  can nominate a repo. Completion scans now visit only repos with a session active within the window;
  imported recent sessions still nominate a completion repo, since events are independent of row visibility.
  Shared one window constant with the scan options. Added a projection callback seam and call-count regression
  proving old-finished-only and imported-only repos are never projected, plus exact completion-repo selection.
  Tests cover the inclusive two-hour boundary, old busy, quiet waiting/stale, and deduplication of repo calls.
- Changed: `crates/cli/src/menubar.rs` (implementation and unit tests), `PROGRESS.md` only in this round.
- Verify: Both new tests failed before the filters (actual RED output excerpts):

```text
  left: ["/boundary", "/imported", "/old-busy", "/old-finished", "/recent", "/stale", "/waiting"]
 right: ["/boundary", "/recent", "/stale", "/waiting"]
  left: ["/boundary", "/imported", "/old-busy", "/old-finished", "/recent", "/stale", "/waiting"]
 right: ["/boundary", "/imported", "/recent"]
test result: FAILED. 2 passed; 2 failed; 0 ignored; 0 measured; 45 filtered out; finished in 0.01s
```

  Final `TZ=UTC cargo test --workspace` exited 0: **736 passed / 0 failed / 1 ignored** (sum of actual result
  lines). Relevant test names and all actual result lines pasted below; unrelated per-test/build lines omitted:

```text
test menubar::tests::menubar_watcher_replays_claudes_script_byte_for_byte ... ok
test menubar::tests::held_app_lock_leaves_pane_runs_unclaimed_for_app_and_unlock_restores_pane ... ok
test menubar::tests::projection_never_builds_rows_for_old_finished_or_imported_only_repos ... ok
test menubar::tests::completion_scans_only_repos_with_activity_in_the_window ... ok
test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.52s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.83s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.88s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test menubar_matches_claudes_three_keys_byte_for_byte ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.23s
test pane_matches_all_twelve_pretty_json_keys_byte_for_byte ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.05s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.74s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 63 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

  Remaining prescribed Verify commands, actual output (build path omitted):

```text
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.10s
[exit 0]
$ cargo fmt --check
[exit 0]
$ fixtures/expected/menubar/check.sh
ok   multi-repo
ok   waiting-quiet
ok   open-actions
[exit 0]
$ fixtures/expected/open-jump/check.sh
ok   open open 18000001
ok   open open 18000002
ok   open open 18000003
ok   open open 18000004
ok   open open 18000005
ok   open open 18000006
ok   open open 18000007
ok   open open 18000008
ok   open open 18000009
ok   open open 18000010
ok   open open 18000011
ok   open fork 18000001
ok   open fork 18000002
ok   open fork 18000003
ok   open fork 18000004
ok   open fork 18000005
ok   open fork 18000006
ok   open fork 18000007
ok   open fork 18000008
ok   open fork 18000009
ok   open fork 18000010
ok   open fork 18000011
[exit 0]
$ fixtures/expected/phase12/check.sh
ok   links.json
ok   overview.json
ok   graph-1.1.json
[exit 0]
$ fixtures/expected/phase13/check.sh
ok   links.json
[exit 0]
$ fixtures/expected/phase14/check.sh
ok   live.json
[exit 0]
$ fixtures/expected/phase9/check.sh 6
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   graph-1.1.json (^9[1-6])
ok   graph-2.1.json (^9[1-6])
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   stderr unreadable: 1
ok   open open 92000001
ok   open open 92000002
ok   open open 92000003
ok   open open 92000004
ok   open open 92000005
ok   open open 92000006
ok   open open 92000007
ok   open open 92000008
ok   open open 92000009
ok   open open 92000010
ok   open open 92000011
ok   open open 92000012
ok   open open 92000013
ok   open open 92000014
ok   open open 94000001
ok   open open 94000002
ok   open open 94000003
ok   open open 94000004
ok   open open 94000006
ok   open open 94000007
ok   open fork 92000001
ok   open fork 92000002
ok   open fork 92000003
ok   open fork 92000004
ok   open fork 92000005
ok   open fork 92000006
ok   open fork 92000007
ok   open fork 92000008
ok   open fork 92000009
ok   open fork 92000010
ok   open fork 92000011
ok   open fork 92000012
ok   open fork 92000013
ok   open fork 92000014
ok   open fork 94000001
ok   open fork 94000002
ok   open fork 94000003
ok   open fork 94000004
ok   open fork 94000006
ok   open fork 94000007
note lanes.json is checked by the 9.5 worker-to-App test, not here
[exit 0]
```

- Review (Claude, 2026-11-12): re-review clean. Re-ran Verify: 736 passed / 0 failed / 1 ignored, clippy and fmt
  clean, all six `check.sh` exit 0, pane keys unchanged. Warm tick 208 / 211 ms (was 245–259 ms). Earlier text:
  Ready for Claude re-review and warm CPU remeasurement. No claim that the CPU budget is met;
  Claude's original 245–259 ms finding is the motivation, not a measurement of this fix. All report, watch,
  and pane keys remain byte-for-byte matches. No keys or pane implementation changed in this round.
- Gotchas: The first full-suite attempt observed `held_app_lock_leaves_pane_runs_unclaimed_for_app_and_unlock_restores_pane`
  fail at its post-release assertion (`48 passed; 1 failed` in the CLI unit suite). The test now explicitly
  unlocks before dropping the descriptor, so a parallel fork/exec cannot extend a drop-only lock lifetime.
  The subsequent full workspace run passed. Nothing staged, committed or pushed; pre-existing files left alone.

### 2026-11-12 — Step 19.1: Menu-bar JSON watcher and pane delivery lock — Codex
- Did: Added the all-repo report and CLI flags under the exact Phase 19 M1/M2 pins. Shared the pane's existing
  projection, detail enrichment, completion reader, runtime and watch loop. Ordered repos from shown sessions,
  counted waiting/running, flattened sections, and read the first active step's title from repo/PLAN.md.
  Pane claims now probe the app lock nonblockingly; menubar claims ignore it. Added byte-for-byte tests for
  all three report keys and the pane watch-script replay against the menubar event key, a temporary repo title,
  and held-lock suppression, app claims (two hand-offs to one Codex id), and pane delivery after unlock.
- Changed: `crates/cli/src/menubar.rs`, `crates/cli/src/pane.rs`, `crates/cli/src/main.rs`,
  `crates/cli/tests/menubar.rs`, `docs/output-schema.md`, `PROGRESS.md`.
- Verify: From repo root with the prescribed Rust PATH; `TZ=UTC cargo test --workspace` exited 0:
  **734 passed / 0 failed / 1 ignored** (sum of the actual result lines below). Actual output excerpts follow
  (all result lines, plus the relevant named tests; build paths and unrelated per-test lines omitted):

```text
test menubar::tests::menubar_watcher_replays_claudes_script_byte_for_byte ... ok
test menubar::tests::held_app_lock_leaves_pane_runs_unclaimed_for_app_and_unlock_restores_pane ... ok
test pane::watch_tests::watch_scripts_match_byte_for_byte_with_and_without_notify ... ok
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.84s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.84s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test menubar_reads_active_step_title_from_temporary_repo_plan ... ok
test menubar_matches_claudes_three_keys_byte_for_byte ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.28s
test pane_matches_all_twelve_pretty_json_keys_byte_for_byte ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.04s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.69s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 63 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.74s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

  `cargo clippy --workspace --all-targets -- -D warnings` exited 0; actual output excerpt (build path omitted):

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.72s
```

  Actual formatting and fixture-check output (each exit recorded by the runner):

```text
$ cargo fmt --check
[exit 0]
$ fixtures/expected/menubar/check.sh
ok   multi-repo
ok   waiting-quiet
ok   open-actions
[exit 0]
$ fixtures/expected/open-jump/check.sh
ok   open open 18000001
ok   open open 18000002
ok   open open 18000003
ok   open open 18000004
ok   open open 18000005
ok   open open 18000006
ok   open open 18000007
ok   open open 18000008
ok   open open 18000009
ok   open open 18000010
ok   open open 18000011
ok   open fork 18000001
ok   open fork 18000002
ok   open fork 18000003
ok   open fork 18000004
ok   open fork 18000005
ok   open fork 18000006
ok   open fork 18000007
ok   open fork 18000008
ok   open fork 18000009
ok   open fork 18000010
ok   open fork 18000011
[exit 0]
$ fixtures/expected/phase12/check.sh
ok   links.json
ok   overview.json
ok   graph-1.1.json
[exit 0]
$ fixtures/expected/phase13/check.sh
ok   links.json
[exit 0]
$ fixtures/expected/phase14/check.sh
ok   live.json
[exit 0]
$ fixtures/expected/phase9/check.sh 6
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   graph-1.1.json (^9[1-6])
ok   graph-2.1.json (^9[1-6])
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   stderr unreadable: 1
ok   open open 92000001
ok   open open 92000002
ok   open open 92000003
ok   open open 92000004
ok   open open 92000005
ok   open open 92000006
ok   open open 92000007
ok   open open 92000008
ok   open open 92000009
ok   open open 92000010
ok   open open 92000011
ok   open open 92000012
ok   open open 92000013
ok   open open 92000014
ok   open open 94000001
ok   open open 94000002
ok   open open 94000003
ok   open open 94000004
ok   open open 94000006
ok   open open 94000007
ok   open fork 92000001
ok   open fork 92000002
ok   open fork 92000003
ok   open fork 92000004
ok   open fork 92000005
ok   open fork 92000006
ok   open fork 92000007
ok   open fork 92000008
ok   open fork 92000009
ok   open fork 92000010
ok   open fork 92000011
ok   open fork 92000012
ok   open fork 92000013
ok   open fork 92000014
ok   open fork 94000001
ok   open fork 94000002
ok   open fork 94000003
ok   open fork 94000004
ok   open fork 94000006
ok   open fork 94000007
note lanes.json is checked by the 9.5 worker-to-App test, not here
[exit 0]
```

  RED evidence: before implementation, both CLI tests failed with `unrecognized subcommand 'menubar'`.
  The title test initially exposed an incomplete synthetic session record (missing required `procStart`),
  corrected in the test setup. Reverting held-lock suppression made
  `held_app_lock_leaves_pane_runs_unclaimed_for_app_and_unlock_restores_pane` fail on
  `assertion failed: !output.contains("codex_done")` (exit 101); disabling menubar notifications made
  `menubar_watcher_replays_claudes_script_byte_for_byte` fail on exact output equality (exit 101).
  Both deliberate mutations were restored before the passing workspace run.
- Review (Claude, 2026-11-12): one finding, tick CPU 245–259 ms over the 250 ms budget (every all-time repo was
  projected and scanned for completions); fixed in Review round 1. `PLAN.md`, all expected fixtures and
  `crates/cli/tests/pane.rs` remain unchanged.
  No staging, commit or push. Existing untracked files were left alone.
- Gotchas: Tick CPU measurement deliberately skipped per the owner; Claude owns that real-data measurement.
  The lock always lives in the watched home's `.multi-link/menubar.lock`, including fixture mode;
  the existing `MULTI_LINK_HOME` override still selects only the shared state/marker directory.
  A completion skipped while locked stays unclaimed; as pinned in M2, releasing the lock restores future
  transitions, without replaying completions that were already observed during the accepted loss window.

### 2026-11-12 — Step 19.0: Facts on this Mac, design pins, answer keys — Claude
- Did: Scratch spikes (never committed) of an ad-hoc signed SwiftPM `MenuBarExtra` app: notifications, rebuild
  behaviour, signing requirement, terminal jump and Open TUI from inside the app, `flock`, and Swift in the Codex
  sandbox. Owner answers: "Allowed, saw banner", "No banners" (plain ad-hoc rebuild), "Allowed, saw B1", "Yes, B2
  showed" (bundle-id requirement), "Prompt, I allowed" (Automation). Pinned M1–M4 in design "Phase 19 decisions";
  aligned PLAN M1 (step rule) and M2 (reuse the pane's event line). Keys generated from today's `pane` and `sessions`
  output by a scratch generator (`p190/gen19.py`), so the menubar rows match the pane's byte for byte.
- Changed: `docs/design.md`, `PLAN.md`, `fixtures/expected/menubar/{multi-repo,waiting-quiet,open-actions}.json`,
  `fixtures/expected/menubar/watch-codex-done.jsonl`, `fixtures/expected/menubar/check.sh`, `PROGRESS.md`.
- Verify: `fixtures/expected/menubar/check.sh` → `menubar subcommand missing`, exit 2 (expected: keys ahead of code).
  No Rust code changed since the Step 18.4 run (730 passed / 0 failed / 1 ignored). Spike commands and outputs are in
  design "Phase 19 decisions" → Real facts.
- Review: Claude's own step; the Codex plan critique (10 findings, all accepted) is in the PLAN Review log.
- Gotchas: Codex's `swift build` fails without `--disable-sandbox` (`sandbox-exec: sandbox_apply`) and the three
  cache `--add-dir`s; `codex exec -C <scratch>` needs `--skip-git-repo-check`. Notifications from an app in
  `/private/tmp` are refused with no prompt.

### 2026-11-12 — Step 18.4: Real check, README (owner confirm pending) — Claude
- Did: Real checks on Terminal.app in `~/multi-link-sandbox/repo-p17`: the CLI jump and the TUI `o` (PTY harness,
  home → repo-p17 → Enter → `o`) each brought the session's tab to the front from another front window, checked with
  AppleScript (`/dev/ttys006, true`). The committed Terminal script prints `not found` for a tty with no tab. Wrote
  `docs/phase18-check.md`; README: the jump answers table, the Automation note, the pane button, the TUI key, and
  the limitations (iTerm2 untested).
- Changed: `docs/phase18-check.md`, `README.md`, `PROGRESS.md`.
- Verify (whole phase, Claude): `TZ=UTC cargo test --workspace` → 730 passed / 0 failed / 1 ignored; clippy
  `-D warnings` and `cargo fmt --check` clean; `open-jump`, `phase12`, `phase13`, `phase14` and `phase9 6` check.sh
  all exit 0; `claude plugin validate` → `✔ Validation passed` (existing catalog notice); `claude plugin test` →
  134 pass / 0 fail. Real-check commands and outputs are in `docs/phase18-check.md`. Owner's answer: pending.
- Review: Claude's own step (docs only); no code changed.
- Gotchas: the first sandbox session exited before the TUI check, and a session with no prompt has no transcript, so
  lists skip it; a new one was started with a prompt. On home, `o` on a repo card asks for the repo view first
  (existing behaviour). The Automation prompt did not appear because the caller was already allowed.

### 2026-11-12 — Step 18.3: Review round 1 — Codex
- Did: Fixed both Claude findings. Missing `opens` normalizes to null for rows and children before validation;
  false, numbers, objects, arrays and unknown destination strings remain invalid. Successful presses keep
  non-empty stdout, else say `opened in app` (app/legacy) or `jumped to terminal` (terminal). Failed presses
  keep non-empty stderr, else say `could not open session`.
- Changed: `mod/multi-link-pane/hooks/register.tsx`, `mod/multi-link-pane/hooks/register.test.tsx`, `PROGRESS.md`.
  Accepted pane keys and Rust code were not changed in this round.
- Verify: Added 20 regression cases across terminal, desktop, vscode and mobile: a Phase 17 line with
  `opens` omitted from every row and child renders and expands; invalid values on either row type preserve
  the last good report; whitespace-only stdout/stderr gets the correct app/terminal success or failure text.
  Before implementation, `claude plugin test mod/multi-link-pane` → actual output:
  ```text
   114 pass
   20 fail
  Ran 134 tests across 1 file. [7.06s]
  ```
  Failures included the legacy line staying on `reading sessions…` and the empty-output footer staying
  `checked 0s ago`. After implementation, same command → exit 0, actual output:
  ```text
   134 pass
   0 fail
  Ran 134 tests across 1 file. [7.58s]
  ```
  `claude plugin validate mod/multi-link-pane` → exit 0, `✔ Validation passed`; existing non-fatal catalog
  notices `LICENSE_MISSING` / `ICON_MISSING` remain.
- Verify: `TZ=UTC cargo test --workspace` → exit 0. Actual summary printed by Python summing the captured
  `test result:` lines:
  ```text
  Workspace: 68 targets; 730 passed; 0 failed; 1 ignored
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, actual output:
  ```text
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
  ```
  `cargo fmt --all -- --check` and `git diff --check` → no output, exit 0. Rust PATH set per AGENTS.md.
  Re-ran the byte-comparison script below against `1728932` → actual output:
  ```text
  PASS: all 14 original pane files remain byte-identical after removing 59 opens fields.
  ```
- Review: Claude, 1 round. Keys checked additive by Claude (removing `opens` from all ten changed keys gives HEAD's
  bytes; `terminal.json` is new). Findings: (1) the mod required `opens`, so a 1.14 binary's lines would all be
  rejected and the pane would sit on `reading sessions…` → a missing `opens` is null; (2) an app-link open prints
  nothing, so the footer went blank → fallbacks `opened in app` / `jumped to terminal` / `could not open session`.
  Claude re-ran Verify: 730 passed, 0 failed, 1 ignored; clippy, fmt clean; `claude plugin validate` ✔, `claude
  plugin test` 134 pass, 0 fail. Real data: with a `claude` running in a Terminal.app tab (sandbox), `open
  --dry-run` answered `jump` / `terminal` / `/dev/ttys006`; the watcher's warm tick with that session present
  (snapshot every pass) was **226 ms CPU** over 60 s with `--notify` (budget ≤ 250 ms; Phase 17: 219 ms). Codex's
  256 ms was one whole pass, not a watcher tick.
- Gotchas: No new timing run: this round only changes mod parsing and footer text, not the Rust pass.
  The previous single warm release sample and its budget limitation still apply. Nothing staged or committed;
  existing untracked files untouched.

### 2026-11-12 — Step 18.3: Pane opens and Jump to terminal — Codex
- Did: Added `opens` immediately after `openable` in both row types (`v` remains 1), using the existing
  once-per-pass resolver: AppLink → `app`, Jump → `terminal`, everything else → null. The resolver action is
  evaluated once per visible session; no extra resolver build or process snapshot was added. `openable` is
  true exactly when `opens` is non-null. Updated mod types, input validation, embedded keys, labels and argv.
  The footer uses trimmed CLI stdout on success and trimmed stderr on failure, even if both streams contain
  text. Review round 1 adds explicit empty-output fallbacks and accepts Phase 17 input without `opens`.
  Spawn exceptions show `could not open session`.
- Changed: `crates/cli/src/pane.rs`, `crates/cli/tests/pane.rs`,
  `mod/multi-link-pane/{hooks/register.tsx,hooks/register.test.tsx,types/index.d.ts}`,
  `fixtures/expected/pane/{ambiguous,busy,child-other-repo,children-exact-likely,done-2h,idle-recent,openable,unseen,waiting-quiet,waiting-stale,terminal}.json`,
  `PROGRESS.md`. Fixture edits are explicitly authorized by the owner's Step 18.3 request.
- Verify: Test-first: the new Rust integration test failed with `missing opens: 18000001`; the mod tests
  failed on the old argv/terminal label. Final `cargo test -p multi-link --test pane` → actual output:
  ```text
  test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.36s
  ```
  After extending the Rust case table to exercise nested children, final `TZ=UTC cargo test --workspace`
  exited 0. Summary printed by Python summing its captured `test result:` lines:
  ```text
  Workspace: 68 targets; 730 passed; 0 failed; 1 ignored
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, actual output:
  ```text
      Checking multi-link v0.1.0 (repo/crates/cli)
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.53s
  ```
  (Compiler's checkout prefix replaced with `repo` above to avoid committing a personal path.)
  `cargo fmt --all -- --check` and `git diff --check` → no output, exit 0. Rust PATH set per AGENTS.md.
- Verify: `claude plugin validate mod/multi-link-pane` → exit 0, `✔ Validation passed`.
  Its additional catalog checks report existing `LICENSE_MISSING` and `ICON_MISSING`; those checks explicitly
  do not affect normal validation (no directory profile requested). `claude plugin test mod/multi-link-pane`
  → exit 0, actual output:
  ```text
   114 pass
   0 fail
  Ran 114 tests across 1 file. [6.07s]
  ```
  The 24 new mod cases cover app/terminal/null × CLI success/failure on terminal, desktop, vscode and mobile;
  each exercises top-level and child actions, exact argv (including shell-looking id data), both-stream
  stdout/stderr precedence, footer text, and absence of an open button for null. Existing Phase 16–17 cases
  still pass, including redraw, overflow, copy, strip, watch lifecycle, ticks and notifications.
- Verify (additivity): Regenerated all 11 old pretty JSON pane keys using the command table in
  `crates/cli/tests/pane.rs`; empty and watcher files need no edits. New `terminal.json` uses the new table
  entry with `sessions-phase18`. The following byte check ran against base `1728932` (HEAD at implementation):
  ```python
  import pathlib, re, subprocess
  paths = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', '1728932',
                                   'fixtures/expected/pane'], text=True).splitlines()
  for path in paths:
      before = subprocess.check_output(['git', 'show', f'1728932:{path}'])
      after = pathlib.Path(path).read_bytes()
      assert re.sub(rb'^\s*"opens": (?:"app"|"terminal"|null),\n', b'',
                    after, flags=re.M) == before, path
  ```
  Actual comparison output (including a separate parsed comparison of embedded mod keys):
  ```text
  PASS: all 14 tracked pane files reproduce HEAD byte for byte after removing 59 opens fields; old openable values unchanged.
  PASS: 11 embedded mod keys equal the regenerated fixture JSON.
  PASS: Phase 18 key equals pre-change bytes after removing opens and reverting openable only for terminal rows ['18000001', '18000002', '18000009'].
  ```
  The last comparison used the same Phase 18 pane command captured before implementation. Terminal/iTerm2
  and newer-dead/older-live rows gain `openable`; all other bytes match after removing `opens` and reverting
  those three bools. No other expected directory changed.
- Verify (timing): Temporarily timed the complete `run` pass closure with `Instant::now()` before refresh
  and elapsed milliseconds after strip resolution; used one watcher process per case, measuring its first
  and second passes. Removed instrumentation in a `finally` block and rebuilt clean binaries afterwards.
  No `--notify`; stdout discarded, timing-only stderr saved outside the repo. Commands: `cargo build -p multi-link`
  (debug cases), then `cargo build --release -p multi-link` (release case); `multi-link pane --json --watch`
  with the corresponding pane test arguments (phase14 session `14110000-0000-4000-8000-000000001411`),
  or real home with `--cwd <checkout> --session timing-unseen`. Actual measured output:
  ```text
  phase18 fixture (snapshot, no strip): cold pane_pass_ms=9.027; one warm pane_pass_ms=5.572
  phase14 fixture (cached strip): cold pane_pass_ms=16.735; one warm pane_pass_ms=6.067
  real home (unseen session, no strip): cold pane_pass_ms=6986.920; one warm pane_pass_ms=1213.812
  release real home (unseen session, no strip): cold pane_pass_ms=1831.120; one warm pane_pass_ms=256.990
  ```
- Review: Claude accepted the additive keys. Both round 1 mod findings are fixed in the entry above;
  pending Claude re-review. No staging, commit or push; PLAN.md untouched.
- Gotchas: The release warm real-home sample is 6.990 ms above the 250 ms target; this is not a ten-tick
  median or a real current-session strip benchmark. Performance follow-up belongs with Claude before
  declaring the budget met; no out-of-scope optimization attempted. The fixture snapshot uses fixture-ps,
  not real `ps`; existing resolver snapshot-count tests remain green. Real terminal jumps and Automation
  prompts were not invoked. Existing untracked `.claude/` and mod `tsconfig.json` were left untouched.

### 2026-11-12 — Step 18.2: Running the jump in the CLI and TUI — Codex
- Did: Replaced both temporary Jump arms with one runner. Embedded the answer-key scripts using `include_str!`,
  one `-e` per line and tty last; no shell or text interpolation. Drained stdout/stderr concurrently, polled the
  child against the five-second production deadline, killed and reaped on timeout, and returned failure without
  falling back to Resume. Added `--no-resume` and kept `--app-only` restrictive. TUI shows the same result without
  suspending terminal modes; Enter still navigates.
  Review round 1: included optional unavailable detail after the reason in parentheses on CLI and TUI;
  absent-detail messages remain byte-identical. Added exact-message regression tests on both surfaces.
- Changed: `crates/cli/src/main.rs`, `crates/cli/src/tui.rs`, `crates/cli/tests/open_jump.rs`,
  `crates/cli/tests/support/jump_runner.rs`, `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md. Test-first CLI run exited 101 with the expected missing behavior:
  ```text
  failures:
      jump_exact_argv_and_success_for_both_apps
      jump_failures_never_resume
      no_resume_accepts_app_link_and_refuses_resume

  test result: FAILED. 3 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.99s
  ```
  `cargo test -p multi-link --test open_jump` exited 0 after implementation (actual output):
  ```text
  running 6 tests
  test app_only_refuses_jump_without_running_script ... ok
  test no_resume_accepts_app_link_and_refuses_resume ... ok
  test open_jump_keys_match_bytes_and_exit_codes ... ok
  test jump_exact_argv_and_success_for_both_apps ... ok
  test old_open_keys_match_bytes_and_exit_codes ... ok
  test jump_failures_never_resume ... ok

  test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.63s
  ```
  `cargo test --workspace --no-fail-fast` exited 0: 727 passed, 0 failed, 1 ignored across 68 actual result
  lines; the ignored case is existing `real_counts`. Decisive actual output excerpts:
  ```text
  test jump_tests::hung_child_is_killed_and_reaped_at_injected_deadline ... ok
  test tui::tests::fresh_jump_shows_runner_message_without_suspending_and_enter_navigates ... ok
  test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
  test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.22s
  test real_counts ... ignored
  ```
  Strengthened the timeout test after that run to check the production five-second default and that the
  injected 500 ms deadline actually elapses. `cargo test -p multi-link --bin multi-link hung_child` exited 0:
  ```text
  running 1 test
  test jump_tests::hung_child_is_killed_and_reaped_at_injected_deadline ... ok

  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 43 filtered out; finished in 0.55s
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` exited 0 (final actual output):
  ```text
  Checking multi-link v0.1.0 (/Users/jarednarvaez/multi-link/crates/cli)
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.02s
  ```
  `cargo fmt --all -- --check` and `git diff --check` exited 0 with no output.
  Review round 1 verification (2026-11-12): both new tests failed before the fix, each exit 101; CLI lacked
  `(tmux)` and TUI lacked `(Ghostty)`. After the fix, `cargo test -p multi-link unavailable_message_includes_detail`
  exited 0 for both regression tests. `cargo test --workspace --no-fail-fast` exited 0: 729 passed, 0 failed,
  1 existing ignored across 68 result lines. Actual output excerpts:
  ```text
  test tui::tests::unavailable_message_includes_detail_and_preserves_absent_detail ... ok
  test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
  test unavailable_message_includes_detail_and_preserves_absent_detail ... ok
  test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.20s
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` exited 0 (actual output):
  ```text
      Checking multi-link v0.1.0 (/Users/jarednarvaez/multi-link/crates/cli)
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.04s
  ```
  `cargo fmt --all -- --check` and `git diff --check` exited 0 with no output.
- Review: Claude, 1 round. The runner matched design I2 (scripts embedded, one `-e` per line, tty last, no shell;
  pipes drained on threads; 5 s deadline then kill and reap; -1743 → the Automation text; `not found` → "its tab
  is gone"; only `window N tab N` counts as success; no resume on any failure; `--no-resume` and `--app-only` as
  I3; the TUI's `o` runs the same runner without suspending; Enter still navigates). Finding: unavailable messages
  dropped the new `detail` → now `reason (detail)` in the CLI and TUI, other messages byte-identical. Accepted: the
  TUI waits up to 5 s for `osascript` on its own thread of control. Claude re-ran Verify: 729 passed, 0 failed, 1
  ignored; clippy and fmt clean; every `check.sh` exit 0; sample: `terminal_not_supported (Ghostty)`.
- Gotchas: Every new execution test uses a fake osascript (both apps' exact argv and success, not found, -1743,
  other error, malformed output, missing executable, timeout). The injected timeout is 500 ms to allow shell
  startup under load; production is fixed at 5 s. The timeout test verifies kill/reap with waitpid/ECHILD because
  this sandbox denies ps; the initial ps-based assertion failed with Operation not permitted. The TUI test drives
  `o` through the fresh resolver and checks rendered messages, mode calls and Enter navigation. AppLink acceptance
  uses dry-run so tests never open a real application. No live terminal/Automation checks were attempted.

### 2026-11-12 — Step 18.1: Resolver: Jump and the process snapshot — Codex
- Did: Implemented `Jump`, terminal app names, the six pinned unavailable reasons and optional `detail` in JSON
  order. Added an injectable snapshot seam, real ps through `RealProcessProbe::ps_command` (C locale, UTC),
  fixture table reads, whole-snapshot rejection, time-based record identity selection and the bounded ancestor
  walk. Scanner remembers whether its last assembled report needs a snapshot, then takes at most one per
  resolver build; opening performs no extra liveness probes. CLI dry runs serialize Jump; CLI/TUI execution
  temporarily refuse. Updated only the Open schema. The owner approved the TUI compatibility edit, and Claude
  added it to PLAN.md; the previous blocker is resolved.
- Changed: `crates/core/src/{open.rs,probe.rs,scanner.rs,lib.rs}`, `crates/core/tests/open_jump.rs`,
  `crates/cli/src/{main.rs,tui.rs}`, `crates/cli/tests/open_jump.rs`, `docs/output-schema.md`, `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md. Before implementation, `fixtures/expected/open-jump/check.sh` exited 1
  (11 open FAIL, 11 fork ok); the new CLI key test failed as expected (`left: Some(3)`, `right: Some(0)`).
  After implementation, `cargo test --workspace` exited 0; all 68 result lines were ok (721 passed, 0 failed,
  1 ignored, from the actual log). Decisive actual output excerpts:
  ```text
  test result: ok. 63 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.86s
  test real_counts ... ignored
  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  The existing `real_counts` test is ignored. Added an old-key byte comparison and strengthened query-count
  assertions after the workspace run; targeted reruns exited 0:
  `cargo test -p multi-link --test open_jump`:
  ```text
  running 3 tests
  test jump_execution_is_temporarily_refused ... ok
  test open_jump_keys_match_bytes_and_exit_codes ... ok
  test old_open_keys_match_bytes_and_exit_codes ... ok
  test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.93s
  ```
  `cargo test -p multi-link-core --test open_jump`:
  ```text
  running 4 tests
  test no_snapshot_for_finished_unknown_app_link_or_empty_reports ... ok
  test one_batch_and_one_snapshot_per_resolver_build_not_per_action ... ok
  test missing_fixture_table_preserves_old_answer_and_read_errors_are_unknown ... ok
  test real_snapshot_uses_exact_argv_c_locale_utc_and_one_process ... ok
  test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
  ```
  `cargo test -p multi-link-core --lib jump_tests` exited 0: whole-snapshot failures never resume,
  no tty/multiplexer/unsupported/missing/cyclic ancestors, the 64-row limit, padded days and canonical tty:
  ```text
  test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 59 filtered out; finished in 0.03s
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` exited 0:
  ```text
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 12.39s
  ```
  `cargo fmt --all -- --check` and `git diff --check` → no output, exit 0.
  `git diff --name-only -- fixtures` → no output, exit 0 (keys/checkers unchanged).
  `fixtures/expected/open-jump/check.sh` → exit 0, actual output:
  ```text
ok   open open 18000001
ok   open open 18000002
ok   open open 18000003
ok   open open 18000004
ok   open open 18000005
ok   open open 18000006
ok   open open 18000007
ok   open open 18000008
ok   open open 18000009
ok   open open 18000010
ok   open open 18000011
ok   open fork 18000001
ok   open fork 18000002
ok   open fork 18000003
ok   open fork 18000004
ok   open fork 18000005
ok   open fork 18000006
ok   open fork 18000007
ok   open fork 18000008
ok   open fork 18000009
ok   open fork 18000010
ok   open fork 18000011
  ```
  `fixtures/expected/phase9/check.sh 6` → exit 0, actual output:
  ```text
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   graph-1.1.json (^9[1-6])
ok   graph-2.1.json (^9[1-6])
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   stderr unreadable: 1
ok   open open 92000001
ok   open open 92000002
ok   open open 92000003
ok   open open 92000004
ok   open open 92000005
ok   open open 92000006
ok   open open 92000007
ok   open open 92000008
ok   open open 92000009
ok   open open 92000010
ok   open open 92000011
ok   open open 92000012
ok   open open 92000013
ok   open open 92000014
ok   open open 94000001
ok   open open 94000002
ok   open open 94000003
ok   open open 94000004
ok   open open 94000006
ok   open open 94000007
ok   open fork 92000001
ok   open fork 92000002
ok   open fork 92000003
ok   open fork 92000004
ok   open fork 92000005
ok   open fork 92000006
ok   open fork 92000007
ok   open fork 92000008
ok   open fork 92000009
ok   open fork 92000010
ok   open fork 92000011
ok   open fork 92000012
ok   open fork 92000013
ok   open fork 92000014
ok   open fork 94000001
ok   open fork 94000002
ok   open fork 94000003
ok   open fork 94000004
ok   open fork 94000006
ok   open fork 94000007
note lanes.json is checked by the 9.5 worker-to-App test, not here
  ```
- Review: Claude, no findings. Checked against design I1: one `ps -axo pid,ppid,lstart,tty,comm` through the probe's
  `ps_command` only when a running non-app Claude session exists; whole-snapshot failures → `process_unknown` (10
  cases tested); live = pid present with equal start time (zero- vs space-padded day tested); one/none/two live →
  jump/`process_changed`/`ambiguous_process`; the walk (64 steps, cycle stop, multiplexer before app, `.app` name)
  and `/dev/` tty; `Absent` (no fixture table) keeps `running_without_app_link`; `--fork` unchanged. The one scope
  stop (`tui.rs` compile compatibility) was approved by Claude and logged. Claude re-ran Verify: 722 passed, 0
  failed, 1 ignored; clippy and fmt clean; `open-jump`, phase 9/12/13/14 `check.sh` all exit 0. The 18.0 Terminal
  session had exited, so a real `--dry-run` waits for 18.4.
- Gotchas: No jump execution or osascript command was run. Resolver construction uses the states/mode from
  the preceding sessions/refresh pass on the same root; standalone metadata-only construction keeps the legacy
  behavior. Missing fixture tables preserve older answers; unreadable/malformed tables return process_unknown.
  No dependencies, answer keys or scripts changed; nothing staged, committed or pushed. PLAN.md's scope
  amendment and pre-existing untracked files belong to the owner/Claude and were left untouched.

### 2026-11-12 — Step 18.0: Facts on real processes, design pins, answer keys — Claude
- Did: `codex-quota.py` → `CODEX: OK` (18.0 is a Claude step). Real facts in `~/multi-link-sandbox/repo-p17`: started
  `claude` in a new Terminal.app window (`osascript … do script`, cwd by argv); its record had `pid` and `procStart`
  `Thu Nov 12 08:37:52 2026`; `ps` (`TZ=UTC`) gave the same `lstart`, tty `ttys006`, chain `claude` → `-zsh` →
  `login` → `/System/Applications/Utilities/Terminal.app/Contents/MacOS/Terminal` (`??`, ppid 1). The Terminal
  script by argv returned `window 1 tab 1` and the front selected tab's tty was then `/dev/ttys006`; `/dev/ttys999`
  and bare `ttys006` returned `not found`. No Automation prompt (already allowed); -1743 not seen yet. Wrote design
  pins, the fixture root (11 Claude sessions, 13 records, a raw `ps` table) by a scratch generator, `open.json`,
  `check.sh` and the two scripts (Terminal compiled with `osacompile`; iTerm2 cannot compile here, not installed).
- Changed: `docs/design.md`, `fixtures/sessions-phase18/**`, `fixtures/expected/open-jump/**`, `PLAN.md`,
  `PROGRESS.md`.
- Verify: `TZ=UTC cargo test --workspace --no-fail-fast` → 711 passed, 0 failed, 1 ignored; `fixtures/expected/open-jump/check.sh` → 11
  FAIL (`open`, expected before 18.1), 11 ok (`fork`); W10 scan clean.
- Review: Claude's own step (facts, pins, keys); no code.
- Gotchas: records zero-pad the day (`Nov 09`), `ps` space-pads it (`Nov  9`): compare as times, not strings. The
  Terminal `tty` needs the `/dev/` form.

### 2026-11-11 — Step 17.4: Real check, README (owner confirm pending) — Claude
- Did: mode check `codex-quota.py` → `CODEX: OK` (17.4 is a Claude step; Codex used once to review Claude's edit).
  Scratch PTY harness, two interactive `claude --plugin-dir mod/multi-link-pane` sessions in
  `~/multi-link-sandbox/repo-p17` (one PLAN.md step), `osascript` logging shim (title and body length only), pane A
  ran one synthetic `codex exec` for Step 1.1. Four runs; runs 1–2 failed on Claude Code's trust prompt (harness
  keys), run 3 worked without a step in the hand-off, run 4 is the result. Found and fixed two mod bugs (pane sized
  from the whole screen; `unexpected argument` from a 1.13 binary looped) with tests that fail without the fixes.
  Wrote `docs/phase17-check.md`, updated the README pane section and limitation line.
- Changed: `mod/multi-link-pane/hooks/register.tsx`, `mod/multi-link-pane/hooks/register.test.tsx` (Review log
  "Step 17.4 scope"), `docs/phase17-check.md`, `README.md`, `docs/design.md` (H6 date), `PLAN.md`, `PROGRESS.md`.
- Verify: run 4 events — both panes' watchers with `--notify`; strip seen 9 s after the prompt; `osascript calls: 1;
  new markers: 1`; shim line `4:-e:codex finished (1.1)` with a 2-character body; `tick CPU, one watcher, 30 s idle:
  195 ms/tick`; pane data 3.64 s (A) and 4.27 s (B) from launch; `watchers 5 s after exit: 0`. `claude plugin
  validate` ✔; `claude plugin test` 90 pass, 0 fail (86 before + 4 new; the 4 new fail with the fix stashed). NFR:
  tick CPU ≤ 250 ms passes; pane data is slower than Phase 16's 2.81 s from launch (two sessions starting at once,
  and the first pass now also builds the strip and reads linked rollouts), recorded as a limitation. Owner check:
  pending.
- Review: Claude's edit reviewed by Codex (read-only): "Clean—no findings in the `mod/` diff."
- Owner check (2026-11-11): screenshot of the desktop pane showing the strip, an open row and `copied id`; the
  notification needed Script Editor allowed once in macOS (first three test hand-offs claimed but not shown, macOS
  dropped them); the fourth showed `codex finished (17.4)`. Owner: "Saw both". Phase 17 closed.
- Gotchas: Claude Code's trust prompt defaults to "No, exit"; an arrow key sent as one escape sequence right after
  launch can read as Esc. Trust the sandbox folder once. `e.viewport` in `ui.render` is the screen, not the pane.

### 2026-11-11 — Step 17.3: Review round 1 — Codex
- Did: Addressed the S1 styling finding with optional styled segments on Line, rendered as a row Box
  of Text elements only for the strip heading and part lines. Heading: dim step/title, bold id.
  Parts: harness colour only on the harness, default-colour kind, success for done, warning for
  running/waiting, dim unknown mark and minutes; this session's marker uses the Claude colour and
  every segment of its part is bold. The complete line is sanitized and cut once to the column
  budget, then the surviving text retains its segment styles. Other line rendering is unchanged.
  Tests check heading and done/running/unknown/waiting segment styles on both surfaces, plus
  shared truncation at 5, 10, 22 (wide title characters) and zero columns. The existing text helper
  now joins each segmented strip row so all previous row-text assertions remain in force.
- Changed: `mod/multi-link-pane/hooks/register.tsx`,
  `mod/multi-link-pane/hooks/register.test.tsx`, `PROGRESS.md`.
- Verify: New regression tests first produced `76 pass / 10 fail` before implementation.
  Final `claude plugin test mod/multi-link-pane` → exit 0; actual output excerpt:
```text
(pass) S1 heading and part segments use board styles on terminal [42.65ms]
(pass) strip segments share one column budget 5 on terminal [33.87ms]
(pass) strip segments share one column budget 10 on terminal [46.81ms]
(pass) strip segments share one column budget 22 on terminal [32.83ms]
(pass) strip segments share one column budget 0 on terminal [33.70ms]
(pass) S1 heading and part segments use board styles on desktop [52.22ms]
(pass) strip segments share one column budget 5 on desktop [32.52ms]
(pass) strip segments share one column budget 10 on desktop [49.22ms]
(pass) strip segments share one column budget 22 on desktop [33.88ms]
(pass) strip segments share one column budget 0 on desktop [32.70ms]
 86 pass
 0 fail
Ran 86 tests across 1 file. [4.57s]
```
  `claude plugin validate mod/multi-link-pane` → exit 0; actual result/advisory headings excerpt
  (absolute local paths omitted):
```text
✘ Found 1 problem the plugin directory refuses a submission for:
⚠ Found 1 plugin directory warning:
✔ Validation passed
```
  `git diff --check` → exit 0, no output.
- Review: Claude, 1 round. Behaviour matched H2–H5 on first pass (strip fold and `+N earlier`, one open row, `Open
  in app` only when `openable` via `open --app-only` argv, `Copy id`, `--notify` only when on and macOS, fixed
  `osascript` argv, failure in the footer without retry, overflow order, no empty-section rule, `checked Ns ago`).
  Finding vs board 17 S1: strip lines were one colour each → styled segments (dim `step`/title, bold id, harness
  colour, `success`/`warning` marks, dim minutes). Claude re-ran Verify: `claude plugin validate` ✔ passed;
  `claude plugin test` 86 pass, 0 fail. Accepted: no highlight on the open row (no theme background key in 2.1.322).
- Gotchas: Existing directory-only missing license/icon advisories remain. No Rust, PLAN.md,
  manifest or types changes in this round. Nothing staged, committed or pushed.


### 2026-11-11 — Step 17.3: The mod: strip, row actions, notifications, polish — Codex
- Did: Implemented H2–H5 against board 17's S1 / A1 / N2. The strip keeps at most four parts,
  preserving this session's newest part when needed and folding older parts. Rows and children are id-keyed
  Buttons; one row expands with width-cut asked/said and Copy id, plus Open in app only when openable.
  Open uses the explicit H3 `--app-only` argv (the shorter Verify wording omits that flag).
  Expansion survives redraws and closes when its id leaves the data. Overflow drops recent, waiting,
  oldest strip parts, said, asked, working, then an unpinned open row with its buttons; pinned-child
  overflow keeps the selected child and buttons together while folding siblings.
  Notifications default on through the manifest's boolean option, gated by local `uname -s` = Darwin;
  no shell or event text is interpolated into AppleScript. The lifecycle remembers attempted run ids across
  watcher retries; Rust's permanent tombstones remain the cross-pane delivery authority.
  Separate action notices keep a notification failure visible when the following data line arrives.
  Updated the state shape tag for the added open-row/action-notice fields.
- Changed: `mod/multi-link-pane/.claude-plugin/plugin.json`,
  `mod/multi-link-pane/hooks/register.tsx`, `mod/multi-link-pane/hooks/register.test.tsx`,
  `mod/multi-link-pane/types/index.d.ts`, `PROGRESS.md`.
- Verify: `claude plugin test mod/multi-link-pane` → actual output:
```text
hooks/register.test.tsx:
(pass) reading before the first line on terminal [43.26ms]
(pass) reading before the first line on desktop [22.15ms]
(pass) ambiguous watcher line on terminal [39.98ms]
(pass) busy watcher line on terminal [40.64ms]
(pass) child-other-repo watcher line on terminal [33.69ms]
(pass) children-exact-likely watcher line on terminal [45.58ms]
(pass) done-2h watcher line on terminal [35.83ms]
(pass) empty watcher line on terminal [32.51ms]
(pass) idle-recent watcher line on terminal [45.55ms]
(pass) openable watcher line on terminal [45.61ms]
(pass) unseen watcher line on terminal [43.24ms]
(pass) waiting-quiet watcher line on terminal [54.79ms]
(pass) waiting-stale watcher line on terminal [37.81ms]
(pass) split lines, errors, heartbeat and footer age on terminal [102.31ms]
(pass) missing and retry on terminal [80.53ms]
(pass) old and retry on terminal [88.07ms]
(pass) exit and retry on terminal [195.55ms]
(pass) overflow 20x6 on terminal [35.12ms]
(pass) overflow 30x12 on terminal [34.44ms]
(pass) ambiguous watcher line on desktop [44.39ms]
(pass) busy watcher line on desktop [40.36ms]
(pass) child-other-repo watcher line on desktop [34.17ms]
(pass) children-exact-likely watcher line on desktop [45.11ms]
(pass) done-2h watcher line on desktop [37.55ms]
(pass) empty watcher line on desktop [32.91ms]
(pass) idle-recent watcher line on desktop [48.20ms]
(pass) openable watcher line on desktop [48.22ms]
(pass) unseen watcher line on desktop [46.81ms]
(pass) waiting-quiet watcher line on desktop [41.99ms]
(pass) waiting-stale watcher line on desktop [33.28ms]
(pass) split lines, errors, heartbeat and footer age on desktop [98.71ms]
(pass) missing and retry on desktop [71.37ms]
(pass) old and retry on desktop [70.73ms]
(pass) exit and retry on desktop [158.99ms]
(pass) overflow 20x6 on desktop [33.84ms]
(pass) overflow 30x12 on desktop [36.32ms]
(pass) session.end during streaming cancels child and timers on terminal [35.14ms]
(pass) session.end during backoff cancels child and timers on terminal [28.62ms]
(pass) the command reopens the pane without starting a second watcher on terminal [36.13ms]
(pass) overflow removes recent before waiting before working on terminal [33.31ms]
(pass) session.end during streaming cancels child and timers on desktop [31.89ms]
(pass) session.end during backoff cancels child and timers on desktop [33.97ms]
(pass) the command reopens the pane without starting a second watcher on desktop [22.79ms]
(pass) overflow removes recent before waiting before working on desktop [31.21ms]
(pass) board C text and harness colors on terminal [51.19ms]
(pass) data after a retry resets the backoff on terminal [43.47ms]
(pass) board C text and harness colors on desktop [56.70ms]
(pass) data after a retry resets the backoff on desktop [40.49ms]
(pass) strip matches every key on terminal [140.91ms]
(pass) row actions survive redraw and close when removed on terminal [109.81ms]
(pass) ticks reset checked age without replacing data on terminal [57.47ms]
(pass) notification option true on terminal [44.27ms]
(pass) notification option false on terminal [50.35ms]
(pass) open row overflow at 8 on terminal [49.16ms]
(pass) open row overflow at 7 on terminal [44.17ms]
(pass) open row overflow at 6 on terminal [41.09ms]
(pass) strip matches every key on desktop [127.74ms]
(pass) row actions survive redraw and close when removed on desktop [108.84ms]
(pass) ticks reset checked age without replacing data on desktop [59.68ms]
(pass) notification option true on desktop [51.86ms]
(pass) notification option false on desktop [39.95ms]
(pass) open row overflow at 8 on desktop [49.73ms]
(pass) open row overflow at 7 on desktop [44.58ms]
(pass) open row overflow at 6 on desktop [43.98ms]
(pass) four part cap keeps this sessions newest part on terminal [42.36ms]
(pass) short viewport keeps the expanded child and its buttons together on terminal [56.48ms]
(pass) four part cap keeps this sessions newest part on desktop [35.16ms]
(pass) short viewport keeps the expanded child and its buttons together on desktop [71.10ms]
(pass) non macOS host never claims or displays notifications on terminal [28.50ms]
(pass) notification fallback and spawn failure false on terminal [60.16ms]
(pass) notification fallback and spawn failure true on terminal [60.48ms]
(pass) copy refusal is shown in the footer on terminal [46.05ms]
(pass) non macOS host never claims or displays notifications on desktop [30.63ms]
(pass) notification fallback and spawn failure false on desktop [61.81ms]
(pass) notification fallback and spawn failure true on desktop [54.78ms]
(pass) copy refusal is shown in the footer on desktop [50.73ms]

 76 pass
 0 fail
Ran 76 tests across 1 file. [4.29s]
```
  `claude plugin validate mod/multi-link-pane` → actual result/advisory headings excerpt
  (absolute local paths omitted from the log):
```text
✘ Found 1 problem the plugin directory refuses a submission for:
⚠ Found 1 plugin directory warning:
✔ Validation passed
```
  Default validation exits successfully; the directory submission profile still reports the pre-existing
  missing LICENSE and icon. No license was selected or artwork added in this step.
  `git diff --check` → no output.
  Test-first checks initially produced `16 pass / 48 fail` for the new behavior. An additional expanded-child
  overflow regression then produced `66 pass / 2 fail`, fixed by dropping sibling lines while retaining the
  selected child's row and actions. Final tests include all eleven current pane keys, a synthetic eight-part
  strip, terminal/desktop presses, action refusal, notification success/failure/null fallbacks, disabled and
  non-macOS notifications, duplicate events across watcher retries, and the Phase 16 lifecycle/overflow cases.
- Review: Claude accepted behavior in round 1 and requested S1 segment styling; addressed in the
  review round 1 entry above, awaiting re-review. No staging, commits or pushes.
- Gotchas: The approved types file was present and used. Plugin validate/test both run in this sandbox;
  real terminal/desktop rendering, macOS delivery attribution and timing remain Step 17.4.
  No Rust or PLAN.md changes. Pre-existing untracked `.claude/` and
  `mod/multi-link-pane/tsconfig.json` were left untouched.


### 2026-11-11 — Step 17.2: Review round 2 — Codex
- Did: Removed last_activity from strip_fingerprint per PLAN.md Review log "Step 17.2 tick cost", round 2.
  The session key now hashes only id and state; links, this row's step and the 30 s rebuild cap remain.
  Updated the seam test so an activity-only change at 35 s keeps the previous strip. Another activity-only
  change at 66 s (29 s since the last rebuild) also reuses it; at 67 s the 30 s cap rebuilds it anyway.
  State, new-link, existing-link, id and step invalidation assertions remain in the same test.
- Changed: `crates/cli/src/pane.rs`, `PROGRESS.md` (this round only).
- Verify: Rust PATH set per AGENTS.md. Updated seam test was red before the fix, actual output:
  ```text
assertion `left == right` failed
  left: 6
 right: 5
test pane::tests::strip_cache_rebuilds_only_on_input_changes_or_thirty_seconds ... FAILED
  ```
  After the fix, `TZ=UTC cargo test -p multi-link strip_cache_rebuilds -- --nocapture` → exit 0:
  ```text
test pane::tests::strip_cache_rebuilds_only_on_input_changes_or_thirty_seconds ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 41 filtered out; finished in 0.01s
  ```
  `TZ=UTC cargo test --workspace > /tmp/multi-link-step17.2-round2-test.log 2>&1` → exit 0;
  66 targets, 711 passed, 0 failed, 1 existing ignored (`real_counts`). Actual relevant output:
  ```text
test pane::watch_tests::watch_scripts_match_byte_for_byte_with_and_without_notify ... ok
test pane::tests::strip_cache_rebuilds_only_on_input_changes_or_thirty_seconds ... ok
test pane_matches_all_eleven_pretty_json_keys_byte_for_byte ... ok
test notify_flag_uses_full_refresh_and_writes_permanent_invocation_marker ... ok
  ```
  Actual result lines from the full workspace run:
  ```text
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.95s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.04s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.62s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.93s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.79s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, actual output:
  ```text
    Checking multi-link v0.1.0 (/Users/jarednarvaez/multi-link/crates/cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.64s
  ```
  `cargo fmt --all -- --check` → exit 0, no output.
  `git diff --check` → exit 0, no output.
- Review: Claude, 2 rounds. Round 1: (1) tick CPU failed the NFR on the host: 363 ms/tick with the strip, 388 ms
  with `--notify`, 189 ms without a strip, so the strip's per-pass graph was the cost → rebuild only on input change
  or every 30 s (Review log "Step 17.2 tick cost"); (2) two caches parsed the same rollouts → one `TurnCache`. Round
  2: the change key held `last_activity`, which moves on nearly every tick while a session works → removed. Claude
  re-ran Verify: 711 passed, 0 failed, 1 ignored; clippy, fmt, phase 9 check exit 0; all keys byte for byte.
  **Host CPU (release, real `~`, this repo, `--notify`, 60 s window after the first line, 20 ticks, includes two
  30 s rebuilds): 219 ms/tick (budget ≤ 250 ms: pass)**; round 1 over 30 s: 190 / 193 ms. All lines were `tick`
  (quiet repo). Accepted: three log entries for one step (initial + two rounds) are left as Codex wrote them.
- Gotchas: **Host warm CPU, strip + --notify: 219 ms/tick (Claude, 60 s window on a quiet repo; a working session no longer
  forces a rebuild per tick).**
  Did not run sandbox-blocked ps. The measurement command remains in the original Step 17.2 entry below;
  no new CPU result is claimed.

### 2026-11-11 — Step 17.2: Review round 1 — Codex
- Did: Applied PLAN.md Review log "Step 17.2 tick cost" without changing PLAN.md. The pass closure now caches
  the strip and calls scanner.graph only on a step change, an all-time session `(id, state, last_activity)`
  or link change, or after 30 s. The fingerprint hashes those session fields and every link field directly;
  sorted per-record hashes avoid rebuilds caused only by scanner ordering. Removing the current step clears
  the strip cache. Failed rebuilds do not refresh its timestamp or fingerprint. The rebuild seam verifies
  reuse at 29 s, rebuild at exactly 30 s, state/new-link/step changes, last_activity/id/link-field changes,
  reordered sets and unrelated prompt metadata, timer reset after each rebuild, and step disappearance/return.
  Removed SaidCache and the separate raw_said path: one TurnCache now supplies both details' row said and
  done_runs' turn windows. A parser seam checks one read for unchanged input, shared access by both message
  and completion lookup, and invalidation on growth or same-length rewrite with a new mtime. Original message
  text, completed-turn selection, compressed rollout support and all accepted watcher delivery logic remain.
- Changed: `crates/cli/src/pane.rs`, `PROGRESS.md` (this review round only; prior Step 17.2 integration test
  changes in `crates/cli/tests/pane.rs` are still present).
- Verify: Rust PATH set per AGENTS.md. Both new seam checks were first red (2 rebuilds/reads vs expected 1),
  then green. `TZ=UTC cargo test --workspace > /tmp/multi-link-step17.2-round1-test.log 2>&1` → exit 0;
  66 targets, 711 passed, 0 failed, 1 existing ignored (`real_counts`). Actual relevant output:
  ```text
test pane::tests::turn_cache_shares_parses_and_reads_growth_and_new_mtime ... ok
test pane::tests::turn_cache_reads_original_said_from_compressed_rollouts ... ok
test pane::watch_tests::watch_errors_continue_and_unchanged_success_restores_data ... ok
test pane::watch_tests::simultaneous_watchers_only_one_claims ... ok
test pane::watch_tests::broken_pipe_after_claim_leaves_tombstone ... ok
test pane::watch_tests::watch_first_change_ticks_and_all_phrases_are_filtered_and_flushed ... ok
test pane::watch_tests::watch_parent_one_exits_before_clock_or_pass ... ok
test pane::watch_tests::disappeared_run_and_failed_scan_keep_state_and_body_is_capped ... ok
test pane::watch_tests::watch_scripts_match_byte_for_byte_with_and_without_notify ... ok
test pane::tests::strip_cache_rebuilds_only_on_input_changes_or_thirty_seconds ... ok
test pane::watch_tests::full_runs_include_hidden_old_imports_and_use_each_handoff_window ... ok
test pane_matches_all_eleven_pretty_json_keys_byte_for_byte ... ok
test notify_flag_uses_full_refresh_and_writes_permanent_invocation_marker ... ok
  ```
  Actual result lines from the full workspace run:
  ```text
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.41s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.82s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.04s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.64s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.02s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, actual output:
  ```text
    Blocking waiting for file lock on build directory
    Checking multi-link v0.1.0 (/Users/jarednarvaez/multi-link/crates/cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.60s
  ```
  `cargo fmt --all -- --check` → exit 0, no output.
  `git diff --check` → exit 0, no output.
- Review: Both round 1 findings implemented; awaiting Claude review. All eleven pane JSON keys and both
  watch replay keys still match byte for byte. PLAN.md's existing Claude edits are preserved; no mod edits,
  staging, commits or pushes.
- Gotchas: **Host warm CPU, strip + --notify: 193 ms/tick over 30 s (Claude); superseded by round 2's 219 ms over 60 s.**
  Did not run sandbox-blocked ps. Use the existing release/ps measurement command in the original Step 17.2
  entry below. Software verification passes; the CPU NFR is not claimed as passing.

### 2026-11-11 — Step 17.2: Watcher ticks and at-most-once Codex-done events — Codex
- Did: Added watch-only `--notify`. Completion detection uses all-time refresh links/sessions before pane
  visibility, imports, nesting or the 2 h display window. Each exact/heuristic child run uses the graph's
  half-open window (next same-child hand-off, any step/parent in the repo) and last turn's task_complete
  evidence. Missing rollouts remain known as not done. Baselines, later already-done arrivals, failed passes
  and disappearing runs follow H4. Only notifying watchers attempt a `create_new` claim at
  `<state dir>/notified/<invocation id>`; tombstones are never removed, even after a broken output pipe.
  Events precede the data/tick line and cap original final-message text at 120 Unicode characters.
  Shared the existing raw completion reader with the per-rollout turn cache, preserving whitespace and
  avoiding the graph reader's earlier normalization/truncation. Both caches use file length + mtime.
  Replaced the 30 s data heartbeat with H5 tick lines every unchanged successful tick; errors still reset
  the data change key. Parent teardown and immediate flushing are preserved.
- Changed: `crates/cli/src/pane.rs`, `crates/cli/tests/pane.rs`, `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md. Red/green: the tick/flush test first failed (2 flushes vs 3), then
  passed; the unavailable-rollout baseline first failed ([] vs two not-done runs), then passed; the original
  final-message test first failed (`second answer` vs `second\nanswer`), then passed after sharing raw turns.
  Replay tests consume every tick of `fixtures/expected/pane/watch-script.json`, a fresh marker directory
  and the injected runtime; both watch keys match byte for byte. Coverage includes two hand-offs sharing
  a child id, concurrent watchers forced into the same-tick claim with a barrier, a sequential restart,
  failed scans/disappearance, no claims without notify, Unicode body cap, a broken pipe after claiming,
  full-refresh old/imported runs, exact/heuristic links, half-open windows and cache invalidation.
  A CLI integration test appends completion to a copied synthetic rollout and checks event/data ordering,
  next-tick suppression and the exact permanent marker. All eleven one-shot pane keys still pass.


  Final `TZ=UTC cargo test --workspace > /tmp/multi-link-step17.2-test.log 2>&1` → exit 0;
  66 test targets, 710 passed, 0 failed, 1 existing ignored (`real_counts`). Actual relevant output:
  ```text
test pane::watch_tests::watch_parent_one_exits_before_clock_or_pass ... ok
test pane::watch_tests::watch_errors_continue_and_unchanged_success_restores_data ... ok
test pane::watch_tests::disappeared_run_and_failed_scan_keep_state_and_body_is_capped ... ok
test pane::watch_tests::broken_pipe_after_claim_leaves_tombstone ... ok
test pane::watch_tests::watch_first_change_ticks_and_all_phrases_are_filtered_and_flushed ... ok
test pane::watch_tests::simultaneous_watchers_only_one_claims ... ok
test pane::watch_tests::watch_scripts_match_byte_for_byte_with_and_without_notify ... ok
test pane::watch_tests::full_runs_include_hidden_old_imports_and_use_each_handoff_window ... ok
test pane_matches_all_eleven_pretty_json_keys_byte_for_byte ... ok
test notify_flag_uses_full_refresh_and_writes_permanent_invocation_marker ... ok
  ```
  Actual result lines from the complete workspace run:
  ```text
test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.01s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.82s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.04s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.67s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.93s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.79s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, actual output:
  ```text
    Checking multi-link v0.1.0 (/Users/jarednarvaez/multi-link/crates/cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.78s
  ```
  `cargo fmt --all -- --check` → exit 0, no output.
  `git diff --check` → exit 0, no output.
- Review: Pending Claude (implementer did not sign off the step). All Step 17.2 software checks passed;
  host CPU NFR still pending. No staging, commits, PLAN.md changes or mod changes.
- Gotchas: Per owner's instruction, did not attempt sandbox-blocked `ps`. **Warm CPU ms/tick:
  first build 363 ms (failed; see the round 2 entry's Review for the final 219 ms).** Run the following outside
  the sandbox from the repo root, setting `PANE_SESSION` to a current Claude session with a step.
  The command checks that the strip is populated, drains stdout, samples CPU after the first data line and
  30 s later, and divides the difference by ten ticks. It prints only the measured CPU, never message text.
  `--notify` exercises H4 too and uses the normal permanent marker store (claims count as delivery attempts).

  ```sh
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
cargo build --release -p multi-link
PANE_SESSION='<current Claude session id with a step>' python3 - <<'MEASURE'
import json
import os
import subprocess
import threading
import time

watcher = subprocess.Popen(
    ["target/release/multi-link", "pane", "--json", "--watch", "--notify",
     "--cwd", os.getcwd(), "--session", os.environ["PANE_SESSION"]],
    stdout=subprocess.PIPE, text=True,
)

def cpu_seconds():
    raw = subprocess.check_output(
        ["ps", "-p", str(watcher.pid), "-o", "time="], text=True
    ).strip()
    return sum(float(part) * 60 ** index
               for index, part in enumerate(reversed(raw.split(":"))))

def drain():
    for _ in watcher.stdout:
        pass

try:
    first = json.loads(watcher.stdout.readline())
    assert first.get("strip") is not None, "Use a seen session with a step/strip"
    threading.Thread(target=drain, daemon=True).start()
    before = cpu_seconds()
    time.sleep(30)
    after = cpu_seconds()
    print("warm tick CPU ms =", (after - before) * 1000 / 10)
finally:
    watcher.terminate()
    watcher.wait()
MEASURE
  ```

### 2026-11-11 — Step 17.1: Pane JSON: strip, asked/said, openable — Codex
- Did: Implemented H1 in the pinned field order: top-level strip, row/child asked/said/openable.
  Strip uses graph order and live_parts, latest-part waiting and same-chain done rules,
  floor/minimum active minutes, and null for an unseen, stepless or nested-only current session.
  Added fresh `open --app-only` resolution, refusing Resume with exit 3 before spawning.
  Updated only the watch test's top-level field count; watcher behavior and mod remain unchanged.
- Changed: `crates/cli/src/{pane.rs,main.rs}`, `crates/core/src/graph.rs`, `crates/cli/tests/pane.rs`,
  `crates/cli/Cargo.toml`, `Cargo.lock`, `PROGRESS.md`.
- Review round 1: Addressed both Claude findings. `graph.rs` now contains only the extracted
  `pub fn said(last: Option<&Turn>) -> Option<String>` and its graph call; no rollout parser or I/O
  remains in that helper. The private raw_said reader lives in pane.rs, preserving the explanation that
  normalized text is already cut to 119 characters plus ellipsis. Added the approved workspace-owned
  multi-link-codex CLI path dependency and let Cargo update the lockfile (one dependency-list line;
  no outside crates). The private SaidCache lives outside run's pass closure and is borrowed by details
  on each pass, keyed by transcript path, file length and modification time. Unchanged rollouts reuse
  raw said, including null; changed files are re-read, and read/metadata errors discard cached values.
  A read-count seam proves one read across two unchanged calls, a re-read after growth, and another
  re-read after a same-length rewrite with a new mtime. Added compressed-rollout coverage too.
- Verify: Rust PATH set per AGENTS.md. Original pane tests were red (2 passed, 3 failed). The new
  cache test was first red with undeclared SaidCache, then passed after implementation. Fresh round 1
  `TZ=UTC cargo test --workspace` → exit 0; 66 test targets, 704 passed, 0 failed,
  1 existing ignored (`real_counts`). Actual relevant output:
  ```text
test pane::tests::raw_said_reads_compressed_rollouts ... ok
test pane::tests::said_cache_reuses_unchanged_rollout_and_reads_growth_and_new_mtime ... ok
test pane::tests::strip_statuses_use_current_parts_and_preserve_graph_order ... ok
test pane::watch_tests::watch_first_change_heartbeat_and_all_phrases_are_filtered_and_flushed ... ok
test nested_current_session_has_no_strip ... ok
test said_uses_last_completed_real_turn_and_cuts_at_120_unicode_characters ... ok
test app_only_accepts_app_link_and_refuses_resume_after_app_record_disappears ... ok
test pane_matches_all_eleven_pretty_json_keys_byte_for_byte ... ok
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.42s
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; actual final output:
  ```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.19s
  ```
  `cargo fmt --all -- --check` and `git diff --check` → no output, exit 0.
  `TZ=UTC bash fixtures/expected/phase9/check.sh 6` → exit 0, actual output:
  ```text
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   graph-1.1.json (^9[1-6])
ok   graph-2.1.json (^9[1-6])
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   stderr unreadable: 1
ok   open open 92000001
ok   open open 92000002
ok   open open 92000003
ok   open open 92000004
ok   open open 92000005
ok   open open 92000006
ok   open open 92000007
ok   open open 92000008
ok   open open 92000009
ok   open open 92000010
ok   open open 92000011
ok   open open 92000012
ok   open open 92000013
ok   open open 92000014
ok   open open 94000001
ok   open open 94000002
ok   open open 94000003
ok   open open 94000004
ok   open open 94000006
ok   open open 94000007
ok   open fork 92000001
ok   open fork 92000002
ok   open fork 92000003
ok   open fork 92000004
ok   open fork 92000005
ok   open fork 92000006
ok   open fork 92000007
ok   open fork 92000008
ok   open fork 92000009
ok   open fork 92000010
ok   open fork 92000011
ok   open fork 92000012
ok   open fork 92000013
ok   open fork 92000014
ok   open fork 94000001
ok   open fork 94000002
ok   open fork 94000003
ok   open fork 94000004
ok   open fork 94000006
ok   open fork 94000007
note lanes.json is checked by the 9.5 worker-to-App test, not here
  ```
  `TZ=UTC bash fixtures/expected/phase12/check.sh` → exit 0, actual output:
  ```text
ok   links.json
ok   overview.json
ok   graph-1.1.json
  ```
  `TZ=UTC bash fixtures/expected/phase13/check.sh` → exit 0, actual output:
  ```text
ok   links.json
  ```
  `TZ=UTC bash fixtures/expected/phase14/check.sh` → exit 0, actual output:
  ```text
ok   live.json
  ```
- Review: Claude, 2 findings in round 1, both fixed: (1) `graph.rs` got a two-mode `said(last, rollout)` with a
  rollout parser inside; now graph keeps a one-mode `said(last)` and the raw reader is private in `pane.rs` (needs
  the workspace `multi-link-codex` crate, approved in the Review log); (2) watch mode re-read every visible rollout
  each pass; now cached by (length, mtime) with a read-count test. Two scope stops (no helper to make `pub`; the
  CLI dependency) were approved and logged. Claude re-ran Verify: 704 passed, 0 failed, 1 ignored; clippy, fmt,
  phase 9/12/13/14 checks exit 0, matching the entry. On real data the strip read plan ✓ → implement ✓ → review ✓
  → fix round 1 ✓ → review running. Accepted as is: the cache never evicts (bounded by sessions seen in one
  watcher's life).
- Gotchas: Raw said stays in pane.rs because H1 needs 120 characters plus an ellipsis, whereas graph's
  normalized message was already cut at 119 plus ellipsis. The synthetic temp-rollout test covers lengths
  0/119/120/121 with multibyte characters, an imported completion and a later open turn. The raw reader
  uses the existing open_rollout API, retaining compressed JSONL support. App-record removal in the
  desktop fixture yields Unavailable (app_session); Resume refusal is tested separately with a CLI
  fixture, including without dry-run. AppLink uses dry-run and existing command-construction coverage.
  No keys/check scripts or mod changed; no watcher loop changes. Claude's PLAN.md edits and existing
  untracked files left untouched. Nothing staged or committed.

### 2026-11-11 — Step 17.0: Rules, pane keys, API check — Claude
- Did: mode check `codex-quota.py` → `primary: 2%`, `secondary: 61%`, `CODEX: OK` (17.0 is a Claude step). **H6
  spike** (scratch mod `mlspike17`, never in the repo): in an interactive Claude Code 2.1.322 PTY session in
  `~/multi-link-sandbox/repo-a` (`CLAUDE_CODE_*` cleared), `session.start` ran `osascript -e 'on run argv' -e 'display
  notification (item 2 of argv) with title (item 1 of argv)' -e 'end run' <synthetic title> <synthetic body>` by
  `$.process.run` → `exit=0 stderr=""`; `$.ui.notify` → `{"isSent":false,"reason":"no-channel"}`; `ui.open` →
  `{"isPlaced":true} surface=terminal`. Scripted ctrl+x tab / hotkey presses did not reach the pane. `claude plugin
  test` on the spike: a `Button` press runs `onPress`, which calls `$.ui.copy` → received `synthetic-id-123@terminal`
  and `@desktop` (1 pass); a `ui.press` hook without `{ element }` is skipped by the engine. Codex `said` on real
  data: 16.4's `graph` output had it for `01a1cc93` (length only noted, text not copied here).
  **Keys** by a scratch writer (`k17/gen.py`): base = HEAD's `pane --json`; `asked` from `sessions --json --since all`;
  Codex `said` read straight from each rollout's last non-imported `task_complete` (`last_agent_message`, 120-char cut);
  `openable` from `open --dry-run` (`action == app_link`); `strip` from `graph --json` plus a Python mirror of
  `graph::live_parts` and the H1 status pins. Checked: removing the new fields from each new key gives the old key
  exactly (10/10). Strips: `busy` 3.2 [codex implement running 54m]; `children-exact-likely` 2.4 [claude plan done
  15m ▌, codex implement running 12m]; `ambiguous` 5.1 [claude plan done ▌, codex implement null, claude review null
  ▌]; `idle-recent` 2.5 (7 parts, ▌ on two); null for `waiting-stale`, `done-2h`, `unseen`, `child-other-repo`,
  `empty`, `waiting-quiet`, `openable`. `openable: true` only for `92000002` in `openable.json`. Watcher keys from the
  synthetic `watch-script.json` (7 ticks: run a done at tick 1 → event; an already-done run at start and one first
  seen done at tick 4 → no event; a failed pass at tick 3 → error line, then data; a second hand-off on the same Codex
  id done at tick 6 → event).
- Changed: `docs/design.md`, `fixtures/expected/pane/{busy,waiting-stale,done-2h,children-exact-likely,unseen,ambiguous,child-other-repo,idle-recent,empty,waiting-quiet}.json`
  (H1 fields only), `fixtures/expected/pane/{openable.json,watch-script.json,watch-codex-done.jsonl,watch-tick.jsonl}`,
  `PLAN.md` (17.0 text and Review log "Step 17.0 amendments"), `PROGRESS.md`. Board 17 was committed with the plan.
- Key commands (`TZ=UTC`, `multi-link pane …`): the ten 16.0 commands unchanged (Step 16.0 entry), plus
  `openable`: `pane --json --root fixtures/sessions-phase9 --now 2026-11-09T06:00:00Z --cwd /home/dev/repo-p9c --session 92000002-0000-4000-8000-000092000002`.
- Verify: `TZ=UTC cargo test --workspace --no-fail-fast` → 696 passed, 2 failed, 1 ignored; the two failures are the
  key readers (`pane_matches_all_ten_pretty_json_keys_byte_for_byte`, and
  `pane::watch_tests::watch_first_change_heartbeat_and_all_phrases_are_filtered_and_flushed`, which counts the key's
  top-level fields), as expected after the additive key change. H6 recorded above. W10 scan
  `rg -l "/Users/|\bsk-|api_key" fixtures --glob '!raw/**' --glob '!README.md'` → nothing.
- Review: Claude's own step (rules, keys, scratch spike); no code. Keys checked against independent sources as above.
- Gotchas: zsh does not split `$var` into words (`set -- $spec` and `bash dir/$s` both break); use Python or `${=var}`.
  `open --dry-run` prints JSON then a message line and exits non-zero for unavailable; parse up to the last `}`.

### 2026-11-11 — Step 16.4: Real check, README, owner confirm — Claude
- Did: Release build of `93194cc`. Scratch PTY harness (session scratchpad, not in the repo): `claude --plugin-dir
  /Users/jarednarvaez/multi-link/mod/multi-link-pane` at 150 × 44 in this repo, `target/release` first on `PATH`,
  a draining reader thread and a screen model; timed first pane text and first data, `ps` after start and 5 s after
  `/exit`, saved screens. Run 1 inherited `CLAUDE_CODE_CHILD_SESSION` (no transcript saved → `not seen yet`, correct);
  runs 2–4 cleared `CLAUDE_CODE_*` and set `CLAUDE_CODE_FORCE_SESSION_PERSISTENCE=1`. One run with the installed 1.12
  binary. Watcher CPU per tick: a fresh watcher, `ps -o time` after its first line and 30 s later. README section
  "Claude Code pane" (the captured pane text, not an image) and a limitation line; `docs/phase16-check.md`.
- Changed: `README.md`, `docs/phase16-check.md`, `PROGRESS.md` (also corrects the 16.2 Review note about a phase9
  FAIL: it was my zsh loop, not the check).
- Verify: whole-phase Verify: `TZ=UTC cargo test --workspace` → 698 passed, 0 failed, 1 ignored; clippy `-D warnings`
  and `fmt --check` clean; `bash fixtures/expected/phase9/check.sh 6` rc 0, `phase12`, `phase13`, `phase14` ok;
  `git diff --name-only c29c516 -- fixtures/expected` shows only Phase 15 `tui/loading-*` and Phase 16 `pane/*`;
  `claude plugin validate mod/multi-link-pane` → `✔ Validation passed`; `claude plugin test` → 46 pass, 0 fail;
  W10 scan → nothing (rc 1). Real check: pane text first seen 0.94 / 0.83 / 0.84 s and data 2.81 / 2.90 / 2.53 s
  after launching `claude` (from the pane opening: 1.87 / 2.07 / 1.69 s, median 1.87 s ≤ 2.5 s); run 2 after one
  prompt: `▌● this session` under `recent (2h)`, the coordinator under `working` with its four Codex runs nested `✓`;
  `ps` one watcher with parent `claude`, none 5 s after `/exit` in all five runs; old binary → `This multi-link is too
  old for the pane. Update it.`; warm CPU 1.40 s over 10 ticks = 140 ms per tick (≤ 250 ms).
- Owner steps (pending; one command each, absolute paths): install the 1.13 binary
  `cargo install --path /Users/jarednarvaez/multi-link/crates/cli --locked`; turn the mod on for every Claude Code
  session including the desktop app `python3 -c 'import json,os;p=os.path.expanduser("~/.claude/settings.json");d=json.load(open(p)) if os.path.exists(p) else {};d.setdefault("env",{})["CLAUDE_CODE_PLUGIN_DIRS"]="/Users/jarednarvaez/multi-link/mod/multi-link-pane";json.dump(d,open(p,"w"),indent=2)'`,
  then restart the desktop app.
- Review: Claude's own step (docs only; no code), as 15.4. Numbers in README and the check doc are from the scratch
  `events.txt` of runs 1–4 and `old`, and the CPU script output. Board check: the captured pane has board 16 C's
  header, rules, `▌` row and `└` children. Desktop half: done by the owner 2026-11-11 (screenshot; recorded in
  `docs/phase16-check.md` → "Desktop app"). Owner's answer (2026-11-11): "Looks good". Phase 16 closed.
- Gotchas: a `claude` started from a Claude Code session inherits `CLAUDE_CODE_CHILD_SESSION` and saves no
  transcript, so the pane cannot find it; clear `CLAUDE_CODE_*` for real checks. In zsh, `bash dir/$s` with
  `s="check.sh 6"` is one word.

### 2026-11-11 — Step 16.3: The mod — Codex
- Did: Added the `multi-link-pane` mod (0.1.0), `/multi-link`, pane `multi-link` on session start, and the exact
  `multi-link pane --json --watch --cwd <cwd> --session <id>` argument vector. Buffered stdout until newlines;
  malformed/unknown-version lines are ignored, error lines keep the report, valid lines refresh the receipt clock.
  Spawn rejection and a quick unrecognized-subcommand exit show G4's actionable messages. Other exits restart
  after 2/4/8/16/32/60/60… seconds; valid data resets the delay. Session end cancels the child, retry and age timers.
  Unload relies on the host's stream/timer ownership, without a plugin unload hook (see Gotchas).
  Draws surface-resolved Box/Text only, using viewport rows/columns: sections, harness colours, this-session marker,
  nested children, likely links, hand-off outcomes, updated-age/error footer. Overflow removes whole other-session
  rows from recent, waiting, working (counting rows in `+N more`); pins this session, caps its children at three and
  counts the remainder in `+N runs`. At 20×6 the section heading yields and one child fits; four are summarized.
- Changed: `mod/multi-link-pane/.claude-plugin/plugin.json`, `mod/multi-link-pane/hooks/hooks.json`,
  `mod/multi-link-pane/hooks/register.tsx`, `mod/multi-link-pane/types/index.d.ts`,
  `mod/multi-link-pane/hooks/register.test.tsx`, `PROGRESS.md`.
- Verify: Both requested commands ran inside the sandbox; neither needed an install. Test-first run against the
  empty register: `0 pass`, `34 fail` (missing drawings, as expected). Final requested runs:
  `claude plugin validate mod/multi-link-pane` → exit 0; actual decisive output (directory notices also pasted):
  ```text
  ❯ ./register.tsx hooks: session.start, session.end, command.run{command=multi-link}, ui.render{component=Pane, requestId=multi-link}
  ❯ ./register.tsx gating hook without .catch: command.run{command=multi-link}
  ❯ ./register.tsx calls: $.clock.after, $.clock.every, $.clock.now, $.command.register, $.process.spawn, $.session.id, $.state.get, $.state.set, $.ui.open, $.ui.resolve
  ❯ ./register.tsx state writes: multi-link-pane.pane
  ❯ ./register.tsx state reads: multi-link-pane.pane
  ✘ Found 1 problem the plugin directory refuses a submission for:
  ❯ LICENSE: License missing (LICENSE_MISSING). This plugin has no LICENSE file and plugin.json has no `license`. The directory requires one of them.
  ⚠ Found 1 plugin directory warning:
  ❯ .claude-plugin/plugin.json icon: No icon (ICON_MISSING). No icon found in the plugin. The validator looked for `icon` in plugin.json and for a file at .claude-plugin/icon.png or .svg, assets/icon.png or .svg, or a similar path.
  These plugin directory checks do not change the result below. To fail on problems the directory refuses a submission for, run this command with --profile directory.
  Digest: sha256-GGW56/gZb5unli2mEWe31AaFQQlKxmwDMOPj6fIafag=
  ✔ Validation passed
  ```
  `claude plugin test mod/multi-link-pane` → exit 0; actual summary:
  ```text
   46 pass
   0 fail
  Ran 46 tests across 1 file. [2.39s]
  ```
  Tests run on terminal and desktop: all ten 16.0 keys fed as watcher lines, a literal board C drawing and harness
  colours, all G4 states, split lines, malformed/unknown versions, error preservation, heartbeat/age, retry cap and
  reset, 20×6/30×12 working-only overflow, recent/waiting/working removal order, command reopening without another
  watcher, and session-end cleanup while streaming and during backoff (no further child starts or clock updates).
  An independent Python comparison of the embedded test keys against the read-only JSON files printed:
  ```text
  10 embedded watcher keys equal fixtures/expected/pane/*.json exactly.
  ```
  `git diff --check` → no output, exit 0 (tracked changes); nothing staged, committed or pushed.
- Review: Claude, 2026-11-11, mode Codex loop (`codex-quota.py` → `CODEX: OK`). Diff read: all of
  `mod/multi-link-pane/` (manifest, hooks list, `register.tsx`, `types/index.d.ts`, tests); no file outside the step.
  No findings. Checked by reading: a stdout chunk is buffered to newlines; an `error` line sets the footer notice and
  keeps the last report; spawn rejecting before any chunk → `missing`; a fast exit with `unrecognized subcommand` →
  `too old`; else backoff 2→60 s, reset by data; `session.start` stops a previous lifecycle (hot reload) and
  `session.end` cancels the stream, the retry timer and the 1 s footer tick; overflow keeps header, this session and
  footer, drops `recent` → `waiting` → `working`, `+N more` counts sessions. Colours follow board 16 C (glyph and
  row in the harness colour); G3's green/amber are not used because the approved board does not use them.
  Unload: the kit cannot raise one (Codex's note is right); the engine's contract kills a spawn on unload and 16.0's
  spike proved it with `ps` (G5 (d)); 16.4 checks it again on the real mod. Re-ran Verify: `claude plugin validate
  mod/multi-link-pane` → `✔ Validation passed` (directory-only notices: no icon, no licence file); `claude plugin test
  mod/multi-link-pane` → 46 pass, 0 fail (23 cases × terminal and desktop). Matches Codex's paste.
- Gotchas: The mod/test environment has no Node and cannot import JSON files, so the test embeds the ten read-only
  answer-key objects, checked for exact equality above; it does not edit the keys. A process.spawn mock must return
  `{ value: { code, signal } }`, and a session.end mock must return `{ sessionId }`. Earlier test iterations exposed
  and corrected those mock shapes. The kit exposes no unload call. An attempted `$.ui.close` from its Engine was
  undefined; the API types further say `PaneCloseOrigin`'s `unload` does **not** run the opener's hooks, so a ui.close
  cleanup hook would not provide unload teardown and was removed. The host explicitly owns cancellation of each
  spawned stream and each clock timer on module unload (types: process.spawn and clock docs); the 16.0 spike also
  observed child death on hot reload. Claude still needs a real unload check with this mod, including backoff.
  Validation's optional directory license/icon notices were left visible: no license chosen for the unlicensed
  repo, no submission assets added. Existing untracked `.claude/` was left alone. No npm, tsc or new installs.

### 2026-11-11 — Step 16.2: `pane --watch` — Codex
- Did: Added `--watch` around the existing pane projection and exact `tui::refresh_with_live` pass, with one
  long-lived Scanner and one shared timestamp per pass. Used monotonic elapsed time for the heartbeat and
  subtracted pass time from the 3 s wait. Change detection removes `now` and every nested `phrase` only;
  errors produce escaped compact `{ v: 1, error }` lines and the next good pass restores the data.
  Every emitted line ends in a newline and flush. Captured parent pid before setup, checked before/after passes,
  and exited before a pass if already orphaned. Added injectable clock, wait and parent checks and deterministic
  tests for immediate/flush ordering, suppression of row/child phrases and now, a child's real state change,
  the 30 s heartbeat, error recovery and pid 1. Real process tests launch via an exiting Python parent and
  check watcher pipe closure; the orphan-at-start child waits for getppid == 1 before exec.
- Changed: `crates/cli/src/pane.rs`, `crates/cli/tests/pane.rs`, `PROGRESS.md`.
- Verify: Each cargo command used `export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"` first.
  `cargo test --workspace` → exit 0 after review fix round 1; actual result lines
  (full log in `/tmp/multi-link-16.2-r1-tests.log`):
```text
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.77s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.14s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.65s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.89s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.78s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
  Sum from those result lines: 698 passed, 0 failed, 1 ignored. Actual watcher/key test output:
```text
test pane::watch_tests::watch_parent_one_exits_before_clock_or_pass ... ok
test pane::watch_tests::watch_errors_continue_and_unchanged_success_restores_data ... ok
test pane::watch_tests::watch_first_change_heartbeat_and_all_phrases_are_filtered_and_flushed ... ok
test watcher_started_with_parent_one_exits_immediately ... ok
test pane_matches_all_ten_pretty_json_keys_byte_for_byte ... ok
test watcher_exits_within_one_tick_after_parent_exits ... ok
```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0 after review fix round 1; actual final output:
```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.57s
```
  `cargo fmt --all -- --check` → exit 0 after review fix round 1, no output.
  The check.sh, release build and timing results below are retained from initial verification; not rerun for
  this parent-API-only review fix.
  `fixtures/expected/phase14/check.sh` → exit 0, actual output:
```text
ok   live.json
```
  `cargo build --release -p multi-link` → exit 0, actual final output:
```text
    Finished `release` profile [optimized] target(s) in 1.88s
```
  `python3 /tmp/multi-link-16.2-measure.py` → exit 0, actual output:
```text
release real-home first line: 1774.634417 ms; v=1; compact NDJSON=True
pass 0 1591.376167 ms
pass 1 161.857917 ms
pass 2 161.893958 ms
pass 3 156.428542 ms
pass 4 162.767042 ms
pass 5 174.744083 ms
pass 6 195.196458 ms
pass 7 174.229459 ms
pass 8 164.117541 ms
pass 9 181.272209 ms
pass 10 158.223084 ms
10-tick exact pane refresh median: 163.442292 ms
```
  Timing method: real home (no `--root`, no fixed `--now`), current checkout, `--session phase16-measurement`.
  Python measured process spawn through the first complete stdout line from the unmodified release binary,
  parsed it, then terminated/reaped it. A scratch crate in `/tmp/multi-link-16.2-timing` copied the exact CLI
  sources, used this workspace's path dependencies and lockfile, and was built with `cargo build --release
  --manifest-path /tmp/multi-link-16.2-timing/Cargo.toml` (CARGO_TARGET_DIR set to this checkout's target).
  Its only instrumentation surrounded the watch pass closure (refresh_with_live + project + JSON conversion)
  with Instant, wrote each duration to stderr, and exited after the cold pass plus ten real 3 s ticks.
  The median excludes pass 0 and includes exactly passes 1–10. Raw timings are in
  `/tmp/multi-link-16.2-pass-times.log`; measurement output in `/tmp/multi-link-16.2-timing-results.log`.
  First-line ≤ 2500 ms and refresh median ≤ 250 ms both pass. No timing hooks or harness sources added to the repo.
- Review: Claude, 2026-11-11, mode Codex loop (`codex-quota.py` → `CODEX: OK`). Diff read: `pane.rs` (watch loop,
  runtime trait, `without_times`), `tests/pane.rs` (two process tests). Round 1, one finding: a hand-written `unsafe
  extern "C" getppid` → replaced with the safe `std::os::unix::process::parent_id()` under `cfg(unix)` (non-Unix:
  backstop off; Windows is a non-goal). Re-review clean. Checked by reading: the change key drops `now` and every
  nested `phrase`; an error resets the key so the next success prints; a broken stdout pipe exits (the mod is gone).
  Re-ran Verify: `TZ=UTC cargo test --workspace` → 698 passed, 0 failed, 1 ignored; clippy `-D warnings` and
  `fmt --check` clean; `phase9/check.sh 6` rc 0 (an earlier FAIL was my zsh loop passing `check.sh 6` as one file
  name, not the check; found in 16.4), `phase12`, `phase13`, `phase14` ok; `git diff HEAD -- fixtures/expected`
  empty. Codex timed in its sandbox, where `ps` probes fail soft, so I re-timed outside it (release, real `~`, this
  repo and this coordinator session, 3 fresh watchers): first line 2012 / 1569 / 1640 ms, **median 1640 ms** (≤ 2500),
  `seen: true`, the session under `working`. The 163 ms refresh median is Codex's sandbox figure.
- Gotchas: Initial new unit tests failed to compile because WatchRuntime/watch did not exist yet. The first
  process-test version failed at spawning `ps` with EPERM in the sandbox; replaced its exit observation with
  inherited-pipe EOF (no ps dependency in the tests). Real-home measurement used this same sandbox, where process
  probes fail soft when ps is unavailable. Pre-existing untracked `.claude/` left untouched. A broken output pipe
  returns an error/exits because it cannot deliver an error line; refresh and clock errors keep retrying.
  Workspace unsafe search also found pre-existing `unsafe extern` / `unsafe` fcntl calls in
  `crates/claude/src/invocation.rs:319,325`; left untouched per this review's scope. The pane now has no unsafe;
  the entire workspace is not unsafe-free. Non-Unix compilation was not run; the Unix parent process tests passed.

### 2026-11-11 — Step 16.1: `multi-link pane --json` — Codex
- Did: Implemented the pane projection from the TUI worker's `refresh_with_live` and public
  `view::{sessions,home_label,placing,status,outcome}` functions, with a single `Options.now` and the fixed 2 h
  window. Retained quiet waiting/stale rows, flattened started Claude children into rows, preserved overview
  placement and cross-repo boundaries, and sorted sections/children per the pins. Local serializable pane types;
  no core type changes. `tui.rs` changes only `refresh_with_live` to `pub(crate)`.
  Added all ten exact Step 16.0 command/key comparisons (pretty JSON including the final newline, `TZ=UTC`),
  unit tests for started Claude children and unlinked hand-offs, ordering/imports, hidden-parent promotion,
  public repo-view labels/phrases, and a test building a temporary git repo plus worktree and refreshing in real
  repo mode from two nested cwds. Initial golden test failed with `unrecognized subcommand 'pane'` before implementation.
- Changed: `crates/cli/src/{pane.rs,main.rs,tui.rs}`, `crates/cli/tests/pane.rs`, `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md. All commands below exited 0; pasted actual output (workspace output reduced
  to target headings and result lines). Workspace totals from those lines: 693 passed, 0 failed, 1 ignored
  (the pre-existing optional real-file test).
  - `TZ=UTC cargo test --workspace`:

```text
     Running unittests src/main.rs (target/debug/deps/multi_link-7cedda362c6e1306)
test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
     Running tests/cli.rs (target/debug/deps/cli-e0787a43f85482d2)
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.89s
     Running tests/graph.rs (target/debug/deps/graph-d837f77b20803abc)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
     Running tests/hook_check.rs (target/debug/deps/hook_check-2c16d8f4ae4adc11)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.82s
     Running tests/live.rs (target/debug/deps/live-c6d920ae5b40705e)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/pane.rs (target/debug/deps/pane-178a448141df46ab)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
     Running tests/phase12.rs (target/debug/deps/phase12-a59e2b11bb6caddf)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/phase13.rs (target/debug/deps/phase13-7e99d17abc451c6e)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/phase9.rs (target/debug/deps/phase9-10b001eb96b77347)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running unittests src/lib.rs (target/debug/deps/multi_link_claude-3007d227ffc8769a)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/app_record.rs (target/debug/deps/app_record-c6763d08001f1e6c)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/calls.rs (target/debug/deps/calls-45eb22dd6795bdda)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/fixtures.rs (target/debug/deps/fixtures-499f3a2e254ccd24)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
     Running tests/invocation.rs (target/debug/deps/invocation-3e16b8bc37be96e1)
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
     Running tests/last_turn.rs (target/debug/deps/last_turn-b71057714a3b5979)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/phase6.rs (target/debug/deps/phase6-e2db30bd15f79be3)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/phase7.rs (target/debug/deps/phase7-2c8b6d1063153907)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running unittests src/lib.rs (target/debug/deps/multi_link_codex-e78d5e99d41da51c)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/calls.rs (target/debug/deps/calls-586748b917de5a55)
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/compressed.rs (target/debug/deps/compressed-a0fe57bc51027b49)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.66s
     Running tests/fixtures.rs (target/debug/deps/fixtures-ba5f82572d1c4a8d)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/phase6.rs (target/debug/deps/phase6-d1fad84a09bb2e55)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running unittests src/lib.rs (target/debug/deps/multi_link_core-b9f5577635e977c9)
test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.88s
     Running tests/compressed.rs (target/debug/deps/compressed-23fd67b2786194f6)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/graph.rs (target/debug/deps/graph-367a1c66ea7fcc9e)
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/graph_commit.rs (target/debug/deps/graph_commit-413fc54f4401db0c)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
     Running tests/hook.rs (target/debug/deps/hook-9d2c70dbcc7efb25)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
     Running tests/links.rs (target/debug/deps/links-f8b4f67c998fff9c)
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
     Running tests/live.rs (target/debug/deps/live-e242d43fd303903b)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/overview.rs (target/debug/deps/overview-958464a7d8ad16d7)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
     Running tests/parallel_scan.rs (target/debug/deps/parallel_scan-e4e2339dd6a5af37)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.78s
     Running tests/phase6.rs (target/debug/deps/phase6-a939478b5426845c)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/repo_probe.rs (target/debug/deps/repo_probe-489df8ea92fb062c)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
     Running tests/session_end.rs (target/debug/deps/session_end-97da6fcc150d54f9)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/sessions.rs (target/debug/deps/sessions-59bcd1019b17c9c4)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
     Running tests/started.rs (target/debug/deps/started-d7b62c3a1b08ef27)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
     Running tests/transcript_exit.rs (target/debug/deps/transcript_exit-27b68d6dc1e2bf21)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
     Running unittests src/lib.rs (target/debug/deps/multi_link_hook-62ceaf9ce3262d6f)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/hook.rs (target/debug/deps/hook-312d13ac9a0e316c)
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
     Running tests/settings.rs (target/debug/deps/settings-22861df00bc1ebd7)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/multi_link_model-df157fa80d5a3417)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/calls.rs (target/debug/deps/calls-5289ef8606320dd4)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/graph.rs (target/debug/deps/graph-bb878fdee95626eb)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/live.rs (target/debug/deps/live-9d6ecd5fdabf883c)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/model.rs (target/debug/deps/model-452af6df96c138b7)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running unittests src/lib.rs (target/debug/deps/multi_link_plan-b5ab79d96e45b0d3)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/fixtures.rs (target/debug/deps/fixtures-1edf5b4804af5d92)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/multi_link_tui-21d83cafd7849385)
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
     Running tests/graph.rs (target/debug/deps/graph-005a83afaca3727a)
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s
     Running tests/home.rs (target/debug/deps/home-5a2605d8fc0f75cd)
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
     Running tests/live.rs (target/debug/deps/live-5cd55c828b339aac)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running tests/loading.rs (target/debug/deps/loading-0fd56bea0a6c1888)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
     Running tests/loading_app.rs (target/debug/deps/loading_app-5bd2efe9d42b5517)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/phase6_shape.rs (target/debug/deps/phase6_shape-e35530f2f0f05366)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/phase8_shape.rs (target/debug/deps/phase8_shape-abfb141048af4856)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/started.rs (target/debug/deps/started-da6f7bfb46539738)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/theme.rs (target/debug/deps/theme-501b876d9ddbbc6b)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/timeline.rs (target/debug/deps/timeline-e7b6c91cdfec15b4)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/view.rs (target/debug/deps/view-065c7d3f4df7468d)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_claude
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_codex
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_hook
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_model
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_plan
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_tui
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

  New unit tests from that run:

```text
test pane::tests::unlinked_handoff_has_words_and_no_child ... ok
test pane::tests::this_first_then_flat_newest_first_then_id_and_no_imports ... ok
test pane::tests::started_claude_child_is_its_own_row_in_its_state_section ... ok
test pane::tests::hidden_parent_promotes_codex_but_preserves_placing_step_and_label ... ok
test pane::tests::rows_labels_and_phrases_equal_public_repo_view_functions ... ok
test pane::tests::temporary_git_worktree_cwds_share_one_pane_from_real_refresh ... ok
```

  - `TZ=UTC cargo test -p multi-link --test pane -- --nocapture` (runs each of the ten exact commands listed in 16.0):

```text
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.17s
     Running tests/pane.rs (target/debug/deps/pane-178a448141df46ab)

running 1 test
test pane_matches_all_ten_pretty_json_keys_byte_for_byte ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
```

  - `cargo clippy --workspace --all-targets -- -D warnings`:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.46s
```

  - `cargo fmt --all -- --check`, `git diff --check`, `git diff --exit-code -- fixtures/expected`:
    no output, exit 0 each (keys and check scripts unchanged).
  - `TZ=UTC bash fixtures/expected/phase9/check.sh 6`:

```text
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   graph-1.1.json (^9[1-6])
ok   graph-2.1.json (^9[1-6])
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   stderr unreadable: 1
ok   open open 92000001
ok   open open 92000002
ok   open open 92000003
ok   open open 92000004
ok   open open 92000005
ok   open open 92000006
ok   open open 92000007
ok   open open 92000008
ok   open open 92000009
ok   open open 92000010
ok   open open 92000011
ok   open open 92000012
ok   open open 92000013
ok   open open 92000014
ok   open open 94000001
ok   open open 94000002
ok   open open 94000003
ok   open open 94000004
ok   open open 94000006
ok   open open 94000007
ok   open fork 92000001
ok   open fork 92000002
ok   open fork 92000003
ok   open fork 92000004
ok   open fork 92000005
ok   open fork 92000006
ok   open fork 92000007
ok   open fork 92000008
ok   open fork 92000009
ok   open fork 92000010
ok   open fork 92000011
ok   open fork 92000012
ok   open fork 92000013
ok   open fork 92000014
ok   open fork 94000001
ok   open fork 94000002
ok   open fork 94000003
ok   open fork 94000004
ok   open fork 94000006
ok   open fork 94000007
note lanes.json is checked by the 9.5 worker-to-App test, not here
```

  - `TZ=UTC bash fixtures/expected/phase12/check.sh`:

```text
ok   links.json
ok   overview.json
ok   graph-1.1.json
```

  - `TZ=UTC bash fixtures/expected/phase13/check.sh`:

```text
ok   links.json
```

  - `TZ=UTC bash fixtures/expected/phase14/check.sh`:

```text
ok   live.json
```

- Review: Claude, 2026-11-11, mode Codex loop (`codex-quota.py` → `primary: 2%`, `CODEX: OK`). Round 0 was a
  file-scope stop (Review log "Step 16.1 file scope"); round 1 implemented. Diff read: `pane.rs`, `main.rs` wiring,
  `tui.rs` (one `pub(crate)`), `tests/pane.rs`; no other file touched. No findings. Notes, not defects: `seen` is
  true only for a visible row (a session with no activity in the window and not waiting reads as unseen; a new session
  has activity after its first prompt, so G4's placeholder covers the gap); `--json` is `required` so 16.2's
  `--watch` sits beside it. Re-ran Verify: `TZ=UTC cargo test --workspace` → 693 passed, 0 failed, 1 ignored; clippy
  `-D warnings` and `fmt --check` clean; all ten 16.0 key commands → `cmp` equal 10/10; `phase9/check.sh 6` (bash),
  `phase12`, `phase13`, `phase14` all `ok`; `git diff HEAD -- fixtures/expected` empty. Matches Codex's paste.
- Gotchas: `--root` selects fixture mapping; the worktree test explicitly uses `RepoMode::Real` with synthetic
  transcripts under a temporary root. No network or real agent data needed. Existing PLAN.md edits and untracked
  `.claude/` left untouched. No files staged or committed. `--watch` remains Step 16.2.

### 2026-11-11 — Step 16.0: Rules, pane goldens, API check — Claude
- Did: **G5 spike first** (mode check `python3 -I tools/codex-quota.py` → `primary: 2%`, `secondary: 57%`, `CODEX: OK`;
  16.0 is a Claude step, so no Codex hand-off). A throwaway mod `mlspike` in this session's scratchpad (never in the
  repo; `session.start` logs ids, registers `/mlspike`, opens pane `mlspike`, a `$.clock.every(1000)` atom tick, a
  `$.process.spawn` child `/bin/sh -c 'while :; do echo line $i; sleep 1; done' mlspike-child-<v>`; `classic.SessionStart`
  and `session.end` log ids; `ui.render` logs the tick it read) loaded with `claude --plugin-dir <mod>` in an
  interactive Claude Code 2.1.322 session in a Python PTY (160 × 50, reader thread draining the master) in
  `~/multi-link-sandbox/repo-a`. The harness sent one prompt, typed `/mlspike`, edited the module's `VERSION`
  (`v1` → `v2`, a hot reload = unload + load), ran `ps` after start, after the reload and 5 s after `/exit`.
  Logged values (run 4, `a64ec0fe-9409-4ef6-a2f7-70f3f658bfa4`):
  - (a) `classic.SessionStart session_id=a64ec0fe-… transcript_path=~/.claude/projects/<encoded sandbox cwd>/a64ec0fe-9409-4ef6-a2f7-70f3f658bfa4.jsonl source=startup`;
    `session.start sessionId=a64ec0fe-… surface=terminal interactive=true surfaces=["terminal"]`; `session.end
    sessionId=a64ec0fe-… reason=prompt_input_exit`. The file exists (49 records) and every record's `sessionId` is
    `a64ec0fe-9409-4ef6-a2f7-70f3f658bfa4`. **Pass; G1 keeps `--session <id>`.** The file appears only after the first
    prompt (runs 1–3 without a prompt or with inherited env had none).
  - (b) `ui.open -> {"isPlaced":true}`; render props `{"title":"mlspike","isFocused":false,"bodyColumns":71,
    "placement":"dock","scroll":{"offset":0,"bodyRows":43},"view":{}}`; renders logged at `tick=0,5,10,15,20` one per
    5 s (18:55:32.570, :37.595, :42.609, :47.621, :52.634Z), plus one per child line. The PTY shows the pane at the
    right edge with `MLSPIKE tick=…` updating. **Pass (terminal).**
  - (c) `command.register -> {"command":"mlspike"}`; `/mlspike` printed `mlspike:mlspikecommandran;paneplaced=true`
    (spaces lost to cursor moves) and logged `command.run mlspike`. **Pass.**
  - (d) chunks `{"stream":"stdout","text":"line 1\n"}` … `line 5`; `ps` after start: `16957 16902 /bin/sh -c … mlspike-child-v1`
    (ppid 16902 = `claude`); after the reload: only `17199 16902 … mlspike-child-v2`; v2 logged `spawn loop ended`
    (18:57:41.492Z) before `session.end` (18:57:41.747Z); 5 s after exit: none. **Pass.**
  - (e) `claude plugin test <mod>` with a test mounting the pane per surface → `(pass) pane mounts on terminal`,
    `desktop`, `vscode`, `mobile`; `4 pass 0 fail`. **Pass.** (Needs `on('fs.write', …)` in the test, since the mod
    writes its log.)
  - Desktop-app half of (a) and (b): **pending for 16.4** (owner turns the mod on).
  Then wrote "Phase 16 decisions" (G1–G6, G5 results, pins) into `docs/design.md`, FR16 moved from Backlog to the pane,
  and the nine G1 keys by hand from the `overview`/`links` JSON with a scratch script written from the pins, each
  checked against the TUI repo view (`home_html --repo`, 140 × 30, `--offset 0`) and `live`/`links`. PLAN.md Review
  log: "Step 16.0 amendments" (started Claude children are rows; placement is the overview's T11 rule).
- Changed: `docs/design.md`, `fixtures/expected/pane/{busy,waiting-stale,done-2h,children-exact-likely,unseen,ambiguous,child-other-repo,idle-recent,empty,waiting-quiet}.json`,
  `PLAN.md` (Review log only), `PROGRESS.md`. Board 16 was already committed (53e2268), unchanged.
- Key commands (16.1 must print each key byte for byte; run with `TZ=UTC`):
  - `busy`: `pane --json --root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-p --session 14150000-0000-4000-8000-000000001415`
  - `waiting-stale`: `pane --json --root fixtures/sessions-phase9 --now 2026-11-09T07:30:00Z --cwd /home/dev/repo-p9c --session 92000012-0000-4000-8000-00009200000c`
  - `done-2h`: `pane --json --root fixtures/sessions-phase9 --now 2026-11-09T12:00:00Z --cwd /home/dev/repo-p9l --session 95000004-0000-4000-8000-000095000004`
  - `children-exact-likely`: `pane --json --root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-m --session 14120000-0000-4000-8000-000000001412`
  - `unseen`: `pane --json --root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-m --session aaaaaaaa-0000-4000-8000-000000000000`
  - `ambiguous`: `pane --json --root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-r --session 141a0000-0000-4000-8000-00000000141a`
  - `child-other-repo`: `pane --json --root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-q --session 14150000-0000-4000-8000-000000001415`
  - `idle-recent`: `pane --json --root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-m --session 14180000-0000-4000-8000-000000001418`
  - `empty`: `pane --json --root fixtures/sessions-phase14 --now 2026-11-10T22:20:00Z --cwd /home/dev/repo-none --session aaaaaaaa-0000-4000-8000-000000000000`
  - `waiting-quiet` (added in review): `pane --json --root fixtures/sessions-phase9 --now 2026-11-09T12:00:00Z --cwd /home/dev/repo-p9c --session 92000012-0000-4000-8000-00009200000c`
- Trace (rows → `live` / `links` for the same root and `--now`):
  - phase14 `live`: heads `14130000` waiting (repo-n), `14110000`/`14120000`/`14160000` busy (repo-m), `14150000` busy
    (repo-p), `14240000` stale (repo-p). Pane: repo-m working = `14120000`, `14110000`, `14160000` (busy, by
    `started_at` 21:52, 21:20, 21:03); repo-p working `14150000`, waiting `14240000` (`stale since 9:50pm`). ✓
  - phase14 `links`: `14120000 → 14220000 heuristic` → child `likely`; `14110000 → 14210000 exact` (22:10, latest over
    `14170000`'s 21:40) → under `14110000`, once; `14160000 → 14280000/14290000 exact` (the 21:32 tie on 14290000 goes
    to `14160000`, as T11 and `parent_links` both pick) → `exact`; `14150000 → 14250000/14260000 exact` (repo-p) →
    children; `14150000 → 14270000 exact` crosses to repo-q: `live` nests it under `14150000`, the pane keeps it a
    top-level `step 3.2` row in repo-q and leaves it out of repo-p (`child-other-repo`, `seen: false` there).
  - phase14 `links`: `141a0000` ambiguous at 21:58, candidates `142d0000`, `142e0000` → `handoffs: [{ step: null,
    outcome: "2 possible runs", candidates: [both ids] }]`; the candidates stay rows (`done 10:01pm`/`10:02pm`); the
    TUI shows ` · 2 possible runs` on those rows. `142f0000` (imported, repo-m) is absent.
  - phase9 at 07:30 `live`: `92000012` waiting, `92000009` stale (repo-p9c) → waiting section; no repo-p9c links.
    Recent: `92000014`, `92000013` unknown, `92000011` finished, `92000010` idle (`last active 5:40am`).
  - phase9 at 12:00 `live`: `92000012` waiting, `92000009` stale, neither with activity in 10:00–12:00 →
    `waiting-quiet`: both in `waiting`, `working`/`recent` empty (the repo view would show nothing).
  - phase9 at 12:00 `links`: `95000004 → 95c00004 exact` → child `exact` (`done 11:01am`); `95c00001` (from
    `95000001`) and `95c00002` (latest from `95000003`, step 7.1) have parents outside the repo → top-level `done` rows.
  - Every label and phrase equals the TUI repo view's text for the same lane (W3/W5; minus `main · ` and the
    candidate suffix); `confirmed` ↔ `exact`, `matched by time` ↔ `likely`.
- Not covered by a committed fixture (16.1 unit tests instead, no invented data): a started Claude child (S3) as its
  own row; an `unlinked` hand-off (`no codex run found`; no phase14/phase9 key repo has one in its window).
  Covered: merged home rows flatten (phase9 repo-p9c at 07:30 has a merged row of quiet sessions; `waiting-stale`).
- Verify: `cargo test --workspace` → 65 `test result: ok`, 686 passed, 0 failed, 1 ignored (nothing reads the keys
  yet); G5 (a)–(e) recorded above with logged values (desktop half pending for 16.4); every key's rows traced above.
  W10 scan `rg -l "/Users/|\bsk-|api_key" fixtures --glob '!raw/**' --glob '!README.md'` → nothing (rc 1).
- Review: one fresh adversarial Claude subagent (wrote none of it), one round. Its own script from the pins and the
  Rust code regenerated all nine keys byte for byte; `home_html` renders of repo-m/p/q/r/p9c/p9l and `live` agree
  (differences only where pinned). 9 findings: 7 accepted (quiet waiting/stale rows stay visible → new
  `waiting-quiet` key; flat sort order; `seen`, Codex step, `repo`, titled `step`, candidate order pinned), 2 rejected
  with reasons (PLAN.md Review log, "Step 16.0 amendments"). After the fixes the scratch script regenerates all ten
  keys unchanged.
- Gotchas: a `claude` child started from a Claude Code session inherits `CLAUDECODE` / `CLAUDE_CODE_*` and writes no
  transcript: run it under `env -i HOME USER LOGNAME TERM LANG PATH`. The transcript file appears only after the
  first prompt. Spawn chunks are raw text, not lines. `claude plugin test` needs `on('fs.write', …)` (or any `$`
  call the mod makes) answered in the test. `time`'s local offset worked on macOS with threads running (reviewer
  check), but the keys still run with `TZ=UTC`.


### 2026-11-11 — Step 15.4: Phase 15 real check, README, owner confirm — Claude
- Mode: Claude step (no Codex hand-off). Quota check anyway: `python3 -I tools/codex-quota.py` → `primary: 0%`,
  `secondary: 56%`, `CODEX: OK`.
- Did: Timed F3 (a)–(d) on real `~` with the release build of `5a63110`: (a) `/usr/bin/time -p` × 3; (b)/(d) with the
  15.1 scratch timing crate copied to this session's scratchpad (3 fresh processes each; warm = passes 2–11 of one
  process). Two series: series 1 at load 12–17, series 2 at load 4.7–6.2 (recorded). (c) and the smoke with a scratch
  Python PTY harness (reader thread drains the PTY all the time, tiny CUP/ED/EL screen model, restore check = alt
  screen left + cursor shown + PTY back to echo/canonical/isig). Rendered `home_html --loading 4.0 --size 76x22`,
  screenshot with headless Chrome 2x, compared by eye with a paused scratch copy of board 15 (option E only). Wrote
  the README "Loading screen" section, Release 1.12 line, a feature bullet and two limits lines; wrote
  `docs/phase15-check.md`.
- Changed: `docs/phase15-check.md` (new), `docs/img/loading-phase15-fixture.png` (new), `README.md`, `PROGRESS.md`.
- Verify (whole phase):
```
cargo test --workspace → exit 0; 65 `test result: ok`, 686 passed, 0 failed, 1 ignored (real_counts)
cargo clippy --workspace --all-targets -- -D warnings → exit 0;  cargo fmt --all -- --check → exit 0
fixtures/expected/phase9/check.sh 6 → 47 ok, 0 FAIL;  phase12/check.sh → 3 ok;  phase13 → 1 ok;  phase14 → 1 ok
git diff --stat c29c516 -- fixtures/expected → 25 files, all fixtures/expected/tui/loading-* (15.0's goldens only)
W10: grep -nE "/Users/|\bsk-|api_key|local_[0-9a-f]{8}" README.md docs/phase15-check.md → nothing
series 2 (a) overview --json: real 1.27 user 9.53 sys 1.19 | real 1.16 user 9.25 sys 1.18 | real 1.16 user 9.39 sys 1.26 → 1.16 s
series 2 (b) no live: 1446 = 1006+440 | 1598 = 1157+441 | 1525 = 1082+443                 → median 1525 ms
             --live:  1613 = 1098+439+75 | 1581 = 1058+444+79 | 1629 = 1113+440+77      → median 1613 ms
series 2 (d) warm median of 10: 130 ms (126–132) no live; 194 ms (187–216) --live; bound 1.2 × 191–212 = 229–254 ms
series 1 (load 12–17): (a) 1.54/1.18/1.13 s; (b) 1531/1641/1657 ms, --live 1716/1853/2212 ms; (d) 130 / 199 ms
(c) PTY 76x22, first home frame: tui 1.821/1.952/1.989 s → 1.952 s; tui --live 1.969/2.086/1.939 s → 1.969 s
smoke15.py → 14/14 ok, twice (counter 36 → 176 / 1069; 60x16 title row 9; 30x6 no art; q/Ctrl-C 2–6 ms, exit 0)
```
- Review: Claude's own step (docs and image only; no code), so no Codex review, as in 14.4. Numbers in README and
  the check doc were checked against the scratch logs (`f3-run1.log`, `f3-run2.log`, `f3-warm.log`,
  `home-run2.log`, `smoke-run2.log`). Board check: same fish, six `*` meets and specks as the board's paused
  `drawC(4.0)`; rule-made differences only (F6 title and bar colours, fixture counter). Owner's answer
  (2026-11-11): "Looks good". Phase 15 closed.
- Gotchas: (1) Both series are 0.1–0.2 s slower than 15.1's (load ≈ 3.7 then, 1049 → 1069 files); not isolated.
  (2) One series-1 `--live` run took 2212 ms at load 17: F3 uses medians, so it passes, but it is disclosed in the
  README limits. (3) "During the fade" is inferred from timing (`ready ✓` after 1.0 s means the fade has started);
  the harness reads text, not colours. (4) At 76 columns the live grid's footer does not contain `↑↓ groups`; detect
  the `multi-link › live` header instead. (5) A shell loop over `"phase9/check.sh 6"` as one word runs nothing
  (exit 127); call it as `bash fixtures/expected/phase9/check.sh 6`.

### 2026-11-11 — Step 15.3: Wiring: show, skip, fade, progress — Codex
- Did: Added explicit startup state and injected instants to App, leaving default App behavior and all existing
  app tests unchanged. Loading waits for both the first report and start + 1.0 s, then uses elapsed deadlines for
  125 ms School / 125 ms Text, then home. Skip is consumed once; pre-report skip uses the existing loading footer;
  errors and worker disconnect show the existing error footer immediately; NO_COLOR never fades.
  The worker creates and installs LoadProgress before its first pass, sends a clone first, and finishes it only
  after sending the first refresh. The CLI maps phase strings and counts every frame. Kept the existing worker
  and 50 ms poll loop, with no new production thread or timer and no join on quit. Extracted that loop for tests.
  The loading draw bypasses the home's NO_COLOR postpass, retaining the renderer's removal of BOLD and DIM.
- Changed: `crates/tui/src/app.rs`, `crates/cli/src/tui.rs`, `crates/tui/tests/loading_app.rs`, `PROGRESS.md`.
- Verify: Used `export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"` before cargo.
  `cargo test --workspace` → exit 0; full output in scratch `/tmp/multi-link-15.3-tests-final.log`.
  Summing its 65 actual `test result` lines produced:
  ```text
  Workspace result lines: 65
  Totals: 686 passed; 0 failed; 1 ignored
  ```
  Actual output excerpts (CLI unit tests, new injected-clock app tests, final doc-test target):
  ```text
  Running unittests src/main.rs (target/debug/deps/multi_link-7cedda362c6e1306)

  running 24 tests
  test tests::child_exit_never_passes_the_child_code_through ... ok
  test tests::labels_use_serde_names_for_multiword_variants ... ok
  test tests::open_commands_run_without_a_shell_in_cwd ... ok
  test tests::graph_text_cuts_non_ascii_session_ids_by_chars ... ok
  test tui::tests::core_snapshots_map_to_tui_phases_and_exact_counts ... ok
  test tui::tests::control_c_uses_the_same_quit_effect_as_q ... ok
  test tui::tests::partial_setup_and_restore_failure_still_leave_normal_terminal ... ok
  test tui::tests::cli_loading_draw_preserves_no_color_and_control_c_quits ... ok
  test tui::tests::disconnected_worker_draws_error_before_quitting ... ok
  test tui::tests::drawn_finished_but_fresh_running_never_resumes_and_errors_never_execute ... ok
  test tui::tests::loading_quit_does_not_wait_for_blocked_worker ... ok
  test tui::tests::terminal_restores_after_success_spawn_failure_and_unwind ... ok
  test tui::tests::graph_steps_are_tagged_plan_steps_then_unplanned_by_first_handoff ... ok
  test tui::tests::started_steps_include_children_outside_the_tui_window ... ok
  test tui::tests::graph_refresh_on_phase8_fixture ... ok
  test tui::tests::phase9_codex_lane_untagged_has_no_step ... ok
  test tui::tests::phase9_codex_lane_same_repo_beats_newer_cross_repo ... ok
  test tui::live_tests::live_flag_and_worker_report_follow_screen_request ... ok
  test tui::tests::phase9_codex_lane_parent_outside_window ... ok
  test tui::tests::stale_threshold_changes_status_without_file_change ... ok
  test tui::tests::coordinator_s_end_to_end ... ok
  test tui::tests::live_indices_follow_worker_step_and_freshness ... ok
  test tui::tests::app_open_running_fixture_uses_fresh_pass_and_never_resumes ... ok
  test tui::tests::worker_sends_progress_before_report_and_finishes_shared_handle ... ok

  test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
  Running tests/loading_app.rs (target/debug/deps/loading_app-5bd2efe9d42b5517)

  running 4 tests
  test injected_counts_are_rendered_and_no_color_never_fades_or_adds_bold ... ok
  test report_deadlines_drive_both_fade_stages_without_extra_frames ... ok
  test errors_and_quit_end_loading_immediately ... ok
  test skip_is_consumed_before_and_after_report_and_during_fade ... ok

  test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  Doc-tests multi_link_tui

  running 0 tests

  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; actual output:
  ```text
      Checking multi-link v0.1.0 (/Users/jarednarvaez/multi-link/crates/cli)
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.55s
  ```
  `cargo fmt --all -- --check` → exit 0, no output. `git diff --check` → exit 0, no output.
  Focused new app tests initially failed to compile on the missing loading API; after implementation all four
  passed. The CLI quit test initially failed on the missing extracted loop; after wiring the CLI tests passed.
- Review: Claude, 2026-11-11, mode Codex loop (`codex-quota.py` → `primary: 27%`, `secondary: 55%`, `CODEX: OK`).
  Diff read against Step 15.3 and F4/F5/F8: no findings, no fix round. Files as named (plus PROGRESS.md); the tui
  crate still has no core dependency; no new timer or thread (the 50 ms loop sets the instant each frame); skip
  consumed once, `q`/Ctrl-C quit, error and disconnect end loading with the error footer, NO_COLOR never fades and the
  home NO_COLOR postpass is skipped on the loading screen; `finish()` runs after the first report is sent. Behaviour
  change noted: a disconnected worker now shows `refresh failed: refresh worker stopped` instead of exiting at once
  (F5). Re-ran Verify: `cargo test --workspace` → 686 passed, 0 failed, 1 ignored (same totals); clippy `-D warnings`
  and `fmt --check` clean. PTY smoke on real `~` (release, 76×22, scratch harness with a tiny screen model, not in
  the repo): `listing transcripts…` at 0.004 s, `reading transcripts` rising 33 → 1029 / 1049 by 0.87 s,
  `linking sessions…` 0.93 s, `ready ✓  opening home` 1.46 s (first report), home drawn 1.75 s (fade + next frame),
  `q` exited in 0.011 s, status 0. Space at 0.32 s → `loading sessions… · q quit` at once. Ctrl-C at 0.41 s during
  reading → the process exited and closed the PTY. Not checked here (left for 15.4): resize, `--live` start, terminal
  state after Ctrl-C/fade quit.
- Gotchas: No manifest changes, core dependency, fixture updates, stages or commits. Real-data timing / PTY smoke
  remains Step 15.4. An unrelated untracked `.claude/` directory appeared during verification and was left untouched.

### 2026-11-11 — Step 15.2: TUI loading screen render — Codex
- Did: Implemented the pure loading renderer with TUI-owned `LoadPhase` / `LoadingOptions`, the pinned fish
  geometry, per-frame head map, last-write-wins cells, size tiers, rounded progress bar and both fade palettes.
  NO_COLOR preserves glyphs and drops all modifiers without changing the shared theme helper. Added the HTML
  example's `--loading SECONDS OUT.html --size WxH` variant. Tests read every loading text/style answer key,
  use the existing `support::dump_styles` format without any updater, check 200 frames and NO_COLOR parity,
  bounded sizes/origins, redraws, six-digit counts and overflow-safe bar rounding.
- Changed: `crates/tui/src/loading.rs`, `crates/tui/src/lib.rs`, `crates/tui/tests/loading.rs`,
  `crates/tui/examples/home_html.rs`, `PROGRESS.md`.
- Verify: All cargo commands used `export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"` first.
  Test-first run against an empty render: `cargo test -p multi-link-tui --test loading` → exit 101:
  ```text
  test result: FAILED. 0 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  Final `cargo test -p multi-link-tui > /tmp/multi-link-15-2-tui.log 2>&1` → exit 0; actual loading output:
  ```text
  running 5 tests
  test listing_padding_bar_rounding_and_second_fade ... ok
  test bar_clamps_counts_without_overflow ... ok
  test all_loading_goldens ... ok
  test sizes_origins_redraw_and_six_digit_counts_are_bounded ... ok
  test two_hundred_frames_use_only_f6_roles_and_no_color_preserves_glyphs ... ok

  test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
  ```
  `cargo test --workspace > /tmp/multi-link-15-2-workspace.log 2>&1` → exit 0; actual summary of result lines
  from both logs (parsed and checked for failures/errors/warnings):
  ```text
  workspace: 64 test suites; 676 passed; 0 failed; 1 ignored
  tui: 12 test suites; 242 passed; 0 failed; 0 ignored
  ```
  Existing ignored test: `real_counts`; not run. Every existing TUI snapshot passed.
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, actual output:
  ```text
      Checking multi-link-tui v0.1.0 (/Users/jarednarvaez/multi-link/crates/tui)
      Checking multi-link v0.1.0 (/Users/jarednarvaez/multi-link/crates/cli)
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.95s
  ```
  `cargo fmt --all -- --check` → exit 0, no output.
  `cargo run -q -p multi-link-tui --example home_html -- --loading 4.0 /tmp/multi-link-loading.html --size 76x22`
  → exit 0, no output; `wc -c /tmp/multi-link-loading.html` → actual output:
  ```text
    110592 /tmp/multi-link-loading.html
  ```
  `git diff --exit-code -- fixtures/expected crates/tui/Cargo.toml crates/tui/src/app.rs crates/tui/src/theme.rs`
  and `git diff --check` → exit 0, no output.
- Review: Claude, 2026-11-11, mode Codex loop (`codex-quota.py` → `primary: 20%`, `secondary: 54%`, `CODEX: OK`).
  Round 0: Codex stopped on a `crates/tui/Cargo.toml` dependency on core; Claude rejected the manifest change and
  gave tui its own input type (PLAN.md Review log). Diff review of the 4 named files plus PROGRESS.md: in scope, no
  manifest, `app.rs`, `theme.rs` or golden change; pinned rules followed (`floor(v + 0.5)`, speck wrap and drop,
  last-write-wins, per-frame head map, tail one step dimmer, bar `(60N + T) div 2T` with clamp, small-size rows,
  fade steps, NO_COLOR drops BOLD locally). No fix round. Notes, none blocking: (1) the `listing` counter text is
  not in F5 — accepted, Review log; (2) the 200-frame role check also allows `Color::Reset`, which truecolor never
  uses (harmless, looser than needed); (3) the example's loading variant fixes phase and counts (enough for the
  README image). Claude's re-run: `cargo test --workspace` → every result `ok`, 677 passed, 0 failed, 1 ignored (my
  sum over all `test result:` lines; Codex's parsed 676, a counting difference, no failure either way);
  `cargo test -p multi-link-tui --test loading` → 5 passed; clippy `-D warnings` and `fmt --check` clean;
  `git diff 8a7e892 -- fixtures/expected` → empty (goldens unchanged since 15.0). **Board 15 E comparison
  (styled):** the browser pane was hidden, so instead of screenshots Claude ran a copy of the board (scratch) with
  `paused = true` (frame `drawC(4.0)`) and `put` logging every canvas-C write with glyph, RGB and weight, kept the
  last write per cell, and compared art rows 0–14 with `loading-76x22.{txt,styles.txt}` (which the render equals
  cell for cell): **225 / 225 art cells same glyph, 0 differ**; every cell's colour family matches (board orange →
  `claude*`, blue → `codex*`, pale `*` → `text` BOLD, 6 meets); quantisation is monotone in the board's brightness
  (max channel): BOLD heads 187–232, plain-role heads 142–183 and tails 131–162, `_dim` cells 65–138. Known,
  rule-made differences: the board bolds heads at depth > 0.7, F6 at > 0.66 (one blue `<` at 187 is BOLD here, not on
  the board); the board's title and bar are continuous orange→blue gradients, here `claude` / `text` / `codex` and
  two halves (F6); the counter is real progress, not the board's simulated one.
- Gotchas: No manifest dependency needed (PLAN Review log, "Step 15.2 input type"). Step 15.3 must translate
  the core phase string/counts, select the fade stage by deadline, and wire the screen into the app. No app wiring,
  theme-helper or answer-key changes. Pre-existing PLAN.md edits and `.claude/` were left untouched. Nothing staged,
  committed or pushed.

### 2026-11-11 — Step 15.1: Parallel cold parse, F2 measurement, progress handle — Codex
- Did: Parallel speculative prefill with pre-read stamps, original serial assembly/revalidation/diagnostics,
  one-generation atomic progress and root-reset survival. Round 1 re-exported `LoadSnapshot` and added complete
  out-of-report diagnostic comparisons plus malformed-Claude coverage. Round 2 implements Claude's F1 amendment:
  app-record jobs come from `app_record_paths`; cache every successful `read_app_record` result (Record/Ignored/Bad)
  with its listing stamp; discard speculative errors, including NotFound. Existing serial app-record loop unchanged.
  App-record misses count in progress and parse stats exactly like other jobs.
- Changed: `crates/core/src/scanner.rs`, `crates/core/src/lib.rs`, `crates/core/tests/parallel_scan.rs`, `PROGRESS.md`.
  This round changed scanner, parallel_scan tests and this log only; Claude's PLAN.md amendment was left untouched.
- Verify: All cargo commands used `export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"` first.
  `cargo test --workspace > /tmp/multi-link-15-r2-tests.log 2>&1` → exit 0; actual summary of result lines:
  ```text
  test result lines: 63
  passed: 672 failed: 0 ignored: 1
  ```
  The ignored test is the existing `real_counts`. New round 2 unit regression failed before implementation
  (0 files/0 bytes vs expected 3 files/65 bytes); after implementation its actual result was:
  ```text
  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
  ```
  `cargo test -p multi-link-core --test parallel_scan` → exit 0; actual result:
  ```text
  test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.77s
  ```
  Coverage includes 1-vs-4 reports/diagnostics/stats on good/ignored/bad app records; direct prefill cache and
  progress assertions (3 misses, 65 bytes), unchanged serial bad-record reporting, and a full warm refresh with
  0 files/0 bytes parsed. All earlier race, diagnostic and progress tests remain in the suite.
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; actual final line:
  ```text
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 18.10s
  ```
  `cargo fmt --all -- --check` → exit 0, no output. Four script results (summarized from actual output):
  ```text
  bash fixtures/expected/phase9/check.sh 6 → 47 ok, exit=0
  bash fixtures/expected/phase12/check.sh → 3 ok, exit=0
  bash fixtures/expected/phase13/check.sh → 1 ok, exit=0
  bash fixtures/expected/phase14/check.sh → 1 ok, exit=0
  ```
  `git diff --exit-code -- fixtures/expected docs/design.md`, `git diff --check` → exit 0, no output.
  `git diff --cached --name-only` → no output. Full round 2 logs: `/tmp/multi-link-15-r2-{tests,clippy,checks}.log`.
- Review: Claude (Codex loop; quota check `CODEX: OK`, 5 h window 2 %, weekly 51 %), 2 rounds. Round 1 (3 findings):
  re-export `LoadSnapshot` (15.2 must name it); 1-vs-many test must compare out-of-report diagnostics (bad records,
  incomplete, unreadable, hook conflicts) on a root that has them; a malformed Claude record reported once — fixed.
  Round 2: Claude measured the serial desktop-app record loop at 544–554 ms of each cold pass (166 files, 87 MB) and
  amended F1 to prefetch them (PLAN.md Review log) — implemented. Checked in the diff: serial loops remain the only
  assembler and reporter, stamps are taken before the read, failed reads are not cached, the progress phase is only
  published on the first generation. Claude re-ran Verify: `cargo test --workspace` 64 result lines, 672 passed,
  0 failed, 1 ignored; clippy `-D warnings` clean; `fmt --check` clean; phase9 (`6`), 12, 13, 14 `check.sh` exit 0;
  `git diff --exit-code -- fixtures/expected` clean. F2: no `Arc` branch accepted (warm reader pair 13 ms in total, so
  clones are well under 50 ms; cold clones 1.3 ms). F3 re-timed by Claude after round 2 (scratch harness as in 15.0,
  release, real `~`, load average ≈ 3.7 from other sessions):
  ```
  (a) overview --json: real 1.22 user 9.48 sys 1.60 | real 0.95 user 8.62 sys 1.02 | real 0.97 user 8.97 sys 1.04 → median 0.97 s
  (b) no live: 1397 = 997+398 | 1336 = 948+386 | 1433 = 1041+391                       → median 1397 ms
      --live:  1405 = 929+393+80 | 1457 = 988+388+78 | 1408 = 932+394+80               → median 1408 ms
  (d) warm median of 10: 110 ms (no live), 169 ms (--live); 14.4 was 191–212 ms
  ```
  Before round 2 (same harness, Claude): (a) 1.76 / 1.48 / 1.47 s, (b) 1842 / 1881 ms medians — a 6 % margin, which
  is why round 2 was taken. All F3 budgets pass; no 15.1a.
- Gotchas: Corrupt rollouts intentionally remain uncached and retry on warm passes; the zero-parse warm test uses
  a separate root with cacheable good/ignored/bad app records. No F3 runs in either review round, no new dependency,
  no writes to agent data, no stage/commit/push. Pre-existing untracked `.claude/` left alone.

**F2 measurements retained (release, real `~`, before the app-record amendment).** Temporary instrumentation timed
cache-hit clones in the serial reader pair and total overview duration minus speculative prefill. Cold serial
remainder **683.485 ms** (prefill 918.986 ms), reader pair 586.923 ms, clones **1.312 ms**. Ten warm overview serial
remainders: median **47.6025 ms**; across both reader calls per warm worker refresh, clone median **1.1255 ms**,
range 0.976–1.686 ms (reader median 12.916 ms). Earlier probe: cold readers 644.254 ms, clones 1.555 ms.
**Branch: no Arc cache conversion**, below >300 ms cold / >50 ms warm clone gates. Arc only backs the progress
handle. Instrumentation was removed before verification; scratch log `/tmp/multi-link-15-f2-full.log`.

**F3 first miss remains visible (before the app-record amendment).** Against 15.0 baseline medians (a) 7.99 s,
(b) 8624 / 8514 ms without/with live, the first series gave (a) 1.516947 s, (b) **2033.849 / 2059.297 ms** — misses.
Worker runs without live: 2033.849 / 2035.126 / 1997.225 ms; with live: 2059.297 / 1963.257 / 2077.807 ms.
That series overlapped other verification/development. A final uninstrumented pre-review series after checks
finished gave (a) **1.617987 s**, (b) **1944.862 / 1944.432 ms**, (d) **113.848 / 177.7325 ms** warm medians.
These are historical measurements, not measurements of the amended implementation. F3 stays held for Claude;
no 15.2 or disk-cache work started. Raw logs: `/tmp/multi-link-15-f3.log`, `/tmp/multi-link-15-f3-final.log`.
Harness: `/tmp/multi-link-15-timing/`, release, real process probe and root, overview at 2h + all-time refresh +
started/codex step maps + optional live; 3 fresh processes for (a)/(b), 10 warm refreshes for (d). It omits graph,
actual TUI/channel/render and the small diagnostic label/Refresh wrapper. Claude re-times after this round.

### 2026-11-11 — Step 15.0: Rules, loading goldens, timing baseline — Claude
- Mode: Claude step (no Codex hand-off in 15.0, so no quota check was needed).
- Did: Wrote F1–F8 and the details the goldens pin (geometry, rounding, speck wrap, draw order, role ladders, bar
  rounding, small-size rows, fade steps) as "Phase 15 decisions" in `docs/design.md`; added the cold baseline to the
  NFR "Speed" row. Committed the approved board source. Amended PLAN.md F5 (centre y = 0.48 × art rows, the board's
  `cy = 7.2` of 15) and F6 (to-do bar cells `border`), logged in the Review log. Timed baseline (a) with
  `/usr/bin/time -p` on the release binary at `c29c516` (already built; `cargo build --release` was a no-op) and (b) with
  a scratch crate (scratchpad only, path deps on core/plan/model) that runs the worker's `refresh_with_live` body with
  no graph: overview over the 2 h window, the all-time `refresh`, `started_steps` / `codex_steps`, then `Scanner::live`
  when `--live`; one fresh process and `Scanner` per run. Generated the 25 goldens with a scratch Python port of
  `drawC` + chrome that follows the pinned rules, in the existing dump format (`crates/tui/tests/support`).
- Changed: `docs/design.md`, `docs/boards/board15-loading.html` (new, as approved), `fixtures/expected/tui/loading-*`
  (25 new files), `PLAN.md` (F5/F6 amendment + Review log), `PROGRESS.md`.
- Verify:
```
cargo test --workspace → 62 `test result: ok`, 660 passed, 0 failed, 1 ignored (nothing reads the goldens yet)
(a) target/release/multi-link overview --json, real ~, 3 fresh processes (/usr/bin/time -p):
    real 9.02 user 7.27 sys 0.80 | real 7.99 user 7.25 sys 0.67 | real 7.95 user 7.23 sys 0.68 → median 7.99 s
(b) first report, scratch harness, 3 fresh processes each (ms: total = overview + all-time + live):
    no live: 9344 = 8820+524+0 | 8624 = 8168+455+0 | 8446 = 8010+435+0          → median 8624 ms
    --live:  8510 = 7997+447+65 | 8514 = 8007+444+63 | 8550 = 8045+439+65        → median 8514 ms
goldens: 25 files = 7 sizes × (txt + styles) + 76x22-256.styles + 76x22-nocolor ×2 + ready/linking/zero/fade1 ×2
board check: rows 0–14 of loading-76x22.txt vs the board's drawC(4.0) cell grid → 0 differing cells (225 glyphs)
```
- Board check (how): the board's Pause button freezes at the current animation time, not at 3, so a scratch copy of
  the board with one change (`paused = true` at load, the board's own reduced-motion path, which calls `drawAll(3)` →
  `drawC(4.0)`) plus a `put` hook that records each glyph per cell of canvas C was served from the scratchpad and opened
  in the built-in browser; the recorded grid was compared with the golden in Python, and the canvas was screenshotted
  (scratch `board-paused-drawC4.jpg`, `-zoom.png`) and looked over by eye: same fish, `*` and speck positions. Expected
  differences, all from the pinned rules: the board's counter (`ready ✓`, simulated) is replaced by the fixture counter
  and a 21-cell bar; two specks the board draws on the title row (row 15, columns 60 and 74) are dropped; 35 cells
  where the canvas overprints two glyphs keep the later one.
- Review: one fresh adversarial Claude subagent (wrote none of it), one round. Its own Node port of `drawC` + the
  pinned rules regenerated all 25 goldens byte for byte, and the board's real script run in Node with a stub canvas
  gave 0 differing art cells at 76×22; it confirmed 660/0/1 tests and the medians. 10 findings, none blocking; fixed:
  `*` rule reworded as a per-frame head map (the "cell holds" reading changes two 60×16 cells); F7 tier order stated
  (29×6 was ambiguous; PLAN + design); `-zero` changed to phase reading `0 / 0` (as linking it equalled `-linking`;
  PLAN goldens list amended); "7.3 s CPU" → "7.25 s user"; "cell for cell" → "glyph for glyph (weights and
  colours per F6)"; thresholds, centring and speck wrap stated; NO_COLOR-drops-BOLD gotcha for 15.2; board charset
  note in design. Not changed: `.claude/` stays untracked (only the step's files are staged).
- Gotchas: (1) `cargo build --release -p multi-link-cli` fails; the CLI package is `multi-link`. (2) The board without
  a `<meta charset>` decodes as Windows-1252 when served by `python -m http.server` (glyphs become `Â·`); the scratch
  copy adds one. (3) Board rounding is `Math.round` (half up); Rust's `f64::round` differs on negative halves, so 15.2
  must use `floor(v + 0.5)`. (4) `loading-76x22-zero` is phase reading `0 / 0` (a linking `-zero` would equal `-linking`).
  (5) The baseline (a) median is 7.99 s, below the 8.79 s in the plan text (single run, same day); both miss 2 s by 4×,
  so the plan stands.

### 2026-11-10 — Step 14.4: Phase 14 real check, README, owner confirm — Claude
- Mode: Claude step (no Codex hand-off in 14.4, so no quota check was needed).
- Did: Timed the refresh with the grid open on real `~` with a scratch program that repeats the worker's
  `refresh_with_live` (no graph) on one `Scanner`: cold, 10 warm passes 2 s apart, one pass after a card's transcript
  grew; two runs, the second split by part. Counted cards from those passes and `multi-link live --json`. PTY smoke of
  the release TUI at 120×40 (`l`, arrows, `esc`, `l`, three resizes, `q`; `--live`, `↓`, `esc`, `q`) plus a
  forced-full-frame run for the `selected` line. Rendered the fixture report with `home_html --live-grid` and
  headless Chrome (2x) into `docs/img/live-phase14-fixture.png`. README: Release 1.11 line and bullet, "Live grid"
  section (image, cards, rows, feed, keys, sizes, CLI), `l` in Run, `multi-link live --json`, live-grid limits.
  Wrote `docs/phase14-check.md` (counts and timings only).
- Changed: `docs/phase14-check.md` (new), `docs/img/live-phase14-fixture.png` (new), `README.md`, `PROGRESS.md`.
- Verify (whole phase):
```
cargo test --workspace → exit 0; 62 `test result: ok`, 660 passed, 0 failed, 1 ignored (real_counts)
  incl. live_matches_phase14_key ... ok, six_live_goldens_text_and_styles ... ok (120×40, 100×40, 120×30, 60×20 + 2)
cargo clippy --workspace --all-targets -- -D warnings → exit 0;  cargo fmt --all -- --check → exit 0
fixtures/expected/phase9/check.sh 6 → 47 ok, 0 FAIL;  phase12/check.sh → ok links, overview, graph-1.1
phase13/check.sh → ok links.json;  phase14/check.sh → ok live.json
git diff --stat f1fbec6 -- fixtures/expected → phase14 keys, live-phase14* goldens, and the 14.3 home/repo footer
  goldens only (16 files, 2 lines each)
W10: rg "/Users/|\bsk-|api_key|<real ids>" README.md docs/phase14-check.md → nothing; image is the fixture render
timing run 1 → cold 9354 ms; warm median 191 ms (174–218); after growth (+493 B) 197 ms
timing run 2 → cold 9156 ms (overview 8607 · all-time 447 · live 102); warm median 212 ms (187–248);
  after growth (+6638 B) 205 ms
cards → 2 (busy 2, waiting 0, stale 0, Codex 0, no child rows), 1 during run 1 passes 6–11; idle 14–15; feed 4
smoke14.py → 10/10 ok, `q` exit 0 both runs
```
- Review: Claude's own step (docs and image only; no code). **Deviation from the step's Verify:** the cold pass is
  9.2–9.4 s, not ≤ 2 s. The live build is 102 ms of it; the rest is the overview's first parse, documented since
  Step 8.5 (6.4 s then) and outside Phase 14's scope. Warm and changed-file passes pass. Owner's answer (2026-11-11,
  after the cold-pass deviation was disclosed): "look good".
- Gotchas: real data had no Codex card, nothing waiting or stale, and at most two lone cards, so pairing, `──▸`
  rows and moving across a row were not exercised on real data (fixture and 14.3 app tests only). The PTY harness must
  wait out `loading sessions…` (about 9 s) before sending keys, and strip escape codes before matching text.

### 2026-11-10 — Step 14.3: Wiring: `l`, `--live`, keys, refresh, call cache — Codex
- Did: Implemented the amended step. App live state preserves the opening home/repo view, selection by full
  session id (same flattened index, clamped, when removed), group/child arrows at wide widths and flattened
  arrows at narrow widths. Every draw recomputes the visible row after resize. `enter` opens the card's repo
  and selects its lane; `s` uses its step or G9 started-child picker; `o`/`f` request the existing fresh lookup.
  Added `tui --live` in TuiArgs (main.rs already delegates clap flags to that type). The worker requests live
  reports only while the grid is open; stepping into a graph suspends that request and esc restores it.
  Added the per-session call/turn cache inside the worker-owned Scanner, keyed by sorted main/subagent paths,
  lengths, mtimes and inodes. Reports are rebuilt, not cached. A test-only read counter proves per-session
  invalidation and zero reads for clock-only changes. Added six app/worker/cache tests; existing started-test
  Refresh constructor gains `live: None`. Home/repo drawing changes only the footer word.
- Changed: `crates/cli/src/tui.rs`, `crates/core/src/live.rs`, `crates/core/src/scanner.rs`,
  `crates/tui/src/app.rs`, `crates/tui/src/home.rs`, `crates/tui/src/timeline.rs`,
  `crates/tui/tests/started.rs`, `PROGRESS.md`. Every changed home/repo golden:
  - `fixtures/expected/tui/home-full.styles.txt`
  - `fixtures/expected/tui/home-full.txt`
  - `fixtures/expected/tui/home-hook-full.styles.txt`
  - `fixtures/expected/tui/home-hook-full.txt`
  - `fixtures/expected/tui/home-hook.styles.txt`
  - `fixtures/expected/tui/home-hook.txt`
  - `fixtures/expected/tui/home-phase4.styles.txt`
  - `fixtures/expected/tui/home-phase4.txt`
  - `fixtures/expected/tui/home-phase6.styles.txt`
  - `fixtures/expected/tui/home-phase6.txt`
  - `fixtures/expected/tui/home.styles.txt`
  - `fixtures/expected/tui/home.txt`
  - `fixtures/expected/tui/repo-now.styles.txt`
  - `fixtures/expected/tui/repo-now.txt`
  - `fixtures/expected/tui/repo-phase6.styles.txt`
  - `fixtures/expected/tui/repo-phase6.txt`
- Verify: Rust PATH set per AGENTS.md. Final `cargo test --workspace` → exit 0; 660 passed,
  0 failed, 1 ignored (`real_counts`, the existing opt-in real-data test). Actual result lines, extracted
  verbatim from the full run with `rg '^test result:' /tmp/multi-link-14-final-tests.log`:
  ```text
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.77s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.54s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.72s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.66s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 54 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.64s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  Actual feature/snapshot test lines from that same run:
  ```text
test tui::live_tests::live_flag_and_worker_report_follow_screen_request ... ok
test live::cache_tests::per_session_call_cache_tracks_all_dependencies_and_clock_only_refresh ... ok
test app::live_tests::live_card_open_fork_step_and_enter ... ok
test app::live_tests::live_entry_return_and_start ... ok
test app::live_tests::live_two_children_and_resize_scroll_to_exact_rows ... ok
test app::live_tests::live_moves_retains_ids_and_clamps_removed_cards ... ok
test six_live_goldens_text_and_styles ... ok
  ```
  Cache test covers main growth/removal/recreation/replacement and mtime-only changes; subagent addition,
  growth/removal/replacement and mtime-only changes; Codex rollout growth/replacement; root change; and
  a clock-only refresh that changes state counts but reads zero call/turn details. Replacement tests preserve
  length and mtime so only inode changes. All other sessions' read counters stay unchanged.
- Verify: `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; actual completion line:
  ```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.44s
  ```
  `cargo fmt --all -- --check` → exit 0, no output. `git diff --check` → exit 0, no output.
- Verify: `fixtures/expected/phase9/check.sh 6` → exit 0; actual output:
  ```text
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   graph-1.1.json (^9[1-6])
ok   graph-2.1.json (^9[1-6])
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   stderr unreadable: 1
ok   open open 92000001
ok   open open 92000002
ok   open open 92000003
ok   open open 92000004
ok   open open 92000005
ok   open open 92000006
ok   open open 92000007
ok   open open 92000008
ok   open open 92000009
ok   open open 92000010
ok   open open 92000011
ok   open open 92000012
ok   open open 92000013
ok   open open 92000014
ok   open open 94000001
ok   open open 94000002
ok   open open 94000003
ok   open open 94000004
ok   open open 94000006
ok   open open 94000007
ok   open fork 92000001
ok   open fork 92000002
ok   open fork 92000003
ok   open fork 92000004
ok   open fork 92000005
ok   open fork 92000006
ok   open fork 92000007
ok   open fork 92000008
ok   open fork 92000009
ok   open fork 92000010
ok   open fork 92000011
ok   open fork 92000012
ok   open fork 92000013
ok   open fork 92000014
ok   open fork 94000001
ok   open fork 94000002
ok   open fork 94000003
ok   open fork 94000004
ok   open fork 94000006
ok   open fork 94000007
note lanes.json is checked by the 9.5 worker-to-App test, not here
  ```
- Verify: `fixtures/expected/phase12/check.sh` → exit 0; actual output:
  ```text
ok   links.json
ok   overview.json
ok   graph-1.1.json
  ```
- Verify: `fixtures/expected/phase13/check.sh` → exit 0; actual output:
  ```text
ok   links.json
  ```
- Verify: `fixtures/expected/phase14/check.sh` → exit 0; actual output:
  ```text
ok   live.json
  ```
- Verify: Text non-footer lines and style runs outside the footer were compared with `67cdabd`;
  actual check summary:
  ```text
PASS: 16 changed home/repo goldens; all other rows identical to 67cdabd
  ```
  Full footer-only diff, pasted from `git diff --unified=0 67cdabd -- 'fixtures/expected/tui/home*' 'fixtures/expected/tui/repo*'`:
  ```diff
diff --git a/fixtures/expected/tui/home-full.styles.txt b/fixtures/expected/tui/home-full.styles.txt
index 9756372..9b413d3 100644
--- a/fixtures/expected/tui/home-full.styles.txt
+++ b/fixtures/expected/tui/home-full.styles.txt
@@ -493 +493,5 @@
-79 52-89 - #0E1014 -
+79 52-54 - #0E1014 -
+79 55-55 #D7DAE0 #0E1014 -
+79 56-56 - #0E1014 -
+79 57-60 #9AA1AE #0E1014 -
+79 61-89 - #0E1014 -
diff --git a/fixtures/expected/tui/home-full.txt b/fixtures/expected/tui/home-full.txt
index d0135b1..c7330a9 100644
--- a/fixtures/expected/tui/home-full.txt
+++ b/fixtures/expected/tui/home-full.txt
@@ -80 +80 @@
- ↑↓ move   enter repo view   o open session   q quit                                      times UTC
+ ↑↓ move   enter repo view   o open session   l live   q quit                             times UTC
diff --git a/fixtures/expected/tui/home-hook-full.styles.txt b/fixtures/expected/tui/home-hook-full.styles.txt
index 8b0e5a2..fcecfbd 100644
--- a/fixtures/expected/tui/home-hook-full.styles.txt
+++ b/fixtures/expected/tui/home-hook-full.styles.txt
@@ -573 +573,5 @@
-79 52-89 - #0E1014 -
+79 52-54 - #0E1014 -
+79 55-55 #D7DAE0 #0E1014 -
+79 56-56 - #0E1014 -
+79 57-60 #9AA1AE #0E1014 -
+79 61-89 - #0E1014 -
diff --git a/fixtures/expected/tui/home-hook-full.txt b/fixtures/expected/tui/home-hook-full.txt
index 33a3f7b..e394525 100644
--- a/fixtures/expected/tui/home-hook-full.txt
+++ b/fixtures/expected/tui/home-hook-full.txt
@@ -80 +80 @@
- ↑↓ move   enter repo view   o open session   q quit                                      times UTC
+ ↑↓ move   enter repo view   o open session   l live   q quit                             times UTC
diff --git a/fixtures/expected/tui/home-hook.styles.txt b/fixtures/expected/tui/home-hook.styles.txt
index 4b53469..0e50001 100644
--- a/fixtures/expected/tui/home-hook.styles.txt
+++ b/fixtures/expected/tui/home-hook.styles.txt
@@ -445 +445,5 @@
-29 52-89 - #0E1014 -
+29 52-54 - #0E1014 -
+29 55-55 #D7DAE0 #0E1014 -
+29 56-56 - #0E1014 -
+29 57-60 #9AA1AE #0E1014 -
+29 61-89 - #0E1014 -
diff --git a/fixtures/expected/tui/home-hook.txt b/fixtures/expected/tui/home-hook.txt
index bd61a98..6257665 100644
--- a/fixtures/expected/tui/home-hook.txt
+++ b/fixtures/expected/tui/home-hook.txt
@@ -30 +30 @@
- ↑↓ move   enter repo view   o open session   q quit                                      times UTC
+ ↑↓ move   enter repo view   o open session   l live   q quit                             times UTC
diff --git a/fixtures/expected/tui/home-phase4.styles.txt b/fixtures/expected/tui/home-phase4.styles.txt
index e2e8b38..5686f9f 100644
--- a/fixtures/expected/tui/home-phase4.styles.txt
+++ b/fixtures/expected/tui/home-phase4.styles.txt
@@ -361 +361,5 @@
-29 52-149 - #0E1014 -
+29 52-54 - #0E1014 -
+29 55-55 #D7DAE0 #0E1014 -
+29 56-56 - #0E1014 -
+29 57-60 #9AA1AE #0E1014 -
+29 61-149 - #0E1014 -
diff --git a/fixtures/expected/tui/home-phase4.txt b/fixtures/expected/tui/home-phase4.txt
index 6425bec..5051f16 100644
--- a/fixtures/expected/tui/home-phase4.txt
+++ b/fixtures/expected/tui/home-phase4.txt
@@ -30 +30 @@
- ↑↓ move   enter repo view   o open session   q quit                                                                                                  times UTC
+ ↑↓ move   enter repo view   o open session   l live   q quit                                                                                         times UTC
diff --git a/fixtures/expected/tui/home-phase6.styles.txt b/fixtures/expected/tui/home-phase6.styles.txt
index b06f17a..b2f8af6 100644
--- a/fixtures/expected/tui/home-phase6.styles.txt
+++ b/fixtures/expected/tui/home-phase6.styles.txt
@@ -455 +455,5 @@
-39 52-74 - #0E1014 -
+39 52-54 - #0E1014 -
+39 55-55 #D7DAE0 #0E1014 -
+39 56-56 - #0E1014 -
+39 57-60 #9AA1AE #0E1014 -
+39 61-74 - #0E1014 -
diff --git a/fixtures/expected/tui/home-phase6.txt b/fixtures/expected/tui/home-phase6.txt
index 2d39fa4..05a597c 100644
--- a/fixtures/expected/tui/home-phase6.txt
+++ b/fixtures/expected/tui/home-phase6.txt
@@ -40 +40 @@
- ↑↓ move   enter repo view   o open session   q quit                       2 Codex Desktop imports hidden · times UTC-8
+ ↑↓ move   enter repo view   o open session   l live   q quit              2 Codex Desktop imports hidden · times UTC-8
diff --git a/fixtures/expected/tui/home.styles.txt b/fixtures/expected/tui/home.styles.txt
index 9791fc1..a7848ec 100644
--- a/fixtures/expected/tui/home.styles.txt
+++ b/fixtures/expected/tui/home.styles.txt
@@ -395 +395,5 @@
-29 52-89 - #0E1014 -
+29 52-54 - #0E1014 -
+29 55-55 #D7DAE0 #0E1014 -
+29 56-56 - #0E1014 -
+29 57-60 #9AA1AE #0E1014 -
+29 61-89 - #0E1014 -
diff --git a/fixtures/expected/tui/home.txt b/fixtures/expected/tui/home.txt
index 69fcc2a..8cb7e35 100644
--- a/fixtures/expected/tui/home.txt
+++ b/fixtures/expected/tui/home.txt
@@ -30 +30 @@
- ↑↓ move   enter repo view   o open session   q quit                                      times UTC
+ ↑↓ move   enter repo view   o open session   l live   q quit                             times UTC
diff --git a/fixtures/expected/tui/repo-now.styles.txt b/fixtures/expected/tui/repo-now.styles.txt
index fe69165..3347d13 100644
--- a/fixtures/expected/tui/repo-now.styles.txt
+++ b/fixtures/expected/tui/repo-now.styles.txt
@@ -493 +493,5 @@
-39 96-107 - #0E1014 -
+39 96-98 - #0E1014 -
+39 99-99 #D7DAE0 #0E1014 -
+39 100-100 - #0E1014 -
+39 101-104 #9AA1AE #0E1014 -
+39 105-107 - #0E1014 -
diff --git a/fixtures/expected/tui/repo-now.txt b/fixtures/expected/tui/repo-now.txt
index e1e2554..835f7da 100644
--- a/fixtures/expected/tui/repo-now.txt
+++ b/fixtures/expected/tui/repo-now.txt
@@ -40 +40 @@
- ↑↓ select   o open   f fork   s step   +/- zoom   z unfold idle   i imports   esc home   q quit            times UTC-8
+ ↑↓ select   o open   f fork   s step   +/- zoom   z unfold idle   i imports   esc home   l live   q quit   times UTC-8
diff --git a/fixtures/expected/tui/repo-phase6.styles.txt b/fixtures/expected/tui/repo-phase6.styles.txt
index 09d187f..67de59f 100644
--- a/fixtures/expected/tui/repo-phase6.styles.txt
+++ b/fixtures/expected/tui/repo-phase6.styles.txt
@@ -500 +500,5 @@
-39 96-107 - #0E1014 -
+39 96-98 - #0E1014 -
+39 99-99 #D7DAE0 #0E1014 -
+39 100-100 - #0E1014 -
+39 101-104 #9AA1AE #0E1014 -
+39 105-107 - #0E1014 -
diff --git a/fixtures/expected/tui/repo-phase6.txt b/fixtures/expected/tui/repo-phase6.txt
index 98e27af..9f76a8d 100644
--- a/fixtures/expected/tui/repo-phase6.txt
+++ b/fixtures/expected/tui/repo-phase6.txt
@@ -40 +40 @@
- ↑↓ select   o open   f fork   s step   +/- zoom   z unfold idle   i imports   esc home   q quit            times UTC-8
+ ↑↓ select   o open   f fork   s step   +/- zoom   z unfold idle   i imports   esc home   l live   q quit   times UTC-8
  ```
- Review: Claude, diff + Step 14.3 (amended Files), no fix round needed. Round 1 was a scope stop (L9 cache needs
  core files); Claude amended the Files list (Review log) and Codex redid the step. Checked: App row model matches the
  renderer's fit and `↓ N more` rows; cache evicts on missing files and on sessions leaving; goldens footer-only.
  Minor, not fixed: stateless bottom-anchored scroll; long Verify paste. Re-ran Verify: `cargo test --workspace`
  660 passed, 0 failed, 1 ignored (matches); clippy `-D warnings` clean; fmt clean; phase9 `check.sh 6` 47 ok,
  phase12/13/14 all `ok`. `cargo build --release -p multi-link` ok; PTY smoke 120×40 on `sessions-phase14`
  (`tui --live`, ↓, →, esc → home footer `l live`, `l` → grid, esc, `q` → exit 0) passed (scratch only).
- Gotchas: Initial red tests failed on missing live/cache APIs. An intermediate cache test exposed needless
  eviction of an unchanged child's details when its parent disappeared; retaining details for still-listed
  sessions fixes that, with dependencies checked on reuse. Intermediate snapshot failures were resolved by
  the authorized footer updates; the scroll test's expected fit was corrected to the renderer's 10-row card /
  11-row stride (two rows fit at height 40). Final Verify above is green. Live goldens and JSON keys are
  unchanged. No reader API or dependency was added. Real timing and PTY/owner checks belong to 14.4.

### 2026-11-10 — Step 14.2: TUI render to board 14 — Codex
- Did: Implemented the pure live grid renderer: header, packed feed, wide paired rows and extra children, narrow
  indented cards, clipped selection, remaining-card count, idle line, selected/asked details, legend and footer.
  Reuses existing theme roles, applies the fixed offset, cuts styled text by grapheme/display width and preserves
  glyphs in NO_COLOR. Added the fixture-backed HTML live variant (`--live-grid` avoids colliding with graph `--live`).
  All six goldens are compared as text and styles without an update path. Added exact tests for unselected waiting,
  Claude link outcomes, untagged notable hand-offs, the fixture's wide prompt and combining marks, worktree place,
  empty reports/feed, clipped children, small/offset areas, NO_COLOR bounds and replacement of old cell modifiers.
- Changed: `crates/tui/src/live.rs`, `crates/tui/src/lib.rs`, `crates/tui/tests/live.rs`,
  `crates/tui/examples/home_html.rs`, `PROGRESS.md`.
- Verify: Mode `Codex loop`; quota check `CODEX: OK 48%/47%` (run by Claude before the hand-off).
  Rust PATH set per AGENTS.md. `cargo test --workspace` → exit 0; actual target/result lines pasted below
  (individual passing test names omitted except the new renderer suite). Computed from this run: 62 targets,
  654 passed, 0 failed, 1 ignored (`real_counts`, existing real-data test).
  ```text
     Running unittests src/main.rs (target/debug/deps/multi_link-7cedda362c6e1306)
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
     Running tests/cli.rs (target/debug/deps/cli-e0787a43f85482d2)
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.73s
     Running tests/graph.rs (target/debug/deps/graph-d837f77b20803abc)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.65s
     Running tests/hook_check.rs (target/debug/deps/hook_check-2c16d8f4ae4adc11)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.78s
     Running tests/live.rs (target/debug/deps/live-c6d920ae5b40705e)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/phase12.rs (target/debug/deps/phase12-a59e2b11bb6caddf)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/phase13.rs (target/debug/deps/phase13-7e99d17abc451c6e)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/phase9.rs (target/debug/deps/phase9-10b001eb96b77347)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running unittests src/lib.rs (target/debug/deps/multi_link_claude-3007d227ffc8769a)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/app_record.rs (target/debug/deps/app_record-c6763d08001f1e6c)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/calls.rs (target/debug/deps/calls-45eb22dd6795bdda)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/fixtures.rs (target/debug/deps/fixtures-499f3a2e254ccd24)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
     Running tests/invocation.rs (target/debug/deps/invocation-3e16b8bc37be96e1)
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
     Running tests/last_turn.rs (target/debug/deps/last_turn-b71057714a3b5979)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/phase6.rs (target/debug/deps/phase6-e2db30bd15f79be3)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/phase7.rs (target/debug/deps/phase7-2c8b6d1063153907)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running unittests src/lib.rs (target/debug/deps/multi_link_codex-e78d5e99d41da51c)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/calls.rs (target/debug/deps/calls-586748b917de5a55)
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/compressed.rs (target/debug/deps/compressed-a0fe57bc51027b49)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.71s
     Running tests/fixtures.rs (target/debug/deps/fixtures-ba5f82572d1c4a8d)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/phase6.rs (target/debug/deps/phase6-d1fad84a09bb2e55)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running unittests src/lib.rs (target/debug/deps/multi_link_core-b9f5577635e977c9)
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.16s
     Running tests/compressed.rs (target/debug/deps/compressed-23fd67b2786194f6)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/graph.rs (target/debug/deps/graph-367a1c66ea7fcc9e)
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/graph_commit.rs (target/debug/deps/graph_commit-413fc54f4401db0c)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
     Running tests/hook.rs (target/debug/deps/hook-9d2c70dbcc7efb25)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.90s
     Running tests/links.rs (target/debug/deps/links-f8b4f67c998fff9c)
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
     Running tests/live.rs (target/debug/deps/live-e242d43fd303903b)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/overview.rs (target/debug/deps/overview-958464a7d8ad16d7)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s
     Running tests/phase6.rs (target/debug/deps/phase6-a939478b5426845c)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/repo_probe.rs (target/debug/deps/repo_probe-489df8ea92fb062c)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
     Running tests/session_end.rs (target/debug/deps/session_end-97da6fcc150d54f9)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/sessions.rs (target/debug/deps/sessions-59bcd1019b17c9c4)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
     Running tests/started.rs (target/debug/deps/started-d7b62c3a1b08ef27)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
     Running tests/transcript_exit.rs (target/debug/deps/transcript_exit-27b68d6dc1e2bf21)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
     Running unittests src/lib.rs (target/debug/deps/multi_link_hook-62ceaf9ce3262d6f)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/hook.rs (target/debug/deps/hook-312d13ac9a0e316c)
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
     Running tests/settings.rs (target/debug/deps/settings-22861df00bc1ebd7)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/multi_link_model-df157fa80d5a3417)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/calls.rs (target/debug/deps/calls-5289ef8606320dd4)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/graph.rs (target/debug/deps/graph-bb878fdee95626eb)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/live.rs (target/debug/deps/live-9d6ecd5fdabf883c)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/model.rs (target/debug/deps/model-452af6df96c138b7)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running unittests src/lib.rs (target/debug/deps/multi_link_plan-b5ab79d96e45b0d3)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/fixtures.rs (target/debug/deps/fixtures-1edf5b4804af5d92)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/multi_link_tui-21d83cafd7849385)
test result: ok. 56 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
     Running tests/graph.rs (target/debug/deps/graph-005a83afaca3727a)
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
     Running tests/home.rs (target/debug/deps/home-5a2605d8fc0f75cd)
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running tests/live.rs (target/debug/deps/live-5cd55c828b339aac)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running tests/phase6_shape.rs (target/debug/deps/phase6_shape-e35530f2f0f05366)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/phase8_shape.rs (target/debug/deps/phase8_shape-abfb141048af4856)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/started.rs (target/debug/deps/started-da6f7bfb46539738)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/theme.rs (target/debug/deps/theme-501b876d9ddbbc6b)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/timeline.rs (target/debug/deps/timeline-e7b6c91cdfec15b4)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/view.rs (target/debug/deps/view-065c7d3f4df7468d)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_claude
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_codex
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_hook
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_model
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_plan
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_tui
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  New renderer suite from that same workspace run:
  ```text
     Running tests/live.rs (target/debug/deps/live-5cd55c828b339aac)

running 12 tests
test clipped_selected_child_keeps_its_indent_and_connector ... ok
test wide_and_combining_prompt_is_cut_at_display_width_with_ellipsis ... ok
test fixture_wide_prompt_cuts_without_splitting_a_character ... ok
test no_color_respects_the_render_area ... ok
test waiting_card_without_selection_uses_waiting_background_and_light_border ... ok
test worktree_place_and_promptless_card ... ok
test empty_report_and_feed_have_explicit_messages ... ok
test redraw_replaces_previous_cells_and_modifiers ... ok
test claude_handoff_link_variants_and_untagged_notable ... ok
test render_is_bounded_for_small_and_offset_areas ... ok
test six_live_goldens_text_and_styles ... ok
test no_color_keeps_all_glyphs_and_words_in_each_layout ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, actual completion:
  ```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 13.79s
  ```
  `cargo fmt --all -- --check`, `git diff --check`, and
  `git diff --exit-code -- PLAN.md docs/design.md docs/output-schema.md fixtures/expected` → no output, exit 0.
  `cargo run -q -p multi-link-tui --example home_html -- --live-grid fixtures/expected/phase14/live.json
  /tmp/multi-link-live.html --offset -8 --size 120x40` → no output, exit 0. Parsed the HTML rows and compared
  them with `live-phase14.txt`; actual output:
  ```text
HTML live variant: 40 rows match live-phase14.txt
  ```
- Review: Claude, diff + Step 14.2, no fix round needed. Re-ran Verify: `cargo test --workspace` 654 passed,
  0 failed, 1 ignored (matches the paste); clippy `-D warnings` and `fmt --check` clean; `git diff 67cdabd --
  fixtures/expected/tui` outside `live-phase14*` is empty (every existing snapshot unchanged); all six goldens
  compared as text and styles in `six_live_goldens_text_and_styles`. Files match the step. Non-blocking notes:
  (1) `live.rs` repeats graph.rs's `✓` label rule (cargo/pytest/npm test) instead of sharing it; (2) the renderer
  does not scroll to the selection itself: the caller passes `first_row` (14.3 must keep the selection visible);
  (3) a card with no `last_asked` prints `no prompt` (grey), a fallback the design does not name. Board check: the
  fixture render (`home_html --live-grid`, 120×40, UTC-8) against board 14a (`project/B14Live.dc.html`): same
  backgrounds (selected #262C3A, busy Claude #17130F, Codex #121A26), heavy border on the selection, harness border
  colours, `──▸` joint at the card's 5th row, feed columns at x 75/93. Differences are all decided in PLAN.md, not
  bugs: header counts order `◆ waiting · ● working · ○ stale` (L5; board: `● working · waiting · idle`); Codex line 1
  is `last_asked`, not a G3 part kind like `fix round 1` (L3); no chain words (`· continue 9.6a`, critique 5); the
  feed has no `codex back T` (critique 4); `↓ N more` when cards overflow (L7). The HTML export draws box lines with
  small gaps (export font), not a TUI difference.
- Gotchas: Initial empty renderer failed the golden comparison, then passed after implementation. Boundary tests
  caught NO_COLOR modifying outside the requested area, clipped children losing their indent/connector, and redraws
  retaining modifiers; each was observed failing before its fix. Two wide-text assertions initially budgeted one
  extra character and were corrected to the actual display-cell budget. An initial clippy `int_plus_one` finding
  was fixed; final checks above are clean. Goldens pin inner `┄` rules to the theme's `border` role and selected
  title glyph/bottom-label text without BOLD; the renderer follows those cell styles. Theme unchanged. Existing
  untracked `.claude/` left untouched. Selection retention, moving and refresh/call caching remain 14.3.


### 2026-11-10 — Step 14.1: `core::live` and `multi-link live` — Codex
- Fix round 1 (2026-11-10, Codex): windowed extra Codex turn reads (all links for `--since all`),
  precomputed head activity lookup without `unwrap`, added boundary/all-time test (red: `0 passed; 1 failed`;
  green: `1 passed; 0 failed`). `cargo test --workspace` → exit 0, 642 passed / 0 failed / 1 ignored
  (actual result-line totals in `/tmp/multi-link-14.1-fix1-tests.log`);
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.74s``;
  `cargo fmt --all --check` → exit 0, no output; `fixtures/expected/phase14/check.sh` → exit 0, `ok   live.json`.
  Phase 9 `check.sh 6` → 47 `ok` lines; Phase 12 → `ok   links.json`, `ok   overview.json`, `ok   graph-1.1.json`;
  Phase 13 → `ok   links.json`; all exit 0; all 13 Phase 8 graphs equal committed keys.
  Changed this round: `crates/core/src/live.rs`, `PROGRESS.md`; no fixture changes, staging or commits.
- Did: implemented L0–L5 LiveReport/Card/CardLink/FeedRow types, a pure live builder and Scanner G5 reader, and
  `live` JSON/text CLI output (overview flags, default 2h). Cards, idle counts, groups, latest incoming links,
  latest tagged steps, feed, outcomes and notable timestamps follow the schema and pins. Extracted shared G6
  summary with latest label time/result; graph strips the timestamp so its JSON is unchanged. Moved both step maps
  and their two unit tests to core; the CLI worker integration tests still exercise the imported functions.
  Mode: Codex loop. Quota check (Claude, before the hand-off): `CODEX: OK` (33% / 44%).
- Changed: `crates/model/src/lib.rs`, `crates/model/tests/live.rs`, `crates/core/src/live.rs`,
  `crates/core/src/lib.rs`, `crates/core/src/graph.rs`, `crates/core/tests/live.rs`, `crates/cli/src/main.rs`,
  `crates/cli/src/tui.rs`, `crates/cli/tests/live.rs`, `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md for every cargo command. Initial red test,
  `cargo test -p multi-link --test live` → actual decisive output (exit 101):
  ```text
  error: unrecognized subcommand 'live'
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
  ```
  Final `cargo test --workspace` → exit 0; 641 passed, 0 failed, 1 ignored (summed from these actual result lines,
  pasted in command order; full output retained in scratch `/tmp/multi-link-14.1-tests.log`):
  ```text
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.39s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.68s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.75s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.16s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.90s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 56 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  The core `tests/live.rs` target ran 12 tests: all states/import exclusion/old stale outside the window;
  parent precedence and same-time report-position ties; three children, cross-repo parents, equal-time children
  ordered by full id, lone heads with no `started_by` fallback; waiting/busy/stale head time/id ordering including
  busy-without-burst and stale-without-state_since fallbacks; latest notable timestamp/result and hand-off labels;
  every CardLink variant including non-card-parent `from`; resumed running-before-done, earliest completed turn,
  and exact/no-rollout null outcome; inclusive feed boundaries, tie order, empty report and missing-parent repo;
  both moved step-map tests. The CLI key test and two model shape/round-trip tests also ran in the suite.
- Verify: `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; actual final output:
  ```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.87s
  ```
  `cargo fmt --all -- --check` → exit 0, no output.
- Verify: `bash fixtures/expected/phase14/check.sh` → exit 0; actual output:
  ```text
ok   live.json
  ```
- Verify: `bash fixtures/expected/phase9/check.sh 6` → exit 0; actual output:
  ```text
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   graph-1.1.json (^9[1-6])
ok   graph-2.1.json (^9[1-6])
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   stderr unreadable: 1
ok   open open 92000001
ok   open open 92000002
ok   open open 92000003
ok   open open 92000004
ok   open open 92000005
ok   open open 92000006
ok   open open 92000007
ok   open open 92000008
ok   open open 92000009
ok   open open 92000010
ok   open open 92000011
ok   open open 92000012
ok   open open 92000013
ok   open open 92000014
ok   open open 94000001
ok   open open 94000002
ok   open open 94000003
ok   open open 94000004
ok   open open 94000006
ok   open open 94000007
ok   open fork 92000001
ok   open fork 92000002
ok   open fork 92000003
ok   open fork 92000004
ok   open fork 92000005
ok   open fork 92000006
ok   open fork 92000007
ok   open fork 92000008
ok   open fork 92000009
ok   open fork 92000010
ok   open fork 92000011
ok   open fork 92000012
ok   open fork 92000013
ok   open fork 92000014
ok   open fork 94000001
ok   open fork 94000002
ok   open fork 94000003
ok   open fork 94000004
ok   open fork 94000006
ok   open fork 94000007
note lanes.json is checked by the 9.5 worker-to-App test, not here
  ```
- Verify: `bash fixtures/expected/phase12/check.sh` → exit 0; actual output:
  ```text
ok   links.json
ok   overview.json
ok   graph-1.1.json
  ```
- Verify: `bash fixtures/expected/phase13/check.sh` → exit 0; actual output:
  ```text
ok   links.json
  ```
- Verify: for each of the 13 Phase 8 keys, ran `target/debug/multi-link graph --root fixtures/sessions-phase8
  --now 2026-11-06T18:00:00Z --repo <key.repo> --step <key.step.id> --json` and compared parsed JSON with
  **`git show HEAD:fixtures/expected/phase8/graph-<id>.json`** (the committed key, not a generated baseline).
  Every command exited 0; actual comparison output:
  ```text
ok   fixtures/expected/phase8/graph-1.1.json (committed HEAD key)
ok   fixtures/expected/phase8/graph-1.10.json (committed HEAD key)
ok   fixtures/expected/phase8/graph-1.11.json (committed HEAD key)
ok   fixtures/expected/phase8/graph-1.12.json (committed HEAD key)
ok   fixtures/expected/phase8/graph-1.13.json (committed HEAD key)
ok   fixtures/expected/phase8/graph-1.2.json (committed HEAD key)
ok   fixtures/expected/phase8/graph-1.3.json (committed HEAD key)
ok   fixtures/expected/phase8/graph-1.4.json (committed HEAD key)
ok   fixtures/expected/phase8/graph-1.5.json (committed HEAD key)
ok   fixtures/expected/phase8/graph-1.6.json (committed HEAD key)
ok   fixtures/expected/phase8/graph-1.7.json (committed HEAD key)
ok   fixtures/expected/phase8/graph-1.8.json (committed HEAD key)
ok   fixtures/expected/phase8/graph-1.9.json (committed HEAD key)
13/13 phase 8 graphs unchanged
  ```
- Verify: ran the full `git diff 67cdabd -- fixtures/expected` (saved in scratch
  `/tmp/multi-link-14.1-expected.diff`) and byte-compared it with `git diff 67cdabd HEAD -- fixtures/expected`:
  identical (14.0's committed keys only). `git diff --stat 67cdabd -- fixtures/expected` actual output:
  ```text
 fixtures/expected/phase14/check.sh                 |  17 +
 fixtures/expected/phase14/live.json                | 690 +++++++++++++++
 fixtures/expected/tui/live-phase14-100.styles.txt  | 660 +++++++++++++++
 fixtures/expected/tui/live-phase14-100.txt         |  40 +
 .../expected/tui/live-phase14-children.styles.txt  | 930 +++++++++++++++++++++
 fixtures/expected/tui/live-phase14-children.txt    |  51 ++
 .../expected/tui/live-phase14-short.styles.txt     | 376 +++++++++
 fixtures/expected/tui/live-phase14-short.txt       |  30 +
 .../expected/tui/live-phase14-stale.styles.txt     | 600 +++++++++++++
 fixtures/expected/tui/live-phase14-stale.txt       |  40 +
 fixtures/expected/tui/live-phase14-tiny.styles.txt | 244 ++++++
 fixtures/expected/tui/live-phase14-tiny.txt        |  20 +
 fixtures/expected/tui/live-phase14.styles.txt      | 799 ++++++++++++++++++
 fixtures/expected/tui/live-phase14.txt             |  40 +
 14 files changed, 4537 insertions(+)
  ```
  `git diff --exit-code HEAD -- fixtures/` → exit 0, no output;
  `git diff --cached --exit-code` → exit 0, no output;
  `git diff --check` → exit 0, no output.
- Review: Claude, one round, 2 findings, both fixed in fix round 1: (1) `Scanner::live` read turns for children of
  every link, not only windowed ones (schema Inputs, L9 cost); (2) the group sort did an `unwrap`ed linear lookup per
  comparison. Scope: only the step's files changed; no key edited. Claude re-ran the whole Verify on the final tree:
  `cargo test --workspace` 642 passed / 0 failed / 1 ignored; clippy `-D warnings` and `fmt --check` clean;
  `phase14/check.sh` `ok   live.json`; `phase9/check.sh 6` 47 `ok` + 1 note; `phase12` 3 `ok`, `phase13` 1 `ok`;
  13/13 Phase 8 `graph` outputs equal the committed keys; `git diff 67cdabd -- fixtures/expected` = 14 files, 4537
  insertions, 0 deletions (14.0's new keys only). Matches what Codex pasted.
- Gotchas: initial compilation caught timestamp Result/error conversion and import placement; clippy caught
  nested-if style and a test Vec that should be an array; corrected before the final full Verify above. No fixtures
  or PLAN edits. Existing untracked `.claude/` left alone. L3 phrases are represented by structured data here;
  their drawing is 14.2. The refresh call cache and TUI live wiring belong to 14.3, per PLAN.md.

### 2026-11-10 — Step 14.0: Phase 14 rules, fixture, key and goldens — Claude
- Did: wrote L0–L10 into `docs/design.md` as "Phase 14 decisions" with the pins the keys needed (report fields, live
  child choice, outcome order, feed colours and narrow packing, header drop order, row layout, card cells, narrow layout,
  bottom block, `<period>`, golden inputs) and the board differences the rules override; FR12 and screen 5 rows moved
  out of Backlog. Added the `live` report to `docs/output-schema.md` (fields, groups, Card, CardLink, FeedRow, Phase 14
  fixture results). Built the synthetic root `fixtures/sessions-phase14/` (generated by a scratch script; 21 listed
  sessions, 13 links) covering every case Step 14.0 lists, including the critique's extras. `live.json` was built by an
  independent scratch script from the fixture lines plus the `sessions`/`links` reports and checked by hand; the six
  goldens were drawn by a scratch renderer from the pins and checked region by region against board 14a (and 14b for
  waiting/stale styling).
- Changed: `docs/design.md`, `docs/output-schema.md`, `fixtures/sessions-phase14/**`, `fixtures/expected/phase14/live.json`,
  `fixtures/expected/phase14/check.sh`, `fixtures/expected/tui/live-phase14{,-100,-short,-tiny,-children,-stale}.{txt,styles.txt}`,
  `PROGRESS.md`
- Verify: `cargo test --workspace` → 628 passed, 0 failed, 1 ignored (every `test result: ok`);
  `fixtures/expected/phase14/check.sh` → `FAIL live.json: command failed` / `error: unrecognized subcommand 'live'`
  (exit 1, the expected failure); W10 scan `rg -l "/Users/|\bsk-|api_key" fixtures --glob '!raw/**' --glob '!README.md'`
  → nothing (rc 1); no real ids or names from board 14a in the new files.
- Review: a fresh adversarial subagent (wrote none of it) rebuilt `live.json` with the real call/turn readers and found
  no wrong value, and found every golden cell consistent with the pins and its `.txt`. Eight findings, no defects in
  values: (1) goldens showed only the first 3 cards → added `-children` (120×51) and `-stale` (120×40) goldens and listed
  what stays a 14.2 unit test; (2) no-rollout, 21:32 tie order and non-card-parent `from` cannot reach the key → stated
  in output-schema and Current state for 14.1's unit tests; (3) `times UTC-8` spacing ambiguous → pinned "≥ 3 blank
  cells"; (4) `<period>` format and `--since all` → pinned; (5) undeclared board differences → listed; (6) W5 vs L3/L4
  words → stated L3/L4 win on this screen; (7) feed heading and key colours → pinned; (8) Codex step wording "listed
  Claude session" → "listed session" as the moved code checks.
- Gotchas: the task-output sentence must start its own line in the tool result (`Output is being written to:` is
  matched per line), or a background launch falls back to the heuristic. Fixture mode makes `repo` = `checkout`, so no
  worktree place. ratatui's `set_stringn` resets the cell after a wide character (fg, bg and modifiers to Reset), which
  is why the wide prompt sits outside every golden.

### 2026-11-10 — Step 13.3: Phase 13 real check and README — Claude
- Did: Ran the real check (scratch, counts only, W10): 8f643fd vs 46b4a36 release links, paired per hand-off; masked
  check of the 32 exact launch links; `tools/resize-key-check.py` 5 rounds; real TUI smoke. Wrote
  `docs/phase13-check.md`. README: `resume --last` paragraph (interactive resume needs an id; option-first ids), new
  "Launch folder" paragraph with C1's limits, the ambiguous-limitation now says 0 real ambiguous, removed the
  resize/key limitation (fixed by D1), added the "no hidden-id test for `cd`" limitation.
- Changed: `docs/phase13-check.md` (new), `README.md`, `PROGRESS.md`.
- Verify:
```
python3 -I tools/resize-key-check.py target/release/multi-link 5   → round 1–5: pass, exit 0
masked13.py (scratch) → right heuristic 26, right nearest_run 3, wrong 0, unlinked 1, still exact 2
links before/after (scratch) → unlinked 49 → 12, exact 58 → 59 (1 new hand-off), heuristic 132 → 170, ambiguous 1 → 0
smoke.py (scratch) → first frame 2975 bytes, redraw 10709 bytes, exit 0 on one q
```
- Review: Claude's own step (docs only). Owner's answer: "Looks good" (Phase 13 confirmed).
- Gotchas: Recounting the saved link files gives 32 `exec` unlinked → heuristic without `launch_dir` and 49 unlinked
  before; the 13.1 entry's 31 and 48 were off by one. All 37 newly linked hand-offs have a `cd` in the real command
  (36 tool calls; one call holds two hand-offs).

### 2026-11-10 — Step 13.1: Launch folder resolver — fix round 1 — Codex
- Did: Addressed all three findings from Claude review. Single `&` is a distinct tokenizer operator and cannot
  be a C1 cd boundary; a launch immediately after it remains unsupported, preserving the existing
  `cd /x & codex exec` no-hand-off rule. Restricted eval/source/./exec/builtin/command invalidation to after
  a C1 cd. Non-literal cd (after wrappers), pushd and popd still make the folder unknown even without an earlier
  C1 cd, per the clarified pushd/popd exception. Interactive `codex resume` requires an explicit UUID;
  id-less `--last` remains exclusive to `codex exec resume`. Explicit option-first interactive UUIDs still work.
  Added one unit-style regression test per finding in `crates/claude/tests/invocation.rs`, covering both sides
  of each boundary. Each was observed failing before its fix and passing afterward.
- Changed (this round only): `crates/claude/src/invocation.rs`, `crates/claude/tests/invocation.rs`, `PROGRESS.md`.
- Verify: All cargo commands used AGENTS.md's Rust PATH export. Targeted RED→GREEN commands:
  `cargo test -p multi-link-claude --test invocation single_ampersand`,
  `cargo test -p multi-link-claude --test invocation folder_builtin_deny`, and
  `cargo test -p multi-link-claude --test invocation interactive_resume_requires`.
  Actual test/result lines before and after each fix:

```text
test single_ampersand_is_not_a_literal_cd_boundary ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
test single_ampersand_is_not_a_literal_cd_boundary ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
test folder_builtin_deny_list_is_scoped_to_a_prior_literal_cd ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
test folder_builtin_deny_list_is_scoped_to_a_prior_literal_cd ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
test interactive_resume_requires_an_explicit_uuid_without_last ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
test interactive_resume_requires_an_explicit_uuid_without_last ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```

  `cargo test -p multi-link-claude --test invocation` → exit 0 (actual test/result lines):

```text
test help_options_are_not_launches ... ok
test folder_builtin_deny_list_is_scoped_to_a_prior_literal_cd ... ok
test directory_options_distinguish_current_directory_from_moves ... ok
test interactive_resume_requires_an_explicit_uuid_without_last ... ok
test earlier_unquoted_directory_words_refuse_relative_redirects ... ok
test literal_cd_tracks_only_groups_open_at_the_move ... ok
test option_first_resume_id_obeys_known_option_boundaries ... ok
test redirect_targets_are_literal_and_on_the_launch_segment ... ok
test recognition_table ... ok
test resume_last_after_exec_options_preserves_directory_and_prompt_boundaries ... ok
test resume_last_recognition_obeys_option_and_segment_boundaries ... ok
test single_ampersand_is_not_a_literal_cd_boundary ... ok
test shell_groups_heredocs_and_wrappers ... ok
test resume_last_skips_all_supported_option_values_without_consuming_equals_forms ... ok
test unknown_cd_forms_and_folder_builtins_disable_heuristics ... ok
test redirect_hard_links_are_refused_until_only_one_link_remains ... ok
test repeated_tool_use_ids_keep_the_first_occurrence_in_each_file ... ok
test extracts_only_assistant_bash_and_pairs_results_by_id_within_each_file ... ok
test redirect_outside_roots_is_refused_before_open_and_missing_is_retriable ... ok
test literal_cd_resolves_against_record_cwd_without_filesystem_access ... ok
test step_comes_from_raw_text_of_the_first_launch_segment_only ... ok
test fixture_invocations_match_all_answer_key_tuples ... ok
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
```

  `cargo test --workspace --no-fail-fast` → exit 0; 628 passed, 0 failed, 1 ignored.
  Actual result lines from the complete workspace run:

```text
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.66s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.65s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.76s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.69s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.24s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.90s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.68s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 56 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0:

```text
    Checking multi-link-model v0.1.0 (/Users/jarednarvaez/multi-link/crates/model)
    Checking multi-link-plan v0.1.0 (/Users/jarednarvaez/multi-link/crates/plan)
    Checking multi-link-codex v0.1.0 (/Users/jarednarvaez/multi-link/crates/codex)
    Checking multi-link-tui v0.1.0 (/Users/jarednarvaez/multi-link/crates/tui)
    Checking multi-link-claude v0.1.0 (/Users/jarednarvaez/multi-link/crates/claude)
    Checking multi-link-core v0.1.0 (/Users/jarednarvaez/multi-link/crates/core)
    Checking multi-link-hook v0.1.0 (/Users/jarednarvaez/multi-link/crates/hook)
    Checking multi-link v0.1.0 (/Users/jarednarvaez/multi-link/crates/cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 18.52s
```

  `cargo fmt --all -- --check` → no output, exit 0.
  `fixtures/expected/phase13/check.sh` → exit 0:

```text
ok   links.json
```

  `fixtures/expected/phase12/check.sh` → exit 0:

```text
ok   links.json
ok   overview.json
ok   graph-1.1.json
```

  `fixtures/expected/phase9/check.sh 6` → exit 0, 47/47 checks:

```text
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   graph-1.1.json (^9[1-6])
ok   graph-2.1.json (^9[1-6])
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   stderr unreadable: 1
ok   open open 92000001
ok   open open 92000002
ok   open open 92000003
ok   open open 92000004
ok   open open 92000005
ok   open open 92000006
ok   open open 92000007
ok   open open 92000008
ok   open open 92000009
ok   open open 92000010
ok   open open 92000011
ok   open open 92000012
ok   open open 92000013
ok   open open 92000014
ok   open open 94000001
ok   open open 94000002
ok   open open 94000003
ok   open open 94000004
ok   open open 94000006
ok   open open 94000007
ok   open fork 92000001
ok   open fork 92000002
ok   open fork 92000003
ok   open fork 92000004
ok   open fork 92000005
ok   open fork 92000006
ok   open fork 92000007
ok   open fork 92000008
ok   open fork 92000009
ok   open fork 92000010
ok   open fork 92000011
ok   open fork 92000012
ok   open fork 92000013
ok   open fork 92000014
ok   open fork 94000001
ok   open fork 94000002
ok   open fork 94000003
ok   open fork 94000004
ok   open fork 94000006
ok   open fork 94000007
note lanes.json is checked by the 9.5 worker-to-App test, not here
```

  Re-ran `git diff 8d2cc22 -- fixtures/expected` (saved the full patch). Its stat:

```text
 fixtures/expected/phase12/links.json |  11 +-
 fixtures/expected/phase13/check.sh   |  17 ++
 fixtures/expected/phase13/links.json | 569 +++++++++++++++++++++++++++++++++++
 3 files changed, 592 insertions(+), 5 deletions(-)
```

  Relative to Claude's Step 13.0 commit (`git diff b13656d -- fixtures/expected`), the only extra change
  is Claude's case (k) correction, already present when this fix round began:

```diff
diff --git a/fixtures/expected/phase12/links.json b/fixtures/expected/phase12/links.json
index e6c2022..165b12a 100644
--- a/fixtures/expected/phase12/links.json
+++ b/fixtures/expected/phase12/links.json
@@ -486,7 +486,8 @@
       "match": "unlinked",
       "child": null,
       "evidence": null,
-      "candidates": []
+      "candidates": [],
+      "launch_dir": "/home/dev/repo-p12k/sub"
     },
     {
       "step": "1.2",
```

  Compared `git diff -- fixtures/expected` at the start and end of this round with `cmp`:

```text
Answer-key diff unchanged during fix round: exit 0
git diff --check: exit 0
git diff --cached --name-only: exit 0
```

  Both git checks produced no diff/file output. No existing recognition test expectations changed in this round.
  The original C1/C3 expectation changes remain named in the previous Step 13.1 entry.
- Review: Claude, 1 fix round. Round 0: Codex stopped on a Claude key error: Phase 12 case (k) runs
  `cd /home/dev/repo-p12k/sub && codex exec …`, which C1 resolves, so its link gains `launch_dir` (still
  `unlinked`, no run there); Claude fixed the key. Claude's 13.0 entry was wrong to say no other committed key had a
  `cd` before a codex call (the grep missed it). Findings sent in round 1, all fixed with tests: a single `&` was
  read as a newline boundary; the builtin deny-list fired with no earlier C1 `cd` (would have unlinked e.g.
  `command -v codex && codex exec`); interactive `codex resume --last` became an R7 resume. Existing test changes
  accepted (C1: `cd /elsewhere;` resolves; C3: `codex exec -s read-only resume <id>` is a resume). Claude re-ran:
  `cargo test --workspace` 628 passed / 0 failed / 1 ignored; clippy, fmt clean; phase12 and phase13 `check.sh` all
  `ok`; phase9 47/47; `git diff 8d2cc22 -- fixtures/expected` = 13.0's keys plus the case (k) fix. Real run (release,
  scratch, counts only), per hand-off against the 8f643fd build: 31 `exec` unlinked → heuristic (mostly `cd` into the
  session's own folder, so no `launch_dir`), 1 with `launch_dir`, 1 unlinked → `nearest_run`, 3 `resume` unlinked →
  `resume_turn`, the last ambiguous → `nearest_run`, 1 exact `exec` → exact `resume` / `resume_arg`; none lost a link.
- Gotchas: At the start of this round the tokenizer retained a single `&` as a word rather than an operator;
  it now explicitly records `&` without broadening launch recognition immediately after it. The pushd/popd
  exception in the review instruction is preserved, as the Phase 13 negative fixture requires.
  Claude's Phase 12 key amendment was pre-existing and was not edited by Codex. PLAN.md and all other
  pre-existing worktree changes were left untouched this round.


### 2026-11-10 — Step 13.1: Launch folder and option-first id in the resolver — Codex
- Did: Implemented C1 literal `cd` resolution from the record cwd with lexical normalisation, strict path/operator
  rules and invalidating folder builtins. Tokenizer tracks group depth: closing a group open at the cd invalidates
  it; groups opened later and opaque command substitutions do not. Added C3 known-option scanning for explicit
  resume UUIDs, preserving option-value consumption, equals forms, `--last`, and first-positional boundaries.
  C2 carries optional `launch_dir` through Invocation and Link, omits it when None, and uses launch folder for
  creation candidates, both K1 map accesses and R7 later turns. Redirect evidence rules stay intact.
  Added literal/unknown/group/normalisation/C3 tables and JSON omission/round-trip coverage. The new
  `literal_cd_tracks_only_groups_open_at_the_move` and `option_first_resume_id_obeys_known_option_boundaries`
  tests were observed failing before implementation and passing afterward.
- Changed: `crates/claude/src/invocation.rs`, `crates/claude/tests/invocation.rs`, `crates/model/src/lib.rs`,
  `crates/model/tests/model.rs`, `crates/core/src/link.rs`, `crates/core/src/overview.rs`,
  `crates/core/tests/graph.rs`, `crates/core/tests/links.rs`, `PROGRESS.md`.
- Verify: All cargo commands used the AGENTS.md Rust PATH export. Every Step 13.1 Verify item was run;
  this step is blocked, not complete. Final `cargo test --workspace --no-fail-fast` exited 101: 624 passed,
  1 failed, 1 ignored. Below are the actual result/error lines from that run (not the very large JSON assertion):

```text
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.51s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.13s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.06s
test phase12_links_match_answer_key ... FAILED
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s
error: test failed, to rerun pass `-p multi-link --test phase12`
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.73s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.20s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 56 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
error: 1 target failed:
    `-p multi-link --test phase12`
```

  `fixtures/expected/phase13/check.sh` → exit 0:

```text
ok   links.json
```

  `fixtures/expected/phase12/check.sh` → exit 1:

```text
FAIL links.json
485a486
>       "launch_dir": "/home/dev/repo-p12k/sub",
ok   overview.json
ok   graph-1.1.json
```

  `fixtures/expected/phase9/check.sh 6` → exit 0, 47/47 checks:

```text
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   graph-1.1.json (^9[1-6])
ok   graph-2.1.json (^9[1-6])
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   stderr unreadable: 1
ok   open open 92000001
ok   open open 92000002
ok   open open 92000003
ok   open open 92000004
ok   open open 92000005
ok   open open 92000006
ok   open open 92000007
ok   open open 92000008
ok   open open 92000009
ok   open open 92000010
ok   open open 92000011
ok   open open 92000012
ok   open open 92000013
ok   open open 92000014
ok   open open 94000001
ok   open open 94000002
ok   open open 94000003
ok   open open 94000004
ok   open open 94000006
ok   open open 94000007
ok   open fork 92000001
ok   open fork 92000002
ok   open fork 92000003
ok   open fork 92000004
ok   open fork 92000005
ok   open fork 92000006
ok   open fork 92000007
ok   open fork 92000008
ok   open fork 92000009
ok   open fork 92000010
ok   open fork 92000011
ok   open fork 92000012
ok   open fork 92000013
ok   open fork 92000014
ok   open fork 94000001
ok   open fork 94000002
ok   open fork 94000003
ok   open fork 94000004
ok   open fork 94000006
ok   open fork 94000007
note lanes.json is checked by the 9.5 worker-to-App test, not here
```

  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0:

```text
    Checking multi-link-model v0.1.0 (/Users/jarednarvaez/multi-link/crates/model)
    Checking multi-link-plan v0.1.0 (/Users/jarednarvaez/multi-link/crates/plan)
    Checking multi-link-codex v0.1.0 (/Users/jarednarvaez/multi-link/crates/codex)
    Checking multi-link-tui v0.1.0 (/Users/jarednarvaez/multi-link/crates/tui)
    Checking multi-link-claude v0.1.0 (/Users/jarednarvaez/multi-link/crates/claude)
    Checking multi-link-core v0.1.0 (/Users/jarednarvaez/multi-link/crates/core)
    Checking multi-link-hook v0.1.0 (/Users/jarednarvaez/multi-link/crates/hook)
    Checking multi-link v0.1.0 (/Users/jarednarvaez/multi-link/crates/cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 18.85s
```

  `cargo fmt --all -- --check` → no output, exit 0. `git diff --check` → no output, exit 0.
  `git diff 8d2cc22 -- fixtures/expected` was saved and compared with
  `git diff 8d2cc22 b13656d -- fixtures/expected` (Claude's Step 13.0 commit) using `cmp`:

```text
Key diff matches Step 13.0 exactly: exit 0
```

  `git diff b13656d --exit-code -- fixtures/expected` and
  `git diff HEAD --exit-code -- fixtures/expected` both produced no diff and exited 0.
  Existing recognition tests changed (no other existing recognition expectations changed):
  - `resume_last_recognition_obeys_option_and_segment_boundaries`: C1 now resolves the bare absolute
    `cd /elsewhere;` before resume; expects `dir_moved == false` and `launch_dir == "/elsewhere"`.
  - `resume_last_after_exec_options_preserves_directory_and_prompt_boundaries`: C3 now recognises
    `codex exec -s read-only resume <uuid>` as Resume with the explicit id, rather than Exec.
- Review: Pending Claude (the non-author). No staging, commits or key edits. Existing untracked `.claude/`
  was present at the start and left untouched.
- Gotchas: The only remaining failure is a binding schema/key conflict: Phase 12 case (k),
  `toolu_p12_12a00c01_0`, has a valid bare absolute cd to `/home/dev/repo-p12k/sub`, with no matching rollout there.
  C1/C2 and the Link launch_dir row require the resolved folder even though the link remains unlinked; its key
  omits that field (check diff above). Suppressing the field merely because no child matched would violate the
  schema. Stopped implementation and asked for Claude's key correction or clarification of an intended exception;
  did not edit the key or weaken the comparison. PLAN.md was not edited.


### 2026-11-10 — Step 13.2: Keys read with `use-dev-tty` — Codex
- Did: enabled crossterm's `use-dev-tty` feature (D1); lockfile regenerated. Ran alongside Claude's 13.0 (Codex told
  to touch only `Cargo.toml` / `Cargo.lock` and report its entry instead of editing PROGRESS.md; Claude pasted it).
- Changed: `Cargo.toml`, `Cargo.lock` (adds `filedescriptor 0.8.3`, `thiserror 1.0.69`, `thiserror-impl 1.0.69`).
- Verify (Codex): `cargo build --release -p multi-link` exit 0; `python3 -I tools/resize-key-check.py
  target/release/multi-link 5` → rounds 1–5 `pass`, exit 0; `cargo test --workspace --no-fail-fast` 619 passed / 0
  failed (before Claude's Phase 13 tests existed); clippy `-D warnings` and `fmt --check` clean; no snapshot changed.
- Review: Claude, clean (one-line feature change). Claude re-ran: release build, resize check 5 of 5 `pass`
  (8d2cc22's build: 5 of 5 FAIL); clippy and fmt clean; with 13.0's files present `cargo test` 618 passed / 2
  failed, exactly 13.0's two listed tests; no TUI snapshot or key changed.
- Gotchas: `use-dev-tty` reads keys from `/dev/tty` with poll(2); the TUI already requires a terminal.

### 2026-11-10 — Step 13.0: Rules, fixtures, keys and the resize check — Claude
- Did: C1–C3 and D1 into `docs/design.md` and `docs/output-schema.md`; synthetic root `fixtures/sessions-phase13/`
  (cases a–j, scratch generator, ids `13a…` / `13c…`, tempting runs beside every negative form); Claude-written
  `fixtures/expected/phase13/links.json` with `check.sh` and `crates/cli/tests/phase13.rs`; the Phase 12 key value C3
  changes; `tools/resize-key-check.py` (revised per critique 7–8: clean exit required, third final size).
- Changed: `docs/design.md`, `docs/output-schema.md`, `fixtures/sessions-phase13/**`, `fixtures/expected/phase13/**`,
  `crates/cli/tests/phase13.rs`, `fixtures/expected/phase12/links.json`, `tools/resize-key-check.py`, `PROGRESS.md`.
- Listed key change: `phase12/links.json` `toolu_p12_12a00901_2` (`codex exec resume -c model=x <id> …`): `exec` /
  `unlinked` → `resume` / `exact` / `resume_arg`, child `12c00901`. No other committed key has a `cd` before a codex
  call (grep over every fixture root), so C1 changes nothing else.
- Verify: `cargo test --workspace --no-fail-fast` 618 passed / 2 failed: `phase13_links_match_answer_key`,
  `phase12_links_match_answer_key` (as listed); `phase12/check.sh` FAIL only on links.json; `phase13/check.sh` FAIL
  (no code yet). Resize check at 8d2cc22's build: 5 of 5 `FAIL (key delayed until a second key)`; a throwaway
  `use-dev-tty` build: 5 of 5 `pass`. W10 scan of the new files: nothing.
- Review: Claude step.
- Gotchas: today `cd /x & codex exec …` is not recognised as a hand-off at all (single `&`); the key keeps it absent.
  `pushd` was not treated as a move before; C1 makes it unknown (case e changes `heuristic` → `unlinked`).

### 2026-11-10 — Step 12.2: Real check, README, owner confirm — Claude
- Did: real before/after transitions by hand-off (release builds 04b4233 and 437adf4, scratch, counts and gaps only);
  masked check with the built resolver (scratch harness: per exact launch link, that checkout's real transcripts in a
  temp root, child id replaced in the parent transcript); `docs/phase12-check.md`; README nearest-run bullet and the
  ambiguous-links limitation rewritten; corrected "59 exact" to 53 in the 12.1 Review line.
- Changed: `docs/phase12-check.md`, `README.md`, `PROGRESS.md`.
- Verify: transitions on the 233 exported ids equal K4 (53 exact, 6 resume_turn, 117 heuristic, 47 unlinked
  unchanged; 1 heuristic → unlinked, own gap 9.4 s vs 2.2 s to a later hand-off that picks it; 8 ambiguous →
  nearest_run; 1 ambiguous stays at 0.91 s / 7.88 s). Masked: 30 links, 23 + 2 right, 0 wrong, 3 unlinked (1 child
  before the hand-off, 2 claimed by a second exact launch of the same child), 2 still exact. Code unchanged since
  437adf4 (tests 619/0/1 there). W10 scan of the changed files: nothing.
- Review: Claude step. Owner looked and answered "Looks good" (2026-11-10).
- Gotchas: the masked unlinked count (3) is higher than the planning simulation (1) because the harness keeps other
  exact launches, which claim a shared child (plugin wrapper + inner exec). Correct under the existing claim rule.

### 2026-11-10 — Step 12.1: K1–K3 in the link resolver (key correction pending) — Codex
- Did: Implemented K1 with original-choice minimum gaps per checkout/run, followed by K2/K3 on survivors.
  Exact reservations and R7 turn matching retain their existing paths. Added `LinkEvidence::NearestRun`, optional
  graph edge serialization and conditional TUI words through the existing shared lane helper. Added tests for
  boundaries, ties, order, original choices (including no fixed-point iteration), exclusions, R7, serialization
  and the words at 120/80 columns. The owner amended model-test scope after the initial stop.
- Changed: `crates/model/src/lib.rs`, `crates/model/tests/graph.rs`, `crates/core/src/link.rs`,
  `crates/core/src/graph.rs`, `crates/core/tests/links.rs`, `crates/tui/src/view.rs`, `crates/tui/src/graph.rs`,
  `crates/tui/tests/timeline.rs`, `crates/tui/tests/graph.rs`, `crates/cli/tests/phase12.rs` (rustfmt only), `PROGRESS.md`.
- Verify: Pre-change commands below used the required Rust PATH. Actual output excerpts (large assertion
  value dumps omitted); these are failures, not completion checks.

  `cargo test -p multi-link --test phase12` →
  ```text
  running 3 tests
  test phase12_graph_uses_the_nearest_run ... FAILED
  test phase12_links_match_answer_key ... FAILED
  test phase12_overview_reparents_nearest_runs ... FAILED

  failures:
      phase12_graph_uses_the_nearest_run
      phase12_links_match_answer_key
      phase12_overview_reparents_nearest_runs

  test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s

  error: test failed, to rerun pass `-p multi-link --test phase12`
  ```

  `cargo test -p multi-link-core --test links ambiguous_variant_equals_answer_key` →
  ```text
  running 1 test
  test ambiguous_variant_equals_answer_key ... FAILED

  failures:
      ambiguous_variant_equals_answer_key

  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.33s

  error: test failed, to rerun pass `-p multi-link-core --test links`
  ```

  `cargo test -p multi-link-model every_expected_report_round_trips` →
  ```text
  thread 'every_expected_report_round_trips' (6091371) panicked at crates/model/tests/model.rs:115:62:
  called `Result::unwrap()` on an `Err` value: Error("unknown variant `nearest_run`, expected one of `stdout`, `resume_arg`, `thread_ready`, `task_output`, `redirect_file`, `resume_turn`, `hook`", line: 0, column: 0)
  note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

  failures:
      every_expected_report_round_trips

  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.02s

  error: test failed, to rerun pass `-p multi-link-model --test model`
  ```

  Post-change commands below used the required Rust PATH. Output excerpts are pasted verbatim; workspace output
  includes every target's result, with individual passing test names and large assertion dumps omitted.
  `cargo test -p multi-link-core --lib link::tests` → exit 0:
  ```text
running 4 tests
test link::tests::resume_turns_do_not_use_nearest_run_margins_or_creation_competition ... ok
test link::tests::competitors_keep_equal_gaps_and_ignore_other_checkouts_moves_and_resumes ... ok
test link::tests::competitors_use_original_choices_once_independent_of_order ... ok
test link::tests::nearest_run_boundaries_and_original_candidate_evidence ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 48 filtered out; finished in 0.00s
  ```
  `cargo test --workspace --no-fail-fast` → exit 101:
  ```text
     Running unittests src/main.rs (target/debug/deps/multi_link-cac680f01a88ebba)
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
     Running tests/cli.rs (target/debug/deps/cli-8c01f063f9a0379e)
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.66s
     Running tests/graph.rs (target/debug/deps/graph-f001f1d8121ee529)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
     Running tests/hook_check.rs (target/debug/deps/hook_check-152098d6643c7d4f)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.77s
     Running tests/phase12.rs (target/debug/deps/phase12-1b421b346333a6ee)
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
error: test failed, to rerun pass `-p multi-link --test phase12`
     Running tests/phase9.rs (target/debug/deps/phase9-12c6295aea63b214)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running unittests src/lib.rs (target/debug/deps/multi_link_claude-3007d227ffc8769a)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/app_record.rs (target/debug/deps/app_record-c6763d08001f1e6c)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/calls.rs (target/debug/deps/calls-45eb22dd6795bdda)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/fixtures.rs (target/debug/deps/fixtures-499f3a2e254ccd24)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
     Running tests/invocation.rs (target/debug/deps/invocation-3e16b8bc37be96e1)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
     Running tests/last_turn.rs (target/debug/deps/last_turn-b71057714a3b5979)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/phase6.rs (target/debug/deps/phase6-e2db30bd15f79be3)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/phase7.rs (target/debug/deps/phase7-2c8b6d1063153907)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running unittests src/lib.rs (target/debug/deps/multi_link_codex-e78d5e99d41da51c)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/calls.rs (target/debug/deps/calls-586748b917de5a55)
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/compressed.rs (target/debug/deps/compressed-a0fe57bc51027b49)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.72s
     Running tests/fixtures.rs (target/debug/deps/fixtures-ba5f82572d1c4a8d)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/phase6.rs (target/debug/deps/phase6-d1fad84a09bb2e55)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running unittests src/lib.rs (target/debug/deps/multi_link_core-b9f5577635e977c9)
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.16s
     Running tests/compressed.rs (target/debug/deps/compressed-23fd67b2786194f6)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/graph.rs (target/debug/deps/graph-367a1c66ea7fcc9e)
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/graph_commit.rs (target/debug/deps/graph_commit-413fc54f4401db0c)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
     Running tests/hook.rs (target/debug/deps/hook-9d2c70dbcc7efb25)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s
     Running tests/links.rs (target/debug/deps/links-f8b4f67c998fff9c)
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
     Running tests/overview.rs (target/debug/deps/overview-958464a7d8ad16d7)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.68s
     Running tests/phase6.rs (target/debug/deps/phase6-a939478b5426845c)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/repo_probe.rs (target/debug/deps/repo_probe-489df8ea92fb062c)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
     Running tests/session_end.rs (target/debug/deps/session_end-97da6fcc150d54f9)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/sessions.rs (target/debug/deps/sessions-59bcd1019b17c9c4)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
     Running tests/started.rs (target/debug/deps/started-d7b62c3a1b08ef27)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
     Running tests/transcript_exit.rs (target/debug/deps/transcript_exit-27b68d6dc1e2bf21)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
     Running unittests src/lib.rs (target/debug/deps/multi_link_hook-62ceaf9ce3262d6f)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/hook.rs (target/debug/deps/hook-312d13ac9a0e316c)
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
     Running tests/settings.rs (target/debug/deps/settings-22861df00bc1ebd7)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/multi_link_model-df157fa80d5a3417)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/calls.rs (target/debug/deps/calls-5289ef8606320dd4)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/graph.rs (target/debug/deps/graph-bb878fdee95626eb)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/model.rs (target/debug/deps/model-452af6df96c138b7)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running unittests src/lib.rs (target/debug/deps/multi_link_plan-b5ab79d96e45b0d3)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/fixtures.rs (target/debug/deps/fixtures-1edf5b4804af5d92)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/multi_link_tui-548055adc3cdb49a)
test result: ok. 56 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
     Running tests/graph.rs (target/debug/deps/graph-3ad73ce1cd9b6cc6)
test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
     Running tests/home.rs (target/debug/deps/home-7fa1d0c4e28679e3)
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running tests/phase6_shape.rs (target/debug/deps/phase6_shape-1ebc76e16856c05b)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/phase8_shape.rs (target/debug/deps/phase8_shape-6208c77eec7f7412)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/started.rs (target/debug/deps/started-0c445af505d452c8)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/theme.rs (target/debug/deps/theme-2754cdbe81372a77)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/timeline.rs (target/debug/deps/timeline-9a45a9c85f7f2dd2)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/view.rs (target/debug/deps/view-9ba5ceb92b04b29b)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_claude
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_codex
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_hook
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_model
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_plan
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_tui
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
error: 1 target failed:
    `-p multi-link --test phase12`
  ```
  Named acceptance/serialization/TUI results from that same workspace run:
  ```text
test phase12_graph_uses_the_nearest_run ... ok
test phase12_links_match_answer_key ... FAILED
test phase12_overview_reparents_nearest_runs ... ok
test ambiguous_variant_equals_answer_key ... ok
test nearest_run_edges_round_trip_and_false_is_omitted ... ok
test every_expected_report_round_trips ... ok
test nearest_run_words_follow_edge_evidence_in_graph_and_list ... ok
test nearest_run_words_survive_lane_truncation_at_120_and_80 ... ok
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, final output:
  ```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 27.51s
  ```
  `cargo fmt --all -- --check` and `git diff --check` → both exit 0, no output.
  `fixtures/expected/phase12/check.sh` → exit 1:
  ```text
FAIL links.json
512,513d511
<     "12c00101-0000-4000-8000-000012c00101",
<     "12c00201-0000-4000-8000-000012c00201",
519d516
<     "12c00701-0000-4000-8000-000012c00701",
521d517
<     "12c00a01-0000-4000-8000-000012c00a01",
524d519
<     "12c00b01-0000-4000-8000-000012c00b01",
526d520
<     "12c00b03-0000-4000-8000-000012c00b03",
532d525
<     "12c00b09-0000-4000-8000-000012c00b09",
ok   overview.json
ok   graph-1.1.json
  ```
  `fixtures/expected/phase9/check.sh 6` → exit 0 (47 `ok` lines):
  ```text
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   graph-1.1.json (^9[1-6])
ok   graph-2.1.json (^9[1-6])
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   stderr unreadable: 1
ok   open open 92000001
ok   open open 92000002
ok   open open 92000003
ok   open open 92000004
ok   open open 92000005
ok   open open 92000006
ok   open open 92000007
ok   open open 92000008
ok   open open 92000009
ok   open open 92000010
ok   open open 92000011
ok   open open 92000012
ok   open open 92000013
ok   open open 92000014
ok   open open 94000001
ok   open open 94000002
ok   open open 94000003
ok   open open 94000004
ok   open open 94000006
ok   open open 94000007
ok   open fork 92000001
ok   open fork 92000002
ok   open fork 92000003
ok   open fork 92000004
ok   open fork 92000005
ok   open fork 92000006
ok   open fork 92000007
ok   open fork 92000008
ok   open fork 92000009
ok   open fork 92000010
ok   open fork 92000011
ok   open fork 92000012
ok   open fork 92000013
ok   open fork 92000014
ok   open fork 94000001
ok   open fork 94000002
ok   open fork 94000003
ok   open fork 94000004
ok   open fork 94000006
ok   open fork 94000007
note lanes.json is checked by the 9.5 worker-to-App test, not here
  ```
  `git diff 04b4233 -- fixtures/expected` → exit 0, diff headers (only Step 12.0's keys):
  ```text
diff --git a/fixtures/expected/links-amb-hidden.json b/fixtures/expected/links-amb-hidden.json
diff --git a/fixtures/expected/phase12/check.sh b/fixtures/expected/phase12/check.sh
diff --git a/fixtures/expected/phase12/graph-1.1.json b/fixtures/expected/phase12/graph-1.1.json
diff --git a/fixtures/expected/phase12/links.json b/fixtures/expected/phase12/links.json
diff --git a/fixtures/expected/phase12/overview.json b/fixtures/expected/phase12/overview.json
  ```
  `git diff --exit-code HEAD -- fixtures/expected` → exit 0, no output: no answer keys changed here.
  `git diff --cached --name-only` → exit 0, no output. Real-data run deliberately left to Claude.
- Review: Claude, round 1 clean (no fix round). Codex stopped once on scope (Edge in `crates/model/tests/graph.rs`;
  Files amended, Review log). Codex was right about the key: Claude's `phase12/links.json` kept the 7 newly chosen
  children in `unparented`; Claude removed them (key fix, Claude-owned). Claude re-ran Verify: `cargo test --workspace`
  619 passed / 0 failed / 1 ignored; clippy `-D warnings` and `fmt --check` clean; `phase12/check.sh` 3/3 `ok`;
  `phase9/check.sh 6` 47/47; `git diff a746a8d -- fixtures/expected` = only that key fix. Resolver read against K1–K3:
  own-choice map from original lists, one pass, ties kept, R7 single-candidate only. Real run (release build, scratch,
  counts only): 236 links now (3 new since the export); transitions on the exported ones equal K4: 8 ambiguous →
  heuristic / nearest_run, 1 ambiguous stays, 1 heuristic → unlinked, 117 heuristic, 53 exact + 6 resume_turn + 47
  unlinked unchanged. Minor, not fixed: the graph's 19-cell hand-off box cuts the new words to `matched by time · …`.
- Gotchas: Changed `heuristic_window_is_inclusive_and_checkout_is_an_exact_string` in `core/tests/links.rs`:
  its two candidates have gaps 0 and 60000 ms, so K2 chooses A and K3 requires `nearest_run` (original list size 2).
  A therefore leaves `unparented` under the existing schema. The expected-link helper now treats `nearest_run`
  and `resume_turn` evidence as heuristic rather than exact, as K0/K3 and R7 require. No other existing links unit
  expectation changed. The graph's fixed-width connector label retains its existing clipping; the full phrase
  survives in its selected-origin sub line and narrow incoming-edge row, tested at 120 and 80 columns.
  The Phase 12 links key's `unparented` mismatch is independently confirmed by comparing parsed JSON: every link
  field agrees, and these seven entries occur only in the key's `unparented` list:
  - `12c00101-0000-4000-8000-000012c00101`
  - `12c00201-0000-4000-8000-000012c00201`
  - `12c00701-0000-4000-8000-000012c00701`
  - `12c00a01-0000-4000-8000-000012c00a01`
  - `12c00b01-0000-4000-8000-000012c00b01`
  - `12c00b03-0000-4000-8000-000012c00b03`
  - `12c00b09-0000-4000-8000-000012c00b09`
  The schema says selected heuristic children cannot be unparented; the existing core report assembler already
  implements that rule, so it was not changed to fit this inconsistent key. The amended legacy ambiguous key also
  removes its newly selected child from `unparented`. All other workspace targets pass; one pre-existing test is
  ignored. The pre-existing untracked `.claude/` and the owner's PLAN.md amendment were left untouched.

### 2026-11-10 — Step 12.0: Rules, fixtures and answer keys — Claude
- Did: K0–K4 into `docs/design.md` and `docs/output-schema.md`; synthetic fixture root `fixtures/sessions-phase12/`
  (16 checkouts, cases a–k; built by a scratch generator, ids `12a…` / `12c…`); Claude-written keys `links.json`,
  `overview.json` (projection), `graph-1.1.json` with `check.sh` and `crates/cli/tests/phase12.rs`; the one committed
  key value K1 changes. PLAN amended (K1 "own choice", K2 scope, K3 words + `Edge.nearest_run`, Files; Review log).
- Changed: `docs/design.md`, `docs/output-schema.md`, `fixtures/sessions-phase12/**`, `fixtures/expected/phase12/**`,
  `crates/cli/tests/phase12.rs`, `fixtures/expected/links-amb-hidden.json`, `crates/core/tests/keys-before-phase6.json`,
  `PLAN.md`, `PROGRESS.md`.
- Listed key change: `links-amb-hidden.json` (and its copy in `keys-before-phase6.json`) amb-1
  `toolu_01REKU3KvENtvGohuKSvtKD3`: `ambiguous` [d501 746, d953 1822] → `heuristic` d501, `nearest_run`, candidates
  [d501 746]; d501 leaves `unparented`. d501 is amb-1's true child (exact in the unmasked fixture). Every other
  committed links key: no change by simulation over all 8 fixture roots (phase 8 and 9 ambiguous cases stay).
- Verify: `cargo test --workspace --no-fail-fast` 607 passed / 5 failed, exactly the 5 listed in Current state;
  `fixtures/expected/phase12/check.sh` runs, FAIL on links.json, overview.json, graph-1.1.json (no code yet), and its
  diffs touch only cases a, b, e, f2, i, j1, j2, j5; the Rust and jq overview projections agree (checked by swapping
  in today's projection: test passed); `check.sh 6` 47/47 `ok`; W10 scan of the new files and added doc lines: nothing.
- Review: Claude step. Real-data simulation of the amended K1: same transitions as the K4 table.
- Gotchas: fixture ids must be uuid-shaped (36 chars) or exact evidence is silently ignored. The option-first
  explicit-id resume form is recognised as `exec` today (case h pins it; backlog).

### 2026-11-10 — Step 11.3: Real check, README images, owner confirm — Claude
- Did: real check of the release TUI (120×40 PTY harness, scratch only) through a real sandbox `claude -p` →
  `codex exec` → `resume --last` cycle on a synthetic one-step plan; README sections for the now line and live parts
  and the new image; Phase 11 decisions table in `docs/design.md`; re-shot the repo image and added the live step
  image from fixtures; `docs/phase11-check.md` (Files amended, Review log).
- Changed: `README.md`, `docs/design.md`, `docs/phase11-check.md`, `docs/img/repo-phase6-fixture.png`,
  `docs/img/step-live-fixture.png`, `PLAN.md` (Files amendment), `PROGRESS.md`.
- Verify: `cargo test --workspace` 609 passed / 0 failed / 1 ignored; clippy `-D warnings` and `fmt --check` clean;
  JSON keys unchanged since the plan commit 2584519; `check.sh 6` 47/47 `ok`; TUI snapshots changed since 2584519:
  `repo-phase6` (N1–N3 cells, reviewed in 11.1) and new `repo-now`, `step-live`; `step-phase8` unchanged. Image text
  equals `repo-phase6.txt` and `step-live.txt` (40/40 lines each). Real frames (step 1.2, 20 graph frames): live part
  was implement → plan (1 frame, between parts) → review → fix round 1 (superseded implement not live) → review →
  none; idle frames had no now-colour cell. W10 scan over changed files: no `/Users/`, no real ids.
- Review: Claude step (coordinator session ran it, owner asked 2026-11-10). Owner looked at the real TUI and
  answered "Looks good" (2026-11-10).
- Gotchas: home order changes while a repo appears, so the harness resets to the top before each ↓ walk. A working
  bar can end one cell before `┃` (last record a few seconds old); documented, not changed.

### 2026-11-10 — Step 11.2: Step graph live box — Codex
- Did: Implemented L1–L7 / boards 11d, 11f, 11g, 11i. Core `graph::live_parts` selects the latest
  interval per session (ties by part index), requires Working/available Codex or Busy/fresh Claude,
  and suppresses Claude while a later Codex part in its chain is live. An unavailable resume's
  hand-off dates its window, so the old run cannot stay live. Commits do not suppress resumed work.
  The all-time worker computes indices; Refresh/App keep or reject them together with their graph,
  clearing them on a step switch. No model or serialized JSON fields changed.
  Live boxes have bold heavy harness-coloured borders and a now-coloured badge; L3 dims only inactive
  borders/dots, retaining harness words, text, arrows, selection, and grey unavailable boxes.
  Hidden-part indicators count parts across slot gaps. Narrow titles reserve badge width before cutting;
  scrolled-out live titles get up/down indicators. Added `home_html --live 1,3` for fixture screenshots.
- Changed: `crates/core/src/graph.rs`, `crates/core/tests/graph.rs`, `crates/cli/src/tui.rs`,
  `crates/tui/src/graph.rs`, `crates/tui/src/app.rs`, `crates/tui/tests/graph.rs`,
  `crates/tui/tests/started.rs` (one empty Refresh initializer only), `crates/tui/examples/home_html.rs`,
  new `fixtures/expected/tui/step-live.txt`, new `fixtures/expected/tui/step-live.styles.txt`, `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md. `cargo test --workspace --offline` → exit 0.
  Actual relevant target output (copied from the final run):
  ```text
  CLI unit tests:
  test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
  CLI tests/graph.rs:
  test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
  Core tests/graph.rs:
  test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
  TUI unit tests:
  test result: ok. 56 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
  TUI tests/graph.rs:
  test result: ok. 86 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
  ```
  Python aggregation of that run's `test result:` lines printed:
  ```text
  workspace totals: 56 targets; 609 passed; 0 failed; 1 ignored
  ```
  Existing ignored test: `real_counts`. All 13 Phase 8 graph JSON keys match via
  `every_phase8_key_matches`; Phase 9 graph keys match via the check script below.
  New tests cover the 5-min boundary, non-Busy states, latest/superseded/unavailable runs,
  independent parents and A→B→A chains, same-chain suppression, work after a commit, worker step/freshness
  transitions, rejected stale reports, 120/119/80-column rendering, both live marks, exact hidden-part counts
  across same-harness gaps, long-title reservation, list scrolling, NO_COLOR selection, grey boxes, unchanged
  text/arrows, and the new snapshot and live HTML example.
- Verify: `cargo clippy --workspace --all-targets --offline -- -D warnings` → exit 0; actual final line:
  ```text
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.09s
  ```
  `cargo fmt --all -- --check` and `git diff --check` → no output, exit 0.
  `fixtures/expected/phase9/check.sh 6` → exit 0; actual output ends:
  ```text
  ok   open fork 94000003
  ok   open fork 94000004
  ok   open fork 94000006
  ok   open fork 94000007
  note lanes.json is checked by the 9.5 worker-to-App test, not here
  ```
  Counting its output printed `phase9/check.sh 6: 47 ok; 0 FAIL`.
  `git diff --exit-code -- fixtures/expected/tui/step-phase8.txt fixtures/expected/tui/step-phase8.styles.txt`
  → no output, exit 0. Existing JSON answer keys unchanged. W10 added-line/new-snapshot scan for personal
  paths/names and token patterns printed `W10 added-line/new-snapshot scan: no matches`; new snapshot content
  uses only the synthetic Phase 8 graph-1.5 fixture, with displayed session ids limited to eight characters.
- Review: Claude, 1 pass, no findings (session coordinator ran this step itself, owner asked 2026-11-10). Codex
  first stopped on an unlisted file (`started.rs` Refresh initializer); Claude widened Files to `crates/tui/tests/**`
  (Review log). Diff touches only the amended Files; `started.rs` is a one-line initializer. Checked: `live_parts`
  follows L1 (last part per session by `(from or hand-off at, index)`, Codex `Working` + run, Claude `Busy` + `to`
  within 5 min + no later live Codex in its chain); worker computes it with the graph, App keeps it only with an
  accepted graph; `step-live` shows the heavy border and `● now` on fix round 1, plan/implement borders in
  `#8A6448`/`#4F79B3`, harness words and text unchanged. Re-ran Verify: 609 passed / 0 failed / 1 ignored; clippy
  `-D warnings` and `fmt --check` clean; JSON keys, `step-phase8` and `repo-*` snapshots unchanged; `check.sh 6`
  47/47 `ok`. Mode: Codex loop, quota `primary 33%, secondary 37%, CODEX: OK`.
- Gotchas: Claude amended the Files list before implementation; the old file-list blocker is resolved.
  L3/Review log take precedence over the Verify sentence requesting dim harness words.
  Initial test setup incorrectly assumed graph-1.1 had a commit and seven list items overflowed: corrected by
  supplying an explicit commit and a longer list. A regression failed with old run index 1 live after an
  unavailable resume; using its known hand-off time fixed it. The live HTML check initially searched raw HTML
  for contiguous badge text, but each cell has its own span; checking rendered text fixed it. Final suite is green.
  PLAN.md's pre-existing Claude amendment and untracked `.claude/` were left untouched. Nothing staged or committed.
  Owner-supplied quota: primary 33%, secondary 37%, CODEX: OK. Real-data transition/owner confirmation is Step 11.3.

### 2026-11-10 — Step 11.1: Repo view now line — Codex
- Did: Implemented N0–N4 / boards 11b and 11e: dedicated now palette, presentation-only column resolver,
  `▼` on the axis rule and `┃` after links/before lanes, preserving backgrounds and bar/text/completion paint.
  Now labels reserve their space first, shift left at the edge, and displace overlapping folds/end clocks/ticks.
  Added fixed-fixture tests for fitted/unfitted, folds (including zero-width end), palette/NO_COLOR, 79 columns,
  past windows, completion collisions, and short scrolled screens. Added repo-now snapshot with a Codex bar at now.
  Fix round 1: addressed Claude's one finding under amended N1 by starting the line at the same
  `body_start.saturating_sub(1)` row as the grid, closing the row-5 gap below `▼`.
  Round-1 files: `crates/tui/src/timeline.rs`, `crates/tui/tests/timeline.rs`, both repo-phase6 snapshots,
  both repo-now snapshots, and `PROGRESS.md`. The owner's PLAN.md amendment was left untouched.
- Changed: `crates/tui/src/theme.rs`, `crates/tui/src/timeline.rs`, `crates/tui/tests/theme.rs`,
  `crates/tui/tests/timeline.rs`, `fixtures/expected/tui/repo-phase6.txt`,
  `fixtures/expected/tui/repo-phase6.styles.txt`, `fixtures/expected/tui/repo-now.txt`,
  `fixtures/expected/tui/repo-now.styles.txt`, `PROGRESS.md`.
- Changed existing tests (exhaustive; all other existing tests untouched):
  - `theme.rs::environment_selection_and_exact_palette_table`: now in RGB/indexed/NO_COLOR expectations;
    its `colors` helper includes now.
  - `theme.rs::every_text_pair_has_wcag_aa_contrast_in_both_palettes`: now included on all existing backgrounds.
  - `timeline.rs::fold_near_end_keeps_length_and_shortens_end_to_now` →
    `fold_near_end_yields_to_full_now_label`: full clock + now takes priority over fold length (N3).
  - `timeline.rs::fold_at_end_keeps_length_when_even_now_does_not_fit` →
    `trailing_fold_with_zero_width_end_keeps_now_label`: now label survives the trailing fold; marker/line at last cell.
  - `timeline.rs::explicit_interior_now_precedes_ticks_and_falls_back_when_end_blocks_it` →
    `explicit_interior_now_precedes_ticks_and_end_clock`: near-end now label shifts left and displaces end clock.
- Fix-round-1 affected tests (in addition to the original existing-test list above):
  - `timeline.rs::fitted_now_line_keeps_background_and_avoids_chrome`: line includes row 5; chrome exclusion updated.
  - `timeline.rs::short_scrolled_screen_limits_now_to_visible_lanes`: row 5 now asserts `┃`.
  - `src/timeline.rs::tests::folded_now_paints_last_fold_cell_and_short_axis_keeps_now_word`: starts at row 5,
    checks the line there and retains a neighbouring fold glyph assertion.
- repo-phase6 cell audit (exhaustive, zero-based **(row,column)**; styles expanded to cells, blanks normalized
  exactly as the snapshot dumper): 20 visible cells changed, all N1–N3. No background or modifier changed.
  - Foreground only, secondary `#9AA1AE` → now `#E58AE5`: (3,107) `1`, (3,108) `0`, (3,109) `:`,
    (3,110) `2`, (3,111) `5`, (3,112) `p`, (3,113) `m`, (3,115) `n`, (3,116) `o`, (3,117) `w`.
    The blank (3,114) remains unchanged in normalized snapshot output.
  - (4,117): `─` / border `#2A2F3A` → `▼` / now `#E58AE5`, screen background `#0E1014`.
  - (5,117): blank → `┃` / now `#E58AE5`, screen background `#0E1014` preserved (fix round 1).
  - (6,117), (7,117): blank → `┃` / now `#E58AE5`, selected background `#262C3A` preserved.
  - (8,117): blank → `┃` / now `#E58AE5`, screen background `#0E1014` preserved.
  - (10,117): blank → `┃` / now `#E58AE5`, child background `#12151B` preserved.
  - (11,117), (12,117), (13,117), (14,117): blank → `┃` / now `#E58AE5`, screen background `#0E1014` preserved.
    (9,117) keeps its existing green `✓`; text/style run splitting changes no other cells.
- Fix-round snapshot delta: repo-phase6 and repo-now each change only (5,117), blank → `┃`,
  foreground reset → `#E58AE5`, background `#0E1014` and modifiers unchanged. All other snapshot cells
  are unchanged from the initial Step 11.1 implementation.
- Verify: Rust PATH set per AGENTS.md before cargo. `cargo test --workspace` → exit 0,
  601 passed / 0 failed / 1 ignored (sum of actual result lines). Actual result output pasted below
  (`cargo test --workspace > /tmp/multi-link-11-r1-tests.log 2>&1`; result lines extracted from that log):
```text
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.92s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.68s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.75s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.67s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.18s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.92s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 81 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, actual result line
  (compile-path lines omitted for W10):
```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.10s
```
  `cargo fmt --all -- --check` → no output, exit 0.
  `fixtures/expected/phase9/check.sh 6` → exit 0, actual output (47 `ok` lines):
```text
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   graph-1.1.json (^9[1-6])
ok   graph-2.1.json (^9[1-6])
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   stderr unreadable: 1
ok   open open 92000001
ok   open open 92000002
ok   open open 92000003
ok   open open 92000004
ok   open open 92000005
ok   open open 92000006
ok   open open 92000007
ok   open open 92000008
ok   open open 92000009
ok   open open 92000010
ok   open open 92000011
ok   open open 92000012
ok   open open 92000013
ok   open open 92000014
ok   open open 94000001
ok   open open 94000002
ok   open open 94000003
ok   open open 94000004
ok   open open 94000006
ok   open open 94000007
ok   open fork 92000001
ok   open fork 92000002
ok   open fork 92000003
ok   open fork 92000004
ok   open fork 92000005
ok   open fork 92000006
ok   open fork 92000007
ok   open fork 92000008
ok   open fork 92000009
ok   open fork 92000010
ok   open fork 92000011
ok   open fork 92000012
ok   open fork 92000013
ok   open fork 92000014
ok   open fork 94000001
ok   open fork 94000002
ok   open fork 94000003
ok   open fork 94000004
ok   open fork 94000006
ok   open fork 94000007
note lanes.json is checked by the 9.5 worker-to-App test, not here
```
  `git diff --name-only -- 'fixtures/expected/*.json' 'fixtures/expected/**/*.json'` → no output;
  no JSON keys or JSON answer files changed. `git diff --check` → no output, exit 0.
- Review: Claude, 2 passes (session coordinator ran this step itself, owner asked 2026-11-10). Round 1: 1 finding
  (`timeline.rs` draw_repo started the line on the first lane row, leaving a gap under `▼`; board 11b draws `┃` on the
  grid row too; PLAN.md N1 amended to match the board). Fixed in round 1; round 2 clean. Diff touches only the step's
  files (plus Claude's N1 wording in PLAN.md). Re-ran Verify: 601 passed / 0 failed / 1 ignored; clippy `-D warnings`
  and `fmt --check` clean; JSON keys unchanged; `check.sh 6` 47/47 `ok`; `repo-now` columns: `▼` row 4, `┃` rows 5–14
  at x=117 except bar rows (6, 9), matching board 11b. Mode: Codex loop, quota `primary 23%, secondary 35%, CODEX: OK`.
  (implementer does not decide the step is done). W10 audit of added lines and new
  snapshots: no personal paths, secrets, real prompts or real session ids; fixture prompts/ids remain synthetic.
  Home and step snapshots unchanged. All edited/new files are in Step 11.1's Files list. Nothing staged or committed.
- Gotchas: Fix-round-1 regression before the production change:
  `cargo test -p multi-link-tui --test timeline fitted_now_line_keeps_background_and_avoids_chrome`
  failed at row 5, `left: " "`, `right: "┃"`; 0 passed / 1 failed / 26 filtered out. Final workspace run passes.
  Initial regression test failed as expected (`─` vs `▼`); initial compile exposed the missing Theme.now
  field before that glyph-only regression run. Tests then exposed expected N3 changes and existing completion marks
  painting over the line; assertions preserve those marks. First clippy run caught unnecessary test parentheses
  (removed). First workspace run failed `invalid_flags_are_clap_errors` with `Os { code: 2, kind: NotFound,
  message: "No such file or directory" }` at `crates/cli/tests/cli.rs:20:10` while other cargo verification commands
  were active; both subsequent sequential workspace runs passed that test. No unrelated CLI file changed.
  The temporary snapshot writer was removed after generating the authorized outputs; comparisons remain immutable.
  Existing untracked `.claude/` untouched. Quota supplied by owner: primary 23%, secondary 35%, CODEX: OK.


### 2026-11-10 — Step 10.2: README setup, real merge, owner confirm — Claude
- Did: Rewrote README "Run" as "Install" (`cargo install --path crates/cli --locked`) plus "Hook setup": `multi-link
  hook check`, its findings and exit codes, and the printed command in place of "merge the JSON"; CLI examples use the
  installed `multi-link`; limitations gain H4 (2-space rewrite, duplicate keys and numbers like `1e5`/`1.50` refused),
  H6 (millisecond race, no lock) and H7 ("current" means that one file). Release 1.6 line updated (owner confirmed
  2026-11-09). `docs/design.md` gains "Phase 10 decisions" (H0–H7, amending the "Hook install" major decision).
  Stop-and-ask: the installed binary (Nov 9 12:40) predated 10.1 (`unrecognized subcommand 'check'`), so the owner got
  one reinstall command, then ran the merge command `hook check` printed from `~/.cargo/bin/multi-link` (before the
  merge: the four `target/release` entries `outdated`, `SessionEnd` `ok`), then started `claude` in this repo, sent one
  message and `/exit`ed. Claude never ran the merge or edited settings.
- Changed: `README.md`, `docs/design.md`, `PROGRESS.md`.
- Verify: `cargo test --workspace` → exit 0, 594 passed / 0 failed / 1 ignored. `cargo clippy --workspace
  --all-targets -- -D warnings` → exit 0. `git status --short fixtures` → empty (every answer key unchanged).
  `git diff --check` clean; W10 scan of added lines (home paths, ids ≥ 9 hex, `local_` ids, tokens) → no matches.
  `~/.cargo/bin/multi-link hook check` → exit 0 (home shortened to ~):
  ```text
File: ~/.claude/settings.json
PostToolUse: ok
PostToolUseFailure: ok
PermissionRequest: ok
Notification: ok
SessionEnd: ok
multi-link hooks in ~/.claude/settings.json are current
  ```
  Backup beside the settings: `~/.claude/settings.json.multi-link-backup-20261110T104151Z-s0XHpw` (mode 644, 2055
  bytes; live file 1995 bytes, mode 644: 60 bytes = 4 paths × 15 characters shorter).
  `multi-link sessions --json --since all` (counts only, W10): 700 sessions; Claude `finished`/`transcript_exit` 244,
  `finished`/`session_end` 2, `idle` 4, `busy` 1, `unknown` 3; Codex 446. The one Claude session started after the
  merge (10:41:51Z): `708143e6`, started 10:42:22Z, last activity 10:42:48Z, in multi-link → `finished` /
  `session_end`, so the merged `SessionEnd` hook ran.
- Review: Claude step (docs only, no code). Owner's answer, relayed by the coordinator: "done".
- Gotchas: A zsh `${PIPESTATUS[0]}` printed empty, so exit codes were re-taken without pipes. The installed binary
  must be reinstalled after a code step before an owner check that uses it.

### 2026-11-09 — Step 10.1: `hook check` and `--merged` — Codex
- Did: Implemented H0–H7 and the Phase 10 critique: one shared snippet; pure source refusal,
  recognition, per-event findings and conservative merge in the hook crate; CLI-only path resolution,
  file checks, mode, UTC timestamp and printed POSIX command. The binary never writes settings.
  Added eight pure settings tests and six CLI tests using temporary directories only, including the
  executed shell command, byte-identical backups, preserved modes/key order/user entries, symlinks,
  absent files, special paths, re-read edits, errors and failed-merge cleanup. The scope test sets
  CLAUDE_CONFIG_DIR to a temporary directory. Enabled workspace serde_json preserve_order only.
- Changed: `Cargo.toml`, `Cargo.lock`, `crates/hook/src/{lib,settings}.rs`,
  `crates/hook/tests/settings.rs`, `crates/cli/src/hook.rs`, `crates/cli/tests/hook_check.rs`, `PROGRESS.md`.
- Verify: Mode: Codex loop. Quota check (`python3 -I tools/codex-quota.py`, run by Claude before the hand-off, exit 0):
  `primary: 12% of 300 min window, secondary: 34% of 10080 min window, CODEX: OK`.
  Rust PATH set per AGENTS.md.
  `cargo test --workspace` → exit 0; summed over all `test result:` lines: 594 passed, 0 failed, 1 ignored
  (the existing ignored Codex calls test); new suites `hook_check` 6 passed, `settings` 8 passed.
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; actual output (home shortened to ~):
  ```text
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on build directory
    Checking multi-link-hook v0.1.0 (~/multi-link/crates/hook)
    Checking multi-link v0.1.0 (~/multi-link/crates/cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.57s
  ```
  `cargo fmt --check` → no output, exit 0.
  `cargo build --release` → exit 0; actual output (home shortened to ~):
  ```text
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
   Compiling multi-link-hook v0.1.0 (~/multi-link/crates/hook)
   Compiling multi-link v0.1.0 (~/multi-link/crates/cli)
    Finished `release` profile [optimized] target(s) in 3.06s
  ```
  Real run: `target/release/multi-link hook check` → exit 1. Read-only; only the file line,
  events and findings are pasted here (paths shortened to ~):
  ```text
File: ~/.claude/settings.json
PostToolUse: ok
PostToolUseFailure: ok
PermissionRequest: ok
Notification: ok
SessionEnd: outdated (binary: ~/.cargo/bin/multi-link -> ~/multi-link/target/release/multi-link)
  ```
  `git diff --exit-code -- fixtures` → no output, exit 0: every existing answer key unchanged.
  `git diff --check` → no output, exit 0. W10 scan of added lines and new step files → no matches.
- Review: Claude, one pass, no fix round (0 blocking findings). Diff touches only the step's files. Re-ran Verify:
  `cargo test --workspace` 594 passed / 0 failed / 1 ignored; clippy `-D warnings` and `fmt --check` clean;
  `git status fixtures` empty (keys unchanged); release `hook check` on real settings → exit 1 with the same six
  lines as above, and `shasum` of `~/.claude/settings.json` identical before and after (read-only confirmed).
  Every Verify case is covered (unit: shapes, refusals incl. `1e400` and `18446744073709551616`, `disableAllHooks`,
  `Stop`, `echo multi-link …`; CLI: sh-run command, modes 0644/0600, symlink, absent, FIFO, unreadable, dangling,
  stub-binary failure, `- space ' $(never) \`never\`` path). Non-blocking notes for 10.2's README: (1) H4 is applied
  literally, so a number written as `1e5` or `1.50` is refused, not only out-of-range ones; (2) `customized` lines
  say "edit multi-link entries and matcher by hand" without naming the key; (3) `disableAllHooks: true` gives exit 1
  with no command; (4) the timestamp in the printed command is generated when `check` runs.
- Gotchas: Initial CLI RED run: four failures because `check` did not exist. Additional RED→GREEN
  regressions preserve pre-existing empty user groups and recognize a bare `~/.cargo/bin/multi-link`
  as outdated rather than missing. Explicit null matchers remain customized. Initial clippy rejected
  a decimal zero permission literal in a test; changed it to `0o0`, then reran all Verify commands.
  The existing ignored Codex test remains ignored (shown above). Real settings were only read;
  the printed command was never executed on them. Customized/unknown entries and disableAllHooks
  require a hand edit; the documented read-to-mv race remains per H6. Step 10.2 is next (Claude).


### 2026-11-09 — Phase 9 close: owner confirm — Claude (coordinator)
- Did: recorded the owner's Step 9.7 answer and closed Phase 9.
- Changed: `docs/phase9-check.md` (Owner check → confirmed), `PLAN.md` (Phase 9 heading marked closed), `PROGRESS.md`.
- Verify: the owner's words after reinstalling, adding `SessionEnd` (settings.json now has it) and viewing
  `multi-link tui --since 24h` and the fixture demo: "looks good". Tests at 2c359d0: 580 passed, 0 failed.
- Review: no code change.
- Gotchas: hand-merge instructions for the hook were too fiddly for the owner; a one-command merge (backup, add only
  the missing entry) worked. The backlog's `hook check` prints such a command.

### 2026-11-08 — Step 9.7: Real check, README, owner confirm — Claude
- Mode: Claude step (no Codex, no worktree; no `codex exec` ran, so no quota check). Docs reviewed by an independent
  adversarial Claude subagent (Review below).
- Did: whole-phase Verify. Built release binaries of 6a2c306, 0d490ce, 531aeeb, fb2da53, d6edf92, 6e82309, 8416429
  (each `git archive` into scratch, own target dir) and HEAD; ran each on real `~` at `--now 2026-11-09T00:34:07Z`,
  `--since all`, counts only, plus a per-commit scratch Rust harness (that commit's core) for Claude state × entrypoint
  × open action. Explained ambiguous 1 → 9, counted R7's real gaps (rollout `task_started` times, plain and `.zst`,
  read-only), rendered step 8.7's real graph (scratch only). Re-shot `docs/img/{home,repo}-phase6-fixture.png` and
  `step-phase8-fixture.png` with HEAD (12-hour clock); added `step-phase9-fixture.png` (graph-2.1). Wrote
  `docs/phase9-check.md`; README: Release 1.6, "States and links" section, limits updated.
- Changed: `docs/phase9-check.md`, `README.md`, `docs/img/home-phase6-fixture.png`, `docs/img/repo-phase6-fixture.png`,
  `docs/img/step-phase8-fixture.png`, `docs/img/step-phase9-fixture.png`, `PROGRESS.md`
- Verify (HEAD a223dc1): `cargo test --workspace` → exit 0, 54 result lines all `ok`, 580 passed, 0 failed, 1 ignored;
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; `git diff --stat 6a2c306 -- fixtures/expected` →
  `phase9/` (7 files) plus `sessions.json`, `overview.json`, `hook/{overview,sessions,sessions-2h}.json`,
  `tui/home-hook{,-full}{,.styles}.txt`, exactly the keys 9.0 listed. Fixture renders: p6 overview, p8 graph, p9 graph
  == keys → `True True True`; HTML text == `home-phase6.txt`, `repo-phase6.txt`, `step-phase8.txt` → 40/40 lines,
  `True` each. Real counts (my summary of the per-commit JSON lines kept in scratch; full tables in the check doc):
  ```text
6a2c306 codex 223 (zst 0) claude unknown/no_signal 232 links 222: unlinked 124 heuristic 69 exact 28 ambiguous 1
531aeeb codex 425 (zst 202)  claude unknown 232          links 222: unlinked 59 heuristic 126 exact 28 ambiguous 9
fb2da53 claude finished/transcript_exit 229, unknown 3   links unchanged from 531aeeb
d6edf92 links 222: unlinked 53 heuristic 116 exact 44 (redirect_file 16) ambiguous 9
a223dc1 links 221: unlinked 47 heuristic 121 (resume_turn 5) exact 44 ambiguous 9
0d490ce == 6a2c306; 6e82309 == 8416429 == d6edf92 (all counts); every exit 0, stderr empty
  ```
  W10 over the committed files: no `/Users/` path, no UUID or real 8-hex id (only commit hashes and synthetic fixture
  ids), no prompt text, no secrets; images are synthetic fixtures. Owner's answer: **waiting** (stop-and-ask).
- Review: one adversarial subagent round (wrote none of it; recomputed counts from scratch, reran `r7.py`, checked
  rules against `crates/`). Nothing serious; every count traced. 10 findings, 8 accepted and fixed: "never resumed in a
  terminal" was false for `process_exit` (code resumes it whatever the entrypoint); 9.1's 65 is 57 heuristic + 8
  ambiguous; the unlinked drop also came from `redirect_file`; the "never a wrong `finished`" guarantee softened to
  "designed to fall back"; kept a caveat that `rollout may be gone` is still wrong for some (3 `cd` cases); `resume
  --last` behaviour marked as seen so far; dates in local time (2026-11-08) with the UTC clock; README gained the
  append-log refusal and R8's full rule. Rejected 2: the two-decimal R7 gaps trace to the links' `gap_ms`, not only to
  `r7.py` (now said so); the 55th "test result" line is a test named `result_without_is_error_…`, so 54 result lines
  is right.
- Gotchas: building several archived commits into one shared `--target-dir` silently reused the first build
  (identical binaries; cargo's fingerprints did not see the changed sources), so each commit needs its own target dir;
  check with `md5`. The hook snippet embeds the path of the binary that prints it, so the owner should print it with
  `~/.cargo/bin/multi-link` after reinstalling. `links` `invocation` is the tool-use id, not command text. On today's
  data no session sits under the notes folder at any commit (the 9.6a session returned to the repo), so R8 shows no
  real count change.

### 2026-11-08 — Step 9.6: Fix rounds from `codex exec resume --last` — Codex
- Did: Implemented R7, mode **Codex loop**. Quota check supplied by the coordinator:
  `primary 75%, secondary 30%, CODEX: OK`. Recognised the flag only in resume options, including
  wrappers and value-taking options; help and prompt occurrences do not select this timing path.
  Added `Rollout::real_turns`, sharing the import predicate with graph details while preserving
  the existing all-turn list used by state handling. Link candidates skip the first real turn,
  use inclusive 0–10 s starts in the same checkout, ignore exact-launch claims, deduplicate each
  run at its earliest eligible gap, and never fall back to creation timing. Observed child ids win;
  a directory move still suppresses timing. A unique candidate carries `ResumeTurn` evidence.
  Added boundary, concurrent-run, import, exact-precedence and recognition tests; named Phase 9
  links/graph tests compare parsed JSON with Claude's keys. Exact panel assertions cover plan,
  both reviews handing fixes off, and the final review with no hand-offs. Renderer rule unchanged.
- Did (Claude review round 1): Fixed both findings in the invocation recogniser. Scan past exec dash
  options and their values to the first positional word; only an unquoted, unredirected `resume`
  selects the resume option scan. Stop at prompts, redirects and `--`. Added all requested value-taking
  options to one helper shared by subcommand, last-run and help scans; equals forms consume no next word.
  Regression tests cover the project's full option-first command, `$PWD` versus an elsewhere `-C`,
  wrappers, quoted resume, prompt/redirect boundaries, every new option and its equals form, option
  values that look like flags/subcommands, and help on either side of the resume subcommand.
  UUID choice: preserved the existing immediate `codex exec resume <uuid>` / `codex resume <uuid>`
  branch and its id evidence; `codex exec -s read-only resume <uuid>` was Exec before review and
  remains Exec. No extension of UUID-resume syntax. Round 1 changed only
  `crates/claude/src/invocation.rs`, `crates/claude/tests/invocation.rs`, and this log.
- Changed: `crates/claude/src/invocation.rs`, `crates/claude/tests/invocation.rs`,
  `crates/codex/src/lib.rs`, `crates/codex/tests/phase6.rs`, `crates/core/src/link.rs`,
  `crates/core/tests/links.rs`, `crates/model/src/lib.rs`, `crates/cli/tests/phase9.rs`,
  `crates/tui/tests/graph.rs`, `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md. Before implementation, `cargo test -p multi-link --test phase9`
  exited 101 with actual output:
  ```text
failures:
    phase9_graph_fix_rounds_match_answer_key
    phase9_links_match_answer_key

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
  ```
  Round 1 regression-first run, `cargo test -p multi-link-claude --test invocation resume_last`
  → exit 101 before the fixes; actual result:
  ```text
failures:
    resume_last_after_exec_options_preserves_directory_and_prompt_boundaries
    resume_last_skips_all_supported_option_values_without_consuming_equals_forms

test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  After the fixes, `cargo test -p multi-link-claude --test invocation` → exit 0:
  ```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
  ```
  First round 1 workspace Verify attempt → exit 101; actual failure:
  ```text
---- sentinels_never_in_output stdout ----
called `Result::unwrap()` on an `Err` value: Os { code: 2, kind: NotFound, message: "No such file or directory" }

failures:
    sentinels_never_in_output

test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.28s
  ```
  This happened at `crates/cli/tests/graph.rs:30` while spawning the CLI, before any output assertion.
  The key checker was also invoking Cargo; overlapping binary rebuilds are the suspected cause.
  After it finished, the binary existed and an isolated full workspace rerun passed without changes.
  Final round 1 `cargo test --workspace` → exit 0, 580 passed / 0 failed / 1 ignored across 54 targets
  (totals summed from result lines). Actual target/result output, omitting individual old passing tests:
  ```text
     Running unittests src/main.rs (target/debug/deps/multi_link-08847752104a7bdc)
test tui::tests::phase9_codex_lane_same_repo_beats_newer_cross_repo ... ok
test tui::tests::phase9_codex_lane_parent_outside_window ... ok
test tui::tests::phase9_codex_lane_untagged_has_no_step ... ok
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
     Running tests/cli.rs (target/debug/deps/cli-99e8775e49a659be)
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.19s
     Running tests/graph.rs (target/debug/deps/graph-8a591d0ed3e9b725)
test sentinels_never_in_output ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.54s
     Running tests/phase9.rs (target/debug/deps/phase9-d050da92124c0d2f)
test phase9_links_match_answer_key ... ok
test phase9_graph_fix_rounds_match_answer_key ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running unittests src/lib.rs (target/debug/deps/multi_link_claude-37ca4495ce1206e7)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/app_record.rs (target/debug/deps/app_record-14488ec49e14100e)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/calls.rs (target/debug/deps/calls-4a8609cb04da380b)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/fixtures.rs (target/debug/deps/fixtures-545ec70c001e488e)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
     Running tests/invocation.rs (target/debug/deps/invocation-69120c3c058e55ba)
test resume_last_after_exec_options_preserves_directory_and_prompt_boundaries ... ok
test resume_last_recognition_obeys_option_and_segment_boundaries ... ok
test resume_last_skips_all_supported_option_values_without_consuming_equals_forms ... ok
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
     Running tests/last_turn.rs (target/debug/deps/last_turn-1af0001944e6c552)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/phase6.rs (target/debug/deps/phase6-262ba0e4a0142f5a)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/phase7.rs (target/debug/deps/phase7-548c379cabf6cea8)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running unittests src/lib.rs (target/debug/deps/multi_link_codex-2b22e6fd27ea8e2c)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/calls.rs (target/debug/deps/calls-5452a130312fc135)
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/compressed.rs (target/debug/deps/compressed-5a94df3ada6149f0)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.68s
     Running tests/fixtures.rs (target/debug/deps/fixtures-4d2173f54dd8b4de)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/phase6.rs (target/debug/deps/phase6-6d75b82e8f960ed4)
test real_turn_order_excludes_imports_before_and_between_resumes ... ok
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running unittests src/lib.rs (target/debug/deps/multi_link_core-a452195a4da16e95)
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.87s
     Running tests/compressed.rs (target/debug/deps/compressed-71c9213e610d9e26)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/graph.rs (target/debug/deps/graph-d9ed96fb16eda702)
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/graph_commit.rs (target/debug/deps/graph_commit-55ee0f30653b8b5a)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
     Running tests/hook.rs (target/debug/deps/hook-c034f80c961b9f68)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.68s
     Running tests/links.rs (target/debug/deps/links-0901c1739e7aa862)
test resume_last_deduplicates_runs_ignores_launch_claims_and_preserves_ambiguity ... ok
test phase9_redirect_cases_match_claudes_keys ... ok
test resume_last_observed_id_wins_and_directory_move_suppresses_timing ... ok
test resume_last_matches_only_later_real_turns_in_ten_second_window ... ok
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
     Running tests/overview.rs (target/debug/deps/overview-13e977653dfe64b1)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.52s
     Running tests/phase6.rs (target/debug/deps/phase6-4e85c5da5a6a761d)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/repo_probe.rs (target/debug/deps/repo_probe-876d1c98833cd571)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
     Running tests/session_end.rs (target/debug/deps/session_end-12bbde3eecf1cfcb)
test phase9_session_end_and_clear_match_claude_keys_on_cold_and_warm_passes ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/sessions.rs (target/debug/deps/sessions-1bd3186c797cc2ce)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
     Running tests/started.rs (target/debug/deps/started-5b4b43f4a847b7f5)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
     Running tests/transcript_exit.rs (target/debug/deps/transcript_exit-a77a141a2f0c06f8)
test phase9_clean_exit_states_and_fresh_open_match_keys_on_cold_and_warm_passes ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
     Running unittests src/lib.rs (target/debug/deps/multi_link_hook-8a546e8bd5fab307)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/hook.rs (target/debug/deps/hook-14fcede7590f0bd8)
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
     Running unittests src/lib.rs (target/debug/deps/multi_link_model-9b7a21c997bc424d)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/calls.rs (target/debug/deps/calls-e1f238057aadea12)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/graph.rs (target/debug/deps/graph-461cb86389d0acb3)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/model.rs (target/debug/deps/model-9bfcd1efda99cd6c)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running unittests src/lib.rs (target/debug/deps/multi_link_plan-1644ecda9bd1d599)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/fixtures.rs (target/debug/deps/fixtures-5e10167c48b03dc8)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/multi_link_tui-2ee3edd1699db90c)
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
     Running tests/graph.rs (target/debug/deps/graph-1db84d80984236b3)
test phase9_plan_and_reviews_show_their_own_handoffs ... ok
test result: ok. 81 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
     Running tests/home.rs (target/debug/deps/home-4d93152f7c1a3add)
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running tests/phase6_shape.rs (target/debug/deps/phase6_shape-bba810f63b64a9ca)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/phase8_shape.rs (target/debug/deps/phase8_shape-e67bcf447fc87aec)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/started.rs (target/debug/deps/started-2c966f5b73e0809d)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/theme.rs (target/debug/deps/theme-c6758922202a4b01)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/timeline.rs (target/debug/deps/timeline-6c17575291aa24f4)
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/view.rs (target/debug/deps/view-bb9d43cfd306927e)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_claude
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_codex
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_hook
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_model
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_plan
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_tui
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, actual final output
  (omitting package-check lines containing machine-local paths):
  ```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 15.18s
  ```
  `cargo fmt --check` → exit 0, no output.
  `fixtures/expected/phase9/check.sh 6` → exit 0, actual output:
  ```text
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   graph-1.1.json (^9[1-6])
ok   graph-2.1.json (^9[1-6])
ok   sessions.json (^9[1-6])
ok   links.json (^9[1-6])
ok   stderr unreadable: 1
ok   open open 92000001
ok   open open 92000002
ok   open open 92000003
ok   open open 92000004
ok   open open 92000005
ok   open open 92000006
ok   open open 92000007
ok   open open 92000008
ok   open open 92000009
ok   open open 92000010
ok   open open 92000011
ok   open open 92000012
ok   open open 92000013
ok   open open 92000014
ok   open open 94000001
ok   open open 94000002
ok   open open 94000003
ok   open open 94000004
ok   open open 94000006
ok   open open 94000007
ok   open fork 92000001
ok   open fork 92000002
ok   open fork 92000003
ok   open fork 92000004
ok   open fork 92000005
ok   open fork 92000006
ok   open fork 92000007
ok   open fork 92000008
ok   open fork 92000009
ok   open fork 92000010
ok   open fork 92000011
ok   open fork 92000012
ok   open fork 92000013
ok   open fork 92000014
ok   open fork 94000001
ok   open fork 94000002
ok   open fork 94000003
ok   open fork 94000004
ok   open fork 94000006
ok   open fork 94000007
note lanes.json is checked by the 9.5 worker-to-App test, not here
  ```
  `git diff --check` → exit 0, no output.
  `git diff --name-only -- fixtures PLAN.md docs` and `git diff --cached --name-only` → no output.
- Review: Claude (step session). Round 1, 2 findings, both fixed: (1) `resume --last` after exec options
  (`codex exec -s … -C … resume --last`, valid Codex syntax, checked with `--help`) was `Exec`; (2) the value-taking
  option list missed `-m/-i/-p`, `--enable/--disable`, `--output-schema` and others from `codex exec [resume] --help`
  (0.164). Round 1 diff re-read: correct; it also makes option-first `--help` help-only, which is right. Claude re-ran
  Verify: `cargo test --workspace --no-fail-fast` → passed 580 failed 0 ignored 1; clippy exit 0; `cargo fmt --check`
  exit 0; `fixtures/expected/phase9/check.sh 6` → exit 0, 47 `ok` lines. Matches Codex's numbers.
  **Resume matching off** (scratch copy, `resume_last` forced `false` in `link.rs`): `cargo test -p multi-link --test
  phase9` → `FAILED. 0 passed; 2 failed` (`phase9_links_match_answer_key`, `phase9_graph_fix_rounds_match_answer_key`);
  core `links resume` → `resume_last_matches_only_later_real_turns_in_ten_second_window`,
  `resume_last_deduplicates_runs_ignores_launch_claims_and_preserves_ambiguity` FAILED.
  **Real run** (release builds of `8416429`+fixtures and of this diff, back to back, `links --since all --now
  2026-11-09T00:26:54Z`, counts only): before 222 hand-offs = 44 exact, 116 heuristic, 53 unlinked, 9 ambiguous; after
  221 = 44 exact, 121 heuristic (5 `resume_turn`), 47 unlinked, 9 ambiguous. Per-invocation diff: 5 `exec/unlinked →
  resume/heuristic/resume_turn`, 5 `exec/exact/redirect_file → resume/exact/redirect_file`, 3 `exec/unlinked →
  resume/unlinked`; the one dropped hand-off is this session's `codex exec -s read-only resume --help` probe. Step 8.7
  graph (`--repo` multi-link `--step 8.7`), before and after identical: plan, implement 21:29:59–21:32:05 (exact),
  review, fix round 1 21:32:27–21:34:18 (exact), review, fix round 2 21:34:47–21:39:42 (exact), verify + commit. So the
  expected result (both fix rounds linked, implement about 2 min) holds, but via 9.3's `redirect_file` (the 8.7 commands
  redirect stdout to a log), not R7; the screenshot predates 9.3. R7 adds the 5 links above.
- Gotchas: No fixture, answer key or check.sh edits, including existing keys; docs needed no wording fix.
  An initial compile caught the import helper shadowing a local variable; renamed it `is_imported_turn`.
  The new panel test initially expected six padding spaces after `from`; the renderer uses three,
  so only the assertion spacing was corrected. All final checks above passed. Pre-existing untracked
  `.claude/` left untouched. Nothing staged, committed or pushed.

### 2026-11-08 — Step 9.5: Codex lane's step from all-time hand-offs — Codex
- Did: Implemented R6, mode **Codex loop**. Quota check output supplied by Claude (the coordinator's step session):
  `primary 68%, secondary 29%, CODEX: OK`. Added `Refresh::codex_steps`, computed from the
  worker's existing all-time pass. Only Codex children with a listed same-repo parent and a tagged
  incoming hand-off count; max `(at, position)` wins. App prefers that map for Codex rows, with
  window `own_step` as fallback; Claude rows retain their existing behavior.
- Changed: `crates/cli/src/tui.rs`, `crates/tui/src/app.rs`, `crates/tui/tests/started.rs`, `PROGRESS.md`.
- Verify: Round 1 rerun (Rust PATH per AGENTS.md):
  `cargo test --workspace` → **570 passed; 0 failed; 1 existing ignored** (summed actual test results), exit 0.
  `cargo clippy --workspace --all-targets -- -D warnings` →
  ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.31s``, exit 0.
  `cargo fmt --check` → no output, exit 0 (after correcting match-arm formatting).
  `fixtures/expected/phase9/check.sh 5` → **46/46 `ok`**, exit 0.
  `git diff --check` and fixture/key diff checks → no output, exit 0; goldens unchanged.
  RED→GREEN: both tagged worker-to-App cases initially returned `[]` instead of `Graph(Some(...))`;
  the untagged case already passed. The map unit test initially returned `{}` instead of
  `{"x": "2.1"}`; the App precedence test initially returned window step `1.3` instead of all-time
  step `1.4`. All now pass. Coverage includes older same-repo over newer cross-repo, untagged and
  missing parents skipped, timestamp ordering, later-position ties, Codex-only entries, App map
  precedence, unchanged Claude selection, and the existing window fallback tests.
- Review: Claude round 1 fixes: `Refresh::clone_box` now clones `codex_steps`;
  both tagged worker-to-App cases assert the expected map entry and their exact same-repo parent
  is absent from every overview group's session rows before pressing `s`, proving the window rule
  cannot supply the expected step. Corrected quota attribution.
  Claude (2026-11-08): round 1 had 3 findings (clone_box dropped `codex_steps`; lane tests did not prove the
  window could not answer; quota attributed to the user), all fixed and re-reviewed clean. Claude re-ran Verify:
  `cargo test --workspace` 570 passed / 0 failed / 1 ignored; clippy and fmt clean; `check.sh 5` 46 `ok` plus the
  `lanes.json` note; `git diff -- fixtures/` empty (keys and goldens unchanged). Committed by Claude.
- Gotchas: All three key lanes are present with `fixture_options`' original 2 h window at
  `2026-11-09T12:00:00Z`; no window change needed. Existing untracked `.claude/` left untouched.

### 2026-11-08 — Step 9.4: `SessionEnd` hook record — Codex
- Did: Implemented R5, mode **Codex loop**. Quota check supplied by user:
  `primary 60%, secondary 28%, CODEX: OK`. Claude's scope amendment resolved the earlier blocker;
  this entry replaces the blocked entry. Added only the own main-transcript maximum turn timestamp
  to the Claude parser, using the existing `is_turn` definition and leaving clean-exit file order intact.
  Hook writer emits exactly one ended line, requires a string reason (unknown strings accepted), forces
  ended-only nullable fields to null, and skips polling. Other kinds omit reason. Reader validates ended
  records fail-soft; only a valid record can clear earlier ended evidence. Latest ended uses `(at, line)`;
  strictly later own main turns or valid hooks clear it, even if an older async line is appended last.
  Session records/process exit still win; session_end beats transcript_exit and hook waiting. Snippets
  print SessionEnd with the same async/timeout shape for install and uninstall.
- Changed: `crates/claude/src/lib.rs`, `crates/claude/tests/last_turn.rs`, `crates/hook/src/lib.rs`,
  `crates/hook/tests/hook.rs`, `crates/core/src/hook.rs`, `crates/core/src/sessions.rs`,
  `crates/core/tests/session_end.rs`, `crates/cli/src/hook.rs`, `crates/cli/tests/cli.rs`, `PROGRESS.md`.
- Verify: All four commands exited 0; actual output excerpts pasted below. Workspace: 565 passed,
  0 failed, 1 existing ignored test. Phase 9: 46 lines, every line ok, including whole sessions/links keys.
  New coverage: precedence with FakeProcessProbe::Gone, record wins, ended over transcript exit/waiting,
  transcript exit over waiting, late async arrival after newer turns/waiting/handoff/ended, warm resume
  invalidating finished/open, equal timestamp boundaries, malformed records, arbitrary reasons, old-id-only
  clear from Claude's keys, main-turn classification (including tool_result), and exact writer/CLI/snippet JSON.
  Initial tests failed on missing ended support and snippet entry before implementation. During verification,
  corrected the synthetic record's timestamp format to epoch milliseconds and qualified the CLI test's
  Timestamp import; final commands below are after those fixes.
- Review: Claude, 2026-11-08. Round 0 (before code): Codex stopped on the Files list; Claude added
  `crates/claude/{src/lib.rs,tests/**}` for `last_turn_at` only (PLAN.md Review log). Round 1 review of
  `git diff`: no findings. Checked: record (so `process_exit`) still wins; `ended` cleared by a strictly later turn
  or any later valid hook record, including a late append on a warm cache; meta, local-command echoes, inherited
  ids and subagent records do not clear it; `reason` rejected on other kinds; snippet shape. Files within the
  amended list. No answer keys changed (`git diff --stat -- fixtures/expected` empty). Claude re-ran Verify: workspace
  565 passed / 0 failed / 1 ignored, clippy exit 0, fmt exit 0, `check.sh 4` exit 0 with all 46 lines `ok`
  (same as below).
- Sandbox PTY `/exit` (Claude, Claude Code 2.1.319, Expect PTY, `env -i HOME USER LOGNAME TERM LANG PATH
  MULTI_LINK_HOME=<scratch>`, snippet from `target/debug/multi-link hook install --scope local` written to
  `~/multi-link-sandbox/repo-a/.claude/settings.local.json`; `~/.claude/settings.json` untouched):
  - Run 1, `/exit` with no prompt: exit 0; hook file `ae233a93….jsonl` holds exactly one line,
    `{"kind":"ended","event":"SessionEnd","reason":"prompt_input_exit",…}`. Claude Code wrote no transcript for a
    prompt-less session, so the reader ignores it (hook records never create a session) — expected.
  - Run 2, one prompt then `/exit`: the CLI is **still logged out** ("Not logged in · Please run /login"), so the
    turn ended in an auth error; not signed in, as instructed. `SessionEnd` **still fired**: one `ended` line,
    `prompt_input_exit`, at 23:33:18.438Z, after the last turn (assistant 23:33:05.221Z). The transcript ends with two
    `cost-state` records and no `/exit` echo. `multi-link sessions --since all --json` (real `~/.claude`, scratch
    hook dir) → `66960480…` `finished` / `session_end`, `record: null`; `open 66960480… --dry-run` → `resume`
    `claude --resume 66960480…` in repo-a. The missing sign-in did not block the check.
Commands run with `export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"`.
`cargo test --workspace` → actual result lines (all test targets and doc-tests, in command order):

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.06s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.67s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.86s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 80 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo clippy --workspace --all-targets -- -D warnings` → exit 0, actual completion line:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.80s
```

`cargo fmt --check` → exit 0, stdout/stderr empty.

`fixtures/expected/phase9/check.sh 4` → exit 0, complete actual output:

```text
ok   sessions.json (^9[1-4])
ok   links.json (^9[1-4])
ok   graph-1.1.json (^9[1-4])
ok   sessions.json (^9[1-4])
ok   links.json (^9[1-4])
ok   stderr unreadable: 1
ok   open open 92000001
ok   open open 92000002
ok   open open 92000003
ok   open open 92000004
ok   open open 92000005
ok   open open 92000006
ok   open open 92000007
ok   open open 92000008
ok   open open 92000009
ok   open open 92000010
ok   open open 92000011
ok   open open 92000012
ok   open open 92000013
ok   open open 92000014
ok   open open 94000001
ok   open open 94000002
ok   open open 94000003
ok   open open 94000004
ok   open open 94000006
ok   open open 94000007
ok   open fork 92000001
ok   open fork 92000002
ok   open fork 92000003
ok   open fork 92000004
ok   open fork 92000005
ok   open fork 92000006
ok   open fork 92000007
ok   open fork 92000008
ok   open fork 92000009
ok   open fork 92000010
ok   open fork 92000011
ok   open fork 92000012
ok   open fork 92000013
ok   open fork 92000014
ok   open fork 94000001
ok   open fork 94000002
ok   open fork 94000003
ok   open fork 94000004
ok   open fork 94000006
ok   open fork 94000007
```

### 2026-11-08 — Step 9.3: Hand-off child from the redirect file — Codex
- Did: Implemented R4, mode **Codex loop**. Quota check: `primary 44%, secondary 25%, CODEX: OK`.
  Recognise the launch segment's `>`, `>>`, `1>`, `&>`, `&>>`, `>|`, glued/spaced targets and optional
  `2>&1`. Require one stdout redirect and a literal target; relative targets refuse earlier cd, pushd
  or subshell independently of the existing heuristic's `dir_moved` flag. Quoted/escaped prompt text
  cannot create a redirect operator; expansions/globs are refused.
  Shared `Launch::redirect_child` opens with O_NONBLOCK, fstats the opened descriptor, refuses non-regular
  files and hard links, checks canonical cwd/temp containment before opening and on the opened descriptor,
  and reads at most 64 KiB. Truncating forms may supply an id from a larger log's bounded prefix; append
  forms refuse oversized files before/after reading. macOS uses descriptor F_GETPATH; Linux uses /proc/self/fd. No validation reopens
  the target pathname. Fixture lookup is root-relative with only fixture cwd/tmp/private/tmp allowed;
  real mode also accepts absolute TMPDIR. Exactly one `session id:` line is required, including invalid
  and repeated lines in the count within the bounded read. Oversized append logs are refused even with
  an id in the initial bytes (round 1 corrected the initial refusal for truncating forms).
  Core reads redirects after task_output and before persisted hook evidence, without caching log content.
  Hook writes the first null record before polling and at most one update, respects stdout/thread/task
  precedence, polls missing/unfinished logs for at most 25 s, and immediately stops on an unsafe redirect.
  Task-output opens in the hook also use O_NONBLOCK plus fstat to avoid blocking on special files.
- Changed: `crates/claude/src/invocation.rs`, `crates/claude/tests/invocation.rs`,
  `crates/model/src/lib.rs`, `crates/hook/src/lib.rs`, `crates/hook/tests/hook.rs`,
  `crates/core/src/link.rs`, `crates/core/src/hook.rs`, `crates/core/tests/links.rs`, `PROGRESS.md`.
- Verify (red → green): Initial core 93-key comparison failed (0 passed, 1 failed); initial hook redirect
  forms test failed (expected 2 records, observed 1). Both passed after implementation. A stricter FIFO
  hook regression then failed with simulated elapsed `25s` vs `0ns`; unsafe files now stop immediately,
  and the FIFO test asserts zero clock advance plus a real one-second receive deadline. Core's FIFO test
  runs the actual links report under the same deadline. Both suites exercise symlinks escaping to home,
  oversized logs and duplicate/invalid id lines; the core also checks exactly 64 KiB remains readable.
  Six separately named hook redirect tests each cover spaced/glued targets, with/without `2>&1`.
  A delayed-id test proves first-null-before-poll, child update at 3 s, and no third record.
  All 14 `93…` cases compare parsed JSON directly with Claude's unchanged key, including gone-log hook
  persistence, missing-log heuristic, background redirect and no-rollout exact evidence.
- Verify: `fixtures/expected/phase9/check.sh 3` → exit 0, actual stdout (only ok lines):
  ```text
ok   sessions.json (^9[1-3])
ok   links.json (^9[1-3])
ok   graph-1.1.json (^9[1-3])
ok   stderr unreadable: 1
ok   open open 92000001
ok   open open 92000002
ok   open open 92000003
ok   open open 92000004
ok   open open 92000005
ok   open open 92000006
ok   open open 92000007
ok   open open 92000008
ok   open open 92000009
ok   open open 92000010
ok   open open 92000011
ok   open open 92000012
ok   open open 92000013
ok   open open 92000014
ok   open fork 92000001
ok   open fork 92000002
ok   open fork 92000003
ok   open fork 92000004
ok   open fork 92000005
ok   open fork 92000006
ok   open fork 92000007
ok   open fork 92000008
ok   open fork 92000009
ok   open fork 92000010
ok   open fork 92000011
ok   open fork 92000012
ok   open fork 92000013
ok   open fork 92000014
  ```
- Verify: `cargo test --workspace` → all 51 result blocks ok. Full output:
  `/tmp/ml93-workspace-final.log`. Actual per-target result lines, in output order:
  ```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.08s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.06s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.64s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.87s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.66s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 80 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  Actual reducer output from those result lines:
  ```text
workspace result blocks: 51
passed: 548 ; failed: 0 ; ignored: 1
non-ok result blocks: 0
  ```
- Verify: `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, no warnings/errors;
  full output `/tmp/ml93-clippy-final.log`, final line:
  ```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.45s
  ```
  `cargo fmt --check` → no output, exit 0; `git diff --check` → no output, exit 0.
  All cargo commands used `export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"` first.
- Verify (real release, counts only): Extracted baseline with `git archive 6a2c306 | tar -x -C
  /tmp/multi-link-93-before` after creating that scratch directory. Built with
  `cargo build --release --manifest-path /tmp/multi-link-93-before/Cargo.toml
  --target-dir /tmp/multi-link-93-target-before` → exit 0, actual final line:
  ```text
    Finished `release` profile [optimized] target(s) in 19.07s
  ```
  After: `cargo build --release --target-dir /tmp/multi-link-93-target-after` → exit 0:
  ```text
    Finished `release` profile [optimized] target(s) in 19.05s
  ```
  `/tmp/ml93-counts.py` ran each release binary's `links --since all --now <same fixed clock> --json`
  with cwd a fresh scratch folder. Stdout was parsed in memory; only aggregate counts were saved to
  `/tmp/ml93-real-counts.log`. Actual output:
  ```text
fixed now: 2026-11-08T23:04:29Z
before exit: 0
before hand-offs: 214
before matches: {"ambiguous": 1, "exact": 28, "heuristic": 66, "unlinked": 119}
before exact evidence: {"resume_arg": 16, "stdout": 10, "thread_ready": 2}
before stderr bytes: 0
after exit: 0
after hand-offs: 214
after matches: {"ambiguous": 9, "exact": 30, "heuristic": 121, "unlinked": 54}
after exact evidence: {"redirect_file": 2, "resume_arg": 16, "stdout": 10, "thread_ready": 2}
after stderr bytes: 0
  ```
  Exact links 28 → 30; both new exact observations are redirect_file. Baseline non-exact classifications
  also predate R1, so their changes are not attributed to this step. No transcripts/ids/content saved.
- Review round 1 (Claude, 2026-11-08): All five findings addressed, with regression tests; re-review pending.
  1. `Launch::stdout_append` preserves the redirect mode. Truncating `>`, `1>`, `&>`, `>|` read only the
     first 64 KiB even for a larger log; `>>` and `&>>` refuse size > 64 KiB both before and after reading.
     New core and hook tests cover all six operators, with an id first and another id beyond the read cap.
     Existing oversized-refusal tests now explicitly exercise append mode.
  2. Canonicalize the target and require allowed-root containment before opening, preserving NotFound
     for hook retries. Keep O_NONBLOCK, fstat and opened-path containment after open. A new test asserts
     the pre-open PermissionDenied error for an outside FIFO and a symlink to /dev/null; a missing target
     still returns NotFound. The post-open unsafe-file error is distinct, so this tests precheck order.
  3. Earlier segments scan all words for unquoted cd/pushd, including help-only segments, while retaining
     the original first-command directory detection (including a quoted builtin name). Relative redirects
     are refused for brace/if, !, time and arbitrary earlier word positions; absolute paths remain usable.
     A narrow unquoted `then` branch introducer is recognised before the existing launch wrappers so the
     same-line if form's absolute redirect is supported. Quoted/escaped arguments to echo do not flag cd.
  4. Opened metadata must have nlink == 1, checked before and after the bounded read. A new fs::hard_link
     test refuses the log with two names, then reads its id after removing the other name.
  5. Restored propagation of non-NotFound task-file open/read/metadata errors, matching the existing
     record_and_write doc comment. A new symlink-loop task-file test requires an error after the first
     null record, with no polling time. The task reader retains O_NONBLOCK and regular-file fstat.
  Red evidence: all six added test functions failed before their fixes (three claude, one core, two hook);
  extra cases caught help-only and quoted-first-command regressions before the directory scan was finalised.
  Initial workspace verification failed in cli `sentinels_never_in_output` at graph.rs:30 with ENOENT
  spawning the debug CLI while the key runner's cargo run rebuilt that same binary. The isolated unchanged
  test passed (1 passed, 0 failed) once the key runner finished; final workspace checks run without a
  concurrent debug CLI build. Initial failure output: /tmp/ml93-r1-workspace.log; isolated rerun:
  /tmp/ml93-r1-graph-rerun.log.
- Verify (round 1 final): `fixtures/expected/phase9/check.sh 3` → exit 0, stdout only these 32 ok lines
  (full capture `/tmp/ml93-r1-keys-final.log`):
  ```text
ok   sessions.json (^9[1-3])
ok   links.json (^9[1-3])
ok   graph-1.1.json (^9[1-3])
ok   stderr unreadable: 1
ok   open open 92000001
ok   open open 92000002
ok   open open 92000003
ok   open open 92000004
ok   open open 92000005
ok   open open 92000006
ok   open open 92000007
ok   open open 92000008
ok   open open 92000009
ok   open open 92000010
ok   open open 92000011
ok   open open 92000012
ok   open open 92000013
ok   open open 92000014
ok   open fork 92000001
ok   open fork 92000002
ok   open fork 92000003
ok   open fork 92000004
ok   open fork 92000005
ok   open fork 92000006
ok   open fork 92000007
ok   open fork 92000008
ok   open fork 92000009
ok   open fork 92000010
ok   open fork 92000011
ok   open fork 92000012
ok   open fork 92000013
ok   open fork 92000014
  ```
  `cargo test --workspace` → exit 0, full output `/tmp/ml93-r1-workspace-final.log`;
  actual reducer output (summed from every per-target result line):
  ```text
cargo test --workspace exit: 0
workspace result blocks: 51
passed: 554; failed: 0; ignored: 1
non-ok result blocks: 0
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, no warnings/errors;
  full output `/tmp/ml93-r1-clippy-final.log`, actual final line:
  ```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.52s
  ```
  `cargo fmt --check` → exit 0, no output (`/tmp/ml93-r1-fmt-final.log`).
  `git diff --check` → exit 0, no output. All cargo commands use the required Rust PATH export.
  Round 1 changed only `crates/claude/src/invocation.rs`, `crates/claude/tests/invocation.rs`,
  `crates/core/tests/links.rs`, `crates/hook/src/lib.rs`, `crates/hook/tests/hook.rs`, `PROGRESS.md`.
- Verify (round 1 real release, counts only): Rebuilt the completed source with
  `cargo build --release --target-dir /tmp/multi-link-93-target-after` → exit 0, actual final line:
  ```text
    Finished `release` profile [optimized] target(s) in 5.18s
  ```
  Ran `/tmp/ml93-counts.py` again, using the archived `6a2c306` release baseline and the new release
  binary from fresh scratch cwd, same fixed `--now`, `links --since all --json`; only counts retained.
  Actual output `/tmp/ml93-r1-real-counts-final.log`:
  ```text
fixed now: 2026-11-08T23:15:45Z
before exit: 0
before hand-offs: 215
before matches: {"ambiguous": 1, "exact": 28, "heuristic": 66, "unlinked": 120}
before exact evidence: {"resume_arg": 16, "stdout": 10, "thread_ready": 2}
before stderr bytes: 0
after exit: 0
after hand-offs: 215
after matches: {"ambiguous": 9, "exact": 38, "heuristic": 116, "unlinked": 52}
after exact evidence: {"redirect_file": 10, "resume_arg": 16, "stdout": 10, "thread_ready": 2}
after stderr bytes: 0
  ```
  Exact links 28 → 38 (10 redirect_file); initial implementation's earlier 28 → 30 run remains above
  as historical evidence, not the round 1 result. No agent-data writes or transcript content retained.
- Review: Claude (2026-11-08), 1 round, 5 findings, all fixed and re-reviewed clean: (1) oversized log was refused;
  R4 says cap at 64 KiB, so truncating forms read the prefix while `>>`/`&>>` over 64 KiB stay refused (an older
  run's id alone in the prefix would be a wrong exact); (2) open came before containment, so `/dev/…` could be
  opened: canonical pre-check added, opened-fd check kept; (3) `cd` after `{`, `if`, `!`, `time` was missed: any
  earlier unquoted `cd`/`pushd` word now counts; (4) hard links could pass F_GETPATH containment: `nlink == 1`;
  (5) hook doc comment vs error handling. Claude re-ran Verify: `check.sh 3` → 32 `ok`, 0 other lines;
  `cargo test --workspace` → 554 passed, 0 failed, 1 ignored; clippy `-D warnings` and `fmt --check` clean;
  `git diff --stat -- fixtures PLAN.md docs` empty. Matches the pasted output. Files all inside 9.3's list.
  Codex's own notes: No answer keys or PLAN.md edits;
  no staging, commits, agent-data writes or network calls from project code. Existing untracked
  `.claude/` was present before this step and was left untouched.
- Gotchas: Old logs may already be gone; only persisted hook evidence can retain exactness then.
  Unsafe redirect failures are silent/no evidence; missing files and readable logs without one unique
  id retain bounded polling. No settings snippet change is needed for R4. An initial focused run caught
  subshell handling changing the old heuristic flag; the redirect restriction is now tracked separately,
  and the original invocation tests pass unchanged.

### 2026-11-08 — Step 9.2: Clean exit from the transcript; reopen by where it ran — Codex
- Did: Implemented R2 using own main-transcript file order, with meta/local-command echoes excluded from
  turns and tool results included. Inherited and subagent records do not set or clear the exit marker.
  Last own string entrypoint is retained even on an unknown record (the record is still skipped/counted).
  Session records retain precedence, including fixture-mode records; transcript exit precedes hook waiting.
  Implemented R3 through cached internal reopening metadata for CLI and the TUI fresh-action path:
  CLI/sdk resume, desktop uses exactly one safe app record, desktop fork is unavailable, unknown entrypoint
  is unavailable, and process_exit retains its existing resume rule. No Session fields added.
  Added one parser test and three core integration tests, covering every 92 fixture on cold/warm passes,
  SIGKILL with Gone, entrypoint changes, mapping addition/removal, duplicate mappings and unsafe hosts.
  Updated the five owned tests via CLI expectations, the core fixture table and the two guard snapshots.
- Changed: `crates/claude/src/lib.rs`, `crates/model/src/lib.rs`,
  `crates/core/src/{open,scanner,sessions}.rs`, `crates/core/tests/transcript_exit.rs`,
  `crates/core/tests/{overview-before-phase5,keys-before-phase6}.json`,
  `crates/cli/src/main.rs`, `crates/cli/tests/cli.rs`, `PROGRESS.md`.
- Verify: All cargo commands below used
  `export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"` first.
  Initial `cargo test -p multi-link-core --test transcript_exit` failed on 92000001's
  `unknown` / `no_signal` versus the key's `finished` / `transcript_exit` (1 passed, 1 failed).
  A separate parser regression failed with `Some("cli")` versus `Some("sdk-cli")` for an unknown
  own record's later entrypoint; moving entrypoint extraction before unknown-type skipping fixed it.
  Final `fixtures/expected/phase9/check.sh 2` → exit 0, actual output:
  ```text
  ok   sessions.json (^9[1-2])
  ok   links.json (^9[1-2])
  ok   graph-1.1.json (^9[1-2])
  ok   stderr unreadable: 1
  ok   open open 92000001
  ok   open open 92000002
  ok   open open 92000003
  ok   open open 92000004
  ok   open open 92000005
  ok   open open 92000006
  ok   open open 92000007
  ok   open open 92000008
  ok   open open 92000009
  ok   open open 92000010
  ok   open open 92000011
  ok   open open 92000012
  ok   open open 92000013
  ok   open open 92000014
  ok   open fork 92000001
  ok   open fork 92000002
  ok   open fork 92000003
  ok   open fork 92000004
  ok   open fork 92000005
  ok   open fork 92000006
  ok   open fork 92000007
  ok   open fork 92000008
  ok   open fork 92000009
  ok   open fork 92000010
  ok   open fork 92000011
  ok   open fork 92000012
  ok   open fork 92000013
  ok   open fork 92000014
  ```
  `cargo test --workspace` → exit 0; all 51 target summaries, actual output (537 passed, 0 failed,
  1 ignored in total):
  ```text
  test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
  test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.60s
  test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.53s
  test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
  test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
  test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
  test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
  test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
  test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s
  test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.68s
  test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
  test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
  test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.87s
  test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
  test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
  test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s
  test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
  test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
  test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
  test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
  test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
  test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
  test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
  test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
  test result: ok. 80 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
  test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
  test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
  test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
  test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
  test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, final output:
  ```text
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.84s
  ```
  `cargo fmt --check` → no output, exit 0. `git diff --check` → no output, exit 0.
  `python3 /tmp/multi-link-92-audit-snapshots.py` → actual output:
  ```text
  overview-before-phase5.json: authorized semantic changes only: {'state': 24, 'state_evidence': 24, 'tier': 1, 'repo-c row reorder': 1}
  keys-before-phase6.json: authorized semantic changes only: {'state': 57, 'state_evidence': 57, 'tier': 1, 'repo-c row reorder': 1, 'window removal': 1}
  ```
  The audit compares parsed snapshots to HEAD, permits changes only for the 12 listed ids and the listed
  hook repo-c tier/order and window removal, and rejects all other changes. The guard tests also pass.
- Verify (real run, counts only): Baseline extracted without changing this checkout:
  `mkdir -p /tmp/multi-link-92-before` then `git archive 531aeeb | tar -x -C /tmp/multi-link-92-before`.
  Built with `cargo build --release --manifest-path /tmp/multi-link-92-before/Cargo.toml
  --target-dir /tmp/multi-link-92-target-before` (exit 0); after build:
  `cargo build --release --target-dir /tmp/multi-link-92-target-after` (exit 0).
  Scratch Rust harnesses at `/tmp/multi-link-92-counts/{before,after}/` depend on the respective core/model
  crates and serde_json, call `Scanner::sessions` with real repo mapping and `RealProcessProbe`,
  `since: None`, fixed `now: 2026-11-08T22:48:58Z`, and compute open actions without executing them.
  Before uses `open::action`; after uses `Scanner::open_resolver(...).action`. Entrypoints are read
  from own main-transcript records. Only aggregate counts are printed, with no ids, paths or content.
  Commands: `cargo run --release --manifest-path /tmp/multi-link-92-counts/before/Cargo.toml
  --target-dir /tmp/multi-link-92-target-before` and the same command with both `before` replaced by `after`;
  both exit 0. Output files: `/tmp/multi-link-92-counts-{before,after}.json`; aggregation saved to
  `/tmp/multi-link-92-real-counts.txt`, actual output:
  ```text
  before: {"claude": 241, "open_actions": {"app_link": 10, "unavailable": 231}, "states": {"other": 10, "unknown": 231}, "unknown_by_entrypoint": {"claude-desktop": 195, "cli": 27, "missing": 4, "sdk-cli": 5}}
    {"action": "app_link", "count": 10, "entrypoint": "claude-desktop", "reason": "", "state": "other"}
    {"action": "unavailable", "count": 195, "entrypoint": "claude-desktop", "reason": "not_known_finished", "state": "unknown"}
    {"action": "unavailable", "count": 27, "entrypoint": "cli", "reason": "not_known_finished", "state": "unknown"}
    {"action": "unavailable", "count": 4, "entrypoint": "missing", "reason": "not_known_finished", "state": "unknown"}
    {"action": "unavailable", "count": 5, "entrypoint": "sdk-cli", "reason": "not_known_finished", "state": "unknown"}
  after: {"claude": 241, "open_actions": {"app_link": 133, "resume": 30, "unavailable": 78}, "states": {"finished": 228, "other": 10, "unknown": 3}, "unknown_by_entrypoint": {"cli": 2, "missing": 1}}
    {"action": "app_link", "count": 123, "entrypoint": "claude-desktop", "reason": "", "state": "finished"}
    {"action": "unavailable", "count": 72, "entrypoint": "claude-desktop", "reason": "app_session", "state": "finished"}
    {"action": "resume", "count": 25, "entrypoint": "cli", "reason": "", "state": "finished"}
    {"action": "unavailable", "count": 3, "entrypoint": "missing", "reason": "unknown_entrypoint", "state": "finished"}
    {"action": "resume", "count": 5, "entrypoint": "sdk-cli", "reason": "", "state": "finished"}
    {"action": "app_link", "count": 10, "entrypoint": "claude-desktop", "reason": "", "state": "other"}
    {"action": "unavailable", "count": 2, "entrypoint": "cli", "reason": "not_known_finished", "state": "unknown"}
    {"action": "unavailable", "count": 1, "entrypoint": "missing", "reason": "not_known_finished", "state": "unknown"}
  ```
  Release CLI cross-check: `/tmp/multi-link-92-target-after/release/multi-link sessions --since all
  --now 2026-11-08T22:48:58Z --json` piped directly to a counts-only Python reducer → exit 0,
  actual output `{"finished": 228, "idle": 10, "unknown": 3}`; stderr 0 bytes.
  No app/terminal action executed. Agent directories were read only; all builds and helper files are in scratch.
  Counts differ from Phase 9's historical 227 unknown because real sessions have accumulated; both measured
  populations contain 241 Claude sessions. The 10 other-state desktop sessions keep their app-link behavior.
- Review: Claude, round 1, no fix round needed. Code follows R2 (own main-record order; meta and
  local-command echoes are not turns; `tool_result` is) and R3 (entrypoint + app mapping via an internal
  resolver, no Session key; CLI and TUI both use it through `fresh_action` / `open_resolver`). Claude diffed both
  guard snapshots against HEAD with its own parsed-JSON script: the only changes are the 12 listed ids'
  state/evidence, hook repo-c tier and row order, and 11c1f13c gone from `hook/sessions-2h.json`, as listed.
  No answer key or PLAN.md touched. Claude re-ran Verify: `check.sh 2` all 32 lines `ok`; `cargo test --workspace`
  537 passed, 0 failed, 1 ignored; clippy and fmt clean. Same as pasted. Notes, not defects: `SessionEnd` joins the
  enum early (R0 lists it; 9.4 produces it); the resolver-free `open::action` wrapper would say
  `unknown_entrypoint` for a clean exit, so new callers must use `Scanner::open_resolver`.
- Gotchas: `SessionEnd` is recognized as an exit-evidence enum/open decision input, but reading ended hook
  records remains Step 9.4. A desktop session without exactly one safe mapping stays unavailable, with the
  manual command shown. The conservative `open::action` wrapper has no reopening metadata; production CLI
  and TUI paths use the resolver. All real counts are observations, not committed fixtures.

### 2026-11-08 — Step 9.1: Read compressed Codex rollouts — Codex
- Did: Implemented R1 in mode **Codex loop**. Quota check output (supplied for this run):
  `CODEX: OK 27%/23%`. Added pure-Rust `ruzstd` 0.8.3 and one shared Codex opener for both discovery
  paths, single-file scan, calls and turns. Compressed frames are fully decoded and checksum-checked
  before parsing, with a combined 64 MiB decompressed cap (including concatenated frames); plain input
  remains streamed and uncapped. Plain counterparts take precedence before reading; parsed metadata ids
  also prefer plain copies with different names. The core scanner uses the same discovery helper and
  retains its file-key cache. Added eight tests for fixtures/graph, cap boundaries, corruption, duplicate
  preference, and cache reuse/invalidation. PLAN.md, docs and all answer keys are unchanged.
- Changed: `Cargo.toml`, `Cargo.lock`, `crates/codex/Cargo.toml`, `crates/codex/src/lib.rs`,
  `crates/codex/tests/compressed.rs`, `crates/core/src/scanner.rs`, `crates/core/tests/compressed.rs`,
  `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md. Before implementation, the two initial compressed regression
  tests failed (missing compressed session and no parsed metadata); `check.sh 1` failed sessions, graph
  and stderr while links already passed. Final `cargo test -p multi-link-codex -p multi-link-core --test compressed`
  → exit 0, actual result lines (Codex, then core):
  ```text
  test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.77s
  test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
  ```
- Verify: `fixtures/expected/phase9/check.sh 1` from the repo root → exit 0, actual output:
  ```text
  ok   sessions.json (^9[1-1])
  ok   links.json (^9[1-1])
  ok   graph-1.1.json (^9[1-1])
  ok   stderr unreadable: 1
  ```
- Verify: `cargo test --workspace --no-fail-fast` ran every target. The actual output ends with:
  ```text
  error: 12 targets failed:
      `-p multi-link --test cli`
      `-p multi-link-claude --test fixtures`
      `-p multi-link-codex --test fixtures`
      `-p multi-link-core --lib`
      `-p multi-link-core --test hook`
      `-p multi-link-core --test overview`
      `-p multi-link-core --test phase6`
      `-p multi-link-core --test sessions`
      `-p multi-link-model --test model`
      `-p multi-link-tui --lib`
      `-p multi-link-tui --test home`
      `-p multi-link-tui --test theme`
  ```
  Summing its per-target result lines: 488 passed, 45 failed, 1 ignored. Audited every failing name
  against Current state's 45 (including its 16 `app::graph_tests::*` cases): **newly failing tests: none**.
  Reran `cargo test --workspace --` with one `--skip <full test name>` for each of those 45;
  actual audit/rerun output:
  ```text
  Known failures: 45; newly failing tests: none
  Outside the documented 45: 488 passed; 0 failed; 1 ignored; 45 filtered out
  cargo test exit: 0
  ```
  Full workspace output, exact failure list and clean rerun output are in
  `/tmp/multi-link-91-{workspace,failure-audit,unblocked}.log` (outside git).
- Verify: `cargo fmt --all` and `cargo fmt --all -- --check` → no output, exit 0.
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, actual final line:
  ```text
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.60s
  ```
  `cargo build --release -p multi-link` → exit 0, actual final line:
  ```text
      Finished `release` profile [optimized] target(s) in 0.12s
  ```
- Verify: Copied the release binary into a fresh `$TMPDIR/multi-link-91-real-*` folder outside the repo,
  then ran `./multi-link sessions --since all --json` and `./multi-link links --since all --json` with
  that folder as cwd. Parsed stdout in memory and saved only counts and stderr to its `counts.txt`
  (also `/tmp/multi-link-91-real-counts.log`). Actual output:
  ```text
  sessions exit: 0
  Codex sessions: 419
  sessions stderr: (empty)
  links exit: 0
  Hand-offs: 212
  unlinked: 54; heuristic: 121; exact: 28; ambiguous: 9
  links stderr: (empty)
  ```
  Before at `0d490ce` (measured by Claude with a release build of that commit): 216 Codex sessions; 211 hand-offs =
  119 unlinked, 63 heuristic, 28 exact, 1 ambiguous; stderr empty. These are live-data measurements
  at different times: the after run has one additional hand-off. No transcript content was saved in git.
- Review: Claude (2026-11-08), no fix rounds. Diff stays inside 9.1's Files (`graph.rs` needed no change:
  `read_details` itself now goes through `open_rollout`). No key, doc or PLAN.md change (`git diff --stat HEAD --
  fixtures docs PLAN.md` empty). Re-ran Verify: `check.sh 1` → the same four `ok` lines, exit 0; clippy and
  `fmt --check` exit 0; `cargo test --workspace --no-fail-fast` → 488 passed, 45 failed, 1 ignored, and every one of
  the 45 failing names is in Current state's list (16 `app::graph_tests::*`). Notes, not defects: path-based
  plain preference drops a `.zst` twin before reading, so a corrupt twin of a plain file is not counted
  `unreadable` (R1 says read the plain one); a multi-frame stream with a skippable frame is untested.
- Gotchas: Real data: ambiguous hand-offs rose 1 → 9 now that ~200 more rollouts are candidates for the 60 s
  creation match; 9.7's count should look at them (likely parallel runs in one checkout). The workspace remains red only for the 45 expected 9.2 failures; no keys were changed to
  hide them. A corrupt/oversize compressed file yields no partial rollout or graph details. As with
  existing scanner readers, failed reads invalidate cache entries and are retried on later passes.

### 2026-11-08 — Step 9.0: Rules, fixtures and answer keys — Claude
- Did: Wrote R0–R6 into `docs/design.md` ("Phase 9 decisions", decision 2's Exact sources gain the redirect file, the
  Finished state row, Opening, the open question on finding a finished desktop session's `local_` id, the timeline)
  and the contract into `docs/output-schema.md` (Phase 9 changes; State rows and the Turn definition; Exact evidence
  4b; hook `ended` / `reason` / `redirect_file`; Ended reader rules; Open rows and reasons; hook and overview variant
  notes; "Phase 9 fixture results" with the R1 rule and a case table). Built the synthetic root
  `fixtures/sessions-phase9/` (82 files; a scratch generator, not committed; `.zst` made with
  `/opt/homebrew/bin/zstd --no-check`, one frame like the real ones, whose structure only was read). Wrote the keys
  in `fixtures/expected/phase9/` and `check.sh`. Updated the five existing keys and four hook goldens R2 changes.
  How the keys were made: fields R1–R6 do not touch come from the current build run on a decompressed copy of the
  root; every Phase 9 value is set by hand in a scratch script, one line per value with its rule (sessions: 9 R2
  values, 2 R5 values, `skipped: []`; links: 6 `redirect_file`, 1 `hook`; open: written out per case). For the
  existing keys, which sessions flip was decided by an independent Python R2 checker (12 of the `fixtures/sessions`
  transcripts, 0 elsewhere); the mechanical consequences (row order by state rank, repo-c's tier, 11c1f13c leaving the
  2 h window, golden renders) came from a throwaway scratch "oracle" (the 12 ids forced to `transcript_exit` in a
  copy of the code; never committed), and every resulting diff was checked to be only `state`, `state_evidence`,
  `state_since`, `tier`, row order and that one removal. Edits keep each key's hand layout (text-level, minimal diff).
  The oracle run also found the 5 tests that need edits in 9.2 (Current state).
- Changed: `docs/design.md`, `docs/output-schema.md`, `fixtures/sessions-phase9/**` (new),
  `fixtures/expected/phase9/{sessions,links,open,graph-1.1,lanes}.json`, `fixtures/expected/phase9/check.sh` (new),
  `fixtures/expected/{sessions,overview}.json`, `fixtures/expected/hook/{sessions,sessions-2h,overview}.json`,
  `fixtures/expected/tui/home-hook{,-full}{,.styles}.txt`, `PROGRESS.md`
- Verify: `cargo test --workspace --no-fail-fast` → `passed 476 failed 45 ignored 1`; the 45 failing tests are exactly
  the list in Current state (compared by name, before and after adding the Phase 9 root: "same failing set"); 3 of them
  fail with ``unknown variant `transcript_exit` `` directly, the rest on an assertion or an untagged `Row` that no
  longer deserialises. W10: `rg -l "/Users/|\bsk-|api_key" fixtures --glob '!raw/**' --glob '!README.md'` → prints
  nothing (exit 1); no scratch path, user name or email in the new files. Every later Verify names a key that exists:
  9.1 `sessions.json` + `graph-1.1.json`, 9.2 `sessions.json` + `open.json`, 9.3 `links.json`, 9.4 `sessions.json`,
  9.5 `lanes.json` (all present). `fixtures/expected/phase9/check.sh 5` on today's code → `ok` for every unchanged
  case (`links` at 9.1, the `open` rows of 92000007/09/10/12/13/14 and 94000003/04/06/07), `FAIL` for every Phase 9 case, as
  expected before any code.
- Review: Codex read-only review (`codex exec -s read-only`, quota `CODEX: OK` 20%/22%): it confirmed the existing-key
  changes are exactly the 12 qualifying transcripts and that the `.zst` fixtures decode (corrupt one fails). 3 findings,
  all accepted and fixed: (1) `check.sh` ignored the CLI's exit status (process substitution / `|| true`) → each
  command's output now goes to a temp file and a non-zero exit is a FAIL, the stderr check too; (2) both resumed cases
  had a later `assistant`, so an assistant-only turn test would pass → added `92000013` (user prompt only) and
  `92000014` (`tool_result` only) after the `cost-state`, both `unknown` / `no_signal` / `not_known_finished`;
  (3) no case had a hook record after `ended` → added `94000007` (`ended` then `waiting`, no `cost-state`):
  `waiting` / `hook` / `running_without_app_link`. Re-ran Verify after the fixes: unchanged (476 / 45 / 1, same failing
  set; W10 clean); `check.sh 5` → `ok` for the unchanged cases (incl. the three new ones), FAIL for every Phase 9 case.
- Gotchas: (1) The plan's literal Verify (`diff <(jq -S . key) …`) cannot hold for a single shared root until the
  last step, so keys are final-state and each step compares its own and earlier id prefixes (`check.sh`); the full
  diff holds from 9.4. (2) The `keys-before-phase6.json` / `overview-before-phase5.json` snapshot guards in
  `crates/core/tests/` were outside 9.0's Files; 9.2 updates only the listed values there. (3) 11c1f13c was the hook
  variant's only `waiting`/`hook` row; R2 makes it `transcript_exit` (correct under R5's order), so the Phase 9 root
  adds 92000012 (still waiting) and 92000011 (exit beats waiting). (4) While 9.0 ran, the coordinator committed R7 /
  Step 9.6 (580eb3f, PLAN.md only) and renumbered the real check to 9.7; on the coordinator's request a follow-up
  docs-only commit adds R7 to the Phase 9 decisions table, `resume_turn` to R0, decision 2 and the Link `evidence`
  row (the only value allowed on a `heuristic` link), and the resume-`--last` candidate rule to the schema. No R7
  fixtures or keys (Step 9.6 writes those). (5) Fixture `92000010`'s id ends `…9200000a` (hex), so its file name differs from its case number.

### 2026-11-08 — Step 9.6a: A Claude session's folder ignores moves outside it — Codex
- Did: Implemented amended R8: the first own known-record string cwd sets home, using the parser's existing
  activity_id ownership test. Before home is set all known records update metadata as before. Afterward,
  whole-component path comparison accepts home/descendants without filesystem access; accepted/no-cwd records
  update branch. Per-call checkouts remain unchanged. Tests cover all five original cases, returning home,
  outside movement after a worktree, and outside hand-off checkout. Fix round 1 adds inherited→own tests for
  matching and absent sessionId, an own record without cwd before home, and an inherited-only file retaining
  the latest cwd/branch. Updated the existing synthetic unknown-record test to use a descendant folder.
- Changed: `crates/claude/src/lib.rs`, `crates/claude/tests/fixtures.rs`, `PROGRESS.md`.
- Verify: Rust PATH exported per AGENTS.md. Committed fixtures used; fixtures/raw absent. Fix-round tests
  were run before the correction: `cargo test -p multi-link-claude --lib` → exit 101, actual output excerpts:
  ```text
  test tests::inherited_only_file_keeps_newest_folder_and_branch ... FAILED
  test tests::inherited_folders_do_not_set_the_own_session_home ... FAILED
  test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  They returned `/r/old` instead of `/r/new` and `/r/repo`, respectively. After correction,
  `cargo test --workspace` → exit 0; 525 passed, 0 failed, 1 ignored across 48 targets. Actual result lines:
  ```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.21s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.54s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.54s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 80 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0:
  ```text
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.95s
  ```
  `cargo fmt --all -- --check` → no output, exit 0.
  `cargo build --release` → exit 0:
  ```text
      Finished `release` profile [optimized] target(s) in 4.64s
  ```
  `target/release/multi-link overview --json --since 2h` → exit 0; JSON kept only in scratch.
  Only the two requested checks extracted (checkout home prefix rendered as `~`):
  ```text
repos under side-session-notes: 0
session c205b4c0 checkout: ~/multi-link/.claude/worktrees/step-9.6a
  ```
  `git diff --name-only -- fixtures/expected` and `git diff --check` → no output, exit 0.
- Review: Claude (Codex loop; `codex-quota.py` → `CODEX: OK`, 5-hour 16%, weekly 21%). Round 1 stopped correctly:
  R8 as first written moved resumed session `85a715ba…` to repo-f; Claude amended R8 (home from the session's own
  records) and Codex fixed it. Diff matches R8, nothing outside Files. Claude re-ran Verify: 525 passed, 0 failed,
  1 ignored; clippy clean; no fixture or key changed; real run 0 repos under `side-session-notes`, `c205b4c0` in
  this worktree. Not fixed here: Claude Code's `gitBranch` reads `HEAD` on 106 of this session's 226 records with
  `cwd` (in the main checkout too), so its branch cell shows `HEAD`; U6's place word already avoids `gitBranch`.
  Worked on branch `step/9.6a` in a worktree beside Phase 9; the coordinator merges it.
- Gotchas: A resumed file's inherited records cannot establish home, but still update metadata until an
  own record with cwd does. Files containing only inherited records keep the previous newest-record behavior.
  PLAN.md was already modified by Claude and remains untouched by Codex. This replaces the original step
  log entry and blocked Current state; no second entry was added.

### 2026-11-08 — Phase 8 close: owner confirm — Claude (coordinator)
- Did: recorded the owner's Step 8.6 answer and closed Phase 8.
- Changed: `docs/phase8-check.md` (Owner check → confirmed), `PLAN.md` (Phase 8 heading marked closed), `PROGRESS.md`.
- Verify: the owner's words, after reinstalling (`cargo install --path crates/cli --force`) and running the TUI:
  "it looks good", then "Yes, close Phase 8". Tests at HEAD 956d965: 521 passed, 0 failed; clippy clean.
- Review: no code change.
- Gotchas: `~/.cargo/bin` is not on the owner's terminal PATH (no zsh startup file adds it); run
  `~/.cargo/bin/multi-link` or add it to `~/.zshrc`.

### 2026-11-08 — Step 8.7: TUI: 12-hour clock on screen — Codex
- Did: Shared `view::clock_label` drives view/repo/graph clocks, with lowercase suffixes and optional seconds;
  preserved date/relative-time rules. Graph strip reserves the actual clock width. Updated docs and test expectations.
  Regenerated all 8 TUI text/style pairs from renderer output; removed temporary writers and kept compare logic.
- Changed: `crates/tui/src/{view,timeline,graph,app}.rs` (app test literals only),
  `crates/tui/tests/{view,timeline,graph,started}.rs`, `fixtures/expected/tui/*`, `docs/design.md`, `README.md`, `PROGRESS.md`.
- Verify: Required Rust PATH exported. `cargo test --workspace` → exit 0; 521 passed, 0 failed across 48 suites.
  Per-suite passed counts (all `test result: ok`): CLI main 15, cli 23, graph 7;
  Claude lib 1, app_record 11, calls 5, fixtures 9, invocation 8, phase6 2, phase7 3;
  Codex lib 0, calls 7, fixtures 9, phase6 9; core lib 48, graph 51, graph_commit 1, hook 6, links 16,
  overview 5, phase6 7, repo_probe 11, sessions 13, started 13; hook lib 0, hook 14;
  model lib 0, calls 3, graph 2, model 9; plan lib 0, fixtures 9;
  TUI lib 52, graph 80, home 18, phase6_shape 3, phase8_shape 5, started 10, theme 5, timeline 22, view 9;
  doc-tests 0 each for claude/codex/core/hook/model/plan/tui. Offset, seconds and exact 24 h cut-over tests pass.
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, Finished dev profile; no warnings.
  `grep -hoE '[0-9]{1,2}:[0-9]{2}(:[0-9]{2})?([ap]m)?' fixtures/expected/tui/*.txt | grep -vE '[ap]m$'`
  → no output, exit 1 (no 24-hour clocks).
  `git diff ddd91f8 -- fixtures/expected/tui | grep '^[-+] ' | grep -vE '[0-9]{1,2}:[0-9]{2}'`
  → no output, exit 1 (only time-bearing text lines changed).
  `git diff --exit-code ddd91f8 -- fixtures/expected ':!fixtures/expected/tui' crates/cli crates/core crates/model`
  → no output, exit 0. Additional style-row audit: every changed style row belongs to a time-bearing text row.
  `git diff --check` → no output, exit 0. Temporary renderer-write gate search → no matches.
- Review: Claude, 1 round, 0 findings to fix. Read the whole diff: one helper, no 24-hour format left in
  `crates/tui/src`; the graph strip end label now right-aligns by its real width (it was a fixed 5). 7.3 axis tests: the
  only expectation changes drop ticks the wider end label overlaps (P4) and move the "now fits" case from minute 65 to 63,
  so both branches (now shown, now dropped) stay covered; no assertion removed. Claude re-ran every Verify: 48 suites
  `ok`, 521 passed, 1 ignored; clippy clean; `cargo fmt --all --check` clean; all three greps/diffs empty, exit 0.
- Gotchas: Mode: Codex loop. `python3 -I tools/codex-quota.py` at start → `CODEX: UNKNOWN (no rate_limits in recent
  rollouts)`, exit 2; before the later rounds → `CODEX: OK` (5-hour 3–5%, weekly 19%). Codex stopped twice, correctly,
  on Claude's own constraints (no golden updater for the pinned goldens; `app.rs` not in Files). Claude ruled on both
  and amended Files (PLAN.md Review log). Goldens came from a temporary env-gated writer that was then removed. The wider
  clocks cut two lines that hold a time: home-full's `repo-a-wt` sentence at 100 columns ends `codex…` (was `codex
  done`), and the step graph header shows `cargo test --worksp… (reported)`. Existing cut rules, accepted; the owner's
  check may revisit them.

### 2026-11-08 — Step 8.6: Real check, README, owner confirm — Claude
- Mode: Claude step (no Codex, no worktree; the quota check is for `codex exec` only and none ran). Docs reviewed by
  an independent adversarial Claude subagent, 2 rounds.
- Did: real step graphs of 6.7 (board 9's own), 6.6 (an unlinked part) and 3.7 (a compressed rollout) from the
  release build (`graph --json` + `home_html --graph`, `--now 2026-11-09T00:00:00Z`), scratch only (W10); board 9's
  source read from the owner's canvas for the comparison. Wrote `docs/phase8-check.md` (method, fixture image, board 9
  in words, 6.7 vs board 9 region by region, 6.6, 3.7, missing-rollout counts, Step 8.4's graph, whole-phase Verify,
  owner check waiting). README: step graph section (keys, membership, G3/G5/G6 limits, missing runs), `s` and `graph`
  in Run, step graph limits, Phase 7 marked approved. Found that most "lost" rollouts are on disk as `.jsonl.zst`
  (Current state).
- Changed: `docs/phase8-check.md`, `docs/img/step-phase8-fixture.png`, `README.md`, `PROGRESS.md`
- Verify (whole phase, HEAD 0ec1283): `cargo test --workspace` → exit 0, 49 test binaries, 519 passed, 0 failed, 1
  ignored; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; phase8 fixture report == `graph-1.1.json`
  → True, its HTML text == `step-phase8.txt` (diff empty); 8.1 golden history → only 8448da9; amended key check
  `git diff --diff-filter=MD e5f6c78 -- fixtures/expected ':!fixtures/expected/tui' | grep '^[-+] ' | grep -v '"title"'
  | grep -v '"started_by"' | wc -l` → 0; text goldens changed since e5f6c78 → only `home-hook*` (8.4b) and
  `repo-phase6*` (8.5 footer); real 6.7 vs board 9 → same slots, edges, panel; differences named in the check doc.
  W10 over the 4 committed files: no `/Users/`, no UUID or 8-hex session id, no prompt text, no secrets; the image is
  synthetic. Owner's answer: **waiting** (stop-and-ask).
- Review: round 1, 9 findings, all accepted and fixed: Step 8.4's later runs are not "separate tasks"; "compressed,
  not lost" over-claimed (now 81/17/18 of 116, one confirmed by content, 24 after the last `.zst` folder); 17 plain-file
  link misses added; implement total is 15 not 9, fix round 1's read→run shift marked untraced; "Claude part covers the
  run" is usually, not always (6.6); three PROGRESS known limits added to README; G5/G6 attribution; PROGRESS said
  committed ahead of the commit (committed right after review); "13 answer-key files". Round 2: 3 findings, fixed:
  8.4's 24 runs come from 28 calls (first ones untagged via a wrapper), "likely" link misses, a re-wrap.
- Gotchas: `links` gives `codex exec resume --last` `form: "exec"`. Rollout file names carry local time. Oct 30
  renders need `--offset -7` (PDT).

### 2026-11-08 — Step 8.5: TUI: `s`, moving along a step, refresh — Claude (Superpowers)
- Mode: Claude-only. `python3 -I tools/codex-quota.py` at the start → `primary: 95% of 300 min window, resets Sun
  01:48` / `secondary: 18% of 10080 min window, resets Thu 00:53` / `CODEX: LOW -> Claude-only mode until the window
  resets` (exit 1). Plan in the session scratchpad; implementer subagents (TDD) on `step/8.5`; one adversarial review
  subagent per round given only `git diff main...step/8.5` and the step.
- Did: measured G11 first: a warm `Scanner::graph` for real step 8.4 took 4.7–4.9 s, 4.5 s of it a fresh
  `multi_link_claude::scan`; PLAN.md Files amended (core graph/scanner, tui graph.rs visibility; Review log). Core: the
  graph reuses the scanner's file-key cache of parsed Claude files (`Scanner::claude_transcripts`, stats untouched).
  Repo footer gains `s step` after `f fork`. App: `s` (own step / Codex incoming / coordinator picker / `no step on this
  session`), graph mode with `←→`/`↑↓` by drawn width, `[ ]`, `o`/`f` with the panel's rule, `esc`, refresh keeping the
  part by session and position, stale reports for another step ignored. CLI worker: `Effect::Graph` → `Request::Graph`,
  immediate rebuild, graph + `graph_steps` on every refresh with a graph open, `started_steps` from an all-time pass on
  every refresh.
- Changed: `PLAN.md`, `crates/core/src/{graph,scanner}.rs`, `crates/tui/src/{app,graph,timeline}.rs`,
  `crates/tui/tests/started.rs` (Refresh literal), `crates/cli/src/tui.rs`, `fixtures/expected/tui/repo-phase6.txt` and
  `.styles.txt` (footer row 39 only), `PROGRESS.md`.
- Verify (re-run by the step session on `step/8.5` at 8af1d20): `cargo test --workspace` → exit 0, 519 passed, 0
  failed, 1 ignored; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; `cargo fmt --all --check` → exit
  0. `git diff --stat fda4e56 -- fixtures/expected` → only `repo-phase6.txt` (1 line) and `repo-phase6.styles.txt` (all
  changed lines on row 39); every other answer key unchanged (`git diff --exit-code fda4e56 -- fixtures/expected
  ':!…repo-phase6.txt' ':!…repo-phase6.styles.txt'` → exit 0); home and step goldens unchanged. Footer before/after:
  `↑↓ select   o open   f fork   +/- zoom   …   q quit` → `↑↓ select   o open   f fork   s step   +/- zoom   …   q quit`,
  `times UTC-8` at the same column, so the 6.1 pair fits. Tests the step names: app `graph_tests` (19: `s` with and
  without a step, Codex lane, coordinator lists direct children only / own wins / none → footer, picker, arrows at both
  ends, narrow + resize across 120, `[ ]` ends, stale report ignored, `esc` restores repo selection and window, `o`/`f`
  on done/running/unlinked/ambiguous parts, refresh adding a part keeps the selection, refresh error keeps the screen,
  empty graph, no `sentinel-` text in renders of all 13 phase8 reports); tui.rs (`graph_steps` order, phase8 graph
  refresh equals its key, stale threshold: graph 1.5 part 2 `working` at 18:00 → `stale` at 18:06 with no file change,
  coordinator end to end on `fixtures/sessions-started`, `started_steps` rules incl. a child outside the 2 h window);
  scanner `warm_graph_parses_no_claude_file_again` (`Some(0)` warm, `Some(1)` after one touched file; fails with the fix
  reverted). G11 on real data (scratch bench, release, same calls as the worker's refresh): step 8.4 (largest: 25 tagged
  hand-offs, 51 parts) warm whole refresh 229–236 ms; cold first 6.4 s (overview's first parse). Real data stayed in
  scratch.
- Review: round 1 (adversarial subagent): 0 Critical, 2 Important, 7 Minor. Fixed: coordinator list used the windowed
  overview (now all-time `started_steps`), cache test passed with the fix reverted, `s` during an open check, Codex
  lane step from another repo. Parked with rulings: rapid `[ ]` builds intermediate graphs, keys before the first graph
  after a failed refresh, double all-time pass, `q quit` hidden at 101–109 columns. Round 2: all four ADDRESSED; one
  new Important parked (CLAUDE.md caps review at 2 rounds): a Codex lane whose parent is outside the window says `no
  step on this session`. PLAN.md Review log has both rounds.
- Gotchas: core fills `SessionRow.handoffs` only for Codex rows; a Claude lane's tagged hand-offs come from the links
  report. The `repo-phase6` golden test has no update path; the footer row was edited by hand.


### 2026-11-08 — Step 8.4c: Sessions started by a session nest under it in the repo view — Claude (Superpowers)
- Mode: Claude-only. `python3 -I tools/codex-quota.py` → `primary: 95% of 300 min window, resets Sun 01:48`,
  `secondary: 18% of 10080 min window, resets Thu 00:53`, `CODEX: LOW -> Claude-only mode until the window resets`
  (exit 1). Worktree `.claude/worktrees/step-8.4c`, branch `step/8.4c` from `main` (914f979).
- Did: Claude wrote the synthetic fixture root and answer keys first (db194e0: keys derived from the 8.4b output by
  the S1–S3 rules, script in scratch). Three implementer subagents (TDD; Task 2 and the finding-7 fix admitted code
  before tests): app-record reader + `started_by` in sessions (3926b98), overview nesting + CLI test + docs (3154572),
  repo view to any depth (ca0ca18). All three keys matched on the first run.
- Changed: `crates/claude/src/{app_record,lib}.rs`, `crates/claude/tests/app_record.rs`, `crates/model/src/lib.rs`,
  `crates/model/tests/model.rs`, `crates/core/src/{scanner,sessions,overview,open}.rs`, `crates/core/tests/{started,
  graph,overview,phase6,sessions}.rs`, `crates/tui/src/{timeline,view,app}.rs`, `crates/tui/tests/started.rs`,
  `crates/cli/src/main.rs` (count only), `crates/cli/tests/cli.rs`, `fixtures/sessions-started/**`,
  `fixtures/expected/started/**`, 9 existing keys (added `started_by: null` only), `docs/{design,output-schema}.md`,
  `PLAN.md` (Files line + Review log), `PROGRESS.md`.
- Verify (re-run by the step session on 8fc31f9): `cargo test --workspace` → exit 0, 490 passed, 0 failed, 1 ignored
  (49 suites); `cargo clippy --workspace --all-targets -- -D warnings` → clean; `cargo fmt --check` → clean;
  `git diff 914f979 -- fixtures/expected ':!fixtures/expected/started' | grep '^[-+] ' | grep -v started_by` → prints
  nothing; `git diff 914f979 --stat -- fixtures/expected/tui` and `-- fixtures/expected/phase8` → empty (every home
  golden, the 8.1 golden and the 6.1 goldens unchanged). W10: no personal paths or non-synthetic ids in new fixture
  files. Real run in scratch (counts only, `--since all`): 139 app records, 134 with `cliSessionId`, 66 with
  `spawnedFrom`, 4 fork/btw, 62 eligible (was 59; new sessions since planning); 412 sessions, 233 Claude, 62 with
  `started_by`, all 62 nested, 0 top level (no cross-repo or unlisted parent), 2 whose parent is itself started
  (unchanged); no diagnostics on stderr.
- Review: fresh adversarial subagent on `git diff main...step/8.4c` + the step: round 1, 9 findings (0 critical,
  2 important). Fixed 1 (recursive CLI count), 2 (record-only sessions got `started_by`), 3 (empty hand-off
  comparison), 4 (cycles after `--since`), 5 (deep indent over bars), 7 (deleted-mid-scan record counted bad) in
  8fc31f9; ruled 6 (W6 order kept) and 9 (shared app id reading); parked 8 (`home::selectable`). Round 2: all six
  ADDRESSED, no new breakage. Step session re-ran Verify; matches the implementers' reports.
- Gotchas: Rulings: null fields count as absent; the conflict check counts only eligible records; repo view order is
  the reverse of the JSON order. `priorCliSessionIds` exists on 19 real records (resumed sessions); not read (S1
  uses `cliSessionId` only), so a child whose transcript id changed on resume would not link.

### 2026-11-08 — Step 8.4b: Rows named by title and place; header names the latest hand-off — Codex, then Claude (Claude-only after quota LOW)
- Did: Implemented U1 own/main-file title parsing (last non-blank string, controls removed, trimmed), title
  propagation to sessions and overview rows, U2 title/step/prompt ordering with per-candidate background-preamble
  rejection, U5 latest-tagged-hand-off header wording, and U6 checkout place words on repo/home rows. Added tests
  for title ownership, blank/non-string/malformed/inherited/subagent cases, scanner refresh, children/merged rows,
  header differences and an untagged newer link, prompt fallback, recorded-branch preservation, real temporary git
  main/worktree mapping for both harnesses, and a changed main branch. Existing two-parent Codex placing test passes.
  Updated docs and the title-only JSON answers. Kept original scrubbed `<name>` title placeholders; no real titles
  or prompts were read or added.
- Mode: Codex loop. Owner reported `python3 -I tools/codex-quota.py` printed `CODEX: OK`, 84% of the 5-hour window,
  17% weekly. This is the supplied hand-off preflight, not a command rerun by this agent.
- Changed:
  - `PROGRESS.md`
  - `README.md`
  - `crates/claude/src/lib.rs`
  - `crates/core/src/open.rs`
  - `crates/core/src/overview.rs`
  - `crates/core/src/repo.rs`
  - `crates/core/src/sessions.rs`
  - `crates/core/tests/graph.rs`
  - `crates/core/tests/overview.rs`
  - `crates/core/tests/phase6.rs`
  - `crates/core/tests/sessions.rs`
  - `crates/model/src/lib.rs`
  - `crates/model/tests/model.rs`
  - `crates/tui/src/home.rs`
  - `crates/tui/src/timeline.rs`
  - `crates/tui/src/view.rs`
  - `crates/tui/tests/view.rs`
  - `docs/design.md`
  - `docs/output-schema.md`
  - `fixtures/expected/hook/overview.json`
  - `fixtures/expected/hook/sessions-2h.json`
  - `fixtures/expected/hook/sessions.json`
  - `fixtures/expected/overview.json`
  - `fixtures/expected/phase4/overview.json`
  - `fixtures/expected/phase5/overview.json`
  - `fixtures/expected/phase6/overview.json`
  - `fixtures/expected/phase6/sessions.json`
  - `fixtures/expected/sessions.json`
  - Claude, in review: `PLAN.md` (Verify amendment), `crates/core/src/repo.rs`, `crates/tui/src/{home,timeline}.rs`,
    `fixtures/expected/tui/home-hook{,-full}{.txt,.styles.txt}`
- Verify: Rust PATH set per AGENTS.md. `cargo test --workspace` → exit 101, home snapshot target fails (decisive
  output below); `cargo test --workspace --no-fail-fast` → exit 101, same two failures, every other target passed.
  Actual full-run result tally: `Test totals: 443 passed; 2 failed; 1 ignored.` Actual failure output:
  ```text
  failures:
      home_hook_full_snapshot_100x80
      home_hook_snapshot_100x30

  test result: FAILED. 16 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

  error: test failed, to rerun pass `-p multi-link-tui --test home`
  ```
  Full-run final output:
  ```text
  error: 1 target failed:
      `-p multi-link-tui --test home`
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, final actual output:
  ```text
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.77s
  ```
  `cargo fmt --all -- --check` and `git diff --check` → no output, exit 0.
  `git diff c45fbcd -- fixtures/expected | grep '^[-+] ' | grep -v '"title"'` → **no output** (final grep exit 1,
  meaning no selected lines). Independent recursive JSON comparison against c45fbcd, stripping `title` only from
  Session/SessionRow objects (retaining PLAN titles), printed:
  ```text
  PASS: 28 committed expected JSON files preserve every existing key and value; only Session/SessionRow title ignored.
  ```
- Verify (golden rows): **no changed rows in any of the four 6.1 goldens**:
  `home-phase6.txt`: none; `home-phase6.styles.txt`: none; `repo-phase6.txt`: none;
  `repo-phase6.styles.txt`: none. 8.1 `step-phase8.txt` and `step-phase8.styles.txt` are also unchanged.
  `git diff --exit-code c45fbcd --` with those six paths → no output, exit 0.
- Review (Claude, 2026-11-08): Codex's run left 2 failing hook snapshots: the key check's grep also read
  `fixtures/expected/tui`. Claude amended PLAN.md (Review log "Step 8.4b Verify amendment": the grep excludes
  `fixtures/expected/tui`; home goldens fall under the goldens rule) and regenerated `home-hook.txt`,
  `home-hook-full.txt` and their style files: **only row 14 changes** (`claude · no prompt` → the fixture's scrubbed
  `<name>` title, same colours). Before a fix round, `python3 -I tools/codex-quota.py` → `primary: 95% … CODEX: LOW`
  (exit 1), so the rest of the step ran Claude-only (CLAUDE.md mode switch). Round 1 (fresh adversarial Claude
  subagent, diff + step only): 3 should-fix, all fixed by Claude: the home branch cell cut `worktree step-8.4b` to
  `worktree…` (new `place_fit`: `wt <folder>`, then char cut); the U5 suffix was the first thing `fit` dropped
  behind a real PLAN title (new `plan_fit` cuts the title first; home and repo-view headers); a removed
  `.claude/worktrees/<name>` cwd fell back to `main` (now `worktree <name>`). Tests added for each. Nits recorded as
  known limits in Current state. Round 2 (new subagent, the fixes only): no blockers, nits only. Claude re-ran Verify:
  `cargo test --workspace` → 447 passed, 0 failed, 1 ignored; `cargo clippy --workspace --all-targets -- -D warnings`
  → exit 0; `cargo fmt --all -- --check` → exit 0; `git diff c45fbcd -- fixtures/expected ':!fixtures/expected/tui' |
  grep '^[-+] ' | grep -v '"title"'` → no output; `git diff --stat c45fbcd -- fixtures/expected/tui` → only the four
  hook files (4 insertions, 16 deletions); 8.1 golden and the four 6.1 goldens unchanged. W10: added lines scanned
  for home paths, emails, keys and session UUIDs → none; fixture titles are `null` or `<name>` only. Real run in
  scratch, counts only (`overview --json --since all --now 2026-11-08T23:00:00Z`, 8.4a binary vs this build, 411
  rows): before 212 prompt / 91 step / 108 no-prompt; after 179 title / 40 step / 92 prompt / 100 no-prompt.
- Gotchas: (Codex, before review) **Verification conflict, not a pass.** `home-hook.txt` and `home-hook-full.txt` would change row 14
  (0-based) from the old `claude · no prompt` label to the existing `<name>` title; their corresponding style files
  would change that same row's runs. Those goldens are left untouched: changing them would make the exact required
  pipeline print non-title lines. No fixture records were blanked or otherwise changed to evade this conflict.
  Asked the owner whether to keep this blocker or exclude TUI goldens from the JSON key check; no answer yet.
  `SessionRow.place` and `RepoGroup.latest_tagged_handoff` are Rust-only display metadata (`serde(skip)`), preserving
  the recorded JSON branch and latest_handoff. Main-branch git results (including failure) are memoised per checkout,
  refreshed when `.git/HEAD` contents change; linked worktrees use checkout basename. Fixture mode performs no git
  display lookups. Real-data count/render run intentionally left to Claude. Nothing staged or committed; pre-existing
  untracked `.claude/` untouched. Earlier verification runs caught missing title answers and legacy test key stripping;
  those were fixed without removing existing PLAN-title coverage.

### 2026-11-08 — Step 8.4a: Busy is busy; help is not a hand-off — Codex
- Did: Implemented U3: confirmed-live Claude busy records stay busy at any age; unavailable probes retain
  the 300 s limit, gone processes remain finished. Implemented U4 using the existing token quote/escape
  metadata: skip help-only exec/resume segments, skip option values, stop options at `--` or a positional
  word, and continue to later command segments. Updated the schema for both rules. Added regression
  tests for the four FakeProcessProbe cases, W5 state_since, help flags, quoted/escaped text, option
  values, wrappers and the second segment's step tag.
- Changed: `crates/core/src/state.rs`, `crates/claude/src/invocation.rs`, `crates/core/tests/sessions.rs`,
  `crates/claude/tests/invocation.rs`, `docs/output-schema.md`, `PROGRESS.md`.
- Mode: Codex loop. Quota check before hand-off (provided by Claude/user):
  `primary: 81% of 300 min window / secondary: 16% of 10080 min window / CODEX: OK` (exit 0).
- Verify: Rust PATH set per AGENTS.md. Initial regression runs against the old implementation:
  `cargo test -p multi-link-core --test sessions busy_record_uses_process_liveness_before_age` →
  `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s`
  (Alive at 20 min was Stale, expected Busy).
  `cargo test -p multi-link-claude --test invocation help_options_are_not_launches` →
  `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s`
  (`codex exec --help` was recognised, expected None).
- Verify: `cargo test --workspace` → exit 0; actual result lines:
  ```text
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.11s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.86s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 80 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
- Verify: `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; no warnings; actual final line:
  ```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.03s
  ```
- Verify: `rustfmt --edition 2024 --check crates/core/src/state.rs crates/claude/src/invocation.rs crates/core/tests/sessions.rs crates/claude/tests/invocation.rs`
  → no output, exit 0. `git diff --exit-code fixtures/expected` → no output, exit 0.
  `git diff --check` → no output, exit 0.
- Real-data comparison (Claude; scratch only, counts): old build (HEAD e03f65c) and new build run back to back with
  `--since all --json --now 2026-11-08T12:00:00Z`. `links`: 213 → 202; 11 links dropped, 0 added, 0 changed. All 11
  are help-only invocations (their commands, read from the transcripts by tool_use id, all have a `--help`/`-h` word;
  3 are `resume`); 5 were `unlinked`, 6 `heuristic` with a child. Of those 4 distinct children: 2 kept their other,
  non-help link (child no longer double-claimed; no child moved to a new link), 2 had only help links and are now
  unparented (`unparented` 112 → 114). `sessions`: 409 both, same ids; 1 session differs, `stale` → `busy` (`state`,
  `state_since`; a busy record with a live process). No other difference.
- Review (Claude): clean, 0 fix rounds. Re-ran Verify: 440 passed, 0 failed, 1 ignored; clippy exit 0;
  `git diff --exit-code fixtures/expected` exit 0 — matches the pasted output. Notes, no change asked:
  (1) invocation.rs `help_only`: `codex exec resume <uuid> --help` stays linked (the step's "before the first
  non-option word"); (2) value options outside the step's list (`-m`, `-p`, `-i`) are not skipped, so `codex exec -m X
  --help` stays linked; (3) the long forms added to the value list go beyond the step's five but follow its intent
  and are documented. Scope: only the step's files changed. Codex had rewritten "Current state"; Claude replaced it.
- Gotchas: No real agent data accessed. Claude owns the before/after comparison at the fixed clock
  `2026-11-08T12:00:00Z`. One intermediate test run incorrectly expected null state_since for stale;
  corrected the assertion to the existing W5 statusUpdatedAt rule (no implementation change to W5).
  No staging or commits; existing untracked `.claude/` left untouched.

### 2026-11-07 — Step 8.4: TUI: step graph render to board 9 — Codex (hybrid)
- Did: `crates/tui/src/graph.rs`, a pure render of `GraphReport` per G8 and G10: header, scroll hints, slots and boxes,
  hand-off and return edges, selection panel, `where the time went` strip, legend, footer; short screens (strip, then
  legend, then panel down to its first line); the list form below 120 columns; NO_COLOR keeps every glyph and word; the
  empty step. Four theme roles. `home_html --graph`. No App wiring.
  How it ran (hybrid mode, CLAUDE.md): Superpowers `writing-plans` split the step into 7 tasks in the session scratchpad
  (header/footer/empty; boxes/edges/scroll; panel; strip + full golden; short screens + NO_COLOR; narrow list; example).
  Branch `step/8.4` in worktree `.claude/worktrees/step-8.4`, cut from 9f66465. Per task: `codex exec` wrote the failing
  tests, Claude ran the red gate and committed `(red, Step 8.4)`, `codex exec resume` wrote the code, Claude ran the green
  gate (filter, full `cargo test`, clippy, red assertions unchanged, fixtures unchanged) and committed. Claude subagents
  reviewed spec then code quality; fixes went back to Codex and re-passed the green gate. Tasks 1–5 got both reviews (with
  re-reviews); then the owner stopped hybrid mode (Gotchas), so Task 6 had its spec review and one fix round, Task 7 one
  combined review and one fix round, and their re-reviews and Task 6's quality review were stopped unfinished. A fresh
  Claude subagent reviewed `git diff main...step/8.4`: APPROVED with 6 findings; 4 fixed in one Codex round. 27 commits,
  fast-forwarded into `main`; branch and worktree left in place.
- Changed: `crates/tui/src/{graph,lib,theme}.rs`, `crates/tui/tests/graph.rs`, `crates/tui/examples/home_html.rs`, `PROGRESS.md`
- Verify (Claude, after the last fix, at c6c18dc): `cargo test --workspace` → 46 `test result: ok` lines, 438 passed, 0
  failed, 1 ignored; `cargo clippy --workspace --all-targets -- -D warnings` → `Finished dev profile`, no warnings;
  `phase8_render_equals_golden` (whole text and styles vs `step-phase8.txt` / `.styles.txt`) → `1 passed`; `git diff
  --exit-code --diff-filter=MD e5f6c78 -- fixtures/expected/` → exit 0; `git diff --exit-code main -- fixtures/` → exit 0
  (the four 6.1 goldens and the 8.1 golden unchanged). Tests named by the step all exist (slot rule with same-harness
  neighbours, 7 parts scrolled at each end, unlinked and ambiguous, `│` vs `╎`, 1-second strip part, list at 119/100/80/60
  with no cut words, 39 and 30 rows, NO_COLOR, empty step). No golden mismatch was blamed on a key or the golden.
- Review: Claude. Gates per task (red / green; no gate failed, 0 gate failures in all):
  - Task 1 header/footer/empty: red pass (fails on the missing `render_graph` and theme fields) / green pass; 2 fix rounds
    (spec 3 of 4 findings sent, 1 moved to Task 2; quality 8 of 9, 1 rejected), each re-passed green; re-review approved.
  - Task 2 boxes/edges/scroll: red pass (5 assertion failures) / green pass; 2 fix rounds (spec 5 of 8 sent, 2 notes
    rejected, 1 kept; quality 9 of 9); re-review approved.
  - Task 3 panel: red pass (6) / green pass; 2 fix rounds (spec 9 of 10; quality 8 of 9, 1 moved to Task 6); re-review approved.
  - Task 4 strip + full golden: red pass (4) / green pass, the full golden matched on the first green run; 1 fix round
    (spec 5 of 5); quality approved with 4 low notes, none sent; re-review approved.
  - Task 5 short screens + NO_COLOR: red pass (2 assertion failures; the NO_COLOR and tiny-buffer tests passed on arrival)
    / green pass; 2 fix rounds (spec 7 of 8, 1 rejected; quality 7 of 10 plus 1 spacing note from the spec re-review);
    spec re-review approved; quality re-review stopped unfinished.
  - Task 6 narrow list: red pass (4) / green pass; 1 fix round (spec 9 of 9, run as two Codex calls: Codex stopped to ask
    before changing 3 earlier assertions those decisions contradicted; Claude approved exactly those 3); spec re-review and
    quality review stopped unfinished.
  - Task 7 example: red pass (1) / green pass; 1 fix round (combined review 6 of 7, 1 rejected); re-review stopped unfinished.
  - Final review: APPROVED, 6 findings, 4 fixed (stale time, empty-session count, narrow footer keeps `esc`/`q`,
    `GraphOptions` derives), 2 not changed (lanes clipped under ~26 rows is the decided behaviour; the example test needs a
    full `cargo test`, noted in Current state).
  **Measures (real numbers only):**
  - Wall-clock: 1 h 20 min 33 s (22:34:07–23:54:40 PST, step session start to writing this entry; merge follows).
  - Tests added: 80, all in `crates/tui/tests/graph.rs` (358 → 438 passed).
  - Implementer runs: 28 `codex exec` calls (7 red, 7 green, 14 fix calls; one fix call was wasted on a missing findings
    file, one continued a stopped fix). Codex time from the per-call timer: 3 033 s in all.
  - Reviewer runs: 22 Claude subagents started: 18 finished (13 spec/quality reviews, 4 re-reviews, 1 final review), 4
    stopped unfinished when hybrid mode ended.
  - Findings: 98 raised by the reviewers that finished (counted from their reports, notes included); 81 sent to Codex and
    fixed, the rest rejected with a reason, moved to a later task, or low notes left as they are.
  - Fix rounds: 14 Codex fix calls over 8 review targets (7 tasks + the final review); no task or the final review went
    past 2 rounds.
  - Gate failures: 0 red, 0 green.
  - Tokens: the 27 Codex logs that survive print `tokens used`, 2 371 575 in total (the wasted call's log was
    overwritten). Resumed calls may count the session's earlier context again, so this is an upper bound per call. Claude
    subagent tokens not summed (not measured).
- Gotchas:
  - **Hybrid mode was stopped mid-step after Task 5** (owner, via the coordinator, 2026-11-07): to save Codex usage, with
    the 5-hour window at 70%. Tasks 6 and 7 were already implemented test-first behind the gates when the message came, so
    no single hand-off call was needed; the per-task two-stage reviews stopped there, and the final review plus at most 2
    Codex fix rounds finished the step.
  - **Hand-offs must name the step on the `codex exec` command line** (coordinator, mid-step): multi-link recognises a
    hand-off only when `codex exec …` appears in the command itself and reads the step from that text. The first calls
    went through a `codex.sh` wrapper with the prompt in a file, so they show as untagged (`step: null`) and the resume
    calls as "GREEN half…". From Task 1's quality fix on, every call was a direct `codex exec` naming `Step 8.4`.
  - `codex exec resume` takes no `-s` / `--add-dir`: the sandbox and the writable `~/.cargo`, `~/.rustup` roots were passed
    with `-c sandbox_mode=…` and `-c sandbox_workspace_write.writable_roots=[…]`. With tasks overlapping, resumes used the
    session id, not `--last`.
  - Red and green halves overlapped other tasks' reviews to save wall-clock, so later tasks' fixes landed on top of earlier
    tasks' code; each fix call was told to keep every test passing.
  - Review decisions sometimes changed behaviour an earlier task's test pinned. Codex correctly stopped and asked rather
    than edit assertions; the gate rule "red assertions unchanged" needs an explicit "unless Claude approves" path.
  - One reviewer subagent ran `git checkout --detach` in the worktree by mistake and put it back at once (branch clean);
    later prompts said never to check out there.

### 2026-11-07 — Step 8.3: Core: graph report, CLI `graph` — Claude (Superpowers trial)
- Did: Built the step graph report per G1–G4, G6, G7. Report types are in `crates/model`. `crates/core/src/graph.rs` has a
  pure `build` (owned and tagged hand-offs, chains, Codex runs from turns, Claude cuts and names, edges, calls in parts) and
  `Scanner::graph` for the I/O: one `--since all` refresh, on-demand `claude::calls` / `codex::read_details` for the step's
  parents, other in-repo parents that resume the same children, and their children only. It also has `commit_for_step`
  (`git log`, exact `(Step <id>)`, real mode only). CLI `multi-link graph --repo R --step ID [--json]`.
  How it ran (owner's trial, CLAUDE.md rules): Superpowers 7.0.0 `writing-plans` wrote a 6-task plan to the session
  scratchpad (not the repo). `subagent-driven-development` with `test-driven-development` then ran each task as one
  implementer subagent plus one task-reviewer subagent, then a final whole-branch review. Codex reviewed
  `git diff main...trial/step-8.3`, 2 rounds. Branch `trial/step-8.3` in worktree `.claude/worktrees/step-8.3`, cut from
  ddca184; 9 commits, then fast-forwarded into `main`. The branch and worktree are left in place for the owner.
  **Readings added in review** (decided by the step session; PLAN.md and design.md were outside the step's files, so a
  planner should copy these into G2/G6 at the next PLAN.md edit):
  1. **No Claude part covers a run that is still going.** G2's bold rule ("a Codex run's interval belongs to Codex and no
     Claude part covers it") and "kept only when no tagged run of the chain is still going" conflicted with the pinned
     "G2 No interval" text ("the next Claude part starts at that same hand-off"). Taken literally, the pinned text lets a
     Claude part overlap an earlier run that is still going. Chosen: a Claude part after any hand-off, with or without an
     interval, starts at the latest of its usual start and the `to` of earlier interval runs of the chain still going
     then, with `edge_in` `return` at that time. The part is dropped when `from ≥ to`; an open earlier run still means
     no last part. Why: the bold rule is the invariant the screen depends on, and the pinned text describes the common
     case where no run is going. No key changes. Codex contested the return edge after a no-interval hand-off. It then
     accepted it under output-schema's "a Claude part that starts at a run's end": without it, that earlier run shows no
     return anywhere.
  2. Every Claude part's `from` is clamped to the chain start (the previous differently tagged run's end); a part that
     starts there gets no return edge (that run is not this chain's).
  3. With no last part, a read in no part counts as `waiting_reads` up to the latest run end of the chain, capped at the
     next differently tagged hand-off.
  4. A call exactly where two Claude parts meet belongs to the earlier part only; G3's `verify + commit` uses the same rule.
  5. A run window also ends at a resume of the same child by another in-repo parent with no tagged hand-off (G4 "next
     hand-off of the same child"). A resume from another repo does not cap it (the spec is silent).
  `untagged_handoffs` stays exactly the schema's `from < at ≤ to`. An earlier ruling of mine moved unheld untagged
  hand-offs onto the previous Claude part; Codex showed it contradicted the schema, and it was reverted.
  **Trial measures (Step 8.2's baseline is in its entry):**
  - Wall-clock: 1 h 38 min 51 s (20:08:54–21:47:45 PST, step session start to committing this entry; merge follows). Codex review
    round 1 alone took 46 min 34 s (20:53:04–21:39:38); round 2 took 55 s.
  - Tests added: 62 run by default (296 → 358).
  - Implementer runs: 9 Claude subagents: 6 tasks, 1 task fix round (Task 4), 1 final-review fix wave, 1 Codex-review fix.
  - Reviewer runs: 9 Claude subagents (6 task reviews, 1 Task 4 re-review, 1 final whole-branch review, 1 re-review of
    its fixes) plus 2 Codex rounds.
  - Findings from the independent (Codex) review: 7 in round 1, all medium; 6 accepted, 1 rejected with a reason. Round 2:
    0 new; all 7 verdicted. Superpowers' own reviews found 4 Important (Task 3: 1, Task 4: 1, final: 2), plus Minors.
  - Fix rounds: 3 (Task 4 ×1, final-review wave ×1, Codex ×1).
  - Tokens, as printed by the harness per subagent run: implementers 1,106,287 and Claude reviewers 867,202. Codex
    87,248 + 55,969 as printed by `codex exec`. The controller session's own tokens were not measured.
  - Rough comparison only: the steps differ in size and kind (8.3 is the core of the phase; 8.2 was readers).
- Changed: `crates/model/src/lib.rs`, `crates/model/tests/graph.rs`, `crates/core/src/graph.rs`, `crates/core/src/lib.rs`,
  `crates/core/tests/graph.rs`, `crates/core/tests/graph_commit.rs`, `crates/cli/src/main.rs`, `crates/cli/tests/graph.rs`,
  `PROGRESS.md`.
- Verify (re-run by the step session in the worktree at dfc8264, with
  `export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"`): `cargo test --workspace` → exit 0. There are 44
  result lines, all `ok`: 358 passed, 0 failed, 1 ignored (the existing opt-in `real_counts`). The result lines, counted
  (`uniq -c`, timings cut):

```text
  11 test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
   2 test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
   1 test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
   1 test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
   1 test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
   1 test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
   1 test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
   2 test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
   1 test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
   1 test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
   1 test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
   3 test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
   1 test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
   4 test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
   1 test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
   3 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
   2 test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
   1 test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
   6 test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

  `cargo test -p multi-link --test graph` → 7 passed, including `every_phase8_key_matches` (13 keys) and
  `sentinels_never_in_output`. `cargo test -p multi-link-core --test graph_commit` → 1 passed (`newest_exact_step_commit`:
  temp repo, `(Step 6.7)` vs `(Step 6.70)`). `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  `cargo fmt --all -- --check` → exit 0. `git diff --exit-code --diff-filter=MD e5f6c78 -- fixtures/expected/` → exit 0,
  no output. `git diff --check main...HEAD` → exit 0.
  Verify items and their tests (crates/core/tests/graph.rs unless named):
  - G2 boundaries:
    - burst start: `chain_starts_at_burst_start`
    - chain split: `a_b_a_is_two_chains`
    - no Claude part during a silent run: `no_claude_part_during_silent_run`
    - parent silent at the return: `parent_silent_at_return_no_last_part`
    - overlapping background runs: `overlapping_background_runs_are_neighbours`
  - G3 names:
    - no commit: `last_part_without_commit_is_review`
    - a chain with no plan part: `no_plan_when_first_handoff_opens_burst`, `previous_run_still_going_leaves_no_plan`
  - G4 two resumes in one burst: `two_resumes_in_one_burst_are_two_runs`
  - A child file removed before reading: an injected reader returns NotFound for the 8201 rollout through
    `Scanner::graph_with_reader` (a `#[doc(hidden)]` seam; `Scanner::graph` is unchanged), which gives `unavailable`. Also
    `child_without_rollout_is_unavailable` and key 1.11.
- Review: Superpowers task reviews, 6 approved; Task 4 needed 1 fix round, and the rest were clean. The final
  whole-branch review (opus) said "with fixes": 2 Important, 5 Minor. Its fix wave was re-reviewed, 5 of 5 addressed.
  Codex round 1 gave 7 medium findings:
  - (1) a window past the next-step cap: accepted
  - (2) untagged counted outside `from < at ≤ to`: accepted, my ruling reverted
  - (3) commit naming vs placement: accepted
  - (4) a return edge after a no-interval hand-off: rejected, reason above, which Codex accepted in round 2
  - (5) parts before the chain start: accepted
  - (6) no real file-removal test: accepted
  - (7) thin text mode: accepted

  Codex round 2: 7 of 7 verdicted, no new bugs, every changed file within the step's list. The step session re-ran
  Verify (above) and it matches what the implementers reported.
- Gotchas:
  - `Scanner::graph` parses every Claude transcript (`claude::scan`) per build; it fails soft on a scan error. G11's cost
    is 8.5's to measure.
  - Untagged hand-offs made during a run count nowhere (schema rule).
  - A resume from another repo, or by a parent the scan does not list, never caps a run window.
  - `%h` can exceed 7 characters on an ambiguous hash.
  - Text-mode `✓ committed` has no test, because fixture mode never has a commit.
  - The extra-parents load in `Scanner::graph` has only a pure-build test: no fixture has two parents resuming one child.
  - The worktree has the Superpowers scratch folder `.superpowers/sdd/` (self-ignored, never committed).

### 2026-11-06 — Step 8.2: Calls from both transcripts — Codex (review: Claude)
- Did: Added on-demand owned calls for Claude main/subagent transcripts, deduplication and result joins; Codex calls
  and per-turn details with imported turns excluded, open turns retained, and unknown records counted. Added shared
  fixed-label classification without executing transcript input. Scanner parsing and existing report contracts are unchanged.
  Review fixes: shared the scanner's response-user filter for per-turn prompt fallback, with first-event precedence;
  labels inspect all simple commands, prefer commit, and omit heredoc bodies while preserving first-command kind.
  Ignored `additional_tools`; missing subagent files are skipped while missing main files still error. Added synthetic
  tests for these cases and absent `is_error` (unknown result). All five requested findings addressed in one fix round.
  Trial baseline, measured by the step session: wall-clock 53 min 01 s (21:48:06–22:41:07 PST, start of the step session to
  writing this entry; commit follows). Of that, Codex 42 min 34 s (implement) + 3 min 17 s (fix). Tests added: 15 run by
  default (281 → 296) plus 1 opt-in. Implementer runs: 2 (1 implement, 1 fix). Reviewer runs: 2, both Claude in the step
  session (diff, then the fix); no separate reviewer agent. Independent review findings: 5 (2 high, 3 low), 1 of them a test
  gap; 5 fixed. Fix rounds: 1. Tokens: Codex 87,991 + 64,476 as printed by `codex exec`; Claude tokens not measured.
- Changed: `crates/model/src/lib.rs`, `crates/model/tests/calls.rs`, `crates/claude/src/lib.rs`,
  `crates/claude/tests/calls.rs`, `crates/codex/src/lib.rs`, `crates/codex/tests/calls.rs`, `PROGRESS.md`.
- Verify: With `export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"`, `cargo test --workspace`
  → exit 0; actual result lines (296 passed in total, one opt-in test ignored):

```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.10s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; actual output (repo path replaced with `<repo>`):

```text
    Blocking waiting for file lock on build directory
    Checking multi-link-model v0.1.0 (<repo>/crates/model)
    Checking multi-link-plan v0.1.0 (<repo>/crates/plan)
    Checking multi-link-codex v0.1.0 (<repo>/crates/codex)
    Checking multi-link-tui v0.1.0 (<repo>/crates/tui)
    Checking multi-link-claude v0.1.0 (<repo>/crates/claude)
    Checking multi-link-core v0.1.0 (<repo>/crates/core)
    Checking multi-link-hook v0.1.0 (<repo>/crates/hook)
    Checking multi-link v0.1.0 (<repo>/crates/cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.00s
```

  `git diff --exit-code --diff-filter=MD e5f6c78 -- fixtures/expected/` → exit 0, no output.
  `git diff --check` → exit 0, no output.
  `cargo fmt --all -- --check` → exit 0, no output.
  `MULTI_LINK_COUNTS_ROOT="$HOME" cargo test -p multi-link-codex --test calls real_counts -- --ignored --exact --nocapture`
  → exit 0; actual counts-only output from read-only access to all non-imported rollouts under the real home root:

```text
rollouts=67 reads=505 edits=14 runs=535 handoffs=0 total=1054 ok_null=154 ok_null_share=0.146110 records_skipped=0 unreadable=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 2.44s
```

- Review: Claude read the diff against Step 8.2, G4, G5 and the pinned rules, and also checked real data, because the
  fixtures could not show two of the bugs. Round 1, 5 findings, all fixed: (1) high: `Turn.prompt` was null on every real
  turn; real turns have 0 `user_message` events and hold the prompt in a `response_item` message. (2) high: `$ git commit`
  was never produced, as real commits follow `git add`; label now reads every simple command and skips heredoc bodies.
  (3) low: `additional_tools` counted as skipped (all 68). (4) low: a missing subagent file failed the parent. (5) test gap:
  absent `is_error`. Round 2: clean. Claude re-ran Verify: `cargo test --workspace` → exit 0, 296 passed, 0 failed,
  1 ignored; clippy exit 0; `cargo fmt --check` exit 0; pinned `git diff` exit 0; real counts →
  `rollouts=67 reads=507 edits=14 runs=537 handoffs=0 total=1058 ok_null=154 ok_null_share=0.145558 records_skipped=0
  unreadable=0`. Codex's pasted line says 1054 total: the difference is 4 calls added to the rollouts on disk between
  the two runs, most likely Codex's own fix run finishing (same 67 rollouts, same null count; not traced call by call).
  Real cross-checks from a scratch program, counts only, nothing kept in the repo: 70 turns, 70 completed, 0 null
  prompts, 0 rollouts differ from the scanner's turns; Claude 207 transcripts, 12,982 calls (ok null 6,732), 0 read errors;
  180 invocations, 179 owned hand-offs, the 1 other owned by another session (a fork copy); of 152 unique Bash commands
  mentioning `git commit`, 112 have it as a command and all 112 get the label, the other 40 only mention it in text. 1,955
  Claude records skipped, equal to the scanner's own count. Bash results always carry `is_error` (6,998 false, 192
  true in raw records); other tools omit it on success, so `ok` null there means "no error flag", as G5 words it.
  No files outside the step's list changed. The `real_counts` ignored test reads the home folder only when asked; kept
  because Verify needs the counts. PLAN.md and design.md do not yet state the label reading (see Current state).
- Gotchas: Transcript fields and exec JavaScript wrappers are undocumented assumptions. Unreadable tool input becomes
  a run with no label; absent `exit_code` gives unknown `ok`, with no success inferred from prose. Imported/foreign-owned
  records are intentionally excluded, separately from malformed/unsupported records counted as skipped. The real-data
  counter is explicitly opt-in; normal tests use synthetic records and committed fixtures only. No real transcript text
  was copied into the repo or printed. This fix round initially tried serializing the internal `Calls` type in a test;
  it has no serialization API, so the test now asserts the exact `None` values directly. Initial implementation checks caught a recogniser API mismatch, two incorrect
  fixture expectations, and two clippy style issues; all corrected before the final successful commands above.

### 2026-11-06 — Step 8.1: Contract and target screen — Claude
- Did: Wrote G0–G11 into `docs/design.md` (Phase 8 decisions, rules pinned while writing the keys, Target screen, palette
  additions with contrast ratios, differences from board 9); FR11 reworded and mapped to need 4; Glossary, screen table,
  user flow and plan updated. Added the `graph` report, Calls and turns, and Phase 8 fixture results to
  `docs/output-schema.md`. Built the synthetic root `fixtures/sessions-phase8/` (28 sessions, 13 PLAN.md steps) with a scratch
  generator, typed 13 answer keys by hand, drew the 120×40 styled golden, and added the shape test (5 tests). No project
  code changed. Scratch tools (generator, recompute, painter) stayed out of the repo.
- Changed: `docs/design.md`, `docs/output-schema.md`, `fixtures/sessions-phase8/**`, `fixtures/expected/phase8/**`,
  `fixtures/expected/tui/step-phase8.txt`, `fixtures/expected/tui/step-phase8.styles.txt`, `crates/tui/tests/phase8_shape.rs`,
  `PROGRESS.md`
- Verify: Rust PATH set per AGENTS.md. `cargo test --workspace` → 37 test binaries, 281 passed, 0 failed (276 before + 5). Shape
  test lines:
  ```text
test every_step_in_the_fixture_plan_has_a_key_and_no_extra_keys_exist ... ok
test golden_boxes_sit_on_the_slot_grid_with_the_selection_on_implement ... ok
test golden_is_120_by_40_and_every_cell_is_styled_once ... ok
test keys_have_every_field_and_consistent_parts ... ok
test keys_hold_only_fixed_labels ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → clean. `git diff --exit-code --diff-filter=MD e5f6c78 --
  fixtures/expected/` → exit 0. `multi-link sessions` on the root → 28 sessions, `skipped` `[]`; `links` → 26.
- Review: Codex (`codex exec -s read-only`), 2 rounds. Round 1: 2 P2, both accepted and fixed in the docs. (1) `Call` had no id
  for the G1 ownership join; (2) `Turn` conflated completion with an open turn's last record. Round 2: both resolved. Codex could
  not run cargo in its sandbox (build lock); it ran the built shape-test binary, 5 passed.
  **Keys:** Codex recomputed all 59 parts of all 13 keys from the JSONL and `links`: no mismatch. My own scratch recompute also
  gives 0 mismatches; on first run it caught two hand-count slips of mine in the 119-character cut (`asked`, `said` in 1.1), fixed.
  Trace below: per part, the line of `from`, of `to`, the call lines it covers, and the hand-off line, in that session's file (`s` =
  its subagent file). Derived values follow from these lines: `active_ms` from bursts, `gap_ms` = `from − at`, `edge_in` return
  `at` = `from`, `asked`/`said` from the turn's `user_message` and `task_complete` lines. A hand-off on a part's first line belongs
  to the part before it. `waiting_reads`: 1.1 plan = 8101 lines 8, 10, 12, 14, 16, 18; 1.5 plan = 8105 lines 6, 10, 12. Titles and
  Verify: `repos/repo-g/PLAN.md`.
  ```text
1.1 (PLAN.md heading + Verify; 5 parts)
  0 plan 8101: from 1, to 6, calls 2,4,6
  1 implement 8201: from 2, to 35, calls 4,6,8,10,12,14,16,18,20,22,24,26,28,30,32; hand-off 8101:6
  2 review 8101: from 20, to 33, calls 21,23,25,27,29,31,33
  3 fix round 1 8201: from 36, to 53, calls 38,40,42,44,46,48,50; hand-off 8101:33
  4 verify + commit 8101: from 34, to 51, calls 35,37,39,41,43,45,47,49
1.2 (PLAN.md heading + Verify; 5 parts)
  0 plan 8102: from 1, to 4, calls 2,4
  1 implement -: no run; hand-off 8102:4 (unlinked, links)
  2 review 8102: from 4, to 8, calls 4,6,8
  3 fix round 1 -: no run; hand-off 8102:8 (ambiguous, links)
  4 review 8102: from 8, to 12, calls 8,10
1.3 (PLAN.md heading + Verify; 7 parts)
  0 plan 8103: from 1, to 4, calls 2,4
  1 implement 8203: from 2, to 17, calls 4,6,8,10,12,14; hand-off 8103:4
  2 review 8103: from 5, to 10, calls 6,8,10
  3 fix round 1 8203: from 18, to 25, calls 20,22; hand-off 8103:10
  4 review 8103: from 11, to 14, calls 12,14
  5 fix round 2 8203: from 26, to 33, calls 28,30; hand-off 8103:14
  6 verify + commit 8103: from 15, to 20, calls 16,18
1.4 (PLAN.md heading + Verify; 6 parts)
  0 plan 8141: from 1, to 4, calls 2,4
  1 implement 8241: from 2, to 7, calls 4; hand-off 8141:4
  2 review 8141: from 5, to 8, calls 6
  3 plan 8142: from 1, to 4, calls 2,4
  4 implement 8242: from 2, to 7, calls 4; hand-off 8142:4
  5 verify + commit 8142: from 5, to 10, calls 6,8
1.5 (PLAN.md heading + Verify; 3 parts)
  0 plan 8105: from 1, to 4, calls 2,4
  1 implement 8251: from 2, to 9, calls 4,6; hand-off 8105:4
  2 fix round 1 8252: from 2, to 9, calls 4,6,8; hand-off 8105:8
1.6 (PLAN.md heading + Verify; 6 parts)
  0 plan 8106: from 1, to 4, calls 2,4
  1 implement 8261: from 2, to 7, calls 4; hand-off 8106:4
  2 review 8106: from 5, to 8, calls 6,8
  3 plan 8106: from 9, to 12, calls 10,12
  4 implement 8261: from 8, to 13, calls 10; hand-off 8106:12
  5 verify + commit 8106: from 13, to 16, calls 14
1.7 (PLAN.md heading + Verify; 3 parts)
  0 plan 8106: from 5, to 8, calls 6,8
  1 implement 8271: from 2, to 7, calls 4; hand-off 8106:8
  2 review 8106: from 9, to 12, calls 10,12
1.8 (PLAN.md heading + Verify; 5 parts)
  0 plan 8108: from 1, to 4, calls 2,4
  1 implement 8281: from 2, to 7, calls 4; hand-off 8108:4
  2 review 8108: from 5, to 12, calls 6,8,10,12
  3 fix round 1 8281: from 8, to 13, calls 10; hand-off 8108:12
  4 review 8108: from 13, to 16, calls 14
1.9 (PLAN.md heading + Verify; 5 parts)
  0 plan 8109: from 1, to 4, calls 2,4
  1 implement 8291: from 2, to 7, calls 4; hand-off 8109:4
  2 review 8109: from 5, to 8, calls 6,8
  3 fix round 1 8291: from 8, to 13, calls 10; hand-off 8109:8
  4 review 8109: from 9, to 12, calls 10,12
1.10 (PLAN.md heading + Verify; 3 parts)
  0 plan 8109: from 9, to 12, calls 10,12
  1 implement 8291: from 14, to 19, calls 16; hand-off 8109:12
  2 review 8109: from 13, to 14, calls -
1.11 (PLAN.md heading + Verify; 3 parts)
  0 plan 8111: from 1, to 4, calls 2,4
  1 implement 82a1: no run; hand-off 8111:4 (exact, links)
  2 review 8111: from 4, to 8, calls 4,6
1.12 (PLAN.md heading + Verify; 8 parts)
  0 plan 8112: from 1, to s4, calls 2,4,s2,s4
  1 implement 82c1: from 2, to 7, calls 4; hand-off 8112:s4
  2 review 8112: from s5, to 6, calls 6
  3 fix round 1 82c1: from 8, to 13, calls 10; hand-off 8112:6
  4 review 8112: from 7, to 10, calls 8
  5 plan 8113: from 10, to 13, calls 11,13
  6 implement 82c1: from 14, to 19, calls 16; hand-off 8113:13
  7 review 8113: from 14, to 15, calls -
1.13 (PLAN.md heading + Verify; empty: no link has step 1.13)
  ```
  **Golden against board 9:** rendered both with colour side by side (scratch image only) and compared region by region; Codex
  compared the board source too and reconstructed all 4,800 golden cells with no mismatch and no unnamed difference. Layout, box
  columns, edge columns, panel columns, strip columns and every colour match. Differences, all named in design.md with reasons:
  header has no `✓ committed` (no git in fixture mode, G7); counts in reads, edits, runs order (G6); `1½ min` → `1 min` (W5); one
  fixed label per line, so `$ cargo test, clippy, fmt ✓` and `$ cargo clippy, test ✓` split and `--workspace` goes (G6); `$ git
  commit → hash` → `$ git commit` (G6); second hand-off `│ confirmed` on purpose (the step asks for one of each); **edit counts:
  `6 reads · 3 edits · 6 runs` against the board's 5 edits and 4 runs, because the two `python3 … write` commands count as runs
  (G5)**; quoted text ends on a word boundary; strip widths 8, 32, 13, 10, 27 against 8, 32, 14, 10, 26 by the written rule;
  legend wording and `✓`/`✗` meaning (W8, G6); `times UTC-8` for `times PST`.
  **W10, all 47 new files:** prompts and agent messages listed and read, all invented; every UUID matches the synthetic `8[12]…`
  scheme; no board 9 id, hash, time or prompt text; `grep` for personal paths and secret words finds nothing; fixture rule 5 `rg`
  prints nothing. Design.md names `timeline.rs` from the board's box; it is this public repo's own file. Board 9 itself is not
  committed. Scope: only the step's files; `.claude/` stays untracked.
- Gotchas: `links` reports a hand-off copied into a fork a second time, so G1 needed an ownership rule. PLAN.md's G2 left the Claude
  part after a no-interval run open; design.md pins it, see Current state. A run of `--repo` in `multi-link graph` takes the session
  `repo` (`/home/dev/repo-g`), unlike `steps`, which takes `repos/repo-g`. `.claude/launch.json` is local and must stay out of commits.

### 2026-11-03 — Step 7.5: Label quotes the latest prompt, not the first — Codex
- Did: Added `last_asked` to Session/SessionRow, both parsers, report construction and TUI labels while
  preserving `asked`. Claude excludes meta, compact-summary, interruption, inherited, tool-result and
  wrapper-only records. Codex keeps event priority and imported-turn exclusions for latest prompts.
  Added synthetic tests for resumed prompts, response-item fallback, skipped trailing records, labels,
  cold/changed/warm scanner passes and overview propagation; updated schema, P5 and the README label sentence.
- Changed: `README.md`, `crates/claude/src/lib.rs`, `crates/claude/tests/phase7.rs`, `crates/codex/src/lib.rs`, `crates/codex/tests/phase6.rs`, `crates/core/src/open.rs`, `crates/core/src/overview.rs`, `crates/core/src/sessions.rs`, `crates/core/tests/overview.rs`, `crates/core/tests/phase6.rs`, `crates/core/tests/sessions.rs`, `crates/model/src/lib.rs`, `crates/model/tests/model.rs`, `crates/tui/src/view.rs`, `crates/tui/tests/home.rs`, `crates/tui/tests/phase6_shape.rs`, `crates/tui/tests/view.rs`, `docs/design.md`, `docs/output-schema.md`, `fixtures/expected/hook/overview.json`, `fixtures/expected/hook/sessions-2h.json`, `fixtures/expected/hook/sessions.json`, `fixtures/expected/overview.json`, `fixtures/expected/phase4/overview.json`, `fixtures/expected/phase5/overview.json`, `fixtures/expected/phase6/overview.json`, `fixtures/expected/phase6/sessions.json`, `fixtures/expected/sessions.json`, `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md. `cargo test --workspace > /tmp/step75-tests.log 2>&1` → exit 0;
  all 36 actual result lines (276 passed, 0 failed):
  ```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.10s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.59s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.54s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  `cargo clippy --workspace --all-targets -- -D warnings > /tmp/step75-clippy.log 2>&1` → exit 0:
  ```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.72s
  ```
  `cargo fmt --all -- --check` and `git diff --check` → no output, exit 0.
  `python3 /tmp/step75-verify.py` (loads every tracked JSON from 081cdcc, recursively removes only
  `last_asked` from current JSON, compares parsed values) → exit 0, actual output:
  ```text
PASS: 42 committed JSON files equal 081cdcc after removing last_asked.
  ```
  `git diff --exit-code 26af6f2 -- fixtures/expected/tui/home-phase6.txt fixtures/expected/tui/home-phase6.styles.txt fixtures/expected/tui/repo-phase6.txt fixtures/expected/tui/repo-phase6.styles.txt`
  → no output, exit 0.
  `cargo build --release -p multi-link > /tmp/step75-release.log 2>&1` → exit 0:
  ```text
    Finished `release` profile [optimized] target(s) in 5.42s
  ```
  `python3 /tmp/step75-counts.py` (runs `target/release/multi-link sessions --since all --json`,
  retains JSON only in memory, prints counts) → exit 0:
  ```text
claude: {"sessions": 186, "asked_null": 11, "last_asked_null": 14, "different": 89, "first_only": 3, "last_only": 0}
codex: {"sessions": 294, "asked_null": 154, "last_asked_null": 154, "different": 6, "first_only": 0, "last_only": 0}
  ```
  `python3 /tmp/step75-exclusions.py` (read-only inspection of first-only Claude records; counts only)
  → exit 0:
  ```text
Claude first-only sessions: 3; excluded eligible-record categories by session: {"isMeta": 3}
  ```
- Review: Claude, 2026-11-03 — clean, no rounds needed. Round 0: Codex correctly stopped because
  `crates/model/tests/model.rs` was outside the file list; Claude added `crates/model/tests/**` to the step
  and re-ran. Diff read against P5: Claude keeps `asked` first-eligible and sets `last_asked` only past the
  `isMeta` / `isCompactSummary` / `[Request interrupted by user` filter; Codex keeps user_message over
  response-item fallback, both inside the existing imported-turn gate; label order last made step ›
  `last_asked` › `asked` › no prompt. All files are in the step's list (`open.rs` is a constructor field only).
  W10: added lines carry synthetic prompts only; no home paths or secrets; PROGRESS holds counts only.
  Claude re-ran Verify: 276 passed / 0 failed, clippy clean, pinned 6.1-golden diff exit 0, 42 tracked JSON
  files equal 081cdcc without `last_asked`, real counts Claude 186 / 11 / 14 / 89, Codex 295 / 154 / 154 / 6
  (Codex 295 vs 294 pasted: one new live rollout between runs). Note, not a defect: the 3 Claude
  `last_asked`-null sessions keep their old label, because `asked` (W3, unchanged) is their `isMeta` text.
- Gotchas: Initial verification exposed old exact-key assertions in model/core and additive-field
  stripping in historical compatibility tests; updated these within the allowed tests. Clippy's
  collapsible-if warnings were fixed. Final workspace run is green. The three Claude first-only
  sessions have only `isMeta` eligible text, so P5 correctly leaves `last_asked` null and labels fall
  back to `asked`; Codex null counts match. Phase 7 owner confirmation remains pending.


### 2026-11-02 — Step 7.4: Real check and owner confirm — Claude
- Did: Ran the whole-phase Verify. Rendered real `~` at `--now 2026-11-03T05:19:00Z` (UTC−8) with
  `crates/tui/examples/home_html.rs`: home at `--since 2h` and `16h`, multi-link repo view at both (the 16 h
  view has three folds, so the fixture was not needed); screenshots by headless Chrome, scratch only, shown to
  the owner. Wrote `docs/phase7-check.md` (counts before/after, P1–P4 on screen in words, Verify, pending
  owner check). README: release status, the Claude prompt rule (P1), clock times (P3), the named window (P2),
  the axis label order (P4), and the stale `no prompt` / `last 2h` limitations replaced by the 11 remaining.
- Changed: docs/phase7-check.md, README.md, PROGRESS.md
- Verify: `cargo test --workspace` → exit 0, 36 `test result: ok`, 0 failed; `cargo clippy --workspace
  --all-targets -- -D warnings` → exit 0; `git diff --exit-code 26af6f2 -- fixtures/expected/tui/home-phase6.txt
  fixtures/expected/tui/home-phase6.styles.txt fixtures/expected/tui/repo-phase6.txt
  fixtures/expected/tui/repo-phase6.styles.txt` → no output, exit 0; real count jq (Phase 7 Verify) →
  `[185,11]` at 2026-11-03T05:18:57Z (baseline `[181,43]`; 7.1 `[182,11]`; the five newest Claude sessions
  all have a prompt). Renders: `no prompt` 0 times in all four; home 16h has `last 16h` in header and card,
  `last 2h` 0 times. W10 over the three committed files: no `/Users/`, no hex id over 8 chars, no secrets;
  207 distinct 40-char prefixes of real `asked` (length ≥ 20) grepped over all 235 tracked files plus the
  new doc → hits only for the project's own `Read AGENTS.md, then do Step…` prompt template in older
  fixtures/docs, none in this step's files. Codex count for README: `[292,154,3]` (total, null, non-imported
  null). Owner's answer: pending (stop-and-ask).
- Review: adversarial Claude subagent (diff + Step 7.4 + P1–P4 + W10; re-ran Verify: same 36 ok, clippy clean,
  goldens exit 0, `[185,11]`; checked every doc number against the scratch renders): 0 P1, 3 P2, 7 P3, all
  accepted and fixed in one round: README kept a Codex `no prompt` limitation (3 older non-imported runs);
  W10 result written into Verify; README's Codex prompt rule names the 6.6 fallback; phase7-check splits the
  two Phase 6 renders, says only 2 of P2's 3 phrases were on screen, and says the 11 were not re-classified;
  README wording for yesterday's clock, unclosed reminders, interior-now ticks and the PST date.
- Gotchas: `--now` in the past does not hide sessions that started after it, so a back-dated render to show P3
  printed future times; dropped, P3 rests on the 7.2 tests. zsh: `rm` on `$S/*` is blocked by the harness;
  use `"${S:?}"`.

### 2026-11-02 — Step 7.3: Repo view: axis labels never hide each other — Codex
- Did: Reserve occupied axis cells in P4 order, keep fold lengths visible, shorten or drop current-time
  labels on collision, and suppress ticks matching the end/interior-now clock. Keep a blank cell between
  labels for readability. Added five exact-output rendering tests, including interior now overlapping a
  different-minute tick and full/short/dropped label forms.
- Changed: `crates/tui/src/timeline.rs`, `crates/tui/tests/timeline.rs`, `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md. `cargo test --workspace` → exit 0; actual result lines:
  ```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.98s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.66s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, actual output:
  ```text
    Checking multi-link-tui v0.1.0 (workspace crate)
    Checking multi-link v0.1.0 (workspace crate)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.70s
  ```
  (Only the local workspace paths on the Checking lines are omitted.)
  `cargo fmt --all -- --check` → no output, exit 0.
  `git diff --exit-code 26af6f2 -- fixtures/expected/tui/home-phase6.txt fixtures/expected/tui/home-phase6.styles.txt fixtures/expected/tui/repo-phase6.txt fixtures/expected/tui/repo-phase6.styles.txt`
  → no output, exit 0. `git diff --check` → no output, exit 0.
- Review: Claude, 2026-11-02 (diff + Step 7.3): clean, no rework round. Only the step's files changed; tests use
  synthetic fixtures (W10 ok). Re-ran Verify: `cargo test --workspace` → same 36 result lines, all ok (22
  timeline tests); clippy `-D warnings` exit 0; fmt check exit 0; pinned 6.1-golden diff vs 26af6f2 exit 0.
  Notes, not fixed: the blank-cell gap is stricter than P4's wording (kept, readability); tick suppression
  compares `clock(at)`, so a multi-day midnight tick (`Nov 2`) is skipped when the end is exactly 00:00
  (harmless); no test for a tick at the minute of a plain end label in a range ending before now (same code path
  as the tested `HH:MM now` case).
- Gotchas: Explicit ranges do not fold, so their collision tests exercise interior now against ticks and
  the end label. Initial targeted run: 20 passed, 2 failed (adjacent labels joined words); adding blank
  separation and correcting the near-fold test geometry resolved both. Final workspace run includes
  all 22 passing timeline tests. Pre-existing untracked `.claude/` left untouched.

### 2026-11-02 — Step 7.2: TUI: home names its window; clock for the last 24 h — Codex
- Did: Passed the existing report window into home on every render, including after refresh; reused the repo
  period formatter for all three home phrases. Passed the HTML example's window to home. Changed the shared
  clock to use elapsed ages from 0 up to (excluding) 24h, retaining the old local-date rule for future times. Added exact-value tests for 2h/16h
  phrases (including no hidden activity), app 16h/all windows before/after refresh, previous-day 23h59m vs
  exactly 24h in UTC/UTC-8/UTC+5:30, clock times across local midnight, and future timestamps on the same
  and next local dates. Restored all four hook golden files after the review fix.
- Changed: `crates/tui/src/{home.rs,view.rs,app.rs,timeline.rs}` (timeline only exposes `period` within the crate),
  `crates/tui/tests/{home.rs,view.rs}`, `crates/tui/examples/home_html.rs`,
  `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md. `cargo test --workspace` (review round 1 rerun) → exit 0; actual result lines:
  ```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.81s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.66s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
- Verify: `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; final output:
  ```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.80s
  ```
  `cargo fmt --all -- --check` and `git diff --check` → no output, exit 0.
- Verify: `cargo run -q -p multi-link-tui --example home_html -- fixtures/expected/phase6/overview.json
  fixtures/expected/links.json /tmp/multi-link-7.2-r1-home.html --since 16h --offset -8 --size 160x80` → no output,
  exit 0. Python HTMLParser extracted text and asserted the three phrases and absence of `last 2h`; output:
  ```text
All repos · last 16h
4 other sessions, 2 active in the last 16h
active about 13 min in the last 16h, in 3 stretches
  ```
- Verify: `git diff --exit-code 26af6f2 -- fixtures/expected/tui/home-phase6.txt
  fixtures/expected/tui/home-phase6.styles.txt fixtures/expected/tui/repo-phase6.txt
  fixtures/expected/tui/repo-phase6.styles.txt` → no output, exit 0.
- Verify: `git diff --exit-code 6a5282a -- fixtures/expected/tui/` → no output, exit 0; all TUI goldens
  match the requested pre-step commit, including the four restored hook files.
- Review: Claude, 2026-11-02 — 1 Codex fix round. Round 1: `view.rs` `now - at < 24h` also held for future
  times (negative age), so the hook fixture's waiting `state_since` (~7 h after its `now`) read `since 06:08`;
  fixed with a 0 ≤ age guard, future times keep the old local-date rule, hook goldens restored, two tests added.
  Re-review clean. Diff stays within the step's files; no personal paths or real data (W10). Claude re-ran
  Verify: `cargo test --workspace` → every `test result: ok`, 0 failed, exit 0; clippy `-D warnings` → exit 0;
  pinned 6.1-golden `git diff --exit-code 26af6f2` → exit 0; `git diff --exit-code 6a5282a --
  fixtures/expected/tui/` → exit 0; `home_html … --since 16h` → `All repos · last 16h` ×1, `in the last 16h`
  ×2, `last 2h` ×0. Matches what Codex pasted.
- Gotchas: The hook fixture's waiting `state_since` is after its `now`; the initial negative-age comparison
  changed its date incorrectly. Those snapshot updates were a regression, not required P3 changes; all
  four hook files are now restored and the entire TUI golden directory matches 6a5282a. Initial new test
  expectations were also corrected against fixture values (first burst 05:05Z, 2 active hidden sessions,
  13 whole activity minutes) and existing period syntax (`1h20`, not `1h20m`). The example still accepts
  durations only; `--since all` is tested through App. Existing untracked `.claude/` is untouched. Nothing
  staged, committed or pushed.


### 2026-11-02 — Step 7.1: Core: Claude prompt after a system reminder — Codex
- Did: Strip repeated exact leading `<system-reminder>` blocks through the first closing tag, trimming
  whitespace after each, before W3 filtering/normalization. Added synthetic regression cases for both
  string and text-block content: single/repeated reminders, reminder-only fallback, other wrappers,
  missing close, extra tag suffix, attributes, plain text, inline tags and first-close (no nesting) behavior.
  Documented Phase 7 P1–P4 and updated the binding Claude `asked` rule.
- Changed: `crates/claude/src/lib.rs`, `crates/claude/tests/phase7.rs`, `docs/design.md`,
  `docs/output-schema.md`, `PROGRESS.md`. No expected JSON changes were needed: existing fixture tests pass.
- Verify: Rust PATH set per AGENTS.md. `cargo test --workspace` → exit 0; actual result lines:
  ```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.89s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test leading_system_reminders_preserve_prompt_selection_and_normalization ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.68s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, final output:
  ```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.58s
  ```
  `cargo build --release -p multi-link` → exit 0, final output:
  ```text
Finished `release` profile [optimized] target(s) in 4.95s
  ```
  `cargo fmt --all -- --check` → no output, exit 0 (after formatting the new test).
  `git diff --exit-code 26af6f2 -- fixtures/expected/tui/home-phase6.txt fixtures/expected/tui/home-phase6.styles.txt fixtures/expected/tui/repo-phase6.txt fixtures/expected/tui/repo-phase6.styles.txt`
  → no output, exit 0.
  `target/release/multi-link sessions --since all --json | jq -c '[.sessions[]|select(.harness=="claude")] | [length, (map(select(.asked==null))|length)]'`
  → before rebuild `[182,44]`, after rebuild `[182,11]` (both exit 0). Compared with the phase baseline
  `[181,43]`, live data has one additional Claude session and one additional old-rule null; P1 recovers
  33 prompts on this run, leaving the same 11 nulls. Only counts were emitted; no real prompt data saved.
- Review: Claude, 2026-11-02 — clean, no findings, no Codex rounds. Diff stays within the step's files (no
  fixture JSON needed). `strip_prefix("<system-reminder>")` cannot match attributes or `<system-reminder-extra>`;
  `split_once(...)?` skips an unclosed reminder; first closing tag ends the block (no nesting), as P1 says. Tests
  cover all seven Verify cases for string and text-block content. W10: only synthetic text added. Re-ran Verify:
  `cargo test --workspace` all `ok` (same 36 result lines), clippy `-D warnings` clean, pinned 6.1 goldens
  exit 0, real count `[182,11]` — matches Codex's paste.
- Gotchas: Other leading tags still skip the record; no nested-tag parsing. P2–P4 are documented for later
  steps, not implemented here. Existing untracked `.claude/` left untouched.


### 2026-11-02 — Step 6.7: Repo view: `✓ done` label after a finished Codex run — Codex
- Did: Draw completion labels after the lanes, checking the seven cells after the final bar for the
  area edge, bars, link geometry and fold bands. With room, draw a blank plus green `✓ done`;
  otherwise retain the adjacent green `✓`. Link clearance uses geometry so NO_COLOR works too.
  Added exactly three tests: room (including exact fit, done colour and NO_COLOR), edge fallback,
  and a following bar or confirmed/heuristic link fallback (including NO_COLOR).
  Claude review round 1: fixed the off-by-one clearance range to `x+1..=x+7`, including bars, folds
  and link columns; reject `x + 7 >= axis.cells`. Corrected the six-cell-label comment. Exact fit
  now uses end=66 (final `e` at screen column 117); end=67 is covered by the edge fallback test.
- Changed: `crates/tui/src/timeline.rs`, `crates/tui/tests/timeline.rs`, `PROGRESS.md`.
- Verify: Review round 1 rerun; Rust PATH set per AGENTS.md. `cargo test --workspace` → exit 0; actual result lines:
  ```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.19s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.66s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  Actual new-test output:
  ```text
test done_label_without_room_at_edge_keeps_adjacent_check ... ok
test done_label_with_room_uses_done_colour_and_survives_no_color ... ok
test done_label_with_following_bar_or_link_within_six_cells_keeps_check ... ok
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, no warnings; actual final output:
  ```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.63s
  ```
  `cargo fmt --all -- --check` → no output, exit 0.
  Python byte comparison against `git show HEAD:<path>` → exit 0, actual output:
  ```text
repo-phase6.txt: byte-for-byte unchanged from HEAD
repo-phase6.styles.txt: byte-for-byte unchanged from HEAD
  ```
  `git diff --check` → no output, exit 0. Existing immutable text/style test passes.
- Review: Claude, 1 Codex fix round. Round 1 P2: the label `" ✓ done"` is 7 cells but the check covered 6, so a bar
  ending at axis col 67 (width 120) drew `e` at screen col 118, outside the bar area; fixed to x+1..=x+7, exact fit
  is now end=66 and end=67 falls back to `✓`. Files: only the step's list (no `repo*.txt` golden changed; the
  phase6 pair is byte-identical to HEAD). The three Verify tests are present. Claude re-ran Verify: `cargo test
  --workspace` all ok (the three new tests ok), clippy `-D warnings` clean, fmt clean (matches the pasted output).
  Real data (scratch only, release build, `home_html --repo route-impact --since 2h --offset local` at
  2026-11-03T00:43Z, headless-Chrome screenshot): green `✓ done` one blank cell after the 16:02 PST Codex bar.
- Gotchas: Review round 1 found the prior check omitted the final `e`: the six-cell label needs
  seven cells of clearance including its leading blank. Full Verify rerun passes after the fix.
  Initial targeted run: 16 passed, 1 failed because the new link test expected a blank where
  an existing grid tick is drawn. Corrected the expected cells; targeted rerun: 17 passed, 0 failed.
  Real-data screenshot part of Verify skipped at the owner's request; Claude handles it.
  No golden files changed; no staging or commits.

### 2026-11-02 — Step 6.6: Linker: launches in subshells and after heredocs; prompt labels — Codex
- Did: Skip queued heredoc bodies (quoted/plain delimiters, tab stripping, EOF); recognise shell groups and
  the specified stacked wrappers; keep command substitutions opaque and directory moves conservative.
  Add Codex response-item prompt fallback outside imports, retaining event prompt priority.
  Add synthetic parser and end-to-end linker regressions. Document X1–X4. Update only seven `asked`
  values per affected older JSON key, from existing scrubbed fixture placeholders; no real prompt text added.
- Changed: `crates/claude/src/invocation.rs`, `crates/claude/tests/invocation.rs`, `crates/codex/src/lib.rs`, `crates/codex/tests/phase6.rs`, `crates/core/tests/links.rs`, `docs/design.md`, `docs/output-schema.md`, `fixtures/expected/hook/overview.json`, `fixtures/expected/hook/sessions-2h.json`, `fixtures/expected/hook/sessions.json`, `fixtures/expected/overview.json`, `fixtures/expected/sessions.json`, `PROGRESS.md`.
- Verify:
  - before (owner-supplied): links --since all: 124 launches; exact 12, heuristic 78, ambiguous 8, unlinked 26. sessions --since 2h: 1 Codex session, 1 with asked null.
  - after (Claude, 2026-11-02 16:26 PST, release rebuilt from this diff): links --since all: 149 launches; exact 12,
    heuristic 97, ambiguous 8, unlinked 32 (+25 recognised: +19 heuristic, +6 unlinked). sessions --since 2h:
    2 Codex sessions, 0 with asked null (the before session plus the Codex run of this step).
  - `cargo test --workspace` (Rust PATH set per AGENTS.md; exit 0), review-round-1 rerun, actual result lines:
    ```text
    test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
    test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.02s
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
    test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
    test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
    test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s
    test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.66s
    test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
    test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s
    test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
    test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
    test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
    test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
    test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
    test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
    test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    ```
  - `cargo clippy --workspace --all-targets -- -D warnings` → exit 0:
    ```text
        Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
    ```
  - `cargo fmt --all -- --check` and `git diff --check` → no output, exit 0.
  - Read-only count over real Claude main transcripts in all project folders, applying unchanged W3
    session-id / tool-result / first-text / wrapper filters; only aggregate output printed:
    ```text
    {'total': 175, 'no_prompt': 41, 'wrapper_only': 37, 'no_eligible_text': 4}
    ```
    Thus 41 main transcripts still have no prompt: 37 have only wrapper-prefixed candidate text and
    4 lack eligible nonblank text. This counts transcripts, not record-only sessions or a time window.
- Review: Claude, 1 Codex fix round. Files: all in Step 6.6's list; `docs/output-schema.md` was added to the list in
  review (round-1 P2: its `asked` rule and the 6.2 note still said older Codex fixtures get `null`; fixed). Every
  X1–X4 shape is tested, including the exact route-impact launch with a synthetic brief holding `'` and `"`, an
  unterminated heredoc, `$(…)` not a launch, `cd` in a group, stacked wrappers and rejected `env -i` / `timeout
  --signal` / `sudo`; core test links the heredoc + subshell launch by time and keeps the `cd` guard. The 6.1
  goldens (`fixtures/expected/phase6`, `fixtures/expected/tui`) are byte-identical to HEAD. Claude re-ran Verify:
  `cargo test --workspace` all ok, clippy `-D warnings` clean (matches the pasted output). The five older JSON
  keys change only `asked` on Codex rows, to the existing scrubbed `CASE-<name>: <prompt>` strings. W10: added
  lines hold no real prompts, paths or secrets; the one uuid is an existing committed fixture id. Real data
  (scratch only, headless-Chrome render of `home_html --repo route-impact --since 2h`): the Codex run started
  16:02 PST now sits nested under Claude `08ce1b62` with a `╎` matched-by-time line (link `heuristic`, 1 s after the
  call) and its prompt as the label; the selected line reads `handed work to codex at 16:02`. Known limits kept
  (not in X1–X4): `((a << 2))` arithmetic would read as a heredoc; an unquoted `#` comment with an apostrophe
  still breaks quote tracking; the subshell launch's `background` stays `false` (it comes from the tool call).
- Gotchas: Initial compile attempts caught an invalid slice field and a nonexistent test evidence variant;
  corrected. Initial workspace run had 4 CLI key failures from newly populated older `asked` values;
  updated those allowed JSON values only. Initial clippy found two style issues; fixed before final runs.
  Step 6.1 phase6 JSON keys and all text/style goldens remain untouched. Unsupported shell grammar stays
  unsupported; this is a recogniser, not a shell interpreter. Existing untracked `.claude/` left untouched.

### 2026-11-02 — Step 6.5: Visual check and README — Claude
- Did: Extended `crates/tui/examples/home_html.rs` with `--repo NAME` (exact path, or a name that ends exactly one
  repo path; several matches is an error) and `--since` (the CLI's rule, no `all`) so the repo view gets the TUI's
  window, drawn with `render_repo` (fitted, folds on, imports hidden, first lane selected). Rendered the phase6 fixture
  (home + repo-r) to `docs/img/{home,repo}-phase6-fixture.png` (headless Chrome, 2x). Rendered real `~` at
  2026-11-02T22:07:21Z with `--since 2h` and `--since 16h` (home + multi-link repo view) to scratch only and looked
  at them. Wrote `docs/phase6-check.md` (method, fixture images, boards 7c/8c in words with the canvas link, real-data
  differences in words and counts, Verify, pending owner check). Recorded the owner's 2026-11-01 Phase 5 verdict in
  `docs/phase5-check.md`. README: release status, home and repo-view sections with fixture images, keys, limitations
  (W1 gaps, Codex done, `no prompt` on real Codex runs, `last 2h` wording, JSON exports prompt text).
- Changed: crates/tui/examples/home_html.rs, docs/phase6-check.md, docs/phase5-check.md, README.md,
  docs/img/home-phase6-fixture.png, docs/img/repo-phase6-fixture.png, PLAN.md (Review log), PROGRESS.md
- Verify: `cargo test --workspace` → exit 0, 35 `test result: ok`, 0 failed (incl.
  `phase6_matches_immutable_text_and_styles ... ok`, `phase6_repo_matches_immutable_text_and_styles ... ok`);
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, 0 warning/error lines; existing keys vs 1739a67
  with the four fields removed → 7 of 7 equal (overview, sessions, hook/{overview,sessions,sessions-2h},
  phase4/overview, phase5/overview); fixture HTML text vs `home-phase6.txt` / `repo-phase6.txt` → True / True;
  `p6-overview.json` == `fixtures/expected/phase6/overview.json` → True. W10 scan of the changed files: no
  `/Users/`, secrets or ids over 8 chars except the canvas id (recorded exception); the 10 real `asked` strings from
  the scratch JSON checked against all 231 tracked files + new doc → 0 hits. Owner's answer: pending (stop-and-ask).
- Review: adversarial Claude subagent (diff + Step 6.5 + W0–W10, re-ran Verify: same counts): 0 P1, 2 P2, 5 P3, all
  accepted and fixed (PLAN.md → Review log); it also noted Phase 5's committed `docs/img/home-real.png` and
  `board7-target.png`: checked, they show repo names, step titles, counts and times only, no prompts; kept because
  docs/phase5-check.md uses them.
- Gotchas: the example used to hard-code a 2 h repo window; with 16 h JSON that drew a fold labelled `1h58` and no
  bars. Pass `--since` to both the CLI and the example. zsh: pass CLI flags as an array.

### 2026-11-02 — Step 6.4: TUI repo view to board 8c — Codex
- Did: Replaced session-span timelines with recorded-activity swimlanes: W3 labels, W5 status phrases,
  newest-first roots and nested children, confirmed/time-matched link lines, ambiguous candidates at top
  level, completion/waiting marks, selected-session details and hidden-import counts. Implemented the
  fitted axis, largest-three idle-gap folding, integer segment allocation, local-midnight tick ladder,
  explicit zoom/pan and reset, import toggle, full-id selection/refresh/open/fork, scrolling and list
  form below 80 columns. Shared the existing home plan wording and retired its unused short-id helper.
  Added done/child theme roles with exact RGB/indexed palette and contrast tests. Tick-step selection
  stops on the first spacing failure rather than allocating minute ticks for years of history.
  - Fix round 1: Use HH:MM for axis ticks/end/now and header ranges across local midnight; allow
    date labels on multi-day midnight ticks without changing W5 phrases. Select the open hint by
    harness. Clamp zoom-out to the report window and pan within both edges while preserving length.
    Replace the vacuous empty-window assertion with a bar-area cell scan and axis-row assertion.
    Add midnight/header and Codex-selection regressions; extend edge/zoom/reset assertions and
    update the prior navigation test to pan left before now.
- Changed: `crates/cli/src/tui.rs`, `crates/tui/src/app.rs`, `crates/tui/src/home.rs`, `crates/tui/src/lib.rs`, `crates/tui/src/theme.rs`, `crates/tui/src/timeline.rs`, `crates/tui/tests/theme.rs`, `crates/tui/tests/timeline.rs`, `fixtures/expected/tui/timeline-repo-a-zoom.styles.txt`, `fixtures/expected/tui/timeline-repo-a-zoom.txt`, `fixtures/expected/tui/timeline-repo-a.styles.txt`, `fixtures/expected/tui/timeline-repo-a.txt`, `fixtures/expected/tui/timeline-repo-b-amb-zoom.styles.txt`, `fixtures/expected/tui/timeline-repo-b-amb-zoom.txt`, `fixtures/expected/tui/timeline-repo-b-amb.styles.txt`, `fixtures/expected/tui/timeline-repo-b-amb.txt`, `PROGRESS.md`
- Retired (screen replaced; no replacement generated snapshots):
  - `fixtures/expected/tui/timeline-repo-a-zoom.styles.txt`
  - `fixtures/expected/tui/timeline-repo-a-zoom.txt`
  - `fixtures/expected/tui/timeline-repo-a.styles.txt`
  - `fixtures/expected/tui/timeline-repo-a.txt`
  - `fixtures/expected/tui/timeline-repo-b-amb-zoom.styles.txt`
  - `fixtures/expected/tui/timeline-repo-b-amb-zoom.txt`
  - `fixtures/expected/tui/timeline-repo-b-amb.styles.txt`
  - `fixtures/expected/tui/timeline-repo-b-amb.txt`
- Verify: Fix round 1; Rust PATH set per AGENTS.md. `cargo test --workspace` → exit 0;
  253 passed, 0 failed. Actual result lines, in command order (routine per-test lines omitted):

```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.24s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.52s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

  Regression and immutable-golden test output from that run:

```text
test app::repo_tests::app_repo_matches_immutable_target ... ok
test app::repo_tests::explicit_range_after_pan_and_back_with_zero ... ok
test phase6_matches_immutable_text_and_styles ... ok
test phase6_repo_matches_immutable_text_and_styles ... ok
test fitted_range_crossing_local_midnight_uses_clock_labels ... ok
test empty_window ... ok
test selected_codex_opens_in_codex ... ok
```

  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; actual output:

```text
    Blocking waiting for file lock on package cache
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.30s
```

  `cargo fmt --all -- --check` → exit 0, no output. `git diff --check` → exit 0, no output.
  `git diff -- fixtures/expected/tui/{home,repo}-phase6{,.styles}.txt` → no output:
  all four 6.1 goldens untouched. `git diff --cached --stat` → no output; nothing staged.
- Review: Claude, 1 Codex fix round. Files: all inside Step 6.4's list (`crates/tui/src/**`, `crates/tui/tests/**`, `crates/cli/src/tui.rs`, the eight retired `timeline-*` goldens); the four 6.1 goldens are byte-identical to HEAD. Claude re-ran Verify: `cargo test --workspace` → 253 passed, 0 failed (matches the pasted output); clippy `-D warnings` clean. W6–W8 checked in the code: fitted range = first burst..now inside the report window, explicit range turns folds off; at most 3 folds (longest, then earliest), 5-column bands; tick ladder with ≥ 13-column spacing, never at `now`; `│`/`╎` from the parent's bar row to the child's; ambiguous candidates stay top-level with `N possible runs`; `i` shows imports marked `import ·`; list form below 80. Every Verify test is present by name. Round-1 findings (all fixed): (1) P1 axis ticks and header range showed `Nov 1` for times before local midnight (seen on real data at 00:13: `Nov 1  Nov 1  Nov 1  00:13 now`) → HH:MM always, dates only at midnight ticks of multi-day axes; (2) `o open in Claude` hard-coded for Codex lanes → `open in Codex`; (3) zoom/pan could leave the report window (W7) → clamped; (4) a vacuous `empty_window` assertion → checks every bar cell. W10: the diff adds only synthetic 6.1 fixture ids/prompts, no real prompts, paths or secrets. Real data (scratch only): route-impact and multi-link repo views at 120 and 79 columns match board 8c's structure and colours (nested children, ╎ lines at the hand-off, ✓ green, selected line, legend). Readability points for 6.5 / the owner, not fixed here: (a) Codex children on real data mostly read `codex codex · no prompt` — the W3 fallback repeats the harness beside the harness column, and real `codex exec` rollouts get `asked` null (core, 6.2 — worth checking in 6.5); (b) W5 times after local midnight lose the clock (`started Nov 1`, `done Nov 1`), a W5 wording rule, left for the owner; (c) no fold appeared on today's real data (work was continuous), so folds are only seen on the fixture; (d) the legend lists `◆`/`≈` even when none is on screen, as board 8c does.
  Own scope/privacy check:
  only allowed files changed; no PLAN.md/core/schema/fixture-input changes, new dependencies, real prompts,
  real session ids, secrets or personal paths added. All new prompt/id inputs come from the synthetic 6.1
  keys. No stage/commit/push. The pre-existing untracked `.claude/` directory was left untouched.
- Gotchas: Small-height testing found a three-row viewport could hide its selected lane; fixed by dropping
  header space first and covered at heights 3/8/16/40. Development test failures from guessed fixture ids,
  marker columns and trailing spaces were corrected against actual fixture/render values before the final
  workspace gate. No real-data visual/owner check here; that remains Step 6.5. The 6.1 goldens were never
  edited or regenerated, and their comparisons never call either snapshot updater.


### 2026-11-01 — Step 6.3: TUI home to board 7c — Codex
- Did: Replaced Phase 5 bars/axis and combined lanes with W3–W5 home cards, idle sentences and quiet repos.
  Flattened children/merged members for attention and counts; excluded imports; added prompt/step fallbacks,
  clock/date/relative times and evidence-based hand-off outcomes. Enter opens the repo view from every target.
  Kept or rewrote refresh, scrolling, identity, navigation and fresh-open tests; added 120/100/80/60 widths,
  NO_COLOR parity, yesterday's waiting session, import-only repos, all W5 phrases and no generated `?` cases.
  Quiet selections highlight their own entry; narrow quiet entries wrap to keep every target reachable.
  Review round 1: reserved the full status phrase before label space (minimum three cells for a quoted
  ellipsis), retaining word boundaries/closing quotes and dropping branches below 80. Restored the red FR17
  header count after stale and a single warning line under each affected card's session rows. Footer drops
  the imports note before navigation keys and keeps the timezone until only move/quit remain. Added three
  regression tests covering every attention row's status at 120/100/80/60, warning text/colour/position,
  and all navigation keys plus timezone at 80. Updated the four sessions/hook home text/style snapshot pairs;
  the Phase 6 goldens and Phase 4 snapshot pair are unchanged by this round.
  Review round 2 (final): corrected header drop priorities to hand-offs/repos, done-or-unknown, idle,
  shared checkouts, then working now. Calculated one status column per card from the largest phrase/branch
  space requirement so labels shrink consistently. Added regressions for the home.txt fixture's 100-column
  working count and every attention card's shared phrase column at 80/60 (also column 66 at 120).
  Refreshed only `home.txt` / `home.styles.txt` and `home-full.txt` / `home-full.styles.txt` this round.
- Changed: `PROGRESS.md`, `crates/tui/src/app.rs`, `crates/tui/src/home.rs`, `crates/tui/src/view.rs`, `crates/tui/tests/home.rs`, `crates/tui/tests/theme.rs`, `crates/tui/tests/view.rs`, `fixtures/expected/tui/home-full.styles.txt`, `fixtures/expected/tui/home-full.txt`, `fixtures/expected/tui/home-hook-full.styles.txt`, `fixtures/expected/tui/home-hook-full.txt`, `fixtures/expected/tui/home-hook.styles.txt`, `fixtures/expected/tui/home-hook.txt`, `fixtures/expected/tui/home-phase4.styles.txt`, `fixtures/expected/tui/home-phase4.txt`, `fixtures/expected/tui/home-phase5.styles.txt`, `fixtures/expected/tui/home-phase5.txt`, `fixtures/expected/tui/home.styles.txt`, `fixtures/expected/tui/home.txt`
- Retired: `fixtures/expected/tui/home-phase5.txt`, `fixtures/expected/tui/home-phase5.styles.txt`, and
  `crates/tui/tests/home.rs::phase5_matches_immutable_text_and_styles`. Replaced the render test with
  `phase6_matches_immutable_text_and_styles` (no update path); retargeted the style-dumper test to Phase 6.
  Refreshed earlier home snapshots; left every 6.1 golden unchanged.
- Verify: Rust PATH set per AGENTS.md. Initial targeted home run: 15 passed, 2 failed because the old
  home/home-full header style snapshots still expected the shared-checkout count. Refreshed those two pairs;
  each exact snapshot test then passed (1 passed, 0 failed). No Phase 6 golden update was attempted.
  `cargo test --workspace` (review round 2 rerun) → exit 0; 253 passed, 0 failed.
  Actual result lines pasted below (all suites, in command order; routine per-test lines omitted):

```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.32s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.68s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.52s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, actual completion output (build paths omitted for W10):

```text
    Blocking waiting for file lock on build directory
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.58s
```

  `cargo fmt --all -- --check`, `git diff --check`, and
  `git diff --exit-code -- fixtures/expected/tui/home-phase6.txt fixtures/expected/tui/home-phase6.styles.txt`
  → exit 0, no output. The immutable text/style render test also passed in the workspace run.
- Review: Claude, 2 Codex fix rounds. Files: all inside Step 6.3's list; the 6.1 goldens are byte-identical to HEAD
  (`git diff --quiet`). Round 1: (1) at 80 columns the status phrase was cut (`waiting on you · since…`) while the label kept
  room, `home.rs` draw_session; (2) the V8 `⚠ N shared checkouts` header part and the card warning line were dropped (FR17
  regression), draw_header; (3) at 80 the footer dropped `enter repo view` but kept the imports note, draw_footer. Round 2:
  (1) the 100-column fixture header then lost `● 2 working now` to the shared-checkout part; (2) status phrases started at
  different columns within one card at 80/60. All fixed with tests; re-rendered at 80/60 by Claude: phrases whole and aligned.
  Checked against design.md (6.1 contract): busy duration from the last burst start and `codex state unknown` are the
  contract's wording; `made` is newest first, so `made.first()` is the latest step (W3). W5 phrases, clock/date/relative
  times, import-only repo, yesterday's waiting session and no generated `?` (all labels and steps blanked, 4 widths) are
  tested. Behaviour tests: removed Phase 5 lane/split/combined tests went with the lanes; refresh, scroll, identity and
  timeline-return tests were rewritten (`refresh_hides_and_reveals_imported_attention_rows_without_leaving_repo`,
  `navigation_open_refresh_and_zoom_preserve_identity`, `enter_opens_cards_summaries_and_quiet_repos_and_escape_restores_identity`).
  Verify re-run by Claude: `cargo test --workspace` → all `ok`, 253 passed, 0 failed (matches Codex's last paste);
  clippy `-D warnings` clean. Real render (release build, real `~`, `home_html --size 120x40 --offset local`, scratch only,
  compared with board 7c): same cards, sentence, quiet line, colours and footer; readability notes in "Current state".
  W10: added prompts are the synthetic phase6 ones; ids in tests are synthetic fixture ids; no real prompts, personal paths
  or secrets in the diff. Codex's own W10 check: added prompt text is
  synthetic and comes from the existing Phase 6 fixtures; test ids and repo paths are synthetic. No real agent
  data was read or written. No new dependencies, core/CLI implementation edits, staging or commits.
- Gotchas: The target's `enter shows all` wording is retained, though 6.4's repo view will show only activity
  in the window. The repo screen remains the old timeline until 6.4. No golden mismatch remains. Pre-existing
  untracked `.claude/` was left untouched.

### 2026-11-01 — Step 6.2: Core: imports, activity, asked, state_since — Codex
- Did: Added the file-order imported-turn state machine without changing legacy turn/state parsing; collected Claude main/subagent and Codex activity independently of legacy timestamps; normalized first prompts; built sorted/deduplicated 300-second bursts with window clipping; copied all four fields into overview rows. Added every edge case listed in Step 6.2 Verify, including second-pass cache invalidation, synthetic prompt filtering, multibyte truncation and winning state evidence.
- Changed: `PROGRESS.md`, `crates/claude/src/lib.rs`, `crates/claude/tests/phase6.rs`, `crates/codex/src/lib.rs`, `crates/codex/tests/phase6.rs`, `crates/core/src/open.rs`, `crates/core/src/overview.rs`, `crates/core/src/scanner.rs`, `crates/core/src/sessions.rs`, `crates/core/tests/keys-before-phase6.json`, `crates/core/tests/overview.rs`, `crates/core/tests/phase6.rs`, `crates/core/tests/sessions.rs`, `crates/model/src/lib.rs`, `crates/model/src/time.rs`, `crates/model/tests/model.rs`, `fixtures/expected/hook/overview.json`, `fixtures/expected/hook/sessions-2h.json`, `fixtures/expected/hook/sessions.json`, `fixtures/expected/overview.json`, `fixtures/expected/phase4/overview.json`, `fixtures/expected/phase5/overview.json`, `fixtures/expected/sessions.json`
- Verify: Rust PATH set per AGENTS.md. `cargo test --workspace` → exit 0, 35 `test result: ok` lines, 255 passed,
  0 failed (incl. `core/tests/phase6.rs`: phase6 sessions/overview equal their keys on cold and warm passes, every
  existing key equals its committed value with the four fields removed); `cargo clippy --workspace --all-targets --
  -D warnings` → `Finished`, no warnings. (Codex pasted the full ~470-line test output; Claude shortened it to these
  result lines after re-running both commands and getting the same 35 result lines.)
- Review: Claude, 0 fix rounds. Files: all inside Step 6.2's list (`keys-before-phase6.json` is under `crates/core/tests/`). W0: an independent script stripped the four fields from each of the 7 changed keys and compared them with `git show HEAD:` → all equal; phase6 keys and TUI goldens unchanged. W1: sort, dedup, split at > 300 000 ms, clip to `[now − since, now]`; `session_meta`, imported-turn and other-`sessionId` records excluded; subagent records are checked against the parent id from the path. W2: the turn state machine matches output-schema (inclusive matching complete, the next `task_started` closes, EOF closes, unmatched complete keeps it open); an all-import rollout gets `[]` / null / no completion. W3: Claude skips inherited, tool-result, blank and `<…>` records and uses the first text block; Codex takes the first `user_message` outside imports; 119 chars + `…`. W5: hook `state_since` uses the same winning record (max `(at, line)`) the scanner uses for `waiting`; record waiting/stale → `statusUpdatedAt`; Codex Unknown/TurnsComplete → last real `task_complete`. Every Verify-listed unit test is present and asserts real values. W10: new prompts are `Synthetic …` / `界`; key `asked` values are the scrubbed `CASE-…: <prompt>` placeholders; no new personal paths, secrets or real ids. Verify re-run by Claude matches Codex's paste. Accepted: the localized `large_enum_variant` allow on `Row`. Noted, not changed (contract is silent): a Codex `user_message` that is blank gives `asked` `""`, not null.
- Gotchas: Kept `Row` inline with a localized, explained `large_enum_variant` allowance: the additive fields cross clippy's size threshold; boxing would change the public API and require edits outside this step. No dependency or TUI rendering changes. A silent tool over five minutes remains a gap by W1. Initial runs exposed expected stale answer-key/literal assertions and the enum lint; those were corrected before the successful Verify runs above.

### 2026-11-01 — Step 6.1: Contract and target screens — Claude
- Did: Wrote W0–W10 into `docs/design.md` ("Phase 6 decisions", FR10 reworded to the swimlane repo view, target
  screen rules for home and repo view, palette additions with contrast, differences from boards 7c/8c) and the
  four additive fields plus the Phase 6 fixture results into `docs/output-schema.md`. Built the synthetic root
  `fixtures/sessions-phase6/` (6 Claude transcripts + 1 subagent file, 8 session records, 5 Codex rollouts,
  repo-p and repo-r PLAN/PROGRESS; prompts invented). Took the CLI output at d4d862a for `sessions` and
  `overview` (`--since 2h`) and added the four fields by hand per session; an independent scratch script
  recomputed them from the fixture lines and caught one hand error (6002 has 4 bursts, not 3), fixed. Drew the
  two 120×40 styled goldens with a scratch script from the written rules, rendered them to HTML and checked them
  by eye against boards 7c/8c. Wrote the shape test. While writing the keys, clarified W1: the Codex
  `session_meta` header is not activity (PLAN.md W1 edited, logged).
- Changed: `docs/design.md`, `docs/output-schema.md`, `fixtures/sessions-phase6/**`,
  `fixtures/expected/phase6/{sessions,overview}.json`, `fixtures/expected/tui/{home,repo}-phase6.txt`,
  `fixtures/expected/tui/{home,repo}-phase6.styles.txt`, `crates/tui/tests/phase6_shape.rs`, `PLAN.md` (W1, 6.2
  Verify tests, Review log)
- Verify: `cargo test --workspace` → 32 `test result: ok`, 0 failed (incl. `phase6_shape`: 3 passed);
  `cargo clippy --workspace --all-targets -- -D warnings` → clean; keys parse; cross-check script → `checked 13
  mismatches 0`; a deleted styles line makes the shape test fail (mutation check, restored).
  Traceability (tags = last 4 id digits): repo-p ← `.claude/projects/-home-dev-repo-p/{6001,6002}.jsonl`,
  `.claude/sessions/{76001–76004}.json`, Codex `rollout-…-{6021,6022}.jsonl`, `repos/repo-p/`; repo-q ← `6031.jsonl`,
  `76031.json`; repo-r ← `{6011,6012}.jsonl`, `6011/subagents/agent-a6011sub0000001.jsonl`, `76011.json`,
  `76012.json`, `rollout-…-{6013,6014}.jsonl`, `repos/repo-r/`; repo-s ← `76041.json`; repo-t ← `rollout-…-6051.jsonl`.
  Links ← the Bash `tool_use` in 6002 (+ its `session id:` tool result) and 6011. Contrast (new roles): green 9.78
  screen / 9.39 child / 7.18 selected (256: 10.79 / 9.82 / 8.72); every text colour on the child row ≥ 6.04 (256 ≥ 5.72).
- Review: adversarial Claude subagent (given the diff, Step 6.1, W0–W10 and board descriptions): recomputed every
  key value and golden cell, all matched; 0 P1, 3 P2, 6 P3, all doc or test wording, all accepted (one partly; PLAN.md
  → Review log). Eye check, home vs 7c: header same parts and colours, `◆ 1 need you` instead of the hook note
  (fixture has a waiting session) and V8 counts instead of in-window idle; cards: same rows and columns (title,
  blank, session rows with prompt at 15 / status at 66 bold / branch right, rest line, hand-off line), first card
  yellow with a waiting row from yesterday, V4 tint kept; idle sentence and quiet line as 7c with `last active` /
  `done`; footer `enter repo view` and `times UTC-8`. Repo vs 8c: header, axis (`session`, 5-min ticks, `HH:MM now`),
  selected parent with `▌`, nested child with `└▸` on the child background, `╎` link line, `✓`, second session,
  hidden-imports row, selected line, legend and footer all in place; differences (PLAN wins): fold 5 columns with
  `42m` in the axis row, W5 sub lines, "active" not "worked", legend `│ confirmed link` + `✓ done`, `o open` +
  `i imports` in the footer, Phase 5 selected colour and track-coloured grid. W10: every new file has synthetic
  prompts and ids only (pattern `<tag>0000-…-00000000<tag>`), no `/Users/` paths or secrets (`rg` scrub empty);
  the canvas artifact id is a recorded exception.
- Gotchas: zsh does not word-split `$F`; pass CLI flags as an array. A synchronous `codex exec` makes its Claude
  parent silent for the run's length, so the parent's bursts split around it (W1 limitation, in the fixture).
  Kept for the owner: `· enter shows all` on a card opens the repo view, which shows only in-window sessions.

### 2026-10-30 — Step 5.5: Visual check on real data and README — Claude
- Did: Wrote `crates/tui/examples/home_html.rs` (overview + links JSON → 120×40 home buffer → HTML, one
  full-height block per row so backgrounds touch like terminal rows; truecolor only, errors on any other
  colour; DIM blends text toward its background; wide symbols span their cells; `--offset H[:MM]|local`).
  Rendered the phase5 fixture (text equals `home-phase5.txt`) and real `~` at 2026-10-31T00:08:56Z
  (`--since 2h`). Opened each page and board 7 (`project/HomeTarget.dc.html` read from the canvas artifact
  with the Artifact tool, rendered locally without the canvas runtime) in the built-in browser pane and
  screenshotted them; committed PNGs captured from the same HTML with local headless Chrome (pane
  screenshots are small JPEG previews). Checked every image: repo names, step titles, counts, times only.
  Wrote `docs/phase5-check.md` (method, images, 18 differences from board 7, Verify, owner check pending).
  README home-screen section rewritten from these outputs with the real image. Recorded the owner's
  Phase 4 verdict in `docs/phase4-check.md` and closed Phase 4 as not meeting the need.
- Changed: `crates/tui/examples/home_html.rs`, `docs/phase5-check.md`, `docs/phase4-check.md`, `README.md`,
  `docs/img/{home-real,home-phase5-fixture,board7-target}.png`, `PROGRESS.md`. (`.claude/launch.json`, a
  local static server for the pane, is untracked and not committed.)
- Verify (whole phase): Rust PATH set per AGENTS.md.
  ```text
  cargo test --workspace → exit 0, 31 `test result: ok`, 0 failed (re-run after review fixes: 31 ok)
    test phase5_matches_immutable_text_and_styles ... ok
    test older_keys_only_add_made_and_hook ... ok
  cargo clippy --workspace --all-targets -- -D warnings → exit 0, 0 warning/error lines (re-run after fixes)
  existing keys vs 7d399ba with made/hook removed → overview.json, hook/overview.json, phase4/overview.json
    all equal-after-strip
  p5.html text vs fixtures/expected/tui/home-phase5.txt → golden-text-equal
  ```
  Screenshots in `docs/phase5-check.md`. Owner's answer: not yet (stop-and-ask).
- Review: adversarial Claude subagent (diff + Step 5.5 + images + scratch inputs), 9 findings. Accepted
  1–8: (1) inline `<pre>` left page-colour strips between rows, hiding that terminal tracks merge into one
  slab — renderer fixed, images retaken, now difference 1; it also removed a false "blank row inside the
  card" claim; (2) idle count: 7 quiet lanes + 5 out of window, not 8 + cap; (3) "14 links in the window"
  false — reviewer said 6, Claude's own count from `real-links.json` is 5 (one at 22:08:41 is before the
  22:08:56 window start), doc says 5 and logs the header-count issue; (4) phase4-check intro still said
  "not closed"; (5) README omitted the window condition; (6) selection and 256-colour not shown by images —
  listed; (7) card-title count colour difference added; (8) non-RGB fallback, wide chars, DIM on bg,
  fractional offsets — fixed in the example. Rejected for this step (9): the verdict quote differs from
  PLAN.md/design.md wording; PLAN.md is outside 5.5's files and only the owner knows the exact words —
  flagged in Current state.
- Gotchas: zsh does not word-split `$F` flag strings. The phase5 key was made with `--since all`; with
  `--since 2h` the header reads `9 done or unknown`, not the golden's 10. The built-in pane cannot screenshot
  `file://` pages; a `127.0.0.1` static server from `.claude/launch.json` works.

### 2026-10-30 — Step 5.4: TUI home to board 7 — Codex
- Did: Added a pure home lane model that flattens merged members, combines eligible Claude/Codex
  pairs, ranks attention outside the window/cap, orders quiet history, and exposes hidden rows.
  Rebuilt home with rounded tinted cards, explicit theme colours, step labels, independent overlaid
  bars, plain link words, honest idle summaries, local dates/offsets, word-boundary truncation and
  selection scrolling. App uses the same targets; enter expands idle repos, splits combined lanes,
  or reveals history; o opens the combined Claude parent. Refresh/return preserves session identity.
  CLI supplies the theme; App reads the offset once per successful refresh, falling back to UTC.
- Changed: `Cargo.toml`, `Cargo.lock`, `crates/tui/src/{lib,view,home,app}.rs`,
  `crates/tui/tests/{home,view}.rs`, `crates/cli/src/tui.rs`,
  `fixtures/expected/tui/home{,-full,-hook,-hook-full,-phase4}{,.styles}.txt`, `PROGRESS.md`.
  The phase5 text/style acceptance pair and PLAN.md are unchanged. No files staged or committed.
- Verify (round-1 rerun): Rust PATH set per AGENTS.md. `cargo test --workspace` → exit 0; actual result lines:
  ```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.95s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  Acceptance test in that run: `test phase5_matches_immutable_text_and_styles ... ok` (direct text
  and style equality; deliberately has no snapshot-update path). Tests also pin yesterday's waiting
  row, nine quiet rows capped at eight, child ending after parent, differing/nullable steps, placing
  links, independent bars and clip markers, midnight and offsets −7/0/+5:30, widths 120/100/80/60/40
  at height 10, NO_COLOR tracks, header counts/partial evidence, the three enter cases, combined o,
  and selection across refresh/timeline. Legacy home snapshots regenerated for the new design.
- Verify: `cargo fmt --all -- --check` → no output, exit 0;
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, actual output:
  ```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.13s
  ```
  `git diff --check` → no output, exit 0;
  `git diff --name-only -- fixtures/expected/tui/home-phase5.txt fixtures/expected/tui/home-phase5.styles.txt`
  → no output, exit 0.
- Review: Claude (2026-10-30), 1 fix round. Round 1 (4 findings, all fixed): header cut inside parts (`3 repos · 4…`, `●…`); lane labels ended `·…` / `→…`; footer cut keys and lost `q quit`; plan wording cut to one word (`latest…`). Claude re-read the phase5 renders at 120/100/80/60/40 cols after the fix: 120 equals the golden, narrower widths drop whole fields, no dangling separators. `git diff 05cd20b` on `home-phase5.txt` / `.styles.txt` → empty (byte-identical). Every changed file is in the step list. Checked `local_offset` in a multi-threaded process: time 0.3.55 no longer refuses (soundness check removed in 0.3.37). Claude re-ran Verify: `cargo test --workspace` → 31 `test result: ok`, 239 passed, 0 failed (matches the paste); clippy `-D warnings` clean; fmt clean. Not fixed (left for 5.5 on real data): waiting label loses its time at 80 cols; no "more below" hint when lanes exceed the height.
- Gotchas: One offset per frame uses the offset at now across DST, as specified in V7. Timeline
  rendering is unchanged. Initial verification exposed legacy tests that selected by core's old
  row order/merged UI; these now select via the home model and test the replacement interactions.
  A new split-bar test initially put its quiet child beyond now (correctly hidden by V3); it now
  checks future clipping while combined, then moves the child inside the window for the split check.


### 2026-10-30 — Step 5.3: TUI theme and styled goldens — Codex
- Did: Added a pure V4 theme selected from passed COLORTERM / NO_COLOR values, with every RGB and
  xterm index fixed to the design palette. CLI reads both variables once; nonempty NO_COLOR clears
  colours and DIM after rendering, preserving glyphs, dotted tracks and selection. Existing coloured
  home and timeline renders are unchanged; palette application awaits 5.4.
  Added a shared maximal-run style dumper: uppercase RGB, Debug names for other colours, every
  modifier, blank-cell normalization, full coverage, relative coordinates. Generated nine legacy
  style goldens once using `UPDATE_STYLE_SNAPSHOTS=1 cargo test -p multi-link-tui` (exit 0).
  Added exact palette, environment precedence, 48 text/background contrast pairs, NO_COLOR home / timeline,
  every-modifier and blank normalization tests. Parsed the fixed 120×40 Phase 5 text/style pair into
  a buffer and verified byte-for-byte reproduction of its style file.
- Changed: `crates/tui/src/{lib.rs,theme.rs}`, `crates/tui/tests/{home.rs,timeline.rs,theme.rs,support/mod.rs}`,
  `crates/cli/src/tui.rs`, nine new `fixtures/expected/tui/*.styles.txt` (home, home-full, home-hook,
  home-hook-full, home-phase4, timeline-repo-a, timeline-repo-a-zoom, timeline-repo-b-amb,
  timeline-repo-b-amb-zoom), `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md. `cargo fmt --all -- --check` → no output, exit 0.
  `cargo test --workspace` → exit 0; actual target/result lines:
  ```text
     Running unittests src/main.rs (target/debug/deps/multi_link-f69ed8de1bb427a5)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
     Running tests/cli.rs (target/debug/deps/cli-89cbdb7a788bfb89)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.92s
     Running unittests src/lib.rs (target/debug/deps/multi_link_claude-002e7034494a8445)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/fixtures.rs (target/debug/deps/fixtures-aaba9febfe95def0)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
     Running tests/invocation.rs (target/debug/deps/invocation-0eaccf444a8ae875)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
     Running unittests src/lib.rs (target/debug/deps/multi_link_codex-204a903019c5c359)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/fixtures.rs (target/debug/deps/fixtures-b55b5b49100073b1)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running unittests src/lib.rs (target/debug/deps/multi_link_core-9171feebdd5225a7)
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
     Running tests/hook.rs (target/debug/deps/hook-0019750ae5ccafd6)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
     Running tests/links.rs (target/debug/deps/links-fc0002f57806efaa)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
     Running tests/overview.rs (target/debug/deps/overview-92debc92993de7c8)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.48s
     Running tests/repo_probe.rs (target/debug/deps/repo_probe-75c327fc37ed8c3d)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
     Running tests/sessions.rs (target/debug/deps/sessions-06ff65e7cbcbd48c)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
     Running unittests src/lib.rs (target/debug/deps/multi_link_hook-466f5073014ef46a)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/hook.rs (target/debug/deps/hook-e8fbeeb40f3e2efd)
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
     Running unittests src/lib.rs (target/debug/deps/multi_link_model-0c1ee82a039f47e5)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/model.rs (target/debug/deps/model-1d2bf893dec8dc40)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running unittests src/lib.rs (target/debug/deps/multi_link_plan-ef4f88aa29a57d10)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/fixtures.rs (target/debug/deps/fixtures-b74ce1dc050185a3)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/multi_link_tui-240b89d3261ac0d5)
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/home.rs (target/debug/deps/home-7b5aca6f3e439c8d)
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/theme.rs (target/debug/deps/theme-fc7945a11b97b413)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/timeline.rs (target/debug/deps/timeline-4b1fd284057fee64)
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
   Doc-tests multi_link_claude
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_codex
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_hook
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_model
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_plan
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_tui
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; final actual line:
  ```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.91s
  ```
  Byte comparison of every tracked TUI golden against `git show HEAD:<file>` → actual output:
  ```text
PASS: all 11 existing TUI golden files are byte-identical to HEAD (text and Phase 5 styles).
  ```
  `git diff --check` → no output, exit 0.
- Review: Claude, 0 fix rounds. Files all inside the step's list; `home-phase5*.txt` and every tracked text
  golden unchanged (`git diff --quiet HEAD -- fixtures/expected/tui/*.txt`). Palette RGB and 256 indexes equal
  docs/design.md's table exactly; dump format reproduces the 5.1 golden byte for byte (round-trip test). The nine
  new style files show only what the legacy renders use today (default colours, DIM/BOLD, named ANSI in the
  timeline). Claude re-ran `cargo test --workspace` → 30 `test result: ok`, 0 failed (matches the paste);
  `cargo clippy --workspace --all-targets -- -D warnings` clean. Nits left for 5.4: the theme is built but only
  used for the NO_COLOR post-pass (not yet passed to the renderers); NO_COLOR also drops DIM from the legacy
  timeline (glyphs stay); style and text goldens use two different update env vars.
- Gotchas: Legacy snapshots intentionally retain DIM and timeline ANSI colours until 5.4; the new
  palette's text styles use no DIM. NO_COLOR follows nonempty-value semantics (an empty value does
  not enable it). `UPDATE_STYLE_SNAPSHOTS=1` updates only generated render styles; the hand-drawn
  Phase 5 round-trip test always asserts against its existing file and never rewrites it.

### 2026-10-30 — Step 5.2: Core: `made` and `hook` — Codex
- Did: Added SessionRow `made` (distinct parent-link steps by newest `(at, invocation)`, any match;
  always empty on Codex) and OverviewReport `hook` (all valid parsed records, count and maximum `at`).
  Scanner reuses the current refresh's parsed hook data, including cached files, without another read.
  Added zero/one/several-link tests, invocation tie ordering, untagged/unlinked/other-parent cases;
  overview comparisons for all four keys; absent/empty/old/invalid/pending/unreadable hook cases,
  cache reuse and removal/recovery. Updated only the two added fields in the three older keys.
- Changed: `crates/core/src/{overview,hook,scanner}.rs`, `crates/core/tests/overview.rs`,
  `crates/core/tests/overview-before-phase5.json`, `crates/model/src/lib.rs`,
  `crates/model/tests/model.rs`, `fixtures/expected/{overview,hook/overview,phase4/overview}.json`,
  `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md. `cargo test --workspace` → exit 0; actual result lines:
  ```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.97s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
- Verify: `cargo clippy --workspace -- -D warnings` → exit 0; actual output:
  ```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s
  ```
- Verify: `cargo fmt --all -- --check` and `git diff --check` → no output, exit 0.
  `cargo run -q -p multi-link -- overview --root fixtures/sessions-phase5 --now 2026-10-29T23:20:00Z --since all --json`
  followed by parsed JSON equality and baseline comparison to `git show HEAD:fixtures/expected/<key>` →
  exit 0; actual check output:
  ```text
PASS: phase5 CLI overview equals its answer key
PASS: regression baseline equals all three committed overview keys
  ```
- Review: Claude, 2026-10-30, 0 fix rounds. Files all inside the step's list (`overview-before-phase5.json`
  falls under `crates/core/tests/**`). Checked with jq: each of the three older keys with `made` / `hook`
  removed equals its HEAD version, and the saved baseline equals HEAD for all three. `made` is non-empty
  only in phase5 (no tagged links in the older fixtures). `hook/overview.json` `{11, 2026-10-30T06:08:18.433Z}`
  matches an independent count of valid `v: 1` lines in `fixtures/hook/.multi-link/hook/`. Claude re-ran
  Verify: `cargo test --workspace` → 29 `test result: ok`, 0 failed; clippy `-D warnings` clean; release
  binary `overview --root fixtures/sessions-phase5 --now 2026-10-29T23:20:00Z --since all --json` equals
  `phase5/overview.json` (parsed). Notes, not blocking: the "before" check compares to a saved copy
  rather than git, which is fine since it was verified against HEAD here.
- Gotchas: The saved regression baseline contains the three pre-5.2 committed keys (scrubbed fixture
  data), so the test remains independent of git and network access. Hook metadata includes records newer
  than `now` as well as records older than `--since`; neither flag filters the hook count.
  Initial new overview test failed because its hook overlay retained the background task-output file;
  corrected the setup to remove it per the Hook variant contract, then reran the full workspace successfully.
  Permission-denied coverage uses mode 000 on a temporary hook directory and restores permissions before
  assertions; an additional ENOTDIR case checks diagnostics and recovery.

### 2026-10-30 — Step 5.1: Contract and target screen — Claude
- Did: Wrote V0–V10 and the target-screen rules into `docs/design.md` (Phase 5 decisions: layout, bar
  mapping, palette with contrast ratios, styled-golden format, differences from board 7) and the additive
  `made` / `hook` fields plus the Phase 5 fixture results into `docs/output-schema.md`. Built the isolated
  synthetic root `fixtures/sessions-phase5/` in board 7's shape: repo-l has a waiting session from
  yesterday, a busy main session, four step chains 4.1/4.3/4.4/4.5 (4.1 exact by stdout; the others matched
  by time; child 5026 ends after its parent), two no-hand-off sessions, one manual Codex run and one quiet
  session before the window. repo-m has a PLAN.md; repo-n is Codex-only. I predicted every row by hand from
  the fixture design, then checked it against the CLI output at 7d399ba, which agreed. Wrote
  `phase5/{overview,links}.json` from that output and added `made` / `hook` by hand. Drew the 120×40 target
  golden (text + styles) with a scratch script from the written layout, rendered it to HTML and checked it
  by eye against board 7.
- Changed: `docs/design.md`, `docs/output-schema.md`, `fixtures/sessions-phase5/**` (9 Claude transcripts,
  9 session records, 9 Codex rollouts, repo-l and repo-m PLAN/PROGRESS), `fixtures/expected/phase5/overview.json`,
  `fixtures/expected/phase5/links.json`, `fixtures/expected/tui/home-phase5.txt`,
  `fixtures/expected/tui/home-phase5.styles.txt`, `PLAN.md` (Review log only)
- Verify: both keys parse; key == CLI output with `made`/`hook` removed → `overview ... True`, `links ...
  True`; styles cover `4800` cells, text `40` rows; `cargo test --workspace` → `29 test result: ok`, 0
  failed; scrub `rg -l "/Users/|\bsk-|api_key" fixtures …` → nothing (exit 1).
  Traceability: repo-l rows ← `.claude/projects/-home-dev-repo-l/…{5001,5002,5003,5011–5016}.jsonl`,
  `.claude/sessions/{70501,70502,70511–70516}.json`, Codex `rollout-…-{5021,5023,5024,5025,5026}.jsonl`;
  links ← the Bash `tool_use` in 5011/5013/5014/5015 (+ the `session id:` tool result in 5011); plan ←
  `repos/repo-l/`; repo-m ← `70531.json`, `rollout-…-{5041,5042}`, `repos/repo-m/`; repo-n ←
  `rollout-…-{5051,5052}`.
  Contrast (truecolor, on screen / waiting card / busy card / selected): text 13.60/13.04/13.19/9.97,
  grey 7.33/7.03/7.11/5.38, orange 9.27/8.89/9.00/6.80, blue 8.85/8.49/8.59/6.49, yellow
  11.36/10.89/11.02/8.33, red 6.29/6.03/6.10/4.61. 256 (screen / card / selected): text 13.40/12.19/10.83,
  grey 7.88/7.17/6.36, orange 10.30/9.37/8.32, blue 8.57/7.80/6.92, yellow 13.50/12.28/10.91, red
  6.29/5.72/5.08. All ≥ 4.5.
- Review: adversarial Claude subagent (given the diff, Step 5.1 and board 7's HTML); 10 findings, all accepted
  (PLAN.md → Review log). Eye check against board 7, region by region. Header: same parts and colours;
  `◆ 1 need you` instead of the hook note, because the fixture has a waiting row. Axis: `14:20 · 15:20 · 16:20
  now` above the bars, legend on the left. Card: yellow border and tint (tier waiting, not board 7's busy
  orange); title `◆ repo-l  latest step 4.6 Real check and README · Verify unclear` with
  `1 waiting · 1 busy · 6 idle`. Step lanes: a thin dim `━` with a `█` block for 4.1/4.3/4.4/4.5, like board 7;
  an extra waiting lane first, with a lone `◂`; the Codex no-parent lane sits by `started_at`. Below the lanes:
  `1 earlier session · enter shows all` on its own line, then `last: … 15:09 (matched by time)`. Idle lines:
  one bordered line each, with harness/state counts and `last HH:MM`. Footer: keys plus `times in UTC-7`.
- Gotchas: board 7's track colour was almost invisible as a one-cell strip, so the track is `#22262F`.
  The golden assumes the bar window stays 2 h even with `--since all`; that is how the quiet session before the
  window shows up as `1 earlier session`. A logged Verify that is neither pass nor fail reads
  `Verify unclear`, not board 7's `not reported`.

### 2026-10-30 — Step 4.6: Real check and README — Claude
- Did: Built release at `383537e` and `9b1b374` (detached `git worktree` in Claude's scratch dir, removed
  afterwards; this checkout's branch never switched). Re-ran multi-link link counts on real `~`, unpinned and
  with both builds at pinned `--now 2026-10-30T22:15:00Z`. Captured `tui` in a pty (stdlib Python VT
  emulator, 600×100 and 40×100, default window and `--since all`, each before/after pair back to back).
  Wrote `docs/phase4-check.md` (repo names, header lines and counts only) and a README "Home screen"
  section. No code changed.
- Changed: `README.md`, `docs/phase4-check.md`, `PROGRESS.md`, `PLAN.md` (Review log only).
- Verify: link counts, checkout exactly this repo's path, `links --since 24h --json`:
  ```text
  baseline (Phase 4 plan): 43 calls → 42 unlinked / 1 heuristic / 0 ambiguous
  unpinned 22:14 UTC:      49 calls → 12 unlinked / 33 heuristic / 4 ambiguous (12 unlinked, 0 candidates)
  pinned, 9b1b374:         49 calls → 48 unlinked / 1 heuristic / 0 ambiguous
  pinned, 383537e:         49 calls → 12 unlinked / 33 heuristic / 4 ambiguous
  ```
  Home screen, non-blank rows excluding footer (600-row pty): `--since 2h` 28 → 15; `--since all` 246 → 30.
  Real list height (last non-blank row): 31 → 18 and 262 → 46. All 8 tui runs exit 0.
  `cargo test --workspace` → exit 0, 29 `test result: ok`, 0 failed. `cargo clippy --workspace --all-targets
  -- -D warnings` → exit 0, 0 warning/error lines (re-run after touching crate roots).
  Owner's answer: **pending** (stop-and-ask).
- Review: adversarial Claude subagent given only the diff, Step 4.6 and the raw captures; 9 findings
  (0 P1, 4 P2, 5 P3), all accepted and applied (see PLAN.md Review log). It re-derived every pasted count
  from the evidence files and found them matching.
- Gotchas: No pty emulator package is installed (no `pyte`), so a 126-line stdlib emulator was written in
  scratch; it treats every character as one column. zsh does not word-split `$var` in `for` loops. Pinned
  `--now` earlier than live activity shows `… ahead` and `▸` bar ends. Counting non-blank rows
  undercounts screen height (one blank row per repo); both are recorded. Collapsed repos hide their latest
  hand-off line (4 → 2 hand-off lines with `--since all`): a trade-off for the owner to judge.

### 2026-10-30 — Step 4.5: TUI: mini time bars on home lanes — Codex
- Did: Added a fixed right-hand bar column for individual home lanes, including children, using
  `TimeWindow::default_for(report)`. A separate shared axis header below U4's summary shows UTC
  endpoints and labels the span as including idle gaps. Tracks show the window; spans use `━`,
  `◂` / `▸` mark clipped ends, and zero-length spans occupy one cell. No bars on merged or collapsed
  rows, below 60 columns, or where the terminal is too short for summary, axis, body and footer.
  Added exact-cell tests for clipping, zero-length child spans, shared alignment with different
  metadata/indent lengths, out-of-window spans, endpoint inclusion, report-now changes across midnight,
  59/60-column cutoff, offset areas, selection reversal, and merged/collapsed rows. Regenerated all
  five home goldens. Existing long-metadata tests use 160 columns to leave room for both text and bars.
- Changed: `crates/tui/src/home.rs`, `crates/tui/tests/home.rs`,
  `fixtures/expected/tui/{home,home-full,home-hook,home-hook-full,home-phase4}.txt`, `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md. `cargo test -p multi-link-tui` → exit 0; actual result lines
  (unit, home integration, timeline integration, doc tests respectively):
  ```text
  test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
  test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
  test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, actual final line:
  ```text
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.14s
  ```
  `cargo fmt --all -- --check` → no output, exit 0.
  `git diff --check` → no output, exit 0.
  `UPDATE_SNAPSHOTS=1 cargo test -p multi-link-tui --test home` → exit 0; actual result line
  before the final endpoint test was added:
  ```text
  test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
  ```
- Review: Claude (2026-10-30): clean, 0 fix rounds; only step files changed. Goldens checked cell by cell
  (2 h window, 5 min/cell, bars all start at col 76 of 100): phase4 repo-h 40b4 22:30–23:14 → cells 15–23,
  40b3 / 40b2 / 40b1 one cell at 19 / 17 / 9, repo-k 40e1 22:59–23:02 → 20–21; repo-i `codex ×4` and
  `claude ×4` and every collapsed `▸` line have no bar. Hook: repo-c 6358ec7e 19:40–23:12 → `◂` at 0 then
  to 23 (clipped), 00000000…0010 23:05–23:13:30 → 22–23, 11c1f13c / 85a715ba (37h ago) empty tracks; repo-a
  d1f3b188 (6h ahead, after the window) empty; parents 80cce3e4 / ef72378c and their `└` children share the
  column despite different indents. Accepted deviation: the axis is its own line, not on the header line
  (logged in PLAN.md Review log). Claude re-ran `cargo test -p multi-link-tui` → same 24/24/22/0 ok;
  `cargo test --workspace` → 29 `test result: ok`, 0 failed; clippy `-D warnings` and fmt clean.
  Codex's own note: Codex inspected all five
  golden diffs: summary counts retained; phase4 repo-h's four lanes and repo-k's stale lane have bars;
  repo-i's two merged rows and all collapsed summaries have none. Hook goldens include repo-a's
  parent/child bars; repo-c's old sessions have empty tracks, its long busy span clips left. The short
  hook golden now hides five lines rather than four because the axis consumes one body row.
- Gotchas: Bar width is 24 cells, shrinking to 12–23 cells at widths 60–71, with at least 47 text
  cells plus a gap. Long lane metadata clips before the bar column; full timeline remains available.
  Initial focused test run had 17 passed / 1 failed because the synthetic child cloned its parent's
  busy state but the summary expectation counted only the parent; corrected the expected count to 2.
  No PLAN.md changes, staging, commits, or agent-data writes. Real-screen acceptance remains Step 4.6.

### 2026-10-30 — Step 4.4: TUI: header summary and collapsed quiet repos — Codex
- Did: Added U4 summary from all overview rows, children and merged members; full hand-off total
  uses `Refresh.links`. Quiet repos render one selectable summary with step and Verify before the
  path; stale sessions (including children and merged members) prevent collapse. Repo expansion
  and merged expansion use separate sets; refresh keeps expansion by repo path and selects a
  newly collapsed summary instead of a hidden session. Removed child-count text for Codex merges.
  Added Phase 4 golden (repo-i expanded with both merges), navigation/refresh/stale tests, explicit
  header totals and selection-highlight coverage. Fixed Claude's review findings: Enter expands
  the quiet repo in the CLI safety test; the timestamp is included only when the entire header fits;
  warnings/repos/sessions/hand-offs use `plural`; shared stable identity capture/restore keeps
  refresh and timeline Escape in the same repo with first-visible-target fallback and clamping.
  Added regression tests for busy/stale expansion, timeline visibility changes and singular counts;
  reflowed the render_home doc comment.
- Changed: `crates/tui/src/{app.rs,home.rs,lib.rs}`, `crates/tui/tests/home.rs`,
  `crates/cli/src/tui.rs` (test module only),
  `fixtures/expected/tui/{home.txt,home-full.txt,home-hook.txt,home-hook-full.txt,home-phase4.txt}`,
  `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md. `cargo fmt --all` → no output, exit 0.
  `UPDATE_SNAPSHOTS=1 cargo test -p multi-link-tui --test home` → exit 0; actual result:
  ```text
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
  ```
  Compared all five home goldens against copies of the pre-review working tree: every line after
  the header is identical (79 lines in each full golden; 29 in each other golden). Snapshot mode
  was limited to the home test target; timeline goldens were not regenerated.
  Required `cargo test --workspace` → exit 0; actual result lines, in execution order:
  ```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.99s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.65s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  Includes `drawn_finished_but_fresh_running_never_resumes_and_errors_never_execute`,
  `refresh_keeps_newly_expanded_repo_summary_in_the_same_repo`,
  `timeline_escape_restores_home_identity_after_repo_visibility_changes`,
  `header_counts_hidden_members_children_and_the_full_link_report` (100/160 columns), and
  `header_uses_singular_counts`, all `ok`.
  Required `cargo clippy --workspace -- -D warnings` → exit 0; actual output:
  ```text
    Blocking waiting for file lock on build directory
    Checking multi-link-tui v0.1.0 (/Users/jarednarvaez/multi-link/crates/tui)
    Checking multi-link v0.1.0 (/Users/jarednarvaez/multi-link/crates/cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.31s
  ```
  `git diff --check` → no output, exit 0.
- Review: Claude, 2026-10-30, 1 fix round. Round 1 findings (all fixed): CLI test needed `enter` on
  the collapsed repo (planner widened Files to `crates/cli/src/tui.rs` test module); P2 header clipped
  mid-timestamp at 100 cols; P3 `1 warnings`; P3 refresh fallback to a raw index when a selected
  collapsed repo expands; P3 timeline `esc` restoring a raw index; doc wrap. No files outside the step.
  Claude re-ran Verify: 29 `test result: ok`, 0 failed; clippy `--all-targets -D warnings` and fmt clean.
  Goldens checked against the overview keys (state counts per group via jq, tiers, `concurrency`, and
  13 links in `links.json` / `hook/links.json`): `home.txt` / `home-full.txt` header 0/2/0, 1 warning,
  8 repos, 28 sessions, 13 hand-offs; repo-c busy 00000000…0010, 6358ec7e…cedc, unknown 11c1f13c…3b43,
  85a715ba…66f9, FR17 warning, hand-off 11c1f13c → - unlinked; collapsed project (1 idle), repo-a
  (6 unknown, step 0.4 Verify ?), repo-b (9 unknown, step 3), repo-a-wt (2 unknown), scratch (1 idle),
  repo-d (2 idle · 1 unknown), repo-e (2 unknown). `home-hook*.txt` header 2/2/0, 29 sessions; repo-a
  waiting d1f3b188, codex 01a189e5, 80cce3e4 └ 01a189e4, ef72378c └ 01a189e2 (2 hand-offs), bb792d9c,
  hand-off 80cce3e4 → 01a189e4 exact; repo-c waiting 11c1f13c, busy 00000000…0010, 6358ec7e, unknown
  85a715ba, warning, hand-off; collapsed project, repo-b, repo-a-wt, scratch, repo-d, repo-e (short golden
  clips after scratch). `home-phase4.txt` header 0/1/1, 0 warnings, 5 repos, 18 sessions, 0 hand-offs;
  repo-h busy …40b4 then unknown …40b3/…40b2/…40b1 (no merge); repo-j collapsed 1 idle, step 0.4
  Verify ?; repo-k stale …40e1 expanded; repo-g collapsed 4 unknown; repo-i expanded `codex ×4` (no
  child text) then `claude ×4`. All match. Approved.
- Gotchas: Standalone `render_home` lacks a link report and displays `? hand-offs`; callers
  needing full counts use `render_home_with_options`. App always supplies the full report.
  The old CLI scope blocker is resolved by the planner's test-only allowance. No production CLI
  code changed. PLAN.md's existing planner edit is untouched. Nothing staged or committed.

### 2026-10-30 — Step 4.3: Core: `codex ×N` merge — Codex
- Did: Merge quiet top-level rows independently by harness at four members, retaining each harness's
  first position and member order. Added required MergedRow `harness`, harness-specific home labels,
  and refresh selection keyed by repo plus harness for merged rows. Added threshold/active-row,
  interleaved two-merge, Phase 4 answer-key, model round-trip, and App selection/expansion coverage.
- Changed: `crates/core/src/overview.rs`, `crates/core/tests/overview.rs`, `crates/model/src/lib.rs`,
  `crates/model/tests/model.rs`, `crates/tui/src/app.rs`, `crates/tui/src/home.rs`,
  `fixtures/expected/overview.json`, `fixtures/expected/hook/overview.json`, `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md. `cargo test --workspace` → exit 0; actual result lines and new tests:
  ```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.24s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test overview::tests::codex_merge_threshold_excludes_active_rows ... ok
test overview::tests::interleaved_harnesses_merge_independently_at_their_first_member ... ok
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test phase4_overview_matches_answer_key ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test app::tests::refresh_preserves_selected_merged_harness_and_enter_expands_both ... ok
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  `cargo clippy --workspace -- -D warnings` → exit 0, no warnings; actual final output:
  ```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.20s
  ```
  `cargo fmt --all -- --check` and `git diff --check` → no output, exit 0.
- Review: Claude, 2026-10-30, round 1, clean (no fix round). Diff touches only the step's files (no
  `fixtures/expected/tui/home*.txt` change; existing goldens have only `claude ×N`, so none needed).
  Merge matches output-schema Overview → Merge: per harness, threshold 4, slot = first member, order kept,
  waiting/busy/stale excluded; Codex children are never top-level so cannot merge. App keeps (repo, harness)
  for merged rows. Claude re-ran Verify: `cargo test --workspace` → 29 `test result: ok`, 0 failed, same
  counts and new tests as pasted above; clippy `-D warnings` clean; CLI `overview --root
  fixtures/sessions-phase4 --now 2026-10-29T23:15:00Z --since all --json` == phase4 key (parsed: True).
  Non-blocking, for 4.4: `crates/tui/src/home.rs:359` prints `codex ×4  0 codex children` on a Codex merged
  row (children count only means something for Claude); 4.4 owns that line's rendering and goldens.
- Gotchas: Initial Verify build failed because the new test called `.unwrap()` on `StepsReport`;
  removed that call and reran the full workspace successfully. Existing overview keys only gained
  `"harness": "claude"`; Phase 4 keys and existing home snapshots remain unchanged. Merged expansion
  remains keyed by repo, so Enter expands both harnesses together per U3. Quiet rows with handoffs
  remain mergeable when top-level; child placement still occurs before merging.

### 2026-10-30 — Step 4.2: Contract: `codex ×N`, collapsed repos, header — Claude
- Did: Wrote U1–U5 into `docs/design.md` ("Phase 4 decisions", plus the heuristic paragraph's `-C` note).
  `docs/output-schema.md`: Phase 4 changes block; Heuristic gains the U1 rule; Merge now covers
  `claude ×N` and `codex ×N` (quiet top-level Codex rows, decided per harness, threshold 4, one MergedRow
  per harness at its first member's place, MergedRow `harness`); new "Phase 4 fixture results". New
  isolated synthetic root `fixtures/sessions-phase4/` (repo-e formats: 13 Codex rollouts, 4 Claude
  transcripts, record `70401.json`, plan copy of repo-a for repo-j) and hand-written
  `fixtures/expected/phase4/overview.json`. `fixtures/sessions` and every earlier key untouched.
- Changed: `docs/design.md`, `docs/output-schema.md`, `fixtures/sessions-phase4/**` (20 files),
  `fixtures/expected/phase4/overview.json`, `PROGRESS.md`.
- Verify: `python3 -m json.tool fixtures/expected/phase4/overview.json` → valid JSON. Rows traced (key has
  18 session rows; current CLI `sessions --root fixtures/sessions-phase4 --now 2026-10-29T23:15:00Z --since all`
  → 18 sessions, `skipped: []`): repo-h (busy) `…40b4` busy/open_turn, `…40b3`, `…40b2`, `…40b1`
  ← `.codex/sessions/2026/10/29/rollout-*-…40b{1..4}.jsonl` (3 quiet, no merge); repo-j `…40d1` idle
  ← `.claude/sessions/70401.json`, plan ← `repos/repo-j/` (latest step 0.4); repo-k `…40e1` stale/open_turn
  ← `rollout-*-…40e1.jsonl`; repo-g merged codex ×4 `…40a4..a1` ← `rollout-*-…40a{1..4}.jsonl`; repo-i merged
  codex ×4 `…40c4..c1` ← `rollout-*-…40c{1..4}.jsonl`, then merged claude ×4 `…41c4..c1`
  ← `.claude/projects/-home-dev-repo-i/…41c{1..4}.jsonl`. Cross-check: current `overview` CLI on the root,
  merged rows flattened → equal to the flattened key (only the codex merge + `harness` differ, as 4.3 adds).
  Scrub `rg -l "/Users/|\bsk-|api_key" fixtures/sessions-phase4 fixtures/expected/phase4` → no output.
  `cargo test --workspace` (with 4.1's uncommitted claude changes in the tree) → 28 `test result: ok`, 0 failed.
- Review: adversarial Claude subagent (diff + Step 4.2 only) re-derived the key by hand and by CLI: correct
  and fully traceable; 0 P1, 2 P2, 5 P3, all accepted: U3 now defines `enter`/`esc` against today's
  merged-row expansion (two stages, separate per-repo set, `esc` clears both); collapse status of all 5
  groups stated, 4.4 golden must show repo-i expanded; "busy between quiet" reworded to "does not count
  towards the 4"; U3/U4 counts defined (all shown sessions; FR17 = Warning objects); U1 several `-C` → moves
  if any moves, bare `-C` moves (matches 4.1's code); repo-j plan headings renamed; selection vs expansion
  identity noted.
- Gotchas: repo-h's busy row only "sits between" quiet ones by `started_at`; in output it sorts first. A
  valid uuid needs 12 hex digits in the last group (first draft had 11; open refuses non-uuids).

### 2026-10-30 — Step 4.1: Linker current-directory options — Codex
- Did: Tokeniser preserves whether a word contains an expandable dollar sign. Directory-option
  recognition consumes separate or attached values for `-C` / `--cd`; `.` and expandable `$PWD` /
  `${PWD}` keep `dir_moved` false. Literal/escaped dollar signs, other values, and preceding `cd`
  still count as moves; `--` ends option scanning. Added exact table assertions for these cases,
  repeated options, missing/empty values, and option-shaped values.
- Changed: `crates/claude/src/invocation.rs`, `crates/claude/tests/invocation.rs`, `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md. `cargo test -p multi-link-claude` → exit 0; actual result lines
  (unit, fixtures, invocation, doc-tests respectively):
  ```text
  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
  test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  `cargo clippy -p multi-link-claude -- -D warnings` → exit 0; actual output:
  ```text
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.31s
  ```
- Review: Claude, 2026-10-30. Diff stays in the step's files; matches U1 (separate/attached `-C`,
  `--cd value`, `--cd=value`; `--` ends scanning; a preceding `cd` still moves). No fix round. Accepted
  limitation: `expandable_dollar` is one flag per word (as the plan specified), so contrived mixed-quote
  forms such as `-C "$P"WD` or `-C $'PWD'` read as "not moved"; nobody writes these. Claude re-ran
  `cargo test -p multi-link-claude` → same four `test result: ok.` lines (1 / 9 / 6 / 0 passed) and
  clippy `-D warnings` → clean. Workspace gate left to the coordinator (4.2 runs in parallel).
  Real-`~` re-measure (Claude, `cargo build --release -p multi-link`, then
  `target/release/multi-link links --since 24h --json`, run 2026-10-30 21:49 UTC). The 24h window has moved
  since the Phase 4 baseline, so the call totals differ. Checkout exactly `/Users/jarednarvaez/multi-link`:
  46 calls → 30 `heuristic`, 4 `ambiguous`, 12 `unlinked` (all 0 candidates). Counting
  `multi-link/fixtures/expected` too: 47 → 30 / 4 / 13. Baseline: 43 → 1 `heuristic`, 0 `ambiguous`,
  42 `unlinked`. The latest unlinked call (21:07 UTC) is probably this step's own still-running Codex
  run; not verified. Why the other 12 are unlinked was not investigated.
- Gotchas: Only crate-local verification ran while Step 4.2 runs in this checkout. Workspace gate
  and real-home link-count rerun remain with the coordinator/Claude. Existing fixture answer keys
  are untouched. Missing/empty directory-option values conservatively remain moved. PROGRESS.md
  updated under the user's explicit authorization for this parallel step.

### 2026-10-30 — Step 3.8: Real check, README, Release 1 close — Codex
- Did: Read AGENTS.md, PLAN Step 3.8 and Current state. Built release CLI; ran real-home timing,
  workspace tests, clippy and Expect PTYs. Recorded the original sandbox observations and folded
  in Claude's host evidence; updated README for Release 1 completion with documented limitations.
- Review round 1: Kept sandbox ps denial, 23/35 ms warm timings and no-resize control; added host
  timing (Run B satisfies k ≥ 1), crossterm resize root cause, draining resize, Ctrl-C, synthetic
  scratch-HOME Resume stub/missing-program checks and clean-exit record deletion. Replaced pending
  README claims with the two evidenced limitations and updated Current state. No code or PTY reruns.
- Changed: `README.md`, `docs/phase3-check.md`, `PROGRESS.md` only.
- Verify: `cargo run --release -q -p multi-link-core --example refresh_timing -- 3` → exit 0, actual output:
  ```text
  pass 1: 3794 ms, 25 sessions, 4 repos, 441 files re-parsed, 508526132 bytes re-parsed
  pass 2: 23 ms, 25 sessions, 4 repos, 1 files re-parsed, 1216599 bytes re-parsed
  pass 3: 35 ms, 25 sessions, 4 repos, 2 files re-parsed, 1654498 bytes re-parsed
  ```
- Verify: `cargo build --release -q -p multi-link` → no output, exit 0.
  `cargo test --workspace` → exit 0; all actual `test result:` lines are pasted in
  `docs/phase3-check.md`; the overview key comparison printed:
  ```text
  test overview_json_matches_answer_key_and_text_has_one_line_per_repo ... ok
  ```
  `cargo clippy --workspace -- -D warnings` → exit 0:
  ```text
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.31s
  ```
- Verify: `/usr/bin/expect <scratch>/simple.exp` → exit 0, actual output:
  ```text
  REAL_REPORT=seen
  TUI_EXIT=0
  PTY_WAIT=0
  ```
  `diff -u <scratch>/simple.before <scratch>/simple.after` → no output,
  exit 0 (`DIFF_EXIT=0`). Full identical `stty -a` snapshots and scrubbed repo-name screen
  evidence are in `docs/phase3-check.md`.
- Verify: `/usr/bin/expect <scratch>/check.exp normal` (real home, resize 40x160 →
  32x120, then `q`) → exit 1, actual decisive output:
  ```text
  RESIZE=40x160->32x120
  FAIL quit timeout
  ```
  After snapshot absent (`RESIZED_AFTER=missing (TUI did not return)`); no successful diff
  claimed for this sandbox resize run. Reproduced with and without the runner's outer PTY.
  A no-resize control waiting for the real report then exited cleanly. Checks stopped at that
  point in the original run; Claude's host investigation and passing runs follow.
- Verify (Claude, host): `/bin/ps -p $$ -o lstart=` → ok.
  `cargo run --release -q -p multi-link-core --example refresh_timing -- 3` → exit 0.
  Run A actual output:
  ```text
  pass 1: 3770 ms, 25 sessions, 4 repos, 441 files re-parsed, 516819435 bytes re-parsed
  pass 2: 34 ms, 25 sessions, 4 repos, 0 files re-parsed, 0 bytes re-parsed
  pass 3: 37 ms, 25 sessions, 4 repos, 0 files re-parsed, 0 bytes re-parsed
  ```
  Run B (used for the k>=1 requirement), actual output:
  ```text
  pass 1: 3950 ms, 27 sessions, 5 repos, 443 files re-parsed, 518399847 bytes re-parsed
  pass 2: 31 ms, 27 sessions, 5 repos, 1 files re-parsed, 183074 bytes re-parsed
  pass 3: 28 ms, 27 sessions, 5 repos, 1 files re-parsed, 185194 bytes re-parsed
  ```
- Verify (Claude, host): `check.exp` reproduced `FAIL quit timeout`; traced to crossterm 0.29
  dropping mio edge-triggered TTY readiness when SIGWINCH and a key share one wake, not a
  multi-link bug. Draining `host.exp` mode resize, actual output:
  ```text
  RESIZE=40x160->32x120 / RESIZE=32x120->40x160 / TUI_EXIT=0 / PTY_WAIT=0 / STTY_DIFF=0
  ```
  `stty -a` before/after diff → exit 0. `host.exp` mode ctrlc, real `~`, actual output:
  ```text
  TUI_EXIT=0 / PTY_WAIT=0 / STTY_DIFF=0
  ```
- Verify (Claude, host): Real `~` had 367 sessions: unknown 343, idle 22, waiting 1, stale 1;
  0 finished. Synthetic record in `HOME=<scratch>/fakehome`, copied sandbox transcript,
  real ps, no `--root`: `multi-link open --dry-run 80f7add3` → exit 0 (id scrubbed):
  ```json
  {"action":"resume","program":"claude","args":["--resume","80f7add3…"],"cwd":"~/multi-link-sandbox/repo-a"}
  ```
  Stub after `o`, actual output:
  ```text
  CHILD_OUTPUT=STUB_CLAUDE_RAN args=2 / FOOTER=returned to dashboard
  TUI_EXIT=0 / PTY_WAIT=0 / STTY_DIFF=0
  ```
  Missing program after `o`, captured footer:
  ```text
  cannot run "claude": No such file or directory (os error 2)
  ```
  Keys `t`, Esc and `q` worked in both checks; missing-program actual output:
  ```text
  TUI_EXIT=0 / PTY_WAIT=0 / STTY_DIFF=0
  ```
- Verify (Claude, host): Clean-environment interactive Claude in sandbox repo-a, one prompt,
  `/exit` → exit status 0. Record present while alive; 2 s after exit: record GONE, transcript kept.
  `multi-link open --dry-run 9c0a9a1d` → exit 3 (id scrubbed):
  ```json
  {"action":"unavailable","reason":"not_known_finished","command":"claude --resume 9c0a9a1d…","cwd":"~/multi-link-sandbox/repo-a"}
  ```
  `sessions --json` → state unknown, record null. Whole-phase Verify passes with the documented
  dependency and record-lifetime limitations; full host evidence is in `docs/phase3-check.md`.
- Verify (review round 1, Codex): `git diff --check` → no output, exit 0.
- Review: Claude (step session), 1 Codex fix round. Diff stays in `README.md`, `docs/phase3-check.md`, `PROGRESS.md`
  (plus Claude's PLAN.md backlog line). Codex's sandbox could not run `ps` or write `~/.claude`, so Claude ran the
  host checks and Codex folded them in (round 1). Resize "quit timeout": reproduced on host, root-caused to crossterm
  0.29 (edge-triggered tty readiness dropped when SIGWINCH and a key share one kevent wake; first `q` delayed, second
  quits); not a multi-link bug → limitation + backlog. Draining resize, Ctrl-C, stub and missing-program pty runs pass
  with `stty -a` unchanged. Child checks needed a synthetic record in a scratch HOME (real `~` has 0 finished);
  stated plainly in the doc. Sandbox exit: first runs invalid (inherited `CLAUDE_CODE_*` env); clean-env run shows
  the record is deleted on `/exit` → `unknown` → `Unavailable`. Claude re-ran `cargo test --workspace` (28 result
  lines, all ok, same counts as pasted, overview key ok) and clippy `-D warnings` (clean). Scrub check: no absolute
  home paths, ids ≤ 8 chars. Not covered by the pty: nonzero diagnostics in the footer (App tests cover rendering).
- Gotchas: Original sandbox `/bin/ps -p $$ -o lstart=` printed exactly
  `zsh:1: operation not permitted: /bin/ps`; host probing succeeded. First host interactive
  Claude attempts inherited `CLAUDECODE` / `CLAUDE_CODE_*` and wrote neither a record nor their
  own transcript, so were invalid; rerun with `env -i HOME USER LOGNAME TERM LANG PATH` supplied
  the valid record-on-exit evidence. Footer persistence across refreshes was not separately
  asserted in the PTY (Step 3.7 App tests cover it); nonzero diagnostics were not separately
  verified in these PTYs. Private captures/helpers remain temporary and uncommitted; evidence
  omits session text, titles, long ids and absolute home paths. Nothing staged, committed or pushed.

### 2026-10-30 — Step 3.7: `multi-link tui`: loop, refresh, keys, bell, open — Codex
- Did: Added `tui [--root] [--now] [--since]`. One background worker owns one Scanner, refreshes every 2 s
  (a slow pass completes before the next), injects plan lookup, and handles fresh full-id open/fork checks with
  the history window off. UI-thread input polling never reads disk and pauses naturally during child `status()`.
  Pure `App::update(Event) -> Vec<Effect>` manages selection, per-repo merged-row expansion, home/timeline/esc,
  quit, timeline lane scrolling, +/- zoom, left/right pan, 0 reset, and one bell per id newly entering waiting;
  first report and continued waiting are silent. Selection follows the full id when refresh sorting changes.
  Open effects contain only id/fork: executable actions come exclusively from core's fresh check. Unavailable,
  missing ids, lookup failures and child failures appear in the footer. Terminal guards restore raw mode and
  alternate screen after children, restore normal mode/cursor on exit, and cover setup failure and unwinding.
- Review round 1 fixes: Accepted all three findings. Open result messages now survive successful refreshes
  until the next key press; the in-flight checking message survives refreshes and keys until `Opened` arrives.
  Refresh errors have separate state, with display priority open/progress message > refresh error > key help
  plus diagnostics. Home and timeline help precedes diagnostics, truncated only at the terminal width.
  Runtime maps Ctrl-C to q's existing quit effect, preserving ordinary c and other key codes. Added exact
  App/render regressions for persistent results, checking lifetime, priority, combined help/diagnostics,
  width truncation, plus a runtime Ctrl-C mapping test and fixture PTY Ctrl-C restoration check.
- Decision (T9 open item): Keep the existing whole-pass error for a non-NotFound Claude transcript/subagent
  directory failure. Show the error and retain the last good report; disable `o`/`f` until a successful refresh.
  This meets T9's explicit last-good-report rule and avoids silently presenting an incomplete set as current.
  Converting to `unreadable` requires core/parser changes, which are outside the step's allowed files. The
  deterministic ENOTDIR regression makes one transcript's `<id>` a regular file, so its subagent discovery
  fails even under privileged tests. NotFound behavior and existing per-file diagnostics remain as before.
- Changed: `crates/cli/Cargo.toml`, `crates/cli/src/{main,tui}.rs`, `crates/cli/tests/cli.rs`,
  `crates/tui/src/{lib,app,home,timeline}.rs`, `Cargo.lock`, `PROGRESS.md`.
- Verify (fresh after Review round 1 fixes): Rust PATH set per AGENTS.md. `cargo test --workspace` → exit 0.
  Actual output excerpts: every target/result line plus App/runtime tests (other passing test-name lines omitted).
  ```text
     Running unittests src/main.rs (target/debug/deps/multi_link-f69ed8de1bb427a5)
test tui::tests::partial_setup_and_restore_failure_still_leave_normal_terminal ... ok
test tui::tests::control_c_uses_the_same_quit_effect_as_q ... ok
test tui::tests::terminal_restores_after_success_spawn_failure_and_unwind ... ok
test tui::tests::drawn_finished_but_fresh_running_never_resumes_and_errors_never_execute ... ok
test tui::tests::app_open_running_fixture_uses_fresh_pass_and_never_resumes ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
     Running tests/cli.rs (target/debug/deps/cli-89cbdb7a788bfb89)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.95s
     Running unittests src/lib.rs (target/debug/deps/multi_link_claude-002e7034494a8445)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/fixtures.rs (target/debug/deps/fixtures-aaba9febfe95def0)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
     Running tests/invocation.rs (target/debug/deps/invocation-0eaccf444a8ae875)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
     Running unittests src/lib.rs (target/debug/deps/multi_link_codex-204a903019c5c359)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/fixtures.rs (target/debug/deps/fixtures-b55b5b49100073b1)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running unittests src/lib.rs (target/debug/deps/multi_link_core-9171feebdd5225a7)
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
     Running tests/hook.rs (target/debug/deps/hook-0019750ae5ccafd6)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
     Running tests/links.rs (target/debug/deps/links-fc0002f57806efaa)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
     Running tests/repo_probe.rs (target/debug/deps/repo_probe-75c327fc37ed8c3d)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
     Running tests/sessions.rs (target/debug/deps/sessions-06ff65e7cbcbd48c)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
     Running unittests src/lib.rs (target/debug/deps/multi_link_hook-466f5073014ef46a)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/hook.rs (target/debug/deps/hook-e8fbeeb40f3e2efd)
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
     Running unittests src/lib.rs (target/debug/deps/multi_link_model-0c1ee82a039f47e5)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/model.rs (target/debug/deps/model-1d2bf893dec8dc40)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running unittests src/lib.rs (target/debug/deps/multi_link_plan-ef4f88aa29a57d10)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/fixtures.rs (target/debug/deps/fixtures-b74ce1dc050185a3)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/multi_link_tui-240b89d3261ac0d5)
test app::tests::each_new_waiting_id_gets_one_bell ... ok
test app::tests::refresh_preserves_selected_full_id_when_order_changes ... ok
test app::tests::checking_message_survives_refreshes_and_keys_until_opened ... ok
test app::tests::failed_refresh_keeps_report_and_blocks_open_until_success ... ok
test app::tests::footer_prioritizes_open_then_error_then_help_with_diagnostics ... ok
test app::tests::timeline_scroll_keeps_last_selected_lane_visible_even_in_one_body_row ... ok
test app::tests::merged_rows_expand_before_action_and_timeline_navigates ... ok
test app::tests::open_results_survive_successful_refresh_until_a_key_press ... ok
test app::tests::bell_only_on_entry_to_waiting_and_not_on_first_report ... ok
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/home.rs (target/debug/deps/home-7b5aca6f3e439c8d)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/timeline.rs (target/debug/deps/timeline-4b1fd284057fee64)
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
   Doc-tests multi_link_claude
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_codex
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_hook
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_model
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_plan
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests multi_link_tui
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
- Verify (fresh after Review round 1 fixes): `cargo clippy --workspace -- -D warnings` → exit 0,
  actual final output line:
  ```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.52s
  ```
- Verify: Additional fixture-only Python PTY harness launched `target/debug/multi-link tui --root fixtures/sessions
  --now 2026-10-29T23:15:00Z --since all`, navigated home/timeline, zoom/selection, resize, Esc and q;
  the controlling-terminal wrapper compared termios before/after the dashboard. Actual output:
  ```text
PASS: fixture PTY home, timeline, zoom, selection, resize, Esc, q; exit 0; termios unchanged
  ```
- Verify (Review round 1): Fixture-only Python PTY controlling-terminal wrapper launched the same `tui`
  fixture command above, sent byte 0x03 (Ctrl-C), and compared termios before/after. Actual output:
  ```text
PASS: fixture PTY Ctrl-C quits; exit 0; termios unchanged
  ```
- Review: Claude (step session), 1 round. Diff stays in the step's files. Round 1: (1) P2 a refresh overwrote the
  open result (Unavailable command text) within 2 s → kept until the next key; (2) P3 diagnostics hid the key help
  → shown together; (3) P3 Ctrl-C ignored in raw mode → quits like `q`. All fixed with tests. Claude re-ran
  `cargo test --workspace` (exit 0, all `ok`, tui 18 + cli unit 8 incl. the four required App/open cases) and
  clippy `-D warnings` (exit 0): matches Codex's output. T9 decision accepted: open's fresh pass would hit the
  same failing dir, so blocking `o`/`f` until a good refresh costs nothing. Done.
- Gotchas: Real-home timings, real harness-exit evidence and the real PTY stub-child checks remain Step 3.8.
  Child failure/panic restoration here is unit-tested with injected mode transitions; the fixture PTY smoke
  covers navigation/normal exit, not a real resumed harness. Initial local tests exposed incorrect test
  assumptions (a project entry that is a plain file is skipped, the busy fixture cwd is repo-c, and the first
  row has no children); corrected the tests against the fixtures and actual subagent-discovery path. Initial
  ad-hoc PTY checks also needed a controlling-terminal wrapper to inspect termios before the session leader
  exits; the final wrapper check above passes. A vanished timeline repo shows the existing missing-repo note;
  Esc returns home. The worker is not joined on exit, so a slow scan cannot delay terminal restoration.

### 2026-10-30 — Step 3.6: TUI repo timeline render — Claude (implementer)
- Did: `render_timeline(&OverviewReport, &LinksReport, repo, TimeWindow, area, buf)`, pure, no clock reads;
  `TimeWindow::{new, ending_at, default_for}` (default 2 h ending at the report's `now`). One row per session in
  overview order; merged `claude ×N` rows expanded into their members; Codex children under their parent as the
  overview nests them (T11), unplaced Codex rows top-level. Child branch shows placement: exact placing hand-off →
  ` └ `, otherwise ` └~`. Row labels name the state as a word (`wait busy stale idle unk done`), because `●` `○` `?`
  are markers here. Bar = session span `started_at`..`last_activity`; legend says "idle gaps included (not busy
  time)"; no busy swimlanes. Hand-off markers at `at` on the parent's row: exact `●`, heuristic `○`, ambiguous `?N`
  (`?` with no candidates), unlinked `×`. Collisions: least certain wins the cell; several in one cell → `+` next to
  it; `?N` anchored at its column, digits only if they fit. Spans clipped with `◂`/`▸`; links outside not drawn;
  zero-length span = 1 cell; empty/inverted window lists rows without lanes; unknown repo shows a note; cut rows
  counted ("↓ N more rows"). Ticks: smallest step leaving label + 3 cells. Goldens at 100×16: repo-a and repo-b
  ambiguous (links-amb-hidden.json), each at the default 2 h window and a 22:58–23:02 zoom. `home.rs`: only
  `short_id`, `basename` made `pub(crate)` (home goldens unchanged).
- Changed: crates/tui/src/timeline.rs (new), crates/tui/src/lib.rs, crates/tui/src/home.rs,
  crates/tui/tests/timeline.rs (new), fixtures/expected/tui/timeline-repo-a{,-zoom}.txt,
  timeline-repo-b-amb{,-zoom}.txt (new)
- Verify: `cargo test -p multi-link-tui` → unit `ok. 9 passed`; tests/home.rs `ok. 15 passed`; tests/timeline.rs
  `ok. 22 passed` (incl. the four snapshots, span_clipped_by_the_window, link_outside_the_window_is_not_drawn,
  zero_length_span_draws_one_cell, small_sizes_do_not_panic); 0 failed. `cargo clippy -p multi-link-tui --all-targets
  -- -D warnings` → clean.
- Review: Claude (step session) re-ran both commands (same results). Goldens checked column by column against
  overview.json / links.json / links-amb-hidden.json (zoom 240 s / 69 cells): repo-a bb792d9c ● 7, ef72378c ● 15 with
  child 01a189e2 under it (latest of exec + resume, T11), 80cce3e4 ● 38 with child 01a189e4…9fb5, 01a189e5 top-level;
  repo-b-amb 11bf1b58 `?2` at 45, f4d83303 ○ 45 with d953 as `└~`, d501 top-level. Adversarial Claude review round 1
  accepted: state glyphs collided with markers, markers hid each other, non-exact placement looked exact, one-row body
  dropped its note, `?N` shifted off its time, empty candidates drew `?0`. Rejected: both-edge clip arrow (cosmetic),
  East-Asian-ambiguous glyph width (same as home), RFC 3339 round-trip in epoch_ms (needs a model accessor; backlog).
  Coordinator workspace gate: `cargo test --workspace` 28 binaries ok, clippy `--workspace --all-targets -D warnings`
  clean, sessions / links / overview fixture diffs `same`.
- Gotchas: overview.json nests d501 under 11bf1b58 (computed from links.json), so the ambiguous golden uses a
  test-derived overview (`amb_overview` in tests/timeline.rs); if core's placement rule changes, that helper must
  follow. `●○◂▸━` are East-Asian-ambiguous width. `UPDATE_SNAPSHOTS=1` rewrites the home goldens too. No
  selection/scroll state in the timeline yet; 3.7 adds keys.

### 2026-10-30 — Step 3.6a: `ps` runs in the C locale — Claude (implementer)
- Did: Both `ps` calls (batch and single) now go through one helper, `RealProcessProbe::ps_command`, which removes
  `LANG`, the standard `LC_*` names and every inherited `LC_*` key, then sets `LC_ALL=C` and `TZ=UTC`, so `lstart`
  always prints English names in UTC like `procStart`. Unit test checks the built Command's envs; the batch and
  single stub `ps` scripts also require `LC_ALL=C` and unset `LANG`/`LC_TIME`. New integration test
  `real_probe_ignores_non_english_locale` re-runs the test binary as a child with `LC_ALL`/`LANG`/`LC_TIME=de_DE.UTF-8`;
  the child checks German really changes `ps` output, then requires `Alive` from `probe` and `batch`; the parent
  checks the child printed `LOCALE-CHILD-OK` and `1 passed`, and on macOS `LOCALE-CHANGES-PS`. Review round 1:
  `own_pid_matches_start_time_and_rejects_reused_identity` now takes its reference start from `lstart(pid, "C")`.
  Mutation check: with the old TZ-only command the locale test, the env unit test and both stub tests fail.
- Changed: crates/core/src/probe.rs, crates/core/tests/repo_probe.rs
- Verify: `cargo test -p multi-link-core` → lib `ok. 32 passed`; hook `ok. 6 passed`; links `ok. 15 passed`;
  repo_probe `ok. 11 passed`; sessions `ok. 12 passed`; doc-tests `ok. 0 passed`; 0 failed. Same counts with
  `LC_ALL=fr_FR.UTF-8 LANG=de_DE.UTF-8 LC_TIME=ja_JP.UTF-8` in the outer env. Locale test ran for real (no SKIP):
  de_DE.UTF-8 gives `Fr 30 Okt 08:44:59 2026` vs C `Fri Oct 30 08:44:59 2026`. `cargo clippy -p multi-link-core
  --all-targets -- -D warnings` → clean.
- Review: Claude (step session) read the diff: in scope, both paths share one helper. Adversarial Claude review:
  nothing blocking; on this Mac `LC_ALL=C` beats LC_TIME/LANG/LANGUAGE/__CF_USER_TEXT_ENCODING, ps ignores
  AppleLocale, padding handled by the whitespace-split compare. Round 1 accepted: un-pinned reference start in the
  own_pid test; macOS must show `LOCALE-CHANGES-PS`. Rejected: dropping the fixed env key list (keeps LANG/LC_TIME
  removal explicit). Step session re-ran Verify outside the sandbox (German test ran, no SKIP). Coordinator re-ran
  `cargo test -p multi-link-core` (same counts, 0 failed) and clippy (clean); workspace gate runs after 3.6.
- Gotchas: Never `set_var` the locale in tests (edition 2024 makes it unsafe and it races threads); use the child
  re-exec pattern. Any test's reference `ps` call must pin `LC_ALL=C` too. In `format!` stub scripts, shell
  `${VAR+x}` is written `${{VAR+x}}`. Off macOS, a missing de_DE.UTF-8 prints a SKIP note but still requires `Alive`.

### 2026-10-30 — Step 3.5: TUI crate: home screen render — Claude (implementer)
- Did: New lib crate `multi-link-tui` (deps: model, ratatui 0.29.0, crossterm 0.28.1, time). `render_home(&OverviewReport, Option<usize>, Rect, &mut Buffer)` is pure (ages from `report.now`). Layout: a title line (`N repos`, now), then one block per repo in report order: header (`◆` waiting / `●` busy, basename, session counts by state in T5 order, full path); steps line (no PLAN.md / unsupported / none logged / `latest <id> <title> · Verify ✓|✗|? (<date>)`, plus `active …`); one lane per session (state glyph, harness, short id, state, age, `@<checkout basename>` when checkout ≠ repo, then step tag and match from the lane's link: for a child its placing hand-off (latest link from that parent, T11), for a top-level row its latest link, `N hand-offs` when more than one, `no linked hand-off` when none); Codex children indented with `├`/`└`; a MergedRow as one `▸ claude ×N` line; FR17 `!` warning lines; the `↳ hand-off parent → child` line (`? (N candidates)` when ambiguous). Footer keys: ↑↓ select · enter expand/timeline · t timeline · o open · f fork · q quit. `selectable(report)` lists the selectable lines (top rows, children, one per merged row) in screen order. `selection` indexes it and the body scrolls just enough to keep it visible. When the body is clipped, the last body row reads `↓ N more lines`. Order, nesting and merging are taken from the report as given. Review round 1: u16-safe row math, placing hand-off for child lanes, `no linked hand-off`, clip marker, plurals, test guard.
- Changed: Cargo.toml, Cargo.lock, crates/tui/{Cargo.toml,src/lib.rs,src/home.rs,tests/home.rs}, fixtures/expected/tui/{home,home-hook,home-full,home-hook-full}.txt
- Verify: `cargo test -p multi-link-tui` → unit `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`;
  tests/home.rs `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` (incl. `home_snapshot_100x30`,
  `home_hook_snapshot_100x30`, `home_full_snapshot_100x80`, `home_hook_full_snapshot_100x80`, `small_sizes_do_not_panic`
  at 0×0, 1×1, 0×5, 5×0, 20×5, 40×10, 2×2, 3×3 with selection None / first / last / past the end).
  `cargo clippy -p multi-link-tui -- -D warnings` → clean (also with `--all-targets`).
- Review: Claude (step session) re-ran both Verify commands: 3 + 15 tests ok, clippy clean. Goldens checked line by line
  against the overview keys (T4). `overview.json` → home-full.txt: repo-c ● busy, 2 busy · 2 unknown, rows 00000000…0010
  busy, 6358ec7e busy, 11c1f13c / 85a715ba unknown, FR17 warning with both busy ids, latest hand-off unlinked exec bg;
  project bed52539 idle; repo-a plan latest 0.4 Verify ?, manual Codex 01a189e5 top-level first, 80cce3e4 → 01a189e4…9fb5
  exact, ef72378c → 01a189e2 exact · 2 hand-offs (placed under the later resume parent), bb792d9c no child; repo-b plan
  latest 3, `claude ×5` with 4 Codex children; repo-a-wt c86d374f → 01a189e4…b109; scratch 9d02a552 idle; repo-d three
  rows; repo-e Claude …000e and top-level Codex …000f with no linked hand-off. `hook/overview.json` → home-hook-full.txt:
  waiting tier repo-a then repo-c, both ◆; d1f3b188 and 11c1f13c waiting first in their repos; FR17 warning kept; other
  tiers unchanged. The 100×30 goldens equal rows 2–28 of the full goldens plus `↓ 21/22 more lines`.
  Adversarial Claude review: goldens clean; 9 findings elsewhere. Fixed in round 1: u16 overflow near y≈65535, "no
  hand-off" overclaimed for ambiguous candidates, child lane uses its placing hand-off (T11), clip marker, plurals, test
  underflow; `time` dep noted in Gotchas. Rejected: scroll anchoring (3.7 owns scroll state); `@basename` for a worktree
  with the repo's folder name (backlog). Coordinator workspace gate (with 3.4 in the tree): `cargo test --workspace`
  all ok, clippy `--workspace --all-targets -D warnings` clean.
- Gotchas: Codex ids are UUIDv7, so the first 8 hex digits are a shared timestamp; lanes show the first 8 and last 4
  characters (`01a189e4…2eb3`). `time` is an extra dependency, only to turn `Timestamp` into epoch seconds (model has no
  accessor). crossterm is declared but unused until 3.7. A last activity after the fixed `now` reads `6h ahead`. No
  fixture has a checkout ≠ repo, a step tag, active steps, an ambiguous latest hand-off or a second parent with a later
  link, so tests pin those on modified reports.

### 2026-10-30 — Step 3.4: Core open actions + CLI `open --dry-run` — Claude (implementer)
- Did: New `core::open`: `OpenAction` (`Resume{program,args,cwd}` | `AppLink{url}` |
  `Unavailable{reason,command,cwd}`, serde tag `action`, snake_case, the schema's JSON shape) and
  `Reason`; `action(&Session, fork)` is a pure function walking output-schema "Open" top to bottom
  (unsafe id → Codex → finished+transcript → finished record-only → running with `^local_[A-Za-z0-9-]+$`
  host (app link / fork refused) → running without → unknown). `is_safe_id` (lowercase 8-4-4-4-12 hex,
  hand-written, no new dep) and `is_app_host_id`. Lookup: `open::find(scanner, root, options, prefix)`
  does a fresh `Scanner::sessions` pass with `since: None` and returns `Lookup::{Found, NoMatch, Ambiguous}`
  (prefix match on full ids; empty prefix = no match); `open::fresh_action(…, full_id, fork)` is the
  helper 3.7's TUI calls right before executing (exact id only, `None` when gone).
  CLI `multi-link open <id-prefix> [--fork] [--dry-run] [--root] [--now]` (no `--since`: the window is
  always off for open). `--dry-run` prints the action JSON on stdout. Exit codes are the same in both
  modes: 0 runnable (dry-run) or child exited 0 (executed); child non-zero or killed by a signal →
  `"<program>" exited with <status>` on stderr, exit 1 (the child's own code is never passed through, so it
  cannot collide with 2/3/4); `Unavailable` → `cannot open "<id>": <reason>; run it yourself in "<cwd>":
  <command>` on stderr, exit 3 (dry-run prints the JSON first, then exits 3, so
  the exit code always answers "can this be opened"); several matches → `N sessions match <p>:` plus one
  `"<id>"  harness  state` line each on stderr, exit 2; none → `no session matches <p>`, exit 4. Executing:
  `open_command` builds a `std::process::Command` (program + argv, `current_dir(cwd)`, no shell; app link
  = absolute `/usr/bin/open <url>`, no PATH lookup; `claude` stays a PATH lookup on purpose), run with `.status()` (inherited stdio, waits). Spawn failure → exit 1 with the error.
  Untrusted text on stderr (session id, `cwd`) is printed with `{:?}` (control characters escaped, paths
  with spaces quoted); `command` holds only a validated uuid and stays plain.
  Review round 1 (adversarial Claude review) applied: child exit code mapping, `{:?}` on stderr,
  `/usr/bin/open`, fixture table also asserts no fixture session is `finished`. Rejected there: an exact
  id that is a prefix of another → exit 2 (matches the spec; uuids cannot collide that way).
- Changed: `crates/core/src/open.rs` (new), `crates/core/src/lib.rs` (`pub mod open`),
  `crates/cli/src/main.rs` (`Open` subcommand, `open`, `open_command`, `child_exit`, `home()` / `state_dir()` helpers
  shared with the other commands, 2 unit tests), `crates/cli/tests/cli.rs` (4 open tests)
- Verify: `cargo test -p multi-link-core -p multi-link` → all `ok`: core lib 31 passed (10 new `open::tests`),
  core hook 6, links 15, repo_probe 10, sessions 12; cli unit 3 (2 new), cli integration 20 (4 new); 0 failed (after review round 1).
  `cargo clippy -p multi-link-core -p multi-link -- -D warnings` (also with `--all-targets`) → clean.
  Verify bullets → tests:
  - fixture table, every session × {open, fork}, plain (28) and hook variant (29), no session `finished` and no `Resume`, reasons per
    schema "Fixture coverage" with exact counts (5 desktop app_link / fork_needs_finished, 70001 + hook
    d1f3b188 / 11c1f13c running_without_app_link, 9 Codex codex_not_supported, 13 / 12 transcript-only
    not_known_finished): `open::tests::fixture_table_never_resumes_and_matches_schema_coverage`
  - FakeProcessProbe: record + transcript + Gone → Resume (args, `--fork-session`, transcript cwd):
    `record_transcript_and_gone_process_resume_in_cwd`; record-only + Gone → `no_transcript`:
    `record_only_gone_is_no_transcript`; two records, newer Gone + older Alive → app_link / fork refused,
    never Resume: `one_gone_and_one_alive_record_never_resumes`; Alive with/without host:
    `alive_record_with_and_without_host_id`; probe Unavailable (busy/idle/waiting/other status) never
    Resume: `unavailable_probe_follows_record_status_and_never_resumes`; transcript-only Claude and
    completed-turn Codex with command text: `transcript_only_claude_and_completed_codex_offer_commands`;
    unsafe ids (`-rf`, uppercase, wrong length, bad separator), Codex in every state, bad host ids:
    `pure_rules_cover_unsafe_ids_codex_states_and_bad_host_ids`; JSON shape: `json_shape_matches_the_schema`
  - id older than `--since`: core `lookup_finds_old_ids_and_reports_none_or_ambiguous` (0d7faed4 absent
    from a 2h `sessions` pass, found by `find` with the same 2h options) and CLI
    `open_lookup_no_match_ambiguous_and_old_ids` (absent from `sessions` default window, found by `open`)
  - no match → exit 4, ambiguous (`01a189e4`, 4 Codex) → exit 2: `open_lookup_no_match_ambiguous_and_old_ids`
  - `--dry-run` CLI: app-linked running Claude 6358ec7e (+ fork → fork_needs_finished, exit 3):
    `open_dry_run_prints_app_link_for_a_running_desktop_session`; transcript-only Claude 0d7faed4 and
    Codex 01a189e2 × {open, fork}: `open_dry_run_transcript_only_claude_and_codex_are_unavailable_exit_three`;
    without `--dry-run` an Unavailable prints nothing on stdout and runs nothing:
    `open_without_dry_run_runs_nothing_for_unavailable`
  - execution argv/cwd without spawning: cli unit `open_commands_run_without_a_shell_in_cwd`
    (inspects `Command::get_program/get_args/get_current_dir`; app link argv is `/usr/bin/open <url>`);
    child exit mapping: cli unit `child_exit_never_passes_the_child_code_through` (0 → 0; exit 2/3/4 and
    SIGKILL → 1 with message, via `ExitStatusExt::from_raw`). Escaped stderr: exact-text assertions in
    `open_without_dry_run_runs_nothing_for_unavailable` and the ambiguous listing. No test executes a harness or opens a URL.
- Review: Claude (step session) read the full diff: rule order, JSON shape, lookup and exit codes match
  output-schema "Open"; only the 4 named files changed. Adversarial Claude review (fresh subagent, diff + step +
  schema only): no Resume-safety hole in 3.4's code. Round 1, accepted and fixed: child exit code passed
  through (collided with 2/3/4) → mapped to 1; untrusted id/cwd on stderr → `{:?}`; app link via
  `/usr/bin/open`; fixture table also asserts no fixture is `finished`. Rejected: an exact id that is a prefix
  of another gives exit 2 (spec says so; uuids cannot collide); PATH lookup for `claude` (the user's PATH).
  **Open P1, upstream, outside 3.4's files:** `core/src/probe.rs` runs `ps` with only `TZ=UTC`; `lstart` is
  locale-dependent (confirmed: `LC_ALL=de_DE.UTF-8` → `Fr 30 Okt …`), so under a non-English locale a live
  record's start time mismatches → `Gone` → `finished` → 3.4 would offer `Resume` on a live session. Needs a
  probe.rs fix (`LC_ALL=C` on both `ps` calls + test) before 3.7 at the latest. Verify re-run by Claude:
  same counts (core lib 31, hook 6, links 15, repo_probe 10, sessions 12, cli unit 3, cli integration 20, 0
  failed); clippy `--all-targets -D warnings` clean. Coordinator workspace gate (with 3.5 in the tree):
  `cargo test --workspace` all ok, clippy `--workspace --all-targets -D warnings` clean, sessions / links /
  overview fixture diffs `same`.
- Gotchas: Fixture mode can never produce `finished`, so `Resume` and the real spawn are only unit-tested
  (core via FakeProcessProbe, CLI via the built `Command`); the actual `claude --resume` / `open <url>`
  run is untested until 3.7/3.8. Exit 2 is shared with clap usage errors. `open` without `--dry-run` in
  fixture mode on a desktop record would really run `open claude://…` (fixture data, but a real call),
  so tests only use `--dry-run` or Unavailable sessions. Cargo printed "Adding windows…" lock lines
  during the run: that is 3.5's parallel `Cargo.toml` change, not this step (3.4 touched no Cargo file).

### 2026-10-30 — Step 3.3: Core: new Session fields, overview report, CLI `overview` — Claude (implementer)
- Did: Model: `Session.host_session_id` + `checkout`; `OverviewReport`, `RepoGroup`, `Tier`, `Plan`,
  `LatestStep`, `ConcurrencyWarning`, `Row` (untagged; `SessionRow` / `MergedRow` each write their own
  `"kind"`, since the keys put `"kind": "session"` on children and merged members too), `SessionRow`,
  `MergedRow`. Claude: `SessionRecord.host_session_id` kept verbatim (non-string → `None`). Core: several
  records per id per the schema (`finished` only when every record is `Gone`; else the not-gone record with
  the greatest `updatedAt`; all gone → greatest `updatedAt`; ties by file name); `checkout` from the memoised
  git mapping (fixture = `cwd`); new `core::overview` (`build`, `plan_folder`, merge at `MERGE_AT = 4`) and
  `Scanner::overview(root, options, plans)` = one `refresh` + build, returning `OverviewOutput {report,
  sessions, links}`. CLI `overview [--root] [--now] [--since] [--json]` uses one Scanner pass; text mode is
  one line per repo (tier, session count, last activity, latest step or convention, repo, FR17 marker).
  `refresh_timing` now runs `Scanner::overview` each pass (with the CLI's plan lookup) and prints the repo
  count.
  Review round 1 fixes (P3, planner-decided): `plan_folder` returns `None` → plan `missing` without calling
  the lookup when the fixture basename would escape `<root>/repos` (`/`, `""`, `.`, `..`); a placed Codex
  child attaches once, to the first matching parent in member order (two listed sessions sharing an id);
  comment on the stricter Claude-parent check; `multi-link-plan` is a dev-dependency of core so the example
  times the real plan parse (planner widened 3.3's files to `crates/core/Cargo.toml` + `Cargo.lock` for this).
  Tests: overview unit tests for every "Required 3.3 unit test" (cross-repo link, merge at 3 / 4, busy and
  stale Codex child under an `unknown` parent, stale/finished order, equal-activity repos by name, one plan
  basename for two repos, several records for one id in `core/tests/sessions.rs`) plus T11 placement,
  FR17 / active steps and logged-step-not-in-plan; claude `hostSessionId` absent / different from the
  transcript id / non-string; real-mode `checkout` on a temp repo with a worktree; model round-trip of
  `overview.json` and the hook keys; CLI overview JSON + text tests; hook-variant CLI test covers `overview`.
- Changed: `crates/model/src/lib.rs`, `crates/model/tests/model.rs`, `crates/claude/src/lib.rs`,
  `crates/claude/tests/fixtures.rs`, `crates/core/src/{lib.rs,scanner.rs,sessions.rs,overview.rs (new)}`,
  `crates/core/tests/{sessions.rs,repo_probe.rs}`, `crates/core/examples/refresh_timing.rs`,
  `crates/cli/src/main.rs`, `crates/cli/tests/cli.rs`, `crates/core/Cargo.toml` + `Cargo.lock` (dev-dep only,
  round 1), `PROGRESS.md`. Answer keys untouched.
- Verify (re-run after round 1): `cargo test --workspace` → 24 `test result: ok` lines, 137 tests, 0 failed.
  JSON diffs (parsed compare) →
  ```text
sessions: same
links: same
overview: same
steps: same
hook/links: same
hook/sessions: same
hook/overview: same
hook/sessions-2h: same
  ```
  (`overview --root fixtures/sessions --now 2026-10-29T23:15:00Z --since all --json` vs `overview.json`; hook
  variant = fixtures/sessions + fixtures/hook overlay, `task-output/be01y5dv6.output` removed, `overview
  --since all --json` vs `hook/overview.json`.) `cargo clippy --workspace --all-targets -- -D warnings` →
  clean. `cargo build --release -p multi-link-core --example refresh_timing` → `Finished`. Not run on real
  `~` (left to the reviewer).
- Review: Claude (planner) read the diff against Step 3.3 and the schema: matches the contract. An adversarial
  Claude review found 0 P1 / 0 P2 and 6 P3. Accepted: plan folder escape → `missing`; child attached once;
  placement comment; plan dev-dependency so the timing is real (planner widened the step's files). Rejected:
  non-UTF-8 plan folder (backlog nit). Test gaps were covered by the two new tests. One round. Planner re-ran
  Verify: 24 binaries ok (137 tests); all 8 parsed diffs `same`; clippy clean. `refresh_timing -- 3` on real
  `~` → `pass 1: 3460 ms, 44 sessions, 5 repos, 374 files re-parsed, 468392182 bytes re-parsed` / `pass 2: 25
  ms, 44 sessions, 5 repos, 0 files re-parsed, 0 bytes re-parsed` / `pass 3: 45 ms, 44 sessions, 5 repos, 2
  files re-parsed, 2147839 bytes re-parsed`. Approved.
- Gotchas: core does not depend on `multi-link-plan` at runtime (only as a dev-dependency for the example), so
  `Scanner::overview` takes a `plans: &mut dyn FnMut(&Path) -> RepoSteps` lookup; the CLI and `refresh_timing`
  pass the plan crate's `steps_report`. Core decides the folder (`plan_folder`: real `<repo>`, fixture
  `<root>/repos/<basename>`, `None` if it would escape). The record choice now probes every record of an id (the batch probe already covered all of them, so still one `ps` per pass).

### 2026-10-30 — Step 3.2: Output contract: checkout, host id, overview, open rules — Claude
- Did: Wrote the Phase 3 contract. `docs/design.md`: "Phase 3 decisions" T1–T11 plus 3.2 clarifications, NFR
  "Incremental reads" revised per T1 and "Speed" with the 3.1 numbers, FR18 table rewritten to T7.
  `docs/output-schema.md`: Session `checkout` / `host_session_id`, several records per id, new "`overview`"
  section (groups, tiers, T5 order, T11 placement, `claude ×N` for 4+ quiet rows, FR17, latest hand-off,
  plan lookup incl. fixture `<root>/repos/<basename>`) and "Open" (rules table, reasons, lookup, known limits).
  New fixture: synthetic busy record `70001.json` (second busy session in repo-c for FR17; no `hostSessionId`),
  documented in `fixtures/README.md`. No worktree fixture: fixture mode has no git, so `checkout` ≠ `repo`
  can only be a temp-repo unit test. Answer keys hand-labelled: the session keys got the two fields by line
  edits (`checkout` = `cwd` in fixture mode, host ids read from each record) plus the new row; the overview
  layout (order, nesting, merge, tiers, warnings, latest hand-off, plan) was typed by hand in a small
  assembler that only copies Session fields / Link objects by id from the existing keys.
- Changed: `docs/design.md`, `docs/output-schema.md`, `fixtures/README.md`, `fixtures/expected/sessions.json`,
  `fixtures/expected/hook/sessions.json`, `fixtures/expected/hook/sessions-2h.json`,
  `fixtures/expected/overview.json` (new), `fixtures/expected/hook/overview.json` (new),
  `fixtures/sessions/.claude/sessions/70001.json` (new), `PLAN.md` (Review log; T7, 3.2/3.4/3.5 wording per
  review), `PROGRESS.md`.
- Verify: `python3 -m json.tool` on the 5 changed/new keys + `70001.json` → `parse ok` ×6; key check →
  ```text
sessions.json: 28 sessions, all have checkout + host_session_id
hook/sessions.json: 29 sessions, all have checkout + host_session_id
hook/sessions-2h.json: 25 sessions, all have checkout + host_session_id
overview.json: 8 repos, 28 sessions = every id in sessions.json once; tiers, order, merge, nesting, FR17, latest hand-off consistent
hook/overview.json: 8 repos, 29 sessions = every id in hook/sessions.json once; tiers, order, merge, nesting, FR17, latest hand-off consistent
  ```
  Scrub `rg` → nothing (exit 1); local `check-scrub.py` → exit 0. `cargo test --workspace` → red as expected
  until 3.3 (cli tests: `phase_one_json_matches_answer_keys` and 3 others FAILED on the new fields/row).
- Review: adversarial Claude review (fresh subagent, diff + steps + fixtures only), 2 rounds: keys correct by
  hand-check; 12 findings, 11 accepted, 1 partly (see PLAN.md Review log, Step 3.2).
- Gotchas: the git-ignored `fixtures/raw/check-scrub.py` needed `70001.json` added to its expected file set
  (local only, not committed). Overview keys are pretty-printed with multi-line Link objects (~700 lines);
  tests compare parsed JSON. `open` resume is crash-only in practice if Claude deletes records on clean exit
  (3.8 checks).

### 2026-10-30 — Step 3.1: Refresh speed: scanner cache, memoised git, one ps — Codex
- Did: Added Scanner caches keyed by path, length, mtime and inode for main/subagent Claude
  transcripts, session records, Codex rollouts, hook files and task output. Cached results retain
  invocation evidence and unknown-record timestamps for waiting. Recompute state/window/link
  decisions each pass; map only listed cwds, memoising git by cwd. Report wrappers use fresh Scanners;
  `refresh` returns sessions + links from one scan and one batch probe. Real batch ps deduplicates
  nonzero pids, uses UTC and fails soft without per-pid process fallbacks. Hook read failures become
  Scanner diagnostics and existing `unreadable` diagnostics instead of aborting the report.
  Added cache invalidation, warm answer-key parity, batch-call count, ps argument/UTC/failure,
  task-output change/removal, hook-directory recovery and repo-window/memoisation tests.
  Round 1: Routed `scan()` main transcripts and subagents through `parse_file()`, removing
  the duplicate transcript fold and reader; existing Claude fixtures now exercise the production
  parser. Updated the missing/empty-file unit test to call that parser. Confirmed Codex `discover()`
  and `parse_file()` still share `parse()` without changes. Replaced production `FakeProcessProbe`
  with private `BatchProcessProbe` in scanner.rs, retaining pid/start lookup and unavailable fallback.
  Claude fixes (after adversarial review): `probe.rs` `MAX_PID` (macOS 99 999, else `i32::MAX`) keeps
  out-of-range pids out of the batch `ps` (macOS `ps -p 1,100000` → "process id too large", rc 1, which
  made every pid Unavailable) and answers them Unavailable alone; `scanner.rs` `rollout_paths` skips a
  rollout that vanishes between `read_dir` and stat instead of failing the pass; hook files are read only
  if regular (symlinks skipped, as the old `DirEntry::file_type` reader did); `Stamp` doc notes the
  fine-grained-mtime assumption. Tests: `MAX_PID+1` cases in both batch tests, new
  `symlinked_hook_files_are_skipped`. The rollout NotFound skip is a race and is untested.
- Changed: `crates/claude/src/lib.rs`, `crates/codex/src/lib.rs`,
  `crates/core/src/{lib.rs,scanner.rs,sessions.rs,links.rs,link.rs,hook.rs,probe.rs}`,
  `crates/core/examples/refresh_timing.rs`, `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md. `cargo test --workspace` → exit 0; actual result lines
  (workspace runner order, including doc tests):
  ```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
- Verify: Phase 1 Step 1.9's three literal `diff <(cargo run … | jq -S .) <(jq -S …)`
  commands (sessions, links, steps) → exit 0, actual stdout:
  ```text
same
same
same
  ```
  Python subprocess comparison of the pre-change CLI binary against the rebuilt binary, using a
  temporary copy of plain fixtures and the hook overlay (background output removed, as in hook tests).
  Compared stdout/stderr bytes and exit codes; parsed JSON compared with untouched answer keys.
  `git diff --exit-code HEAD -- fixtures/expected` → exit 0, no output. Comparison stdout:
  ```text
plain sessions JSON/text stdout/stderr/exit bytes: same
plain links JSON/text stdout/stderr/exit bytes: same
plain steps JSON/text stdout/stderr/exit bytes: same
hook sessions JSON/text stdout/stderr/exit bytes: same
hook sessions CLI diff: same
hook links JSON/text stdout/stderr/exit bytes: same
hook links CLI diff: same
hook sessions 2h bytes and CLI diff: same
fixtures/expected vs HEAD bytes: same
  ```
- Verify: `cargo run --release -p multi-link-core --example refresh_timing -- 3` on real home,
  read-only, one Scanner, full sessions + links refresh per pass → exit 0. First run, all three lines:
  ```text
pass 1: 3332 ms, 64 sessions, 360 files re-parsed, 452929165 bytes re-parsed
pass 2: 10 ms, 64 sessions, 1 files re-parsed, 721216 bytes re-parsed
pass 3: 11 ms, 64 sessions, 1 files re-parsed, 722820 bytes re-parsed
  ```
  Repeated after the hook-diagnostic plumbing and per-transcript waiting-key adjustment, exit 0:
  ```text
pass 1: 3552 ms, 63 sessions, 360 files re-parsed, 453447435 bytes re-parsed
pass 2: 11 ms, 63 sessions, 0 files re-parsed, 0 bytes re-parsed
pass 3: 19 ms, 63 sessions, 1 files re-parsed, 1297835 bytes re-parsed
  ```
  Both runs beat the 2000 ms warm target here. First run measured changing files on both warm passes;
  repeat pass 2 observed no changes and therefore does not satisfy that pass's k ≥ 1 condition.
- Verify: `cargo clippy --workspace --all-targets -- -D warnings` → exit 0:
  ```text
    Checking multi-link-core v0.1.0
    Checking multi-link v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.90s
  ```
  (Checkout paths omitted.) `cargo fmt --all -- --check` and `git diff --check` → no output, exit 0.
- Verify: Round 1 (Rust PATH set per AGENTS.md): `cargo test --workspace` → exit 0;
  actual result lines (runner order, including doc tests):
  ```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.93s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0 (checkout paths omitted):
  ```text
    Checking multi-link-claude v0.1.0
    Checking multi-link-core v0.1.0
    Checking multi-link-hook v0.1.0
    Checking multi-link v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.94s
  ```
  `cargo fmt --all -- --check` → no output, exit 0. The three literal Step 1.9 diffs
  (sessions, links, steps) → each exit 0; actual combined stdout:
  ```text
same
same
same
  ```
  `git diff --exit-code HEAD -- fixtures/expected`, `git diff --check`, and
  `git diff --cached --stat` → no output, exit 0. Answer keys unchanged; nothing staged.
  `refresh_timing` not rerun, per request (Claude ran it with real ps).
- Verify (Claude, final tree, outside the sandbox): `cargo test --workspace` → all `test result: ok`
  (core scanner suite 10 passed), clippy `-D warnings` and `cargo fmt --check` clean, `git diff --check`
  clean; the three Step 1.9 diffs → `same` `same` `same`; `git diff --exit-code HEAD -- fixtures/expected`
  → exit 0. `/bin/ps -o pid=,lstart= -p $$` works here. `cargo run --release -p multi-link-core --example
  refresh_timing -- 3` on real `~` (second run on the final build; the first had k=0 on pass 3):
  ```text
pass 1: 3058 ms, 61 sessions, 365 files re-parsed, 458819690 bytes re-parsed
pass 2: 29 ms, 61 sessions, 1 files re-parsed, 659860 bytes re-parsed
pass 3: 31 ms, 61 sessions, 2 files re-parsed, 1177474 bytes re-parsed
  ```
  Codex's sandbox timings above (10–19 ms) ran without `ps`; these replace them. Warm CLI
  `multi-link sessions --json`: 5.91 s real (was 16.4–16.9 s).
- Review: Claude, 2 rounds. Round 1 (Claude → Codex): duplicate Claude transcript fold (crate tests covered
  `scan`, production ran `parse_file`) and `FakeProcessProbe` in the live path — both fixed. Then (owner:
  no more Codex) an adversarial Claude subagent review, 10 findings: accepted #1 batch `ps` sunk by one pid
  ≥ 100000 (confirmed on macOS), #2 vanishing rollout aborts the pass, #6 hook symlinks now followed, #9
  doc the mtime assumption — fixed by Claude, and that edit reviewed by a second Claude subagent (clean;
  MAX_PID cutoff confirmed with `ps -p 1,99999` rc 0 vs `1,100000` rc 1; tests fail without each fix).
  Rejected: #3 one bad Claude file fails the pass (pre-existing; T9 scopes hook errors; carried to 3.7 in
  Current state); #4 repo memo never expires / memoises the git fallback (`RepoMode::map` gives no failure
  signal, and memoising only non-fallbacks would re-run git for every repo-root cwd; noted for 3.2+);
  #5 changed Claude files read twice, bytes counter ≈ half the real reads (wall-clock target still met);
  #7 hook errors listed in both diagnostics and Codex `unreadable` (acceptable under T9); #8 prune keeps
  existing-but-undiscovered paths (memory only); #10 rebuilt `io::Error` drops `raw_os_error` (output same).
  Files changed match the step's list (+ PROGRESS.md).
- Gotchas: Sandbox blocks real ps: `/bin/ps -o pid=,lstart= -p $$` produced
  `zsh:1: operation not permitted: /bin/ps`. No workaround; RealProcessProbe returns Unavailable.
  A read-only git rev-parse in the real home checkout succeeded (exit 0). Timings consequently
  do not establish unrestricted ps performance or real liveness; batch semantics are tested with a
  deterministic local stub. The CLI sessions command still invokes both fresh report wrappers for its
  hook-conflict diagnostic; callers retaining a Scanner should use `refresh` for the shared pass.
  Changed Claude files use the existing invocation recogniser as a second read; unchanged files do
  neither read. Counters count changed files and their metadata lengths once, not physical read I/O.
  An initial task-output test selected the first background invocation (a negative case) and failed
  (`6 passed; 1 failed`); fixed the test to select the known background parent's id.

### 2026-10-30 — Step 2.8: Real hand-off and waiting check in the sandbox — Claude
- Did: Built release `multi-link`; put the `hook install --scope local` snippet (each command prefixed with
  `MULTI_LINK_HOME=~/multi-link-sandbox/state`) in `~/multi-link-sandbox/repo-a/.claude/settings.local.json`.
  After a marker, ran `claude -p` hand-offs tagged `Step 9.1`: fg, bg, devnull under `env -i`; bg2 (`env -i`)
  and bg3 (inherited env) after bg's hook child came back `null`, with a temporary payload-saving hook; then an
  `expect` session held at a permission prompt for 60 s and answered. Deleted the three background task output
  files, ran `links --json` / `sessions --json` with the sandbox `MULTI_LINK_HOME`, removed the hook config.
- Changed: `docs/phase2-hook.md` (new "Step 2.8 results" section), `PROGRESS.md`.
- Verify: `find ~/.codex/sessions -newer <marker> -name 'rollout-*.jsonl' | wc -l` → `5`, one per marker.
  Per-case compare of `session_meta.payload.id` with the hook's last non-null `child`:
  `fg … hook_child=01a18ba2-f020-… rollout=01a18ba2… equal`; `bg3 … hook_records=2 hook_child=01a18ba5-1239-…
  rollout=01a18ba5… equal`; bg, bg2, devnull `hook_child=null`. `links --json`: fg `exact`/`stdout`/`9.1`;
  bg3 `exact`/`hook`/`9.1`, 0 candidates; devnull `ambiguous`/`null`/`9.1`, 2 candidates; bg `ambiguous`;
  bg2 `heuristic` (correct child). `sessions --json` for 80f7add3 during the hold (07:11:08):
  `{"state":"waiting","state_evidence":"session_record"}`, hook file had 2 `waiting` records
  (`PermissionRequest`, `Notification`/`permission_prompt`); after the answer: `idle`/`session_record`; after
  exit: `unknown`/`no_signal`. `ls ~/multi-link-sandbox/repo-a/.claude/settings.local.json` → No such file.
- Review: Codex (read-only, `review-2.8.txt` in the session scratchpad), 2 findings, both accepted and fixed:
  the verdict now says "qualified" (devnull `ambiguous`, waiting via `session_record`, both against PLAN.md's
  wording), and bg's missing `scratchpad_dir` is marked an inference (only bg2's key list was observed).
  Codex confirmed ids, counts, times and link values match the evidence and the section holds no conversation
  content.
- Gotchas: clean-env `claude -p` (2.1.314) omits `scratchpad_dir` → no hook child for background hand-offs
  (open decision in Current state). `rm "$VAR/…"` is blocked by a safety check; use literal paths. The
  `touch` in the perm session leaves no hook record (only Codex hand-offs are recorded). The
  links JSON field is `match` (there is no `confidence`).

### 2026-10-30 — Step 2.7: CLI `hook record | install | uninstall` — Codex
- Did: Added CLI-only I/O wiring to `multi-link-hook`. `hook record` enters before clap parsing, installs
  the no-op panic hook and starts the watchdog before stdin, retains at most 1 MiB and drains excess,
  rejects oversized input, builds HookEnv from HOME / MULTI_LINK_HOME, and calls record_and_write inside
  catch_unwind with SystemTime / Instant / thread::sleep. Missing HOME, input/write errors and panics
  return silently with exit 0. Debug-only panic/deadline injection is ignored by release builds.
  Install/uninstall print only JSON on stdout and target / cleanup instructions on stderr, with all four
  events, shell-quoted current_exe, async true and timeout 60; user/local scopes resolve the required
  targets without writing settings. Both reports print nonzero hook_conflicts on stderr.
- Changed: `crates/cli/Cargo.toml`, `Cargo.lock`, `crates/cli/src/{main.rs,hook.rs}`,
  `crates/cli/tests/cli.rs`, `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md. `cargo test --workspace` → exit 0. Actual result lines below,
  labeled by target (full local output: `/tmp/multi-link-2.7-workspace.log`):
  - unittests src/main.rs: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - tests/cli.rs: `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.95s`
  - unittests src/lib.rs: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - tests/fixtures.rs: `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s`
  - tests/invocation.rs: `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s`
  - unittests src/lib.rs: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - tests/fixtures.rs: `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s`
  - unittests src/lib.rs: `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - tests/hook.rs: `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s`
  - tests/links.rs: `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s`
  - tests/repo_probe.rs: `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s`
  - tests/sessions.rs: `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s`
  - unittests src/lib.rs: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - tests/hook.rs: `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s`
  - unittests src/lib.rs: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - tests/model.rs: `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - unittests src/lib.rs: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - tests/fixtures.rs: `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - Doc-tests multi_link_claude: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - Doc-tests multi_link_codex: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - Doc-tests multi_link_core: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - Doc-tests multi_link_hook: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - Doc-tests multi_link_model: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - Doc-tests multi_link_plan: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  CLI integration tests cover the synthetic foreground/resume/plugin/failure/waiting/background
  payloads, exact written metadata and wall-time bounds, custom state, malformed/oversized input,
  unwritable state, missing HOME, silent panic, open stdin with 500 ms deadline and 2 s upper bound,
  both setup scopes/defaults, unchanged temporary-home files/directories, executable path quoting,
  stderr-only conflicts and Phase 1 answer keys.
- Verify: `cargo clippy --workspace --all-targets -- -D warnings` → exit 0:
  ```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.37s
  ```
  `cargo fmt --all -- --check` and `git diff --check` → no output, exit 0.
  `cargo build --release -p multi-link` → exit 0:
  ```text
    Finished `release` profile [optimized] target(s) in 2.75s
  ```
- Verify: `/usr/bin/time -p sh -c 'for i in $(seq 50); do target/release/multi-link hook record < /tmp/multi-link-2.7-non-codex.json; done'`
  (synthetic PostToolUse Bash payload with command `echo synthetic`) → exit 0; actual timing:
  ```text
real 0.31
user 0.06
sys 0.05
  ```
- Verify: Each Phase 1 command from `docs/output-schema.md`, using `target/release/multi-link`, piped
  through `jq -S .` and compared with `diff` to `jq -S . fixtures/expected/<command>.json`, followed by
  `&& echo same` → exit 0; actual output in sessions / links / steps order:
  ```text
same
same
same
  ```
- Verify: Release subprocesses with each test env value, stdin held open for 50 ms before sending a
  PermissionRequest, stayed alive and then wrote exactly one waiting record, exit 0, empty stdout/stderr:
  ```text
release ignores panic: ok
release ignores deadline_ms=1: ok
  ```
- Review: Claude, 0 rounds. No code path opens a file for writing (install/uninstall only print; temp-HOME
  snapshot test); `hook record` enters before clap, panic hook + catch_unwind + watchdog first, no stdout,
  exit 0; snippet = 2.1's events/matchers, all `async: true`; CLI holds no parsing/linking logic. Claude
  re-ran: workspace tests all ok, clippy clean, Phase 1 diffs `same` ×3, speed `real 0m0.125s` (bash
  `time`, 50 non-Codex payloads), release watchdog with stdin held open by a FIFO → exit 0 after 30 s,
  0 bytes output; Hook variant `links` / `sessions` → stderr `hook_conflicts: 1`. Nits not fixed:
  `sessions` runs `links_report` just for the count (double parse; a links error fails `sessions`);
  input over 1 MiB is dropped whole rather than truncated (a truncated payload couldn't parse anyway);
  `unreachable!()` Hook arms in `run`.
- Gotchas: `hook_conflicts` exists only on LinksOutput, so `sessions` calls links_report in addition
  to sessions_report to obtain it within this step's file list (extra reads and linker I/O).
  Setup output is directly parseable JSON on stdout; instructions are exclusively on stderr.
  Oversized stdin is drained without retaining excess and produces no record. Watchdog thread spawn
  failure returns before reading stdin. The debug panic/deadline controls are for tests only.

### 2026-10-30 — Step 2.6: Core reads hook records: hook evidence and waiting — Codex
- Did: New `crates/core/src/hook.rs`: fail-soft reader for `<state>/hook/<uuid>.jsonl` (fixture mode
  `<root>/.multi-link`, else `Options.state_dir`). Linker joins handoff records on (parent, tool_use_id):
  transcript sources win, a differing hook child → `hook_conflicts` (on `LinksOutput`, not JSON); otherwise an
  observed hook child → exact / hook, claimed before heuristics. State: record `status: "waiting"` → waiting /
  session_record (H11); waiting / hook only for a transcript session without a record whose latest hook record
  is waiting and later than every main + subagent line; waiting sessions bypass `--since`. CLI fills
  `state_dir` only.
- Changed: crates/core/src/{lib.rs,hook.rs,link.rs,links.rs,sessions.rs,state.rs},
  crates/core/tests/{hook.rs,links.rs,sessions.rs}, crates/cli/src/main.rs
- Verify: `cargo test -p multi-link-core` → all ok (unit 2, hook 6, links 15, repo_probe 9, sessions 11), incl.
  hook_variant_equals_all_three_answer_keys (hook_conflicts == 1); clippy and fmt clean. Coordinator: workspace
  24 binaries ok, clippy clean.
- Review: Claude, 0 rounds. Exact only with an observed child; H11/H8 order right; equal timestamp clears
  waiting; Codex-id hook records ignored. Nits not fixed: a non-NotFound read error on the hook dir fails the
  report; hook_conflicts counts before the `--since` filter; links_report reads hook files twice.
- Gotchas: the Hook variant test copies hidden `.multi-link` and `.claude` dirs explicitly. Printing
  hook_conflicts on stderr is Step 2.7's job.

### 2026-10-30 — Step 2.5: Hook recorder library — Codex
- Did: `multi_link_hook::record(payload, &HookEnv) -> Option<HookRecord>` (no I/O), `write(&HookRecord,
  &HookEnv)`, and `record_and_write(payload, env, &mut impl Clock)`: writes the first record at once, then for a
  background call with no child polls dirname(scratchpad_dir)/tasks/<backgroundTaskId>.output every 250 ms up
  to 25 s and appends one `task_output` record (≤ 2 per tool_use_id). PostToolUse/PostToolUseFailure + Bash +
  `multi_link_claude::recognise` → handoff (child from stdout, or `error` for failures); PermissionRequest and
  Notification permission_prompt|elicitation_dialog → waiting; else None. State dir = MULTI_LINK_HOME else
  HOME/.multi-link, absolute, never inside HOME/.claude or HOME/.codex (also after resolving symlinks);
  O_NOFOLLOW, O_APPEND, mode 0600, one write per line < 4 KiB. 14 synthetic tests.
- Changed: crates/hook/src/lib.rs, crates/hook/tests/hook.rs
- Verify: `cargo test -p multi-link-hook` → 14 passed; clippy -D warnings clean; rustfmt --check clean.
  Coordinator: `cargo test --workspace` with 2.5 + 2.6 together → 24 binaries ok; workspace clippy clean.
- Review: Claude, 0 rounds. Safety checks pass (no stdout, Err not panic, writes stay in the state dir, symlinks
  refused, task id validated, first record before polling). Nits not fixed: hard-linked record file still
  appended to; a FIFO at the task output path blocks open (2.7's watchdog covers it); kind/event are Strings.
  API adds record_and_write and write takes HookEnv (two-record background flow needs it).
- Gotchas: PLAN 2.5 says 25 s in text and 30 s in Verify; implemented 25 s, under 2.7's 30 s watchdog. A
  symlinked MULTI_LINK_HOME is refused. Not yet run as a real hook (2.7/2.8).

### 2026-10-30 — Step 2.4: Hook record contract and fixtures — Claude
- Did: output-schema.md "Hook records": layout `<state>/hook/<uuid>.jsonl` (`$MULTI_LINK_HOME`, else
  `$HOME/.multi-link`; fixture mode `<root>/.multi-link`); v1 record (kind handoff|waiting, at = hook clock at
  write, session_id = file stem, agent_id, event PostToolUse|PostToolUseFailure|PermissionRequest|Notification,
  tool_use_id, cwd, child, child_evidence stdout|thread_ready|task_output, notification_type); background
  calls write a child-null record first, then append the child; reader rules; link join on (session_id,
  tool_use_id), transcript wins, conflicts counted in stderr-only `hook_conflicts`; state `waiting` /
  `session_record` (H11) first, `waiting` / `hook` only for a transcript session without a record. Hook
  variant = fixtures/sessions + overlay fixtures/hook/ − task-output/be01y5dv6.output; 8 synthetic hook files
  + scrubbed 2.1 waiting session record 98798.json.
- Changed: docs/output-schema.md, fixtures/README.md, fixtures/hook/**, fixtures/expected/hook/{links,sessions,sessions-2h}.json;
  PLAN.md 2.4 Verify wording updated for H11 by the coordinator.
- Verify: `jq . fixtures/expected/*.json fixtures/expected/hook/*.json > /dev/null` → ok; bg link via hook →
  exact/hook, same child as task_output; conflict exec link stays exact/stdout; waiting rows in hook/sessions.json
  and hook/sessions-2h.json → [11c1f13c hook, d1f3b188 session_record]; `git diff --stat -- fixtures/expected/*.json`
  → empty; privacy `rg --hidden` → nothing. Re-run by the coordinator: identical.
- Review: Codex (read-only), 3 findings, all accepted and fixed: `at` ambiguous for the appended bg record;
  no fixture where record and hook disagree (added 1f47e9b3: hook waiting vs idle record → stays idle);
  agent_id example was an agent_type.
- Gotchas: fixture dirs are hidden (.multi-link, .claude): plain rg skips them, use --hidden. 98798.json times
  are later than the fixture --now; waiting has no freshness check. Expected hook keys were produced as
  hand-chosen jq edits of the hand-written Phase 1 keys, each edit listed in the schema.

### 2026-10-30 — Step 2.3: Step tags in links, model changes, hook crate skeleton — Codex
- Did: `multi_link_plan::step_tag` (new `tag.rs`): ASCII boundaries checked by hand (no lookahead in Rust
  `regex`), maximal candidate run then `markdown::valid_id`, so one id grammar. `segments()` tracks byte
  spans; `Launch`/`Invocation` gain `step` from the raw launch-segment text only; the linker copies it into
  `Link.step`. Model: `Link.step`, `LinkEvidence::Hook`, `StateEvidence::Hook`; `StepSource::Hook` dropped.
  `multi-link-claude` depends on `multi-link-plan`. Skeleton `crates/hook`, no logic.
- Changed: Cargo.lock, crates/model/{src/lib.rs,tests/model.rs}, crates/plan/{src/lib.rs,src/tag.rs,tests/fixtures.rs},
  crates/claude/{Cargo.toml,src/invocation.rs,tests/invocation.rs}, crates/core/{src/link.rs,tests/links.rs},
  crates/hook/{Cargo.toml,src/lib.rs}
- Verify: `cargo test --workspace` → 22 binaries all ok (the two red tests from 2.2 are green); clippy
  `-D warnings` clean; `cargo fmt --check` clean; Step 1.9 CLI diffs → same ×3. Re-run by the step session and
  the coordinator (22 ok).
- Review: Claude, 0 rounds. Tag scanner equivalent to the schema regex; step read only from the raw launch
  segment; hook crate skeleton only. Nits not fixed: `step` is the first struct field (schema lists it last);
  the Tagged test re-serializes lines (key order changes, content the same).
- Gotchas: a tag split by quotes or escapes (`'St'ep 1`, `Step\ 1.2`) gives null, by design. Nothing produces
  hook evidence yet.

### 2026-10-30 — Step 2.1: Hook payload spike — Claude
- Did: Throwaway async capture hook (allowlist: ids, paths, enums; other strings → `<string:N>`) in
  `~/multi-link-sandbox/repo-a/.claude/settings.local.json` on PostToolUse(Bash), PermissionRequest,
  Notification, PostToolUseFailure, Stop. Ran 8 `claude -p` haiku sessions (fg, bg, bg2, resume, plugin,
  denied permission, two failing codex runs) and 4 interactive ones via `/usr/bin/expect`. Removed the hook
  afterwards. Results: (e) go, (f) go (PermissionRequest/Notification have no tool_use_id), (g) go (no path in
  tool_response; path = dirname(scratchpad_dir)/tasks/<backgroundTaskId>.output), (h) go (-p: PermissionRequest
  only; interactive: PermissionRequest, then Notification permission_prompt ~6 s later), (i) go (no transcript
  line during the hold; the session record shows `status: "waiting"` + `waitingFor`), (j) go (needs env -i,
  --permission-mode default, trust dialog), (k) no-go in -p (unfinished async hooks killed at teardown) →
  write first, append the child later.
- Changed: docs/phase2-hook.md (plus ignored fixtures/raw/hook/)
- Verify: sections `## (e)`…`## (k)` each with a `Decision:` line; allowlist check → no output; sandbox
  `settings.local.json` removed. Re-checked by the coordinator: 7 sections, 7 Decision lines, settings file gone.
- Review: Codex (read-only), 4 findings, all accepted and fixed (kept session-record snapshots and expect logs
  in fixtures/raw/hook/; cited exit stamps; timing reported as an upper bound).
- Gotchas: --allowedTools takes several values and swallowed the prompt; put the prompt first. The trust
  dialog defaults to "No, exit". An interactive session started with the desktop environment writes no
  transcript or session record (use env -i). Esc on a prompt interrupts the turn and fires no Stop.
  Failed commands fire PostToolUseFailure, with the Codex `session id:` line in `error`.

### 2026-10-29 — Step 2.2: Output contract: step tags, hook evidence, waiting — Claude
- Did: output-schema.md: Link gains `step` (string|null) with a "Step tags" section (H6: first tag in the
  launch segment's text, P11 id grammar, read from the command run, indirect prompts → null). Link `evidence`
  gains `"hook"` as source 5 after the four transcript sources (H5: transcript wins, conflict counted, details
  in 2.4); every exact link incl. hook claims its child for the heuristic exclusion unless `resume`.
  `state_evidence` lists `hook`, reserved for 2.4's `waiting` row. Segments unchanged; `step_source` is
  `unknown | prompt_tag`. New "Tagged variant" section. Expected files: every link gains `"step": null`; new
  links-tagged.json = links.json with exec link `toolu_01EGmZWsd8PbTTNmjK67yApw` `step: "1.2"`.
  Answer key first: two Phase 1 tests are red until 2.3.
- Changed: docs/output-schema.md, fixtures/expected/{links,links-amb-hidden,links-tagged}.json; PLAN.md
  Step 2.2 Verify corrected by the coordinator.
- Verify: `jq . fixtures/expected/*.json > /dev/null` → ok; `jq '[.links[]|has("step")]|all' fixtures/expected/links*.json`
  → true true true; tagged vs plain diff without `step` → same; `cargo test --workspace --no-fail-fast` → FAILED
  only model `every_expected_report_round_trips` and cli `phase_one_json_matches_answer_keys` (expected until 2.3).
  Re-run by the coordinator: identical.
- Review: Codex (read-only), 1 P2: the step's Verify named the wrong red tests (core passes because serde drops
  the unknown `step`). Accepted; PLAN.md Verify line fixed. Verdict: schema, fixtures and tag grammar match.
- Gotchas: no struct uses `deny_unknown_fields`, so a new answer-key field never fails a deserialize-only test;
  only Value-level comparisons (model round-trip, CLI) catch it. The tag regex uses lookahead, which the Rust
  `regex` crate lacks, so 2.3 checks the trailing boundary by hand.

### 2026-10-29 — Phase 2 plan: hook — Claude
- Did: Split Phase 2 into Steps 2.1–2.8 with files, Verify, Depends on / Parallel with. Checked hook facts
  against the current Claude Code hooks docs (claude-code-guide agent): events, `PostToolUse` fields,
  `PermissionRequest`, `async` semantics, `-p` behaviour. Recorded decisions H1–H10 in `docs/design.md`
  (spike first; `PostToolUse`/Bash for hand-offs; `PermissionRequest` + `Notification` for needs-me; async,
  silent, watchdog-capped hook; hook evidence after transcript evidence; `Step <id>` tag from the launch
  command; tag on links only; waiting rule; hook crate off core; one append-only file per session).
  Codex critique (read-only): 10 findings, 8 accepted, 1 partly accepted, 1 rejected; table in PLAN.md
  Review log "Phase 2 plan".
- Changed: `PLAN.md`, `docs/design.md`, `PROGRESS.md`
- Verify: PLAN.md has `### Step 2.1` … `### Step 2.8`, each with `- Verify:`, `- Depends on:` and a Review
  log entry "Phase 2 plan" → see the commit; `grep -c '^### Step 2\.' PLAN.md` → `8`.
- Review: Codex (critique of the plan, read-only), verdict "Revise before implementation"; revisions applied.
- Gotchas: no code written. Nothing in the plan writes to `~/.claude/` or `~/.codex/`; the sandbox test uses
  `repo-a/.claude/settings.local.json`. Waiting may need one owner-run interactive check if `expect` can't
  hold a prompt (2.1 (j)); Phase 2 stays open until it passes.

### 2026-10-29 — Step 1.10: Probe compares start times in UTC — Codex
- Did: Set `TZ=UTC` on the probe's `ps` command; kept comparison logic unchanged.
  Added a deterministic stub-ps regression test: matching UTC start → `Alive`, local-time
  rendering → `Gone`. Aligned the own-pid test's reference `ps` command with UTC.
- Changed: `crates/core/src/probe.rs`, `crates/core/tests/repo_probe.rs`, `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md; `cargo test --workspace` → exit 0;
  actual result lines by target:
  - Running unittests src/main.rs: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - Running tests/cli.rs: `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s`
  - Running unittests src/lib.rs: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - Running tests/fixtures.rs: `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s`
  - Running tests/invocation.rs: `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s`
  - Running unittests src/lib.rs: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - Running tests/fixtures.rs: `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s`
  - Running unittests src/lib.rs: `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - Running tests/links.rs: `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s`
  - Running tests/repo_probe.rs: `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s`
  - Running tests/sessions.rs: `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s`
  - Running unittests src/lib.rs: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - Running tests/model.rs: `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - Running unittests src/lib.rs: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - Running tests/fixtures.rs: `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - Doc-tests multi_link_claude: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - Doc-tests multi_link_codex: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - Doc-tests multi_link_core: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - Doc-tests multi_link_model: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - Doc-tests multi_link_plan: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
- Verify: `cargo fmt --all -- --check` → no output, exit 0.
- Review: Claude, 2026-10-29. Diff is the one-line `TZ=UTC` plus a fake-`ps` regression test; no findings.
  Real-data Verify run outside the sandbox: Claude states `{"busy":2,"idle":17}`, matching the 19 live
  `~/.claude/sessions` records exactly (2 busy, 17 idle); before the fix all 19 were `finished`.
- Gotchas: Real-data Verify was not run, per the owner's instruction; left for Claude to run
  outside the sandbox with a desktop session open. Existing real-ps tests can return early
  when the probe is unavailable; the new stub test does not require real process access.

### 2026-10-29 — Step 1.9: CLI commands — Codex
- Did: Wired sessions/links to `multi_link_core` report functions with the same Options, and steps to
  `multi_link_plan::steps_report`. Explicit root selects fixture repo mode and process probe; HOME
  selects real mode/probe. Clap validates RFC 3339 now and checked integer s/m/h/d durations (or all).
  Default step repos preserve first-seen listed-session order and absolute strings; explicit repos
  preserve caller order and duplicates. JSON stays on stdout, non-empty diagnostics get one stderr
  count line per category, report I/O errors exit 1, invalid flags exit 2. Text tables include item
  rows and report-specific count footers. Eight built-binary tests cover answer keys, exact text rows, flags,
  duration units/offsets, repo defaults/order, I/O failure, and all three diagnostic categories.
  Round 1: removed the extra links session scan; links footer counts unparented sessions; steps
  footer counts repos and missing/unsupported repos get rows. Tables pad to column maxima including
  headers, use two-space separators, omit trailing spaces, and put steps titles last. Enum labels
  use serde names; a unit test checks `ProcessExit` → `process_exit`. JSON remains unchanged.
- Separate cleanup (no behavior change): collapsed the four clippy-reported nested conditionals in
  `crates/claude/src/invocation.rs` into equivalent let chains, only within the approved cleanup scope.
- Changed: `crates/cli/src/main.rs`, `crates/cli/tests/cli.rs`, `crates/cli/Cargo.toml`, `Cargo.lock`,
  `PROGRESS.md`; separate cleanup: `crates/claude/src/invocation.rs`.
- Verify: Round 1 rerun, Rust PATH set per AGENTS.md. `cargo test --workspace` → exit 0;
  actual result lines in execution order:
  ```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.92s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
- Verify: the three Step 1.9 `diff <(cargo run ... | jq -S .) <(jq -S . fixtures/expected/...) && echo same`
  commands, run verbatim for sessions, links, steps → each exit 0, actual stdout:
  ```text
same
same
same
  ```
- Verify: `rustfmt --edition 2024 --check crates/cli/src/main.rs crates/cli/tests/cli.rs`
  → no output, exit 0; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0,
  actual completion line:
  ```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.28s
  ```
- Prior implementation real-data smoke (not rerun in Round 1): exit 0; counts were:
  ```text
19
1324
  ```
- Review: Claude, 1 fix round. Round 1 findings (all fixed): links text re-ran sessions_report for a
  footer (double scan); steps footer said "0 skipped lines" with --repo and hid missing repos; enum labels
  came from lowercased Debug, not serde names; columns unaligned. Checked: cli has no parsing/state/linking
  logic; --root → Fixture + FixtureProcessProbe; JSON unchanged. Claude re-ran Verify: workspace tests ok,
  `same` ×3, clippy -D warnings clean, fmt clean. Smoke (release, real ~): sessions 22 listed / 259 all,
  1324 skipped (all Claude unknown_type), 5.6 s; links 35 / 2 unparented, 7.8 s; no crash. Found the
  probe UTC bug (see Current state). Codex's smoke numbers (19 / 1324) were real at its run time.
- Gotchas: Initial Round 1 workspace run caught one missing space in the expected steps header
  (7 integration tests passed, 1 failed); corrected the expectation and reran the whole workspace
  successfully. Plan report I/O behavior keeps its existing missing/unsupported conventions.

### 2026-10-29 — Step 1.8: Linker and links report — Codex
- Did: Implemented evidence priority (resume_arg, stdout, thread_ready, task_output), UUID validation,
  missing-output fallback, inclusive 60 s exact-checkout heuristic, exclusion by exact non-resume
  links, directory-move suppression, deterministic ordering, unparented sessions, and output-only
  history filtering. Added public links_report using sessions Options and diagnostic output fields.
  Ten integration tests compare both complete answer-key reports as LinksReport model values and
  cover all Step 1.8 Verify cases, plus window endpoints, exact path strings, evidence precedence,
  moved commands with exact evidence, and real task-output lookup. Round 1 fixes: check metadata
  for a regular file before reading task output, ignore all metadata/read errors, and sort links
  by (at, invocation). Added a real-mode directory-output regression matching missing-file fallback.
- Changed: `crates/core/src/lib.rs`, `crates/core/src/link.rs`, `crates/core/src/links.rs`,
  `crates/core/tests/links.rs`, `PROGRESS.md`.
- Verify: Round 1 rerun, Rust PATH set per AGENTS.md; `cargo test -p multi-link-core` exited 0.
  Actual test output (runner paths and compilation preamble omitted):
  ```text
  running 2 tests
  test repo::tests::groups_only_common_dirs_named_dot_git ... ok
  test probe::tests::classifies_ps_output_conservatively ... ok

  test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

  running 10 tests
  test resume_does_not_claim_child_from_earlier_exec ... ok
  test old_exact_link_excludes_child_from_recent_heuristic_even_when_hidden ... ok
  test heuristic_window_is_inclusive_and_checkout_is_an_exact_string ... ok
  test since_lists_links_when_either_endpoint_is_visible_and_only_visible_unparented ... ok
  test moved_launch_directory_disables_heuristic_but_preserves_exact ... ok
  test directory_task_output_falls_back_like_missing_output_in_real_mode ... ok
  test exact_evidence_precedence_and_real_task_path ... ok
  test fixture_report_equals_answer_key ... ok
  test missing_background_output_falls_back_to_heuristic ... ok
  test ambiguous_variant_equals_answer_key ... ok

  test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s

  running 8 tests
  test fixture_mode_preserves_even_nonexistent_relative_paths ... ok
  test fake_and_fixture_probes_are_deterministic ... ok
  test own_pid_matches_start_time_and_rejects_reused_identity ... ok
  test reaped_child_is_gone ... ok
  test missing_ps_is_unavailable ... ok
  test real_mapper_falls_back_outside_git_or_for_missing_directory ... ok
  test mapper_ignores_inherited_git_environment ... ok
  test nested_main_and_worktree_map_to_shared_repo ... ok

  test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

  running 10 tests
  test latest_codex_turn_governs_and_open_turn_uses_last_activity ... ok
  test claude_freshness_status_and_probe_precedence ... ok
  test file_diagnostics_are_separate_from_schema_skips ... ok
  test report_trusts_stale_busy_record_when_probe_is_unavailable ... ok
  test since_boundary_is_inclusive_and_future_activity_is_retained ... ok
  test merge_uses_transcript_fields_latest_record_and_bypasses_since ... ok
  test transcript_only_and_incomplete_transcripts_have_no_invented_fields ... ok
  test report_uses_gone_probe_for_merged_and_record_only_sessions ... ok
  test fixture_report_equals_every_expected_field ... ok
  test since_two_hours_filters_only_transcript_history ... ok

  test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

  running 0 tests

  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  Round 1 rerun: `cargo clippy -p multi-link-core --tests` exited 0. Actual output excerpts (four existing
  Claude dependency warnings; diagnostic source excerpts/help omitted):
  ```text
  warning: this `if` statement can be collapsed
     --> crates/claude/src/invocation.rs:96:9
  warning: this `if` statement can be collapsed
     --> crates/claude/src/invocation.rs:97:13
  warning: this `if` statement can be collapsed
     --> crates/claude/src/invocation.rs:202:17
  warning: this `if` statement can be collapsed
     --> crates/claude/src/invocation.rs:267:13
  warning: `multi-link-claude` (lib) generated 4 warnings (run `cargo clippy --fix --lib -p multi-link-claude -- ` to apply 4 suggestions)
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.36s
  ```
- Review: Claude, 1 fix round. #1 `link.rs:122` a task-output read error other than NotFound (directory,
  permission, invalid UTF-8) aborted the whole report, and a FIFO named in a transcript could block the read:
  now only regular files are read and any failure gives no evidence (test with a directory path). #2 `link.rs:70`
  links sorted by (at, parent, invocation); schema says (at, invocation). Re-ran Verify: core 2+10+8+10 passed
  (same as pasted), `cargo test --workspace -q` all ok, clippy warnings only the 4 known in crates/claude.
  Checked against the step: exact only from observed ids in schema order; heuristic uses the exact checkout
  string and inclusive [at, at+60 s]; resume links don't claim children; `dir_moved` → unlinked, `[]`;
  `since` filters output only. Noted, not fixed: transcripts are scanned twice per report (Phase 3 cache).
- Gotchas: Links reuse sessions_report for visibility/diagnostics and separately scan all evidence,
  so transcripts are read more than once per report (no cache yet). All task-output metadata/read
  errors (including non-NotFound and invalid UTF-8) and non-regular paths give no evidence and allow
  heuristic fallback. Only paths whose metadata identifies a regular file are read. Exact ids need not have a
  discovered rollout. PLAN.md, schema, answer keys, crates/claude, and Cargo files unchanged.

### 2026-10-29 — Step 1.7: Session state and sessions report — Codex
- Did: Added pure Claude/Codex state functions and a sessions report that assembles parser
  results, mapped repos, merged records, built segments, filtered history, sorted sessions/skips, and
  returned bad-record/unreadable diagnostics beside the report. Round 1: decoupled process probes
  from repo mode and documented caller-selected fixture probes on Options. Replaced probe-isolation
  test with temp-root report tests for Gone (including different-start identity) on merged and
  record-only sessions, and Unavailable on a stale busy record. Added sorted root-relative `incomplete`
  diagnostics for missing Claude metadata; tests assert missing cwd/timestamps and record-only fallback.
  FixtureProcessProbe report still equals the complete answer key. Ten session tests now cover these
  cases alongside freshness, Codex turns, merge/latest-record selection and history boundaries.
- Changed: `crates/core/Cargo.toml`, `Cargo.lock`, `crates/core/src/{lib.rs,state.rs,sessions.rs}`,
  `crates/core/tests/sessions.rs`, `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md. Round-1 `cargo fmt` → no output, exit 0.
  `cargo test -p multi-link-core` → exit 0; actual test output:
  ```text
running 2 tests
test repo::tests::groups_only_common_dirs_named_dot_git ... ok
test probe::tests::classifies_ps_output_conservatively ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

running 8 tests
test fake_and_fixture_probes_are_deterministic ... ok
test fixture_mode_preserves_even_nonexistent_relative_paths ... ok
test reaped_child_is_gone ... ok
test own_pid_matches_start_time_and_rejects_reused_identity ... ok
test missing_ps_is_unavailable ... ok
test real_mapper_falls_back_outside_git_or_for_missing_directory ... ok
test mapper_ignores_inherited_git_environment ... ok
test nested_main_and_worktree_map_to_shared_repo ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

running 10 tests
test latest_codex_turn_governs_and_open_turn_uses_last_activity ... ok
test claude_freshness_status_and_probe_precedence ... ok
test file_diagnostics_are_separate_from_schema_skips ... ok
test report_trusts_stale_busy_record_when_probe_is_unavailable ... ok
test since_boundary_is_inclusive_and_future_activity_is_retained ... ok
test merge_uses_transcript_fields_latest_record_and_bypasses_since ... ok
test transcript_only_and_incomplete_transcripts_have_no_invented_fields ... ok
test report_uses_gone_probe_for_merged_and_record_only_sessions ... ok
test since_two_hours_filters_only_transcript_history ... ok
test fixture_report_equals_every_expected_field ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
- Verify: `cargo clippy -p multi-link-core --all-targets` → exit 0, no warnings for core.
  Actual diagnostic headers and final line (source/help excerpts omitted):
  ```text
warning: this `if` statement can be collapsed
   --> crates/claude/src/invocation.rs:96:9
warning: this `if` statement can be collapsed
   --> crates/claude/src/invocation.rs:97:13
warning: this `if` statement can be collapsed
   --> crates/claude/src/invocation.rs:202:17
warning: this `if` statement can be collapsed
   --> crates/claude/src/invocation.rs:267:13
warning: `multi-link-claude` (lib) generated 4 warnings (run `cargo clippy --fix --lib -p multi-link-claude -- ` to apply 4 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.35s
  ```
- Review: Claude, 1 fix round. #1 `sessions.rs:137` the probe was gated on `repo_mode != Fixture`, tying two
  settings together and leaving the report-level `finished` path untested → fixed (probe always used; caller picks
  `FixtureProcessProbe`; temp-root tests for Gone / different start on merged and record-only, Unavailable → stale).
  #2 `sessions.rs:50` transcripts missing cwd/timestamps were dropped silently → fixed (`incomplete`, tested).
  Clippy warnings are in Step 1.6's file, not 1.7's Verify: follow-up. Checked: every state carries its evidence;
  no-record and completed/no-turn Codex are `unknown`, never busy/waiting; stale only for busy record / open turn
  (300 s busy, 301 s stale; idle 3 h → idle); `now` passed in, no system clock in core; skipped merged and sorted,
  bad/unreadable files counted beside the report; fixture report equals `sessions.json` as parsed JSON;
  `--since 2h` keeps record-backed sessions; no `invocations` use or linking. Scope: only the named files.
  Claude re-ran Verify: `test result: ok.` 2 / 8 / 10 / 0 passed; `cargo test --workspace -q` all ok; fmt clean.
- Gotchas: Four pre-existing `collapsible_if` warnings in `crates/claude/src/invocation.rs` (Step 1.6)
  remain a follow-up for a later step; that file is unchanged and Clippy is not 1.7's Verify command.
  Multiple records with the same session id use the latest `updated_at`; equal timestamps keep the
  ascending-first path. Missing Claude metadata is reported instead of fabricated; any record stays
  record-only. Timestamp arithmetic uses its canonical RFC3339 form because the model exposes no
  arithmetic API (no model changes). Initial pre-review test run had 8 session tests pass and 1 fail:
  invalid UTF-8 was incorrectly expected to be a Codex file-read error. The shared JSONL reader
  classifies it as `invalid_json`; corrected the test. New sessions tests never invoke ps/git;
  the existing Step 1.5 tests still exercise those tools. Nothing staged, committed, or pushed.

### 2026-10-29 — Step 1.6: Claude hand-off calls and invocation recogniser — Codex
- Did: Added a hand-written quote/escape-aware tokenizer and first-launch recognition for exec,
  UUID-argument resume, and plugin task. Extracted assistant Bash calls from main/subagent transcripts;
  paired raw user tool results by tool_use_id within each file, including text blocks and stdout.
  Added recognition table tests, exact comparison of all 13 fixture tuples, mention exclusion,
  resume argument / plugin output checks, and synthetic metadata/result-pairing coverage.
  Round 1: Kept the first assistant tool_use occurrence per file and id; attached top-level stdout
  only to records with exactly one tool_result block. Added duplicate-record/file regression coverage
  and checked both ids in the two-result record plus stdout on a single-result record.
- Changed: `crates/claude/src/lib.rs`, `crates/claude/src/invocation.rs`,
  `crates/claude/tests/invocation.rs`, `PROGRESS.md`.
- Verify: `export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"`;
  `cargo test -p multi-link-claude` → exit 0, actual output (build path omitted):
  ```text
     Compiling multi-link-claude v0.1.0
      Finished `test` profile [unoptimized + debuginfo] target(s) in 0.90s
       Running unittests src/lib.rs (target/debug/deps/multi_link_claude-289d071f5a06d4fb)

  running 1 test
  test tests::transcript_missing_at_open_is_omitted_but_empty_file_is_read ... ok

  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

       Running tests/fixtures.rs (target/debug/deps/fixtures-d6481f144bfac96e)

  running 8 tests
  test unreadable_session_record_is_counted ... ok
  test missing_directories_are_empty ... ok
  test unexpected_transcript_shapes_do_not_panic_or_fabricate_metadata ... ok
  test unknown_types_contribute_nothing_and_later_lines_are_read ... ok
  test subagents_are_sorted_scanned_and_do_not_change_parent_metadata ... ok
  test malformed_session_records_are_omitted_without_stopping_discovery ... ok
  test transcript_fields_and_skips_match_answer_key ... ok
  test all_session_record_fields_match_source_files ... ok

  test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

       Running tests/invocation.rs (target/debug/deps/invocation-c599b1c275de6260)

  running 4 tests
  test recognition_table ... ok
  test repeated_tool_use_ids_keep_the_first_occurrence_in_each_file ... ok
  test extracts_only_assistant_bash_and_pairs_results_by_id_within_each_file ... ok
  test fixture_invocations_match_all_answer_key_tuples ... ok

  test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s

     Doc-tests multi_link_claude

  running 0 tests

  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```
  `cargo fmt -p multi-link-claude -- --check` → exit 0, no output.
- Review: Claude, 1 fix round. #1 `invocation.rs:186` a tool_use id repeated in two assistant records gave two
  launches (schema: one per tool_use) → fixed (first occurrence per file + id; test). #2 `invocation.rs:195`
  record-level `toolUseResult.stdout` was attached to every tool_result block in the record → fixed (only when
  the record has exactly one; test). Checked: only schema forms count (`--version`, `ls ~/.codex/sessions`,
  `echo codex exec`, `FOO=1`, `task-resume-candidate` → none); resume carries the UUID argument and equals the
  expected child; plugin parent/subagent come from the subagent records; `api-request-blob` ignored; no
  exact-evidence parsing, task-output lookup or linking (1.8 scope). Thread-vs-Turn id and missing task-output
  are 1.8's to test. Scope: only the named files. Claude re-ran Verify: `test result: ok. 1 passed` / `8 passed`
  / `4 passed` / `0 passed`, fmt clean; `cargo test --workspace -q` all ok. Matches the paste.
- Gotchas: Recognition is the schema's limited grammar, not shell evaluation. `invocations` returns
  `io::Result` so permission/read failures are visible. Raw text can repeat stdout present in both
  content and toolUseResult on single-result records; it is retained without text deduplication. Exact evidence and linking remain 1.8.

### 2026-10-29 — Step 1.5: Repo mapper and process probe — Codex
- Did: Repo mapper (`repo.rs`): real/fixture modes, shared main root for worktrees, cwd fallback. Process
  probe (`probe.rs`): real (`ps -p <pid> -o lstart=`), fixture and fake probes with three outcomes. Round 1
  (Claude findings): `repo` = parent of the common dir only when it is `.git` (pure `repo_root` + unit test);
  remove every inherited `GIT_*` variable, not three.
- Changed: `crates/core/src/{lib.rs,repo.rs,probe.rs}`, `crates/core/tests/repo_probe.rs`
- Verify: `cargo test -p multi-link-core` → (Codex, sandbox) `test result: ok. 2 passed`, `test result: ok.
  8 passed`, `test result: ok. 0 passed` (doc); the own-pid and reaped-child tests printed SKIP (sandbox blocks ps).
- Review: Claude, 2 findings, both fixed in 1 round: (1) `repo.rs` took the common dir's parent for
  submodules (`.git/modules/x`) and bare repos (`repo.git`), giving bogus repo paths; (2) only 3 of the `GIT_*`
  env vars were removed. Claude re-ran Verify outside the sandbox with `--nocapture`: `test result: ok. 2
  passed`, `test result: ok. 8 passed`, `test result: ok. 0 passed`, no SKIP lines (real `ps` tests ran: own
  pid → Alive, wrong start → Gone, reaped child → Gone, missing ps → Unavailable). Scope: only the named files.
- Gotchas: ps locale assumed to match `procStart`'s; stderr from ps is treated as `Unavailable`.
- Note: this step's code and PROGRESS.md lines landed in the Step 1.2 commit (a9d72e6): Claude staged them in the
  shared index just as the 1.2 session committed the whole index. Content is identical to what was reviewed;
  history not rewritten. Lesson for parallel steps: commit through a private `GIT_INDEX_FILE`, never the shared index.

### 2026-10-29 — Step 1.2: Claude transcripts and session records — Codex
- Did: `scan(root)` discovers main transcripts (`projects/*/*.jsonl`, one level) and subagents
  (`<id>/subagents/agent-*.jsonl`), reads cwd / branch / first and last timestamp from known records,
  skips unknown types (subagent lines included), and parses `sessions/*.json` records. Round 1: files
  missing at open are left out; bad record files are listed in `bad_records`.
- Changed: `crates/claude/src/lib.rs`, `crates/claude/tests/fixtures.rs`.
- Verify: `cargo test -p multi-link-claude` → exit 0:
  `test result: ok. 1 passed` (unit), `test result: ok. 8 passed` (tests/fixtures.rs), `test result: ok. 0 passed` (doc);
  `cargo fmt -p multi-link-claude -- --check` → no output, exit 0.
- Review: Claude, 1 round. #1 `crates/claude/src/lib.rs:165` a transcript deleted between `read_dir` and
  open failed the whole scan (live `~/.claude`) → fixed (NotFound → omitted; unit test). #2
  `crates/claude/src/lib.rs:124` bad session records dropped silently, against "skip and count" → fixed
  (`bad_records`, 12 bad files asserted). Checked: 13 transcripts' 7 fields equal sessions.json, 5 records
  (1 busy, 4 idle), Claude skipped = repo-e lines 3/4/5, known type list equals the schema, field paths
  match `docs/phase0.md`, no network code, only the step's files changed. Claude re-ran Verify and fmt:
  matches the paste.
- Gotchas: Field paths are from Phase 0 fixtures, not a spec. A vanished main transcript also hides its
  subagents. `.json` entries in `sessions/` that are directories count as bad records. This entry and the
  1.2 "Current state" line landed in the Step 1.4 commit (21d36dd): its `git commit -- PROGRESS.md` took the
  whole working file while this step was mid-edit; the code is in the Step 1.2 commit.

### 2026-10-29 — Step 1.4: PLAN.md / PROGRESS.md parser and steps report — Codex
- Did: Implemented `steps_report`, pure parsers, strict step ids (`N` / `N.M` + one lowercase letter),
  fenced-code exclusion, Verify extraction, exact-id progress attachment, and filesystem fallbacks
  (missing / unsupported, lossy UTF-8, CRLF). Added the fixture comparison and edge-case tests.
- Changed: `crates/plan/src/lib.rs`, `crates/plan/src/markdown.rs`, `crates/plan/src/plan.rs`,
  `crates/plan/src/progress.rs`, `crates/plan/tests/fixtures.rs`
- Verify: `cargo test -p multi-link-plan` → `tests/fixtures.rs`: `test result: ok. 8 passed; 0 failed`
  (fixture_report_matches_answer_key, strict_ids_and_plan_order, fences_boundaries_crlf_and_first_verify,
  progress_results_are_reported_and_case_insensitive, progress_missing_verify_and_invalid_headings,
  report_joins_exact_ids_and_keeps_latest_unmatched_entry, absolute_repo_missing_progress_and_lossy_utf8,
  missing_unsupported_and_unreadable_files); unit tests and doc-tests 0, ok.
- Review: Claude, 0 fix rounds. Re-ran Verify: same 8 passed. Checked against the step: both id styles
  (fixture `0.4` / `3`) and both Verify styles (bold / plain) via the answer key; `0.2a`, `3b`, `0.10` ≠ `0.1`,
  invalid ids rejected; pass / ok / fail / other → unknown; absent PLAN.md → `missing`; no headings or
  unreadable → `unsupported`, never a panic; only the step's files changed. All 16 `### Step` headings in
  this repo's PLAN.md fit the grammar. Non-blocking: titles are not trimmed (trailing spaces in a heading
  leak into `title`); an unclosed ``` fence hides the rest of the file (CommonMark does the same).
- Gotchas: Codex hit a non-ASCII byte literal in a test on first compile; fixed. No dependencies changed.

### 2026-10-29 — Step 1.3: Codex rollouts — Codex
- Did: Sorted recursive discovery of `.codex/sessions/**/rollout-*.jsonl`, first-`session_meta` fields
  (replayed parent metadata ignored), `task_started`/`task_complete` pairing by `turn_id`, latest-turn
  completion helper, skipped lines incl. `unknown_type`. All 9 fixture sessions match.
- Changed: `crates/codex/src/lib.rs`, `crates/codex/tests/fixtures.rs`
- Verify: `cargo test -p multi-link-codex` →
  ```text
  test fixture_sessions_turns_and_skips_match_answer_key ... ok
  (+ 8 unit tests: missing dir, open last turn, earlier unmatched + completed last turn, unknown_type,
   blank/invalid/partial tail, first metadata wins, unusable metadata, recursive sorted discovery)
  test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
  ```
  `cargo clippy -p multi-link-codex --all-targets` → no warnings.
- Review: Claude, re-ran Verify (same 9 passed, clippy clean). Fields match `docs/phase0.md`
  (`session_meta.payload.{id,cwd,git.branch,timestamp}`, `event_msg.payload.{type,turn_id}`); fail-soft
  on unknown records (skip + count, later lines read); no network; only the step's files changed.
  2 low findings accepted, not fixed (recorded in Current state): `crates/codex/src/lib.rs:85` a
  traversal error aborts the whole discovery; `crates/codex/src/lib.rs:177` a `task_started` without a
  timestamp is dropped.
- Gotchas: File read failures go to `Discovery.unreadable` and scanning continues; symlinks are not
  followed; turn times are top-level record timestamps, not `payload.started_at`.

### 2026-10-29 — Step 1.1: Workspace, output model, JSONL reader — Codex
- Did: Created the six-package workspace with shared Phase 1 dependencies; internal dependencies are
  model-only. Added every schema object and enum (Serialize + Deserialize), UTC Timestamp with
  millisecond precision, and a path/BufRead JSONL iterator with recoverable skips and a pending byte
  remainder. Added CLI subcommands/flags with the temporary exit-2 behavior. Nine model tests cover
  the schema, all four expected report round trips, timestamps, malformed fixture continuation,
  CRLF/whitespace, invalid UTF-8, pending valid/incomplete JSON, and I/O errors. Fixed Claude review
  round 1: truncate timestamps at construction, assert same-millisecond equality, and compact this log.
- Changed: `Cargo.toml`, `Cargo.lock`, `crates/model/{Cargo.toml,src/lib.rs,src/time.rs,src/jsonl.rs,tests/model.rs}`,
  `crates/{claude,codex,plan,core}/{Cargo.toml,src/lib.rs}`, `crates/cli/{Cargo.toml,src/main.rs}`, `PROGRESS.md`.
- Verify: Rust PATH set per AGENTS.md; review-fix reruns:
  `cargo fmt --all -- --check` → no output, exit 0;
  `cargo test --workspace` → exit 0, decisive output by package/target:
  - multi-link (unit): `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - multi-link-claude (unit and doc, each): `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - multi-link-codex (unit and doc, each): `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - multi-link-core (unit and doc, each): `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - multi-link-model (unit and doc, each): `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - multi-link-model (tests/model.rs): `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - multi-link-plan (unit and doc, each): `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
- Verify: `cargo build --workspace --all-targets --locked` → ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.20s``, exit 0.
- Verify: `cargo run -q -p multi-link -- --help` → exit 0; Commands list:
  ```text
  Commands:
    sessions
    links
    steps
    help      Print this message or the help of the given subcommand(s)
  ```
- Verify: `cargo run -q -p multi-link -- sessions --root fixtures/sessions --now 2026-10-29T23:15:00Z --since all --json` → `not implemented`, exit 2;
  `cargo run -q -p multi-link -- links --root fixtures/sessions --now 2026-10-29T23:15:00Z --since all --json` → `not implemented`, exit 2;
  `cargo run -q -p multi-link -- steps --root fixtures/sessions --now 2026-10-29T23:15:00Z --since all --json --repo repos/repo-a --repo repos/repo-b` → `not implemented`, exit 2.
- Review: Claude, 1 round. #1 `crates/model/src/time.rs:10` Timestamp kept sub-ms precision for Eq/Ord while
  printing ms, breaking the schema's (started_at, id) order → fixed (truncate at construction). #2 Verify
  paste was ~200 lines → compacted. Checked: all 13 Session keys, enum strings vs schema, round trip of all
  4 expected files, no `skip_serializing_if`, no network code. Claude re-ran fmt, `cargo test -p` for all
  6 packages, `cargo test --workspace` (12 ok results, 9 model tests), `--locked` build, `--help`, stub
  exit 2: matches the paste. All fixture `.jsonl` files end in a newline, so pending-line buffering drops nothing.
- Gotchas: The malformed fixture has six lines, not seven; the initial test assertion was corrected
  after inspecting the fixture (initial test run: 8 passed, 1 failed; review-fix rerun: 9 passed).
  Even complete valid JSON without a newline remains pending. Pending bytes preserve partial UTF-8.
  Timestamp construction truncates to whole milliseconds, including before the Unix epoch, so
  Eq/Ord/Hash match serialized precision. Parser, state-machine and linker logic remain for later
  steps. Options serialize as null without omission.

### 2026-10-29 — Step 0.6: Revise design from Phase 0 — Claude
- Did: Merged `docs/phase0-deps.md` into `docs/phase0.md` as (c) and (d) (content kept; added a results
  table and the Step 0.2 review corrections up top; noted in the method that agtrace was later run).
  Updated `docs/design.md`: new "Phase 0 results and decisions" table (P1–P12, each with a why), linking
  algorithm (forms, evidence order, 60 s window, task-output fallback), state table (waiting only via the
  Phase 2 hook; freshness for busy only; liveness = pid + `procStart`), Reuse row (own parsers, zoetrope
  split credited), step-id suffix, edge cases, plan, open questions. Split Phase 1 into Steps 1.1–1.9
  (Cargo workspace, one crate per parser so parallel steps can't break each other's build), marked
  1.2–1.5 parallel; noted Phase 2/3 changes forced by Phase 0. Ran the Codex critique and applied it.
- Changed: `docs/phase0.md`, `docs/phase0-deps.md` (deleted), `docs/design.md`, `docs/output-schema.md`
  (outside the step's named files: id suffix row, Codex latest-turn rule, `cd`/`-C` heuristic rule —
  the schema is the binding contract, so decisions had to land there too), `PLAN.md`, `PROGRESS.md`
- Verify: `grep -n '^### 2026-10-29 — Phase 0' PLAN.md` →
  `273:### 2026-10-29 — Phase 0: design revision and Phase 1 split (Step 0.6, Codex critique, read-only)`;
  Phase 1 step headings vs Verify lines (awk over PLAN.md between `### Step 1.1` and `### Phase 2`) →
  `steps: 9, verify lines in Phase 1 steps: 9`; `test -e docs/phase0-deps.md` → absent;
  `grep -c '^Decision: ' docs/phase0.md` → `4`.
- Review: Codex critique (`codex exec -s read-only`), 10 findings, all P2: 9 accepted, 1 partly accepted
  (`checkout` output field deferred to Phase 3). Main fixes: probe outcomes Alive/Gone/Unavailable, Codex
  latest-turn rule, no heuristic after `cd`/`-C`, concrete missing-task-output result (474 ms, checked),
  merge and `--since` visibility tests, broken smoke `jq`, design's resume-exclusion wording, PROGRESS.md /
  git-index rule for parallel steps. Table in PLAN.md Review log.
- Gotchas: `procStart` is `ps lstart` text in local time (e.g. `Thu Oct 29 19:40:25 2026`), so the
  liveness check compares strings on the same machine; it is meaningless across machines (fixture mode
  turns the probe off). The `Notification` hook as the waiting source is an assumption for Phase 2.

### 2026-10-29 — Step 0.5: Output schema and expected outputs — Claude
- Did: Wrote `docs/output-schema.md` (flags, value formats, ordering; every field of `sessions`, `links`,
  `steps`; state table with evidence; recognised invocation grammar; exact-evidence order incl. task-output
  lookup; 60 s heuristic; skipped-line reasons; PLAN/PROGRESS parsing). Hand-wrote the expected files from
  `ground-truth.md`, the scrubbed transcripts and session records (values read with ad-hoc jq/python in the
  scratchpad, no project code). 27 sessions (13 Claude transcripts, 5 record-only, 9 Codex): 1 busy,
  4 idle, 22 unknown; 4 skipped lines. 13 links: 8 exact (exec, resume on the same child, plugin via
  subagent, bg via task output, par-main/par-wt distinct, amb-1/amb-2), 5 organic repo-c `unlinked`;
  mention has no link; manual and the synthetic Codex file are `unparented`. Derived variant
  `links-amb-hidden.json`: amb-1 `ambiguous` (746/1822 ms), amb-2 `heuristic` (447 ms).
- Changed: `docs/output-schema.md`, `fixtures/expected/{sessions,links,links-amb-hidden,steps}.json`, `PROGRESS.md`
- Verify: `jq . fixtures/expected/*.json > /dev/null && echo OK` → `jq: OK (4 files)`.
  Trace (`jq -r '.links[]|[.parent,.child,.match]'` + `grep -n "| <parent> | <child>" fixtures/raw/ground-truth.md`):
  ```text
  exact bb792d9c… -> 01a189e2-d57f… : ground-truth.md line 11
  exact ef72378c… -> 01a189e2-d57f… : ground-truth.md line 12
  exact 9d9a0ed8… -> 01a189e3-a439… : ground-truth.md line 13
  exact 07c80c2e… -> 01a189e3-f419… : ground-truth.md line 14
  exact 80cce3e4… -> 01a189e4-7f31… : ground-truth.md line 16
  exact c86d374f… -> 01a189e4-8084… : ground-truth.md line 17
  exact 11bf1b58… -> 01a189e4-d501… : ground-truth.md line 18
  exact f4d83303… -> 01a189e4-d953… : ground-truth.md line 19
  unparented 01a189e5-5b32… : ground-truth.md line 20 (manual)
  mention parent df403bfa… in links: 0 (line 15)
  ```
  Deviation: the 5 `unlinked` organic links (parents 85a715ba…, 11c1f13c…) have no ground-truth row
  (organic data); they trace to `fixtures/README.md:116` and `docs/phase0.md` (organic launches, no child id).
- Review: Codex (read-only), 3 findings, all accepted and fixed in the schema; expected files unchanged:
  (1) state rows overlapped → first-match order, process-exit first, probe-unavailable and unknown-status rules;
  (2) `--since` link visibility undefined → window filters output only, all files parsed, link listed if
  parent or child listed, `--now` scope corrected; (3) partial EOF line → buffered, not `invalid_json`.
- Gotchas: Scrubbed plan samples live in `repos/`, not at any session cwd, so `steps` takes `--repo`.
  `api-request-blob` records repeat `tool_use` ids; only `assistant` records count. Claude file
  85a715ba… starts with 351 inherited records of session a93413d9… (resumed session); id = file stem.
  The busy record (60243.json) is fresh only for `--now` ≤ 23:17:39.714Z.

### 2026-10-29 — Step 0.4: Scrubbed fixtures — Codex
- Did: Scrubbed all 13 Claude transcripts (including the plugin subagent), 5 session records,
  8 Codex rollouts and the background task output. Preserved physical line counts/order, JSON
  structure, session/tool IDs, timestamps/config and linking IDs/output paths; replaced prose and output
  bodies with placeholders and mapped paths/project names. Added synthetic malformed Claude/Codex
  transcripts, dotted/integer step and bold/plain Verify samples, and a ground-truth-matched case index.
- Did (Claude review fixes): Narrowed key redaction to real key shapes/bearer tokens, regenerated
  intact task kinds/commands, replaced creator_user_id with user-dev, and scrubbed organic quoted
  prompts/heredoc bodies with launch-segment recognition. Preserved repo-f and Library separately;
  added independent regression cases, organic length checks and directory-injectivity assertions.
- Changed: `fixtures/sessions/**`, `fixtures/README.md`, `PROGRESS.md`; ignored local helpers
  `fixtures/raw/scrub.py`, `fixtures/raw/check-scrub.py` (plus Python's ignored raw cache).
- Verify: `python3 fixtures/raw/scrub.py` → actual output:
  ```text
PASS: scrubbed 26 JSON/JSONL files (2350 lines) and 1 task-output file (51 lines).
  ```
  `rg -l "/Users/|\bsk-|api_key" fixtures --glob '!raw/**' --glob '!README.md'` → no output (exit 1).
  `rg -il "jarednarvaez|narvaez|jarjar|gmail" fixtures --glob '!raw/**'` → no output (exit 1).
  `python3 fixtures/raw/check-scrub.py` → actual output:
  ```text
PASS: 26 raw JSON/JSONL files, 2350 lines: identical line counts/order, JSON keys, array lengths and scalar types.
PASS: 9703 id/timestamp string values and all numeric/epoch values unchanged at their original locations.
PASS: 14789 type/subtype/role/stop_reason/subagent_type/tool-name values unchanged; session records and Codex metadata/config checked.
PASS: 21 tool-result linking evidence lines retain IDs and scrubbed output paths; task-output banner/header and 51 lines retained.
PASS: synthetic Claude (1 truncated + 1 empty + unknown type) and Codex (1 truncated) cases; no unexpected JSON files; 10 case rows match ground truth; privacy scan includes hidden files.
PASS: 8 task kinds and 2 companion task commands restored; real key/bearer patterns redacted; 8 account IDs replaced with user-dev.
PASS: 442 organic commands checked; quoted prompts/heredoc bodies scrubbed, false launches removed; organic strings <=80 characters after placeholder/id/path normalization.
PASS: 18 distinct raw home directories (including cwd ancestors) map to distinct scrubbed directories; repo-f and Library remain separate.
  ```
  `git status --short --ignored fixtures` → actual output:
  ```text
 M fixtures/README.md
?? fixtures/sessions/
!! fixtures/raw/
  ```
  `rg --hidden -il '/Users/|\bsk-|api_key|jarednarvaez|narvaez|jarjar|gmail|osm-validate|pib-ingest|ta<redacted>' fixtures/sessions` → no output (exit 1).
- Review: Claude, 2026-10-29, 2 rounds. Round 1 findings, all fixed: (1) `sk-` redaction corrupted
  `task-*` values (origin.kind, `codex-companion.mjs task-resume-candidate`) — Verify regex was too broad,
  changed to `\bsk-` in PLAN.md; (2) Codex `creator_user_id` account id kept → `user-dev`; (3) organic
  codex commands kept real prompts and heredoc scripts → `<prompt>`/`<script>`/`<command>`; (4) `osm-validate`
  and `~/Library` both mapped to `/home/dev/project` → `repo-f` and `/home/dev/Library`. Re-review after the fixes: clean.
  Claude also replaced the real name `osm-validate` left in `fixtures/README.md` layout text (doc-only edit).
  Re-ran Verify: step rg (incl. `--hidden`) and name rg print nothing; case-by-case comparison of the
  10 ground-truth rows vs README: MATCH; `check-scrub.py` output identical to above; swept every unique string
  value in the fixtures by eye; nothing under `fixtures/raw/` tracked (`git ls-files fixtures/raw` → 0).
- Gotchas: The task result retains its scrubbed runtime temp path; the corresponding output copy is
  `fixtures/sessions/task-output/be01y5dv6.output` (documented in the layout). JSON validity checks
  exclude that plain-text output and intentionally malformed/empty synthetic lines. Raw helpers
  stay ignored. No stale live-process fixture was made; static old idle records do not prove liveness.

### 2026-10-29 — Step 0.3: Phase 0 spike: dependencies (c, d) — Claude (subagent), run by Claude
- Did: Subagent read zoetrope @b1f31dd and agtrace @7649b7e via `gh api` (no clone/run, per the
  auto-mode safety check). After owner approval, Claude cloned both outside the repo, built agtrace and
  ran it on copies of the sandbox sessions under a throwaway `HOME`.
- Changed: `docs/phase0-deps.md` (new)
- Verify: `grep -n 'Decision:' docs/phase0-deps.md` → `116:Decision: no-go`, `123`/`125` (definition
  lines), `213:Decision: go`. agtrace index: 9 claude_code + 10 codex sessions, 0 with `parent_session_id`.
- Review: Codex, read-only on local clones. 2 P2 findings, both accepted and applied: the live in-memory
  parent path (`workspace/view.rs::link`) is now covered; "neither survives" softened (short commands
  survive whole). Decisions unchanged.
- Gotchas: Codex's read-only sandbox has no network even with the network flag; give it local clones
  via `--add-dir`. agtrace's CLI package is `agtrace` (crate dir `agtrace-cli`).

### 2026-10-29 — Step 0.2: Phase 0 spike: hand-offs and state signals — Codex
- Did: Copied sandbox parents, plugin subagent, all generated Codex children, organic samples,
  live Claude session records, and background output into ignored raw fixtures. Cross-checked all
  10 ground-truth cases: all observed positive child IDs and negative parent expectations agree.
  Documented JSON paths for each invocation form, child origin metadata, real state fields/events,
  the approval-event gap, and a proposed 300-second freshness policy.
- Changed: `docs/phase0.md`, `PROGRESS.md`; ignored `fixtures/raw/.claude/**`,
  `fixtures/raw/.codex/**`, `fixtures/raw/task-output/be01y5dv6.output`,
  `fixtures/raw/inspect-phase0.py`, `fixtures/raw/phase0-evidence.json`,
  `fixtures/raw/check-phase0-privacy.py`.
- Verify: `grep -n 'Decision:' docs/phase0.md` → actual output:
  ```text
  116:Decision: go
  197:Decision: no-go
  ```
  `python3 fixtures/raw/check-phase0-privacy.py` → actual output:
  ```text
  PASS: 0 source-text overlaps of 12 consecutive normalized words across 22 raw transcript/output files.
  PASS: 0 personal-home paths or token patterns; sections (a) and (b) each end with one Decision line.
  Privacy check covers verbatim overlap; metadata-only prose also requires reviewer inspection.
  ```
  `git check-ignore fixtures/raw/phase0-evidence.json` → actual output:
  ```text
  fixtures/raw/phase0-evidence.json
  ```
- Review: Claude, 2026-10-29. Spot-checked exec, plugin and background child ids against
  ground truth independently: all match. Findings, carried into Step 0.6 (no doc edits):
  1. `docs/phase0.md:176` — 300 s freshness must apply to `busy` only. Outside the sandbox,
     `ps -p <pid> -o lstart=` shows all 5 recorded pids alive, including the 3 idle records aged
     6131–9224 s: idle desktop sessions legitimately go hours without an update.
  2. `docs/phase0.md:186` — liveness is confirmable with `ps` (pid + start time vs `procStart`);
     the "unverified" gap was a Codex-sandbox limit, not a data gap.
  3. `docs/phase0.md:197` — the no-go is about *waiting* only; busy / idle / finished are supportable.
     Waiting for Claude needs another source (Phase 2 hook, e.g. a Notification hook); Codex waiting
     stays unsupported. Not a stop-and-ask: the design already has stale/unknown and hook bridges.
  4. `docs/phase0.md:50` — background exact links depend on a temp task-output file that can vanish;
     links must fall back to heuristic when it's gone (doc already says unresolved; design to decide).
- Gotchas: Background child ID requires the task output file; plugin uses a thread ID in a subagent
  result, alongside a different turn ID. Both close-together cases are exact when output IDs exist.
  Found 3 old organic idle records, but process probes returned EPERM and `ps` was sandbox-denied;
  live process identity remains unverified. No explicit approval/waiting event types were found
  in 91 local Codex transcripts. Existing `CLAUDE.md` changes and `docs/phase0-deps.md` were left
  untouched; nothing staged, committed, or pushed.

### 2026-10-29 — Step 0.2a: Generate known hand-offs — Claude
- Did: Created `~/multi-link-sandbox/{repo-a,repo-b}` plus worktree `repo-a-wt`; ran 9 `claude -p` sessions
  (haiku) with fixed `--session-id`s that handed work to Codex in every form, plus one manual `codex exec`.
  Matched each child Codex id by a unique `CASE-<name>:` marker in its prompt.
- Changed: none committed (`fixtures/raw/ground-truth.md`, `~/multi-link-sandbox/`)
- Verify: `grep -c '^| ' fixtures/raw/ground-truth.md` → `12` (header + separator + 10 cases: exec, resume,
  plugin, bg, mention, par-main, par-wt, amb-1, amb-2, manual); "Not made" section explains the stale case.
- Review: no code; Codex cross-checks the ground truth against the transcripts in Step 0.2.
- Gotchas: The plugin rephrases the prompt (marker lost) and launches Codex from a subagent
  (`<session>/subagents/agent-*.jsonl`), not the parent transcript. In zsh, `$CX` with spaces isn't
  word-split, so the first manual run failed and was redone by hand.

### 2026-10-29 — Step 0.1: Repo scaffold — Claude
- Did: Created project rule files, plan, README, gitignore, fixtures rules, and a pointer to the design doc.
- Changed: AGENTS.md, CLAUDE.md, PLAN.md, PROGRESS.md, README.md, .gitignore, docs/design.md, fixtures/README.md
- Verify: `git status --short --untracked-files=all` → `.gitignore`, `AGENTS.md`, `CLAUDE.md`, `PLAN.md`,
  `PROGRESS.md`, `README.md`, `docs/design.md`, `fixtures/README.md` (8 files, all `??`);
  `git check-ignore fixtures/raw/x.jsonl` → `fixtures/raw/x.jsonl`. Matches expected.
- Review: Codex, 2026-10-29. 4 findings, all accepted and fixed: privacy `rg` matched its own README
  (now excludes README.md); sandbox sessions vs the no-writes rule (rule now forbids edits, allows
  harness-written transcripts); PROGRESS.md missing from step file lists (now always allowed);
  `--root fixtures` vs `fixtures/sessions` (Phase 1 now uses `fixtures/sessions`).
- Gotchas: Repo was empty on GitHub; work started on branch `setup/scaffold` (since landed on `main` as 5a61cc1).
