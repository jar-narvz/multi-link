# PROGRESS.md — multi-link

## Current state
**Resume here (2026-10-30).** Phases 0, 1 and 2 are done and committed locally (not pushed). Next: **plan Phase 3**
(TUI home + repo timeline + open, Release 1), which is still "to be split later" in PLAN.md. Use one planning
session, as for Phase 2: decisions with a why, Steps 3.x with Files / Verify / Depends on / Parallel with,
Codex critique logged in PLAN.md's Review log, then one fresh session per step (CLAUDE.md "Autonomous mode").

- **What exists:** Rust workspace `crates/{model,claude,codex,plan,core,hook,cli}`. `multi-link sessions | links |
  steps` (JSON or text; `--root` fixture mode, `--now`, `--since`) and `multi-link hook record | install |
  uninstall` (install only prints). `cargo test --workspace` → 24 binaries ok; clippy `-D warnings` clean.
- **Answer keys:** `fixtures/expected/{sessions,links,steps}.json` (Phase 1), `links-tagged.json`,
  `links-amb-hidden.json`, `fixtures/expected/hook/*` (Hook variant). Verify commands are in PLAN.md 1.9 / 2.x.
- **Key decisions to carry into Phase 3:** exact links only with an observed child id; waiting comes from the
  Claude session record first (H11), hook second; freshness limit applies to `busy` only; liveness = pid +
  start time in UTC (1.10); step ids `N`, `N.M` (letter-suffix decision in design.md); multi-link never writes
  to `~/.claude/` or `~/.codex/`.
- **Open for Phase 3 planning:** (1) background hook evidence fails in clean-environment `claude -p` (no
  `scratchpad_dir`); candidate fallback is the transcript's "Output is being written to" path (PLAN.md Review
  log, Phase 2 close #3). (2) Phase 3 Verify needs the ≤ 2 s home refresh measured on real `~` (not yet measured;
  a full `sessions --json` on real data took ~6 s including a release build, so measure warm).
  (3) Small nits left in 2.5/2.6 entries (hook dir read errors fail the report; hook files read twice).
- **Parallel-step rule:** parallel sessions don't commit; the coordinator commits each step separately
  (learned in 1.2/1.5).
- **Leftovers outside the repo:** `~/multi-link-sandbox/` (sandbox repos, deps clones, `state/hook/` from 2.8)
  and `~/.multi-link/spike/` (Step 2.1 capture). Safe to keep; delete by hand if unwanted.

## Log (newest first)

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
