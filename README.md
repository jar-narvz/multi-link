# multi-link

**Status: design stage. Nothing is implemented yet.**

A terminal dashboard for every Claude Code and Codex session on your machine, across all your repos.
What it aims to add over existing session viewers:

- **Hand-off links:** shows which Claude session handed work to which Codex session, and marks each
  link as exact (child id observed), heuristic, or ambiguous.
- **Plan awareness:** groups sessions by the PLAN.md step they serve, with each step's Verify status.

It only reads local files (`~/.claude`, `~/.codex`, each repo's PLAN.md / PROGRESS.md) and makes no
network calls.

## Docs
- Design: [`docs/design.md`](docs/design.md)
- Plan and progress: [`PLAN.md`](PLAN.md), [`PROGRESS.md`](PROGRESS.md)

## Limitations (known now)
- Claude Code and Codex transcript formats are undocumented and may change.
- Opening a desktop-app session uses an undocumented `claude://` link.
- macOS only for now.
