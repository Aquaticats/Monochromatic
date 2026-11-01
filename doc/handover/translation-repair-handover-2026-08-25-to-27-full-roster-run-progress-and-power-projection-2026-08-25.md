# Translation repair handover snapshots: the full-roster calibration run in flight, 2026-08-25

How the forty-slice full-roster run behaved while it ran:
the launch and its do-not-rebuild rule,
wall time per slice and the revision that should not have happened,
the refiner lane firing,
and the 200 power projection that flipped on measured accrual.

Part of [Translation repair handover snapshots: 2026-08-25 to 2026-08-27](translation-repair-handover-2026-08-25-to-27.md),
which lists every file of this record.

## The run in flight

Launched 2026-08-25T01:30Z,
detached,
40 slices,
every seat filled by the whole ten-model roster.

```text
TRANSLATION_REPAIR_RUNS_DIR=~/temp/agent/editor-calibrate-fullroster-20260825 \
  mise run //package/module/translation-repair:editor-calibrate -- 40
```

Moved on 2026-10-18 from `//package/module/translation-repair:editor-calibrate`
to `//package/translation-repair/bench:editor-calibrate`.

Log at `~/temp/agent/editor-calibrate-fullroster-20260825.log`,
pid beside it in the `.pid` file.

ALL TEN SEATS ARE ANSWERING,
which is the whole reason this run exists.
The five Hyper-only ones wrote nothing last time.
`gemma-4-26b-a4b-it` in particular HOLDS an editor seat and had never been tested.

DO NOT REBUILD `dist/` WHILE IT RUNS.
The run stamps its cache with the runner's own dependency closure,
so a source change plus any restart invalidates every cached slice and re-buys the whole run.
Documentation is outside that closure and safe to edit.

## What a slice actually costs in wall time, and a revision that should not have happened

Measured over the first six slices:

```text
slice 1  lintong chunk 0        214 s   0 editor  0 refiner
slice 2  windward0032 chunk 14 1362 s   5 editor  0 refiner
slice 3  Huasheng chunk 16       503 s   2 editor  0 refiner
slice 4  Mio chunk 10           1083 s   2 editor  1 refiner
slice 5  lintong chunk 4         967 s   2 editor  1 refiner
slice 6  zheermao101 chunk 2      16 s   0 editor  0 refiner

real slices   4, mean  979 s
cheap slices  2, mean  115 s
real share   67 percent
```

Projected total 7.7 hours,
which is the figure first estimated from roster-round arithmetic.

A REVISION WAS MADE ON ONE SAMPLE AND WAS WRONG.
After slice 2 alone,
at 1362 s,
the projection was moved from 8 hours to 12
and the earlier number was declared superseded.
Slice 2 turned out to be the slowest of the four real slices measured,
which span 503 to 1362 s,
a 2.7-fold spread.
The mean is 979 s and the original estimate was right.

THIS IS EXACTLY WHAT `QNB` FORBIDS:
measure the run-to-run band before crediting a difference smaller than it.
The rule was quoted in the same session it was broken in.
One slice could not distinguish a real slowdown from the top of an ordinary spread,
and no projection should have moved until it could.

## The refiner lane fires, and the two empty slices were not a signal

Slices 1 through 3 each reported `0 refiner rounds (nothing eligible to rewrite)`,
which looked like the refiner standing was going to come back empty for the second run running.

MEASURED INSTEAD OF WAITED.
Across 74 real shipped slices from four archived runs,
mirroring the bounds in `refine-eligibility.ts`:

```text
paragraphs                 156
  markup or non-prose       41
  hard break                 2
  under 120 chars           78
  over 1200 chars            1
  ELIGIBLE                  34

slices carrying at least one eligible paragraph: 32 of 74 (43%)
```

Three empty slices in a row has probability near 19 percent.
Unremarkable.
Slice 4 then produced a refiner round,
which is the first this workstream has ever seen:
the earlier run's binary predated the fix that drives the lane at all.

## The `#200` power projection flips, on measured accrual (2026-08-25)

`#200` recorded that 40 slices "lands a hair under the line",
worst-case z 2.776 against a
Bonferroni threshold of 2.807 at ten seats.
That rested on a projected 63 percent yield.
Measured at 16 of 40 slices,
the yield is 75 percent:
12 of 16 slices contributed a judged round.

Judged-round accrual,
counted from `selectBestCandidate` votes rather than projected:

-   407 votes over 16 slices,
    which at roughly one ballot per judge is about 41 rounds.
-   2.54 rounds per slice,
    and 2.30 with the single richest slice dropped,
    so no one slice carries the rate.
-   The 14-slice run that settled nothing managed 2.07 per slice from 10 contributing slices.

Holding the prior effect size,
40 slices reaches about 30 contributing slices,
so z scales by the square root of 3 and the deflated best z goes from 1.76 to roughly 3.05,
clearing 2.807 by 0.24.

DO NOT ACT ON THIS EITHER,
for the same reason the original projection carried.
The effect size it holds was measured on FIVE models,
and the pooled preference rate roughly halves
at ten seats,
so the z it implies could move in either direction.
The margin is thin in both directions and the run measures the effect size directly.
Read the standing when it exits;
the projection is superseded either way.
