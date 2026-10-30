# PROGRESS.md — multi-link

## Current state
- Step 0.1 done, committed and reviewed by Codex (4 findings, all fixed).
- Autonomous loop set up (CLAUDE.md "Autonomous mode"): Claude commits locally after review, never pushes.
  Rust 1.99.1 installed; Codex sandbox smoke-tested building a crate with a crates.io dependency.
- `docs/design.md` now holds the full design (exported from claude.ai, repo copy is source of truth).
- PLAN.md revised: Step 0.2a (Claude generates known hand-offs in `~/multi-link-sandbox/`), fixtures
  split into 0.4 (Codex scrubs) and 0.5 (Claude writes schema + expected outputs), design revision is 0.6.
- Step 0.2a done: 10 generated cases with ground truth in `fixtures/raw/ground-truth.md` (stale case not made).
- Step 0.2 investigation complete (Codex), awaiting Claude review: `docs/phase0.md` records
  (a) conditional go for observed exact child IDs and (b) no-go for authoritative state detection.
  Approval/waiting events and a confirmed live-but-stale process remain unverified.
- Step 0.2 reviewed (Claude): (a) go, (b) no-go for *waiting* only; see review notes in the 0.2 entry.
- Step 0.3 done: zoetrope no-go as a library (write our own parsers); agtrace go (doesn't link
  Claude → Codex; confirmed by source and by running it on sandbox copies).
- Step 0.4 done (Codex, reviewed by Claude): scrubbed fixtures in `fixtures/sessions/` (home-shaped root for
  `--root fixtures/sessions`); case list in `fixtures/README.md` matches `fixtures/raw/ground-truth.md`.
  For Step 0.5: ids, timestamps, line numbers and record shapes equal raw, so `docs/phase0.md` line refs
  (e.g. exec result at line 27) hold for the scrubbed copies. Paths: sandbox → `/home/dev/multi-link-sandbox/repo-{a,a-wt,b}`,
  organic → `repo-c`, `repo-d`, `repo-f`; multi-link itself → `/home/dev/project`; temp → `/tmp/dev/...`.
  The bg tool result names `/tmp/dev/.../tasks/be01y5dv6.output`; the copy is `task-output/be01y5dv6.output`, so
  the loader needs a task-output lookup by filename (design choice for 0.5/0.6). Extras: malformed Claude
  (repo-e) and Codex (`00000000-...000f`) files; plan samples in `repos/repo-a` (`N.M`, bold Verify) and
  `repos/repo-b` (`N`, plain Verify), one style per file. Sessions' prompts are placeholders, so step
  assignment from prompt text has no real evidence here; 0.5 must decide what steps.json expects.
  Verify regex changed to `\bsk-` (PLAN.md Review log). Regenerate with `python3 fixtures/raw/scrub.py`.
- Step 0.5 done (Claude, reviewed by Codex): `docs/output-schema.md` is the binding JSON contract;
  expected files in `fixtures/expected/` (sessions, links, steps, links-amb-hidden). Phase 1 Verify:
  `--root fixtures/sessions --now 2026-10-29T23:15:00Z --since all --json` for sessions/links;
  `steps --root fixtures/sessions --repo repos/repo-a --repo repos/repo-b --json`.
  Decisions Step 0.6 must carry into design.md/PLAN.md: transcript-only sessions are `unknown`, not
  `finished` (no exit evidence); fixture mode (`--root`) turns off `ps` and git lookups, so `repo` = `cwd`
  and repo-a-wt is its own repo there; step tags are not recognised yet (tag format is an open question),
  so every segment is `step: null`; heuristic window is 60 s after the call, same exact checkout;
  background task output is read from `<root>/task-output/<file>` in fixture mode.
  Phase 1 tasks the fixtures can't cover (unit tests): `stale` and `finished` states, open Codex turn,
  git root + worktree grouping, Verify `pass`/`fail` parsing, and the derived amb-hidden variant
  (built in a test from a temp copy; see schema "Ambiguous variant").
  Gotcha for 0.6: this repo's own PLAN.md uses ids like `0.2a`, which the supported `N`/`N.M` convention
  doesn't parse; decide whether to allow a letter suffix.
- Step 0.6 done (Claude, critiqued by Codex): Phase 0 closed; `docs/phase0-deps.md` merged into
  `docs/phase0.md`; `docs/design.md` has decisions P1–P12 ("Phase 0 results and decisions"); Phase 1 split
  into Steps 1.1–1.9 in PLAN.md. No stop-and-ask: (a) go, (b) no-go for waiting only, (c) no-go, (d) go.
  Step ids may carry one letter suffix (`0.2a`). Schema changes this step (`docs/output-schema.md`): id
  suffix, Codex "latest turn governs" + `no_signal` row, no heuristic when a `cd` / `-C` moves the launch.
  Expected files unchanged (no fixture hits these cases).
