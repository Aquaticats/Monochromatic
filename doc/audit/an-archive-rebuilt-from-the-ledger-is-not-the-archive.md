# An archive rebuilt from the ledger is not the archive

Date:
2026-08-19.
Zero quota:
every number here comes from settled artifacts and the pinned corpus.

## What went wrong

`findIntroducedRepetitions` decides whether the pipeline ADDED a repetition by counting a
phrase in the archive and in the shipped document.
The archive is therefore the denominator of every verdict it gives.

Two settled runs predate `#128`,
so their artifacts carry no `archiveText`.
To read them,
a scratch instrument rebuilt the archive by joining `incumbentText`
across the per-slice delivery ledger.
That reconstruction is not the archive.
It covers only the SLICED regions.

For `lintong` the gap is exact and measurable:

```text
whole page.en.md at the pinned commit   1057 characters
artifact preparation.targetChars        1057 characters
ledger reconstruction                    925 characters
unsliced remainder                       132 characters
```

A phrase living in those 132 characters is invisible to the reconstruction,
so a repetition the AUTHOR wrote is scored as one the PIPELINE introduced.

## What it made me record that was false

The handover recorded that the check reproduces "the known `lintong` duplication",
identified as a phrase of six words and thirty-three characters,
archive once and shipped twice.

Against the real archive that phrase occurs TWICE in `page.en.md` itself.
It is the author's own repetition.
The pipeline neither introduced nor removed it.

```text
phrase 6w/33ch:90bddeb5   archive 2   baseline repair 2   new repair 2   faithful
```

Re-reading all five settled entries against their true archives gives a different pool
than the one recorded:

```text
BASELINE, true archives
  lintong        repair 0    (the ledger reading said 1)
  saurikissa     translate 1  4w/27ch:11f6602e   the only real finding in the pool
  dogesir_       0 both lanes
  GLaDOSister    0 both lanes
  wangzihao980   0 both lanes
```

The recorded claim of a finding in `dogesir_` is also stale:
it was measured before the content-word gate landed,
and that finding carries one content word,
so the gate drops it.

## The production path was never affected

`assembleRepair` passes `archiveText: targetText`,
and `targetText` is the whole incumbent document:
the same value the function uses as `incumbentText` and splices slices over.
The artifact agrees,
since `preparation.targetChars` equals the whole file length.

So the shipped check has always counted against the whole archive.
Only the offline reader was wrong.
That is the good direction to be wrong in,
and it is worth stating plainly rather than leaving the reader to infer it.

## What the corrected reading actually shows

The instrument does fire on real introduced repetition,
just not on the phrase previously claimed:

```text
NEW RUN, true archives
  lintong        repair 1    6w/27ch:29296d3a   archive 1, shipped 2, REAL
  saurikissa     0 both lanes
```

`29296d3a` is new.
The baseline repair document contains it once.
The run under the relocation window contains it twice.
That run also carried the anchoring hole,
so the same attribution caveat applies:
the difference is measured,
the cause is not yet isolated.

## Gate A was answering a different question

`window-gates.mjs` derives its phrases from a BASELINE SIGNATURE:
two occurrences in the baseline repair document and one in the baseline translate document.
It never consults the archive.

That signature does not identify introduced duplication.
It equally identifies a phrase the TRANSLATE lane dropped a copy of,
which is the failure the gate's own header calls the worse one.

Measured against the archive,
every phrase gate A derived is faithful in the new run:

```text
phrase             archive  base-repair  new-repair   gate A said        truth
6w/33ch:90bddeb5      2          2           2        STILL DUPLICATED   faithful
4w/15ch:00abe124      1          2           1        ok                 no longer repeated
3w/13ch:8e291bbe      1          2           1        ok                 no longer repeated
```

So the gate's single FAULT is a false alarm,
and its two passes understate what happened:
two genuine duplications are gone rather than the document merely being left alone.

ATTRIBUTION,
STATED CAREFULLY.
The run these numbers come from carried the relocation window AND the `NEARBY_RULE` hole
fixed in `afc7854b4`,
and that hole changed WHICH SLICES were edited at all by collapsing quote anchoring.
So "the window removed two duplications and added one" is more than one confounded
A/B over two entries can support.
What is established is that the documents differ in this way.
Isolating the window needs the re-run on the fixed build.

Both phrases the window fixed sit below the production content-word gate,
which requires two words of at least five letters.
Fifteen characters over four words and thirteen over three cannot clear it.
The instrument is therefore deliberately blind to short repetitions,
which is the tuning that took an earlier reading from five findings to one real one.
That trade is now measured rather than asserted.

## The rule this leaves behind

A derived denominator needs the same scepticism as a derived measurement.
When an instrument reconstructs one of its own inputs,
check the reconstruction against the real input before trusting any verdict built on it,
and prefer a length equality that must hold if the reconstruction is complete.
Here `targetChars` was in the artifact the whole time and would have caught it at once.

## The shipped check cannot see the damage it was built for

Found the same day,
while checking the correction above.

