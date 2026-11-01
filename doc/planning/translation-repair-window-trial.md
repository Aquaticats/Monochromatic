# The window trial: what it measures, what it cost to make it measure that, and what it still cannot say

CLOSED 2026-08-16:
the trial is DROPPED,
and no fourth sham-context arm is bought.
The effect was smaller than the noise band it was measured against,
and the practical answer,
do not widen the judge's context,
is the same whether the null is real or the design is underpowered.
This one was decided on its merits rather than on its price:
a fourth arm would refine a MEASUREMENT whose resulting action does not change either way,
which the best-quality guideline does not buy.
Recorded in `doc/decision/translation-repair-four-answers.md`.
WHAT DOES NOT LAPSE WITH IT:
the negative control's finding that the per-slice preserve-or-replace decision
is about 19 percent unstable between identical runs.
That constrains how any single pass may be read,
and it outlives the question it was collected for.

Tracks `#108`,
which exists because of `#107`:
per-slice judging cannot tell a relocation from a fabrication,
and condemns the archive for both.

## The question

Some slices are flagged by a size screen as relocation candidates:
the translation of one passage appears to have been moved into another passage's slot by the human translator.
Judged in isolation,
the receiving slice carries English the original does not account for,
which reads as fabrication,
and the judges replace it.

The hypothesis is that showing the judges the neighbouring Chinese section lets them recognise a move,
and keep the archive's wording.

## The design that survived

Per slice:
produce ONE slate of candidates,
then put that same slate to the judges three times.
Two arms see no window;
one sees the neighbouring original.

The two narrow arms exist because judges are stochastic.
A narrow-to-wide difference means nothing until it beats the difference between two narrow runs over the same slate.

One slate is what `#109` split `runTranslateStage` to allow.
Before the split,
asking a slice twice resampled the candidates,
so two answers differed in the slate as well as in the evidence
and no reading could say which moved the verdict.

Rows are appended to a JSONL ledger as each arm completes,
keyed on a protocol digest built from the roster,
the corpus pin,
the head sha and the trial's own version,
so a rerun after any of them moved cannot pool two experiments.

## The measured draw

Measured against the pinned corpus,
not estimated:

-   93 entries,
    of which 29 contribute a flagged slice.
-   109 slices:
    19 `relocation-high`,
    21 `relocation-low`,
    24 `other-imbalance`,
    12 `target-only`,
    4 `untranslated`,
    29 `control-unflagged`.
-   327 judgings and 2616 model exchanges at roster width 6.
-   Zero slices have a blank neighbouring window.

## Six defects found before spending anything

Every one of these would have produced a report that read clean.
Listed with what each would have done to the number.

### Resumption never matched an arm it had bought

`trialKey` joined its fields on a NUL byte;
the runner joined the same fields on a space.
Both render identically in every reader checked,
including the Read tool used to verify it,
so `completedArms` returned keys the runner could never find.
Every resumed run would have re-bought every arm already on disk,
and had it also completed a partly bought slice it would have judged the remaining arms over a fresh slate.

Found by pasting the raw files to a reviewer that reads bytes.
Both callers now go through `trialKey`,
which encodes rather than joins.
The regression test runs the arms twice against a real ledger file;
it was shown to fail with the mismatch re-introduced.

### Resumption re-introduced the confound `#109` removed

A kill between arms left a partial triple,
and the next run finished it over a freshly produced slate.
Nothing recorded slate identity,
so the ledger held a triple whose arms disagreed about the candidates as well as the evidence,
undetectably.
A partly bought slice is now skipped rather than finished.

### A lost judge looked like a kept archive

`gatherStageVoices` retries only to a quorum of half the roster and then proceeds,
so an arm that lost three of six judges was written as an ordinary decision.
The wide arm sends the longest sheets under the same per-call deadline,
so degradation lands asymmetrically on the arm under test,
and a degraded round declines,
which keeps the archive:
the bias runs in exactly the direction the trial predicts.

Rows now carry the panel each arm decided on,
and the report drops any triple where an arm was short,
counting those separately from missing arms.
This also excludes the two short-circuit paths in `judgeTranslateSlate`,
`no-candidate` and `sole-candidate`,
which return `EMPTY_TALLY` and would otherwise have entered the tally as judge-decided keeps.

