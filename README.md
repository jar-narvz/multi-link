# multi-link

A local terminal dashboard for Claude Code and Codex sessions across your repos.
Release 1 is complete (see [check results](docs/phase3-check.md)).

- Groups sessions by repo, with needs-you sorting, states and PLAN.md / PROGRESS.md status.
- Links Claude hand-offs to Codex sessions, marking exact, heuristic and ambiguous matches.
- Shows session spans in a repo timeline, with scrolling, zoom and pan.
- Opens supported Claude desktop sessions or resumes sessions confirmed finished; rechecks the
  action before executing. Unsupported actions show a command for you to run.

Scanning only reads local agent files (`~/.claude`, `~/.codex`), repo metadata and optional
hook state in `~/.multi-link`. It makes no network calls. Opening a session launches the
corresponding external program; hook recording writes multi-link's own state directory.

## Run

Requires macOS, Rust/Cargo and git. From the repo:

```sh
cargo build --release -p multi-link
target/release/multi-link tui
```

Use ↑/↓ to select, Enter to expand a merged row or open its timeline, `t` for the timeline,
`o` to open, `f` to fork, Esc to return/collapse, and `q` to quit. In the timeline,
`+`/`-` zoom, ←/→ pan, and `0` resets the window. The default history window is `2h`;
use `--since all` for all history.

The CLI also exposes the data and safe open previews:

```sh
target/release/multi-link overview --json
target/release/multi-link sessions --json
target/release/multi-link links --json
target/release/multi-link steps --repo "$PWD" --json
target/release/multi-link open <session-id-prefix> --dry-run
```

For fixture data, add `--root fixtures/sessions --now 2026-10-29T23:15:00Z --since all`
to `tui` or the report commands. `--root` enables fixture mode and disables real process probing.
`hook install --scope local` prints configuration to paste manually; it does not edit Claude
settings. See the [design](docs/design.md) for hook behavior and state rules.

## Limitations and validation

- Claude Code and Codex transcript formats are undocumented and may change. Unknown records
  are skipped and counted; missing evidence can leave states or links unknown.
- Codex waiting detection and automatic Codex opening are unsupported.
- Claude desktop opening uses an undocumented `claude://` link.
- Resume requires a transcript and positive process-exit evidence. If process probing is
  unavailable, the app cannot confirm that a session has finished.
- A cleanly exited terminal Claude session deletes its record, so `o` shows Unavailable with
  the resume command. Resume is available only when a session's record outlives its process.
- A keypress arriving together with a terminal resize can be delayed until the next key
  (crossterm 0.29).
- macOS only for now.

Workspace tests and clippy passed in the Step 3.8 run. Real-home timing output, terminal evidence
and validation limits are in [docs/phase3-check.md](docs/phase3-check.md).

## Docs

- [Design](docs/design.md) and [output schema](docs/output-schema.md)
- [Plan](PLAN.md) and [progress](PROGRESS.md)
