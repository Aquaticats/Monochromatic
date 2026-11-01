# The third rendering: slices that cut through a page element, from the lost will to the container widening

The fail-open structural floor and its fix,
and how much of the corpus a mid-element cut puts at risk;
the document-level data loss behind the lost will,
the census of cuts and the finding that container tags belong to no block;
the widening of block extents at their origin,
with what was pre-committed,
what it measured and where the pre-commit was wrong,
and the run 7 outcome on the pre-committed slice.

Part of [The third rendering](the-third-rendering.md),
which lists every file of this record.

## The fail-open floor, fixed

Issue #153,
landed in `0cf2f759b` and `393ee5486`,
GFP-proven.

`validateTranslatedSlice` turned a page the strict grammar refused into no page
at all,
so the floor had no block list and fell back to comparing the candidate
against the original alone.
A check that could not run answered yes.

The page side now downgrades to plain markdown;
the candidate side stays strict.
That split is what the two decisions already in the code imply rather than a
reversal of either:
`inspect-paragraph.ts` refuses the fallback for a CANDIDATE,
which this pipeline asked for and may refuse,
while `parse-document.ts`
downgrades a whole PAGE,
which is archive text nobody here wrote.

The relaxed reading deliberately claims no atoms.
Plain markdown collapses an
html region into one opaque node,
so the references and code inside it are
invisible,
and reporting an empty atom list as complete would make the atom
check pass by ignorance.
Only the block sequence is claimed.

### What the regression diff showed

The floor section was rerun over all thirteen bed slices with the patched
validator.
`Zha_Ke#1` shipped flips from valid to invalid,
with the finding
naming a page of two blocks against a candidate of one.
No other verdict
anywhere in the scorecard moved.

### Nothing ships at a slice cut through an element, and that is correct

Two guard cases pin this,
because it is a consequence worth knowing rather than
an oversight.
A candidate reproducing the cut page exactly is refused at the
CANDIDATE parse,
which stays strict.
A candidate that writes the element
correctly is refused at the FLOOR,
because the two grammars disagree about what
the same element is:
the page read loosely calls it an html block,
a candidate
read strictly calls it an `mdxJsxFlowElement`,
and the kinds never match.

Fail-closed both ways is the safe answer for a span that is not a well-formed
fragment.
Every candidate is refused,
the incumbent stays,
and the content
survives.
It is not shipping,
and making such slices shippable belongs to the
slicer,
which is #154.

### The downgrade is recorded, and the source-side twin is not a live problem

The fallback was itself silent at first,
which the repo's own parser policy
forbids.
A refusal already shows which reading produced it,
since the finding
names the blocks compared,
but a pass carried no evidence at all.
The verdict
now carries the grammar that read the page,
and `translate-repair.ts` records
`translate-page-downgraded` beside the existing `translate-unvalidated`.

The source arm of the same check was probed and left alone.
It returns `unknown`
on a strict refusal,
`translate-repair.ts` already records that as
`translate-unvalidated`,
and zero of the thirteen bed slices reach it.
A
recorded fail-open with no live case is not worth changing without one.

## How much of the corpus the mid-element cut puts at risk

Measured while run 7 was in flight,
since it needs no quota.
Counts only:
no
passage is reproduced.

    -   18 of 92 English pages carry block-level HTML.
    -   12 of 92 entries use `<details>` specifically, which is the multi-line
        wrapper a slice boundary can land inside. `XingZ60` uses six of them.
    -   Of the sixth bed's thirteen slices, exactly one page needed the relaxed
        grammar, and it is `Zha_Ke#1`.

The probe that produced the first two numbers was validated against a positive
control before it was believed:
an earlier version asked
`validateTranslatedSlice` for the grammar behind a pass,
which cannot work,
because the floor refuses such a page before any verdict carries a grammar.
It
reported zero,
including for `Zha_Ke#1`,
whose page is known to refuse the
strict grammar.
Reading `readPageSkeleton` directly reports `Zha_Ke#1` and one
other nothing,
which is the answer.

