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

## Autonomous mode (owner-approved 2026-10-29)
Run the loop step after step without asking the owner:
1. Plan the step in PLAN.md (files + Verify). For a phase split, get a Codex critique first and log it.
2. Hand it to Codex (command below). Codex implements, runs Verify, writes the PROGRESS.md entry.
3. Review `git diff` against the step. Findings go back to Codex (`codex exec resume --last` or a new
   exec with the findings), at most 2 rounds. If Claude edits code itself, Codex reviews that edit.
4. Re-run Verify yourself; compare with what Codex pasted. Fill in the entry's Review line.
5. Commit (rules below) and move to the next step.

Codex command (from the repo root):
```
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
codex exec -s workspace-write -c sandbox_workspace_write.network_access=true \
  --add-dir "$HOME/.cargo" --add-dir "$HOME/.rustup" -C "$PWD" -o <scratchpad>/codex-last.txt "<prompt>"
```
Always close stdin (`< /dev/null`) when running it in the background, or it waits for input forever.
Network is on only so cargo can fetch crates; project code still makes no network calls.
Prompts start with "Read AGENTS.md, then do Step <id> in PLAN.md." and add only what the step lacks.

Stop and ask the owner only when:
- a Phase 0 result changes the goal or differentiator (e.g. agtrace already links Claude → Codex).
  A no-go on (a) is not a stop: the plan already falls back to heuristic links.
- a step needs something only the owner can do (pasting into `~/.claude/settings.json`, a sign-in).
- the same step fails Verify after 3 Codex rounds.
- anything destructive or hard to undo, any push, publishing, or installing software beyond crates.

### One session per step (keeps context fresh)
- A coordinator session starts one fresh Claude session per PLAN.md step (sharing this checkout, no
  worktree). It does no step work itself; it only starts sessions and checks results.
- A step session reads `CLAUDE.md`, `AGENTS.md`, the step in `PLAN.md` and "Current state" in
  `PROGRESS.md`, does exactly one step through the loop above (Codex → review → Verify → commit),
  updates "Current state" with what the next step needs to know, and ends. It never starts the next step.
- Everything a later step needs goes in PROGRESS.md or the repo, never only in a session's chat.
- Parallel steps share one git index, so their sessions do NOT commit or stage: each leaves its files
  uncommitted and reports its paths plus its PROGRESS.md entry text; the coordinator then commits each
  step separately. (Learned in 1.2/1.5: a parallel `git add` landed 1.5's code in 1.2's commit.)
- When a step session ends, the coordinator checks `git log -1` and PROGRESS.md: a new commit for that
  step → start the next step's session; no commit or a "stop and ask" → report to the owner and wait.

Sandbox repos for generating real test sessions live in `~/multi-link-sandbox/` (outside this repo).
Claude may create them and run `claude -p` / `codex exec` there to produce hand-offs on purpose.

## Git
- Claude commits locally once a step's Verify passes and its review is clean. Author is the owner's
  configured git identity. Stage specific files only; short imperative subject; no AI attribution or
  `Co-Authored-By` lines. One step (or smaller) per commit.
- Never push, force-push, rewrite history or delete branches. The owner pushes.
- Codex never commits or stages (see AGENTS.md).
