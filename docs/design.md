# multi-link — Design Doc

Oct 29, 2026 · Status: draft

> Exported from the claude.ai doc (working name "cclink") on 2026-10-29. **This repo copy is now the
> source of truth.** Changes made since the export: name → multi-link; supported PLAN / PROGRESS
> convention matches the owner's real format; `hook install` prints a snippet instead of writing
> `~/.claude/settings.json`. See `PLAN.md` → Review log. The architecture diagram was not exported.
>
> Revised in Step 0.6 (2026-10-29) with the Phase 0 results (`docs/phase0.md`): see "Phase 0 results
> and decisions" under Solution design. Sections changed: Glossary, Major decisions (Reuse), Linking
> algorithm, Repo and step assignment, Session state, Edge cases, Proposed plan, Open questions.
>
> Revised in Step 8.1 (2026-11-06): "Phase 8 decisions" (G0–G11, target screen) under Solution design;
> FR11 reworded to the step graph and mapped to need 4; Glossary, screen table, user flow and plan updated.
>
> Revised in Step 6.1 (2026-11-01): "Phase 6 decisions" (W0–W10, target screens) under Solution design;
> FR10 reworded to the swimlane repo view.
>
> Revised in Step 3.2 (2026-10-30): "Phase 3 decisions" (T1–T11) under Solution design, the NFRs
> "Incremental reads" and "Speed", and the "Opening a session (FR18)" table.

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
| Session record | Claude Code's `~/.claude/sessions/<pid>.json` for a running session: `pid`, `procStart`, `status`, `statusUpdatedAt`, `cwd`. |
| Task output file | The temp file a background Bash call writes to; its path appears in the tool result. It can vanish. |
| Part | One box in the step graph: a stretch of a Claude session, or one Codex run, cut at hand-offs. |
| Chain | One parent's hand-offs for a step with no hand-off for another step between them. |
| Run | The Codex turns one hand-off started; one session can hold several runs. |
| Fixture mode | Running with `--root <dir>`: reads a saved home-shaped folder; no process probing and no git lookups. |

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
| FR10 | Repo view (Phase 6, replaces the repo timeline): one swimlane per session active in the window, Codex rows nested under their Claude parent, bars only where transcripts show activity, an axis fitted to the work with long idle gaps folded, a link line per hand-off (solid confirmed, dashed matched by time), ambiguous links shown as such. | 2, 5 |
| FR11 | Step graph (Phase 8): for one PLAN.md step, its Claude parts and Codex runs in order, each with what it did, how it ended, its link confidence and where the time went, from tagged hand-offs only; an untagged or missing run is marked, never guessed. Replay is out of scope. | 4, 5, 6 |
| FR12 | Live grid (Phase 14): one card per working, waiting or stale session under a feed of hand-offs. | 1, 2, 5 |
| FR13 | Optional hook records exact links when the child id is observable, and explicit step tags. | 2, 3, 6 |
| FR14 | Optional alert, fired only on a supported "waiting on me" signal: terminal bell or macOS notification. | 1, 5 |
| FR15 | Backlog: desktop / menu-bar app on the same core, with native notifications. | 1, 5 |
| FR16 | Claude Code pane (Phase 16): a mod beside the chat showing the current repo's sessions by state (working, waiting, recent), this session marked, its Codex runs nested with `exact` / `likely`; data from `multi-link pane --watch --json`. Read-only. | 1, 2 |
| FR17 | Warn about concurrent activity in one checkout: 2+ busy sessions with the same working folder. Detecting actual file writes is out of scope. | 1, 6 |
| FR18 | Open a session where it came from (`open_in = origin \| terminal \| app`): finished → resume; running → jump to it, or show "unavailable" if it can't be reached; never resume a running session. `f` forks. | 1, 4, 5 |

### Non-functional requirements

- **Local and read-only.** No network calls; never writes to either harness's folders (`~/.claude/`, `~/.codex/`, including settings), PLAN.md or PROGRESS.md. multi-link's own state lives in `~/.multi-link/`.
- **Fails soft.** Unknown transcript records are skipped and counted, never fatal (both formats are undocumented).
- **Incremental reads (revised in Phase 3, T1).** A refresh re-parses only the files whose (path, length, mtime, inode) changed since the last pass, each from the start; unchanged files reuse their cached parse, and linking, state and windowing are recomputed from the cached results every pass, so an old parent is still found when a new child references it. A partial last line is buffered, never skipped; a truncated or replaced file is just a changed key. Byte-offset tailing was the original plan; it is deferred because whole-file re-parse of the few changed files was fast enough (Step 3.1), and it returns as its own step only if a later measurement misses the speed threshold.
- **Speed (pass threshold).** The home screen refreshes in 2 s or less on my laptop with my real `~/.claude` and `~/.codex`. Step 3.1 measured the sessions + links refresh on one warm `Scanner` at 29 ms and 31 ms (passes 2 and 3, real `~`, `PROGRESS.md` 3.1); the full home refresh including the overview is measured in Step 3.8. **Cold first pass, Phase 15 baseline (Step 15.0, 2026-11-11, release `c29c516`, real `~`, 3 fresh processes):** `multi-link overview --json` 9.02 / 7.99 / 7.95 s wall (median 7.99 s; user 7.23–7.27 s, sys 0.67–0.80 s); the TUI worker's first report 9.34 / 8.62 / 8.45 s without `--live` and 8.51 / 8.51 / 8.55 s with it (medians 8.62 s and 8.51 s). Both miss the threshold; Phase 15 F3 sets the budgets.
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
| Reuse | Write our own parsers, borrowing zoetrope's per-provider split (defensive wire model, path discovery, per-file stream) with credit; copy c9watch detection files with credit only if they fit | Phase 0 (c): zoetrope's public API drops the tool output and long commands we link on | Depending on zoetrope: lossy, private parsers, pulls in ratatui (see Phase 0 results) |
| Data source | Read files, never write them | Read-only is safe and matches every comparable tool | Wrapping the CLIs: breaks my existing workflow |
| Hook install | `hook install` prints the settings snippet; the owner pastes it | Keeps the read-only rule absolute; install stays visible and reversible | Writing `~/.claude/settings.json`: one exception makes the rule harder to trust |

### Phase 0 results and decisions

Evidence: `docs/phase0.md` (10 generated hand-offs with recorded ground truth, plus organic samples).
Field names are observed on real data, not documented, and may change with a harness update.

| # | Result | Decision | Why |
| --- | --- | --- | --- |
| P1 | (a) go: the child Codex id is observable for every launch form | Exact links per form: `exec` → `session id: <uuid>` line in the tool result; `resume` → the `<uuid>` argument; plugin → `[codex] Thread ready (<uuid>)` in the **subagent transcript** (`<session>/subagents/agent-<agentId>.jsonl`), linked to the parent session; background → the `session id:` line in the **task output file** named by the tool result | Each is an id the harness printed, so the link is a fact, not a guess |
| P2 | (a) the task output file is a temp file | When it is gone, the background link falls through to the heuristic; never marked exact | Exact means observed; a vanished file is no evidence |
| P3 | (b) no-go for *waiting*: neither harness writes a pending-approval signal | Phase 1 never emits `waiting`. "Needs me" comes from the Phase 2 hook (`PermissionRequest` / `Notification`, decision H3); without the hook it shows as unknown. Codex waiting stays unsupported | Showing "waiting" from silence would be a guess, and alerts (FR14) fire only on a real signal |
| P4 | (b) busy / idle come from the session record's `status` | The 300 s freshness limit applies to `busy` only: an old `busy` → `stale`; `idle` at any age stays `idle` | Live idle desktop sessions were seen 1.7–2.6 h without an update; a busy session updates as it works |
| P5 | (b) liveness is checkable | Live = `ps` shows the record's `pid` with a start time equal to `procStart`; gone or different start → `finished` (`process_exit`). If the probe can't run, trust the record and skip `finished` | pid + start time defeats pid reuse; confirmed outside the sandbox with `ps -p <pid> -o lstart=` |
| P6 | (b) transcripts carry no exit evidence | A transcript-only session is `unknown`, never `finished` | A missing record proves nothing (`claude -p` may never write one); a completed Codex turn ends the turn, not the process |
| P7 | (c) no-go on zoetrope as a library | Write our own parsers; borrow its per-provider split (wire model / discovery / stream) and credit it (MIT) in NOTICE | Its public API truncates commands to 200 chars and drops tool output, which is exactly where the child id lives |
| P8 | (d) go: agtrace links only within one harness | The differentiator (Claude → Codex links + PLAN.md steps) holds; re-check agtrace before publishing a comparison | Confirmed by its source at 7649b7e and by running it on sandbox copies: 0 cross-harness parents |
| P9 | Fixtures need a deterministic, machine-independent run | Fixture mode (`--root`): no `ps`, no git lookups (`repo` = `cwd`), task output read from `<root>/task-output/<file name>` | Saved pids and paths belong to another machine; tests must not depend on it |
| P10 | No prompt-tag convention exists yet | Phase 1 recognises no step tags: each session is one segment with `step: null` | The fixtures' prompts are placeholders, so a tag parser would have no evidence to test against |
| P11 | This repo's own PLAN.md uses `0.2a` | Step ids are `N` or `N.M` with an optional lowercase letter suffix on the last part (`0.2a`, `3b`) | multi-link must read its own plan, and inserting a step without renumbering is a real need; the grammar stays strict otherwise |
| P12 | Heuristic needs a bounded window | Candidates: same exact checkout, Codex start within 60 s after the call | All sampled launches started within 2 s; 60 s leaves room for slow starts while keeping candidates few. A ranking value, not calibrated |

The binding field-level contract for all of this is `docs/output-schema.md`.

### Phase 2 decisions (hook)

Planned 2026-10-29. Hook facts come from the Claude Code hooks docs (code.claude.com/docs/en/hooks, read
2026-10-29, Claude Code 2.1.303) and stay assumptions until Step 2.1 checks them on real payloads.

| # | Decision | Why |
| --- | --- | --- |
| H1 | A payload spike (Step 2.1) comes before the hook record contract and any hook code; it stores an allowlist of ids and shapes, not whole payloads | Payload fields for background calls, `-p` behaviour and when Notification fires are not documented; Phase 0 showed field guesses are where plans break |
| H2 | Hand-offs from `PostToolUse` (matcher `Bash`) | It carries `tool_use_id`, the same id as the transcript's `tool_use.id`, so a record joins an invocation exactly; it fires after the call, when its output exists |
| H3 | "Needs me" from `PermissionRequest` and `Notification` (`permission_prompt`, `elicitation_dialog`); not `idle_prompt` | These fire when Claude is blocked on me mid-task. An idle session already shows `idle` from its session record, so `idle_prompt` would only duplicate it |
| H4 | Every entry `async: true`; the hook prints nothing, exits 0, swallows errors and panics (no-op panic hook), and a watchdog exits it at 30 s | Claude never waits on it, and the docs say decision fields of async hooks have no effect, so a bug can't approve, deny or block anything. Async hooks have no enforced timeout, and stdin or a write can stall, hence a process-wide watchdog |
| H5 | Hook evidence (`hook`) ranks after the four transcript sources; on disagreement the transcript wins and the conflict is counted | The transcript is what the harness printed; the hook only adds value where the transcript lost evidence (a vanished background task file) |
| H6 | Step tag = first `Step <id>` (P11 grammar) in the recognised launch segment, read from transcripts | The command actually run is what Codex received, even after the plugin rewrites the prompt; transcripts make it work without the hook and on history. CLAUDE.md's hand-off prompt already starts "Read AGENTS.md, then do Step <id> in PLAN.md." |
| H7 | Phase 2 puts the step on the link (`Link.step`); session segments stay one per session | Splitting Codex sessions per hand-off would make `sessions` depend on the linker; the TUI can join links to lanes. Revisit in Phase 3 |
| H8 | `waiting` = the session's latest hook `waiting` record is newer than every transcript line (or the clearing signal 2.1 picks); checked after `finished`, before `busy`; no freshness limit; an uncleared waiting session is always listed | A prompt answered or denied writes a transcript line (to confirm in 2.1); a session can wait overnight, so neither age nor the 2 h window may hide it; a dead process isn't waiting |
| H9 | Hook crate depends on model + claude, not core, and has its own `session id:` parser | Keeps the per-Bash-call hook path small and lets Steps 2.5 and 2.6 run in parallel; ~10 duplicated lines, tested on the same evidence lines |
| H10 | One file per session, `<state>/hook/<session_id>.jsonl`, single `O_APPEND` writes under 4 KiB, ids and enums only; `<state>` = `$MULTI_LINK_HOME` or `~/.multi-link` | Parallel tool calls append without locks; a UUID check on `session_id`, refusing a state dir inside `~/.claude` / `~/.codex`, and never following symlinks keep every write inside the state dir; no conversation content leaves the transcripts |

| H11 | Waiting for Claude comes first from the live session record (`status: "waiting"`, `waitingFor`), process identity checked as for busy/idle; the hook's `waiting` (H8) is the fallback when no record exists. Supersedes P3's "no record field signals waiting" | Step 2.1 saw the record switch to `waiting` during a held permission prompt and back to `idle` after it; Phase 0 sampled only 5 records, none waiting. The record needs no hook install, so needs-me works out of the box; the hook still covers `-p` sessions and missing records |

Codex waiting stays unsupported (no Codex hook in scope). Enabling the hook in real repos is the owner's paste.

### Phase 3 decisions (TUI home, timeline, open)

Planned 2026-10-30 from a real-data measurement (PLAN.md → Phase 3 header; Codex critique in PLAN.md →
Review log). Binding field rules: `docs/output-schema.md` → Session, Overview, Open (Step 3.2).

| # | Decision | Why |
| --- | --- | --- |
| T1 | Speed first, offsets only if needed: a core `Scanner` caches each file's parse keyed by (path, length, mtime, inode); git lookups memoised per `cwd`, only for listed sessions; one `ps` per pass for every record pid | Measured before planning: `sessions --json` took 16.4–16.9 s warm, ~10 s of it per-session `git` / `ps` spawns and ~6 s re-parsing every file. Between refreshes only a few files change, so re-parsing them whole is cheap; offset tailing would rewrite every parser as a resumable fold. 3.1 then measured 29–31 ms warm |
| T2 | The overview (repo groups, needs-you order, Codex nesting, `claude ×N` merge, FR17 warnings, latest hand-off, per-repo plan line) is computed in `core::overview`, printed by `multi-link overview --json` and answer-keyed | Front ends contain no logic; the desktop app reuses it; a JSON key is reviewable where a screen snapshot is not |
| T3 | TUI stack `ratatui` + `crossterm`, crate `crates/tui` depending on `model` only | Standard pair; `TestBackend` renders without a terminal; no ncurses; depending on `model` only lets TUI steps run beside core steps |
| T4 | Screen snapshots are generated once, then reviewed line by line against `overview.json`, and pinned | Hand-typing a 100×30 screen is error-prone; the semantic truth is the overview key |
| T5 | Needs-you order: repos with a `waiting` session first, then `busy`, then the rest; inside a tier latest activity first, ties by repo path. Rows in a repo: waiting, busy, stale, idle, unknown, finished, then latest activity | The screen answers "which one needs me"; activity order keeps the hottest repo on top |
| T6 | Bell: one `\a` when a session id enters `waiting` between two refreshes (not on the first refresh, not repeated while it waits) | A bell per refresh trains me to ignore it; macOS notifications are desktop backlog (FR14/FR15) |
| T7 | Open is a pure decision in core, `open::action(&Session, fork)`: `resume` only for Claude `finished` (positive exit evidence); a running Claude (`busy`, `idle`, `waiting`, `stale`) gets the app link when its record has `hostSessionId`, else `unavailable`; Claude `unknown` and every Codex session get `unavailable` with the command as text. The action is recomputed from a fresh pass for that one id right before it runs. `codex://` links stay out | `unknown` can be a live process (a completed Codex turn, a record-less `-p` session), so only exit evidence makes resume safe; testable without a terminal; a stale report could have been resumed elsewhere since |
| T8 | Session gains `checkout` (git toplevel of `cwd`; fixture mode `cwd`) and `host_session_id` (record `hostSessionId`) in the one Session type | Worktree labels need the checkout (Codex 0.6 #1); the app link needs the host id, which differs from the transcript id |
| T9 | Refresh every 2 s on a background thread that owns the `Scanner`; the UI thread never blocks on disk. Running a program suspends raw mode, the alternate screen and input, and restores them on success, spawn failure and panic. A failing pass keeps the last good report and shows the error; hook read errors are diagnostics | A dashboard that dies on one bad file is worse than a stale one; a broken terminal after `claude` exits is worse still |
| T10 | Background hand-offs whose hook child is `null` (clean-env `-p`) keep today's rule: exact through the task output file while it exists, heuristic after. Teaching the hook the output path is backlog | It changes link evidence, not the TUI; the fallback is already honest |
| T11 | A Codex session sits under the parent of its latest same-repo hand-off, listed once, every hand-off kept in its `handoffs`; children ordered by that hand-off; a parent's place follows its own state; a repo's tier is the best of its sessions. A merged `claude ×N` row (4 or more quiet Claude rows) is not actionable: it expands before `o` / `f` | One child can have several parents (exec + resume); a fixed rule keeps the screen stable and open always targets one full id |

Step 3.2 clarifications (output-schema.md): the merge takes top-level Claude rows that are `idle`,
`unknown` or `finished` with no busy or stale child; several records for one id are `finished` only when
all are gone; `open` looks ids up with the window off; a child is nested only under a parent in its own repo (else top-level, its
`handoffs` still list the link); `fork` on a running session is `unavailable` with the fork command (the
app link cannot fork); when the process probe cannot run no session is `finished`, so nothing resumes and
record sessions follow their record status; ids that are not lowercase uuids are never put in argv.

### Phase 4 decisions (home screen cleanup)

Planned 2026-10-30 after the owner found the real home screen far noisier than mockup screen 6 (PLAN.md →
Phase 4 header has the real-`~` measurement; Codex critique in PLAN.md → Review log). Binding field rules:
`docs/output-schema.md` → Overview → Rows (Step 4.2); answer key `fixtures/expected/phase4/overview.json`.

| # | Decision | Why |
| --- | --- | --- |
| U1 | `-C` / `--cd` whose value is `.`, or `$PWD` / `${PWD}` written unquoted or inside double quotes, is not a directory move (`dir_moved` stays false). Single-quoted or backslash-escaped `$PWD` is literal text and still counts as moved, as does any other value (`/tmp/x`, `"$HOME/x"`, `"$PWD/x"`). With several `-C` / `--cd` flags the call moves if any of them moves; a `-C` with no value counts as moved. The tokeniser keeps, per word, whether a `$` was expandable; a preceding `cd` segment still moves. A literal absolute path equal to the cwd is out of scope | The grammar does no expansion, but these forms always mean the shell's cwd, which is the call's checkout. Measured: our own hand-off command passes `-C "$PWD"`, so 42 of 43 multi-link Codex calls skipped the heuristic and stayed `unlinked`. The recogniser never sees the cwd, so it cannot compare a literal path |
| U2 | `codex ×N`: quiet top-level Codex rows (state `idle`, `unknown` or `finished`; a top-level Codex row is one with no placing hand-off) merge into one MergedRow when a group has 4 or more, by the same rule and threshold as `claude ×N`. MergedRow gains `harness`. A merged row never mixes harnesses and never holds `waiting` / `busy` / `stale` rows; a group can show one merged row per harness | Measured: ~30 one-line Codex rows in one repo, nearly all finished unlinked runs. A finished unlinked run is history, exactly like a quiet Claude row; one rule for both keeps the screen predictable |
| U3 | Quiet repos collapse: a group whose tier is `other` and that has no `stale` session (rows, children, merged members) renders as one line (repo, its session count — every session shown in the group: rows, children and merged members — latest step and Verify) until `enter` on that line expands it. Expanding a repo shows its rows as today, merged rows still merged; `enter` on a merged row then expands it as today (two stages). Repo expansion is its own per-repo set, separate from today's merged-row expansion (also keyed by repo, so `enter` on one merged row still opens every merged row of that repo). `esc` keeps today's meaning: it clears both sets, collapsing every quiet repo again, and moves selection to the top. TUI presentation state only, keyed by repo path so it survives refresh; the overview report is unchanged | Mockup screen 6 shrinks idle repos. `stale` can be unfinished work (an open Codex turn, a busy record gone quiet), so a group with one stays expanded |
| U4 | Header summary: the top line counts `waiting`, `busy` and `stale` sessions (every session shown: rows, children and merged members), FR17 warnings (the number of Warning objects across groups), repos (groups) and sessions from the overview, and hand-offs as the listed links in `Refresh.links` | Answers "does anything need me" before scrolling. The overview keeps only each group's latest hand-off, so it cannot give a hand-off count; the refresh's link list can |
| U5 | Mini time bars: each session lane gets its span `started_at`..`last_activity` (idle gaps included) on one shared axis, `TimeWindow::default_for(report)`, in a fixed bar column for every lane; no bar below 60 columns; merged rows get no bar | The mockup's main visual. A bar means "existed", not "running" (state glyphs say what runs). A fixed column keeps one axis across lanes; merged members' spans can be disjoint, so one bar would lie |

Non-goals for Phase 4: Codex waiting detection; changing the default `--since 2h`; timeline changes; colour
themes.

### Phase 5 decisions (home redesign to the approved target)

Planned 2026-10-30 after the owner found the Phase 4 screen shorter but "still very hard to easily read and
understand, nothing like the mockup". The owner approved **board 7** on the mockup canvas (claude.ai
artifact R5Bh9iX6bsn3jSu8F8D58C, "7 · Home target: real data, 30 Oct 16:19") as the acceptance picture and
asked to prioritise UX/UI, readability and colour (PLAN.md → Phase 5; Codex critique in PLAN.md → Review
log). Binding JSON rules: `docs/output-schema.md` (SessionRow `made`, Overview `hook`, Phase 5 fixture
results). Binding picture: the styled target golden `fixtures/expected/tui/home-phase5.txt` +
`home-phase5.styles.txt` (Step 5.1, drawn by hand from these rules; Step 5.4 must match it cell for cell).

