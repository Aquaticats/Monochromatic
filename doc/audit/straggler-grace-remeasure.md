# What widening the straggler grace actually did, measured 2026-08-17

The re-measure `doc/decision/translation-repair-straggler-grace.md` called for.
Same two entries,
same roster,
same corpus pin,
one constant changed:
`STRAGGLER_GRACE_MS` from 60000 to 180000.

## The headline

VOICE LOSS FELL BY ABOUT THREE AND A HALF TIMES,
and it fell for the reason the fix predicted:
the
window stopped cutting voices that were merely slow.

```text
                 now              before             change
Aniloviraw    1 of 18  0.056    5 of 22  0.227      -0.172
zheermao101   2 of 59  0.034    6 of 58  0.103      -0.070
both          3 of 77  0.039   11 of 80  0.138      -0.099
```

Counted as PER-STAGE VOICE COMPLETENESS,
the definition the decision fixed in advance:
a stage
counts once when it heard fewer voices than it seated.
Loss lines and judgings are different
denominators and are not comparable to these.

## Read it on the at-risk population instead, which is a stronger result

THE DENOMINATOR ABOVE IS NOT ONE POPULATION.
Those 77 stages are four kinds seating two different
roster sizes,
and only the six-voice kinds were ever exposed to this window:

```text
                                   now                before
critic  (seats 6)             2 of 20  0.100      4 of 20  0.200
panel   (seats 6)             1 of 19  0.053      7 of 18  0.389
checker (seats 3)             0 of 19  0.000      0 of 21  0.000
probe   (seats 3)             0 of 19  0.000      0 of 21  0.000

six-voice stages only         3 of 39  0.077     11 of 38  0.289
all stages                    3 of 77  0.039     11 of 80  0.138
```

THE THREE-VOICE STAGES NEVER LOST A VOICE,
in either run,
0 of 38 and 0 of 42.
They cannot show this
effect and they dilute both rates,
so the all-stages figure understates the change on the population
that was actually at risk.
Both readings are kept because the decision fixed the all-stages
definition in advance and moving the goalposts after seeing the result is exactly what invalidates a
comparison.

READING IT THIS WAY ALSO ANSWERS THE OBJECTION A CAREFUL READER WILL RAISE about the table above,
that the per-entry denominators moved (22 to 18,
58 to 59) so the rates are not measured over the
same thing.
On the six-voice population they barely move at all,
and `Aniloviraw`'s does not move:

```text
                     now             before
Aniloviraw       1 of 10  0.100    5 of 10  0.500
zheermao101      2 of 29  0.069    6 of 28  0.214
```

An identical denominator on one entry and a difference of one on the other.
The change is in the
numerator.

## What the window still cuts

ONE ABANDONMENT SURVIVED,
at 180000 ms,
on `hf:zai-org/GLM-5.2`.
So the tail the decision warned
about is real:
the 88.6 second maximum behind the new window came from bench slices of 94 to 497
characters,
and a slower call exists outside that sample.

Against 91 abandonments in the run this replaces,
one is a different situation rather than a smaller
version of the same one.

THE OTHER TWO SURVIVING LOSSES ARE `schema-mismatch`,
named from `#75`'s sub-kind diagnostics in the
same log rather than left as an open question.
Six such lines appear,
on
`hf:zai-org/GLM-4.7-Flash` (five) and `hf:openai/gpt-oss-120b` (one);
loss LINES count retries,
so
six lines land on two stages.
This is a reply that did not satisfy the wire contract,
which no
timing window has ever had an opinion about,
and it was present at the same order under the old
window.
Nothing here regressed;
the window simply stopped hiding it behind ninety-one abandonments.

## What it cost

```text
                 now ms      before ms    change
Aniloviraw      2273456      2121755      +7.1%
zheermao101     5039467      3917494     +28.6%
both            7312938      6039249     +21.1%
```

MORE THAN THE ESTIMATE THE DECISION CARRIED,
which said under half a percent.
That estimate was
wrong,
and it is left standing in the decision with this correction beside it.

It was incomplete rather than backwards.
The same entries did MORE WORK:

```text
                issues      accepted     resolved
zheermao101   50 vs 37     29 vs 22     26 vs 20
Aniloviraw    27 vs 28     12 vs 12     11 vs 11
```

`zheermao101` found a third more issues and resolved a third more of them.
Panels that hear six
voices find more than panels that hear five,
and finding more costs more.
The extra time bought
extra work rather than extra waiting.

THE SPREAD IS THE WARNING:
+7.1 percent on one entry and +28.6 on the other,
from one run each.
No
unchanged-build repeat exists,
so the run-to-run band is unmeasured and a 21 percent aggregate
cannot be separated from ordinary provider variance.
Arithmetic also attributes only minutes of
`zheermao101`'s 18.7 to the window itself,
since only 6 of its 58 stages were being cut and each
would wait tens of seconds longer.
The rest is the wider judging that surviving voices produce.

## The telemetry's first production emission

