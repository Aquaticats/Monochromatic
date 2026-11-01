# The heading aligner plan: the aligner cannot leave two headings unpaired

Part of [Two features were built and never wired, and what to do about the aligner](wire-the-heading-aligner.md),
which lists every file of this record.

## The aligner cannot leave two headings unpaired, and that breaks attempt five

Found 2026-08-13 by a source-level review,
and confirmed against both the code
 and the corpus.
This CORRECTS the verdict in [the index of this record](wire-the-heading-aligner.md),
which
 called `alignHeadings` a working aligner that nothing calls.
It is not working.

### The arithmetic

`alignHeadings` scores three transitions per cell,
in `align-sections-order.ts`:

```text
  paired     = scores[diagonal] + headingAffinity(source, target)
  sourceGap  = scores[above]    - GAP_PENALTY
  targetGap  = scores[left]     - GAP_PENALTY
```

`headingAffinity` returns a value in `[0, 1]` and never goes negative.
`GAP_PENALTY` is `0.35`.

So pairing two headings that share NOTHING scores `diagonal + 0`,
while leaving
 both of them unpaired scores `diagonal - 0.70`.
`Math.max` therefore prefers
 the unsupported pairing every single time,
no matter how little evidence exists.
There is no value of `GAP_PENALTY` above zero that changes this,
because the
 comparison is between `0` and `-2 * GAP_PENALTY`.

### Why this is fatal rather than cosmetic

`headingAffinity` scores only shared Latin tokens of three or more characters.
A Chinese heading and its CORRECT English translation usually share none,
so
 the affinity of a correct pair is legitimately `0.00`.

The algorithm therefore cannot distinguish "correct pair,
no shared tokens" from
 "wrong pair,
no shared tokens".
Every zero-evidence path through the table ties
 at the same score,
and which pairing emerges is decided by the traceback's tie
 order,
which tries source gap,
then target gap,
then pairing.
That order is a
 PRIOR about where omissions fall,
not a result computed from the headings.

### Confirmed by probing `alignHeadings` directly

An earlier version of this section cited the `#70` corpus sweep,
which found 0
 unpaired sections across 92 entries and 284 pairs,
as independent confirmation.
That was a CONFLATION and is withdrawn:
the sweep ran `alignDocumentSections`,
 the production path,
which never calls `alignHeadings` at all.
It confirms the
 proportional fallback pairs everything,
which was already known and is what
 `#71` is about.
It says nothing about the arithmetic.

The claim needs the actual function,
so here it is on the actual function.
The
 count that matters is MUTUAL gaps,
meaning a source and a target both left
 unpaired where they could have been paired together,
since a length difference
 forces surplus gaps no matter what the scores say:

```text
  equal length, zero affinity (3v3)            pairs 3   mutual gaps 0
  equal length, one strong anchor misplaced    pairs 3   mutual gaps 0
  unequal 5v3, zero affinity                   pairs 3   mutual gaps 0
  middle omission with anchors either side     pairs 3   mutual gaps 0
```

Never once.
Every gap it emits is the forced surplus of a length difference,
 and it never CHOOSES to withhold a pairing on evidence.
That is what the
 arithmetic predicts,
and `#71` requires the opposite.

The second row is the more damaging one.
A perfect anchor at affinity `1.00`
 sat at source 0 and target 2,
and the aligner ignored it and paired by position
 instead,
because reaching the anchor costs four reciprocal gaps at `-1.40`
 against a gain of `1.00`.
So a fixed scalar penalty does not merely fail to
 withhold pairs,
it discards the strongest evidence the affinity function can
 produce.

### What it does to attempt five

Attempt five adds a soft structural penalty strictly between `0` and
 `-2 * GAP_PENALTY`.
That is bounded by exactly the quantity at issue,
so it
 cannot make mutual gaps competitive with a zero-evidence pairing either.
It fixes `XingZ60` by encoding the prior "omissions occur at the END",
which is
 true of `XingZ60` and is not a general fact.
