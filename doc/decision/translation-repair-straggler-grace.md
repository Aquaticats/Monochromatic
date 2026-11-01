# The straggler grace moves from 60 to 180 seconds, on the latency distribution

## Current status, 2026-10-22

The archive block review round also waits the writer window,
a classification this session made and the owner has not confirmed
(addendum 2026-10-21).

The grace may now start on a short bench that holds exactly its quorum,
so it can cut the last live seat,
by the owner's answer "Let the grace open" (decision 2026-10-22).
No window changed.

## Status of 2026-09-06

Three windows are built in,
each the owner's:
every round waits 120000 ms after quorum (`STRAGGLER_GRACE_MS` in `stage-round.ts`,
decided 2026-09-03);
the writer rounds,
editor,
refiner,
translate and consolidate,
wait 180000 ms (`WRITER_GRACE_MS` in `writer-grace-override.ts`,
decided 2026-09-06),
or the round window when a launch made that the longer one;
and the editor calibration runs every round at 300000 ms (decided 2026-08-26).
`TRANSLATION_REPAIR_STRAGGLER_GRACE_MS` and `TRANSLATION_REPAIR_WRITER_GRACE_MS` move the first two for one
launch.

## Superseded status, 2026-08-29

Built-in pass grace remains 180000 milliseconds.
Generation 12 changes when it starts:
exact-half participation opens grace for every production stage,
and remaining seats may answer until bound expires.
Every-seat participation is no longer a requirement.
Departed GLM-4.7-Flash,
GLM-5.2,
qwen3.8-max,
and Nemotron evidence remains historical and does not set current roster latency.

Active entry-level timing work is
`doc/planning/translation-repair-entry-time-to-complete.md`.
No grace change is authorized before active runs finish and matched evidence exists.

Taken 2026-08-17 on the best-quality guideline,
against measurement rather than preference.

THIS OVERRIDES A NUMBER THE OWNER CHOSE.
Sixty seconds was their figure on 2026-08-15,
picked
between a grace window and cutting at quorum outright.
It is flagged here for a cheap veto.
What
changed is not the judgement behind it but the evidence:
the constant's own comment recorded that it
was "not derived from the latency distribution,
and it should be revisited against one",
and that
revisit is what this document is.

## The question the queue asked

The task queue and the handover both framed it as a choice:
widen the deadline for `hf:zai-org/GLM-5.2`,
or
seat a replacement.
The measurement answers it,
and it answers against replacement.

## What the loss actually is

Over the two-lane cost run of 2026-08-16,
under the production roster,
96 voices were lost:

-   91 were `abandoned 60000ms after quorum`,
    which is this window firing.
-   6 were `schema-mismatch`,
    a different fault entirely.
-   By model:
    `hf:zai-org/GLM-5.2` 68,
    `hf:zai-org/GLM-4.7-Flash` 27,
    `hf:moonshotai/Kimi-K3` 1.

So the loss is almost entirely this window,
and almost entirely the two GLM models.

## Why replacement is the wrong answer

THE MODELS ARE NOT DEFECTIVE.
Whole-call latency over the 602 bench exchanges in
`doc/audit/translation-repair-lane-budget.md`:

```text
hf:openai/gpt-oss-120b                  p50  4.4 s   p95  8.3 s   max 10.6 s
hf:moonshotai/Kimi-K3                   p50  9.5 s   p95 30.0 s   max 46.5 s
hf:nvidia/NVIDIA-Nemotron-3-Super       p50 12.9 s   p95 41.1 s
hf:Qwen/Qwen3.6-27B                     p50 23.7 s   p95 38.3 s   max 48.0 s
hf:zai-org/GLM-5.2                      p50 24.0 s   p95 74.0 s   max 85.5 s
hf:zai-org/GLM-4.7-Flash                p50 30.5 s   p95 72.9 s   max 88.6 s
```