So `Zha_Ke#1` is not a one-off.
Roughly one entry in eight carries an element a
slice boundary can cut,
several of those pages run to hundreds of lines,
and
after the floor fix a slice cut through one ships nothing at all.
That makes
#154 a shipping blocker for that fraction of the corpus rather than a tidiness
issue,
and it should be read alongside #71,
which found a different defect on
the same `XingZ60` entry.

## The slice boundary that cut a page element: the lost will, the census and the container tags

### #154 is document-level data loss, and the pairing already knew

Two questions were open on #154:
where the boundary came from,
and whether the
assembly recovers what a slice drops.
Both are answered from archived state,
at
no quota cost.

The boundary came from a fallback,
and the pairing had already refused the job.
The `Zha_Ke` artifact records `alignmentPairCount: 1` against `sliceCount: 4`,
with two findings,
one per model asked:

```text
block-pairing unusable (hf:zai-org/GLM-4.7-Flash: pairing moves backwards on the translation side at position 3)
block-pairing unusable (hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4: pairing moves backwards on the translation side at position 3)
```

So the LLM-assisted pairing of #131 was asked,
both answers were rejected as
non-monotone,
and the run then fell back to assigning boundaries by character
fraction.
The entry carries 303 source characters against 4388 target
characters,
because the page holds a long `<details>` block the Chinese never
transcribes.
Splitting a 14-to-1 asymmetry proportionally hands a 41-character
Chinese content warning a 3875-character English region.

That region is mid-element.
It contains `<details>`,
`<summary>` and
`</summary>`,
and it stops before `</details>`.
This is why the page fails the
strict grammar,
and why #153 mattered:
the floor used to read the refusal as
"no page" and skip the check entirely.

Assembly does not recover it.
Splicing the run-6 slices back over the archive
text gives a page of 680 characters where the archive had 4388,
losing 3708.
The tag counts show the shape of the damage:

```text
<details>   archive x1   assembled x0
</details>  archive x1   assembled x1
<summary>   archive x1   assembled x0
```

The opening tag and the whole body are gone and the closing tag survives with
nothing to close.
So the assembled document both deletes the will and emits
unbalanced markup.
This settles #154 as urgent before release rather than
urgent eventually.

It also names three separate fixes rather than one.

The first is the fallback itself,
which is #71's unfinished half:
a pairing the
roster declared unusable must not be replaced by a confident wrong answer.
The
signal is recorded in the artifact and nothing reads it,
which is now the third
time that pattern has been found.

The second is that a slice boundary must never fall inside an element,
whoever chose it.
That is a structural invariant the assembler can enforce
regardless of pairing quality.

The third is #90 and #100:
a page region with no counterpart in the original,
such as this `<details>` block,
is one-sided content.
It should be recognised
and held out of pairing,
not handed proportionally to whichever source chunk
happens to sit nearby.

### The census: mid-element boundaries are not a fallback problem

Sizing #154's structural invariant meant asking how often a real slice boundary
lands inside an element.
Every settled artifact carrying slice regions was
scanned,
which is 18 unique entries and 124 slices,
by walking each region's
tags and asking whether every opener it holds is closed inside it and every
closer it holds was opened inside it.

Three slices of 124 cut an element,
across two entries of 18.
That eighth-of-
the-corpus figure matches the earlier page-level count,
where 12 of 92 English
pages use `<details>` and 18 of 92 carry block HTML.

The important result is the second entry.
`AkiraComplex` splits a `<BlurBlock>`
across its two slices,
with the opener in one and the closer in the other,
and
its `alignmentFindings` is empty.
No pairing was declared unusable,
no fallback
ran,
and the asymmetry is a mild 451 source characters against 714 target
characters.
The accepted pairing placed that boundary itself.

So the fallback is not the cause of mid-element boundaries,
only the cause of
the `Zha_Ke` one.
Fixing the fallback would have left `AkiraComplex` exactly as
it is.
The structural invariant is required on its own account rather than as
a second line behind the pairing fix,
and it is the only one of the three fixes
that catches both cases.

