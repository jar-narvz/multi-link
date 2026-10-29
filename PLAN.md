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

### Phase 2 — Hook — to be split later
- multi-link never writes to `~/.claude/`, including settings. `multi-link hook install` **prints** the
  settings snippet and the file to paste it into; the owner pastes it. `hook uninstall` prints what to remove.
- The hook writes its records only to multi-link's own state folder (`~/.multi-link/`), never into
  `~/.claude/` or `~/.codex/`.
- Forced by Phase 0 (b): the hook is the only source of "waiting on me" (Claude Code's `Notification`
  hook, to confirm), so splitting this phase must add a step and a Verify for it: a sandbox session that
  hits a permission prompt shows `waiting` with evidence `hook`. Codex waiting stays unsupported.
- Verify: for one real hand-off, the child id the hook records equals the id read independently from
  `~/.codex/sessions`; a hand-off without an observable id stays heuristic.
  The test hand-off runs in `~/multi-link-sandbox/repo-a` with the hook in that repo's own
  `.claude/settings.local.json` (project scope, not `~/.claude/`), so Claude can run it unattended.
  Enabling the hook for the owner's real repos stays the owner's paste.

### Phase 3 — TUI home + timeline + open (Release 1) — to be split later
- Forced by Phase 0 (b): needs-you sorting and the bell use only the hook's `waiting`; without the hook
  those sessions show as unknown, not as needing me.
- Incremental tailing (read from the last offset; truncation → re-read) is built here, where refresh
  speed matters; Phase 1 reads each file whole.
- Worktree labels need a `checkout` field on Session (the git toplevel of `cwd`). Phase 1 computes it
  (Step 1.5) but the schema doesn't emit it yet; add it to `docs/output-schema.md` here (Codex 0.6 #1).
- Verify: snapshot tests of both screens pass on fixtures; home refresh on real `~` is ≤ 2 s;
  opening a running fixture session never calls resume.

### Backlog (ordered, uncommitted)
Step graph · live grid · tmux / Herdr jump · desktop app with notifications · Claude Code mod.

## Review log

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
