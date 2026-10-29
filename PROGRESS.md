# PROGRESS.md — multi-link

## Current state
- Step 0.1 done, committed and reviewed by Codex (4 findings, all fixed).
- Autonomous loop set up (CLAUDE.md "Autonomous mode"): Claude commits locally after review, never pushes.
  Rust 1.99.1 installed; Codex sandbox smoke-tested building a crate with a crates.io dependency.
- `docs/design.md` now holds the full design (exported from claude.ai, repo copy is source of truth).
- PLAN.md revised: Step 0.2a (Claude generates known hand-offs in `~/multi-link-sandbox/`), fixtures
  split into 0.4 (Codex scrubs) and 0.5 (Claude writes schema + expected outputs), design revision is 0.6.
- Next: Step 0.2a.
- Existing organic data for 0.2: Claude transcripts that invoke Codex exist in physaflow, route-impact,
  posting-triage, jarednarvaez-app (counted by grep, not yet read).

## Log (newest first)

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
