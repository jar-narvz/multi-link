# PROGRESS.md — multi-link

## Current state
- Step 0.1 (repo scaffold) verified and committed on `main` (5a61cc1). Codex review still pending.
- No code yet. Next: Step 0.2, Phase 0 spike on real sessions (Codex).
- PLAN.md revised after a roadmap read-through: real step convention, print-only hook install,
  ambiguous fixture added to 0.2 (see Review log, 2026-10-29).
- Existing data for 0.2: Claude transcripts that invoke Codex exist in physaflow, route-impact,
  posting-triage, jarednarvaez-app (counted by grep for `codex exec|resume|codex-companion`, not yet read).
  Parallel/worktree, background, manual-run and stale cases may still need to be created on purpose.

## Log (newest first)

### 2026-10-29 — Step 0.1: Repo scaffold — Claude
- Did: Created project rule files, plan, README, gitignore, fixtures rules, and a pointer to the design doc.
- Changed: AGENTS.md, CLAUDE.md, PLAN.md, PROGRESS.md, README.md, .gitignore, docs/design.md, fixtures/README.md
- Verify: `git status --short --untracked-files=all` → `.gitignore`, `AGENTS.md`, `CLAUDE.md`, `PLAN.md`,
  `PROGRESS.md`, `README.md`, `docs/design.md`, `fixtures/README.md` (8 files, all `??`);
  `git check-ignore fixtures/raw/x.jsonl` → `fixtures/raw/x.jsonl`. Matches expected.
- Review: pending (Codex)
- Gotchas: Repo was empty on GitHub; work started on branch `setup/scaffold` (since landed on `main` as 5a61cc1).