Damage did not land on `AkiraComplex`,
but not because anything prevented it.
Both lanes reproduced the tag they were handed,
so the element survived.
Nothing
required them to.
Each slice on its own carries unbalanced markup,
so the page
grammar refuses it exactly as it refuses the `Zha_Ke` region,
and a candidate
that simply translates the prose it was given drops the tag without being wrong
about anything it was asked.

That failure mode is worse than markup corruption here.
`BlurBlock` is the
content-blurring component from #49,
so an opener dropped while its closer
survives renders as sensitive content with the blur removed.
It is latent rather
than realised,
and it is a reader-protection failure rather than a rendering
one.

One limit on the census:
it reads regions that were assigned to a slice,
so a
page region no slice covers cannot appear in it.
It therefore undercounts
exposure rather than overcounting it.

### The real cause: container tags are in no block at all

The census said the invariant should have caught this,
because
`assertSpanContiguity` already refuses a slice that cuts through a block and it
runs at preparation,
at `document-preparation.ts:293`.
Running the real
preparation over the real `Zha_Ke` pair explains why it did not.

`parseDocument` returns six nodes for that page,
and none of them is the
`<details>` element.
The nodes run to the element's contents directly,
as a
paragraph for the summary and a blockquote for the will,
and the tags
themselves land in the gaps between nodes:

```text
  block/1   [  159,  370) paragraph
  GAP       [  370,  382) NOT IN ANY NODE: "\n\n<details>\n"
  block/2   [  382,  408) paragraph
  block/3   [  409, 4034) blockquote
  GAP       [ 4034, 4047) NOT IN ANY NODE: "\n</details>\n\n"
  block/4   [ 4047, 4103) mdxJsxFlowElement
```

`AkiraComplex` is the same shape,
with `<BlurBlock>` at `[264, 278)` and
`</BlurBlock>` at `[669, 684)`,
both in no node.

So the slice at chunk 1 covers `[159, 4034)` and carries exactly three whole
nodes.
No block is cut,
every block inside the range is one the slice holds,
and
the invariant is satisfied on its own terms.
The opening tag sits in a gap
inside the range and the closing tag sits in a gap outside it.
Assembly replaces
the range,
which deletes the opener,
and copies the untouched remainder,
which
preserves the closer.

That is the whole mechanism behind the assembled page carrying `<details>` zero
times and `</details>` once.

The defect is therefore not that the slicer cuts elements.
It is that the
document node model has no representation for a container's tags,
so a class of
page content exists only as text between nodes.
Every invariant reasons over
nodes,
so none of them can see it,
and any slice whose range happens to span
such a gap deletes what is in it.
`assertSliceCoverage` is blind for the same
reason,
since it asks whether every block reached a slice.

This also explains why `AkiraComplex` needed no fallback to reach the same
state.
Nothing about a bad pairing is required.
An ordinary boundary between two
ordinary blocks does it,
whenever a container happens to open between them.

### The guard, and what it finds across the whole corpus

The parse now reports what it dissolves.
`flattenContainers` returns the
container spans beside the blocks,
`parseDocument` shifts them to absolute
offsets and carries them on `RepairDocument`,
and a new
`assertContainerIntegrity` runs at preparation beside the checks that already
guard blocks.

The rule is both tags or neither,
rather than no tag inside a range.
An element
wholly inside one slice puts both of its tags in that range,
which is the
ordinary healthy case:
the slice text carries them and a candidate that drops
them is caught downstream where the page grammar refuses it.
A rule against any
tag in a range would refuse those pages too,
and they are the majority of the
pages that use containers at all.

Running preparation over all 92 pairs,
with no pairings supplied:

```text
pairs read: 92
pages carrying a container: 16  (containers in total: 24)
ContainerIntegrityError fired on 11
other preparation failures: 0
```

So 11 entries of 92 currently produce a page that loses an element and emits
markup closing nothing.
Five of the 16 container-carrying pages are clean,
which
is the measurement that says the predicate discriminates rather than simply
refusing everything it sees.

