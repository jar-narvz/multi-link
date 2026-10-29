# Phase 0 — hand-offs, state signals and dependencies

Questions (a) and (b) come from Step 0.2 (Codex, reviewed by Claude); (c) and (d) from Step 0.3
(merged here from `docs/phase0-deps.md` in Step 0.6). How each result changed the design:
`docs/design.md` → "Phase 0 results".

| Question | Result | One-line reason |
| --- | --- | --- |
| (a) Is the child Codex id observable per launch form? | go | Seen for exec, resume, plugin (subagent transcript) and background (task-output file) |
| (b) Busy / waiting / idle / finished signals? | no-go for *waiting* only | Busy, idle and finished are supportable; no pending-approval signal in either harness |
| (c) zoetrope core as a library crate? | no-go | Public API is a lossy display vocabulary; parsers are `pub(crate)`; pulls in ratatui |
| (d) Does agtrace already link Claude → Codex? | go (it doesn't) | Links only within one harness; confirmed by source and by running it on sandbox copies |

**Step 0.2 review corrections (Claude).** Section (b) below is Codex's text as written; these
review findings override it where they differ:
1. The 300 s freshness limit applies to `busy` only. Outside the sandbox, `ps -p <pid> -o lstart=`
   showed all 5 recorded pids alive, including 3 idle records aged 6131–9224 s: idle desktop
   sessions legitimately go hours without an update.
2. Liveness is confirmable with `ps` (pid + start time vs `procStart`); the "unverified" gap in (b)
   was a Codex-sandbox limit, not a data gap.
3. The (b) no-go is about *waiting* only. Waiting for Claude needs another source (Phase 2 hook);
   Codex waiting stays unsupported.
4. Background exact links depend on a temp task-output file that can vanish; when it is gone the
   link falls back to the heuristic (never guessed exact).

## (a) Observable child IDs and exact links

Evidence: the 10 cases in `fixtures/raw/ground-truth.md`, copied sandbox
transcripts, and organic transcript samples. All real data remains ignored under
`fixtures/raw/`: 13 Claude transcripts (9 sandbox parents, 1 sandbox subagent,
2 route-impact sessions, 1 physaflow control), 8 Codex transcripts, 5 Claude
session records, and 1 background task output. Source directory structure and
session filenames are preserved below `.claude/` and `.codex/`; the task output
is below `task-output/`. Counts and equality checks are saved in
`fixtures/raw/phase0-evidence.json`; regenerate with
`python3 fixtures/raw/inspect-phase0.py`. No conversation excerpts are included here.

### Record paths by invocation form

Interpret paths relative to one JSONL record. Match `message.content[]` elements
by `type`; do not assume a fixed array index in a parser. Pair an
`assistant` record's `message.content[].type=tool_use` and `.id` with a `user`
record's `message.content[].type=tool_result` and `.tool_use_id`. Require a
recognized launch in `.name=Bash`, `.input.command`; mere word matches do not
establish a launch. Top-level `sessionId`, `cwd`, `timestamp`, and `uuid` provide
parent identity and source provenance.

- **exec:** the sandbox parent's `user` record at line 27 holds the child ID in
  `message.content[0].content`, a string containing the stdout header
  `session id: <uuid>`. The same header is present in top-level
  `toolUseResult.stdout` (1 occurrence), with 0 occurrences in
  `toolUseResult.stderr`. Extract with `(?m)^session id:\s*(<uuid>)\s*$`;
  `<uuid>` means a UUID-shaped token, not an arbitrary text substring.
- **resume:** the parent's `assistant` record at line 26 has
  `message.content[0].input.command` with `codex exec resume <uuid>`.
  The following `user` record at line 27 repeats that ID in
  `message.content[0].content` and `toolUseResult.stdout` using the same
  `session id:` header. The explicit argument identifies the requested existing
  session; the output confirms it actually resumed. Model this as another
  hand-off to the existing child, not as a newly created child session.
- **plugin through subagent:** the parent `assistant` record at line 44 invokes
  `message.content[0].name=Agent`, with `.input.subagent_type`. Its `user` result
  at line 45 has top-level `toolUseResult.agentId`, `.outputFile`, `.isAsync`,
  and `.status`. Follow `.agentId` to
  `<parent-session>/subagents/agent-<agentId>.jsonl`. That transcript retains
  top-level `sessionId` for the parent, `agentId`, and `isSidechain`.
  Its line 16 invokes Bash through `message.content[0].input.command`;
  line 17 is a `user` / `tool_result` whose `message.content[0].content`
  contains `[codex] Thread ready (<uuid>)`. Extract the UUID immediately
  following this label; a separate `Turn` UUID in the same output is not the
  child session ID. There is no top-level `toolUseResult` on that result record.
  The parent's agent ID and the Codex thread ID are different identifiers.
- **background:** the parent's line 26 has
  `message.content[0].input.run_in_background=true`. Line 27 is a `user` /
  `tool_result` with top-level `toolUseResult.backgroundTaskId`; its `.stdout`
  and `.stderr` are empty. `message.content[0].content` names the task output
  file. That file contains the `session id: <uuid>` header; the preserved copy
  is `fixtures/raw/task-output/be01y5dv6.output`. Later tool results at lines 43
  and 49 do not contain the child ID. The task ID alone is insufficient; exact
  linking depends on retaining and reading the named output file. Missing or
  expired output must leave the link unresolved.

### Cross-check against launch ground truth

Comparisons use the full UUIDs in the raw evidence; IDs are not reproduced here.
Each positive child also matches its Codex `session_meta.payload.id`.

- `exec`: 1 output child ID; equals truth: yes.
- `resume`: 1 output child ID; equals truth: yes; the resume argument also
  equals truth and targets the same child as `exec`.
- `plugin`: 1 thread-label child ID; equals truth: yes.
- `bg`: 1 task-output child ID; equals truth: yes.
- `par-main`: 1 output child ID; equals truth: yes (parent result line 27).
- `par-wt`: 1 output child ID; equals truth: yes (parent result line 27).
- `amb-1`: 1 output child ID; equals truth: yes (parent result line 28).
- `amb-2`: 1 output child ID; equals truth: yes (parent result line 27).
- `mention`: 0 observed child IDs; equals truth: yes, no hand-off. Neither
  `codex --version` nor listing session files qualifies as a launch.
- `manual`: 0 parent hand-off observations; equals truth: yes, no parent.
  Its copied Codex transcript's metadata ID equals the truth child ID.

The parallel worktree and main-checkout cases are distinct. The close-together
pair is exact with these outputs available; it becomes ambiguous if those IDs
are withheld and only checkout/time evidence remains. Never manufacture a
parent for the manual case from checkout/time proximity.

The organic route-impact samples contain Bash launch candidates with output
redirection and `toolUseResult.backgroundTaskId`, including
`toolUseResult.backgroundedByUser`; no `session id:` or thread-label child IDs
were recovered from their tool-result content. These are not ground-truth
verified exact links. Searching all 42 physaflow transcripts found 0 tool-use
inputs containing `codex`; textual mentions existed in 42 files. One was copied
as an organic negative control. Raw text search alone had overstated organic
launch coverage. Sandbox cases supply the verified hand-offs across 2 repos.

### What the child records about its origin

For `type=session_meta`, `payload.id` and `payload.session_id` identify the child;
`payload.cwd`, `payload.runtime_workspace_roots`, and `payload.git` locate the
checkout. `payload.timestamp` and each record's top-level `timestamp` provide
time evidence. `type=turn_context` also has `payload.cwd` and `payload.turn_id`.

The sampled CLI children have `payload.originator=codex_exec` and
`payload.source=exec`; the plugin child has `payload.originator=Claude Code`
and `payload.source=vscode`. This indicates launch machinery, not a unique
Claude parent. `payload.thread_source` is also present in the CLI metadata.
No structured parent-Claude ID or environment-variable map was found in the
sampled metadata or turn contexts. Occurrences of `CLAUDE_SESSION_ID`,
`CLAUDECODE`, `CODEX_THREAD_ID`, and `CODEX_INTERNAL_ORIGINATOR_OVERRIDE` in the
8 child transcripts: 0 for each. Do not assume inherited environment variables
are persisted. Cwd, originator, and timestamps can support candidates but cannot
establish an exact parent, especially for manual runs and simultaneous launches.

Exact links are feasible for all 4 sampled forms when their specific evidence
is available. This does not establish universal output stability or exact links
for redirected, truncated, failed, or missing results. Unknown records must be
skipped and counted; unavailable correlation evidence stays unresolved.

Decision: go

## (b) State signals, approval gaps, and freshness

### Claude

The 5 copied `sessions/<pid>.json` records all contain `pid`, `procStart`,
`pidDomain`, `sessionId`, `cwd`, `kind`, `entrypoint`, `startedAt`, `updatedAt`,
`status`, and `statusUpdatedAt`. Also observed: `version`, `hostSessionId`,
`messagingSocketPath`, `peerProtocol`, `peerFeatures`, `name`, `nameSince`, and
optional `nameSource`. Status values in this sample: `busy` in 1 record,
`idle` in 4; 0 explicit waiting or finished values. Millisecond epoch fields
`statusUpdatedAt` and `updatedAt` measure recorded updates; do not assume they
are periodic heartbeats. Identify a process using `pid` plus `procStart` and
account for `pidDomain`; a PID alone can be reused.

- Busy: use fresh `status=busy` as reported busy, conditional on process identity.
  Transcript `assistant` tool-use records and unmatched tool results provide
  activity evidence only, not proof of a live busy process.
- Idle: use fresh `status=idle` as reported idle, under the same process checks.
  A final assistant message alone is not proof that the harness exited.
- Waiting: not established. Copied transcripts have
  `permissionDecision.decision=accept` in 206 records; these are completed
  permission decisions, not pending requests. Also observed: `permissionMode`,
  `sourceToolAssistantUUID`, `message.stop_reason`, and system
  `subtype=stop_hook_summary`. All 67 sampled system stop summaries have an
  empty `stopReason`. None establishes a pending approval or a waiting state.
- Finished: require an independently confirmed matching process exit or explicit
  terminal evidence. Absence of a live-session file or old transcript activity
  alone is insufficient. Historical sessions may have unknown lifecycle state.

### Codex

A metadata-only scan of 91 local Codex transcripts found these `event_msg`
`payload.type` values: `task_started`, `task_complete`, `item_completed`,
`thread_settings_applied`, `token_count`, `user_message`, and `agent_message`.
There were 0 malformed JSONL lines and 0 explicitly named approval/waiting event
types in that scan. Saved counts are in `phase0-evidence.json`. This is an
observed-format result, not proof that other versions never emit such events.

- Busy: `event_msg.payload.type=task_started` gives `turn_id`, `root_turn_id`,
  and `started_at`. Pair subsequent events by turn, rather than treating a file
  as permanently busy. Recent `item_completed`, `token_count`, and response
  tool records support activity but do not prove that a process is still alive.
- Idle / finished turn: `event_msg.payload.type=task_complete` has `turn_id`,
  `root_turn_id`, `started_at`, `completed_at`, and `duration_ms`. It ends that
  turn; it does not establish process exit. A resumed CLI child has 2
  `task_started` and 2 `task_complete` events in one file. An interactive harness
  may remain idle after completion; a CLI run needs exit evidence to be finished.
- Waiting: not established. `turn_context.payload.approval_policy`,
  `.approvals_reviewer`, `.sandbox_policy`, and `.permission_profile` describe
  configuration. They do not record a pending approval. The sampled
  `response_item.payload.type` values include `custom_tool_call` and
  `custom_tool_call_output`; an unmatched call can also mean execution,
  interruption, truncation, or buffering. Never translate it directly to waiting.
- Unknown lifecycle: no confirmed process identity or terminal event means
  unknown; timestamps alone do not distinguish an exited run from a quiet one.

### Proposed freshness and stale-case search

Propose a configurable **300-second** freshness limit for provisional reported
busy/idle state, evaluated against a fixed `now` in tests. This is a conservative
starting policy, not a measured heartbeat interval. Preserve the last reported
status and its age, but label stale evidence `stale/unknown`; do not promote
silence to waiting or finished. Refresh source records before computing age.
Completion events remain historical turn evidence even after this limit.

At the saved evidence sample time, the 5 copied session records had
`statusUpdatedAt` ages of 167, 9069, 6087, 5976, and 163 seconds. Thus 3 organic
idle records exceed the proposed limit. They are copied stale-record candidates.
`os.kill(pid, 0)` returned `EPERM` for all 5 candidate PIDs, while a nonexistent
PID returned `ESRCH`; this is suggestive of existing processes, but identity and
liveness are not confirmed. `ps` was denied by the execution sandbox. Therefore
an independently confirmed **live PID plus old statusUpdatedAt** case is still
unverified, rather than claimed as a completed fixture. The generated cases
also lack such a live-but-stale run, as recorded in the ground truth.

Proceeding with evidence labels and stale/unknown fallback is feasible, but an
authoritative busy/waiting/idle/finished mapping is not proven. Explicit pending
approval and confirmed process-lifecycle samples remain gaps for design review.

Decision: no-go

---

## Method for (c) and (d)

**Method and its limit.** The owner's safety policy for this step: no cloning, downloading to disk,
building, installing or running third-party code. Both projects were read **only through the GitHub
API with `gh`**, streamed to the terminal and never saved. So every answer below comes from reading
source at one commit, not from running anything. PLAN.md Step 0.3 also asked to run agtrace on the
0.2a sandbox sessions; that was not done (see (d), "What running it would add").
After owner approval, agtrace was later built and run on copies of the sandbox sessions
(outside this repo); see (d), "Run on the 0.2a sandbox sessions".

Shared helper used for every file read below (`$REPO` and `$SHA` per section):

```
gh api "repos/$REPO/contents/<path>?ref=$SHA" --jq .content | base64 -d   # piped to grep / sed -n
```

---

## (c) Is zoetrope's core usable as a library crate?

- Repo: https://github.com/furkankly/zoetrope
- Commit read: `b1f31dd26bd4e9e513885e39edb78d0850a5d1fe` (HEAD of `main`, committed 2026-09-11)
- License: MIT (`gh api repos/furkankly/zoetrope --jq .license.spdx_id`, plus `license = "MIT"` in `Cargo.toml`)

Commands used:

```
gh api repos/furkankly/zoetrope/commits/HEAD --jq '.sha+" "+.commit.committer.date'
gh api repos/furkankly/zoetrope --jq '.license.spdx_id, .default_branch'
gh api "repos/furkankly/zoetrope/git/trees/$SHA?recursive=1" --jq '.tree[] | select(.type=="blob") | .path'
# then the contents helper on each file below, filtered with grep -n '^\s*pub ...' or sed -n ranges
```

Files relied on: `Cargo.toml`, `src/lib.rs`, `src/provider/mod.rs`, `src/fact.rs`,
`src/state/mod.rs`, `src/state/graph.rs`, `src/state/session.rs`, `src/tailer/mod.rs`,
`src/provider/summary.rs`, `src/provider/codex/mod.rs`, `src/provider/codex/discovery.rs`,
`src/provider/claude/discovery.rs`, `docs/DISCOVERY.md`, `README.md`.

### Findings

**Layout.** One package, `zoetrope` 0.2.0 (edition 2024, MSRV 1.88), with a lib target
(`src/lib.rs`) and a bin target `zoe` (`src/main.rs`, `required-features = ["native"]`). The
`[workspace]` only exists to exclude `web/wasm` (the browser frontend, its own crate
`zoetrope-web`). Published on crates.io as `zoetrope` (README badge and `cargo install zoetrope`).

**Is there a library?** Yes. `src/lib.rs` documents a "portable core" usable with
`default-features = false`, and that is exactly how `zoetrope-web` uses it. Public modules:
`fact`, `provider`, `state`, `tailer`, `ui`; `autopilot`, `handler`, `tui` are native-only and
explicitly have "no stability promise". `lib.rs` says the parts meant to be depended on are the
domain (`state`) and the vocabulary (`fact`). But the crate docs also say "this is an application",
and it is at 0.x, so there is no API stability guarantee.

**Public parsing API** (`src/provider/mod.rs`):
- `enum Provider { Claude, Codex }` with `name`, `parse`, `all_paths`, `session_file`,
  `session_file_from`, `related_paths`, `project_key`, `stream_for`, `sidecar`.
- `enum Stream` with `push(&mut self, line) -> Option<Statement>` — the per-file parser.
- `fn provider_of(head)` — sniffs the format from the first line's content.
- `struct SessionFile { provider, path, session, role, read, project_key, modified }`,
  `enum FileRole { Root, Agent { parent }, Sidecar }`, `enum ReadMode`, `struct Session`
  (`every_file`, `rescan`), `struct Scope`, `enum Target`, `enum OpenError`.
- Free functions `assemble`, `sweep(scope, only)`, `open(target, only)`.
- The format-specific parsers themselves (`provider::claude`, `provider::codex`, their `wire.rs`
  serde models, and `summary`) are **`pub(crate)`**. A dependent only ever sees `Fact`s.

**Public model** (`src/fact.rs`, `src/state/`):
- `fact::Fact { agent, ts, kind }`, `fact::FactKind` (`Agent { kind, parent, agent_type,
  description, spawned_by, interactive }`, `Activity`, `Label`, `Model`, `Tokens`, `Prompt`,
  `Reasoning`, `ToolStart { call, name, summary }`, `ToolEnd { call, outcome }`, `Spawn`, `Ended`,
  `Session { label, value }`, `Tally`, `Title`), `fact::Statement { at, facts }`,
  `fact::Outcome { Ok, Err }`, `AgentKind`, `AgentStatus`.
- `state::session::SessionModel` (the fold of facts), `AgentInfo`, `ToolCallInfo`, `SpawnContext`.
- `state::AgentFlow` = `rataflow::Flow<AgentNode, AgentEdge>`; `state::graph` (`new_flow`, `sync`,
  `relayout`) is a **UI layout projection** for the TUI, not a general graph model.

**Dependency weight (core, `default-features = false`).** `ratatui` 0.30 (no default features),
`rataflow` 0.1 (with `sugiyama` layout), `serde`, `serde_json`, `chrono`, `web-time`,
`unicode-width`, `imbl` 7. The `native` default adds `tokio` (multi-thread), `crossterm`,
`futures`, `anyhow`. So even the "core" pulls in a terminal UI framework and a graph-layout crate
that multi-link (a CLI that prints JSON) does not need.

**Fit against what multi-link needs** (field names here are assumptions until Phase 0 confirms
them on real data):
1. **Lossy by design.** Tool-call input reaches a dependent only as `ToolStart.summary`, a one-line
   rendering capped at 200 chars (`SUMMARY_MAX` in `src/provider/summary.rs`). Tool output is
   reduced to `ToolEnd.outcome` = `Ok`/`Err`; the text is dropped. multi-link's `exact` link needs
   the child Codex session id, which (per the design) we expect to see in the `codex exec` tool
   result text or in the full command. Output text never survives the `Fact` boundary, and full
   commands are not reliably preserved (short Bash commands ≤ 200 chars survive whole, so a
   `resume <id>` argument can survive; longer ones are cut).
2. **No cross-harness linking.** zoetrope's parent/child is within one harness: Claude subagents
   and workflows (`<uuid>/subagents/agent-*.jsonl`) and Codex child threads
   (`session_meta.source.subagent.thread_spawn.parent_thread_id`). `assemble` groups by
   `(provider, session)`; it never relates a Claude session to a Codex one.
3. **Raw records are private.** We could not reach Codex `session_meta` fields like `originator`
   except via `FactKind::Session { label: "app", value }`, a display string.

### Decision

No-go on depending on it as a crate: the public surface is a lossy display vocabulary (truncated
summaries, outputs reduced to Ok/Err), the raw parsers are `pub(crate)`, the core drags in
`ratatui` + `rataflow` + `imbl`, and the API has no stability promise. We write our own parsers.

Worth learning from (credit zoetrope, MIT, in our README / source comments if we adapt anything):
- The provider split: `wire.rs` (defensive serde: unknown types, missing fields and bad lines parse
  to something skippable, never a panic), `discovery.rs` (pure path logic, at most a head read),
  `mod.rs` (per-file stream). Same shape suits our parsers.
- "A provider states what a file is; the core decides what a session is" (`docs/DISCOVERY.md`).
- Content-based format sniffing (`provider_of`: Codex lines carry `payload` / `session_meta`).
- Layout facts to check in Phase 0 (assumptions, from their docs): Claude
  `~/.claude/projects/<sanitized-cwd>/<uuid>.jsonl` plus `<uuid>/subagents/`; Codex
  `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl`, with `thread_source` and
  `source.subagent.thread_spawn.parent_thread_id` in `session_meta`, and a child rollout replaying
  its parent's `session_meta` further down (only the first one is the file's own).
