# Translation repair history 2026-08-05 to 2026-08-07: the recall benchmark and its first run

Recall made measurable by planting known omissions,
and what recall run 001 found.

Part of [Translation repair history: 2026-08-05 to 2026-08-07](translation-repair-history-2026-08-05-to-07.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Recall becomes measurable

- RECALL IS FINALLY MEASURABLE:
  `repair-benchmark.ts` had no entrypoint,
  so
  recall was unmeasurable in practice however complete the library was.
  `corpus-run/recall-benchmark.ts` plus a mise task plants known omissions into
  clean translations and grades restoration against its own deletions,
  giving a
  denominator of defects that certainly exist rather than defects the pipeline
  chose to report.
  Nine entries,
  three per band,
  27 seeds,
  verified at zero
  quota through `--plan`.

## Recall run 001

- RECALL RUN 001 LANDED 2026-08-06,
  and its DETECTION number was invalid until
  `6bb299773`.
  Raw scorecard:
  `dispatched=7 coverage=0.778 planted=21 detected=8 detectionRate=0.381`
  `judged=21 restored=19 partial=1 strict=0.905 lenient=0.952`.
  The two halves contradict each other,
  which is what exposed the bug:
  the
  pipeline only edits inside envelopes cut from accepted issues,
  so restoring 19
  seeds requires accepted issues at their regions,
  while detection claimed 8.
  CAUSE:
  `gradeSeedDetection` indexed `alignment.pairs` with
  `record.chunkIndex`,
  which is a global SLICE index from `subdivideChunkPair`,
  not a pair index.
  Past the pair count it read nothing and called every issue
  there absent;
  within it,
  it added a pair start offset to a slice-local span
  offset.
  Detection collapsed toward counting only seeds landing in the first
  slice of a pair,
  the one case where pair and slice share a start offset.
  Every dispatched entry subdivided (1 pair to 12 slices,
  6 to 12,
  1 to 7,
  2 to
  7,
  1 to 4,
  2 to 3),
  so only a 1-pair-1-slice entry was unaffected.
  WHAT SURVIVES:
  `strict=0.905` and `lenient=0.952` over 21 judged seeds are
  SOUND.
  The restoration judge compares the needle's meaning against the
  repaired text with the Chinese as anchor and never touches that mapping.
  This is also the POLICY-FREE recall baseline,
  since the run started 23 minutes
  before the house policy landed.
  WHAT DOES NOT:
  `detected=8` and `detectionRate=0.381` mean nothing.
  Do not
  quote them.
  Detection has to be re-measured on a fresh run.
  MILESTONE TWO IS UNAFFECTED,
  now CHECKED rather than assumed.
  `slice-pair.ts`
  and its wiring into the driver first appear in `88eb42add`,
  AUTHORED
  2026-07-23,
  while milestone two was declared 2026-07-18.
  Those runs had no
  subdivision,
  so pairs and slices were the same list and the mismatch could not
  bite.
  The 166/174 detection figure stands.
  Run 001 is the ONLY affected run:
  it is the only artifact anywhere under the
  runs dir carrying `seedDetection`,
  and every post-slicing pass log
  (`pass4`,
  `pass5`,
  `pass6`) records precision passes that emit no detection
  figure at all.
  DATE TRAP,
  worth remembering:
  the rebase rewrote every committer date on this
  branch to 2026-08-05T23:4x.
  Reading `%cI` says slicing landed AFTER the recall
  run started,
  which is false and would have inverted this conclusion.
  Use `%aI`
  for chronology on this branch,
  or better,
  test the tree directly.
  RUN 001 IS POLICY-FREE,
  verified by tree content rather than by timestamps:
  at the run's own recorded tip `2cf7fd453`,
  `house-policy.ts` is ABSENT and
  `critic-prompt.ts` carries zero references to `HOUSE_POLICY_BLOCK`.
  `candidate-select.ts` and `editor-ensemble.ts` are absent too,
  so it is a
  pre-ensemble baseline as well.
  `slice-pair.ts` IS present,
  which is why the
  detection mismatch applied.
  THREE INDEPENDENT SIGNALS said detection was wrong before the code was read:
  the pipeline only edits inside accepted-issue envelopes,
  so 19 restorations
  need accepted issues;
  `statusCounts` records `repaired: 7`,
  every dispatched
  entry shipping a repair;
  and the retired lexical grader put restoration at
  15/21,
  also far above 8/21.
  Two of nine entries were skipped by the 4h dispatch budget,
  giving coverage
  0.778;
  that is the coverage-per-run effect task 50 is about.