### The wide arm was always the last call

Arms were bought narrow,
narrow,
wide,
every time.
Anything drifting across a slice's three back-to-back calls landed entirely on the wide arm,
and the two narrow arms could not detect it,
because they occupied two positions while the wide arm sat at a third neither ever took.
Each slice now takes an order derived from its own identity,
deterministically,
so the wide arm sits first,
second and last in roughly equal numbers across the draw.

### Every control was an opening

At one control per entry the stride was the whole document,
so only position zero survived:
the control class would have been made entirely of the first unflagged slice of each entry,
which the draw's own comment says judges read differently.
The stride is now centred.

### The protocol digest guarded buying but not reading

The ledger is append-only and outlives any one experiment.
Resumption already skipped foreign rows;
the report read every one of them.
`reportWindowTrial` now takes the protocol and excludes the rest.

## Two smaller ones

A window of blank lines passed the emptiness check,
so a slice whose neighbour carried only whitespace would have run a wide arm differing from its narrow arm by nothing.
Now trimmed.

A slice that could not be tried raised out of the walk,
and since a refusal writes no ledger row,
every resumption would have redrawn it,
reached it and died at it again.
Refusals are now counted and stepped over,
with the run stopping if five come in a row,
since a slate is produced before any arm is judged.

## What is reported

Per class:

-   `pairedExcess`,
    the mean over read triples of the two narrow arms'
    replacement rate minus the wide arm's.
    Positive means the window reduced replacement.
    This is the primary number.
-   `entries`,
    the documents those triples came from,
    because slices are not independent observations.
-   The wide-against-narrow transition counts in both directions.
-   The narrow-against-narrow band,
    which is the negative control.
-   `incomplete` and `degraded` counts,
    so the reader can see how much of the class the analysis covers.

## The result

The run finished 2026-08-16T23:58Z after 25622 seconds,
buying all 327 arms of the planned draw with zero refusals.
Ledger:
`/home/user/temp/agent/window-trial-20260816-115844/window-trial/arms.jsonl`,
one throwaway runs directory,
never the shared one.
Every number here was recomputed from that ledger rather than read out of the run log.

### Widening the window did not move selection

Two readings,
because the pre-registered exclusion rule resolved to reporting both.
Under each,
the primary number is the wide arm's disagreement with a narrow arm,
read against the negative control,
which is the two identically configured narrow arms' disagreement with each other.

Strict,
every arm on a full panel,
37 slices over 21 entries:

-   Paired excess overall 0.000,
    and `relocation-high` alone -0.071 over 7 slices.
-   The wide arm decided differently from `narrow-a` on 2 slices.
-   The two narrow arms decided differently from each other on 2 slices.

Relaxed,
every complete triple,
109 slices over 29 entries:

-   Paired excess overall +0.005,
    and `relocation-high` alone exactly 0.000 over its full 19 slices.
-   The wide arm decided differently from `narrow-a` on 18 slices.
-   The two narrow arms decided differently from each other on 21 slices.

The effect is not merely inside the noise band.
In the larger reading it is SMALLER than the noise band:
two runs of the same arm disagree more often than the narrow and wide arms disagree.
The hypothesis that neighbouring Chinese lets judges recognise a relocation and keep the archive's wording
is not supported at any resolution this trial can reach.

### Which exclusion rule fired, and why

The pre-registered rule fixed in "Pre-registered:
how the short-panel exclusion will be decided" asks for
the short-panel rate per arm over every arm bought,
not per triple:

-   `narrow-a` 43 of 109,
    rate 0.394.
-   `narrow-b` 44 of 109,
    rate 0.404.
-   `wide` 48 of 109,
    rate 0.440.

The binomial spread of a difference between two such proportions at this sample size is 0.067,
and the wide arm exceeds `narrow-a` by 0.046 and `narrow-b` by 0.037.
Both differences sit inside the spread,
so degradation is symmetric rather than asymmetric,
and branch 3 fires:
read every complete triple,
and report both readings side by side.
That is what "The result" does.

