# Re-planning the translation-repair milestones

Written 2026-08-12 on the user's instruction to stop and re-plan rather than
start the next fix.
This is a PROPOSAL.
Nothing here is decided,
and it lives in `doc/planning/`
for that reason.

## Where the project actually stands

Three graded rounds of detection precision,
same instrument,
same bar of 0.9:

```text
round one     0.560 / 0.636 / 0.680
round two     0.740 / 0.787 / 0.800
round three   0.791 / 0.810 / 0.814
```

Repair is worse off than that number suggests,
and in a way precision does not
measure:
the repair sheet for round three was NOT graded,
because the repairs damage
text (`#67`),
and the instrument built to catch that damage does not see it
(`#66`).

Two of those premises have since been measured and one did not survive.
`#67` claimed the editor replaces far more text than the defect it was given,
sized at 21 of 50 edits beyond 1.35 times the quoted span.
Re-derived over the
40 DISTINCT regions the draw actually covers,
and against the widest span each
region SERVES rather than the single issue that drew it,
no region
over-replaces.
Envelopes merge overlapping and touching evidence,
so a region is
cut to cover every issue it serves,
and the old figure measured that merge.
What survives is that some replacements DROP source-supported content,
which
replacement width never evidenced.

So the honest summary is that DETECTION is close to its bar and REPAIR is not
fit to be measured yet.

## The finding that should drive the plan

The eight false positives in round three are not eight different problems.
Read with the grader's notes,
five of the eight are one problem:

```text
 4  "adds she"          the original uses she/her throughout
 7  "adds gamer"        the original does say so, in another sentence
41  "adds in heaven"    context shows they went to the afterlife
43  "omits confidant"   it IS translated, elsewhere in the passage
50  "adds She"          pronouns are she/her from context
```

Every one is a claim that some content is unsupported,
filed because the
licensing evidence sits OUTSIDE the window the critic was judging.
The remaining three are a different,
smaller class,
where the critic applies a
stricter literalism than the user's policy:

```text
20  "on that day"       enhances fluency
24  解脱 to "suicide"    no better way to say it in English, and it is not vague
26  总会 to "often"      often is an acceptable rendering
```

If the first class were eliminated,
precision on this sample would be about
39 of 42,
or 0.93,
which clears the bar.
That is the whole gap.

IMPORTANT CAVEAT,
or this plan repeats work already done:
tasks #40 and #41 already widened judged context and already render a source
context window for addition-class claims,
and are both complete.
These five still got through.
So the proposal is NOT "widen the window" again;
it is that a window is the
wrong shape of fix for this class,
because the licensing evidence can be
anywhere in the document,
including in a different section.

## Proposed shape, for discussion

### One: make addition and pronoun claims prove absence document-wide

An addition claim asserts a negative,
that the content appears nowhere in the
source.
A window can never establish that;
only a document-wide check can.
The cheapest version is deterministic rather than another model call:
before accepting an addition-class claim,
search the WHOLE source for the
content the claim says is missing,
and reject the claim when it is found.
Items 7 and 43 are exactly this shape,
and item 43 was already recognised as
"wrongly anchored" by both readers.

Pronoun claims (4, 50) are a narrower special case with a cheaper test still:
if the document uses a gendered pronoun for the subject anywhere,
a claim that
one was "added" is unfounded.

### Two: separate repair SAFETY from repair QUALITY, and gate on safety first

Today one sheet asks "does it fix the defect and break nothing nearby",
which
conflates two questions with very different costs.
Deleting a contributor's name is a different kind of failure from rendering
变故 as "misfortune",
and only the first is a reason to refuse to ship.

Proposed:
a deterministic preservation check at the apply gate,
independent of
any model.
Every source-supported sentence present before an edit must still be
represented after it.

BUILT 2026-08-13,
and calibrated rather than tuned by taste.
`checkPreservation`
in `package/module/translation-repair/src/preservation-check.ts` (moved on 2026-10-18 to
`package/translation-repair/page-pass/src/preservation-check.ts`) states the rule
as:
everything in the replaced text that no accepted issue quoted as defective
must still be present afterwards.
What an issue DID quote is licensed to vanish,
since changing it is the point of the edit.

Measured on the 50 drawn repairs of this very round,
34 of them graded:
it
rejects both damage regions and one wholesale deletion,
and rejects NONE of the
29 repairs graded sound.
The threshold sits in a measured gap,
not on a round
number,
since the worst sound repair loses 57% of its unlicensed tokens and the
deletion loses 92%.

It is NOT yet wired into `applyPatchOperations`.
Envelopes carry `issueIds` but
not the quoted defect text,
so the quotes have to be threaded to the apply site
before the check can gate anything.

WHAT IT CANNOT DO,
so nobody expects it to:
rewording is not deletion.
The edit
that turned "reminiscing" into "pleading" is real damage and passes,
because the
sentence is still there.
Item 37 therefore remains a safety failure this gate
does not catch,
and tuning the gate until it fired would cost sound repairs.

Sized honestly,
that is TWO regions rather than five items:
positions 2,
7,
11
and 15 are one Acheron edit drawn four times,
and 21 is a second.
The 50 drawn
items sit on 40 distinct regions in total,
so every per-item rate in this
document overstates by that factor.

Quality then becomes a separate,
later,
gradeable question.

### Three: fix the measuring instruments before measuring again

-   `#66` the probe reports `noneFound` on damage a reader sees immediately.
    Until that is understood,
    no repair change can be verified cheaply.
-   `#65` the pipeline emits one defect as several accepted issues,
    14 percent
    of the last sample.
    The gate now excludes duplicates,
    but the sheet still
    spends the grader's attention on them.
-   The agent pre-grader is systematically stricter than the user,
    in the same
    direction as the critics.
    Calibrating it is what would let a round be
    pre-screened before it costs human hours.

## What this plan deliberately does NOT propose

Lowering the 0.9 bar.
The bar is the user's and the gap is now explained rather
than mysterious,
which is the situation in which a bar should be kept.

Redrawing round three.
It is graded,
its verdict is recorded,
and its value is
the diagnosis above.

Changing the editor before `#66` is understood.
Without a working damage
detector the only way to tell whether an editor change helped is to hand-read
another sheet,
which is the cost this whole apparatus exists to avoid.

## Open questions for the user

-   Is milestone three still "detection precision at 0.9",
    or does it become
    two milestones,
    detection and repair-safety,
    gated separately?
-   Should the addition-class absence check be deterministic (search the source)
    or another adjudication pass?
    Deterministic is cheaper and explainable;
    a pass is more tolerant of paraphrase,
    which is exactly where item 7 lived.
-   Does repair QUALITY,
    as opposed to safety,
    belong in a milestone at all,
    or
    is it an open-ended improvement measured but never gated?