Sixty seconds sits between the GLM medians and their 95th percentiles.
That is the whole
explanation:
the window cuts those two models and no others because it was set inside their ordinary
operating range.
Their worst observed call still finished in 88.6 seconds.

In the same run,
`hf:zai-org/GLM-5.2` was heard in full:
220 stages reported `6/6 heard`,
against 61
at `5/6` and 2 at `4/6`.
A model that answers in three quarters of the stages it is seated in is
slow,
not broken,
and removing it would discard a working voice to work around a mis-set constant.

## Why the window has never done its job

Across those 602 exchanges the only non-ok outcomes were the 8 straggler cuts,
all on the two GLM
models.
No truncation,
no schema-invalid reply,
no timeout of any other kind.

NOT ONE HUNG CALL WAS RECORDED.
The window exists to stop a model that answers nothing from delaying
the pipeline,
and in 602 exchanges it has never once met that model.
Every voice it has taken was a
slow-but-working one.

## Why three minutes, and not the observed maximum

Setting it at 90 seconds would clear the 88.6 second maximum and would repeat a mistake this
codebase has already made and written down.
`STREAM_IDLE_MS` in `stream-idle-guard.ts` carries the
record:
a 30 second window was once justified by a six-stream sample whose largest gap was 733 ms,
called forty times the worst observation,
and a larger sample then read max 24673 ms,
so the real
margin was about 1.2x rather than 40x.
A maximum over a few hundred samples is not a bound.

180000 ms sits above the 88.6 second maximum by more than a factor of two,
and stays well under
`RUN_PER_CALL_TIMEOUT_MS` of 360000,
so the window still cuts a genuinely hung voice long before its
own deadline would.
The user's rule of 2026-08-15,
that one model's failure must not delay the
pipeline for the day,
is preserved:
a voice that never comes is now abandoned at roughly 193 seconds
from dispatch rather than 73,
against a deadline of 360.

AND THAT MAXIMUM IS NOT A CORPUS BOUND EITHER,
which this document would be repeating its own lesson
to ignore.
The bench's ten slices spanned 94 to 497 incumbent characters,
which the same audit says
samples the corpus only to about its 90th percentile:
slice size runs p99 1512 characters and
`shihai4h` carries 10959 in a single slice.
A model's call grows with what it is asked to read,
so
on the largest slices these two may still exceed 180 seconds and still be cut.

That does not change the decision,
because the fallback is exactly today's behaviour,
a lost voice
on a large slice.
It does mean the re-measure should read losses BY SLICE SIZE rather than as one
rate,
since a residual concentrated in the tail is a different finding from one spread evenly.

## What it costs

MORE THAN THIS SECTION'S ESTIMATE,
and the estimate is left standing because the re-measure contradicted
it and that is worth showing rather than hiding.

The estimate was:
the window is only paid when a voice is late,
a voice that answers before it costs
nothing,
and a voice that would have been cut costs the difference between its arrival and the old
60 second cut.
Over the cost run,
63 of 283 six-voice stages lost a voice;
at roughly 20 seconds
more each that is about 21 minutes across a seven and a half hour run,
under half a percent.

WHAT THE RE-MEASURE ACTUALLY REPORTED for `zheermao101`,
the first entry through:
5039467 ms against
3917494 ms under the old window,
29 percent longer.
That is far more than the estimate.

The estimate is not simply wrong about the window;
it is incomplete about what recovering voices
does.
The same run also reports 50 issues against 37,
29 accepted against 22,
and 26 resolved
against 20.
Panels that hear six voices find more than panels that hear five,
and finding more costs
more.
The extra time is buying extra work,
not waiting.

TWO THINGS THIS DOES NOT ESTABLISH,
both of which need saying:

-   THE RUN-TO-RUN BAND WAS NEVER MEASURED.
    One run against one earlier run cannot separate a 29
    percent change from ordinary provider variance,
    and no unchanged-build repeat exists to say how
    wide that band is.
    The number is what happened,
    not what the window costs.
