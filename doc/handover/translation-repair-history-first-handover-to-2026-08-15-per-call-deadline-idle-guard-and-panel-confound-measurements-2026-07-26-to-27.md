# Translation repair history, first handover document: per-call deadline, idle guard and panel-size confound measurements, 2026-07-26 to 27

The measurements behind the per-call deadline,
with the records of pass 7 runs 013 and 014 that carry them:
censored and uncensored call timing,
the idle guard that proved useless on this provider,
the arithmetic on what a longer deadline costs,
and the check that panel coverage did not change between rounds.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Pass 7 run 014: the deadline raise confirmed by measurement

PASS 7 RUN 014 (2026-07-26,
  tip `6a381eb3c`,
  15401163 ms ~257 min):
  FOUR
settled,
  19/92,
  bands 6 small / 7 medium / 6 large.
  Huasheng repaired (LARGE,
249 issues,
  245 accepted,
  240 resolved,
  33 findings,
  resuming its cached
slices),
  Katerina (45/44/44,
  0 findings),
  Barron12312 (33/33/33,
  1 finding),
Kotori (97/94/94,
  7 findings).
  LCG_Akiball aborted at the 90 min entry cap with
slices cached for a later run.
THE DEADLINE RAISE IS CONFIRMED BY THE MEASUREMENT THAT MOTIVATED IT,
  which is
the cleanest result of the day.
  At 240 s,
  run 013 cut 35 of 783 calls (4.5
percent) and spent 7 retry rounds.
  At 360 s,
  run 014 cut 4 of 768 (0.5 percent)
and spent 1 retry round.
  Voice loss followed:
  1 short-handed stage of 86,
  versus
4 of 85.
  The user's hypothesis that the deadline was truncating real work was
correct,
  and the earlier conclusion against it was wrong.
THE TAIL IS NOW FULLY OBSERVED RATHER THAN CLIPPED.
  Over 764 sampled calls,
time to first byte ran p50 55_229 ms,
  p90 196_881 ms,
  p95 248_239 ms,
  p99
301_951 ms,
  max 347_099 ms. FORTY-FOUR calls finished between 240 s and 347 s,
every one of which the old deadline would have killed.
  The counts taper
properly to zero now (44 at or past 240 s,
  22 past 270 s,
  9 past 300 s,
  1 past
330 s,
  0 past 360 s),
  where at 240 s the distribution was still dense at the
cut.
  360_000 is therefore not merely better but SUFFICIENT:
  nothing reached it.
SIDE FINDING,
  and a second vindication of retiring the idle guard:
  the largest
mid-stream gap in run 014 was 43_845 ms,
  with p99 at 22_961 ms. The retired 30 s
window would have killed healthy streams outright,
  not merely come close.
  Gap
maxima keep growing with sample size (733 ms at 6 streams,
  24_673 ms at 32,
43_845 ms at 764),
  which is the same sample-maximum-is-not-a-bound lesson
arriving a third time.

## Pass 7 run 013: uncensored call timing and the deadline question

PASS 7 RUN 013 (2026-07-26,
  tip `065ab5bcf`,
  15521985 ms ~259 min):
  FIVE
settled,
  15/92,
  bands 5 small / 5 medium / 5 large,
  dead even and halfway to the
~10/10/10 bar.
  Futajuhuacha repaired at last (LARGE,
  214 issues,
  211 accepted,
210 resolved,
  43 findings) in 1266924 ms once runs 011 and 012 had cached 18 of
its 22 slices;
  then ArtsEpiphany (SMALL,
  status=unchanged,
  ZERO issues,
  11509 ms,
the first entry the pipeline found nothing to say about),
  GLaDOSister (87/83/82,
13 findings),
  BI4PBV (20/20/20,
  4 findings),
  Jennife80677612 (51/50/49,
  6
findings).
  Huasheng aborted at the hard cap and carries cached slices for a
later run.
  The soft-budget change (`54b3b6853`) is what did this:
  runs 010 to
012 settled one,
  zero,
  and zero entries respectively,
  and one launch now settles
five.
UNCENSORED CALL TIMING,
  THE MEASUREMENT THE DEADLINE QUESTION NEEDED.
  Run 013
sampled every model call:
  748 succeeded and 35 were killed at the 240 s
