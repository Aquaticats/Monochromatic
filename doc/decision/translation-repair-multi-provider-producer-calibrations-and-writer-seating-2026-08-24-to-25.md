# Translation repair multi-provider: the producer calibrations, the writer seating and the defects the runs exposed

The twelve-round and forty-round producer calibrations of 2026-08-24,
the editor and refiner seats they settled,
the voice-loss and survivorship findings,
the budget-meter and router defects the calibration log exposed,
and the 2026-08-25 correction of the availability inference with the first duty-cycle estimate.

Part of [Translation repair: two providers, and the quota reader that was never wired](translation-repair-multi-provider.md),
which lists every file of this record.

## The twelve-round calibration settles seat one and nothing else

Ten models writing and ten judging the same twelve slices,
scored by the share of DISINTERESTED ballots each producer's work drew,
where disinterested means the judge held no stake in that candidate.
Finished in 3637 seconds.

    hf:Qwen/Qwen3.8-27B         29.7%   27 of 91    z  5.65   CI [20.3, 39.1]
    gemma-4-26b-a4b-it          17.9%   15 of 84    z  1.98   CI [ 9.7, 26.0]
    qwen3.8-max                 17.6%   15 of 85    z  1.93   CI [ 9.5, 25.8]
    hf:openai/gpt-oss-120b      11.8%   10 of 85    z  0.20   CI [ 4.9, 18.6]
    hf:moonshotai/Kimi-K3       10.6%    9 of 85    z -0.14   CI [ 4.0, 17.1]
    minimax-m3                   9.6%    9 of 94    z -0.46   CI [ 3.6, 15.5]
    NVIDIA-Nemotron-3-Super      4.1%    4 of 97    z -2.18   CI [ 0.2,  8.1]
    deepseek-v4-pro-0813         3.4%    3 of 88    z -2.29   CI [-0.4,  7.2]
    hf:zai-org/GLM-5.2           2.7%    2 of 74    z -2.29   CI [-1.0,  6.4]
    deepseek-v4-flash-0731       2.4%    2 of 84    z -2.54   CI [-0.9,  5.6]

The null is the POOLED share,
11.1 percent,
96 wins over 867 ballots:
what a producer drawing ballots at its fair rate would take.
Ten models are compared at once,
so the threshold that survives the multiplicity is Bonferroni's,
`|z| >= 2.81` rather than 1.96.

WHAT IS SETTLED:
`hf:Qwen/Qwen3.8-27B`,
at `z = 5.65`,
with a confidence interval that does not come near the null.
It clears the corrected threshold twice over and is the first writer's seat.

WHAT IS NOT SETTLED,
and this is the part the ranking's ordering hides.
`gemma-4-26b-a4b-it` and `qwen3.8-max` sit at 17.9 and 17.6 percent,
which reads like a second and a third place.
Neither clears the corrected threshold,
BOTH confidence intervals contain the null,
and the 0.3 points between them is far inside the noise of either.
They are tied with each other and unproven against chance.
Seating them in that order would be reading a ranking as a result.

The four at the bottom are consistently below the null,
`z` between -2.18 and -2.54,
and not one of them clears the corrected threshold either.
Four independent models all landing below the null by a similar margin is suggestive,
but no single one of them is individually established as worse.

WHAT IT WOULD TAKE.
For a producer at 17.9 percent to clear the null by three standard deviations
needs about 284 disinterested ballots,
against the 84 it has.
At the observed 7.6 ballots per producer per round that is about 37 rounds.

So the twelve-round pass is not a failed measurement;
it is a measurement whose denominator answers one question and not three.
The owner's standing instruction decides what follows from that:

> If measurement cannot settle a decision under the guideline of "max quality",
> you're not measuring right or enough.

A forty-round pass is therefore running rather than three seats being filled from this table.

## Which seats the forty-round pass actually decides

`RUN_ROSTER` is `ROSTER_MODEL_IDS`,
all TEN,
so the wide roles already fan out
across both providers:
critics,
panel,
judges,
and both stages of the translate lane.
The calibration's roster is therefore production's roster rather than a superset of it.

Only three roles are narrow,
and this is how they stand against the twelve-round table:

    editors    Kimi-K3 10.6% (at chance)   GLM-5.2 2.7% (z -2.29)   Qwen3.8-27B 29.7% (established)
    refiners   the same three
    checkers   Qwen3.8-27B 29.7%   Nemotron 4.1% (z -2.18)   gpt-oss-120b 11.8% (at chance)