Why Phase 4 missed: it was built rule by rule (merge, collapse, header) with colour a non-goal and no
picture to check against. Rows were named by id prefixes, most rows were old `unknown` sessions, bars were
a thin dotted track, links used internal words (`heuristic exec`).

| # | Decision | Why |
| --- | --- | --- |
| V0 | JSON stays stable; presentation lives in the TUI. Core JSON gains only additive fields (V1 `made`, V8 `hook`); every existing value and row order is unchanged. Windowing, ordering, the cap and combined lanes are a home view model in the TUI (`crates/tui/src/view.rs`) built from the overview + links | The JSON is a contract other front ends and tests rely on; what fits on one screen is a display decision |
| V1 | Rows are named by step. SessionRow gains `made`: steps of the links this row made, newest first. A Claude row's label is its latest made step; a Codex row's is its placing link's step. No step: `claude · no hand-off`, `codex · no parent found`. A Claude row with no made step that needs attention names its branch and state: `claude main · busy now`, `claude main · waiting since <time>`, `claude main · stale since <time>` (branch left out when `null`; `<time>` is the row's `last_activity`, the only status time the overview has). Id prefixes leave the home screen (the timeline and `o` keep ids) | The owner thinks in PLAN.md steps, not ids; `handoffs` lists only links into a Codex row, so the parent side needed its own field |
| V2 | One lane per step, only when it is true: a Claude row with exactly one Codex child whose placing step equals the parent's latest made step (or either is `null`) is one lane, label `step X · claude → codex · <link words>`; the Claude span is drawn as `━`, the Codex span as `█`, each mapped on its own times, Codex drawn last (on top). Otherwise parent and children are separate lanes. A combined lane sorts by the Claude row's `started_at` and is inside the window when either row's `last_activity` is. On a combined lane `o` opens the Claude session and `enter` splits it into two lanes | Halves the rows for the normal one-step hand-off without hiding a second child or a mismatched step; a child may start or end outside its parent's span |
| V3 | Rows per repo: `waiting`, `busy` and `stale` lanes always show, whatever their age and outside any cap, ranked by the lane's own state and its child's (waiting, busy, stale). Then quiet lanes by `started_at` ascending (the step chain reads top to bottom), only those whose `last_activity` is inside the bar window, at most 8. Every other quiet lane is counted in one line `N earlier session(s) · enter shows all`. The TUI stops drawing `claude ×N` / `codex ×N` (core still emits MergedRow; the view flattens it) | A prompt can wait overnight (Codex P1); history is reachable but does not crowd out what is running |
| V4 | Colour is a requirement with tested contrast: one theme of explicit fg/bg pairs (table below); bright colours for busy/waiting, dim for idle/done/unknown bars; truecolor when `COLORTERM` is `truecolor`/`24bit`, else 256-colour indexes; every text pair ≥ 4.5:1 on its background in both palettes. The app paints its own background, so the ratios hold whatever the terminal theme. No DIM modifier anywhere (its rendering is terminal-defined, so its contrast cannot be tested); only BOLD. `NO_COLOR`: no colour, the track becomes `·`, spans keep `━` / `█`, every state keeps its glyph. Cards have rounded borders: waiting yellow, busy orange, else the border grey; the border, the tint and the title glyph follow the group's `tier` (waiting > busy > other), so a card with both waiting and busy sessions is yellow | Readability was the owner's first complaint; untested colour can make text unreadable on some themes |
| V5 | Bars like the mockup: a fixed bar column right of the label column; track cells are blanks on the track background; Claude `━` orange, Codex `█` blue; clipped ends `◂` / `▸`; a zero-length span is one cell. Axis labels above the first card in local time, the last one `10:25pm now` | The bar is the screen's main visual; a solid block reads at a glance where a dotted track did not |
| V6 | Plain words: exact → `confirmed`, heuristic → `matched by time`, ambiguous → `N possible runs`, unlinked → `no run found`. Last line per expanded repo: `last: claude handed step X to codex at HH:MM (<link words>)` from `latest_handoff` (`step ?` when the link has no step) | Internal words (`heuristic exec`) meant nothing to the owner |
| V7 | Local time (12-hour, no leading zero, lowercase `am`/`pm` without a space): the UTC offset is read once per refresh for the report's `now` (`time` crate `local-offset`); fallback UTC, with the axis saying `UTC` after the first label. Fractional offsets work. When the window crosses local midnight the first axis label carries the date (`Oct 29 11:20pm`); a time not on `now`'s local date carries it too (`waiting since Oct 28 5:05pm`). JSON timestamps stay UTC. Limitation: a window across a DST change uses the offset at `now` | The owner reads clock time, not UTC; one offset per refresh keeps a frame consistent |
| V8 | Header: `All repos · last 2h` (bold; the bar window), `● N busy` orange, `○ N idle · N done or unknown` grey (done = `finished`), `N repos · N hand-offs` grey (hand-offs = listed links in `Refresh.links`), `◌ N stale` red after the busy count only when N > 0, `⚠ N shared checkouts` red after that only when there are FR17 warnings. At the right `◆ N need you` yellow whenever any session is waiting; with zero waiting and no hook records seen (`hook.records == 0`), `◆ needs-you partial: no hook records seen` yellow instead of `0 need you`; with zero waiting and records seen, nothing. Counts cover every session in the report (rows, children, merged members). Overview gains `hook: {records, latest}` (observed evidence, never "installed") | Answers "does anything need me" first; session records report waiting without a hook, so the count is always real, and only a zero is uncertain |
| V9 | Idle repos, truthful: a group whose tier is `other` with no `stale` session and no FR17 warning is one bordered line: `○`, name (bold, 18 cells), the plan wording (`latest step <id> <title>`, `no PLAN.md`, `PLAN.md not in the step format`, `no step logged yet`), then right-aligned counts by harness and real state (`1 claude idle · 2 codex done or unknown`, claude first, states idle then done or unknown) and `last HH:MM`; no path, no Verify. `enter` expands | The Phase 4 collapsed line claimed states and showed paths the owner did not need; stale or FR17 repos may be unfinished work, so they never collapse |
| V10 | Widths: ≥ 100 columns: board 7 layout, 46-cell labels. 80–99: 30-cell labels, the last line shortened to `last: step X → codex · <link words>`. 60–79: 24-cell labels, header drops the hand-off count. < 60: no bars, text rows only. Labels end with `…` on a word boundary; field priority: state glyph, step, harness, link words. Lists taller than the screen scroll with the selection | Board 7 is drawn at 120 columns; narrower terminals must degrade without cut words |

#### Target screen (Step 5.1): layout rules the golden is drawn from
Fixture `fixtures/sessions-phase5`, `--now 2026-10-29T23:20:00Z`, overview and links with `--since all`
(the keys in `fixtures/expected/phase5/`), fixed offset UTC−7, truecolor palette, nothing selected,
120 × 40 cells. The bar window is `TimeWindow::default_for(report)` (2 h ending at `now`: 21:20–23:20 UTC,
14:20–16:20 local) whatever `--since` was; so the quiet session that ended before 21:20 is in the report
and is counted by V3's `earlier` line. Columns are 0-based; `w` is the width.
- **Rows:** 0 header; 1 rule `─` across the width (border colour); 2 legend + axis; then cards, one blank row
  between cards; `h−2` rule; `h−1` footer. Rows in between are blank.
- **Header (row 0):** starts at column 1, parts separated by 3 blanks; the right part's last character is
  in column `w−2`, aligned with the card's right border.