`#107` describes a positional defect:
one slice shipping wording the NEXT slice also ships,
which the archive said once.
Deriving that signature directly,
rather than by the baseline heuristic gate A used,
finds it immediately:

```text
baseline lintong    repair     slices 2+3   6w/23ch:0a157876   archive 1   both newly shipped
new      lintong    repair     slices 2+3   4w/23ch:189cdc15   archive 1   both newly shipped
new      saurikissa translate  slices 7+8   5w/22ch:ab9538c8   archive 0   both newly shipped
```

The damage is still there at the same slice pair in the run under the window,
and `saurikissa`'s translate lane has one the archive never carried at all.

THE PROBLEM:
`findIntroducedRepetitions` reports NONE of the first or third.
The content-word gate requires two words of at least five letters,
and those phrases do not have them:

```text
0a157876   word lengths [4,3,3,2,3,3]   content words 0   gate BLOCKS
189cdc15   word lengths [5,3,4,8]       content words 2   gate PASSES
ab9538c8   word lengths [3,4,2,6,3]     content words 1   gate BLOCKS
```

So the instrument built to catch `lintong`'s duplication structurally cannot catch it.
The gate that removed the noise removed the target with it.

## Why the answer is scope, not a lower threshold

The content gate is doing real work at DOCUMENT scale,
where any two distant sentences may share ordinary phrasing,
and lowering it took an earlier reading back to mostly noise.

ADJACENCY is a much narrower claim,
so it does not need the gate to be specific.
Measured over every settled artifact that carries a delivery ledger:

```text
lane readings with a ledger                22
adjacent-slice repetitions, no gate         1
of those, both sides newly shipped          1
of those, clearing the content gate         0
```

One hit in twenty-two lane readings,
and it is the documented damage.
Older pools cannot be measured this way at all:
196 lane readings predate the ledger and carry no per-slice text.

So the shape to build is a SECOND check scoped to adjacent slices,
with no content gate,
beside the document-scale one that keeps its gate unchanged.
That leaves the measured noise trade alone
rather than reopening a threshold that is doing its job.

## Gate A, rewritten

The signature derivation is gone.
Gate A now runs the two PRODUCTION checks against the archive read from the pinned corpus,
and reports both scales per lane.

It also asserts the equality that would have caught the original mistake at once:
the archive length against `preparation.targetChars`.
A mismatch is an INSTRUMENT FAULT rather than a finding,
because the corpus moved under the artifact and every count below it is then suspect.

On the two entries settled under the window,
before the quote fix:

```text
lintong     repair     adjacent 1 -> 1    document 0 -> 1    WORSE
lintong     translate  adjacent 0 -> 0    document 0 -> 0    ok
saurikissa  repair     adjacent 0 -> 0    document 0 -> 0    ok
saurikissa  translate  adjacent 0 -> 1    document 1 -> 0    WORSE
```

So the window did NOT fix the duplication it was built for.
`lintong`'s slices 2 and 3 still repeat,
with the archive stating the wording once,
and `saurikissa`'s translate lane gained a repeat across slices 7 and 8
that the archive never carried at all.

THE ATTRIBUTION CAVEAT STILL APPLIES.
That run also carried the `NEARBY_RULE` hole,
which changed which slices were edited at all,
so this measures the window plus the hole rather than the window.
The re-run on the fixed build is what separates them.

## The same mistake in a second instrument, found by reading its hits

Gate B counts sentences severed mid-phrase and spliced onto an unrelated one.
Its detector looks for a function word that cannot end a clause,
immediately followed by a capitalised clause-starting pronoun,
with no punctuation on the left word.

Its pronoun list contained `I`.

English capitalises `I` in every position,
so it carries no sentence-start signal whatsoever,
and the rule fired on `and I`,
`but I` and `that I`.
Those are ordinary prose.

Reading the hits rather than the count separates them at once:

```text
saurikissa repair, slice 7   a dangling `of` followed by `She`   REAL
lintong    repair, slice 3   `But` followed by `I`               false positive
GLaDOSister both lanes       `that` and `and` followed by `I`    false positives
```

Corrected,
with the known-severed case held as a positive control so the fix is not just
a quieter instrument:

```text
                     repair        translate
flagged-20260818     1 of 30       0 of 34
20260817             0 of 32       0 of 37
windowed run         0 of  7       0 of 10
```

One severed join in one hundred and thirty-three shipped rows,
rather than the twelve the broken detector reported.

WHAT THIS CORRECTS:
an earlier reading of the windowed run recorded a severed join at
`lintong` slice 3 and treated it as a regression the window introduced.
There is no such join.
The only real one is in the BASELINE,
at `saurikissa` slice 7,
and the windowed run does not edit that slice at all,
so its zero there remains the lane declining to touch the passage
rather than evidence that editing it became safe.

## The rule this leaves behind, restated

Both instruments failed the same way:
a count was trusted without reading what produced it.
An instrument that reconstructs an input,
or that classifies by a hand-written list,
owes a positive control and a look at its own hits
before any verdict rests on its total.

