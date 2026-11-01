# What a two-lane entry costs, measured end to end on 2026-08-16

The measurement `#92` and `#114` both wanted and nothing had:
BOTH lanes over whole real documents,
under the production roster,
against `HARD_CAP_MINUTES` of 180.
Earlier figures came from bench
calls,
which price a slice but cannot say whether an ENTRY finishes.

Revised 2026-08-17 after per-slice timing was recovered from the aborted entries' caches.
The first
version of this document extrapolated from the two settled entries alone and was wrong about how
many entries the cap stops,
by a factor of about three.
What changed is recorded under "What the
first reading got wrong".

## The headline

TWO OF FOUR ENTRIES NEVER FINISHED.
They ran the full three hours and produced no artifact.

Both of them died in the REPAIR lane,
and neither was close to hopeless:
`aiyysk` had settled 73 of
its 80 slices when the cap cut it,
and `XingZ60` 55 of 83.

Extrapolated over the corpus,
a full pass costs roughly 119 to 128 hours and stops 4 to 6 of the 92
entries at the cap,
wasting 12 to 18 hours on entries that produce nothing.
RAISING THE CAP TO SEVEN
HOURS REMOVES EVERY ABORT and costs about 8 hours more in total,
because the entries that abort are
already being paid for in full.

The conclusion that survives unchanged is the budget one:
at 119 to 128 hours a full pass does not
fit the 72 hour soft budget,
whatever the cap is set to.

## What was run

`mise run //package/module/translation-repair:corpus-pass -- --only
Aniloviraw,zheermao101,aiyysk,XingZ60` (moved on 2026-10-18 to `//package/translation-repair/corpus-pass:corpus-pass`),
into a throwaway runs directory outside the repo,
at
`tip=e79bb338 pipeline=sha256-tree-v1:e327ca8b`,
which is the same pipeline hash the plan check ran
at.
Entries were picked by `page.md` blob size at corpus pin `a41fc607`,
over a distribution whose
minimum is 120 bytes,
first quartile 1557,
median 2497,
third quartile 5050 and maximum 41720,
totalling 396881 bytes across 92 complete pairs.

The four sit near the first quartile and the median,
and at the second largest entry and the
largest.

## What it reported, verbatim

```text
TALLY XingZ60     status=ERROR   ms=10800001 aborted=true error=Timeout: XingZ60 exceeded its 10800000ms deadline
TALLY zheermao101 status=SETTLED slices=15 repairStatus=repaired repairIssues=37 repairAccepted=22
                  repairResolved=20 repairFindings=133 repairChanged=8 translateStatus=complete
                  translateChanged=9 documentsDiffer=10 alignmentFindings=0
                  selection=pending-human-decision ms=3917494
TALLY Aniloviraw  status=SETTLED slices=5 repairStatus=repaired repairIssues=28 repairAccepted=12
                  repairResolved=11 repairFindings=72 repairChanged=5 translateStatus=complete
                  translateChanged=4 documentsDiffer=5 alignmentFindings=0
                  selection=pending-human-decision ms=2121755
TALLY aiyysk      status=ERROR   ms=10800001 aborted=true error=Timeout: aiyysk exceeded its 10800000ms deadline
DONE processed=2 of pending=4; artifacts=2/92 elapsed=27639265ms
```

Seven and a half hours of wall clock for two artifacts.
Six of those hours went to the two entries
that aborted.

## Per-slice cost, recovered from the caches

The first version of this document said per-slice cost was unmeasurable for this run,
because
`slice-cost-log.ts` landed after the launch.
That was wrong.
Every settled slice is written to the
per-entry cache as its own file,
so the file COUNT is slices completed and the file MTIMES are when
each one landed.
The two aborted entries kept their caches,
because the discard runs only on the
settled path.

Read off `slice-cache/XingZ60` and `slice-cache/aiyysk`:

-   `XingZ60`,
    83 slices needed,
    55 settled in 172.3 minutes:
    3.19 minutes per slice on average,
    median gap 2.8 minutes,
    fastest 8.9 seconds,
    slowest 7.3 minutes.
-   `aiyysk`,
    80 slices needed,
    73 settled in 177.7 minutes:
    2.47 minutes per slice.

