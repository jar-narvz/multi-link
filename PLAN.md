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

### Phase 4 — Home screen cleanup (Release 1.1)
Need: from the home screen I can tell in a few seconds which repo needs me and what is running, the way
mockup screen 6 promises. The real screen buries that (owner, 2026-10-30, after Release 1).
- **Measured (2026-10-30, real `~`, release build, `links --since 24h` / `sessions --since 24h`):**
  multi-link has 69 Codex sessions and 1 is linked; its 43 Codex calls are 42 `unlinked` with 0 candidates
  and 1 `heuristic`, although a Codex rollout in the same cwd starts 1–5 s after most of them. Cause: our own
  command passes `-C "$PWD"`, and `recognise` treats any `-C` as a directory move, so `link::resolve` skips
  the heuristic. All 143 Codex sessions are `unknown` (`turns_complete`). Unparented Codex rows never merge,
  so route-impact shows ~30 one-line Codex rows; idle repos render in full; the header has no counts.
- **U1 `-C` that names the current directory is not a move.** `-C` / `--cd` whose value is `.`, or `$PWD` /
  `${PWD}` written unquoted or inside double quotes, leaves `dir_moved` false. Single-quoted or
  backslash-escaped `$PWD` is literal text and still counts as moved, as does any other value. The
  tokeniser keeps, per word, whether a `$` was expandable. Why: the grammar does no expansion, but these
  forms always mean the shell's cwd, which is the call's checkout. A literal absolute path equal to the cwd
  is out of scope (the recogniser never sees the cwd).
- **U2 `codex ×N`.** Quiet top-level Codex rows (unparented; same quiet states and threshold of 4 as
  `claude ×N`) merge into one MergedRow; MergedRow gains `harness`. Never merges `waiting` / `busy` /
  `stale`, and never mixes harnesses. Why: same rule as Claude rows; a finished unlinked run is history.
- **U3 Quiet repos collapse.** A group whose tier is `other` and that has no `stale` session renders as one
  line (repo, counts, latest step and Verify) until `enter` expands it. TUI state only; the overview report
  is unchanged. Why: mockup screen 6 shrinks idle repos; `stale` can be unfinished work, so it stays visible.