## Validating the adjacency check against the source, and the limit it exposes

The archive is a LOOSE incumbent translation.
The source decides content.
So if the Chinese repeats across two adjacent sections,
a translation repeating it is faithful,
and a check comparing only against the archive would call that damage.

Tested on all three findings,
by character n-grams because Chinese does not split on
whitespace:

```text
baseline  lintong     repair     slices 2+3   source 8-gram overlap 0   stands
window    lintong     repair     slices 2+3   source 8-gram overlap 0   stands
window    saurikissa  translate  slices 7+8   source 8-gram overlap 0   stands
```

POSITIVE CONTROL FIRST,
because a zero from a probe that cannot report non-zero says
nothing:

```text
adjacent source pairs examined            95
pairs with non-zero 8-gram overlap         8      largest 42 shingles
```

The probe reports overlap where overlap exists,
so the zeros above are real and all three findings are genuine damage.

THE LIMIT THIS EXPOSES.
Eight of ninety-five adjacent source pairs DO repeat,
some substantially.
For those pairs a faithful translation must repeat too,
and if the archive happened not to,
the shipped check would report damage that is not there.

No such false positive has occurred:
the check fires three times across every artifact carrying a ledger,
and none of the three is one of those eight pairs.
So this is recorded rather than built against,
on the same reasoning that leaves the document-scale content gate alone:
a guard with no driving defect is speculation.

IF IT EVER FIRES,
the fix is known.
Pass the two slices' source text alongside their shipped text
and drop any repeat whose source sides overlap,
which is exactly the measurement above.

## Why the repetition checks REPORT and do not withdraw

`guardFootnoteAssembly` withdraws a replacement that would damage the footnote graph,
so the obvious next step is a guard that withdraws a slice introducing a repetition.
Measured on every finding available,
before building it,
that guard is the wrong shape.

WHICH SIDE TO WITHDRAW IS NOT FIXED:

```text
lintong     repair     slices 2+3   withdraw earlier   1 -> 0 repetitions
                                    withdraw later     1 -> 1
saurikissa  translate  slices 7+8   withdraw earlier   1 -> 1
                                    withdraw later     1 -> 0
```

A rule that always withdrew the later slice would fail on `lintong`,
and one that always withdrew the earlier would fail on `saurikissa`.
That much is solvable:
try each side and keep whichever removes the repeat.

THE COST IS NOT SOLVABLE:

```text
lintong     removing a 23-character repetition costs a 118-character translation
saurikissa  removing a 22-character repetition costs a 638-character translation
```

Withdrawing means shipping the incumbent instead,
and at `saurikissa` slice 8 the incumbent is 638 characters shorter than what the lane wrote.
Discarding a whole slice's translation to delete a five-word duplication
produces a worse document,
not a better one.

SO THE CHECKS STAY TELEMETRY.
The right place to remove an introduced repetition is the editor,
in place,
not the assembler by reverting a slice wholesale.
Recorded here so the guard is not proposed again without new evidence:
what would change the answer is a finding whose repetition is a large fraction
of the slice that introduced it,
which none of the three are.

## Could a neighbour have explained any rejection? By exact wording, no

The panel rejects a claim on relocation grounds when it judges that a neighbouring passage
holds the content the claim says is unsupported here.
That is checkable,
in one direction,
without any model.

For every REJECTED issue,
take the claim's target-side quoted wording
and ask whether it appears in either adjacent ARCHIVE slice,
matching on three-word runs:

```text
                       rejected   neighbour holds the wording
baseline saurikissa          14                             0
baseline lintong              2                             0
window   saurikissa          18                             0
window   lintong               5                            0
fixed    saurikissa          32                             0
fixed    lintong               5                            0
```

CONTROLS FIRST,
because a probe that answers the same way every time is usually broken:

```text
a slice shares three-word runs with ITSELF   10 of 11
  the miss is a two-word heading, which cannot form a three-word run
target quotes on rejected issues             34
  word counts, min / median / max             1 / 9 / 40
  shorter than the three-word window          1
```

The probe can report a match and the quotes are long enough to match with.

### What this does and does not establish

DOES:
in the windowed runs,
no rejected claim's wording is present in either neighbouring
archive passage.
So any rejection that rested on relocation rested on a passage that is
not there.
That is the same conclusion the four-critic inserted-clause case reached by
hand,
now over every rejection rather than one.

DOES NOT:
prove every rejection was wrong.
Two limits,
both real.

FIRST,
the artifact records vote tallies and not per-panelist reasons,
so which rejections invoked relocation at all is unknown.
Rejections on other grounds are counted here and should not be.

SECOND,
and more importantly,
this tests EXACT WORDING.
A neighbouring passage holding the same content in different words is exactly what a
translation produces,
and a three-word-run test cannot see it.
So the zero is a lower bound on what relocation could explain,
not a proof of none.

The baseline rows are the reminder of that:
its panel could not see the neighbours at all,
so its rejections were never relocation-grounded,
and the probe reports zero there for a reason that has nothing to do with the window.