- Their golden-file conformance tests (`assets/<name>/<fixture>.model.txt` / `.timeline.txt`).

Decision: no-go

---

## (d) Does agtrace already link a Claude Code session to the Codex session it launched?

Definition for this section:
- **Decision: go** = agtrace does NOT already link a Claude session to the Codex session it
  launched (Claude → Codex hand-off), so multi-link's differentiator holds.
- **Decision: no-go** = it does.

- Repo: https://github.com/lanegrid/agtrace
- Commit read: `7649b7ec861d3d199985bbc5fb6d25c2009e0048` (HEAD of `main`, committed 2026-10-29;
  workspace version 0.8.0)
- License: `gh api` reports Apache-2.0; the repo ships both `LICENSE-APACHE` and `LICENSE-MIT`.

Commands used:

```
gh api repos/lanegrid/agtrace/commits/HEAD --jq '.sha+" "+.commit.committer.date'
gh api repos/lanegrid/agtrace --jq '.license.spdx_id, .default_branch'
gh api "repos/lanegrid/agtrace/git/trees/$SHA?recursive=1" --jq '.tree[] | select(.type=="blob") | .path'
# keyword sweep over every .rs file under crates/agtrace-{providers,engine,index,runtime,core}/src:
#   contents helper | grep -n -i -E 'codex exec|"codex"|originator|cross.?provider|cross.?harness|
#                                   launched|external.?session|linked_session|child_session|parent_session'
# then targeted reads (grep / sed -n) of the files below; and `grep -n -i codex` over the Claude
# provider files listed below.
```