Symmetric loss was the outcome the rule was written to detect,
and it is the one that arrived:
the wide arm sends the longest sheets under the same deadline,
so had its losses run ahead of the narrow arms',
every relaxed number would have credited lost voices to the window.

### The negative control found something the trial was not looking for

Two arms configured identically,
judging the same slate of candidates,
disagreed on 21 of 109 slices.
The per-slice preserve-or-replace decision is therefore about 19 percent unstable run to run,
and the trial's own primary comparison sits under that.

This is a measurement about the selection stage,
not about the window,
and it constrains everything downstream that reads a single run's per-slice decisions:
`#105`'s decline rate,
`#108`'s replacement rate,
and any future gate gated on one pass.
It also sets the resolution of any future arm of this kind:
an effect smaller than a fifth of slices cannot be seen in one run per arm.

### The ledger also answers `#105`'s decline question, which it was not built for

`#105` wanted a measured decline rate before deciding where an unfilled passage rests,
and planned a
synthetic bench for it.
The trial produced a better one at no extra cost:
327 real judgings,
on real
slices,
under the production roster.

-   Declines overall:
    56 of 327,
    a rate of 0.171.
    Of those,
    55 are `declined-indecision` and one is
    `declined-rejection`.
-   BY PANEL SIZE,
    which is where the rate actually lives:
    a full panel of six declines 12 times in
    192,
    a rate of 0.063.
    A panel that heard five declines 35 times in 122,
    a rate of 0.287.
    A panel
    that heard four declines 9 times in 13,
    a rate of 0.692.

The mechanism is in the source rather than inferred:
`MIN_SELECTION_WEIGHT` is 2 and a self-vote
counts a half (`package/module/translation-repair/src/candidate-select-model.ts` (moved on 2026-10-18 to
`package/translation-repair/house-rule/src/candidate-select-model.ts`)),
so a winner needs
agreement between two judges,
and every lost voice makes that threshold harder to reach.
Of the four
decline reasons visible in the surviving tail of the run log,
all four are
`winner short of the minimum vote weight` rather than a tie or an all-declined round.

So a decline is mostly a fact about the ROUND,
not about the passage.

### And it answers the retry question too, because every slate was judged three times

The two narrow arms are identically configured and judge the SAME candidate slate,
which is exactly
a retry:

-   Across both narrow arms,
    37 judgings declined.
    A second judging of the same slate decided in 21
    of them,
    a rate of 0.57.
-   Both identically configured judgings declined on 8 of 109 slices,
    a rate of 0.073.

Reading all three arms together,
37 slices had at least one declining judging and 32 of those had
another judging of the same slate decide,
but that figure includes the wide arm,
which saw different
evidence.
The honest retry number is the 0.57 above.

WHAT THIS SUPPORTS,
stated as evidence rather than as a decision,
since `#105`'s policy is the
user's:
retrying a declined judging once resolves the majority of declines,
and the passages that
decline under repeated identical judging are about 7 percent of slices rather than the 17 percent
the raw rate suggests.
Reducing voice loss would do more than any retry policy,
since a full panel
declines at a rate of 0.063.

### Short panels cost two thirds of the strict population

135 of 327 arms lost at least one judge,
a rate of 0.413,
with panels seating 6 and hearing 4,
5 or 6.
72 of 109 slices had at least one short-panelled arm,
which is what shrank the strict reading to 37.
The run log attributes most of it to `hf:zai-org/GLM-5.2` being abandoned 60000 ms after quorum,
with `hf:openai/gpt-oss-120b` returning empty content occasionally.
Neither the deadline nor the roster was changed mid-run,
so the rate is the honest one for this configuration.

## What this still cannot say

Stated plainly because a number without these caveats is worse than no number.

### The screen does not provide relocation ground truth

`classifyDisplacement` produces relocation CANDIDATES,
not verdicts.
A lower replacement rate on this population establishes
that neighbouring Chinese changed decisions on a size-screened mixture.
It does not establish that judges recognised relocations.
Narrowing that claim needs the candidates hand-labelled.

### The primary population is 19 slices

