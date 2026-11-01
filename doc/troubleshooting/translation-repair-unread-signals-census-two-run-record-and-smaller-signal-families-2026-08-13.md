# Unread signals: census two of a run's record and the smaller signal families, 2026-08-13

What a run records across the 56 settled entries,
and the smaller families checked against it:
the rejected-value family,
the refiner that goes silent on whole entries,
and dropped merge opinions that do not explain the duplicates.

Part of [Signals the pipeline emits and nobody reads](translation-repair-unread-signals.md),
which lists every file of this record.

## Census two: what a run records across the 56 settled entries

Taken 2026-08-13 from `node_modules/.monochromatic/translation-repair-runs`,
 collapsing each finding's parenthesised payload so kinds group.
Each pair is occurrences,
then how many entries carry the kind at least once.

```text
    509  55  refine-skip block/0          43  16  group-index-out-of-range
    461  54  refine-skipped               41  18  refine-selected
    405  54  editor-candidates            34   7  stage-quorum-unmet
    405  54  editor-envelope-select       32  15  refine-recheck-passed
    405  54  editor-chunk-select          30  21  ambiguous-quote
    253  51  refine-skip block/1          24   8  missing-verdict
    137  44  refine-skip block/2          12   6  quote-outside-blocks
    225  45  quote-not-found              10   9  refine-declined
    129  33  empty-quote                   7   3  unknown-regrade-severity
    129  36  refine-candidates             5   5  alignment structure-mismatch
     64  30  refine-skip block/3           4   3  alignment sections-merged
```

The rest of the tail runs to single figures:
`no-quotes`,
`unknown-severity`,
 `unknown-vote`,
`duplicate-check`,
`missing-check`,
`refine-rolled-back`,
 `non-translation dominance`,
and the per-quorum `non-translation votes stand`
 lines.

Most of it is ordinary bookkeeping.
Two families are not.

## The rejected-value family is small, and checked so nobody re-opens it

Every `unknown-` finding carries the value the model actually sent,
so the
 census can read them off rather than guess.
Over the same 56 entries there are
 nine in total:

```text
  4  unknown-regrade-severity ()
  2  unknown-regrade-severity (massive)
  1  unknown-regrade-severity (small)
  1  unknown-severity (minor,)
  1  unknown-vote (support)
```

Severity accepts `neutral`,
`minor`,
`major`,
`critical`.
So four are the model omitting the field,
three are it inventing a word outside
 the vocabulary,
and one is an invented vote.

The ninth is the interesting one.
`minor,` is a VALID severity rejected for a
 trailing comma,
which is the same shape as the Kimi-K3 channel-marker defect:
 a good answer lost to a formatting artefact rather than to disagreement.
It is one occurrence out of 4098 adjudicated issues,
so it is recorded rather
 than acted on.
Should this family grow,
punctuation trimming before the
 vocabulary check is the cheap half of the fix.

## The refiner goes silent on whole entries

Recorded in full in `doc/planning/naturalness-lane-reach.md`,
because it bears
 on a decision waiting there.
In summary:
`stage-quorum-unmet` is 34
 occurrences and every one is `refiner 0/1`,
which is the artifact stating the
 roster size itself.
The partition is exact,
29 entries heard from it always
 and 7 never,
so the cause is entry-determined.

ALREADY FIXED,
and the census predates the fix.
That population ran to
 2026-08-11,
and `eb21ffa6b` on 08-12 took the lane from one refiner to three.
Quorum is two now,
so a single lost voice cannot empty the stage,
and `pass13`
 shows every `refine-candidates` finding at `3/3 heard`.
The finding stands as
 a description of the old population and not of the current pipeline.

`quorumMet` itself is read where it matters.
`restoration-judge.ts` and
 `derivability-probe.ts` both mark a seed unjudged rather than accept a
 minority verdict.
The other eight callers of `gatherStageVoices` do not read
 it,
relying on their own downstream guards.

## Dropped merge opinions do not explain the duplicate issues

`group-index-out-of-range` is the adjudication panel naming a cluster number
 that does not exist;
`adjudicate-wire.ts` drops that merge opinion and records
 the finding.
A dropped merge opinion leaves claims unmerged,
which looked like
 a candidate mechanism for the duplicates in
 `doc/planning/duplicate-accepted-issues.md`.

Measured over the same 56 entries,
split by whether an entry carries any
 dropped merge opinion:

```text
  entries WITH a dropped merge opinion   16 entries  1503 accepted  36.8% duplicate
  entries WITHOUT                        40 entries  1147 accepted  35.1% duplicate
```

No effect,
and no monotone relation inside the affected group either:
the entry
 with the most dropped opinions has a lower duplicate rate than several
 carrying one.
The hypothesis is refuted.

The duplicate rate here is higher than the 21.5% recorded in the planning doc
 because the key is looser.
This measurement groups on an issue's first
 target-side span,
where the planning doc required every span to match.
They
 count one phenomenon at two strictnesses,
and neither contradicts the other.