Files relied on:
- Where transcripts are read: `crates/agtrace-providers/src/claude/` (`decoder.rs`, `header.rs`,
  `discovery.rs`, `tools.rs`, `tool_mapping.rs`, `mapper.rs`) and
  `crates/agtrace-providers/src/codex/` (`header.rs`, `records.rs`, `collab.rs`, `exec.rs`,
  `discovery.rs`); roots in `crates/agtrace-core/src/roots.rs`.
- Where sessions are related: `crates/agtrace-runtime/src/ops/index.rs` (fills
  `parent_session_id` / `root_session_id`), `crates/agtrace-index/src/schema.rs`,
  `crates/agtrace-index/src/queries/session.rs` (`get_children`),
  `crates/agtrace-index/src/db.rs` (`get_child_sessions`),
  `crates/agtrace-runtime/src/client/sessions.rs`, `crates/agtrace-engine/src/workspace/parent.rs`.
- `README.md` (feature claims).

### Findings

**What agtrace links (all within one harness):**
1. **Codex → Codex child threads.** `codex/header.rs` reads `session_meta.parent_thread_id` or
   `source.subagent.thread_spawn.parent_thread_id` (plus `agent_path`, `agent_nickname`) into
   `agent.parent` via `AgentId::codex_thread`. `codex/collab.rs` handles Codex multi_agent v2
   `spawn_agent` / `send_message` addressing (`/root/judge`).