The elements involved are `<details>` on seven entries,
and one each of
`<BlurBlock>`,
`<div>`,
`<Hexagon>` and `<p>`.
`XingZ60` carries nine containers
by itself.
Two entries,
`hulicaijia` and `lin10104`,
fire the mirror case,
where
a range holds the closing tag without the opening one.

Zero other preparation failures means the change refuses nothing it did not aim
at.

Two honest limits.
The dry run supplies no block pairings,
so production
subdivides differently and the slice indices above will not match it;
what
carries over is the exposure and the fact that the invariant runs in production
too.
And the guard converts silent loss into a loud refusal rather than into a
shipped page,
so those 11 entries now stop at preparation.
That is the right
trade against deleting a will,
and it is why the slicer still owes a treatment:
binding a container's blocks into one slice where its interior is paired
content,
and holding the region out where it is not.

### The guard fails when removed, per GFP

The invariant was neutered with an early return,
rebuilt,
and the suite run.
Exactly the three refusal cases failed and the four acceptance cases still
passed,
which is the expected shape:
an acceptance case passes vacuously against
a check that does nothing,
so only the refusals carry the proof.
The invariant
was then restored from the commit and the suite is green again at `exit=0` with
no failing lines.

## Container tags: the defect has one origin and two consumers

Recorded 2026-08-21,
before the fix is written,
so the expectations below count as evidence.

### The guard's census, and what it hides

`assertContainerIntegrity` refuses 11 of the 92 readable pairs.
Naming them,
because the next measurement has to reproduce this exact set:
`AkiraComplex`,
`SevenBird`,
`XingZ60`,
`Zha_Ke`,
`cheonwoomaeng`,
`gqt`,
`hulicaijia`,
`lin10104`,
`mikaela_khara`,
`shihai4h`,
`zhangyubaka`.
Sixteen entries carry a container at all,
so five carry one and are not refused.

The refusals do not share a cause,
and reading them as one cause is what sent the first fix at the wrong file.
Eight of the 11 hold a container wholly inside one paired chunk,
and the cut happens when subdivision splits that container's blocks across two slices.
Three do not:
`SevenBird`,
`mikaela_khara` and `XingZ60` hold a container whose closing tag
falls outside the chunk that holds its own children.
On `mikaela_khara` the chunk ends at 8581,
the closer occupies 8581 to 8593,
and the next chunk begins at 8595,
so the tag sits in a 14 character gap between two chunks.
On `SevenBird` the closer begins exactly at its chunk's end offset.

That is the same defect one level up.
Chunk boundaries are minted at `src/chunk-document.ts:190` from `first.startOffset` and `last.endOffset`.
Slice boundaries are minted in `runToChunk` at `src/slice-pair.ts:76` from the same two fields.
Both read node extents,
container tags belong to no node,
so both draw boundaries that cut tags off.
A fix at subdivision alone would leave `SevenBird` and `mikaela_khara` refused,
because their damage is done before subdivision runs.

### Widening at the origin rather than binding at the consumers

The first design was atomicity at subdivision:
treat a container as indivisible,
carve over atoms,
widen at the edges.
It is rejected,
and the measurement is what rejects it.
It repairs one of the two consumers,
and it leaves any range derivation added later exposed to the same defect.

The fix instead widens the extents where the package first writes them:
a block that carries a container's opening tag owns that tag,
and a block that carries the closing tag owns that one.
Every consumer that reads `startOffset` and `endOffset` inherits the correction without knowing containers exist.

Two facts make this safe,
and both are measured rather than assumed.
Across both sides of the whole corpus there are 53 containers and not one of them
has a tag region overlapping any block node,
so widening cannot collide with a neighbour.
Nesting exists on `XingZ60` alone,
four pairs across its two sides,
and it composes:
a block takes the minimum opener start among containers it opens
and the maximum closer end among containers it closes.

