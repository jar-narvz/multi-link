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
- Next: Step 0.5 (Claude writes schema and expected outputs). Phase 0 decisions so far:
  (a) go, (b) no-go for waiting, (c) no-go, (d) go.
- Organic inspection corrected the earlier grep inference: route-impact has Bash launch candidates;
  physaflow has textual mentions but no tool-use inputs containing `codex` in the inspected files.

## Log (newest first)

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