2. **Claude → Claude subagents, forks and teammates.** `engine/workspace/parent.rs`
   (`teammate_parent`, `team_lead_agent`, `RuntimeAliases`, `teammate_spawns`) puts a teammate under
   the agent whose log holds its `AgentSpawn`, else under the team lead (`leadSessionId`).
3. **The index.** In `runtime/src/ops/index.rs`, `parent_session_id` is set by provider:
   `ProviderId::Codex` → the Codex parent thread's session id; `ProviderId::ClaudeCode` → the team
   lead. There is no third branch and no case that crosses providers.
4. **Project grouping, not session linking.** `index/src/schema.rs` notes a repository hash that
   "allows cross-provider session grouping": sessions of both tools in the same repo are listed
   together. That says "same project", not "this Claude session launched that Codex run".

**What it does not link:** a Claude Code session to the Codex session it launched (e.g. a Bash
tool call running `codex exec`, or a Codex MCP call). Evidence:
- The keyword sweep found no `codex exec`, `originator`, or cross-provider/session-link code in
  any of the five library crates. The only `codex` hits are provider registration
  (`registry.rs`, `traits.rs`), log roots, and tests.
- `grep -i codex` over the Claude provider files (`decoder.rs`, `tools.rs`, `tool_mapping.rs`,
  `mapper.rs`, `discovery.rs`, `header.rs`) and `tool_analyzer.rs` returned nothing: the Claude
  side never looks for Codex in a command or tool result.
