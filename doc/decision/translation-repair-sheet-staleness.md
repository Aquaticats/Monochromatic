# When a grading sheet stops measuring anything

Decided 2026-08-13.
The user asked whether two outstanding sheets were still
 relevant,
and accepted discarding both.

## What was decided

The repair sheet and the damage sheet of the round-three draw are retired as
 pending work.
Neither will be graded further,
and the tasks that named them are
 re-scoped onto a fresh draw rather than onto the user's attention.

The FILES stay where they are.
They are not deleted,
and this is deliberate:
 they live outside the repository in the gitignored runs directory,
so removing
 them is irreversible,
and the graded portion is the provenance of a landed
 threshold.
`LOSS_FRACTION_LIMIT` in `src/preservation-check.ts` was calibrated
 on those graded items.
Destroying the evidence behind a shipped constant to
 tidy a directory would be a bad trade.

## Why

A sheet is bound to the pipeline commit that produced the artifacts it draws
 from,
not merely to a corpus commit and a seed.
The draw manifest records
 `seed` and `corpusSha`,
and both were still valid:
the corpus pin
 `a41fc607ea5a70d8a7625cc67d5ed8c444f53379` has not moved.

What moved was the pipeline.
The 17 entries behind the 50 items settled at tips
 `9533b0ba8` and `351eb1a4d`,
on 2026-08-06 and 2026-08-07.
There have been 153
 commits to `package/translation-repair/*/src` since,
and among them are
 every change that alters what those items would now be:

-   the aligner that can refuse,
    replacing the one that slid a whole document
    when heading counts differed;
-   the preservation gate at the apply site;
-   merging accepted issues that name one defect in one place;
-   the verse rule and the computed line-structure signal;
-   the channel-marker fix,
    which changes which critics are heard at all.

The decisive one is the preservation gate,
because it was CALIBRATED ON THIS
 VERY SHEET.
It rejects the repairs at items 2,
7,
11,
15,
21 and 48.
Those
 edits no longer ship.
Grading them would produce a number about a pipeline that
 does not exist,
and the number would look exactly like a number about the one
 that does.

Two smaller confirmations point the same way.
The section-alignment finding recorded that the damages at
 items 5,
16 and 19 are the aligner defect rather than repair quality,
and
 deduplication now merges roughly one accepted issue in seven that the sheet
 lists as separate items.

## The rule this generalizes to

A draw is stale when any landed change would alter what the pipeline produces
 for the drawn items,
not when it is merely old.
Concretely,
before asking a
 human to grade a sheet,
check whether the artifacts it draws from carry the
 current `tip`,
and treat a difference as a question to answer rather than a
 detail to note.

The cheap version of that check is the one used here:
compare the artifacts'
 `tip` against `HEAD`,
then read the intervening commits to `src` for anything
 that touches detection,
adjudication,
editing or the apply gate.

## What this costs, stated plainly

The three tasks that named these sheets
 (the probe's false-positive sample,
its calibration and the prober disagreement) were recorded
 as blocked on the user.
They were not.
They need a fresh draw from entries
 settled under the current pipeline,
and until enough have settled there is
 nothing to draw.
That is a real delay,
and it is the honest one;
the
 alternative was a measurement that could not support the decision resting on
 it.
