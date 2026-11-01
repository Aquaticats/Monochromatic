# Translation repair history, first handover document: round two accumulation: the detached launch, band ordering, the coverage bar and slice-rate projection, 2026-07-27

The detached launch that survived where harness launches were killed,
the largest entry that needed one more run,
band ordering computed once per run,
the coverage bar reached and the round two sheet drawn,
and the correction that one slice rate projects a schedule only when many slices remain.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Round two accumulation: the detached launch, band ordering, the coverage bar and slice-rate projection

- DETACHED LAUNCH CONFIRMED AS THE FIX for the run kills (2026-07-27).
  Run 017,
  launched with `setsid nohup mise run ... > log 2>&1 < /dev/null & disown`,
  ran
  4.79 hours and exited NORMALLY on its own soft budget
  (`DONE processed=4 of pending=71; artifacts=25/92 elapsed=17247028ms`).
  The two
  runs before it,
  launched as harness background tasks,
  were killed by signal at
  2h38m and about 2 minutes with the signature
  `sh exited with non-zero status: no exit status` and no OOM evidence.
  A run
  that survives 4.79 hours in its own session,
  after two died in the harness
  session at unrelated ages,
  is what the process-group explanation predicts.
  Keep launching this way;
  do NOT chain a launch with a commit in one call,
  which is what made an earlier kill look like a failed commit task.
- NIGHT81473140 (12301 B,
  LARGE,
  the biggest entry attempted so far) hit the
  90 min per-entry hard cap in run 017 with 26 slices cached,
  so it did not
  settle.
  Recoverable exactly as Dethelly and Futajuhuacha were:
  resume-first
  ordering picks it up next run and slice progress is monotone.
  It needs ONE further run,
  not several,
  and that is measured rather than
  inferred from its size.
  Run 017 logged the entry as `10 chunk pairs, 41
  slices`,
  and it finished 26 of those 41 from a COLD start inside the 90 min
  cap,
  so 3.46 min/slice.
  The 15 remaining come to about 52 minutes,
  well inside
  one cap.
  General lesson:
  the pipeline prints its slice count when an entry
  starts,
  so "how many more runs does this need" is a log lookup and a division,
  never a guess from byte size.
  An earlier draft of this note guessed from the
  12 KB figure and got it wrong.
- BAND ORDERING IS COMPUTED ONCE PER RUN,
  so a long run can overshoot a band.
  `rankWithinBands` and `resumableIds` are both evaluated at run START and the
  `pending` array is sorted once;
  nothing re-ranks as entries settle DURING the
  run.
  Observed in run 018,
  which started at 8 small / 9 medium / 8 large and
  settled five entries:
  by the time it picked MocaKawai (medium),
  the live
  counts were 10 / 10 / 9 and a large entry should have led,
  but the frozen
  ordering still had medium ahead.
  Result was 10 / 11 / 9 instead of 10 / 10
  / 10.
  Same cause,
  second symptom:
  an entry that a run leaves partly cached does NOT
  resume later in that SAME run,
  because `resumableIds` predates its cache
  directory.
  Susiethegamer aborted at the hard cap with 12 of 17 slices,
  and run
  018 went on to other entries rather than returning to it.
  NOT WORTH FIXING for this milestone,
  and the reason is not laziness:
  the
  ordering self-corrects on the next launch,
  both symptoms cost at most one
  entry of drift,
  and re-ranking mid-loop would make the processing order depend
  on completion times,
  which is a new nondeterminism in the thing that decides
  what gets measured.
  Prefer the stale-but-deterministic order.
  Revisit only if
  runs get long enough that intra-run drift exceeds one entry per band.
- COVERAGE BAR REACHED 2026-07-27:
  31 settled entries,
  10 small / 11 medium /
  10 large,
  against the stratified ~10/10/10 target.
  Accumulation was then
  STOPPED DELIBERATELY (run 019 killed mid-entry) so the pool is fixed and the
  sheet's provenance is unambiguous rather than drifting under the draw.
  Accepted-issue pool at the draw:
  2257 over 31 entries.
  Per band:
  small 238
  (10.5 percent),
  medium 745 (33.0),
  large 1274 (56.4).
  ROUND-TWO SHEET DRAWN to
  `node_modules/.monochromatic/translation-repair-runs/grading-sheet-milestone-three-precision-round-two.md`,
  50 items,
  17 small / 17 medium / 16 large,
  seed
  `milestone-three-precision-round-two`,
  corpus pin unchanged.
  Validated before handing over:
  the sheet spreads across 30 of the 31 settled
  entries,
  so no single entry dominates it,
  and its 50 (entry,
  claim) pairs have
  ZERO overlap with round one's graded 50.
  That check matters because scoring a
  round on items already used to calibrate it would read better than the
  pipeline is;
  the seed rule alone was not treated as proof.
- SLICE-RATE PROJECTION CORRECTED.
  The rule recorded earlier ("a log lookup and
  a division") is right about WHERE the numbers come from and wrong about how
  far one rate projects.
  Susiethegamer had 5 of 17 slices left at a cold-portion
  rate of 7.5 min/slice,
  so 37 minutes was projected;
  it took 80.9.
  The resume's
  five slice cycles ran 7.2,
  8.7,
  about 28,
  12.9,
  and 14.6 minutes,
  so per-slice
  cost varies roughly fourfold WITHIN one entry.
  NIGHT81473140's projection
  landed (52 projected,
  50.6 actual) only because 15 remaining slices averaged
  that variance out.
  So:
  project from remaining slices only when MANY remain,
  and treat a
  small-remainder estimate as an order of magnitude,
  never a schedule.
  Same
  small-sample lesson as the timing cohorts and the panel-coverage sub-rates,
  arriving for the third time in one session through a different door.
  One unexplained observation left in place rather than explained away:
  between
  the panel stage at 16:12:01Z and the next critic stage at 16:40:20Z there is a
  28 minute gap with no editor or checker line logged,
  though the panel had
  issued 11 issues.
  Cause not established;
  do not assume it is provider latency.
