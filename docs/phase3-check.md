# Phase 3 real check — 2026-10-30

**Step 3.8 whole-phase Verify passes; Phase 3 / Release 1 is complete with the limitations
below.** Codex's sandbox run and Claude's host results are recorded separately. The resize/key
issue is in crossterm 0.29, not multi-link; a draining resize run passes. Cleanly exited terminal
Claude sessions delete their records, leaving `unknown` / `Unavailable` with a resume command.
No code changed. Claude reviews.

## Environment and privacy

Codex sandbox run: macOS, release build, real `~` (no `--root` or fixed `--now`), default history window.
Rust commands used the AGENTS.md PATH prefix. `/usr/bin/expect` drove the PTYs with
`TERM=xterm-256color`. Captures and helpers stayed in temporary storage outside the repository.
The screen evidence below is a whitelist projection of captured repo names: session rows,
PLAN/PROGRESS titles, ids, prompts and paths are omitted. No transcripts were changed.

Process probing was unavailable in the Codex sandbox run. Actual command and error:

```text
/bin/ps -p $$ -o lstart=
zsh:1: operation not permitted: /bin/ps
```

Both agent directories were readable by the timing example and TUI; no agent-directory read
error was observed. Timings therefore cover real-home scanning, overview and plan parsing,
but not successful process probing. Claude supplied the host timing and liveness-dependent
checks below with working `ps`.

## Codex sandbox run: real-home refresh timing

Command (exit 0):

```sh
cargo run --release -q -p multi-link-core --example refresh_timing -- 3
```

Actual output:

```text
pass 1: 3794 ms, 25 sessions, 4 repos, 441 files re-parsed, 508526132 bytes re-parsed
pass 2: 23 ms, 25 sessions, 4 repos, 1 files re-parsed, 1216599 bytes re-parsed
pass 3: 35 ms, 25 sessions, 4 repos, 2 files re-parsed, 1654498 bytes re-parsed
```

Passes 2 and 3 meet the 2000 ms target, each with changed files re-parsed. Host confirmation
with process probes is recorded below.

## Codex sandbox run: PTY real home, no-resize control — pass

Expect spawned `/bin/sh -c` with this sequence: `stty -a` to a before file, the absolute
repo-built `target/release/multi-link tui`, capture its status, `stty -a` to an after file,
then print `TUI_EXIT`. The binary path was constructed from `[pwd]`, not a fixture root.
The PTY began at 40 rows by 160 columns. Expect waited for `quit`, then a repo name in the
real report (`route-impact`), waited 5500 ms, sent `q`, and waited for status and EOF.

Actual Expect output:

```text
REAL_REPORT=seen
TUI_EXIT=0
PTY_WAIT=0
```

Scrubbed screen projection (repo names only, from the capture):

```text
multi-link
route-impact
jarednarvaez-app
posting-triage
```

The capture contained the alternate-screen leave sequence. Before and after `stty -a`
were identical; the complete output for both was:

```text
speed 9600 baud; 40 rows; 160 columns;
lflags: icanon isig iexten echo echoe -echok echoke -echonl echoctl
	-echoprt -altwerase -noflsh -tostop -flusho -pendin -nokerninfo
	-extproc
iflags: -istrip icrnl -inlcr -igncr ixon -ixoff ixany imaxbel -iutf8
	-ignbrk brkint -inpck -ignpar -parmrk
oflags: opost onlcr -oxtabs -onocr -onlret
cflags: cread cs8 -parenb -parodd hupcl -clocal -cstopb -crtscts -dsrflow
	-dtrflow -mdmbuf
cchars: discard = ^O; dsusp = ^Y; eof = ^D; eol = <undef>;
	eol2 = <undef>; erase = ^?; intr = ^C; kill = ^U; lnext = ^V;
	min = 1; quit = ^\; reprint = ^R; start = ^Q; status = ^T;
	stop = ^S; susp = ^Z; time = 0; werase = ^W;
```

`diff -u <before> <after>` produced no output, exit 0. Actual summary:

```text
DIFF_EXIT=0
```

## Codex sandbox run: PTY resize, then quit — FAIL; stopped

Same real-home invocation and initial geometry. Expect waited for the report and 5500 ms,
then ran `/bin/stty -f $spawn_out(slave,name) rows 32 columns 120 2>@1`, waited 500 ms,
sent `q`, and waited for `TUI_EXIT`. Output from the final run:

```text
RESIZE=40x160->32x120
```

