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
5. **Backlog (ordered, uncommitted):** step graph (screen 3), live grid (screen 5), tmux / Herdr jump, desktop app with notifications, mod.

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
