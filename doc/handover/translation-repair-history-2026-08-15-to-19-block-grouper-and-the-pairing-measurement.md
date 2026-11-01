# Translation repair history 2026-08-15 to 2026-08-19: the block grouper and the measurement that blocked it

The contiguity guard and the source-first grouper,
both added inert,
the reflow defect two reviews found,
and the corpus measurement showing that a source-only step is not evidence of an untranslated passage.

Part of [Translation repair history: 2026-08-15 to 2026-08-19](translation-repair-history-2026-08-15-to-19.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The block grouper, and the measurement that blocked landing it

### Landing four has its guard and its grouper, and is not wired

Two commits,
both inert:
`40d335504` adds the check the landing needs before its
producer changes,
and `70f46b590` adds the grouper beside the existing one.
The
slicing every run uses is untouched,
so this can be proven before it decides a
corpus.

THE GUARD FIRST,
because it is the one correction on the landing's list with
nothing behind it.
A content span's offsets come from the first and last node of
its run and its text is sliced from those offsets,
so a block lying between two
members of the run but missing from it is INSIDE the range,
agrees with the
document byte for byte,
and passes every check there was.
Assembly then writes
over the range,
replacing a block no lane ever read.
`assertSpanContiguity` runs
at preparation,
where the document's whole node sequence is in hand,
and checks
by identity rather than by count:
a slice carrying one block from outside its
range and one fewer from inside would count correctly and describe two different
passages.
Every slice produced today passes it,
which is expected,
since
consecutive grouping cannot skip a block.
A run built by FILTERING can,
and that
is what the new grouper would have done naively.

THE GROUPER,
in `group-source-first.ts`.
A source run with no counterpart
becomes its own unit carrying the BOUNDARY its translation belongs at,
rather
than being folded into a neighbour that already covers text and has nowhere to
put a rendering.
A paired unit never spans such a gap.
Target intervals are
taken as a slice of the whole target sequence between the first and last
supported index,
so a target-only block inside the interval belongs to the unit
rather than falling out of its run;
a target-only run that pairs with nothing
joins the unit before it,
or the one after it when there is none,
rather than
becoming a block no slice covers and nobody reviews.

SPLIT FROM THE ALIGNING on purpose,
which is also why the tests read the way
they do.
`groupAlignedSteps` takes the steps;
`groupSourceFirst` is the wrapper
that computes them.
Which block pairs with which is the aligner's judgement from
similarity,
and the first draft of these tests asserted an anchor's boundary
after feeding Chinese and English through the real aligner:
it failed,
because
the aligner had paired the blocks differently and the test was measuring that
rather than the grouping.
The structural cases now write their steps out.

WHAT THE WIRING STILL OWES,
which is the rest of landing four:

-   `subdivideChunkPair` calls `groupSourceFirst` instead of `groupNodesAligned`,
    turns a paired unit into a `ChunkPair` as it does now,
    and turns an anchored
    unit into a pair whose target side is `makeInsertionChunk` at the boundary.
    The boundary is a target NODE INDEX;
    the offset is that node's start,
    or the
    section's end when the index is the block count.
-   `mergeOneSidedRuns` and the unreachable proportional branch come out with it.
-   Several existing fixtures will break,
    and correctly:
    any that assert a
    one-sided run folded into a neighbour are asserting the behaviour this
    replaces.
-   `alignmentPairCount` stops meaning what it says once insertions enter the
    pairs,
    per `#100`.

WHAT IT WILL COST IN CACHE TERMS:
nothing measurable.
Re-slicing changes slice
texts,
which changes both lanes' keys,
and the runs directory holds no record
under either lane's current version.

### Landing four is blocked, and the measurement that blocked it

The wiring was reviewed before it was written,
which is why nothing shipped a
duplicated paragraph into the archive.
Two independent reviews found the same
reflow defect,
and one found the thing that stops the landing.

THE REFLOW DEFECT,
fixed in `0495fb1a5`.
An orphan translation run attached to a
paired unit on the far side of an anchor stretches that unit's span past the
boundary the anchor names,
so the anchor sits inside a span that precedes it in
slice order and `assertPlacementLayout` refuses the whole preparation.
Both
attachment directions have the fault,
so the rule is about the anchor rather
than about the direction:
anchors partition the units into regions,
and an
orphan joins a paired unit only inside its own region,
the one before it where
that exists.
A region with no paired unit leaves its blocks uncovered,
which
costs review and nothing else,
since assembly writes nothing there.

BOTH GUARDS WERE SHOWN TO FAIL against the reflow as `70f46b590` had it,
and one
of them had to be rewritten first:
a target-only step inside an open group joins
that group's interval,
so the case never built a source-less unit and passed
against both implementations.
A budget flush between the two blocks is what
closes a group with no source side.
The first probe of the pair was also
unfaithful,
restoring the region rule's absence but not the original's trailing
attach,
and it reported everything green;
that is the shape of a null result
from a probe that cannot show a difference.

THREE SMALLER CORRECTIONS rode along.
`groupNodes` moved to `group-nodes.ts`,
because the wiring would have had `slice-pair.ts` and `group-source-first.ts`
importing each other.
The anchor boundary became a value naming a BLOCK or the
section end,
rather than an index into a sequence the holder has to guess,
which
is the same class of confusion `#99` fixed for slice indices.
Alignment steps
naming a block that is not there are now refused rather than dropped,
since
dropping shortens a run that then covers a span it does not carry.

WHAT STOPS THE LANDING:
a `source-only` step is not evidence that a passage is
untranslated.
`alignBlocks` can pair one with one,
skip a source block,
or skip a
target block,
and that is all;
it cannot say that two source paragraphs were
rendered as one.
So a merged pair spends the aligner's only available move and
the second paragraph arrives as `source-only`,
identical to an omission.

MEASURED OVER THE PINNED CORPUS,
92 entries and 275 two-sided sections:
2290
paired steps,
95 source-only,
132 target-only.
Twenty-three entries carry at
least one,
and two of them carry sixty of the ninety-five.
A length signal
(how far the neighbouring pair's target-to-source ratio exceeds the section
median) has a long tail,
p90 at 2.41,
and 67 of the 95 sit under 1.2.

A HAND SAMPLE OF TWELVE,
six from each end,
says the length signal separates the
top and says nothing about the bottom.
At the top the steps are merges:
one
entry renders four consecutive Chinese lines as a single English block,
so three
arrive as source-only,
and another does the same to a blockquote.
At the bottom
they are mostly MISPAIRINGS:
an English footnote definition paired with the
following Chinese footnote leaves the preceding one reading as untranslated with
its translation sitting right there,
and a narration line paired with the
translation of a line three blocks later leaves two quoted lines apparently
unrendered.

SO THE ANCHOR DESIGN RESTS ON BLOCK PAIRING BEING TRUSTWORTHY,
and the sample
says it is not.
This is `#74` restated one level down:
that task found the
section-level scoring broken,
and the same weakness decides block pairing.
The
paths out are in `#106` and the question for the morning is in the decisions
doc,
because they differ in expense rather than in correctness.

WHAT IS STILL TRUE AND UNBLOCKED:
the guard (`40d335504`),
the grouper and its
tests,
the reflow rule,
the boundary value,
and every invariant landed earlier.
Nothing calls the grouper,
so none of it decides a corpus yet.
