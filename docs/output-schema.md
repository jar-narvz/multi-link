# Output schema — `multi-link sessions | links | steps --json`

Binding contract for Phases 1 and 2. `docs/design.md` → Data model is illustrative only; this file wins.
Hand-written expected outputs: `fixtures/expected/{sessions,links,steps}.json` and the derived variants
`fixtures/expected/links-amb-hidden.json` (Links → Ambiguous variant) and
`fixtures/expected/links-tagged.json` (Links → Tagged variant); Phase 2 hook answer keys in
`fixtures/expected/hook/` (Hook records → Hook variant).

**Phase 2 changes (Step 2.2):** Link gains `step` (Step tags); Link `evidence` gains `"hook"`;
`state_evidence` gains `"hook"` (its State row is defined in Step 2.4); Segment `step_source` loses `"hook"`.

**Phase 2 changes (Step 2.4):** new section "Hook records" (file layout, record fields, reader rules,
Hook variant); State gains two `waiting` rows, session record first (design H11), hook second (H8);
a `waiting` session is always listed; hook lines appear in `sessions` → `skipped`.

Transcript formats are undocumented. Every field path below was observed in the Phase 0 fixtures
(`docs/phase0.md`), not in any spec.

## Common rules

### Flags
| Flag | Meaning |
| --- | --- |
| `--root <dir>` | Home-shaped root. Reads `<dir>/.claude/projects/`, `<dir>/.claude/sessions/`, `<dir>/.codex/sessions/`. Default: `$HOME`. When given (fixture mode): no process probing, no git lookups, task outputs come from `<dir>/task-output/` (see Links), hook records from `<dir>/.multi-link/hook/` (see Hook records). |
| `--now <RFC 3339>` | Replaces the clock. Used by two things only: the `--since` window and the 300 s freshness check in State. |
| `--since <duration>\|all` | History window, default `2h`: a transcript session is listed if its `last_activity` ≥ now − window. Session-record sessions and `waiting` sessions are always listed (active discovery, FR1; State → Visibility). `all` disables the window. The window filters **output only**: Phase 1 parses every transcript under the root, so exact evidence from old files still excludes heuristic candidates. A link is listed when its `parent` or `child` is a listed session; `unparented` lists only listed sessions. |
| `--json` | Print the JSON below (pretty-printed; tests compare parsed JSON, not bytes). |
| `--repo <dir>` (`steps` only, repeatable) | Repos to read. Relative paths resolve against `--root`. Default: the `repo` of every listed session. |

**Phase 1 Verify commands** (each must equal its expected file as parsed JSON):
```
multi-link sessions --root fixtures/sessions --now 2026-10-29T23:15:00Z --since all --json
multi-link links    --root fixtures/sessions --now 2026-10-29T23:15:00Z --since all --json
multi-link steps    --root fixtures/sessions --repo repos/repo-a --repo repos/repo-b --json
```
`--since all` is needed because the organic and malformed fixtures are older than 2 h at that `--now`.

### Values
- **Ids:** always full ids, exactly as in the source (Claude: transcript file stem / `sessionId`;
  Codex: `session_meta.payload.id`; tool calls: `tool_use.id`; subagents: `agentId`). Prefixes are a display concern.
- **Timestamps:** RFC 3339 UTC with exactly 3 fractional digits, e.g. `2026-10-29T00:00:00.000Z`.
  Epoch-ms fields (session records) are converted to the same form.
- **Paths:** `transcript` / `record` are relative to the root. `cwd`, `checkout`, `repo` are the
  strings recorded in the source (scrubbed fixtures use `/home/dev/...`), never re-rooted.
- **Order:** sessions by (`started_at`, `id`); links by (`at`, `invocation`); skipped by (`file`, `line`);
  id lists ascending.
  String order of the timestamp format above equals time order.
- Every listed field is always present; "none" is `null` or `[]`, never an omitted key.

## `sessions`

```json
{
  "now": "2026-10-29T23:15:00.000Z",
  "sessions": [ Session, ... ],
  "skipped": [ {"file": "<path relative to root>", "line": 3, "reason": "invalid_json"}, ... ]
}
```

