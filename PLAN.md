# PLAN.md — multi-link

Source of truth for what gets built, in order. Only Claude edits this file.
Full design and rationale: `docs/design.md`.

## Goal
One terminal screen showing every Claude Code and Codex session across my repos: which ones need me,
which Claude session handed work to which Codex session, and where each repo stands in its PLAN.md.

**Release 1** = core + `multi-link` CLI + TUI home screen + repo timeline.
Everything after Release 1 is an ordered, uncommitted backlog.

## Non-goals (Release 1)
Controlling sessions · harnesses other than Claude Code and Codex · cost tracking and history search ·
remote viewing · editing PLAN.md / PROGRESS.md · Codex → Claude links · desktop app · Claude Code mod.

## Steps

### Step 0.1 — Repo scaffold
- Files: `AGENTS.md`, `CLAUDE.md`, `PLAN.md`, `PROGRESS.md`, `README.md`, `.gitignore`,
  `docs/design.md`, `fixtures/README.md`
- Verify: `git status --short --untracked-files=all` lists exactly those 8 files, and `git check-ignore fixtures/raw/x.jsonl`
  prints `fixtures/raw/x.jsonl`.

### Step 0.2a — Generate known hand-offs (Claude)
- In `~/multi-link-sandbox/repo-a` and `repo-b` (small git repos, outside this repo), Claude runs
  `claude -p` sessions that hand work to Codex in each form: `codex exec`, `codex exec resume`,
  the Codex plugin, a background launch; two parallel sessions in repo-a (one in a worktree);
  two hand-offs seconds apart in one checkout; one manual `codex exec` with no Claude parent;
  one Claude session that only mentions `codex` without launching it.
- Claude records the ground truth (which Claude session / tool call launched which Codex id, and when)
  in `fixtures/raw/ground-truth.md`. Ground truth comes from the launches, not from parsing.
- Files: none committed (`~/multi-link-sandbox/`, ignored `fixtures/raw/`).
- Verify: `fixtures/raw/ground-truth.md` lists every case above, or says why one couldn't be made.

### Step 0.2 — Phase 0 spike: hand-offs and state signals (questions a, b)
- Use the sandbox sessions from 0.2a plus organic ones from the owner's repos.
- Copy real sessions into `fixtures/raw/` (git-ignored): 3 Claude → Codex hand-offs across 2 repos,
  including two parallel Claude sessions in one repo (one in a worktree), one `codex resume`,
  one background launch, one manual Codex run in the same folder, one stale session, and one
  ambiguous case: two hand-offs seconds apart in the same checkout. Cases not found in existing
  data are created on purpose; list any that couldn't be made.
- Record in `docs/phase0.md`:
  - (a) for each invocation form (exec, resume, plugin, background): where the child Codex id is
    observable, with field names. Go / no-go for exact links.
  - (b) which fields give busy / waiting / idle / finished for each harness, and a proposed freshness limit.
- Files: `docs/phase0.md` only (plus ignored `fixtures/raw/`).
- Verify: `docs/phase0.md` has sections (a) and (b), each ending in `Decision: go` or `Decision: no-go`,
  with field names only (no conversation content).

### Step 0.3 — Phase 0 spike: dependencies (questions c, d) — can run in parallel with 0.2
- (c) Is zoetrope's core usable as a library crate (public API for parsing and graph model)?
- (d) Does agtrace already link a Claude session to the Codex run it launched? Test it on the 0.2a
  sandbox sessions only (copied into a throwaway `HOME`), checked against `fixtures/raw/ground-truth.md`.
  Read its source first; don't run it if it writes into `~/.claude` / `~/.codex` or makes network calls.
- Third-party sources are cloned / built outside this repo (`~/multi-link-sandbox/deps/`).
- Files: `docs/phase0-deps.md` (separate from 0.2's `docs/phase0.md` so both can run at once;
  merged in Step 0.6).
- Verify: sections (c) and (d) exist, each ending in `Decision: go` / `Decision: no-go`, with the
  version (commit) tested and the command used.

### Step 0.4 — Scrubbed fixtures (Codex)
- Scrub the raw sessions into `fixtures/sessions/` (rules in `fixtures/README.md`), laid out like
  `~/.claude` and `~/.codex` so `--root fixtures/sessions` works. Add a malformed-line case.
  Add sample PLAN.md / PROGRESS.md files covering both step-id and both Verify styles.
- Files: `fixtures/sessions/**`, `fixtures/README.md` (layout section)
- Verify: `rg -l "/Users/|\bsk-|api_key" fixtures --glob '!raw/**' --glob '!README.md'` prints nothing; the case list in
  `fixtures/README.md` matches `fixtures/raw/ground-truth.md` case for case.

### Step 0.5 — Output schema and expected outputs (Claude)
- Claude writes `docs/output-schema.md` (fields of `sessions`, `links`, `steps` JSON) and hand-writes
  `fixtures/expected/{sessions,links,steps}.json` from the ground truth and the scrubbed files,
  including negative and ambiguous cases. Claude writes the answer key so the implementer (Codex)
  never writes the answers its code is tested against.
- Files: `docs/output-schema.md`, `fixtures/expected/*.json`
- Verify: `jq . fixtures/expected/*.json > /dev/null` succeeds; every link in `links.json` traces to a
  line in `ground-truth.md`; Codex reviews the schema and the expected files.

### Step 0.6 — Revise design from Phase 0
- Claude merges `docs/phase0-deps.md` into `docs/phase0.md`, updates `docs/design.md` and this plan with the go / no-go results, gets a Codex critique,
  then splits Phase 1 into steps.
- Verify: Review log below has a Phase 0 entry; Phase 1 has numbered steps with Verify commands.