Widening also dissolves the atomicity requirement rather than implementing it.
When a container's blocks split across two slices,
the first slice carries the opening tag inside its own text
and the last slice carries the closing tag inside its own text.
Each tag is wholly held by some slice,
neither is half held,
and the invariant is satisfied by construction.
The `AkiraComplex` shape stops being a merge problem.

### What is expected after the change, committed in advance

The census must report 0 `ContainerIntegrityError` across all 92 pairs,
with no qualification.
`XingZ60` carries a ninth container in a target-only section that no paired chunk reaches;
it needs no treatment,
because no slice reaches it,
neither tag is ever held,
and `spliceSlices` copies the region verbatim.
So the expected number is 0,
not "0 except one".

Every entry without a container must produce byte-identical slices.
Seventy-six of the 92 have none,
and any difference there means the widening reached text it had no business reaching.

On the 16 entries that do carry one,
every tag region must fall inside some slice's range.

Cache misses on those 16 are expected,
and they are invalidation rather than breakage:
the slice text changed,
so the key changed.

### What the guard is for afterwards

Its job changes,
and saying so now prevents a later session reading it as dead weight.
Before the widening it stood between a container and a document that loses a will.
After the widening,
preparation cannot produce a half held tag at all,
so the guard stands against a future regression in range derivation rather than against damage in flight.

The residual exposure moves rather than disappearing.
A lane can still drop a closing tag from its own candidate,
and that is now a candidate fault rather than a preparation fault.
The machinery for it already exists:
the floor reads a page the strict grammar refuses,
and a deleted region is a named fault bounded to a silent original.
Expected steady state,
committed in advance:
a container defect that used to stop preparation
now appears as a contest-visible fault on one candidate,
and a different candidate ships.
No new machinery is to be built for it in this task.

### What the widening measured, including where the pre-commit was wrong

Landed 2026-08-21 as `fdfeb1108`,
with tests in `d081efb91`.

The census reports 92 of 92 pairs preparing clean,
0 `ContainerIntegrityError` and 0 other failures.
That is the pre-committed number exactly,
`XingZ60`'s target-only ninth container included.

One pre-committed expectation was stated wrong and is corrected here rather than quietly restated.
It said 76 entries carry no container and all 76 must produce byte-identical slices.
That count came from parsing the English side only.
Measuring both sides,
75 entries carry no container anywhere and 17 carry one on some side.
All 75 are byte-identical.
The entry that exposed the miscount is `interrgned`,
which carries a `<blockquote>` container
on its Chinese side and none on its English side,
so widening correctly changed its source-side slices
while its English side has no container at all.
There is no unexplained drift.

The guard fails when the widening is removed,
which is what makes it evidence rather than decoration.
Neutering `widenNodesToContainers` to return its input and rebuilding takes the refusals from 0 to 16,
naming every entry whose English side carries a container and leaving the other 76 untouched.
Restoring it returns the census to 0.

### The old rule was measured wrong, not reasoned wrong

Worth recording because the sequence is the useful part.
The widening landed first while the guard still enforced its original rule,
that one slice range must hold both of a container's tags or neither.
The census then reported 12 refusals rather than 0:
`SevenBird` and `Zha_Ke` were fixed,
and `Chinatsu_Suzuki`,
`Huasheng` and `LCG_Akiball` were newly refused.

That is the fix working and the guard disagreeing with it.
Once tags ride inside blocks,
a container whose blocks fall in different slices is the healthy case:
the first slice carries the opening tag in its own text,
the last carries the closing tag in its own,
each lane sees the tag it must reproduce,
and the page stays balanced.
The old rule called that a defect because it was written for a world where tags floated between blocks.

So the rule became two narrower ones.
Every tag of a container that holds blocks must lie wholly inside one of them,
and no slice range may end part way through a tag.
Both fire only on a regression in how extents or ranges are derived,
which is what the guard is now for.
A lane dropping a tag from its own candidate is a candidate fault,
carried by the floor that reads a page the strict grammar refuses and by the named deleted-region fault.

### Verification state