`slice-cost-log.ts` and `slice-cost-read.ts` ran in production for the first time.
40 rows,
0
refused,
read back through the shipped `readSliceCosts` rather than a throwaway parser,
so the pair
is verified across the artifact-consumer boundary rather than only in tests.

Every row carried `exit=computed`,
which is correct for a cold run:
nothing was cached,
so no slice
could resume,
and no slice was left unfilled or found untranslatable.

## What it says about cost against slice size, which is less than hoped

```text
ms = 102673 + 774.0 * sourceChars      n = 40,  R2 = 0.076
source chars: min 31, p50 77, max 189
elapsed ms:   p50 152107, p90 342819, max 600675
```

THE FIT EXPLAINS ALMOST NOTHING.
An R-squared of 0.076 means slice size accounts for about seven
percent of the variance in what a slice cost,
so neither the intercept nor the slope should be
quoted as a cost model,
and the apparent 103 second fixed cost is an artifact of fitting a line
through a cloud.

What is safe to read:
OVER THE RANGE THIS RUN SAMPLED,
31 to 189 source characters,
a slice's cost
does not track its size.
Something else dominates,
and voice latency is the obvious candidate given
a p50 of 2.5 minutes against a p90 of 5.7.

WHAT THIS DOES NOT SETTLE is `#92`'s question,
because the range is far too narrow.
Corpus slices
run to 10959 characters,
roughly 58 times the largest sampled here.
A run over entries with large
slices is what would answer it,
and until then the honest statement is that size does not explain
cost among small slices.

## What the retry will cost, priced from these same artifacts

`#105`'s retry landed after this run,
so it did NOT run here.
Its price is still readable from what
did,
because a retry is bought exactly when a slate declines,
and both runs record every decision:

```text
                        declines / slices      all declines were
new window   grace      8 of 40   0.200        declined-indecision
old window   cost run   4 of 40   0.100        declined-indecision
```

EVERY DECLINE IN BOTH RUNS WAS `declined-indecision`,
which is on the retry list,
and `no-candidate`
never occurred at all.
So the exclusion written into `RETRIED_DECLINES`,
that a slate nobody proposed
anything for is not worth judging twice,
protects nothing on this evidence.
It stays because it is
right in principle and costs nothing,
but it is not what keeps the retry cheap.

THE RETRY IS THEREFORE BOUGHT ON ROUGHLY ONE SLICE IN FIVE TO ONE IN TEN,
not on the 0.063 a
full-panel rate would predict.
What one extra judging round costs against a whole slice is NOT
measured:
`SLICE-COST` times the slice,
not its halves,
so the share judging takes of it is unknown.
That is the measurement to add if the retry ever has to be justified on cost.

DECLINES DOUBLED IN THE RUN WHERE VOICE LOSS FELL,
which is the opposite of what the window trial's
own numbers predict.
That trial measured decline at 0.063 on a full panel against 0.287 on a panel of
five,
so recovering voices should have LOWERED the rate.
Two reasons not to read anything into
either direction:

-   THE COUNTS ARE TINY.
    Fisher's exact test on 8 of 40 against 4 of 40 gives a two-sided p of 0.348.
-   PER-SLICE DECISIONS ARE ABOUT 19 PERCENT UNSTABLE between identical runs,
    measured by the window
    trial's negative control,
    so a swing of four slices is inside the noise by construction.

What survives is the planning number:
budget the retry at one slice in five,
not one in sixteen.

## Every comparison from here is two changes deep

THIS RUN IS A CLEAN ONE-VARIABLE COMPARISON and it is the last one that will be.
It changed
`STRAGGLER_GRACE_MS` alone;
`#105`'s retry landed afterwards.
Any future run measured against the
2026-08-16 baselines carries BOTH changes,
and no arithmetic separates them after the fact.

TWO CONSEQUENCES,
both worth acting on rather than remembering:

-   THE BAND REPEAT STILL WORKS at `HEAD`.
    It measures the spread between two identical builds,
    which
    needs no baseline at all.
-   ATTRIBUTING THE 21 PERCENT TO THE WINDOW IS NO LONGER REACHABLE from `HEAD`.
    It would need a
    build with the retry reverted,
    which is a worse use of an hour than measuring the band.

NAME THE PIPELINE DIGEST BESIDE ANY TIMING RECORDED FROM HERE ON.
Every run prints it on its own
`START` line,
it is taken over built output so it moves when either change does,
and it is the only
thing in the record that says which of these two configurations a number came from.

## What follows

-   THE WINDOW STAYS.
    It did what it was changed to do,
    and the residual is one call rather than a
    class of them.
-   THE COST NEEDS A BAND BEFORE IT NEEDS A DECISION.
    One unchanged-build repeat of one entry would
    say whether 21 percent is signal,
    and costs about half an hour.
-   `#92` NEEDS SLICES THAT DIFFER IN SIZE,
    not more slices that do not.
-   THE RETRY IS PRICED BUT NOT MEASURED.
    One slice in five buys a second judging round;
    what that
    round costs against a whole slice needs `SLICE-COST` to time the halves separately.