- `codex/exec.rs` is about Codex's *own* `exec` tool (a JS program inside a Codex session calling
  `tools.exec_command`), not the `codex exec` CLI launched from Claude.
- The README describes the tree as "the lead, its teammates, background subagents, forks, and
  Codex child threads"; it claims no Claude→Codex linking.

**Confidence:** high for this commit. The stored relation is one column (`parent_session_id`)
whose only writer branches on provider as described. The live tree also assigns parents in memory
(`workspace/view.rs::link`, found in Codex review), but it is fed only by each provider's own spawn
evidence (Claude teammates/subagents, Codex `thread_spawn`); neither provider emits a cross-provider
spawn target, so no Claude→Codex edge can form there either. Limits: this is from
reading source, not running it, and grep can miss a link built under a name we did not search for
(the parent derivation code was read directly to reduce that risk). agtrace moves fast (HEAD was
committed today), so re-check before publishing a comparison.

**Run on the 0.2a sandbox sessions** (owner approved, 2026-10-29, after the source read): built
`agtrace` at 7649b7e outside the repo; ran with `HOME` and `AGTRACE_PATH` set to a throwaway folder
holding copies of only the sandbox transcripts (`agtrace init --all-projects`, then
`agtrace session list --all-projects --format json`). Its index held 9 `claude_code` and 10 `codex`
sessions; `select count(*) from sessions where parent_session_id is not null` → 0. It grouped
sessions by checkout (`project_root`) but linked no Claude session to its Codex child, matching the
source reading. The original list of what a run would add, kept for reference:
- Confirm end to end that `agtrace session show` / the agent tree / MCP `get_agent_tree` show the
  Claude session and the Codex run it launched as separate, unlinked roots (expected from source).
- Confirm it parses the owner's real Claude Code and Codex versions at all (README: Claude Code
  ≥ 2.1.24x, Codex ≥ 0.153), checked against `fixtures/raw/ground-truth.md`.
- Show whether the repository-hash grouping puts both sessions under the same project, which is the
  closest thing it has to a link and the baseline multi-link is compared against.
- If ever run: point it at a throwaway tree with `AGTRACE_CLAUDE_HOME` / `AGTRACE_CODEX_HOME`
  (`agtrace-core/src/roots.rs`), and check where its SQLite index is written and whether `init`
  touches `~/.claude` / `~/.codex` before doing so.

Decision: go
