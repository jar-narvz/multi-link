# Phase 2 — hook payload spike (Step 2.1)

Claude Code 2.1.303, codex-cli 0.164.0, run 2026-10-30 (UTC) in `~/multi-link-sandbox/repo-a`.
A throwaway capture hook in that repo's own `.claude/settings.local.json` (project scope, removed
afterwards) listened on `PostToolUse` (matcher `Bash`), `PermissionRequest`, `Notification` (all types),
`PostToolUseFailure` and `Stop`. Every entry was `"async": true` and ran
`python3 -I capture.py <event> <mode> >/dev/null 2>&1 || true`, so it printed nothing and always exited 0.
`PostToolUse` had three entries: `plain` (write at once), `sleep10` (log a start line, sleep 10 s, then
write) and `poll` (for a background call, poll the task output file for `session id:`, then write).

Stored data: `~/.multi-link/spike/<event>.jsonl` holds the allowlist only. `session_id`,
`transcript_path`, `cwd`, `hook_event_name`, `tool_name`, `tool_use_id`, `notification_type` and
`permission_mode` are kept verbatim. Every other string is replaced by `<string:N>` (its length), and
booleans and numbers are kept. Found values are added under `_found`: `/….output` paths and
`session id:` / `Thread ready (` uuids. The hook also stores its own receive time `_recv`. Raw
payloads, the capture script, the `expect` script, its run logs (`hold-perm-i.log`, `hold-deny-i.log`:
UTC stamps plus `~/.claude/sessions/<pid>.json` snapshots) and the case → session id list are in the
git-ignored `fixtures/raw/hook/`. Sandbox prompts were placeholders. This document names fields and
counts only.

Cases (all `claude -p --model claude-haiku-5-5 --session-id <uuid>` unless marked interactive):

| Case | What ran | Hook events seen |
| --- | --- | --- |
| fg | foreground `codex exec` | PostToolUse, Stop |
| bg | `codex exec` with `run_in_background`, then waited for it | PostToolUse, Stop ×2 |
| bg2 | `codex exec` with `run_in_background`, reply without waiting | PostToolUse, Stop ×2 |
| resume + devnull | `codex exec resume <uuid>`, then `codex exec … > /dev/null 2>&1` | PostToolUse ×2, Stop |
| plugin | Skill `codex:rescue` → Agent `codex:codex-rescue` → `node …/codex-companion.mjs task` | PostToolUse ×2 (parent + subagent), Stop ×2 |
| perm-p | non-allowed `touch` (auto-denied in `-p`) | PermissionRequest, Stop |
| fail, fail2 | `codex exec` with a bad flag; `codex exec -m <no such model>` (exit ≠ 0) | PostToolUseFailure, Stop |
| perm-i (interactive, `expect`) | `touch`, prompt held 30 s, then answered Yes | PermissionRequest, Notification, PostToolUse, Stop |
| deny-i (interactive, `expect`) | `touch`, prompt held 10 s, then Esc | PermissionRequest, Notification |

Two more interactive runs were discarded as setup errors; their findings are under (j).

## (e) Do `settings.local.json` hooks run in `-p`?

Yes. All 8 `-p` sessions fired hooks from `repo-a/.claude/settings.local.json`, and so did all 4
interactive sessions. Nothing was written to `~/.claude/settings.json`. In `-p`, project hooks ran with no
trust prompt. Interactively, the folder-trust dialog has to be accepted first (see (j)).

Decision: go

## (f) Does `session_id` equal the transcript stem, and `tool_use_id` the transcript's `tool_use.id`?

Yes. Checked on every record whose transcript exists: 8 `PostToolUse` and 2 `PostToolUseFailure` records
(which carry `tool_use_id` too). The two discarded interactive runs (see (j)) wrote no transcript.
- `session_id` equals the stem of `transcript_path` in every record of every event (41 records).
- Each `PostToolUse` `tool_use_id` appears exactly once as an `assistant` `tool_use.id`: in
  `transcript_path` itself, or for the plugin's subagent call in `<stem>/subagents/agent-*.jsonl`.
  The check over `~/.multi-link/spike/PostToolUse{,Failure}.jsonl` printed one match for each of the 10.