deadline,
  a censoring rate of 4.5 percent.
  Time to first byte over the 748 runs
min 412 ms,
  p25 2807 ms,
  p50 45_837 ms,
  p75 118_770 ms,
  p90 163_296 ms,
  p95
182_867 ms,
  p99 218_976 ms,
  max 235_151 ms. By threshold:
  45.3 percent take at
least 60 s,
  24.6 percent at least 120 s,
  13.9 percent at least 150 s,
  6.1
percent at least 180 s,
  2.0 percent at least 210 s,
  and 0 reach 240 s,
  the last
only because 240 s is where they are cut.
READING:
  this is a right-censored heavy tail with real density right up to the
boundary and NO cliff before it,
  which is the signature of a distribution being
clipped rather than of connections hanging.
  A call finishing at 245 s would be
unremarkable next to the 15 observed between 210 s and 235 s.
  So the 35 killed
calls are most likely slow-but-real,
  and THE USER'S HYPOTHESIS IS SUPPORTED:
240 s does truncate genuine work.
RECONCILING IT WITH THE CORRELATED-BATCH EVIDENCE,
  which looked contradictory:
run 013 lost 35 calls across only 7 retry rounds,
  so timeouts still arrive about
five at a time rather than independently.
  Both hold at once if the provider
slows ALL concurrent calls together under load,
  so a batch crosses 240 s
together.
  That explains correlation without requiring hangs,
  and it still means
a longer deadline would let those calls through.
  It also means the extra waiting
lands precisely during congested periods.
WHY NOT RAISE IT IMMEDIATELY ANYWAY:
  the guard fired ZERO times in run 013,
  and
it did not exist for the first ten entries,
  so all 15 settled entries share one
call-timing configuration (240 s total deadline,
  no silence aborting).
  The pool
is currently CLEAN despite the earlier mixed-cohort worry,
  and raising the
deadline now is what would split it.
  Voice loss under 240 s is also mild right
now:
  81 of 85 stages heard a full 7/7,
  with two critic stages at 5/7 and two
panel stages at 6/7,
  so the retry ladder is absorbing most of the censoring.
MID-STREAM GAPS,
  for the record:
  p50 64 ms,
  p90 673 ms,
  p99 9455 ms,
  max
28_116 ms,
  and nothing at or above 30 s.
  The retired 30 s window would have had
a 1.07x margin against the observed maximum,
  tighter still than the 1.2x that
condemned it.

## Idle guard shown not to work on this provider

THE IDLE GUARD DOES NOT WORK ON THIS PROVIDER,
  AND THE USER'S DEADLINE
HYPOTHESIS IS NOW THE BETTER-SUPPORTED ONE (2026-07-26,
  commit `68f11f602`).
A full sentinel probe on Aniloviraw settled it,
  and it reverses three claims
made earlier the same day.
FIRST,
  EVERY STALL IS FIRST-BYTE.
  The probe recorded 34 stalls and 34 of 34
carried phase `first-byte`;
  NOT ONE was `body`.
  Mid-stream death,
  the failure
mode the guard was built to catch,
  did not occur at all.
  Long first-byte silence
is normal operation here:
  across 32 successful streams,
  time to first byte ran
p50 95.6 s,
  p75 123 s,
  p90 134 s,
  max 147.5 s.
  No silence window can separate
"stalled and silent" from "working and silent" when working looks like that.
SECOND,
  THE MID-STREAM WINDOW WAS UNSAFE,
  and its safety argument was the
clearest reasoning error of the day.
  It was justified on six streams whose
largest inter-chunk gap was 733 ms,
  described as a 40x margin under a 30 s
window.
  At 32 streams the gap distribution reads p50 86 ms,
  p90 3833 ms,
  max
24_673 ms,
  with three streams past 20 s.
  The real margin was about 1.2x.
  A
maximum over a handful of samples is not a bound,
  and treating it as one
inverted a safety claim.
THIRD,
  THE GUARD COST THROUGHPUT rather than saving it:
  the probe took 45.8 min
against the 23.9 min the same entry took ungarded in run 009,
  which is what
killing 34 in-flight calls and re-dispatching them buys.
  Result comparison
against that run:
  29 issues / 28 accepted / 5 findings versus 27 / 27 / 6,
  so