### Phase 1 — Core + CLI (MVP)
Parsers, repo mapper, state machine, step parser, linker; `multi-link sessions | links | steps`.
Split in Step 0.6. The binding contract is `docs/output-schema.md`; the answer key is `fixtures/expected/`
(Claude-written; Codex never edits either). Decisions P1–P12: `docs/design.md` → "Phase 0 results and decisions".
- Supported PLAN / PROGRESS convention (the owner's real format, seen in route-impact and this repo):
  - PLAN.md step heading: `### Step <id> — <title>`, where `<id>` is `N` or `N.M`, optionally followed by
    one lowercase letter (`0.2a`; decision P11).
  - Verify line inside the step: `- Verify:` or `- **Verify:**`.
  - PROGRESS.md entry heading: `### YYYY-MM-DD — Step <id>: <title>`, optionally `— <agent>`.
  - Anything else → steps `unknown`. Fixtures must include both id styles and both Verify styles.
- Layout: a Cargo workspace, `crates/*` (members by glob), one crate per concern:
  `model` (output types, timestamps, JSONL reader), `claude`, `codex`, `plan` (parsers), `core`
  (repo mapper, process probe, state, linker, reports), `cli` (binary `multi-link`). Packages are named
  `multi-link-<dir>`; the CLI package is `multi-link`. Why: a format change touches one crate (swappable
  parsers), and parallel step sessions share one checkout, so a crate boundary keeps one step's
  half-written code from breaking another step's `cargo test -p`.
- Rules for every Phase 1 step: tests read fixtures via `env!("CARGO_MANIFEST_DIR")/../../fixtures/...`;
  JSON is compared as parsed `serde_json::Value`, never as bytes. Only Step 1.1 and steps that say so may
  change a `Cargo.toml` or `Cargo.lock`; needing another crate elsewhere is a stop-and-say.
- "Parallel with" = no shared code files, no dependency, and neither compiles the other's crate. Steps
  that share a crate never run in parallel. `PROGRESS.md` and the git index are still shared, so in
  parallel steps: Codex does not edit `PROGRESS.md` but ends its run with the entry text; the step's
  Claude session re-reads `PROGRESS.md` and prepends the entry just before committing, and commits with
  `git add <paths>` then `git commit -m "…" -- <paths>` so another session's staged files never ride along.
- Verify (whole phase): `cargo test --workspace` passes; each command in `docs/output-schema.md` →
  "Phase 1 Verify commands" equals its expected file; negative fixtures produce no link; the ambiguous
  variant produces `ambiguous`. Step 1.9 runs exactly this.

### Step 1.1 — Workspace, output model, JSONL reader
- Files: `Cargo.toml` (workspace, `members = ["crates/*"]`), `Cargo.lock`,
  `crates/model/{Cargo.toml,src/lib.rs,src/time.rs,src/jsonl.rs,tests/model.rs}`; skeletons (Cargo.toml +
  empty `src/lib.rs`, or `src/main.rs` for cli) for `crates/{claude,codex,plan,core,cli}`, declaring every
  external crate Phase 1 needs (serde, serde_json, a date-time crate, clap; tempfile as a dev-dependency)
  and each crate's dependency on `model` only. `core` must not depend on `claude`/`codex`/`plan` yet.
- Model: serde types for every object in `docs/output-schema.md` (Session, Segment, Skipped, Link,
  Candidate, StepsReport, RepoSteps, Step, Entry, Verify) with every field always serialised (`null`/`[]`,
  never omitted); a `Timestamp` that prints RFC 3339 UTC with exactly 3 fractional digits.
- JSONL reader: yields `(line_no, serde_json::Value)` or a `Skipped` with `invalid_json` / `empty_line`;
  an unterminated last line is buffered, not reported. `unknown_type` is left to each parser.
- CLI skeleton: `multi-link` with subcommands `sessions`, `links`, `steps` and the flags in the schema;
  each prints "not implemented" and exits 2 for now.
- Verify: `cargo test --workspace` → `test result: ok` for every crate, with tests in `tests/model.rs`:
  (1) a Session with all-null optional fields serialises with all 13 keys; (2) epoch ms `1793315559714`
  → `"2026-10-29T23:12:39.714Z"`; (3) reading the malformed Claude fixture
  (`.claude/projects/-home-dev-repo-e/…0e.jsonl`) gives skipped line 3 `invalid_json` and line 4
  `empty_line` and still yields line 5 and later lines; (4) a temp file whose last line has no newline
  yields no skipped entry for it. And `cargo run -q -p multi-link -- --help` lists `sessions`, `links`, `steps`.

### Step 1.2 — Claude transcripts and session records
- Depends on: 1.1. Parallel with: 1.3, 1.4, 1.5
- Files: `crates/claude/src/**`, `crates/claude/tests/fixtures.rs`
- Discovers `<root>/.claude/projects/*/*.jsonl` (main transcripts) and `<id>/subagents/agent-*.jsonl`;
  parses `<root>/.claude/sessions/*.json`. Per transcript: `id`, `transcript`, `cwd`, `branch`,
  `started_at`, `last_activity`, `subagents` by the schema's Session rules; unknown top-level `type` →
  `unknown_type` skipped (known list in the schema). Per record: `pid`, `procStart`, `sessionId`, `cwd`,
  `status`, `statusUpdatedAt`, `startedAt`, `updatedAt`.
- Verify: `cargo test -p multi-link-claude` → `test result: ok`, including a test that, for each of the 13
  Claude sessions in `fixtures/expected/sessions.json` with a non-null `transcript`, the fields `id`,
  `transcript`, `cwd`, `branch`, `started_at`, `last_activity`, `subagents` are equal; 5 records parsed
  (1 `busy`, 4 `idle`); the Claude entries of `skipped` equal the expected file's (repo-e lines 3, 4, 5).

### Step 1.3 — Codex rollouts
- Depends on: 1.1. Parallel with: 1.2, 1.4, 1.5
- Files: `crates/codex/src/**`, `crates/codex/tests/fixtures.rs`
- Discovers `<root>/.codex/sessions/**/rollout-*.jsonl`. Per file: `id` (first `session_meta.payload.id`),
  `transcript`, `cwd`, `branch`, `started_at`, `last_activity`, and turns (`task_started` /
  `task_complete` paired by `turn_id`); unknown top-level `type` → `unknown_type`.
- Verify: `cargo test -p multi-link-codex` → `test result: ok`, including: for each of the 9 Codex sessions
  in `fixtures/expected/sessions.json`, the same 6 fields are equal; rollout `01a189e2-…` (exec + resume)
  has 2 turns, both complete; the Codex `skipped` entry (synthetic file line 2, `invalid_json`) matches.

### Step 1.4 — PLAN.md / PROGRESS.md parser and steps report
- Depends on: 1.1. Parallel with: 1.2, 1.3, 1.5
- Files: `crates/plan/src/**`, `crates/plan/tests/fixtures.rs`
- Implements `docs/output-schema.md` → `steps` for a list of repo folders (relative paths resolve
  against the root).
- Verify: `cargo test -p multi-link-plan` → `test result: ok`, including: the report for
  `fixtures/sessions` with repos `repos/repo-a`, `repos/repo-b` equals `fixtures/expected/steps.json`;
  unit tests for id `0.2a`, `0.10` ≠ `0.1`, results `pass` / `ok` / `fail` / other → `unknown`, a folder
  without PLAN.md → `missing`, a PLAN.md without step headings → `unsupported` (temp dirs).

### Step 1.5 — Repo mapper and process probe
- Depends on: 1.1. Parallel with: 1.2, 1.3, 1.4
- Files: `crates/core/src/{lib.rs,repo.rs,probe.rs}`, `crates/core/tests/repo_probe.rs`
- Repo mapper: real mode resolves `cwd` → git root of the main checkout (`git rev-parse
  --show-toplevel` / `--git-common-dir`, local only) while keeping the checkout; fixture mode returns
  `cwd`. Probe: a trait with a real `ps -p <pid> -o lstart=` implementation (alive iff the output equals
  `procStart`) and a fake for tests. Three outcomes: `Alive`; `Gone` (`ps` ran and printed nothing for the
  pid, or a different start time = pid reused); `Unavailable` (`ps` missing, failed to run, permission
  error). Fixture mode always gives `Unavailable`.
- Verify: `cargo test -p multi-link-core` → `test result: ok`, including: a temp git repo plus `git
  worktree add`, with sessions in a nested subfolder of each → both map to the main repo root with
  distinct checkouts; fixture mode returns `cwd`; real probe: the test's own pid with its `lstart` →
  `Alive`; same pid with a wrong start string → `Gone`; a spawned child that has exited and been reaped
  → `Gone`; probe pointed at a non-existent `ps` binary → `Unavailable`.

### Step 1.6 — Claude hand-off calls and invocation recogniser
- Depends on: 1.2. Parallel with: 1.3, 1.4, 1.5 (no shared crate)
- Files: `crates/claude/src/**`, `crates/claude/tests/{fixtures.rs,invocation.rs}`
- Extracts Bash `tool_use` calls from `assistant` records only (main and subagent transcripts; ignore
  `api-request-blob` copies) with `id`, `at`, `checkout`, `subagent`, `command`, `run_in_background`,
  and the matching `tool_result` text (`message.content` and `toolUseResult.stdout`). Recognises launches
  by the schema's grammar (`exec` / `resume` / `plugin`).
- Verify: `cargo test -p multi-link-claude` → `test result: ok`, including table tests: `codex exec "x"` →
  exec; `codex exec resume <uuid>` and `codex resume <uuid>` → resume with the id; `cd d && codex exec
  "a; b"` → one exec; `node /p/codex-companion.mjs task …` → plugin; `… task-resume-candidate`,
  `codex --version`, `ls ~/.codex/sessions`, `FOO=1 codex exec` → none. Fixture test: exactly 13 recognised
  launches, and their `(parent, invocation, form, background, at, checkout, subagent)` equal those of
  the 13 links in `fixtures/expected/links.json`; the mention session has none.

### Step 1.7 — Session state and sessions report
- Depends on: 1.2, 1.3, 1.5. Not parallel with 1.6 (core will compile the claude crate).
- Files: `crates/core/Cargo.toml`, `Cargo.lock` (add `claude`, `codex` deps),
  `crates/core/src/{lib.rs,state.rs,sessions.rs}`, `crates/core/tests/sessions.rs`
- State rules in schema order (process exit first; 300 s freshness for `busy` only), segments, record-only
  sessions, `--since` window, ordering, merged `skipped`.
- Verify: `cargo test -p multi-link-core` → `test result: ok`, including: the sessions report for
  `fixtures/sessions`, now `2026-10-29T23:15:00Z`, since all, fixture mode, equals
  `fixtures/expected/sessions.json`; unit tests: busy record at `statusUpdatedAt` + 301 s → `stale`;
  idle record 3 h old → `idle`; fake probe "gone" and "different start" → `finished`/`process_exit`;
  open Codex turn ≤ 300 s → `busy`/`open_turn`, older → `stale`; an earlier unmatched turn followed by a
  completed last turn → `unknown`/`turns_complete`; probe `Unavailable` → record trusted, never
  `finished`; `--since 2h` drops old transcript sessions but keeps record-only ones. Merge test (temp
  root): a record whose `sessionId` equals a transcript's id gives one session with the transcript's
  fields, `record` set and the record's state, and it stays listed when the transcript is outside `--since`.

### Step 1.8 — Linker and links report
- Depends on: 1.6, 1.7
- Files: `crates/core/src/{lib.rs,link.rs,links.rs}`, `crates/core/tests/links.rs`
- Exact evidence in schema order (`resume_arg`, `stdout`, `thread_ready`, `task_output` with the
  fixture-mode lookup `<root>/task-output/<file name>`), then the 60 s same-checkout heuristic,
  `unparented`, `--since` visibility.
- Verify: `cargo test -p multi-link-core` → `test result: ok`, including: links report for
  `fixtures/sessions` equals `fixtures/expected/links.json`; the "Ambiguous variant" test from the schema
  (temp copy with the two `session id:` lines replaced) equals `fixtures/expected/links-amb-hidden.json`;
  a temp copy without `task-output/be01y5dv6.output` turns the bg link (parent `07c80c2e…`) into
  `heuristic`, child `01a189e3-f419-79a1-b801-38f3fecd7480`, `evidence: null`, candidates
  `[{id: that child, gap_ms: 474}]`, `unparented` unchanged; a command `cd x && codex exec …` and one with
  `codex exec -C x …`, without exact evidence, give `unlinked` with `candidates: []` even when a Codex run
  started in the record's cwd; a resume link to a child doesn't remove that child from an earlier exec's
  heuristic candidates. `--since` (temp roots, `--since 1h`): old parent + recent child → link listed;
  recent parent + old child → listed; an old exact link still excludes its child from a recent call's
  candidates; `unparented` lists only listed sessions.

### Step 1.9 — CLI commands
- Depends on: 1.4, 1.7, 1.8
- Files: `crates/cli/src/**`, `crates/cli/tests/cli.rs`, `crates/cli/Cargo.toml`, `Cargo.lock`
  (add `core`, `plan` deps)
- `sessions`, `links`, `steps` with the schema's flags; `--json` prints the reports; without `--json`, a
  plain table (one line per session / link / step). `--root` turns on fixture mode.
- Verify: `cargo test --workspace` → all `ok`; and each of these prints `same`:
  ```
  diff <(cargo run -q -p multi-link -- sessions --root fixtures/sessions --now 2026-10-29T23:15:00Z --since all --json | jq -S .) <(jq -S . fixtures/expected/sessions.json) && echo same
  diff <(cargo run -q -p multi-link -- links --root fixtures/sessions --now 2026-10-29T23:15:00Z --since all --json | jq -S .) <(jq -S . fixtures/expected/links.json) && echo same
  diff <(cargo run -q -p multi-link -- steps --root fixtures/sessions --repo repos/repo-a --repo repos/repo-b --json | jq -S .) <(jq -S . fixtures/expected/steps.json) && echo same
  ```
  Real-data smoke (read-only, output not saved): `cargo run -q -p multi-link -- sessions --json >
  /tmp/ml-smoke.json && jq '(.sessions|length), (.skipped|length)' /tmp/ml-smoke.json && rm
  /tmp/ml-smoke.json` exits 0 and prints two numbers; paste only the numbers.

### Step 1.10 — Probe compares start times in UTC (bug found by the 1.9 real-data smoke test)
- Cause: `~/.claude/sessions/<pid>.json` `procStart` is UTC; `ps -o lstart=` prints local time, so every
  live session's start time mismatches and is reported `finished` (19/19 Claude sessions on real data).
- Fix: run `ps` with `TZ=UTC` in its environment; add a unit test that feeds a UTC `procStart` and a
  matching UTC `lstart` line, and one where only the local-time rendering would match (must not be alive).
- Files: `crates/core/src/probe.rs`, `crates/core/tests/repo_probe.rs`
- Verify: `cargo test --workspace` → all `ok`; and on real data
  `cargo run -q -p multi-link -- sessions --json | jq '[.sessions[]|select(.harness=="claude")|.state]|group_by(.)|map({(.[0]):length})|add'`
  shows at least one Claude session that is not `finished` while a desktop session is open.

### Phase 2 — Hook
An optional Claude Code hook records hand-offs and "needs me" moments as they happen. Decisions H1–H10:
`docs/design.md` → "Phase 2 decisions". Hook facts below come from the Claude Code hooks docs
(code.claude.com/docs/en/hooks, read 2026-10-29, Claude Code 2.1.303); they are assumptions until Step 2.1
confirms them on real payloads, like Phase 0 did for transcripts.
- **Safety rules (every step):**
  - multi-link never writes to `~/.claude/` or `~/.codex/`, settings included. `multi-link hook install`
    **prints** the settings snippet and the file to paste it into; `hook uninstall` prints what to remove.
    Neither writes anything anywhere.
  - The hook writes only below `$MULTI_LINK_HOME` (default `~/.multi-link`), in `hook/<session_id>.jsonl`;
    a `session_id` that is not UUID-shaped is dropped, so a payload can't steer the path. Containment: the
    state dir must be absolute, and after resolving it the hook refuses (writes nothing) if it is inside
    `~/.claude` or `~/.codex`; `hook/` and the record file are opened without following symlinks
    (`symlink_metadata` check + `O_NOFOLLOW`), so a planted link can't redirect the append (Codex P1 #1).
  - Every hook entry is `"async": true`: Claude doesn't wait for it, and any decision it printed would be
    ignored, so the hook can't approve, deny or block anything (docs: decision fields have no effect on
    async hooks). It also prints nothing, always exits 0, and stops itself within 30 s (async hooks get no
    enforced timeout): a watchdog thread started first calls `process::exit(0)` at the deadline, which also
    covers stalled stdin or a blocked write; a no-op panic hook plus `catch_unwind` keep panics silent.
  - Records hold ids, times and enum values only: no command text, prompt, or tool output.