THE SEAT IN QUESTION IS GLM-5.2's.
It holds an editor seat AND a refiner seat while scoring third from bottom as a writer,
and it is also the second worst model measured on reliability,
losing 2 voices of 21.
Those two facts point the same way,
which is rare enough to be worth saying;
the reliability finding and the quality finding are independent measurements
and they agree.

IT IS NOT YET GROUNDS TO MOVE IT.
`z = -2.29` does not clear the Bonferroni threshold of 2.81,
so on this evidence GLM-5.2 is suggestively weak rather than measurably weak.
The seat's own comment already calls the third writer provisional
and names this calibration as what settles it.

CHECKER SEATS ARE A DIFFERENT QUESTION and this table does not answer them.
The standing measures who WRITES well,
and a checker does not write;
Checker width was settled separately,
across 231 rounds.
Nothing here argues Nemotron should leave the checker roster
merely because it writes below the null.

## Two models lose voices and eight do not

Measured on the 2026-08-24 producer calibration,
ten models writing and ten judging the same slices,
over 46.5 minutes and 216 model streams.

Streams that completed against streams cut off after quorum:

-   `qwen3.8-max`,
    17 completed and 3 cut,
    the worst at about 15 percent.
-   `hf:zai-org/GLM-5.2`,
    19 completed and 2 cut,
    about 9.5 percent.
-   Every other model lost nothing at all:
    `deepseek-v4-flash-0731` 21,
    `deepseek-v4-pro-0813` 22,
    `gemma-4-26b-a4b-it` 21,
    `minimax-m3` 22,
    `hf:moonshotai/Kimi-K3` 21,
    `hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4` 23,
    `hf:openai/gpt-oss-120b` 24,
    `hf:Qwen/Qwen3.8-27B` 22.

THE LOSSES ARE A MODEL PROPERTY RATHER THAN A PROVIDER ONE,
which the split across providers settles:
`qwen3.8-max` is served only by Hyper and GLM-5.2 was called under its Synthetic spelling,
so both stacks lost voices while eight models on those same two stacks lost none.
That distinguishes this from the 2026-08-24 degradation window recorded in the handover,
where losses were spread and no single model explained them.

`qwen3.8-max` FAILS IN ONE RECOGNISABLE WAY.
All three of its cuts delivered 0 content characters
after 33000 to 36000 characters of reasoning,
each opening with the same words,
and each was abandoned at the 180-second post-quorum deadline.
It is not answering slowly;
it is not answering.

## The router was observed crossing providers

One stream in the same log carries the bare Hyper spelling `gpt-oss-120b`
where every other call to that model carries the Synthetic spelling `hf:openai/gpt-oss-120b`.
The two providers name the shared models differently,
so the wire label is direct evidence of the routing decision
rather than an inference from it.

## The budget layer was reading its meters 3.4 times too often

Same log:
158 quota reads and 158 credit reads in 46.5 minutes,
against a 60-second freshness window that should allow about 46 of each.

The cause is the shape this session already fixed once in `provider-router.ts`:
state checked before an `await` and written after it.
Every call arriving while a reading was in flight saw the old stamp and started its own.
Fixed in `88a092d32` by collapsing "fresh" and "in flight" into one idea:
a reading STARTED inside the window is the reading every caller uses,
so the stamp goes on before the await.
GFP-proven by restoring the late stamp,
which fails the new case.

ONE CONSEQUENCE IS NAMED RATHER THAN HIDDEN:
the first caller's signal governs the shared reading,
so its abort resolves the reading WET for every sharer.
That is already this file's answer for an unreadable meter,
and the router still recovers a real refusal through failover.

VERIFIED AT THE USER BOUNDARY on the forty-round pass,
which is a live run of the fixed build rather than a test double:
0.50 quota reads and 0.50 credit reads per minute over a twelve-minute window,
against 3.40 of each before the fix
and a ceiling of 1.00 that the sixty-second window permits.
A 6.8-fold reduction,
and under the ceiling rather than merely nearer it.

## The forty-round pass seats the writers, 2026-08-24

40 rounds,
ten models writing a candidate for the same slices
and every other model judging them:
2492 disinterested ballots against a pooled null of 13.48 percent.
That is past the ~284 ballots the twelve-round pass computed as the requirement
to separate the contenders it left tied.

