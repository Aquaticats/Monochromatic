# Translation repair history, first handover document: pass 7 run records, throughput and the first round two signals, 2026-07-26 to 27

The detached launch that survived the harness kills and the run records of the second accumulation round,
newest first,
except runs 013 and 014,
which sit with the deadline measurements.
It also holds the soft budget that limited throughput,
the Dethelly entry that needed several runs,
the band ordering fix,
and the first comparisons of accepted counts between the two rounds.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Corpus runs launched detached from the harness

LAUNCH CORPUS RUNS DETACHED,
  NOT AS HARNESS BACKGROUND TASKS (2026-07-27).
Two consecutive runs died by signal with the same mise signature,
  `sh exited
with non-zero status: no exit status`:
  run 015 at about 2 h 38 min and run 016
after roughly four seconds of work.
  Neither was resource exhaustion (63.9 GB
total with 26.9 GB available,
  no OOM kill and no memory-pressure entry in the
journal),
  and both had been launched as harness background tasks,
  so the signal
is reaching the task's process group rather than arising in the run.
REMEDY:
  launch through `setsid nohup ... < /dev/null &` from the worktree,
  which
puts the run in its own session where a process-group signal cannot reach it.
Run 017 launched this way survived.
  The cost is that the harness no longer
reports completion,
  so pair the launch with a monitor over the log;
  check
liveness with `kill -0 <pid>` on the recorded pid rather than by matching the
command name,
  which self-matches (see the trap recorded for run 015).

## Pass 7 run 015 killed externally and the process-check trap

PASS 7 RUN 015 (2026-07-27,
  tip `a03997506`):
  KILLED EXTERNALLY at about 2 h
38 min,
  not finished and not aborted by its own budget.
  Two settled before it
died,
  21/92,
  bands 7 small / 7 medium / 7 large.
  LCG_Akiball repaired (59
issues,
  56 accepted,
  8 findings,
  resuming cached slices),
  CuspariaKLSY (59/57/57,
4 findings).
  MTF_0615 hit the 90 min entry cap and cached its slices.
  A further
entry was mid-flight,
  having just logged `chunk 0: repaired, 24/24`,
  and its
finished slices are cached too,
  so nothing is lost.
CAUSE UNKNOWN AND NOT RESOURCE EXHAUSTION:
  mise reported `sh exited with
non-zero status: no exit status`,
  which is death by signal,
  and the journal
shows no OOM kill or memory error in the window.
  Treat a killed run as ordinary
weather:
  slice caching makes progress monotone,
  so the next run resumes.
PROCESS-CHECK TRAP worth keeping:
  `pgrep --full 'corpus-pass'` reports a live
process even when none exists,
  because the pattern matches the very shell
command running the search.
  Confirm with `pgrep --full --list-full
'node.*corpus-pass\.ts'` or a `ps` listing filtered against grep itself before
concluding a run survived;
  the false positive nearly caused a duplicate launch
to be withheld on the strength of a phantom.

## Pass 7 runs 011 and 010: the soft budget as the bottleneck

PASS 7 RUN 011 (2026-07-26,
  5400002 ms = the full 90 min):
  ZERO settled,
  still
10/92.
  Futajuhuacha (LARGE,
  5448 B,
  3 chunk pairs,
  22 slices) ABORTED at the
hard cap having adjudicated chunks 0 through 10,
  and 11 of its 22 slices are
cached.
  Recoverable exactly as Dethelly was:
  resume-first ordering picks it up
next run and slice progress is monotone.
  Three of those 11 chunks ran
short-handed (chunk 3 critic 6/7,
  chunk 5 critic 5/7,
  chunk 7 panel 5/7) out of
26 logged per-call timeouts,
  the rest of which retried back to 7/7.
THROUGHPUT BOTTLENECK IDENTIFIED:
  `SOFT_BUDGET_MINUTES = 25` in
`corpus-pass.ts`,
  not the 90 min hard cap.
  Because `BANDS` puts large first
within a rank,
  a run starts a large entry,
  that entry alone exceeds 25 minutes,
and the soft-budget check then refuses to start anything else,
  so a run settles
at most one entry.
  Runs 010 and 011 both show it.
  The hard cap costs nothing but
launch round-trips now that slice resumability exists,
  so raising IT is not the
lever;
  its docblock note that large entries "need slice-level resumability,
tracked separately" is STALE,
  that work landed.
  Reaching 10/10/10 needs about 20
more entries,
  which at one entry per launch is roughly 27 launches and 27 to 40
