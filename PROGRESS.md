# PROGRESS.md — multi-link

## Current state
- Step 0.1 done, committed and reviewed by Codex (4 findings, all fixed).
- Autonomous loop set up (CLAUDE.md "Autonomous mode"): Claude commits locally after review, never pushes.
  Rust 1.99.1 installed; Codex sandbox smoke-tested building a crate with a crates.io dependency.
- `docs/design.md` now holds the full design (exported from claude.ai, repo copy is source of truth).
- PLAN.md revised: Step 0.2a (Claude generates known hand-offs in `~/multi-link-sandbox/`), fixtures
  split into 0.4 (Codex scrubs) and 0.5 (Claude writes schema + expected outputs), design revision is 0.6.
- Step 0.2a done: 10 generated cases with ground truth in `fixtures/raw/ground-truth.md` (stale case not made).
- Next: Step 0.2 (Codex).
- Existing organic data for 0.2: Claude transcripts that invoke Codex exist in physaflow, route-impact,
  posting-triage, jarednarvaez-app (counted by grep, not yet read).

## Log (newest first)

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