The raw standing,
each rate over the rounds that model produced a candidate in:

    qwen3.8-max              27.0%   47 of 174 ballots, over 28 candidates
    hf:Qwen/Qwen3.8-27B      24.1%   61 of 253, over 37
    gemma-4-26b-a4b-it       18.3%   46 of 252, over 40
    hf:moonshotai/Kimi-K3    14.1%   36 of 255, over 40
    hf:openai/gpt-oss-120b   13.9%   35 of 252, over 40
    minimax-m3               12.3%   32 of 260, over 40
    deepseek-v4-pro-0813     10.7%   28 of 262, over 40
    deepseek-v4-flash-0731    8.4%   20 of 239, over 40
    hf:zai-org/GLM-5.2        8.2%   23 of 280, over 40
    NVIDIA-Nemotron-3-Super   3.0%    8 of 265, over 40

READ THE CANDIDATE COLUMN BEFORE THE RATE COLUMN.
Two models did not answer every round,
and a rate computed over the rounds a model survived
credits it for the rounds it skipped.
Charging each model zero for every round it missed:

    hf:Qwen/Qwen3.8-27B      22.3%   z  4.27   clears Bonferroni 2.81
    qwen3.8-max              18.9%   z  2.50   does not clear
    gemma-4-26b-a4b-it       18.3%   z  2.22   does not clear
    hf:moonshotai/Kimi-K3    14.1%
    hf:openai/gpt-oss-120b   13.9%
    minimax-m3               12.3%
    deepseek-v4-pro-0813     10.7%
    deepseek-v4-flash-0731    8.4%
    hf:zai-org/GLM-5.2        8.2%
    NVIDIA-Nemotron-3-Super   3.0%   z -4.99   the first established WORST

### qwen3.8-max's headline is survivorship, and it was proved rather than asserted

The suspicion is cheap and the proof is not,
so the log was parsed into its 40 round segments
and the OTHER models' median answer length taken as a proxy for slice size.

Rounds `qwen3.8-max` missed ran a median of 588 characters.
Rounds it answered ran 366.
The ratio is 1.61,
Mann-Whitney `z = +3.51`,
`p ≈ 0.0004`,
and the quartiles barely overlap:
512 to 687 against 259 to 462.

THE PARSE VALIDATED ITSELF.
It found 12 missed rounds and 28 answered,
and 28 is exactly the candidate count the standing reports independently.

So the model that tops the raw table is the model that skips the large slices,
and it also took 29 cuts,
more than any other model on the roster.
It is a good writer of easy passages and absent on hard ones.

### What was seated

`editorModelIds` and `refinerModelIds` both become:

-   `hf:moonshotai/Kimi-K3`,
    kept:
    40 of 40 candidates,
    at the null.
-   `hf:Qwen/Qwen3.8-27B`,
    kept:
    established in both passes,
    the only model clearing Bonferroni.
-   `gemma-4-26b-a4b-it`,
    seated:
    40 of 40 candidates,
    zero cuts,
    18.3 percent.

`hf:zai-org/GLM-5.2` leaves both seats.
It is below the pooled null in both passes,
and it was independently the second worst model on reliability.

`qwen3.8-max` is not seated and keeps every wide role.
A wide role loses one ballot when a voice is lost;
a three-seat stage facing quorum of two loses the stage.
Those are different bargains and the survivorship finding decides between them.

THE WRITER SEATS NOW CROSS PROVIDERS,
which none of them did before.
The argument runs BOTH WAYS and only one direction was written down at first.
Every previous writer seat was Synthetic-served,
so a Synthetic outage emptied the editor and refiner stages entirely
however well the router covered the wide roles.
Charm Hyper serves `gemma-4-26b-a4b-it`.

THE SYMMETRIC COST IS REAL AND WAS OBSERVED THE SAME DAY.
A run started 2026-08-24 15:36 UTC found Charm Hyper out of budget from its first second,
and the five models it refused were exactly the five Hyper-only ones:
`qwen3.8-max`,
`minimax-m3`,
`gemma-4-26b-a4b-it`,
`deepseek-v4-pro-0813` and `deepseek-v4-flash-0731`.
The five Synthetic-served models kept streaming and both meter endpoints kept reading.