Suite exits 0 with zero failing cases.
Lint reports 0 errors and 1 warning,
the inherited `no-unsafe-type-assertion` on `unwrap-container.ts`,
already shown to be load-bearing:
removing the assertion loses the borrowed-foreign marking
and `prefer-readonly-parameter-types` errors in its place.

Two documents that told the old story were corrected rather than left standing.
The `unwrap-container.ts` header said the tags "fall outside every promoted span" and that this was safe;
it now says where they go instead and that a caller reading blocks while dropping containers
reintroduces the defect in full.
Its test case asserting that no promoted block contains a container tag was asserting the defect itself,
so it is reversed rather than deleted.

### Run 7, the pre-committed slice

`Zha_Ke#1` was pre-committed to ship nothing,
with the will preserved.
It shipped nothing and the incumbent was retained,
so the outcome held.
The mechanism did not:
the pre-commit predicted every candidate refused by the floor,
and what happened was `declined-indecision` at the judge,
with the incumbent kept for want of agreement.
Recording the difference because a right outcome reached by an unpredicted route
is weaker evidence than one reached by the predicted mechanism,
and the distinction should not be lost.

### The instrument that proved it, and the one that could not

The first attempt at an end-to-end check assembled each container entry with identity replacements,
every slice handing back exactly the text it was cut from,
and compared the result to the archive.
All 16 came back byte-identical with no tag lost.
That result is worthless,
and the positive control is what showed it:
neutering both the widening and the guard and running the same probe returns the same clean answer.
Identity replacement writes back the slice text it was given,
and a tag inside a slice range
is inside that slice's text,
so the probe round-trips whatever it is handed.
It cannot distinguish a fixed pipeline from a broken one.

The discriminating question is narrower,
and it is the asymmetry the widening actually removes.
A tag inside some slice's range is text a lane rewrites.
A tag inside no slice's range is text assembly copies through untouched.
When a container has one of each,
the two tags are decided by different authorities:
one can be dropped by a candidate while the other is guaranteed to survive,
which is precisely how a page ends up with markup closing nothing.

Measured across the target sides that carry containers,
24 containers reached in both builds:

    before  6 containers with both tags in a slice range, 7 with neither, and 11 SPLIT
    after   23 with both tags in a slice range, 1 with neither, and 0 SPLIT

The 11 name themselves:
`SevenBird`,
`XingZ60` three times,
`Zha_Ke`,
`cheonwoomaeng`,
`gqt`,
`hulicaijia`,
`lin10104`,
`mikaela_khara`,
`zhangyubaka`.
The single remaining "neither" is `XingZ60`'s target-only section,
which no slice reaches by design.

WHAT THIS DOES NOT CLAIM.
It does not claim a lane can no longer drop a tag.
It claims that both tags of a container are now decided by the same authority,
so dropping one is a candidate fault visible to the contest
rather than a structural certainty invisible to every invariant.
The residual exposure is named in the section on what the guard is for,
and it is unchanged by this.

### Three things the widening changes that are not defects

Recorded so a later session does not read any of them as a regression.

Run 7's slices were cut BEFORE the widening,
from the cached bed state.
Scoring run 7 against run 6 stays valid,
because both read the same slices,
but per-slice comparison on the container entries ends at run 7:
any freshly prepared run re-slices them.
Run 7 must not be credited with the container fix,
which landed after its slices were cut.

Seventeen entries will re-buy,
not sixteen.
The source side widens too,
so `interrgned` re-buys on its Chinese-side `<blockquote>`,
and the five container entries that previously settled clean
(`Huasheng`,
`Chinatsu_Suzuki`,
`LCG_Akiball`,
`NIGHT81473140`,
`hakureico`)
re-buy against new content hashes.
Cache misses on those 17 are invalidation,
because the slice text genuinely changed.

The residual expectation is recorded and NOT yet observed.
A lane dropping a container tag should now surface as a contest-visible candidate fault,
carried by the floor and the deleted-region rule.
Nothing has exercised that path since the widening landed,
so it stays an expectation until the first post-widening run touches a container entry.
