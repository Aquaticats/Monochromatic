# Translation repair history 2026-08-23: pairing and slicing documents across languages

The section aligner blind across languages and the model-assisted section pairing built for it,
the adversarial pairing sweep and the defects it found,
slicing of a section nothing rendered,
and the measurement showing the equal-count fast path is safe.

Part of [Translation repair history: 2026-08-23, the work recorded for that day alone](translation-repair-history-2026-08-23-single-day.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The section aligner is blind across languages, and one page ships untranslated

Measured 2026-08-23 over all 92 pinned pairs at sha `a41fc607`.
This is `#189`,
and it supersedes the "entirely untranslated page" note that closes `#100`:
the page short-circuits for a much earlier reason than the one that note guessed at.

### What ships today

`XIEPT2` produces ZERO slices.
Its English page carries nine headings and 246 characters of body against 7365 characters of
Chinese,
a target/source ratio of 0.17 against a corpus median of 2.65,
the lowest in the corpus.
Nothing pairs,
so no slice exists,
so neither lane is ever asked about it,
and the page ships exactly as the archive holds it.

`ArtsEpiphany` and `gaoyanger` also read as covering under half their source,
and CHECKED,
both
are the measure's fault rather than the pipeline's:
that ratio divides slice source text by the
whole file,
which includes front matter,
heading lines and JSX markup.
`ArtsEpiphany` is one
`mdxJsxFlowElement` holding 33 characters of text,
all of which reach a slice;
`gaoyanger`'s two
sections are both covered.
Neither is a defect,
and a coverage measure that means anything has to
divide by chunked body text rather than by file length.

### Why: the affinity grid is all zeros

`headingAffinity` is token overlap,
and Chinese headings share no tokens with English ones.
Cell by cell:

-   `XIEPT2`:
    all 72 of its 8 by 9 affinity cells read 0.00.
-   `XingZ60`:
    192 of 195 cells read 0.00.
    The three nonzero cells read 1.00,
    and they are exactly the three English headings carrying a
    romanised name that also appears in the Chinese heading.

With an all-zero grid the lexicographic score has nothing to rank pairings by except its
gap-count term,
so every pairing is equally supported,
`scanOptimalPaths` reports many optimal
paths,
and the forced aligner refuses.
All 11 unpaired source sections corpus-wide come back `ambiguous`,
and every insertion anchor
comes back `may-pair`.

THE `proven` ANCHOR NEVER FIRES ON REAL DATA.
`#100` landing 5 and its corroboration gate are
correct and GFP-proven,
and they are also unreachable:
the refusal happens one step earlier,
inside the aligner.
The gate's evidence remains invented fixtures,
and that is now recorded
rather than assumed.

### What `may-pair` is actually reporting

It is written to mean "some optimal alignment matches this against existing translation,
so
inserting risks duplication".
On `XIEPT2` it means "I have no information".
For 6 of the 11 unpaired source sections EVERY possible partner is a target section with a
ZERO-character body,
so no content could be duplicated by any of them.
`XIEPT2`'s target section bodies are `[7,0,0,0,0,0,0,0,239]`.

### The LLM-assisted mechanism exists and is gated behind the blind one

`#131` landed LLM-assisted pairing,
cached,
settling both entries cleanly.
But `SectionBlockPairing` is keyed by `sectionIndex`:
it pairs BLOCKS WITHIN a section the
deterministic aligner already aligned.
On `XIEPT2` no section aligns,
so the roster is never asked anything at all.

### Scope, measured before building

Only TWO entries in the whole corpus would be asked,
because 85 of 92 have equal section shape
and skip the aligner entirely,
and 5 of the remaining 7 align with no refusal:

-   `XingZ60`,
    15 source against 13 target sections,
    4 refusals,
    49921 characters whole.
-   `XIEPT2`,
    8 source against 9 target sections,
    17 refusals,
    7654 characters whole.

SECTIONS GO WHOLE,
UNTRUNCATED.
The smallest context window in `synthetic-catalog.ts` is 131072
tokens and the largest sheet is under 50000 characters,
so a cap would buy nothing and could only
withhold the body text the pairing question is about.
Cost is not a constraint here:
two entries,
once each,
cached.

### Rejected

-   Deterministic cross-language heading affinity,
    by transliteration or ordinal matching.
    It already scores 1.00 on precisely the cells it can reach,
    and would do nothing for
    `XIEPT2`,
    whose grid is entirely zero and whose headings carry no romanised names.
-   Proportional-by-character fallback.
    That is the design `#71` was opened on,
    and it corrupted
    `XingZ60` by sliding every pairing two sections along.

### What landed, and what it does to the two entries

Section-scale LLM-assisted pairing,
mirroring the block-scale stage `#131` already built:

-   `pair-sections-wire.ts` builds the sheet.
    Sections go WHOLE and the policy tells the model
    that a section whose body was never translated STILL CORRESPONDS,
    which is the instruction
    `XIEPT2` turns on.
-   `pair-sections-read.ts` refuses anything unusable.
    STRICTER THAN THE BLOCK READER on exactly
    one point:
    strictly increasing on both sides,
    because a `ChunkPair` carries one section on
    each side and a split or a merge would silently drop whichever section lost the race.
-   `pair-sections-stage.ts` asks the roster and keeps what two voices agreed on,
    per pair.
-   `pair-sections-steps.ts` turns the pairing into the aligner's own step vocabulary,
    so nothing
    downstream learns a model was involved.
-   `prepare-section-round.ts` buys the round,
    and only where the deterministic aligner refused.
-   `alignDocumentSections` takes the pairing as DATA,
    exactly as it already takes a block
    pairing,
    and a supplied pairing REPLACES the deterministic decision rather than supplementing
    it.
    Preparation stays pure and synchronous.

The section round is bought FIRST,
before the block rounds,
because those are asked one aligned
section at a time and are therefore questions about an alignment that has to exist already.

### Verified live, both entries, 2026-08-23

Six of six voices answered usably on each.

-   `XIEPT2`:
    8 of 8 sections paired,
    at `0->1 1->2 ... 7->8`.
    The roster put the EXTRA English
    section at the front,
    which no hand-written guess in this session got right.
    Slices go from
    ZERO to 9,
    and source reaching a slice goes from 0 to 7288 of 7365 characters.
-   `XingZ60`:
    13 of 15 sections paired,
    straight through.
    The two originals left out become
    INSERTIONS anchored at offset 33450 of 33451,
    which is the end of the page,
    and that is
    exactly where `#71` recorded the Chinese carrying sections the English lacks.
    Slices go from
    84 to 91,
    and source reaching a slice goes from 10617 to 16467 of 16733 characters.

Corpus-wide that is 13138 of the 13147 characters the slice census reported reaching no slice.
The `proven` insertion anchor fires on real data for the first time.

### The proven anchor comes from the pairing, not from the table

Two paired sections either side of an unpaired one pin it to the span between them,
and when
that span is one boundary wide the anchor is PROVEN in exactly the sense the deterministic path
meant by it.
Both ends are handled:
an original nothing precedes anchors at the front rather than
at offset zero where front matter sits,
and one nothing follows anchors past the last translation.
That closes the "entirely untranslated page" item `#100` left open,
by a different route than the
one that item guessed at.

### A failing test the corroboration gate had left behind

`translate-document.unit.test.ts` had been failing since `#100` landing 5 landed,
and the
"suite 556 PASS" recorded above was measured before the last wiring of that landing.
The case
appends an untranslated passage to a pair whose translation runs 3.4 English code points per
source point against a corpus median of 2.65,
so the page is not missing anything and the gate
correctly refuses to write into it.
The fixture now uses a pair that runs 1.09,
leaving a
shortfall of 156 against the 45 points the appended passage would render into.

A second case now covers the refusal itself,
which nothing tested end to end:
it asserts that
NOTHING was bought,
so the passage carries no findings and the run costs exactly what the same
document costs with nothing appended.
GFP-proven:
disabling the gate fails that case alone.

Suite 562 PASS,
exit 0.
Lint 0 warnings 0 errors.
Types clean.

## The adversarial pairing sweep: two real defects, a third they were masking, and three answered items

`#100` had three hardening items left,
all of the form "a roster pairing might produce a shape
the deterministic path does not".
Reasoning about them had gone as far as it could,
so the
question was handed to a sweep instead:
generate randomised monotone block pairings,
feed them to
`prepareDocumentPair`,
and tally what breaks.

### Validating the instrument came first, and it is what made the result mean anything

The first run reported `SliceCoverageError` on 368 of 910 pairings and a bare `Error` on 123.
That
number was worth nothing at the time,
because the generator had never shown that the shapes it
invents are shapes the production wire accepts.
A pairing `readBlockPairing` would refuse can
never reach `prepareDocumentPair`,
so a failure on one says nothing about the pipeline.

Filtering every generated pairing through `readBlockPairing` refused 0 of 910.
A zero refusal rate
is exactly what an inert filter also reports,
so a positive control followed:
the filter refuses
backwards-on-source,
backwards-on-target,
out-of-range on either side,
and a duplicated
correspondence,
and accepts the legal split and the legal merge.
Only then was the tally evidence.

### Two defects in `mergeOneSidedRuns`, which disposed of held blocks three different ways

The function holds blocks from one-sided runs until something can carry them,
and it let them go
at three sites under three rules.
Two of those rules were wrong,
and they disagreed with each
other in the same file.

The flush ahead of an insertion run fired on EITHER held side being non-empty and then pushed a
`paired` run built from both held lists.
With only one side held,
that is a run with nothing on
the other.
A run's span is cut from its first node to its last,
so `runToChunk` had no span to cut
and threw a bare `unreachable` error:
123 of 910.

The tail flush fired only on BOTH sides being non-empty and discarded whatever was left.
That
defeated a deliberate gate.
`declinedTargetIds` declines nothing unless the pairing placed every
original,
precisely so that an unclaimed translation block stays in review rather than leaving it,
and the grouping then dropped that block anyway:
534 of 3000 pairings,
counted after excluding
declines exactly as `assertSliceCoverage` excludes them.

Both now go through `placeHeldRuns`,
which pushes a new run only when both sides are held and
otherwise folds.
The sides fold differently,
and the asymmetry is not a special case but a
consequence of what an insertion run is:
it carries originals and a translation OFFSET rather than
translation blocks.
Held translations may therefore fold back past one,
because it contributes
none of them.
Held originals may not,
because its own originals sit between,
so they join the
insertion instead.

### The before-and-after was a per-pairing vector, not two totals

Comparing two summary counts would not have shown whether a pairing that passed before now fails.
Both runs wrote a per-pairing outcome keyed by entry and round,
and the diff over the same 910:
419 passed before and still pass,
60 that crashed now pass outright,
431 moved to a later
assertion.
Zero regressions,
stated as a fact about individual pairings rather than about a total.

### The third defect, which the first two had been hiding

Those 431 arrive at `PlacementLayoutError`.
The synthetic end-to-end sweep reports the identical
689 failures with the fix stashed and with it applied,
so it is pre-existing rather than caused,
and the smallest case is two paragraphs against two with only the first pair named:

```text
steps  P0/0 S1 T1
runs   paired[src block/0 | tgt block/0, block/1]   insertion[src block/1 | @24]
target block/0 = 0..22, block/1 = 24..46
PlacementLayoutError: slice at position 1 starts at 24 while the slice before it runs to 46
```

`anchorOffsets` reads the monotone walk,
and the walk describes the layout right up until merging
folds an unclaimed translation into a neighbour and stretches that run's span over it.
The anchor
named a boundary that stopped being one.
Anchoring now runs after merging,
in
`group-run-anchor.ts`,
reading each insertion's boundary off its settled neighbours.

### What that leaves for the three items `#100` was holding

Crossing refusal turns out to need nothing downstream,
because the shape cannot be said at the
wire:
`readBlockPairing` refuses `A, B, A` on either side and plain interleaving,
since monotone
on both sides forbids all three.
Two of those three had no test and now do.

Many-to-one ownership shows no duplication across 7910 reader-legal pairings whose generator
merges on a fifth of its steps,
measured by an assertion that every block appears exactly once.

Complete-interval materialization is a null from `assertSpanContiguity`,
and it is worth something
only because the detector was checked first:
it carries no declined-block exemption,
and a
positive control confirms it refuses a span whose offsets cover three blocks while carrying two
and accepts the same span carrying all three.
That null could not have been taken before today,
because `assertPlacementLayout` runs first and 431 of the 910 died there.

Every sweep is now clean:
910 corpus pairings,
4000 synthetic end to end,
3000 grouping
invariants.

### Reach in production, recorded so nobody re-derives it

None of this fired in production,
and the reason is worth keeping.
Every cached production pairing
on disk is straight-through or straight-through with one split.
`Zha_Ke`'s `0-0 1-1 2-4 3-5` skips
two translations but places every original,
so declines apply and those blocks leave cleanly.
The
live `XIEPT2` pass,
the least-translated page in the corpus and the likeliest to leave originals
unrendered,
prepared 8 aligned units with no assertion firing.

The randomised generator skips a source or a target a fifth of the time each,
which is far more
unpaired than any roster reply seen so far.
These were latent defects.
The fixes are what keep
them latent when a roster answers differently,
which is a property of a model rather than a
guarantee.

### An incident with `git stash` worth not repeating

`git stash push -- <path>` on a path with no changes creates NO stash entry,
so a paired
`git stash pop` pops whatever was already on the stack.
Here that was an unrelated `autostash`,
and it left `mise.toml` conflicted.
Repaired with `git checkout HEAD -- mise.toml`,
both stash
entries preserved.
Check that a push actually stashed before pairing it with a pop.

Suite 564 PASS,
exit 0.
Lint 0 warnings 0 errors.
Types clean.

## A section nothing rendered is now sliced, and the sparse case turns out to be cured elsewhere

`#90`'s last defect was that subdivision is framed by the TARGET's block runs,
so a section with no
translation at all had nothing to frame by and came back as ONE slice however long its original
was.
The comment at that branch said slicing it needed a driver that inserts rather than replaces.
That driver landed in `#89`,
and `spliceSlices` gained zero-length-span writing and equal-offset
ordering in `610ea11b9`,
so the stated reason had been spent for a week.

Measured on `XingZ60` under the pairing the roster really returned:
two insertion slices of 915 and
1459 source characters become 16 whose largest is 384.
Nothing had to be split,
because those
sections hold 6 and 23 blocks whose largest member is 384 against a budget of 400.

### The sparse-but-not-empty case needed no slicing code at all

`XIEPT2` looks like the same defect and is not.
Its sections ARE paired,
against target sections
that are a bare heading and a dozen characters of body,
so the target chunk is real and the
insertion branch never applies.
Its collapse is cured by BLOCK pairing instead,
and the mechanism
is worth stating because it is not obvious:
a roster asked to pair a section whose translation is
one heading returns exactly one correspondence,
the remaining seventeen originals arrive as
`source-only` steps,
`anchorOffsets` proves their anchors,
and they become budget-bounded
insertions of their own.

```text
XIEPT2         deterministic   9 slices, max 1639,  8 over budget
               + block pairing 32 slices, max  416,  5 over budget
shi_Yumiaoya   deterministic   7 slices, max 1313,  4 reducible over budget
               + block pairing 17 slices, max  409,  0 reducible
cheonwoomaeng  deterministic  10 slices, max  832
               + block pairing 12 slices, max  784,  0 reducible
```

The 32 matches what the live `XIEPT2` pass really produced,
which is how the mechanism was
confirmed rather than assumed:
its cached block pairings read `0-0` on seven of eight sections.

### What is left is a floor rather than a defect

Over the whole pinned corpus on the deterministic path,
12 of 1259 slices exceed the 400-character
source budget,
and 7 of those are a SINGLE BLOCK.
Splitting a paragraph is out of scope by
decision,
so those seven are the floor.
The other five are the sparse-target shape,
and block
pairing takes them to marginal run-closing overshoot or to a single block.

### The assembly side was checked rather than assumed

`spliceSlices` is what has to absorb sixteen insertions at one boundary,
and its existing test uses
two.
Driven directly with sixteen:
all sixteen written,
in slice order,
blank-line separated,
the
surrounding text intact,
and the whole group placed between the passages either side.
The two-item
test already catches a reversal,
so no near-duplicate case was added;
this is the record that the
count the new slicing really produces was exercised.

Suite 565 PASS,
exit 0.
Lint 0 warnings 0 errors.
Types clean.

## The equal-count fast path is safe, and this time the null was taken with an instrument that can disagree

`#98` held that `alignDocumentSections` takes a fast path whenever the two sides have equal section
counts,
so a document that omits one section and gains an unrelated one later is index-paired
straight through and reports nothing.
Its own fix order said not to route that path through the
deterministic aligner just to route it,
and to wait until heading scoring had a signal for
handle-free headings.
`#189` built that signal,
so the question could be put.

Every entry the fast path serves that has a choice to get wrong was asked.
Of 92 complete pairs,
90
have equal counts;
34 of those are single-section documents,
which pair the only way they can.
The
other 56 each went to the production roster of six.

```text
entries asked                             56
agreed with INDEX ORDER on every pair     56
disagreed anywhere                         0
paired EVERY section                      56
rounds with fewer than two usable voices   0
usable voices                     335 of 335 heard
```

### Why this null counts and the earlier one did not

The 2026-08-15 measurement ran the deterministic aligner over the same entries and also found
nothing.
That was worthless as evidence,
because `headingAffinity` is token overlap and reads 0.00
on handle-free cross-language headings,
so agreeing with position is the only thing it could do.

This one used a roster that reads both documents,
and the SAME roster answers non-identity when the
documents warrant it:
on `XIEPT2` it returned `0-1 1-2 ... 7-8`,
putting the extra English section
at the front,
and on `XingZ60` it paired 13 of 15 and left two originals unpaired.
The positive
control was already in hand rather than constructed for the occasion.

The fast path therefore stays.
Removing it would buy 336 more calls per pass to reproduce an answer
measured identical on every entry it serves.
What changed is that the risk is measured at zero
rather than unobserved.
The probe is rerunnable on any new corpus at `~/temp/agent/fastpath-ask.mjs`.
