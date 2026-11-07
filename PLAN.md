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

### Phase 8 — Step graph (Release 1.5)
Need (design need 4): for one PLAN.md step I can follow who did what, in what order, how many fix rounds it took
and how it ended, across Claude and Codex, without opening transcripts. The owner approved **board 9** on the
mockup canvas (claude.ai artifact R5Bh9iX6bsn3jSu8F8D58C) on 2026-11-06: board 3's step graph drawn on the
120×40 cell grid from real multi-link step 6.7. It is the acceptance picture; it shows real prompts, so it is
described, never committed as an image (W10). The planning session called this Phase 7; that number went to the
polish phase. **Owner's approval covered the board only.** Three questions got no answer, so these are Claude's
defaults (G1, G9, non-goals), cheap to change before 8.4: what the view answers (the need above), where it
lives (`s` from the repo view), replay (not in this phase).
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

### Step 8.3 — Core: graph report, CLI `graph` (Codex)
- Depends on: 8.2.
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

### Step 8.5 — TUI: `s`, moving along a step, refresh (Codex)
- Depends on: 8.4.
- App wiring and CLI loop per G9, G11.
- Files: `crates/tui/src/{app,timeline}.rs`, `crates/tui/tests/**`, `crates/cli/src/tui.rs`, `crates/cli/tests/**`,
  `fixtures/expected/tui/repo*.txt` (only the footer's `s` key; the 6.1 pair is unchanged if the key fits, else stop
  and say so), `PROGRESS.md`
- Verify: `cargo test --workspace` → all `ok`, clippy clean; app tests: `s` with and without a step, `←→` at both ends,
  `[` `]` over steps with no graph, `esc` restores repo selection, `o`/`f` on a done, running, unlinked and ambiguous
  part, a refresh adding a part keeps the selection, a refresh error keeps the screen, an unchanged transcript
  crossing the stale threshold changes the status, resize across 120 columns; the whole refresh on the largest real
  step measured against G11 (milliseconds in PROGRESS); sentinel strings absent from renders.

### Step 8.6 — Real check, README, owner confirm (Claude)
- Depends on: 8.5.
- Real renders of steps 6.7 (board 9's own), 6.6 (unlinked part) and one with a lost rollout, in scratch only (W10).
  `docs/phase8-check.md`: fixture render as an image, board 9 described, differences in words and counts. README:
  step graph section, G3 and G6 limits, missing rollouts. Then stop and ask the owner to run `multi-link tui`, press
  `s`, and confirm the need.
- Files: `docs/phase8-check.md`, `README.md`, `docs/img/**`, `PROGRESS.md`
- Verify: whole-phase Verify; W10 check over every committed file; the owner's answer recorded.

### Backlog (ordered, uncommitted)
Hook reads the background task output path (T10) · Lost key on resize: crossterm 0.29 drops tty readiness when SIGWINCH and a key arrive in one wake (upgrade or workaround; 3.8) · Resume for cleanly exited terminal sessions (their record is deleted on exit, so they show `unknown`; 3.8) · `@basename` label for a worktree named like its repo · live grid · tmux / Herdr jump · desktop app with notifications · Claude Code mod.

## Review log

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