An intermediate `stty -a` showed `32 rows; 120 columns;` and raw flags
(`-icanon -isig -iexten -echo`), so the resize reached the PTY. The capture included a draw
on row 32. The subsequent Expect result was:

```text
FAIL quit timeout
```

The final run used a 15 s Expect timeout and also ran with the command runner's own PTY
enabled; an earlier run without that outer PTY also timed out. The no-resize control above
then passed after a real report. This isolates an observed failure after resizing; it does
not establish its root cause by itself; Claude's host investigation below identifies it.

No `TUI_EXIT` or after snapshot was produced in resized runs. Actual file check:

```text
RESIZED_AFTER=missing (TUI did not return)
```

An after/before diff for this failed sandbox resize run cannot be claimed. The passing host
run below restores the original geometry before comparing terminal flags. No code fix was
attempted in this step.

## Codex sandbox run: open action and checks deferred at the time

`multi-link sessions --since all --json` was captured privately and reduced to state counts:

```text
sessions: 367
states: {'busy': 4, 'idle': 21, 'unknown': 342}
finished Claude with transcript: 0
```

No Resume candidate was available with the denied process probe. A fresh dry-run on a busy
Claude session returned `app_link`; it was not executed. Stub and missing-program PTY checks,
Ctrl-C and the record-on-exit check were deferred at the resize failure in this run. Their host
results follow. No diagnostic labels were observed in the retained real-home captures; a
nonzero-diagnostic display was not verified in these PTYs.

## Claude, host: environment and real-home refresh timing — pass

Claude ran these checks outside the Codex sandbox on 2026-10-30, using the same release binary
built by Codex, with no intervening code changes; Claude Code version 2.1.314.
`/bin/ps -p $$ -o lstart=` succeeded. Both runs used real `~` and real process probes.

`cargo run --release -q -p multi-link-core --example refresh_timing -- 3` → exit 0.
Run A actual output:

```text
pass 1: 3770 ms, 25 sessions, 4 repos, 441 files re-parsed, 516819435 bytes re-parsed
pass 2: 34 ms, 25 sessions, 4 repos, 0 files re-parsed, 0 bytes re-parsed
pass 3: 37 ms, 25 sessions, 4 repos, 0 files re-parsed, 0 bytes re-parsed
```

Run B actual output (the k ≥ 1 result):

```text
pass 1: 3950 ms, 27 sessions, 5 repos, 443 files re-parsed, 518399847 bytes re-parsed
pass 2: 31 ms, 27 sessions, 5 repos, 1 files re-parsed, 183074 bytes re-parsed
pass 3: 28 ms, 27 sessions, 5 repos, 1 files re-parsed, 185194 bytes re-parsed
```

Run B passes 2 and 3 each re-parsed a changed file and meet the 2000 ms target.

## Claude, host: resize root cause and draining resize run — pass with limitation

Codex's `check.exp` reproduced on the host: `FAIL quit timeout`. During the hang, `sample`
showed the TUI alive in `crossterm::event::poll` → mio `kevent`, still redrawing, not deadlocked.
Claude's variants (drain means Expect keeps reading output before the resize):

```text
wrap=0 query=0 drain=1 OK
wrap=1 query=0 drain=1 OK
wrap=0 query=1 drain=1 OK
wrap=0 query=0 drain=0 HUNG
wrap=1 query=1 drain=0 HUNG
```

A stalled reader + resize, then draining for 3 s before `q`, passed twice. Stalled reader +
resize + `q`, then draining for 4 s and sending a second `q`, passed (first key delayed, second
quits). A Python raw-mode control blocked on a 200 kB write with the reader stalled still read
`GOT=b'q'`: the kernel kept the key.

Claude traced the cause to crossterm-0.29.0 `src/event/source/unix/mio.rs`, `try_read`.
When one kevent wake reports both `SIGNAL_TOKEN` (SIGWINCH) and `TTY_TOKEN`, the signal arm
returns Resize immediately and drops the TTY readiness. Mio's kqueue registration is
edge-triggered, so the pending key is not read until another byte arrives. In the failing
check the reader stalled for 5.5 s, blocking the TUI in a write while both events arrived.
This is a crossterm 0.29 dependency issue, not a multi-link bug: one keypress can be delayed
until the next key while the TUI keeps running. An upstream fix or workaround is a backlog
item; no fix was made here.

With the reader draining (`host.exp` mode resize), Claude resized 40x160 → 32x120, waited 3 s,
restored 40x160, waited 3 s and sent `q`. Actual output:

```text
RESIZE=40x160->32x120 / RESIZE=32x120->40x160 / TUI_EXIT=0 / PTY_WAIT=0 / STTY_DIFF=0
```