A middle omission still slides an
 anchor-bounded region.

The 90-identical,
2-changed,
0-regressed validation does not catch this,
because
 92 corpus documents mostly do not contain middle omissions,
and the invented
 middle-gap case was recorded as "undecidable rather than wrong".
It is worse
 than undecidable:
the aligner answers it confidently and the answer is a
 coin-flip fixed by traceback order.

This is the same failure as attempt three.
That one was defeated by a saturating
 term destroying the name signal and passed all 92 entries anyway.
The lesson
 repeats:
corpus-wide agreement measures how rare the hard case is,
not whether
 the algorithm handles it.

### The shape a correct fix needs

Do not combine affinity,
structure and gaps into one clamped scalar.
A fixed
 scalar penalty cannot work:
displacing an anchor by two positions requires four
 reciprocal gaps,
costing `1.40`,
which defeats even a perfect affinity of `1`.

Score lexicographically instead,
comparing in this order:

-   Trusted heading anchors,
    maximised.
    A trusted anchor is a high-affinity
    candidate UNIQUE on both its source row and its target column,
    so a repeated
    name never becomes a hard anchor.
-   Unpaired sections,
    minimised.
-   Remaining weak affinity,
    maximised.

That ordering gives the semantics the signal actually has:
`0` means no
 information rather than mismatch,
a sure name cannot be outvoted by an
 accumulation of gap penalties,
and weak evidence can position a gap that is
 already required without manufacturing new ones.

Ties must NOT be broken arbitrarily.
A pairing is safe only when it is the only
 possible partner for that source across all optimal paths,
the only possible
 partner for that target,
and neither item can be a gap on another optimal path.
Everything else is `ambiguous` and is withheld from the critics rather than
 guessed.
That is what `#71` asked for and what nothing currently does.

The return type has to change to carry this,
from parallel indices with `-1`
 sentinels to a discriminated union of `paired`,
`source-only` and `target-only`,
 so a caller cannot accidentally index a gap and pair it.

### Tests that would actually falsify the design

The existing suite cannot,
because it asserts on `alignHeadings` in isolation and
 its unequal-length case expects the gaps to fall at the END,
which encodes the
 prior rather than testing it.

-   Drive `alignDocumentSections` on a structural mismatch and assert the CRITIC
    INPUTS,
    not the aligner's internal steps.
-   Middle source omission,
    with anchors on both sides of a zero-evidence region:
    assert the region comes back ambiguous rather than shifted.
-   Middle target insertion,
    mirroring it.
-   Reciprocal gaps with equal counts,
    one source-only before an anchor and one
    target-only after:
    catches algorithms that minimise gaps before respecting
    anchors.
-   Long displacement,
    an anchor needing several reciprocal gaps:
    catches a fixed
    scalar penalty drowning sure evidence.
-   All-zero unequal sequences:
    assert NO confident pairs,
    rather than asserting
    gaps land at the end.
-   Repeated Latin token across several headings:
    assert it is not a hard anchor.
-   Affinity saturation,
    `alpha beta` against `alpha` and against `alpha beta`:
    if both score the same,
    affinity cannot support "high means surely".
-   Exact coverage:
    every input appears exactly once as paired or unpaired,
    and
    no unpaired item reaches a critic.

### Consequence for the ranking

Option B is still the right destination but it is BIGGER than this document
 estimated.
It is not an adapter plus one scoring term.
It is a new scoring
 model,
a changed return type,
an ambiguity path through
 `alignDocumentSections`,
and a test suite that exercises the production entry
 point.
The estimate of "one constant and a mismatch branch" was wrong.

Option C,
pair by index and leave the tail unpaired,
gains ground on this
 finding.
It is honest about having no evidence,
where B-as-designed was
 confidently wrong.
C remains a prior about tails,
but it is a STATED prior
 rather than one hidden in a traceback tie order.