`relocation-high` is the only class the treatment can mechanically reach,
and the corpus yields 19 of them at this screen.
The earlier measured replacement rate is 0.83 to 0.94,
so most slices will replace in both arms and discordant pairs will be few.
A null on 19 slices is weak evidence,
and no amount of care in the instrument changes that.

### The wide prompt changes several things at once

Semantic neighbouring information,
prompt length,
evidence position and context-limit exposure all move together.
Separating the semantic part needs a fourth arm carrying length-matched IRRELEVANT Chinese:
real neighbour against sham neighbour tests semantic relevance,
sham against narrow tests the effect of extra context as such.
That is a design expansion,
not a fix,
so it is a question rather than a change.

### The denominator is rows that exist

The report derives its population from the ledger,
so a slice that failed before writing any row is absent rather than incomplete.
A frozen draw manifest,
validated before the client is built,
would fix that.
The refusal counter and the run log cover the operational need for now.

## Pre-registered: how the short-panel exclusion will be decided

Written before the numbers exist,
because a rule chosen after seeing them is not a rule.

The first arms bought show a short panel on 4 of 8,
which is far higher than the roughly 4.5 percent of calls the 360 second deadline was tuned to stop clipping.
At that rate an all-arms-full-panel requirement leaves almost nothing:
if each arm is independently full about half the time,
fewer than one triple in seven survives,
and `relocation-high` has 19 to begin with.

The losses so far land on narrow arms as well as the wide one,
which matters,
because the reason for excluding was never that a short panel is noisy.
It was that the wide arm sends the longest sheets under the same deadline,
so its degradation would be ASYMMETRIC,
and asymmetric in the direction the trial predicts.
Symmetric loss is noise;
asymmetric loss is bias.

So the exclusion will be decided from the run's own record,
by this rule,
fixed now:

1.  Compute the short-panel rate per arm over every arm bought,
    not per triple.
2.  If the wide arm's rate exceeds each narrow arm's by more than the binomial spread at that sample size,
    keep the strict exclusion.
    Degradation is asymmetric
    and reading those triples would credit lost voices to the window.
3.  If the three rates sit within that spread of each other,
    relax to reading every complete triple,
    and report BOTH readings side by side with the rates that justified the choice.

Nothing about this needs re-buying:
every row records the panel it decided on,
so the reading is revisable from the ledger alone.
The counterbalanced arm order also helps here,
since the wide arm no longer occupies the position most exposed to a filling queue.

## Open questions for the user

1.  Is a fourth sham-context arm worth roughly a third more quota,
    to separate semantic relevance from extra context as such?
2.  Should the relocation candidates be hand-labelled before the result is quoted as being about relocations,
    or is the narrower claim,
    that neighbouring Chinese moved decisions on a size-screened population,
    enough?
3.  `relocation-low` slices are bought and reported but the treatment cannot reach them:
    neighbouring CHINESE cannot say where missing ENGLISH went.
    Keep buying them as a within-trial control,
    or drop them and spend the quota on repeats of the high endpoints?

## Commits

Reachable on `translation-repair-rebased`:

-   `f18476ae7` compose the window trial probe
-   `84b21b3f3` live window check,
    build entry,
    mise task
-   `d950af5e8` four validity fixes (partial-triple resume,
    short panel,
    control stride,
    refusal wedge)
-   `45cb13e07` resumption key mismatch
-   `e3f830344` protocol filtering,
    relocation endpoint split,
    blank window
-   `6890cd2bf` counterbalanced arm order
-   `a969e0293` paired estimate as the primary number

## Next action

The run is finished and read;
see "The result".
What remains is a decision rather than a measurement,
and it is the user's:

The window treatment does not earn its place in the pipeline on this evidence,
so the choice is whether to drop the idea,
or to buy the fourth sham-context arm from "The wide prompt changes several things at once" and ask a sharper question.
Dropping it does not resolve `#107`,
which is why the trial existed:
per-slice judging still cannot tell a relocation from a fabrication,
and the remaining routes to that are hand-labelled relocation candidates or a whole-document pass over the pair.

The instability the negative control found is a separate matter and needs no decision to act on:
it belongs in `#105` and `#108` as a constraint on how their numbers may be read,
and it is recorded there.