hours of wall clock.
  Raising the soft budget is measurement-neutral (it changes
only when a run stops starting entries,
  never what the pipeline finds) and
chains several entries per launch.
  Apply it between runs,
  never during one.
DO NOT touch `RUN_PER_CALL_TIMEOUT_MS` (240 s):
  it is the one budget that
changes what the pipeline finds,
  and all ten settled round-two entries were
produced under it,
  so moving it would leave a mixed-configuration corpus and the
round-two number would mean nothing.
PASS 7 RUN 010 (2026-07-26,
  tip `d9ecde7fc`,
  1594033 ms ~27 min):
  1 settled,
10/92.
  Everythings99 repaired (MEDIUM,
  21 issues,
  20 accepted,
  20 resolved,
  2
findings).
  Bands 3 small / 4 medium / 3 large.

## Pass 7 runs 009 to 007: Dethelly and the 90 minute cap

PASS 7 RUN 009 (2026-07-26,
  tip `a3cf36dbf`,
  1634756 ms ~27 min):
  TWO settled,
9/92.
  Dethelly repaired at last (LARGE,
  270 issues,
  260 accepted,
  257 resolved,
36 findings) in only 200917 ms ~3.3 min because runs 007 and 008 had already
cached 23 of its 24 slices;
  then Aniloviraw repaired (SMALL,
  27 issues,
  27
accepted,
  27 resolved,
  6 findings).
  Bands 3 small / 3 medium / 3 large,
  dead
even,
  which confirms the `countSettledPerBand` rank-offset fix (`61487a893`)
does what the band-starvation bug needed.
PASS 7 RUN 008 (2026-07-26,
  tip `afe844305`,
  5400006 ms):
  ZERO settled,
  7/92.
Dethelly ABORTED at the hard cap a second time,
  reaching 23 of 24 slices.
  Two
consecutive full-budget aborts on the same entry rule out transient API
throughput as the cause;
  the remaining hypothesis is that this entry's slices
are individually more expensive,
  which run 009's 3.3 min finish for the last
slice is consistent with but does not prove.
PASS 7 RUN 007 (2026-07-26):
  Dethelly ABORTED at the 90 min hard cap
(status=ERROR,
  aborted=true),
  11 of its 24 slices cached.
  Recoverable by
design:
  resume-first ordering picks it up next run and progress is monotonic.
NOT A GENERAL SLOWDOWN,
  measured rather than assumed.
  Across the five entries
settled in BOTH rounds,
  round two is FASTER in total:
  172.9 min -> 151.4 min,
ratio 0.88 (Acheron 0.67,
  AkiraComplex 0.72,
  Chinatsu_Suzuki 0.85,
Considerate_cat 0.88;
  only AmbeR_the_anpa slower at 1.13).
  So the added prompt
policy and identity block did not cost throughput.
DETHELLY IS ENTRY-SPECIFIC AND REPRODUCIBLE.
  Round one:
  24 slices in 66 min
(2.75 min/slice),
  373 accepted,
  settled in ONE run.
  Round two run 007:
  11 of
the SAME 24 slices in 90 min (8.2 min/slice).
  Run 008 resumed and reached 23 of
24,
  so 12 more slices in another 90 min (7.5 min/slice).
  Slice count is
identical across rounds,
  so the aligner did not fragment it.
TRANSIENT API THROUGHPUT IS NOW RULED OUT:
  the rate reproduced across two
independent runs hours apart,
  at 8.2 then 7.5 min/slice against round one's
2.75.
  The remaining hypothesis is that the slices now carry DIFFERENT CONTENT
-- the aligner pairs different blocks,
  so the models face different (and
plausibly more real) work than the misaligned pairings gave them.
  That is
consistent with the whole point of the fix,
  but it is a hypothesis,
  not a
measurement;
  confirming it needs a per-slice timing comparison that nothing
currently records.
OPERATIONAL CONSEQUENCE:
  the 90 min HARD_CAP_MINUTES was calibrated at ~5.5
min/slice for entries up to ~16 slices.
  At 7.5 to 8.2 min/slice a 24-slice
entry needs three runs instead of one.
  Resume makes that correct but slow.
Entries far larger (aiyysk 77 slices,
  hulicaijia 65) were already known to
exceed any single-run ceiling.
  If more large entries start needing three runs,
raising the cap is the cheap lever,
  but do NOT raise it without first checking
that per-slice time is genuinely higher rather than one entry being unusual.
Dethelly was already round one's slowest entry and its heaviest by accepted
count,
  so it is the expected place for a cap to bite first.