- **Cards:** border columns 1 and `w−2`, rounded corners (`╭╮╰╯`, `─`, `│`), border cells on the screen
  background; inside cells (columns 2..`w−3`) on the card background (waiting and busy cards tinted, others
  the screen background); content from column 3 to `w−4`. Title row: state glyph, blank, repo name (bold),
  3 blanks, plan wording (V9 words, default colour), then ` · Verify ✓` grey, ` · Verify ✗` red,
  ` · Verify unclear` grey, or nothing when there is no entry (the ` · ` takes the Verify part's colour); right-aligned to column `w−4`: `N waiting`
  yellow, `N busy` orange, `N stale` red, `N idle` grey, joined by grey ` · `, zero counts left out (done or unknown are not counted here: those lanes are on screen; the idle line, which hides its lanes, counts them). On the title row and the idle line the plan wording is cut with `…` on a word boundary so at least 3 blanks remain before the right-aligned counts.
- **Lanes:** label at columns 3..48 (46 cells): state glyph (`◆` waiting yellow, `●` busy orange,
  `◌` stale red, `○` idle grey, `?` unknown grey, `✓` finished grey; a combined lane takes the first of its
  two states in the order waiting, busy, stale, idle, unknown, finished), a blank, then the V1/V2 text: the
  step words (`step X`) and a branch in the default colour, the word `claude` orange on busy/waiting lanes,
  ` · waiting since …` yellow (its separator included), everything else grey. Columns 49–50 blank. Bar cells: columns 51..`w−4`
  (`W = w − 54` cells; 66 at 120).
- **Bar mapping:** window `[s, e]`; a time `t` maps to cell `floor((t − s) · W / (e − s))` (integer
  milliseconds), capped at `W − 1`. A span `[a, b]` fills cells `cell(a)..cell(b)` (at least one); `a < s`
  starts at cell 0 and draws `◂` there; `b > e` ends at `W − 1` and draws `▸` there; a span entirely
  before the window is a single `◂` in cell 0, entirely after a single `▸` in cell `W − 1`. Span colour:
  bright on waiting/busy lanes, dim otherwise; the clip glyph takes the span's colour.
- **Legend + axis (row 2):** columns 3..48: `━ claude  █ codex  bright busy · dim idle/done` (`━` orange,
  `█` blue, words grey). Axis grey: start time left-aligned at column 51; the time at `s + window/2`
  centred on its cell (`col = 51 + floor(W/2)`, the 5-character label starts 2 cells left of it);
  `HH:MM now` right-aligned to column `w−4`.
- **Below the lanes:** `N earlier session(s) · enter shows all` (only when N > 0), then the V6 `last:`
  line, both grey from column 3.
- **Idle line (V9):** `○` grey at column 3, name bold default colour at 5..22, plan wording grey from 24,
  counts + ` · last HH:MM` grey right-aligned to column `w−4`.
- **Footer (row `h−1`):** from column 1, keys in the default colour, their words grey, joined by grey ` · `:
  `↑↓ move · enter expand / open · t timeline · o open session · f fork · q quit`; at the right, ending
  at column `w−2`, `times in UTC-7` grey (`times in UTC` on the fallback; the offset as `UTC+5:30` when
  fractional).

**Palette** (V4). Contrast ratios are WCAG 2.x relative-luminance ratios, computed for Step 5.1; Step 5.3's
test recomputes them.

| Role | Truecolor | 256 |
| --- | --- | --- |
| Screen background | `#0E1014` | 233 |
| Card background, waiting | `#17150E` | 234 |
| Card background, busy | `#17130F` | 234 |
| Selected row background | `#262C3A` | 235 |
| Text | `#D7DAE0` | 253 |
| Secondary grey | `#9AA1AE` | 248 |
| Claude orange (bright) | `#F2A36B` | 215 |
| Claude orange (dim, bars only) | `#8A6448` | 137 |
| Codex blue (bright) | `#7DB3FF` | 111 |
| Codex blue (dim, bars only) | `#4F79B3` | 67 |
| Needs-you yellow | `#E8C468` | 221 |
| Warning red | `#E86F6F` | 203 |
| Track (bar background) | `#22262F` | 236 |
| Card border, rules | `#2A2F3A` | 238 |

| Text colour | on screen | on waiting card | on busy card | on selected | 256: screen | waiting/busy card | selected |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Text | 13.60 | 13.04 | 13.19 | 9.97 | 13.40 | 12.19 | 10.83 |
| Grey | 7.33 | 7.03 | 7.11 | 5.38 | 7.88 | 7.17 | 6.36 |
| Orange | 9.27 | 8.89 | 9.00 | 6.80 | 10.30 | 9.37 | 8.32 |
| Blue | 8.85 | 8.49 | 8.59 | 6.49 | 8.57 | 7.80 | 6.92 |
| Yellow | 11.36 | 10.89 | 11.02 | 8.33 | 13.50 | 12.28 | 10.91 |
| Red | 6.29 | 6.03 | 6.10 | 4.61 | 6.29 | 5.72 | 5.08 |

Dim bar colours are not text and are not held to 4.5:1 (on the track: orange-dim 2.88, blue-dim 3.40
truecolor). The track is lighter than board 7's `#1A1D24` (1.13:1 on the screen) so a one-cell strip still reads as a track in a terminal: `#22262F` is 1.26:1 on the screen and 1.21:1 on the waiting card; the state glyph, not the bar, carries the state. 256-colour index values are the xterm cube
(`16 + 36r + 6g + b`, levels 0/95/135/175/215/255) and grey ramp (`8 + 10(i − 232)`).

**Styled golden format** (`*.styles.txt`, Step 5.3 writes the dumper): first line a `#` comment; then one
line per run, rows top to bottom, columns left to right, covering every cell:
`<row> <first>-<last> <fg> <bg> <modifiers>` (0-based, inclusive). `fg` / `bg` are `#RRGGBB` in the
truecolor palette (`-` for the terminal default); `modifiers` is `-` or `+`-joined ratatui names (`BOLD`,
`UNDERLINED`, `REVERSED`, …). A blank cell (`" "`) is written with fg `-` and without BOLD / DIM / ITALIC,
which do not show on a blank (UNDERLINED, REVERSED and CROSSED_OUT stay); a run is a maximal stretch of
cells equal after that rule. So the golden fixes what is visible, not how spans were split.

**What the target differs from board 7 in, and why:** the fixture adds a waiting row from yesterday (the
plan's acceptance case for V3), so the header shows `◆ 1 need you` instead of the hook note, the
repo-l card is yellow with a waiting tint instead of orange, and a `◆ claude main · waiting since Oct 28
17:05` lane with a lone `◂` comes first. Lanes start with a state glyph (V10's first field; needed for
`NO_COLOR`). Quiet lanes follow `started_at` (V3), so `codex · no parent found` sits between steps 4.4
and 4.5 rather than last. The hidden-session count is its own line (V3), not appended to `last:`. The
title has no `Phase 4 ·` (the plan parser has no phases) and says `Verify unclear` where board 7 said
`Verify not reported`: a logged Verify whose result is neither pass nor fail is reported, just not
recognised. Idle lines name harness and state counts (`1 claude idle · 2 codex done or unknown`) instead
of `5 codex runs`. The footer adds `f fork` and says which offset times use instead of the mockup's data
note. Step 4.1 reads `confirmed` (the plan asks for one exact link) where board 7 has `matched by time`. The second idle repo is Codex-only (`no PLAN.md`, `2 codex done or unknown`) where board 7's had an idle Claude session on a branch. The plan wording has no pill border (a terminal cannot draw one inline). The hook note, when shown, reads `needs-you partial: no hook records seen` instead of `needs-you off: hook not installed` (V8: records are evidence, not installation). The track is lighter (Palette). Repo names are fixture placeholders (`repo-l`, `repo-m`, `repo-n`).

Non-goals for Phase 5: Codex waiting detection; installing the hook for the owner (it stays print-only);
the timeline screen; themes beyond the one palette.

### Phase 6 decisions (home in words, repo swimlanes)

Planned 2026-11-01 after the owner ran Phase 5 on the real screen: the bars on home and the timeline screen
were "really hard to read … random blocks and timestamps on top". Phase 5 closes as not meeting the need for
its bars. The owner approved two targets on the mockup canvas (claude.ai artifact R5Bh9iX6bsn3jSu8F8D58C),
both drawn on the 120×40 cell grid from real data: **board 7c** (home: no timeline, one sentence per session)
and **board 8c** (repo view: board 2's swimlanes on real data). They show real prompts, so they are described
here, never committed as images (W10). Codex critique: PLAN.md → Review log. Binding JSON rules:
`docs/output-schema.md` (the four added fields, Phase 6 fixture results). Binding pictures: the styled target
goldens `fixtures/expected/tui/home-phase6.txt` / `.styles.txt` and `repo-phase6.txt` / `.styles.txt` (Step
6.1, drawn by hand from the rules below; Steps 6.3 and 6.4 must match them cell for cell).

Why Phase 5's bars failed: a bar ran from a session's start to its last activity, so a session open for two
days filled the window; real work comes in 3–5 minute bursts, which a 2-hour axis turns into specks; 6 of the
rows on the owner's screen were Codex Desktop imports, not runs. Measured 2026-11-01 on local data: 140 of 260
Codex rollouts are Codex Desktop imports; a 5-minute burst gap splits 2 of 117 closed turns in the 12 newest
Claude transcripts.

| # | Decision | Why |
| --- | --- | --- |
| W0 | JSON stays stable; presentation stays in the TUI. Every existing key keeps its value (`last_activity`, `state`, window membership, groups, row order, merges); imported sessions stay in every report. Additive only: Session and SessionRow gain `activity`, `asked`, `imported`, `state_since`. Hiding imports, counting them and every screen rule live in the TUI view models | The JSON is a contract; the screen is what failed, not the data |
| W1 | Recorded activity = bursts. Sources: every timestamped record of the session's own file and (Claude) of its subagent files, minus records inside imported turns (W2), minus the Codex `session_meta` header (a file header, not work; in an import it carries the copy time) and minus Claude records whose `sessionId` differs from the session's (inherited history, as in W3). An `imported` rollout has no activity. Timestamps sorted and deduplicated; consecutive ones at most 300 s apart form one burst `[first, last]`; a single record is `[t, t]`; bursts are clipped to the report window `[now − since, now]` (none after `now`; `--since all` clips only at `now`). Screens say "active", never "worked" or "busy time". Limitation: a tool call silent for over 5 min shows as a gap (the phase6 fixture has one: Claude waits 6.5 min on a Codex run) | Start-to-last-activity bars filled the window; bursts show when work happened. A turn-based rule was rejected: one real turn left waiting stayed open for 52 h |
| W2 | Codex Desktop imports. In file order, an imported turn opens at a `task_started` whose `turn_id` starts `external-import` and closes at its matching `task_complete` (inclusive), the next `task_started`, or end of file; every record from open to close is excluded from `activity` and `asked`. `imported` = the rollout has at least one `task_started` and all of them are imported. State and `last_activity` unchanged (W0). Undocumented format, observed 2026-11-01 (all such rollouts have originator `Codex Desktop`): an assumption | Imports are Claude chats copied into Codex Desktop, stamped at copy time; they are not runs |
| W3 | `asked` = the session's first prompt. Claude (main transcript only): the first `user` record whose `sessionId` equals the session id, not a tool result, with string content or a first `text` block; leading whitespace trimmed; skipped when blank or when it then starts with `<`. Codex: the first `user_message` event (`payload.message`) outside an imported turn. Whitespace runs collapse to one space; cut at 120 characters on a char boundary with `…`. Label order on screen: Claude latest made step (`step 6.3`) › `asked` in quotes › `claude · no prompt`; Codex its placing link's step › `asked` › `codex · no prompt`. Ids stay in `o`, the selected line and JSON. JSON now exports prompt text (README) | The owner recognises a session by what they asked, not by an id |
| W4 | Home is words (board 7c): no bars, no axis. Header as V8 with `● N working now`. A repo with a waiting, busy or stale session is a card; a repo with other activity in the window is one bordered sentence; repos with no activity in the window share one `quiet` line; imported rows are hidden and counted in the footer. Layout: Target screens below | The owner reads sentences faster than bars |
| W5 | Status phrases and times. `state_since`: waiting/stale → the record's `statusUpdatedAt` or the winning hook record's `at`; Codex `unknown` / `turns_complete` → the last non-imported `task_complete`; otherwise null. Phrases: `working now · for N min` (from the last burst's start; `working now` with no burst), `waiting on you · since T`, `stale since T`, `idle · last active N min ago` (last burst end, else `last_activity`), Codex `done T`. Codex "done" means its last turn completed; there is no process-exit evidence. Times: only `last active …` is relative (`N min ago` under 60 min, else a clock time); every other time (`since`, `done`, `at`, `started`, `last hand-off`, `active T1–T2`) is a clock time: `9:05pm` for the last 24 h (P3, including the previous local date), `Mon D` at 24 h or older. Minutes are whole minutes rounded down, at least 1. Hand-offs: `handed step X to codex` / `handed work to codex` (untagged); outcome `codex done` (child `turns_complete`), `codex running` (child busy or stale), `codex state unknown` (other child states), `no codex run found` (unlinked), `N possible runs` (ambiguous). No generated `?` placeholder anywhere (prompts may contain `?`) | Times the owner can act on, from evidence the core already has |
| W6 | The repo view (board 8c) replaces the timeline screen; `enter` and `t` on a home repo open it. Rows: top-level non-imported sessions with activity in the window, by `started_at` descending (newest first), then id; each Codex child nested under its parent. Two text rows per lane: label + bars, then a sub line. Bars are W1 bursts as `█`; `✓` after a `done` Codex run; `◆` after a waiting session's last burst. No rows: `no recorded activity in the last 2h` | Board 2's swimlanes, now drawn from when work happened |
| W7 | The axis fits the work: fitted range = first burst start of the shown rows to `now`, never outside the report window. Idle stretches over 15 min with no burst on any shown row fold: at most 3 (the longest; ties to the earlier), each a 5-column band with its length in the axis row; `z` unfolds. Ticks from the ladder 1, 2, 5, 10, 15, 30 min, 1, 2, 6, 12 h, 1 d. `+`/`-`/`←`/`→` switch to an explicit range (folds off, the last label shows its time, `now` only when in range); `0` returns to fitted | Short bursts get wide bars; a 43-minute lunch does not eat the screen |
| W8 | Link lines: at the hand-off column, a vertical line between the parent's bar row and the child's: `│` confirmed, `╎` matched by time; an ambiguous child stays top-level with `N possible runs`. `i` shows imported rows (dim, marked `import`). Bottom `selected` line: harness, first 8 characters of the id, started, `active about N min in K stretches`, its hand-off; `o` opens, `f` forks (Phase 3 safety rules) | The need: which run came from which session, and how sure the link is |
| W9 | Colour and widths: Phase 5 roles keep their values and contrast tests; two roles are added (done green, child row background) and pass the same ≥ 4.5:1 test. NO_COLOR keeps every glyph and word. Repo view below 80 columns drops the bars and shows the lanes as a list (same labels and sub lines). Home below 80 columns drops the branch, then shortens labels with `…` on a word boundary | Board 8c needs a done mark and a nested-row tint the Phase 5 palette lacked |
| W10 | Privacy: fixtures use synthetic prompts only. Real renders, board screenshots and JSON with real prompts stay in scratch storage. Each step's Review checks every committed file for real prompts, real ids, secrets and personal paths | Screens and JSON now carry prompt text; the repo is public |

Step 6.6 extends recognition and W3 (2026-11-02):
- **X1 Heredocs:** outside quotes, `<<WORD`, `<<'WORD'`, `<<"WORD"` and `<<-WORD`
  queue bodies to skip after the current line. End delimiters match the whole line; `<<-`
  permits leading tabs. Unterminated bodies consume through EOF; body prose is never tokenised.
- **X2 Grouping:** unquoted parentheses split segments; whole-word unquoted braces are dropped.
  A `cd` within a group still disables directory-based matching. Command substitutions remain opaque
  words, including their quoted contents; their text cannot create a launch.
- **X3 Wrappers:** skip leading `nohup`, `env` with optional `NAME=value` assignments and no options,
  and `timeout` / `gtimeout` with one nonnegative numeric duration (optional `s/m/h/d` suffix).
  These wrappers can stack; other prefixes remain unsupported.
- **X4 Prompt fallback:** when no non-imported `user_message` supplies a prompt, use the first
  non-imported `response_item` user message whose first `input_text`, after leading whitespace,
  is nonblank and starts with neither `# AGENTS.md instructions` nor `<`. Apply W3 normalization.
  Event prompts retain priority, even when blank or later in the file. Claude's wrapper rule is
  unchanged: a prompt inside a command tag is skipped, without unwrapping.

Non-goals for Phase 6: hook-free needs-you detection; the step graph (board 3, backlog); live grid; changing
`--since`.

#### Target screens (Step 6.1): layout rules the goldens are drawn from
Fixture `fixtures/sessions-phase6`, `--now 2026-11-03T06:25:00Z`, `--since 2h` (the TUI default; keys in
`fixtures/expected/phase6/`, links = `multi-link links` with the same flags), fixed offset UTC−8 (`now` is
Nov 2 22:25 local), truecolor palette, 120 × 40 cells. Columns are 0-based, inclusive; `w` = 120, `h` = 40.
Every cell paints its background (screen unless stated). Text styles are BOLD or nothing (V4: no DIM).

**Home (`home-phase6.*`), nothing selected.**
- **Rows:** 0 header; 1 rule `─` at columns 1..`w−2` (border colour); 2 blank; then the cards, then the idle
  sentences (each in overview repo order), then the quiet line (always last, its groups in overview order), one
  blank row between blocks; `h−2` rule as
  row 1; `h−1` footer.
- **Header (row 0):** from column 1, parts separated by 3 blanks: `All repos · last 2h` (text, bold),
  `● N working now` (orange; N = busy sessions), `◌ N stale` (red, only when N > 0), `○ N idle · N done or
  unknown` (grey; done or unknown = `unknown` and `finished`), `N repos · N hand-offs` (grey; repos = groups
  in the overview, hand-offs = listed links). Imported sessions are not counted. Right part ending at column
  `w−2`: `◆ N need you` yellow when any session waits, else V8's hook note, else nothing.
- **Card** (a group with a waiting, busy or stale non-imported session): rounded box at columns 1..`w−2`,
  border yellow (tier waiting) / orange (busy) / red (only stale); border cells on the screen background;
  inside cells (2..`w−3`) on the V4 card tint (waiting / busy; screen for stale). Rows inside:
  1. title: tier glyph at 3 (`◆` yellow / `●` orange / `◌` red), repo name (last path component) bold at 5,
     2 blanks, plan wording: `step <id> <title>` in text colour, or `no PLAN.md` / `PLAN.md not in the step
     format` / `no step logged yet` grey; right-aligned to column `w−5`: `N waiting` yellow, `N working now`
     orange, `N stale` red, joined by grey ` · `, zero counts left out.
  2. blank.
  3. one row per waiting, busy or stale session (that order, then `started_at`): state glyph at 3, harness at 5
     (`claude` orange / `codex` blue), label at 15 in text colour (W3 order; a prompt is quoted `“…”`; cut on a
     word boundary with `…` inside the closing quote so it ends by column 63), status phrase at 66 (W5; bold;
     yellow waiting / orange working / red stale), branch grey right-aligned to column `w−5` (left out when
     null).
  4. the rest (the group's other non-imported sessions, children and merged members included), grey at 5:
     `N older session(s), none active in the last 2h · enter shows all` or `N other session(s), K active in the
     last 2h · enter shows all`; with no rest, `active about M min in the last 2h, in K stretch(es)` over the
     card's sessions (M = total burst length, W5 minutes).
  5. hand-off: when the group has a `latest_handoff`, its own row, grey at 5: `last hand-off  T · claude handed
     step X to codex · <outcome>` (two blanks after `last hand-off`); otherwise ` · no hand-offs` is appended to
     row 4.
- **Idle sentence** (a group with no card but a burst in the window): grey box 3 rows high; `○` grey at 3, name
  bold at 5, then 2 blanks and grey `idle · last active T · <hand-off>` (T = the latest burst end of the group's
  non-imported sessions, W5) where the hand-off is `claude handed
  step X to codex at T, <outcome>` / `claude handed work to codex at T, <outcome>`, or `no hand-offs`.
- **Quiet line** (every remaining group, i.e. no burst in the window): `quiet` grey at 3, then at 9 the groups
  joined by ` · `, each `name (last active T)` from the group's `last_activity`, or `name (only Codex Desktop
  imports)` when all its sessions are imported.
- **Footer (row `h−1`):** from column 1, key in text colour, a blank, its words grey, 3 blanks between pairs:
  `↑↓ move`, `enter repo view`, `o open session`, `q quit`. Right-aligned to `w−2`, grey: `N Codex Desktop
  import(s) hidden · ` (only when N > 0) then `times UTC-8` (`times UTC` on the fallback; `UTC+5:30` when
  fractional).

**Repo view (`repo-phase6.*`) on repo-r, its newest lane (the parent) selected.**
- **Row 0:** `Sessions in <name>` bold at 1, 2 blanks, plan wording grey; right-aligned to `w−2`, grey: `last
  2h · active T1–T2` (first burst start – last burst end of the shown rows, `HH:MM`). Row 1 rule.
- **Row 3 (axis):** `session` grey at 2; bar columns 44..`w−3` (74 at 120); tick labels and fold lengths
  grey; `HH:MM now` right-aligned to `w−3`. Row 4 rule.
- **Mapping:** the fitted range minus the folded gaps is a list of segments. With F folds, the segments share
  `A = 74 − 5F` columns in proportion to their length in integer milliseconds: `floor` each share, then hand the
  remaining columns to the largest fractional parts (ties to the earlier segment). Segment i starts after the
  previous segment and its fold band; a time `t` in segment `[a, b]` of width `wᵢ` maps to `xᵢ + min(wᵢ − 1,
  floor((t − a) · wᵢ / (b − a)))`. A burst fills `col(from)..col(to)` (at least one cell).
- **Ticks:** at local times that are multiples of the step (from local midnight), inside a segment, never at
  `now`. The step is the smallest ladder value for which every two adjacent ticks in one segment are ≥ 13
  columns apart (a 5-character label plus 8 blanks). A tick's label is drawn at its column when it fits inside
  its segment and leaves at least one blank before the `now` label. Grid `│` (track colour) at every tick column.
- **Fold band:** 5 columns `≈ ≈ ≈` (border colour) right after its segment; its length in the axis row starting at
  band + floor((5 − length) / 2) (`42m`; `1h05` from 1 h;
  `13h` from 10 h; `2d` from 100 h), grey.
- **Grid and fold rows:** from row 5 to the row after the last lane's sub line; under bars, text and lines.
- **Lanes:** from row 6; a parent lane is its bar row and sub line, then each child's two rows after one blank
  row, then one blank row before the next parent. Parent: harness at 2, label at 9 (cut to end by column 41);
  sub line grey at 4: `<branch> · <W5 status phrase>` (no branch → the phrase alone). Child (both rows on the
  child background across the width): `└▸` grey at 3, harness at 6, label at 12 (cut to end by 41); sub line
  grey at 6: `<link words> · N s after the call` (`confirmed` / `matched by time`; N = child `started_at` −
  link `at` in whole seconds, `N min` from 60 s). Bars: `█` orange (Claude) / blue (Codex). `✓` green in the
  cell after a done Codex run's last burst; `◆` yellow after a waiting session's last burst (both only inside
  the bar columns).
- **Link line:** at `col(link at)`, on every row strictly between the parent's bar row and the child's bar row:
  `│` (confirmed) or `╎` (matched by time), grey.
- **Selected lane:** both rows on the selected background across the width, `▌` in the harness colour at
  column 0.
- **Hidden imports:** every imported session of the group in the report, times from their `last_activity`;
  one row after the grid, grey at 2: `▸ N Codex Desktop import(s) at HH:MM hidden (copies
  of Claude chats, not runs)` (`between HH:MM and HH:MM` when they differ).
- **Bottom block:** row `h−11` rule; row `h−10` `selected` grey at 2, then at 12 in text colour `<harness>
  <id[0..8]> · started T · active about M min in K stretch(es) · <hand-off words>` (`handed work to codex at
  T`, `handed step X to codex at T`, or `no hand-offs`); row `h−9` at 12: `o open in Claude` and `f fork` (keys
  in text colour, words grey, 3 blanks between). Row `h−4` legend from column 1, items 3 blanks apart: `█
  claude active` (█ orange), `█ codex active` (█ blue), `│ confirmed link`, `╎ matched by time`, `◆ waiting on
  you` (glyph and words yellow), `≈ idle folded`, `✓ done` (✓ green); every other glyph (`│`, `╎`, `≈`) and word grey. Row `h−2` rule, row `h−1` footer like home:
  `↑↓ select`, `o open`, `f fork`, `+/- zoom`, `z unfold idle`, `i imports`, `esc home`, `q quit`; right
  `times UTC-8`.

**Palette additions** (W9; every other role is the Phase 5 table). Ratios as in Phase 5 (WCAG 2.x).

| Role | Truecolor | 256 |
| --- | --- | --- |
| Done green (text) | `#7CCB8E` | 114 |
| Child row background | `#12151B` | 234 |

| Text colour | on child row | on selected | 256: child | 256: selected |
| --- | --- | --- | --- | --- |
| Text | 13.05 | 9.97 | 12.19 | 10.83 |
| Grey | 7.04 | 5.38 | 7.17 | 6.36 |
| Orange | 8.90 | 6.80 | 9.37 | 8.32 |
| Blue | 8.50 | 6.49 | 7.80 | 6.92 |
| Yellow | 10.90 | 8.33 | 12.28 | 10.91 |
| Red | 6.04 | 4.61 | 5.72 | 5.08 |
| Green | 9.39 (screen 9.78) | 7.18 | 9.82 (screen 10.79) | 8.72 |

Grid lines use the track colour and fold bands the border colour; neither is text.

**What the targets differ from boards 7c / 8c in, and why** (PLAN.md wins over the boards):
- Words: "active" for "worked" (W1); `done` for "finished" (W5); `idle · last active N min ago` for "idle
  since HH:MM" / "last work" (W5); `enter repo view` for "repo timeline" (W6 replaces the timeline).
- Home: the fixture has a waiting session from yesterday, so the header shows `◆ 1 need you` instead of the
  hook note and the first card is yellow with two session rows; cards keep the V4 tint (board 7c drew none);
  the header keeps V8's counts (`○ 5 idle · 3 done or unknown   5 repos · 2 hand-offs`), where board 7c showed
  only in-window idle; the hand-off row uses W5 words; zones are offsets (`times UTC-8`), not names (`PST`),
  because only the offset is known.
- Repo view: the fold is 5 columns with its length in the axis row (W7), where board 8c drew 7 columns and
  "43 min idle" under the grid; sub lines use the W5 phrase; the legend says `│ confirmed link` (any exact
  evidence, not only the hook) and adds `✓ done`; the footer swaps `enter open` for `o open` (W8) and adds
  `i imports`; selected-line keys are in text colour like the footer; the selected background is the Phase 5
  role `#262C3A` and grid lines use the track colour (board 8c: `#1B2130`, `#252A33`).
- Repo names and prompts are synthetic placeholders.
- W10 exception: the private canvas artifact id (`R5Bh9iX6bsn3jSu8F8D58C`, cited in this doc and PLAN.md) is a
  link to the owner's own private canvas, not a session id; W10 reviews accept it.
- Known wording issue (kept, PLAN W4 is literal): the card's `· enter shows all` opens the repo view, which shows
  only sessions active in the window, so it does not show "all" older sessions. Step 6.3 / 6.5 should report it
  to the owner rather than change the golden.

### Linking algorithm

1. **Recognised invocations only.** A hand-off is a Bash `tool_use` in an `assistant` record whose command has a segment (split on `&&`, `||`, `;`, `|`, newlines outside quotes) of the form `codex exec …`, `codex [exec] resume <uuid>`, or `node …/codex-companion.mjs task …` (the plugin, called from a subagent). Other commands that merely mention `codex` are ignored. Exact grammar: `docs/output-schema.md`.
2. **Exact.** A link is exact only when the child Codex session id is explicitly observed, checked in this order: the `resume` argument; a `session id: <uuid>` line in the tool result; `[codex] Thread ready (<uuid>)` in the result (plugin); a `session id:` line in the task output file the result names (background); *(Phase 9, R4)* exactly one `session id:` line in the file the launch segment's stdout is redirected to (a literal path under the session's `cwd` or the temp dir, read safely). The hook records the same evidence; it does not make a guess exact. *(Phase 9, R7)* A `codex exec resume --last` fix round is matched to a turn of an existing run, which is a timing match, not an observed id, so it is `heuristic` with evidence `resume_turn` (the one evidence value a non-exact link may carry); an observed id for the same hand-off still makes it exact.
3. **Heuristic.** Without an observed id: candidates are Codex sessions whose working folder equals the invocation's exact checkout (not the merged repo) and whose start falls within 60 s after the call, excluding children created by another invocation's exact link (a `resume` link doesn't claim its child, so a resume can't hide the run that created it). If a `cd` or `-C` / `--cd` in the command moves the launch elsewhere (`-C .` and `-C "$PWD"` do not move it, U1), no heuristic runs (unlinked), since the checkout is then unknown. One candidate → `heuristic` with a ranking value (`gap_ms`); several → `ambiguous`, listing them; none → unlinked. The ranking value is not a probability.
4. **Resume.** `codex resume` adds a hand-off to an existing Codex session instead of creating one, so a Codex session can have several hand-offs over time.

### Repo and step assignment

1. **Repo.** Each transcript records the folder it ran in (Claude's project folder name is lossy, so the core reads the path from inside the file). The core resolves the git root for grouping but keeps the exact checkout (main or which worktree) on every session.
2. **Supported convention** (the owner's real format):
   - PLAN.md step heading `### Step <id> — <title>`, where `<id>` is `N` or `N.M`, optionally followed by one lowercase letter (`0.2a`; decision P11), with a `- Verify:` or `- **Verify:**` line inside the step.
   - PROGRESS.md entries headed `### YYYY-MM-DD — Step <id>: <title>`, optionally `— <agent>`, newest first, with a Verify line.
   - Anything else → steps unknown.
3. **Work segments.** A session is split into time ranges; each gets a step only from explicit evidence (a `Step <id>` tag in the hand-off command, H6), else `unknown`. A long Claude session that plans step 4 and reviews step 5 gets two segments. Phase 1 recognises no tags yet (decision P10): one segment per session, step `unknown`. Phase 2 puts the tag on the link only (H7).
4. **Verify per step.** `{command, result, time, checkout, source}` where source is `reported` (PROGRESS.md) or `observed` (the command's run seen in a transcript). Marked stale if the checkout changed since. There is no repo-level "last Verify".
5. A repo without PLAN.md still shows; its sessions have no steps.

### Session state

Confirmed in Phase 0 (b) on real data; rule order and exact conditions are in `docs/output-schema.md` → State.

| State | Evidence |
| --- | --- |
| Busy | Claude: session record `status=busy`, updated ≤ 300 s ago. Codex: last `task_started` turn has no `task_complete`, last activity ≤ 300 s ago. |
| Waiting on me | Claude: the live session record's `status: "waiting"` + `waitingFor` (H11, found in Step 2.1); Phase 2 hook `waiting` as fallback when no record exists (H3, H8). Codex: unsupported. |
| Idle | Claude: session record `status=idle`, at any age (P4). |
| Finished | Claude: the record's process is gone, or its pid now has a different start time than `procStart` (P5). *Phase 9:* with no record, an uncleared `SessionEnd` hook record (R5), or a transcript `cost-state` after the last turn (R2). |
| Stale | `busy` evidence older than 300 s (record or open Codex turn). |
| Unknown | No usable signal: transcript-only sessions, Codex sessions whose turns all completed (P6). |

Silence alone never means busy or waiting, and Phase 9's `finished` rows rest on positive markers only (R2, R5). Alerts (FR14) fire only on a supported waiting signal (session record or hook).

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
| FR11 (need 4) | `core::graph` builds the report from tagged hand-offs, Codex turns and on-demand calls (G0–G6); `multi-link graph` prints it; the TUI draws it (G8–G10) |
| FR12 | Phase 14: `multi-link live` and the live grid screen (`l`, `tui --live`); Phase 14 decisions |
| FR15 | Backlog |
| FR16 | Phase 16: `multi-link pane` (`crates/cli`) and the mod in `mod/multi-link-pane/`; Phase 16 decisions |
| FR13 | `multi-link hook install` prints the hook entry for the owner to paste; the hook writes to `~/.multi-link/` (H2, H4, H10); step tags from hand-off commands (H6) |
| FR17 | Core compares the working folders of busy sessions |
| FR18 | See "Opening a session" below; Phase 3 |

### Opening a session (FR18)

Phase 3 rules (decision T7; exact table in `docs/output-schema.md` → Open). "Running" = `busy`, `idle`,
`waiting` or `stale`; "finished" = `finished` (session record whose process is gone, or pid reused).

| Session | Running | Finished | Unknown (no exit evidence) |
| --- | --- | --- | --- |
| Claude, record has `hostSessionId` (desktop app) | `open "claude://code/continue?session=<hostSessionId>"` (tested once, 2026-10-29); `f` → unavailable, fork command shown | `claude --resume <id>` (`--fork-session` with `f`) in its `cwd`, return on exit | — (a record session is never `unknown` unless its status is new) → unavailable, command shown |
| Claude, no `hostSessionId` (terminal) | unavailable, `claude --resume <id>` shown; jumping to a tmux / Herdr pane is backlog | Same resume as above | unavailable, `claude --resume <id>` shown |
| Codex | unavailable, `codex resume <id>` (`codex fork <id>` with `f`) shown | Codex has no exit evidence, so never finished | Same |

`codex://threads/<id>` (from repo-sessions' code) stays out until tested.

- **App vs terminal:** a running Claude session's `~/.claude/sessions/<pid>.json` has a `hostSessionId` (`local_…`) only when the desktop app started it. That id differs from the transcript id `claude --resume` uses.
- **Fork:** `claude --resume <id> --fork-session` or `codex fork` starts a side conversation without touching the original.
- **Unreachable running session:** no app link → "unavailable" with the command for me to run. Resume only once the session is finished; a record-only finished session with no transcript is unavailable too.
- **Fresh decision:** the action is recomputed from a fresh pass for the full id right before it runs, so a session resumed elsewhere since the last draw is not resumed twice.
- **Phase 9 (R3):** a session `finished` from its transcript or a `SessionEnd` record reopens by where it ran: `cli` / `sdk-cli` → resume in a terminal; `claude-desktop` → the app link when exactly one app record maps it to a `local_` id, else unavailable with the command; unknown entrypoint → unavailable.
- **Phase 3 scope:** resume for finished Claude sessions and the app link for running app sessions. Jumping to tmux / Herdr panes and `codex://` links are backlog.

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
- **Background task output gone:** the link falls back to the heuristic (P2).
- **Idle for hours:** still `idle`, not stale; freshness applies to `busy` only (P4).
- **Pid reused by another process:** start time differs from `procStart` → `finished` (P5).
- **Plugin hand-off:** the launch is in the subagent transcript; the link's parent is the main session (P1).
- **Step id with a letter suffix (`0.2a`):** supported (P11).

## Solution implementation

### Mockup

The terminal mockup (claude.ai artifact, private: https://claude.ai/artifact/R5Bh9iX6bsn3jSu8F8D58C) has six screens, all with sample data. Screen 6 is the home screen; 2, 3 and 5 are its drill-downs.

| Screen | Phase | Shows |
| --- | --- | --- |
| 6 · `multi-link tui` home | 3 (Release 1) | Every active repo, needs-you first. Per repo: steps in progress with Verify, one mini lane per session (Codex nested under its parent, checkout labelled), a concurrent-activity warning, the latest hand-off. |
| 2 · Repo timeline | 3 (Release 1) | Swimlanes for one repo, Codex rows under their Claude parent; solid = exact link, dashed = heuristic; ambiguous and unlinked runs shown as such. |
| 4 · CLI (plumbing) | 1 (MVP) | Not a daily view. Prints the core's output as a table or JSON for tests and other front ends. |
| 3 · Step graph | 8 (Release 1.5) | One step as Claude plan → Codex implement → Claude review. Board 9 is its approved form; the scrubber is dropped (Phase 8 non-goals). |
| 5 · Live grid | 14 (Release 1.11) | One card per working, waiting or stale session (a Claude card paired with the Codex runs it started) under a feed of hand-offs. Board 14a is its approved form (Phase 14 decisions). |
| 1 · Mod strip | Backlog | A strip above the Claude Code prompt. |

### Phase 7 decisions

Planned 2026-11-02 to polish the approved Phase 6 screens. P1 partially replaces W3's Claude rule;
P2–P4 are presentation rules for the following steps. P5 adds latest-prompt labels.

- **P1 Claude prompt after a system reminder:** after trimming leading whitespace, repeatedly remove an
  exact leading `<system-reminder>` through its first following `</system-reminder>`, then trim leading
  whitespace again. No attributes or nesting are parsed. An unclosed reminder skips the record. Apply W3
  to the remainder: blank or starting with `<` skips this record and tries the next. Other tags, including
  `<system-reminder-extra>`, stay skipped. Normalize and truncate eligible text as before.
- **P2 Home names its window:** all three home window phrases use `period(window)`, like the repo title:
  `2h` or the fold-length form such as `16h`. `HomeOptions` receives the report window; its default stays
  2h. `--since all` uses first burst to now.
- **Step 8.7 clock format (all TUI screens):** use a shared helper for `h:mmam/pm` (`12:00am`,
  `12:00pm`, `9:05pm`), with seconds `h:mm:ssam/pm` (`8:05:43am`); no leading hour zero,
  suffix space or uppercase. Ranges include both suffixes (`9:23pm–10:24pm`). This replaces the
  `HH:MM` / `HH:MM:SS` clock notation in earlier screen descriptions and boards; date-label rules,
  relative times, durations and P4 placement priority stay unchanged. JSON stays RFC 3339 UTC;
  CLI graph text keeps its UTC 24-hour clock. Existing images predate this change.
- **P3 Last 24 h keep the clock:** `view::clock` shows `9:05pm` for times less than 24 h before now, even
  across local midnight; 24 h or older shows `Mon D`. Relative `N min ago` stays unchanged. This replaces
  W5's local-date boundary rule.
- **P4 Axis labels never hide each other:** place fold lengths first, then the end label (full clock with
  `now` when applicable, falling back to `now` or nothing), then an interior `now` in an explicit range
  (clock plus `now`, then `now`, then nothing), then ticks. Each later label must cover no earlier label.
  Suppress ticks whose clock equals the end or `now` label's clock. A range ending before now uses a clock
  end label or nothing. Keeping a fold length may shorten the current clock to `now`.

- **P5 Latest prompt labels:** Session and SessionRow gain additive `last_asked` (string or null).
  `asked` keeps its first-prompt meaning and value. Claude takes the last W3 + P1 eligible own user
  record, also skipping `isMeta: true`, `isCompactSummary: true` and text starting
  `[Request interrupted by user`. Codex takes the last non-imported `user_message` string, or, if none,
  the last X4-eligible response-item user message. Normalize and truncate as W3. Screen labels use
  custom `title` › latest made/placing step › `last_asked` › `asked` › harness `· no prompt`. Changed transcripts are
  re-parsed by the cache on refresh. The selected line and session opening stay unchanged.

No new screens. The four pinned Phase 6 text/style goldens remain unchanged. Other Claude
wrappers and the backlog remain outside this phase.

### Phase 8 decisions (step graph)

Planned 2026-11-06 (PLAN.md → Phase 8, which holds the measurements these rules rest on). Need 4: for one PLAN.md
step I can follow who did what, in what order, how many fix rounds it took and how it ended, across Claude and
Codex, without opening transcripts. The owner approved **board 9** on the mockup canvas (claude.ai artifact
R5Bh9iX6bsn3jSu8F8D58C) on 2026-11-06: board 3's step graph drawn on the 120×40 cell grid from a real step. It is the
acceptance picture; it shows real prompts, so it is described here, never committed as an image (W10). Binding JSON
rules: `docs/output-schema.md` → `graph`. Binding keys: `fixtures/expected/phase8/graph-<id>.json`. Binding
picture: the styled golden `fixtures/expected/tui/step-phase8.txt` / `.styles.txt` (Step 8.1, drawn from the rules
below; Step 8.4 must match it cell for cell). Every Codex field named here (`custom_tool_call`, `exit_code`,
`last_agent_message`, the `exec` input text) was observed on local rollouts on 2026-11-06 and is undocumented: an
assumption, and the readers fail soft on anything else.

| # | Decision | Why |
| --- | --- | --- |
| G0 | New report; nothing existing moves. A new `core::graph` report and CLI `multi-link graph --repo R --step ID --json`. No Session, SessionRow, link or overview key changes; every committed JSON key stays equal. The TUI draws the report and holds no linking or cutting logic (T2) | Answer-keyed like every other view, and the desktop app can reuse it |
| G1 | Which sessions belong to a step: exactly the hand-offs with `link.step == id` in that repo, their parent Claude sessions and their children. Steps are never guessed from time or files. No tagged hand-off means no graph: `no hand-offs tagged step 3.5 (a Claude-only step, or its prompt did not name it)`. An untagged hand-off inside a chain is not drawn and does not split it; its Claude part counts it (`N untagged hand-offs here`) | `link.step` is the only step evidence (0 of 362 session segments have a step) |
| G2 | Chains and Claude parts. Per parent, the step's hand-offs in time order form chains; a hand-off tagged with another step between two of them splits the chain (A → B → A is two chains of A). Chains are ordered by first hand-off, across parents. Inside a chain, **a Codex run's interval belongs to Codex and no Claude part covers it**: Claude parts are the time left. `plan` = chain start → first hand-off. Between parts = a run's end → the next hand-off, kept only when no tagged run of the chain is still going, so a hand-off during a run makes two Codex parts neighbours. Last part = the last run's end → chain end. An unlinked or unavailable run has no interval, so Claude's part runs through it. Chain start = start of the parent's W1 burst holding the first hand-off, not before the end of the previous differently tagged run in that session. Chain end = end of the parent's last burst that overlaps the time after the last hand-off, not after the next differently tagged hand-off; if that burst ended before the last return, there is no last part. Report windows are `--since all`, so bursts are unclipped | Hand-offs are the only events both harnesses record |
| G3 | Names by position. A Claude part before the chain's first hand-off is `plan`, whatever it holds; first Codex `implement`, later Codex `fix round N`; a Claude part with a later hand-off is `review`; a last Claude part is `verify + commit` if it holds a `git commit` call, else `review`. A positional name is a reading, not evidence (a second run could be new work); README says so | No transcript says "this is a fix round" |
| G4 | Codex parts come from turns. A hand-off's window runs to the next hand-off of the same child (else unbounded). `codex_turns(path)` gives each turn's start, end, prompt and `last_agent_message`; a run is the turns that start in its window, so two resumes inside one W1 burst stay two parts, and a later resume tagged to another step stays out. `from`/`to`, `asked`, `said`, status and activity come from those turns (activity = bursts intersected with them), never from the whole session. `✓ done HH:MM` (W5 meaning: last turn complete), `working now` on an open turn. **Link confidence and run availability are separate fields.** Unlinked: dim `no codex run found`, no return arrow, `(the rollout may be gone)` in the panel. Ambiguous: dim `N possible runs`, no id, `o` off. Linked but the child has no session, or its file vanishes before reading: the edge keeps its `│` or `╎`, the part is dim `run unavailable`, `o` off | Session-level times merge resumed runs. Rollouts do disappear, and the graph must say so |
| G5 | Calls. New `calls` readers in the `claude` and `codex` crates, read on demand for the step's own sessions only: never in Session or the scanner's parse. Each call: `at`, `kind` (`read` / `edit` / `run` / `handoff`), `label` or null, `ok` (true / false / null). Claude: a session's owned calls are those in its main file and subagent files, by the same ownership rule as W1 (records with another `sessionId` are inherited and skipped), deduplicated by tool id, results joined by tool id; a subagent's calls count for the parent. `Read`/`Grep`/`Glob` read; `Edit`/`Write`/`NotebookEdit` edit (file basename); `Bash` by its first command word after `export`/`cd`/`VAR=` prefixes: `cat sed grep rg head tail ls awk wc find` and `git diff\|status\|log\|show` read, else run; a recognised launch is `handoff`; other tools run; `mcp__ccd_*` not counted (app bookkeeping). `ok` from `is_error`. Codex: one `custom_tool_call`/`function_call` record = one call, outside imported turns; `tools.apply_patch` edit (basename of its first `*** Update\|Add File:`), first `cmd:"…"` classified as Bash, `write_stdin` run; unreadable input run with null label; `ok` from the first `exit_code` in the output, else null. Unknown shapes fail soft (skipped and counted). **Shell commands that write files count as runs** | The largest real transcript is 15 MB and a report needs 2–3 sessions. Telling writes from shell text would be a guess, so real boxes show fewer `edit` lines than board 9, which counted `python3 … write` and `cat >>` as edits |
| G6 | What a box shows. Line 1: `N reads · N edits · N runs`, zero parts left out, hand-offs not counted. Then at most 3 notable calls, newest last, each a **fixed label chosen by category, with no text copied from the command**: `▸ codex exec · step 6.7` (`· continue 6.7` when the hand-off is not the chain's first), `$ git diff`, `$ git commit`, `$ cargo test\|clippy\|fmt\|build`, `$ pytest`, `$ npm test`, and `edit <basename>` (repeats merged). No other command text reaches the report. `✓`/`✗` mean the call exited 0 / non-zero, not "tests passed" (a pipe to `grep` hides a failure); legend and README say so. Claude calls during a run add one grey line, `N reads while codex ran`. `asked` and `said` are prompt text: real reports stay in scratch (W10) | Commands hold paths and tokens. A fixed list cannot leak them |
| G7 | Header. Repo › step id, PLAN.md title; right `✓ committed <hash> · HH:MM`, from `git log` with the exact `(Step <id>)` in the subject, newest first, nothing when absent (fixture mode runs no git; a core test uses a temp repo). Line 2: date, range, length, `N claude session(s) · N codex runs · N fix rounds`; right `Verify <command> (reported)`, the PLAN.md text cut to fit. Never a pass or fail mark | 55 of 57 real Verify results parse as `unknown`, and reported is not observed |
| G8 | Screen: board 9, rules under Target screen below. Claude lane rows 4–12, edges 13–15, Codex lane rows 16–24, 30-wide boxes, x = 1 + 22 × slot; next slot is +1 when the harness differs, +2 when the same. Up to 5 slots show; more parts scroll with the selection. Colours are Phase 5's theme plus the board's selection; NO_COLOR keeps every glyph and word | The picture the owner approved |
| G9 | `s` on a repo-view lane opens that lane's latest tagged step (a lane with none: footer `no step on this session`). `←→` moves along parts; `[` `]` previous/next step with tagged hand-offs in the repo, in PLAN.md order; `o`/`f` keep FR18's rules for the selected part; `esc` returns to the repo view with its selection; a refresh rebuilds the report and keeps the selected part by session and position | You reach a step from the session you are looking at, and home stays the 7c answer |
| G10 | Narrow and short. Below 120 columns: the same parts as a list, two rows each (`claude · plan`, then sub line and counts), edges as indented lines; `←→` becomes `↑↓`. Below 40 rows: drop the strip, then the legend, then the panel down to its first line. Same order for fixture and real data | The grid needs 120×40; a smaller terminal still answers the need |
| G11 | Speed. The derived report is rebuilt on every refresh, so stale state, new rollouts, hook records, commits and PLAN.md edits show as everywhere else (Incremental reads). Parsed calls and turns may be cached by file key, as the scanner does. The whole refresh with the graph open stays inside the 2 s threshold on the largest real step (8.5 measures) | Clock-driven state changes with no file change |

**Pinned while writing the keys (Step 8.1).** PLAN.md's G rules left these open; the keys depend on them.
- **G1 Owned hand-offs:** a link counts only when its `invocation` is one of its `parent`'s owned calls (G5). `links`
  reports a hand-off copied into a fork a second time, under the fork's id and with the same `invocation`; that copy
  is not the fork's hand-off, and it opens no chain and ends no window.
- **G4/G5 Interfaces:** to make the next rules computable, a call also carries its `id` (Claude tool id, Codex `call_id`),
  and a turn gives `completed_at` (null while open) and `last_record_at` in place of a single end
  (`docs/output-schema.md` → Calls and turns).
- **G2 No interval:** an unlinked, ambiguous or unavailable run has no interval. Its hand-off still ends the Claude
  part before it, and the next Claude part starts at that same hand-off, so Claude's time is unbroken across it and
  the part after it has no return edge. A Claude part exists only when `from < to`; a chain whose first hand-off opens
  its burst, or follows a run of another step that was still going, has no `plan`.
- **G2 Previous and next:** "differently tagged" means tagged with another step; an untagged hand-off never starts,
  ends or splits anything. The previous run is the one launched by the latest such hand-off before the chain's first,
  in the same parent, and its end caps the chain start only when that run has an interval.
- **G2 Open run:** an open run's `to` is its last record. While the chain's last run is open there is no last part.
- **G3/G6 Per chain:** `implement`, `fix round N` and `step`/`continue` restart in each chain.
- **G4 Status:** a run whose last turn has `task_complete` is `done`; otherwise `working`, or `stale` when its last
  record is over 300 s before `now` (the State rule).
- **G5 Hand-off words:** every recognised form, `plugin` included, gets the fixed `▸ codex exec` label.
- **G6 Which part holds a call:** an owned call belongs to the Claude part with `from ≤ at ≤ to`; a hand-off belongs to
  the part it ends (`from < at ≤ to`). Inside the chain, a read in no part is added to `waiting_reads` of the last
  Claude part before it, and a tagged hand-off in no part is a notable call of that same part; other kinds in no part
  appear nowhere. Only the chain's own tagged hand-offs are notable. Codex calls belong to the run whose turn holds
  them.
- **G6 Merge:** a label seen twice in a part keeps one entry, at its latest position, with the latest `ok`; then the
  last 3 are kept.
- **G6 Marks:** the report gives `ok` for every notable call. The screen draws `✗` on any `ok: false`, `✓` only on
  `$ cargo …`, `$ pytest` and `$ npm test` with `ok: true`, and nothing else. A box has 3 rows under its counts: with
  `waiting_reads` the grey line takes the last row and only the 2 newest notable calls show.
- **Stretches:** one per burst that meets the part (Claude) or per burst ∩ turn (Codex), single records included;
  `active_ms` sums their lengths. Screens use W5 minutes: `active about N min in K stretch(es)`.
- **Dim** in G4 means the grey role, never DIM (V4 allows BOLD only).

Non-goals for Phase 8: replay and the scrubber (board 3: needs event-level history, and the strip answers where time
went); inferring steps for untagged hand-offs; fixing `verify` parsing or `unknown` results; explaining or recovering
missing rollouts; per-call detail view; Codex subagents; changes to home or the repo view except the `s` key and
footer; the live grid.

#### Target screen (Step 8.1): layout rules the golden is drawn from
Fixture `fixtures/sessions-phase8`, `--now 2026-11-06T18:00:00Z`, step 1.1 (key `graph-1.1.json`), the `implement` part
selected, fixed offset UTC−8, truecolor palette, 120 × 40 cells. Columns are 0-based, inclusive; `w` = 120. Every cell
paints its background (screen unless stated). Text styles are BOLD or nothing (V4). Times are `HH:MM`, except edges and
the panel range (`HH:MM:SS`). A length is `N s` under 60 s, else `N min`, rounded down, at least 1.

- **Row 0:** repo name (last path component) bold at 1, grey `›`, `step <id>` bold, 2 blanks, PLAN.md title in text
  colour. Right-aligned to `w−2`, green: `✓ committed <hash> · HH:MM`, nothing when `commit` is null.
- **Row 1, grey:** from 1: `<Mon D of the first from> · HH:MM–HH:MM · <length> · N claude session(s) · N codex run(s) · N
  fix round(s)`. The range runs from the earliest `from` to the latest `to`. Sessions = distinct Claude `session`s,
  runs = parts with `run_available` true, fix rounds = parts named `fix round N`; zero counts stay. Right-aligned to
  `w−2`: `Verify  <verify> (reported)`, cut with `…` to keep 2 blanks after the left text; left out when `verify` is
  null. **Row 2:** rule `─` at 1..`w−2` (border colour), as rows 26 and 38.
- **Row 3:** scroll hints, grey: `‹ N earlier` at 1, `N later ›` right-aligned to `w−2`; N = hidden parts.
- **Slots:** the first part is slot 0; each next part is +1 when its harness differs, +2 when the same, chain
  boundaries included. Five slots show, the first visible one moving only as far as the selection needs; a part at
  visible slot v has x = 1 + 22 v.
- **Box** (9 rows × 30 cells; Claude top row 4, Codex top row 16): rounded border in the harness colour. Fill: Claude
  none; Codex `#121A26`; selected part (either harness) `#1B2130`, border included. Inside, from the top: `●` harness
  colour at x+2, harness bold in that colour at x+4, ` · <kind>` in text colour (BOLD on the selected part only);
  sub line at x+2; `┄` at x+1..x+28 (border colour); counts at x+2 (order reads, edits, runs; singular for 1); up to 3
  lines at x+2 in text colour, cut with `…` to 27 cells (`▸` orange, `✓` green and `✗` red one blank after the label).
- **Sub line:** Claude grey `HH:MM · <length>`, or nothing. Codex `✓ done HH:MM` green + grey ` · <length>`; `working now`
  orange + grey ` · <length>`; `stale since HH:MM` red. No interval: grey `no codex run found`, `N possible runs` or
  `run unavailable`, border and text grey, no fill unless selected, no counts.
- **Hand-off edge,** x = Codex box x + 4: rows 13–15 `╎ ╎ ▼` (`│ │ ▼` when `match` is `exact`) in text colour; at x+2, row
  13 `handed step <id>` (`implement`) or `handed fix` in text colour, row 14 the clock grey, row 15 grey `confirmed`,
  `matched by time`, `N possible runs` or `no run found` (`╎` for the last two).
- **Return edge,** on a Claude part with `edge_in.kind` `return`, x = previous Codex box x + 26, all grey: rows 13–15 `▲ │
  │`; at x+2, row 13 `back to claude`, row 14 the clock.
- **Panel, row 27:** `selected` grey at 2; at 12 the harness bold in its colour, then in text colour ` · <kind> · <id[0..8]>
  · HH:MM:SS–HH:MM:SS · active about N min in K stretch(es)`. A part with no run ends after the kind with its sub line
  words. **Row 28** (Codex with a run): `asked` grey at 12, the prompt quoted `“…”` at 19. **Row 29:** `from` grey at 12, at
  19: Codex `claude <id[0..8]> · <parent part kind> · handed at HH:MM:SS · <confirmed|matched by time|N possible runs|no
  run found> · started N s after the call` (the last phrase only with `gap_ms`; `N min` from 60 s); Claude `handed step X
  to codex at T`, `handed fix to codex at T` or `no hand-offs`, for the part's last hand-off. **Row 30:** Codex `said`
  grey at 12, quoted at 19, left out when null; Claude grey at 19 `N untagged hand-off(s) here` when N > 0; unlinked
  grey at 19 `(the rollout may be gone)`. Quoted text is cut on a word boundary with `…` inside the closing quote so it
  ends by column 108. **Row 31** at 12, with a session to open: `o open in Claude|Codex`, 3 blanks, `f fork` (keys in
  text colour, words grey); blank when `o` is off.
- **Strip, row 33:** `where the time went` grey at 2. Columns 23..116, one blank between parts, in part order, every
  part of the report. Part length = `to − from` in ms, 0 with no interval. Label = the kind, `fix round N` → `fix`, at
  the part's first column + 1, in the harness colour. A part's minimum is label + 2 cells; the columns left over are
  shared by length: `floor` each share, then hand the rest to the largest fractional parts (ties to the earlier part).
  Fill: Claude `#3A2A1F`, Codex `#1E2A3D`, selected `#1B2130`. When the minimums do not fit, every part gets 1 cell plus
  its share and no label. Overlapping parts each count in full, so the strip is shares, not a clock. **Row 34:** first
  `from` at 23, last `to` right-aligned to 116, grey.
- **Row 36, legend** from 1, 3 blanks apart, words grey: `▭ claude part` (orange), `▭ codex run` (blue), `╎ matched by
  time`, `│ confirmed link` (text colour), `✓ done, or exit 0` (green), `✗ exit non-zero` (red).
- **Row 39, footer** like the repo view: `←→ move along the step`, `[ ] prev / next step`, `o open`, `f fork`, `esc repo
  view`, `q quit`; right `times UTC-8`.
- **Empty step:** rows 0–2 as above with row 1 holding only `Verify`, `empty` grey at 2 on row 4, rows 38–39 as above.

**Palette additions** (G8; every other role is the Phase 5 and W9 tables). Ratios as in Phase 5 (WCAG 2.x); only the
pairs the screen draws are listed. In 256 colours the two strips share an index and differ by their label colours.

| Role | Truecolor | 256 |
| --- | --- | --- |
| Graph selection background | `#1B2130` | 235 |
| Codex box background | `#121A26` | 234 |
| Claude strip background | `#3A2A1F` | 236 |
| Codex strip background | `#1E2A3D` | 236 |

| Text colour | on selection | on Codex box | on Claude strip | on Codex strip | 256: selection | 256: box | 256: strip |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Text | 11.47 | 12.48 | | | 10.83 | 12.19 | |
| Grey | 6.19 | 6.73 | | | 6.36 | 7.17 | |
| Orange | 7.83 | | 6.68 | | 8.32 | | 7.26 |
| Blue | 7.47 | 8.13 | | 6.72 | 6.92 | 7.80 | 6.04 |
| Green | 8.26 | 8.98 | | | 8.72 | 9.82 | |
| Red | 5.31 | 5.77 | | | 5.08 | 5.72 | |

The border colour on `┄` and the rules is decoration, not text.

**What the target differs from board 9 in, and why** (PLAN.md wins over the board). The data is synthetic, so repo,
step, title, ids, times, prompts and file names differ throughout; the regions below name everything else.
- **Header:** no `✓ committed <hash> · HH:MM`, because fixture mode runs no git (G7); a core test covers it.
- **Claude boxes:** counts read reads, edits, runs (G6; the board's last box says `4 reads · 3 runs · 1 edit`); `1½ min` is
  `1 min`, whole minutes as W5; `$ cargo test, clippy, fmt ✓` is one label per line, and with at most 3 lines `$ cargo
  test ✓` shows (G6); `$ git commit → <hash>` is `$ git commit` (G6: nothing is copied from a call's result).
- **Edges:** the second hand-off is `│` and `confirmed`, where the board shows two `╎`, because the fixture has one link
  of each kind on purpose.
- **Codex boxes:** `6 reads · 3 edits · 6 runs` where the board has 5 edits and 4 runs: the fixture's two `python3 … write`
  commands are runs (G5), and the board counted them as edits. `edit timeline.rs + tests` is `edit parser.rs`, `$ cargo
  test --workspace ✓` is `$ cargo test ✓`, `$ cargo clippy, test ✓` is two lines (G6).
- **Panel:** the quoted text ends on a word boundary where the board cut mid-word at the same column.
- **Strip:** widths come from the rule above (8, 32, 13, 10, 27 against the board's 8, 32, 14, 10, 26).
- **Legend:** `matched by time` and `confirmed link` as W8 (any exact evidence, not only the hook), and `✓`/`✗` say what
  they mean (G6).
- **Footer:** `times UTC-8`, not `times PST`: only the offset is known (as Phase 6).

### User flow: one PLAN.md step

1. I keep `multi-link tui` open in a terminal pane started from `~`. It lists route-impact and posting-triage as busy, physaflow idle.
2. In route-impact I ask Claude to plan step 4. The overview shows step 4 as current and one busy Claude session.
3. Claude hands the step to Codex. The hook records the pair and the step id; the route-impact row shows a Codex lane and the hand-off.
4. Meanwhile posting-triage's Verify fails and its Claude session asks for permission. The terminal bells; that repo jumps to the top with ◆.
5. I press `l` on route-impact to see the live grid, or `t` for its timeline, then go back.
6. After Claude's review passes, PROGRESS.md logs step 4 and the overview shows "Verify ✓ step 4". Later, `s` on that session's lane shows the whole step across both agents for my Review notes.

## Proposed plan / timeline

**Release 1** (the portfolio release) is the core, the `multi-link` CLI, the TUI home screen and the repo timeline. Everything after is an ordered but uncommitted backlog. Each phase ends only when its Verify gives the expected result. Step-level detail lives in `PLAN.md`.

1. **Phase 0: Spike with go / no-go.** On real data, answer: (a) for each invocation form (exec, resume, plugin, background), where is the child Codex id observable? (b) which signals give busy / waiting / idle for each harness? (c) is zoetrope's core a usable library crate? (d) does agtrace already link Claude → Codex? Collect scrubbed fixtures, including negative cases (`codex` mentioned but not launched; a manual Codex run in the same folder), an ambiguous case (two hand-offs seconds apart in one checkout), a resume, a background launch, a stale session and a malformed line. Hand-label the expected outputs.
   - Verify: `docs/phase0.md` answers (a)–(d), each marked go or no-go, and `fixtures/expected/{sessions,links,steps}.json` exist. A no-go on (a) means links are heuristic-only, and this doc is revised before Phase 1.
   - **Done 2026-10-29:** (a) go, (b) no-go for waiting only, (c) no-go, (d) go. See Phase 0 results and decisions.
2. **Phase 1: MVP, core + CLI.** Parsers, repo mapper, state machine, step parser, linker; `multi-link sessions`, `multi-link links`, `multi-link steps`. A Cargo workspace with one crate per parser (Claude, Codex, PLAN/PROGRESS) plus shared model, core and CLI crates: a format change touches one crate (swappable parsers), and parallel steps can't break each other's build.
   - Verify: `cargo test` passes, and each of the three commands run with `--root fixtures/sessions --now <fixed time> --json` equals its expected file (exact commands in `docs/output-schema.md`). Negative fixtures produce no link; the ambiguous variant produces `ambiguous`.
3. **Phase 2: Hook.** `multi-link hook install` (prints the snippet); exact links when the child id is observable, step tags from hand-off commands, and "waiting on me" from `PermissionRequest` / `Notification` (decisions H1–H10). Steps 2.1–2.8 in `PLAN.md`.
   - Verify: for one real hand-off, the child id the hook records equals the id read independently from `~/.codex/sessions`; a hand-off without an observable id stays heuristic.
4. **Phase 3: TUI home + timeline + open (Release 1).** Screens 6 and 2, needs-you sorting, bell, FR18 resume and app link. Needs-you sorting and the bell depend on the Phase 2 hook; without it, those sessions show as unknown.
   - Verify: snapshot tests of both screens on fixtures pass; home refresh on my real `~` is 2 s or less; opening a running fixture session never calls resume.
5. **Phase 4: Home screen cleanup (Release 1.1).** `codex ×N`, collapsed quiet repos, header counts, mini time bars (U1–U5). Built, but the owner judged the real screen still hard to read, so it closed as not meeting the need.
6. **Phase 5: Home redesign to the approved target (Release 1.2).** Board 7 as the acceptance picture: step-named rows, combined lanes, tested colour, mockup-like bars, plain words, local time (V0–V10).
   - Verify: the board-7 fixture's styled render equals the hand-drawn target golden cell for cell; a real-`~` render sits next to board 7 in `docs/phase5-check.md`; the owner confirms.
7. **Phases 6 and 7:** home in words, repo swimlanes (W0–W10), then polish (P1–P5). See their decisions above.
8. **Phase 8: Step graph (Release 1.5).** Board 9 as the acceptance picture (G0–G11).
   - Verify: the fixture render equals the Step 8.1 golden cell for cell; a real step's render is compared with board 9 region by region; the owner confirms.
9. **Phase 9: Truthful states and links (Release 1.6).** `.zst` rollouts, clean exit from the transcript and `SessionEnd`, reopening by entrypoint, exact links from a redirect file, a Codex lane's step all-time, fix rounds from `codex exec resume --last` (R0–R7).
   - Verify: `cargo test`, clippy, every key in `fixtures/expected/phase9/` matches (`check.sh`), real counts before/after, the owner confirms.
10. **Backlog (ordered, uncommitted):** live grid (screen 5), tmux / Herdr jump, desktop app with notifications, mod.

### Open questions

- [x] Can a terminal program open a running desktop-app session? Yes: `open "claude://code/continue?session=local_<id>"` switched the app to a live route-impact session (tested 2026-10-29). Format found in repo-sessions' code; undocumented.
- [x] Phase 0 (a)–(d): child-id observability, state signals, zoetrope crate, agtrace overlap. Answered in `docs/phase0.md`; decisions P1–P12 above.
- [x] May step ids carry a letter suffix (`0.2a`)? Yes, one lowercase letter (P11).
- [ ] Do `PermissionRequest` / `Notification` fire for every permission prompt and question, so they can drive "waiting"? Planned answer: Step 2.1 (h).
- [x] How to find the `local_` app id of a finished desktop-app session. Phase 9 (R3): the app record whose `cliSessionId` is the transcript id (Step 8.4c reader); used only when exactly one record maps it.
- [ ] Does `codex://threads/<id>` open a Codex desktop thread?
- [x] Should hand-off prompts carry a step tag? Yes: the first `Step <id>` in the launch command (H6).
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

### Phase 8 row amendment (U1, U2, U5, U6)
Claude titles use the last non-blank own main-file `custom-title.customTitle` string, with controls
removed and outer whitespace trimmed. Inherited and subagent titles do not name the session; Codex has no title.
Labels prefer title, placing/made step, latest prompt, first prompt, then the harness's no-prompt wording.
Each prompt candidate starting with `Background (from the session that started this one):` is skipped independently.
The repo header appends `· latest hand-off step <id>` when its latest tagged hand-off differs from the logged step.
Place words use checkout identity: a linked worktree shows `worktree <folder>`; the main checkout uses its
memoised git branch, falling back to recorded `gitBranch`. These display values never replace JSON `branch`.

### Phase 8 amendment: sessions started by a session (Step 8.4c, S1-S3)
Extends need 2 from Claude to Codex to Claude to Claude. The desktop app keeps one record per app session at
`<home>/Library/Application Support/Claude/claude-code-sessions/<a>/<b>/local_<id>.json` (macOS). This is
undocumented app behaviour: every field is an assumption confirmed only by the fixture and a counts-only real run.
- **S1 Started-by link.** Session C is started by P when C's record has `cliSessionId` = C's transcript id,
  `spawnedFrom.sessionId` names a record whose `cliSessionId` = P's transcript id, both are listed Claude
  sessions with a transcript (a record-only session, with no transcript, is neither end), P differs from C, and `spawnedFrom.userFork` / `btw` are not true (user acts, not a session's).
  Two records with one `cliSessionId` naming different parents, or two records sharing a `sessionId`, give no
  link for the affected child. "Sharing a `sessionId`" is read narrowly: only links whose parent names that shared
  app id are dropped (it cannot say which record is meant); the two records still each link as children of
  whatever they name. An edge closing a cycle is dropped for every session on it; cycles are found on all
  transcript-backed sessions before the `--since` window is applied, so a cycle with a member outside the window
  keeps every member top level. Always exact; there is no
  time or brief guessing. An absent directory or missing parent record is normal; a malformed file or wrong field
  type is a bad record; an unreadable directory is the existing unreadable diagnostic; never an error exit.
- **S2 Contract.** `Session` and `SessionRow` gain `started_by`. It is not a `Link`: never in `links`,
  `handoffs`, `unparented` or any Codex hand-off count, and step graphs ignore it. Without app records the
  output equals Step 8.4b's plus `started_by: null`.
- **S3 Nesting.** In the repo group (mapped repo, not `cwd`) a started Claude row nests under its starter at any
  depth; cross-repo or unlisted starters stay top level. Children are ordered by time (Codex: placing hand-off;
  started: `started_at`), ties by id. These keys decide the order; the direction follows each surface. The JSON
  overview lists children ascending. The repo view keeps W6's newest-first direction at every level, so a parent's
  children read in the opposite direction there; this is deliberate. A parent's own order and state are unchanged; quiet-merge eligibility looks at
  all descendants. Why: the owner's coordinator sessions start step sessions, and the hierarchy is the one thing
  the flat list hid. Home stays flat (non-goal).

### Phase 9 decisions (truthful states and links, Step 9.0; R7 added for Step 9.6)
Planned 2026-11-08 (PLAN.md → Phase 9 holds the real-data counts these rules rest on). Need 1 and 2: the screens
exist, but much of what they say is "unknown". A session that ended should say so and be reopenable; a hand-off whose
Codex run is still on disk should find it. No new screen. Every transcript and app field below is undocumented and
observed (Claude Code 2.1.319, Codex 0.164.0); only the `SessionEnd` hook event is documented. Binding rules and
fixture tables: `docs/output-schema.md` (Phase 9 changes, State, Exact evidence 4b, Hook records → Ended, Open,
Phase 9 fixture results).

| # | Decision | Why |
| --- | --- | --- |
| R0 | **Contract, additive only.** `state_evidence` gains `transcript_exit`, `session_end`; link `evidence` and hook `child_evidence` gain `redirect_file`; link `evidence` also gains `resume_turn` (R7, the only value allowed on a `heuristic` link; every other value still means `exact`); hook records gain `kind: "ended"` with `reason`; open `reason` gains `app_session`, `unknown_entrypoint`. Session gains no key. Every committed value stays equal except the R2 changes 9.0 lists in PROGRESS.md. Answer keys are Claude's: 9.0 writes every new key and every changed existing key; later steps never edit a key (one that looks wrong stops the step for Claude). Each Verify compares parsed JSON from `cargo run -q -p multi-link --` with `--now 2026-11-09T12:00:00Z`, never the installed binary. | The JSON is the contract (W0); keys written before code cannot be fitted to the code. |
| R1 | **Compressed rollouts.** `rollout-*.jsonl.zst` is decoded (pure-Rust `ruzstd`, no C build step) and parsed exactly like `.jsonl`. One opener in the codex crate serves discovery (codex crate and core scanner), `scan` and `read_details`. Same id in both forms → the plain file. Decoded size capped at 64 MiB for `.zst` only; over the cap or undecodable → `unreadable`, never an error exit. Cached by file key. | 202 of the real rollouts are `.zst` (folders 2026-10-16 to 10-30), each one zstd frame of the same JSONL; most "lost" Codex runs were these. Who compresses them is a non-goal. |
| R2 | **Clean exit from the transcript.** No live session record, and the last `cost-state` of the main transcript (own `sessionId` or none) comes after its last turn → `finished` / `transcript_exit`. A *turn* is an `assistant` record or a non-meta `user` record that is not a local-command echo (`<command-name>`, `<local-command-stdout>`, …); a `tool_result` record is a turn. Position is file order (`cost-state` has no timestamp). A turn after it → today's rule. A live record always wins. `state_since` stays `null`. | Positive evidence (P1): every clean exit mode tested (`/exit`, Ctrl-C, SIGTERM, SIGHUP) wrote it after the last turn and deleted the record; 0 of 10 live transcripts had one; SIGKILL writes none and keeps the record (`process_exit`). If the format changes, sessions fall back to `unknown`, never to a wrong `finished`. The `/exit` echo after it depends on the version, hence the echo rule. |
| R3 | **Reopening by where it ran.** `finished` from `transcript_exit` or `session_end` uses the transcript's last `entrypoint`: `cli` / `sdk-cli` → `claude --resume <id>` in its `cwd` (fork allowed); `claude-desktop` → `claude://code/continue?session=local_<id>` when exactly one app record (8.4c reader) maps the transcript id to a safe `local_` id and no fork, else `unavailable` / `app_session` with the command; missing or other entrypoint → `unavailable` / `unknown_entrypoint`. `process_exit` keeps its rule (resume). The entrypoint and mapping reach `open` through an internal core resolver. | Resuming a desktop session in a terminal makes two writers if the app reopens it; the app link cannot fork. Unknown launchers are refused rather than guessed. |
| R4 | **Hand-off child from the redirect file.** When a recognised launch segment redirects stdout (`>`, `>>`, `1>`, `&>`, `&>>`, `>|`, with or without `2>&1`) to a literal path, that file is read like the task output file (64 KiB; the hook polls it 25 s; the core reads it while it exists). Relative paths resolve against `cwd` only with no earlier `cd` / subshell; exactly one stdout redirect on the segment. Safe read: non-blocking open, `fstat` must say regular file (a FIFO never blocks a refresh), the opened file's canonical path must be under `cwd` or a temp dir, else no read; `$VAR` / quoted expansions are never read. Exact (`redirect_file`) only when the file holds exactly one `session id:` line. Order: stdout → thread_ready → task_output → redirect_file → hook. Fixture mode reads root-relative. **Amends decision 2 (Exact):** this file is an explicitly observed Codex output, so it is a fifth exact source. | 32 of 37 hook-era hand-offs redirect Codex's stdout to a file, so the `session id:` line never reaches the hook. Paths come from a command, hence the containment and file-type checks. With `>>` an older run's id can share the file, hence "exactly one". Full shell semantics are not modelled; anything unclear just gives no exact link. |
| R5 | **`SessionEnd` hook record.** The hook writes `{"kind":"ended","event":"SessionEnd","reason":…}`. Precedence for a Claude session: live record → `process_exit` → `session_end` → `transcript_exit` → hook `waiting` → `no_signal`. An `ended` record counts only when no turn has a later `timestamp` and no later hook record exists (a resume clears it, even if its async record lands late). `reason: "clear"` ends the old id only. The snippet change needs the owner to re-merge it (stop-and-ask at 9.7). | A documented event, so it outlives transcript format changes; it fired for every clean mode tested. It cannot cover history, which R2 does. Hook `waiting` comes after both exit rows because an exit while a prompt was open leaves the `waiting` record uncleared. |
| R6 | **A Codex lane's step looks all-time.** A Codex lane whose Claude parent is outside the TUI window takes its step from the worker's all-time hand-offs (per Codex id). `own_step`'s rules stay: the parent must belong to the lane's repo group (by the parent session's repo), and the latest tagged hand-off wins. | The 8.5 limit: a long background run outlives its parent's window and lost its step. |

| R7 | **A `codex exec resume --last` fix round is a turn of an existing run.** `codex exec resume --last` is classified as a resume, apart from `resume <id>` (which keeps `resume_arg`). Its match searches existing runs in the same checkout for a non-imported turn, not that run's first, starting 0–10 s after the hand-off; it ignores exact-launch claims and never falls back to the 60 s creation match (`dir_moved` suppression stays). One child → `heuristic`, evidence `resume_turn`, `gap_ms` = turn start − hand-off; none → unlinked; two or more → `ambiguous`. An observed id (stdout, R4 redirect file) still wins and makes it exact. **Amends R0 and decision 2:** `resume_turn` is the one `evidence` value allowed on a non-exact link. | Fix rounds write no new rollout, so the 60 s creation match finds nothing and they showed `no codex run found` (owner's step 8.7 screenshot; real rollout: three turns, each 0–2 s after its hand-off). `heuristic` because decision 2 keeps `exact` for an observed id; 10 s is conservative and uncalibrated (9.7 counts more). The graph already splits a Codex part at each hand-off (G4). |

R7 was added to Phase 9 after Step 9.0 was planned; Step 9.6 writes its fixtures, keys and any existing-key changes.

Non-goals: the lost key on resize (upstream crossterm-rs/crossterm#1154); reading the task path from the transcript
(T10 as first written, ~0 of 37 gain); pipes to `tail` / `head`; the live
grid and pane jump; who compresses rollouts; Windows / Linux paths; `state_since` for the new `finished` rows.

**Existing keys under R2.** 12 `fixtures/sessions` transcripts (all `claude-desktop`, none with a record) end with a
`cost-state` after their last turn, so `sessions.json`, `overview.json` and the three hook keys change for them, and
with them the hook home goldens (11c1f13c stops being `waiting` / `hook`). The exact values and the tests that fail
until 9.2 are listed in PROGRESS.md (Step 9.0). The Phase 9 root carries the hook-waiting case instead (92000012).

### Phase 10 decisions (hook setup in one command, Step 10.1)
Planned 2026-11-09 (PLAN.md → Phase 10). Need: setting up multi-link should not mean hand-merging JSON into
`~/.claude/settings.json`, and the user should be able to tell whether their hooks are current. The owner's real
settings showed why: a hand merge had left four hooks on `target/release/multi-link` and `SessionEnd` on
`~/.cargo/bin/multi-link`, so cleaning `target/` would silently break four hooks. These H0–H7 are Phase 10's and are
separate from Phase 2's H1–H11. **Amends the "Hook install" row of Major decisions:** multi-link still never writes
settings, but it now checks them and prints the command that does.

| # | Decision | Why |
| --- | --- | --- |
| H0 | **Read-only stays absolute.** No `--write`. `hook check` reads and reports; `hook check --merged` prints the whole merged settings to stdout. The printed command writes with POSIX tools only (`mktemp`, `cp`, `chmod`, `mv`) plus multi-link. | One exception would make the read-only rule harder to trust. `jq` and `python3` are not guaranteed on a fresh Mac, and the user should not need a second tool. |
| H1 | **One snippet definition** in the hook crate (`multi_link_hook::settings::snippet`, a function of the binary path), used by `install`, `uninstall` and `check`. | They cannot drift apart. |
| H2 | **A multi-link entry** is a hook object with `"type": "command"` whose `command` is a binary named `multi-link` (single-quoted or bare) followed by exactly ` hook record`. Anything else is the user's and is never changed or reported. | A narrow match means a user's own command that mentions multi-link (`echo multi-link hook record`) is left alone. |
| H3 | **Findings per snippet event:** `ok` (exactly one entry, generated shape, equal to this binary's snippet entry), `missing`, `outdated` (one entry, generated shape, differing in binary path, `async` or `timeout`; both paths shown), `duplicate` (two or more, all generated shape), `customized` (any other key such as `if`, or another matcher). Entries under events the snippet lacks → `unknown event`. `disableAllHooks: true` is reported. For an event with no snippet matcher, absent, `""` and `"*"` are equal. | "This binary" is the reference because the hook should run the binary the user just checked with; showing both paths makes the choice visible. |
| H4 | **Merge** fixes `missing`, `outdated` and `duplicate` only: remove those entries, drop a group left empty, append the snippet group at the end of the event's list. `customized` and `unknown event` stay. Other keys, events, the user's entries and key order are kept (serde_json `preserve_order`). Output is 2-space pretty JSON with a trailing newline, so whitespace may change; values may not, so a duplicate object key or a number serde_json would not write back as the same text is refused (exit 2) before any merge. | A user's `if` or narrower matcher is intent, not staleness (critique 3). Refusing beats silently changing `1e5` to `100000.0` or dropping a duplicate key. |
| H5 | **The file and its errors.** `--settings <path>` overrides `--scope`; `--scope user` reads `$CLAUDE_CONFIG_DIR/settings.json` or `~/.claude/settings.json`; `--scope local` the repo's `.claude/settings.local.json`. The path is made absolute and a symlink followed to its target R (the link stays). Absent file → all `missing`, and the command creates it. Exit 2, reason, no command for a dangling symlink, non-regular file, unreadable file, a path holding a newline, invalid or empty JSON, a non-object top level, wrong-typed `hooks` / event / group, or H4's refusals. `--merged` runs the same checks. | A bad file never reaches the write. |
| H6 | **The printed command:** `tmp=$(mktemp 'R.multi-link-XXXXXX') && cp -p 'R' "$(mktemp 'R.multi-link-backup-TS-XXXXXX')" && 'E' hook check --settings 'R' --merged > "$tmp" && chmod M "$tmp" && mv "$tmp" 'R' \|\| { rm -f "$tmp"; false; }` (absent file: `mkdir -p`, no backup, `[ ! -e 'R' ]` before `mv`). Paths absolute and POSIX single-quoted; `E` this binary, `M` R's mode, `TS` UTC time when `check` ran. | `mktemp` gives unique private files, so runs cannot collide; `mv` in one folder is atomic, so the live file is old or new, never half-written (critique 1); any failure removes the temp file and returns non-zero. `--merged` re-reads R when the command runs, so earlier edits are kept. Known limit: an edit in the milliseconds between that read and `mv` is lost (no lock exists that Claude Code also honours). |
| H7 | **Output and exit.** `check`: the file name, one line per event, then (if a fix applies) one line of explanation and the command on its own line; exit 0 all `ok`, 1 otherwise, 2 on error. "Current" means current in that one file, not that other settings levels or a managed policy let the hooks run. `--merged`: only the JSON on stdout, exit 0 (also on a no-op) or 2, diagnostics on stderr (critique 6). `install` adds one stderr line pointing to `check`. | The command line can be copied whole; scripts can rely on the exit code. |

Non-goals: `--write` or any write by multi-link; `--json` for `check`; removing hooks (`uninstall` still prints what to
remove); keeping whitespace; merging customized entries; reading other settings levels or managed policy;
`--scope project`; locking; Windows; checking that the binary path exists.

### Phase 11 decisions (where is now?, Steps 11.1–11.2)
Planned 2026-11-10 (PLAN.md → Phase 11), from board 11 (approved by the owner 2026-11-10). Need: make it clear where
the agents and the current task are, in the repo swimlanes and the step graph. Comparables: calendar/Gantt "today"
lines, CI graphs that mark the running job on its node, htop/k9s (one colour per state).

| # | Decision | Why |
|---|---|---|
| N0 | One colour, `Theme::now` (`#E58AE5`, indexed 176), means now and nothing else, in both views. | A colour reused for two meanings stops meaning either. NO_COLOR keeps the glyphs. |
| N1 | Repo view: `▼` on the axis rule and `┃` from the grid row under it to the last lane row, at a presentation-only now column (axis end → last cell; inside a fold → the fold's last cell). | Board 11b; bar and link time mapping stay untouched. |
| N2 | Paint order grid → links → now line → lanes: grid, link and fold glyphs give way; bars, `✓`, `◆` and text draw over it. | A bar that reaches now plugs into the line; text is never cut. |
| N3 | The now label is placed first, shifted left to stay inside the axis, and replaces the end clock when now is the end. | Board 11e: a trailing fold label used to hide `now`. |
| N4 | Now off the axis (panned, past window, below 80 columns) → today's view. Home unchanged. | The line only claims what is on screen. |
| L1 | Live parts (core, presentation only, no JSON change): Codex `Working` and its child's newest window; Claude = its session's newest part, session `Busy`, last activity within 5 min, and no live Codex part later in its chain. | Evidence-bound: an old step whose session is busy elsewhere is not live; a superseded run window is not live. |
| L2 | A live part's box: heavy border in its harness colour, bold, `● now` badge in the top border. | Board 11d. |
| L3 | With a live part, others' borders and `●` use the dim harness colour; harness words and text keep theirs; grey (missing-run) boxes stay grey. | Board 11d dimmed the harness word too, but on the selected background that is 3.06:1 / 3.61:1, under the 4.5:1 text rule. |
| L4 | Live part off screen: `● now · N later ›` / `‹ N earlier · ● now`, N in parts, from the renderer's own slots. | Board 11g. |
| L5 | Below 120 columns: `● now` after the live item's title (width reserved), `● now ↑/↓` when scrolled away. | Board 11i. |
| L6 | Live never comes from a guess: only the existing Codex status and Claude session state rules. | Decision 2 (no guess presented as fact). |
| L7 | The CLI worker computes L1 with the graph and sends `Refresh::graph_live`; the App keeps it only with an accepted graph. | The TUI crate has no core dependency and no sessions. |

### Phase 12 decisions (settle ambiguous links with evidence, Step 12.0)
Planned 2026-11-10 (PLAN.md → Phase 12). Need (design need 1): see which Codex run came from which Claude session, and
how sure that is. 9 real hand-offs were `ambiguous` though the data made the answer plain in 8. Ground truth: 29 exact
launch links (child observed); the true child started 0.45–3.18 s after the hand-off in 27, 6.2 s in one, and was the
nearest run in 4 of the 4 with rivals. Hiding each observed id and running K1–K3: 28 right, 0 wrong, 1 unlinked.

| # | Decision | Why |
|---|---|---|
| K0 | Link `evidence` gains `"nearest_run"`, only on `heuristic`. `candidates` becomes the runs left after K1 (survivors); a stated contract revision. Never `exact`. | `exact` means an observed id (decision 2); a choice by time stays `matched by time`. |
| K1 | Competitors = launch hand-offs using the 60 s creation window (not exact, not `dir_moved`, no resume id, not `resume --last`). A run leaves hand-off h when another competitor would itself choose it (K2/K3 on its own original list) at a strictly smaller gap. One pass over original lists. | Codex creates a run within seconds of its launch. Using what the other hand-off would choose (not just its nearest) means a run is never taken away for a hand-off that then picks nothing. One pass: order cannot matter. |
| K2 | With two or more runs left, choose the nearest when it is ≤ 5 s and every other is ≥ 10 s (inclusive, ms). Launch hand-offs only; id-less `resume --last` keeps R7. | 27 of 28 true gaps ≤ 3.2 s; rivals ≥ 17 s. Margins wider than the data, provisional; a borderline case stays ambiguous. |
| K3 | One run chosen → `heuristic`, `evidence: "nearest_run"` when the original list had more than one run; none left → `unlinked`; else `ambiguous`. Overview and the graph treat the child like any heuristic child. Graph Edge gains `nearest_run: true` (omitted otherwise). TUI words: `matched by time · nearest run`. | The edge's candidate count is the survivor count, so the graph needs its own flag; omitted when false keeps every existing key. |
| K4 | Real-data changes are listed (PLAN.md K4, phase12-check), never hidden. Committed fixture change: `links-amb-hidden.json` amb-1 → `heuristic` d501 / `nearest_run`. | amb-1's true child is d501 (exact in the unmasked fixture): the masked case gets the right answer. |

**Assumption and its failure case.** K1 trusts creation order: if A's run starts late (background, slow start) just after
B launches and B starts none, A's run is given to B. Today both would claim it; K1 removes the duplicate, and the link
still says `matched by time`. K2's known failure: a manual run at +1 s and a slow true child at +12 s picks the manual run.

### Phase 13 decisions (known folders and a key that is never lost, Step 13.0)
Planned 2026-11-10 (PLAN.md → Phase 13). Needs: design need 1 (which run came from which session) and a TUI that
answers every key. Measured on real data (counts only): 35 `cd`-first hand-offs were `unlinked`; the 31 with a literal
target all resolve under C1, and each of the 29 `exec` ones has exactly one run in the resolved folder within 60 s.

| # | Decision | Why |
|---|---|---|
| C1 | A bare `cd` with a literal absolute / `./` / `../` target, joined by `&&` / `;` / newline, with no folder-changing builtin or closing `)` before the launch, names the launch folder (lexical). Anything else stays unknown. | External commands and `$( )` run in child processes and cannot change the shell's folder; only builtins can. Plain names stay out (`CDPATH`); `$`, `~`, globs, braces are not literal. |
| C2 | Link `launch_dir` (only when it differs from `checkout`); creation candidates, K1 grouping and R7 use it. | `checkout` stays where Claude was; the run lives where the `cd` went. Omitting the key keeps every existing answer. |
| C3 | After `resume`, known options are skipped; `--last` keeps R7; an unknown option, `--` or redirect gives no id; else a uuid-shaped first positional is the exact resume id. | An unknown option might take the uuid as its value; an exact claim needs certainty. |
| D1 | crossterm `use-dev-tty` (poll(2), level-triggered, tty read before the resize pipe). | 0.28.1's mio source drops edge-triggered tty readiness when SIGWINCH arrives in the same wake; `tools/resize-key-check.py` fails 5/5 without it, passes 5/5 with it. |

**Assumption.** After `;` the `cd` is assumed to have succeeded. If it failed, the run starts in the old folder, the
resolved folder usually has no run, and the hand-off stays `unlinked`; a wrong link needs an unrelated run starting
in the target folder within 60 s.

### Phase 14 decisions (live grid: what is each agent doing now?, Step 14.0)
Planned 2026-11-10 (PLAN.md → Phase 14). Need (owner): see what each agent is doing right now (its last few actions,
its state, how long it has been at it) without switching terminal tabs; design needs 1, 2 and 5, FR12, screen 5. The
owner approved **board 14** on the mockup canvas (claude.ai artifact R5Bh9iX6bsn3jSu8F8D58C) on 2026-11-10: 14a is real
data (described here, never committed as an image, W10), 14b made-up waiting and stale cards. Where the board and these
rules differ, the rules win (Codex critique, PLAN.md Review log): card line 1 is `last_asked` for both harnesses (14a's
`fix round 1` needs chain positions the grid does not compute), the feed has no `codex back T` and no chain words, the
header counts follow L5, `✓` follows G6 (14a's `$ git diff ✓` has no mark), counts keep G6's order `reads · edits ·
runs` (14a writes `39 runs · 3 edits`), the idle line always names its repos (14b's has none), and 14b's `no parent
found` first line is `last_asked` like every card. On this screen L3/L4 replace W5's hand-off words: `no run found`
(W5: `no codex run found`) and `codex running` only for a busy child (W5 also counts stale). Binding JSON rules:
`docs/output-schema.md` → `live`. Binding key: `fixtures/expected/phase14/live.json` (`check.sh`). Binding pictures:
`fixtures/expected/tui/live-phase14{,-100,-short,-tiny,-children,-stale}.txt` / `.styles.txt` (Step 14.0, drawn from the rules below and
checked region by region against 14a; Step 14.2 must match them cell for cell).

| # | Decision | Why |
| --- | --- | --- |
| L0 | A report, then a screen. `multi_link_core::live` builds a `LiveReport` (model crate) from the sessions, links and overview pass the TUI already runs, plus G5 calls for card sessions only. `multi-link live [--root] [--now] [--since] [--json]` prints it; plain text is a list (one line per card, then the feed). No existing JSON key changes. The TUI only draws the report (T2) | Answer-keyed like every other view; the desktop app (FR15) can reuse it |
| L1 | Which cards: every non-imported session, top-level or child, whose `state` is `waiting`, `busy` or `stale`, from the same windowed sessions pass the TUI runs (a stale Codex run last active before the window is not listed, so it is history, not a card). Every other non-imported session with activity in the window is counted on one line: `N idle sessions (repo · repo) · home shows them` (distinct repo names, sorted, cut with `…` to the width). No cards and nothing idle: `nothing working right now` plus, if any, `· last hand-off T` | Cards only where something is happening (owner choice); home already answers the rest |
| L2 | Groups, rows and moving. A Codex card's parent is the Claude session of its latest incoming non-ambiguous link by `at` (ties: the later one in the links report); no `started_by` fallback; the parent may be in another repo. A group is a Claude card plus the live Codex cards whose parent it is (by link `at`, ties by full id), or a lone card. Groups by head: waiting (oldest wait start), busy (oldest last-burst start), stale (oldest stale start), ties by full id; wait/stale start = `state_since`, else `last_activity`. Flattened order = groups, head then children. Wide: head at x 1 (56 wide), first child at x 61 (58 wide) joined by `──▸` on the card's 5th row; each further child takes the next row, right side, joined by `└──▸` from x 57; a lone Codex card sits on the left. The selection is kept by full id across refreshes, else the same flattened index (clamped). `↑↓` previous / next group head, `←→` along the group; narrow: both along the flattened order | Pairs show who launched what; a stable selection avoids htop's jump on re-sort |
| L3 | Card (10 rows), border in the harness colour, title in the top border: state glyph (`●` busy, `◆` waiting, `○` stale), harness word bold, `· <repo>`, `· step <id>` (Claude: its latest tagged hand-off; Codex: the `codex_steps` rule, moved to core in 14.1). Rows: (1) `“last_asked”`; (2) W5 phrase `working now · for N min` (last burst's start; `for over <period>` when that burst starts at the window start; `working now` with no burst), `waiting on you · since T (N min)`, `stale · no activity since T (N min)`; (3) G6 counts for the whole session; (4) `┄` rule; (5–7) the last 3 notable calls, newest last, `T  label` with G6 marks, every recognised hand-off as `▸ codex exec` (+ ` · step X`); (8) link line: Claude with a live child → `waiting on codex <id8> · N min`; else its latest hand-off in the window → `codex <id8> done T` / `N possible runs` / `no run found` / `codex <id8>`; else blank. Codex → `└ from claude <id8> · confirmed link\|matched by time` or `not linked to any claude session`. Bottom border `<place> · <id8>`. Text cut at the inner width with `…` | One glance answers the need; fixed labels cannot leak commands (G6) |
| L4 | Feed: `HAND-OFFS newest first · all repos`, then up to 4 hand-offs whose `at` is in the window, ambiguous and unlinked included. Row: `T  <repo, 14 wide>  claude handed step X to codex` (`work` when untagged; `claude resumed codex for …` for a resume form), W8 words at x 75, outcome at x 93: `codex running` when the child is busy and this is its latest incoming link; `codex done T` when a turn starting at or after `at` completed (the first such turn); else blank. None: `no hand-offs in the last <period>` | agtrace's feed: what was handed off, how sure, and how it ended |
| L5 | Header: `multi-link › live  what every agent is doing now`, right `◆ N waiting · ● N working now · ○ N stale   T`. Bottom: `selected` line, `asked` line, legend, keys and `times UTC-8`, as board 14a | The counts that matter first; the rest as on every screen |
| L6 | Keys: `l` opens the grid from home and the repo view (footers gain `l live`, Step 14.3); `multi-link tui --live` starts on it; `esc` returns to the opener (`--live`: home); `enter` repo view, `s` step graph (G9 rules), `o`/`f` FR18, `q` quits. Refresh every pass, in place | k9s: drill in, back out, refresh in place |
| L7 | Sizes. Width ≥ 120: board 14a. Below 120: one column, a Codex child under its parent indented 4, joined by `└▸`. Narrower still: the same, text cut with `…`. Rows: the feed drops to 2 rows below 40 rows, none below 32; cards that do not fit leave `↓ N more` and the view scrolls to keep the selection visible; when not even one card fits, the selected card is drawn clipped and `↓ N more` takes the last row. All cutting by display width, never bytes | A small terminal still answers the need |
| L8 | Colour: existing theme roles only (harness colours, `needs_you`, dim harness colours for stale, `busy_card` / `codex_box` / `waiting_card`, heavy border plus `selected` for the selection, `done` / `warning` for `✓` / `✗`). NO_COLOR keeps every glyph and word | No new colour to test |
| L9 | Speed: the live report is built only while the grid is open (`Refresh::live`); calls are read only for card sessions and cached per session by its dependency set (path, length, mtime, inode of the main transcript and every subagent file); clock-only changes re-read nothing. The refresh stays within the 2 s NFR (measured in 14.4) | The largest transcripts are megabytes; a card needs one session's calls |
| L10 | Privacy: fixtures synthetic (W10); real renders stay in scratch; README images from fixtures only | The JSON and screen carry prompt text |

**Pinned while writing the keys (Step 14.0).** L0–L10 left these open; the keys and goldens depend on them.
- **Report fields** (`docs/output-schema.md` → `live`): `window_from` is `now − since` (`null` for `--since all`); a busy
  card's `since` is its last activity burst's `from` (already clipped to the window), so "clipped" is `since ==
  window_from`; a busy card with no burst has `since: null` and sorts by `last_activity`. `notable` items carry `at` (the
  label's latest call). `last_handoff` is the greatest `at` of the listed links. Card `link` is one of `waiting_on`,
  `handoff`, `from`, `none` (Codex with no parent) or `null` (Claude with nothing to say).
- **Which live child a Claude card waits on:** the last child in group order (its latest hand-off); `N min` since that
  child's parent link `at`. **Outcome order:** `running` is checked before `done`, so the child's latest link with a busy
  child is `running` even when an earlier turn after it completed. A Claude card's `handoff` line never says running
  (a busy child of its own would be a card and give `waiting_on`); a `running` outcome there prints `codex <id8>`.
- **Feed:** ties on `at` newest first by reverse links-report order; `repo` is the parent session's repo (the link's
  `checkout` if the parent is not listed); an `unlinked` hand-off's W8 column says `no run found` (grey). Colours:
  `confirmed link` text, `matched by time` and `no run found` grey, `N possible runs` `needs_you`, `codex running`
  codex colour, `codex done T` `done`; times grey; `claude handed` / `to codex` / `resumed codex for` grey, the step
  (or `work`) text colour; repo text colour cut to 14 with `…`. Time at x 1, repo at x 9, sentence at x 23 (a 7-cell
  time leaves one space). **Below 120 columns** the two right columns pack: W8 words at 23 + the widest shown sentence
  + 3, outcome at that + the widest shown W8 word + 3; the row is cut at the right margin.
- **Header:** left `multi-link` and `live` text BOLD, ` › ` and the subtitle grey; right `◆ N waiting` `needs_you`,
  `● N working now` claude colour (as home), each grey when 0; `○ N stale` always grey (the stale glyph is grey on cards,
  as 14b); separators ` · ` grey, three spaces, then the clock in text colour ending at x w−2. When the two sides would
  come closer than 2 cells, drop in order: the subtitle, the clock, then the count words (`◆ 1 · ● 11 · ○ 1`).
- **Rows (top to bottom):** header 0, rule 1 (`─` x 1..w−2, border colour); with a feed of k rows (4 at ≥ 40 rows, 2 at
  32–39): `HAND-OFFS` row 2 (`HAND-OFFS` grey BOLD, ` newest first · all repos` grey), feed rows 3..2+k, one blank, cards from 4+k; with no feed: one blank, cards from 3. The
  feed always reserves its k rows. Bottom block, fixed: idle line h−10, rule h−9, `selected` h−8, `asked` h−7, blank,
  legend h−5, blank, rule h−3, keys h−2, blank h−1. Card rows start at the first card row and repeat every 11 rows
  (10 + one blank); a card row is drawn only if it ends by h−12. `↓ N more` (grey, x 1) goes on the row right under
  the last drawn card row; N counts the cards after the last one drawn (cards above the view are not counted; there is
  no `↑` line). Clipped case: the selected card from the first card row through h−12, `↓ N more` on h−11.
- **Card cells:** border, glyph and title as board 14a/14b: selected `┏━ ┓ ┃ ┗━ ┛ ━` BOLD with the `selected`
  background, else `╭─ ╮ │ ╰─ ╯ ─` not bold. Border colour: busy the harness colour, waiting `needs_you`, stale the
  dim harness colour. Background: selected `selected`; else busy Claude `busy_card`, busy Codex `codex_box`, waiting
  `waiting_card`, stale none (screen). Glyph colour: busy harness, waiting `needs_you`, stale grey; harness word BOLD in
  the harness colour (dim when stale); ` · repo · step X ` text BOLD. Content at x+2, `w − 4` cells wide; `┄` border
  colour. Row 1 text colour (`no prompt` grey when null); the `“…”` line is cut as one string, so a cut line ends `…`
  and loses its closing quote. Row 2: `working now` harness colour, `waiting on you` `needs_you` BOLD, `stale` grey
  BOLD, the rest grey. Row 3 text colour, `1 read` / `1 edit` / `1 run` singular, empty when all zero. Rows 5–7: time
  grey, two spaces, label text colour (`▸` in the harness colour), ` ✗` `warning` on `ok: false`, ` ✓` `done` only on
  `$ cargo …`, `$ pytest`, `$ npm test` with `ok: true`; rows fill from the top. Row 8: `waiting on codex …` codex
  colour; `codex <id8> done T`, `codex <id8>`, `no run found` grey; `N possible runs` `needs_you`; `└ from claude <id8>
  · ` grey then the W8 words text colour; `not linked to any claude session` grey. Bottom: ` <place> · <id8> ` grey.
  `──▸` / `└──▸` / `└▸` codex colour on the screen background.
- **Narrow layout:** a head or lone card at x 1, `w − 2` wide; a child at x 5, `w − 6` wide, `└▸` at x 2–3 on its 5th
  row. The view scrolls by whole cards (wide: by whole card rows).
- **Bottom block:** `selected` grey at x 2; harness word BOLD in its colour at x 12; then ` · <id8> · <repo>[, <place>]
  · started T · active about N min in K stretch(es)` text colour (the place only for a worktree, as 14b; the active
  part omitted with no activity; N = W5 minutes of the summed bursts). `asked` grey at x 12, `“last_asked”` at x 19.
  Both cut with `…` at x w−2. Legend and keys: items separated by three spaces, whole items dropped from the end until
  they fit (keys: the key text colour, its word grey); `times UTC-8` (grey, ending at x w−2) only when every key fits
  and at least 3 blank cells stay between the last key and the label's first cell. Legend colours:
  `●` claude, `◆` `needs_you`, `│` `╎` text, `✓` `done`, `✗` `warning`, words and `○ stale` grey. Idle line grey; when it
  is too wide, repo names drop from the end behind one `…`. Truecolor palette, offset UTC−8.
- **`<period>`:** `now − window_from` as whole hours `Nh`, else whole minutes `Nm` (`2h`, `90m`). With `--since all`
  (`window_from` null) no burst is ever clipped, so `for over` never shows, and an empty feed says `no hand-offs`.
- **Golden inputs** (`--now 2026-11-10T22:20:00Z`, window 2h): `live-phase14` 120×40, selection 0 (the waiting card),
  first row 0; `live-phase14-100` 100×40, selection 2 (Codex `1421`), first card 1 (shows the narrow child and a
  selected Codex card); `live-phase14-short` 120×30 and `live-phase14-tiny` 60×20, selection 0, first 0; `live-phase14-children` 120×51, selection 6 (`1425`), first row 3 (the `1415` row:
  three children, two `└──▸` rows, `for over 2h`, a cross-repo child with no step); `live-phase14-stale` 120×40,
  selection 11 (`1429`), first row 7 (a `└──▸` row whose head is above the view, then the unselected stale lone Codex
  card `1424` with `not linked to any claude session`).
- **Not in any golden** (14.2 unit tests instead): an unselected waiting card; the Claude card link lines `N possible
  runs`, `no run found` and `codex <id8>`; an untagged `▸ codex exec`; and the wide-character prompt (Codex `1428`,
  never in view): ratatui resets the cell after a wide character (symbol, colours and
  modifiers), an implementation detail a hand-drawn styles file should not freeze.

**Fixture** `fixtures/sessions-phase14/` (synthetic, `--now 2026-11-10T22:20:00Z`, ids `<tag>0000-0000-4000-8000-
00000000<tag>`, repos `/home/dev/repo-m … repo-s`, no plan folders, no hook folder): 21 listed sessions, 13 cards in
6 groups (waiting `1413`; busy `1411`+`1421`, `1412`+`1422`, `1415`+`1425`/`1426`/`1427`, `1416`+`1428`/`1429`; stale
`1424`), 7 idle, 13 links. Cases: confirmed and matched-by-time live pairs; a waiting Claude whose child finished; a stale
manual run with no parent; two- and three-child Claude cards; a child with two incoming links from different parents
(`1421`: `1417` at 21:40, `1411` resumes at 22:10, latest wins) and one with two at the same `at` (`1429`, `1418` and
`1416` at 21:32, the later invocation wins); a cross-repo child (`1427` in repo-q, no step); a burst crossing the window
start (`1425`, `for over 2h`); a resumed child whose earlier hand-off says `codex done`, not running; an exact hand-off
with no rollout (`142c`); an ambiguous untagged hand-off (2 runs); labels repeated with a changed `ok` (`1421`, `1425`,
`1429`), a repeated hand-off label (`1416`), `✗`, label-less runs; a wide-character prompt (`1428`); a busy Codex Desktop
import (`142f`, hidden); a stale run before the window (`142b`, not listed).

### Phase 15 decisions (fast first load and a loading screen, Step 15.0)
Planned 2026-11-11 (PLAN.md → Phase 15). Need (owner): "when I open multi-link it should be ready fast, and while it
loads it should be fun to look at." The cold first pass misses the Speed NFR (baseline below: 8.0 s wall, 7.25 s user time on
14 cores, 87 % of it in `Scanner::read_claude` → `multi_link_claude::parse_file`, single-threaded), and the screen
during the wait is one footer line. The owner approved **board 15, concept E · School** on 2026-11-11 (claude.ai
artifact 7i51gqe1AbVGFurwnkfjqM; source `docs/boards/board15-loading.html`, function `drawC`; the file has no
`<meta charset>`, so open it from a server that sends UTF-8 or the glyphs show as mojibake): two schools of fish in
Claude orange and Codex blue only. Where the board and these rules differ, the rules win (Codex critique, PLAN.md Review
log): below 60 × 16 there is no art (the board's notes say it shrinks), the counter is real progress (the board
simulates it), colours are existing theme roles (the board's shades are continuous). Binding pictures:
`fixtures/expected/tui/loading-*` (Step 15.0, drawn from the rules below; Step 15.2 must match them cell for cell).

| # | Decision | Why |
| --- | --- | --- |
| F1 | Parse in parallel, same results. At the start of `Scanner::read`, list every Claude transcript and subagent file and every Codex rollout with its stamp, take the cache misses and parse them on `std::thread::available_parallelism()` scoped threads (`std::thread::scope`, no new crate); each result is stored with the stamp taken before its parse. The serial loops then run as today through `Cache::read`, which re-parses when a stamp no longer matches. A failed parallel parse is not cached and its diagnostics are dropped; the serial loop re-parses it and is the only reporter. A file deleted in between is handled by the serial loop as today | The hot spot is parallel per file; one serial assembly keeps every output's order, which the committed keys prove |
| F2 | Clones only if they still cost: after F1, if cache-hit clones in `read_claude` / `read_codex` still take > 0.3 s cold or > 50 ms warm, cache values go behind `Arc` and assembly clones only the fields it moves out; otherwise nothing changes. Only in `crates/core/src/scanner.rs`, no public type change | The 12 % clone share was measured single-threaded |
| F3 | Budgets (real `~`, release, median of 3 fresh processes): (a) cold `multi-link overview --json` ≤ 2.0 s wall; (b) the TUI worker's first report (`refresh_with_live` with the all-time pass, without and with `--live`) ≤ 2.0 s; (c) first home frame = max(first report, 1.0 s) + 0.25 s ≤ 2.25 s; (d) warm refresh median of 10 ≤ 2 s and ≤ 20 % above 14.4's 191–212 ms. A miss of (a) or (b) after 15.1 stops the phase for a disk-cache step (15.1a); no disk cache otherwise | Measured budgets, not a feeling; a disk cache brings invalidation and format-version risk |
| F4 | Honest progress: an atomics handle created by the CLI worker, set on its `Scanner` (kept across the root reset), a clone sent to the UI before the first pass. One generation for the first load: `files_total` published once after F1 lists both harnesses, `files_done` counts parsed misses and only rises, phase `listing → reading → linking → done` (done when the first report is sent, after the all-time pass); zero misses goes straight to `linking`. Later passes never touch it | The counter on screen is real files, never a timer |
| F5 | Loading screen (board 15 E). Shown until the first report has arrived and s + 1.0 s, then the fade (F8), then home. A first refresh error or a disconnected worker ends it at once (no fade) with the existing error footer. Any key but `q` skips it once (to `loading sessions…` if no report yet, else home); `q` / Ctrl-C quit as everywhere. No new timer: the CLI loop draws every iteration with its 50 ms poll, each frame from the clock. At W × H: art rows `0..=H-8`, title `m u l t i · l i n k` row H-7, a 30-cell bar row H-5, counter row H-3 (`reading transcripts  N / T`, `linking sessions…`, `ready ✓  opening home`), `any key skips · q quits` row H-2, each centred by display width. The art is `drawC(t)` (the board draws it at `t + 1`, so its paused frame is `drawC(4.0)`), x radii scaled by W/76, y radii by (art rows)/15: 96 fish, school i mod 2, an orange ellipse and a blue figure-eight in the other direction, `>`/`<` heads (`›`/`‹` when far), `=`/`-` tails, `·` when near, a pale `*` where both schools meet, 46 specks. A pure function of (time, size, progress, theme) | Testable frames; the approved picture |
| F6 | Colour, quantised to existing roles: fish depth > 0.66 → school role BOLD, 0.33–0.66 → role, < 0.33 → `_dim`; tails one step dimmer; specks `_dim`; the meet `*` `text` BOLD; title `m u l t i` `claude`, `l i n k` `codex`, `·` `text`; bar done cells first half `claude`, second half `codex`, to-do cells `border`; counter `text`, hint `secondary`. 256-colour uses the roles' existing indexes; `NO_COLOR` keeps every glyph and drops every colour and the BOLD/DIM modifiers | No new colour to test |
| F7 | Sizes, checked in this order: W < 30 or H < 6: the counter line only, cut with `…`; W ≥ 60 and H ≥ 16: as F5; otherwise (W 30–59 or H 6–15): no art, no title, no bar; counter and hint centred. 0 rows or columns: nothing drawn, no panic. Any origin. Counts drawn in full and cut by display width | A small terminal still says what is happening |
| F8 | Fade: 250 ms by deadline (not frames): the art switches to `_dim` roles, then to `secondary`, then home. None under `NO_COLOR`, after a skip or after an error | The board's hand-over without a slow transition |

**Pinned while writing the goldens (Step 15.0).** F5–F8 left these open; the goldens depend on them.
- **Geometry.** Art rows A = H − 7. Centre x = W / 2 (real division), centre y = 7.2 · A / 15 (the board's `cy = 7.2`
  at 15 rows; F5 first said A / 2, which would move every fish off the board's frame). School 0:
  x = cx + 27·(W/76)·r·cos a, y = cy + 5.6·(A/15)·r·sin a + 0.4·sin(1.3t + ph); school 1 with b = −a:
  x = cx + 30·(W/76)·r·sin b, y = cy + 6.2·(A/15)·r·sin 2b + 0.4·sin(1.1t + ph) (the 0.4 wobble is not scaled);
  a = ph + 0.42·sp·t; direction from the sign of −sin a (school 0) or −cos b (school 1), ≥ 0 is right;
  depth = 0.5 + 0.5·sin(a + ph/2). Rounding is `floor(v + 0.5)` (JavaScript `Math.round`), not Rust's `round`.
  A fish whose rounded row is outside `0..A` is skipped; cells outside the width are dropped.
- **Specks** at speck time t′ = 0.6 t: speck i has s = 0.3 + 0.12·(i mod 5), base x = 37.3 i mod W, base y = 11.7 i
  mod A, position round(wrap(x − 2.2·s·t′, W)), round(wrap(y − 0.5·s·t′, A)); glyph `·` when s > 0.6 else `.`; role
  `claude_dim` when i mod 3 = 0 else `codex_dim`. A speck that rounds to column W or row A is dropped (the board draws
  such a speck on the title row; at `drawC(4.0)` two of them, row 15 columns 60 and 74).
- **Cells, last write wins** (the board's canvas overprints; at `drawC(4.0)` 35 cells hold two different glyphs and the
  goldens keep the later one): specks, then fish 0..95 (head or `*`, then tail at x − dir, then `·` at x − 2·dir when
  depth > 0.8), then the chrome. A head becomes `*` when a head of the other school was drawn at that cell earlier in
  this frame: a per-frame head map, not the cell's current glyph (a later tail or `·` there does not clear it; a meet
  does not update the map; a same-school head re-records it). Reading it as "the cell's current glyph" changes two
  cells of the 60×16 golden. Tails: `=` when depth > 0.5 else `-`.
- **Thresholds and centring.** Head BOLD role and `>`/`<` when depth > 0.66, role when depth ≥ 0.33, else `_dim`;
  tail `=` when depth > 0.5; the `·` when depth > 0.8. Every centred line starts at ⌊(W − width)/2⌋ (the board's
  rule and ratatui's). Speck bases wrap at W and A; speck speeds are not scaled.
- **Roles.** Head BOLD role / role / `_dim` as F6; its tail `role` / `_dim` / `_dim` (one step dimmer, floor `_dim`);
  the `·` `_dim`; `*` `text` BOLD. Every cell's background is `screen`. Title glyphs BOLD.
- **Bar.** `reading`: T = 0 → 0 cells, else min(30, (60N + T) div 2T) (round half up); `linking` and `done`: 30.
  Done cells 0–14 `claude`, 15–29 `codex`; to-do `─` `border` (the board's track shade; `secondary` would be as bright
  as the hint). **Counter** N is padded on the left with spaces to the digit count of T, so the line does not shift.
- **Small sizes.** W 30–59 or H 6–15: counter on row ⌊(H − 2)/2⌋, hint on the next. Below 30 × 6: counter on row
  ⌊(H − 1)/2⌋, cut to W − 1 glyphs plus `…` when wider than W.
- **Fade steps.** First 125 ms: every fish cell its school's `_dim`, no BOLD, the `*` `secondary`, specks unchanged;
  next 125 ms: every art cell `secondary`; chrome unchanged throughout.
- **Goldens** (`fixtures/expected/tui/`), all at `drawC(4.0)`, counter `reading transcripts  612 / 871` unless named:
  `loading-{120x40,76x22,60x16,59x16,50x12,29x6,28x5}` text + truecolor styles; `loading-76x22-256.styles.txt`;
  `loading-76x22-nocolor` text + styles; `loading-76x22-ready` (phase done), `-linking`, `-zero` (phase reading,
  `0 / 0`, empty bar: guards T = 0; F4 sends zero misses to linking, which `-linking` already shows), `-fade1` (phase
  done, first fade step). The 76 × 22 art rows equal the board's own `drawC(4.0)` glyph for glyph (weights and
  colours per F6; Step 15.0 check, PROGRESS.md). Six-digit counts are left to 15.2's bounded-buffer test.
- **NO_COLOR** drops BOLD too, unlike `Theme::apply_no_color` (which removes only DIM, as every other screen
  wants): the loading screen sets no BOLD under NO_COLOR rather than changing that helper.

### Phase 16 decisions (a multi-link pane inside Claude Code, Step 16.0)
Planned 2026-11-11 (PLAN.md → Phase 16). Need (owner): "while I work in Claude Code I want to see what's running in
this repo, and the Codex runs this session handed off, without switching to the TUI." The owner picked a side pane
over a strip above the prompt, then **board 16 C · Repo view** on 2026-11-11 (claude.ai artifact
2ksgLic1phpGMUD9yojosg, source `docs/boards/board16-mod-pane.html`). Comparables: repo-sessions (a mod with swimlanes;
busy/idle only, Codex rows not tied to the Claude session that started them), the TUI's live grid (Phase 14; C is its
one-repo cousin), Claude Code's built-in tasks pane (one list, state as a glyph). None nests a Codex run under the
Claude session that started it with `exact` vs `likely` said out loud. Measured (release `a97bd1a`, real `~`): a fresh
process is a cold pass (1.16 s), a warm in-process refresh 130–194 ms, so the pane keeps one long-lived process.
Binding answer keys: `fixtures/expected/pane/*.json` (Step 16.0, hand-checked; 16.1 must print them byte for byte).

| # | Decision | Why |
| --- | --- | --- |
| G1 | `multi-link pane --json --cwd <dir> --session <id>` prints exactly what the pane draws: the TUI repo view's rows (W6, W8) for the repo the cwd maps to (`RepoMode::map`), split into `working` (busy), `waiting` (waiting or stale) and `recent` (every other row); a Codex child nested under its placing parent, appearing once; an ambiguous or unlinked hand-off never a child but a `handoffs` entry on its parent (`N possible runs` / `no codex run found`); `match` is `exact` only for an exact link. Shape `{ v, repo, now, this, seen, sections: [ { name, rows: [ { id, harness, label, step, state, phrase, is_this, children: [ { id, harness, state, phrase, match } ], handoffs: [ { step, outcome, candidates } ] } ] } ] }`. Built in `crates/cli` from the TUI's repo-view model with the TUI worker's refresh and one shared `now`; no core type change | The rules exist, are tested and were chosen with the owner; a second derivation would let the two screens drift |
| G2 | `pane --watch --json` keeps one `Scanner` warm and prints one G1 object per line (NDJSON) every 3 s, only when the object minus `now` and every time phrase changed, or every 30 s (heartbeat); first line right after the cold pass; errors print `{ v: 1, error }` and it goes on; it exits when killed, and within one tick when its parent pid changes (or was 1 at start) | Warm ~0.15 s against ~1.2 s cold; the pid check is a backstop if the mod's teardown is missed |
| G3 | The mod (`mod/multi-link-pane/`: `plugin.json`, `hooks/hooks.json`, `hooks/register.tsx`, `types/index.d.ts`) registers `/multi-link`, opens pane `multi-link` on `session.start`, spawns the G2 watcher with `$.process.spawn` for the session's life, keeps the latest object in a `$.state` atom and draws board 16 C (header `multi-link · <repo name>`; `working`, `waiting`, `recent (2h)`; `●` live / `✓` done in harness colour; `▌` this session; `└` children; `likely`; footer `updated Ns ago`). Teardown on `session.end` and unload ends the loop and cancels retry timers. Overflow keeps header, this session (≤ 3 children, then `+N runs`) and footer; other rows drop from the bottom of `recent`, then `waiting`, then `working`, with `+N more` (sessions) | The owner's pick; it shows the coordinator plus one session per step |
| G4 | States: no line yet `reading sessions…`; spawn rejects `multi-link isn't on PATH. See the README.`; unknown command `This multi-link is too old for the pane. Update it.`; `seen: false` pins `▌ this session · not seen yet` at the top of `working`; an `error` line keeps the last data, error dimmed in the footer; watcher exit restarts after 2, 4, 8 … 60 s with `watcher stopped, retrying`; empty `No sessions in this repo in the last 2 h.` | Every failure says what to do |
| G5 | The mod API was confirmed by a spike before any code (results below) | The API is new to this project |
| G6 | No new installs: `claude plugin validate` and `claude plugin test` only; loading is `claude --plugin-dir <repo>/mod/multi-link-pane` (terminal) or `CLAUDE_CODE_PLUGIN_DIRS` (desktop app), the owner's one paste | Stop-and-ask rule on installs |

**G5 spike results (Step 16.0, 2026-11-11, Claude Code 2.1.322, terminal).** A throwaway mod in the session
scratchpad, loaded with `claude --plugin-dir` in an interactive session in a PTY (draining reader thread) in a sandbox
repo, run under `env -i HOME USER LOGNAME TERM LANG PATH` (inherited `CLAUDE_CODE_*` variables make a child session
that writes no transcript). Logged values are in PROGRESS.md (Step 16.0).
- (a) **Pass (terminal).** `$.session.id()` in `session.start`, `session.end`'s `sessionId` and the classic
  `SessionStart` hook's `session_id` are one id, equal to the transcript file name and to every record's `sessionId`
  in it; `transcript_path` = `~/.claude/projects/<encoded cwd>/<id>.jsonl`. G1 keeps `--session <id>`. The transcript
  file appears only after the first prompt, so a new session is `seen: false` until then (G4's placeholder).
- (b) **Pass (terminal).** `$.ui.open` resolved `{ isPlaced: true }`; the pane docks at the right (`placement: "dock"`,
  71 × 43 body in a 160 × 50 terminal); a `$.clock.every(1000)` timer updating an atom re-ran the `ui.render` hook each
  second (ticks 0, 5, 10, 15, 20 logged from inside the render).
- (c) **Pass.** `$.command.register` → `{ command }`; typing it ran the `command.run` hook and printed its text.
- (d) **Pass.** `$.process.spawn` streamed a long-lived child: chunks `{ stream: "stdout", text: "line 1\n" }` (raw
  text, not lines, so the mod buffers to newlines, as G3 says). The child's parent pid is the `claude` process (G2's
  check works). A hot reload (unload) killed the old child and the new module started one (`ps`: only the new child);
  `/exit` ended the loop before `session.end` and no child was left 5 s later. `$.state` atoms survive a reload;
  module variables do not.
- (e) **Pass.** `claude plugin test` mounted the pane on `terminal`, `desktop`, `vscode` and `mobile`. 16.3 tests
  `terminal` and `desktop`. The test kit has no `fs.write` implementation unless the test answers it.
- **Pending for 16.4:** the desktop-app half of (a) and (b) (needs the owner to turn the mod on).

**Pinned while writing the keys (Step 16.0).** G1 left these open; the keys depend on them.
- **Rows.** The repo view's lanes (`timeline.rs` `lanes`, imports off) with one change: a row is visible when not
  imported and either has activity in the 2 h window or is `waiting` or `stale` (the repo view hides a quiet waiting
  session; the pane must not, or a session waiting on the owner, or the owner's own session after 2 h quiet, would
  vanish; home attention and `live` list them too). A row nests under its overview parent only when that parent is
  visible. Every Claude lane, at any depth, is a pane row: a started Claude child (S3, a step session started by the
  coordinator) is its own row, not nested, because sections are by state and the coordinator plus its step sessions
  are what the pane is for. A Codex lane under a Claude lane is that row's child; any other Codex lane is a row. A
  child's parent is the overview's placement (T11: the latest hand-off whose parent is a listed Claude session in the
  child's repo); a child whose only parents are in another repo is a top-level row in its own repo and is absent
  from the parent's repo (unlike `live`, which nests across repos; showing it on the parent's pane is a backlog
  candidate).
- **Order.** Sections always appear, in the order `working`, `waiting`, `recent`, possibly with no rows. Within a
  section: the `this` row first, then a flat sort by `started_at` newest first, then id (a started child does not
  follow its parent, unlike the repo view's pre-order); children newest first, then id.
- **Fields.** `repo` = the cwd mapped by `RepoMode::map` (the overview group's `repo` string; in fixture mode the
  cwd itself), also when no group exists (`empty`); `now` = the report's `now`; `this` = `--session` as given;
  `seen` = the id is a row's or child's id in this object (an id that exists only in another repo is `false`; this
  narrows G1's "in the data"); `is_this` only on rows. `label` = the repo view's own label (`view::home_label`:
  title, else W3), where a Codex row's step comes from `view::placing(row, P)` with P its overview parent (visible or
  not) or, with none, its latest hand-off of any parent (`handoffs` sorted by `at`, invocation, last one), which is
  how `95c00002` and `14270000` get `step 7.1` / `step 3.2`; `step` = that same step (Claude `made[0]`), also when
  a title wins the label, else null; `state` as the reports spell it; `phrase` = W5 (`view::status`) without the
  repo view's `<place> · ` prefix or its ` · N possible runs` suffix on candidate rows (minutes from a burst start
  clipped to the window, as the TUI: a run longer than 2 h reads `for 120 min`); child `match` from its placing link
  (`exact` / `likely` for heuristic). `handoffs` = the 2 h `links` report's links with this Claude row as parent and
  match `ambiguous` or `unlinked`, by (`at`, invocation): `step` (or null), `outcome` (W5 words), `candidates` = the
  candidate ids in the links report's order (empty when unlinked); Codex rows have `[]`.
- **Times.** Phrases use the local offset at `now`, as the TUI; the keys are UTC and their commands run with
  `TZ=UTC`.
- **Text.** `pane --json` prints pretty JSON (2-space indent, non-ASCII as is, trailing newline) like every other
  `--json`; `--watch` prints each object compact on one line.

### Phase 17 decisions (pane strip, row actions, Codex-done notifications, Step 17.0)

Board 17 (`docs/boards/board17-pane-strip.html`): the owner picked S1 · Chain, A1 · Expand in place and N2 · What it
said (2026-11-11). Full text of H1–H6 is in PLAN.md → Phase 17; the pins below are what the answer keys encode.

| Id | Decision |
|---|---|
| H1 | `pane --json` gains `strip` (top level, after `seen`), and `asked`, `said`, `openable` on rows (after `phrase`) and children (after `phrase`, before `match`). `v` stays 1. **Strip:** null unless `seen` and the top-level row whose id is `this` has a step (a session seen only as a nested child gets null); else `{ step, title, parts }` from `multi-link graph` for (repo, that step), parts in graph order, each `{ harness, kind, status, active_min, is_this }`. `active_min` = `active_ms` ÷ 60 000 rounded down, at least 1. `is_this` = the part's session is `this` (several parts can be). **Status:** Codex: graph `done` → `done`, `stale` → `stale`, `working` and in `live_parts` → `running`, otherwise null. Claude: in `live_parts` → `running`; else its session waits and this is the session's latest part → `waiting`; else a later part exists in the same chain → `done`; else null. **asked** = the session's W3 `asked`. **said** = Codex only: the `last_agent_message` of the last completed non-imported turn of the rollout, cut to 120 characters with `…` appended when cut, null when none; Claude null. **openable** = the `open` resolver answers `AppLink` (Claude only). |
| H2 | Strip layout S1 (PLAN.md H2): heading `step <id> <title>`, one line per part, the newest 4 (this session's newest part kept), `+N earlier`. Drop order with an open row as PLAN.md H2. |
| H3 | Rows are `Button`s with `onPress` (17.0: a press runs `onPress`; a `ui.press` hook must answer `{ element }`, so the mod uses `onPress`, not a hook). `Copy id` = `$.ui.copy({ text, surface: e.surface })`; `Open in app` = `multi-link open --app-only <id>`. |
| H4 | Watcher `--notify` only. A run = one hand-off (`run` = the link's invocation id). Order within one tick: event lines first, then that tick's data or tick line. Event: `{"v":1,"event":{"kind":"codex_done","run","id","step","said"}}`, compact. Tombstone `<state dir>/notified/<run>` by `create_new`, never removed. Baselines and at-most-once delivery as PLAN.md H4. Keys: `watch-script.json` (inputs), `watch-codex-done.jsonl` (with `--notify`), `watch-tick.jsonl` (without). |
| H5 | No rule for a section with no rows; `{"v":1,"tick":"<now>"}` on every tick that prints no data line; the footer `checked Ns ago` counts from the last line of any kind. An error line resets the change key (16.2), so the next success prints data. |
| H6 | Checked 2026-11-11 in a PTY terminal session (Claude Code 2.1.322, scratch mod): `osascript` with the H4 argv from `$.process.run` → exit 0, empty stderr (the credited app is seen by the owner in 17.4); `$.ui.notify` → `{ isSent: false, reason: "no-channel" }` in the terminal too, so `osascript` is the one path; `claude plugin test`: a `Button` press runs its `onPress` and `$.ui.copy` receives `{ text, surface }` on terminal and desktop; Codex `said` exists on real rollouts (16.4's graph showed it for `01a1cc93`). Keyboard presses in a scripted PTY did not reach the pane (focus chord), so the press is proven by the test kit and the owner in 17.4. |

## Phase 18 decisions

Step 18.0, 2026-11-12. PLAN.md Phase 18 I1–I5, pinned. Keys: `fixtures/expected/open-jump/` (`open.json`,
`check.sh`, `scripts/{terminal,iterm2}.applescript`), fixture root `fixtures/sessions-phase18`.

| Id | Pin |
| --- | --- |
| I1 snapshot | `ps -axo pid,ppid,lstart,tty,comm` through the probe's `ps_command` (C locale, `TZ=UTC`), one per resolver build, only when a running Claude session has no app link. Parse: header line skipped; per line `pid ppid` then the 5 `lstart` tokens (`Thu Nov 12 08:37:52 2026`, a 1-digit day is space-padded), then `tty`, then the rest of the line as `comm` (paths may hold spaces). Any parse failure, non-zero exit, stderr text, empty output, or two rows with one pid → the whole snapshot is unknown. Fixture mode reads `<root>/fixture-ps.txt` (same text); no file → no snapshot, and the answer stays today's (`running_without_app_link`), so older fixture keys are unchanged. |
| I1 identity | A record is live when its pid is in the snapshot and that row's `lstart`, compared as a time, equals the record's `procStart` (records use a zero-padded day: `Mon Nov 09 05:30:00 2026`). Of a session's records: exactly one live → its pid; none → `process_changed`; two or more → `ambiguous_process`. Unknown snapshot → `process_unknown`. |
| I1 walk | From the live pid upward by `ppid` (at most 64 steps, a cycle or a missing parent stops it): `comm` basename `tmux`, `screen` or `zellij` → `inside_multiplexer`, `detail` = that name; a `comm` path containing `/Terminal.app/Contents/MacOS/Terminal` → app `terminal`; `/iTerm.app/Contents/MacOS/iTerm2` → app `iterm2`; any other `comm` path with a `.app/` component → `terminal_not_supported`, `detail` = the first `X.app` component's `X` (`Visual Studio Code`, `Ghostty`); reaching pid 1 or 0 → `terminal_not_supported`, no `detail`. The tty comes from the session's own row: `??` or `-` → `no_tty`; else `/dev/` + the value. |
| I1 JSON | Jump: `{"action":"jump","app":"terminal"|"iterm2","tty":"/dev/ttysNNN","pid":N,"cwd":…}`, exit 0 on `--dry-run`. Unavailable keeps `action, reason, command, cwd` and adds `detail` after `reason` only when it has one. New reasons: `inside_multiplexer`, `terminal_not_supported`, `no_tty`, `process_changed`, `ambiguous_process`, `process_unknown`. Their `command` is today's `claude --resume <id>` text, unchanged. `--fork` on a running session stays `running_without_app_link`. |
| I2 scripts | Exactly `scripts/terminal.applescript` and `scripts/iterm2.applescript`, run with one `-e` per script line and the tty last: argv `["osascript","-e",<line 1>,…,"-e",<line N>,<tty>]` (no shell, no file, no text spliced into the script). iTerm2 is addressed by bundle id (`com.googlecode.iterm2`) so no "Where is iTerm2?" dialog can appear by name; it is untested (not installed here). Answer `window W tab T` (digits) → jumped; `not found` → `tab_gone`; stderr containing `-1743` → `automation_denied`; anything else, missing `osascript`, or 5 s passed (then killed) → `jump_failed`. |
| I2 texts | `jumped to Terminal · window 2 · tab 3 (ttys004)` (app name `Terminal` / `iTerm2`, tty without `/dev/`); `cannot jump: its tab is gone`; `cannot jump: blocked by macOS. Allow it in System Settings → Privacy & Security → Automation`; `cannot jump: <reason>`. Failures exit 3 and never resume. |
| I3 pane | `opens` after `openable` in rows and children: `"app"` for `AppLink`, `"terminal"` for `Jump`, else null. |

Real facts (18.0, 2026-11-12, this Mac, sandbox `repo-p17`): `claude` started in a new Terminal.app window by
`osascript … do script` wrote a record with `pid` and `procStart` (`Thu Nov 12 08:37:52 2026`, UTC); `ps` with
`TZ=UTC` printed the same `lstart`, tty `ttys006`, and the chain `claude` → `-zsh` → `login` → Terminal's binary
(`??` tty, ppid 1). AppleScript's `tty of tab` is `/dev/ttys006`. The Terminal script by argv answered `window 1 tab
1` and the front window's selected tab then had tty `/dev/ttys006`; `/dev/ttys999` and the bare `ttys006` answered
`not found`. No Automation prompt appeared (the caller was already allowed); the -1743 text is from Apple's
`errAEEventNotPermitted` and is still to be seen live (18.4).


## Phase 19 decisions

Step 19.0, 2026-11-12. PLAN.md Phase 19 M1–M5, pinned. Keys: `fixtures/expected/menubar/` (`multi-repo.json`,
`waiting-quiet.json`, `open-actions.json`, `watch-codex-done.jsonl`, `check.sh`). Fixture roots are the existing
`sessions-phase14`, `sessions-phase9` and `sessions-phase18`; no new fixtures were needed.

| Id | Pin |
| --- | --- |
| M1 report | One JSON object per line: `{"v":1,"now","waiting","running","repos":[…]}`. `now` as the pane prints it. Flags as `pane`: `--root`, `--now`, `--json`, `--watch`, `--notify`; no `--cwd` or `--session` (no "this session"). |
| M1 repos | Every Session `repo` (from `sessions --since all`) whose pane, run for that repo with a session id that matches nothing, has at least one row. So the pane's visibility rule decides (activity in 2 h, or `waiting` / `stale` at any age, never imported). Each: `repo`, `name` (last path part), `tier`, `last_activity`, `step`, `rows`. |
| M1 order | Overview's RepoGroup order applied to the shown sessions (rows and children): `tier` `waiting` if any is `waiting`, else `busy` if any is `busy`, else `other` (`stale` is `other`); then `last_activity` (greatest of the shown sessions) descending; then `repo` ascending. |
| M1 rows | The pane's rows for that repo, flattened in section order (`working`, `waiting`, `recent`), each with `section` first and `is_this` removed; everything else (children, `openable`, `opens`, `match`, `handoffs`) byte-for-byte the pane's. |
| M1 step | From the repo's rows in order: the distinct non-null `step` of top-level rows whose `state` is `busy`, `waiting` or `stale`. None → `null`; else `{"id": first, "title": <its PLAN.md title at repo/PLAN.md, or null>, "more": count − 1}`. (Revised from "overview `active_steps`": that field is empty for a quiet waiting repo outside the window, and has no order.) Fixtures have no PLAN.md, so every key's `title` is null; 19.1 tests a title with a temp repo. |
| M1 counts | `waiting` = Claude rows and children (any repo) with `state` `waiting` or `stale`. `running` = any row or child `busy`. |
| M2 events | The pane's event line, unchanged: `{"v":1,"event":{"kind":"codex_done","run","id","step","said"}}` (`run` = invocation, the marker key; `id` = Codex session; `said` cut to 120 characters). One contract for both watchers; the app builds the title `codex finished (<step>)` (or `codex finished` when `step` is null) and the body from `said`, as the mod does. Two hand-offs to one Codex session give two events (`watch-codex-done.jsonl`, runs `toolu_watch_a` / `toolu_watch_b`). |
| M2 lock | `~/.multi-link/menubar.lock` (in the home the watcher reads; `<root>/.multi-link/…` in fixture mode). The pane watcher with `--notify` tests it with a non-blocking `flock(LOCK_EX)` right before `claim`: if another process holds it, skip the claim and the event for that run (the run stays unclaimed); else unlock at once and claim as today. The menubar watcher never checks the lock (it is the app's own watcher). |
| M3 signing | `codesign --force --sign - -r='designated => identifier "dev.multi-link.menubar"'`. With a plain ad-hoc signature the designated requirement is the code hash, and macOS ties the notification permission to it, so every rebuild loses the permission. Pinning the requirement to the bundle id keeps it (fact below). |
| M3 permission | Don't trust `requestAuthorization`'s first callback: on this Mac it returned `false` ("Notifications are not allowed") the moment the owner clicked Allow, and the banner still showed. The app treats `getNotificationSettings().alertSetting == .enabled` as "can deliver", re-read at launch, when the popover opens and every 60 s. That value drives the M2 lock. |
| M3 sandbox | Codex builds Swift with `--add-dir "$HOME/.cache/clang" --add-dir "$HOME/Library/Caches/org.swift.swiftpm" --add-dir "$HOME/Library/org.swift.swiftpm"` and `swift build --disable-sandbox` / `swift test --disable-sandbox` (SwiftPM's own `sandbox-exec` can't nest in Codex's sandbox). |
| M4 first jump | The app's first terminal jump shows macOS's "allow multi-link to control Terminal" prompt while `osascript` waits; the CLI kills `osascript` after 5 s. A slow click then fails with `osascript timed out after 5 s`. The app's footer adds "if macOS asked for permission, try again" to that one message; the CLI is unchanged. |

Real facts (19.0, 2026-11-12, this Mac, macOS 15.7.3, Swift 6.2.4, scratch spikes; owner answers quoted):
- A SwiftPM executable wrapped as an `.app` (`LSUIElement`, bundle id, `LSMinimumSystemVersion` 15.0,
  `NSAppleEventsUsageDescription`), signed ad hoc, passed `codesign --verify --strict` and showed its `MenuBarExtra`.
- Launched from `/private/tmp/…`, notifications were refused with no prompt (`UNErrorDomain` 1). Copied to
  `~/multi-link-sandbox/` and launched with `open`, macOS asked; owner: "Allowed, saw banner" (title, body, sound
  with the `[.banner, .sound]` foreground delegate). So `build.sh` must not run the app from a temp folder.
- Plain ad-hoc rebuild (new code hash): `authorizationStatus` denied, `add` returned no error, owner: "No banners".
  `com.apple.ncprefs` showed one requirement entry per code hash.
- With `-r='designated => identifier "…"'`: first launch prompt, owner "Allowed, saw B1"; after a rebuild with a new
  code hash, `authorizationStatus` 2 (authorized), `requestAuthorization` true, owner "Yes, B2 showed".
- From inside the app: `multi-link open --no-resume <id>` (binary path containing a space) printed `jumped to
  Terminal · window 2 · tab 1 (ttys006)`, exit 0, after macOS asked to let the app control Terminal (owner: "Prompt,
  I allowed"). `osascript -e 'on run argv' -e 'tell application "Terminal" to do script (item 1 of argv)' …` with the
  single-quoted path plus ` tui` opened Terminal running the TUI.
- `flock(LOCK_EX|LOCK_NB)` on a file opened `O_CREAT|O_RDWR|O_CLOEXEC`: held while the app ran (a second process got
  `EWOULDBLOCK`), not visible in its child's open files (`lsof`), free right after `pkill`.

## Phase 20 decisions

Step 20.0, 2026-11-12. PLAN.md Phase 20 R1–R6, pinned. The Claude Code pane is retired: the menu-bar app does its job
across every repo.

| Id | Pin |
| --- | --- |
| R2 module | `crates/cli/src/pane.rs` → `crates/cli/src/rows.rs`. Kept: row projection and enrichment, the watch loop, `done_runs`, `FileMarkers`, `TurnCache`. Removed: the `pane` subcommand, the step strip, `--cwd`/`--session`/`is_this`, `PaneMarkers`. |
| R3 notify | The app runs `menubar --json --watch --notify` only while it can deliver, and without `--notify` otherwise (rotation as in 19.2). `DeliveryGate` and the 10 s health rule go. `~/.multi-link/menubar.lock` stays, held for the app's whole life as a single-instance guard: a second instance shows "multi-link is already running", starts no watcher and quits. |
| R4 keys | Two keys captured from `menubar --json` at the pane keys' other times: `phase9-early.json` (phase9, 06:00) and `phase9-noon.json` (phase9, 12:00); with the three 19.0 keys they cover every pane key's root and time. `watch-script.json` copied; `watch-tick.jsonl` rebuilt menubar-shaped (report lines `{"v":1,"now","waiting":0,"running":false,"repos":[]}`). Tests that used `pane/children-exact-likely.json` as a pass value use `menubar/multi-repo.json`. |
| R4 cross-check | All 12 pane keys, 55 rows: each pane row minus `is_this` equals the menubar row with the same id in the same repo minus `section`, and the id sets match (scratch script, 2026-11-12). |