So the seat that protects the editor stage from a Synthetic outage
also exposes it to a Hyper one,
which it was not exposed to before.
Under either outage the stage keeps two of three seats and clears quorum,
which is the property that matters and is why the swap still stands;
what changed is WHICH outage costs the seat,
not whether one can.
Before the swap,
all three writer seats fell together on a single Synthetic outage,
so the stage went from three-of-three exposure on one provider
to two-of-three on each.
That is the trade,
stated in full.

The wide roles were unaffected on the observed day:
they carry all ten,
so five going dark costs ballots rather than the stage.

### Neither provider can be restored on demand, and that settles the seat

Stated by the owner on 2026-08-24,
correcting an assumption made when the
outage was found:

>   I cannot reset charm hyper on demand.
>   I can only reset synthetic,
>   and only sometimes.

So Charm Hyper capacity returns on its own schedule and cannot be bought back,
and Synthetic capacity can be restored only sometimes.
BOTH PROVIDERS GO DRY AND NEITHER IS RELIABLY RECOVERABLE.

### Correction, 2026-08-25: half of that inference was wrong

The two sentences above the rule read more into the owner's words than they say,
and one half of it has since been falsified outright.

WHAT WAS STATED was that Charm Hyper cannot be RESET on demand.
WHAT WAS INFERRED was that its capacity therefore returns on its own schedule
and cannot be bought back.
Neither half follows from the quote,
and buying is not resetting.

On 2026-08-25 the owner bought 10,000 hypercredits
and `GET /v1/credits` read `10000` within the minute.
The balance had read `0` continuously through the whole of 2026-08-24,
before,
during and after a pass,
so there is no evidence it returns on its own schedule
and direct evidence it can be bought back.

Charm Hyper is a prepaid balance priced per token,
not a rate limit.
`package/translation-repair/README.md` now says so.

WHAT THIS DOES TO THE DECISION:
nothing,
and it strengthens the reasoning.
The count of editors each arrangement keeps when a provider goes dry does not rest on recoverability at all;
it rests on how many editors survive each provider going dry,
and those counts are unchanged.
The correction bears only on the quality-times-availability counter-argument,
where a Hyper outage now ends whenever the owner is willing to pay
rather than whenever the provider decides.
That raises Hyper's effective duty cycle,
which argues FOR the seat the section was already arguing for.

The claim that both providers go dry stands.
The claim that neither is reliably recoverable does not:
Synthetic recovers on its own schedule and resets only sometimes,
while Hyper recovers on purchase.

That is the fact the seating should be judged against,
and it argues FOR the
swap more strongly than the quality table did.
Count what each arrangement does when a provider goes dry:

    three Synthetic seats  Synthetic dry -> 0 editors, the stage is empty
                           Hyper dry     -> 3 editors, untouched
    two Synthetic, one Hyper
                           Synthetic dry -> 1 editor, a candidate is still produced
                           Hyper dry     -> 2 editors, quorum clear

The old arrangement had one outage that emptied the stage outright.
The new one has none.
An empty stage is a different kind of failure from a thin one:
a thin stage still produces a candidate to judge,
and the old shape's whole exposure sat on the provider the owner
can restore only sometimes.

THE COUNTER-ARGUMENT,
weighed and rejected.
A model that is dark contributes nothing,
whatever it scores when awake,
so the honest comparison is quality times availability.
The best Synthetic-served alternative for the third seat is
`hf:openai/gpt-oss-120b` at 13.9 percent against `gemma-4-26b-a4b-it` at 18.3,
which breaks even when Charm Hyper is up 76 percent of the time.
Hyper's duty cycle has not been measured,
so that ratio cannot be evaluated yet.
It does not decide the seat either way,
because it prices only the THIRD seat's marginal quality
while the count of editors each arrangement keeps prices the whole stage's survival,
and survival is the larger term.

THE OUTAGE TABLE STOPPED BEING AN ARGUMENT THE SAME AFTERNOON.
The 15:36 UTC pass reached the naturalness lane with Charm Hyper still dry
and logged both halves of the prediction within ten milliseconds
(log times,
UTC):

```text
17:01:28.240  chunk 2: nothing to edit, unchanged
17:01:28.250  refiner gemma-4-26b-a4b-it: NoProviderForModelError: no provider can take it
17:04:18.932  refinement from hf:Qwen/Qwen3.8-27B won
```