## Pass 7 runs 006 and 005

PASS 7 RUN 006 (2026-07-26,
  tip `2441b4150`,
  2709298 ms):
  1 settled,
  7/92
(large 2 / medium 3 / small 2).
  Considerate_cat status=repaired (27 issues,
  25
accepted,
  23 resolved).
PASS 7 RUN 005 (2026-07-26,
  tip `61487a893`,
  1543107 ms):
  TWO settled (small
entries are fast),
  6/92 = large 2 / medium 2 / small 2.
  Acheron
status=repaired (19 issues,
  17 accepted,
  1041617 ms) and AkiraComplex
status=repaired (13 issues,
  13 accepted,
  501485 ms).
  The corrected band
ranking worked:
  the starved small band led this run.

## Accepted counts falling between rounds and the unmeasured recall

ACCEPTED COUNTS ARE FALLING SHARPLY,
  and the pattern has a natural control.
Across the four entries settled in BOTH rounds:
  192 -> 111 accepted,
  ratio
0.58.
  Per entry:
AmbeR_the_anpa 42 -> 41 (-2%),
  Acheron 45 -> 17 (-62%),
  AkiraComplex 28 -> 13
(-54%),
  Chinatsu_Suzuki 77 -> 40 (-48%).
THE CONTROL IS THE INTERESTING PART:
  AmbeR_the_anpa is the one of the four that
contributed NO clear false positive to the graded fifty,
  and it is the one that
barely moved.
  The three that carried a known false positive (Acheron item 7
identity,
  AkiraComplex item 16 unanchored,
  Chinatsu_Suzuki item 40 critical
PhotoScroll) all dropped by half or more.
  That is the shape a targeted fix
should produce,
  and it is not what indiscriminate suppression would look like.
ARITHMETIC CONSISTENCY,
  NOT PROOF:
  round one's graded precision was 0.56 to
0.68,
  so of 192 accepted roughly 107 to 131 were true positives;
  111 survive.
That is consistent with the fixes removing mostly false positives,
  but it
assumes the removal was perfectly targeted,
  which only grading can establish.
Do not quote the ratio as a precision measurement.
RECALL IS NOW THE OPEN RISK AND THE GATE DOES NOT COVER IT.
  The M3 headline
gate is PRECISION ONLY,
  so a pipeline that suppressed real defects along with
false ones would still pass it.
  Before declaring milestone three,
  either add a
recall check (the seeded-defect benchmark already in this package is the
natural instrument:
  `repair-benchmark.ts` measures restoration of KNOWN
injected defects,
  so it is unaffected by the precision sample) or state
explicitly that recall is unmeasured this round.
  Flag to the user;
  do not
decide it unilaterally.
FIX A AND F SIGNALS on their target entries,
  keyword-counted not graded:
Acheron identity-mentioning accepted claims 27 -> 4 (the regex matches any
summary containing name/alias/Acheron,
  so treat as indicative only);
AkiraComplex accepted claims carrying NO source span 5 -> 0,
  and Acheron 2 ->
0.
The unanchored drop was NOT gated by fix F,
   which only labels the case on
the sheet,
   so it is a side effect of the alignment and prompt changes and must
not be attributed to F.

## Container unwrapping confirmed on the entry it was built for

FIX B CONFIRMED ON THE ENTRY IT WAS BUILT FOR (2026-07-26).
   Chinatsu_Suzuki
re-settled in pass 7 run 004,
   and it is the container-nesting entry whose en
page collapses its gallery into one `<details>`.
   Round one vs round two,
accepted issues:
   PhotoScroll-referencing issues 13 -> 0,
   critical 7 -> 2,
accuracy/addition 34 -> 4,
   total accepted 77 -> 40.
WHY THIS ONE COUNTS as evidence where the AmbeR_the_anpa comparison did not:
the 13 PhotoScroll claims were exactly the graded false positive (sheet item
40,
   `accuracy/omission` CRITICAL) and its siblings;
   the change is CATEGORICAL
(to exactly zero,
   not a smaller number);
   and the mechanism is DETERMINISTIC --
unwrapping makes those blocks peers on both sides,
   so the critic no longer sees
a component present on one side and absent on the other.
   This is not
model-noise variation.
WHAT IT STILL DOES NOT SHOW:
   accepted fell 77 -> 40,
   and some of that drop
could be genuine defects no longer reported (recall loss),
   which only grading
can separate from the intended removal of misalignment artifacts.
   Do NOT quote
the accepted-count drop as a precision improvement.

## Band ordering fixed mid-pass

