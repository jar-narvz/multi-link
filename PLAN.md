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

### Step 0.2 — Phase 0 spike: hand-offs and state signals (questions a, b)
- Copy real sessions into `fixtures/raw/` (git-ignored): 3 Claude → Codex hand-offs across 2 repos,
  including two parallel Claude sessions in one repo (one in a worktree), one `codex resume`,
  one background launch, one manual Codex run in the same folder, one stale session.
- Record in `docs/phase0.md`:
  - (a) for each invocation form (exec, resume, plugin, background): where the child Codex id is
    observable, with field names. Go / no-go for exact links.
  - (b) which fields give busy / waiting / idle / finished for each harness, and a proposed freshness limit.
- Files: `docs/phase0.md` only (plus ignored `fixtures/raw/`).
- Verify: `docs/phase0.md` has sections (a) and (b), each ending in `Decision: go` or `Decision: no-go`,
  with field names only (no conversation content).

### Step 0.3 — Phase 0 spike: dependencies (questions c, d)
- (c) Is zoetrope's core usable as a library crate (public API for parsing and graph model)?
- (d) Does agtrace already link a Claude session to the Codex run it launched? Test on the raw fixtures.
- Files: `docs/phase0.md`
- Verify: sections (c) and (d) exist, each ending in `Decision: go` / `Decision: no-go`, with the
  version tested and the command used.

### Step 0.4 — Scrubbed fixtures and hand-labelled expected outputs
- Scrub the raw sessions into `fixtures/` (rules in `fixtures/README.md`). Add a malformed-line case.
- Hand-write `fixtures/expected/{sessions,links,steps}.json`, including negative and ambiguous cases.
- Files: `fixtures/**` (not `raw/`), `fixtures/expected/*.json`
- Verify: `rg -l "/Users/|sk-|api_key" fixtures --glob '!raw/**'` prints nothing, and the three
  expected files parse with `jq . fixtures/expected/*.json > /dev/null`.

### Step 0.5 — Revise design from Phase 0
- Claude updates `docs/design.md` and this plan with the go / no-go results, then splits Phase 1 into steps.
- Verify: Review log below has a Phase 0 entry; Phase 1 has numbered steps with Verify commands.

### Phase 1 — Core + CLI (MVP) — to be split in Step 0.5
Parsers, repo mapper, state machine, step parser, linker; `multi-link sessions | links | steps`.
- Verify: `cargo test` passes; each command with `--root fixtures --now <fixed time> --json` equals its
  expected file; negative fixtures produce no link; the ambiguous fixture produces `ambiguous`.

### Phase 2 — Hook — to be split later
- Verify: for one real hand-off, the child id the hook records equals the id read independently from
  `~/.codex/sessions`; a hand-off without an observable id stays heuristic.

### Phase 3 — TUI home + timeline + open (Release 1) — to be split later
- Verify: snapshot tests of both screens pass on fixtures; home refresh on real `~` is ≤ 2 s;
  opening a running fixture session never calls resume.

### Backlog (ordered, uncommitted)
Step graph · live grid · tmux / Herdr jump · desktop app with notifications · Claude Code mod.

## Review log

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
