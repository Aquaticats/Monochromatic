# When the archive's wording is put to a vote, it survives about one time in twenty

Measured 2026-08-18 over `~/translation-repair-runs-20260817/artifacts`,
the SIX NATURALLY ACCUMULATED entries,
50 slices,
read from `lanes.translate.result.sliceSelections`.
This pool is not hand picked,
so the figure is quotable.

## The measurement

```text
by decision   judged 39   sole-candidate 7   no-candidate-backed 4
by origin     fresh 37    incumbent 13

slices actually judged, meaning a real contest        39
of those, the incumbent won                            2
mean winning weight, fresh candidates               3.31
mean winning weight, incumbent                      0.42
```

ELEVEN OF THE THIRTEEN INCUMBENT RETENTIONS ARE DEFAULTS rather than wins.
`sole-candidate` means nothing was offered against it
and `no-candidate-backed` means what was offered had no support,
so in both the archive stands because no alternative arrived.
Only two slices in the pool show the archive beating a fresh translation on the ballots.

## Why this matters more than the replacement rate does

The delivery ledger reports 37 of 50 slices replaced,
0.740,
and that figure has been quoted repeatedly.
It mixes two different events:
slices where the archive lost a vote,
and slices where no vote happened.

READ ON CONTESTED SLICES ONLY the rate is 37 of 39,
0.949.

THAT IS THE NUMBER LATER COMPARISONS NEED.
Any question of the form "does the pipeline replace more often HERE than elsewhere"
is asked against a baseline of about 0.95,
not 0.74,
and a handful of slices all replacing is unremarkable against 0.95.
Six consecutive replacements happen 0.73 of the time at that rate.

## Two ways it could have been an artefact, both checked

IT IS NOT A LABELLING ARTEFACT.
`translate-judge.ts` puts the incumbent on the ballot ANONYMOUSLY,
and the module says why in as many words:
the existing translation deliberately does not travel as evidence,
because showing it twice would tell the judges which candidate is the incumbent.
Judges therefore cannot see which candidate is the archive's.

IT IS NOT A POSITION ARTEFACT EITHER.
Candidates reach judges in caller-fixed order rather than shuffled,
so a fixed slot for the incumbent would have been a real confound.
Counted over the 39 judged slices,
the incumbent's position in the slate varies:

```text
slot      0   1   2   3   4   5
slices   13  10   5   3   3   5
```

The archive's wording therefore competes blind,
from a position that changes,
and loses.

## What it does not say

IT IS NOT EVIDENCE THAT THE JUDGES ARE WRONG.
A fresh translation by a six-model ensemble beating a single archived rendering
is a plausible outcome,
and `#84` exists to measure judge quality on exactly this decision.
What this measures is the OUTCOME DISTRIBUTION,
not its correctness.

IT DOES SAY THE CONTEST IS NEARLY ONE-SIDED,
which bears on how the pipeline should be described.
A stage that preserves the incumbent one time in twenty
is closer to a translator than to a repairer,
whatever the lane is called,
and `#83` entered the incumbent into selection on the premise that it would sometimes win.

## What to check next, needing no new run

-   Whether the two genuine incumbent wins share anything:
    slice length,
    entry,
    which models backed them.
-   Whether the same ratio holds in the flagged pool once it settles,
    which is a different population and must be read separately.
-   What the two genuine wins look like beside the 37 losses,
    which is the only place a human could calibrate whether the judges are right.