### Session
| Field | Type | Rule |
| --- | --- | --- |
| `id` | string | Claude: transcript file stem (what `claude --resume` takes), or `sessionId` of a record-only session. Codex: `session_meta.payload.id`. |
| `harness` | `"claude"` \| `"codex"` | |
| `transcript` | string \| null | Main transcript path. `null` for a record-only session. |
| `record` | string \| null | `.claude/sessions/<pid>.json` whose `sessionId` equals `id`, else `null`. |
| `cwd` | string | Claude: `cwd` of the last record in file order that has one (a resumed file can carry earlier records with another `sessionId` and other cwds). Record-only: record `cwd`. Codex: `session_meta.payload.cwd`. |
| `repo` | string | Git root of `cwd`. In fixture mode (`--root`) git lookups are off, so `repo` = `cwd`; the worktree `repo-a-wt` is therefore its own repo in the fixtures. Git-root and worktree grouping is a Phase 1 unit test on a temp repo. |
| `branch` | string \| null | Claude: last `gitBranch`. Codex: `session_meta.payload.git.branch`. Record-only: `null`. |
| `started_at` | timestamp | Claude: first top-level `timestamp` in file order (any record, including inherited ones). Codex: `session_meta.payload.timestamp`, else first record `timestamp`. Record-only: `startedAt`. |
| `last_activity` | timestamp | Last top-level `timestamp` in file order (main transcript only). Record-only: `updatedAt`. |
| `state` | enum | See State. |
| `state_evidence` | enum | See State. |
| `segments` | Segment[] | See Segments. `[]` for record-only sessions. |
| `subagents` | string[] | Claude: `agentId`s of `<id>/subagents/agent-<agentId>.jsonl` files. A subagent is part of its parent session, never a session of its own. Codex: `[]`. |

### State
`state` ∈ `busy | waiting | idle | finished | stale | unknown`. `waiting` means Claude is blocked on the
user (a permission prompt, H3). Transcripts carry no waiting signal (Phase 0 (b)); Step 2.1 found that the
live session record does (`status: "waiting"` + `waitingFor`, design H11), and the hook adds a fallback
for sessions without a record (H8). Codex waiting is unsupported.

`state_evidence` ∈ `process_exit | session_record | open_turn | turns_complete | no_signal | hook`.

Rows are checked top to bottom; the first match wins. If the process probe cannot run (fixture mode,
`ps` denied, permission error) the record is trusted as written and the `finished` row is skipped.

| Situation | `state` | `state_evidence` |
| --- | --- | --- |
| Claude session record whose process is confirmed gone: `ps` shows no `pid`, or the `pid` has a different start time than `procStart` (pid reused). Checked first. | `finished` | `process_exit` |
| Claude session record, `status=waiting` (any age: a prompt can wait overnight; `waitingFor` is not read, its only observed value is `"permission prompt"`). Design H11. | `waiting` | `session_record` |
| Claude session record, `status=busy`, now − `statusUpdatedAt` ≤ 300 s | `busy` | `session_record` |
| Claude session record, `status=busy`, older than 300 s | `stale` | `session_record` |
| Claude session record, `status=idle` (any age; freshness applies to `busy` only, Step 0.2 review) | `idle` | `session_record` |
| Claude session record with any other `status` value | `unknown` | `session_record` |
| Claude transcript, no session record, and the session's latest hook record is an uncleared `waiting` (Hook records → Waiting). Design H8: after `finished` (a record session never reaches this row, so the record's own status always wins), before the transcript-only fallback. | `waiting` | `hook` |
| Claude transcript, no session record | `unknown` | `no_signal` |
| Codex, last `task_started` turn has no `task_complete`, `last_activity` ≤ 300 s ago | `busy` | `open_turn` |
| Codex, open turn, older than 300 s | `stale` | `open_turn` |
| Codex, last `task_started` turn has its `task_complete` (the latest turn governs; an earlier unmatched turn was interrupted and is ignored) | `unknown` | `turns_complete` |
| Codex, no `task_started` in the file | `unknown` | `no_signal` |

Why `unknown` and not `finished` for transcript-only sessions: Phase 0 found no exit evidence in
transcripts. A missing session record proves nothing (`claude -p` may never write one), and a completed
Codex turn ends the turn, not the process. Showing `finished` would be a guess.

