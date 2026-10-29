# multi-link — Design Doc

Oct 29, 2026 · Status: draft

> Exported from the claude.ai doc (working name "cclink") on 2026-10-29. **This repo copy is now the
> source of truth.** Changes made since the export: name → multi-link; supported PLAN / PROGRESS
> convention matches the owner's real format; `hook install` prints a snippet instead of writing
> `~/.claude/settings.json`. See `PLAN.md` → Review log. The architecture diagram was not exported.

## Abstract

multi-link is a terminal dashboard for every Claude Code and Codex session on my machine, across all my repos. Two things set it apart from c9watch, agtrace and `claude agents`: it links each Codex session to the Claude session that handed it work, and it groups sessions by PLAN.md step, with that step's progress and Verify status.

One Rust core reads both transcript formats, assigns each session to a repo and a step, and links hand-offs. A TUI is the first front end: an all-repos overview as the home screen, drilling into a repo's timeline, step graph and live grid. A desktop/menu-bar app on the same core comes later; a Claude Code mod is optional. The MVP is the core plus a `multi-link` CLI, verified against saved transcripts.

## Glossary

| Term | Meaning in this doc |
| --- | --- |
| Harness | The program around a model that runs the agent loop, executes tools, enforces permissions and writes logs. Claude Code and Codex CLI are two harnesses. |
| Session | One conversation in a harness, with its own id and transcript. |
| Transcript | The JSONL log a harness writes per session: Claude Code under `~/.claude/projects/`, Codex under `~/.codex/sessions/`. Both formats are undocumented. |
| Subagent | A child agent a harness starts inside one session; it appears in that session's own transcript. |
| Hand-off | A Claude tool call that starts Codex work (the Codex plugin or `codex exec`). |
| Link | A recorded parent → child pair: Claude session id → Codex session id, with time and how it was matched. |
| Heuristic match | A link inferred from logs (same folder, close start times); has a ranking score. |
| Hook match | A link recorded at hand-off time by a Claude Code hook; exact only when the child id is observed. |
| Core | The Rust library that parses both transcript formats and computes links. |
| CLI | The `multi-link` command that prints the core's output as JSON or text. |
| Mod | A Claude Code plugin that draws UI (a strip above the prompt, a side pane) inside Claude Code. Written in TypeScript. |
| TUI | A full-screen terminal app (like zoetrope) drawn with ratatui. |
| Swimlane | One horizontal row per session on a shared time axis, with bars where it was busy. |
| Worktree | A second checkout of the same git repo on its own branch, used so parallel sessions don't edit the same files. |

## Problem statement

### Current state

I work in a two-agent loop across several repos at once (route-impact, posting-triage, physaflow and others): Claude plans and reviews, Codex implements each PLAN.md step. Claude Code and Codex are separate harnesses that write separate transcripts and don't know about each other. When Claude hands a step to Codex, Claude's log shows only one shell command; everything Codex did lives in a different file. To follow one step I switch between sessions and guess which Codex run belongs to which Claude session, and to know which repo needs me I check each one.

I looked at four existing tools. Each solves part of the problem; none links the two harnesses.

