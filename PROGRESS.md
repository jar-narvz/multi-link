# PROGRESS.md — multi-link

## Current state
**Resume here (2026-11-07, latest).** **Step 8.3 is committed (Superpowers trial, done); Step 8.4 is next** (normal
Codex loop again; the trial is over). What 8.4 needs:
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
  - Wall-clock: 1 h 40 min (20:08:54–21:49 PST, step session start to writing this entry; merge follows). Codex review
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