The newly seated Hyper model went dark in the seat it had just been given,
the stage ran on its two remaining seats,
and a refinement still won and shipped.
That is the two-of-three row of the table,
observed rather than reasoned about,
and it is the case the swap was defended on.

WHAT WOULD CHANGE THIS:
a measured Hyper duty cycle well under 76 percent,
at which point `hf:openai/gpt-oss-120b` takes the seat on quality as well.
Recording provider-dry intervals across runs is how that gets measured.

### A first estimate off the existing logs, and why it does not decide anything

Every log already timestamps `NoProviderForModelError`,
so a first estimate needed no instrumentation and no quota.
759 agent logs were scanned,
and THE DENOMINATOR WAS VALIDATED BEFORE THE RATE:
a log written before the second provider existed cannot show a refusal,
so only the 26 that name a Hyper-only model can count as evidence either way.
Of those,
24 are wet and 2 are dry,
and they fall in one clean block each.

    wet   2026-08-24T09:41:17Z .. 14:58:53Z   24 logs, no refusal in any
    ????  14:58:53Z .. 15:04:28Z              nothing ran, 5m35s wide
    dry   2026-08-24T15:04:28Z .. 17:53:00Z   2 logs plus a model-health probe

Read as fractions of the 8h11m43s observed:
wet 64.6 percent,
dry 34.3,
unknown 1.1.
That is BELOW the 76 percent break-even.

IT STILL DOES NOT MOVE THE SEAT,
for three reasons,
and the third is decisive:

-   ONE TRANSITION IS NOT A DUTY CYCLE.
    There is a single wet-to-dry edge in the
    whole record.
    A rate over one event is the mistake this document already
    caught once today,
    in `qwen3.8-max`'s standing.

-   THE DRY INTERVAL IS RIGHT-CENSORED.
    It was still dry at the last observation,
    so 2h48m is a floor and the fraction could move either way once it lifts.

-   THE DRYNESS IS PROBABLY SELF-INFLICTED.
    A 40-round calibration ran 11:22 to
    14:32 UTC and spent 937 streams,
    and a corpus pass ran to 14:58 UTC.
    Hyper was dry by
    15:04 UTC.
    So this measures how fast heavy verification exhausts a budget and how
    long it takes to refill,
    which is a fact about the day's usage rather than
    about the provider.
    Production traffic has a different shape entirely.

WHAT WOULD SETTLE IT is the same measurement over days rather than one afternoon,
which is why the availability recorder persists transitions rather than sampling.
The number worth having is not the mean but the LONGEST dry interval,
since that is what decides whether a seat can be relied on at all.

`gemma-4-26b-a4b-it` carries `readsImages: false`,
which costs nothing in these seats.
Pictures are read in `image-reading-stage.ts` over `RUN_READER_MODELS`,
a roster derived from the catalog rather than written by hand,
and `document-lanes.ts` records that the repair lane never asks what a picture says.
Its catalog `maxOutputLength` of 25_600 is likewise not a constraint here:
nothing in production reads that field,
and the model answered 40 rounds of production-sized slices without a cut.

### What this still does not decide

THE INSTRUMENT IS ONE STEP FROM THE SEAT IT WAS USED ON.
`producer-calibrate.ts` drives `runTranslateStage`,
so the thing measured is a model writing a slice from the SOURCE
while every other model votes on the result.
An editor and a refiner both write a slice and are both judged that way,
but from the archive text and a set of critic claims rather than from the source.

That gap is worth naming rather than glossing,
and the same honesty is what keeps this table off the checker seats.
It is nonetheless the nearest instrument that exists,
the seats it moved rested on NO measurement at all before this,
and the direction it points is not marginal:
the model leaving was below the pooled null in two independent passes
and second worst on reliability,
while the model arriving was above it with 40 candidates in 40 rounds.
An editor-role calibration would settle the seat outright.
It is not a variation on the existing runner:
the editor's input includes critic claims,
so the instrument has to buy a critic stage before it can ask an editor anything.

CHECKER SEATS.
`checkerModelIds` still seats `NVIDIA-Nemotron-3-Super`,
which this pass establishes as the worst WRITER on the roster at `z = -4.99`.
That is not evidence about checking.
The instrument measures who writes a candidate other models vote for;
a checker writes nothing.
Moving a checker seat needs a checker-side measurement,
and the checker-width measurement is the shape such a measurement takes.
Acting on this table would be reading the wrong instrument.
