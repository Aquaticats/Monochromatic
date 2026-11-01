# Duplicate accepted issues are deduplicated at emission

Ratified 2026-08-13 by the user,
closing the question of whether duplicate accepted issues count against precision.

## The decision

The pipeline stops emitting two accepted issues that share a chunk,
a category
 and a span set.
Deduplication happens at EMISSION,
not at scoring.

## Why emission rather than scoring

Both would fix the reported number.
Only deduplicating at emission stops the
 waste:
a duplicate that reaches the editor spends repair budget twice on one
 defect and can produce two overlapping envelopes for it.
Discounting at scoring
 corrects the arithmetic and leaves the work duplicated.

## The measurement behind it

13.4% of accepted issues are exact-place duplicates:
58 across 40 groups over
 `pass13`'s 433 accepted issues.
It survives the concentration guard,
13.4%
 again after dropping the largest contributor,
with a per-entry spread of 5% to
 26%,
so it is a population property rather than one entry's behaviour.

INDEPENDENTLY CORROBORATED by the human grader,
who marked 7 of the 50 drawn
 items `Duplicate` while grading detection,
which is 14%.
Two different methods,
 one automatic and one by hand,
agreeing to within a point.

## What must not be lost

Attribution already distinguishes one critic repeating itself from several
 critics agreeing,
and that distinction must survive deduplication.
It lives
 beside the claim rather than inside it,
keyed by claim id,
so collapsing
 duplicate ISSUES does not collapse the proposer record.

## The care this needs

A shared span set is narrow but not proof of identity.
The implementation must
 not merge two genuinely distinct defects that happen to occupy the same span,
 which is why the key is chunk AND category AND span set rather than span alone.
