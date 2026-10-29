# PROGRESS.md — multi-link

## Current state
- Step 0.1 (repo scaffold) verified, not yet committed or reviewed.
- No code yet. Next: Step 0.2, Phase 0 spike on real sessions.

## Log (newest first)

### 2026-10-29 — Step 0.1: Repo scaffold — Claude
- Did: Created project rule files, plan, README, gitignore, fixtures rules, and a pointer to the design doc.
- Changed: AGENTS.md, CLAUDE.md, PLAN.md, PROGRESS.md, README.md, .gitignore, docs/design.md, fixtures/README.md
- Verify: `git status --short --untracked-files=all` → `.gitignore`, `AGENTS.md`, `CLAUDE.md`, `PLAN.md`,
  `PROGRESS.md`, `README.md`, `docs/design.md`, `fixtures/README.md` (8 files, all `??`);
  `git check-ignore fixtures/raw/x.jsonl` → `fixtures/raw/x.jsonl`. Matches expected.
- Review: pending (Codex)
- Gotchas: Repo was empty on GitHub; work is on branch `setup/scaffold`, so `main` doesn't exist locally yet.