- Subagent call: `session_id` and `transcript_path` are the **parent's**. `agent_id` (17 chars) and
  `agent_type` (`codex:codex-rescue`) are added, and `<parent stem>/subagents/agent-<agent_id>.jsonl` exists
  (`_agent_file_exists: true`). The `tool_use_id` is found in that subagent file, not in the parent
  transcript. The parent's own Bash call (`task-resume-candidate`) has no `agent_id`.
- `PermissionRequest` and `Notification` carry **no** `tool_use_id`. They join a session only through
  `session_id`. `PermissionRequest` also has `tool_name` and `tool_input`; `Notification` has neither.

Child ids found by the hook (from `tool_response.stdout`, or `error` for a failure) equal the rollout
in `~/.codex/sessions` for fg, bg2 (via the task file), resume (the resumed id), plugin
(`Thread ready (`) and fail2. devnull had no id (stdout empty), as expected.

Decision: go

## (g) Background `tool_response`, the task output path, and when `session id:` appears

- `tool_response` for a background Bash call has `stdout` (length 0), `stderr` (0), `interrupted`,
  `isImage`, `noOutputExpected` and `backgroundTaskId` (9 chars). It has **no path**. A foreground call
  has the same keys minus `backgroundTaskId`. `tool_input.run_in_background` is `true`.
- The output path is not in the payload, but it can be derived:
  `dirname(scratchpad_dir) + "/tasks/" + backgroundTaskId + ".output"`. `scratchpad_dir` is a
  top-level payload field (`<tmp>/claude-<uid>/<project slug>/<session_id>/scratchpad`). For both
  background cases, the derived path occurs verbatim in the transcript's `tool_result` text
  ("Output is being written to: …"), and in bg2 it already existed when the hook received the payload.
- Timing: the `poll` hook found the `session id:` line within **316 ms** of receiving the payload
  (bg2, one sample: `_recv` → `_wrote`; 314 ms of that was polling). This is an upper bound on when
  the line appeared, not its exact time. In bg the derivation wasn't in place yet, so there's no timing for it.
- `Stop` also carries `background_tasks[]` (`id`, `type`, `status`, `description`, `command`), listing the
  task while it is still running.

Decision: go (derive the path from `scratchpad_dir`. If that field is absent, record the hand-off with
child `null`)

## (h) Which events fire for a permission prompt, with which `notification_type`?

- `-p` (perm-p): **`PermissionRequest` only**. No `Notification`, no `PostToolUseFailure`. It fires
  even though `-p` auto-denies, and *after* the denial is written: the `tool_result` (`is_error: true`)
  line is 0.11 s older than the hook's `_recv`.
- Interactive (perm-i, deny-i, and the discarded env run): `PermissionRequest` 0.16–0.22 s after
  the `tool_use` line, then `Notification` with `notification_type: "permission_prompt"` 5.9–6.0 s
  after `PermissionRequest` (3 runs), while the prompt was still open.
- `idle_prompt` was seen once: about 60 s after a turn ended in an interactive session. `elicitation_dialog`
  was not exercised.
- Payloads carry no event time. The hook must stamp its own receive time, which trails the
  event by about 0.1–0.2 s because the hook is async.

Decision: go (`PermissionRequest` is the earliest and fires in both modes; `Notification`
`permission_prompt` is a later, interactive-only duplicate)

## (i) Between prompt and answer, does Claude write a transcript line or a session-record update?

- **Transcript: no.** In perm-i, the `tool_use` line is at prompt time. The next timestamped line is
  the `tool_result`, 0.56 s after the answer was sent (30 s later). deny-i is the same: the
  `tool_result` (`is_error: true`) and an interrupt `user` text line come right after Esc.
  Transcript writes are buffered: the `PostToolUse` hook ran 0.15 s after the `tool_result`
  timestamp, but the file size had not changed yet. Compare line timestamps, never file size or
  mtime.
- **Session record: yes. This contradicts Phase 0 (b).** `~/.claude/sessions/<pid>.json` changed to
  `status: "waiting"` with a new field `waitingFor: "permission prompt"` at the prompt
  (`statusUpdatedAt` 8 ms before `expect` saw the prompt; snapshots in `hold-perm-i.log`, `hold-deny-i.log`). It stayed unchanged through the hold:
  snapshots 2 s in and 2 s before the answer were identical. It then became `status: "idle"`,
  with no `waitingFor`, after the answer (perm-i) and after Esc (deny-i). Phase 0's sample had 5
  records and none was waiting, so `waiting` was never seen there.