the drain is not proven behavior-neutral,
  though the pipeline is stochastic
across seven models and run-to-run variance is expected regardless.
REMEDY:
  both windows now sit ABOVE the 240 s per-call deadline so the guard
never fires.
  It is retained purely as instrumentation,
  because the incremental
drain is what made any of this observable,
  and the total deadline is once again
the only thing that kills a call.
  NO ARTIFACT EVER SETTLED UNDER THE ACTIVE
GUARD,
  so nothing in the pool is contaminated by it;
  runs 011 and 012 both spent
their whole budget on Futajuhuacha without settling anything.
WHAT THIS MEANS FOR THE OPEN QUESTION:
  healthy first-byte reaches at least
147.5 s against a 240 s deadline,
  and that 147.5 s is the GUARD'S OWN SHADOW,
not the true tail,
  because the 150 s window aborted anything slower instead of
recording it.
  So the true healthy tail is unmeasured and may well cross 240 s.
The user's hypothesis that the deadline is truncating real work is now the
better-supported reading,
  and the retry evidence does not contradict it.
  Stream
sampling is therefore UNFILTERED from `68f11f602` on (`NOTABLE_FIRST_BYTE_MS`
and `NOTABLE_GAP_MS` both 0,
  one log line per model call),
  because any positive
threshold censors exactly the tail the question turns on.
  The next corpus run
yields the first uncensored time-to-first-byte distribution;
  read it before
deciding the deadline.

## Stream idle guard landed and the censored sample mistaken for the healthy range

STREAM IDLE GUARD LANDED,
  AND IT PARTLY REOPENS THE DEADLINE QUESTION
(2026-07-26,
  commits `cacc1fa8b` guard plus drain,
  `3cf83fab1` tests,
`b59a81329` retune,
  `8b2c3670f` correction).
  User chose "land now,
  keep the 10
settled" over the recommendation to finish round two first,
  so the round-two
pool now spans two configurations by explicit decision.
  Mitigation,
  so the cost
is measurable instead of merely accepted:
  every artifact from here carries
`callConfig` (`RUN_CALL_CONFIG` in `run-config.ts`),
  and its ABSENCE identifies
the ten pre-guard entries exactly,
  so round-two precision can be split by cohort
at analysis time.
  Deliberately NOT shown on the grading sheet,
  since a grader
who can see the cohort is a worse instrument.
DESIGN:
  `armIdleGuard` (`stream-idle-guard.ts`) aborts on silence rather than
elapsed time,
  armed BEFORE the request so it also covers a provider that never
sends headers.
  It aborts its OWN controller,
  never the caller's,
  which is what
makes `attemptExchange` treat the failure as transient so `exchangeWithRetry`
re-dispatches at transport level on a ~1 s backoff instead of the stall
escalating into another stage round.
  `drainBody` (`stream-drain.ts`) replaces
`response.text()` with a `getReader` loop that timestamps chunks and still hands
the concatenated text to the existing reassembler,
  so parsing behavior above the
transport seam is unchanged.
A CENSORED SAMPLE WAS BRIEFLY MISTAKEN FOR THE HEALTHY RANGE,
  corrected in
`8b2c3670f`;
  the reasoning trap is worth keeping.
  A sentinel probe logged six
healthy calls reaching first byte at 84,
  104,
  122,
  132,
  135,
  and 147 s,
  and
those were written into two docblocks and a commit body as "the measured healthy
range".
  They cannot be:
  the drain only logged exchanges slower than its own
60 s notability filter,
  so everything faster was absent BY CONSTRUCTION.
  Pass-7
stage timings refute it independently,
  since a stage ends only when its slowest
voice returns and the tenth percentile of succeeding rounds is 9 s,
  which no
84 s first byte allows.
  The six bound the healthy tail at 147 s or more and say
nothing else.
  Commit `b59a81329`'s message still carries the overstatement and
is not amended per GCA.
CONSEQUENCE FOR THE USER'S HYPOTHESIS:
  the healthy first-byte tail reaches at
least 147 s against a 240 s deadline,
  a much narrower margin than the earlier
framing implied,
  and the shape of that tail above 147 s is UNKNOWN.
  So whether
240 s cuts into real work is OPEN,
  not settled.
  The retry-recovery evidence
