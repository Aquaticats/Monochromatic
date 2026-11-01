# The heading aligner plan: the derivability probe and the block-count gap

Part of [Two features were built and never wired, and what to do about the aligner](wire-the-heading-aligner.md),
which lists every file of this record.

## Wiring the derivability probe: where it goes

The aligner is blocked on `#70`.
The derivability probe is not blocked on
 anything,
so its touch points were traced too.

`repair-benchmark.ts` already computes two recall figures side by side:

```text
  judgeRestored   verdict === 'restored'
  judgeLenient    verdict === 'restored' OR verdict === 'partial'
```

Strict counts only full restorations;
lenient forgives every partial.
The probe
 exists to sit between them,
forgiving a partial ONLY where the deleted
 sentence was not derivable from the Chinese source,
so the editor is not
 charged for failing to invent information the source never carried.

Three touch points,
all in code that already has the shape for it:

-   `runRestorationJudge` is already an injectable default parameter on the
    benchmark,
    so `runDerivabilityProbe` can arrive the same way rather than
    being reached for globally.
-   The dispatched record already carries `seedJudgments` keyed by seed id;
    derivability is keyed the same way and would sit beside it.
-   The count itself is one more filter next to the two above.

The important property is that this is ADDITIVE.
A third figure appears next to
 strict and lenient;
neither existing figure changes.
So wiring it does not
 invalidate any recall number taken so far,
it explains the gap between the two
 that are already reported.
That materially lowers the risk of acting on it,
 and it is why this half of `#74` can be decided independently of `#70`.

The cost is provider capacity:
the probe is an ensemble,
so it adds a fan-out
 per partial seed on every recall run.
Its own design already limits the
 exposure,
since an unjudged seed defaults to `derivable` and therefore excuses
 nothing.

### Sized, and at the current scale it is not worth wiring

The benefit was asserted before it was measured.
Measuring it closes the
 question the other way.

The probe's ONLY power is to excuse a partial restoration.
So its ceiling is
 the number of partials,
and the last recall run
 (`recall-scorecard.json`,
2026-08-10) has:

```text
  judged seeds     27
  restored         23
  PARTIAL           1

  strict rate    85.2%   (23/27)
  lenient rate   88.9%   (24/27)
```

The strict and lenient rates sit 3.7 points apart,
and that entire gap is ONE
 seed.
A third figure between them cannot exist:
with one partial the probe can
 only reproduce 85.2% or 88.9%,
never anything in between.

So the earlier claim here,
that recall figures are "pessimistic by an unmeasured
 amount",
is now measured.
The amount is at most one seed.
Wiring an ensemble
 fan-out to move a single seed is a poor return,
and this half of `#74` should
 be deferred rather than acted on.

WHAT WOULD CHANGE IT:
a larger recall run.
Nine entries and 27 seeds is a small
 population,
and partials are the rare class within it.
If a future run produces
 partials in double figures,
the probe starts earning its calls and this
 conclusion should be re-taken rather than inherited.

## The block-count gap re-measured, which `#71` required

`#71` asked for the corpus-wide block-count gap to be re-measured,
on the
 grounds that the figure was computed from the same suspect pairing and so
 measured alignment quality as much as translation coverage.
It matters because
 `doc/decision/translation-repair-output-goal.md` rests on it.

Measured against the current build over all 92 entries:

```text
  aligned chunk pairs          284
  differing in block count     103   (36%)

  the decision record cites     60 of 172   (35%)

  XingZ60, the only mispairing entry, contributes 12 of the 103
  excluding it                  91 of 271   (34%)
```

THE DECISION'S EVIDENCE SURVIVES.
The rate is 36% against the recorded 35%,
and
 removing the mispaired entry entirely leaves 34%.
The absolute counts differ
 only because the record measured 56 settled entries and this measures all 92.

TWO OF ITS THREE CITED EXTREMES ARE ARTIFACTS,
and one is not.
The record cites
 `XingZ60 76 against 5` and `XingZ60 62 against 1`,
both of which come from the
 entry whose pairing slides,
so those specific numbers describe the defect
 rather than the corpus.
`XIEPT2 24 against 1` comes from an entry that pairs
 CORRECTLY,
so it is genuine evidence of a barely-translated section.

That is the useful distinction:
the phenomenon the decision was taken about is
 real and survives independently of the aligner defect,
while two of the
 headline numbers illustrating it should be replaced with ones from correctly
 paired entries when that record is next touched.

`XIEPT2` alone supplies three:
24 against 1,
18 against 1,
and 18 against 2.
