# Output schema — `multi-link sessions | links | steps | overview | graph --json`, `open`

Binding contract for Phases 1–8. `docs/design.md` → Data model is illustrative only; this file wins.
Hand-written expected outputs: `fixtures/expected/{sessions,links,steps}.json` and the derived variants
`fixtures/expected/links-amb-hidden.json` (Links → Ambiguous variant) and
`fixtures/expected/links-tagged.json` (Links → Tagged variant); Phase 2 hook answer keys in
`fixtures/expected/hook/` (Hook records → Hook variant); Phase 3 `fixtures/expected/overview.json` and
`fixtures/expected/hook/overview.json` (Overview); Phase 4 `fixtures/expected/phase4/overview.json`
(Overview → Phase 4 fixture results); Phase 5 `fixtures/expected/phase5/{overview,links}.json` (Overview →
Phase 5 fixture results); Phase 6 `fixtures/expected/phase6/{sessions,overview}.json` (Phase 6 fixture
results); Phase 8 `fixtures/expected/phase8/graph-<id>.json` (`graph` → Phase 8 fixture results).

**Phase 2 changes (Step 2.2):** Link gains `step` (Step tags); Link `evidence` gains `"hook"`;
`state_evidence` gains `"hook"` (its State row is defined in Step 2.4); Segment `step_source` loses `"hook"`.

**Phase 2 changes (Step 2.4):** new section "Hook records" (file layout, record fields, reader rules,
Hook variant); State gains two `waiting` rows, session record first (design H11), hook second (H8);
a `waiting` session is always listed; hook lines appear in `sessions` → `skipped`.

**Phase 4 changes (Step 4.2):** MergedRow gains `harness`; the merge covers quiet top-level Codex rows
(`codex ×N`, design U2), one merged row per harness; Heuristic gains the `-C .` / `-C "$PWD"` rule (U1).
New isolated fixture root `fixtures/sessions-phase4/` with answer key `fixtures/expected/phase4/overview.json`
(Overview → Phase 4 fixture results); `fixtures/sessions` and every earlier key are untouched until Step 4.3
adds `"harness": "claude"` to the existing keys' merged rows.

