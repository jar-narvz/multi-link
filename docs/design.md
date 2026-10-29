# multi-link — design doc

The living design doc is on claude.ai (private): https://claude.ai/artifact/PYqQLcPNkk8vHykZuBSJys
(it still uses the earlier working name "cclink").

TODO (owner): export it as Markdown (Export → Markdown in the doc) and replace this file with the export,
so the design is versioned with the code and visible on GitHub.

## Summary until then
- One Rust core reads Claude Code and Codex transcripts, assigns sessions to repos and PLAN.md steps,
  computes a per-session state from evidence, and links Codex sessions to the Claude tool call that
  launched them (`exact` only when the child id is observed; else `heuristic`, `ambiguous` or unlinked).
- Front ends draw the core's output: CLI first (plumbing and tests), then a TUI home screen and repo
  timeline (Release 1). Step graph, live grid, desktop app and mod are backlog.
- Phase 0 answers four go / no-go questions on real data before any code: child-id observability,
  state signals, zoetrope crate usability, agtrace overlap. See `PLAN.md`.
