# Translation repair history 2026-08-19 to 2026-08-21: what the window did to the repair lane

The first entry settled under the window,
with resolved issues falling from 23 to 6,
the reading of the 17 lost resolutions and the prediction recorded before the re-run,
the correction that a missing region means the panel said no,
and the panel's own votes with the roster controlled.

Part of [Translation repair history: 2026-08-19 to 2026-08-21](translation-repair-history-2026-08-19-to-21.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## `saurikissa` settled under the window, and the repair lane got much quieter

First entry of the minimal validation,
settled 2026-08-20 at 01:51Z after 102 minutes.
The roster is identical on both sides,
11 critic stages and 65 critic voices heard,
so none
of what follows is a voice-loss artefact.

```text
saurikissa repair lane      baseline    new
critic claims raised              94     83
issues surviving adjudication     43     48
issues RESOLVED                   23      6
slices shipped                     9      3
```

```text
repair shipped slices   baseline  1,2,3,4,5,7,8,9,10
                        new       1,2,5
translate shipped       baseline  1,2,3,5,6,7,8,9
                        new       1,2,3,5,6,7,8,9,10
```

THE TRANSLATE LANE IS UNMOVED,
which is the control:
the window was wired into the repair
lane only,
and the translate lane shipped 8 slices before and 9 now.

### Gate B is NOT the clean pass it first reads as

The severed-sentence census returns 0 of 3 repair rows,
against 1 of 30 in the baseline pool.
That looks like the defect is fixed.
It is not evidence of that.

THE SEVERED SENTENCE WAS AT SLICE 7,
AND SLICE 7 IS NO LONGER EDITED.
The baseline shipped a
repair there;
this run does not.
So the census reads zero because the lane declined to touch
the slice,
not because it edited it without severing anything.
A gate that cannot tell those
apart has not cleared the defect.

WHICH IS NOT NOTHING,
and may be the point.
Slice 7 is unflagged but sits beside the flagged
run at 8 and 9,
and `#107` recorded that its shipped text carried three passages belonging to
neighbouring slices.
A lane shown its neighbours declining to edit exactly there is the
behaviour the window was built to produce.
But "declined to edit" and "edited safely" are
different claims and only the first is evidenced.

### The number that needs a decision: resolved fell 23 to 6

More issues now survive adjudication,
43 to 48,
and far fewer are resolved,
23 to 6,
with
slices shipped falling 9 to 3.
Two readings fit and this entry cannot separate them:

-   THE LANE STOPPED MAKING EDITS IT COULD NOT JUSTIFY.
    The editor sheet now forbids removing
    wording on the grounds that a neighbour ought to carry it,
    and much of what it used to
    "resolve" at relocation-adjacent slices was exactly that.
    Fewer and safer changes.
-   THE LANE LOST REAL REPAIR CAPACITY,
    failing to fix seventeen issues it previously fixed.

WHAT WOULD SEPARATE THEM:
read the 17 issues resolved in the baseline and not now,
and ask
whether their baseline repairs were improvements or damage.
That is a reading of two settled
artifacts and needs no further quota.
It is the next thing to do and it is not done.

DO NOT TUNE THE WINDOW ON THIS.
One entry,
and the two readings have opposite remedies.

## What the 17 lost resolutions were, and what the re-run should do to them

Owed since `saurikissa` settled under the window,
and done now,
before the re-run settles,
so the reading is a prediction rather than a rationalisation.

Zero quota:
two settled artifacts.
No corpus text:
categories,
severities,
slice indices and counts only.
Note that `quote-not-found` findings carry the NEEDLE,
which is corpus text,
so they can be counted but never quoted into a document.

### What was lost

```text
resolved issues            baseline 23   window 6

by category
  accuracy/addition               10 ->  0
  accuracy/omission                8 ->  4
  accuracy/non-translation         4 ->  1
  accuracy/mistranslation          2 ->  0
  style/register                   1 ->  0
  extension/interpretive-ambiguity 1 ->  0
  fluency/grammar                  1 ->  1

by severity
  critical  5 -> 1      major  9 -> 1      minor  9 -> 4

by slice
  1: 5->4   2: 3->1   3: 1->0   4: 5->0   5: 1->1
  7: 2->0   8: 2->0   9: 2->0  10: 2->0
```

THE WHOLE TAIL STOPPED BEING REPAIRED.
Slices 7 through 10 resolved eight issues between them in the baseline and none under the
window.

### The mechanism, localised

Claims were still raised at a similar rate,
94 against 83,
so the critics did not go quiet.
What changed is what happened to a claim after it was raised:

```text
                no-region      not-selected    shipped
TOTAL           16 -> 37        1 -> 4         26 -> 7

slice 8          2 -> 14        0 -> 0          2 -> 0
slice 4          4 ->  7        0 -> 0          5 -> 0
slice 3          0 ->  4        0 -> 1          1 -> 0
```

Slice 8 alone accounts for twelve of the twenty-one added `no-region` issues,
and its critics raised MORE claims under the window,
fourteen against eighteen.
More to say and nowhere to put it is the signature of a critic quoting the nearby blocks:
a quote copied from next door cannot be located in the passage under review,
so the claim is discarded whole and the issue gets no region.
That is what `afc7854b4` forbids.

### What the re-run should show, recorded in advance

The fixed build forbids quoting the nearby blocks while keeping the window itself.
Three outcomes,
each meaning something different:

```text
no-region stays near 37            the quote rule did not take; cause is NOT anchoring
no-region falls toward 16 AND
  additions return to about 10     the window buys nothing for additions; reconsider #107
no-region falls AND additions
  return only part of the way      the window works and the hole was anchoring
```

The third is what the fix predicts.
`accuracy/addition` is the class the window exists for:
wording that looks added here
because it belongs to a neighbouring passage.
Suppressing SOME addition claims is the window doing its job.
Suppressing all ten,
alongside a collapse in every other category,
is the anchoring failure
dragging them down with it.

ONE STOCHASTIC RUN PER ARM,
so read direction and magnitude,
never a single number.
A rerun that moves `no-region` from 37 to 30 has not shown much;
one that returns it near 16 has.

## CORRECTION: `no-region` means the panel said no, not that a quote failed to anchor

The fixed-build re-run settled on 2026-08-20 and refuted the mechanism recorded above.

THE EQUIVALENCE,
over every settled artifact carrying issues:

```text
issue rows examined                     574
no-region rows                          251
accepted rows                           323
accepted AND no-region                    0
not accepted AND carrying a region        0
```

`no-region` is exactly `status !== 'accepted'`,
with no exception in either direction.
It is not a downstream consequence of anything.
An issue the adjudication panel does not accept has no repair region
because nothing should repair it.

SO THE CHAIN RECORDED EARLIER IS WRONG IN ITS MIDDLE.
It read:
critic quotes a nearby block,
the quote cannot anchor,
`quote-not-found` doubles,
`no-region` more than doubles,
nothing is left to edit.
The first two steps are real and the quote fix addressed them.
The step from `quote-not-found` to `no-region` does not exist.

WHAT ACTUALLY MOVED,
and it moves the same way on both entries:

```text
panel acceptance     baseline   window   fixed
  saurikissa            63%       35%      28%
  lintong               79%       70%      57%
```

The window makes the adjudication panel accept fewer claims,
which is what the window is FOR:
it tells the panel that wording explained by a neighbouring passage is not a defect.

THE QUOTE FIX DID NOT RESTORE ACCEPTANCE,
it lowered it further.
That is coherent rather than surprising.
With the critic no longer quoting next door,
its claims are better formed and survive
to be judged,
and the panel then rejects them on the merits
instead of their dying earlier and invisibly at anchoring.

`quote-not-found` did respond to the fix,
at `saurikissa` 16 down to 7,
below the baseline's 9.
It is a real but much smaller effect than the acceptance change,
and it was never the thing driving the collapse.

### What this leaves open, stated plainly

Whether the rejections are RIGHT.
Acceptance falling from 63% to 28% is the window working as designed
or the window suppressing real defects,
and no count distinguishes those.
It is the same shape of question as `doc/planning/which-lane-ships.md`:
a judgement about meaning,
which telemetry cannot settle.

WHAT WOULD SETTLE IT:
reading a sample of claims the baseline ACCEPTED and the
windowed run REJECTED,
and asking whether the neighbouring passage really does
explain them.
That is a reading of settled artifacts and needs no quota.

### The panel's own votes, with the roster controlled

The acceptance change is not a threshold artifact.
The individual panellists vote differently:

```text
saurikissa panel votes    supported   unsupported   supported share
  baseline                      402           124              76%
  fixed                         230           212              52%
```

ROSTER CONTROLLED FIRST,
because a rate read over a shrunken roster says nothing:

```text
                    critic stages   voices heard   min per stage   claims   claims per voice
  baseline                     11             65               5       94               1.45
  window                       11             65               5       83               1.28
  fixed                        11             64               5       79               1.23
```

Sixty-five,
sixty-five and sixty-four voices,
never fewer than five at any stage.
The critics raise slightly fewer claims,
and the panel supports far fewer of them.
The second effect is much the larger.

So the window acts where it was aimed:
on the judgement of whether a claim is supported,
made by panellists who can now see the neighbouring passage.
