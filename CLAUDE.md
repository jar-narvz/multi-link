# CLAUDE.md — rules for Claude (planner and reviewer)

Read `AGENTS.md` first; its rules apply to you too. Design: `docs/design.md`.

## Your role here
- Own `PLAN.md`: it is the source of truth, and only you edit it.
- Plan one step at a time. Every step names its files and has a **Verify** command with an expected result.
- Review Codex's work: read only `git diff` for the changed files plus the PLAN.md step.
  Max 10 findings, most severe first, each with `file:line`. Check that the Verify output pasted
  in PROGRESS.md is real and matches the expected result. Flag changes to files the step didn't name.
- Plan critiques come from Codex (`codex exec`). Weigh each point, accept or reject it with a reason,
  and record it in PLAN.md's Review log.

## Project-specific cautions
- Transcript formats for Claude Code and Codex are undocumented. Treat every field as an assumption
  until Phase 0 confirms it on real data, and say so when you rely on one.
- A link is `exact` only when the child Codex id is observed. Never present a guess as exact.
- The `claude://code/continue?session=local_<id>` link is undocumented app behaviour (tested once, 2026-10-29).

## Git
- Don't run `git commit` or `git push`. Give the owner exact commands, staging specific files.