In fixture mode the session records are trusted as written (process probing is off; their pids belong
to another machine). With `--now 2026-10-29T23:15:00Z` the one busy record (`statusUpdatedAt`
23:12:39.714Z, age 140 s) is `busy`; the four idle records are `idle`; all transcript sessions are
`unknown`. `stale` and `finished` have no fixture: Phase 1 covers them with state-machine unit tests
(busy record at now + 301 s → `stale`; open Codex turn; process gone → `finished`). `waiting` has no row
in the plain fixtures; the Hook variant has one of each evidence (Hook records → Hook variant).

**Visibility:** a session whose `state` is `waiting` is always listed, whatever `--since` says (record
sessions already are; this keeps a transcript-only session that waits overnight on screen, Codex P2 #7).

### Segments
`{"from": timestamp, "to": timestamp, "step": string|null, "step_source": "unknown"}`

A step comes only from explicit evidence (FR7). Phase 2 defines the tag (Links → Step tags) but puts it
on the link only (design H7): every transcript session still has exactly one segment
`[started_at, last_activity]` with `step: null`, `step_source: "unknown"`. Splitting sessions into
per-hand-off segments is deferred to Phase 3; `step_source` value `prompt_tag` stays reserved for it.
`hook` is no longer a `step_source` value: the hook records no step of its own (a tag comes from the
transcript command, H6), so `step_source` ∈ `unknown | prompt_tag`.

### Skipped
One entry per line not used, `reason` ∈:
- `invalid_json` — non-empty, newline-terminated line that does not parse (truncated record).
  An unterminated last line is a record still being written: it is buffered for the next read
  (design: incremental reads) and never reported as skipped. The fixture's bad lines are newline-terminated.
- `empty_line` — blank line.
- `unknown_type` — valid JSON whose top-level `type` is not known for that harness.

Known Claude types (all seen in fixtures): `agent-name`, `api-request`, `api-request-blob`,
`api-request-shape`, `assistant`, `atis-latch`, `attachment`, `cost-state`, `custom-title`,
`file-history-delta`, `file-history-snapshot`, `last-prompt`, `mode`, `queue-operation`, `system`, `user`.
Known Codex types: `event_msg`, `response_item`, `session_meta`, `token_usage_record`, `turn_context`, `world_state`.
Subagent transcripts are scanned too. A skipped line never stops the file; later lines are still read.

## `links`

```json
{
  "links": [ Link, ... ],
  "unparented": [ "<codex id>", ... ]
}
```
`unparented`: Codex sessions that no `exact` or `heuristic` link points to (a candidate in an
`ambiguous` link is still unparented). The manual run must be here, never given a parent.

### Link — one per recognised hand-off invocation
| Field | Type | Rule |
| --- | --- | --- |
| `parent` | string | Claude session id (for a subagent call, the parent session, = the subagent records' `sessionId`). |
| `invocation` | string | `tool_use.id` of the Bash call that launched Codex. |
| `subagent` | string \| null | `agentId` when the call is in a subagent transcript (plugin form). |
| `form` | `"exec"` \| `"resume"` \| `"plugin"` | See Recognised invocations. |
| `background` | bool | `tool_use.input.run_in_background == true`. |
| `at` | timestamp | Top-level `timestamp` of the `assistant` record holding the `tool_use`. |
| `checkout` | string | `cwd` of that record. |
| `match` | `"exact"` \| `"heuristic"` \| `"ambiguous"` \| `"unlinked"` | |
| `child` | string \| null | Codex id for `exact` / `heuristic`; `null` otherwise. |
| `evidence` | `"stdout"` \| `"resume_arg"` \| `"thread_ready"` \| `"task_output"` \| `"hook"` \| null | What made it exact; `null` unless `exact`. `hook` only in Phase 2, see Exact evidence. |
| `candidates` | `{"id": string, "gap_ms": int}`[] | Heuristic candidates (1 for `heuristic`, ≥ 2 for `ambiguous`); `[]` for `exact` and `unlinked`. Ordered by `gap_ms`, then `id`. |
| `step` | string \| null | Step tag of the launch segment (Step tags), set on every link whatever its `match`. `null` when the segment has no tag. |

### Recognised invocations
Only `tool_use` blocks inside `type=assistant` records count (`api-request-blob` records repeat the
same `tool_use` ids and must be ignored). `name=Bash`; split `input.command` into segments on `&&`,
`||`, `;`, `|` and newlines outside quotes; a segment is a launch when its tokens are:
- `codex exec resume <uuid>` or `codex resume <uuid>` → `resume`;
- `codex exec …` (anything else) → `exec`;
- `node <path ending in codex-companion.mjs> task …` → `plugin` (exactly `task`; `task-resume-candidate` is not a launch).

Anything else is not a hand-off and produces no link (`codex --version`, `ls ~/.codex/sessions`).
One launch per `tool_use`. Env-var prefixes (`FOO=1 codex exec`) are not recognised in Phase 1.

### Exact evidence, checked in this order
1. `resume` with a `<uuid>` argument → `child` = argument, `evidence: "resume_arg"`. The output repeats the
   id; Phase 0 confirmed they agree. A resume is another hand-off on an existing child, so one Codex
   session can have several links.
2. Tool result (matched by `tool_use_id`) content has a line `session id: <uuid>` → `"stdout"`.
3. Tool result content has `[codex] Thread ready (<uuid>)` → `"thread_ready"` (the `Turn` uuid nearby is not it).
4. Tool result content names a task output file (`Output is being written to: <path>.output`; background
   calls and calls the user backgrounded) and that file has a `session id: <uuid>` line → `"task_output"`.
   **Lookup:** real mode reads the named path. Fixture mode reads `<root>/task-output/<file name>`
   (the named `/tmp/dev/...` path does not exist). A missing file is not an error; it just gives no
   evidence, and the link falls through to the heuristic. Temp task files vanish, so real-mode
   background links will often end up heuristic — by design, never guessed exact.
5. *(Phase 2)* A hook record for this invocation (joined on `tool_use_id` = `invocation`) names a child
   the hook observed → `"hook"` (design H5). Checked only after sources 1–4 found nothing; when a
   transcript source exists and the hook names a different child, the transcript wins and the conflict
   is counted, never shown as the link. A hook record with no child gives no evidence. Record format,
   join and conflict rules: Hook records → Links. Without hook records (Phase 1, or the hook not
   installed) this source never fires.

Every `exact` link, whatever its evidence (`hook` included), claims its child for the heuristic
exclusion below unless its `form` is `resume`.

### Heuristic (no exact evidence)
**Launch directory must be certain.** If the launch segment follows a `cd` in the same command, or the
`codex` call has `-C` / `--cd`, the child's directory is not `checkout`: no heuristic runs and the link
is `unlinked` (no fixture has this; Phase 1 unit test).

Candidates: Codex sessions with `cwd` == `checkout` (exact string) and `started_at` in
`[at, at + 60 s]`, excluding sessions that are the child of another invocation's `exact` link with
`form` ≠ `resume` (that invocation created them). `gap_ms` = `started_at − at`. 0 → `unlinked`; 1 → `heuristic`; ≥ 2 → `ambiguous`.
`gap_ms` is a ranking value, not a probability.

### Step tags (Phase 2, design H6)
`step` is the first step tag in the launch segment's text (the segment as split above, quotes and all;
other segments of the same command are not searched). A tag is `step` in any letter case, at the start
of the segment or after a character that is not a letter, digit or `_`, then one or more spaces or tabs,
then an `<id>` in the PLAN.md step-id grammar (`steps` → Step `id`: digits, optionally `.` digits,
optionally one lowercase letter), ending at the end of the text or before a character that is not a
letter, digit or `_` and is not a `.` followed by a digit. As a regex (case-insensitivity on `step` only):
`(?:^|[^A-Za-z0-9_])[Ss][Tt][Ee][Pp][ \t]+([0-9]+(?:\.[0-9]+)?[a-z]?)(?![A-Za-z0-9_]|\.[0-9])`
(a specification in PCRE style; Rust's `regex` crate has no lookahead, so code may check the boundaries by hand).

| Segment text contains | `step` |
| --- | --- |
| `Read AGENTS.md, then do Step 1.2 in PLAN.md.` | `"1.2"` |
| `step 4`, `Step 0.2a`, `Step 0.10` | `"4"`, `"0.2a"`, `"0.10"` |
| `Steps 1`, `step1.2`, `Step 1.2.3`, `Step 1.2ab` | `null` |
| `Step 2.1 … Step 2.2` | `"2.1"` (the first) |

The tag is read from the command actually run (the transcript's `tool_use.input.command`), not from a
prompt Claude was given: the Codex plugin's subagent rewrites its prompt, and a rewrite that drops the
tag gives `null`, never a guess. A prompt passed indirectly (`"$(cat prompt.txt)"`, a heredoc on another
line, a file) has no tag in the segment, so `step: null`. The hand-off convention (CLAUDE.md) starts each
prompt with `Read AGENTS.md, then do Step <id> in PLAN.md.` Tags come from transcripts, so they work
without the hook and on history. The fixtures' prompts are placeholders (`CASE-<name>: <prompt>`), so
every link in `links.json` and `links-amb-hidden.json` has `step: null`.

### Fixture results
- 8 sandbox hand-offs, all `exact` (ground truth rows exec, resume, plugin, bg, par-main, par-wt, amb-1, amb-2):
  exec and resume point at the same child; par-main and par-wt are distinct links in distinct checkouts;
  plugin is found through subagent `aee8ad2d0134ca48a`; bg through `task-output/be01y5dv6.output`.
- `mention` has no link. `manual` and the synthetic Codex file are `unparented`.
- 5 organic hand-offs (repo-c) are `unlinked`: no `session id` in their results, their task-output
  files were not kept, and no Codex fixture has cwd `/home/dev/repo-c`. Not in the ground truth.

### Ambiguous variant (`fixtures/expected/links-amb-hidden.json`)
The amb pair is exact in the fixtures because both results show the child id. Phase 1 builds the
ambiguous case in a test: copy `fixtures/sessions` to a temp dir and, in
`11bf1b58-….jsonl` line 28 and `f4d83303-….jsonl` line 27, replace the `session id: <uuid>` line with
`<stdout>` in both `message.content[].content` and `toolUseResult.stdout`. `links` on that copy must equal
`links-amb-hidden.json`: amb-1 → `ambiguous` (candidates d501 at 746 ms, d953 at 1822 ms); amb-2 →
`heuristic` d953 (447 ms; d501 started before amb-2's call, so it is outside the window); d501 joins
`unparented`. Everything else is unchanged.

### Tagged variant (`fixtures/expected/links-tagged.json`)
Built in a test like the Ambiguous variant: copy `fixtures/sessions` to a temp dir and, in the exec case
(`bb792d9c-….jsonl`, the `type=assistant` record holding `tool_use` `toolu_01EGmZWsd8PbTTNmjK67yApw`),
change only that `tool_use`'s `input.command` from
`codex exec -s workspace-write --skip-git-repo-check "CASE-exec: <prompt>" | tail -n 1` to
`codex exec -s workspace-write --skip-git-repo-check "Read AGENTS.md, then do Step 1.2 in PLAN.md. CASE-exec: <prompt>" | tail -n 1`.
Tool results and every other record (including any `api-request-blob` copy) stay unchanged. `links` on
that copy must equal `links-tagged.json`: the exec link has `step: "1.2"`; every other field and link equals
`links.json` (the resume link to the same child keeps `step: null`, since its own command has no tag).

## `steps`

```json
{
  "repos": [
    {
      "repo": "repos/repo-a",
      "convention": "supported",
      "latest_logged_step": "0.4",
      "steps": [ Step, ... ]
    }
  ]
}
```
| Field | Type | Rule |
| --- | --- | --- |
| `repo` | string | As given to `--repo` (or the session `repo`). Repos in that order. |
| `convention` | `"supported"` \| `"missing"` \| `"unsupported"` | `missing`: no `PLAN.md`. `unsupported`: `PLAN.md` has no `### Step <id> — <title>` heading. Then `steps: []`, `latest_logged_step: null`. |
| `latest_logged_step` | string \| null | Step id of the first (newest) PROGRESS.md entry. |
| `steps` | Step[] | PLAN.md order. |

### Step
| Field | Type | Rule |
| --- | --- | --- |
| `id` | string | `N` or `N.M`, optionally followed by one lowercase letter (`0.2a`, `3b`; design decision P11), from `### Step <id> — <title>`; a string so `0.10` ≠ `0.1`. No fixture uses a suffix; Phase 1 covers it with a unit test. |
| `title` | string | Heading text after ` — `. |
| `verify_command` | string \| null | Inside the step: first `- Verify:` or `- **Verify:**` line; the text in its first backticks. |
| `progress` | Entry[] | PROGRESS.md entries for this id, file order (newest first). |

Entry: `{"date": "YYYY-MM-DD", "agent": string|null, "verify": {"command": string|null, "result": "pass"|"fail"|"unknown", "result_text": string, "source": "reported"}}`
from heading `### YYYY-MM-DD — Step <id>: <title>[ — <agent>]` and the entry's first Verify line:
`command` = first backticked text; `result_text` = text after `→` on that line, trimmed (`""` if none);
`result` = `pass` if `result_text` starts with `pass`/`ok` (any case), `fail` if it starts with `fail`,
else `unknown`. `source` is always `reported` in Phase 1; `observed` (Verify seen run in a transcript),
staleness and checkout are deferred. Fixture entries say "not run …" → `unknown`; `pass`/`fail` are
Phase 1 unit tests.

Steps never come from session prompts in Phase 1 (see Segments), so `steps` lists no sessions.

## Hook records

Phase 2's optional Claude Code hook (design H1–H11) writes these files; `sessions` and `links` read them.
They are multi-link's own format, so unlike transcripts this section is a spec, not an observation. The
payload fields the hook reads are the ones Step 2.1 saw on real payloads (`docs/phase2-hook.md`).

### File layout
- One file per Claude session: `<state>/hook/<session_id>.jsonl` (H10). `<state>` is `$MULTI_LINK_HOME`
  when set, else `$HOME/.multi-link`. Fixture mode (`--root <dir>`): `<dir>/.multi-link`, and the
  environment is ignored. A missing folder or file means "no hook records", never an error.
- The reader takes only files whose name is `<uuid>.jsonl` (lowercase 8-4-4-4-12 hex, the shape the hook
  writes); other files are ignored and not listed.
- One JSON object per line, newline-terminated, appended with one `write()` each (`O_APPEND`, under
  4 KiB). Line order is append order; parallel hook processes may interleave, so readers order records
  by `at`, then line number.

### Record (`v: 1`)
Every field is always present; "none" is `null`. Records hold ids, times and enum values only: no command
text, prompt, tool output or message.

| Field | Type | Rule |
| --- | --- | --- |
| `v` | `1` | Format version. |
| `kind` | `"handoff"` \| `"waiting"` | |
| `at` | timestamp | The hook's own clock when the record is written (payloads carry no event time, 2.1 (h)); RFC 3339 UTC, 3 fractional digits. A first or only record is written on receipt, so `at` is the receive time and trails the event by about 0.1–0.2 s (async hook). The appended background child record (below) has the time the child was found, later than its first record's `at`. |
| `session_id` | string | Payload `session_id` (= transcript stem, 2.1 (f)); equals the file stem. For a subagent call it is the **parent's** id. |
| `agent_id` | string \| null | Payload `agent_id` (subagent calls only, e.g. `aee8ad2d0134ca48a` for the Codex plugin's `codex:codex-rescue` subagent; 17 chars in 2.1). Informational: the join below does not use it. |
| `event` | string | Source hook event: `PostToolUse` or `PostToolUseFailure` for `handoff`; `PermissionRequest` or `Notification` for `waiting`. |
| `tool_use_id` | string \| null | Payload `tool_use_id` (= the transcript's `tool_use.id`, 2.1 (f)) for `handoff`; `null` for `waiting` (`PermissionRequest` / `Notification` have none). |
| `cwd` | string | Payload `cwd`. Informational. |
| `child` | string \| null | `handoff` only: the Codex session id the hook observed (lowercase uuid), else `null`. Always `null` for `waiting`. |
| `child_evidence` | `"stdout"` \| `"thread_ready"` \| `"task_output"` \| null | Where `child` was seen; `null` exactly when `child` is `null`. `stdout`: a `session id: <uuid>` line in `tool_response.stdout`, or for `PostToolUseFailure` in `error` (2.1: a failed run prints its id there; `event` tells the two apart). `thread_ready`: `[codex] Thread ready (<uuid>)` in that text (plugin). `task_output`: a `session id:` line in the background task's output file, at `dirname(scratchpad_dir)/tasks/<tool_response.backgroundTaskId>.output` (2.1 (g)). |
| `notification_type` | string \| null | `Notification` only: `"permission_prompt"` or `"elicitation_dialog"`; `null` for every other event. |

**What the hook writes (Step 2.5):**
- `PostToolUse` / `PostToolUseFailure` with `tool_name` `Bash` whose command has a recognised launch
  segment (Links → Recognised invocations) → `handoff`. Child from the output text first. A background
  call (`tool_input.run_in_background: true`) with no id in its output writes a first record with
  `child: null` **at once**, then polls the task output file and appends a second record with the child
  if one appears. Reason: `-p` kills unfinished async hooks at teardown (2.1 (k)), so a hook that only
  wrote after polling would lose the hand-off. At most two records per `tool_use_id`. If `scratchpad_dir`
  or `backgroundTaskId` is missing, the first record is the only one. A `resume` launch's child is read
  from its output like any other (its `resume_arg` evidence comes from the transcript, source 1).
- `PermissionRequest` (any tool) → `waiting`, `notification_type: null`. It fires in `-p` and
  interactively (2.1 (h)).
- `Notification` with `notification_type` `permission_prompt` or `elicitation_dialog` → `waiting`
  (interactive only; about 6 s after `PermissionRequest` for the same prompt, a harmless duplicate).
- Anything else (other events or notification types, e.g. `idle_prompt`; non-launch Bash; invalid
  payloads) → nothing is written.

### Reader rules
- Lines go to `sessions` → `skipped` like transcript lines, with `file` relative to the root when the
  state folder is inside it (always in fixture mode: `.multi-link/hook/<id>.jsonl`), else absolute.
  `links` has no `skipped` list; it uses the same valid records.
- Blank line → `empty_line`. Non-empty, newline-terminated line that is not a JSON object →
  `invalid_json`. An unterminated last line is buffered, not skipped (as for transcripts).
- Valid JSON that is not a `v: 1` record this reader understands → `unknown_type`: `v` ≠ `1` or missing,
  `kind` not `handoff` / `waiting`, a missing field, a wrong type, an `event` / `child_evidence` /
  `notification_type` value not allowed for its `kind`, `child` and `child_evidence` not both null or
  both set, or `session_id` ≠ the file stem. Never a crash; later lines are still read.
- Hook records never create a session. Records whose `session_id` is not a Claude session found from a
  transcript or session record (a Codex id, an unknown id) are ignored.

### Links: join and conflicts (Exact evidence source 5)
- A `handoff` record applies to the link with `parent` = `session_id` and `invocation` = `tool_use_id`.
  No such link → ignored.
- The hook child of a link is the `child` of its records with a non-null child. None → no hook evidence
  (a `child: null` record never changes a link). Two records naming different children → no hook
  evidence, one conflict counted.
- Sources 1–4 found a child: the link keeps it and its evidence. A hook child that differs → one conflict
  counted, the link unchanged. A hook child that agrees changes nothing.
- Sources 1–4 found nothing and there is a hook child → `match: "exact"`, `child` = hook child,
  `evidence: "hook"`, `candidates: []`. Like every non-resume exact link, it claims its child for the
  heuristic exclusion.
- `hook_conflicts` (count of links with a conflict) is a diagnostic, printed on stderr with the others
  (Step 2.7); it is not part of the JSON.

### Waiting (State row `waiting` / `hook`)
- Only for a Claude session with a transcript and **no** session record: when a record exists, its
  `status` decides (design H11; the record was seen switching to `waiting` and back, 2.1 (i)).
- The session's latest valid hook record (by `at`, ties by line number) must be `kind: "waiting"`, and
  its `at` must be later than every top-level `timestamp` in the session's main transcript **and** its
  subagent transcripts. Clearing signal (2.1 (i)): Claude writes nothing during a held prompt, and the
  first line after it (the `tool_result` on answer or denial, or the interrupt line) clears it; a later
  `handoff` record clears it too. Compare line timestamps, never file size or mtime (transcript writes
  are buffered, 2.1 (i)).
- No freshness limit: a prompt can wait overnight. A transcript-only session has no process evidence,
  so a crashed session stays `waiting` until a line clears it; a `-p` denial fires `PermissionRequest`
  after the denial line (2.1 (h)), so it shows `waiting` until the next line (about 1 s).

### Hook variant (`fixtures/expected/hook/`)
Built in a test like the Ambiguous variant: copy `fixtures/sessions` to a temp dir, copy the overlay
`fixtures/hook/` on top of it (`fixtures/hook/.multi-link/hook/*.jsonl` → `<tmp>/.multi-link/hook/`,
`fixtures/hook/.claude/sessions/98798.json` → `<tmp>/.claude/sessions/`), and delete
`<tmp>/task-output/be01y5dv6.output`. Hook records are synthetic (Step 2.1 kept no full payloads); they
use existing fixture ids. `98798.json` is the scrubbed 2.1 perm-i session record snapshot.
Commands (`--root <tmp> --now 2026-10-29T23:15:00Z --json`):

| Command | Expected file |
| --- | --- |
| `links --since all` | `hook/links.json` |
| `sessions --since all` | `hook/sessions.json` |
| `sessions` (default `2h` window) | `hook/sessions-2h.json` |

| Hook file (`.multi-link/hook/`) | Records | Expected effect |
| --- | --- | --- |
| `07c80c2e-….jsonl` (bg) | `handoff` `child: null`, then `child` `01a189e3-f419-…` / `task_output` | bg link `exact` / `hook`, same child as `links.json` (its task output file is deleted, so sources 1–4 find nothing; without the hook it would be `heuristic`) |
| `85a715ba-….jsonl` (repo-c) | `handoff` `child: null` for `toolu_01DYjrRidN29nXMRN9Rci1uz` | stays `unlinked` |
| `bb792d9c-….jsonl` (exec) | `handoff` naming `01a189e5-5b32-…` (the manual run) | conflict: exec link keeps `stdout` / `01a189e2-d57f-…`; manual stays `unparented`; `hook_conflicts` = 1 |
| `9d9a0ed8-….jsonl` (plugin) | `handoff` with `agent_id` `aee8ad2d0134ca48a`, same child as the transcript (`thread_ready`) | agrees: no change, no conflict |
| `11c1f13c-….jsonl` (repo-c, last line 2026-10-28T10:03:04.840Z) | `waiting` `PermissionRequest` 10:03:05.020Z, then `Notification` `permission_prompt` 10:03:11.004Z | `waiting` / `hook`; listed in `sessions-2h` although its last line is ~37 h old |
| `0d7faed4-….jsonl` (repo-d, last line 2026-10-27T10:04:27.890Z) | line 1 `waiting` at 10:04:20.150Z; line 2 truncated JSON; line 3 a `v: 2` `waiting` at 10:05:00.000Z | cleared: stays `unknown` / `no_signal` and out of `sessions-2h`; skipped line 2 `invalid_json`, line 3 `unknown_type` (a reader that accepted either would show `waiting`) |
| `d1f3b188-….jsonl` (record session) | `waiting` `PermissionRequest` and `Notification` | the record decides: `waiting` / `session_record`, not `hook` |
| `1f47e9b3-….jsonl` (record session `64044.json`, `status: idle`, `updatedAt` 2026-10-29T20:44:16.946Z) | `waiting` `PermissionRequest` at 20:45:02.318Z, after every record time | the record disagrees and wins: stays `idle` / `session_record` (a reader letting an uncleared hook `waiting` override a record would show `waiting`) |

`.claude/sessions/98798.json` (`status: "waiting"`, `waitingFor: "permission prompt"`) adds the
record-only session `d1f3b188-efd4-469b-8cce-7caec214cc3a` (cwd `/home/dev/multi-link-sandbox/repo-a`,
`started_at` 2026-10-30T06:08:09.990Z, `last_activity` 06:08:12.305Z, both after `--now`; ids and epoch
values are kept from the raw snapshot): `waiting` / `session_record`.

Resulting answer keys, each otherwise equal to its Phase 1 file: `hook/links.json` = `links.json` with
the bg link's `evidence` `"hook"`. `hook/sessions.json` = `sessions.json` with 11c1f13c `waiting` /
`hook`, the d1f3b188 row appended (latest `started_at`), and the two `0d7faed4` skipped lines appended
(`.multi-link/` sorts after `.claude/` and `.codex/`). `hook/sessions-2h.json` = `hook/sessions.json`
without the four sessions whose `last_activity` is before 2026-10-29T21:15:00Z and that are neither
record sessions nor `waiting` (0d7faed4, 85a715ba, the synthetic `…000e` and `…000f`); `skipped` is not
windowed. Exactly one session is `waiting` / `hook` (11c1f13c) and one `waiting` / `session_record`
(d1f3b188) in both session files.
