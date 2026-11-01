# Translation repair history, first handover document: how round two's precision may be read: the retracted cohort split, an ungraded drop and band balance, 2026-07-27

The promise to split precision by timing cohort,
retracted for lack of grading effort,
the drop in accepted issues recorded as no evidence of precision,
and the finding that the headline precision averages per-band precisions,
with both readings to be reported from the same grades.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## How round two's precision may be read: the retracted cohort split, an ungraded drop and band balance

- COHORT-SPLIT PROMISE RETRACTED (2026-07-27,
  `run-config.ts`).
  The stamp's
  docblock claimed precision could be split by call-timing cohort at analysis
  time,
  making the mixed pool "a number rather than an unknown".
  It cannot.
  At
  the coverage bar the pool is about 30 entries split near evenly,
  so 50 graded
  items give roughly 25 per cohort and a standard error near 8 points;
  the
  binding constraint is human grading effort,
  not compute,
  so the several
  hundred per cohort that would resolve a meaningful difference is unavailable.
  Report the mixed pool QUALITATIVELY alongside the panel-coverage sub-rates.
  Note this weakens the stated basis of the user's twice-made "keep the settled
  entries" decision,
  so it is surfaced rather than quietly adjusted;
  it does not
  change what to do,
  since discarding the compute was rejected both times.
- UNGRADED OBSERVATION,
  first quantitative sign fixes A-F changed behavior at
  all:
  accepted issues per entry fell from 99 (round one,
  2871 over 29 entries)
  to 75 (round two,
  1731 over 23),
  a 24 percent drop,
  in the direction the fixes
  intended.
  NOT EVIDENCE OF PRECISION and must not be recorded as such:
  nothing
  here is graded,
  fewer accepted issues is equally consistent with the fixes
  suppressing true positives,
  and the two pools cover different entries.
  It
  earns a mention only because it is measurable now and the graded answer is not.
- HEADLINE PRECISION IS BAND-BALANCED,
  NOT POOL-WEIGHTED (found 2026-07-27,
  before the round-two draw).
  `drawStratifiedSample` splits the 50 slots about
  evenly across bands (round one drew 17 small / 17 medium / 16 large),
  but the
  bands do NOT hold even shares of the accepted-issue pool.
  Measured at 25
  settled entries:
  small 175 accepted (9.2 percent),
  medium 667 (34.9),
  large
  1070 (56.0).
  So a small-band issue is roughly 3.6 times likelier to be sampled
  than its share of the population,
  and the raw sample proportion estimates the
  AVERAGE OF PER-BAND PRECISIONS rather than the precision of the accepted-issue
  population.
  Size of the discrepancy,
  using round one's per-band precisions (small 0.60,
  medium 0.57,
  large 0.73) against round two's pool shares:
  band-balanced 0.635,
  pool-weighted 0.665.
  Round one failed at 0.56 to 0.68 under every reading,
  so
  the distinction could not change that verdict.
  Against a 0.9 bar it can.
  RESOLUTION,
  adopted because it costs nothing and sacrifices nothing:
  report
  BOTH numbers from the SAME 50 grades.
  Band-balanced stays the headline,
  since
  that is what round one reported and comparability with the baseline is the
  whole point of round two.
  Pool-weighted is reported beside it,
  because "accepted-issue
  precision" read plainly means "of the issues the pipeline accepts,
  how many
  are real",
  which is the pool-weighted quantity.
  No extra grading is needed:
  per-band counts plus pool shares give both.
  The only genuine decision is which number faces the bar IF they straddle 0.9,
  and that is the user's call.
  Do not pre-empt it;
  if both clear or both fail,
  it never needs asking.
  Related caution:
  round one's verdict called precision "roughly flat across
  bands" on 0.60 / 0.57 / 0.73.
  At 15 graded items per band the standard error
  is about 0.12,
  so those are not distinguishable,
  but "flat" overstates what
  n=15 per band can show.
  The band comparison is underpowered in the same way
  the timing-cohort split is;
  say "not distinguishable",
  never "flat".
