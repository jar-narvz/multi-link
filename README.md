# multi-link

A local terminal dashboard for Claude Code and Codex sessions across your repos.
Release 1 is complete (see [check results](docs/phase3-check.md)). Release 1.1 (home screen
cleanup) did not meet the need ([owner verdict](docs/phase4-check.md#owner-check-stop-and-ask)).
Release 1.2 rebuilds the home screen to an approved mockup; it is built and awaits the owner's check
of the real screen ([visual check](docs/phase5-check.md)).

- Groups sessions by repo, with needs-you sorting, states and PLAN.md / PROGRESS.md status.
- Links Claude hand-offs to Codex sessions, marking exact, heuristic and ambiguous matches.
- Shows session spans in a repo timeline, with scrolling, zoom and pan.
- Opens supported Claude desktop sessions or resumes sessions confirmed finished; rechecks the
  action before executing. Unsupported actions show a command for you to run.

## Home screen

![Home screen on real data](docs/img/home-real.png)

*HTML render of the home buffer on the owner's real `~` at 120×40, 2026-10-30 17:08 local time
(`crates/tui/examples/home_html.rs`; method and every difference from the approved mockup in
[docs/phase5-check.md](docs/phase5-check.md)). The real terminal uses your own font.*

- **Top line:** busy, idle and done-or-unknown counts, repos and hand-offs, and on the right `◆ N need you`
  in yellow when a session is waiting. With nothing waiting and no hook records seen, it says
  `◆ needs-you partial: no hook records seen` instead of claiming zero. When space is short the
  hand-off count is dropped first (as in the image above).
- **One card per repo** with a rounded border: orange when a session is busy, yellow when one is waiting.
  The card title shows the latest PLAN.md step and its Verify result.
- **Lanes named by step**, e.g. `step 5.3 · claude → codex · matched by time`. A Claude session and the one
  Codex run it handed a step to share a lane; sessions without a hand-off read `claude · no hand-off` or
  `codex · no parent found`. Link words are plain: `confirmed`, `matched by time`, `N possible runs`,
  `no run found`.
- **Bars on a shared 2-hour axis** in local time, ending at `now`: Claude a thin orange line, Codex a
  solid blue block, bright when busy, dim when idle or done; `◂` / `▸` mark spans cut by the window.
- **Waiting, busy and stale sessions always show.** Up to 8 quiet lanes with activity in
  the 2-hour window follow, oldest first; older or extra sessions are one `N earlier sessions · enter shows
  all` line. The last line names the latest hand-off.
- **Quiet repos are one line:** name, plan step, counts by harness and state, last activity.
- **Colour** uses truecolor when `COLORTERM` is `truecolor`/`24bit`, else 256 colours; `NO_COLOR` keeps
  every glyph and word. Narrower terminals shorten labels; below 60 columns bars are dropped.

Known rough edges on real data (from the check): lane tracks touch in a terminal, so a card's bar area
reads as one dark block; a hand-off with no step prints `step ?` on the `last:` line; a lane whose
sessions both lack a step shows no step; and the hand-off count covers every link in the scan, not only
those inside the window.

Scanning only reads local agent files (`~/.claude`, `~/.codex`), repo metadata and optional
hook state in `~/.multi-link`. It makes no network calls. Opening a session launches the
corresponding external program; hook recording writes multi-link's own state directory.

## Run

Requires macOS, Rust/Cargo and git. From the repo:

```sh
cargo build --release -p multi-link
target/release/multi-link tui
```

Use ↑/↓ to select, Enter to expand a quiet repo, split a shared lane, show hidden sessions or
open the timeline, `t` for the timeline,
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

Workspace tests and clippy passed in the Step 3.8, 4.6 and 5.5 runs. Real-home
timing output, terminal evidence and validation limits are in [docs/phase3-check.md](docs/phase3-check.md);
the Phase 4 home check is in [docs/phase4-check.md](docs/phase4-check.md) and the Phase 5 visual check in
[docs/phase5-check.md](docs/phase5-check.md).

## Docs

- [Design](docs/design.md) and [output schema](docs/output-schema.md)
- [Plan](PLAN.md) and [progress](PROGRESS.md)