BAND ORDERING FIXED MID-PASS (2026-07-26,
   commit `a0fb61f6d`).
   The ~10/10/10
bar was UNREACHABLE on a fresh pass and runs 001-002 exposed it by settling two
mediums and nothing else.
   Measured corpus band split:
   31 small / 32 medium / 29
large.
   The driver sorted the small band LAST,
   so the first small entry could
only start after all 61 non-small settled:
   at the 36-57 min per entry measured
here,
   over a day of compute before the small band opens at all.
   Round one only
reached small=9 because those entries had settled in earlier passes;
   archiving
`attempts.json` and the artifacts wiped that inheritance,
   turning a mild skew
correction into a starved band.
FIX:
   order is now resume-first -> INTERLEAVE bands by within-band rank ->
larger band leads within a rank -> fewest attempts.
   This keeps the original
intent (small entries finish inside one run while large ones consume it,
   so
early settling over-represents small) but solves it symmetrically,
   reaching ten
per band in ~30 entries instead of ~71.
   Band logic extracted to
`corpus-run/band-order.ts` to keep the driver under max-lines (never disabled).
VERIFIED on `--plan`:
   first=Arita(large),
   Considerate_cat(medium),
Acheron(small),
   Chinatsu_Suzuki(large),
   Everythings99(medium).
OVERLAP WITH ROUND ONE IS NOT HIGH,
   correcting an earlier claim in this
session.
   Anilovr is second in the fresh queue yet was NOT among round one's 29,
so round one's settled set was never the head of this queue;
   it was shaped by
its own pass-4/5/6 attempt history.
   Round two's ORDER is deterministic,
   but it
does not reproduce round one's SET.
   Compare precision on the actual
intersection once the bands fill;
   do not assume it.

## Pass 7 runs 003 to 001 and the deterministic entry order

PASS 7 RUN 003 (2026-07-26,
   tip `a0fb61f6d`,
   3443744 ms ~57 min):
   1 settled,
3/92 (large 1 / medium 2 / small 0).
   Arita status=repaired (86 issues,
   85
accepted,
   85 resolved,
   5 findings).
   First run under the interleaved ordering.
PASS 7 RUN 002 (2026-07-26,
   tip `60a0ad3a6`,
   2789962 ms ~46 min):
   1 settled,
2/92.
   Anilovr status=repaired (81 issues,
   78 accepted,
   77 resolved,
   8
findings).
PASS 7 RUN 001 (2026-07-26,
   tip `c911b31a6`,
   2171621 ms ~36 min):
   1 settled,
1/92.
   AmbeR_the_anpa status=repaired (41 issues,
   41 accepted,
   41 resolved,
   4
findings,
   6 chunks).
   Same entry in round one:
   44 issues,
   42 accepted,
   42
resolved,
   4 findings,
   7 chunks.
   Observable deltas:
   chunk count 7->6 (the
aligner pairs differently),
   severity lost its lone `critical` and lone
`neutral`,
   and the category mix moved (accuracy/mistranslation 27->18,
accuracy/omission 11->15,
   accuracy/addition now 5).
   NOT EVIDENCE OF A
PRECISION CHANGE and must not be recorded as one:
   nothing here is graded,
   the
seven models are individually unreliable so counts move run to run regardless,
and this is a single entry.
   It is also a WEAK test of the fixes by construction
-- AmbeR_the_anpa contributed no CLEAR false positive to the graded fifty (its
two sampled items were the "Yes-ish" Bilibili gloss and a true positive),
   so
nothing here was expected to change.
   Only a graded round-two sample answers the
question.
ENTRY ORDER IS DETERMINISTIC AND MATCHES ROUND ONE's queue:
   `corpus-pass.ts`
sorts with a STABLE `toSorted` on resumable-first,
   then non-small-before-small,
then fewest-attempts.
   With the archive in place no id is resumable and every
attempt count is 0,
   so two of three keys are identically zero and ordering
collapses to the band split,
   with stability preserving the pinned-commit
listing order inside each group (`--plan` confirms:
first=AmbeR_the_anpa,Anilovr,Arita,Chinatsu_Suzuki,Considerate_cat).
   So round
two starts from the same queue round one did;
   an earlier claim in this session
that it would settle a DIFFERENT set was wrong.
   Divergence is confined to the
tail,
   where changed slice counts shift which entries fit inside each run's
soft budget.
   Consequence:
   do NOT add entry-id pinning to the driver for
comparability;
   if the settled sets diverge,
   compare precision on the
INTERSECTION,
   which is computable from the artifacts after the fact.
