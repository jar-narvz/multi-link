# Output schema — `multi-link sessions | links | steps --json`

Binding contract for Phase 1. `docs/design.md` → Data model is illustrative only; this file wins.
Hand-written expected outputs: `fixtures/expected/{sessions,links,steps}.json` and
`fixtures/expected/links-amb-hidden.json` (derived variant, see Links → Ambiguous variant).

Transcript formats are undocumented. Every field path below was observed in the Phase 0 fixtures
(`docs/phase0.md`), not in any spec.

## Common rules

### Flags
| Flag | Meaning |
| --- | --- |
| `--root <dir>` | Home-shaped root. Reads `<dir>/.claude/projects/`, `<dir>/.claude/sessions/`, `<dir>/.codex/sessions/`. Default: `$HOME`. When given (fixture mode): no process probing, no git lookups, task outputs come from `<dir>/task-output/` (see Links). |
| `--now <RFC 3339>` | Replaces the clock. Used by two things only: the `--since` window and the 300 s freshness check in State. |
| `--since <duration>\|all` | History window, default `2h`: a transcript session is listed if its `last_activity` ≥ now − window. Session-record sessions are always listed (active discovery, FR1). `all` disables the window. The window filters **output only**: Phase 1 parses every transcript under the root, so exact evidence from old files still excludes heuristic candidates. A link is listed when its `parent` or `child` is a listed session; `unparented` lists only listed sessions. |
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
`state` ∈ `busy | idle | finished | stale | unknown`. `waiting` is reserved for the Phase 2 hook;
Phase 1 never emits it (Phase 0 (b): no waiting signal in transcripts).

Rows are checked top to bottom; the first match wins. If the process probe cannot run (fixture mode,
`ps` denied, permission error) the record is trusted as written and the `finished` row is skipped.

| Situation | `state` | `state_evidence` |
| --- | --- | --- |
| Claude session record whose process is confirmed gone: `ps` shows no `pid`, or the `pid` has a different start time than `procStart` (pid reused). Checked first. | `finished` | `process_exit` |
| Claude session record, `status=busy`, now − `statusUpdatedAt` ≤ 300 s | `busy` | `session_record` |
| Claude session record, `status=busy`, older than 300 s | `stale` | `session_record` |
| Claude session record, `status=idle` (any age; freshness applies to `busy` only, Step 0.2 review) | `idle` | `session_record` |
| Claude session record with any other `status` value | `unknown` | `session_record` |
| Claude transcript, no session record | `unknown` | `no_signal` |
| Codex, last `task_started` turn has no `task_complete`, `last_activity` ≤ 300 s ago | `busy` | `open_turn` |
| Codex, open turn, older than 300 s | `stale` | `open_turn` |
| Codex, every `task_started` turn has its `task_complete` | `unknown` | `turns_complete` |

Why `unknown` and not `finished` for transcript-only sessions: Phase 0 found no exit evidence in
transcripts. A missing session record proves nothing (`claude -p` may never write one), and a completed
Codex turn ends the turn, not the process. Showing `finished` would be a guess.

In fixture mode the session records are trusted as written (process probing is off; their pids belong
to another machine). With `--now 2026-10-29T23:15:00Z` the one busy record (`statusUpdatedAt`
23:12:39.714Z, age 140 s) is `busy`; the four idle records are `idle`; all transcript sessions are
`unknown`. `stale` and `finished` have no fixture: Phase 1 covers them with state-machine unit tests
(busy record at now + 301 s → `stale`; open Codex turn; process gone → `finished`).

### Segments
`{"from": timestamp, "to": timestamp, "step": string|null, "step_source": "unknown"}`

A step comes only from explicit evidence (hook tag or prompt tag, FR7). The prompt-tag format is still
an open question in `docs/design.md`, so Phase 1 recognises no tags: every transcript session has
exactly one segment `[started_at, last_activity]` with `step: null`, `step_source: "unknown"`.
The fixtures' prompts are placeholders (`CASE-<name>: <prompt>`), so this is also what they support.
`step_source` values `prompt_tag` and `hook` are reserved for the step that defines tags.

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
| `evidence` | `"stdout"` \| `"resume_arg"` \| `"thread_ready"` \| `"task_output"` \| null | What made it exact; `null` unless `exact`. |
| `candidates` | `{"id": string, "gap_ms": int}`[] | Heuristic candidates (1 for `heuristic`, ≥ 2 for `ambiguous`); `[]` for `exact` and `unlinked`. Ordered by `gap_ms`, then `id`. |

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

### Heuristic (no exact evidence)
Candidates: Codex sessions with `cwd` == `checkout` (exact string) and `started_at` in
`[at, at + 60 s]`, excluding sessions that are the child of another invocation's `exact` link with
`form` ≠ `resume` (that invocation created them). `gap_ms` = `started_at − at`. 0 → `unlinked`; 1 → `heuristic`; ≥ 2 → `ambiguous`.
`gap_ms` is a ranking value, not a probability.

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
| `id` | string | `N` or `N.M` from `### Step <id> — <title>`; a string so `0.10` ≠ `0.1`. |
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
