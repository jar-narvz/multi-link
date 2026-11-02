# multi-link

A local terminal dashboard for Claude Code and Codex sessions across your repos.
Release 1 is complete (see [check results](docs/phase3-check.md)). Release 1.1 (home screen
cleanup) did not meet the need ([owner verdict](docs/phase4-check.md#owner-check-stop-and-ask)), and
neither did Release 1.2's bars ([owner verdict](docs/phase5-check.md#owner-check-stop-and-ask--answered-2026-11-01)).
Release 1.3 rebuilds home in plain words and the repo view as swimlanes, to two approved mockups; it is
built and awaits the owner's check of the real screen ([visual check](docs/phase6-check.md)).

- Groups sessions by repo, with needs-you sorting, states and PLAN.md / PROGRESS.md status.
- Links Claude hand-offs to Codex sessions, marking exact, heuristic and ambiguous matches.
- Shows when each session was active in a per-repo swimlane view, with Codex runs nested under the
  Claude session that started them.
- Opens supported Claude desktop sessions or resumes sessions confirmed finished; rechecks the
  action before executing. Unsupported actions show a command for you to run.

## Home screen

![Home screen on the phase6 fixture](docs/img/home-phase6-fixture.png)

*HTML render of the home buffer on the synthetic fixture `fixtures/sessions-phase6` at 120×40
(`crates/tui/examples/home_html.rs`; method, the approved mockup in words and real-data differences in
[docs/phase6-check.md](docs/phase6-check.md)). Real renders are not committed because they show prompts.
The real terminal uses your own font.*

- **Top line:** working, stale, idle and done-or-unknown counts, repos and hand-offs, and on the right
  `◆ N need you` in yellow when a session is waiting. With nothing waiting and no hook records seen, it
  says `◆ needs-you partial: no hook records seen` instead of claiming zero.
- **A card for each repo with a waiting, working or stale session:** a rounded
  card (yellow when waiting, orange when working) titled with its latest PLAN.md step. Each such session is
  one line: harness, a label (its latest step, else its first prompt in quotes, else `claude · no prompt`), a status
  phrase (`waiting on you · since 21:40`, `working now · for 5 min`, `stale since …`) and its branch. A grey
  line counts the other sessions, and the last line names the latest hand-off in words (`claude handed
  step 6.3 to codex · codex done`).
- **Idle repos are one sentence:** `idle · last active 1 min ago · claude handed work to codex at 22:23,
  codex done`. Repos with no activity in the window share one `quiet` line.
- **Codex Desktop imports are hidden** (they are Claude chats copied into Codex Desktop, not runs); the
  footer counts them.
- **Colour** uses truecolor when `COLORTERM` is `truecolor`/`24bit`, else 256 colours; `NO_COLOR` keeps
  every glyph and word. Below 80 columns the branch drops first, then labels shorten on a word boundary.

## Repo view

![Repo view on the phase6 fixture](docs/img/repo-phase6-fixture.png)

*Same fixture, repo `repo-r`, its newest lane selected.*

- **Enter** (or `t`) on a home repo opens it. One lane per session with activity in the window, newest
  first; each Codex run sits under the Claude session that handed it work (`└▸`).
- **Bars are recorded activity:** timestamps at most 5 minutes apart form one burst. Claude bursts are
  orange, Codex blue; `✓` marks a finished Codex run, `◆` a session waiting on you.
- **Links:** a line from the hand-off point to the child, solid `│` when confirmed, dashed `╎` when matched
  by time, and the sub line says how (`matched by time · 12 s after the call`).
- **The axis fits the work:** it starts at the first activity; idle stretches over 15 minutes with no
  activity on any lane fold to a `≈` band with their length when there is room (at most 3). `z` unfolds, `+`/`-` zoom, ←/→ pan,
  `0` returns to the fitted range, `i` shows hidden imports.
- The `selected` line gives the session's short id, start, how long it was active and its hand-off; `o`
  opens it and `f` forks it. Below 80 columns the bars drop and the lanes become a list.

Scanning only reads local agent files (`~/.claude`, `~/.codex`), repo metadata and optional
hook state in `~/.multi-link`. It makes no network calls. Opening a session launches the
corresponding external program; hook recording writes multi-link's own state directory.

## Run

Requires macOS, Rust/Cargo and git. From the repo:

```sh
cargo build --release -p multi-link
target/release/multi-link tui
```

Use ↑/↓ to select, Enter (or `t`) to open a repo view, `o` to open, `f` to fork, Esc to return, and
`q` to quit. In the repo view, `+`/`-` zoom, ←/→ pan, `0` resets the range, `z` unfolds idle stretches
and `i` shows imports. The default history window is `2h`; use `--since all` for all history.

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
- Activity is inferred from record timestamps: a tool call that writes nothing for over 5 minutes
  shows as a gap, and screens say "active", never "worked".
- Codex "done" means its last turn completed; there is no process-exit evidence.
- Real `codex exec` rollouts seen on 2026-11-02 keep the prompt in a record the parser does not read, so
  Codex lanes show `no prompt`; sessions started by another session can show `no prompt` too.
- With `--since` other than `2h`, the home screen still says `last 2h` (the repo view follows the window).
- JSON output now includes prompt text: `sessions` and `overview` carry each session's first prompt
  (`asked`, up to 120 characters). Check before sharing JSON.
- macOS only for now.

Workspace tests and clippy passed in the Step 3.8, 4.6, 5.5 and 6.5 runs. Real-home
timing output, terminal evidence and validation limits are in [docs/phase3-check.md](docs/phase3-check.md);
the Phase 4 home check is in [docs/phase4-check.md](docs/phase4-check.md), the Phase 5 visual check in
[docs/phase5-check.md](docs/phase5-check.md) and the Phase 6 visual check in
[docs/phase6-check.md](docs/phase6-check.md).

## Docs

- [Design](docs/design.md) and [output schema](docs/output-schema.md)
- [Plan](PLAN.md) and [progress](PROGRESS.md)