The `stty -a` before/after diff exited 0; original geometry and terminal flags were restored.

## Claude, host: Ctrl-C on real home — pass

`host.exp` mode ctrlc, real `~`, actual output:

```text
TUI_EXIT=0 / PTY_WAIT=0 / STTY_DIFF=0
```

## Claude, host: stub and missing program — pass with synthetic Resume candidate

Real `~` had **0 finished sessions**. `sessions --since all --json` returned 367 sessions:
unknown 343, idle 22, waiting 1, stale 1. All 24 live records were desktop-app sessions, so no
real-home session offered Resume.

To exercise child launching, Claude used a **synthetic record in a scratch HOME**, with real
process probing and no `--root`: `HOME=<scratch>/fakehome` held a copy of sandbox repo-a's
transcript `80f7add3` and synthetic `sessions/99999.json`. Record fields came from a real
record, with non-running pid 99999, entrypoint cli and cwd `~/multi-link-sandbox/repo-a`.
Nothing under the real `~/.claude` was changed for these child checks.

`HOME=<scratch>/fakehome multi-link open --dry-run 80f7add3` → exit 0, scrubbed output:

```json
{"action":"resume","program":"claude","args":["--resume","80f7add3…"],"cwd":"~/multi-link-sandbox/repo-a"}
```

The TUI showed `claude 80f7add3… finished` (id scrubbed to eight characters).
With `PATH=<scratch>/stub:/usr/bin:/bin`, the stub `claude` printed
`STUB_CLAUDE_RAN args=$#` and exited 0. After `o`, actual output:

```text
CHILD_OUTPUT=STUB_CLAUDE_RAN args=2 / FOOTER=returned to dashboard
TUI_EXIT=0 / PTY_WAIT=0 / STTY_DIFF=0
```

Keys `t`, Esc and then `q` confirmed continued TUI operation. With
`PATH=<scratch>/empty:/usr/bin:/bin` and no `claude`, after `o` the captured footer was:

```text
cannot run "claude": No such file or directory (os error 2)
```

Keys `t`, Esc and `q` again worked; actual output:

```text
TUI_EXIT=0 / PTY_WAIT=0 / STTY_DIFF=0
```

Footer persistence across refreshes was not separately asserted from the PTY; Step 3.7 App
tests cover it. Nonzero diagnostics after key help were not separately verified in these PTYs.

## Claude, host: sandbox record on exit — confirmed limitation

Gotcha: the first attempts were invalid. Launching `claude` from a Claude Code session inherited
`CLAUDECODE` / `CLAUDE_CODE_*` environment variables and wrote neither a record nor its own
transcript. Claude reran with a clean environment: `env -i HOME USER LOGNAME TERM LANG PATH`.

In a 40x120 Expect PTY, cwd `~/multi-link-sandbox/repo-a`, interactive `claude` received one
prompt, then `/exit` → exit status 0. While alive, `~/.claude/sessions/<pid>.json` existed with
entrypoint cli, kind interactive, status idle, that cwd and session prefix `9c0a9a1d`.
Its transcript also existed. Two seconds after exit the record was **GONE**; the transcript
remained.

`multi-link open --dry-run 9c0a9a1d` → exit 3, scrubbed output:

```json
{"action":"unavailable","reason":"not_known_finished","command":"claude --resume 9c0a9a1d…","cwd":"~/multi-link-sandbox/repo-a"}
```

`sessions --json` returned state `unknown`, record `null`. A terminal Claude session that exits
cleanly deletes its record → `unknown` → `o` is Unavailable with the resume command text.
Resume requires a record that outlives its process (for example after a crash/kill), along with
a transcript and positive process-exit evidence. This is a Release 1 limitation.

## Codex sandbox run: whole-phase automated verification

`cargo build --release -q -p multi-link` → no output, exit 0.
`cargo test --workspace` → exit 0; actual result lines:

```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.03s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.65s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The CLI's overview answer-key comparison ran in that command:

```text
test overview_json_matches_answer_key_and_text_has_one_line_per_repo ... ok
```

`cargo clippy --workspace -- -D warnings` → exit 0, actual final output:

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.31s
```

Whole-phase Verify passes with the documented crossterm resize/key and clean-exit record
limitations. Host Run B confirms warm timing with k ≥ 1; draining resize, Ctrl-C, stub and
missing-program checks return to a restored terminal. Child launching used a synthetic
record in a scratch HOME; PTY footer persistence and nonzero diagnostics were not separately
asserted. Claude reviews.