| Tool | What works | Gap for my workflow |
| --- | --- | --- |
| [zoetrope](https://github.com/furkankly/zoetrope) | Live graph of one session: agents, subagents, tool calls, scrubbable timeline. Reads both Claude and Codex logs. Read-only, no network. | One session at a time. A Claude → Codex hand-off shows as a single tool chip. |
| [c9watch](https://github.com/minchenlee/c9watch) | All sessions on the machine in one Mac app: status, needs-attention first, cost, history, JSON CLI. | Mac window, not terminal. Sessions are a flat list; no parent → child link across harnesses. |
| [repo-sessions](https://github.com/gblacher2/repo-sessions) | Claude Code mod: Claude and Codex sessions in the current repo as swimlanes on a shared timeline, beside the chat. | Shows busy/idle only. Codex rows aren't tied to the Claude session that started them. Unproven (0 stars). |
| [agtrace](https://github.com/lanegrid/agtrace) | Live terminal tree of every agent, including Codex's own child threads; inter-agent message feed. | README doesn't say it links a Claude session to the Codex run it launched. Untested by me. |

Where the flow feels awkward today: I can see *that* Codex is busy, but not *which Claude session or PLAN.md step it serves*; no tool shows all my repos with their step progress; and I can't replay a finished step across both agents.

### Desired state (needs)

1. From one screen, I can see every repo with active Claude or Codex sessions, and which ones need me.
2. I can tell which Claude session started each Codex session, and when.
3. I can see where each repo stands in its PLAN.md: the current step, who is working on it, and whether its Verify passed.
4. I can follow one step from Claude's plan, through Codex's work, back to Claude's review, without opening transcripts.
5. I can do this in the terminal now, and later from an always-visible desktop app with notifications.
6. I can trust what I see: guessed links and guessed step assignments are marked as guesses.

## Solution statement

### Functional requirements

| ID | Requirement | Need |
| --- | --- | --- |
| FR1 | Discover sessions in two ways: every active session (live process or session record), whatever its age; plus history changed within a window (default 2 h, `--since`). | 1 |
| FR2 | Assign each session to a repo (git root; outside git, the folder) while keeping its checkout identity (main or which worktree). | 1 |
| FR3 | Report each session's state from a defined state machine (busy, waiting on me, idle, finished, stale/unknown), with the evidence used. | 1, 6 |
| FR4 | Link each Codex session to the Claude tool call that launched or resumed it, with hand-off time. | 2 |
| FR5 | Mark each link `exact` (child id observed), `heuristic` (with a ranking score, not a probability), `ambiguous` (several candidates) or unlinked. | 6 |
| FR6 | Read PLAN.md and PROGRESS.md in a supported convention: steps, steps in progress, and Verify per step (command, result, time, checkout, source: reported or observed; stale or unknown when unsure). | 3, 6 |
| FR7 | Split each session into work segments (time ranges) and assign each a step from explicit evidence (hook tag or a `step N` tag in the hand-off prompt), else `unknown`. Inferring steps from history is deferred. | 3, 6 |
| FR8 | `multi-link` CLI prints sessions, segments, links and steps as JSON or plain text. | 2, 3 |
| FR9 | TUI home screen (`multi-link tui`): every active repo, sorted needs-you first. Per repo: steps in progress with their Verify, one mini lane per session with its step and checkout, Codex lanes nested under their parent. Over 4 sessions, lanes merge into "claude ×N". | 1, 3, 5 |
| FR10 | Repo timeline: swimlanes, Codex rows nested under their Claude parent, hand-off arrows, ambiguous links shown as such. | 2, 5 |
| FR11 | Backlog: step graph, Claude plan → Codex work → Claude review for one step, scrubbable. | 4, 5 |
| FR12 | Backlog: live grid, one card per active session under a feed of hand-offs. | 1, 2, 5 |
| FR13 | Optional hook records exact links when the child id is observable, and explicit step tags. | 2, 3, 6 |
| FR14 | Optional alert, fired only on a supported "waiting on me" signal: terminal bell or macOS notification. | 1, 5 |
| FR15 | Backlog: desktop / menu-bar app on the same core, with native notifications. | 1, 5 |
| FR16 | Backlog: Claude Code mod with a strip above the prompt. | 1 |
| FR17 | Warn about concurrent activity in one checkout: 2+ busy sessions with the same working folder. Detecting actual file writes is out of scope. | 1, 6 |
| FR18 | Open a session where it came from (`open_in = origin \| terminal \| app`): finished → resume; running → jump to it, or show "unavailable" if it can't be reached; never resume a running session. `f` forks. | 1, 4, 5 |

### Non-functional requirements

- **Local and read-only.** No network calls; never writes to either harness's folders (`~/.claude/`, `~/.codex/`, including settings), PLAN.md or PROGRESS.md. multi-link's own state lives in `~/.multi-link/`.
- **Fails soft.** Unknown transcript records are skipped and counted, never fatal (both formats are undocumented).
- **Incremental reads.** Each file is tailed from its last offset; a partial last line is buffered; a truncated or replaced file is re-read from the start. Minimal metadata (ids, checkout, links) is kept so an old parent can be backfilled when a new child references it.
- **Speed (pass threshold).** The home screen refreshes in 2 s or less on my laptop with my real `~/.claude` and `~/.codex`. Checked in Phase 3; not yet measured.
- **Testable.** Parsers, repo assignment, state, linking and step assignment run on saved fixtures with a fixed clock and hand-labelled expected outputs, including negative and ambiguous cases.
- **Swappable parsers.** Each harness's format lives behind one interface, so a format change touches one module.
- **One core, many front ends.** Front ends contain no parsing, state or linking logic.
- **Licensing.** Reused MIT code keeps its copyright notice; NOTICE file credits zoetrope, c9watch and repo-sessions.

### Non-goals

- Controlling sessions (starting, stopping, replying). `claude agents` and the desktop app already do that.
- Harnesses other than Claude Code and Codex.
- Cost tracking and history search (c9watch covers them).
- Remote or multi-machine viewing.
- Editing PLAN.md or PROGRESS.md; multi-link only reads them.
- Linking Codex → Claude hand-offs (my workflow only goes Claude → Codex).

## Solution design

One repo, one Rust core, thin front ends. The core reads every recent session on the machine, assigns it to a repo and a PLAN.md step, and links hand-offs; every front end only draws that output. The TUI ships first, the desktop app later.

Transcripts, PLAN.md / PROGRESS.md and optional hook records flow into the core; the CLI, TUI, desktop app and mod all read the same model.

### Major decisions

| Decision | Choice | Why | Alternative rejected |
| --- | --- | --- | --- |
| Scope | A dashboard for all repos, not one repo | One place to see what needs me; the overview is the home screen and the per-repo views are drill-downs | Single-repo tool: smaller, but I'd still need another tool for the overview |
| Differentiator | Hand-off links **and** PLAN.md step awareness | c9watch, agtrace and `claude agents` show sessions; none link Claude → Codex or know which plan step a session serves | Links only: weaker story for a multi-repo dashboard |
| First front end | TUI, desktop app later | I work in the terminal; one front end proves the core before a second one | Desktop first: Tauri work before the core is proven |
| Repo shape | One monorepo: `core/`, `cli/`, `tui/`, later `app/` and `mod/` | One design, one set of fixtures; front ends can't drift apart | Separate repos: duplicated parsing |
| Core language | Rust | zoetrope's engine and c9watch's detection code are Rust; fast file tailing; Tauri (desktop) is Rust too | TypeScript: can't reuse either |
| Mod ↔ core bridge | Mod runs `multi-link` and reads JSON | Mods must be TypeScript; a process boundary is the simplest cross-language link | WASM or native bindings: more build complexity |
| Link strategy | Hook record first, heuristic as fallback | Hooks are exact but only cover new sessions; heuristics cover history | Heuristic only: can't be trusted when sessions overlap |
| Reuse | Depend on zoetrope's crate if Phase 0 shows it is usable; copy c9watch detection files with credit | Depending gets upstream fixes; copied code is small and stable | Fork c9watch: weak portfolio story |
| Data source | Read files, never write them | Read-only is safe and matches every comparable tool | Wrapping the CLIs: breaks my existing workflow |
| Hook install | `hook install` prints the settings snippet; the owner pastes it | Keeps the read-only rule absolute; install stays visible and reversible | Writing `~/.claude/settings.json`: one exception makes the rule harder to trust |

### Linking algorithm

1. **Recognised invocations only.** A hand-off is a Claude tool call in a supported form: `codex exec …`, `codex resume <id>`, or the Codex plugin's call (exact forms recorded in Phase 0). Other commands that merely mention `codex` are ignored.
2. **Exact.** A link is exact only when the child Codex session id is explicitly observed: in the tool result (`stdout`, a background task's output) or as the `resume` argument. Codex's review of my local files found Claude tool calls carry an `id`, results carry `tool_use_id` plus `stdout` / `stderr` or `backgroundTaskId`, and only some results contain the Codex id. The hook records the same evidence; it does not make a guess exact.
3. **Heuristic.** Without an observed id: candidates are Codex sessions whose working folder equals the invocation's exact checkout (not the merged repo) and whose start falls inside the invocation's time interval. One candidate → `heuristic` with a ranking score; several → `ambiguous`, listing them; none → unlinked. The score is not a probability until checked against labelled fixtures.
4. **Resume.** `codex resume` adds a hand-off to an existing Codex session instead of creating one, so a Codex session can have several hand-offs over time.

### Repo and step assignment

1. **Repo.** Each transcript records the folder it ran in (Claude's project folder name is lossy, so the core reads the path from inside the file). The core resolves the git root for grouping but keeps the exact checkout (main or which worktree) on every session.
2. **Supported convention** (the owner's real format):
   - PLAN.md step heading `### Step <id> — <title>`, where `<id>` is `N` or `N.M`, with a `- Verify:` or `- **Verify:**` line inside the step.
   - PROGRESS.md entries headed `### YYYY-MM-DD — Step <id>: <title>`, optionally `— <agent>`, newest first, with a Verify line.
   - Anything else → steps unknown.
3. **Work segments.** A session is split into time ranges; each gets a step only from explicit evidence (hook tag, or a `step N` tag in the hand-off prompt), else `unknown`. A long Claude session that plans step 4 and reviews step 5 gets two segments.
4. **Verify per step.** `{command, result, time, checkout, source}` where source is `reported` (PROGRESS.md) or `observed` (the command's run seen in a transcript). Marked stale if the checkout changed since. There is no repo-level "last Verify".
5. A repo without PLAN.md still shows; its sessions have no steps.

### Session state

| State | Evidence (to confirm in Phase 0) |
| --- | --- |
| Busy | Codex: a `task_started` event without its `task_complete`. Claude: the live session record's `status`, recently updated. |
| Waiting on me | Claude: session record status or the optional hook bridge reports a permission prompt or question. Codex: approval events, if Phase 0 finds them. |
| Idle | Process alive, last turn complete. |
| Finished | No live process. |
| Stale / unknown | Process alive but no update within a freshness limit (set in Phase 0), or no usable signal. |

Signals found in my local files during review: Claude's `~/.claude/sessions/<pid>.json` has `pid`, `procStart`, `status`, `statusUpdatedAt`; Codex events carry `event_msg.payload.type` values such as `task_started` and `task_complete`. Silence alone never means busy. Alerts (FR14) fire only on a supported waiting signal.

### Data model (CLI output)

Illustrative only; the binding schema is `docs/output-schema.md` (Step 0.5).

```json
{
  "repos": [
    {"path": "~/route-impact", "needs_me": 1,
     "steps": [{"n": 4, "status": "in_progress",
                "verify": {"command": "pytest tests/test_roads.py", "result": "pass", "at": "2026-10-29T14:20:00Z", "checkout": "main", "source": "reported"}}]}
  ],
  "sessions": [
    {"id": "a1b2…", "harness": "claude", "repo": "~/route-impact", "checkout": "main",
     "state": "waiting", "state_evidence": "session_record",
     "segments": [{"from": "14:00:02", "to": "14:02:11", "step": 4, "step_source": "prompt_tag"}]}
  ],
  "links": [
    {"parent": "a1b2…", "invocation": "toolu_…", "child": "019a…", "at": "2026-10-29T14:02:11Z", "match": "exact", "evidence": "stdout"},
    {"parent": "d4e5…", "invocation": "toolu_…", "child": null, "match": "ambiguous", "candidates": ["019a9c…", "019ab2…"]}
  ]
}
```

### How each requirement is met

| FR | Met by |
| --- | --- |
| FR1 | Active: live process and session records. History: files changed inside the window |
| FR2, FR6, FR7 | Repo + step mapper, as above |
| FR3, FR14 | Session state table, as above |
| FR4, FR5 | Linker |
| FR8 | `cli/` serialises the core's model |
| FR9, FR10 | `tui/`: home overview and repo timeline |
| FR11, FR12, FR15, FR16 | Backlog |
| FR13 | `multi-link hook install` prints the hook entry for the owner to paste; the hook writes to `~/.multi-link/` |
| FR17 | Core compares the working folders of busy sessions |
| FR18 | See "Opening a session" below; Phase 3 |

### Opening a session (FR18)

| Session | Running | Finished |
| --- | --- | --- |
| Claude, desktop app | `open "claude://code/continue?session=local_<id>"` (verified 2026-10-29) | Same link, once the `local_` id can be found (Phase 0) |
| Claude, terminal | Jump to its tmux / Herdr pane; other terminals later | `claude --resume <id>` in the TUI's terminal, return on exit |
| Codex, desktop app | `open "codex://threads/<id>"` (from repo-sessions' code, untested) | Same |
| Codex, terminal | Jump to its pane | `codex resume <id>` |

- **App vs terminal:** a running Claude session's `~/.claude/sessions/<pid>.json` has a `hostSessionId` (`local_…`) only when the desktop app started it. That id differs from the transcript id `claude --resume` uses.
- **Fork:** `claude --resume <id> --fork-session` or `codex fork` starts a side conversation without touching the original.
- **Unreachable running session:** if the app link fails or the pane is unknown, show "unavailable". Resume only once the session is finished.
- **Phase 3 scope:** resume for finished sessions and the app link for running app sessions. Jumping to tmux / Herdr panes is backlog.

### Edge cases

- **Parallel sessions in one repo:** each session keeps its own lane, segments and state.
- **Parallel hand-offs in one checkout:** shown as `ambiguous` with both candidates, never guessed. Separate worktrees narrow the candidates but don't make a link exact; only an observed child id does.
- **Concurrent activity in one checkout:** FR17 warns only; it never stops a session.
- **`codex` mentioned but not launched** (e.g. reading logs): not a hand-off.
- **Codex resumed by Claude:** a new hand-off on an existing Codex session.
- **Codex started by hand:** no parent; shown as a top-level row.
- **Session waiting overnight:** still shown, because active sessions are found outside the history window.
- **Opening a running session:** never resume it; jump to it or show "unavailable".
- **App link changes:** undocumented app behaviour; on failure, "unavailable" for running sessions.
- **Worktrees:** grouped under their repo, labelled with their checkout.
- **No PLAN.md, or an unsupported format:** steps unknown.
- **PROGRESS.md edited by hand:** re-read on the next refresh.
- **Sessions outside any repo:** grouped under the folder name, collapsed.
- **Format change after a harness update:** unknown records skipped and counted; `multi-link doctor` reports the count.
- **Partial, truncated or replaced files:** partial line buffered; truncation or replacement → re-read from the start.
- **Session ids:** always store full ids; display prefixes only.

## Solution implementation

### Mockup

The terminal mockup (claude.ai artifact, private: https://claude.ai/artifact/R5Bh9iX6bsn3jSu8F8D58C) has six screens, all with sample data. Screen 6 is the home screen; 2, 3 and 5 are its drill-downs.

| Screen | Phase | Shows |
| --- | --- | --- |
| 6 · `multi-link tui` home | 3 (Release 1) | Every active repo, needs-you first. Per repo: steps in progress with Verify, one mini lane per session (Codex nested under its parent, checkout labelled), a concurrent-activity warning, the latest hand-off. |
| 2 · Repo timeline | 3 (Release 1) | Swimlanes for one repo, Codex rows under their Claude parent; solid = exact link, dashed = heuristic; ambiguous and unlinked runs shown as such. |
| 4 · CLI (plumbing) | 1 (MVP) | Not a daily view. Prints the core's output as a table or JSON for tests and other front ends. |
| 3 · Step graph | Backlog | One step as Claude plan → Codex implement → Claude review, scrubbable. |
| 5 · Live grid | Backlog | One card per session under a feed of hand-offs. |
| 1 · Mod strip | Backlog | A strip above the Claude Code prompt. |

### User flow: one PLAN.md step

1. I keep `multi-link tui` open in a terminal pane started from `~`. It lists route-impact and posting-triage as busy, physaflow idle.
2. In route-impact I ask Claude to plan step 4. The overview shows step 4 as current and one busy Claude session.
3. Claude hands the step to Codex. The hook records the pair and the step id; the route-impact row shows a Codex lane and the hand-off.
4. Meanwhile posting-triage's Verify fails and its Claude session asks for permission. The terminal bells; that repo jumps to the top with ◆.
5. I press `l` on route-impact to see the live grid, or `t` for its timeline, then go back.
6. After Claude's review passes, PROGRESS.md logs step 4 and the overview shows "Verify ✓ step 4". Later, `g` replays the whole step across both agents for my Review notes.

## Proposed plan / timeline

**Release 1** (the portfolio release) is the core, the `multi-link` CLI, the TUI home screen and the repo timeline. Everything after is an ordered but uncommitted backlog. Each phase ends only when its Verify gives the expected result. Step-level detail lives in `PLAN.md`.

1. **Phase 0: Spike with go / no-go.** On real data, answer: (a) for each invocation form (exec, resume, plugin, background), where is the child Codex id observable? (b) which signals give busy / waiting / idle for each harness? (c) is zoetrope's core a usable library crate? (d) does agtrace already link Claude → Codex? Collect scrubbed fixtures, including negative cases (`codex` mentioned but not launched; a manual Codex run in the same folder), an ambiguous case (two hand-offs seconds apart in one checkout), a resume, a background launch, a stale session and a malformed line. Hand-label the expected outputs.
   - Verify: `docs/phase0.md` answers (a)–(d), each marked go or no-go, and `fixtures/expected/{sessions,links,steps}.json` exist. A no-go on (a) means links are heuristic-only, and this doc is revised before Phase 1.
2. **Phase 1: MVP, core + CLI.** Parsers, repo mapper, state machine, step parser, linker; `multi-link sessions`, `multi-link links`, `multi-link steps`.
   - Verify: `cargo test` passes, and each of the three commands run with `--root fixtures --now <fixed time> --json` equals its expected file. Negative fixtures produce no link; the ambiguous fixture produces `ambiguous`.
3. **Phase 2: Hook.** `multi-link hook install` (prints the snippet); exact links and step tags when the child id is observable.
   - Verify: for one real hand-off, the child id the hook records equals the id read independently from `~/.codex/sessions`; a hand-off without an observable id stays heuristic.
4. **Phase 3: TUI home + timeline + open (Release 1).** Screens 6 and 2, needs-you sorting, bell, FR18 resume and app link.
   - Verify: snapshot tests of both screens on fixtures pass; home refresh on my real `~` is 2 s or less; opening a running fixture session never calls resume.
5. **Backlog (ordered, uncommitted):** step graph (screen 3), live grid (screen 5), tmux / Herdr jump, desktop app with notifications, mod.

### Open questions

- [x] Can a terminal program open a running desktop-app session? Yes: `open "claude://code/continue?session=local_<id>"` switched the app to a live route-impact session (tested 2026-10-29). Format found in repo-sessions' code; undocumented.
- [ ] Phase 0 (a)–(d): child-id observability, state signals, zoetrope crate, agtrace overlap.
- [ ] How to find the `local_` app id of a finished desktop-app session.
- [ ] Does `codex://threads/<id>` open a Codex desktop thread?
- [ ] Should Claude's hand-off prompts carry a `step N` tag by convention, so step assignment is explicit? (A `/progress-entry` or hand-off skill could add it.)
- [x] Final project name: multi-link.

## Review log

Codex critique, `codex exec` in a read-only sandbox, 2026-10-29. It also inspected my local `~/.claude` and `~/.codex` file formats (field names only). Verdict: ready with changes.

| # | Finding | Decision | Reason and change |
| --- | --- | --- | --- |
| 1 | "Exact" hook links lack a correlation protocol | Accepted | Exact only when the child id is observed; Phase 0 question (a) is a go / no-go. Codex found the id in some tool results, not all. |
| 2 | One step per session can't model the workflow | Accepted | Work segments with step + evidence; MVP uses explicit tags or `unknown`; history-based inference deferred. |
| 3 | Transcript activity doesn't establish busy / waiting | Accepted | Session state table with evidence and stale / unknown; alerts only on supported waiting signals. |
| 4 | Heuristic can invent parents; confidence is uncalibrated | Accepted | Recognised invocation forms only, exact-checkout matching, `ambiguous` result, "score" not probability. Also fixed my earlier claim that worktrees make links exact. |
| 5 | Verify has no evidence contract | Accepted | Supported PLAN / PROGRESS convention; Verify per step with source (reported / observed) and staleness. |
| 6 | Verify gates can pass without proving the claims | Accepted | Negative and ambiguous fixtures, fixed clock, expected files for all three commands, hook ids checked independently, TUI snapshots, 2 s as a pass threshold. |
| 7 | Time window hides active sessions | Accepted | Active sessions found from live records, separate from the history window; incremental-read recovery rules. |
| 8 | Dependencies unresolved while scope expands | Partly accepted | Phase 0 gets go / no-go questions; Release 1 defined; step graph, live grid, desktop and mod moved to backlog. Rejected cutting Release 1 to CLI + overview only: the repo timeline is the view I most want and is cheap once the core exists. |
| 9 | Session opening contradicts its own safety rule | Accepted | Running + unreachable → "unavailable"; resume only when finished; FR18 placed in Phase 3 with a test. |
| 10 | Concurrent activity mislabelled as concurrent writing | Accepted | Renamed to "concurrent activity in one checkout"; write detection out of scope. |

Later decisions (step convention, print-only hook install, autonomy gaps) are in `PLAN.md` → Review log.

### Sources

[zoetrope](https://github.com/furkankly/zoetrope) · [c9watch](https://github.com/minchenlee/c9watch) · [repo-sessions](https://github.com/gblacher2/repo-sessions) · [agtrace](https://github.com/lanegrid/agtrace) (READMEs read 2026-10-29)