Neither cache holds a single `translate.` prefixed file,
which places both deaths in the repair
lane.
That agrees with `document-lanes.ts:302` and `document-lanes.ts:321`:
the lanes run in
sequence,
repair first,
so an entry that never finishes repairing never starts translating.

The two settled entries give the split between the lanes,
from their log timestamps:

-   `Aniloviraw`:
    repair 22.95 minutes,
    translate 11.06 minutes,
    repair taking 67.5 percent.
-   `zheermao101`:
    repair 47.6 minutes,
    translate 16.0 minutes,
    repair taking 74.8 percent.

Applying about 71 percent to the projected repair lanes puts the whole-entry cost of `aiyysk` near
281 minutes and `XingZ60` near 385 minutes,
so 4.7 and 6.4 hours against a cap of 3.

## The rate, and what is safe to read off it

Four entries,
whole-entry minutes against source bytes:

-   `Aniloviraw`,
    1481 bytes:
    35.4 minutes measured,
    1434 ms per byte.
-   `zheermao101`,
    2323 bytes:
    65.3 minutes measured,
    1687 ms per byte.
-   `aiyysk`,
    21455 bytes:
    about 281 minutes projected,
    786 ms per byte.
-   `XingZ60`,
    41720 bytes:
    about 385 minutes projected,
    554 ms per byte.

PER-BYTE COST IS NOT CONSTANT.
It falls by roughly a factor of three from the smallest entry to the
largest,
so a rate taken from small entries badly over-prices large ones,
which is exactly the error
the first reading made.

A power law fits the four within 16 percent at worst:

```text
cost_minutes = 0.2585 * bytes^0.694
```

The exponent below 1 is the whole point:
cost grows SUBLINEARLY in document size.
A straight line
through the same points fits far worse,
over-predicting the smallest entry by 66 percent,
though it
agrees on every conclusion below.

Per-slice cost is not interchangeable either,
and moves the opposite way:
7.07,
4.35,
3.19 and 2.47
minutes per slice as entries get larger.
Slice count does not track size cleanly,
since
`zheermao101` needs 15 slices for 2323 bytes while `Aniloviraw` needs 5 for 1481,
so slicing follows
document structure rather than length.

THIS IS FOUR ENTRIES,
two of them measured to completion and two projected from their own repair
lanes at 91 and 66 percent done.
The projections carry the assumption that the remaining slices cost
what the completed ones did,
and that translate costs what it did on the settled pair.

## Where the cap falls

Under the power law,
against the real 92 entry distribution:

-   At the current 180 minute cap:
    6 entries abort,
    18 hours wasted,
    111 hours spent.
-   At 240 minutes:
    3 abort,
    12 hours wasted,
    115 hours spent.
-   At 300 minutes:
    1 aborts,
    5 hours wasted,
    117 hours spent.
-   At 420 minutes:
    none abort,
    nothing wasted,
    119 hours spent.

The linear fit puts the abort count at 4 rather than 6 and reaches zero by 420 minutes as well,
so
the two disagree on the count and agree on the shape.

A CAP SET AT EXACTLY 420 MINUTES WOULD HAVE NO HEADROOM.
`XingZ60` projects at 385 minutes,
which
leaves 35 minutes of margin inside a fit whose residuals reach 16 percent and whose largest input is
itself a projection from a lane that was 66 percent done.
If that entry runs 10 percent slow it
burns the full seven hours and still produces nothing,
which is the outcome this whole measurement
exists to avoid.
A cap chosen to clear the corpus should carry margin,
so eight hours rather than
seven,
or it should be set after the queue lands and the rate is re-measured.

## What this does not say

-   NOTHING ABOUT QUALITY.
    Both settled entries report `selection=pending-human-decision`,
    so this
    run priced the pipeline and decided nothing about the text.
-   NOTHING ABOUT A DIFFERENT ROSTER.
    This describes the configuration as launched,
    deliberately,
    including the `hf:zai-org/GLM-5.2` voice loss that `#105` addresses.
    It is the baseline any
    change to the roster or the deadline is measured against.
-   NOTHING ABOUT WHICH SLICES DISAGREED.
    Both settled entries report the lanes differing on most
    slices (`documentsDiffer=10` of 15,
    and 5 of 5),
    and the window trial's negative control
    measured the per-slice preserve-or-replace decision as about 19 percent unstable between
    identical runs.
    The disagreement rate is worth noting;
    which slices disagreed is not evidence
    from one pass.