-   HOW MUCH OF IT IS THE WINDOW.
    Arithmetic says little of it:
    6 of 58 stages lost a voice under
    the old window,
    each would now wait at most tens of seconds longer,
    which is minutes rather than
    the 18.7 observed.
    The rest is the wider judging that more surviving voices produce.

The honest summary is that the window costs more than a rounding error and buys more than voices.
Whether the trade is worth it is a quality question the corrected numbers can now be argued from.

## What was rejected, and why it is worth recording

CUTTING ON SILENCE RATHER THAN ELAPSED TIME,
which is the obvious better instrument and does not
work on this provider.
`stream-idle-guard.ts` records the measurement:
of the stalls a full sentinel
probe recorded,
34 of 34 were `first-byte` and not one was `body`,
and across 32 successful streams
time to first byte ran p50 95.6 s,
p75 123 s,
p90 134 s.
Long first-byte silence IS normal operation
here,
so a silence window cannot separate stalled from working.
Both idle constants are set to
600000 so they never fire,
and the total deadline does the killing instead.

That is the same shape of answer as this one:
when a guard cannot discriminate,
do not tighten it,
move the killing to the deadline that can.

## The baseline to compare against, on one definition

THREE DIFFERENT RATES ARE IN CIRCULATION and none of them measure the same thing.
The trial's 0.413
counts judgings;
the cost run's loss LINES number 96 and count retries,
so one voice lost twice
counts twice;
and stage completeness is a third number again.
A re-measure against the wrong one
would report a change that is only a change of denominator.

The definition this fix is measured on is PER-STAGE VOICE COMPLETENESS:
a stage counts as losing
when it heard fewer voices than it seated,
whatever the roster size,
counted once per stage.
Under
that definition,
recomputed from the same cost-run log the fix was diagnosed from:

```text
Aniloviraw     5 of  22 stages   0.227
zheermao101    6 of  58 stages   0.103
aiyysk        30 of 208 stages   0.144
XingZ60       22 of 195 stages   0.113
whole run     63 of 483 stages   0.130
```

That 0.130 is the number the new window has to beat,
and it is not comparable to the 0.413 quoted
elsewhere.

## What must follow

RE-MEASURE THE DECLINE RATE,
which is the recorded next step and the reason this was first in the
queue.
Every measurement taken through a panel that lost voices was taken through this defect,
so
nothing resting on one should be trusted until the rate is read again.

TWO CONSTRAINTS ON THAT RUN,
both learned from getting them wrong elsewhere:

-   COMPARE ON PER-STAGE VOICE COMPLETENESS,
    not on loss lines and not on judgings.
-   RUN ENTRIES THAT ACTUALLY LOST VOICES,
    or a clean result proves nothing.
    The settled pair
    qualifies:
    `Aniloviraw` lost on 5 of its 22 stages and `zheermao101` on 6 of 58,
    so 11 of 80
    stages is the positive control,
    and a run of those two that still loses at that rate says the
    fix did not work rather than that there was nothing to fix.
-   READ THE RESIDUAL BY SLICE SIZE,
    per the caveat about the bench's slice range.

The per-slice cost telemetry now carries an `exit` key,
so the same run reports what the wider
window costs directly rather than by the estimate in "What it costs".
That run is also the telemetry's first
emission in production,
so it doubles as the user-boundary check that `SLICE-COST` lines appear and
parse.

## A second reading, on a fresh population, 2026-08-17 evening

THE RE-MEASURE THIS DOCUMENT ASKED FOR WAS ALREADY DELIVERED,
the same morning,
in
`doc/audit/straggler-grace-remeasure.md`.
It ran the named control,
`Aniloviraw` and `zheermao101`,
on the definition fixed here,
and recorded 3 of 77 stages against 11 of 80,
or 0.039 against 0.138.
Nothing in the second reading displaces it.