**Phase 5 changes (Step 5.1, both additive, design V0/V1/V8):** SessionRow gains `made`; the overview report
gains `hook`. No existing value, key or row order changes. New isolated fixture root
`fixtures/sessions-phase5/` (board 7's shape) with answer keys `fixtures/expected/phase5/overview.json` and
`links.json` (Overview → Phase 5 fixture results). Step 5.2 adds `made` / `hook` to the existing overview
keys (`overview.json`, `hook/overview.json`, `phase4/overview.json`) and nothing else.

**Phase 6 changes (Step 6.1, all additive, design W0–W3, W5):** Session and SessionRow gain `activity`,
`asked`, `imported` and `state_since` (Session → Phase 6 fields). No existing value, key, window membership,
group or row order changes; imported sessions stay in every report. **The JSON now exports prompt text**
(`asked` and, since Step 7.5, `last_asked`). New isolated fixture root `fixtures/sessions-phase6/` (boards 7c/8c's shape, synthetic prompts) with
answer keys `fixtures/expected/phase6/sessions.json` and `overview.json` (Phase 6 fixture results). Step 6.2
adds the four fields to every existing key and nothing else. `asked` there follows W3 as written: the
`fixtures/sessions` Claude transcripts whose first own prompt is `CASE-<name>: <prompt>` get that string; the
phase4/phase5 Claude prompts (`<prompt>`) get `null`. Step 6.2 gave every older Codex fixture `null`
when it had no `user_message`; Step 6.6 changed the older Codex fixtures' `asked` values where the X4
response-item fallback finds an eligible prompt (the existing scrubbed `CASE-<name>: <prompt>` strings).

**Phase 8 changes (Step 8.1, design G0):** new report `graph` and its section below. No Session, SessionRow, link or
overview key changes; every committed key stays equal. New isolated fixture root `fixtures/sessions-phase8/` (board 9's
shape, synthetic prompts, the Codex `exec` record shape) with answer keys `fixtures/expected/phase8/graph-<id>.json`.

**Phase 9 changes (Step 9.0, all additive, design R0–R6):** `state_evidence` gains `"transcript_exit"` and
`"session_end"`; Link `evidence` and hook `child_evidence` gain `"redirect_file"`; hook records gain `kind: "ended"`
(event `SessionEnd`, field `reason`); OpenAction `reason` gains `"app_session"` and `"unknown_entrypoint"`. Session
gains no key (Step 9.6 adds `resume_turn` for R7, written there). Codex rollouts are also read from `rollout-*.jsonl.zst`. State, Exact evidence, Hook records and Open
below carry the rules. Existing keys change only where R2 applies (12 `fixtures/sessions` transcripts end with a
`cost-state` after their last turn; the values are listed in PROGRESS.md, Step 9.0). New isolated fixture root
`fixtures/sessions-phase9/` with answer keys `fixtures/expected/phase9/` (Phase 9 fixture results, at the end).

**Phase 3 changes (Step 3.2):** Session gains `checkout` and `host_session_id`; new sections "Overview"
(`multi-link overview`, design T2, T5, T11) and "Open" (`multi-link open`, design T7). New fixture: the
synthetic busy record `.claude/sessions/70001.json` (a second busy session in `/home/dev/repo-c`, for the
FR17 warning; no `hostSessionId`), which adds one row to every sessions key.

Transcript formats are undocumented. Every field path below was observed in the Phase 0 fixtures
(`docs/phase0.md`), not in any spec.

## Common rules

### Flags
| Flag | Meaning |
| --- | --- |
| `--root <dir>` | Home-shaped root. Reads `<dir>/.claude/projects/`, `<dir>/.claude/sessions/`, `<dir>/.codex/sessions/`. Default: `$HOME`. When given (fixture mode): no process probing, no git lookups, task outputs come from `<dir>/task-output/` (see Links), hook records from `<dir>/.multi-link/hook/` (see Hook records), plans from `<dir>/repos/<basename of repo>/` (see Overview → Plan). |
| `--now <RFC 3339>` | Replaces the clock. Used by two things only: the `--since` window and the 300 s freshness check in State. |
| `--since <duration>\|all` | History window, default `2h`: a transcript session is listed if its `last_activity` ≥ now − window. Session-record sessions and `waiting` sessions are always listed (active discovery, FR1; State → Visibility). `all` disables the window. The window filters **output only**: Phase 1 parses every transcript under the root, so exact evidence from old files still excludes heuristic candidates. A link is listed when its `parent` or `child` is a listed session; `unparented` lists only listed sessions. |
| `--json` | Print the JSON below (pretty-printed; tests compare parsed JSON, not bytes). |
| `--repo <dir>` (`steps` only, repeatable) | Repos to read. Relative paths resolve against `--root`. Default: the `repo` of every listed session. |
| `--repo <repo>`, `--step <id>` (`graph`, both required) | The session `repo` (real or fixture path, as `sessions` prints it) and the PLAN.md step id. `graph` takes no `--since`. |

**Phase 1 Verify commands** (each must equal its expected file as parsed JSON):
```
multi-link sessions --root fixtures/sessions --now 2026-10-29T23:15:00Z --since all --json
multi-link links    --root fixtures/sessions --now 2026-10-29T23:15:00Z --since all --json
multi-link steps    --root fixtures/sessions --repo repos/repo-a --repo repos/repo-b --json
```
`--since all` is needed because the organic and malformed fixtures are older than 2 h at that `--now`.
**Phase 3 Verify command:** `multi-link overview --root fixtures/sessions --now 2026-10-29T23:15:00Z --since all --json`
must equal `fixtures/expected/overview.json` (parsed); the Hook variant has its own row (Hook variant).

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
| `record` | string \| null | `.claude/sessions/<pid>.json` whose `sessionId` equals `id`, else `null`. **Several records with one `sessionId`** (a crashed record left behind plus a live resume, Phase 3 review #1): the session is `finished` only when **every** matching record is confirmed gone; otherwise `record` (and the state, `host_session_id`) come from the matching record that is not confirmed gone with the greatest `updatedAt`; when all are gone, from the greatest `updatedAt`. Ties by file name. 3.3 unit test. |
| `host_session_id` | string \| null | The record's `hostSessionId` (`local_…`, present when the Claude desktop app started the session; design → Opening a session), exactly as written. `null` when there is no record, the record has no such field or it is not a string, and for every Codex session. It differs from `id` (Codex P2 #3: the reader must keep it, not drop it). |
| `cwd` | string | Claude: `cwd` of the last record in file order that has one (a resumed file can carry earlier records with another `sessionId` and other cwds). Record-only: record `cwd`. Codex: `session_meta.payload.cwd`. |
| `repo` | string | Git root of `cwd`. In fixture mode (`--root`) git lookups are off, so `repo` = `cwd`; the worktree `repo-a-wt` is therefore its own repo in the fixtures. Git-root and worktree grouping is a Phase 1 unit test on a temp repo. |
| `checkout` | string | Git toplevel of `cwd` (`git rev-parse --show-toplevel`, the same call that gives `repo`): the main checkout or the worktree the session ran in (design T8). For a worktree `repo` is the main repo and `checkout` the worktree; for a main checkout both are equal. Fixture mode, and a failed git lookup: `checkout` = `cwd`, like `repo`. Real-mode values are a unit test on a temp repo with a worktree (fixture mode cannot show them). |
| `branch` | string \| null | Claude: last `gitBranch`. Codex: `session_meta.payload.git.branch`. Record-only: `null`. |
| `started_at` | timestamp | Claude: first top-level `timestamp` in file order (any record, including inherited ones). Codex: `session_meta.payload.timestamp`, else first record `timestamp`. Record-only: `startedAt`. |
| `last_activity` | timestamp | Last top-level `timestamp` in file order (main transcript only). Record-only: `updatedAt`. |
| `state` | enum | See State. |
| `state_evidence` | enum | See State. |
| `segments` | Segment[] | See Segments. `[]` for record-only sessions. |
| `subagents` | string[] | Claude: `agentId`s of `<id>/subagents/agent-<agentId>.jsonl` files. A subagent is part of its parent session, never a session of its own. Codex: `[]`. |
| `activity` | `{"from": timestamp, "to": timestamp}`[] | Phase 6, design W1: recorded activity as bursts, oldest first. See Phase 6 fields. |
| `asked` | string \| null | Phase 6, design W3: the session's first prompt. See Phase 6 fields. |
| `title` | string \| null | U1: last own main-file Claude custom title, trimmed with controls removed; otherwise null. Codex and record-only sessions: null. |
| `started_by` | string \| null | Step 8.4c, design S1: the transcript id of the Claude session that started this one (from the desktop app's session records), else null. Always null for Codex and record-only sessions. Not a `Link`. |
| `last_asked` | string \| null | Step 7.5, design P5: the latest eligible prompt. |
| `imported` | bool | Phase 6, design W2: a Codex Desktop import. Always `false` for Claude. |
| `state_since` | timestamp \| null | Phase 6, design W5: when the current `state` began, from the evidence `state` used. |

### Phase 6 fields
Transcript formats are undocumented; the import marker and the `user_message` event were observed on local
data on 2026-11-01 and are assumptions (design W2).
- **`activity` (W1).** Timestamps: Claude, every top-level `timestamp` of the main transcript and of each
  `<id>/subagents/agent-*.jsonl`, except records whose `sessionId` is present and differs from the session id
  (inherited history of a resumed or forked file, as W3 skips it; subagent records carry the parent's id and
  count); Codex, every record's `timestamp` except the `session_meta` record and records inside imported turns.
  An `imported` rollout has `activity` `[]` and `asked` `null` whatever else it holds (real all-import files can
  carry an `event_msg` outside every turn, e.g. `thread_settings_applied`). Sort, drop duplicates, then split where two consecutive timestamps are more
  than 300 s apart (300 s exactly stays in one burst); each burst is `{"from": first, "to": last}` (one record:
  `from` = `to`). Clip to the report window: drop bursts that end before `now − since` or start after `now`,
  and cut `from` / `to` to the window (with `--since all` only `now` cuts). Record-only sessions: `[]`.
- **Imported turns (W2).** Walking the rollout in file order, an imported turn opens at an `event_msg`
  `task_started` whose `payload.turn_id` starts with `external-import` and closes at the first of: the
  `task_complete` with the same `turn_id` (that record is inside), the next `task_started` (outside; it opens
  the next turn), end of file. `imported` is `true` when the rollout has at least one `task_started` and every
  `task_started` opens an imported turn. Imported records still count for `state`, `started_at` and
  `last_activity` (unchanged).
- **`asked` (W3, P1).** Claude, main transcript only: the first record with `type` `user` whose `sessionId`
  equals the session id and whose `message.content` is a string, or an array holding a block of `type` `text`
  (the first such block's string `text` is used, whatever blocks come before it) (records whose content holds a `tool_result` are skipped); trim leading whitespace. P1 (Step 7.1): while the text starts with exactly
  `<system-reminder>` (no attributes), remove through the first following `</system-reminder>` and trim
  leading whitespace again. No nesting is parsed; an unclosed reminder skips the record. No other tag
  is stripped (`<system-reminder-extra>` does not match). Skip the record if the remaining text is blank
  or starts with `<`, and keep looking. Codex: `payload.message` of the first `event_msg`
  `user_message` outside an imported turn. This event prompt keeps priority, even if blank or later in the
  file. If no such event supplies a string prompt, X4 (Step 6.6) falls back to the first eligible
  `response_item` outside an imported turn with `payload.type` `message` and `payload.role` `user`:
  take the first `input_text` block's string `text` from `payload.content`, trim leading whitespace, and
  skip the item if it is blank or starts with `# AGENTS.md instructions` or `<`; keep looking for an
  eligible item. Then every whitespace run becomes one space, trailing space is
  trimmed, and text over 120 characters is cut to 119 characters (on a char boundary) plus `…`. None found or
  record-only: `null`.
- **`last_asked` (P5).** The latest eligible prompt, with the same normalization and 120-character
  limit as `asked`. Claude uses the last own main-transcript user record accepted by W3 + P1, also
  excluding `isMeta: true`, `isCompactSummary: true` and text starting `[Request interrupted by user`.
  Codex uses the last non-imported `user_message` string; if none supplies a string, the last eligible
  X4 response-item user message. All-import rollouts and record-only sessions yield `null`.
  `asked` retains its existing value and meaning. Labels prefer `title`, then the latest step, then `last_asked`,
  then `asked`, then the harness's no-prompt fallback. Cache refresh re-parses changed transcripts.
- **`state_since` (W5).** `waiting` or `stale` with `state_evidence` `session_record`: the record's
  `statusUpdatedAt`; `waiting` by `hook`: the `at` of the hook record that made it waiting; Codex `unknown` /
  `turns_complete`: the `timestamp` of the last `task_complete` outside an imported turn (`null` if every turn
  was imported); every other case `null` (busy, idle, finished, `no_signal`, Codex `stale` / `open_turn`).
- SessionRow copies these fields from its Session (children and merged members too), placed after `made`.


### State
`state` ∈ `busy | waiting | idle | finished | stale | unknown`. `waiting` means Claude is blocked on the
user (a permission prompt, H3). Transcripts carry no waiting signal (Phase 0 (b)); Step 2.1 found that the
live session record does (`status: "waiting"` + `waitingFor`, design H11), and the hook adds a fallback
for sessions without a record (H8). Codex waiting is unsupported.

`state_evidence` ∈ `process_exit | session_record | open_turn | turns_complete | no_signal | hook | transcript_exit | session_end`
(the last two since Phase 9).

Rows are checked top to bottom; the first match wins. If the process probe cannot run (fixture mode,
`ps` denied, permission error) the record is trusted as written and the `finished` row is skipped.

| Situation | `state` | `state_evidence` |
| --- | --- | --- |
| Claude session record whose process is confirmed gone: `ps` shows no `pid`, or the `pid` has a different start time than `procStart` (pid reused). Checked first. | `finished` | `process_exit` |
| Claude session record, `status=waiting` (any age: a prompt can wait overnight; `waitingFor` is not read, its only observed value is `"permission prompt"`). Design H11. | `waiting` | `session_record` |
| Claude session record, `status=busy`, process probe `Alive` (any age; U3) | `busy` | `session_record` |
| Claude session record, `status=busy`, process probe `Unavailable`, now − `statusUpdatedAt` ≤ 300 s | `busy` | `session_record` |
| Claude session record, `status=busy`, process probe `Unavailable`, older than 300 s | `stale` | `session_record` |
| Claude session record, `status=idle` (any age; freshness applies to `busy` only, Step 0.2 review) | `idle` | `session_record` |
| Claude session record with any other `status` value | `unknown` | `session_record` |
| *(Phase 9, R5)* Claude transcript, no session record, and the session has an uncleared `ended` hook record (Hook records → Ended). | `finished` | `session_end` |
| *(Phase 9, R2)* Claude transcript, no session record, and the last `cost-state` record of its main transcript (own `sessionId` or none) comes after its last turn (below). | `finished` | `transcript_exit` |
| Claude transcript, no session record, and the session's latest hook record is an uncleared `waiting` (Hook records → Waiting). Design H8: after `finished` (a record session never reaches this row, so the record's own status always wins), before the transcript-only fallback. | `waiting` | `hook` |
| Claude transcript, no session record | `unknown` | `no_signal` |
| Codex, last `task_started` turn has no `task_complete`, `last_activity` ≤ 300 s ago | `busy` | `open_turn` |
| Codex, open turn, older than 300 s | `stale` | `open_turn` |
| Codex, last `task_started` turn has its `task_complete` (the latest turn governs; an earlier unmatched turn was interrupted and is ignored) | `unknown` | `turns_complete` |
| Codex, no `task_started` in the file | `unknown` | `no_signal` |

**Turn (R2, R5).** A record of the session's main transcript whose `sessionId` is the transcript id or
absent, and that is either `assistant`, or `user` with `isMeta` not `true` whose text (string content, or the
first `text` block) does not start, after leading whitespace, with `<command-name>`, `<command-message>`,
`<command-args>`, `<local-command-stdout>`, `<local-command-stderr>` or `<local-command-caveat>`. A `user`
record holding only `tool_result` blocks is a turn. Subagent files are not read for this. `cost-state` has no
timestamp, so R2 compares file positions (line order); R5 compares a turn's `timestamp` with the record's `at`.
Every other record (`system`, `last-prompt`, `mode`, a `/exit` echo) neither clears nor sets the marker.
`state_since` is `null` for both Phase 9 rows (a `cost-state` has no time; `finished` is `null` under W5).

Why `unknown` and not `finished` for transcript-only sessions *(Phase 1–8 text; Phase 9 adds the two rows above
on positive evidence only)*: Phase 0 found no exit evidence in
transcripts. A missing session record proves nothing (`claude -p` may never write one), and a completed
Codex turn ends the turn, not the process. Showing `finished` would be a guess.

In fixture mode the session records are trusted as written (process probing is off; their pids belong
to another machine). With `--now 2026-10-29T23:15:00Z` the two busy records are `busy` (`60243.json`, `statusUpdatedAt`
23:12:39.714Z, age 140 s; synthetic `70001.json`, 23:13:30.000Z, age 90 s); the four idle records are `idle`; all transcript sessions are
`unknown` until Phase 9, which makes 12 of them `finished` / `transcript_exit` (their last `cost-state` follows
their last turn: 0d7faed4, 85a715ba, 11c1f13c, bb792d9c, ef72378c, 80cce3e4, c86d374f, 07c80c2e, 11bf1b58,
9d9a0ed8, df403bfa, f4d83303; all `claude-desktop`, none with a record). `stale` and `finished` have no fixture: Phase 1 covers them with state-machine unit tests
(busy record with unavailable process probe at now + 301 s → `stale`; open Codex turn; process gone → `finished`). `waiting` has no row
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
| `evidence` | `"stdout"` \| `"resume_arg"` \| `"thread_ready"` \| `"task_output"` \| `"redirect_file"` \| `"hook"` \| null | What made it exact; `null` unless `exact`. `hook` only in Phase 2, `redirect_file` since Phase 9; see Exact evidence. |
| `candidates` | `{"id": string, "gap_ms": int}`[] | Heuristic candidates (1 for `heuristic`, ≥ 2 for `ambiguous`); `[]` for `exact` and `unlinked`. Ordered by `gap_ms`, then `id`. |
| `step` | string \| null | Step tag of the launch segment (Step tags), set on every link whatever its `match`. `null` when the segment has no tag. |

### Recognised invocations
Only `tool_use` blocks inside `type=assistant` records count (`api-request-blob` records repeat the
same `tool_use` ids and must be ignored). `name=Bash`; split `input.command` into segments on `&&`,
`||`, `;`, `|` and newlines outside quotes; a segment is a launch when its tokens are:
- `codex exec resume <uuid>` or `codex resume <uuid>` → `resume`;
- `codex exec …` (anything else) → `exec`;
- `node <path ending in codex-companion.mjs> task …` → `plugin` (exactly `task`; `task-resume-candidate` is not a launch).

**U4: help is not a hand-off.** Exclude a Codex segment when `--help` or `-h` is an unquoted,
unescaped word after `exec`/`resume` and before the first non-option word. Skip option values for
`-c`, `-o`, `-C`, `-s`, `--add-dir` (and their long forms `--config`, `--output-last-message`, `--cd`,
`--sandbox`); after `--`, nothing is an option. Quoted/escaped help text and help inside a prompt or
option value remain launches. Continue checking the other segments: `codex exec --help; codex exec
"Step 1.2 implement"` recognises the second launch.

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
4b. *(Phase 9, R4)* The launch segment redirects stdout to a literal path and that file holds **exactly one**
   `session id: <uuid>` line → `"redirect_file"`. Checked after source 4.
   - **Redirect forms** on the launch segment only (the segment Recognised invocations chose): `>`, `>>`, `1>`,
     `>|`, `&>`, `&>>`, each with or without `2>&1`; the target is the next word or the text glued to the
     operator. Exactly one stdout redirect on the segment, else nothing is read. A target that is not a
     literal path (contains `$`, a backtick, a glob or a quoted expansion) → nothing is read.
   - **Path.** An absolute target is used as is. A relative one resolves against the record's `cwd` (the hook:
     the payload `cwd`) only when no earlier segment of the same command is `cd`, `pushd` or a subshell
     (`dir_moved`, as for the heuristic); otherwise nothing is read.
   - **Safe read.** Open non-blocking and `fstat` the opened file: anything but a regular file (FIFO, device,
     directory, socket) → no read and no wait. Then the opened file's canonical path must be under the
     canonical `cwd` or a temp dir (`$TMPDIR`, `/tmp`, `/private/tmp`); else no read. Read at most 64 KiB.
     A missing or unreadable file gives no evidence and no diagnostic, like a gone task output file.
   - **Fixture mode** (`--root`): the path is read root-relative, `<root>/<path without its leading />`, and a
     relative target is `<root>/<cwd without its leading />/<target>`. Containment is checked against
     `<root>/<cwd>`, `<root>/tmp` and `<root>/private/tmp` (canonical); `$TMPDIR` is ignored.
   - **Why exactly one id:** with `>>` an older run's header can be in the same file, and the file does not say
     which run this invocation started. Two or more ids → no exact; the heuristic then runs as today.
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
is `unlinked` (no fixture has this; Phase 1 unit test). **Phase 4 (design U1):** a `-C` / `-C<value>` /
`--cd <value>` / `--cd=<value>` whose value is `.`, or `$PWD` / `${PWD}` unquoted or inside double quotes,
is not a move. With several such flags the call moves if any one moves; a flag with no value moves. Single-quoted (`'$PWD'`) or backslash-escaped (`\$PWD`) values are literal text and still
move, as does any other value (`"$PWD/x"`, `"$HOME/x"`, `/tmp/x`); a preceding `cd` segment still moves;
`-C` after `--` is not a flag. Step 4.1 unit tests; no fixture has these forms.

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
| `kind` | `"handoff"` \| `"waiting"` \| `"ended"` | `ended` since Phase 9 (R5). |
| `at` | timestamp | The hook's own clock when the record is written (payloads carry no event time, 2.1 (h)); RFC 3339 UTC, 3 fractional digits. A first or only record is written on receipt, so `at` is the receive time and trails the event by about 0.1–0.2 s (async hook). The appended background child record (below) has the time the child was found, later than its first record's `at`. |
| `session_id` | string | Payload `session_id` (= transcript stem, 2.1 (f)); equals the file stem. For a subagent call it is the **parent's** id. |
| `agent_id` | string \| null | Payload `agent_id` (subagent calls only, e.g. `aee8ad2d0134ca48a` for the Codex plugin's `codex:codex-rescue` subagent; 17 chars in 2.1). Informational: the join below does not use it. |
| `event` | string | Source hook event: `PostToolUse` or `PostToolUseFailure` for `handoff`; `PermissionRequest` or `Notification` for `waiting`; `SessionEnd` for `ended`. |
| `tool_use_id` | string \| null | Payload `tool_use_id` (= the transcript's `tool_use.id`, 2.1 (f)) for `handoff`; `null` for `waiting` (`PermissionRequest` / `Notification` have none). |
| `cwd` | string | Payload `cwd`. Informational. |
| `child` | string \| null | `handoff` only: the Codex session id the hook observed (lowercase uuid), else `null`. Always `null` for `waiting`. |
| `child_evidence` | `"stdout"` \| `"thread_ready"` \| `"task_output"` \| `"redirect_file"` \| null | Where `child` was seen; `null` exactly when `child` is `null`. `stdout`: a `session id: <uuid>` line in `tool_response.stdout`, or for `PostToolUseFailure` in `error` (2.1: a failed run prints its id there; `event` tells the two apart). `thread_ready`: `[codex] Thread ready (<uuid>)` in that text (plugin). `task_output`: a `session id:` line in the background task's output file, at `dirname(scratchpad_dir)/tasks/<tool_response.backgroundTaskId>.output` (2.1 (g)). |
| `notification_type` | string \| null | `Notification` only: `"permission_prompt"` or `"elicitation_dialog"`; `null` for every other event. |
| `reason` | string | **`ended` only, and only there** (other kinds have no `reason` key, so older records stay valid): the payload `reason` verbatim. Observed: `prompt_input_exit` (`/exit`, Ctrl-C), `other` (SIGTERM, SIGHUP); documented: `clear`, `logout` and others. Any string is accepted. |

`redirect_file` (Phase 9, R4): a `session id:` line in the file the launch segment's stdout is redirected to
(Links → Exact evidence 4b, same path rules, the payload `cwd`). An `ended` record has `tool_use_id`, `child`,
`child_evidence`, `notification_type` and `agent_id` all `null`.

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
- *(Phase 9, R4)* A hand-off with no id in its output text (and, for a background call, none in the task
  output file) whose launch segment has a readable stdout redirect (Exact evidence 4b) polls that file the way
  it polls the task output (64 KiB, 25 s, first record `child: null` at once) and appends a second record with
  `child_evidence: "redirect_file"` when exactly one `session id:` line appears. Still at most two records per
  `tool_use_id`.
- *(Phase 9, R5)* `SessionEnd` → one `ended` record with the payload `reason`. Snippet entry shape as the
  others (`async: true`, `timeout: 60`).
- Anything else (other events or notification types, e.g. `idle_prompt`; non-launch Bash; invalid
  payloads) → nothing is written.

### Reader rules
- Lines go to `sessions` → `skipped` like transcript lines, with `file` relative to the root when the
  state folder is inside it (always in fixture mode: `.multi-link/hook/<id>.jsonl`), else absolute.
  `links` has no `skipped` list; it uses the same valid records.
- Blank line → `empty_line`. Non-empty, newline-terminated line that is not a JSON object →
  `invalid_json`. An unterminated last line is buffered, not skipped (as for transcripts).
- Valid JSON that is not a `v: 1` record this reader understands → `unknown_type`: `v` ≠ `1` or missing,
  `kind` not `handoff` / `waiting` / `ended`, an `ended` record without a string `reason` or with a non-null
  `tool_use_id` / `child` / `child_evidence` / `notification_type`, a missing field, a wrong type, an `event` / `child_evidence` /
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

### Ended (State row `finished` / `session_end`, Phase 9, R5)
- Only for a Claude session with a transcript and **no** session record (a record always wins: it says the
  process lives). Precedence: session record → `process_exit` → `session_end` → `transcript_exit` → hook
  `waiting` → `no_signal`.
- The session's latest `ended` record (by `at`, ties by line) counts only when **no turn** (State → Turn) in
  the main transcript has a `timestamp` later than its `at`, **and** no other valid hook record of that session
  has a later `at` (a resume clears it, also when its async hook record lands late).
- `reason: "clear"` ends the record's own `session_id` (the old id) only; the id the process continues under
  has its own transcript and record.

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
| `overview --since all` (Phase 3) | `hook/overview.json` |

| Hook file (`.multi-link/hook/`) | Records | Expected effect |
| --- | --- | --- |
| `07c80c2e-….jsonl` (bg) | `handoff` `child: null`, then `child` `01a189e3-f419-…` / `task_output` | bg link `exact` / `hook`, same child as `links.json` (its task output file is deleted, so sources 1–4 find nothing; without the hook it would be `heuristic`) |
| `85a715ba-….jsonl` (repo-c) | `handoff` `child: null` for `toolu_01DYjrRidN29nXMRN9Rci1uz` | stays `unlinked` |
| `bb792d9c-….jsonl` (exec) | `handoff` naming `01a189e5-5b32-…` (the manual run) | conflict: exec link keeps `stdout` / `01a189e2-d57f-…`; manual stays `unparented`; `hook_conflicts` = 1 |
| `9d9a0ed8-….jsonl` (plugin) | `handoff` with `agent_id` `aee8ad2d0134ca48a`, same child as the transcript (`thread_ready`) | agrees: no change, no conflict |
| `11c1f13c-….jsonl` (repo-c, last line 2026-10-28T10:03:04.840Z) | `waiting` `PermissionRequest` 10:03:05.020Z, then `Notification` `permission_prompt` 10:03:11.004Z | Phases 2–8: `waiting` / `hook`; listed in `sessions-2h` although its last line is ~37 h old. **Phase 9:** its transcript ends with a `cost-state` after the last turn, and `transcript_exit` comes before hook `waiting` (R5 order) → `finished` / `transcript_exit`, `state_since` `null`, and it leaves `sessions-2h` (not `waiting`, last line before the window) |
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
(d1f3b188) in both session files. **Phase 9 (R2, R5):** 11c1f13c is `finished` / `transcript_exit` and drops out of
`hook/sessions-2h.json`, so the hook variant has no `waiting` / `hook` row any more; that row moved to the Phase 9
root (92000012, with 92000011 for the precedence). The other 11 R2 sessions change as in `sessions.json`. (Phase 3: the synthetic `70001.json` row is in `sessions.json`, so it is
in all three session keys; as a record session it is never windowed out.)

## `overview`

Phase 3 (design T2): the home screen's data, computed in core so every front end shows the same thing.
`multi-link overview [--root] [--now] [--since] [--json]` takes the flags of `sessions`. Text mode (one line
per repo) is not compared by any test.

```json
{
  "now": "2026-10-29T23:15:00.000Z",
  "hook": {"records": 0, "latest": null},
  "repos": [ RepoGroup, ... ]
}
```

**`hook` (Phase 5, design V8):** what the reader observed of the optional hook, never a claim that it is
installed. `records`: the number of valid `v: 1` records in every `<uuid>.jsonl` file of the hook folder
(Hook records → File layout and Reader rules; every `kind`, every age: `--since` does not filter it;
skipped lines are not counted). `latest`: the greatest record `at`, or `null` when `records` is 0. No folder
or an empty folder: `{"records": 0, "latest": null}`. A folder that exists but cannot be read gives the same
value plus a diagnostic on stderr (the report never fails for it). The TUI shows `◆ needs-you partial: no
hook records seen` only when no session is `waiting` and `records` is 0.

**Inputs.** One refresh (`Scanner::refresh`, Step 3.1) gives the listed sessions and listed links, exactly
as `sessions` and `links` with the same flags would list them; the plan comes from the steps parser.
Every listed session appears **exactly once** in the report: as a top-level row, as a child row, or as a
member of a merged row. No other session appears. Subagents are never rows (they are part of their session).

### RepoGroup — one per distinct Session `repo` among the shown rows
| Field | Type | Rule |
| --- | --- | --- |
| `repo` | string | Session `repo`, the recorded string (fixtures: `/home/dev/...`). |
| `tier` | `"waiting"` \| `"busy"` \| `"other"` | Best tier of any session shown in the group (rows, children, merged members): `waiting` if any is `waiting`, else `busy` if any is `busy`, else `other` (design T5; `stale` is `other`). |
| `last_activity` | timestamp | Greatest `last_activity` of every session shown in the group. |
| `plan` | Plan | See Plan. |
| `active_steps` | string[] | Distinct non-null Link `step` of the group's active hand-offs, ascending (string order). A hand-off (listed link) belongs to the group that shows its `parent`; it is active when its parent's `state`, or its `child`'s `state` if the child is listed, is `waiting`, `busy` or `stale`. Fixtures: `[]` everywhere (no tags, nothing active). |
| `concurrency` | Warning[] | FR17: one `{"cwd": string, "sessions": [id, ...]}` per `cwd` with **2 or more** `busy` sessions shown in the group (any harness; Session `cwd`, exact string; `stale` does not count). `sessions` ascending; warnings by `cwd`. `[]` if none. |
| `latest_handoff` | Link \| null | Among listed links whose `parent` is shown in this group, the greatest (`at`, `invocation`), any `match` (an `unlinked` launch is still the latest hand-off). The Link object is copied from `links` unchanged. `null` if none. |
| `rows` | Row[] | Top-level rows, see Rows. |

Repos are ordered by `tier` (`waiting`, `busy`, `other`), then `last_activity` descending, then `repo`
ascending.

### Rows
A Row is a SessionRow (`"kind": "session"`) or a MergedRow (`"kind": "merged"`).

SessionRow: `{"kind": "session", "id", "harness", "state", "state_evidence", "checkout", "branch",
"started_at", "last_activity", "handoffs": Link[], "made": string[], "activity", "asked", "last_asked", "title", "started_by", "imported",
"state_since", "children": SessionRow[]}` (Phase 6 fields and P5: Session → Phase 6 fields). The plain fields are copied
from the Session. `handoffs`: for a Codex row, **every** listed link whose `child` is this id, ordered by
(`at`, `invocation`), Link objects copied unchanged; `[]` for a Claude row. `children`: Codex rows placed
under this Claude row and, from Step 8.4c, Claude rows it started (below); always `[]` on a Codex row.
`made` (Phase 5, design V1): for a Claude row, the distinct non-null `step` values of the listed links whose
`parent` is this id, ordered by each step's newest link (`at`, `invocation`), newest first; a step that
several links carry appears once, at its newest link's place. Any `match` counts (an `unlinked` launch still
names the step Claude handed off). `[]` for a Claude row with no such link or only untagged links, and
always `[]` on a Codex row (its step is its placing link's `step`, already in `handoffs`). Children and
merged members carry `made` like any SessionRow.

**Codex placement (design T11).** A Codex session's *placing hand-off* is the greatest (`at`,
`invocation`) among listed links whose `child` is its id and whose `parent` is a listed session with the
same `repo` as the Codex session. With one, the Codex row is a child of that parent's row; otherwise it
is a top-level row of its own repo (the manual run, ambiguous candidates, a child whose parent is outside
the window or in another repo; its `handoffs` still lists every link). A child is listed once, however many
hand-offs it has. Children are ordered by their placing hand-off (`at`, `invocation`). The same-repo
condition keeps every row inside its own repo's group; no fixture has a cross-repo link (3.3 unit test).

**Started-by nesting (Step 8.4c, design S3).** A Claude row whose `started_by` names a listed Claude session in
the **same repo group** (the mapped `repo`, never the app record's `cwd`) is a child of that row, to any depth
(coordinator, step session, its Codex run). Cross-repo or parent not listed: the row stays top level. A parent
is the first listed session with that id; a cycle never hides its members (the closing edge is ignored). Every
session appears exactly once. `children` of one row are one list ordered by time, then id: a Codex child by its
placing hand-off (`at`, `invocation`), a started child by its `started_at`. A parent's own place and state never
depend on its children. `handoffs`, `made`, `links`, `unparented` and every hand-off count ignore started-by.
With no app records every `started_by` is null and the overview equals the Step 8.4b shape.

**Top-level order (design T5).** `state`: `waiting`, `busy`, `stale`, `idle`, `unknown`, `finished`; then
`last_activity` descending; then `id` ascending. A parent's place follows its own `state` and
`last_activity`, never its children's.

**Merge (`claude ×N`, `codex ×N`).** After ordering, the *quiet* rows of a harness are:
- Claude: the top-level Claude SessionRows whose `state` is `idle`, `unknown` or `finished` **and** whose
  descendants (Codex or started, at any depth) are all `idle`, `unknown` or `finished` (a busy or stale child
  keeps its parent visible, review #4; Step 8.4c makes the test recursive).
- Codex (Phase 4, design U2): the top-level Codex SessionRows (no placing hand-off, so not a child) whose
  `state` is `idle`, `unknown` or `finished`. Their `handoffs` do not matter: an `unlinked`, `ambiguous`
  or cross-repo link leaves the row top-level and still mergeable.

Each harness is decided on its own. If a group has **4 or more** quiet rows of one harness, they are
removed and one MergedRow `{"kind": "merged", "harness": "claude" | "codex", "count": N, "rows":
[SessionRow, ...]}` takes the place of the first of them in the ordered list; `rows` keeps their order and
(Claude) their children. Fewer than 4 of a harness: that harness does not merge. So a group shows at most
two merged rows, one per harness, ordered by where their first member stood; a merged row never mixes
harnesses. `waiting` / `busy` / `stale` rows never merge and do not count towards the 4 (the top-level
order already places them before every quiet row). A MergedRow is not actionable: the TUI expands it before
`o` / `f`, so open always gets one full session id (T11). Its `harness` equals every member's `harness`;
the App identifies a merged row by (repo, `harness`) for selection, so a refresh keeps the same one selected
(Step 4.3); merged-row expansion stays keyed by repo (design U3).

### Plan
`{"convention": "supported" | "missing" | "unsupported", "latest_step": LatestStep | null}`, from the
steps parser (`steps` rules) on the group's plan folder:
- **Real mode:** `<repo>/PLAN.md` and `<repo>/PROGRESS.md` (the main checkout; worktrees share it).
- **Fixture mode (`--root <dir>`, Codex P2 #5):** `<dir>/repos/<basename of repo>/`, where the basename is
  the last `/`-separated component of the recorded `repo` string (`/home/dev/multi-link-sandbox/repo-a` →
  `<dir>/repos/repo-a/`). The group's `repo` keeps the recorded string. No folder or no `PLAN.md` →
  `missing`. Two repos with the same basename read the same folder and show the same plan (allowed; no
  fixture has it; 3.3 unit test).

`latest_step` is `null` unless `convention` is `supported` and PROGRESS.md has an entry. Then it is
`{"id": latest_logged_step, "title": string | null, "entry": Entry | null}`: `title` and `entry` come from
the steps report's Step with that `id` (`entry` = its `progress[0]`, the newest entry, which is the first
entry of PROGRESS.md); a logged id with no PLAN.md step gives `title: null`, `entry: null`. Entry is the
`steps` Entry (`date`, `agent`, `verify`).

**Required 3.3 unit tests** (no fixture pins them): a cross-repo link (child stays top-level in its repo);
merge at exactly 3 and exactly 4 quiet rows; a busy Codex child under an `unknown` parent (no merge of that
parent); a `stale` row (between `busy` and `idle`) and a `finished` row (last); two repos in one tier with
equal `last_activity` (ordered by `repo`); two repos with one plan basename; several records for one id.

### Fixture results (`overview.json`, `--since all`)
- 8 groups: `/home/dev/repo-c` is `busy` (two busy records); the rest are `other`, by `last_activity`:
  `/home/dev/project`, sandbox `repo-a`, sandbox `repo-b`, sandbox `repo-a-wt`, the `Library/…/scratch-…`
  folder, `/home/dev/repo-d`, `/home/dev/repo-e`.
- repo-c: FR17 warning `{"cwd": "/home/dev/repo-c", "sessions": ["00000000-…-000000000010", "6358ec7e-…"]}`;
  rows: the two busy records (90 s-old `70001` first, by activity), then 11c1f13c, 85a715ba (both
  `finished` / `transcript_exit` since Phase 9; the order is unchanged); latest
  hand-off `toolu_014Qp7cwQso6jcrnVvi6GhGq` (`unlinked`).
- sandbox repo-a (T11): Codex `01a189e2-…` has two hand-offs (exec from bb792d9c 22:58:26, resume from
  ef72378c 22:58:53); it sits under ef72378c (the later one) with both links in `handoffs`; bb792d9c has
  no children. The manual run `01a189e5-…` is a top-level row, first by activity. 3 quiet Claude rows: no
  merge. Plan: `supported`, latest step `0.4` "Parse sample records" with its reported Verify.
- sandbox repo-b: 5 quiet Claude rows → one MergedRow (`count` 5) holding all of them, each with its Codex
  child (plugin, bg, amb-1, amb-2); `mention` (df403bfa) has none. Plan: latest step `3`.
- sandbox repo-a-wt: its own group in fixture mode (`repo` = `cwd`), c86d374f with child `01a189e4-8084-…`.
- repo-e: the synthetic Codex file is a top-level row (unparented).

Hook variant (`hook/overview.json`, `--since all`, built as in Hook records → Hook variant): sandbox repo-a
(d1f3b188 `waiting` / `session_record`, latest activity 2026-10-30T06:08Z) and repo-c (11c1f13c `waiting` /
`hook`) form the `waiting` tier, in that order; no group is `busy`; the `other` order is unchanged. A
waiting row comes first in its group. **Phase 9:** 11c1f13c is `finished` / `transcript_exit`, so repo-c is the
`busy` tier (its two busy records) and its rows read `70001`, 6358ec7e, 11c1f13c, 85a715ba; sandbox repo-a stays
`waiting` (d1f3b188) and comes first. The TUI hook goldens follow (one fewer `need you`, no waiting card row). repo-a now has 4 Claude rows but only 3 quiet ones: still no merge.
The bg child's hand-off carries `evidence: "hook"`.

### Phase 4 fixture results (`phase4/overview.json`, `--since all`)
Verify command (Step 4.3 adds the test): `multi-link overview --root fixtures/sessions-phase4 --now
2026-10-29T23:15:00Z --since all --json` must equal `fixtures/expected/phase4/overview.json` (parsed).
The root is synthetic (hand-written in the synthetic formats of `fixtures/sessions`: the `repo-e` Codex
rollout and Claude transcript shapes and the `.claude/sessions/<pid>.json` record shape) and isolated, so
parser counts and CLI tests on `fixtures/sessions` keep their numbers. It has no hand-offs: every
`latest_handoff` is `null`, every `handoffs` / `children` / `active_steps` / `concurrency` is `[]`.
- 5 groups. U3 collapse status: repo-j, repo-g and repo-i collapse (tier `other`, no `stale`); repo-h
  (`busy`) and repo-k (`stale`) stay expanded. Step 4.4's phase4 home golden must also show repo-i
  expanded (both merged rows visible), so the `codex ×N` label and two merged rows appear in a golden.
- `/home/dev/repo-h` is `busy`; the rest are `other`, by `last_activity`: `repo-j` (23:05:00),
  `repo-k` (23:02:00), `repo-g` (23:01:00), `repo-i` (22:58:00).
- repo-h (no merge): Codex `…40b4` is `busy` / `open_turn` (last activity 23:14:00, 60 s old; it started
  22:30, between the quiet runs) and comes first; then 3 quiet Codex rows `…40b3`, `…40b2`, `…40b1`
  (`unknown` / `turns_complete`). Only 3 quiet rows: no `codex ×N` (the busy row started between the quiet
runs but sorts first and does not count towards the 4).
- repo-j (idle, collapsed): one record-only Claude session `…40d1` (`70401.json`, `idle` /
  `session_record`). Plan: `repos/repo-j/` is a copy of the `fixtures/sessions` repo-a plan (headings renamed to repo-j), so its latest
  step is `0.4` "Parse sample records" with the same reported Verify (the collapsed line's step + Verify).
- repo-k (stale-only, stays expanded): one Codex row `…40e1`, `stale` / `open_turn` (last activity
  23:02:00, 780 s old).
- repo-g (`codex ×4`): 4 quiet Codex rows → one MergedRow `harness: "codex"`, `count` 4, rows `…40a4`,
  `…40a3`, `…40a2`, `…40a1` (by activity).
- repo-i (two merged rows): 4 quiet Claude transcript rows (`unknown` / `no_signal`) and 4 quiet Codex rows,
  interleaved by activity (Codex first). Rows: MergedRow `codex` ×4 (`…40c4`, `…40c3`, `…40c2`, `…40c1`),
  then MergedRow `claude` ×4 (`…41c4`, `…41c3`, `…41c2`, `…41c1`).
- Plans: `missing` everywhere except repo-j.


### Phase 5 fixture results (`phase5/overview.json`, `phase5/links.json`, `--since all`)
Verify commands (Step 5.2 adds the test): `multi-link overview --root fixtures/sessions-phase5 --now
2026-10-29T23:20:00Z --since all --json` must equal `fixtures/expected/phase5/overview.json`, and `links`
with the same flags `fixtures/expected/phase5/links.json` (parsed). The root is synthetic, in the same
synthetic formats as `fixtures/sessions-phase4` (Claude transcript lines, `.claude/sessions/<pid>.json`
records, Codex rollouts with `session_meta` + one turn), plus Bash `tool_use` / `tool_result` pairs shaped
like the sandbox hand-offs in `fixtures/sessions`. Ids are `00000000-0000-4000-8000-00000000<4 hex>`; below
they are named by their last 4 digits. Times are UTC (board 7's local times are UTC−7). No hook folder:
`hook` is `{"records": 0, "latest": null}`. 19 sessions, 0 skipped lines.
- 3 groups: `/home/dev/repo-l` is `waiting` (23:19:30); `other` by `last_activity`: `repo-m` (23:19:00),
  `repo-n` (23:05:00). No `active_steps` (no hand-off touches a waiting/busy/stale session), no FR17
  warning (one busy session).
- repo-l rows: `5001` `waiting` / `session_record` (record `70501.json`, transcript 2026-10-28 23:40 →
  2026-10-29 00:05, yesterday in local time; listed whatever the window); `5002` `busy` / `session_record`
  (record `70502.json`, `statusUpdatedAt` 23:19:30, 30 s old; transcript 21:20:00 → 23:19:30); then one
  MergedRow `claude` ×7 (all quiet: six `idle` records and one transcript-only `unknown`), by
  `last_activity`: `5016` (22:14 → 22:26, no hand-off), `5015` (22:09 → 22:14, child `5026`), `5014`
  (22:01 → 22:09, child `5024`), `5013` (21:55 → 22:02, child `5023`), `5012` (21:50 → 21:57, no hand-off),
  `5011` (21:48 → 21:51, child `5021`), `5003` (20:30 → 20:50, `unknown` / `no_signal`, before the 2 h bar
  window); then Codex `5025` (22:06 → 22:07, `unknown` / `turns_complete`, no link: the manual run). One
  quiet top-level Codex row, so no `codex ×N`. `made`: `5011` `["4.1"]`, `5013` `["4.3"]`, `5014`
  `["4.4"]`, `5015` `["4.5"]`, every other row `[]`. Plan `supported`, latest step `4.6` "Real check and
  README", Verify result `unknown` (`→ all suites green …` is neither pass nor fail). `latest_handoff`: the
  step 4.5 link.
- Links (4, all `form: "exec"`, step tags from `Read AGENTS.md, then do Step <id> in PLAN.md.`):
  `5011` → `5021` `exact` / `stdout` (tool result has `session id: …5021`), step `4.1`;
  `5013` → `5023` `heuristic` (gap 3000 ms), step `4.3`; `5014` → `5024` `heuristic` (gap 2000 ms), step
  `4.4`; `5015` → `5026` `heuristic` (gap 3000 ms, `background: true`), step `4.5`. Child `5026` runs
  22:09:33 → 22:17:00, after its parent's last activity (22:14). `unparented`: `5025`, `5041`, `5042`,
  `5051`, `5052`.
- repo-m: record-only Claude `5031` `idle` (`70531.json`, 22:40 → 23:19), Codex `5042` (22:55 → 22:58) and
  `5041` (22:45 → 22:47), both `unknown`. Plan `supported`, latest step `2.3` "Export the summary", Verify
  `pass`.
- repo-n: Codex only, `5052` (22:58 → 23:05) and `5051` (22:30 → 22:35), both `unknown`; plan `missing`.
- Header counts for the target golden (design V8): 1 busy, 7 idle, 10 done or unknown (`5003`, five Codex
  in repo-l, two in repo-m, two in repo-n), 1 waiting, 3 repos, 4 hand-offs.

### Phase 6 fixture results (`phase6/{sessions,overview}.json`, `--since 2h`)
Verify commands (Step 6.2 adds the tests): `multi-link sessions --root fixtures/sessions-phase6 --now
2026-11-03T06:25:00Z --since 2h --json` must equal `fixtures/expected/phase6/sessions.json`, and `overview` with
the same flags `fixtures/expected/phase6/overview.json` (parsed). `--since 2h` is the TUI default, so the keys
are what the target screens are drawn from (design → Phase 6 decisions → Target screens). The keys are the CLI
output at d4d862a plus the four fields, written by hand per session and cross-checked by an independent
scratch script. The root is synthetic, in the Phase 5 formats plus Codex `user_message` / `agent_message`
events, one Claude subagent file and Codex Desktop imports; prompts are invented sentences. Ids are
`<tag>0000-0000-4000-8000-00000000<tag>` with a 4-digit tag, named by tag below (the 8-character prefix differs
per session). Times are UTC; local (UTC−8) in brackets. No hook folder. 13 sessions, 0 skipped lines.
- **Links** (2, `form: "exec"`): `6002` → `6021` `exact` / `stdout`, step `6.3`, at 05:25:00 [21:25];
  `6011` → `6013` `heuristic` (gap 12000 ms), step `null`, at 06:23:44 [22:23]. `unparented`: `6014`, `6022`,
  `6051`.
- **Groups:** `repo-p` `waiting`, `repo-q` `busy`, then `other` by `last_activity`: `repo-r` (06:24:40),
  `repo-t` (05:50:00), `repo-s` (2026-10-30T19:00:00). `active_steps`: `["6.3"]` in repo-p (its parent is
  busy), else `[]`. No FR17 warning. No merged rows.
- **repo-p** (plan `supported`, latest step `6.3` "Show prompts on the home screen"; `latest_handoff` the 6.3
  link): `6001` `waiting` / `session_record` (transcript and record on Nov 2 00:30–00:41, i.e. yesterday Nov 1
  local; `activity` `[]` after clipping; `state_since` 2026-11-02T00:41:00.000Z = `statusUpdatedAt`); `6002`
  `busy` (bursts 05:10–05:14, 05:20–05:25, 05:31:30–05:33, 06:20–06:24:30: the 6.5 min wait on its Codex run
  is a gap, design W1's limitation), child `6021` (`unknown` / `turns_complete`, burst 05:25:03–05:31:00, its
  `session_meta` at 05:25:02 is not activity; `state_since` 05:31:00); `6004`, `6003` record-only `idle`
  (Nov 1 / Oct 31; `activity` `[]`, `asked` `null`); `6022` the **mixed rollout**: one imported turn at
  04:50:00, then a real turn 05:40–05:44 → `imported` false, `activity` 05:40–05:44, `asked` from the real turn,
  `state_since` 05:44:00.
- **repo-q** (plan `missing`): `6031` `busy`, branch `feature-x`, three bursts (05:05–05:10, 05:50–05:57,
  06:23–06:24:50 = 830 s in all); `asked` from a `text` block, with a `?`.
- **repo-r** (plan `supported`, latest step `2.4` "Fix the date parser"; `latest_handoff` the heuristic link):
  `6011` `idle` (record), bursts 05:29:00–05:33:30 and 06:16:00–06:24:10 — the second exists only because its
  subagent file (`agent-a6011sub0000001`, 06:19:00–06:22:00) bridges a 6:44 gap in the main file; child `6013`
  (`unknown` / `turns_complete`, burst 06:23:57–06:24:40, `state_since` 06:24:40); `6012` `idle`, burst
  05:23:00–05:28:00, first user record is a `<local-command-caveat>` wrapper (skipped), `asked` has a `?`;
  `6014` the **all-import rollout** at 06:13:00 (`imported` true, `activity` `[]`, `asked` `null`,
  `state_since` `null`). The idle stretch 05:33:30–06:16:00 (42.5 min) has no burst on any shown row.
- **repo-t** (plan `missing`): only `6051`, an all-import rollout with two imported turns at 05:50:00.
- **repo-s** (plan `missing`): only `6041`, a record-only `idle` session last active 2026-10-30; listed (records
  always are) with no activity in the window.
- Header counts for the home golden (design W4, imports not counted): 2 working now (`6002`, `6031`), 1
  waiting, 5 idle (`6003`, `6004`, `6011`, `6012`, `6041`), 3 done or unknown (`6021`, `6022`, `6013`), 5
  repos, 2 hand-offs; 2 imports hidden (`6014`, `6051`).

## `graph`

One PLAN.md step across both harnesses (design G0–G7, board 9). Step 8.3 builds it; `multi-link graph` without
`--json` prints a plain text version of the same report. **Like `sessions`, the JSON exports prompt text** (`asked`,
`said`): real reports stay in scratch (W10).

```json
{
  "now": "2026-11-06T18:00:00.000Z",
  "repo": "/home/dev/repo-g",
  "step": { "id": "1.1", "title": "Date parser: reject empty input", "verify": "cargo test --workspace" },
  "commit": null,
  "parts": [ Part, ... ],
  "empty": null
}
```
| Field | Type | Rule |
| --- | --- | --- |
| `now` | timestamp | The report's clock (`--now`). |
| `repo` | string | The session `repo` given to `--repo`. |
| `step.id` | string | As given to `--step`. Never a number: `1.10` ≠ `1.1`. |
| `step.title` | string \| null | PLAN.md heading text; `null` when the plan has no such step. |
| `step.verify` | string \| null | `steps` → Step `verify_command`. Always reported text, never a result (G7). |
| `commit` | `{"hash": string, "at": timestamp}` \| null | Newest `git log` commit whose subject holds exactly `(Step <id>)`: `hash` the 7-character short hash, `at` the commit time. `null` when none, and always in fixture mode (no git). |
| `parts` | Part[] | Chains by first hand-off, each chain in time order. `[]` when `empty` is set. |
| `empty` | string \| null | `no hand-offs tagged step <id> (a Claude-only step, or its prompt did not name it)` when no owned hand-off has this step; else `null`. |

The report reads every session with `--since all`, whatever the TUI's window, so bursts are unclipped (G2).

### Part
Every field is present on every part.

| Field | Type | Rule |
| --- | --- | --- |
| `chain` | int | 0-based chain index (G2). |
| `kind` | string | `plan`, `implement`, `fix round N`, `review` or `verify + commit` (G3); numbering restarts per chain. |
| `harness` | `"claude"` \| `"codex"` | |
| `session` | string \| null | Claude: the parent. Codex: the child; `null` for `unlinked` and `ambiguous`. |
| `from`, `to` | timestamp \| null | Claude: the cut times (G2). Codex: first turn's `task_started`; last turn's `task_complete`, or its last record when open. `null` with no run. |
| `status` | string \| null | Codex: `done`, `working`, `stale`, `unlinked`, `ambiguous` or `unavailable` (design, pinned G4). Claude: `null`. |
| `run_available` | bool \| null | Codex: `true` when the run's turns were read. `false` for the last three statuses. Claude: `null`. Independent of `edge_in.match`: an `exact` link can have `false`. |
| `active_ms` | int | Sum of burst ∩ part (Claude) or burst ∩ turn (Codex), in ms. 0 with no run. |
| `stretches` | int | Number of those pieces. 0 with no run. |
| `asked` | string \| null | Codex: the first turn's first `user_message`, normalized like W3 (119 characters and `…` above 120). Claude: `null`. |
| `said` | string \| null | Codex: the last turn's `task_complete` `last_agent_message`, normalized the same way; `null` if absent or open. Claude: `null`. |
| `counts` | `{"reads": int, "edits": int, "runs": int}` | G5 kinds of the calls the part holds (design, pinned G6). Hand-offs are not counted. |
| `notable` | `{"label": string, "ok": bool \| null}`[] | At most 3, newest last (G6). `ok` null on hand-offs and on calls with no exit evidence. |
| `waiting_reads` | int | Claude: reads of this chain made after this part and before the next Claude part (G6). Codex: 0. |
| `untagged_handoffs` | int | Claude: owned hand-offs with `step: null` and `from < at ≤ to`. Codex: 0. |
| `edge_in` | Edge \| null | What arrives at this part. `null` on a `plan` and on a Claude part after a run with no interval. |

Edge is one of:
- `{"kind": "handoff", "at": timestamp, "match": "exact"|"heuristic"|"ambiguous"|"unlinked", "parent": string, "parent_part":
  int|null, "gap_ms": int|null, "candidates": int}` on every Codex part. `at`, `match`, `parent` are the link's; `parent_part`
  is the index in `parts` of the Claude part that holds or precedes the hand-off (`null` when the chain has none yet);
  `gap_ms` = `from − at`, `null` with no run; `candidates` = the link's `candidates` length.
- `{"kind": "return", "at": timestamp}` on a Claude part that starts at a run's end; `at` = that part's `from`.

### Calls and turns (internal, Step 8.2)
Not in any report, per G4 and G5. `Call {id, at, kind, label, ok}`: `id` is the Claude tool id or the Codex `call_id`, so
`graph` can test a link's `invocation` against the parent's owned calls (design, pinned G1). `Turn {start, completed_at,
last_record_at, prompt, last_agent_message}`: `completed_at` is the `task_complete` time, null while open; `last_record_at`
is the turn's last record, always set, and gives an open run's `to`; `prompt` and `last_agent_message` may each be null, and
a completed turn with a null message is still `done`. Observed on
2026-11-06 and undocumented, the Codex shapes the fixture copies are: a `response_item` `custom_tool_call` with `name: "exec"`,
`call_id`, and `input` holding JavaScript text such as `text(await tools.exec_command({cmd:"…",max_output_tokens:3000}));` or
`text(await tools.apply_patch("*** Begin Patch\n*** Update File: …"));`, then a `custom_tool_call_output` with the same
`call_id` whose `output` blocks hold `{"chunk_id":…,"exit_code":N,…}` as text; and a `function_call` with `name: "js"` and a
`function_call_output`.

### Phase 8 fixture results (`phase8/graph-<id>.json`)
Verify command, once per key (Step 8.3 adds the test): `multi-link graph --root fixtures/sessions-phase8 --now
2026-11-06T18:00:00Z --repo /home/dev/repo-g --step <id> --json` must equal `fixtures/expected/phase8/graph-<id>.json`
(parsed). The keys were typed by hand from the fixture's timelines and cross-checked by an independent scratch script that
recomputes every part from the fixture lines and `links`. The root is synthetic: invented prompts, one repo
(`/home/dev/repo-g`, plan at `repos/repo-g/`), no session records, no hook folder, no task output. Ids are
`<tag>0000-0000-4000-8000-00000000<tag>` with a 4-character hex tag, named by tag below. Times are UTC on 2026-11-06.
`sessions --since all`: 28 sessions, 0 skipped lines. `links`: 26, one of them the fork's copy; `unparented`: `8221`, `8222`.

| Step | Sessions | What it pins |
| --- | --- | --- |
| 1.1 | `8101` → `8201` | Board 9's shape: `plan`, `implement`, `review`, `fix round 1`, `verify + commit`. Background `exec` matched by time (`heuristic`), six reads during the run, then a foreground `resume` (`exact`, `resume_arg`). Two `python3 … write` commands count as runs. Sentinels below. The golden |
| 1.2 | `8102`; `8221`, `8222` | `unlinked`, then `ambiguous` (2 candidates): no interval, Claude's time unbroken, no return edge |
| 1.3 | `8103` → `8203` | Seven parts (scroll), every link `exact`. A turn with a 5 min 19 s silence: `active_ms` 60000 in 2 stretches. `$ cargo test` `ok: false`, `write_stdin`, `function_call` `js` with `ok` null. Parent silent during the run: two Claude bursts |
| 1.4 | `8141` → `8241`, `8142` → `8242` | Two parents, two chains, `plan` again in the second |
| 1.5 | `8105` → `8251`, `8252` | A hand-off during a run: two neighbouring Codex parts, the second hand-off a notable of `plan`, `waiting_reads` 3. `8252`'s turn is open: `working` at the fixed `--now` (`stale` once `now` is past 18:03:30), no last part |
| 1.6, 1.7 | `8106` → `8261`, `8271` | A → B → A. 1.6 has two chains; the first ends at B's hand-off 11:03:00, the second starts at B's run end 11:04:00. 1.7's chain starts at A's run end 11:02:00 and ends at A's hand-off 11:05:00. A resume opening a chain is `implement` with `· step` |
| 1.8 | `8108` → `8281`, `8282` | A → untagged → A: one chain, `untagged_handoffs` 1 on the `review` holding it, `8282` drawn nowhere |
| 1.9, 1.10 | `8109` → `8291` | One rollout, three turns in one burst, three hand-offs, the last tagged 1.10. 1.9 stops at turn 2; 1.10 holds only turn 3 |
| 1.11 | `8111` → `82a1` | `exact` by stdout, no rollout: `unavailable`, `run_available` false, `session` set, `gap_ms` null |
| 1.12 | `8112` (+ subagent `a8112sub0000001`), `8113` → `82c1` | A hand-off from a subagent file: the subagent's read counts for the parent. `8113` is a fork: its file copies `8112`'s main records with `sessionId` `8112`, including the 07:02:40 hand-off, which `links` lists again under `8113`. That link is not owned, so it opens no chain; the copied records are no activity, so `8113`'s chain starts 07:05:00, not 07:00:00 |
| 1.13 | `8114` → `82d1` | Only an untagged hand-off: `empty`, `parts` `[]` |

**Sentinels:** `sentinel-path-81`, `sentinel-token-81` (Claude commands in 1.1) and `sentinel-path-82`, `sentinel-token-82`
(a Codex command in 1.1) must never appear in a report (Step 8.3 tests this).

## Open

Phase 3, FR18, design T7. `open::action(&Session, fork: bool) -> OpenAction` is a pure function of the
Session (no I/O); the CLI (`multi-link open <id-prefix> [--fork] [--dry-run]`, Step 3.4) and the TUI only
execute what it returns. Before executing, the caller recomputes the Session from a fresh pass for that
one full id, never from the last drawn report (Codex P1 #2).

### OpenAction (JSON, as `open --dry-run` prints it)
| `action` | Other fields | Executed as |
| --- | --- | --- |
| `"resume"` | `program`: `"claude"`; `args`: `["--resume", id]`, plus `"--fork-session"` when `fork`; `cwd`: Session `cwd` | Spawn `program args…` in `cwd` with inherited stdio (no shell), wait for it. |
| `"app_link"` | `url`: `"claude://code/continue?session=" + host_session_id` | `open <url>` (macOS). Undocumented app behaviour, tested once (2026-10-29). |
| `"unavailable"` | `reason` (below); `command`: string \| null, the command for me to run myself; `cwd`: Session `cwd` | Nothing runs. CLI: reason on stderr, exit 3; TUI: footer message with `command`. |

**Phase 9 (R3) inputs.** *Entrypoint* = the `entrypoint` string of the last main-transcript record that has one
(own `sessionId` or none); observed values `cli`, `sdk-cli`, `claude-desktop`. *App mapping* = app records (Step 8.4c
reader) whose `cliSessionId` equals the session id; their `app_id` (`sessionId`, else the file stem). Both reach
`open::action` through an internal core resolver; Session gains no key (R0). Why the desktop rule: resuming a desktop
session in a terminal makes two writers if the app reopens it, and the app link cannot fork.

### Rules, checked top to bottom (first match wins)
| Session | `fork` | Action | `reason` / `command` |
| --- | --- | --- | --- |
| `id` is not a lowercase uuid (8-4-4-4-12 hex) | any | `unavailable` | `unsafe_id`, `command: null` (ids reach argv and text; a file stem could start with `-`) |
| Codex (any state) | false | `unavailable` | `codex_not_supported`, `codex resume <id>` |
| Codex (any state) | true | `unavailable` | `codex_not_supported`, `codex fork <id>` |
| Claude `finished` / `process_exit`, `transcript` not null | any | `resume` | (positive exit evidence: record + process gone or pid reused; entrypoint not consulted, as before Phase 9) |
| *(R3)* Claude `finished` / `transcript_exit` or `session_end`, entrypoint `cli` or `sdk-cli` | any | `resume` | |
| *(R3)* same, entrypoint `claude-desktop`, exactly one app record maps the id to an `app_id` matching `^local_[A-Za-z0-9-]+$` | false | `app_link` | `url`: `claude://code/continue?session=<app_id>` |
| *(R3)* same, entrypoint `claude-desktop`, otherwise (no mapping, two or more, an unsafe `app_id`, or `fork`) | any | `unavailable` | `app_session`, `claude --resume <id>` (+ ` --fork-session`) |
| *(R3)* same, entrypoint missing or any other value | any | `unavailable` | `unknown_entrypoint`, `claude --resume <id>` (+ ` --fork-session`) |
| Claude `finished`, `transcript` null (record-only) | any | `unavailable` | `no_transcript`, `claude --resume <id>` (+ ` --fork-session`) |
| Claude `busy` / `idle` / `waiting` / `stale`, `host_session_id` matches `^local_[A-Za-z0-9-]+$` | false | `app_link` | |
| Claude `busy` / `idle` / `waiting` / `stale`, `host_session_id` matches | true | `unavailable` | `fork_needs_finished`, `claude --resume <id> --fork-session` (the app link cannot fork, and multi-link resumes nothing that may be running) |
| Claude `busy` / `idle` / `waiting` / `stale`, `host_session_id` null or not matching | any | `unavailable` | `running_without_app_link`, `claude --resume <id>` (+ ` --fork-session`) |
| Claude `unknown` | any | `unavailable` | `not_known_finished`, `claude --resume <id>` (+ ` --fork-session`) |

**Lookup.** `open <id-prefix>` and the fresh pass before executing search every parsed session with the
`--since` window off (an old finished session is exactly the one to resume). Prefix matching is on full
ids. No match → `no session matches <prefix>` on stderr, exit 4; several → exit 2 listing them. TUI: if
the fresh pass no longer finds the id, a footer message and nothing runs.

**Known limits (Phases 3–8; Phase 9 adds clean-exit resume for terminal sessions, R3).** Resume is reachable only when a record outlives its process (a crash, a kill); if
Claude deletes its record on a clean exit (Step 3.8 checks), a normally closed session is `unknown` →
`not_known_finished`, so FR18 resume is crash-only in practice, by design of the safety rule. A second
process that writes no record (`claude -p --resume <id>`) while the old record remains but its process
has exited cannot be seen; such a session would show `finished` and could be resumed twice. Fork stays refused for `unknown`: the original may be live and
mid-write, and the command text is offered instead.

`command` strings use single spaces and the full id. `resume` is the only action that starts a harness,
and it needs `state: finished`, which before Phase 9 only the process probe could produce. Since Phase 9 a
transcript `cost-state` or a `SessionEnd` record also gives `finished`, in fixture mode too, so the Phase 9 root's
terminal sessions resume (`open.json`); the rest of this paragraph describes the probe-only rows. When the probe cannot run
(fixture mode, `ps` denied) no session is `finished` (State), so nothing resumes; record sessions then
follow their record status (`app_link` or `unavailable`), and opening the app view resumes nothing in this
terminal. A `waiting` / `hook` session has no record, so it is `running_without_app_link`.

Fixture coverage (`--root fixtures/sessions`, probe off, so no `resume`): the five desktop records (busy
6358ec7e, idle 1f47e9b3, f8017fcd, 9d02a552, bed52539) → `app_link`; the synthetic busy `70001` record (no
`hostSessionId`) → `running_without_app_link`; every transcript-only Claude → `not_known_finished`, except, since
Phase 9, the 12 `transcript_exit` sessions (all `claude-desktop`, no app records in that root) → `app_session`, so
the root still resumes nothing; every
Codex → `codex_not_supported`; with `fork`, the desktop records → `fork_needs_finished`. Hook variant:
d1f3b188 (`waiting`, `entrypoint: cli`, no `hostSessionId`) and 11c1f13c (`waiting` / `hook`) →
`running_without_app_link` (Phase 9: 11c1f13c is `finished` / `transcript_exit` → `app_session`). `resume` and `no_transcript` are unit tests with `FakeProcessProbe`.

### Step 8.4b presentation amendment
`title` comes from the last non-blank string `customTitle` in an own (`sessionId == id`), main-file
`custom-title` record. Remove control characters, trim outer whitespace, and ignore empty results,
non-strings, malformed records, inherited records and subagent records. It is copied to SessionRow,
including children and merged members, and refreshed when a transcript changes.
Labels prefer title › placing/made step › `last_asked` › `asked` › harness no-prompt text.
Reject each prompt candidate beginning with the app's `Background (from the session that started this one):`
preamble independently; JSON prompts stay unchanged. The placing-parent rule for Codex is unchanged.
The repo header adds `· latest hand-off step <id>` when the latest tagged hand-off differs from the latest
logged step. Place words show the memoised current main branch (recorded branch as fallback) or
`worktree <folder>` for a linked checkout. Place is display metadata excluded from JSON; `branch` stays recorded.

## Phase 9 fixture results (`fixtures/sessions-phase9/`, Step 9.0)
Synthetic root (W10: no real prompt, id or path), `--now 2026-11-09T12:00:00Z`, `--since all` unless noted. Every
id's first two hex digits name the step whose rule it tests (`91…` = 9.1, …, `95…` = 9.5). Keys in
`fixtures/expected/phase9/` are the **final** Phase 9 output; step 9.N compares the ids with prefixes `91`–`9N`
(`fixtures/expected/phase9/check.sh <N>`), so a later step's cases never block an earlier step and an earlier
step's cases are re-checked at every later one. The full keys (including `skipped: []`) hold from 9.4 on.

**Compressed rollouts (R1).** Discovery takes `rollout-*.jsonl` and `rollout-*.jsonl.zst` under
`.codex/sessions`. A `.zst` file is one or more zstd frames holding the same JSONL; it is decoded (pure-Rust
`ruzstd`) and parsed by the same Codex parser. `transcript` is the path of the file read, so a `.zst` session's
ends in `.jsonl.zst`. The same id in both forms → the plain file is read and the `.zst` ignored (no diagnostic).
Decoded size is capped at 64 MiB (`.zst` only); over the cap or failing to decode → one `unreadable` entry
(stderr `unreadable: N`), never an error exit and never a session. Cached by file key like plain reads.

| Ids | Case | Expected |
| --- | --- | --- |
| `91000001` (repo-p9a) → `91c00001` | Claude hand-off `Step 1.1`, id in stdout; the Codex run exists only as `.zst` | link `exact` / `stdout` (already); session `91c00001` listed with the `.zst` path; `graph-1.1.json`: the part has its calls, turns and `status: done` |
| `91c00002` (repo-p9b) | same id as plain `.jsonl` (two turns) and `.jsonl.zst` (first turn only) | one session, plain path, two turns' values; `unparented` |
| `91c00003` | `.jsonl.zst` with a zstd magic number and no valid frame | no session; stderr `unreadable: 1`; exit 0 |
| `92000001` | clean exit, `cli` | `finished` / `transcript_exit`; open and fork → `resume` |
| `92000002` | clean exit, `claude-desktop`, one app record (`local_92f00002-…`) | `finished` / `transcript_exit`; open → `app_link`; fork → `app_session` |
| `92000003` / `92000004` | clean exit, `claude-desktop`, no / two app records | `finished` / `transcript_exit`; `app_session` |
| `92000005` / `92000006` | clean exit, `sdk-cli` / no `entrypoint` | `finished` / `transcript_exit`; `resume` / `unknown_entrypoint` |
| `92000007` | a turn after the `cost-state` (resumed) | `unknown` / `no_signal`; `not_known_finished` |
| `92000008` | `cost-state`, then the `/exit` echo (meta caveat, `<command-name>`, `<local-command-stdout>`) | `finished` / `transcript_exit` (echoes are not turns); `resume` |
| `92000009` | SIGKILL: record kept (`busy`, `statusUpdatedAt` 05:30:30Z), no `cost-state` | fixture mode trusts the record: `stale` / `session_record`; `running_without_app_link`. `finished` / `process_exit` with `FakeProcessProbe::Gone` is a core test |
| `92000010` | live record (`idle`) and a `cost-state` after the last turn | the record wins: `idle` / `session_record` |
| `92000011` | uncleared hook `waiting` and a `cost-state` after the last turn | `finished` / `transcript_exit` (before hook `waiting`); `resume` |
| `92000012` | uncleared hook `waiting`, no `cost-state` | `waiting` / `hook` (today's rule; `state_since` 06:02:00Z) |
| `92000013` / `92000014` | after the `cost-state`, only a non-meta `user` prompt / only a `tool_result` record (no `assistant`) | `unknown` / `no_signal` (each counts as a turn); `not_known_finished` |
| `93000001` (repo-r1) | `> /tmp/dev/p9/r1.log 2>&1`, log holds one id, rollout 5 s later | `exact` / `redirect_file` (today `heuristic`) |
| `93000002` | log without an id, rollout present | `heuristic` |
| `93000003` | `>>`, log holds an older id and this run's id | `heuristic` (two ids → no exact) |
| `93000004` | `> /home/dev/elsewhere/r4.log` (outside cwd and temp; the file exists and holds an id) | `unlinked` (not read) |
| `93000005` | `> "$S/r5.log"` | `unlinked` |
| `93000006` | `cd sub && codex exec … > out.log` (both `out.log` files exist with an id) | `unlinked` (relative path after `cd`; heuristic off) |
| `93000007` | two stdout redirects, both files hold an id | `unlinked` |
| `93000008` | relative `> logs/r8.log`, no `cd` | `exact` / `redirect_file` |
| `93000009` / `9300000a` / `9300000e` | `&>`, `>>` with one id, `1>`; no rollout on disk | `exact` / `redirect_file` (the file alone is the evidence) |
| `9300000b` | log gone; hook records `child: null` then `redirect_file` | `exact` / `hook` |
| `9300000c` | log gone, no hook record, rollout present | `heuristic` |
| `9300000d` | background call (`run_in_background`), redirect log holds the id | `exact` / `redirect_file` |
| `94000001` | `ended` (`prompt_input_exit`) after the last turn, and a `cost-state` | `finished` / `session_end` (before `transcript_exit`); `resume` |
| `94000002` → `94000003` | `ended` (`clear`) on the old id; the new id has its own transcript and a live `idle` record | old `finished` / `session_end` (`resume`); new `idle` / `session_record` |
| `94000004` | `ended`, then a turn 3 min later (resumed) | `unknown` / `no_signal` |
| `94000006` | live record (`idle`) and an `ended` record | the record wins: `idle` / `session_record` |
| `94000007` | `ended`, then a hook `waiting` record 1 min later; no `cost-state`, no later turn | the later hook record clears `ended`: `waiting` / `hook` (`state_since` 08:12:00Z); `running_without_app_link` |
| `95c00001` (repo-p9l) | only incoming hand-off: `Step 2.1` from `95000001`, 5 h before now | `lanes.json`: `s` opens step 2.1 with the 2 h window |
| `95c00002` | `Step 2.2` from same-repo `95000002` (6 h before), then `codex exec resume` with `Step 7.1` from `95000003` in repo-p9m (90 min before) | `s` opens 2.2 (same-repo rule beats the newer cross-repo parent) |
| `95c00004` | untagged hand-off from `95000004` (inside the window) | footer `no step on this session` |

`skipped` is `[]`: the five `ended` lines and the `redirect_file` hand-off line are valid from 9.4 / 9.3 on (until
then each is one `unknown_type` entry).
