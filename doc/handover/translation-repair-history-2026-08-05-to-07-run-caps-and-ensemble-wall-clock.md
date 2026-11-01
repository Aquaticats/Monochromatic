# Translation repair history 2026-08-05 to 2026-08-07: run caps and the ensemble's wall clock

Run caps raised on measurement,
and the ensemble's wall clock still unmeasured.

Part of [Translation repair history: 2026-08-05 to 2026-08-07](translation-repair-history-2026-08-05-to-07.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Run caps raised on measurement, and the ensemble's unmeasured wall clock

- RUN CAPS RAISED ON MEASUREMENT (task 50 COMPLETE,
  commit `96e7c5ec4`).
  Recall run 001's seven entries were timed end to end from its own log:
  per-slice 3.25 min best,
  5.56 median,
  8.56 worst;
  longest entry 74.7 min for
  12 slices;
  252 min total for seven entries.
  That CONFIRMS the ~5.5 min/slice figure `corpus-pass.ts` already claimed,
  and
  shows `HARD_CAP_MINUTES = 90` was ALREADY marginal before this branch:
  at the
  worst observed rate a 12-slice entry needs 103 min and would have been cut.
  `HARD_CAP_MINUTES` 90 -> 180,
  `SOFT_BUDGET_MINUTES` 240 -> 720,
  recall `BUDGET_HOURS` 4 -> 12.
  The measured rate is PRE-ENSEMBLE:
  it predates per-envelope ballots,
  the chunk
  round,
  and the naturalness lane,
  all of which only add.
  180 is therefore a
  bound against runaway,
  not a tuned value;
  it clears 21 slices at the worst
  observed rate and 32 at the median.
  Re-derive once a post-ensemble pass has
  enough slices to project from,
  and remember the Susiethegamer lesson:
  per-slice
  cost varies about 4x WITHIN one entry,
  so do not project from a handful.
- THE ENSEMBLE'S WALL-CLOCK IS UNMEASURED,
  and this is NOT a cost question.
  An earlier version of this note called it cost and treated it as a gate on
  round three.
  Both were wrong,
  and the user corrected the first directly ("I
  don't think the cost matters").
  The plan is flat rate and quota regenerates
  faster than runs spend,
  which is the user's own directive and the reason
  stages retry lost voices freely,
  so tokens are free.
  What is actually at stake is COVERAGE PER RUN.
  `HARD_CAP_MINUTES = 90` in
  `corpus-pass.ts` aborts one entry's exchanges,
  and its own comment records the
  measured ~5.5 min/slice rate that makes 90 minutes clear about 16 slices.
  Per-envelope ballots run sequentially,
  one round per envelope with more than
  one distinct proposal,
  each now carrying source plus envelope base plus 800
  characters of context,
  so per-slice time rises and the slices an entry can
  finish falls.
  Slice-level resumability means a capped entry resumes next run,
  so the harm is entries covered per run rather than lost work.
  That makes this a CONSTANT TO SET,
  not a gate.
  Read the per-slice rate off the
  first slices of the round-three pass and raise `HARD_CAP_MINUTES` and
  `SOFT_BUDGET_MINUTES` to fit,
  rather than holding the pass for a separate
  measurement run (task 50).
  Do NOT project from a handful of slices:
  the Susiethegamer projection missed
  by 2x (projected 37 min,
  actual 80.9) because per-slice cost varies about 4x
  WITHIN one entry.
  The `editor-candidates`,
  `editor-envelope-select`,
  and `editor-chunk-select`
  findings are the instrument for how often judging actually fires.