What follows is a second reading taken from a different population:
the evening corpus pass,
five entries none of which appear in that audit
or in the baseline table of "The baseline to compare against,
on one definition".
It was read from a frozen snapshot,
`~/temp/agent/grace-remeasure-snapshot.log`,
3271 lines,
taken at 23:34 UTC with four entries settled and the fifth mid-flight.
Frozen first because the live log grows between commands,
and two readings of it disagreed by six stages.

```text
critic    seats 6     4 of  37 stages   0.108
panel     seats 6     6 of  32 stages   0.188
checker   seats 3     0 of  40 stages   0.000
probe     seats 3     0 of  41 stages   0.000
pooled  (critic, panel, checker)  10 of 109   0.092
```

It agrees with the morning audit on shape:
the three-seat stages lose nothing,
the six-seat stages carry every loss,
and the pooled figure sits below the 0.130 recorded here.
Agreement on a population neither reading shares is worth more than either alone.

### What is new, and what it changes

THE DEFINITION CANNOT SEE THE TRANSLATE LANE,
which neither this document nor the morning
audit accounts for,
because both predate it.
`translate-document.ts:391` logs selection only when ZERO translators were heard,
so a stage that seats six and hears five leaves no `heard` line to count.
Of the 30 voices lost in this snapshot,
17 were lost in `select` and one in `refiner`,
neither of which the definition can see.
The instrument counts 10 of 30 actual losses.

This does not invalidate either reading,
since both sides of each comparison are measured the same way.
It does mean the figures understate loss in a two-lane pipeline,
and that any future comparison should extend the definition first.
That work was filed with client-side runaway termination.

EVERY ABANDONED VOICE IN THIS SNAPSHOT IS A GLM,
at a much larger scale than the single
surviving abandonment the morning audit found.
Of 30 `voice lost` lines,
29 attribute to a stage and a model,
and all 29 name `hf:zai-org/GLM-5.2` or `hf:zai-org/GLM-4.7-Flash`.
Zero name `hf:moonshotai/Kimi-K3`,
which appears more often in this log than any other model.
That is a zero-numerator fact rather than a rate,
since appearance counts are log mentions and not calls.
It supersedes the earlier finding that Kimi-K3 dominates voice loss,
which was measured before the channel-marker fix at the parser landed.

ABANDONMENT IS THE ONLY LOSS MODE HERE,
so nothing else is quietly eating voices.
Losses reconcile against abandon lines stage by stage:
critic lost 4 stages against 4 abandons,
checker lost 0 against 0,
panel lost 6 stages against 7 abandons,
the extra being one stage that lost two voices at once.

### The residual is not shown to be a latency finding

The morning audit read the one surviving abandonment as the tail this document warned about,
and priced it against bench slices of 94 to 497 characters.
That reading stands,
but it cannot be pushed further than the evidence goes.

An abandoned call is killed at quorum plus 180 seconds,
so its true duration is censored:
unknown,
and strictly greater.
The bench maxima of 85.5 s and 88.6 s were measured over calls that COMPLETED.
Whether a GLM call still running at 180 seconds is merely slow,
or has stopped producing anything new and would never have finished,
cannot be told apart from anything currently recorded,
because an aborted stream discards the partial text and no raw output is stored anywhere.

The user reports that this provider does not terminate token degeneration.
If that is what some of these calls are doing,
no widening of this window would ever have helped them,
and the remedy is to end them ourselves.
That is `doc/decision/translation-repair-runaway-call-termination.md`.

## Addendum 2026-08-26: the window has a dial, and single runs cannot price it

`TRANSLATION_REPAIR_STRAGGLER_GRACE_MS` (`grace-override.ts`,
landed `4c070f729`) overrides the 180000 ms window
for one run;
a run under an override prints `STRAGGLER GRACE OVERRIDDEN by ...` on its first line.
The built-in
value is unchanged.

MEASURED ON THE FOUR-SLICE EDITOR CALIBRATION,
one run per arm,
same slices,
same build.
Arm C (300000 ms,
one slice in flight) took 53.87 min against arm A's 43.19 (180000 ms),
bought back 2 of 6 cut voices,
all
`qwen3.8-max`,
and burned the full 300 s on four rounds that were cut anyway.
That read as +24.7% wall clock
per 2 voices.

