# Phase 0 — hand-offs and state signals

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