- Clearing signal: the first transcript line timestamped after the waiting record (the `tool_result`
  written on answer or denial). Equivalently, the session record leaving `status: "waiting"`. A
  `PostToolUse` / `PostToolUseFailure` with the same session also follows an answer.

Decision: go. H8's "newer than every transcript line" rule holds on line timestamps. **Plan impact
for the coordinator:** an interactive Claude session's own record reports waiting
(`status: "waiting"` + `waitingFor`) without the hook. Step 2.4's `waiting` row may want the record
as its primary source and the hook as a fallback. That decision belongs to PLAN.md, not this step.

## (j) Can an `expect`-driven interactive session be held at a prompt unattended?

Yes. `fixtures/raw/hook/hold.exp` held perm-i for 30 s and deny-i for 10 s, then answered (Enter = Yes)
or rejected (Esc = interrupt, which ends the turn), with no human input. It needs all three of these:
1. **A clean environment** (`env -i HOME PATH USER LOGNAME SHELL TERM LANG`). Both discarded runs
   inherited the environment of the Claude desktop session that started them. Their child fired
   hooks but wrote **no transcript and no session record**.
2. **`--permission-mode default`.** The owner's default mode is `auto`, which approved the `touch` with no
   prompt (first discarded run).
3. **Accepting the folder-trust dialog.** Its default is "No, exit", so the script sends Down then
   Enter. Claude Code stores that trust in its own config (harness-written, like transcripts). multi-link
   wrote nothing there.

Decision: go

## (k) Does an async hook that is still running survive until it writes?

The `sleep10` hook (start line logged, 10 s sleep, then write):
- **`-p`: killed at teardown.** 4 of 4 runs whose session ended less than 10 s after the hook started
  logged a start line and never wrote (fg, resume, devnull, plugin subagent call). 3 runs survived
  because their `-p` session stayed alive past 10 s (bg, bg2, plugin parent call). A `-p` session with a
  background task stays alive until the task ends, and `Stop` fires again then.
- **Interactive: survives exit.** 3 of 3 wrote. perm-i wrote at 06:08:53.121 (`PostToolUse-sleep10.jsonl`
  `_wrote`), and `expect` saw the process exit (EOF after Ctrl-C ×2) at 06:08:48.032
  (`hold-perm-i.log` `exited`). For the two discarded runs only completion is recorded, not the exit time.

Decision: no-go for "poll, then write once" in `-p`. As planned, 2.5 writes the hand-off record
first (child `null`) and appends a second record when the id appears. The 314 ms poll in (g) makes the
second record likely, but not guaranteed, in `-p`.

## Other findings for Steps 2.3–2.8

- **`PostToolUseFailure` replaces `PostToolUse` when the command exits non-zero.** It has `error`
  (string), `is_interrupt` and `duration_ms`, but no `tool_response`. A Codex run that started and then
  failed (fail2) has its `session id:` line in `error`, and that id is a real rollout. Recommendation:
  the hook also listens on `PostToolUseFailure` (matcher `Bash`) and reads `error` like `stdout`.
- Undocumented payload fields seen on every event: `scratchpad_dir`, `prompt_id` (36 chars), and
  `effort.level` (absent on `Notification`). `PostToolUse` has `duration_ms`. `PermissionRequest` has
  `permission_suggestions[]`. `Stop` has `stop_hook_active`, `last_assistant_message`,
  `background_tasks[]` and `session_crons[]`. `Notification` has `message` but no `permission_mode`.
- In the `-p` denial, the waiting record (`PermissionRequest`) is newer than the denial line. Under H8 it
  is cleared only by the next line (the assistant reply, 1 s later), or by the process ending (`finished`
  is checked first).

## Verify

Commands and their output are in the PROGRESS.md entry for Step 2.1. Because Step 2.2 runs in
parallel, the coordinator adds that entry. `git status --short` then also lists 2.2's files.