THEN ARM A2 REPEATED ARM A UNCHANGED and took 58.95 min:
the calls themselves were slower (stream sum
9294 s against 6312 s),
8 voices were cut instead of 6,
and nothing about the window had moved.
Normalized
as wall clock over stream sum,
A 0.41,
A2 0.38,
C 0.43:
the window's cost sits inside the run-to-run band.
The 2-of-6 recovery is inside what provider speed alone moved.
So this decision stays at 180000 ms not
because the longer window was shown to cost too much,
but because nothing about it is shown at this scale.
What would show it:
interleaved repeats (A,
C,
A,
C) in one sitting,
read normalized,
or arm D's overlap-4
reading (running as this is written),
where the wait a longer window adds is what overlap fills.

DECIDED 2026-08-26,
LATER THE SAME DAY:
the editor calibration runs under 300000 ms together with four slices
in flight,
by the owner's answer to question 12 in `doc/planning/translation-repair-open-decisions.md`,
on arm
D (29.31 min,
318 of 320 voices,
2 cut,
normalized 0.23 against arm B's 0.23).
`adoptCalibrationGrace` in
`grace-override.ts` applies it through the same variable a launch can set.
This value,
180000 ms,
stays the
pass's until the overlap dial gives the pass overlap;
the two move together or not at all.
Record:
`doc/decision/translation-repair-calibration-overlap.md`.

## Addendum 2026-09-02: writer rounds have a dial of their own

`TRANSLATION_REPAIR_WRITER_GRACE_MS` (`writer-grace-override.ts`,
landed `a3da317df`) gives the editor,
refiner,
translate and consolidate gathers a window of their own for one launch;
unset or blank,
they follow
the round window exactly as before,
and the built-in value is unchanged.
A launch that sets it prints
`WRITER GRACE OVERRIDDEN by ...` beside the round note.

WHY A SECOND DIAL:
the four-entry pass of 2026-09-02 ran the round window at 60000 ms for the eight-wide
reader rounds' sake (75 to 83 percent of round time was waiting after quorum,
and the last reader voice adds
one ballot to seven),
and that window cut the top editor,
`hf:zai-org/GLM-5.3-Flash`,
mid-reply in 3 of the
first 22 three-wide editor rounds,
every cut exactly 60000 ms after quorum.
A writer round loses a whole
candidate when the window closes;
a reader round loses a ballot.
One window cannot be sized for both,
and
sizing it for the readers unseats the writers the calibration seated.
Record and the pre-registered rule
for the dial's value:
`doc/planning/translation-repair-roster-calibration-2026-09-01.md`.

This addendum records a launch-time dial;
it moves no decision.
Whether per-role windows become the built-in
design is the owner's call.

## Decision 2026-09-03: the built-in window is 120 seconds

Decided by the owner,
asked with four options after two keyword233 passes on the all-OpenRouter bench
(Synthetic and Hyper keys unset,
everything else identical) put the 60 s launch dial of 2026-09-02 against 120 s:

- 60 s (`~/temp/agent/openrouter-live3-20260903.log`):
  tally 1,247 s,
  14 cut streams,
  every one a reasoning
    stream still working (Qwen3.8-27B 7 of 38 asks,
  DeepSeek Flash 6 of 27,
  DeepSeek Pro 1 of 37),
    19 voices never heard,
  rounds waiting after quorum 82.7 percent of round time,
  0.76 USD.
- 120 s (`~/temp/agent/openrouter-live4-20260903.log`):
  tally 1,321 s,
  7 cut streams (Qwen 6,
  gemma 1),
    8 voices never heard,
  waiting after quorum 91.7 percent,
  0.45 USD.
- The tally difference sits inside the day's run-to-run band on the same entry (957 to 1,321 s over four
    runs),
  so the wall-clock price of the longer window is unresolved at this scale;
  the voices are not.