## What `XingZ60` confirmed before it died

The alignment fix from `#71` and `#74` got its full-scale check anyway,
because preparation runs
before the lanes.
The run log carries:

```text
[XingZ60] [runDocumentLanes] [repairPreparedDocument] 12 chunk pairs, 83 slices, 4 alignment findings
```

Twelve chunk pairs is exactly what `doc/decision/translation-repair-unpairable-section.md` predicted
when it ratified the fix:
`XingZ60` keeps 12 of its 13 pairs and loses only the wrong one.
The
entry that used to have every section slid by two now pairs correctly and reports its 4 unpairable
sections as findings rather than aligning them proportionally into nonsense.

## What follows from it

The cap is not a safety margin a few outliers cross,
but neither is it the wall the first reading
described.
Three responses are available and they are not exclusive:

1.  RAISE THE CAP,
    which is now the cheapest of the three.
    Seven hours removes every abort for about
    8 more hours of total budget,
    and the entries it rescues are ones already being paid for in full
    and thrown away.
    The first version of this document dismissed this option on a projection of
    19.5 hours for `XingZ60` that direct measurement has since replaced with 6.4.
    Set it with
    margin,
    per "Where the cap falls".
2.  MAKE LARGE ENTRIES CHEAPER,
    which is where the per-slice numbers point.
    Both aborts died in the
    repair lane,
    which takes about 71 percent of an entry,
    so anything that cuts repair cost moves
    this number and anything that cuts translate cost barely does.
    Voice loss and retries are the
    `#105` lever.
3.  LET A LARGE ENTRY SPAN RUNS,
    which is weaker than it sounds.
    See "The cache does not span a
    rebuild".

None of the three is decided here.
The measurement exists so the choice is made against numbers.

## The cache does not span a rebuild

The slice cache is GENERATION-STAMPED by pipeline digest,
and a mismatch discards rather than
resumes:
`slice-cache-namespace.ts:463` loads the lane's slices only when `cached === generation`,
and `slice-cache-namespace.ts:470` discards them otherwise.

This is not hypothetical for the very run described here.
The caches still hold 55 and 73 repair
slices stamped `sha256-tree-v1:e327ca8b`,
and the current build digests to
`sha256-tree-v1:d8507690`,
because the sweep that followed the run rebuilt the pipeline.
Those 128
settled slices,
about 6 hours of paid work,
would be discarded on the next open.

So "an entry can span runs" holds only WITHIN ONE PIPELINE GENERATION.
On a repository that rebuilds
between passes,
which is the normal case here,
a resumed run starts from nothing.
Making option 3
real means deciding that some slices survive a pipeline change,
which is a question about what the
digest is protecting and is not answered by this measurement.

## What the first reading got wrong

Recorded rather than quietly edited,
because the wrong number was in circulation and may have been
read.

-   IT SAID 14 TO 16 ENTRIES CANNOT FINISH.
    The measured answer is 4 to 6.
-   IT SAID 42 TO 48 HOURS ARE THROWN AWAY,
    about a third of the budget.
    The measured answer is 12
    to 18 hours,
    closer to a seventh.
-   IT SAID `XingZ60` NEEDS 19.5 HOURS AND `aiyysk` 10.
    Their own caches say about 6.4 and 4.7.
-   IT SAID RAISING THE CAP "does not survive `XingZ60` at 19.5 hours in any case".
    At 6.4 hours a
    seven hour cap clears the entire corpus,
    which makes raising the cap the cheapest option rather
    than a rejected one.
-   IT SAID PER-SLICE COST WAS UNMEASURABLE for this run.
    It was sitting in the cache directories as
    file counts and mtimes.

The single cause of all of them:
the per-byte rate was taken from two entries of 1481 and 2323 bytes
and applied to entries of 21455 and 41720.
Cost is sublinear in size,
so that over-prices the large
end threefold.
The first version said "two points establish a range,
not a law" and then used it as
one.

The total cost of a pass,
119 to 128 hours,
is close to the first reading's 120 to 133 by
coincidence:
it under-counted small entries by about as much as it over-counted large ones.