- Step 1.1 done (Codex, reviewed by Claude, 1 round): Cargo workspace `crates/*`, edition 2024.
  For 1.2–1.5 (run in parallel, one crate each; **no Cargo.toml / Cargo.lock edits**):
  - Packages: `multi-link-model`, `-claude`, `-codex`, `-plan`, `-core`; CLI package and binary `multi-link`.
    Test one crate with `cargo test -p <package>` (Rust PATH from AGENTS.md first).
  - Deps already declared (`[workspace.dependencies]`): claude/codex/core: model, serde, serde_json, time;
    plan: model, serde, serde_json; cli: model, clap, serde_json; tempfile as dev-dep everywhere.
    core does not depend on claude/codex/plan (1.7 adds them); cli gets core/plan in 1.9.
  - `multi_link_model` exports the schema types (`Session`, `Segment`, `Skipped`, `Link`, `Candidate`,
    `SessionsReport`, `LinksReport`, `StepsReport`, `RepoSteps`, `Step`, `Entry`, `Verify`) and enums
    (`Harness`, `State`, `StateEvidence`, `StepSource`, `SkipReason`, `InvocationForm`, `Match`,
    `LinkEvidence`, `Convention`, `VerifyResult`, `VerifySource`); all derive Serialize + Deserialize, so
    tests can load `fixtures/expected/*.json` into them. `Link.match` is `r#match` in Rust.
  - `Timestamp::parse(rfc3339)` / `Timestamp::from_epoch_ms(i64)`; truncated to whole ms at construction,
    prints `YYYY-MM-DDTHH:MM:SS.mmmZ`; Ord/Eq match the printed form.
  - `jsonl::JsonlReader::from_path(path, display_path)` (or `::new(BufRead, display_path)`) iterates
    `io::Result<Result<(line_no, Value), Skipped>>`; 1-based lines, CRLF stripped, blank/whitespace →
    `empty_line`, bad JSON → `invalid_json`; an unterminated last line is held in `pending()`, never
    yielded. `display_path` is copied into `Skipped.file` (pass the root-relative path). `unknown_type`
    is the parser's job.
- **Next: 1.2, 1.3, 1.4, 1.5 can run in parallel** (separate crates: claude, codex, plan, core). Then 1.6
  (after 1.2; parallel with 1.3–1.5), 1.7 (after 1.2, 1.3, 1.5; not with 1.6), 1.8 (after 1.6, 1.7),
  1.9 (after 1.4, 1.7, 1.8). Parallel steps: Codex doesn't edit PROGRESS.md (it returns the entry text);
  Claude prepends it and commits with `git add <paths>` then `git commit -m "…" -- <paths>` (PLAN.md Phase 1 rules).
- Organic inspection corrected the earlier grep inference: route-impact has Bash launch candidates;
  physaflow has textual mentions but no tool-use inputs containing `codex` in the inspected files.
- Step 1.3 done (Codex, reviewed by Claude, 0 fix rounds): `multi_link_codex::discover(root)` →
  `Discovery { rollouts, skipped, unreadable }`; `rollouts` sorted by root-relative path, `skipped` holds
  `invalid_json` / `empty_line` / `unknown_type`. `Rollout { id, transcript, cwd, branch, started_at,
  last_activity, turns }`, `Turn { turn_id, started_at, completed_at: Option }` in `task_started` order.
  For 1.7: `Rollout::last_turn_complete()` → `None` (no `task_started` → `no_signal`), `Some(false)` (open
  turn → busy/stale by `last_activity`), `Some(true)` (`turns_complete`). Build `Session` yourself:
  `repo` = `cwd` in fixture mode, `subagents: []`, one segment. `unreadable` (file read errors) is not a
  schema `skipped` reason; 1.7 decides whether to surface it. Known gaps: a directory traversal error
  (e.g. permission denied on a subfolder) returns `Err` for the whole discovery; a `task_started` without a
  parseable top-level `timestamp` is dropped, so the "latest turn" falls back to the one before it.
- Step 1.4 done (Codex, reviewed by Claude, 0 fix rounds): `multi_link_plan::steps_report(root, &[String])`
  → `StepsReport` (relative repos join `root`, absolute kept; `repo` echoes the input string). Also
  `parse_plan(&str) -> Option<Vec<Step>>` (None = unsupported) and `parse_progress(&str) -> Vec<(id, Entry)>`.
  For 1.9: the CLI only passes `--root` and the `--repo` list (default: each session's `repo`) and
  serialises the report; no I/O errors surface (absent PLAN.md → `missing`, unreadable → `unsupported`,
  absent/unreadable PROGRESS.md → no entries). Gotcha: per the schema, `result` is `pass` only when the
  text after `→` starts with pass/ok, so this repo's own entries (`→` followed by a backticked `test result: ok`) read `unknown`.

- Step 1.2 done (Codex, reviewed by Claude, 1 fix round): `multi_link_claude::scan(root)` → `ClaudeScan {
  transcripts, records, skipped, bad_records }`. `Transcript { id, transcript, cwd, branch, started_at,
  last_activity, subagents, subagent_files: [SubagentFile { id, transcript }] }`; metadata fields are
  `Option` (None if no known record carries them). `SessionRecord { pid, proc_start, session_id, cwd,
  status: String, status_updated_at, started_at, updated_at, record }`. All paths root-relative, sorted.
  For 1.6: re-read `Transcript::transcript` and each `subagent_files[].transcript` (join with root) to
  extract tool calls; `known_type` is private, so check `type == "assistant"` yourself. For 1.7: merge
  records into transcripts by `session_id == id`; `bad_records` (unreadable/invalid record files) is not a
  schema `skipped` reason. Known gaps: a transcript deleted between listing and open is left out; other
  I/O errors (e.g. permission denied) fail the whole scan.
## Log (newest first)

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
  subagents. `.json` entries in `sessions/` that are directories count as bad records.

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