Options offered,
ranked 120 s over 180 s over 60 s over withholding the slow seats from OpenRouter-served
judge benches:
120 is measured where 180 is not on this bench;
60 loses fourteen voices for a saving inside
the noise;
the withheld seats' finished answers were all usable.
The owner chose 120 s.

`STRAGGLER_GRACE_MS` in `stage-round.ts` is now 120_000;
the launch dial is no longer needed for the pass.
Writer rounds follow it unless `TRANSLATION_REPAIR_WRITER_GRACE_MS` is set;
today's passes set 180000 there,
and that per-role figure stays a launch dial,
as the 2026-09-02 addendum records.

## Decision 2026-09-06: writer rounds wait 180 seconds built in

Decided by the owner,
asked with three options once the pass overlap fallback had moved on the same defect
(`doc/decision/translation-repair-pass-overlap.md`):
a production default that has never shipped a page.
Every page that shipped since the writer dial landed ran its writers at 180000 ms through
`TRANSLATION_REPAIR_WRITER_GRACE_MS` while the round window was 120000 ms,
so a launch without the variable would have run a configuration no page had been read under.

The evidence put beside the question,
from the four shipped logs of 2026-09-04
(`luxuanwen3`,
`SS3B_0016`,
`Uekawakuyuurei`,
`MTF_0615`):
writer-round cuts at 180 s were 4,
6,
11 and 20,
almost all in `produceConsolidations`,
against 28,
35,
26 and 135 reader-round cuts at 120 s.
Not measured:
writers at 120 s,
which has no matched arm,
and how many writer voices the extra 60 seconds bought.

Options offered,
ranked build 180000 in over keep the launch dial over writers at the round window:
the built-in reproduces every shipped page's configuration with nothing to remember;
the dial leaves production depending on an operator;
the round window for writers is the one option with no page behind it.
The owner chose the built-in.

`WRITER_GRACE_MS` in `writer-grace-override.ts` is 180_000.
Writers never wait less than every other round,
so under the calibration's 300000 ms they follow the round window,
and under a launch that shortens the round dial to 60000 ms they keep 180000,
which is the 2026-09-02 launch the dial was made for.
The dial still moves the writers for one launch in either direction.
Every launch now prints `WRITER GRACE built in` beside the round note,
so a log never hides which window its writers ran under.

## Addendum 2026-08-27: one same-digest hard-page pair

Fixed-build `Toka_ls` ran at overlap 4 under both windows.
At 180 seconds it took 114.72 minutes,
normalized `0.132`,
and lost 72 voices.
At 300 seconds it took 127.43 minutes,
normalized `0.140`,
and lost 35 voices.
The longer arm produced page with no blocker or major;
shorter arm retained inherited major after 8-voice contest tied 4 to 4.

This does not change built-in value.
Live ballots differed in issue inventory and stream count,
so one arm each cannot attribute output or timing difference to grace.
It does establish next repeated matched comparison has positive control:
same hard page,
overlap 4,
fresh roots,
with contest completeness and actual page quality read beside normalized runtime.

## Addendum 2026-10-21: the archive block review round is a writer round

This session classified the archive block review round as a writer round under the owner's rule of 2026-09-06.
The decision is open to the owner's veto.

The reason is the owner's own:
a cut writer voice is a whole candidate lost.
The review asks each reviewer for the complete corrected block,
so a `revise` reply is a whole candidate,
and the window that cuts it takes that candidate from the judges.
The 2026-09-06 decision listed the writer rounds as editor,
refiner,
translate and consolidate,
and never mentioned the archive review,
so the round ran under the 120000 ms round window by omission,
not by a decision.
In the run `zheermao20` the letter block's Qwen reply was cut at 243 s of a 360 s call deadline,
120000 ms after quorum,
and the block shipped without it.