below still stands on its own,
  and the two are not in conflict:
  a fresh dispatch
recovering 7/7 in a median 88 s is evidence about what re-asking achieves,
  not
about where the healthy tail ends.
WHAT THE GUARD IS AND IS NOT WORTH:
  its mid-stream window is well founded,
  since
the largest gap across six streams carrying up to 745_015 characters was 733 ms,
so 30 s cannot plausibly fire on a healthy stream.
  But those gaps come from
streams that SUCCEEDED and say nothing about how a dying stream behaves.
  If the
real failure mode is first-byte silence,
  the guard buys almost nothing and the
9.4 percent ceiling is untouched.
  The `phase` on `StreamStalledError`
(`first-byte` or `body`) is the instrument that settles it,
  and ONE corpus run
answers it.
  Do not build further on the guard before reading that phase.

## Measured cost of raising the per-call deadline

RAISING THE 240 s PER-CALL DEADLINE WOULD MAKE THE SYSTEM SLOWER,
  MEASURED
(2026-07-26,
  user hypothesis "I suspect increasing the 240s deadline could make
the system overall faster").
  The hypothesis has a real mechanism behind it:
`stage-quorum.ts` grants `STAGE_RETRY_ROUNDS = 3` after the initial fan-out,
  so
one stage can burn four consecutive deadlines,
  and a call killed at 240 s is NOT
retried by `exchangeWithRetry` (a deadline aborts `exchange.signal`,
  and
`attemptExchange` rethrows caller aborts untouched rather than treating them as
transient),
  so the recovery happens one level up at the stage.
  Deadline-induced
waste is therefore real.
  It is also bounded:
  across the twelve pass-7 logs,
14 rounds of 417 timed out,
  each costing exactly the full 240 s,
  totalling 56 of
598 wall minutes,
  or 9.4 percent.
  That 9.4 percent is the CEILING on any
speed-up from eliminating timeouts entirely.
WHAT THE TIMED-OUT ROUNDS ACTUALLY ARE:
  correlated stalls,
  not slow generation.
Every timed-out round lost 4,
  5,
  6,
  or 7 of its 7 voices at once;
  NOT ONE lost
just one or two,
  which is the shape model-specific slowness would take.
  The
decisive measurement is the retry that follows.
  Of 13 retry rounds,
  12 recovered
to a full 7/7,
  and their durations were 27,
  48,
  57,
  67,
  81,
  84,
  88,
  90,
  173,
175,
  213,
  233 s,
  with only the thirteenth spending 240 s and settling at 5/7.
Median 88 s.
  The same voices that could not answer inside 240 s answered inside
88 s on a fresh dispatch.
  That refutes the competing "these are the big-prompt
rounds where all seven genuinely need longer" reading,
  which predicts the retry
times out too.
  A fresh dispatch clears the condition,
  so the wait is not
buying generation progress.
COUNTERFACTUAL,
  stated as arithmetic on those measurements rather than as a
claim about unrun configurations:
  one stall event today costs 240 s wasted plus
an 88 s median retry,
  about 328 s.
  At a 480 s deadline the stalled call still
does not answer,
  so it costs about 568 s,
  roughly 73 percent worse per event and
about +9 percent on total run time.
  The deadline is also already well placed
against healthy work:
  succeeding rounds run p50 60 s,
  p90 187 s,
  p99 240 s,
  so
240 s sits just above the healthy tail and lowering it flatly would start
killing real generations.
THE CHANGE THE INTUITION IS ACTUALLY POINTING AT is an IDLE deadline instead of
a total-duration one.
  `armCallDeadline` arms a plain total-duration timer inside
the limiter slot (`call-deadline.ts`),
  so a healthy long generation and a dead
stream are indistinguishable to it.
  An idle timer,
  aborting after N seconds with
no bytes,
  would catch the correlated stalls in N seconds instead of 240 and
would never kill a healthy long generation,
  improving BOTH throughput and voice
retention.
  Implementable but NOT free:
  `stream-completion.ts` reassembles from a
whole drained `bodyText`,
  so the transport currently has no per-chunk arrival
time and would need a `getReader` loop that timestamps chunks and still hands
the concatenated text to the existing reassembler,
  preserving parsing behavior.
The saving estimate assumes stalls emit no bytes at all rather than trickling,
which the whole-text drain means NOBODY HAS VERIFIED yet;
  verify before quoting
a number.
BLOCKED ON A USER CALL,
  not on analysis:
  `RUN_PER_CALL_TIMEOUT_MS` is the one
budget that changes what the pipeline finds,
  and all round-two entries so far
were produced under 240 s,
  so touching it mid-accumulation leaves a
mixed-configuration corpus and the round-two precision number stops being
comparable to round one.
  Speed versus measurement validity is the user's
tradeoff to make.

## Panel-size confound checked before the round two draw

PANEL-SIZE CONFOUND CHECKED AND CLEARED (2026-07-26),
  before any round-two
sheet is drawn.
  The seven models drop voices under the 240 s per-call deadline,
so a chunk can be adjudicated short-handed;
  if round two lost voices at a
different rate than round one,
  a precision delta would be partly a panel-size
artifact rather than a measure of fixes A-F.
  Measured over every `critic stage:`
and `panel stage:` line.
  Round one (pass 4 + 5 + 6):
  56 short-handed of 724
stages,
  7.7 percent,
  worst case 4/7 on six stages.
  Round two (pass 7 to date):
10 of 205,
  4.9 percent,
  worst case 5/7 and no 4/7 at all.
  Round two therefore
runs with marginally BETTER panel coverage,
  so the confound cannot manufacture
an improvement of the size the gate needs;
  it is small and points the optimistic
way,
  which is the direction that must be disclosed rather than corrected for.
Degraded chunks STAY in the precision denominator:
  the 0.9 bar is for the
pipeline as it actually runs on seven unreliable flat-rate models,
  not for a
full-panel ideal,
  so excluding them would measure a pipeline that does not
exist.
  Report the rate alongside the round-two verdict.

## Panel-size confound re-measured per deadline cohort

CONFOUND RE-MEASURED PER DEADLINE COHORT (2026-07-27),
  because raising
`RUN_PER_CALL_TIMEOUT_MS` to 360_000 mid-accumulation split round two itself
into two timing cohorts and could have widened the very gap just cleared.
Short-handed stages over every `critic stage:` and `panel stage:` line:
round one 56 of 724 (7.7 percent);
  round two under 240 s,
  pass-7 runs 001 to
013,
  17 of 305 (5.6 percent);
  round two under 360 s,
  pass-7 runs 014 onward,
3 of 149 (2.0 percent).
  The 5.6 against 2.0 POOLED COMPARISON IS NOT EVIDENCE
that the longer deadline retains voices,
  and must not be quoted as if it were.
Per-run rates in the 240 s cohort are 0,
  0,
  0,
  0,
  0,
  0,
  4.5,
  4.7,
  8.3,
  8.3,
13.6,
  14.2,
  and 20.0 percent,
  against 1.1,
  2.1,
  and 5.5 percent at 360 s.
  Every
360 s run falls INSIDE that spread and six 240 s runs beat all three of them,
so the pooled gap is driven by runs 011,
  012,
  and 002 rather than by the
deadline.
  Three runs cannot outvote thirteen on a statistic whose per-run
spread is this wide.
The same disqualification applies to the call-level timeout rate:
  4.8 percent
at 240 s (run 013,
  36 of 748) against 0.65,
  2.19,
  and 6.49 percent at 360 s
(runs 014,
  015,
  017),
  where the spread within the 360 s cohort again exceeds
the gap between cohorts.
  Provider load dominates both statistics,
  and no single
run is quotable as a before or after.
What survives is one WITHIN-RUN observation,
  which holds load roughly fixed
instead of comparing across it:
  run 017 drew the cohort's worst call-level rate,
10 timeouts in 154 calls,
  yet lost only 1 stage of 18,
  because the extra
headroom let its retries land.
  That is consistent with the deadline buying voice
retention and is the mechanism to watch,
  but it is a single run and is NOT
demonstration.
  For the confound the gate actually cares about,
  the honest
statement is narrower and still sufficient:
  every round-two sub-rate sits at or
below round one's 7.7 percent,
  so panel coverage did not DEGRADE between rounds
and cannot have manufactured a precision improvement.
  Report both sub-rates and
this reasoning with the round-two verdict;
  claim no deadline effect.
