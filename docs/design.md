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
- **Incremental reads (revised in Phase 3, T1).** A refresh re-parses only the files whose (path, length, mtime, inode) changed since the last pass, each from the start; unchanged files reuse their cached parse, and linking, state and windowing are recomputed from the cached results every pass, so an old parent is still found when a new child references it. A partial last line is buffered, never skipped; a truncated or replaced file is just a changed key. Byte-offset tailing was the original plan; it is deferred because whole-file re-parse of the few changed files was fast enough (Step 3.1), and it returns as its own step only if a later measurement misses the speed threshold.
- **Speed (pass threshold).** The home screen refreshes in 2 s or less on my laptop with my real `~/.claude` and `~/.codex`. Step 3.1 measured the sessions + links refresh on one warm `Scanner` at 29 ms and 31 ms (passes 2 and 3, real `~`, `PROGRESS.md` 3.1); the full home refresh including the overview is measured in Step 3.8.
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
| V5 | Bars like the mockup: a fixed bar column right of the label column; track cells are blanks on the track background; Claude `━` orange, Codex `█` blue; clipped ends `◂` / `▸`; a zero-length span is one cell. Axis labels above the first card in local time, the last one `HH:MM now` | The bar is the screen's main visual; a solid block reads at a glance where a dotted track did not |
| V6 | Plain words: exact → `confirmed`, heuristic → `matched by time`, ambiguous → `N possible runs`, unlinked → `no run found`. Last line per expanded repo: `last: claude handed step X to codex at HH:MM (<link words>)` from `latest_handoff` (`step ?` when the link has no step) | Internal words (`heuristic exec`) meant nothing to the owner |
| V7 | Local time: the UTC offset is read once per refresh for the report's `now` (`time` crate `local-offset`); fallback UTC, with the axis saying `UTC` after the first label. Fractional offsets work. When the window crosses local midnight the first axis label carries the date (`Oct 29 23:20`); a time not on `now`'s local date carries it too (`waiting since Oct 28 17:05`). JSON timestamps stay UTC. Limitation: a window across a DST change uses the offset at `now` | The owner reads clock time, not UTC; one offset per refresh keeps a frame consistent |
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
| W5 | Status phrases and times. `state_since`: waiting/stale → the record's `statusUpdatedAt` or the winning hook record's `at`; Codex `unknown` / `turns_complete` → the last non-imported `task_complete`; otherwise null. Phrases: `working now · for N min` (from the last burst's start; `working now` with no burst), `waiting on you · since T`, `stale since T`, `idle · last active N min ago` (last burst end, else `last_activity`), Codex `done T`. Codex "done" means its last turn completed; there is no process-exit evidence. Times: only `last active …` is relative (`N min ago` under 60 min, else a clock time); every other time (`since`, `done`, `at`, `started`, `last hand-off`, `active T1–T2`) is a clock time: `HH:MM` on `now`'s local date, `Mon D` before. Minutes are whole minutes rounded down, at least 1. Hand-offs: `handed step X to codex` / `handed work to codex` (untagged); outcome `codex done` (child `turns_complete`), `codex running` (child busy or stale), `codex state unknown` (other child states), `no codex run found` (unlinked), `N possible runs` (ambiguous). No generated `?` placeholder anywhere (prompts may contain `?`) | Times the owner can act on, from evidence the core already has |
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
2. **Exact.** A link is exact only when the child Codex session id is explicitly observed, checked in this order: the `resume` argument; a `session id: <uuid>` line in the tool result; `[codex] Thread ready (<uuid>)` in the result (plugin); a `session id:` line in the task output file the result names (background). The hook records the same evidence; it does not make a guess exact.
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
| Finished | Claude: the record's process is gone, or its pid now has a different start time than `procStart` (P5). |
| Stale | `busy` evidence older than 300 s (record or open Codex turn). |
| Unknown | No usable signal: transcript-only sessions, Codex sessions whose turns all completed (P6). |

Silence alone never means busy or waiting. Alerts (FR14) fire only on a supported waiting signal (session record or hook).

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
7. **Backlog (ordered, uncommitted):** step graph (screen 3), live grid (screen 5), tmux / Herdr jump, desktop app with notifications, mod.

### Open questions

- [x] Can a terminal program open a running desktop-app session? Yes: `open "claude://code/continue?session=local_<id>"` switched the app to a live route-impact session (tested 2026-10-29). Format found in repo-sessions' code; undocumented.
- [x] Phase 0 (a)–(d): child-id observability, state signals, zoetrope crate, agtrace overlap. Answered in `docs/phase0.md`; decisions P1–P12 above.
- [x] May step ids carry a letter suffix (`0.2a`)? Yes, one lowercase letter (P11).
- [ ] Do `PermissionRequest` / `Notification` fire for every permission prompt and question, so they can drive "waiting"? Planned answer: Step 2.1 (h).
- [ ] How to find the `local_` app id of a finished desktop-app session.
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
