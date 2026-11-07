# Phase 7 real check — 2026-11-02

Step 7.4 checks the four Phase 7 polish rules (P1–P4, `PLAN.md` → Phase 7, `docs/design.md` → "Phase 7
decisions") on real data and asks the owner to confirm the screen. The real renders show prompts, so they
stayed in scratch storage and are described here in words and counts only (W10). The owner confirmed the
need is met (see [Owner check](#owner-check-stop-and-ask--met)).

**Most important findings on real data (2026-11-02 21:19 PST):**
1. Claude sessions with no prompt: 43 of 181 before Phase 7, 11 of 185 now. Sessions started by another
   session now show their first real line instead of `claude · no prompt`.
2. With `--since 16h` both window phrases on the real home screen (header and card count line) say
   `last 16h`; at the default they still say `last 2h`.
3. The 16 h repo view of multi-link has three idle folds, and all three show their length next to the
   `HH:MM now` end label.

## Real prompt count (P1)

Command (counts only, no prompt text is printed):

```sh
target/release/multi-link sessions --since all --json | jq -c \
  '[.sessions[]|select(.harness=="claude")] | [length, (map(select(.asked==null))|length)]'
```

| When | Commit | Result `[Claude sessions, asked null]` |
| --- | --- | --- |
| Phase baseline, 2026-11-02 | 26af6f2 | `[181,43]` |
| Step 7.1, before the rebuild (old rule) | 7.1 working tree | `[182,44]` |
| Step 7.1, after the rebuild (P1) | 7.1 working tree | `[182,11]` |
| Step 7.4, 2026-11-03T05:18:57Z | f6d769e | `[185,11]` |

The first number grows because live transcripts grow: 7.1 saw one new session (and with it one more
old-rule null), and 7.4 sees three more Claude sessions than 7.1 did. The five newest Claude sessions
(started 04:46–05:18 UTC on 2026-11-03) all have a prompt, so the null count stays 11. 7.4 did not
re-classify those 11; the count equals the plan's, whose 11 were other wrappers (`local-command-caveat`,
`bash-input`, `scheduled-task`), a reminder with no text after it, or no eligible record (the non-goal).

Codex is not changed by P1. For the README's limitation line, the same run counted Codex sessions as
`[total, asked null, non-imported with asked null]` = `[292,154,3]`: the nulls are almost all Codex
Desktop imports (hidden on screen); 3 older non-imported runs still read `codex · no prompt`.

## How the renders were made

Same method as [phase6-check.md](phase6-check.md#how-the-images-were-made): `overview --json` and
`links --json` at a fixed `--now 2026-11-03T05:19:00Z`, then `crates/tui/examples/home_html.rs` at
120×40 with `--offset local` (UTC−8) and the matching `--since`, and headless Chrome at 2× for the
screenshots. Four renders: home at `--since 2h` and `--since 16h`, and the multi-link repo view at both
windows (`--repo` with the repo's full path). The 16 h repo view has folds, so the fixture was not needed.
All files stay in scratch storage and were shown to the owner from there.

## What each rule changed on screen

**P1 — Claude prompt after a system reminder.** On the real home card the working Claude session (this
step's own session, started by the coordinator session) is labelled with the first words of its brief in
quotes, where Phase 6 showed `claude · no prompt`. Across all four renders `no prompt` appears 0 times. In
the repo view, Claude lanes with no PLAN.md step show a quoted prompt too; lanes for a step still show
the step (W3 order, unchanged).

**P2 — Home names its window.** At `--since 16h` the header reads `All repos · last 16h`, the card's grey
line reads `20 other sessions, 16 active in the last 16h`, and no `last 2h` is left on the screen. The third phrase (`active about N min in the last …`, on an idle
repo with hidden activity) did not appear on the real screen; the 7.2 test covers it. At the
default 2 h window the same places read `last 2h` (17 other sessions, 10 active), as before. In Phase 6
the home screen said `last 2h` whatever `--since` was.

**P3 — Times in the last 24 h keep the clock.** Not visible on today's real screens: at 21:19 PST every
time a 2 h or 16 h window shows falls on the same local date (2026-11-02), so both the old and the new rule
give `HH:MM` (for example `last hand-off 21:14`, `handed work to codex at 16:02`, `started 21:18`). The
Phase 6 case (`last active Nov 1` for 23:53 the night before, seen at 14:07 the next day) would now read
`last active 23:53`; that is covered by the Step 7.2 tests (23 h 59 min before now on the previous local
date → `HH:MM`, exactly 24 h → `Mon D`, in UTC, UTC−8 and UTC+5:30), not by a real render.

**P4 — Axis labels never hide each other.** In the 16 h repo view the axis row reads, left to right: a
`1h05` fold, ticks `16:00` and `16:30`, two folds side by side labelled `54m` and `2h56`, and the end label
`21:19 now`. Every fold shows its length, the end label keeps its full `HH:MM now` form because it fits,
and no tick repeats the `21:19` minute. In Phase 6 the 16 h view's overnight fold next to `14:07 now` lost
its length, and in the 2 h view a `14:07` tick printed next to `14:07 now`. The 2 h view has no folds; its axis is three ticks
(`20:50`, `21:00`, `21:10`) and `21:19 now`, with no repeated minute.

**Seen, not changed:** the two adjacent folds read `54m 2h56`; with one blank cell between labels, which
length belongs to which band is clear only from the `≈` bands under them. Left as is (P4 says nothing
about adjacent folds; an owner call if it matters).

## Whole-phase Verify (Claude, 2026-11-02)

```text
cargo test --workspace                                → exit 0, 36 `test result: ok`, 0 failed
cargo clippy --workspace --all-targets -- -D warnings → exit 0
git diff --exit-code 26af6f2 -- fixtures/expected/tui/{home,repo}-phase6{,.styles}.txt → exit 0
real count (jq command above)                         → [185,11]  (baseline [181,43])
```

## Owner check (stop-and-ask) — met

Need: "the screens the owner approved in Phase 6 say true, readable things on real data." To check:

```sh
cargo build --release -p multi-link
target/release/multi-link tui
target/release/multi-link tui --since 16h
```

Owner's answers:
1. 2026-11-03, while checking: "make it so that the quote for the session is the most recent message not the
   first message of that session". Step 7.5 added `last_asked` (P5) and labels now quote it. Real counts at
   9d58365: Claude 186 sessions, 89 with a latest prompt different from the first; Codex 295, 6 different.
2. 2026-11-06, after seeing real renders of home at 2h and 16h and the multi-link repo view at 4d (shown from
   scratch): "Yes, close Phase 7". The need is met; Phase 7 is closed.