- **U4 Header summary.** The top line counts waiting, busy and stale sessions, FR17 warnings, repos and
  sessions from the overview, and hand-offs as the listed links in `Refresh.links` (the overview keeps only
  each group's latest). Why: answers "does anything need me" before scrolling.
- **U5 Mini time bars.** Each session lane gets its session span (`started_at`..`last_activity`, idle
  gaps included, so a bar means "existed", not "running") on one axis, `TimeWindow::default_for(report)`,
  drawn in a fixed bar column shared by every lane (none below 60 columns). Merged rows get no bar (their
  members' spans may be disjoint). Why: the mockup's main visual; state glyphs still say what is running.
- Non-goals: Codex waiting detection; changing the default `--since 2h`; timeline changes; colour themes.
- Verify (whole phase, 4.6 runs it): `cargo test --workspace` and clippy clean; on real `~`, the
  multi-link link counts re-run and pasted next to the measured ones (no predicted numbers); pty captures
  of the home screen; the owner looks at the real screen and confirms the need is met (a stop-and-ask).

### Step 4.1 — Linker: `-C .` / `-C "$PWD"` is not a directory move (Codex)
- Depends on: Phase 3. Parallel with: 4.2 (claude crate only; the coordinator commits each and runs the
  workspace gate once both are in).
- `crates/claude/src/invocation.rs`: per U1, the `-C` / `-Cvalue` / `--cd value` / `--cd=value` forms take
  their value; the tokeniser records per word whether `$` was expandable. A preceding `cd` segment still
  moves.
- Files: `crates/claude/src/invocation.rs`, `crates/claude/tests/invocation.rs`
- Verify (crate-local while 4.2 runs): `cargo test -p multi-link-claude` → all `ok`, with table cases:
  `codex exec -C "$PWD" …`, `-C $PWD`, `-C .`, `-C${PWD}`, `--cd=$PWD`, `--cd .` → not moved;
  `-C '$PWD'`, `-C \$PWD`, `-C /tmp/x`, `-C "$HOME/x"`, `-C "$PWD/x"`, `cd x && codex exec -C .` → moved;
  `-C` after `--` ignored; `cargo clippy -p multi-link-claude -- -D warnings` clean. Coordinator gate:
  `cargo test --workspace` → all `ok` with no fixture answer key changed (if one changes, stop and report).
  Claude then re-runs `links --since 24h --json` on real `~` and pastes the multi-link unlinked /
  heuristic / ambiguous counts next to the measured 42 / 1 / 0.

### Step 4.2 — Contract: `codex ×N`, collapsed repos, header (Claude)
- Depends on: Phase 3. Parallel with: 4.1.
- Write U1–U5 into `docs/design.md` ("Phase 4 decisions"). `docs/output-schema.md`: MergedRow gains
  `harness`; the merge rule covers quiet unparented Codex rows. New isolated fixture root
  `fixtures/sessions-phase4/` (existing `fixtures/sessions` untouched, so parser counts and CLI tests keep
  their numbers): one repo with 4 quiet unparented Codex rows (merge), one with 3 (no merge) and a busy
  Codex row between quiet ones, one with both Claude and Codex quiet rows (two merged rows), one idle repo
  and one stale-only repo (not collapsed). Hand-written key `fixtures/expected/phase4/overview.json`.
- Files: `docs/design.md`, `docs/output-schema.md`, `fixtures/sessions-phase4/**`,
  `fixtures/expected/phase4/**`
- Verify: the new key is valid JSON and every row in it traces to a fixture file (Review line lists them);
  `cargo test --workspace` → all `ok` (nothing reads the new files yet).

### Step 4.3 — Core: `codex ×N` merge (Codex)
- Depends on: 4.1, 4.2.
- `core::overview` merges quiet unparented Codex rows per the 4.2 contract; MergedRow `harness` in model,
  added to the existing overview keys' merged rows (`"harness": "claude"`). The App identifies a merged
  row by (repo, harness), so a refresh keeps `codex ×N` selected rather than jumping to `claude ×N`.
- Files: `crates/core/src/overview.rs`, `crates/core/tests/**`, `crates/model/src/**`,
  `crates/model/tests/**`, `crates/tui/src/{home,app}.rs`, `fixtures/expected/overview.json`,
  `fixtures/expected/hook/overview.json`, `fixtures/expected/tui/home*.txt` (only if the label changes)
- Verify: `cargo test --workspace` → all `ok`; a core test that `overview` on `fixtures/sessions-phase4`
  matches `fixtures/expected/phase4/overview.json`; unit tests for 3 vs 4 quiet Codex rows, a busy Codex
  row between quiet ones, and two merged rows in one group; an App test that a refresh keeps `codex ×N`
  selected; `cargo clippy --workspace -- -D warnings` clean.

### Step 4.4 — TUI: header summary and collapsed quiet repos (Codex)
- Depends on: 4.3.
- `render_home`: U4 header line; U3 one-line quiet groups. `App`: per-repo expanded set; `enter` on a
  collapsed repo expands it, `esc` collapses; selection never lands on hidden rows; expansion survives
  refresh by repo path.
- Files: `crates/tui/src/**`, `crates/tui/tests/**`, `fixtures/expected/tui/home*.txt`;
  `crates/cli/src/tui.rs` test module only (planner, 2026-10-30: a CLI test opens a session in a repo
  that U3 now collapses, so it must press `enter` on the repo line first; no non-test CLI change)
- Verify: `cargo test --workspace` → all `ok`, a new home golden rendered from the phase4 key, all goldens
  reviewed by Claude against the overview keys (Review line names each checked row); App tests for
  expand/collapse, selection skipping collapsed rows, a stale-only group staying expanded and expansion
  surviving a refresh; clippy clean.

### Step 4.5 — TUI: mini time bars on home lanes (Codex)
- Depends on: 4.4.
- Per U5: fixed bar column, shared axis labels on the header line, no bar on merged rows, none below 60
  columns.
- Files: `crates/tui/src/home.rs`, `crates/tui/tests/**`, `fixtures/expected/tui/home*.txt`
- Verify: `cargo test -p multi-link-tui` → `ok`, goldens reviewed as in 4.4, tests for a span clipped by the
  window, a zero-length span, a merged row (no bar), bars starting in the same column across lanes with
  different label lengths, and a width under 60; clippy clean.

### Step 4.6 — Real check and README (Claude)
- Depends on: 4.5.
- On real `~`: link counts re-run; pty captures (repo names only) of `tui` with the default window and with
  `--since all`, each run back to back with a release build of `9b1b374` (scratch copy, not this checkout)
  at the same pinned `--now`, so before / after differ only by what changed between the two runs. Then stop
  and ask the owner to look at the real screen and confirm the need. README: update what the home screen
  shows; numbers only from these outputs.
- Files: `README.md`, `docs/phase4-check.md`
- Verify: outputs pasted in `docs/phase4-check.md`; whole-phase Verify passes; the owner's answer recorded.

### Phase 5 — Home redesign to the approved target (Release 1.2)
Need (unchanged from Phase 4): from the home screen I can tell in a few seconds which repo needs me and what
is running. Owner, 2026-10-30, after Phase 4: the screen is shorter but "still very hard to easily read and
understand, nothing like the mockup"; Phase 4 closes as not meeting the need and Phase 5 replaces its layout.
The owner approved **board 7** on the mockup canvas (claude.ai artifact R5Bh9iX6bsn3jSu8F8D58C, "7 · Home
target: real data, 30 Oct 16:19"), drawn only from real `overview` + `links` output, and asked to
**prioritise UX/UI, readability and colour**. Board 7 is the acceptance picture for this phase.
- **Why Phase 4 missed:** it was built rule by rule (merge, collapse, header) with colour listed as a
  non-goal and no picture to check against. Rows were named by id prefixes, most rows were old `unknown`
  sessions, bars were a thin dotted track, links used internal words (`heuristic exec`).
- **V0 JSON stays stable; presentation lives in the TUI.** Core JSON gains only additive fields (V1 `made`,
  V8 `hook`); every existing value and row order is unchanged. Windowing, ordering, the cap and combined
  lanes are a home view model in the TUI (`crates/tui/src/view.rs`), built from the overview + links.
- **V1 Rows are named by step.** New SessionRow field `made`: steps of the links this row made (links whose
  `parent` is its id), newest first (`handoffs` lists only links into a Codex row). A Claude row's label is
  its latest made step; a Codex row's is its placing link's step. No step: `claude · no hand-off`,
  `codex · no parent found`; the busy main row reads `claude main · busy now`. Id prefixes leave the home
  screen (the timeline and `o` keep ids).
- **V2 One lane per step, only when it is true.** A Claude row with exactly one Codex child whose placing
  step equals the parent's latest made step (or one of them is null) is drawn as one lane: the Claude span as
  a thin line, the Codex span as a solid block, each mapped on its own times (a child may start or end
  outside the parent's span; overlap draws Codex on top). Otherwise parent and children stay separate rows.
  On a combined lane, `o` opens the Claude session and `enter` splits it into its two rows.
- **V3 Rows: actionable always, the rest windowed.** Per repo: waiting, busy and stale rows always show,
  whatever their age and outside any cap (rank by the row's own state and its children's). Then quiet rows
  by `started_at` ascending (the step chain reads top to bottom), only those with activity in the window,
  at most 8; the rest are one line `N earlier sessions · enter shows all`. The TUI stops drawing
  `claude ×N` / `codex ×N` (core still emits MergedRow; the view flattens it).
- **V4 Colour is a requirement, with tested contrast.** One theme: explicit fg/bg pairs for Claude orange,
  Codex blue, needs-you yellow, warning red, secondary grey, track, card border, selected row; bright for
  busy/waiting, dim for idle/done/unknown. Truecolor when `COLORTERM` is `truecolor`/`24bit`, else 256-colour
  indexes; every text pair ≥ 4.5:1 contrast on the card background (test computes it for both palettes).
  `NO_COLOR`: no colour, the track becomes `·`, spans keep `━` / `█`, and every state keeps its glyph and
  word. Cards have rounded borders; busy repo border orange, waiting yellow.
- **V5 Bars like the mockup.** Fixed bar column right of the label column; dark track cells; Claude `━`
  orange, Codex `█` blue; clipped ends `◂` / `▸`; a zero-length span is one cell. Axis labels above the
  first card in local time, last one `now`.
- **V6 Plain words.** exact → `confirmed`, heuristic → `matched by time`, ambiguous → `N possible runs`,
  unlinked → `no run found`. Last line per expanded repo: `last: claude handed step X to codex at HH:MM
  (matched by time)`.
- **V7 Local time.** Offset read once per refresh for the report's `now` (`time` crate `local-offset`);
  fallback UTC with the axis saying `UTC`. Fractional offsets work. When the window crosses midnight the
  first axis label carries the date (`Oct 29 23:20`). JSON timestamps stay UTC. Limitation: a window across
  a DST change uses the offset at `now`.
- **V8 Header.** `All repos · last 2h`, `● N busy`, `○ N idle · N done or unknown`, repos, hand-offs; at the
  right `◆ N need you` in yellow whenever any row is waiting (records report waiting without a hook). With
  zero waiting and no hook records seen, it says `◆ needs-you partial: no hook records seen` instead of
  `0 need you`. Overview gains `hook`: `{"records": N, "latest": ts | null}` (observed evidence, never
  "installed"); an unreadable hook dir stays a diagnostic.
- **V9 Idle repos, truthful.** One bordered line: name (fixed width), the plan wording the screen already
  uses (latest step id + title, `no PLAN.md`, `PLAN.md not in the step format`), then counts by harness and
  real state (`1 claude idle · 5 codex done or unknown`) and `last HH:MM`; no path. Stale or FR17 repos never
  collapse; the FR17 warning stays a red row. `enter` expands.
- **V10 Widths.** ≥ 100 columns: board 7 layout, 46-column labels. 80–99: 30-column labels, the last line
  shortened to `last: step X → codex · matched by time`. 60–79: 24-column labels, header drops the hand-off
  count. < 60: no bars, text rows only. Labels end with `…` on a word boundary; field priority: state glyph,
  step, harness, link words. Lists taller than the screen scroll with the selection.
- Non-goals: Codex waiting detection; installing the hook for the owner (it stays print-only); the
  timeline screen (later phase); themes beyond the one palette.
- Verify (whole phase, 5.5 runs it): `cargo test --workspace` and clippy clean; existing JSON keys differ
  only by the added fields; the styled golden for the board-7 fixture matches the hand-drawn target from
  5.1 cell for cell (characters, fg, bg, modifiers); an HTML render of the real home screen, screenshotted,
  next to board 7 in `docs/phase5-check.md`; the owner looks at the real screen and confirms (stop-and-ask).

### Step 5.1 — Contract and target screen (Claude)
- Depends on: Phase 4 code (383537e).
- Write V0–V10 into `docs/design.md` ("Phase 5 decisions") and `docs/output-schema.md` (SessionRow `made`,
  overview `hook`, both additive). New isolated fixture root `fixtures/sessions-phase5/` shaped like board 7:
  one busy Claude row, four step chains with one Codex child each (one `confirmed`, the rest `matched by
  time`, one child ending after its parent), two Claude rows with no hand-off, one Codex row with no parent,
  one quiet session before the window, one waiting session from yesterday (must show), two idle repos (one
  with a PLAN.md, one Codex-only). Hand-written `fixtures/expected/phase5/overview.json` and the target
  screen as a styled golden: `fixtures/expected/tui/home-phase5.txt` (text grid) plus
  `home-phase5.styles.txt` (one line per run: row, columns, fg, bg, every modifier), 120×40, fixed offset
  UTC−7, truecolor palette, nothing selected.
- Files: `docs/design.md`, `docs/output-schema.md`, `fixtures/sessions-phase5/**`,
  `fixtures/expected/phase5/**`, `fixtures/expected/tui/home-phase5*.txt`
- Verify: keys are valid JSON and every row traces to a fixture file (Review line lists them); the target
  golden is checked by eye against board 7 (Review line names each region: header, axis, card, step lanes,
  idle lines, footer, and what differs from board 7 because of the added waiting row); palette contrast
  ratios listed; `cargo test --workspace` → all `ok` (nothing reads the new files yet).

### Step 5.2 — Core: `made` and `hook` (Codex)
- Depends on: 5.1.
- Overview fills SessionRow `made` and report `hook` per the 5.1 contract; nothing else in the JSON changes.
- Files: `crates/core/src/{overview,hook,scanner}.rs`, `crates/core/tests/**`, `crates/model/src/**`,
  `crates/model/tests/**`, `fixtures/expected/overview.json`, `fixtures/expected/hook/overview.json`,
  `fixtures/expected/phase4/overview.json` (only the added fields)
- Verify: `cargo test --workspace` → all `ok`, phase5 overview equals its key; a test that the existing keys
  equal their committed versions once `made` / `hook` are removed; unit tests for `made` with zero / one /
  several links, `hook` with no dir, an empty dir, old records only, and an unreadable dir (diagnostic);
  `cargo clippy --workspace -- -D warnings` clean.

### Step 5.3 — TUI theme and styled goldens (Codex)
- Depends on: 5.2.
- `crates/tui/src/theme.rs` per V4 (pure function of the env values passed in). Test helper that dumps a
  ratatui buffer in the 5.1 styled-golden format (fg, bg, every modifier); existing home and timeline goldens
  gain `.styles.txt` files generated once and reviewed.
- Files: `crates/tui/src/**`, `crates/tui/tests/**`, `fixtures/expected/tui/*.styles.txt`,
  `crates/cli/src/tui.rs` (reads the environment once)
- Verify: `cargo test --workspace` → all `ok`; theme tests for truecolor, 256 and `NO_COLOR`; contrast test
  ≥ 4.5:1 for every text pair in both palettes; a `NO_COLOR` render keeps every glyph and word and a visible
  track; clippy clean.

### Step 5.4 — TUI home to board 7 (Codex)
- Depends on: 5.3.
- `crates/tui/src/view.rs` (home view model: V2, V3, V10) and `render_home` per V1–V10; local offset per V7.
  App: `enter` expands an idle repo, splits a combined lane, or shows hidden rows; `o` opens the selected
  row's session (Claude on a combined lane); selection survives refresh.
- Files: `Cargo.toml` (`time` `local-offset`), `Cargo.lock`, `crates/tui/src/**`, `crates/tui/tests/**`,
  `fixtures/expected/tui/home*.txt`, `crates/cli/src/tui.rs`, `crates/cli/tests/**` (only tests that read
  home text)
- Verify: `cargo test --workspace` → all `ok`; the phase5 render equals `home-phase5.txt` and
  `home-phase5.styles.txt` from 5.1 exactly (5.1's golden is not edited here; a mismatch Codex blames on the
  golden is reported, not fixed); view tests: yesterday's waiting row shown, 9 quiet rows → 8 + hidden line,
  a child ending after its parent, differing steps → two rows, a midnight-crossing window, offsets −7 / 0 /
  +5:30; renders at 120 / 100 / 80 / 60 / 40 columns and 10 rows follow V10 with no cut words; App tests for
  the three `enter` cases and `o` on a combined lane; clippy clean.

### Step 5.5 — Visual check on real data and README (Claude)
- Depends on: 5.4.
- `crates/tui/examples/home_html.rs`: renders the home buffer with colours to HTML (fixture or real `~`).
  Screenshot the real-`~` render and the phase5 fixture render in the browser pane, place them next to a
  board 7 screenshot in `docs/phase5-check.md` (repo names only, no conversation content); note every
  difference from board 7. README home-screen section and screenshot updated from these outputs. Record the
  owner's Phase 4 verdict in `docs/phase4-check.md`. Then stop and ask the owner to run `multi-link tui`
  and confirm.
- Files: `crates/tui/examples/home_html.rs`, `docs/phase5-check.md`, `docs/phase4-check.md`, `README.md`,
  `docs/img/**`
- Verify: whole-phase Verify; screenshots in `docs/phase5-check.md`; the owner's answer recorded.

### Phase 6 — Home in words, repo swimlanes (Release 1.3)
Need: from the home screen I can tell in a few seconds which repo needs me and what is running; from a repo
I can see which Codex run came from which Claude session, when each was active, and how sure the link is.
Owner, 2026-11-01, after running Phase 5 on the real screen: the bars on home and the timeline screen are
"really hard to read … random blocks and timestamps on top". Phase 5 closes as not meeting the need for its
bars. The owner approved two targets on the mockup canvas (claude.ai artifact R5Bh9iX6bsn3jSu8F8D58C), both
drawn on the 120×40 cell grid from real data at 1 Nov 22:25 PST: **board 7c** (home: no timeline, one sentence
per session) and **board 8c** (repo view: board 2's swimlanes on real data). Boards 7b, 8 and 8b were
rejected. These two boards are the acceptance pictures; they show real prompts, so they are never committed
as images (W10).
- **Why Phase 5's bars failed:** a bar ran from a session's start to its last activity, so a session open for
  two days filled the window; real work comes in 3–5 minute bursts, which a 2-hour axis turns into specks;
  6 of the rows on the owner's screen were Codex Desktop imports, not runs.
- **W0 JSON stays stable; presentation stays in the TUI.** Every existing key keeps its value, including
  `last_activity`, `state`, window membership, groups, row order and merges; imported sessions stay in every
  report. Additive only: Session and SessionRow gain `activity` (`[{from, to}]`, W1), `asked` (string or
  null, W3), `imported` (bool, W2) and `state_since` (timestamp or null, W5). Hiding imports, counting them
  and every screen rule below live in the TUI view models.
- **W1 Recorded activity = bursts.** Sources: every timestamped record of the session's own file and of its
  subagent files (Claude), minus records inside imported turns (W2) and minus the Codex `session_meta`
  header (Step 6.1: in an import it carries the copy time, so it would draw a fake one-cell burst), minus
  Claude records whose `sessionId` differs from the session's (inherited history, as W3); an `imported` rollout
  has no activity (Step 6.1 review). Timestamps are sorted and deduplicated;
  consecutive ones at most 5 min apart form one burst `[first, last]`; a single record is `[t, t]` and draws
  as one cell; bursts are clipped to the report window (none after `now`). Measured 2026-11-01 on the 12 newest
  real Claude transcripts: 2 of 117 closed turns have an inner gap over 5 min (the rule splits them); a
  turn-based rule was rejected because one real turn left waiting stayed open for 52 h. Screens call it
  "active", never "worked" or "busy time". Limitation (README): a tool call silent for over 5 min shows as a gap.
- **W2 Codex Desktop imports.** Observed 2026-11-01: 140 of 260 local rollouts have only `task_started` turn
  ids starting `external-import` (all originator `Codex Desktop`); they are Claude chats copied into Codex
  Desktop, stamped at copy time. Rule, in file order: an imported turn opens at a `task_started` whose
  `turn_id` starts `external-import` and closes at its matching `task_complete` (inclusive), the next
  `task_started`, or end of file; every record from open to close is excluded from `activity` and `asked`.
  A rollout is `imported` when it has at least one `task_started` and all of them are imported. State and
  `last_activity` are unchanged (W0). Undocumented format: an assumption, tested on fixtures shaped like the
  real files, including an unmatched complete, an import interrupted by a real turn, and real work appended
  to a cached all-import rollout.
- **W3 `asked`.** The session's first prompt. Claude: the first `user` record whose `sessionId` equals the
  session id (inherited history is skipped), not a tool result, with string content or a first `text` block;
  leading whitespace trimmed; skipped if blank or if it then starts with `<` (command and system wrappers).
  Codex: the first `user_message` event outside an imported turn. Whitespace collapsed, cut at 120 characters
  on a char boundary with `…`. Screen label order: latest made step (`step 4.3`) › `asked` in quotes ›
  `claude · no prompt`. Ids stay in `o`, the selected line and JSON. JSON now exports prompt text (README).
- **W4 Home is words (board 7c).** No bars, no axis. Header as Phase 5 V8 but `● N working now`. A repo with a
  waiting, busy or stale session is a card: title (name, plan wording, `N working now` / `N waiting`), one
  line per actionable session (glyph, harness, label, status phrase, branch at the right), a count line for
  the rest (`6 older sessions, none active in the last 2h · enter shows all`), and the last hand-off in
  words. A repo with other activity in the window is one bordered sentence (`idle · last active 1 min ago ·
  claude handed work to codex at 22:23, codex done`). Repos the overview lists with no activity in the window
  share one `quiet` line (`repo-a (only Codex Desktop imports)` when that is why). Imported rows are hidden;
  the footer says `N Codex Desktop imports hidden`, then `times <zone>`.
- **W5 Status phrases and times.** `state_since`: waiting/stale → the record's `statusUpdatedAt` or the winning
  hook record's `at` (the same evidence `state` used); Codex `unknown`/`turns_complete` → the last non-imported
  `task_complete`; otherwise null. Phrases: `working now · for N min` (from the current burst's start;
  `working now` alone when there is no burst), `waiting on you · since HH:MM`, `stale since HH:MM`,
  `idle · last active N min ago` (last burst end, else `last_activity`), Codex `done HH:MM` (defined as "last
  turn complete"; no process-exit evidence, design.md says so). Relative under 60 min, `HH:MM` today,
  `Oct 30` before. Hand-offs: `handed step 4.3 to codex` / `handed work to codex`; outcome `codex done` /
  `codex running` / `no codex run found` / `N possible runs`. No generated `?` placeholder anywhere (user
  prompts may contain `?`). `o` and `f` keep Phase 3's safety rules.
- **W6 Repo view is swimlanes (board 8c); it replaces the timeline screen.** `enter` (and `t`) on a home repo
  opens it. Rows: top-level non-imported sessions with activity in the window, newest first, each Codex child
  nested under its parent (`└▸`, child background). Two text rows per lane: label + bar row, then a dim sub
  line (branch · status, or for a child the link in words: `matched by time · 12 s after the call`). Bars are
  W1 bursts as `█` (Claude orange, Codex blue); `✓` after a Codex run that is `done`, `◆` on a waiting
  session. No rows: one line `no recorded activity in the last 2h`.
- **W7 The axis fits the work.** Default (fitted) range: first burst in the window to `now`; the range never
  leaves the report window (`--since` widens it). Idle stretches over 15 min with no burst on any row fold:
  at most 3 folds (the longest), each a 5-column `≈` band with its length (`43m`) in the axis row; `z`
  unfolds. Ticks from the ladder 1, 2, 5, 10, 15, 30 min, 1, 2, 6, 12 h, 1 d: the smallest leaving ≥ 8 columns
  between labels; faint `│` grid lines. `+`/`-`/`←`/`→` switch to an explicit range (folds off, the last label
  shows its time, `now` only when in range); `0` returns to fitted.
- **W8 Link lines and the selected line.** At the hand-off column, a vertical line from the parent's bar row
  to the child's: `│` confirmed, `╎` matched by time; an ambiguous child stays top-level with `N possible
  runs`. `i` shows imported rows (dim, marked `import`). Bottom `selected` line: harness, 8-character id,
  started, `active about N min in K stretches`, its hand-off; `o` opens, `f` forks.
- **W9 Colour and widths.** Phase 5 theme and contrast tests unchanged; NO_COLOR keeps every glyph and word.
  Repo view below 80 columns drops the bar column and shows the rows as a list (same labels and sub lines).
  Home below 80 columns drops the branch, then shortens labels with `…` on a word boundary.
- **W10 Privacy.** Fixtures use synthetic prompts only. Real renders, board screenshots and JSON with real
  prompts stay in scratch storage, never committed or pasted into PROGRESS.md. Each step's Review checks every
  committed file for real prompts, ids longer than 8 characters, secrets and personal paths.
- Non-goals: hook-free needs-you detection; the step graph (board 3, backlog); live grid; changing `--since`.
- Verify (whole phase, 6.5 runs it): `cargo test --workspace` and `cargo clippy --workspace --all-targets --
  -D warnings` clean; existing JSON keys equal their committed versions once the four added fields are
  removed; the phase6 fixture renders equal the hand-drawn styled goldens from 6.1 cell for cell (home and
  repo view); fixture renders in `docs/phase6-check.md` with boards 7c/8c described (not pictured); real
  renders shown to the owner from scratch; the owner looks at the real screen and confirms (stop-and-ask).

### Step 6.1 — Contract and target screens (Claude)
- Depends on: Phase 5 code (1739a67).
- Write W0–W10 into `docs/design.md` ("Phase 6 decisions"; FR10 reworded to the swimlane repo view) and
  `docs/output-schema.md` (the four added fields, the prompt-export note). New isolated fixture root
  `fixtures/sessions-phase6/` with synthetic prompts, shaped like boards 7c/8c: two busy repos (one with a
  PLAN.md), one idle repo with a Claude parent (two bursts, one subagent file) and a matched Codex child that
  is done, a second Claude session with one burst, an idle gap over 15 min, one all-import rollout, one mixed
  rollout, a repo with only an import, a repo listed with no activity in the window, one confirmed link, one
  waiting session from yesterday, a prompt containing `?`. Hand-written `fixtures/expected/phase6/{sessions,
  overview}.json` and styled goldens `fixtures/expected/tui/home-phase6.*` and `repo-phase6.*` (120×40, UTC−8,
  truecolor, repo view on the idle repo with its parent selected). A shape test checks both goldens are
  120×40 and their styles files cover every cell, and that both keys parse as JSON.
- Files: `docs/design.md`, `docs/output-schema.md`, `fixtures/sessions-phase6/**`,
  `fixtures/expected/phase6/**`, `fixtures/expected/tui/home-phase6*.txt`,
  `fixtures/expected/tui/repo-phase6*.txt`, `crates/tui/tests/phase6_shape.rs`
- Verify: `cargo test --workspace` → all `ok` including the shape test; the Review line traces every key value
  to a fixture line, checks both goldens against boards 7c and 8c region by region (and names what differs
  and why), and confirms W10 for every new file.

### Step 6.2 — Core: imports, activity, asked, state_since (Codex)
- Depends on: 6.1.
- Codex parser: W2 turn state machine and `imported`; both parsers collect timestamps for W1 and the W3
  prompt; Claude subagent timestamps join the parent's; scanner cache stores them; sessions and overview fill
  the four fields; `state_since` per W5 from the evidence `state` already uses. No existing value changes.
- Files: `crates/codex/src/**`, `crates/codex/tests/**`, `crates/claude/src/**`, `crates/claude/tests/**`,
  `crates/model/src/**`, `crates/model/tests/**`, `crates/core/src/{scanner,sessions,overview,state,open}.rs`,
  `crates/core/tests/**`, `fixtures/expected/**/*.json` (only the added fields)
- Verify: `cargo test --workspace` → all `ok`; phase6 sessions and overview equal their keys; a test that every
  existing key equals its committed version once the four fields are removed; unit tests: gaps of 299 / 300 /
  301 s, a one-record burst, out-of-order and duplicate timestamps, subagent-only activity, clipping at the
  window edges; all-import / mixed / no-import rollouts, an unmatched complete, an import interrupted by a
  real turn, real work appended to a cached all-import rollout (second scanner pass), an all-import rollout with a record
  outside every turn (still `[]`), inherited Claude records inside the window (not activity); `asked` skipping
  inherited, tool-result, blank and `<…>` records, a text-block prompt, multibyte truncation at 120;
  `state_since` for waiting by record, waiting by hook, stale, Codex turns_complete, and null; clippy clean.

### Step 6.3 — TUI home to board 7c (Codex)
- Depends on: 6.2.
- Home view model and render per W3–W5, W9: remove bars and the axis; cards, sentences, quiet line, hidden
  imports, footer. `enter` on a repo opens the repo view (until 6.4, the current timeline). The Phase 5 home
  goldens and their immutability test retire here (named in the PROGRESS entry); behaviour tests for
  selection, scrolling and refresh are kept or rewritten, not dropped.
- Files: `crates/tui/src/{view,home,app}.rs`, `crates/tui/tests/**`, `fixtures/expected/tui/home*.txt`
  (except the 6.1 goldens), `crates/cli/tests/**` (only tests that read home text)
- Verify: `cargo test --workspace` → all `ok`; the phase6 render equals `home-phase6.txt` and its styles from
  6.1 exactly (a mismatch blamed on the golden is reported, not fixed); view tests for every W5 phrase,
  relative/clock/date times, a repo with only imports, yesterday's waiting session shown, selection surviving
  a refresh that hides or reveals rows; renders at 120 / 100 / 80 / 60 columns with no cut words; no generated
  `?` placeholder in any render; clippy clean.

### Step 6.4 — TUI repo view to board 8c (Codex)
- Depends on: 6.3.
- Replace `crates/tui/src/timeline.rs` with the W6–W8 swimlane view; App wiring for `enter`/`t`/`esc`/`o`/
  `f`/`z`/`i`/zoom/pan/reset. Old timeline goldens retire with the screen.
- Files: `crates/tui/src/**`, `crates/tui/tests/**`, `fixtures/expected/tui/timeline*.txt`,
  `fixtures/expected/tui/repo*.txt` (except the 6.1 goldens), `crates/cli/src/tui.rs`, `crates/cli/tests/**`
  (only tests that read timeline text)
- Verify: `cargo test --workspace` → all `ok`; the phase6 repo render equals `repo-phase6.txt` and its styles
  from 6.1 exactly; tests: fold on/off, gaps of 14 vs 16 min, more than 3 gaps, tick choice at 40 and 90 bar
  columns and at a 7-day window, explicit range after pan and back with `0`, an empty window, `│` vs `╎`, an
  ambiguous child top-level, a bar clipped at the window start, `i` on/off, selection and `o` after `i`,
  nesting and refresh, below 80 columns the list form; clippy clean.

### Step 6.5 — Visual check and README (Claude)
- Depends on: 6.4.
- Extend `crates/tui/examples/home_html.rs` to render the repo view too. Real home and real repo-view renders
  are screenshotted for the owner in scratch only (W10). `docs/phase6-check.md`: the phase6 fixture renders as
  images, boards 7c/8c described with a link to the private canvas, real-data differences in words and
  counts. Record the owner's Phase 5 verdict in `docs/phase5-check.md`. README: home and repo-view sections,
  fixture screenshots, the W1 limitation, prompt text in JSON. Then stop and ask the owner to run
  `multi-link tui` and confirm.
- Files: `crates/tui/examples/home_html.rs`, `docs/phase6-check.md`, `docs/phase5-check.md`, `README.md`,
  `docs/img/**`
- Verify: whole-phase Verify; W10 check over every committed file; the owner's answer recorded.

### Step 6.6 — Linker: launches in subshells and after heredocs; prompt labels (Codex)
- Depends on: 6.5. Owner, 2026-11-02, on the real route-impact repo view: "make it look like 2, the
  connections between the timelines to show the hand-offs". The view already draws board 2's connection
  (`╎` / `│` from the parent's bar to the nested child) whenever a link exists; here the link was missing.
  The Claude session launched Codex with `(codex exec --sandbox workspace-write "$(cat brief.txt)" <
  /dev/null > /dev/null 2>&1 &)` in a Bash call that first wrote the brief with a quoted heredoc. The
  recogniser reads `(codex` as one word, and the heredoc body (free text with apostrophes) breaks its quote
  tracking, so no invocation is found and the Codex run shows top-level as `codex · no prompt`. Counted
  2026-11-02 over all real Claude transcripts: 221 Bash calls mention `codex exec` / `codex-companion`
  (this includes checks such as `pgrep`), 55 have a heredoc before the launch, 11 launch inside `( … )`,
  17 use a `nohup` / `env` / `timeout` prefix; `links --since all` finds 124 launches.
- **X1 Heredocs.** Outside quotes, `<<WORD`, `<<'WORD'`, `<<"WORD"` and `<<-WORD` start a heredoc: its body
  (the lines after the current line, up to a line that is exactly WORD, leading tabs allowed for `<<-`) is
  skipped by the tokeniser; an unterminated heredoc ends at the end of the command.
- **X2 Grouping.** Outside quotes, `(` and `)` separate segments, and `{` / `}` as whole words are dropped,
  so `(codex exec … &)`, `( cd x && codex exec … )` and `{ codex exec …; }` are recognised. A `cd` inside a
  group still sets `dir_moved` (conservative). `$(…)` command substitution stays one word (its text is not
  a launch).
- **X3 Wrappers.** Leading `nohup`, `env` (with any `NAME=value` words and no other options), and
  `timeout` / `gtimeout` with one duration argument are skipped before matching `codex exec` /
  `codex resume` / the companion. Other prefixes stay unsupported (as Phase 1 decided).
- **X4 Codex `asked`.** Newer `codex exec` rollouts carry no `user_message` event; the prompt is a
  `response_item` `message` with role `user` whose first `input_text` does not start with
  `# AGENTS.md instructions` or `<` (environment and instruction wrappers). W3 falls back to the first such
  item outside an imported turn when no `user_message` exists. Claude: a first prompt wrapped in a single
  known command tag is not unwrapped (unchanged); report how many real Claude sessions still show
  `no prompt` and why, without changing the rule.
- Files: `crates/claude/src/invocation.rs`, `crates/claude/tests/**`, `crates/codex/src/lib.rs`,
  `crates/codex/tests/**`, `crates/core/tests/**`, `fixtures/sessions-phase6/**` (one added launch shape
  only if a fixture test needs it), `fixtures/expected/**/*.json` (only values the new launches or prompts
  change), `docs/design.md` (X1–X4 under "Phase 6 decisions"), `docs/output-schema.md` (the `asked` rule and the
  Step 6.2 note on older Codex fixtures; added in review)
- Verify: `cargo test --workspace` → all `ok` and clippy clean; unit tests for each X1–X3 shape (incl. the
  exact route-impact command with a synthetic brief containing `'` and `"`, an unterminated heredoc, `$(…)`
  not a launch, `cd` in a group sets `dir_moved`), X4 with an AGENTS.md item then the prompt; the 6.1
  goldens unchanged; before/after real counts (in PROGRESS, counts only, no prompts): `links --since all`
  launches by match kind, and Codex sessions in `sessions --since 2h` with `asked` null.

### Step 6.7 — Repo view: `✓ done` label after a finished Codex run (Codex)
- Depends on: 6.6. Owner, 2026-11-02, after checking the real repo view: "add the label wording done like
  in the design; other than that it looks good". Board 2 writes `✓ done` (green) right after a finished Codex
  bar; 6.4 draws only `✓`.
- After a `done` Codex bar, draw `✓ done` in the done colour, one blank cell after the bar's end
  (`█ ✓ done`) when the 6 cells fit inside the bar area without covering another bar, a link line or the fold
  band; otherwise `✓` alone right after the bar, as today. Same in the list form below 80 columns? No: the
  list form already says `done HH:MM` in the sub line. NO_COLOR keeps the text.
- Files: `crates/tui/src/timeline.rs`, `crates/tui/tests/timeline.rs`, `fixtures/expected/tui/repo*.txt`
  (only cells this rule changes; the 6.1 `repo-phase6` pair must stay unchanged because its Codex bar ends at
  the right edge), `PROGRESS.md`
- Verify: `cargo test --workspace` → all `ok` and clippy clean; tests: room → `✓ done`, no room at the edge →
  `✓`, a following bar or link line within 6 cells → `✓`; `repo-phase6.txt` and its styles unchanged;
  the real route-impact view shows `✓ done` after the 16:02 Codex run (scratch screenshot only).

### Phase 7 — Polish the Phase 6 screens (Release 1.4)
Need: the screens the owner approved in Phase 6 say true, readable things on real data. Owner, 2026-11-02:
"Polish leftovers first" (the four open points from `docs/phase6-check.md` that 6.6/6.7 did not fix). No new
screens; one additive JSON key (`last_asked`, Step 7.5); W0–W10 stay in force except where a P rule below replaces one.
- **P1 Claude prompt after a system reminder (replaces W3's Claude rule in part).** Counted 2026-11-02 over
  181 real main transcripts: 43 have `asked` null; in 32 of them every candidate record starts with one
  `<system-reminder>…</system-reminder>` block followed by real text (sessions started by another session);
  the other 11 are other wrappers (`local-command-caveat`, `bash-input`, `scheduled-task`), a reminder with no
  text after it, or no eligible record. Rule: after trimming, while the text starts with exactly `<system-reminder>`
  (no attributes; `<system-reminder-extra>` does not match), drop everything up to and including the first
  `</system-reminder>` after it (no nesting is parsed), then leading whitespace; then apply W3 unchanged
  (blank or starting with `<` → skip this record, try the next). No other tag is stripped; a reminder with no
  closing tag is skipped as today. Codex is unchanged.
- **P2 Home names its window (fixes the fixed `last 2h`).** Home's three window phrases (header `All repos ·
  last 2h`, the card count line, `active about N min in the last 2h`) use the same period words as the repo
  view title (`period(window)`: `2h`, else the fold-length form such as `16h`), from the window the TUI already
  computes for the report (`--since`; `--since all` → first burst to now). `HomeOptions` gains the window;
  its default stays 2h, so every existing home golden is unchanged.
- **P3 Times in the last 24 h keep the clock (replaces W5's date rule).** A clock time less than 24 h before
  `now` is `HH:MM` even when its local date is the day before (a time later than now's clock can only be
  yesterday); 24 h or older → `Mon D` as before. `N min ago` is unchanged. Applies wherever `view::clock`
  is used (status phrases, `last hand-off`, `started`, the selected line). The phase6 fixture's only date
  (`since Nov 1`) is 29.7 h old, so the 6.1 goldens stay unchanged.
- **P4 Axis labels never hide each other.** On the repo view's axis row, labels are placed in this order and a
  later one is drawn only where it covers nothing already placed: (1) fold lengths; (2) the end label, in its
  first form that fits: `HH:MM now` (or `HH:MM` when the range ends before now), then `now` (or nothing);
  (3) an interior `HH:MM now` in an explicit range, then `now`, then nothing; (4) ticks. Shortening to `now`
  removes the current clock from the screen in fitted mode (the header shows the activity range, not now);
  accepted, because `now` is the current time by definition and the old screen hid the fold length instead.
  No tick prints a label equal to the end label's or the `now` label's clock (`14:07  14:07 now` → `14:07
  now`).
- Non-goals: the 11 remaining Claude `no prompt` sessions (other wrappers stay skipped); hook install for
  needs-you; anything in the Backlog.
- Verify (whole phase, 7.4 runs it): `cargo test --workspace` and `cargo clippy --workspace --all-targets --
  -D warnings` clean; `git diff --exit-code 26af6f2 -- fixtures/expected/tui/home-phase6.txt
  fixtures/expected/tui/home-phase6.styles.txt fixtures/expected/tui/repo-phase6.txt
  fixtures/expected/tui/repo-phase6.styles.txt` → exit 0 (pinned to the pre-phase commit; every step runs it);
  real count (counts only): `target/release/multi-link sessions --since all --json | jq -c
  '[.sessions[]|select(.harness=="claude")] | [length, (map(select(.asked==null))|length)]'` — baseline
  2026-11-02 at 26af6f2: `[181,43]`; live transcripts grow, so later runs explain any change in the first
  number; the owner looks at the real screen and confirms (stop-and-ask).

### Step 7.1 — Core: Claude prompt after a system reminder (Codex)
- Depends on: nothing. Rule P1.
- Files: `crates/claude/src/lib.rs`, `crates/claude/tests/**`, `docs/design.md` (P1–P4 under a new
  "Phase 7 decisions" section), `docs/output-schema.md` (the `asked` rule), `fixtures/expected/**/*.json` (only
  `asked` values P1 changes), `PROGRESS.md`
- Verify: `cargo test --workspace` → all `ok`, clippy clean; unit tests: reminder then text → that text;
  two reminders then text → the text; reminder only → next record's prompt; reminder then
  `<local-command-caveat>` → skipped; unterminated reminder → skipped; `<system-reminder-extra>…` → skipped
  (not stripped); plain prompt unchanged; the pinned 6.1-golden `git diff` → exit 0; the phase's real count
  (PROGRESS, counts only): `[181,43]` → `[N,11]` with N ≥ 181, or explain each difference.

### Step 7.2 — TUI: home names its window; clock for the last 24 h (Codex)
- Depends on: 7.1. Rules P2, P3.
- Files: `crates/tui/src/home.rs`, `crates/tui/src/view.rs`, `crates/tui/src/app.rs`,
  `crates/tui/src/timeline.rs` (only to share `period`), `crates/tui/tests/**`, `crates/tui/examples/home_html.rs`
  (pass the window), `fixtures/expected/tui/*.txt` (only cells P3 changes; never the four 6.1 `*-phase6*`
  files), `PROGRESS.md`
- Verify: `cargo test --workspace` → all `ok`, clippy clean; tests: home at 16h says `last 16h` in all three
  phrases, default stays `last 2h`; App-level: the window passed to home follows `--since 16h`, `--since all`
  and survives a refresh; `home_html --since 16h` (no `--repo`) renders `last 16h`; `clock` at now−23h59m on
  the previous local date → `HH:MM`, at now−24h → `Mon D`, across a UTC offset; the pinned 6.1-golden
  `git diff` → exit 0.

### Step 7.3 — Repo view: axis labels never hide each other (Codex)
- Depends on: 7.2. Rule P4.
- Files: `crates/tui/src/timeline.rs`, `crates/tui/tests/timeline.rs`, `fixtures/expected/tui/repo*.txt`
  (only cells P4 changes; `repo-phase6` must stay unchanged), `PROGRESS.md`
- Verify: `cargo test --workspace` → all `ok`, clippy clean; tests: a fold just left of `now` shows its
  length and the end label `now`; a fold with no room even for `now` keeps its length and drops the end
  label; a tick at the end label's minute is not printed (fitted and explicit); explicit range with `now`
  inside: an interior `HH:MM now` next to a fold or tick obeys the order; explicit range ending before now:
  plain `HH:MM` end label, no `now`; the pinned 6.1-golden `git diff` → exit 0.

### Step 7.4 — Real check and owner confirm (Claude)
- Depends on: 7.3. Run the whole-phase Verify. Real renders (home at 2h and 16h, a repo view with a fold if
  real data has one, else the fixture) shown to the owner from scratch only (W10). `docs/phase7-check.md`:
  counts before/after, what each P rule changed on screen, in words. README: the P1 prompt rule and P3 times.
  Then stop and ask the owner to run `multi-link tui` and confirm.
- Files: `docs/phase7-check.md`, `README.md`, `PROGRESS.md`
- Verify: whole-phase Verify; W10 check over every committed file; the owner's answer recorded.

### Step 7.5 — Label quotes the latest prompt, not the first (Codex)
- Depends on: 7.4. Owner, 2026-11-03, while checking Phase 7: "make it so that the quote for the session is
  the most recent message not the first message of that session". Phase 7 stays open until the owner
  confirms after this step.
- **P5 `last_asked`.** Additive JSON key on Session and SessionRow (W0: `asked` keeps its value and meaning):
  the session's latest prompt, string or null, same normalization as W3. Claude: the last `user` record that
  W3 + P1 accept, also skipping records with `isMeta: true` (harness-injected text) or `isCompactSummary:
  true`, and text starting `[Request interrupted by user` (counted 2026-11-03 on real data: if these were
  not skipped, 24 sessions would end on an `isMeta` record and 6 on an interruption). Codex: the last `user_message`
  outside an imported turn; if there is none, the last X4-eligible `response_item` user message. Screen label
  order (W3) uses `last_asked` where it used `asked`: latest made step › `last_asked` › `asked` › `… · no
  prompt`. The selected line and `o` are unchanged. The cache re-parses a changed file, so the label follows
  new prompts on the next refresh.
- Files: `crates/model/src/lib.rs`, `crates/model/tests/**` (added 2026-11-03: Codex stopped because the
  model test builds a `Session` and counts its JSON keys), `crates/claude/src/lib.rs`, `crates/claude/tests/**`,
  `crates/codex/src/lib.rs`, `crates/codex/tests/**`, `crates/core/src/sessions.rs`,
  `crates/core/src/overview.rs`, `crates/core/src/open.rs` (constructors only), `crates/core/tests/**`,
  `crates/tui/src/view.rs`, `crates/tui/tests/**`, `fixtures/expected/**/*.json` (only the added key),
  `docs/design.md` (P5), `docs/output-schema.md`, `README.md` (label sentence), `PROGRESS.md`
- Verify: `cargo test --workspace` → all `ok`, clippy clean; tests: two Claude prompts → `last_asked` is the
  second, `asked` the first; a trailing `isMeta`, compact-summary, interruption, tool result or
  `<system-reminder>`-only record does not replace it; a second Codex `user_message` (resume) → it; Codex with
  only response items → the last eligible one; imported turns ignored; the label shows `last_asked`; every
  committed JSON equals its 081cdcc version once `last_asked` is removed (Python check); the pinned 6.1-golden
  `git diff` → exit 0 (every phase6 fixture session has one prompt); real counts (counts only): Claude and
  Codex sessions in `sessions --since all`, those with `last_asked` null (must equal `asked` null except
  for explained cases), and those where `last_asked` ≠ `asked`.

### Phase 8 — Step graph (Release 1.5) — closed 2026-11-08 (owner confirmed)
Need (design need 4): for one PLAN.md step I can follow who did what, in what order, how many fix rounds it took
and how it ended, across Claude and Codex, without opening transcripts. The owner approved **board 9** on the
mockup canvas (claude.ai artifact R5Bh9iX6bsn3jSu8F8D58C) on 2026-11-06: board 3's step graph drawn on the
120×40 cell grid from real multi-link step 6.7. It is the acceptance picture; it shows real prompts, so it is
described, never committed as an image (W10). The planning session called this Phase 7; that number went to the
polish phase. Owner, 2026-11-06, kept Claude's defaults: the view answers
the need above; it lives on `s` from the repo view (G9); replay is not in this phase.
Measured 2026-11-06 on real data (`links`/`sessions --since all`, at e5f6c78):
- 176 hand-offs: 108 unlinked, 54 matched by time, 13 exact, 1 ambiguous. 76 carry a step tag, in 48 repo+step
  pairs (39 of multi-link's 57 PLAN.md steps). Hand-offs per step: 1 (24 steps), 2 (22), 3 (1), 5 (1). Claude
  parents per step: 1 (46 steps), 2 (1), 5 (1). 33 of the 48 have an unlinked or ambiguous hand-off.
- **Codex rollouts disappear.** 159 on disk now; Step 7.5 counted 295 Codex sessions on 2026-11-03. Unlinked
  hand-offs rose 32 → 108, nearly all dated before 2026-10-31, which points to the lost rollouts. Cause not
  investigated; multi-link never writes there. An old step's graph can therefore be missing its
  Codex runs, and must say so.
- 0 of 362 session segments have a step: `link.step`, from the hand-off prompt, is the only step evidence.
- 55 of 57 reported Verify results parse as `unknown` (free text such as `exit 0; actual result lines:`).
- Non-imported Codex rollouts: 985 `custom_tool_call` named `exec`, 5 `function_call` named `js`, 1 other. The
  `exec` input is JavaScript calling `tools.exec_command({cmd:…})` / `tools.apply_patch(…)` / `tools.write_stdin`;
  866 of 991 outputs hold an `exit_code`. 64 of 65 `task_complete` have `last_agent_message`. All undocumented.
- 49 of 57 steps have a commit subject containing `(Step <id>)`.
- **G0 New report; nothing existing moves.** A new `core::graph` report and CLI `multi-link graph --repo R --step
  ID --json`. No Session, SessionRow, link or overview key changes; every committed JSON key stays equal. The TUI
  draws the report and holds no linking or cutting logic (T2). Why a report and not TUI code: it is answer-keyed
  like every other view, and the desktop app can reuse it.
- **G1 Which sessions belong to a step.** Exactly the hand-offs with `link.step == id` in that repo, their parent
  Claude sessions and their children. Steps are never guessed from time or files. A step with no tagged hand-off has
  no graph: `no hand-offs tagged step 3.5 (a Claude-only step, or its prompt did not name it)`. An untagged hand-off
  inside a chain is not drawn and does not split it; the panel of the Claude part holding it says `N untagged
  hand-offs here`.
- **G2 Chains and Claude parts.** Per parent, the step's hand-offs in time order form chains; a hand-off tagged with
  another step between two of them splits the chain (A → B → A is two chains of A). Chains are ordered by first
  hand-off, across parents. Inside a chain, **a Codex run's interval belongs to Codex and no Claude part covers it**:
  Claude parts are the time left. `plan` = chain start → first hand-off. Between parts = a run's end → the next
  hand-off, kept only when no tagged run of the chain is still going, so a hand-off during a run makes two Codex parts
  neighbours. Last part = the last run's end → chain end. An unlinked or unavailable run has no interval, so Claude's
  part runs through it. Chain start = start of the parent's W1 burst holding the first hand-off, not before the end
  of the previous differently tagged run in that session. Chain end = end of the parent's last burst that overlaps
  the time after the last hand-off, not after the next differently tagged hand-off; if that burst ended before the
  last return, there is no last part. Report windows are `--since all`, so bursts are unclipped. Why cut at
  hand-offs: they are the only events both harnesses record.
- **G3 Naming parts by position.** A Claude part before the chain's first hand-off is `plan`, whatever it holds; first
  Codex `implement`, later Codex `fix round N`; a Claude part with a later hand-off is `review`; a last Claude part is
  `verify + commit` if it holds a `git commit` call, else `review`. A positional name is a reading, not evidence (a
  second run could be new work); README says so.
- **G4 Codex parts come from turns.** A hand-off's window runs to the next hand-off of the same child (else
  unbounded). `codex_turns(path)` gives each turn's start, end, prompt and `last_agent_message`; a run is the
  turns that start in its window, so two resumes inside one W1 burst stay two parts, and a later resume tagged to
  another step stays out. `from`/`to`, `asked`, `said`, status and activity come from those turns (activity =
  bursts intersected with them), never from the whole session. `✓ done HH:MM` (W5 meaning: last turn complete),
  `working now` on an open turn. **Link confidence and run availability are separate fields.** Unlinked: dim `no
  codex run found`, no return arrow, `(the rollout may be gone)` in the panel. Ambiguous: dim `N possible runs`, no
  id, `o` off. Linked but the child has no session, or its file vanishes before reading: the edge keeps its `│` or
  `╎`, the part is dim `run unavailable`, `o` off.
- **G5 Calls.** New `calls` readers in the `claude` and `codex` crates, read on demand for the step's own sessions
  only: never in Session or the scanner's parse (the largest real transcript is 15 MB; a report needs 2–3). Each
  call: `at`, `kind` (`read` / `edit` / `run` / `handoff`), `label` or null, `ok` (true / false / null). Claude: a
  session's owned calls are those in its main file and subagent files, by the same ownership rule as W1 (records with
  another `sessionId` are inherited and skipped), deduplicated by tool id, results joined by tool id; a subagent's
  calls count for the parent. `Read`/`Grep`/`Glob` read; `Edit`/`Write`/`NotebookEdit` edit (file basename); `Bash`
  by its first command word after `export`/`cd`/`VAR=` prefixes: `cat sed grep rg head tail ls awk wc find` and `git
  diff|status|log|show` read, else run; other tools run; `mcp__ccd_*` not counted (app bookkeeping). `ok` from
  `is_error`. Codex: one `custom_tool_call`/`function_call` record = one call, outside imported turns;
  `tools.apply_patch` edit (basename of its first `*** Update|Add File:`), first `cmd:"…"` classified as Bash,
  `write_stdin` run; unreadable input run with null label; `ok` from the first `exit_code` in the output, else null.
  Unknown shapes fail soft (skipped and counted). **Shell commands that write files count as runs**: telling from
  text would be a guess, so real boxes show fewer `edit` lines than board 9 (it counted `python3 … write` and `cat >>`
  as edits).
- **G6 What a box shows.** Line 1: `N reads · N edits · N runs`, zero parts left out, hand-offs not counted. Then at
  most 3 notable calls, newest last, each a **fixed label chosen by category, with no text copied from the command**:
  `▸ codex exec · step 6.7` (`· continue 6.7` when the hand-off is not the chain's first), `$ git diff`, `$ git
  commit`, `$ cargo test|clippy|fmt|build`, `$ pytest`, `$ npm test`, and `edit <basename>` (repeats merged). So
  board 9's `$ cargo test --workspace` reads `$ cargo test`. No other command text reaches the report. `✓`/`✗` mean
  the call exited 0 / non-zero, not "tests passed" (a pipe to `grep` hides a failure); legend and README say so.
  Claude calls during a run add one grey line, `N reads while codex ran`. `asked` and `said` are prompt text: real
  reports stay in scratch (W10).
- **G7 Header.** Repo › step id, PLAN.md title; right `✓ committed <hash> · HH:MM`, from `git log` with the exact
  `(Step <id>)` in the subject, newest first, nothing when absent (fixture mode runs no git; a core test uses a
  temp repo). Line 2: date, range, length, `N claude session(s) · N codex runs · N fix rounds`; right `Verify
  <command> (reported)`, the PLAN.md text cut to fit. Never a pass or fail mark: 55 of 57 results are `unknown`, and reported is not observed.
- **G8 Screen (board 9).** Claude lane rows 4–12, edges 13–15, Codex lane rows 16–24, 30-wide boxes, x = 1 + 22 ×
  slot; next slot is +1 when the harness differs, +2 when the same (no overlap). Up to 5 slots show; more parts
  scroll with the selection, row 3 saying `‹ N earlier` / `N later ›`. Edges: `╎` matched by time, `│` confirmed,
  with words and clock; return edges grey `│ ▲ back to claude`. Selected part: background `#1B2130` and a panel
  (rows 27–31): harness, part, 8-char id, range, `active about N min in K stretches` (core gives `active_ms` and
  `stretches` per part); `asked` (Codex: the run's
  `asked`; Claude parts have no such line, the session's prompts do not map to parts); `from` (hand-off words, as W8); `said`
  (Codex `last_agent_message`, normalized like W3; absent for Claude); `o`/`f`. Rows 33–34: the strip `where the
  time went`, each part min label + 2 cells, the rest by its share of the summed part lengths (parts that overlap
  each count in full, so the strip is shares, not a clock); ends `HH:MM`. Row 36 legend, 38–39
  keys. Colours are Phase 5's theme plus the 8c selection; NO_COLOR keeps every glyph and word.
- **G9 Where it lives.** `s` on a repo-view lane opens that lane's latest tagged step (a lane with none: footer
  `no step on this session`). `←→` moves along parts; `[` `]` previous/next step with tagged hand-offs in the repo,
  in PLAN.md order; `o`/`f` keep FR18's rules for the selected part; `esc` returns to the repo view with its
  selection; a refresh rebuilds the report and keeps the selected part by session and position. Why a key and not a
  new home entry: you reach a step from the session you are looking at, and home stays the 7c answer.
- **G10 Narrow and short.** Below 120 columns: the same parts as a list, two rows each (`claude · plan`, then sub line
  and counts), edges as indented lines; `←→` becomes `↑↓`. Below 40 rows: drop the strip, then the legend, then the
  panel down to its first line. Same order for fixture and real data.
- **G11 Speed.** The derived report is rebuilt on every refresh, so stale state, new rollouts, hook records, commits
  and PLAN.md edits show as everywhere else (design.md, Incremental reads). Parsed calls and turns may be cached by
  file key, as the scanner does. The whole refresh with the graph open must stay inside the 2 s threshold on the
  largest real step (8.5 measures).
- Non-goals: replay and the scrubber (board 3: needs event-level history, and the strip answers where time went);
  inferring steps for untagged hand-offs; fixing `verify` parsing or `unknown` results; explaining or recovering
  missing rollouts; per-call detail view; Codex subagents; changes to home or the repo view except the `s` key and
  footer; the live grid.
- Verify (whole phase, 8.6 runs it): `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D
  warnings` clean; `git diff --exit-code --diff-filter=MD e5f6c78 -- fixtures/expected/` → exit 0 (nothing existing
  moved; every step runs it, 8.1's added files aside); the phase8 render equals the 8.1 golden cell for cell; real
  step 6.7 render compared with board 9 region by region, differences named; the owner looks at the real screen and
  confirms (stop-and-ask).

### Step 8.1 — Contract and target screen (Claude)
- Depends on: Phase 7 close (e5f6c78).
- G0–G11 into `docs/design.md` ("Phase 8 decisions"; FR11 reworded, need 4 mapped) and `docs/output-schema.md` (the
  graph report: `now`, `repo`, `step`, `commit`, `parts[]` with `chain`, `kind`, `harness`, `session`, `from`, `to`,
  `status`, `run_available`, `active_ms`, `stretches`, `asked`, `said`, `counts`, `notable`, `waiting_reads`,
  `untagged_handoffs`, `edge_in`, plus `empty`). Fixture root `fixtures/sessions-phase8/`, fixed `--now
  2026-11-06T18:00:00Z`, synthetic prompts, new Codex `exec` record shape, one repo with a PLAN.md of real-format
  ids: 1.1 shaped like board 9 (plan → implement → review → fix → verify + commit, one confirmed and one matched
  link); 1.2 an unlinked and an ambiguous hand-off; 1.3 seven parts (scroll); 1.4 two parents; 1.5 a hand-off during
  a running run; 1.6 and 1.7 interleaved as A → B → A; 1.8 A → untagged → A; 1.9 two resumes in one burst, the later
  one tagged 1.10; 1.11 an exact child with no rollout; 1.12 a subagent hand-off plus a fork with copied calls;
  1.13 no tagged hand-off. Hand-written `fixtures/expected/phase8/graph-<id>.json`; styled golden
  `fixtures/expected/tui/step-phase8.txt` + `.styles.txt` (120×40, UTC−8, truecolor, 1.1, implement selected). A
  shape test as `phase6_shape.rs`.
- Files: `docs/design.md`, `docs/output-schema.md`, `fixtures/sessions-phase8/**`, `fixtures/expected/phase8/**`,
  `fixtures/expected/tui/step-phase8*.txt`, `crates/tui/tests/phase8_shape.rs`, `PROGRESS.md`
- Verify: `cargo test --workspace` → all `ok` including the shape test; the Review line traces every key value to a
  fixture line, checks the golden against board 9 region by region (names each difference and why, including G5's
  edit counts), and confirms W10 for every new file.

### Step 8.2 — Calls from both transcripts (Codex)
- Depends on: 8.1.
- Claude and Codex `calls` readers per G5, `codex_turns` per G4, the classifier shared in `crates/model`; `Call`
  and `Turn` types. Nothing in Session or the scanner's parse.
- Files: `crates/model/src/lib.rs`, `crates/model/tests/**`, `crates/claude/src/lib.rs`, `crates/claude/tests/**`,
  `crates/codex/src/lib.rs`, `crates/codex/tests/**`, `PROGRESS.md`
- Verify: `cargo test --workspace` → all `ok`, clippy clean; tests: each Claude tool, each Bash prefix, `git diff` vs
  `git commit`, `is_error`; Codex `exec_command`, `apply_patch`, `write_stdin`, two calls in one record (one call),
  `function_call` `js`, output without `exit_code` (`ok` null), unreadable input, an unknown record (counted, no
  crash); imported turns give no calls and no turns; subagent calls owned by the parent, a fork's copied calls skipped,
  a tool id seen twice counted once; turns with no `task_complete`; pinned `git diff` above → exit 0; real counts in PROGRESS (counts only): calls
  by kind over all non-imported rollouts, share with `ok` null, records skipped.

### Step 8.3 — Core: graph report, CLI `graph` (Superpowers trial)
- Depends on: 8.2.
- Workflow: owner's Superpowers trial, rules in CLAUDE.md; Codex reviews instead of implementing. Files and Verify
  are unchanged. Steps 8.2 and 8.3 record the trial's measures in PROGRESS.md.
- Report types in `crates/model` (the TUI does not depend on core); computation in `crates/core/src/graph.rs` per
  G1–G4, G6, G7; `multi-link graph --repo R --step ID [--json]`; `git log` lookup behind the existing
  no-git-in-fixture rule.
- Files: `crates/model/src/lib.rs`, `crates/model/tests/**`, `crates/core/src/{graph,lib}.rs`, `crates/core/Cargo.toml`
  (only if needed), `crates/core/tests/**`, `crates/cli/src/main.rs`, `crates/cli/tests/**`, `PROGRESS.md`
- Verify: `cargo test --workspace` → all `ok`, clippy clean; an integration test runs the CLI for every id in
  `fixtures/expected/phase8/` and compares it with its key; tests: every G2 boundary (burst start, chain split, no
  Claude part during a silent run, parent silent at the return, overlapping background runs), G3 names incl. no commit
  and a chain with no plan part, G4 two resumes in one burst, a child file removed before reading; sentinel path and
  token strings in fixture commands never appear in JSON; a temp git repo for G7 (exact `(Step 6.7)`, not `(Step
  6.70)`); pinned `git diff` → exit 0.

### Step 8.4 — TUI: step graph render to board 9 (Codex)
- Depends on: 8.3.
- `crates/tui/src/graph.rs`, a pure render of the report per G8, G10. No App wiring.
- Files: `crates/tui/src/{graph,lib,theme}.rs`, `crates/tui/tests/**`, `crates/tui/examples/home_html.rs` (`--graph
  REPORT.json`), `fixtures/expected/tui/step*.txt` (except the 8.1 golden), `PROGRESS.md`
- Verify: `cargo test --workspace` → all `ok`, clippy clean; the phase8 render equals `step-phase8.txt` and its styles
  from 8.1 exactly (a mismatch blamed on the golden is reported, not fixed); tests: slot rule with same-harness
  neighbours, 7 parts scrolled at each end, unlinked and ambiguous parts, `│` vs `╎`, strip with a 1-second part, list
  form at 119 / 100 / 80 / 60 columns with no cut words, 39 and 30 rows, NO_COLOR, empty step sentence; the four
  6.1 goldens unchanged.

**Phase 8 amendment (owner's real use, 2026-11-07).** The owner watched Step 8.4 run in `multi-link tui` and could not
tell which row was which. Seen on real data (counts only, W10): three app-started sessions shared one label (their first
prompt is the app's `Background (from the session that started this one): …` preamble); the working 8.4 session showed
`stale since 22:33` (its record said `busy` with `statusUpdatedAt` unchanged for 10+ min during one long `codex exec`,
and `activity_state` turns any busy record older than 300 s into `stale` without the probe); `codex exec resume --help`
was linked as a hand-off; the header said `step 8.3` while 8.4 ran in a worktree. Steps 8.4a, 8.4b and 8.4c go between 8.4
and 8.5. **They amend G0:** Session and SessionRow gain two additive keys (`title`, 8.4b; `started_by`, 8.4c), and
existing values may change only where U3/U4 say so (and the overview tree only where `started_by` is non-null); every
other committed JSON value stays equal. **They amend the whole-phase Verify:** the pinned `git diff --exit-code
--diff-filter=MD e5f6c78 -- fixtures/expected/` holds through 8.4; from 8.4a on, each step instead shows that its
changes to existing keys are only the lines its own rules name (8.4b: `title`; 8.4c: `started_by`). They also amend the phase's "no repo/home
changes" line for exactly the rows U2/U5 change.

### Step 8.4a — Busy is busy; help is not a hand-off (Codex)
- Depends on: 8.4 (merged).
- U3 **Busy is busy.** A Claude record with `status: busy` and a probed-`Alive` process is `busy` whatever the record's
  age. `stale` stays for busy records older than 300 s whose process is `Unavailable`; `Gone` stays `finished`. No new
  duration text (`state_since` and W5 unchanged).
- U4 **Help is not a hand-off.** A recognised `codex exec …` / `codex [exec] resume …` segment is not an invocation
  when `--help` or `-h` is an unquoted, unescaped word after `exec`/`resume` and before the first non-option word.
  Option values (`-c v`, `-o p`, `-C d`, `-s m`, `--add-dir d`) are skipped as values; after `--` nothing is an option.
  Other segments of the same command are still checked. (`codex --version` is already not recognised.)
- Files: `crates/core/src/state.rs`, `crates/claude/src/invocation.rs`, `crates/core/tests/**`,
  `crates/claude/tests/**`, `docs/output-schema.md` (U3, U4), `PROGRESS.md`.
- Verify: `cargo test --workspace` → all `ok`; `cargo clippy --workspace --all-targets -- -D warnings` clean; tests:
  U3 through `FakeProcessProbe` (busy + Alive at 20 min → busy; busy + Unavailable at 6 min → stale; busy + Alive at
  1 min → busy; Gone → finished); U4 `codex exec --help`, `codex exec resume --help`, `codex exec -C d --help` not
  linked; `codex exec "use --help"`, `codex exec -o --help.txt "x"` (value) and `codex exec -- --help` linked;
  `codex exec --help; codex exec "Step 1.2 …"` links the second. `git diff --exit-code fixtures/expected` (no committed
  key changes). Real data, counts only: one scratch snapshot of `links --since all --json` and `sessions --json` taken
  before, compared with the new build on the same `--now`; every difference listed and classified (help-only
  invocation dropped, child reassigned, busy↔stale) in PROGRESS.

### Step 8.4b — Rows named by title and place; header names the latest hand-off (Codex)
- Depends on: 8.4a.
- U1 **Title.** Claude transcripts carry `{"type":"custom-title","customTitle":…}` (undocumented; seen in 47 of the 60
  most recent real transcripts, 2026-11-07). Session and SessionRow (children and merged members too) gain additive
  `title`: the last own record (main file, `sessionId` equal to the session's) with a non-blank string, trimmed, control
  characters removed; else null. Inherited, subagent and malformed records are ignored. Codex: null.
- U2 **Label order.** `title` › step (by the existing placing-parent rule, `view.rs`) › `last_asked` › `asked` › the
  harness's no-prompt text. Each prompt candidate is rejected on its own when it starts with the app's
  `Background (from the session that started this one):` preamble, so a rejected `last_asked` falls through to `asked`.
  The JSON prompts are unchanged. Supersedes P5's order (P5 kept otherwise).
- U5 **Header names the latest hand-off.** When the repo's latest tagged hand-off names a step other than the latest
  logged PROGRESS.md step, the header adds `· latest hand-off step <id>`. It never says "done" or "running". Data path:
  the overview already holds the latest hand-off (`overview.rs`); no PLAN.md title is added.
- U6 **Where it runs.** The place word on a row comes from the session's checkout, not the transcript's `gitBranch`:
  Claude Code keeps the branch it started on (8.4: 220 of 220 records say `main` while 181 have `cwd` in
  `.claude/worktrees/step-8.4`; undocumented). Main checkout → the branch, read through the memoised git (3.1), else
  `gitBranch`; a linked worktree → `worktree <folder name>`. Codex rows the same, by their checkout. Folds in the
  backlog's `@basename` item; the home screen's branch cell uses the same word. No JSON key changes (`branch` stays as
  recorded).
- Files: `crates/claude/src/lib.rs`, `crates/model/src/lib.rs`, `crates/model/tests/**`,
  `crates/core/src/{sessions,overview,open,repo}.rs`, `crates/core/tests/**`, `crates/tui/src/{view,timeline,home}.rs`,
  `crates/tui/tests/**`, `fixtures/**` (synthetic titles only, W10), `fixtures/expected/**` (only `title` added and the
  rows named below), `docs/output-schema.md`, `docs/design.md` (label order), `README.md` (label wording), `PROGRESS.md`.
- Verify: `cargo test --workspace` → all `ok`; clippy as in 8.4a; tests: U1 last title wins, blank/non-string/
  inherited/subagent ignored, title change seen after a scanner refresh; U2 order, preamble rejected per candidate,
  placing-parent rule kept for a Codex child with two parents; U6 main checkout shows the branch, a worktree session whose `gitBranch` says `main` shows `worktree <folder>`; U5 header with a newer-step hand-off, same step, no
  tagged hand-off. JSON: every committed key equal except the added `title` (`git diff <8.4a commit> --
  fixtures/expected ':!fixtures/expected/tui' | grep '^[-+] ' | grep -v '"title"'` prints nothing; 2026-11-08, same fix as 8.4c's point 8; the
  `tui` text goldens are excluded because the goldens rule below covers them, amended 2026-11-08 when the hook
  fixture's titled session changed `home-hook*.txt`). Goldens: the 8.1 golden unchanged; the four 6.1 goldens and the
  other home goldens change only in rows whose fixture session gains a title or a header line, each listed in PROGRESS. W10 check over every committed file,
  titles included. Real run in scratch only, counts only: rows labelled by title, by step, by prompt.

### Step 8.4c — Sessions started by a session nest under it in the repo view (Codex)
- Depends on: 8.4b. Owner asked 2026-11-07: sessions a coordinator session started (8.3, 8.4, design, spike) should
  hang off it the way Codex runs hang off their Claude parent. Extends need 2 to Claude → Claude.
- **Evidence (real data, 2026-11-08, counts only; undocumented app behaviour, an assumption until the fixture and the
  real count below confirm it):** the desktop app keeps one record per app session at
  `<home>/Library/Application Support/Claude/claude-code-sessions/<a>/<b>/local_<id>.json` (macOS). Seen: 135 records;
  130 carry `cliSessionId`, all 130 equal a transcript file name under `~/.claude/projects/*/`; 63 carry
  `spawnedFrom {sessionId: local_<parent>, taskId, name, title, sideSession, …}`, every parent has a record with a
  `cliSessionId`. 4 of the 63 have `spawnedFrom.userFork: true` and those 4 include all 3 with `btw: true`, so **59 are
  eligible**; 2 of the 59 have a parent that is itself an eligible child; 1 child has a different `cwd` from its parent
  (not the same as a different repo). The record survives the session's exit (unlike `~/.claude/sessions/<pid>.json`).
  The child's `local_<id>` never appears in its own transcript, so the earlier draft's evidence (the parent's
  tool_result plus the live record's `hostSessionId`) worked only while the child ran.
- S1 **Started-by link.** A new reader in the `claude` crate lists `<root>/Library/Application Support/Claude/
  claude-code-sessions/*/*/local_*.json` (the same relative path in fixture mode, so a fixture root carries its own).
  A record's identity is its string `cliSessionId`. C is started by P when C's record has `cliSessionId` = C's transcript
  id, `spawnedFrom.sessionId` names a record whose `cliSessionId` = P's transcript id, both are listed Claude sessions,
  P ≠ C, and `spawnedFrom.userFork` / `spawnedFrom.btw` are not `true` (a user fork and a side question are the user's
  acts, not a session's). **Conflicts:** two records with one `cliSessionId` that name different parents, or two
  records sharing one `sessionId`, give no link for the affected child. **Cycles:** an edge that closes a cycle is
  dropped for every session on that cycle; those sessions stay top level. Always `exact`: no time or brief matching
  (all 59 real eligible children resolve by record; a guess layer would add risk and no coverage). **Diagnostics:** an
  absent directory or a missing parent record is normal (no link, no diagnostic); a malformed file or a wrong field type
  is a bad record (existing diagnostic); a directory that exists but cannot be read is unreadable (existing diagnostic).
  Never an error exit. Read on every refresh, cached by file key like the other reads.
- S2 **Contract.** Session and SessionRow gain additive `started_by` (P's transcript id, or null). **Amends G0 (as
  8.4b):** existing fixture roots have no app records, so their output equals 8.4b's except the added `started_by:
  null`; the overview's `rows`/`children` structure, merge and ordering change only where `started_by` is non-null
  (the new fixture root). A started-by link is not a `Link`: it is never in `links`, `handoffs`, `unparented` or any
  Codex hand-off count, and G1 graphs ignore it.
- S3 **Nesting in the repo view.** A Claude row whose `started_by` names a listed session in the **same repo group**
  (by mapped repo, not `cwd`) is drawn under it as a child, to any depth (coordinator › step session › its Codex run;
  coordinator › started › started), indented 2 columns per level. Cross-repo or parent not listed: the child stays top
  level. Every session appears exactly once. Ordering among a parent's children: Codex children keep T11's key (placing
  hand-off time); a started child's key is its `started_at`; ties by id. A parent's state and row order come from its own
  state; only T11's quiet-merge eligibility looks at descendants recursively (a parent is quiet only when every
  descendant is). The started child's connector is a separate ancestry mark, not a hand-off edge: solid `│` anchored at
  the child's `started_at`, and the child's sub line reads `started by this session · HH:MM`. Waiting-bell detection,
  selection, `o`/`f`, counts and quiet merge walk descendants recursively; a refresh that reparents a row keeps the
  selection by session id. **Home is unchanged** (its attention cards stay flat by urgency; ancestry in home is a non-goal).
- Non-goals: home layout; reading the app record's `title` (8.4b's U1 reads the transcript's `custom-title`); linking
  forks or `btw` sessions; `local_<id>` jump or open changes; Windows/Linux app paths; step-graph changes (8.5's G9
  amendment); a graph `started by` header (would need a new report key; not planned).
- **Answer keys first:** the step session (Claude) writes the fixture root and its expected keys in the worktree before
  the Codex hand-off; Codex does not edit them (a mismatch blamed on a key is reported, not fixed).
- Files: `crates/claude/src/{lib,app_record}.rs`, `crates/claude/tests/**`, `crates/model/src/lib.rs`,
  `crates/model/tests/**`, `crates/core/src/{scanner,sessions,overview,open}.rs`, `crates/core/tests/**`,
  `crates/tui/src/{timeline,view,app}.rs`, `crates/tui/tests/**`, `crates/cli/src/main.rs` (overview text count only; added in review 2026-11-08), `crates/cli/tests/**`, `fixtures/sessions-started/**`
  (new root, synthetic, W10), `fixtures/expected/started/**` (new keys), `fixtures/expected/**` (only the added
  `started_by: null`), `docs/output-schema.md`, `docs/design.md` (S1–S3, need 2 mapping), `PROGRESS.md`.
- Verify: `cargo test --workspace` → all `ok`; `cargo clippy --workspace --all-targets -- -D warnings` clean. Fixture
  root `sessions-started` (fixed `--now`, `--since all`): coordinator K started A (A has one exact Codex child) and B; A
  started A2 (depth 3); a user fork F of K; a `btw` session; X whose parent record is missing; Y whose parent is in
  another repo; Z with the same `cwd` as K but a different mapped repo; a duplicate `cliSessionId` naming two parents;
  a two-record cycle; a self-link; a malformed record; a wrong-typed `spawnedFrom`; a record without `cliSessionId`.
  Tests: K › A › (A's Codex child, A2) and K › B; F, btw, X, Y, Z, the conflict, the cycle and the self-link stay top
  level; the malformed and wrong-typed records give bad-record diagnostics; an absent app dir gives none; `links`
  JSON identical with and without the app dir; K's hand-off count unchanged; K is not quiet-merged while A2 is busy; a
  waiting A2 rings the bell; select and open A2; one scanner across record added, changed and deleted, and unchanged
  files reused from the cache; removing the app dir puts every row back at top level; `multi-link graph` for a phase8
  step equals its key (G1 ignores `started_by`). Existing keys: `git diff <8.4b commit> -- fixtures/expected
  ':!fixtures/expected/started' | grep '^[-+] ' | grep -v started_by` prints nothing. Goldens: the 8.1 golden, the four
  6.1 goldens and every home golden unchanged. W10 scan over every new file, app records included. Real run in scratch,
  counts only: sessions with `started_by`, nested vs top level (and why), compared with 63 / 59 / 4 / 2 above (numbers
  move as sessions are added; record the new ones).

### Step 8.5 — TUI: `s`, moving along a step, refresh (Codex)
- Depends on: 8.4c.
- App wiring and CLI loop per G9, G11.
- **G9 amendment (owner, 2026-11-07, with 8.4c):** `s` on a lane with its own tagged hand-off opens its latest step
  (unchanged). A lane with none whose **direct** started children (8.4c) have G1 graphs lists those steps (`started:
  step 8.3, step 8.4`, by child `started_at`, one entry per repo+step) and opens the chosen one; children without a
  graph are not listed; with none at all the footer stays `no step on this session`. Graph membership stays G1.
- **G11 fix (Claude, 2026-11-08, during the step):** measured on real data before any change, a warm `Scanner::graph`
  for step 8.4 (51 parts) took 4.7–4.9 s, almost all in `multi_link_claude::scan(root)` (4.5 s, a full uncached parse of
  234 transcripts; `calls` for the parent took 45 ms). So the graph reuses the scanner's cached Claude transcripts
  (same file-key cache as the overview) instead of a fresh scan; the report is unchanged (the 13 phase8 keys).
- Files: `crates/tui/src/{app,timeline}.rs`, `crates/tui/src/graph.rs` (the open rule shared with the keys, visibility
  only), `crates/tui/tests/**`, `crates/cli/src/tui.rs`, `crates/cli/tests/**`, `crates/core/src/{graph,scanner}.rs`
  and `crates/core/tests/**` (G11 fix only), `fixtures/expected/tui/repo*.txt` (only the footer's `s` key; the 6.1
  pair is unchanged if the key fits, else stop and say so), `PLAN.md`, `PROGRESS.md`
- Verify: `cargo test --workspace` → all `ok`, clippy clean; app tests: `s` with and without a step, `←→` at both ends,
  `[` `]` over steps with no graph, `esc` restores repo selection, `o`/`f` on a done, running, unlinked and ambiguous
  part, a refresh adding a part keeps the selection, a refresh error keeps the screen, an unchanged transcript
  crossing the stale threshold changes the status, resize across 120 columns; `s` on a coordinator lane in
  `sessions-started` lists only its direct children's graphed steps, own tagged hand-off wins, none → footer; the whole refresh on the largest real
  step measured against G11 (milliseconds in PROGRESS); sentinel strings absent from renders.

### Step 8.6 — Real check, README, owner confirm (Claude)
- Depends on: 8.5.
- Real renders of steps 6.7 (board 9's own), 6.6 (unlinked part) and one with a lost rollout, in scratch only (W10).
  `docs/phase8-check.md`: fixture render as an image, board 9 described, differences in words and counts. README:
  step graph section, G3 and G6 limits, missing rollouts. Then stop and ask the owner to run `multi-link tui`, press
  `s`, and confirm the need.
- Files: `docs/phase8-check.md`, `README.md`, `docs/img/**`, `PROGRESS.md`
- Verify: whole-phase Verify; W10 check over every committed file; the owner's answer recorded.

### Step 8.7 — TUI: 12-hour clock on screen (Codex)
- Depends on: 8.6. Owner asked 2026-11-08: "change all the times to 12hr clock am/pm".
- C1 **Format.** Every clock time the TUI draws is 12-hour, no leading zero, lowercase suffix, no space: `9:05pm`,
  `12:00am` (midnight), `12:00pm` (noon), with seconds `8:05:43am`. Ranges keep the dash between full times
  (`9:23pm–10:24pm`). Why compact: axis ticks, graph boxes and header ranges are width-limited; `9:05 PM` would
  cost 3 more cells per label than today and drop more axis ticks. One shared helper in `view.rs` used by
  `view::clock`, `timeline.rs` (`RepoScene::clock`, so axis ticks and headers) and `graph.rs` (`clock`, with and
  without seconds). Date labels (`Nov 1`, `Oct 29 23:20` → `Oct 29 11:20pm`) keep their date part and rules.
- Amends: design.md V5, V7, W5 and README's `HH:MM` wording become the 12-hour form. The pinned 6.1 goldens
  (`*-phase6*`, pinned at 26af6f2 since 6.1) may change in clock cells only, and so may every other TUI golden;
  later steps pin them at this step's commit.
- Non-goals: JSON (timestamps stay RFC 3339 UTC); the CLI text output (`graph` text prints UTC `HH:MM:SS`, a UTC
  time with a 12-hour suffix would mislead); a 24-hour option or setting; re-shooting `docs/img/**` (README notes the
  images predate 8.7); relative times (`13 min ago`) and durations (`42m`, `6 min`).
- Files: `crates/tui/src/{view,timeline,graph,home}.rs`, `crates/tui/src/app.rs` (its in-file tests' clock
  expectations only; amended 2026-11-08 when Codex hit them), `crates/tui/tests/**`, `crates/tui/examples/home_html.rs`
  (only if it formats times), `fixtures/expected/tui/**` (clock cells and any axis tick layout the wider labels
  force), `docs/design.md` (V5, V7, W5, the 12-hour rule), `README.md`, `PROGRESS.md`.
- Verify: `cargo test --workspace` → all `ok`; `cargo clippy --workspace --all-targets -- -D warnings` clean. Unit
  tests: 00:00 → `12:00am`, 00:30 → `12:30am`, 09:05 → `9:05am`, 12:00 → `12:00pm`, 13:07 → `1:07pm`, 23:59 →
  `11:59pm`, with seconds 08:05:43 → `8:05:43am`, across a non-UTC offset; the `Mon D` cut-over at 24 h unchanged.
  No 24-hour time left on screen:
  `grep -hoE '[0-9]{1,2}:[0-9]{2}(:[0-9]{2})?([ap]m)?' fixtures/expected/tui/*.txt | grep -vE '[ap]m$'` prints
  nothing. Only time-bearing lines changed in goldens:
  `git diff ddd91f8 -- fixtures/expected/tui | grep '^[-+] ' | grep -vE '[0-9]{1,2}:[0-9]{2}'` prints nothing (a line
  whose ticks moved still carries a time). JSON and CLI untouched:
  `git diff --exit-code ddd91f8 -- fixtures/expected ':!fixtures/expected/tui' crates/cli crates/core crates/model`
  → exit 0. Axis labels never hide each other (7.3's tests still pass with the wider labels).

### Phase 9 — Truthful states and links (Release 1.6) — closed 2026-11-09 (owner confirmed)
Need (design needs 1 and 2): the screens already exist, but much of what they say is "unknown". A session that
ended should say so and be reopenable; a hand-off whose Codex run is still on disk should find it. Owner chose
these fixes over the live grid on 2026-11-08. No new screen.
Measured 2026-11-08 on real data (`links` / `sessions --since all`, at 6a2c306):
- 206 hand-offs: 118 unlinked, 59 matched by time, 28 exact (`resume_arg` 16, `stdout` 10, `thread_ready` 2),
  1 ambiguous.
- 227 Claude sessions `unknown` / `no_signal`, 10 with a live record. 212 Codex `unknown` / `turns_complete`.
- `~/.codex/sessions` holds 202 `rollout-*.jsonl.zst` (folders 2026-10-16 to 10-30) beside 212 plain `.jsonl`.
  Each is a standard zstd frame (magic `28 b5 2f fd`) holding the same JSONL (`session_meta` first). multi-link
  reads only `.jsonl`, so "Codex rollouts disappear" (Phase 8) is mostly this. Who compresses them (Codex
  0.164.0 itself, presumably) was not checked; nothing in `~/.codex/config.toml` mentions it.
- From `.claude/handoff/backlog-findings.md` (2026-11-07, sandbox on Claude Code 2.1.319 plus real data):
  - Every clean exit (`/exit`, Ctrl-C, SIGTERM, SIGHUP) deletes `~/.claude/sessions/<pid>.json` and writes a
    `cost-state` record to the transcript after the last turn. SIGKILL keeps the record, which is today's
    `process_exit`. On real data the rule marked 204 of 205 pre-existing `unknown` sessions as exited, and 0 of
    10 live transcripts had a `cost-state`. `SessionEnd` (a documented hook event) fired for every clean mode.
  - Of 37 hook-era hand-offs, 36 have `child: null`. 32 redirect Codex's stdout to a file (`> f 2>&1`), so the
    `session id:` line never reaches the hook. 4 pipe through `tail`/`head` (not recoverable).
- Every field above is undocumented except the `SessionEnd` event. All counts move as sessions are added.

Decisions (amend design.md in 9.0):
- **R0 Contract.** Additive only. `state_evidence` gains `transcript_exit` and `session_end`; link `evidence`
  gains `redirect_file` and `resume_turn` (R7; allowed on a heuristic link, unlike every other value); Session gains nothing else. Every committed JSON value stays equal except where a fixture
  added in 9.0 says otherwise. Answer keys are Claude's: 9.0 writes every new key and every existing key a phase
  rule changes (with the list of changed values in PROGRESS), and later steps never edit keys; a key that looks
  wrong is a stop for Claude to fix. Each step's Verify compares parsed JSON (`diff <(jq -S . key) <(… --json | jq
  -S .)`) using `cargo run -q -p multi-link --` with 9.0's fixed `--now`, never the installed binary.
- **R1 Compressed rollouts.** `rollout-*.jsonl.zst` is read like `.jsonl` (same parser after decompressing),
  with the pure-Rust `ruzstd` crate (no C build step, no new tool to install). One opener in the codex crate
  serves every reader: discovery in both the codex crate and the core scanner, `scan`, and `read_details` (graph
  calls and turns). The same id in both forms → read the plain one. Decompressed size is capped at 64 MiB for
  `.zst` only (plain reads unchanged); a file over the cap or that fails to decode is counted where Codex failures
  are counted today (`unreadable`), never an error exit. Cached by file key like other reads.
- **R2 Clean exit from the transcript.** No live session record and the last `cost-state` comes after the last
  `assistant` record and after the last user turn (not meta, not a local command echo) → `finished` /
  `transcript_exit`. A turn after it → today's rule. A live record always wins (it says the process lives).
  Why trust an undocumented marker: it is positive evidence (P1), it was right on 10/10 live and on all four clean
  sandbox modes, and if it changes, sessions fall back to `unknown`, never to a wrong `finished`.
- **R3 Reopening an exited session.** `finished` from `transcript_exit` or `session_end` opens like
  `process_exit` today, but by where it ran (the transcript's last `entrypoint`): `cli` or `sdk-cli` →
  `claude --resume <id>` in the cwd; `claude-desktop` → the app link `claude://code/continue?session=local_<id>`
  when exactly one app record (8.4c reader) maps the transcript id to a `local_<id>`; no mapping, two mappings,
  or a missing or other entrypoint → `unavailable` with the command shown. Why: resuming a desktop session in a
  terminal makes two writers if the app reopens it. The entrypoint and the mapping reach `open` through an
  internal core resolver, not new Session keys (R0).
- **R4 Hand-off child from the redirect file.** When the launch segment of a recognised `codex exec` redirects
  stdout (`>`, `>>`, `1>`, `&>`, `&>>`, `>|`, with or without `2>&1`) to a literal path, that file is read like
  the task output file today (64 KiB, the hook's 25 s poll; the core reads it while it exists). A relative path
  resolves against the payload or record `cwd`, and only when no `cd` (or subshell) comes before the launch
  segment in that command; exactly one stdout redirect on the segment, else none is read. Safe reads: open
  non-blocking, `fstat` the opened file and require a regular file (a FIFO, device or directory → no read), then
  check that the opened file's canonical path is under that `cwd` or the system temp dir (`$TMPDIR`, `/tmp`,
  `/private/tmp`); read at most 64 KiB. Any other path, a `$VAR` or a quoted expansion → no read. Exact only when
  the file holds exactly one `session id:` line (`redirect_file`; with `>>` an older run's id may be there too,
  so two or more → no exact); otherwise today's rules. Order:
  stdout → thread_ready → task_output → redirect_file. Fixture mode reads it root-relative, as P9 does.
  Amends design decision 2 (Exact) to list this source.
- **R5 `SessionEnd` hook record.** The hook gains `SessionEnd` → one record `{"kind":"ended","reason":…}`.
  Order of state evidence for a Claude session: live record → `process_exit` → `session_end` → `transcript_exit`
  → today's rules. An `ended` record counts only when no transcript turn and no later hook record comes after its
  `at` (a resumed session clears it, including when the async record lands late). Reason `clear` ends the old id
  only. The snippet change needs the owner to re-merge it into
  `~/.claude/settings.json` (stop-and-ask at 9.6).
- **R7 A `codex exec resume --last` fix round is a turn of an existing run.** Owner's screenshot of step 8.7
  (2026-11-08): both fix rounds showed `no codex run found` while `implement` looked like one 9-minute run.
  Real rollout `01a1bd11` (counts only): one `session_meta`, then three `task_started` → `task_complete` turns;
  the two resume hand-offs (21:32:27, 21:34:45 UTC) each match a `task_started` 0–2 s later in that file; no new
  rollout was written. Rule: `invocation.rs` classifies `codex exec resume --last` as a resume (today it is
  `Exec`), kept apart from `resume <id>` (which keeps `resume_arg`). Its match searches existing runs in the same
  checkout for a non-imported turn that is not that run's first non-imported turn, starting 0–10 s after the
  hand-off (both readers use the same non-imported turn list). It ignores the exact-launch claims and never
  falls back to the 60 s creation match; `dir_moved` suppression stays. One child (candidates deduplicated by
  id) → `heuristic`, evidence `resume_turn`, `gap_ms` = turn start − hand-off; none → unlinked; two or more →
  ambiguous. An observed id (stdout, R4 redirect log) still wins and makes it exact. Why `heuristic`: decision 2
  keeps `exact` for an observed child id. Why 10 s: a conservative, uncalibrated choice; the two known resumes
  started 0–2 s after their hand-offs; 9.7 counts more. The graph needs no new rule: `core::graph` already gives
  a Codex part the turns between its hand-off and the next hand-off to the same child (G4), so linking the
  resumes alone should split step 8.7 into implement, fix 1 and fix 2. R0 amended: `resume_turn` is the one
  `evidence` value allowed on a non-exact link.
- **R6 A Codex lane's step looks all-time.** A Codex lane whose Claude parent is outside the TUI window takes its
  step from the worker's all-time hand-offs (per Codex id), so it no longer says `no step on this session`. The
  rules of `own_step` stay: the parent must belong to the lane's repo group, and the latest tagged hand-off wins.

Non-goals: the lost key on resize (wait for upstream crossterm-rs/crossterm#1154; rare, and a vendored fork costs
more than it fixes); T10 as first written (read the task path from the transcript: about 0 of 37 gain); pipes to `tail`/`head`; the live
grid and pane jump; deciding who compresses rollouts; Windows/Linux paths.

Whole-phase Verify: `cargo test --workspace` → all `ok`; `cargo clippy --workspace --all-targets -- -D warnings`
clean; `git diff --stat 6a2c306 -- fixtures/expected` lists only `phase9/` and the existing keys 9.0 listed in
PROGRESS, and `cargo test` compares every key; W10 scan over committed files.

### Step 9.0 — Rules, fixtures and answer keys (Claude)
- Depends on: 8.7. R0–R6 into `docs/design.md` (and decision 2's Exact sources), the new enum values into
  `docs/output-schema.md`. Synthetic fixtures (W10) and Claude-written answer keys for every later step, before
  any code: a `.zst` rollout (and one with the same id in plain form, one corrupt); a clean-exit `cli` transcript,
  a clean-exit desktop transcript with an app record and one without, a transcript with a turn after its
  `cost-state`, a SIGKILL session with its record; a redirected hand-off with its log holding the id, one whose
  log has no id, one redirected outside cwd and temp, one to `$VAR`; a `SessionEnd` hook record (exit and
  `clear`, and exit → resume → turn); a Codex lane whose parent is outside a 2 h window, one with a newer cross-repo
  parent. Fixed `--now` and file names recorded in PROGRESS. Existing fixtures that already hold a `cost-state`
  after the last turn get their keys updated here, each changed value listed in PROGRESS.
- Files: `docs/design.md`, `docs/output-schema.md`, `fixtures/sessions-phase9/**`, `fixtures/expected/**` (new
  `phase9/` keys and only the listed existing values), `PROGRESS.md`
- Verify: `cargo test --workspace` → all `ok` except tests that compare changed existing keys, which are listed
  by name in PROGRESS and must pass by 9.2; every new fixture file is synthetic (W10 scan); each later step's Verify
  names a key that exists.

### Step 9.1 — Read compressed Codex rollouts (Codex)
- Depends on: 9.0. R1.
- Files: workspace `Cargo.toml`, `Cargo.lock`, `crates/codex/Cargo.toml`, `crates/codex/src/**`,
  `crates/codex/tests/**`, `crates/core/src/{scanner,graph}.rs` (discovery and `read_details` callers only),
  `crates/core/tests/**`, `PROGRESS.md`
- Verify: tests and clippy clean; `multi-link sessions --root fixtures/sessions-phase9 --now <9.0 now> --since all
  --json` equals its 9.0 key for the `.zst`, duplicate and corrupt cases (corrupt counted, exit 0); `graph --json`
  on a step whose Codex run is only in `.zst` equals its key (calls and turns read); every other committed key
  unchanged. Real run in scratch, counts only: Codex sessions and unlinked hand-offs before/after
  (118 unlinked at 6a2c306).

### Step 9.2 — Clean exit from the transcript; reopen by where it ran (Codex)
- Depends on: 9.1. R2, R3.
- Files: `crates/claude/src/{lib,app_record}.rs`, `crates/claude/tests/**`, `crates/model/src/lib.rs`,
  `crates/core/src/{state,open,scanner,sessions}.rs`, `crates/core/tests/**`, `crates/cli/src/main.rs` (only to
  pass the resolver to `open`), `crates/cli/tests/**`, `PROGRESS.md`
- Verify: tests and clippy clean; `sessions --json` on the 9.0 root: clean cli and desktop → `finished
  transcript_exit`, turn after `cost-state` → `unknown no_signal`, live record → its record state (fixture mode
  trusts records); SIGKILL → `finished process_exit` in a core test with `FakeProcessProbe::Gone`;
  `open --dry-run`: cli and sdk-cli → `resume`, desktop with one app record → app link, desktop with none or two →
  `unavailable` with the command; all equal their 9.0 keys. Real run, counts only: Claude `unknown` before/after
  (227 at 6a2c306), split by entrypoint and by open action.

### Step 9.3 — Hand-off child from the redirect file (Codex)
- Depends on: 9.2. R4.
- Files: `crates/claude/src/invocation.rs`, `crates/claude/tests/**`, `crates/model/src/lib.rs`
  (`LinkEvidence::RedirectFile`), `crates/hook/src/lib.rs`, `crates/hook/tests/**`,
  `crates/core/src/{link,hook}.rs` (the hook reader accepts `redirect_file`), `crates/core/tests/**`, `PROGRESS.md`
- Verify: tests and clippy clean; `links --json` on the 9.0 root: redirected with id → `exact redirect_file`; no
  id, two ids (`>>`), outside cwd/temp, `$VAR`, after a `cd`, two redirects → never exact; with the log removed →
  never exact unless a persisted hook record says `redirect_file`; hook and core tests for a FIFO (returns at once,
  no read), a symlink escaping to `$HOME`, and an oversized log; hook unit tests for each redirect form; equal to
  their 9.0 keys. Real run, counts only: exact links before/after (28 at 6a2c306; old logs are
  mostly gone, so expect a small gain now and the hook to help from here on).

### Step 9.4 — `SessionEnd` hook record (Codex)
- Depends on: 9.3. R5.
- Files: `crates/hook/src/lib.rs`, `crates/hook/tests/**`, `crates/cli/src/hook.rs`, `crates/cli/tests/**`,
  `crates/core/src/{hook,state,sessions}.rs`, `crates/core/tests/**`, `crates/claude/src/lib.rs` and
  `crates/claude/tests/**` (only to expose the main transcript's last *turn* timestamp; added 2026-11-08, see
  Review log), `PROGRESS.md`
- Verify: tests and clippy clean; the 9.0 `SessionEnd` fixtures → `finished session_end` (exit) and the `clear`
  case as its key says; exit → resume → turn → not `session_end`; precedence tests (live record, `process_exit`,
  `session_end`, `transcript_exit`, and against hook `waiting` evidence); the printed
  snippet includes `SessionEnd` with the same `async`/`timeout` shape. Sandbox PTY `/exit` in
  `~/multi-link-sandbox/repo-a` with the new snippet in its `settings.local.json` → one `ended` line with reason
  `prompt_input_exit`.

### Step 9.5 — Codex lane's step from all-time hand-offs (Codex)
- Depends on: 9.4. R6.
- Files: `crates/core/src/{scanner,overview}.rs` (only if the worker's data comes from there), `crates/tui/src/app.rs`,
  `crates/cli/src/tui.rs`, `crates/tui/tests/**`, `crates/cli/tests/**`, `PROGRESS.md`
- Verify: tests and clippy clean; a worker-to-App test (not a prefilled map) on the 9.0 fixture: `s` on the Codex
  lane whose parent is outside the 2 h window opens its tagged step; with an older same-repo parent and a newer
  cross-repo one, the same-repo step opens; a Codex lane with no tagged incoming hand-off still says `no step on
  this session`; goldens unchanged.

### Step 9.6 — Fix rounds from `codex exec resume --last` (Codex)
- Depends on: 9.5. R7. Owner added this to Phase 9 on 2026-11-08 after the step 8.7 screenshot. Claude writes
  this step's fixtures and answer keys first (9.0 predates R7): a run with three turns and two `resume --last`
  hand-offs (each turn 1 s after its hand-off), one whose turn starts 15 s late (unlinked), two runs in the same
  cwd both starting a turn in the window (ambiguous), and the graph report for that step.
- The selected panel: on step 8.7 the `plan` part's panel said `from handed fix to codex at 1:34:45pm`. Its rule
  (`crates/tui/src/graph.rs:804`) picks the latest hand-off whose parent part is the selected one, so the missing
  review parts made later fixes point back to `plan`; linking should fix it. Keep that rule and its fallback for
  hand-offs made during Codex work; add exact panel assertions: `plan` reports the implement hand-off and each
  review reports its own fix.
- Files: `crates/codex/src/lib.rs` (non-imported turn list shared by both readers), `crates/codex/tests/**`,
  `crates/claude/src/invocation.rs`, `crates/claude/tests/**`, `crates/model/src/lib.rs`
  (`LinkEvidence::ResumeTurn`), `crates/core/src/link.rs`, `crates/core/tests/**`, `crates/cli/tests/**`
  (Phase 9 key runner), `crates/tui/tests/**` (panel assertions), `fixtures/sessions-phase9/**` and
  `fixtures/expected/phase9/**` (Claude, before code), `fixtures/expected/**` (existing keys whose hand-offs
  become `resume_turn`, each listed by Claude), `docs/design.md` (R7, decision 2), `docs/output-schema.md`
  (evidence on heuristic links), `PROGRESS.md`
- Verify: tests and clippy clean. Recognition tests: `--last` with wrappers and options, help excluded, `--last`
  inside prompt text is not a flag, `resume <id>` still `resume_arg`, a `cd` before it. Link tests: gaps −1 ms,
  0, 10 000 and 10 001 ms; another checkout; two concurrent candidates; imported-only, imported-then-first-real
  and later-real turns; an observed id wins. The named Phase 9 key tests in `crates/cli/tests/` compare `links`
  and `graph` on the 9.0 root with the keys (two resume hand-offs → `heuristic resume_turn` to one run, late →
  `unlinked`, two runs → `ambiguous`; graph: implement, fix 1, fix 2 with a Claude review part between) and fail
  with resume matching turned off (shown once in PROGRESS). Real run, counts only: unlinked hand-offs
  before/after, and step 8.7's parts (expected: both fix rounds linked, implement about 2 min).

### Step 9.6a — A Claude session's folder ignores moves outside it (Codex)
- Owner asked 2026-11-08 (screenshot): a session that ran one shell `cd` into its shared notes folder (outside the
  repo, not a git repo) left multi-link on home, lost its parent nesting and showed as its own repo named after the
  folder, branch `HEAD`. Cause: `crates/claude/src/lib.rs` keeps the newest record's `cwd` and `gitBranch`. Claude
  Code lets a shell `cd` persist inside any of the session's working directories and stamps later records with it
  (undocumented; seen once, session c205b4c0). The newest `cwd` must still win for worktrees: 8.4's sessions have
  `cwd` in `<repo>/.claude/worktrees/step-8.4` and U6 shows them as `worktree step-8.4`.
- **R8 Session folder.** The first record of the session's own with a `cwd` sets the session's home folder H. Own =
  `sessionId` absent or equal to the id the parser already uses for activity (`activity_id`); a resumed session's
  file opens with records inherited from the earlier session (fixture `85a715ba…`: inherited records in repo-f, its
  own in repo-c, key says repo-c). Before H is set, every record updates `cwd` and `gitBranch` as today, so a file
  of inherited records only is unchanged. After H is set, a record's `cwd`
  counts only when it is H or inside H, compared by whole path components (`/r/repo-other` is not inside
  `/r/repo`), no filesystem access. `Transcript.cwd` is the newest counted `cwd`. `gitBranch` is taken only from
  records whose `cwd` counts or that have no `cwd`. Nothing else changes: per-call checkouts (`invocation.rs`) keep
  their own record's `cwd`, and no JSON key changes.
- Files: `crates/claude/src/lib.rs` (the rule and its unit tests), `crates/claude/tests/**`, `PROGRESS.md`
- Verify: `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` clean; every existing
  answer key unchanged. New tests: H then a folder outside H then H's worktree → the worktree, branch from the
  worktree record; H then outside only → H with H's branch; a sibling with H as a string prefix → ignored; a record
  without `cwd` between them keeps today's branch update; the first `cwd` appearing after records without one is H.
  Real run (release build, scratch only): `multi-link overview --json --since 2h` lists no repo under
  `side-session-notes`, and session `c205b4c0` has `checkout` under `/Users/jarednarvaez/multi-link`.

### Step 9.7 — Real check, README, owner confirm (Claude)
- Depends on: 9.6. Real counts before/after for every step (counts only, W10). Re-shoot `docs/img/**` for the
  12-hour clock and the new states. README: what `finished` now means and its evidence, the `.zst` reading, the
  redirect rule and its path limit, and that all of it rests on undocumented formats. Then stop and ask the owner
  to re-merge the hook snippet that `multi-link hook install --scope user` prints into `~/.claude/settings.json`,
  reinstall, and confirm the need.
- Files: `docs/phase9-check.md`, `README.md`, `docs/img/**`, `PROGRESS.md`
- Verify: whole-phase Verify; W10 scan over every committed file; the owner's answer recorded.

### Phase 10 — Hook setup in one command (Release 1.7) — closed 2026-11-10 (owner confirmed)
Need (owner, 2026-11-09: "make it as simple and easy as possible"): someone setting up multi-link should not have to
hand-merge JSON into `~/.claude/settings.json`, and should be able to tell whether their hooks are current. Today
`hook install` prints a snippet and says "merge it"; the owner did that by hand for 9.4's `SessionEnd`.
Measured 2026-11-09 on the owner's real settings (read-only, keys and hook fields only): five multi-link entries,
one per snippet event, all `async: true`, `timeout: 60`, right matchers; four run
`…/multi-link/target/release/multi-link`, `SessionEnd` runs `~/.cargo/bin/multi-link`. So a hand merge already left
two binaries in use; if `target/` is cleaned, four hooks fail silently.

Decisions:
- **H0 Read-only stays absolute.** multi-link never writes a settings file (no `--write`). `hook check` reads and
  reports; `hook check --merged` prints the whole merged settings to stdout. The printed command does the write with
  POSIX shell tools only (`mktemp`, `cp`, `chmod`, `mv`), plus multi-link itself. Why not `jq` or `python3`: neither
  is guaranteed on a fresh Mac, and the user should not need a second tool.
- **H1 One snippet definition.** The snippet (events, matchers, entry fields) moves from `crates/cli/src/hook.rs`
  into the hook crate as one function of the binary path; `install`, `uninstall` and `check` all call it, so they
  cannot drift.
- **H2 What counts as a multi-link entry.** A hook object with `"type": "command"` whose `command` is a binary
  followed by exactly ` hook record`, where the binary (single-quoted as install prints it, or bare) has the file
  name `multi-link`. Anything else is the user's and is never changed or reported.
- **H3 Findings, per snippet event.** For each multi-link entry: *generated shape* = its keys are a subset of
  `type`, `command`, `async`, `timeout`, and its group's `matcher` is the snippet's (for an event whose snippet group
  has no matcher, absent, `""` and `"*"` are equal; `Bash` and `permission_prompt|elicitation_dialog` must match
  exactly). Event findings: `ok` (exactly one entry, generated shape, equal to the snippet entry: this binary's
  path, `async: true`, `timeout: 60`); `missing` (none); `outdated` (one, generated shape, with each difference:
  other binary path with both shown, `async`, `timeout`); `duplicate` (two or more, all generated shape);
  `customized` (any entry with another key, e.g. `if`, `shell`, `args`, or another matcher). Multi-link entries under
  an event the snippet lacks → `unknown event`. A top-level `disableAllHooks: true` is reported. Why "this binary"
  is the reference: the hook should run the binary the user just checked with, and both paths are shown.
- **H4 Merge.** `missing`, `outdated` and `duplicate` events are fixed: remove those multi-link entries, drop a
  group left with no hooks, append the snippet's group at the end of the event's list. `customized` and
  `unknown event` entries are never changed: the check names them and says to edit them by hand (Codex critique 3:
  a user's `if` or narrower matcher is intent, not staleness). Nothing else changes: other top-level keys, other
  events, the user's entries and their order. Key order is kept by enabling serde_json's `preserve_order`
  (workspace feature; answer keys compare parsed JSON, which ignores order). Output is 2-space pretty JSON with a
  trailing newline, so whitespace may change. Values may not: a file with a duplicate object key, or a number whose
  text serde_json would not write back the same (beyond i64/u64, or a float that does not round-trip), is refused
  (exit 2), checked on the source text before any merge.
- **H5 The file and its errors.** `--settings <path>` names the file directly (overrides `--scope`). Otherwise
  `--scope user` reads `$CLAUDE_CONFIG_DIR/settings.json` when that variable is set, else `~/.claude/settings.json`;
  `--scope local` as `install` does today. The path is made absolute; a symlink is followed to its target R (all
  reads and the write use R; the link stays). Output always names the file read. Absent file → every event
  `missing`, and the command creates it. Exit 2, the reason printed, no command: a dangling symlink, a directory,
  FIFO or other non-regular file, an unreadable file, a path holding a newline, invalid JSON, an empty file, a top
  level that is not an object, `hooks` / an event / a group / a group's `hooks` of the wrong type, H4's refusals.
  `--merged` runs the same checks, so a bad file never reaches the write.
- **H6 The printed command** (paths absolute and quoted with the existing POSIX single-quote function; `R` the
  resolved file, `E` this binary, `M` R's current octal mode, `TS` UTC `YYYYmmddTHHMMSSZ`). Existing file:
  `tmp=$(mktemp 'R.multi-link-XXXXXX') && cp -p 'R' "$(mktemp 'R.multi-link-backup-TS-XXXXXX')" && 'E' hook check
  --settings 'R' --merged > "$tmp" && chmod M "$tmp" && mv "$tmp" 'R' || { rm -f "$tmp"; false; }`.
  Absent file: `mkdir -p 'D' && tmp=$(mktemp 'D/settings.json.multi-link-XXXXXX') && 'E' hook check --settings 'R'
  --merged > "$tmp" && [ ! -e 'R' ] && mv "$tmp" 'R' || { rm -f "$tmp"; false; }`.
  Why this shape: `mktemp` makes unique, private (0600) files, so two runs or an existing file cannot collide or be
  followed; `mv` in the same directory is atomic, so the live file is either old or new, never half-written
  (critique 1); `chmod M` keeps the mode; writing to R keeps a symlink a symlink; any failure removes the temp file
  and returns non-zero. `--merged` re-reads R when the command runs, so earlier edits are kept. Remaining limit,
  documented: an edit to R in the milliseconds between that read and the `mv` is lost (single-user local file; no
  lock exists that Claude Code also honours).
- **H7 Output and exit.** `hook check`: plain text on stdout, the file name, one line per event, then (if any fix
  applies) one line of explanation and the command on its own line; `customized` lines say what to edit by hand.
  Exit 0 when every event is `ok` ("multi-link hooks in <file> are current"; it does not prove another settings
  level or a managed policy lets them run), 1 when anything is not `ok`, 2 on error. `hook check --merged`: only the
  merged JSON on stdout, exit 0 on success (also when nothing changes), 2 on error, diagnostics on stderr
  (critique 6). `install` adds one stderr line pointing to `hook check`.

Non-goals: a `--write` flag or any write by multi-link; `--json` output for `check`; removing hooks (`uninstall`
keeps printing what to remove); keeping the file's whitespace; merging customized entries; reading other settings
levels or managed policy; `--scope project`; locking against concurrent writers; Windows; checking that the binary
path exists (reported only as a path difference).

Whole-phase Verify: `cargo test --workspace` → all `ok`; `cargo clippy --workspace --all-targets -- -D warnings`
clean; every committed answer key unchanged; W10 scan over committed files.

### Step 10.1 — `hook check` and `--merged` (Codex)
- Depends on: 9.7. H0–H7.
- Files: workspace `Cargo.toml` (serde_json `preserve_order`), `Cargo.lock`, `crates/hook/src/**` (snippet and the
  pure check/merge/refusal logic on text and `serde_json::Value`, no I/O), `crates/hook/tests/**`,
  `crates/cli/src/hook.rs`, `crates/cli/tests/hook_check.rs`, `PROGRESS.md`
- Verify: tests and clippy clean; every existing answer key unchanged. Hook-crate unit tests: `{}`; the current
  snippet (`ok`); the pre-9.4 snippet (no `SessionEnd` → `missing`); another binary path (`outdated`, both paths);
  `async: false`; `matcher: "*"` on `SessionEnd` (`ok`); matcher `Bash|Edit` on `PostToolUse` (`customized`, kept);
  an `if` key (`customized`, kept); two entries in one event (`duplicate`, merged to one); the user's own entries in
  the same event and group (kept, in order); a multi-link entry under `Stop` (`unknown event`, kept); `echo
  multi-link hook record` (not ours); `disableAllHooks: true` reported; a duplicate key and `1e400` and
  `18446744073709551616` (refused); each wrong-type case of H5. `crates/cli/tests/hook_check.rs`, temp dirs only
  (never `$HOME`, `CLAUDE_CONFIG_DIR` set to a temp dir for the scope test): check exit 0/1/2; `--merged` stdout parses
  as JSON alone and exits 0 on a no-op; run the printed command with `sh -c` → backup byte-identical to the
  original, re-check exits 0, non-hook values and top-level key order unchanged, mode 0644 and 0600 kept, a symlinked
  file stays a symlink and its target is updated, an absent file is created; a path with a space, an apostrophe,
  `$(…)`, a backtick and a leading `-` works; with a stub binary as `E` that exits 2 → the original's bytes
  unchanged, no temp file left, the command exits non-zero; dangling symlink, directory, FIFO, unreadable file and
  invalid JSON → exit 2, no command, file untouched.
  Real run (release build, read-only): `multi-link hook check` on the owner's settings, events and findings only,
  pasted in PROGRESS (expected: `outdated` for the entries whose binary differs from the one running it). Do not run
  the printed command on real settings.

### Step 10.2 — README setup, real merge, owner confirm (Claude)
- Depends on: 10.1. README "Install" uses `hook check` and its printed command instead of "merge the JSON"; the
  limitations section gets H4's whitespace note, H6's millisecond race and H7's "one file" meaning. Stop and ask the
  owner to run the command `hook check` prints from their installed binary (`~/.cargo/bin/multi-link`), then
  start and `/exit` one Claude session.
- Files: `README.md`, `docs/design.md` (H0–H7 as "Phase 10 decisions"), `PROGRESS.md`
- Verify: whole-phase Verify; after the owner's run, `multi-link hook check` exits 0 on real settings and a backup
  file sits beside them; the session started after the merge shows `finished` / `session_end` in `multi-link
  sessions --json` (proof the merged hooks actually run, critique 10); the owner's answer recorded.

### Phase 11 — Where is now? (Release 1.8) — closed 2026-11-10 (owner confirmed)
Need (owner, 2026-11-08): "make it clear where the agents / current task are", in the repo swimlanes and the step
graph. Today the repo view marks the present only with a `now` axis label (and a trailing fold can hide even that,
board 11e), and the step graph marks a running Codex part only with the words `working now` inside its box. Board 11
(canvas R5Bh9iX6bsn3jSu8F8D58C, boards 11a–11i) was approved by the owner on 2026-11-10.
Comparables (board 11 notes): calendar/Gantt "today" lines (one coloured line over every row, head marker on the
axis); CI graphs (the running job marked on its node, finished jobs recede); htop/k9s (one colour per state, never
reused).

Decisions:
- **N0 One colour for now.** Theme gains `now`: truecolor `#E58AE5`, indexed 176. It is used only for "now", in both
  views, never for anything else. NO_COLOR keeps the glyphs (`┃`, `▼`, `●`), which already differ from grid `│` and
  part `●` by shape or position.
- **N1 Repo view now line.** When bars are drawn and `axis.start ≤ now ≤ axis.end` (fitted mode ends the axis at
  `min(now, window.end)`, so a window that ends before now has no line), a presentation-only resolver picks the
  column: `now == axis.end` → the last axis cell (`cells − 1`), even when a trailing fold leaves the end segment zero
  width; now inside a fold → that fold's last cell (`x + 4`); otherwise `axis.col(now)`. Bar and link mapping are not
  changed. That column gets `▼` on the axis rule row and `┃` in `now` on every row below it (from the grid row right under
  the rule, as board 11b, to the last lane row on screen; never the legend, detail or footer rows).
- **N2 Paint order decides what sits on top.** Today: grid → links → lanes/bars → completion labels. The line is
  painted after links and before lanes, so grid `│`, link `│`/`╎` and fold `≈` in that column give way to `┃`, while
  bars, `✓`, `◆`, `✓ done` labels and lane text paint over it (a bar that reaches the present visibly plugs into the
  line; text is never cut by it). Row backgrounds (selected, child, lane tint) are kept under `┃`.
- **N3 Label priority.** The now label is placed first, before fold labels, the end clock and ticks, so nothing
  hides it (11e: today the trailing fold label wins and `now` disappears). It is `<clock> now` when that fits, else
  `now`, starting at the now column and shifted left to end inside the axis when it would overflow. When now is the
  axis end it replaces the end-clock label (as today's `<clock> now`). Labels that would overlap it are dropped, as
  overlaps are today. Existing tests that expect `now` to vanish near the end are updated, each named in PROGRESS.
- **N4 No now on screen → today's view.** Panned or windowed so now is outside the axis, or bars not drawn (below 80
  columns or an empty axis): nothing changes. Home is out of scope.
- **L1 Which parts are live** (a pure core function over the existing `GraphReport` plus the refresh's sessions,
  returning part indices; no JSON key changes). A **Codex part** is live when its status is `Working` and it is the
  last part of its Codex session in the step (a run window superseded by a later resume of the same child is never
  live, even with an uncompleted turn). A **Claude part** is live when all hold: it is the last part of its Claude
  session in the step (ties by part index); its session's state is `Busy`; its `to` is within 5 min of `now` (the
  same threshold as Codex `Stale`), so an old step is never live because its session is busy on a newer one; and no
  later part in its chain is a live Codex part (while Codex runs, Claude's planning part keeps collecting reads,
  but the work is Codex's: board 11d dims plan). Evaluated per session, so two parents in one step can each have a
  live part. No commit condition: work resumed after a commit can be live (critique 2), and the 5-min rule already
  ends old steps.
- **L2 Live box.** A live part's box border is heavy (`┏━┓ ┃ ┗━┛`) in its harness colour, bold, and its top border
  carries a `● now` badge in `now` (11d: `┏━━…━ ● now ━┓`). Its content is unchanged (Codex keeps `working now ·
  <length>`).
- **L3 Finished parts recede.** When at least one part is live, every other part's border and `●` use the harness's
  dim colour (`claude_dim`, `codex_dim`; non-text marks, 3:1 is enough). The harness word and all text keep today's
  colours (critique 9: dim colours on `graph_selected` are 3.06:1 and 3.61:1, under the 4.5:1 text rule). Boxes that
  are grey today (unlinked, ambiguous, unavailable) stay grey. Arrows unchanged. No live part → the graph renders
  exactly as today.
- **L4 Live part off screen** (11g). Visibility comes from the renderer's own slot calculation (five slots; a
  same-harness neighbour advances two). When a live part is not drawn, the parts row shows `● now · N later ›` at the
  right edge (or `‹ N earlier · ● now` at the left; both when live parts are on both sides), `● now` in `now`, N =
  parts (not slots) hidden on that side. Selection and keys unchanged.
- **L5 Narrow list form** (below 120 columns, 11i). The live item's title line ends with `● now` in `now`; its width is
  reserved before the title is cut, so it is never clipped. L3 dims the others' `●`. A live item scrolled out of the
  list shows `● now ↑` / `● now ↓` at the right of the first / last visible list row. Two live parts (11f): each gets
  the L2 / L5 marks.
- **L7 Data flow.** The TUI crate has no core dependency and `Refresh` carries overview rows, not sessions. The CLI
  worker, which already holds the all-time sessions when it builds the graph, calls L1 and sends the live indices
  with that graph in `Refresh` (`graph_live`). A stale or rejected graph report drops its live set with it. NO_COLOR:
  live stays visible by the heavy border and `● now`, selection by its existing marker.
- **L6 Liveness is evidence-bound.** Live never comes from a guess: Codex `Working` is the existing status rule
  (last turn not completed, a record within 5 min); Claude `Busy` is the existing session state. Stale and unknown
  are never live.

Non-goals: a legend entry (the 120-col legend has no room; the axis says now); live tips on bars; sorting active
lanes first; a now header line; opening the graph on the live part; the home screen; new JSON keys.

Whole-phase Verify: `cargo test --workspace` → all `ok`; `cargo clippy --workspace --all-targets -- -D warnings`
clean; JSON answer keys unchanged (`cargo test` compares them, and `fixtures/expected/phase9/check.sh 6` passes 47/47);
`fixtures/expected/tui/` snapshots change only where the step names them, and Claude reviews each change by rendered
cells (style-run splitting makes text diffs look wider); W10 scan over committed files.

### Step 11.1 — Repo view now line (Codex)
- Depends on: 10.2. N0–N4.
- Files: `crates/tui/src/{theme,timeline}.rs`, `crates/tui/tests/{theme,timeline}.rs`,
  `fixtures/expected/tui/repo-phase6.{txt,styles.txt}` (only the now column and the axis label row), new
  `fixtures/expected/tui/repo-now.{txt,styles.txt}`, `PROGRESS.md`
- Verify: tests and clippy clean; JSON keys unchanged; new timeline tests on fixture data at a fixed `--now`: fitted
  view → `┃` in `now` on every lane row of the last axis column, `▼` on the rule, label `now`; a bar reaching now keeps
  its glyph and colour in that cell; a link `╎` at that column becomes `┃`; a selected row keeps its background under
  `┃`; a trailing fold → `now` label still shown (11e); unfitted window with now inside → line at `axis.col(now)`;
  window entirely in the past → no `┃` anywhere; 79 columns → no line; NO_COLOR → `┃` and `▼` present, no colour;
  indexed mode → index 176; a `✓ done` label crossing the now column stays whole; now inside a fold → `┃` on the
  fold's last cell; trailing fold with a zero-width end segment → last cell; a near-end interior now → label shifted
  left, still whole; a short screen with lanes scrolled → line on visible lane rows only; `now` passes the theme
  contrast test. `repo-now` is a new snapshot of a fixture lane that reaches now; the `repo-phase6` change touches
  only N1–N3 cells (listed in PROGRESS).

### Step 11.2 — Step graph live box (Codex)
- Depends on: 11.1. L1–L6.
- Files: `crates/core/src/graph.rs` (the pure live-parts function and its unit tests), `crates/core/tests/**`,
  `crates/cli/src/tui.rs` (worker calls L1, fills `Refresh::graph_live`), `crates/cli/tests/**`,
  `crates/tui/src/{graph,app}.rs` (`GraphOptions` gains the live set), `crates/tui/tests/**` (graph tests,
  plus every exhaustive `Refresh` initializer, e.g. `started.rs`; amended 2026-11-10),
  `crates/tui/examples/home_html.rs` (a live variant for screenshots), new
  `fixtures/expected/tui/step-live.{txt,styles.txt}`, `PROGRESS.md`. `step-phase8` (graph-1.1, all parts done) must
  stay unchanged.
- Verify: tests and clippy clean; JSON keys unchanged (`graph --json` identical on every committed key). Core tests:
  Codex `Working` and last of its child → live; `Done`/`Stale`/unavailable → not; an uncompleted fresh turn in an
  old window followed by a resume of the same child → only the newest part can be live; Claude last part of its
  session + `Busy` + `to` within 5 min → live; same with `to` older than 5 min (an old step, parent busy elsewhere),
  with a live Codex part later in its chain, or with session not `Busy` → not; two parents in one step each live;
  A→B→A chains; work resumed after a commit → live. Worker tests: `Refresh::graph_live` follows a step switch and is
  dropped with a rejected stale report. TUI tests (120×40
  and 80×40): one live Codex part → heavy border in codex colour, `● now` badge in `now`, other parts' border/`●`/
  harness word in dim colours and their text unchanged; no live part → buffer identical to today's render of the same
  report; two live parts → both marked; live part beyond the last visible slot → `● now · N later ›` with N counted in
  parts across a same-harness slot gap, `‹ N earlier · ● now` before the first, both at once; 80 columns → list form
  `● now` on the live item with a long title (badge kept), `● now ↓` when scrolled away; 119 ↔ 120 columns resize;
  selection on a live part keeps both marks; NO_COLOR → live and selected each identifiable; grey (unlinked) boxes
  stay grey under L3. `step-live` is a new snapshot with one live Codex part.

### Step 11.3 — Real check, README images, owner confirm (Claude)
- Depends on: 11.2. Real renders (scratch only, W10) of the multi-link repo view and of a step graph through a full
  sandbox transition in `~/multi-link-sandbox/`: Claude plans (Claude part live) → `codex exec` runs (Codex live, plan
  dims) → Claude reviews (review live) → fix round via `resume` (fix live, old run not live) → idle (no live part,
  graph as today). Compared side by side with boards 11b, 11d, 11e. Re-shoot the repo and step images in `docs/img/` from fixtures. README: one line that
  magenta means now. Then stop and ask the owner to look at the real TUI and confirm the need.
- Files: `README.md`, `docs/design.md` (N0–N4, L1–L7 as "Phase 11 decisions"), `docs/img/**`,
  `docs/phase11-check.md` (the real check record, as phases 6/8/9; amended 2026-11-10), `PROGRESS.md`
- Verify: whole-phase Verify; the comparisons recorded in PROGRESS (what matches the board, what differs and why);
  the owner's answer recorded.

### Phase 12 — Settle ambiguous links with evidence (Release 1.9)
Closed 2026-11-10 (owner confirmed after Step 12.2).
Need (design need 1: see which Codex run came from which Claude session, and how sure that is): 9 real hand-offs
are `ambiguous` because two or more Codex runs in the same checkout started within the 60 s creation window. The
TUI says "2 possible runs" and links nothing, though in most of them the answer is plain from the data. Owner chose
this over the other backlog items on 2026-11-10. Phase 9's check explained the 9 (`docs/phase9-check.md`).
Measured 2026-11-10 on real data (`links` / `sessions --since all`, release build at 04b4233; counts only):
- 234 hand-offs: 124 heuristic, 53 exact, 48 unlinked, 9 ambiguous.
- **Calibration on ground truth.** 29 exact launch links (child id observed, not `resume`): the true child is among
  the 60 s candidates in 28; its gap is 0.45–3.18 s in 27 and 6.2 s in one. 4 of the 29 had other candidates
  (17.0–56.8 s); the nearest was the true child in 4 of 4. Small sample, stated as such.
- **Masked check** (hide each of the 29 observed ids, run K1–K3): 28 right, 0 wrong, 1 unlinked (the run outside the
  window). Only 4 of the 29 had rivals, so this says little near the K2 margins: they are provisional.
- **The 9.** Each has a nearest run at 0.49–1.76 s. In 7, every further candidate starts closer to a different
  hand-off (it is that hand-off's nearest run). In 1, the other runs start ≥ 23.8 s after. In 1 (0.91 s vs 7.88 s,
  the 7.88 s run claimed by nobody) the answer is not plain.

Decisions:
- **K0 Contract (a stated revision, not purely additive).** Link `evidence` gains `"nearest_run"`, allowed only on
  a `heuristic` link (like `resume_turn`). `exact` keeps its meaning: an observed child id; nothing here makes a
  link exact. `candidates` now means the runs left after K1 (survivors), sorted by gap: `heuristic` without
  evidence has 1, `heuristic` / `nearest_run` has 1 or more with `child` = `candidates[0]`, `ambiguous` has 2 or
  more, `unlinked` has none. Runs K1 removed are not listed (the schema doc says so); `docs/output-schema.md`
  records the revision.
- **K1 A run that is another hand-off's own nearest goes there.** Competitors are exactly the hand-offs that use
  the 60 s creation window today: not `exact`, not `dir_moved`, no `resume_id`, and not an id-less `resume --last`
  (R7 compares later turns, not creation, so its gaps cannot compete). Exact launches keep reserving their observed
  child (`claimed`, unchanged). Run r is removed from hand-off h when, for some other competitor h', r is h''s
  own choice (K2/K3 applied to h''s original list alone) and gap(h', r) < gap(h, r). Equal gaps, or h' choosing
  nothing or another run, → r stays. *(Amended in 12.0: was "h''s unique nearest"; see Review log.)*
  One pass over the original candidate lists, so hand-off order cannot matter. **An assumption, stated:** Codex
  creates a run within seconds of its launch (27 of 28 ground-truth gaps ≤ 3.2 s). It can be wrong: A launches in
  the background at 0 s and its run appears at 11 s, while B launched at 10 s failed to start one; K1 then gives
  A's run to B. Today both A and B would claim that run (two `heuristic` links to one child), so K1 removes a
  duplicate attribution rather than adding a new risk; the link still says `matched by time`, never confirmed.
- **K2 A clear nearest (provisional margins).** If two or more runs remain after K1, the nearest is chosen when its
  gap is ≤ 5 s and every other remaining run's gap is ≥ 10 s (inclusive, in ms). K1–K3 apply to launch hand-offs
  only; an id-less `resume --last` keeps R7 exactly (12.0 clarification). Why these: 27 of 28 true gaps are
  ≤ 3.2 s, and the 4 ground-truth rivals started ≥ 17 s after; the margins are chosen wider than that, not fitted to
  the 9. Known failure: a manual run at +1 s and a slow true child at +12 s picks the manual run. 12.2 re-runs the
  masked check; any wrong answer there stops the phase.
- **K3 Result.** After K1/K2: one chosen run → `heuristic`, `child` = it; `evidence: "nearest_run"` when the
  original list had more than one run (else `null`, as today). None left → `unlinked`. Otherwise `ambiguous`, as
  today. Every child change flows on as today's heuristic links do: overview reparents the run under its parent,
  and the step graph uses it for run windows, prompts and return edges. Words in the TUI: `matched by time ·
  nearest run` where the sub line says `matched by time` today. The graph `Edge` (handoff) gains `nearest_run:
  true`, omitted when false, so no existing key changes. *(Amended in 12.0: the edge's candidate count is the
  survivor count, so "nearest of N runs" could not be derived from it.)*
- **K4 Changes to today's answers are listed, not hidden.** Expected transitions on today's data (descriptive; the
  data grows, so these are not acceptance targets): 53 exact → unchanged; 48 unlinked → unchanged; 6 heuristic
  `resume_turn` → unchanged; 118 heuristic launch → 117 unchanged, 1 `unlinked` (its only run is a closer hand-off's
  nearest); 9 ambiguous → 8 `heuristic` / `nearest_run` (7 by K1, 1 by K2), 1 ambiguous. Totals 234 → exact 53,
  heuristic 131, unlinked 49, ambiguous 1. Step 12.0 lists every committed key value that changes.

Non-goals: making any of these `exact`; changing the 60 s window or R7's 10 s; the `cd`-then-`resume --last` case;
using session text or prompts to match; iterating K1 to a fixed point.

Whole-phase Verify: `cargo test --workspace` → all `ok`; `cargo clippy --workspace --all-targets -- -D warnings`
clean; every committed answer key equal except the values Step 12.0 lists; W10 scan over committed files.

### Step 12.0 — Rules, fixtures and answer keys (Claude)
- Depends on: 11.3. K0–K4 into `docs/design.md` ("Phase 12 decisions") and `nearest_run` into
  `docs/output-schema.md`. Synthetic fixtures (W10) in `fixtures/sessions-phase12/`, fixed `--now`, and
  Claude-written keys in `fixtures/expected/phase12/` with a `check.sh`, before any code: (a) two hand-offs 40 s apart
  in one checkout, each with a run 1 s after it (each gets its own run by K1); (b) one hand-off with runs at 1 s and
  30 s, nobody else (K2 → nearest); (c) runs at 1 s and 7 s (stays ambiguous); (d) runs at 6 s and 30 s (stays
  ambiguous: nearest over 5 s); (e) a hand-off whose only run started closer to another hand-off (→ `unlinked`);
  (f1) two hand-offs at the same instant, one run 1 s after (kept by both: both `heuristic` to the same child, as
  today); (f2) the same with two runs at 1 s and 30 s (both pick the 1 s run by K2); (g) an exact launch beside a
  heuristic one (claimed run never a candidate); (h) an id-less `resume --last` and an option-first explicit-id
  resume beside launches (both unchanged); (i) a competitor that stays ambiguous itself (its rival runs are not its
  unique nearest, so nothing is removed elsewhere); (j) K2 boundaries at 4999/5000/5001 ms and 9999/10000/10001 ms;
  (k) a `dir_moved` hand-off (no candidates, never a competitor). Keys: `links.json`, plus `overview.json` (a run
  reparented by K1) and one `graph-*.json` (its run window and return edge), all with one fixed `--now`; a
  `crates/cli/tests/phase12.rs` compares them in `cargo test`. Existing keys that change are listed value by value.
- Files: `docs/design.md`, `docs/output-schema.md`, `fixtures/sessions-phase12/**`, `fixtures/expected/phase12/**`,
  `crates/cli/tests/phase12.rs`, `fixtures/expected/**` (only the listed values), `crates/core/tests/keys-before-phase6.json`
  (the same listed value; amended in 12.0), `PROGRESS.md`
- Verify: `cargo test --workspace` → all `ok` except `phase12` tests and tests comparing changed keys, listed by
  name in PROGRESS (all pass by 12.1); `fixtures/expected/phase12/check.sh` runs (`cargo run`, fixed `--now`, `jq -S`
  comparison) and fails only on the new cases; W10 scan: `git grep -n -E '/Users/|jarednarvaez' -- fixtures docs`
  finds nothing new.

### Step 12.1 — K1–K3 in the link resolver (Codex)
- Depends on: 12.0. K0–K3.
- Files: `crates/model/src/lib.rs` (`LinkEvidence::NearestRun`), `crates/core/src/link.rs` and its unit tests,
  `crates/core/src/graph.rs` (`Edge` `nearest_run`, amended in 12.0), `crates/core/tests/**`, `crates/model/tests/**`
  (Edge constructors; amended in 12.1),
  `crates/tui/src/{view,timeline,graph}.rs` (the `nearest run` words only),
  `crates/tui/tests/**`, `crates/cli/tests/**`, `PROGRESS.md`
- Verify: tests and clippy clean (`phase12` tests compare every Phase 12 key); `fixtures/expected/phase12/check.sh`
  all `ok`; `fixtures/expected/phase9/check.sh 6` 47/47; `git diff 04b4233 -- fixtures/expected` shows only 12.0's
  keys; unit tests: K1 (strictly smaller gap and the other's own choice only, ties kept, result independent of hand-off
  order, other checkouts ignored), K2 (boundaries as written), K3 (evidence only with more than one original
  candidate), exact and R7 links unchanged; the longer words survive the lane's truncation at 120 and 80 columns.
  Real run in scratch, counts only: transitions against K4's table.

### Step 12.2 — Real check, README, owner confirm (Claude)
- Depends on: 12.1. Real counts before/after by K1/K2, each of the 9 described by gaps only (W10), and the one
  heuristic → unlinked case. Re-run the masked check on every exact launch link with the built resolver (scratch
  harness); any wrong answer stops the phase for the owner. README: the "Ambiguous links" limitation rewritten (what still stays ambiguous and why).
  Then stop and ask the owner to look at the repo view's links and confirm the need.
- Files: `docs/phase12-check.md`, `README.md`, `PROGRESS.md`
- Verify: whole-phase Verify; real counts match K4 or each difference is explained; the owner's answer recorded.

### Phase 13 — Known folders and a key that is never lost (Release 1.10)
Closed 2026-11-10 (owner confirmed after Step 13.3).
Need (design need 1: see which Codex run came from which Claude session; and the TUI must answer every key). Three
backlog items, chosen together by the owner on 2026-11-10. Measured 2026-11-10 on real data (release build at
8d2cc22; scratch, counts only; tool calls read from `assistant` records only, since `api-request-blob` copies can
differ):
- **`cd` before the launch.** 35 hand-offs run `cd <dir>` before `codex exec` and are `unlinked` by the `dir_moved`
  rule: 33 `exec`, 2 `resume --last`. In 31 the target is a literal path (no `$`, no `~`); each of the 29 literal
  `exec` ones has exactly one Codex run whose `cwd` is the resolved target, starting 0–60 s after the hand-off.
  Targets: 9 the session's own folder, 2 inside it, 22 elsewhere (before excluding the 4 non-literal).
- **Option-first explicit id.** `codex exec resume -c k=v <id> "…"` is read as a plain `exec` (found in 12.0, fixture
  case h); the id grammar requires the id right after `resume`.
- **Lost key on resize.** `tools/resize-key-check.py` (new, synthetic fixture) blocks the TUI in a write, resizes,
  and sends `q` with one more resize: 5 of 5 rounds FAIL at 8d2cc22 (the key waits for a second key). Cause
  (Phase 3): crossterm 0.28.1's mio source returns `Resize` when SIGWINCH and tty readiness arrive in one wake and
  drops the edge-triggered tty readiness. Its `use-dev-tty` source polls with `poll(2)` (level-triggered) and reads
  the tty before the resize pipe. A throwaway build with that feature passed 5 of 5 (reverted; not committed).

Decisions:
- **C1 A literal `cd` names the launch folder.** A `cd` segment before the launch, in the same command, moves the
  known folder when all of these hold, else the folder is unknown (`dir_moved`, as today):
  - its one argument is literal: after quote removal only `A–Z a–z 0–9 . _ / + @ % : , = -` and spaces, not
    starting with `-`, and absolute or starting `./` / `../` (or exactly `.` / `..`). Plain names (`cd sub`) stay
    unknown because `CDPATH` can redirect them; `$`, `~`, backticks, globs and braces are not literal.
  - it is a bare `cd` (no wrapper such as `env`/`nohup`/`timeout` before it, which would run it in a child).
  - the operator before it is the start, `;`, a newline, `&&` or `(`; the operator after it is `&&`, `;` or a
    newline (`||`, `|`, `&` → unknown).
  - between it and the launch there is no builtin that can change the shell's folder (`cd` that is not itself C1,
    `pushd`, `popd`, `eval`, `source`, `.`, `exec`, `builtin`, `command`), and no `)` closing a group that was open
    at the `cd`. Groups opened and closed after it (`( … )`, `$( … )`) do not matter.
  Absolute → that path; relative → joined to the folder so far (start: the record's `cwd`); lexical normalisation
  only (`.`, `..`, repeated and trailing `/`); no file-system access, no symlinks. A launch `-C` / `--cd` other than
  `.` / `$PWD` still moves (U1). Why: external commands and `$( … )` run in child processes and cannot change the
  shell's folder; only the listed builtins can. **Assumption, stated:** after `;` the `cd` is assumed to have
  succeeded; if it failed, the run is in the old folder and the resolved folder usually has no run (→ `unlinked`).
  Measured: these rules keep all 31 literal real cases (29 `exec` with exactly one run, 2 `resume --last`).
- **C2 Matching uses the launch folder.** `Invocation` gains the resolved folder; link `checkout` keeps the session's
  folder (where Claude was), and Link gains `launch_dir` (string, only when C1 resolved a folder different from
  `checkout`; omitted otherwise, so no existing key changes). Creation candidates (60 s), K1's competitor grouping
  (today keyed by `checkout`, `link.rs` K1 map) and R7's later-turn search all use `launch_dir` when present. Exact evidence and the `claimed`
  rule are unchanged. Redirect rules are unchanged (a relative redirect after a `cd` still gives no evidence).
- **C3 Option-first explicit id.** After `resume` (in `codex exec [opts] resume` and `codex resume`), read options
  before the first positional word: a value option from the existing table takes the next word; `--name=value`
  takes none; a known flag (`--last`, `--json`, `--skip-git-repo-check`, `--full-auto`,
  `--dangerously-bypass-approvals-and-sandbox`, `--ephemeral`, `--all`, `--oss`) takes none. `--last` anywhere among
  them → id-less (R7, unchanged). Any unknown option, `--`, `-` or a redirect before the first positional → no id
  (today's `exec`), because the option might take the uuid as its value. Otherwise a uuid-shaped first positional is
  the resume id (`resume_arg`, exact). The existing `resume <uuid> …` form is unchanged (out of scope).
- **D1 Read keys with crossterm's `use-dev-tty` source.** Workspace `crossterm = { version = "0.28.1", features =
  ["use-dev-tty"] }` (adds the `filedescriptor` crate). No TUI code change. Why: the upstream fix is not released
  for 0.28; a vendored fork costs maintenance; the feature is crossterm's own and fixes the case in the check.

Non-goals: expanding `~` or variables; `pushd`; following symlinks; trusting relative redirects after `cd`; changing
K1–K3 or R7's window; upgrading ratatui or crossterm's major version.

Whole-phase Verify: `cargo test --workspace` → all `ok`; clippy `-D warnings` and `fmt --check` clean; committed keys
equal except 13.0's list; `python3 -I tools/resize-key-check.py target/release/multi-link 5` → 5 of 5 `pass`; W10 scan.

### Step 13.0 — Rules, fixtures, keys and the resize check (Claude)
- Depends on: 12.2. C1–C3 and D1 into `docs/design.md` ("Phase 13 decisions") and `docs/output-schema.md` (Link
  `launch_dir`, Heuristic, Invocation forms). Synthetic fixtures (W10) in `fixtures/sessions-phase13/`, fixed `--now`,
  keys in `fixtures/expected/phase13/` (`links.json`, `check.sh`) and `crates/cli/tests/phase13.rs`, before code:
  (a) `cd /abs && codex exec` to another folder with one run there (→ `heuristic`, `launch_dir`); (b) `cd ./sub && …`
  (resolved under the record's cwd); (c) `cd ../x/./y/ && …` (normalised); (d) two `cd` segments chained, and
  `cd /x; export A=1; codex exec`; (e) `cd $HOME/x`, `cd ~/x`, `cd -`, `cd sub`, `pushd /x` (unknown, `unlinked` as
  today); (f) `(cd /x); codex exec` (closed group → unknown) beside `(cd /x && codex exec …)` and
  `cd /x && S=$(date +%s) && codex exec` (resolved); (f2) `cd /x || codex exec`, `cd /x | codex exec`,
  `cd /x & codex exec`, `false || cd /x && codex exec`, `cd /x && eval true && codex exec`, `env cd /x && codex
  exec`, `cd /repo/{a,b} && …` (all unknown), each with a tempting run in the named folder; (g) `cd /x && codex exec
  -C /y` (moves, unknown) and `cd /x && codex exec -C "$PWD"` (resolved); (h) `cd /x && codex exec resume --last`
  with a later turn in x's run (→ `heuristic` / `resume_turn`, `launch_dir`); (i) two hand-offs from different
  session folders whose launch folders are the same, unequal gaps, in both invocation orders (K1 groups by launch
  folder); (j) option-first `codex exec resume -c k=v <id>`, `codex exec -s x resume --json <id> "p"`, `codex resume
  --oss <id>` (→ `exact` / `resume_arg`); `codex exec resume -c k=v --last <id>` (R7); `codex exec resume --unknown
  <id>` and `codex exec resume -- <id>` (stay `exec`). Phase 12 case (h)'s
  option-first resume becomes `exact` / `resume_arg`: listed. `tools/resize-key-check.py` committed (5/5 FAIL today).
- Files: `docs/design.md`, `docs/output-schema.md`, `fixtures/sessions-phase13/**`, `fixtures/expected/phase13/**`,
  `crates/cli/tests/phase13.rs`, `fixtures/expected/phase12/links.json` (the listed value), `tools/resize-key-check.py`,
  `PROGRESS.md`
- Verify: `cargo test --workspace` → all `ok` except the `phase13` tests and `phase12_links_match_answer_key` (listed);
  `fixtures/expected/phase13/check.sh` fails only on the new cases; the resize check 5 of 5 FAIL at 8d2cc22 (recorded);
  W10 scan of the new files finds nothing.

### Step 13.1 — Launch folder and option-first id in the resolver (Codex)
- Depends on: 13.0. C1–C3.
- Files: `crates/claude/src/invocation.rs` and `crates/claude/tests/**`, `crates/model/src/lib.rs` (Link
  `launch_dir`), `crates/model/tests/**`, `crates/core/src/link.rs` (with its unit tests,
  which build `Invocation`), `crates/core/src/overview.rs` (test helper building `Link`), `crates/core/tests/**`, `crates/cli/tests/**`, `PROGRESS.md`
- Verify: tests, clippy, fmt clean; `fixtures/expected/phase13/check.sh` and `phase12/check.sh` all `ok`
  (`phase13` includes every C1 negative form with a tempting run);
  `fixtures/expected/phase9/check.sh 6` 47/47; `git diff 8d2cc22 -- fixtures/expected` = 13.0's keys only; unit tests
  for C1 (each literal form, each unknown form, closed group, normalisation) and C3 (option-first id, `--last` wins,
  non-uuid stays `exec`); existing recognition tests changed only where C1 resolves a folder, each named in PROGRESS.

### Step 13.2 — Keys read with `use-dev-tty` (Codex)
- Depends on: 13.0 (the check). D1.
- Files: `Cargo.toml`, `Cargo.lock`, `PROGRESS.md`
- Verify: tests, clippy, fmt clean; every TUI snapshot unchanged; `cargo build --release -p multi-link` then
  `python3 -I tools/resize-key-check.py target/release/multi-link 5` → 5 of 5 `pass`.

### Step 13.3 — Real check, README, owner confirm (Claude)
- Depends on: 13.1, 13.2. Real counts before/after (the 35 `cd` hand-offs by outcome, option-first resumes found), a
  masked check of `cd` links where an id was observed (if any), the resize check on the release build and a real TUI
  smoke run (PTY harness, scratch). README: remove the lost-key limitation, describe `cd` handling and its limits.
- Files: `docs/phase13-check.md`, `README.md`, `PROGRESS.md`
- Verify: whole-phase Verify; real counts explained; the owner's answer recorded.

### Phase 14 — Live grid: what is each agent doing now? (Release 1.11)
Closed 2026-11-11 (owner confirmed after Step 14.4; the cold first pass misses the 2 s NFR, disclosed in
`docs/phase14-check.md`).
Need (owner, 2026-11-10): "see what each agent is doing right now (its last few actions, its state, how long it has
been at it) without switching terminal tabs." Design needs 1, 2 and 5; FR12, screen 5. Home answers "which repo needs
me"; the repo view and step graph answer "what happened". Neither shows, for every live session at once, what it is
doing. Board 14 (canvas R5Bh9iX6bsn3jSu8F8D58C, boards 14a–14b) was approved by the owner on 2026-11-10. 14a is real
data (8 Nov 2:19pm, read by a scratch script, not multi-link's code); 14b is made up, to show waiting and stale cards.
Owner choices (2026-11-10): its own screen; cards only for working, waiting or stale sessions. How it opens was left
to Claude: `l` plus `tui --live` (the other options, a size-based default or a remembered screen, hide a rule or need
saved state). Builds on the 2026-11-08 proposal (`.claude/handoff/proposal-live-grid-and-pane-jump.md`, L0–L9),
revised by board 14: rows pair a Claude card with its live Codex card, and each card shows its last 3 notable calls.
Comparables (board 14 notes): agtrace (hand-off feed), c9watch (needs-you first), Herdr's sidebar (one state per agent),
k9s (enter drills in, esc back, refresh in place); htop's flaw (cursor jumps on re-sort) avoided.

Decisions:
- **L0 A report, then a screen.** New `multi_link_core::live` builds a `LiveReport` (model crate) from the same
  sessions, links and overview pass the TUI already runs, plus G5 calls for card sessions only. `multi-link live
  [--root] [--now] [--since] [--json]` prints it; plain text is a list (one line per card, then the feed). No existing
  JSON key changes. The TUI only draws the report (as T2 for home). Why: answer-keyed like every other view, and the
  desktop app (FR15) can reuse it.
- **L1 Which cards.** Every non-imported session, top-level or child, whose state is `waiting`, `busy` or `stale`
  (home's `working now` is `busy`, both harnesses), taken from the same sessions pass the TUI runs, so its window rules
  apply (a stale Codex run last active before the window is history, not a card; stated). All other sessions with
  activity in the window are counted on one
  line: `N idle sessions (repo · repo) · home shows them` (distinct repo names, sorted, cut with `…` to the width).
  No cards and nothing idle: `nothing working right now` plus, if any, `· last hand-off T`.
- **L2 Groups, rows and moving.** A Codex card's parent is the Claude session of its latest incoming non-ambiguous
  link by `at` (ties: the later one in the links report); no `started_by` fallback (core sets it for Claude only); the
  parent may be in another repo. A group is a Claude card plus the live Codex cards whose parent it is (by link `at`,
  ties by full id), or a lone card (a Codex card whose parent is not a card or is unknown; a Claude card with no live
  child). Groups are ordered by their head: waiting first (oldest wait start), then busy (oldest last-burst start),
  then stale (oldest stale start), ties by full id; wait/stale start = `state_since`, else `last_activity` (Codex busy
  and stale never have `state_since`). The flattened card order is group order, head then children. Wide layout: the
  head on the left (x 1, 56 wide), its first child on the right (x 61, 58 wide) joined by `──▸` on the card's 5th row;
  each further child takes the next row, right side, left blank, joined by `└──▸` from x 57; a lone Codex card sits on
  the left. **The selection is kept by full session id** across refreshes; if that session leaves, it goes to the same
  flattened index (clamped to the last). `↑↓` move to the previous / next group's head; `←→` move along the flattened
  order within the group. Narrow layout (L7): `↑↓` and `←→` both move along the flattened order.
- **L3 Card (10 rows).** Border in the harness colour; title in the top border: state glyph (`●` busy, `◆` waiting,
  `○` stale), harness word bold, `· <repo>`, `· step <id>` when the session has a latest tagged hand-off step
  (Claude: its own latest tagged hand-off; Codex: the `codex_steps` rule, moved to core in 14.1). Inner rows:
  (1) `“last_asked”` (P1/P5), both harnesses (G3 part kinds need chain position the grid does not compute); (2) W5
  phrase: `working now · for N min` from the last burst's start (`for over <period>` when that burst starts at the
  window start, i.e. was clipped; `working now` with no burst), `waiting on you · since T (N min)`, `stale · no activity
  since T (N min)` with L2's wait/stale start; (3) G6 counts for the whole session
  (`N reads · N edits · N runs`, zeros left out, hand-offs not counted); (4) `┄` rule; (5–7) the last 3 notable calls,
  from a shared summary extracted from `graph::tally` that keeps each label's latest `at` and `ok` (graph JSON
  unchanged); every recognised hand-off is notable as `▸ codex exec`, plus ` · step X` when tagged (no chain words);
  newest last, each `T  label` with G6 marks; empty rows when fewer; (8) the link line: Claude with a live child →
  `waiting on codex <id8> · N min` (since that link's `at`); else its latest hand-off in the window, with L4's outcome
  → `codex <id8> done T` / `N possible runs` / `no run found` / `codex <id8>` (outcome unknown); else blank. Codex → `└ from claude <id8> · confirmed link|matched by time`, or `not linked to any claude session`.
  Bottom border: `<place> · <id8>` (place as the repo view: `main`, `<name> worktree`). Text is cut at the card's inner
  width with `…`. Why: answers the need in one glance; fixed labels cannot leak commands (G6).
- **L4 Feed.** `HAND-OFFS newest first · all repos`, then up to 4 rows: hand-offs whose `at` is in the window, newest
  first, all repos, ambiguous included. Row: `T  <repo, 14 wide>  claude handed step X to codex` (`work` when
  untagged; `resumed codex for` instead of `handed … to codex` for a resume form), W8 confidence words at x 75
  (`confirmed link`, `matched by time`, `N possible runs`), outcome at x 93 from the child's turns (G4): `codex running`
  when the child is busy and this is its latest incoming link; `codex done T` when a turn starting at or after `at`
  completed (T = that completion; the first such turn); blank otherwise (unlinked, ambiguous, no rollout, or a later
  link superseded it while no turn completed). No hand-offs:
  `no hand-offs in the last <period>`.
- **L5 Header and footer.** Row 0: `multi-link › live  what every agent is doing now`, right: `◆ N waiting · ● N
  working now · ○ N stale   T` (a zero count drawn grey, glyph kept). Bottom: `selected` line (harness, id8, repo and
  place, started, `active about N min in K stretches`), `asked` line, legend (`● working ◆ waiting on you ○ stale │
  confirmed link ╎ matched by time ✓ exit 0 ✗ exit non-zero`), keys row and `times UTC-8`, as board 14a.
- **L6 Keys.** `l` opens the grid from home and the repo view (their footers gain `l live`); `multi-link tui --live`
  starts on it. `esc` returns to the screen that opened it (`--live`: home). `enter` opens the selected card's repo
  view; `s` its step graph (the card's step, G9 rules); `o`/`f` as FR18; `q` quits. Refresh every pass, in place.
- **L7 Sizes.** Width ≥ 120: board 14a. Width < 120: one column; a Codex card sits under its parent, indented 4,
  joined by `└▸`; cards take the full width minus the indent. Narrower still: the same one-column form, text cut
  with `…` (no other screen has a too-small message today, so none is added). Rows: the feed drops to 2 rows below 40
  rows and to none below 32; cards that do not fit leave one line `↓ N more` and the view scrolls to keep the
  selection visible; when not even one card fits, the selected card is drawn clipped at the bottom edge and the
  `↓ N more` line wins the last row. All cutting is by display width (wide and combining characters), never bytes.
- **L8 Colour.** Existing theme roles only: harness colours for borders and words, `needs_you` for `◆` and the
  waiting border, the dim harness colour for stale borders, `busy_card` / `codex_box` / `waiting_card` backgrounds,
  heavy border plus `selected` background for the selection, `done`/`warning` for `✓`/`✗`. NO_COLOR keeps every glyph
  and word. No new colour.
- **L9 Speed.** The live report is built only while the grid is open (`Refresh` gains `live: Option<LiveReport>`, as
  `graph`). Calls are read only for card sessions and cached in the CLI worker per session, keyed by its dependency
  set: (path, length, mtime, inode) of the main transcript and of every subagent file the scanner lists for it (as
  T1), so an added, removed, grown or replaced file re-reads that session and nothing else; clock-only state changes
  re-read nothing. The whole refresh with the grid open stays within the 2 s NFR on real `~` (measured in 14.4).
- **L10 Privacy.** Fixtures synthetic (W10); real renders stay in scratch; README images from fixtures only.

Non-goals: a terminal preview or sending keys (control is a design non-goal); state-change history in the feed (needs
saved state); idle sessions as cards; a per-repo grid; the tmux / Herdr pane jump (backlog); changing home, the repo
view or the step graph beyond the `l` key and its footer word; a new colour.

Whole-phase Verify: `cargo test --workspace` → all `ok`; clippy `-D warnings` and `fmt --check` clean; committed keys
equal except 14.0's list (home/repo footers); `multi-link live --root fixtures/sessions-phase14 --now <t> --json`
equals `fixtures/expected/phase14/live.json`; the fixture render equals the 14.0 goldens cell for cell (120×40, 100×40,
120×30, 60×20); `fixtures/expected/phase9/check.sh 6`, `phase12/check.sh`, `phase13/check.sh`, `phase14/check.sh` all
`ok`; W10 scan.

### Step 14.0 — Rules, fixtures, keys and goldens (Claude)
- Depends on: 13.3. L0–L10 into `docs/design.md` ("Phase 14 decisions", screen 5 row moves from Backlog) and
  `docs/output-schema.md` (`live` report). Synthetic fixture root `fixtures/sessions-phase14/` with a fixed `--now`,
  shaped like boards 14a + 14b: two Claude sessions each with a live, linked Codex child (one confirmed, one matched by
  time), one waiting Claude with a finished child, one stale Codex with no parent, one Claude with two live children,
  two idle sessions in two repos, one imported Codex (hidden), one ambiguous hand-off in the feed, and calls that
  exercise G6 merge and marks (a repeat label, `✗`, a label-less run). Keys: `fixtures/expected/phase14/live.json`,
  `check.sh`, and hand-drawn goldens `fixtures/expected/tui/live-phase14{,-100,-short}.{txt,styles.txt}` checked
  region by region against board 14a, plus `live-phase14-tiny` (60×20). Extra cases (critique): a Codex child with
  two incoming links (latest wins) and two at the same `at`; a parent in another repo; a Claude card with three live
  children; a busy burst that crosses the window start (`for over 2h`); a resumed child whose earlier hand-off must not
  say `running`; a hand-off with no rollout; a label repeated with a changed `ok`; a wide-character prompt.
  Home and repo footer goldens are not touched here (14.3 changes them with the code).
- Files: `docs/design.md`, `docs/output-schema.md`, `fixtures/sessions-phase14/**`, `fixtures/expected/phase14/**`,
  `fixtures/expected/tui/live-phase14*`, `PROGRESS.md`
- Verify: `cargo test --workspace` → all `ok` (nothing reads the new files yet); `fixtures/expected/phase14/check.sh`
  fails only because `multi-link live` does not exist yet; W10 scan of the new files finds nothing.

### Step 14.1 — `core::live` and `multi-link live` (Codex)
- Depends on: 14.0. L0–L5 data (cards, groups, order, feed, counts, idle line); calls via the existing G5 readers;
  the G6 summary extracted from `graph::tally` with each label's latest `at` (graph JSON unchanged); the
  `codex_steps` / `started_steps` rules moved from `crates/cli/src/tui.rs` into core with their tests and tie rules,
  the CLI calling the core versions.
- Files: `crates/model/src/lib.rs` (LiveReport), `crates/model/tests/**`, `crates/core/src/live.rs` (new),
  `crates/core/src/lib.rs`, `crates/core/src/graph.rs` (the shared summary), `crates/core/tests/**`,
  `crates/cli/src/main.rs`, `crates/cli/src/tui.rs` (use the moved step maps only), `crates/cli/tests/**`, `PROGRESS.md`
- Verify: tests, clippy, fmt clean; `fixtures/expected/phase14/check.sh` → `live.json` equal;
  `fixtures/expected/phase9/check.sh 6` 47/47, `phase12/check.sh` and `phase13/check.sh` all `ok`; `multi-link graph`
  output for the phase 8 fixture unchanged; unit tests for L1 (each state, imported hidden, stale outside the window),
  L2 (parent precedence and ties, cross-repo parent, three children, lone cards, order keys incl. Codex fallbacks),
  L3 phrases and link lines, L4 outcomes (resumed child, no rollout); `git diff 67cdabd -- fixtures/expected`
  = 14.0's keys only.

### Step 14.2 — TUI render to board 14 (Codex)
- Depends on: 14.1. L3–L5, L7, L8 drawing; pure render from `LiveReport` plus a selection and scroll.
- Files: `crates/tui/src/live.rs` (new), `crates/tui/src/lib.rs`, `crates/tui/src/theme.rs` (only if a role needs a
  helper; no new colour), `crates/tui/tests/**`, `crates/tui/examples/home_html.rs` (live variant for README images),
  `PROGRESS.md`
- Verify: tests, clippy, fmt clean; the fixture render equals `live-phase14.txt`/`.styles.txt` cell for cell at
  120×40, `-100` at 100×40, `-short` at 120×30, `-tiny` at 60×20; a wide-character prompt is cut by display width;
  NO_COLOR render keeps every glyph; every existing snapshot unchanged (home/repo footers change only in 14.3).

### Step 14.3 — Wiring: `l`, `--live`, keys, refresh, call cache (Codex)
- Depends on: 14.2. L6, L9.
- Files: `crates/tui/src/app.rs`, `crates/cli/src/tui.rs`, `crates/cli/src/main.rs` (`--live`), `crates/tui/src/home.rs`
  and `crates/tui/src/timeline.rs` (footer word only), the home/repo goldens under `fixtures/expected/tui/` whose footer
  gains `l live` (each listed in PROGRESS), `crates/tui/tests/**`, `crates/cli/tests/**`, `crates/core/src/live.rs` and
  `crates/core/src/scanner.rs` (the call cache, amendment below), `crates/core/tests/**`, `PROGRESS.md`
- Amendment (2026-11-10, Codex stopped on scope): the CLI has no reader crates and `Scanner::claude_transcripts` is
  core-private, so the L9 call cache lives in the `Scanner` the CLI worker owns (one per worker, as every other
  cache): `Scanner::live` keeps per-session calls (and Codex turns) keyed by the L9 dependency set and re-reads only
  sessions whose set changed; a test-only counter of reads proves it. No new public reader API, no Cargo change.
- Verify: tests, clippy, fmt clean; app tests: `l` from home and repo view, `esc` back to each, `--live` start and
  `esc` → home, `enter`/`s`/`o` on a card, `↑↓←→` across a two-child row; the selection kept by id across a refresh
  that adds a card above it, and moved to the same flattened index when its session leaves (either column); moving
  in the narrow layout and after a resize; the call cache re-reads a session when its main file or a subagent file
  grows, is added, removed or replaced (inode), and nothing on a clock-only refresh (tests); the changed home/repo
  goldens differ from 67cdabd only in the footer row (diff shown in PROGRESS).

### Step 14.4 — Real check, README, owner confirm (Claude)
- Depends on: 14.3. On real `~` (release build): refresh time with the grid open, cold (first pass) and warm (median
  of 10 passes), and one pass after a card's transcript grows; each ≤ 2 s;
  card counts by state and a PTY smoke run (open with `l`, move, `esc`, `--live`, resize, `q`), scratch only.
  README: the live grid section, its keys, an image from the fixture render, limits (fixed labels hide plain
  reads; Codex part kinds only for tagged hand-offs). `docs/phase14-check.md` with counts only.
- Files: `docs/phase14-check.md`, `README.md`, `docs/img/**`, `PROGRESS.md`
- Verify: whole-phase Verify; real counts and timing recorded; the owner's answer recorded.

### Phase 15 — A fast first load with a school of fish (Release 1.12)
Need (owner, 2026-11-11): "when I open multi-link it should be ready fast, and while it loads it should be fun to
look at." Design NFR "Speed": every refresh ≤ 2 s. Today the cold first pass takes 8.8–9.4 s (`docs/phase14-check.md`;
re-measured 2026-11-11: `multi-link overview --json` real 8.79 s, user 7.10 s, sys 0.77 s) while reading every file
with `cat` takes 0.38 s, so the wait is CPU, not disk. A 7 s `sample` of that run: 87 % in `Scanner::read_claude`,
most of it `multi_link_claude::parse_file`, about 12 % inside `read_claude` itself (cache-value clones). The machine
has 14 cores and parsing runs on one. The screen during the wait is one footer line, `loading sessions… · q quit`.
Board 15 (claude.ai artifact 7i51gqe1AbVGFurwnkfjqM, source `docs/boards/board15-loading.html`) went through three
rounds; the owner picked **E · School** on 2026-11-11 ("lets just go for E, school for now"): round 1 was judged too
colourful, so it uses only Claude orange and Codex blue on the dark ground. Comparables (board notes): the owner's
reference post of ASCII sea animals (dense glyph clusters, sparse trails), demoscene plasma (too loud, rejected in
round 1), `cargo`'s plain progress line (honest counts, no art).
The tmux / Herdr jump was dropped from this phase (owner, 2026-11-11): neither is installed here and the owner's
sessions run in the Claude and Codex desktop apps, so it would have no real user and no real-data check; it stays in
the backlog as an optional feature that never asks a user to install anything.

Decisions:
- **F1 Parse in parallel, same results.** At the start of `Scanner::read` (both harnesses at once), list every
  Claude transcript and subagent file, every Claude desktop-app record (15.1 amendment) and every Codex rollout with
  its stamp, take the cache misses, and parse them on
  `std::thread::available_parallelism()` scoped threads (`std::thread::scope`, no new crate). Each result is stored
  with the stamp taken **before** its parse. The existing serial loops then run as today through `Cache::read`, which
  already re-parses when the stamp no longer matches (a file appended or replaced in between) — so ordering, sorting,
  skips and error handling are unchanged. A parse that fails in the parallel pass is not cached and its partial
  diagnostics are dropped; the serial loop re-parses it and is the only place any diagnostic is reported, so each
  appears exactly once and a failed parse never revives an old value (the cache entry is removed first, as today).
  A file deleted between listing and parsing is skipped by the parallel pass and handled by the serial loop as today.
  Why: the hot spot is parallel per file; keeping the serial loop as the only place that assembles results means no
  output can change order, which the committed answer keys (fixed clock and probe) prove.
- **F2 Clones only if they still cost.** After F1, 15.1 measures the serial remainder of a cold pass (and the warm
  pass). If cache-hit clones in `read_claude` / `read_codex` still take over 0.3 s cold or over 50 ms warm, the cache
  holds values behind `Arc` and assembly clones only the fields it moves out (transcript, invocations, skips), borrowing
  the rest; otherwise nothing changes. Only inside `crates/core/src/scanner.rs`; no public type changes. Why: the
  12 % clone share was measured single-threaded; whether it matters after F1 is a measurement, not a guess.
- **F3 Budgets and the fallback.** On real `~`, release build, median of 3 fresh processes: (a) cold `multi-link
  overview --json` ≤ 2.0 s wall; (b) the TUI worker's first report — `refresh_with_live` as the worker runs it,
  including the all-time pass, once without and once with `--live` — ≤ 2.0 s; (c) first home frame
  = max(first report, 1.0 s) + 0.25 s fade ≤ 2.25 s; (d) warm refresh median of 10 passes ≤ 2 s and no more than 20 %
  above 14.4's 191–212 ms. A scratch harness (as `docs/phase14-check.md`) measures (b)–(d); the 15.0 baseline records
  (a) and (b) before any change. If 15.1 misses (a) or (b), Claude stops and plans a disk cache step (15.1a) instead
  of pushing on; a disk cache is not built otherwise (invalidation and format-version risk for no gain).
- **F4 Honest progress.** A thread-safe progress handle (atomics) created by the CLI worker and given to its
  `Scanner` with a setter that survives the root reset in `Scanner::read`; the worker sends a clone to the UI thread
  before its first pass. One generation covers the first load: `files_total` is published once, after F1 has listed
  both harnesses (so it never grows), `files_done` counts parsed misses and only rises, and the phase goes
  `listing → reading → linking → done` (done when the worker sends its first report, after the all-time pass). Zero
  misses goes straight to `linking`. Later passes never touch it. The counter on screen is these numbers, never a
  timer.
- **F5 The loading screen (board 15 E).** Timeline from start s: it shows until both the first report has arrived
  and s + 1.0 s, then fades (F8), then home. A first refresh **error** or a disconnected worker ends it at once (no
  fade) and shows the existing error footer. Any key except `q` skips it, consumed once: to the existing
  `loading sessions…` state if no report yet, else straight to home. `q` and Ctrl-C quit as everywhere (the CLI's
  existing terminal guard). No new timer: the CLI loop already draws every iteration with a 50 ms input poll, and a
  frame is drawn from the current clock. Layout at width W, height H (76×22 is the board): art rows `0..=H-8`, title
  `m u l t i · l i n k` on row H-7, a 30-cell bar (`━` done, `─` to do) on row H-5, the counter on row H-3
  (`reading transcripts  N / T`, `linking sessions…`, then `ready ✓  opening home`), `any key skips · q quits` on row
  H-2, all centred by display width. The art is board 15 E's `drawC(t)` (called directly — the board draws it at
  `t + 1`, so the board's paused frame is `drawC(4.0)`), with centre (W/2, 0.48 × art rows) and the board's x radii scaled
  by W/76 and y radii by (art rows)/15:
  96 fish in two schools (fish i: school i mod 2, phase i·2.399, radius 0.55 + ((7i) mod 10)/22, speed
  0.9 + ((3i) mod 7)/20); school 0 (Claude orange) circles an ellipse, school 1 (Codex blue) swims a figure-eight
  through it in the other direction; head `>`/`<` (`›`/`‹` when far), tail `=`/`-`, a `·` when near; brightness from
  depth; where fish of both schools meet one cell, a pale `*`; 46 drifting specks `.`/`·` in the dim harness
  colours. The art is a pure function of (time, width, height, theme) so frames are testable.
- **F6 Colour, quantised.** Existing theme roles only, so the board's continuous shades become steps: a fish with
  depth > 0.66 uses its school's role bold, 0.33–0.66 the role, < 0.33 the `_dim` role; tails one step dimmer; specks
  `_dim`; the meet `*` the normal text colour bold; title letters `m u l t i` `claude`, `l i n k` `codex`, `·` text;
  bar done cells: first half `claude`, second half `codex`, to-do cells `border`; counter text, hint dim text. No new
  colour. 256-colour terminals use the roles' existing 256 values; `NO_COLOR` keeps every glyph and drops every colour
  and the dim/bold modifiers.
- **F7 Sizes.** Checked in this order: below 30 × 6 (width < 30 or height < 6): the counter line only, cut with `…`.
  Width ≥ 60 and height ≥ 16: as F5. Otherwise (width 30–59 or height 6–15): no art and no title, the counter
  and hint lines centred. 0 rows or 0 columns: nothing drawn, no
  panic. Every size works at a non-zero origin. Counts are drawn in full (`0 / 0`, six-digit counts) and cut by
  display width, never bytes.
- **F8 Fade.** 250 ms by deadline (not a frame count): the art switches to `_dim` roles, then to dim text, then home.
  None under `NO_COLOR`, after a skip, or after an error. Why: the board's hand-over without a slow transition.

Non-goals: the tmux / Herdr pane jump (backlog); a disk cache unless F3 fails; the Echo (dolphins), Bloom or Manta
art (kept on the board for later); animation after the first load; sound; a new colour; changing any report or JSON.

Whole-phase Verify: `cargo test --workspace` → all `ok`; clippy `-D warnings` and `fmt --check` clean; every
committed key unchanged (`git diff <15.0 commit> -- fixtures/expected` shows 15.0's loading goldens only);
`fixtures/expected/phase9/check.sh 6`, `phase12/check.sh`, `phase13/check.sh`, `phase14/check.sh` all `ok`;
real cold overview ≤ 2.0 s (F3); loading goldens equal cell for cell; W10 scan.

### Step 15.0 — Rules, loading goldens, timing baseline (Claude)
- Depends on: 14.4. F1–F8 into `docs/design.md` ("Phase 15 decisions"; the NFR row records the baseline). Commit the
  approved board source `docs/boards/board15-loading.html`. A scratch timing script (release build, 3 fresh
  processes of `multi-link overview --json` on real `~`, wall/user/sys) recorded as the baseline in PROGRESS.
  Goldens for the loading screen at `drawC(4.0)` with counter `reading transcripts  612 / 871`, hand-checked
  against the board's paused frame: `fixtures/expected/tui/loading-{120x40,76x22,60x16,59x16,50x12,29x6,28x5}.{txt,styles.txt}`
  (truecolor), `loading-76x22-256.styles.txt`, `loading-76x22-nocolor.{txt,styles.txt}`, `loading-76x22-ready`,
  `loading-76x22-linking`, `loading-76x22-zero` (`0 / 0`, phase reading) and `loading-76x22-fade1`.
- Files: `docs/design.md`, `docs/boards/board15-loading.html`, `fixtures/expected/tui/loading-*`, `PROGRESS.md`
- Verify: `cargo test --workspace` → all `ok` (nothing reads the goldens yet); baseline (a) and (b) recorded
  (3 runs each); the 76×22 golden's art rows equal the board's `drawC(4.0)` glyphs (Claude's side-by-side screenshot
  in scratch, described in PROGRESS; the board's simulated counter is replaced by the fixture counter).

### Step 15.1 — Parallel cold parse, `Arc` cache values, progress handle (Codex)
- Depends on: 15.0. F1, F2, F4 (core side only).
- Files: `crates/core/src/scanner.rs`, `crates/core/src/lib.rs` (re-export the progress handle only),
  `crates/core/tests/**`, `PROGRESS.md`
- Verify: tests, clippy, fmt clean; all four `check.sh` scripts `ok` and every `fixtures/expected` key unchanged
  (the pre-change oracle, fixed clock and probe); new tests: 1 thread vs many give byte-identical `overview`,
  `sessions` and `links` JSON, diagnostics and parse stats (thread count set through a `#[doc(hidden)]`
  `Scanner::with_parse_threads(n)` the integration tests can reach, and a test that completes parses in reverse
  order); a malformed record and an unreadable file each reported exactly once, same text as before; a file appended,
  replaced or deleted between listing and the serial loop gives the same result as a serial pass of the new bytes;
  progress snapshots are monotonic through the whole worker first load, `files_total` never changes, zero misses goes
  straight to `linking`; the handle survives the first root reset; a warm pass parses nothing. F2's measurement
  recorded and its branch taken. Real `~` timing (F3 (a), (b) with and without `--live`, (d)), recorded next to the
  baseline. (a) or (b) over 2.0 s → stop (F3).

### Step 15.2 — TUI loading screen render (Codex)
- Depends on: 15.1. F5–F7 drawing as a pure render from (time, progress snapshot, size, theme).
- Files: `crates/tui/src/loading.rs` (new), `crates/tui/src/lib.rs`, `crates/tui/tests/**`, `crates/tui/examples/home_html.rs`
  (a loading variant for the README image), `PROGRESS.md`
- Verify: tests, clippy, fmt clean; renders equal every 15.0 golden cell for cell (truecolor, 256, NO_COLOR);
  a property test over 200 frames at 76×22 finds only the allowed glyphs and only F6's roles; NO_COLOR frames have the
  same glyphs as truecolor and no colour or modifier; bounded-buffer tests at 0×0, 1×1, 29/30 columns, 5/6 rows,
  59/60 × 15/16, a non-zero origin and a six-digit counter draw inside the area without panic; every existing
  snapshot unchanged.

### Step 15.3 — Wiring: show, skip, fade, progress (Codex)
- Depends on: 15.2. F4 (UI side), F5 timing and keys, F8.
- Files: `crates/tui/src/app.rs`, `crates/cli/src/tui.rs`, `crates/tui/tests/**`, `crates/cli/tests/**`, `PROGRESS.md`
- Verify: tests, clippy, fmt clean; app tests with an injected clock: report at 0.3 s → loading until 1.0 s, fade
  1.0–1.25 s, home at 1.25 s; report at 1.6 s → fade 1.6–1.85 s; a key at 0.2 s (consumed, not passed to home) →
  `loading sessions…` until the report, then home with no fade; a key after the report → home at once; a refresh
  error at 0.5 s → error footer at once; worker disconnected → same; NO_COLOR → no fade; progress shown is the
  handle's snapshot. CLI: the existing terminal guard tests still pass, and a test that `q` during loading returns
  without joining a blocked worker. Existing app tests unchanged.

### Step 15.4 — Real check, README, owner confirm (Claude)
- Depends on: 15.3. Release build on real `~`: F3 (a)–(d) (3 fresh runs; warm median of 10); a PTY smoke run
  (loading shows, the counter rises, skip, resize across 60×16 and 30×6, `q` and Ctrl-C during loading and during the
  fade restore the terminal; `--live` start).
  README: the loading screen with an image from the fixture render, the new cold time. `docs/phase15-check.md`.
- Files: `docs/phase15-check.md`, `README.md`, `docs/img/**`, `PROGRESS.md`
- Verify: whole-phase Verify; timings recorded; the owner's answer recorded.

### Phase 16 — A multi-link pane inside Claude Code (Release 1.13)
Need (owner, 2026-11-11): "while I work in Claude Code I want to see what's running in this repo, and the Codex runs
this session handed off, without switching to the TUI." Design FR16 (backlog: a Claude Code mod) and the bridge
decision "mod runs `multi-link` and reads JSON" (design § Solution design). Board 16 (claude.ai artifact
2ksgLic1phpGMUD9yojosg, source `docs/boards/board16-mod-pane.html`) offered A · Tree, B · Tree + step strip and
C · Repo view; the owner first picked a side pane over a strip above the prompt, then **C · Repo view** on 2026-11-11.
Comparables (board and design § comparables): repo-sessions (a mod with swimlanes; busy/idle only, Codex rows not tied
to the Claude session that started them), the TUI's own live grid (Phase 14; C is its one-repo cousin), Claude Code's
built-in tasks pane (one list, state as a glyph). What none does: nest a Codex run under the Claude session that
started it, with `exact` vs `likely` said out loud.
Measured 2026-11-11 (release `a97bd1a`, real `~`): a fresh `multi-link` process is a cold pass (1.16 s wall, all
cores), while a warm in-process refresh is 130–194 ms. So polling a fresh process every few seconds would cost a cold
pass each time; the pane needs one long-lived process per session.

Decisions:
- **G1 One CLI command shapes the pane, from the repo view's rules.** `multi-link pane --json --cwd <dir> --session
  <id>` prints exactly what the pane draws. Rows are the TUI repo view's (W6, W8), for the repo the cwd maps to
  (`RepoMode::map`; a cwd outside git is its own group, as today): top-level non-imported sessions with activity in
  the 2 h window; each Codex child nested under the Claude session of its latest placing link (the existing
  `parent_links` rule in `crates/core/src/live.rs`: latest `at`, then index), so a child appears once; a child whose
  parent is absent or in another repo stays top-level; an ambiguous or unlinked hand-off is never a child: it shows on
  the parent as `N possible runs` / `no codex run found` (W5, W8). Sections by state: `working` = busy; `waiting` =
  waiting or stale (stale marked); `recent` = every other row (idle, finished, unknown, Codex `turns_complete`), so
  every row has a section and no "ended at" time is invented (Codex "done" = its last turn completed, W5; Claude has no
  end time). Shape: `{ v: 1, repo, now, this: <id>, seen: bool, sections: [ { name, rows: [ { id, harness, label,
  step, state, phrase, is_this, children: [ { id, harness, state, phrase, match: "exact" | "likely" } ], handoffs:
  [ { step, outcome, candidates } ] } ] } ] }`; `label` and `phrase` are W3 and W5 (the same words the TUI shows);
  `match` is `exact` only for an exact link (project rule). `seen` = the session id is in the data; `this` sorts first
  in its section, and when `seen` is false the mod pins a placeholder row (G4). Built in `crates/cli` from the TUI's
  existing repo-view model (the CLI already depends on `multi-link-tui`); no core or tui type changes beyond making
  that model reachable. Passes: the same refresh the TUI worker runs (`refresh_with_live` in `crates/cli/src/tui.rs`:
  the 2 h pass, the all-time step pass, live), one shared `now`, so the pane and the TUI agree. Why: the rules already
  exist, are tested and were chosen with the owner; re-deriving them in a second place would let the two screens
  drift.
- **G2 A long-lived watcher, not polling.** `multi-link pane --watch --json …` keeps one `Scanner` warm and prints one
  G1 object per line (NDJSON) every 3 s, and only when the object minus `now` and every time phrase differs from the last
  one printed, or 30 s have passed (heartbeat), so the change test is not defeated by the clock. It prints its first line as soon as the first (cold) pass ends. Errors print
  `{ v: 1, error: "<text>" }` and the watcher keeps going. It exits when killed (the mod's teardown, G3), and on its own within one tick when
  its parent process is gone (the parent pid captured at start changed, or was already 1 at start), so it never
  outlives Claude Code even if the mod's teardown is missed. Why: a warm refresh is ~0.15 s against
  ~1.2 s cold (measured above); one process per session is the cheapest correct shape.
- **G3 The mod draws C.** In the repo under `mod/multi-link-pane/` (design: repo shape lists `mod/`): `plugin.json`,
  `hooks/hooks.json`, `hooks/register.tsx`, `types/index.d.ts`. On `session.start` it registers `/multi-link` (opens
  the pane) and opens a pane `multi-link`. It spawns the G2 watcher with `$.process.spawn` for the session's life (the
  engine kills it when the loop ends or the module unloads), parses each complete line, keeps the latest object in a
  `$.state` atom and redraws the pane from it. Teardown: on `session.end` and on module unload it ends the spawn loop
  (which kills the child) and cancels any pending retry timer, so no watcher is restarted after the session ends. Layout as board 16 C: header `multi-link · <repo name>`, then
  `working`, `waiting`, `recent (2h)` rules, one row per session (glyph `●` live / `✓` done in its harness colour, short
  id or session name, step), `this session` highlighted with a `▌` bar, its children indented with `└`, a likely link
  shown as `likely`, footer `updated Ns ago`. Overflow: the header, the `this session` row
  (with up to 3 children, then `+N runs`) and the footer are always kept; other rows drop from the bottom of `recent`,
  then `waiting`, then `working`, each section ending `+N more` (N counts sessions; a parent's children go with it);
  every text is cut by display width with `…`; below 4 rows only `this session` and the footer. Colours: Claude orange, Codex blue, done green, running amber,
  through the surface's own text colours where it has them. Why C: the owner's pick; it shows the parallel
  sessions the owner actually runs (the coordinator plus one session per step).
- **G4 States.** No watcher line yet: `reading sessions…`. `multi-link` not on PATH (spawn rejects): `multi-link isn't
  on PATH. See the README.` A watcher that answers an unknown command (an older `multi-link`): `This multi-link is too
  old for the pane. Update it.` `seen: false`: the repo view as usual with a pinned `▌ this session · not seen yet` row at the
  top of `working`.
  An `error` line: the last good data stays, with the error dimmed in the footer. The watcher exiting: restart after
  2 s, then 4, 8, up to 60 s, and say `watcher stopped, retrying` in the footer. Empty repo: `No sessions in this repo
  in the last 2 h.`
- **G5 Undocumented, so confirmed first.** Claude Code's mod API is new to this project, so 16.0 runs a spike in an
  interactive terminal session (a PTY, as 15.4's harness) before any code: (a) the engine's `sessionId` equals the
  Claude transcript id multi-link reads (the file name under `~/.claude/projects/`), in the desktop app as well as the
  terminal; (b) a pane opens and redraws from a `$.state` atom, in the terminal and in the desktop app; (c) a slash
  command registers; (d) `$.process.spawn` streams a long-lived child's lines and unloading the module kills it;
  (e) `claude plugin test` mounts the pane on both `terminal` and `desktop`. 16.3's surface list follows what (b) and
  (e) found. If (a) fails, the mod passes the session's `transcript_path` instead and G1 takes
  `--transcript <path>`; if (b) fails in the desktop app, the phase still ships for the terminal and says so in the
  README. Either is a plan change logged in the Review log, not a stop, unless no surface draws the pane.
- **G6 No new installs.** The mod is checked with `claude plugin validate` and `claude plugin test` (both ship with
  Claude Code); no TypeScript compiler, npm package or bundler is added. Loading it for real uses
  `claude --plugin-dir <repo>/mod/multi-link-pane` (terminal) or `CLAUDE_CODE_PLUGIN_DIRS` (desktop app), which is
  the owner's one paste (stop-and-ask, one ready command).

NFRs: the pane's first data ≤ 2.5 s after a session starts on real `~` (cold pass 1.16 s plus spawn); the watcher's
refresh median ≤ 250 ms (a warm pass, 14.4/15.4 range); the watcher costs no CPU between ticks; no network; no orphan
watcher 5 s after its Claude Code process exits.

Non-goals: the step strip (board 16 B) and the step graph in the pane; actions from the pane (open, resume, jump):
read-only for now; a strip above the prompt; notifications; publishing the mod to a marketplace or GitHub (an owner
decision later); changes to `overview`, `live`, `links` or any existing JSON; any core type change; Windows.

Whole-phase Verify: `cargo test --workspace` → all `ok`; clippy `-D warnings` and `fmt --check` clean; every
committed key unchanged except the new `fixtures/expected/pane/` files; `fixtures/expected/phase9/check.sh 6`,
`phase12/check.sh`, `phase13/check.sh`, `phase14/check.sh` all `ok`; `claude plugin validate mod/multi-link-pane` and
`claude plugin test mod/multi-link-pane` clean; real pane check in 16.4; W10 scan.

### Step 16.0 — Rules, pane goldens, API check (Claude)
- Depends on: 15.4. G1–G6 into `docs/design.md` ("Phase 16 decisions"; FR16 updated to the pane). Commit board 16.
  G5 spike in a throwaway mod in the session scratchpad (never in the repo), loaded with `--plugin-dir` in an
  interactive `claude` session in a PTY and, if the owner turns it on, in the desktop app: log `sessionId` and
  `transcript_path` and compare with the transcript file; open a pane that redraws from an atom on a timer; register
  a command; spawn `sleep`-style child and check it dies on unload; run `claude plugin test` on both surfaces.
  If (a) fails, G1 takes `--transcript <path>` instead (a logged plan change). Hand-written answer keys for G1 on the
  committed fixtures (fixed `--now`): `fixtures/expected/pane/{busy,waiting-stale,done-2h,children-exact-likely,
  unseen,ambiguous,child-other-repo,idle-recent,empty}.json`, each checked by hand against the TUI repo
  view (`home_html` render) and `live` / `links` output for the same fixture. PROGRESS lists the exact command for
  each key (`--root`, `--cwd`, `--session`, `--now`).
- Files: `docs/design.md`, `docs/boards/board16-mod-pane.html`, `fixtures/expected/pane/**`, `PROGRESS.md`
- Verify: `cargo test --workspace` → all `ok` (nothing reads the keys yet); G5 (a)–(e) results recorded in
  PROGRESS with the logged values; each key's rows traced to the matching `live`/`links` JSON in PROGRESS.

### Step 16.1 — `multi-link pane --json` (Codex)
- Depends on: 16.0. G1.
- Files: `crates/cli/src/pane.rs` (new), `crates/cli/src/main.rs`, `crates/cli/src/tui.rs` (visibility of
  `refresh_with_live` only, Review log), `crates/cli/tests/**`, `PROGRESS.md`
- Verify: tests, clippy, fmt clean; each 16.0 key's command (as listed in PROGRESS) prints that key byte for byte;
  tests: `is_this` sorts first, a child appears once under its latest placing link, an ambiguous hand-off is never a
  child, `exact` only for exact links, idle and Codex-done rows land in `recent`, imported rows are absent, an
  unknown session gives `seen: false`; a temporary git repo with a worktree (built in the test) maps two cwds to one
  repo, since fixture mode skips git; rows and phrases equal the TUI repo view's for the same input; every existing
  key and `check.sh` unchanged.

### Step 16.2 — `pane --watch` (Codex)
- Depends on: 16.1. G2.
- Files: `crates/cli/src/pane.rs`, `crates/cli/src/main.rs`, `crates/cli/tests/**`, `PROGRESS.md`
- Verify: tests, clippy, fmt clean; tests with an injected clock and tick: the first line after the first pass, no
  line on a tick where only times moved, a line on a real state change, a heartbeat at 30 s, an error line and the watcher keeps running, every line parses as one G1 object; a
  process test that the watcher exits within one tick once its parent is gone (spawn it from a child process that
  exits), and one that starts with parent pid 1 exits at once; real `~` release: first line time and a 10-tick
  refresh median of the exact pane pass recorded (NFRs: ≤ 2.5 s, ≤ 250 ms; over budget → stop and plan, as F3).

### Step 16.3 — The mod (Codex)
- Depends on: 16.2. G3, G4.
- Files: `mod/multi-link-pane/**` (manifest, hooks, types, `*.test.tsx`), `PROGRESS.md`
- Verify: `claude plugin validate mod/multi-link-pane` clean; `claude plugin test mod/multi-link-pane` → all pass on
  each surface 16.0 found the kit mounts (both, unless G5 (e) said otherwise): the pane draws each 16.0 key as board 16 C (sections, `▌` this session,
  `└` children, `likely`), every G4 state (spawn rejects, too-old answer, no line yet, `seen: false`, error line keeps
  data, watcher exit → retry with backoff on the mocked clock, empty repo), overflow as G3 at 20×6, 30×12 and a
  working-only list taller than the pane (this session and footer kept, `+N more` counts sessions); a line split
  across two chunks parses once; `session.end` and unload during streaming and during a backoff wait leave no running
  child and no pending timer.

### Step 16.4 — Real check, README, owner confirm (Claude)
- Depends on: 16.3. Load the mod with `--plugin-dir` in a real terminal session in this repo and (owner paste) in the
  desktop app: the pane matches board 16 C (screenshot side by side), the coordinator and a step session show under
  `working`, a Codex hand-off appears nested with the right `exact`/`likely`, NFRs measured (first data, refresh
  median, no orphan 5 s after quit). README: a "Claude Code pane" section with the screenshot and the one load
  command. `docs/phase16-check.md`.
- Files: `docs/phase16-check.md`, `README.md`, `docs/img/**`, `PROGRESS.md`
- Verify: whole-phase Verify; NFRs recorded; the owner's answer recorded.

### Phase 17 — The pane shows the step, acts on rows, and says when Codex finishes (Release 1.14)
Need (owner, 2026-11-11): "in the pane I want to see how my step is going across both tools, do something with a row,
and know when a Codex run I handed off is done without watching for it." Backlog items "pane step strip and actions
(board 16 B)" and "notifications", plus the owner's two Phase 16 polish notes (`docs/phase16-check.md` →
Observations: empty section rules stay drawn; `updated Ns ago` counts from the last printed line, up to ~30 s).
Board 17 (claude.ai artifact P8y29q3iki7kbmCWKWytfJ, source `docs/boards/board17-pane-strip.html`): the owner picked
**S1 · Chain** (strip), **A1 · Expand in place** (row actions) and **N2 · What it said** (notification) on 2026-11-11.
Comparables: the TUI's own step graph (Phase 9; S1 is its parts folded to a list), Claude Code's built-in "waiting on
you" notification (so that case is not repeated here), GitHub Actions' "workflow finished" notification (one per run,
the outcome in the body). Facts checked 2026-11-11: `multi-link graph --json --repo --step` already returns the step's
parts in order (`kind` plan/implement/review, `harness`, `status`, `from`/`to`, `active_ms`, `asked`, `said`);
`multi-link open <id>` opens a finished Claude session in the app (FR18) and Codex opening is unsupported (README);
Claude Code's `$.ui.notify` answers `no-surface` in the desktop app (mod API types), so notifications go through
macOS's built-in `osascript`, which needs no install.

Decisions:
- **H1 Strip data in the pane JSON (additive).** `pane --json` gains `strip`: null unless this session is seen and its
  row has a step; else `{ step, title, parts: [ { harness, kind, status, active_min, is_this } ] }` from the same
  graph model `multi-link graph` uses, for (repo, this row's step), in graph order. `status` comes from the part itself and
  the graph's existing live rule, never from the session alone (one session can own several parts): `done` (the
  graph's part status done), `running` (the part's index is in `live_parts` and not stale), `stale` (an open Codex
  turn the graph calls stale), `waiting` (a live Claude part whose session waits), else `null` (W5: no guess).
  `active_min` = `active_ms` rounded down to whole minutes, at least 1. Rebuilt every tick as the TUI does (G11).
  Rows and children also gain `asked` (W3, already ≤ 120 characters), `said` and `openable`. `said`: for a Codex
  session, the final message of its last completed turn, from the same code the graph uses for a hand-off's last
  completed turn (`crates/core/src/graph.rs`), applied to the whole rollout; cut at 120 characters on a char
  boundary; null when no turn completed. For Claude, always null this phase (the graph has no Claude `said`).
  `openable`: true exactly when the real `open` resolver (`crates/core/src/open.rs`) returns `AppLink` for that id;
  false for `Resume`, unavailable and every Codex session. JSON `v` stays 1: every change is a new field (W0 additive rule); the ten 16.0 keys gain the new
  fields and nothing else changes in them. Why: the rules exist in Rust already (graph, FR18); the mod stays a drawer.
- **H2 Strip layout (S1).** Under the header: `step <id> <title>` (title cut to fit), then one line per part:
  harness (in its colour), kind, `✓` done / `●` running / `◆` waiting / `·` unknown, then `Nm`; every part of this session
  marked `▌`. At most 4 part lines: the newest 4, except that if none of them is this session's, this session's newest
  part replaces the oldest shown; the rest fold into `+N earlier` on top. Overflow (G3, extended): kept always:
  header, strip heading, this-session row, footer. Dropped first to last: `recent` rows, `waiting` rows, strip part
  lines (oldest first, `+N earlier` grows), an open row's `said` then `asked` line, `working` rows; an open row's
  buttons go last, with the row itself.
- **H3 Row actions (A1).** Every session and child line is a `Button` keyed by its id (one press target per row).
  Pressing a row toggles it open (one open row at a time); an open row shows `asked` and `said` lines (dim, cut to the
  pane width, `—` when null) and its buttons: `Open in app` when `openable` (runs `multi-link open --app-only <id>`
  by argv: a new flag that refuses, with a message, whenever the resolver's answer at press time is not `AppLink`, so a
  session that changed since the last draw never resumes in a terminal; the message goes to the footer), and `Copy id` always (`$.ui.copy`, result said in the footer). The open row
  survives redraws by id; it closes if its row leaves the data. Keys: the surface's own focus and Enter.
- **H4 Codex-done notifications (N2), at most once per hand-off.** Only a watcher started with `--notify` (the mod
  passes it when its `notify` option is on and the host is macOS) looks for events; others never claim anything. A
  **run** is one hand-off: the link's invocation id plus the child Codex id (a resumed Codex session serving two
  hand-offs is two runs). Events come from the full refresh's links and sessions **before** pane filtering (imports,
  the 2 h window and nesting do not hide a run), for links in this repo with a child (exact or time-matched). A run is
  done when its hand-off window's last turn completed (the graph's window rule, completion evidence `turns_complete`).
  Baseline: every run seen in the watcher's first successful pass is known; a run first seen later that is already
  done is also known (only a not-done → done change the watcher observes is announced); a failed scan changes nothing;
  a run that disappears and comes back keeps its state. On a change, the watcher creates
  `<state dir>/notified/<invocation id>` with `create_new`; only the creator emits
  `{ "v": 1, "event": { "kind": "codex_done", "run", "id", "step", "said" } }` (`said` cut to 120 characters as H1).
  Markers are permanent tombstones (one small file per run, never cleaned), so there is no cleanup race. The mod shows
  it with `osascript` by argv (`on run argv` … `display notification (item 2 of argv) with title (item 1 of argv)`:
  no shell, no text spliced into the script), title `codex finished (<step>)` or `codex finished`, body `said` or
  `no final message`. Delivery is **at most once**: if the emitting pane's pipe breaks or `osascript` fails, that run
  is not announced again (the footer says the notification failed). The README states that notifications show the
  first 120 characters of Codex's final message in Notification Center, and that the text passes through a local
  process argument.
- **H5 Polish.** A section with no rows draws no rule (a pinned `this session` row still shows). The watcher prints
  `{ "v": 1, "tick": "<now>" }` on every tick with no data line, and the footer reads `checked Ns ago` from the last
  line of any kind, so it stays under ~3 s while the watcher lives.
- **H6 Confirmed first.** 17.0 checks in a PTY terminal session that `osascript` posts a notification from a mod's
  `$.process.run` (and what app it is credited to), that a pane `Button` press reaches `ui.press` and `$.ui.copy`
  answers, and that the H1 `said` helper returns a value for completed Codex runs on real data. Raw real-data output
  stays in the scratchpad (W10); committed logs record pass/fail and lengths, never prompt or message text, and the
  notification check uses synthetic text. The desktop app
  half is the owner's, in 17.4.

NFRs: the watcher's warm tick (now with the graph rebuild) ≤ 250 ms CPU on real `~`, measured as 16.4 did (`ps -o
time` after the first line and 30 s later, divided by 10 ticks; 16.4: 140 ms); a notification within one tick (3 s)
of the watcher seeing the run done; at most one notification attempt per hand-off run on this machine.

Non-goals: notifications for "waiting on you" (Claude Code already sends them); notifications on Linux or Windows;
opening Codex sessions; clicking a notification; sound; a strip for other rows' steps; changing the TUI or any
existing JSON field; any core type change beyond a `pub` on the graph helper named in H1.

Whole-phase Verify: `cargo test --workspace` → all `ok`; clippy `-D warnings` and `fmt --check` clean; every
committed key unchanged except the pane keys' new fields (17.0) and new `fixtures/expected/pane/` keys;
`bash fixtures/expected/phase9/check.sh 6`, `phase12`, `phase13`, `phase14` all `ok`; `claude plugin validate` and
`claude plugin test` clean; real check in 17.4; W10 scan.

### Step 17.0 — Rules, pane keys, API check (Claude)
- Depends on: 16.4. H1–H6 into `docs/design.md` ("Phase 17 decisions"). Commit board 17. H6 checks with a throwaway
  mod in the session scratchpad (never in the repo), logged values recorded. The ten pane keys regenerated by hand
  with the H1 fields (each new value traced to `graph` / `open --dry-run` output for the same fixture), plus a new key
  `fixtures/expected/pane/openable.json` (strip coverage comes from the existing keys, Review log) and watcher keys
  `fixtures/expected/pane/watch-{tick,codex-done}.jsonl`. The watcher keys come with their inputs pinned in PROGRESS:
  fixture root, `--now` per tick, and the scripted refresh results per tick (a test data file, e.g. tick 1 run open,
  tick 2 done, tick 3 unchanged), so 17.2's tests replay them exactly. PROGRESS lists each key's full command.
- Files: `docs/design.md`, `docs/boards/board17-pane-strip.html`, `fixtures/expected/pane/**`, `PROGRESS.md`
- Verify: `cargo test --workspace` → all `ok` except the tests that read the pane keys (the golden test and 16.2's
  watch test), which now fail on exactly the new fields (recorded); H6 results recorded with logged values; each key traced in PROGRESS.

### Step 17.1 — Pane JSON: strip, asked/said, openable (Codex)
- Depends on: 17.0. H1.
- Files: `crates/cli/src/pane.rs`, `crates/cli/src/main.rs` (`open --app-only`), `crates/core/src/graph.rs` (extract the
  inline Codex `said` expression into one `pub` helper, graph output unchanged; Review log), `crates/cli/Cargo.toml` + `Cargo.lock`
  (the workspace's own `multi-link-codex` as a CLI dependency, review round 1), `crates/cli/tests/**`, `PROGRESS.md`
- Verify: tests, clippy, fmt clean; every 17.0 `pane --json` key byte for byte; tests: strip null without a step or
  unseen, parts in graph order, a busy session owning an old and a current part marks only the current one `running`,
  a stale Codex part is `stale`, `status` null when unknown, `said` cut on a char boundary and null for Claude,
  `openable` true exactly for `AppLink`; `open --app-only` refuses a `Resume` answer (a session whose app record went
  away between draw and press) and opens an `AppLink`; every other key and `check.sh` unchanged.

### Step 17.2 — Watcher: ticks, Codex-done events, one notification per run (Codex)
- Depends on: 17.1. H4 (watcher side), H5 (tick lines).
- Files: `crates/cli/src/pane.rs`, `crates/cli/tests/**`, `PROGRESS.md`
- Verify: `TZ=UTC cargo test --workspace`, clippy `-D warnings`, `fmt --check` clean; with the injected runtime
  and the 17.0 tick scripts: the watch keys line for line; a tick line on every unchanged tick; no events without
  `--notify`; a run done at the first pass, or first seen already done, gives no event; a not-done → done change
  gives one event; two hand-offs sharing one Codex id give two events; a failed scan between ticks changes nothing;
  two watchers on one state dir give one event in total, including both seeing it on the same tick (an injectable
  marker store with a barrier forces the race); a broken output pipe after claiming leaves the tombstone (no second
  attempt); real `~` release: warm tick CPU by the NFR method (≤ 250 ms, else stop and plan).

### Step 17.3 — The mod: strip, row actions, notifications, polish (Codex)
- Depends on: 17.2. H2, H3, H4 (mod side), H5.
- Files: `mod/multi-link-pane/**`, `PROGRESS.md`
- Verify: `claude plugin validate` clean; `claude plugin test` → all pass on terminal and desktop: the strip from
  each 17.0 key as H2 (4-part cap, `+N earlier`, `▌` on this session's part), a press opens and closes a row, one open
  row at a time, `Open in app` only when `openable` and it runs `multi-link open <id>` by argv, `Copy id` calls
  `$.ui.copy`, the watcher gets `--notify` only when `notify` is on, an event line runs `osascript` with exactly the H4 argv, an
  `osascript` failure says so in the footer and is not retried, an open row in a short viewport drops `said`, then
  `asked`, before its buttons, empty sections
  have no rule, the footer says `checked Ns ago` from a tick line; every Phase 16 test still passes.

### Step 17.4 — Real check, README, owner confirm (Claude)
- Depends on: 17.3. Real terminal session (PTY, as 16.4): strip and actions match board 17 S1/A1, a real Codex
  hand-off in `~/multi-link-sandbox/` with a synthetic prompt gives exactly one notification with two panes open
  (one marker file), tick CPU measured by the NFR method. Mocked plugin tests are not the desktop check. The owner: the
  desktop app shows the strip and acts on a row, and macOS shows the notification (allowing notifications for the
  sender if macOS asks). README: the pane section updated (strip, actions, notifications and what they show).
  `docs/phase17-check.md`.
- Files: `docs/phase17-check.md`, `README.md`, `PROGRESS.md`
- Verify: whole-phase Verify; NFRs recorded; the owner's answer recorded.

### Phase 18 — Jump to a running terminal session's own tab (Release 1.15)
Need (owner, 2026-11-12, backlog "tmux / Herdr jump"): "when a Claude session I started in a terminal is still
running, take me to it instead of telling me it's unavailable." Today `open` on a running Claude session without an
app link answers `unavailable` (`running_without_app_link`) with `claude --resume <id>`, which would start a second
copy. Owner's choices (2026-11-12): targets **Terminal.app and iTerm2 tabs** (no install); offered in the **CLI, TUI
and pane**; **Codex left out**. Board 18 (claude.ai artifact Eu2BQxCGZQAue5s7623CCw, source
`docs/boards/board18-terminal-jump.html`): the owner picked **J1 · Jump straight**. Comparables: claudectl (jumps by
tty with AppleScript; silent, raw AppleScript errors when the tab is gone), Herdr's sidebar (always works, but only for
agents started inside it), tmux `choose-tree` (shows the target first; tmux only). Facts checked 2026-11-12: a
running Claude session's record `~/.claude/sessions/<pid>.json` has `pid` and `procStart` (the liveness probe
already uses them); Terminal.app tabs and iTerm2 sessions expose a `tty` property to AppleScript (iTerm2's scripting
docs; claudectl's Terminal.app script; 18.0 confirms Terminal on this Mac); neither tmux nor Herdr is installed here and iTerm2 is not installed either. Why now with
no daily user (Phase 15 dropped it for that reason): the owner chose to try it; Terminal.app needs no install, so
18.0 and 18.4 can check it on real processes.

Decisions:
- **I1 A new open answer, `Jump`.** For a Claude session that is running (`busy`, `idle`, `waiting`, `stale`) and has
  no app link, the resolver takes **one identity-bearing process snapshot** (`ps -axo pid,ppid,lstart,tty,comm`,
  injectable like `ProcessProbe`, fixture mode reads a table), at most once per resolver build and only when such a
  session exists. The target is the record the existing rule already picks (output-schema "newest record not
  confirmed gone"); two records for one session that are both live in the snapshot → unavailable
  `ambiguous_process`. The record's pid must be in the snapshot **with its `lstart` equal to the record's
  `procStart`** (same snapshot as the tty and ancestors, so a reused pid cannot borrow another terminal); otherwise
  unavailable `process_changed`. A failed, denied, truncated or malformed snapshot, or a duplicate pid row, gives
  unavailable `process_unknown`: it is never exit evidence and never turns into `Resume`. ttys are compared in one
  form, `/dev/ttysNNN` (`ps` prints `ttys004`; `??` or `-` means no tty). Walking up from the session's pid:
  the first ancestor that is Terminal.app's or iTerm2's main binary → `Jump { app, tty, pid }`; a `tmux`, `screen` or
  `zellij` before it → unavailable `inside_multiplexer`; another app bundle (VS Code, Ghostty, WezTerm, Warp…) →
  unavailable `terminal_not_supported` with the app's name; no tty → unavailable `no_tty`. Every unavailable answer
  keeps today's `claude --resume <id>` text, with "(after it ends)". Finished and unknown sessions are unchanged.
  Why: the tty is the one id both the process table and the terminal's AppleScript share; the ancestor walk is what
  tells a Terminal tab from a tmux pane that also has a tty.
- **I2 Running the jump.** `osascript` by argv (`on run argv`, the tty as `item 1 of argv`, no text spliced into the
  script, no shell), one fixed script per app: Terminal.app selects the tab whose `tty` matches and brings its window
  forward; iTerm2 selects the session, its tab and its window. The script returns `window W tab T` or `not found`.
  `osascript` gets a 5 s deadline, then is killed. Every other outcome (missing `osascript`, another AppleScript
  error, malformed output, the deadline) is unavailable, exit 3, with its reason. Outcomes: jumped (exit 0, "jumped to Terminal · window 2 · tab 3 (ttys004)"); tab gone (unavailable, exit 3,
  "its tab is gone"); macOS refused (AppleScript error -1743, exit 3, "blocked by macOS: allow it in System Settings →
  Privacy & Security → Automation"). Never falls back to resume. The answer is re-resolved from a fresh pass right
  before it runs (FR18's fresh-decision rule).
- **I3 Where it shows (J1).** `multi-link open <id>` jumps and prints the line above; `--dry-run` prints the action
  JSON with `"action": "jump"`, `app`, `tty` (no window lookup: cheap); `--fork` on a running session stays
  unavailable as today. The TUI's existing open key `o` (Enter stays navigation, `app.rs` `FreshOpen`) jumps on such
  a row and says the same line in its status message; the TUI is not suspended for a jump (nothing interactive runs). The pane:
  rows and children gain `opens`: `"app"` | `"terminal"` | null (additive, `v` stays 1); `openable` becomes true for
  `AppLink` or `Jump`; the row button reads `Open in app` or `Jump to terminal`, and its press runs
  `multi-link open --no-resume <id>` (new flag: refuses `Resume`, accepts `AppLink` and `Jump`; `--app-only` stays
  and keeps refusing `Jump`).
- **I4 Answer keys first.** 18.0 writes the keys: `open --dry-run` JSON for a fixture set covering every I1 branch
  (Terminal, iTerm2, tmux, VS Code, no tty, `??` tty, pid reused (`lstart` differs), snapshot failed / malformed /
  duplicate pid, two live records, newer record dead with an older live one, app link wins, finished unchanged) and
  the AppleScript texts. The pane keys are **not** changed in 18.0 (so 18.1–18.2 stay green); 18.3 adds `opens`
  and Verify proves the change additive. A fake `osascript` in tests asserts the exact argv.
- **I5 macOS permission.** The first jump makes macOS ask whether the caller (Terminal, the Claude app…) may control
  Terminal or iTerm2. The README says so, with the Settings path, and -1743 gets its own message (Phase 17 lesson:
  a silent macOS permission cost the owner three tries).

Non-goals: Codex sessions; tmux, screen, zellij and Herdr panes; Ghostty, WezTerm, Warp, kitty, Alacritty and the VS
Code terminal (named in the message, not driven); Linux; starting a new terminal for a running session; choosing a
split inside an iTerm2 tab beyond selecting its session.

### Step 18.0 — Facts on real processes, design pins, answer keys (Claude)
- Depends on: Phase 17. I1–I5.
- Real-process facts in `~/multi-link-sandbox/` (scratch, never in the repo): start `claude` in a new Terminal.app
  window by `osascript … do script`; read its record's pid, `ps` tty and ancestor chain; run the I2 Terminal script
  by argv from another process and record its return and which tab is selected after; record the -1743 text if macOS
  refuses. iTerm2: not installed, so its script is written from its scripting docs and marked untested.
- Pins in `docs/design.md` → "Phase 18 decisions" (I1–I5: binary paths matched, reason names, message texts, JSON
  field order). Keys: `fixtures/expected/open-jump/*.json` with a fixture process table, the pane keys gaining
  `opens`, `fixtures/expected/open-jump/scripts/{terminal,iterm2}.applescript`.
- Files: `docs/design.md`, `fixtures/**` (new process table and keys), `PLAN.md`, `PROGRESS.md`
- Verify: `cargo test --workspace` → all `ok` (no existing key changes); the raw `ps` tty and the AppleScript tty
  recorded side by side; the Terminal facts recorded with their commands; W10 scan clean.

### Step 18.1 — Resolver: `Jump` and the process snapshot (Codex)
- Depends on: 18.0. I1, I4.
- Files: `crates/core/src/open.rs`, `crates/core/src/probe.rs` (the snapshot trait and its `ps` and fixture
  implementations), `crates/core/src/scanner.rs` (wiring the snapshot into `open_resolver`), `crates/core/src/lib.rs`
  (exports only), `crates/cli/src/main.rs` (`open --dry-run` JSON, and a temporary `Jump` arm that refuses with exit 3
  until 18.2), `crates/cli/src/tui.rs` (compile compatibility only: `..` in `Unavailable` patterns and a
  temporary refusing `Jump` arm, Review log),
  `crates/*/tests/**`, `docs/output-schema.md` (Open), `PROGRESS.md`
- Verify: tests, clippy `-D warnings`, fmt clean; every `open-jump` key byte for byte; process queries counted
  through the seams: the existing liveness batch plus at most one snapshot per resolver build, none when no session
  needs it; no failure branch yields `Resume` (a test per branch); every old `open` key and `check.sh` unchanged.

### Step 18.2 — Running the jump in the CLI and TUI (Codex)
- Depends on: 18.1. I2, I3, I5.
- Files: `crates/cli/src/main.rs`, `crates/cli/src/tui.rs` (the `o` path and its message), `crates/cli/tests/**`,
  `crates/tui/**` (the `FreshOpen` result message only), `PROGRESS.md`
- Verify: tests, clippy, fmt clean; with a fake `osascript` on `PATH`: exact argv per app, no shell, "jumped …" on a
  `window W tab T` answer, exit 3 and "its tab is gone" on `not found`, exit 3 and the Automation message on -1743,
  exit 3 with a reason for missing `osascript`, another AppleScript error, malformed output and a hung child (killed
  at 5 s), no resume ever spawned on any failure, `--no-resume` refuses `Resume` and accepts `AppLink` and `Jump`,
  `--app-only` refuses `Jump`; the TUI's `o` on a jump row runs the same path without suspending the terminal, shows
  the line, and Enter still navigates.

### Step 18.3 — Pane: `opens` and `Jump to terminal` (Codex)
- Depends on: 18.2. I3.
- Files: `crates/cli/src/pane.rs`, `crates/cli/tests/**`, `mod/multi-link-pane/**`, `PROGRESS.md`
- Verify: Rust tests, clippy, fmt clean; the pane keys regenerated with `opens` and proven additive (removing
  `opens` gives the old keys byte for byte, `openable` changes only where `opens` is `terminal`); `claude plugin validate` clean and
  `claude plugin test` all pass: the button label follows `opens`, its press runs `multi-link open --no-resume <id>`
  by argv, the footer shows the CLI's line or error; every Phase 16–17 test still passes.

### Step 18.4 — Real check, README, owner confirm (Claude)
- Depends on: 18.3. A Claude session started in Terminal.app in the sandbox: `multi-link open` brings its tab to the
  front from another window (selected tab's tty checked by AppleScript), the TUI's Enter does the same (PTY), the
  pane's button is checked by its tests and the owner; tab-gone and inside-tmux (if tmux is ever installed; otherwise
  fixtures only) messages recorded; the Automation prompt seen once. README: the open table and the pane section;
  iTerm2 stated untested unless it was installed. `docs/phase18-check.md`. Owner: one jump from the desktop pane to a
  terminal session (stop and ask).
- Files: `docs/phase18-check.md`, `README.md`, `PROGRESS.md`
- Verify (whole phase): `TZ=UTC cargo test --workspace`, clippy `-D warnings`, fmt clean; every `fixtures/expected/*/
  check.sh` exit 0; `claude plugin validate` and `claude plugin test` pass; the real-check items above recorded with
  their commands; the owner's answer recorded.
- Status (2026-11-12): real check, README and docs committed (d492bee). **Owner check pinned for later at the owner's
  request**; the next phase may start before it. Steps are in PROGRESS.md "Current state".

### Backlog (ordered, uncommitted)
`hook check` shipped in Phase 10 · live grid shipped in Phase 14 · faster cold first load in Phase 15 · Claude Code pane in Phase 16 · pane strip, actions and Codex-done notifications in Phase 17 · terminal tab jump in Phase 18 (tmux / Herdr panes stay optional backlog) · desktop menu-bar app.

## Review log

### 2026-11-12 — Step 18.1 scope (Claude, Codex stop)
- Codex stopped: the new `detail` field breaks the exhaustive `OpenAction::Unavailable { reason, command, cwd }`
  pattern in `crates/cli/src/tui.rs` (`open_result`). **Accepted:** 18.1 may touch `tui.rs` for compile compatibility
  only (`..` in such patterns, and a temporary `Jump` arm that refuses like `main.rs`'s); the TUI's real jump stays
  18.2.

### 2026-11-12 — Step 18.0 amendments (Claude, during the step)
- **Which record is the target.** "The existing output-schema pick" was replaced by a rule on the snapshot itself:
  of a session's records, exactly one live (pid present with matching `lstart`) → it; none → `process_changed`; two
  → `ambiguous_process`. It needs no other probe and handles newer-dead/older-live (fixture 18000009) directly.
- **Fixture snapshot.** Fixture mode reads `<root>/fixture-ps.txt` (raw `ps` text, so the real parser is tested); a
  root without it keeps today's answer, so the phase 9 open keys stay unchanged. Snapshot failure, malformed text and
  duplicate pids are whole-snapshot cases and go to 18.1 unit tests, not keys.
- **Unavailable `command` unchanged.** "(after it ends)" was dropped: the `command` text stays today's so existing
  consumers and keys do not change; a new `detail` field (only when present) names the multiplexer or app.
- **iTerm2 by bundle id**, so a Mac without iTerm2 never shows a "Where is iTerm2?" dialog; it stays untested.

### 2026-11-12 — Phase 18 plan critique (Codex, read-only, Codex loop)
`codex-quota.py` → `CODEX: OK`. Nine findings; all accepted.
1. **Snapshot failures had no "never resume" rule.** **Accepted**: any failed, denied, truncated, malformed or
   duplicate-pid snapshot → unavailable `process_unknown`, never exit evidence, never `Resume`; a test per branch.
2. **Liveness and tty came from different `ps` calls (pid reuse).** **Accepted**: one snapshot with `lstart`; the pid
   must match the record's `procStart` in that same snapshot, else `process_changed`.
3. **Which record when a session has several.** **Accepted**: the existing output-schema pick; two live → `ambiguous_
   process`; fixtures for newer-dead/older-live and two live.
4. **TUI Enter is navigation.** **Accepted**: the jump uses the existing open key `o` (`FreshOpen`); Enter unchanged.
5. **18.1 cannot compile without a `Jump` arm in `main.rs`.** **Accepted**: a temporary refusing arm (exit 3) in 18.1.
6. **`scanner.rs` builds the resolver.** **Accepted**: added to 18.1's files; one snapshot per resolver build.
7. **tty forms differ (`ttys004` vs `/dev/ttys004`, `??`).** **Accepted**: one canonical `/dev/ttysNNN`; 18.0 records
   both raw values; fixtures for `??`.
8. **No deadline; the TUI waits on its child.** **Accepted**: 5 s deadline then kill; every other failure is a reason;
   the TUI is not suspended for a jump.
9. **Pane keys changed in 18.0 would fail 18.1–18.2; no whole-phase Verify.** **Accepted**: pane keys change in 18.3
   only, proven additive; Step 18.4 spells out the whole-phase Verify.

### 2026-11-11 — Step 17.4 scope (Claude, real check)
- The real PTY check found two mod bugs: the pane sized itself from `e.viewport` (the whole screen) instead of its
  own body (`e.props.bodyColumns`, `e.props.scroll.bodyRows`), so long rows wrapped and overflow never ran, a Phase 16
  bug that Phase 17's longer lines exposed; and a 1.13 binary's `unexpected argument '--notify'` looped as a crash
  instead of "too old". **Accepted:** `mod/multi-link-pane/hooks/register.tsx` and `register.test.tsx` join 17.4's
  files for these two fixes only. Claude made them, so Codex reviewed them (read-only, clean). Also fixed: the
  wrong date on design H6 (2026-11-12 → 2026-11-11).

### 2026-11-11 — Step 17.2 tick cost (Claude, review)
- Host measurement (release, real `~`, this repo, method of 16.4): warm tick **363 ms CPU** with the strip on,
  388 ms with `--notify`, **189 ms** with a session that has no step (no strip). The 250 ms budget fails, and the
  per-pass `scanner.graph` for the strip is about 175 ms of it. **Decision:** keep the strip but rebuild it only
  when its inputs change: this row's step, or any session's `(id, state, last_activity)` or link set in the
  all-time refresh; and at least every 30 s regardless, so `live_parts`' 5-minute staleness and active minutes
  still move. Between rebuilds the pane reuses the last strip. Same file (`pane.rs`), done as 17.2 review round 1;
  the budget then has to pass with `--notify` on, since 17.3's mod runs with it.
- Round 1 result: 190 ms/tick (193 with `--notify`) on an idle repo. But a working session's `last_activity`
  moves on nearly every tick, which would bring back the per-tick rebuild exactly while someone works. **Round 2:**
  drop `last_activity` from the key; the strip's minutes and live/stale edges move with the 30 s rebuild, which is
  fine for a display in whole minutes.

### 2026-11-11 — Step 17.1 scope (Claude, Codex stop)
- Codex stopped: `graph.rs` has no `said` helper to make `pub`; the logic is inline in the Codex part builder
  (`last.filter(completed).and_then(last_agent_message)`). **Accepted:** 17.1 may extract it into one `pub` helper
  that both graph and pane call, with `graph --json` output unchanged (its golden tests prove it). Nothing else in
  `graph.rs` changes.
- Review round 1 moved the raw-rollout reader into `pane.rs` (graph keeps a one-mode helper), so the CLI needs the
  workspace's own `multi-link-codex` crate (compressed rollouts). **Accepted:** `crates/cli/Cargo.toml` and
  `Cargo.lock` join 17.1's files; no outside crate is added.

### 2026-11-11 — Step 17.0 amendments (Claude, during the step)
- **No `strip-chain` / `strip-none` keys.** With H1 applied, the existing keys already cover the strip: `idle-recent`
  (7 parts, this session on two, so the 4-part fold), `children-exact-likely` (a running Codex part and a done Claude
  part), `ambiguous` (null statuses), `busy` (one part, not this session's), and `waiting-stale` / `unseen` (null).
  Duplicate keys would test nothing new. `openable.json` is added (the only fixture session whose `open` answers
  `AppLink`: phase9 `92000002` at 06:00).
- **Two tests read the keys, not one.** 16.2's watch unit test includes `children-exact-likely.json` and counts its
  top-level fields; it fails with the golden test after the key change and is 17.1's to update.
- **H3 uses `onPress`.** The spike showed a `ui.press` hook must answer `{ element }`; a `Button`'s `onPress` is the
  simple path (design → Phase 17 decisions).

### 2026-11-11 — Phase 17 plan critique (Codex, read-only, Codex loop)
`codex-quota.py` → `primary: 0%`, `secondary: 60%`, `CODEX: OK`. Ten findings; all accepted. (The plan was first
committed in `ada2d4b` before these fixes; this entry and the revised Phase 17 text supersede it.)
1. **`said` had no Claude source and Codex `said` is per hand-off.** **Accepted**: Claude `said` is null this phase;
   Codex `said` is the last completed turn's final message via the graph's helper on the whole rollout (a `pub` on it
   is the only core change).
2. **The marker claimed delivery before it could happen.** **Accepted**: only `--notify` watchers (notify on, macOS)
   claim; delivery is at most once and said so in the README; a failure is shown, not retried.
3. **7-day cleanup raced with claims and broke dedupe.** **Accepted**: markers are permanent tombstones, no cleanup.
4. **A Codex id is not a run.** **Accepted**: a run is a hand-off (invocation id + child); markers keyed by
   invocation id; test two hand-offs sharing one Codex id.
5. **`Open in app` could resume in a terminal if the answer changed.** **Accepted**: `openable` = `AppLink` exactly;
   new `open --app-only` re-checks at press time and refuses anything else.
6. **A busy session could mark old parts running.** **Accepted**: part status from the graph's `live_parts` and
   stale rule, with a `stale` status; test old and current parts of one session.
7. **Event source and baselines were vague.** **Accepted**: events from the refresh before pane filtering, completion
   by the hand-off window's `turns_complete`; baselines for first pass, late discovery, failed scans, reappearance.
8. **Privacy of notification text and check logs.** **Accepted**: body capped at 120 characters; raw real data stays
   in scratch, committed logs scrubbed, synthetic text for the notification check; README states the exposure.
9. **Strip folding and expanded rows had no overflow rule.** **Accepted**: every part of this session is marked; the
   fold keeps this session's newest part; a full drop order including an open row's lines.
10. **Verify inputs were not reproducible.** **Accepted**: 17.0 pins tick scripts and commands; 17.2 gets an
   injectable marker store with a barrier; CPU by 16.4's `ps -o time` method; mocked tests kept apart from the owner's
   desktop check.


### 2026-11-11 — Step 16.1 file scope (Claude, Codex blocker, Codex loop)
- Codex stopped before coding: G1 says the pane runs the TUI worker's exact refresh, but `refresh_with_live` in
  `crates/cli/src/tui.rs:114` is private. **Accepted**: 16.1's Files gain `crates/cli/src/tui.rs` for one change,
  `refresh_with_live` becomes `pub(crate)`, with no behaviour change. Why: copying the pass into `pane.rs` would let
  the pane and the TUI drift, which is what G1 exists to prevent.

### 2026-11-11 — Step 16.0 amendments (Claude, G5 spike and while writing the keys)
- **G5 (a) passed in the terminal**, so G1 keeps `--session <id>` (no `--transcript` change). The desktop-app half of
  (a) and (b) is pending for 16.4 (owner turns the mod on). (e): the test kit mounts the pane on all four surfaces;
  16.3 tests `terminal` and `desktop` as planned.
- **Started Claude children are rows.** G1 said "top-level sessions", but in the repo view a step session started by
  the coordinator (S3) nests under it at any depth. Every Claude lane is now a pane row (only Codex lanes nest), or the
  coordinator's step sessions, the reason board C was picked, would hide inside the coordinator's row. No fixture key
  has a started child (none of the nine cases needs one); 16.1 tests it.
- **Placement is the overview's (T11)**, the repo view's own rule: G1 named `live.rs` `parent_links`, which nests a
  child under a parent in another repo; the repo view keeps such a child top-level in its own repo, which is what G1's
  "a child whose parent is in another repo stays top-level" asks for. In the phase14 fixture the two rules pick the
  same parent for every same-repo child, including the 21:32 tie on `14290000`.
- **Quiet waiting and stale sessions stay** (review finding 1): the repo view's "activity in the window" rule hid a
  waiting session after 2 h quiet, so G1's `waiting` section and the owner's own row could go empty. Visible now also
  means `waiting` or `stale`. New key `waiting-quiet.json` (phase9 at 12:00, both rows have no activity in the window).
- **Review round 1** (fresh adversarial Claude subagent; rebuilt all nine keys from the pins: byte for byte equal).
  Accepted: 1 (above); 2 order is a flat `started_at` sort, not pre-order; 3 `seen` = in this object (narrows G1's
  "in the data"); 5 Codex step = `home_label`'s placing link, else latest hand-off, stated; 6 `repo` = mapped cwd
  even with no group; 7 `step` set even when a title wins; 9 candidates in links-report order. Rejected: 4 a
  cross-repo run missing from its parent's pane (G1 chose "a child appears once, in its own repo"; backlog
  candidate); 8 durations clipped to the window (the TUI shows the same words; the pane matches the TUI).
- **Pinned** (field rules, order, `TZ=UTC` for the keys, pretty vs compact JSON) in `docs/design.md` → Phase 16
  decisions, "Pinned while writing the keys".

### 2026-11-11 — Phase 16 plan critique (Codex, read-only, Codex loop)
`codex-quota.py` → `primary: 0%`, `secondary: 56%`, `CODEX: OK`. Ten findings; all accepted, most by reusing rules
the TUI already has.
1. **"Done" had no evidence behind it** (Claude has no end time; Codex "done" is `turns_complete`). **Accepted**: the
   third section is `recent` = every other row with activity in the window, using W5's phrases; no end time invented.
   Board 16 C relabelled `recent (2h)`.
2. **Nesting could duplicate a child or pick an old parent.** **Accepted**: G1 uses the existing `parent_links` rule
   (latest placing link, then index), a child appears once, and an absent or other-repo parent leaves it top-level.
3. **No place for ambiguous or unlinked hand-offs.** **Accepted**: they are never children; they show on the parent as
   `N possible runs` / `no codex run found` (W5, W8) through a `handoffs` field.
4. **Idle and unknown sessions had no section; `seen` was vague.** **Accepted**: they go in `recent`; `seen` means
   found in the data; an unseen session gets a pinned placeholder row; imported rows are left out as the TUI does.
5. **Change suppression compared clock fields.** **Accepted**: compare the object minus `now` and time phrases, with a
   30 s heartbeat; tests split "only time moved" from a real change.
6. **The API check was too narrow and `-p` is not interactive.** **Accepted**: 16.0 is a spike in an interactive PTY
   session covering session id, pane redraw, command, long-lived spawn and its kill on unload, and the test kit's
   surfaces; 16.3's surface list follows what it finds.
7. **Parent-pid alone is not session cleanup.** **Accepted**: the mod ends the spawn loop and cancels retry timers on
   `session.end` and unload; the pid check stays as a backstop (and covers a parent pid of 1 at start); tests for
   unload during streaming and during backoff.
8. **Answer-key commands were not reproducible, and fixture mode skips git.** **Accepted**: PROGRESS lists each key's
   full command; 16.1 adds a test that builds a temporary git repo with a worktree.
9. **The refresh path was unspecified.** **Accepted**: the pane runs the TUI worker's exact refresh with one shared
   `now`, and 16.2 measures that exact pass against the 250 ms budget (over → stop and plan).
10. **Overflow undefined when working rows alone overflow.** **Accepted**: header, this session (with up to 3
   children) and footer are always kept; other rows drop from `recent`, then `waiting`, then `working`; `+N more`
   counts sessions; display-width cuts; tests at small sizes and a working-only overflow.

### 2026-11-11 — Step 15.2 input type (Claude, Codex blocker, Codex loop)
- Codex stopped before coding: rendering from `multi_link_core::LoadSnapshot` needs `multi-link-core` in
  `crates/tui/Cargo.toml`, outside 15.2's files. **Rejected the manifest change**: `multi-link-tui` depends only on
  `multi-link-model` and ratatui, and the CLI already translates core results into tui-owned types (`app::Refresh`).
  Pulling core (scanner, readers) into the presentation crate for three numbers would break that layering. Instead
  `loading.rs` defines its own plain input (`LoadPhase` enum `Listing | Reading | Linking | Done`, `files_done`,
  `files_total`); in 15.3 the CLI maps `LoadSnapshot` (phase string) to it each frame. F5's "pure function of (time,
  progress, size, theme)" is unchanged; 15.2's Files are unchanged.
- **Listing counter (review).** F5 names no counter for phase `listing` (totals hidden as 0 / 0 by 15.1). Codex drew
  `listing transcripts…` with an empty bar. **Accepted**: it says what is happening, where `reading transcripts  0 / 0`
  would claim a total of zero before one exists. No golden covers it; a test pins the text.

### 2026-11-11 — Step 15.1 F1 amendment (Claude, during review, Codex loop)
- **F1 now also prefetches Claude desktop-app records** (`multi_link_claude::app_record_paths` /
  `read_app_record`, the `app_records` cache). Measured on real `~` after the first 15.1 implementation: 166 records,
  87 MB, read serially in 544–554 ms per cold pass (scratch harness, 3 runs), outside F1's list and about a quarter of
  the 2.0 s budget; F3 (b) was 1842 / 1881 ms medians, a 6 % margin. Same rules as transcripts: stamp before the read,
  failures uncached, the serial loop the only reporter (a `Bad` read stays a bad record there); they count in
  `files_total`. Why: same per-file hot spot, same file, no new risk. Not taken here: the all-time pass's serial
  `git rev-parse` per new cwd (~390 ms cold, `RepoMode::map`), which is repo lookup, not parsing; a lever for 15.1a if
  F3 ever misses.

### 2026-11-11 — Step 15.0 amendments (Claude, while writing the goldens)
- **F5 centre.** "Centre (W/2, art height/2)" puts the 76×22 centre at row 7.5, but the board's `drawC` uses
  `cy = 7.2` at 15 art rows, so the 76×22 golden could not equal the board's frame (15.0 Verify). Now 0.48 × art rows
  (7.2 / 15), scaled like the radii.
- **F6 to-do bar cells** are `border`, not dim text: the board's track shade is rgb(40,44,52), `border` is #2A2F3A;
  `secondary` (dim text) would make the empty bar as bright as the hint.
- **F7 order** stated (tiny first): "width 30–59 or height 6–15" alone put 29×6 in the middle tier.
- **`loading-76x22-zero`** is phase reading with `0 / 0`, not linking: as linking it was byte-identical to
  `-linking` and never drew `0 / 0` or the T = 0 bar branch (adversarial review, 15.0).
- Everything else F5–F8 left open (rounding, speck wrap, draw order, tail roles, bar rounding, small-size rows, fade
  steps) is pinned in `docs/design.md` → Phase 15 decisions, "Pinned while writing the goldens".

### 2026-11-11 — Phase 15 plan critique (Codex, read-only, Codex loop)
Codex gave 10 points; 8 accepted, 2 in part.
1. F2 `Arc` cannot avoid clones where assembly moves fields out — **accepted in part**: F2 is now measurement-gated
   (only if clones still cost > 0.3 s cold / 50 ms warm after F1), clone only the moved fields.
2. Progress lifecycle (two readers, worker's all-time pass, root reset drops the handle) — **accepted**: one
   generation, total published once after listing both harnesses, handle set by the worker and kept across the reset,
   phases listing → reading → linking → done, monotonic tests.
3. Splash could hide a failed first refresh; terminal restore is CLI-side — **accepted**: error / disconnect end it
   at once with the error footer; CLI test that `q` returns without joining; PTY check in 15.4.
4. Retry policy for failed or changed files — **accepted**: results stored with the pre-parse stamp, failed parses
   uncached and their diagnostics dropped, the serial loop the only reporter; append/replace/delete/malformed/
   unreadable tests. Simulating a failure in the middle of a read is rejected (no seam without a fault-injecting
   reader); an unreadable file covers the error path.
5. One-thread mode is not an independent oracle — **accepted in part**: the committed answer keys (pre-change
   outputs) are the oracle; 1-vs-many and reverse-completion tests compare diagnostics and stats too; a
   `#[doc(hidden)]` thread-count constructor reachable from integration tests.
6. Board time and geometry mismatch (`drawC(t + 1)`) — **accepted**: goldens at `drawC(4.0)`, exact rows H-7/H-5/
   H-3/H-2 and radius scaling.
7. Continuous board shades vs discrete theme roles — **accepted**: F6 quantisation rules; truecolor, 256 and
   NO_COLOR style goldens.
8. Tiny-size coverage — **accepted**: boundary goldens and bounded-buffer tests (0×0 … 60×16, origin, big counts).
9. Fade vs timing; no runtime seam — **accepted**: deadline-based 250 ms fade, explicit timeline, no new timer (the
   existing 50 ms loop draws from the clock), skip key consumed once.
10. Gate does not prove the TUI's readiness — **accepted**: F3 budgets (a)–(d), worker path with and without
   `--live`, baseline recorded in 15.0, numeric warm rule.

### 2026-11-10 — Step 14.3 Files amendment (Claude, during the step, Codex loop)
- Codex stopped before coding: L9's per-session call cache needs the scanner's transcript list (core-private) and
  the reader crates (not CLI dependencies). Accepted. Options weighed: (a) cache inside `Scanner::live` — the worker
  already owns one `Scanner` and every other parse cache lives there, so this keeps L9's "cached in the CLI worker"
  meaning (the worker's state) with no API growth; (b) a public reader API plus CLI Cargo deps — more surface for
  no gain. Chose (a): Files gain `crates/core/src/live.rs`, `crates/core/src/scanner.rs`, `crates/core/tests/**`.

### 2026-11-10 — Phase 14 plan critique (Codex, read-only, Codex loop)
Quota check: `CODEX: OK` (31% / 44%). Ten points, each checked against the code; outcome:
1. Busy/stale Codex never have `state_since`, so stale order and `stale since T` had no data → **accepted**: wait /
   stale start = `state_since`, else `last_activity`; busy order uses the last burst's start.
2. Window semantics: cards come from the TUI's windowed pass and bursts are clipped → **accepted**: stated that a
   stale run older than the window is not a card; a clipped burst prints `for over <period>`.
3. Parent choice underspecified; `started_by` is Claude-only → **accepted**: latest non-ambiguous incoming link by
   `at`, ties by report order; no `started_by` fallback; cross-repo parents allowed.
4. G3 kinds need chain position; `back T` undefined; a resumed child could make an old hand-off say `running` →
   **accepted**: card line 1 is `last_asked` for both harnesses; `back T` dropped; outcomes defined from the child's
   turns, `running` only for the child's latest incoming link.
5. `graph::tally` drops `at` → **accepted**: 14.1 extracts a shared summary that keeps the latest `at`; graph JSON
   unchanged and checked; hand-offs on cards are `▸ codex exec` (+ step), no chain words.
6. Continuation rows had no sort key; movement undefined → **accepted**: groups (head + children), flattened order,
   `↑↓` by group head, `←→` within the group, fallback to the same flattened index.
7. Cache keyed on the main file misses subagent files → **accepted**: dependency set (main + subagents, with inode),
   as T1; cold, warm and changed-file timings in 14.4.
8. 14.0 changed footer goldens before 14.3's code; `phase9/check.sh` needs `6` → **accepted**: footer goldens move to
   14.3; every check command spelled out.
9. `codex_steps` is private in the CLI → **accepted**: 14.1 moves `codex_steps` / `started_steps` into core and adds
   `crates/cli/src/tui.rs` to its Files.
10. Small sizes untested → **accepted in part**: a 60×20 golden and display-width cutting are added, and a card that
   cannot fit is clipped with `↓ N more`; arbitrarily small sizes beyond that are not tested (no screen does today).

### 2026-11-10 — Phase 13 plan critique (Codex, read-only, Codex loop)
Quota check: `CODEX: OK`. Nine points; outcome:
1. C1 ignored control flow (`|`, `||`, `;` after a failed `cd`) → **accepted in part**: operator rules added (`||`,
   `|`, `&` → unknown; `||` before the `cd` → unknown). `;` is kept with the stated assumption that the `cd`
   succeeded: 10 real cases use it, and a failed `cd` to a literal path usually leaves no run in that folder.
2. `CDPATH` redirects plain relative names → **accepted**: only absolute, `./` and `../` targets resolve.
3. Brace expansion → **accepted**: literal grammar is an allow-list of characters.
4. Opaque commands and wrapped `cd` → **accepted** as a deny-list of folder-changing builtins plus "bare `cd`
   only". An allow-list of commands was **rejected**: real commands put `export`, `$(date)`, `timeout`, `cat <<EOF`
   between the `cd` and the launch, and external commands cannot change the shell's folder.
5. C3 precedence and unknown options → **accepted**: known flags listed; unknown option, `--`, `-` or redirect → no id.
6. `overview.rs` builds `Link` → **accepted**: added to 13.1's Files.
7. Resize check passes on a crash → **accepted**: startup and a zero exit status are required.
8. Final resize repeated the same size → **accepted**: the last resize goes to a third size. Proving the write
   stalled **rejected**: no portable signal; the 5/5 FAIL at 8d2cc22 shows the condition is reached.
9. K1 grouping must use the launch folder → **accepted**: fixture (i) uses different session folders converging.
Real effect of the stricter C1, recomputed (scratch, counts only): all 31 literal cases still resolve.


### 2026-11-10 — Step 12.0 amendments (Claude, while writing the keys)
- **K1 tightened.** Writing case (i) showed "h''s unique nearest" removes a run even when h' itself stays ambiguous and
  takes nothing, then can hand h a far run (critique point 2 only half closed). K1 now removes r only when r is h''s
  own choice (K2/K3 on its original list). Real data: identical transitions to the K4 table (simulated on the
  2026-11-10 export, counts only).
- **K3 words and graph edge.** With `candidates` = survivors, a K1 pick has an edge count of 1, so "nearest of N runs"
  could not come from it. Words are now `matched by time · nearest run`; the graph `Edge` gains `nearest_run: true`,
  omitted when false (no existing graph key changes). 12.1 Files gain `crates/core/src/graph.rs`.
- **K2 scope** stated: launch hand-offs only; R7 unchanged (was implied by K4 and the 12.1 Verify).
- **Files.** `crates/core/tests/keys-before-phase6.json` holds a copy of `links-amb-hidden.json`, so its amb-1 value
  changes with the key; added to 12.0's Files.
- **Found:** option-first explicit-id resume is not recognised as a resume (case h pins today's behaviour); backlog.

### 2026-11-10 — Step 12.1 amendment (Codex stopped on scope)
- `Edge` is defined in `crates/model/src/lib.rs` and built in `crates/model/tests/graph.rs`; adding `nearest_run` needs
  that test to compile. Accepted: 12.1 Files gain `crates/model/tests/**` (constructor updates only).

### 2026-11-10 — Phase 12 plan critique (Codex, read-only, Codex loop)
Quota check: `primary 0%, secondary 39%, CODEX: OK`. Ten points; outcome:
1. K1 can give a delayed background run to a failed later launch → **accepted in part**: stated as an assumption
   with that counterexample in K1. Kept, because today both hand-offs claim that run (duplicate attribution) and
   the link stays `matched by time`.
2. K1 drops a run even when the other hand-off does not take it → **accepted**: a run is removed only when it is the
   other hand-off's unique nearest original candidate with a strictly smaller gap; fixture (i).
3. Competitor eligibility → **accepted**: competitors are exactly today's creation-window hand-offs (no exact, no
   `dir_moved`, no `resume_id`, no id-less `resume --last`); fixtures (h), (k).
4. Calibration does not validate 5 s / 10 s → **accepted**: margins labelled provisional, the known failure named,
   a masked check on all 29 exact links run now (28 right, 0 wrong, 1 unlinked) and again in 12.2 as a stop gate.
5. Keeping losing runs in `candidates` changes its meaning → **accepted**: `candidates` = survivors after K1, the
   per-match counts written out, and the revision recorded in the schema doc (K0 no longer claims "additive only").
6. Graph and overview consequences → **accepted**: Phase 12 keys include `overview.json` and a `graph-*.json`.
7. `view.rs` missing; graph has no evidence → **accepted**: `view.rs` in 12.1's Files; the graph words come from the
   edge's candidate count, so `Edge` is unchanged; truncation tested.
8. Equal-gap fixture wrong → **accepted**: (f1) and (f2) with exact expected results; ms boundary fixtures (j).
9. Verify commands and invariants → **accepted**: commands named, `phase12.rs` in `cargo test`, order and checkout
   invariance, exact and R7 unchanged. History-window invariance **rejected**: links already use `--since all`.
10. K4 arithmetic → **accepted**: full transition table summing to 234 (118 launch heuristics, not 124; the other
    6 are `resume_turn`), marked descriptive, not an acceptance target.

### 2026-11-10 — Step 11.3 Files amendment (Claude, during the step)
The real check needs a record like `docs/phase9-check.md` (method, frames, board comparison) that the README links
to; `docs/phase11-check.md` added to the Files. Synthetic content only (W10): real frames are summarised, not pasted.

### 2026-11-10 — Step 11.2 Files amendment (Claude, during the step, Codex loop)
Codex stopped before writing code: `Refresh` is built with exhaustive struct initializers in TUI tests outside
`crates/tui/tests/graph.rs` (`started.rs:179`), so adding `graph_live` must touch them. Files widened to
`crates/tui/tests/**`; only initializer additions are allowed outside `graph.rs`.

### 2026-11-10 — Phase 11 plan critique (Codex, read-only, Codex loop)
Quota check: `primary 20%, secondary 35%, CODEX: OK`. Ten points, all accepted (two in part):
1. "Latest part + Busy" marks an old step live when its session is busy on another step → **accepted**: a Claude part
   also needs its `to` within 5 min of now (L1); test an old step whose parent is busy elsewhere.
2. One global latest part suppresses another parent's work; "no commit" hides work resumed after a commit →
   **accepted**: L1 is per session (last part of its session, ties by index); the commit condition is dropped (the
   5-min rule ends old steps).
3. `Working` on a run window superseded by a resume → **accepted**: a Codex part must also be its child's last part;
   live is tracked by part index.
4. TUI has no core dependency and `Refresh` has no sessions → **accepted**: the CLI worker computes L1 and sends
   `Refresh::graph_live`; `crates/cli/src/tui.rs` and its tests added to 11.2 (L7).
5. `Axis::col` is `None` in folds; zero-width end segment; fitted end is `min(now, window.end)` → **accepted**: N1 now
   names a presentation-only resolver (end → last cell, fold → its last cell).
6. Label can overflow near the right edge; end-label priority unclear → **accepted**: N3 shifts it left inside the
   axis and replaces the end clock when now is the end; tests that expect `now` to vanish are updated and named.
7. Paint collisions undefined → **accepted**: N2 fixes the order (after links, before lanes) and the row range.
8. Visibility must match slots; narrow badge can be clipped; no narrow off-screen rule → **accepted** (L4, L5).
9. Dimmed harness word on `graph_selected` is 3.06:1 / 3.61:1, under 4.5:1; grey boxes would turn harness-coloured →
   **accepted in part**: only borders and `●` dim (non-text, 3:1), harness word and text keep today's colours, grey
   boxes stay grey. This is a departure from board 11d, which dimmed the harness word; 11.3 compares it.
10. Verify gaps: no live snapshot, screenshot example not authorised, no transition check → **accepted in part**: new
    `repo-now` and `step-live` snapshots, `home_html.rs` authorised, 11.3 runs plan → implement → review → fix → idle.
    `step-phase8` must stay unchanged (graph-1.1 has no live part).

### 2026-11-09 — Phase 10 plan critique (Codex, read-only, Codex loop)
Quota check: `primary 6%, secondary 33%, CODEX: OK`. Ten points; outcome:
1. `cat > T` can leave half-written JSON → **accepted**: H6 writes a `mktemp` file and `mv`s it onto the resolved
   target (atomic, symlink kept, `chmod` keeps the mode).
2. Fixed temp names collide; lost updates → **accepted in part**: `mktemp` for temp and backup, absent-file branch
   checks `[ ! -e ]` before `mv`. **Rejected**: a lock or source hash check. No lock is shared with Claude Code, and
   the window is milliseconds on a single-user file; documented as a limit instead.
3. Replacing entries drops user fields (`if`, `shell`, `args`) → **accepted**: such entries are `customized`, never
   changed, named for a hand edit.
4. Numbers and duplicate keys silently change → **accepted**: refuse (exit 2) on the source text. Not
   `arbitrary_precision`: it changes `Number` equality workspace-wide and could break answer-key comparisons.
5. Shell quoting and file privacy → **accepted**: absolute paths, the existing `'\''` quoting, newline paths refused,
   `mktemp` 0600, tests for space, apostrophe, `$(…)`, backtick, leading `-`.
6. `--merged` exit 1 would stop the `&&` chain; stdout must be JSON only → **accepted** (H7).
7. `""`/`"*"`/absent matchers are equal; a narrower matcher is intent → **accepted**: equal forms normalised; other
   matchers are `customized`.
8. One file cannot prove hooks are active; `CLAUDE_CONFIG_DIR` → **accepted in part**: `CLAUDE_CONFIG_DIR` honoured,
   output names the file, exit 0 wording limited to that file, `disableAllHooks` reported. **Rejected**: reading other
   settings levels and managed policy (non-goal; out of proportion to the need).
9. Verify misses failure cases → **accepted in part**: failing merge (stub binary), non-regular, dangling and
   unreadable targets added. **Rejected**: a concurrent-run test (not deterministic; the race is documented).
10. 10.2 proves agreement with the checker, not running hooks → **accepted**: 10.2 Verify needs a post-merge session
    that ends `finished` / `session_end`.

### 2026-11-08 — Step 9.4 scope amendment (Claude)
Codex stopped before coding: R5 clears an `ended` record only on a later main-transcript *turn*, and `Transcript`
exposes no turn timestamps (`activity_timestamps` and `last_activity` also count meta records, `/exit` echoes and
subagent activity, so using them could clear a real `/exit`, whose echo records follow the `cost-state`). Accepted:
Files gain `crates/claude/src/lib.rs` and `crates/claude/tests/**`, limited to one new field with the last turn's
timestamp (turn as defined at 9.0). Rejected the alternative (re-reading the transcript in the core hook reader):
a second scan of every file for one value.

### 2026-11-08 — Step 9.6 / R7 critique (Codex, read-only, Codex loop)
Owner added R7 and Step 9.6 to Phase 9 after the step 8.7 screenshot. 8 points; 7 accepted, 1 in part. Accepted:
(1) the graph already cuts Codex parts by turns between hand-offs (G4), so the G2 "one part = one turn" amendment is
dropped and linking alone must split step 8.7; (2) `--last` is classified as a resume, ignores exact-launch claims
and never falls back to the creation match; (3) turns counted on the non-imported list in both readers; (4) R0 and
the schema amended: `resume_turn` is allowed on a heuristic link, `gap_ms` = turn start − hand-off; (6) the panel
rule stays, with exact assertions; (7) `crates/claude/tests/**` and `crates/cli/tests/**` added. In part: (5) the
10 s window stays but is documented as uncalibrated, with boundary tests, and 9.7 counts more real resumes;
(8) key tests are named and must fail with resume matching off; no generated allowed-diff tool (existing key
tests already fail on any other change).

### 2026-11-08 — Phase 9 plan critique (Codex, read-only, Codex loop)
10 points; 8 accepted, 2 in part. Accepted: (1) `SessionEnd` must not outlive a resume: an `ended` record counts
only with no later turn or hook record, and `process_exit` stays ahead of it (R5). (2) Redirect reads: regular
files only via `fstat` on the opened file (a FIFO would hang the refresh), containment checked on the opened file
(R4). (4) `core/src/sessions.rs` added to 9.2 and 9.4. (5) 9.3 adds `model` (`RedirectFile`) and `core/src/hook.rs`
(the reader rejects unknown evidence, so persisted links would be lost). (6) The opener had no entrypoint or app
mapping: an internal core resolver, no Session keys; `sdk-cli` resumes, unknown/ambiguous → `unavailable` (R3).
(7) One `.zst` opener for discovery, `scan` and `read_details`; failures go to `unreadable`; the cap is `.zst`
only; `Cargo.lock` added (R1, 9.1). (8) Fixture mode trusts records, so SIGKILL is a core test with
`FakeProcessProbe::Gone`. (10) R6 keeps `own_step`'s same-repo and latest-tagged rules, tested worker to App.
In part: (3) `>>` reuse and multiple redirects: exact needs exactly one `session id:` line and one stdout
redirect, and a relative path after `cd` is not read; full shell semantics (subshell redirects, `exec >`) are
not modelled, they simply give no exact link. (9) Verify now compares parsed JSON with `cargo run` and a fixed
`--now`, and only Claude edits keys; an explicit per-value allowlist is kept as 9.0's PROGRESS list plus
`git diff --stat`, not a generated diff tool, because the existing key tests already fail on any other change.

### 2026-11-08 — Step 8.7 Files amendment (Claude, during the step, Codex loop)
Codex stopped twice, both times correctly. (1) My hand-off said "regenerate goldens only via the existing update
mechanism", but the pinned phase6/phase8 goldens have none; the step already allows them to change in clock cells, so I
withdrew that rule, kept the step's diff greps as the guard, and said never to hand-edit golden content. (2) `app.rs`
in-file tests assert `21:54` / `22:25 now` on the repo axis; added `crates/tui/src/app.rs` to Files for those clock
expectations only.

### 2026-11-08 — Step 8.5 review rounds 1–2 (Claude, adversarial subagents, Claude-only mode)
Round 1, 0 Critical / 2 Important / 7 Minor. Accepted and fixed: the coordinator list read the overview's windowed
`children`, so children quiet for 2 h vanished (fix: the worker sends all-time `started_steps`, same rule); the cache test
passed with the fix reverted (fix: counter is `Option`, warm `Some(0)`, one touched file `Some(1)`); `s` during a pending
open; a Codex lane took a step from another repo's parent. Rejected (parked, reasons in PROGRESS): rapid `[ ]` builds each
intermediate graph (~230 ms, last wins); keys before the first graph after a failed refresh (`esc` works, error shows);
the all-time pass runs twice with a graph open (35 ms; removing it needs a core API change); `q quit` hidden at 101–109
columns (the footer already cuts from the end below 120). Round 2: the four fixes ADDRESSED; one new Important parked at
the 2-round cap: the Codex-lane fix checks the windowed group, so a Codex lane whose parent is outside the window says
`no step on this session`. For 8.6's README as a known limit; a later fix sends per-Codex-id steps from the worker.

### 2026-11-08 — Step 8.5 Files amendment (Claude, during the step, Claude-only mode)
G11 measured first: warm `Scanner::graph` 4.7 s on real step 8.4, 4.5 s of it a fresh `multi_link_claude::scan`. 8.3's
entry had flagged this ("a cached or per-file path needs the claude crate"). Fix in core instead: reuse the scanner's
file-key cache of parsed Claude files (no claude-crate change needed). Files gain `crates/core/src/{graph,scanner}.rs`,
`crates/core/tests/**` and `crates/tui/src/graph.rs` (make the `o`/`f` drawing rule shareable so keys cannot drift
from it). Readings pinned for the app: a Codex lane's step is its own latest tagged incoming hand-off; a started child
"has a graph" when it has its own tagged hand-off (its latest one is listed); `[ ]` walks PLAN.md steps with a tagged
hand-off from a Claude session in the repo (all time), then tagged ids not in PLAN.md by first hand-off.

### 2026-11-08 — Step 8.4c review round 1 (Claude, adversarial subagent, Claude-only mode)
9 findings, 0 critical. Accepted and fixed in round 1: 1 the plain-text `overview` count is one level deep
(`crates/cli/src/main.rs`; added to Files because S3 says counts walk recursively); 2 record-only sessions (no
transcript) got `started_by` (S1 names both ends by transcript id; now transcript-backed only); 3 the hand-off count
test compared two empty lists (now compares A's `made` and its Codex child's `handoffs`); 4 cycles were found after the
`--since` filter (now on the full record graph first); 5 deep nesting drew over the bars (indent clamped); 7 a record
deleted mid-scan counted as bad (not-found now passes through). Ruled, not code: 6 the repo view keeps W6's
newest-first order at every level; S3's keys decide order, the direction follows W6 (JSON stays ascending);
9 two records sharing one app id drop only links that name that id as parent; both records still link as children
(written into design.md S1). Parked: 8 `home::selectable` (exported, no callers, outside Files) still stops at depth 1.

### 2026-11-08 — Step 8.4b Verify amendment (Claude, during the step)
Codex's run showed the key check (`git diff c45fbcd -- fixtures/expected | grep …`) also reads the TUI text goldens
in `fixtures/expected/tui/`. The hook fixture has one session with a (scrubbed) `custom-title`, so U2 changes row 14
of `home-hook.txt` / `home-hook-full.txt` and their style files, which the grep then flags. The check's purpose is
JSON keys ("every committed key equal except the added `title`"); text goldens have their own rule. Amended: the grep
excludes `fixtures/expected/tui`, and the goldens rule now names the other home goldens too (changed rows listed in
PROGRESS). No goal or differentiator change, so no owner stop.

### 2026-11-08 — Step 8.4c plan (Claude, Codex critique, read-only)
The draft read the parent's `start_session` result plus the live record's `hostSessionId`; while planning, Claude
found the desktop app's own per-session records (survive exit, `cliSessionId` + `spawnedFrom`) and rebuilt S1 on them.
10 points, 8 accepted, 2 in part: 1 G0 amendment contradicts nesting (accepted: tree changes allowed only where
`started_by` is non-null; whole-phase pinned diff amended from 8.4a on); 2 conflicts and cycles (accepted, rules and
tests added); 3 `open.rs` builds a Session, `app.rs` bell walks one level (accepted, both added, recursion everywhere);
4 home has no tree (accepted: home unchanged, ancestry in home a non-goal); 5 connector has no `Link.at` (accepted:
separate ancestry mark at child `started_at`, sub line defined); 6 T11 ordering (accepted: Codex keys kept, started
children by `started_at`, parent state its own, quiet eligibility recursive); 7 diagnostics boundaries (accepted;
refresh tests trimmed to add/change/delete/cache reuse, root switching left out as not reachable from the TUI); 8
executable checks and keys first (accepted: `git diff | grep` check, Claude writes keys before the Codex hand-off,
W10, graph regression); 9 counts overlap (accepted: 59 eligible, 2 grandparents, cwd ≠ repo case); 10 graph part to
8.5 (accepted in part: G9 amendment with precedence, direct children only, in 8.5's text and Verify; the `started
by` graph header dropped, it would need a new report key).

### 2026-11-07 — Steps 8.4a/8.4b plan (Claude, Codex critique, read-only)
10 points, all accepted
(2 in part): 1 split and explicit G0 amendment (accepted); 2 U5 time comparison not implementable, PROGRESS has dates
only (accepted, comparison cut); 3 tags cannot prove done/running (accepted, wording is "latest hand-off"); 4 title must
reach SessionRow, overview, open, model tests (accepted); 5 busy duration has no source (accepted, duration text cut);
6 keep placing-parent rule, per-candidate preamble rejection (accepted); 7 U4 option boundary (accepted, defined;
`--version` dropped as already unrecognised); 8 title ownership and malformed records (accepted; second real session
confirmed as 47/60 transcripts instead of one fixture); 9 executable regression checks, `FakeProcessProbe` for Alive
(accepted); 10 snapshot comparison, W10 for titles, extra files (accepted in part: CLAUDE.md is the coordinator's
change, not the step's).

### 2026-11-06 — Phase 8 plan (Claude, Codex critique, read-only)
Owner approved board 9 and did not answer what the view must answer, where it lives, or replay; Claude's defaults are
marked in the phase. Claude measured real hand-offs, rollouts and Codex record shapes first. Codex checked G0–G11 and
8.1–8.6 against the code. 9 accepted, 1 accepted in part.
| # | Finding | Decision |
| --- | --- | --- |
| 1 | P1: G4 bursts lose or merge resumed runs; status and `asked` are per session | Accepted: runs come from Codex turns (`codex_turns`), per-part status, prompt, message; two-resume fixture |
| 2 | P1: one chain per parent cannot show A → B → A | Accepted: another step's hand-off splits a chain; an untagged one does not (G1 is the evidence rule) and is counted in the panel |
| 3 | P1: an exact link may have no rollout | Accepted: `run_available` separate from link confidence; `run unavailable` part keeps the edge |
| 4 | P2: Claude parts cover silent Codex runs; return may fall outside any burst; naming precedence open | Accepted: Claude parts are the time outside runs; chain-end and precedence rules written; silence and overlap tests |
| 5 | P2: allow-list still copies Verify commands and three words | Accepted in part: notable calls are fixed category labels (`$ cargo test`), sentinel tests. Rejected for the header: its Verify text is the repo's own PLAN.md, not transcript text |
| 6 | P2: Claude call ownership undefined | Accepted: W1's ownership, subagent files, dedupe by tool id, results by id |
| 7 | P2: report lacks stretches; strip with overlaps | Accepted: `active_ms`, `stretches`; strip defined as shares of summed lengths |
| 8 | P2: TUI does not depend on core | Accepted: report types in `crates/model`, added to 8.3 Files |
| 9 | P2: rebuilding only on file change freezes clock-driven state | Accepted: rebuild every refresh, cache parsed calls only; 8.5 measures the whole refresh |
| 10 | P3: fixture ids invalid, CLI Verify not executable | Accepted: numeric ids, fixed `--now`, an integration test over every key |

### 2026-11-02 — Phase 7 plan (Claude, Codex critique, read-only)
Owner chose "Polish leftovers first". Codex (`codex exec -s read-only`) gave 7 points:
1. P4 priorities contradicted themselves (end label first, then dropped for a fold length). **Accepted:** P4
   is now a placement order (fold lengths, end label in shrinking forms, interior now, ticks).
2. "The header still gives the time" is false in fitted mode (header shows the activity range). **Accepted
   the fact, kept the rule:** P4 says the clock can disappear and why that is acceptable; a header change is
   out of scope.
3. `<system-reminder` prefix also matches other tags; "matching" undefined for nesting. **Accepted:** exact
   tag, first closing tag, no nesting, plus a negative test.
4. 7.2 render tests cannot prove the app passes the window. **Accepted:** App-level and `home_html --since`
   tests added.
5. P4 Verify missed explicit-range cases. **Accepted:** tests for end-minute ticks, interior now, range
   ending before now.
6. Real count had no command or baseline. **Accepted in part:** jq command and baseline `[181,43]` at 26af6f2
   added; a frozen scratch copy of the transcripts is rejected (overkill; changes are explained instead).
7. "Byte-identical to HEAD" moves with each commit. **Accepted:** pinned `git diff --exit-code 26af6f2`.

### 2026-11-02 — Step 6.7 review (Claude, diff + Step 6.7)
Phase6 pair unchanged; three tests present; real route-impact view shows `✓ done` after the 16:02 Codex run.
| # | Finding | Decision |
| --- | --- | --- |
| 1 | P2: clearance checked 6 cells for the 7-cell `" ✓ done"`; at width 120 the `e` could land at col 118, outside the bar area | Accepted: check x+1..=x+7; tests moved (fit end=66, fallback end=67) |

### 2026-11-02 — Step 6.6 review (Claude, diff + Step 6.6)
All X1–X4 shapes tested (incl. the route-impact launch with a synthetic brief); 6.1 goldens unchanged; W10 clean.
Real data: launches 124 → 149; 2h Codex `asked` null 1 → 0; the route-impact 16:02 run nests under its parent.
| # | Finding | Decision |
| --- | --- | --- |
| 1 | P2: `docs/output-schema.md` still said older Codex fixtures get `asked` null and named only `user_message` | Accepted: file added to Step 6.6's Files; both passages describe X4 |
| 2 | P3: `((a << 2))` reads as a heredoc; an unquoted `#` comment with `'` breaks quote tracking | Kept as known limits (outside X1–X4), noted in PROGRESS |

### 2026-11-02 — Step 6.5 review (adversarial Claude subagent, diff + Step 6.5 + W0–W10)
The reviewer re-ran `cargo test` (35 ok) and clippy (clean), checked README claims against the code and the counts
in docs/phase6-check.md against each other, and looked at both images (synthetic only). 0 P1, 2 P2, 5 P3; all accepted.
| # | Finding | Decision |
| --- | --- | --- |
| 1 | P2: example `parse_since` panics on `2ч` and on overflow, rejects `90s`, unlike the CLI it claims to follow | Accepted: the CLI's rule (ASCII digits, `s/m/h/d`, `checked_mul`, same message) |
| 2 | P2: README and phase6-check say folds show their length; difference 5 says the real 13 h fold's label is not drawn | Accepted: "when there is room"; the real-data line says the band draws, the label does not |
| 3 | P3: README fallback label written `no prompt`; the screen says `claude · no prompt` | Accepted |
| 4 | P3: "first row selected, as the TUI opens it" is only true from the repo's title line | Accepted: wording in the example and the doc |
| 5 | P3: `--repo NAME` silently takes the first of several matching repos | Accepted: exact path wins; several suffix matches is an error listing them |
| 6 | P3: doc does not say the example's repo render needs `--since 16h` too | Accepted |
| 7 | P3: "A card for each repo that needs you" also covers working/stale-only repos | Accepted: "with a waiting, working or stale session" |

### 2026-11-01 — Step 6.1 review (adversarial Claude subagent, diff + Step 6.1 + W0–W10 + board 7c/8c descriptions)
Claude also clarified W1 while writing the keys: the Codex `session_meta` header is not activity (in an import it carries
the copy time, so every import would draw a fake one-cell burst). The reviewer recomputed all 13 sessions' four fields
from the fixture lines and every golden cell from the written rules: all matched. 0 P1, 3 P2, 6 P3; no key or golden changed.
| # | Finding | Decision |
| --- | --- | --- |
| 1 | P2: schema said old fixtures' `asked` is null; `CASE-<name>: <prompt>` prompts don't start with `<` | Accepted: schema states which old sessions get a string and which get null |
| 2 | P2: W1 counted inherited history (resumed/forked files) as activity | Accepted: Claude records with another `sessionId` are not activity (as W3); W1 and 6.2 Verify updated |
| 3 | P2: W5 never listed which phrases are relative (`started 21:29` could render `56 min ago`) | Accepted: only `last active …` is relative; every other time is a clock time |
| 4 | P3: legend colours ambiguous (`◆ waiting on you` all yellow; `│ ╎ ≈` colour unstated) | Accepted: rule states both |
| 5 | P3: hidden-imports time source, idle `last active` source, quiet-line position, "first text block" unpinned | Accepted: all four pinned in design.md / schema |
| 6 | P3: 2 of 140 real all-import rollouts have an `event_msg` outside every turn → non-empty activity | Accepted: an `imported` rollout has `activity` `[]` and `asked` null; 6.2 test added |
| 7 | P3: PLAN.md W1 edited outside Step 6.1's Files, not logged | Accepted: this entry records it (Claude owns PLAN.md) |
| 8 | P3: shape test counts chars, not display width; accepts CRLF; doesn't check the four fields | Partly accepted: CRLF rejected and four fields checked; width rejected (no width table in deps; 6.3/6.4's cell-for-cell compare catches a wide glyph) |
| 9 | P3: canvas artifact id (22 chars) trips the W10 "long id" check | Accepted: recorded in design.md as a W10 exception (owner's private canvas link, not a session id) |

### 2026-11-01 — Phase 6 plan (Claude, Codex critique, read-only)
Owner rejected Phase 5's bars on the real screen and approved boards 7c (home in words) and 8c (board 2's
swimlanes on real data). Claude measured the burst rule (2 of 117 real turns split at 5 min) and the import
marker (140 of 260 rollouts) before planning. Codex checked W0–W9 and 6.1–6.5 against the code. All 10 accepted.
| # | Finding | Decision |
| --- | --- | --- |
| 1 | P1: 6.2 cannot compile; `model/tests` and `core/src/open.rs` build `Session` literals | Accepted: both added to 6.2 Files |
| 2 | P1: dropping imports in core leaves nothing for `i`; quiet line and zoom need data the window drops | Accepted: imports stay in every report with `imported`; the TUI hides them; quiet line lists only repos the overview lists; zoom stays inside the report window |
| 3 | P1: changing `last_activity` and hiding rows breaks the JSON contract | Accepted: W0 keeps every existing value; the TUI filters; only four additive fields |
| 4 | P1: W5 needs times the model drops; Codex "finished" overclaims | Accepted: `state_since` from the evidence `state` uses; Codex `done` defined as last turn complete; `working now` without a burst |
| 5 | P2: imported-turn boundaries unspecified | Accepted: file-order state machine in W2 with close rules; malformed, interrupted and appended-to-cache tests |
| 6 | P2: first-prompt rule fragile; "no `?`" fails on real prompts | Accepted: `sessionId` filter, text blocks, trimmed `<` check, blanks, char-boundary cut; rule is no generated `?` placeholder |
| 7 | P2: privacy checks come too late; boards hold real prompts | Accepted: W10 from 6.1 on; synthetic fixture prompts; boards described, not pictured; every step reviews committed files |
| 8 | P2: bursts underspecified; "worked" overclaims | Accepted: sources incl. subagents, sort/dedupe, `[t, t]`, clipping; screens say "active" |
| 9 | P2: axis rules contradict pan/zoom and can overflow | Accepted: fitted vs explicit range, longer tick ladder, at most 3 five-column folds with the length in the axis row, empty-window line |
| 10 | P2: acceptance checks not executable; old tests encode removed design | Accepted: 6.1 shape test; 6.3/6.4 retire Phase 5 home and timeline goldens by name and keep behaviour coverage; selection/refresh tests added |

### 2026-10-30 — Step 5.1 review (adversarial Claude subagent, diff + Step 5.1 + board 7 HTML)
The reviewer checked by script: the styles file covers all 4800 cells; every bar cell agrees with the mapping;
columns, counts and local times are right; the CLI output equals both keys apart from `made`/`hook`; the
fixture has every shape item; the contrast table holds; nothing personal is in the files. 10 findings, all accepted.
| # | Finding | Decision |
| --- | --- | --- |
| 1 | The waiting lane's ` · ` is yellow; the docs said only `waiting since …` is yellow | Accepted: the docs now say the separator is included (the golden is unchanged; the phrase reads as one unit) |
| 2 | No precedence for border, tint and title glyph when a card has waiting and busy sessions | Accepted: they follow the group `tier` (waiting > busy > other) |
| 3 | Which times sort a combined lane and put it in the window | Accepted: sorted by the Claude row's `started_at`; in the window when either row's `last_activity` is |
| 4 | Source of `waiting since <time>` not stated | Accepted: the row's `last_activity` |
| 5 | Verify colours ambiguous | Accepted: ✓ grey, ✗ red, unclear grey |
| 6 | Plan wording overflowing into the counts undefined | Accepted: cut with `…` on a word boundary, at least 3 blanks kept |
| 7 | The title counts leave out done/unknown, but the idle line counts them | Accepted as intended, now written down (those lanes are on screen) |
| 8 | Board-7 differences list incomplete | Accepted: confirmed vs matched, Codex-only idle repo, no pill, hook note wording, lighter track added |
| 9 | Track `#1A1D24` nearly invisible on the card (1.08:1) | Accepted: track `#22262F` (1.21–1.26:1; dim bars on it 2.88 / 3.40); golden regenerated |
| 10 | `made` key order in the key differs from the schema | Accepted: the key now puts `made` before `children` |

### 2026-10-30 — Phase 5 plan (Claude, Codex critique, read-only)
Owner approved board 7 and asked to prioritise UX/UI, readability and colour. Codex checked 5.1–5.5 against
the code. Verdict: revise before implementing. All 10 accepted.
| # | Finding | Decision |
| --- | --- | --- |
| 1 | P1: the window and 8-row cap could hide an overnight waiting session | Accepted: V3 always shows waiting/busy/stale rows outside the cap; overflow line keeps the rest reachable; yesterday's waiting row in the 5.1 fixture |
| 2 | P2: filtering/capping in core breaks the JSON contract | Accepted: V0, presentation in a TUI view model; JSON additive only; 5.2 tests existing keys minus new fields |
| 3 | P2: hook presence is not "installed" or "needs-you off" | Accepted: V8 reports observed records; waiting count always shown; "partial: no hook records seen" only at zero waiting |
| 4 | P2: 5.2 lacked `scanner.rs`, which owns window and hook records | Accepted: added to Files |
| 5 | P2: one lane per step is not guaranteed by placement | Accepted: V2 combines only on one child with matching (or null) step; `o` opens Claude, `enter` splits |
| 6 | P2: a child can run past its parent's span | Accepted: independent mapping, overlap rule, fixture child ending after its parent |
| 7 | P2: 46-column labels cannot fit at 60 columns | Accepted: V10 width tiers, field priority, ellipsis, scrolling; renders at 5 widths and 10 rows |
| 8 | P2: colour readability untested; golden format dropped dim/reversed | Accepted: V4 explicit pairs, 4.5:1 contrast test, NO_COLOR track; golden records every modifier |
| 9 | P2: local time underspecified; `time` lacks `local-offset` | Accepted: V7 rules; Cargo.toml in 5.4; offsets −7/0/+5:30 and midnight tests; DST limitation stated |
| 10 | P2: idle summary could claim states the data lacks | Accepted: V9 uses real counts and existing plan wording; stale/FR17 repos never collapse |

### 2026-10-30 — Step 4.6 review (adversarial Claude subagent)
| # | Finding | Decision |
| --- | --- | --- |
| 1 | P2: "worktree was removed" was not yet true | Accepted: removed with `git worktree remove` before commit |
| 2 | P2: non-blank counts called "full length"; blank separator rows hide real height (31 → 18, 262 → 46) | Accepted: heights added; README says "non-blank" |
| 3 | P2: classifier counted the `↓ N more lines` hint as a session row (35 → 34) | Accepted: number fixed, rule's leak noted |
| 4 | P2: an earlier 200-row trial's "18 lines" had no kept output | Accepted: number dropped |
| 5 | P3: "~150-line" script is an estimate (126) | Accepted: exact count |
| 6 | P3: clippy log was a cached run without exit status | Accepted: re-run uncached, exit recorded |
| 7 | P3: unpinned links file identical to pinned after file | Accepted: stated in the doc |
| 8 | P3: bars also need height ≥ 4 | Accepted: README says so |
| 9 | P3: emulator limits (one column per char, no scroll regions, chunked UTF-8) | Accepted: listed as method limits |

### 2026-10-30 — Step 4.5 review (Claude)
| # | Finding | Decision |
| --- | --- | --- |
| 1 | Deviation: shared axis labels drawn on their own line under the U4 summary, not on the header line | Accepted: at 100 columns the summary already uses ~92 columns (`now` dropped), so labels there would cut the counts U4 exists for; costs one body row |
| 2 | P3: lane text is cut one cell before the bar column, so long hand-off metadata clips at 100 columns (two existing tests moved to 160 columns) | Accepted for now; 4.6's real pty captures decide whether it hurts the need |

### 2026-10-30 — Phase 4 plan (Claude, Codex critique, read-only)
Codex (`codex exec -s read-only`) checked Steps 4.1–4.6 against the code. Verdict: revise before implementing.
| # | Finding | Decision |
| --- | --- | --- |
| 1 | P2: tokeniser drops quoting, so `'$PWD'` / `\$PWD` (literal) look like `"$PWD"` | Accepted: U1 keeps per-word expandability; negative cases added to 4.1 |
| 2 | P2: tier `other` includes stale-only groups (unfinished work) | Accepted: U3 collapses only groups with no `stale` session; test in 4.4 |
| 3 | P2: 4.2's "fails only on overview keys" is wrong (model round-trip, snapshots, counts) | Accepted: 4.2 adds an isolated `fixtures/sessions-phase4` root; existing keys change only in 4.3 |
| 4 | P2: new fixtures break hard-coded counts outside 4.3's files | Accepted: isolated fixture root (as #3); 4.3 Files list widened |
| 5 | P2: 4.1 and 4.2 both run the workspace gate in parallel | Accepted: 4.1 verifies crate-local; coordinator runs the workspace gate |
| 6 | P2: App restores a merged selection as (repo, None), so two merged rows swap on refresh | Accepted: 4.3 owns `app.rs`, identity is (repo, harness), with a test |
| 7 | P2: overview keeps only the latest hand-off per group, so it can't give a hand-off count | Accepted: U4 counts `Refresh.links` |
| 8 | P2: merged-row bars undefined; "remaining width" breaks the shared axis | Accepted: no bar on merged rows; fixed bar column; test in 4.5 |
| 9 | P3: cutting 4.5 depended on 4.6, which depended on 4.5 | Accepted by dropping the cut: 4.5 is in scope (bars are what the owner missed most) |
| 10 | P3: two live scans differ; captures don't prove the need | Partly accepted: pinned `--now`, back-to-back runs and an owner confirmation stop. Rejected a frozen scrubbed copy of real `~` (hundreds of MB of private transcripts; fixtures already give the controlled check) |

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
