# multi-link

A local terminal dashboard for Claude Code and Codex sessions across your repos.
Release 1 is complete (see [check results](docs/phase3-check.md)). Release 1.1 (home screen
cleanup) is built and measured; it awaits the owner's check of the real screen
([before / after](docs/phase4-check.md)).

- Groups sessions by repo, with needs-you sorting, states and PLAN.md / PROGRESS.md status.
- Links Claude hand-offs to Codex sessions, marking exact, heuristic and ambiguous matches.
- Shows session spans in a repo timeline, with scrolling, zoom and pan.
- Opens supported Claude desktop sessions or resumes sessions confirmed finished; rechecks the
  action before executing. Unsupported actions show a command for you to run.

## Home screen

- **Top line:** waiting, busy and stale sessions, warnings, repos, sessions and hand-offs, so you can
  see whether anything needs you before scrolling. A second line labels the time axis.
- **Busy and waiting repos expanded first.** Each shows its latest PLAN.md step and Verify result,
  its sessions with a state glyph, and its latest hand-off with the link kind.
- **Quiet repos as one line each** (repo, counts, latest step and Verify) until you press Enter on them.
  A repo with a `stale` session stays expanded.
- **Merged rows:** four or more quiet top-level Claude or Codex sessions in a repo become one
  `claude ×N` / `codex ×N` row; Enter expands it.
- **Mini time bars:** at 60 columns or wider (and 4 rows or taller), each session row ends in a
  bar of its span (start to last activity, idle gaps included) over the last 2 hours. Merged rows and collapsed repos have none.

On the owner's real home directory, at one pinned time, the default 2-hour window went from 28 to 15
non-blank lines and `--since all` from 246 to 30 non-blank lines, compared with Release 1 (full
output and method in [docs/phase4-check.md](docs/phase4-check.md); counts change with what is running).

Scanning only reads local agent files (`~/.claude`, `~/.codex`), repo metadata and optional
hook state in `~/.multi-link`. It makes no network calls. Opening a session launches the
corresponding external program; hook recording writes multi-link's own state directory.

## Run

Requires macOS, Rust/Cargo and git. From the repo:

```sh
cargo build --release -p multi-link
target/release/multi-link tui
```

Use ↑/↓ to select, Enter to expand a quiet repo or merged row or open its timeline, `t` for the timeline,
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

Workspace tests and clippy passed in the Step 3.8 run and again in the Step 4.6 run. Real-home
timing output, terminal evidence and validation limits are in [docs/phase3-check.md](docs/phase3-check.md);
the home screen before / after check is in [docs/phase4-check.md](docs/phase4-check.md).

## Docs

- [Design](docs/design.md) and [output schema](docs/output-schema.md)
- [Plan](PLAN.md) and [progress](PROGRESS.md)
