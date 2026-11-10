# Phase 12 check: settle ambiguous links with evidence

Step 12.2, 2026-11-10. Phase 12 adds a nearest-run rule for launch hand-offs (K0–K4); see
[design](design.md#phase-12-decisions-settle-ambiguous-links-with-evidence-step-120) and
[output schema](output-schema.md) (Heuristic → Phase 12). Nothing real is committed (W10): the exports, ids and paths
stayed in scratch storage; this page gives counts and gaps only.

## Before and after on real data

`multi-link links --since all` on the owner's machine: before = the release build at 04b4233 (export taken while
planning), after = the release build at 437adf4. Compared by hand-off on the 233 invocation ids in the export (234
links; one id appears twice and was counted once); 3 hand-offs are newer than the export and not in the table.

| Before | After | Count |
|---|---|---|
| exact (redirect_file 21, resume_arg 16, stdout 14, thread_ready 2) | unchanged | 53 |
| heuristic, `resume_turn` | unchanged | 6 |
| heuristic | unchanged | 117 |
| heuristic | unlinked | 1 |
| unlinked | unchanged | 47 |
| ambiguous | heuristic, `nearest_run` | 8 |
| ambiguous | ambiguous | 1 |

This is PLAN.md's K4 table exactly. All links now: 236 (132 heuristic, 55 exact, 48 unlinked, 1 ambiguous).

**The 9 that were ambiguous**, by gaps only (run start − hand-off). Each had a nearest run 0.49–1.76 s after it.
- 7 settled by K1: every other run in their window is another hand-off's own choice at a smaller gap.
- 1 settled by K2: the other runs started 23.8 s or more after it.
- 1 stays ambiguous: runs at 0.91 s and 7.88 s; the 7.88 s run is nobody's choice, and 7.88 s is under K2's 10 s.

**The heuristic that became unlinked.** Its only run started 9.4 s after it and 2.2 s after a later hand-off in the
same checkout. That run is the later hand-off's own choice (its only candidate), so it leaves this one (K1). Before
Phase 12 both hand-offs claimed the same run.

## Masked check with the built resolver (the stop gate)

For each exact launch link (child id observed, not `resume`), a scratch harness copied that checkout's real Claude
and Codex transcripts into a temporary root, replaced the child id in the parent transcript, and ran the release
`multi-link links` on it. 30 links (more than at planning, as new hand-offs were recorded):

| Outcome | Count |
|---|---|
| right: the true child, `heuristic` (one candidate) | 23 |
| right: the true child, `heuristic` / `nearest_run` | 2 |
| wrong child | **0** |
| unlinked: the child started before the hand-off (outside the window) | 1 |
| unlinked: another exact launch observed the same child (a plugin call and its inner `codex exec`), so it stays claimed | 2 |
| still exact through other evidence (not masked by the harness) | 2 |

No wrong answer, so the phase was not stopped. The sample is small and few cases have rivals near K2's margins, so
the 5 s / 10 s values stay provisional (PLAN.md K2).

## Fixtures

The committed fixture `links-amb-hidden.json` (the sandbox pair with their ids hidden) now settles amb-1 by K1 to
d501, which is its true child in the unmasked fixture. Synthetic cases a–k in `fixtures/sessions-phase12/` cover K1,
K2 boundaries, ties, exact and resume neighbours and `cd`; keys in `fixtures/expected/phase12/`.

## Differences from the plan

- K1 was tightened while writing the keys: a run leaves a hand-off only when the other hand-off would itself choose
  it, not merely when it is that hand-off's nearest run (PLAN.md Review log, Step 12.0). Real counts are the same.
- The graph's hand-off box is 19 cells wide, so it shows `matched by time · …`; the full `matched by time · nearest
  run` is on the selected part's origin line and the narrow list.
- Found, not changed: `codex exec resume -c k=v <id>` (options before the id) is read as a plain `exec`; backlog.