What changed:
`archive-block-review-stage.ts` passes `grace.writerMs` to its gather,
`archive-block-review` joins `WRITER_STAGE_LABELS`,
and the `WRITER GRACE built in` line names it and says it joined on 2026-10-21.
The selection round after the review is a reading of text that exists,
one ballot in several,
and keeps the round window.

Not measured:
whether the extra 60 seconds converts the cut into a rewrite or into a cap-cut empty reply
(the same block in `zheermao19` ran its Qwen stream to the cap with no content),
and how often a review round reaches the window.
The cost is at most 60 seconds on a review round that has a straggler past 120 seconds,
bounded by the 360000 ms call deadline.

To veto:
remove `archive-block-review` from `WRITER_STAGE_LABELS` and pass `grace.roundMs` again at the gather.

## Decision 2026-10-22: a short bench closes one voice below every live seat, so the grace can cut the last one

Decided by the owner,
asked with two options after the read of the corpus pass `XingZ6017`
(audit ledger B809 and B816).

The evidence put beside the question:
26 of the 27 rounds that took 250 s or more waited on one `inception/mercury-2.5` stream that sent only comment lines,
and the grace never applied to them.
Under the rule of 2026-09-09 a quorum is sized on the reachable bench,
so a consolidation bench of 7 with 3 seats unreachable needed 4 voices,
which is every live seat,
and a repair checker or probe bench of 3 with Kimi-K3 unreachable needed 2 of 2.
The grace opens only once the quorum stands,
so it could not open before the one hung voice.
The first-content window of B809 ends such a stream at 90 s for Mercury,
and says nothing of a stream that is slow and working.

The options,
as put:

- "Keep current rule (Recommended)":
  a short bench still waits on every live seat.
  Pro:
  no verdict decided on fewer voices;
  quality first.
  Con:
  a stuck seat costs wall-clock.
- "Let the grace open":
  allow a short bench to close on a quorum below every live seat once the grace ends.
  Pro:
  faster rounds.
  Con:
  some verdicts decided with one voice fewer.

The owner chose "Let the grace open".
The recommendation was the other option,
so this is the owner's choice against the ranking offered.

What changed:

- `reachableQuorum` (`package/translation-repair/stage/src/stage-reachable-quorum.ts`) returns the bench quorum less one,
  marked short,
  where the reachable seats equal the bench quorum and that quorum is above the two-voice floor.
  Seven seats with three dark close on three of their four live seats.
- The round hears that reduced quorum,
  and then the grace runs for the last live seat.
  A verdict is decided with one voice fewer only when that seat does not answer inside the grace.
- No window changed.
  `STRAGGLER_GRACE_MS` stays 120000 ms and `WRITER_GRACE_MS` stays 180000 ms.
  What moved is whether the grace may start on a short bench.
- A refusal the router gives inside a round sizes the need again,
  so round 0,
  which is sized before the router names any seat,
  opens its grace on the quorum the refusals leave
  (`runGatherRound` in `stage-round.ts`).
- Artifact generation 20 reads a stored naturalness round under the rule it was gathered by,
  and a build of 20 refuses to resume a run directory stamped 19 or earlier.
  The cache versions of every stage that caches a gather moved (audit ledger B816).

What this does not change:
a bench of 3 with 1 unreachable keeps the floor of two voices and still waits on 2 of 2.
Whether that floor should give way on a bench of three was not asked.

How it fits the status of 2026-08-29:
that status says "No grace change is authorized before active runs finish and matched evidence exists".
This answer is the owner's authorization for this change,
given before matched evidence of its effect exists.
How often the reduced quorum decides a verdict,
and how much wall-clock it saves,
are not measured,
since no run has gathered under it.
The measurement that would price it is the share of rounds whose last live seat was cut after the grace,
read from the first pass log written under generation 20,
beside the run-to-run band that the 2026-08-26 addendum shows single runs cannot price.

Record:
`doc/decision/translation-repair-short-bench-share.md`,
addendum 2026-10-22,
and the audit ledger entry B816.