- **Events** (H2, H3): `PostToolUse` matcher `Bash` (hand-offs; `tool_use_id` ties the record to the
  transcript's `tool_use.id`); `PermissionRequest` and `Notification` matcher
  `permission_prompt|elicitation_dialog` ("needs me"). `idle_prompt` is not "needs me": an idle session
  is already `idle` from its session record. Codex waiting stays unsupported.
- **Step tags** (H6): the first `Step <id>` (case-insensitive `step`, `<id>` in the P11 grammar) in a
  recognised launch segment's text is that hand-off's step. Read from the Bash command actually run,
  because the Codex plugin's subagent rewrites the prompt it was given; a rewrite that drops the tag gives
  `step: null`, never a guess. Convention for hand-off prompts (already in CLAUDE.md): start with
  "Read AGENTS.md, then do Step <id> in PLAN.md." Parsed from transcripts, so it works without the hook
  and on history.
- Supported test environment: `~/multi-link-sandbox/repo-a` with the hook in that repo's own
  `.claude/settings.local.json` (project scope; if 2.1 finds `-p` ignores that file, `.claude/settings.json`
  in the same sandbox repo). Claude runs it unattended. Enabling the hook for the owner's real repos stays
  the owner's paste of `hook install` output.
- Parallel rules from Phase 1 apply (Codex returns the PROGRESS entry text; the step session prepends it
  and commits with `git commit -- <paths>`). Only steps that say so change a `Cargo.toml` / `Cargo.lock`.
- Verify (whole phase, Step 2.8 runs it): for one real background hand-off and one foreground hand-off,
  the child id the hook records equals the id read independently from `~/.codex/sessions` (the only new
  rollout since a marker file); a hand-off without an observable id (output sent to `/dev/null`) stays
  `heuristic` (or `unlinked`), never `exact`; a sandbox session held at a permission prompt shows
  `waiting` / `hook` in `multi-link sessions --json`, and no longer does after the prompt is answered.

### Step 2.1 — Hook payload spike (Claude)
- Depends on: Phase 1. Parallel with: 2.2
- In the sandbox, a throwaway capture hook (`async: true`, `… || true`) in
  `repo-a/.claude/settings.local.json` for `PostToolUse` (Bash), `PermissionRequest`, `Notification` (all
  types), `PostToolUseFailure`, `Stop`. It stores an allowlist, not whole payloads (Codex P1 #2): a `jq`
  filter keeps ids, times, paths and enum fields verbatim (`session_id`, `transcript_path`, `cwd`,
  `hook_event_name`, `tool_name`, `tool_use_id`, `agent_id`, `notification_type`,
  `tool_input.run_in_background`, any `/…\.output` path and `session id:` / `Thread ready (` uuid found in
  `tool_response`) and replaces every other value with its JSON type and length, plus the hook's own
  receive time; output to `~/.multi-link/spike/<event>.jsonl`. Claude runs `claude -p` hand-offs with
  `--allowedTools` for codex: foreground `codex exec`, background `codex exec`, `codex exec resume <uuid>`,
  the Codex plugin, and `codex exec … > /dev/null 2>&1`; then one non-allowed command to trigger a
  permission request (`-p` denies it); then tries an interactive `claude` session driven by `/usr/bin/expect`
  (tmux is not installed) held at a permission prompt and then answered.
- Raw payloads go to `fixtures/raw/hook/` (git-ignored); the capture hook is removed afterwards.
- Answer in `docs/phase2-hook.md`, field names only, each question ending `Decision: go` / `no-go`:
  (e) do `settings.local.json` hooks run in `-p`? (f) does `session_id` equal the transcript stem and
  `tool_use_id` the transcript's `tool_use.id`, including for the plugin's subagent call (`agent_id`)?
  (g) what is `tool_response` for a background call, and where is the task output path (or how is it
  derived)? how many ms after the hook fires does the `session id:` line appear in that file?
  (h) which of `PermissionRequest` / `Notification` fire for a permission prompt (interactive and `-p`),
  with which `notification_type`? (i) between the prompt appearing and being answered, does Claude write
  any transcript line or session-record update (if so, `last_activity` can't clear `waiting`; name the
  clearing signal instead)? (j) can an `expect`-driven interactive session be held at a prompt unattended
  (needed for 2.8's waiting Verify)? (k) does an async hook still polling survive until it writes, in `-p`
  (docs: unfinished async hooks are killed at teardown) and interactively? Test with a capture hook that
  sleeps 10 s before writing, on a background hand-off; if killed in `-p`, 2.5 records the hand-off first
  (child `null`) and appends a second record when the id appears (Codex P2 #8).
- Files: `docs/phase2-hook.md` only (plus ignored `fixtures/raw/hook/`).
- Verify: `docs/phase2-hook.md` has sections (e)–(k), each ending in `Decision: go` or `Decision: no-go`;
  `jq -r '.. | strings' ~/.multi-link/spike/*.jsonl | rg -v '^(<[a-z]+:[0-9]+>|[0-9a-f-]{36}|toolu_\w+|/.+|[A-Za-z_]+|[0-9T:.Z-]+)$'`
  prints nothing (only allowlisted value shapes were stored);
  `ls ~/multi-link-sandbox/repo-a/.claude/settings.local.json` fails (capture hook removed);
  `git status --short` shows only `docs/phase2-hook.md` (and PROGRESS.md).
- Stop and ask if (f) is no-go (no way to tie a hook record to a transcript call) or if hooks can run only
  from `~/.claude/` settings.

### Step 2.2 — Output contract: step tags, hook evidence, waiting (Claude)
- Depends on: Phase 1. Parallel with: 2.1 (nothing here depends on hook payloads; the `waiting` row,
  which does, is in 2.4)
- `docs/output-schema.md`: Link gains `step` (string|null, H6 tag of the launch segment); `evidence` gains
  `"hook"`, checked after the four transcript sources (H5); `state_evidence` gains `hook` (row defined in
  2.4); Segments stay one per session with `step: null` (H7: per-hand-off segments deferred);
  `step_source` `hook` removed from the contract.
- Answer key first: after this commit `links.json` has `step` and the Phase 1 tests are red until 2.3
  lands (as in Phase 0.5 → 1.x). 2.3 starts right after; only docs steps may run in between.
  A "Tagged variant": a temp copy in which the exec case's `tool_use.input.command` has
  `Read AGENTS.md, then do Step 1.2 in PLAN.md. ` inserted at the start of its prompt argument (the
  `tool_use` only; results and other records unchanged).
- Expected files: every link in `links.json` and `links-amb-hidden.json` gains `"step": null` (fixture
  prompts are placeholders); new `fixtures/expected/links-tagged.json` (exec link `step: "1.2"`, all
  else equal to `links.json`).
- Files: `docs/output-schema.md`, `fixtures/expected/{links,links-amb-hidden,links-tagged}.json`
- Verify: `jq . fixtures/expected/*.json > /dev/null` succeeds;
  `jq '[.links[]|has("step")]|all' fixtures/expected/links*.json` prints `true` ×3;
  `diff <(jq -S 'del(.links[].step)' fixtures/expected/links.json) <(jq -S 'del(.links[].step)' fixtures/expected/links-tagged.json) && echo same`
  prints `same`; `cargo test --workspace --no-fail-fast` fails only in model
  `every_expected_report_round_trips` and cli `phase_one_json_matches_answer_keys` (expected red until 2.3;
  core stays green because serde drops the unknown `step` key); Codex reviews the schema diff (read-only).

### Step 2.3 — Step tags in links, model changes, hook crate skeleton (Codex)
- Depends on: 2.2. Parallel with: 2.4 (2.4 puts its expected files in `fixtures/expected/hook/`, which the
  model round-trip test's non-recursive scan skips, so 2.4 can't turn 2.3's Verify red)
- Model: `Link.step: Option<String>`; `LinkEvidence::Hook`; `StateEvidence::Hook`; drop `StepSource::Hook`.
- Tag parser: `multi_link_plan::step_tag(text) -> Option<String>` reusing the plan crate's id grammar (one
  grammar, one place); `multi_link_claude::Invocation` gains `step` from the launch segment;
  `multi-link-claude` depends on `multi-link-plan`. Linker copies it into `Link.step`.
- Skeleton `crates/hook/{Cargo.toml,src/lib.rs}` (package `multi-link-hook`, deps model, claude, serde,
  serde_json, time; tempfile dev) so Steps 2.5 and 2.6 never edit a manifest in parallel.
- Model test: `every_expected_report_round_trips` learns `links-tagged.json`.
- Files: `crates/model/src/lib.rs`, `crates/model/tests/model.rs`, `crates/plan/src/**`,
  `crates/plan/tests/fixtures.rs`, `crates/claude/{Cargo.toml,src/invocation.rs}`,
  `crates/claude/tests/invocation.rs`, `crates/core/src/link.rs`, `crates/core/tests/links.rs`,
  `crates/hook/{Cargo.toml,src/lib.rs}`, `Cargo.lock`
- Verify: `cargo test --workspace` → all `ok`, including: `step_tag` table tests (`do Step 1.2 in` → `1.2`;
  `step 4` → `4`; `Step 0.2a` → `0.2a`; `Step 0.10` → `0.10`; `Steps 1` / `step1.2` / `Step 1.2.3` → none;
  two tags → the first); the links report equals `links.json`; the Tagged variant test equals
  `links-tagged.json`; and the three Phase 1 CLI diffs (Step 1.9 Verify) still print `same`.

### Step 2.4 — Hook record contract and fixtures (Claude)
- Depends on: 2.1, 2.2. Parallel with: 2.3 (no code, and its expected files live in a subfolder, see 2.3)
- `docs/output-schema.md` → "Hook records": file layout (`<state>/hook/<session_id>.jsonl`, where
  `<state>` = `$MULTI_LINK_HOME`, else `$HOME/.multi-link`; fixture mode `<root>/.multi-link`), one JSON
  object per line, fields (draft, fixed by 2.1's findings): `v: 1`, `kind` (`handoff` | `waiting`),
  `at` (hook clock, RFC 3339 UTC ms), `session_id`, `agent_id`, `event` (source hook event),
  `tool_use_id`, `cwd`, `child`, `child_evidence` (`stdout` | `thread_ready` | `task_output` | null),
  `notification_type`. Reader rules: unknown `v` / `kind` → `skipped` `unknown_type`; bad line →
  `invalid_json`; hook files are listed in `skipped` like transcripts. State: the `waiting` / `hook` row
  (H8), after `finished` and before `busy`, with the clearing signal chosen in 2.1 (i) (if (i) needs extra
  clearing events, the snippet in 2.7 adds them; if (h) is no-go, `waiting` stays unemitted and this is
  recorded, not worked around). Visibility: a session whose latest hook record is an uncleared `waiting`
  is always listed, like a record session, so an overnight wait survives the 2 h window (Codex P2 #7).
- Synthetic hook fixtures in `fixtures/hook/` (not real payloads, referencing existing fixture ids):
  a `handoff` for the bg invocation (child = its known child, `task_output`); a `handoff` with `child: null`
  for one organic repo-c invocation (stays `unlinked`: null gives no evidence); a `handoff` whose child
  conflicts with a transcript-exact link; a `waiting` record for a transcript-only session older than 2 h
  after its `last_activity`; a `waiting` record before another session's `last_activity` (cleared); one
  malformed line and one `v: 2` line. "Hook variant": a temp copy of
  `fixtures/sessions` with `fixtures/hook/` copied to `.multi-link/hook/` and `task-output/be01y5dv6.output`
  removed. Expected `fixtures/expected/hook/{sessions,sessions-2h,links}.json` (`sessions-2h` = default
  window at the fixture `--now`); existing expected files unchanged.
- Files: `docs/output-schema.md`, `fixtures/hook/**`, `fixtures/expected/hook/*.json`,
  `fixtures/README.md` (hook fixture section)
- Verify: `jq . fixtures/expected/*.json fixtures/expected/hook/*.json > /dev/null` succeeds; in
  `hook/links.json` the bg link is `exact` / `hook` with the same child as in `links.json`, the repo-c
  null-child link is `unlinked`, the conflict link keeps its transcript evidence; `hook/sessions.json` and
  `hook/sessions-2h.json` each have exactly one `waiting` / `hook` session (the old one) plus the H11 `waiting` / `session_record` row; `git diff --stat` touches
  no existing expected file; Codex reviews (read-only).
- H11 (added after 2.1): also add a scrubbed live session record with `status: "waiting"` + `waitingFor`
  (from `fixtures/raw/hook/` snapshots) and its expected `sessions` row: `waiting` / `session_record`.

### Step 2.5 — Hook recorder library (Codex)
- Depends on: 2.3, 2.4. Parallel with: 2.6 (hook compiles model + claude, core compiles neither hook nor
  is compiled by it; neither edits a shared crate)
- `multi_link_hook::record(payload: &[u8], env: &HookEnv) -> Option<HookRecord>` and `write(&HookRecord,
  state_dir)`: `PostToolUse` + `Bash` + `multi_link_claude::recognise` → `handoff` (child from
  `tool_response` stdout `session id:` / `Thread ready (`, else for a background call by polling the task
  output file per 2.1 (g) every 250 ms up to 25 s, reading ≤ 64 KiB; two records if 2.1 (k) says so); `PermissionRequest` /
  `Notification` (`permission_prompt`, `elicitation_dialog`) → `waiting`; anything else → `None`, nothing
  written. One `write()` per record with `O_APPEND`, line < 4 KiB. Clock and sleep injected for tests.
  Its own `session id:` line parser (H9: keeps the hook crate off core so 2.5 ∥ 2.6).
- Files: `crates/hook/src/**`, `crates/hook/tests/hook.rs`
- Verify: `cargo test -p multi-link-hook` → `test result: ok`, including: each payload shape from 2.1
  (synthetic copies with field names from `docs/phase2-hook.md`, no real content) → the record expected by
  the schema; non-codex Bash, `codex --version`, unknown event, invalid JSON, empty stdin, 10 MiB stdin →
  `None`; `session_id` `../x` → nothing written; a background payload whose output file gains the
  `session id:` line after 3 fake seconds → that child; never gains it → `child: null` after the 30 s cap
  (fake clock); writing into a temp `HOME` with `.claude/` and `.codex/` dirs leaves both byte-identical
  (recursive listing + hashes before/after) and creates only `.multi-link/hook/<uuid>.jsonl`; state dir
  `<HOME>/.claude/x`, `<HOME>/.codex`, a relative path, `hook/` as a symlink to another temp dir, and the
  record file as a symlink → nothing written anywhere (target dirs unchanged); an unwritable state dir →
  `Err` returned, no panic; 8 threads appending 100 records each → 800 valid lines.

### Step 2.6 — Core reads hook records: hook evidence and waiting (Codex)
- Depends on: 2.3, 2.4. Parallel with: 2.5
- Reader for `<state>/hook/*.jsonl` per the schema (fail-soft, `skipped`); linker adds `hook` evidence
  after the transcript sources, matched on (`session_id` = parent, `tool_use_id` = invocation); a hook
  child that disagrees with transcript evidence is ignored and counted in a new `hook_conflicts`
  diagnostic; `child: null` gives no evidence. State: the `waiting` / `hook` row. `Options` gains the
  state dir (fixture mode: `<root>/.multi-link`).
- Files: `crates/core/src/{lib.rs,hook.rs,link.rs,links.rs,sessions.rs,state.rs}`,
  `crates/core/tests/{hook.rs,links.rs,sessions.rs}`, `crates/cli/src/main.rs` (only to fill the new
  `Options` field so the workspace builds)
- Verify: `cargo test -p multi-link-core` → `test result: ok`, including: Hook variant links equal
  `hook/links.json`, sessions (`--since all`) equal `hook/sessions.json`, sessions with the default 2 h
  window equal `hook/sessions-2h.json`; the plain fixture reports still equal
  `links.json` / `sessions.json` (no `.multi-link/` there); unit tests: waiting then a later transcript line
  → not waiting; waiting + process gone → `finished`; waiting on a Codex session id → ignored.
- H11 (added after 2.1): state reads `waiting` first from a live session record (`status: "waiting"`,
  evidence `session_record`), then from the hook (`hook`); test: record waiting + process alive → `waiting`;
  record waiting + process gone → `finished`.

### Step 2.7 — CLI `hook record | install | uninstall` (Codex)
- Depends on: 2.5, 2.6
- `multi-link hook record` starts the 30 s watchdog and the silent panic hook first, then reads stdin
  (≤ 1 MiB, rest discarded), calls the hook library, prints nothing, exits 0 on every path. A debug-build
  only env var (`MULTI_LINK_HOOK_TEST`, `panic` / `deadline_ms=<n>`) lets tests inject a panic and shorten
  the deadline; release builds ignore it. `hook install [--scope user|local]` prints the JSON snippet (the
  events above, `async: true`, `timeout` 60, the absolute path of the running binary) and the file to
  paste it into (`~/.claude/settings.json` or `<repo>/.claude/settings.local.json`); `hook uninstall`
  prints the same entries to remove and the state folder the owner may delete. Neither opens any file
  for writing. `sessions` / `links` print `hook_conflicts` on stderr like the other diagnostics.
- Files: `crates/cli/{Cargo.toml,src/**,tests/cli.rs}`, `Cargo.lock`
- Verify: `cargo test --workspace` → all `ok`, including: `hook install` / `uninstall` output parses as
  JSON with the four entries and `async: true`; run with a temp `HOME`, the temp dir is unchanged after
  both; `hook record` with each 2.5 payload → exit 0, empty stdout and stderr, expected file; malformed
  stdin and an unwritable state dir → exit 0, empty stdout and stderr; injected panic → exit 0, empty
  stderr; stdin held open and never written, deadline 500 ms → exits 0 within 2 s, empty output.
  Speed (release build):
  `time (for i in $(seq 50); do target/release/multi-link hook record < <non-codex Bash payload>; done)`
  under 1 s total (paste the `real` line). The three Phase 1 CLI diffs still print `same`.

### Step 2.8 — Real hand-off and waiting check in the sandbox (Claude)
- Depends on: 2.7
- Paste `hook install --scope local` output into `~/multi-link-sandbox/repo-a/.claude/settings.local.json`
  (binary: absolute `target/release/multi-link`, `MULTI_LINK_HOME` set to a scratch dir in the command).
  `touch` a marker, run three `claude -p` hand-offs (foreground exec, background exec, exec to
  `/dev/null`), each tagged `Step 9.1`; then the `expect`-driven permission prompt if 2.1 (j) is go.
  After the hook has recorded the background child, delete that call's task output file (a sandbox temp
  file the sandbox run created), so only the hook can make that link exact (Codex P2 #9).
- Results in `docs/phase2-hook.md` → "Phase 2 Verify" (ids compared, not pasted in full).
- Files: `docs/phase2-hook.md`
- Verify: `find ~/.codex/sessions -newer <marker> -name 'rollout-*.jsonl'` gives the new rollouts; for the
  foreground and background hand-offs, `session_meta.payload.id` of the matching rollout equals the hook
  record's `child` (printed `equal` per pair); `multi-link links --json` shows the foreground one `exact` /
  `stdout` and the background one `exact` / `hook`, both with `step: "9.1"`; the `/dev/null` hand-off is
  `heuristic` or `unlinked` with `evidence: null`; with the expect session at the prompt,
  `multi-link sessions --json` shows that session `waiting` / `hook`, and after the answer it doesn't.
  Phase 2 closes only when all of this passes. If 2.1 (j) was no-go, the waiting part is a stop-and-ask:
  the owner runs it once interactively and the phase stays open until then.
- Remove the sandbox hook afterwards (`ls …/settings.local.json` fails).

### Phase 3 — TUI home + timeline + open (Release 1)
`multi-link tui`: the home screen (mockup screen 6) and the repo timeline (screen 2), needs-you sorting,
the bell, and FR18 open (resume / app link). Decisions T1–T11: `docs/design.md` → "Phase 3 decisions"
(written in Step 3.2; summary here so the steps read on their own).
- **Measured before planning (2026-10-30, warm, release build, real `~`: 165 Claude + 174 Codex JSONL,
  378 MB + 74 MB):** `sessions --json` 16.4–16.9 s real (10.4 s user, 4.5 s sys); with `--root ~` (no
  `ps`, no git) 6.0 s. So ~10 s is one `git rev-parse` per session (run before the history window filters
  them out) plus one `ps` per record, and ~6 s is re-parsing every file. The 2 s target needs both fixed.
- **T1 Speed first, offsets only if needed.** A `Scanner` in core keeps each file's parsed result keyed by
  (path, length, mtime, inode) and re-parses only files whose key changed; git lookups are memoised per
  `cwd` and run only for listed sessions; one `ps` call covers every record pid. Why: between two refreshes
  only a few files change, and the largest is 44 MB, so whole-file re-parse of changed files is cheap
  enough to try; byte-offset tailing needs every parser rewritten as a resumable fold. If 3.1's warm
  refresh misses 2 s, offset tailing becomes its own step before 3.7. This revises the NFR "Incremental
  reads" wording (3.2 updates design.md).
- **T2 `overview` lives in core, not the TUI.** Grouping by repo, needs-you sort, Codex-under-parent
  nesting, the "claude ×N" merge threshold, FR17 concurrency warnings and the latest hand-off are computed
  by `core::overview` into an `OverviewReport` (types in `model`), printed by `multi-link overview --json`
  and answer-keyed like the other reports. Why: "front ends contain no logic"; the desktop app reuses it;
  a JSON answer key is reviewable where a screen snapshot is not.
- **T3 TUI stack:** `ratatui` + `crossterm`, new crate `crates/tui` (lib) depending on `model` only.
  Why: the standard Rust TUI pair, `ratatui::backend::TestBackend` gives text snapshots without a
  terminal, crossterm needs no ncurses. Depending on `model` only lets TUI steps run in parallel with core
  steps (Phase 1 parallel rule).
- **T4 Snapshots are reviewed goldens, not hand-written keys.** Render functions are pure
  (`&OverviewReport`, `&LinksReport`, size → `Buffer`); tests compare the buffer text with
  `fixtures/expected/tui/*.txt`. Codex generates the golden once (`UPDATE_SNAPSHOTS=1`), Claude reviews it
  line by line against `overview.json` before commit; any later diff fails the test. Why: hand-typing a
  100×30 screen is error-prone; the semantic truth is the overview answer key.
- **T5 Needs-you order:** repos with a `waiting` session first, then `busy`, then the rest; inside a
  tier, latest activity first; ties by repo path. Sessions inside a repo: waiting, busy, stale, idle,
  unknown, finished. Why: the screen's job is "which one needs me"; activity order inside a tier keeps
  the hottest repo on top.
- **T6 Bell:** one `\a` when a session id enters `waiting` between two refreshes (not on the first
  refresh, not every refresh while it waits). Terminal bell only; macOS notifications are desktop backlog
  (FR14/FR15). Why: a bell per refresh trains me to ignore it.
- **T7 Open (FR18) is a pure decision in core:** `open::action(&Session, fork) -> OpenAction`
  (`Resume{program, args, cwd}` | `AppLink{url}` | `Unavailable{reason}`); the TUI only executes it.
  Rules: `Resume` only with positive exit evidence, i.e. Claude `finished` (record + process gone) →
  `claude --resume <id>` (`--fork-session` with `f`) in its `cwd`. Claude running (`busy`, `idle`,
  `waiting`, `stale`) → `AppLink` if the record has `hostSessionId`, else `Unavailable`. Everything else
  (Claude `unknown`, every Codex session) → `Unavailable` carrying the command as text
  (`claude --resume <id>` / `codex resume <id>`) for me to run myself, because `unknown` can be a live
  process (a completed Codex turn, a `-p` session without a record). A probe that cannot run makes no
  session `finished`, so nothing resumes; records then follow their status (3.2 review #2). `codex://` links stay out (untested).
  The action is recomputed from a fresh pass for that one full id right before it runs, never taken from
  the last drawn report. Why: "never resume a running session" must hold without a liveness signal and be
  testable without a terminal; Session gains `host_session_id` (from the record) for the app link.
- **T8 Session gains `checkout`** (git toplevel of `cwd`; fixture mode = `cwd`), for worktree labels
  (Codex 0.6 #1). Added to `sessions --json` too, so the schema stays one Session type.
- **T9 Refresh:** every 2 s on a background thread that owns the `Scanner`; the UI thread never blocks on
  disk. Before running an external program the TUI disables raw mode, leaves the alternate screen, pauses
  its input reader, waits for the child, then restores all three on success, spawn failure and panic. A failing pass keeps the last good report and shows the error in the footer; hook dir read errors
  become diagnostics instead of failing the report (Phase 2 nit). Why: a dashboard that dies on one bad
  file is worse than a stale one.
- **T10 Deferred:** background hand-offs whose hook child is `null` (clean-env `-p`, no `scratchpad_dir`)
  stay exact through the task output file (Exact evidence 4) while it exists and fall back to heuristic
  after (P2), unchanged. Teaching the hook to read the "Output is being written to" path → backlog. Why:
  it changes link evidence, not the TUI; the fallback is already honest.
- **T11 Overview layout (Codex P2 #7):** a Codex session sits under the parent of its latest hand-off
  (by `at`), listed once, with every hand-off kept in its `handoffs` list; children are ordered by that
  hand-off time; a parent's place follows its own state, and a repo's tier is the best tier of any of its
  sessions. A merged `claude ×N` row is not actionable: `enter` expands it first, so `o` / `f` always target
  one full session id. Codex sessions with no parent are top-level rows.
- Parallel rules from Phase 1 apply. Only steps that say so change a `Cargo.toml` / `Cargo.lock`. Parallel
  steps run crate-local checks (`cargo test -p …`, `cargo clippy -p …`) and return their PROGRESS entry text;
  the coordinator runs the workspace gate once both are in and writes both entries (Codex P2 #9).
- Verify (whole phase, Step 3.8 runs it): `cargo test --workspace` passes, including the home and
  timeline snapshot tests on fixtures and the open-action tests (no running fixture session ever gets
  `Resume`); the `overview` answer key matches; warm home refresh on real `~` ≤ 2000 ms (pasted
  timings); `multi-link tui` runs against real `~` in a pty and quits cleanly with `q`.

### Step 3.1 — Refresh speed: scanner cache, memoised git, one `ps` (Codex)
- Depends on: Phase 2. Parallel with: none (3.2 edits answer keys this step's tests read).
- `core::Scanner` (new) owns per-file caches for Claude transcripts + subagents, session records, Codex
  rollouts, hook files and task output files, keyed by (path, len, mtime, inode); `Scanner::sessions` /
  `links` take the same `Options` and return the same outputs as `sessions_report` / `links_report`, which
  become thin wrappers over a fresh `Scanner` (CLI unchanged). Expose per-file parse functions from the
  `claude` / `codex` crates as needed (no behaviour change). Repo mapping: memoised per `cwd`, applied
  after the history window. Hook dir read errors become diagnostics, not a failed report (T9).
  `ProcessProbe` gains a batch method; `RealProcessProbe` runs one
  `ps -o pid=,lstart= -p <pid,pid,…>` (same UTC and fail-soft rules as 1.10; per-pid fallback is not
  allowed to spawn more than one process per pass). Example `crates/core/examples/refresh_timing.rs`:
  real `~`, N passes on one `Scanner`, each pass doing the full refresh that exists so far (sessions +
  links; 3.3 adds overview), prints `pass <i>: <ms> ms, <n> sessions, <k> files re-parsed, <b> bytes
  re-parsed`. Passes 2+ run while live sessions are writing, so `k ≥ 1` shows changed files are measured.
- Files: `crates/core/src/**`, `crates/core/examples/refresh_timing.rs`, `crates/claude/src/lib.rs`,
  `crates/codex/src/lib.rs`, their tests.
- Verify: `cargo test --workspace` → all `ok`, the Phase 1 and hook CLI diffs still `same`; new unit
  tests: an appended / truncated / replaced (new inode) file is re-parsed, an unchanged one is not (parse
  counter); a fake batch probe is called once per pass. Then
  `cargo run --release -p multi-link-core --example refresh_timing -- 3` on real `~` → paste all three
  lines; pass 2 and 3 ≤ 2000 ms with `k ≥ 1`. A miss is not a fail of this step if pass 1 improved: log
  it and Claude plans offset tailing (T1) as a step that 3.7 then depends on.

### Step 3.2 — Output contract: checkout, host id, overview, open rules (Claude)
- Depends on: 3.1. Parallel with: none.
- `docs/design.md`: "Phase 3 decisions" (T1–T11), NFR "Incremental reads" revised per T1, FR18 table
  aligned with T7. `docs/output-schema.md`: Session `checkout`, `host_session_id`; new `overview` section
  (repo groups, tiers, nesting, `claude ×N` threshold 4, FR17 warnings = 2+ `busy` sessions with the same
  `cwd`, latest hand-off, per repo the latest logged step + its Verify and the step tags of active
  hand-offs, layout per T11); fixture-mode plan lookup (a session `repo` like `/home/dev/repo-a` reads
  `<root>/repos/<basename>/PLAN.md`, while `repo` in the output keeps the recorded string; Codex P2 #5);
  new "Open" section (T7 table, including the `Unavailable` command text). Answer keys: `sessions.json` and `hook/sessions*.json` gain
  the two fields; new `overview.json` and `hook/overview.json` (the hook variant has a waiting session,
  so needs-you sort is tested; the existing child linked from two parents (exec + resume) tests T11's
  placement; at least one repo shows a real latest step + Verify). Fixture additions only if a case is missing (concurrency warning,
  worktree-shaped `cwd`), scrubbed per `fixtures/README.md`. Leaves `cargo test` red until 3.3 (as 2.2 did).
- Files: `docs/design.md`, `docs/output-schema.md`, `fixtures/expected/**`, `fixtures/sessions/**` (only
  additions), `fixtures/README.md`
- Verify: `python3 -m json.tool` parses every new/changed expected file; a short script checks every
  Session in the changed keys has `checkout` and `host_session_id` keys and each `overview` key's session
  ids equal its own sessions key's ids, each once (`overview.json` ↔ `sessions.json`, hook ↔ hook); adversarial Claude review of the contract (Codex-off mode) logged in the Review log.

### Step 3.3 — Core: new Session fields, overview report, CLI `overview` (Codex)
- Depends on: 3.2. Parallel with: none.
- Model: `Session.checkout`, `Session.host_session_id`, `OverviewReport` types. Core: fill the fields,
  `overview` per the schema (built from one `Scanner` pass + steps report), `Scanner::overview`. CLI:
  `multi-link overview [--root] [--now] [--since] [--json]` (text mode: one line per repo).
- `claude::SessionRecord` keeps `hostSessionId` (today `read_session_record` drops it; Codex P2 #3);
  tests for an absent id and an id different from the transcript id. `refresh_timing` adds overview.
- Files: `crates/model/src/**`, `crates/claude/src/lib.rs` + its tests, `crates/core/src/**`,
  `crates/core/examples/refresh_timing.rs`, `crates/plan/src/**` (only if the steps report needs a
  getter), `crates/cli/src/main.rs`, `crates/cli/tests/cli.rs`, tests.
- Verify: `cargo test --workspace` → all `ok`; `multi-link overview --root fixtures/sessions --now
  2026-10-29T23:15:00Z --since all --json` equals `fixtures/expected/overview.json` (parsed), the hook
  variant equals `hook/overview.json`; the sessions / links / steps diffs print `same`.

### Step 3.4 — Core open actions + CLI `open --dry-run` (Codex)
- Depends on: 3.3. Parallel with: 3.5, 3.6.
- `core::open::action` per output-schema "Open". CLI `multi-link open <id-prefix> [--fork] [--dry-run]`
  with `--root/--now`: prints the action as JSON; without `--dry-run` it executes it (spawn with inherited
  stdio in `cwd`, or `open <url>`); `Unavailable` → reason on stderr, exit 3. Ambiguous prefix → exit 2.
- Files: `crates/core/src/open.rs`, `crates/core/src/lib.rs`, `crates/cli/src/main.rs`,
  `crates/cli/tests/cli.rs`
- Verify (crate-local while 3.5 runs): `cargo test -p multi-link-core -p multi-link` → all `ok`,
  including a table test over every fixture session × {open, fork} asserting `Resume` appears only for
  `finished`; core unit tests with `FakeProcessProbe` for: record + transcript + `Gone` → `Resume`; record-only + `Gone` →
  `Unavailable` `no_transcript`; two records for one id, one `Gone` and one `Alive` → not `Resume`; record + `Alive`
  with/without `hostSessionId` → `AppLink` / `Unavailable`; probe `Unavailable` (record follows its status, never `Resume`), transcript-only Claude,
  completed-turn Codex → `Unavailable` with command text; an id older than the `--since` window is still
  found; no match → exit 4; and `--dry-run` CLI output for an app-linked
  running Claude, a transcript-only Claude and a Codex session (fixture mode can't produce `finished`).

### Step 3.5 — TUI crate: home screen render (Codex)
- Depends on: 3.3. Parallel with: 3.4.
- New crate `crates/tui` (`multi-link-tui`, lib, deps: model, ratatui, crossterm); workspace deps added.
  `render_home(&OverviewReport, selection, area, buf)`: per repo header (◆ when waiting, counts), steps
  line, one mini lane per session (harness, id prefix, state, checkout label when ≠ repo, step tag, age
  from `now`), Codex lanes indented under the parent, `claude ×N` for 4 or more quiet rows, FR17 warning, latest hand-off,
  footer with keys. Snapshot test at 100×30 from `overview.json` and `hook/overview.json` (T4).
- Files: `Cargo.toml`, `Cargo.lock`, `crates/tui/**`, `fixtures/expected/tui/home*.txt`
- Verify (crate-local while 3.4 runs): `cargo test -p multi-link-tui` → `ok`, including renders at 40×10
  and 20×5 that don't panic; goldens reviewed by Claude against the overview keys (Review line names each
  checked row); `cargo clippy -p multi-link-tui -- -D warnings` clean.

### Step 3.6 — TUI: repo timeline render (Codex)
- Depends on: 3.5. Parallel with: 3.4.
- `render_timeline(&OverviewReport, &LinksReport, repo, window, area, buf)`: time axis over the window
  (default 2 h ending at `now`), one row per session (Codex rows indented under their parent per T11), a
  **session span** `started_at`..`last_activity` labelled as such (it includes idle gaps; busy intervals are
  not in the model, so no busy swimlanes; Codex P2 #10), hand-off markers at `at`: exact `●`, heuristic `○`, ambiguous `?`
  with candidate count, unlinked Codex rows top-level; legend line.
- Files: `crates/tui/src/**`, `crates/tui/tests/**`, `fixtures/expected/tui/timeline*.txt`
- Verify: `cargo test -p multi-link-tui` → `ok` with a timeline snapshot for the ambiguous repo and the
  exact-link repo, plus tests for a span clipped by the window, a link outside the window (not drawn) and a
  zero-length span; goldens reviewed as in 3.5.

### Step 3.6a — `ps` runs in the C locale (Claude implementer)
- Depends on: 3.4. Parallel with: 3.6 (core only; the tui crate depends on model only).
- Found in the 3.4 review (P1): `RealProcessProbe` sets only `TZ=UTC`, but `ps -o lstart=` prints month and day
  names in the user's locale (`LC_ALL=de_DE.UTF-8` → `Fr 30 Okt …`), while `procStart` is English. Under a
  non-English locale a live session's start time never matches → `Gone` → `finished` → `open` offers `Resume`
  on a running session. Fix: run both `ps` calls (single and batch) with `LC_ALL=C` (and drop any inherited
  `LANG` / `LC_*` that could override it). Why `C`: it is the locale `procStart`'s format matches, and it is
  always present.
- Files: `crates/core/src/probe.rs`, `crates/core/tests/repo_probe.rs`
- Verify: `cargo test -p multi-link-core` → all `ok`, including a test that runs the real probe on the test
  process's own pid with `LC_ALL=de_DE.UTF-8` and `LANG=de_DE.UTF-8` set in the environment and still gets
  `Alive` against its English start time (skipped with a printed note if `ps` is unavailable in the sandbox), and a
  unit test that the built `Command` carries `LC_ALL=C` and `TZ=UTC`; `cargo clippy -p multi-link-core -- -D warnings` clean.

### Step 3.7 — `multi-link tui`: loop, refresh, keys, bell, open (Codex)
- Depends on: 3.4, 3.5, 3.6, 3.6a. Parallel with: none.
- CLI subcommand `tui [--root] [--now] [--since]` (cli depends on tui). Refresh thread per T9; pure
  `App` state machine (`update(event) -> Vec<Effect>`) for keys: `↑/↓` select, `enter`/`t` timeline,
  `esc` back, `o` open, `f` fork, `q` quit; bell per T6 as an `Effect`; open recomputes the action from a
  fresh pass for the selected full id (T7), then suspends and restores the terminal per T9 (a guard type
  whose `Drop` restores); `Unavailable` shows its reason and command text in the footer. Raw mode is
  restored on panic and on every exit.
- Files: `crates/cli/{Cargo.toml,src/**,tests/**}`, `crates/tui/src/**`, `Cargo.lock`
- Verify: `cargo test --workspace` → all `ok`, including `App` tests: bell once on idle→waiting, not on the
  first report or on waiting→waiting; `o` on a running fixture session yields no `Resume` effect; `o` on a
  session that was `finished` in the drawn report but is running in the fresh pass yields no `Resume`; a
  failed refresh keeps the last report and never feeds `o`; `cargo clippy --workspace -- -D warnings` clean.

### Step 3.8 — Real check, README, Release 1 close (Claude)
- Depends on: 3.7.
- On real `~`: `refresh_timing -- 3` (now including overview) → pass 2–3 ≤ 2000 ms; run `multi-link tui`
  under `script`/`expect` in a pty for ~5 s, capture the screen text (repo names only, no conversation
  content), `q` exits 0 and the terminal is restored (`stty -a` unchanged); in the pty, `o` on a session
  whose action runs a stub program (`PATH` with a fake `claude` that prints and exits, or is missing) comes
  back to a working TUI in both cases; resize the pty once. In the sandbox, end an interactive `claude`
  session and record what its record does on exit (kept → `open --dry-run` prints `Resume`; deleted →
  `unknown` / `Unavailable`, logged honestly as a Release 1 limitation). README: what Release 1 does, how to run it, limitations
  (undocumented formats, Codex waiting unsupported, app link undocumented), numbers only from these
  outputs.
- Files: `README.md`, `docs/phase3-check.md`
- Verify: the timings and checks above pasted in `docs/phase3-check.md`; whole-phase Verify passes.

### Backlog (ordered, uncommitted)
Hook reads the background task output path (T10) · Lost key on resize: crossterm 0.29 drops tty readiness when SIGWINCH and a key arrive in one wake (upgrade or workaround; 3.8) · Resume for cleanly exited terminal sessions (their record is deleted on exit, so they show `unknown`; 3.8) · `@basename` label for a worktree named like its repo · step graph · live grid · tmux / Herdr jump · desktop app with notifications · Claude Code mod.

## Review log

### 2026-10-30 — Step 3.3 review (Claude, adversarial Claude review, Codex-off mode)
A fresh subagent saw only the diff, Step 3.3 and the schema's Session + overview sections. It re-ran Verify
(all `same`) and found 0 P1 / 0 P2. Verdict: approve. One fix round.
| # | Finding | Decision |
| --- | --- | --- |
| 1 | P3: fixture plan folder of `/`, `""` or `..` escapes `<root>/repos` | Accepted: `plan_folder` → `None`, plan `missing`, no lookup; unit test |
| 2 | P3: two listed sessions with one id both get the placed child (breaks "exactly once") | Accepted: the child attaches to the first matching parent; unit test |
| 3 | P3: CLI plan lookup uses `to_string_lossy` (non-UTF-8 repo paths) | Rejected for 3.3: the plan crate takes strings everywhere; backlog nit |
| 4 | P3: `refresh_timing` stubbed the plan parse (core doesn't depend on plan) | Accepted: planner widened 3.3's files to `crates/core/Cargo.toml` + `Cargo.lock` for a dev-dependency only; the example uses the real parser. The injected lookup stays (core runtime deps unchanged) |
| 5 | P3: placement also requires a Claude parent (stricter than the contract) | Accepted as is, with a comment: link parents are always Claude |
| 6 | P3: test gaps (odd basename, exactly-once) | Covered by the tests for #1 and #2 |

### 2026-10-30 — Step 3.2 contract (Claude, adversarial Claude review, Codex-off mode)
A fresh general-purpose subagent saw only the diff, the Phase 3 header, Steps 3.2–3.4 and the fixtures.
Round 1: answer keys all correct by hand-check; 10 findings on the contract. Round 2 (fixes only): 2 P3
wording fixes, both accepted; verdict "clean, 3.3 can start". No stop-and-ask.
| # | Finding | Decision |
| --- | --- | --- |
| 1 | P1: several records per `sessionId` (a crashed one plus a live resume) could make a running session `finished` → resumed twice | Accepted: `finished` only when every matching record is gone; otherwise state from the live record with the greatest `updatedAt`; 3.4 test. Rejected the extra "transcript ≤ record `updatedAt` + 5 s" guard: a Session carries no record `updatedAt` for a transcript session; the record-less `-p --resume` case is written down as a known limit |
| 2 | P2: T7 / 3.4 said "probe unavailable → Unavailable", but `action(&Session)` can't see the probe and fixture keys rely on `app_link` | Accepted: reworded T7 and 3.4: a probe that cannot run makes nothing `finished`, records follow their status |
| 3 | P2: 3.4 "record + Gone → Resume" contradicts `no_transcript` | Accepted: 3.4 now says record + transcript + Gone → Resume; record-only → `no_transcript` |
| 4 | P2: merge decided by the parent's state could hide a busy Codex child in `claude ×N` | Accepted: quiet needs every child `idle` / `unknown` / `finished`; keys unchanged (all repo-b children `unknown`) |
| 5 | P2: 3.5 said "over 4", schema "4 or more"; no fixture on the boundary | Accepted: 3.5 reworded; 3.3 unit tests at exactly 3 and 4 |
| 6 | P2: open lookup unspecified (window, no match, TUI miss) | Accepted: window off, no match → exit 4, TUI footer; added to 3.4 Verify (round 2) |
| 7 | P2: resume is crash-only if Claude deletes records on clean exit; consider fork on `unknown` | Accepted the limit sentence (3.8 checks the real behaviour). Rejected fork on `unknown`: the original may be live and mid-write; command text is offered |
| 8 | P3: 3.2 Verify text "every overview id in sessions.json" fails for the hook key | Accepted: each overview key pairs with its own sessions key, id sets equal |
| 9 | P3: fixture plan lookup by basename can collide | Accepted: allowed, same plan, 3.3 unit test |
| 10 | P3: stale / finished row place, busy child, equal-activity tie unpinned | Accepted: "Required 3.3 unit tests" paragraph in the schema |
| R2-1 | P3: known-limit sentence named the wrong condition | Accepted: "old record remains but its process has exited" |
| R2-2 | P3: 3.4 Verify didn't test window-off lookup or exit 4 | Accepted: added |

### 2026-10-30 — Phase 3 plan (Claude, Codex critique, read-only)
Planned from a real-data measurement (Phase 3 header): `sessions --json` 16.4–16.9 s warm, ~10 s of it
per-session `git` / `ps` spawns. Decisions T1–T11. No stop-and-ask: nothing changes the goal or needs the
owner. Codex verdict: "Revise before implementation"; 2 P1, 8 P2.
| # | Finding | Decision |
| --- | --- | --- |
| 1 | P1: `unknown` (completed Codex turn, record-less Claude) can be a live process; a y/n confirm doesn't make resume safe | Accepted: `Resume` only for `finished` (positive exit evidence); `unknown`, all Codex and an unavailable probe → `Unavailable` with the command as text. Rejected cutting open from Release 1: FR18 is in the Release 1 Verify, and with this rule the risk is bounded |
| 2 | P1: open could act on a stale report (resumed elsewhere since, or a failed refresh kept the old one) | Accepted: T7 recomputes the action from a fresh pass for the full id right before running; 3.7 tests finished→running and failed refresh |
| 3 | P2: `read_session_record` drops `hostSessionId`; 3.3's file list can't add it | Accepted: `crates/claude/src/lib.rs` added to 3.3 with absent / different-id tests |
| 4 | P2: fixture mode can't produce `finished`, so the dry-run `Resume` check had no input | Accepted: `Resume` tested in core with `FakeProcessProbe`; CLI dry-run checks only the non-resume cases; 3.8 records honestly what a real record does on exit |
| 5 | P2: fixture `repo` is `/home/dev/repo-a`, plans live under `fixtures/sessions/repos/`; overview would find no plan | Accepted: 3.2 specifies the fixture-mode lookup `<root>/repos/<basename>` and requires a real latest step + Verify in the key |
| 6 | P2: unchanged rescans don't measure refresh; 3.8 couldn't edit the example | Accepted: the example reports files/bytes re-parsed and needs `k ≥ 1`; 3.3 extends it with overview; 3.7 depends on offset tailing if 3.1 misses |
| 7 | P2: one Codex child has two parents (exec + resume); nesting and `claude ×N` selection undefined | Accepted as T11: under its latest hand-off's parent, all hand-offs kept, merged rows expand before `o` / `f` |
| 8 | P2: suspending for an external program needs raw mode off, input paused, restore on every path | Accepted: T9 sequence + `Drop` guard in 3.7; 3.8 pty check with a stub and a missing program, plus a resize; small-size renders in 3.5 |
| 9 | P2: 3.4 ∥ 3.5 workspace tests / clippy read each other's unfinished work | Accepted: parallel steps run crate-local checks; the coordinator runs the workspace gate and writes both PROGRESS entries |
| 10 | P2: `started_at..last_activity` is not busy time | Accepted: drawn and labelled as a session span; clip / out-of-window / zero-length tests in 3.6; no busy swimlanes |

### 2026-10-30 — Phase 2 close (Step 2.8 results, coordinator)
| # | Finding | Decision |
| --- | --- | --- |
| 1 | The `/dev/null` hand-off came out `ambiguous`, not "heuristic or unlinked" as the Verify words it | Accepted: the Verify's intent is "never `exact` without an observed id", which held. Read the Phase 2 / 2.8 wording as "not `exact`" |
| 2 | The held prompt showed `waiting` / `session_record`, not `waiting` / `hook` | Accepted: H11 (record first) postdates the Verify text; the hook's `waiting` records agreed. Read the wording as "`waiting`, from the record or the hook" |
| 3 | A clean-environment `claude -p` sends no `scratchpad_dir`, so background hand-offs get only the `child: null` hook record and fall back to heuristic/ambiguous once the temp output file is gone | Open: decide in Phase 3 planning (candidate fallback: read the "Output is being written to" path from the transcript `tool_result`). Sample is 1 clean + 1 inherited run; the controlling env var was not isolated |

Phase 2 is closed. Next: plan Phase 3 (TUI, Release 1).

### 2026-10-30 — Step 2.1 finding: session records show waiting (H11)
| # | Finding | Decision |
| --- | --- | --- |
| 1 | During a held permission prompt, `~/.claude/sessions/<pid>.json` showed `status: "waiting"` + `waitingFor`, back to `idle` after the answer. Phase 0 P3 said no record field signals waiting (only 5 records sampled, none waiting) | Accepted as H11: record first, hook as fallback. Folded into 2.4 (fixture + expected row) and 2.6 (state). Why: no hook install needed for needs-me; the hook still covers `-p` sessions and missing records |
| 2 | Failed commands fire `PostToolUseFailure` with the Codex id in `error` | Accepted: already flagged for 2.5 in PROGRESS "Current state"; 2.7's snippet must register it |
| 3 | `-p` kills unfinished async hooks at teardown | Accepted: 2.5 writes the hand-off record first, then appends the child |

### 2026-10-29 — Phase 2 plan (Claude, Codex critique, read-only)
Hook facts were checked against the Claude Code hooks docs (claude-code-guide agent, 2026-10-29): 33 events
exist, including `PermissionRequest`; `PostToolUse` carries `tool_use_id`; `async: true` hooks can't decide
and have no enforced timeout; `-p` runs project settings hooks and kills unfinished async hooks at
teardown; background `tool_response` and Notification firing rules are undocumented (hence Step 2.1).
Decisions H1–H10 in `docs/design.md`. No stop-and-ask: nothing needs a write to `~/.claude/` (the sandbox
uses project-scope settings). Codex verdict: "Revise before implementation"; 3 P1, 6 P2, 1 P3.
| # | Finding | Decision |
| --- | --- | --- |
| 1 | P1: a UUID check doesn't contain writes; `$MULTI_LINK_HOME` or a symlink could point into `~/.claude` / `~/.codex` | Accepted: absolute state dir, refuse if inside `~/.claude` / `~/.codex`, no symlink following (`symlink_metadata` + `O_NOFOLLOW`); five refusal tests in 2.5 |
| 2 | P1: the spike's `cat >>` stores whole payloads (commands, outputs) | Accepted: the capture hook keeps an allowlist (ids, times, paths, enums, found uuids) and reduces every other value to type + length; 2.1 Verify checks the stored value shapes. Sandbox prompts are placeholders anyway, but the rule should hold for the spike too |
| 3 | P1: no process-wide deadline (stalled stdin, blocked write) and `catch_unwind` alone still prints the panic message | Accepted: watchdog thread `exit(0)` at 30 s started first, no-op panic hook; 2.7 tests an injected panic and held-open stdin via a debug-only env var |
| 4 | P2: 2.2 adopts the waiting rule before the spike, so it can't run parallel with 2.1 | Partly accepted: the `waiting` row moved to 2.4 (after 2.1). Rejected serialising the rest: step tags and `hook` link evidence don't depend on any payload field, so 2.2 ∥ 2.1 stays |
| 5 | P2: the model test rejects unknown expected filenames, so 2.4's new files turn 2.3 red; 2.4 can't edit that test | Accepted, fixed differently: 2.4's files go in `fixtures/expected/hook/` (the test's scan is non-recursive), keeping 2.3 ∥ 2.4; 2.3 adds `links-tagged.json` to the test. Also made explicit that 2.2's answer-key change leaves Phase 1 tests red until 2.3 |
| 6 | P2: amb-1 keeps its transcript-exact child in the Hook variant, so "not exact" was wrong | Accepted: the null-child fixture now targets an organic repo-c invocation (stays `unlinked`) |
| 7 | P2: a transcript-only session waiting overnight drops out of the 2 h window | Accepted: an uncleared `waiting` session is always listed; `hook/sessions-2h.json` tests the default window |
| 8 | P2: `-p` kills unfinished async hooks, so background polling may never write | Accepted: spike question (k); if killed, 2.5 writes the hand-off first and appends the child later |
| 9 | P2: the real Verify can pass on `task_output` without exercising the hook reader; phase could close without waiting | Accepted: 2.8 deletes the sandbox task output file after the hook records, then requires `exact` / `hook`; Phase 2 stays open until the waiting check passes (owner-run if `expect` can't do it) |
| 10 | P3: 2.3 bundles five crates (model, tags, invocation, linker, hook skeleton) | Rejected: every piece is a few lines implementing one contract change from 2.2 under one Verify; the skeleton (2 files) sits there so 2.5 ∥ 2.6 never edit a manifest in parallel; a separate step would cost a full session round for no review benefit |

### 2026-10-29 — Phase 0: design revision and Phase 1 split (Step 0.6, Codex critique, read-only)
Phase 0 decisions: (a) go, (b) no-go for waiting only, (c) no-go, (d) go; none changes the goal or the
differentiator. Design decisions P1–P12 in `docs/design.md`. Codex verdict: "broadly sized for individual
runs and commits", needs contract and Verify fixes. All 10 were P2.
| # | Finding | Decision |
| --- | --- | --- |
| 1 | Session has no `checkout` field, so a worktree session in a subfolder can't be identified | Partly accepted: Step 1.5 now tests nested subfolders; the output field is deferred to Phase 3 (noted there). Phase 1 keeps `cwd`, which is what the linker's exact-checkout match uses; only the TUI's worktree labels need the toplevel, and adding it now would rewrite all 27 expected sessions for no Phase 1 consumer |
| 2 | `cd d && codex exec` makes the heuristic match against the wrong directory | Accepted: a `cd` before the launch or `-C` / `--cd` → no heuristic, `unlinked` (schema, design, Step 1.8 test). Conservative: never guess. No fixture has either, so expected files are unchanged (checked) |
| 3 | Probe "can't tell" doesn't separate gone from unavailable | Accepted: `Alive` / `Gone` / `Unavailable`, with real-adapter tests for an exited child and a missing `ps` (Step 1.5) |
| 4 | Codex state uncovered when an earlier turn is open but the last one completed | Accepted: the latest turn governs; added a `no_signal` row for files with no `task_started`; Step 1.7 test. No fixture has either case (all 9 rollouts have matched turns) |
| 5 | Real-data smoke `jq` expression fails (`.sessions|length, (.skipped|length)` binds wrong) | Accepted: `(.sessions|length), (.skipped|length)` on a saved temp file, so the CLI's exit status is checked too |
| 6 | No test proves a record and a transcript with the same id merge | Accepted: merge test in Step 1.7 (fixtures have no such pair) |
| 7 | `--since` link visibility can pass untested because Verify uses `--since all` | Accepted: four visibility tests in Step 1.8 |
| 8 | Parallel steps still share `PROGRESS.md` (and the git index) | Accepted: in parallel steps Codex returns the entry text, Claude prepends it at commit time and commits with `git commit -- <paths>`, so another session's staged files can't ride along |
| 9 | Missing-task-output test leaves the answer to the implementer | Accepted: concrete expected result (heuristic, child `01a189e3-f419…`, gap 474 ms). Checked: call 22:59:39.748Z, child start 22:59:40.222Z; the amb children 57 s later in repo-b are excluded as exact children |
| 10 | Design excludes any exact-linked child from candidates; schema excludes only non-resume | Accepted: design now says non-resume (schema was right); regression test in Step 1.8 |
Not raised by Codex, decided by Claude in this step: step ids may carry one letter suffix (P11, schema
`id` row updated); incremental tailing moved to Phase 3 (Phase 1 reads whole files); Phase 2 must add a
`waiting` step via the hook, Phase 3 needs-you uses only that signal.

### 2026-10-29 — Step 0.4 review (Claude)
| # | Finding | Decision |
| --- | --- | --- |
| 1 | Verify pattern `sk-` matches ordinary `task-*` values (`task-notification`, `task-resume-candidate`), so the scrub had to corrupt real data to pass | Changed to `\bsk-`: still catches `sk-` keys, no longer matches inside words |

### 2026-10-29 — Claude roadmap read-through (design doc vs PLAN.md)
| # | Finding | Decision |
| --- | --- | --- |
| 1 | Design's supported convention (`## Step N: title`) doesn't match the owner's real PLAN.md (`### Step <id> — <title>`, dotted ids); multi-link couldn't read its own plan | Accepted: Phase 1 now names the real convention. Sampled only 2 repos with a PLAN.md (route-impact, multi-link) |
| 2 | Design's `hook install` writes `~/.claude/settings.json`, contradicting AGENTS.md "never write to `~/.claude/`" | Accepted: install prints a snippet for the owner to paste; keeps the read-only rule absolute and the install visible and reversible. Rejected an opt-in `--write` flag: one exception makes the rule harder to trust |
| 3 | Step 0.2 omitted the ambiguous fixture (two hand-offs seconds apart, one checkout) that Phase 1's Verify needs | Accepted: added to Step 0.2 |
Design doc (claude.ai) still has the old convention and install wording; update it in Step 0.6.

### 2026-10-29 — Gaps for autonomous running (owner asked for an unattended loop)
| # | Gap | Decision |
| --- | --- | --- |
| 1 | Claude couldn't commit, so the loop stopped at every step | Owner approved local commits under their identity; never push. Rules in CLAUDE.md |
| 2 | No Rust toolchain | Installed rustup (owner approved); stable 1.99.1 |
| 3 | Codex's sandbox can't fetch crates | Codex runs with network on and `~/.cargo`, `~/.rustup` writable; smoke-tested `cargo build` with a crates.io dep |
| 4 | Phase 0 cases depended on the owner happening to create them | New Step 0.2a: Claude generates them in `~/multi-link-sandbox/` with recorded ground truth, which also makes labelling exact |
| 5 | Codex would write both the answer key and the code tested against it | Split: Codex scrubs fixtures (0.4), Claude writes schema + expected outputs (0.5) |
| 6 | Expected outputs had no defined schema | `docs/output-schema.md` in Step 0.5 |
| 7 | Phase 2 Verify needed the owner to paste a hook | Test in a sandbox repo with project-scope settings; real repos stay the owner's paste |
| 8 | Codex couldn't read the claude.ai design doc | Exported into `docs/design.md`; the repo copy is now the source of truth |

### 2026-10-29 — Codex critique of the design doc (`codex exec`, read-only)
Verdict: ready with changes. Full table in `docs/design.md` → Review log.
| # | Finding | Decision |
| --- | --- | --- |
| 1 | Exact links lack a correlation protocol | Accepted: exact only when the child id is observed; Phase 0 (a) is go / no-go |
| 2 | One step per session can't model the workflow | Accepted: work segments; explicit tags or `unknown` |
| 3 | Transcript activity doesn't establish state | Accepted: evidence-based state machine with stale / unknown |
| 4 | Heuristic can invent parents | Accepted: recognised invocations, exact checkout, `ambiguous`, score not probability |
| 5 | Verify has no evidence contract | Accepted: per-step Verify with source and staleness |
| 6 | Verify gates can pass without proof | Accepted: negative / ambiguous fixtures, fixed clock, snapshots, 2 s threshold |
| 7 | Time window hides active sessions | Accepted: active discovery separate from history window |
| 8 | Scope expands before dependencies are proven | Partly accepted: Phase 0 go / no-go, backlog; kept repo timeline in Release 1 |
| 9 | Session opening contradicts its safety rule | Accepted: unreachable running session → "unavailable" |
| 10 | Concurrent activity ≠ concurrent writing | Accepted: renamed warning |
