# The heading aligner plan: the finding, the options and the ranking

Part of [Two features were built and never wired, and what to do about the aligner](wire-the-heading-aligner.md),
which lists every file of this record.

## The finding

`alignHeadings` in `align-sections-order.ts` is a Needleman-Wunsch aligner that
 allows gaps on either side.
It is exported from `index.ts`,
covered by
 `align-sections-order.unit.test.ts`,
and carries the gap-placement fix from
 `110fc3909`.

It is called by nothing.
A search across the repository finds it in its own
 definition,
its own test file,
and the `index.ts` re-export,
and nowhere else.
`chunk-document.ts` does not use it.

The pipeline aligns at `repair-translation.ts`,
which calls
 `alignDocumentSections` from `chunk-document.ts`.
That function still does
 what `#71` describes:
on a structure mismatch it aligns proportionally by
 character fraction and MERGES adjacent source sections to force the counts to
 match.

So the record in `doc/handover/translation-repair-history.md` saying the section
 aligner now pairs `XingZ60` correctly is true of `alignHeadings` and false of
 the production path.
The fix was built,
tested,
committed and never connected.

## The evidence, side by side

`XingZ60` carries 14 source headings against 12 target headings,
and the two
 absent sections are at the END.

What the production path produces today,
after merging source sections 0
 through 1 and 2 through 3 into the first two pairs:

```text
  ## 参与救助          || ## Introduction
  ### 其一：伊良子       || ## Engagement in Trans Aid
  ### 其二：铃语        || ## Memories by Friends
  ### 其三：绘都        || ### Irako
  ### 其六：Mikä       || ### Ann
  ### 其七：wing       || ### Shinonome
  ### 其八：白毛 suki    || ### Mikä
```

What `alignHeadings` produces on the same input:

```text
  简介            || Introduction              affinity 0.00
  参与救助          || Engagement in Trans Aid   affinity 0.00
  友人的回忆         || Memories by Friends       affinity 0.00
  其一：伊良子        || Irako                     affinity 0.00
  其二：铃语         || Lingyu                    affinity 0.00
  其三：绘都         || HiYku                     affinity 0.00
  其四：无常         || Ann                       affinity 0.00
  其五：东云         || Shinonome                 affinity 0.00
  其六：Mikä       || Mikä                      affinity 1.00
  其七：wing       || wing                      affinity 1.00
  其八：白毛 suki    || Baimao suki               affinity 1.00
  其九：空白         || __                        affinity 0.00
  其十：锦心         || (gap)                     affinity 0.00
  致曾划过夜空的流星     || (gap)                     affinity 0.00
```

That is correct,
and it is checkable independently of the affinity scores.
Three pairs are pinned by shared Latin names at affinity 1.00,
and three more
 are confirmed transliterations that the aligner scored at zero and got right
 from order alone:
铃语 is the pinyin of Lingyu,
东云 is the Japanese reading
 Shinonome,
and 伊良子 is Irako.
The two gaps land on the two sections that
 genuinely have no translation.

The merge is the mechanism.
Collapsing two source sections into one pair shifts
 every later pair by one,
so two merges shift by two,
which is exactly the
 offset `#71` recorded.

## Blast radius, re-measured on the current build

`#71` asked for this to be re-measured,
since the earlier corpus-wide gap
 figure was computed from the same suspect pairing.

```text
  92 entries aligned, 284 pairs produced
  7 entries fall back to proportional alignment

  Aniloviraw   source 1 chunks,  target 1 chunks
  Hangmster    source 1,         target 1
  yingying     source 1,         target 1
  noname       source 4,         target 4
  interrgned   source 5,         target 5
  XIEPT2       source 8,         target 9
  XingZ60      source 15,        target 13
```

Five of the seven have EQUAL counts,
where proportional assignment lands on
 index pairing anyway.
The remaining two were inspected pair by pair:
 `XIEPT2` pairs correctly (`遇见` with Meeting,
`阴影` with Shadow,
`事故` with
 Accident,
and so on),
and only `XingZ60` is wrong.

So the blast radius is one entry,
which confirms the earlier session's count by
 a different route.
The defect is narrow but total on the entry it hits:
every
 critic call there compared the wrong original against the wrong translation.

## Options

### Option A: leave it

Pros:
 no change,
no cache invalidation,
no restart,
and the defect is confined to
 one entry of 92.

Cons:
 that entry produces confident wrong output rather than no output,
so its
 issues are noise,
its repairs damage correct text,
and its probe agrees
 because it was handed the same wrong source;
 the aligner that would fix it is already written,
tested and paid for.

### Option B: wire `alignHeadings` into `alignDocumentSections`

"Wiring" needs qualifying,
because the two functions speak different units.
`alignHeadings` returns `sourceIndex` and `targetIndex` over the HEADING arrays
 it was given,
while `alignDocumentSections` returns pairs of CHUNK objects
 carrying text ranges.
Something has to carry indices across.

That something is mechanical rather than inferential,
and it was checked rather
 than assumed.
`chunkByHeadings` starts a fresh chunk at every heading node and
 puts anything before the first heading in a preamble chunk,
so chunk count
 equals heading count plus one whenever a document does not open with a
 heading.
Measured over all 184 documents at this pin,
that correspondence
 holds EXACTLY:

```text
  184 documents, correspondence exact 184, off 0
```

So heading index `h` maps to chunk index `h + 1` when a preamble exists,
and to
 `h` when it does not.

Pros:
 reuses a function that exists,
is tested,
and carries the gap-placement fix,
 against a heading-to-chunk mapping proven exact on every document at the pin;
 handles a gap ANYWHERE in the sequence rather than only a trailing one;
 produces explicit unpaired sections,
which is what `#71` asks for when a
 section cannot be paired confidently.

Cons:
 NOT pure wiring:
the adapter must still pair the preamble chunks with each
 other and build pair objects for gap steps,
which is small but is new code
 with its own failure modes;
 unpaired sections need a destination,
and that is exactly the open question in
 `#70`,
so this cannot land without answering it;
 alignment feeds every later stage,
so it invalidates the slice cache and costs
 a pass restart.

### Option C: stop merging from the front, pair by index, leave the tail unpaired

Pros:
 smaller change than B,
confined to the mismatch branch;
 verified to produce the correct pairing for `XingZ60`.

Cons:
 hard-codes the assumption that surplus sections are trailing,
which `XingZ60`
 happens to satisfy and nothing guarantees;
 duplicates in a special case what `alignHeadings` already does in general;
 needs the same answer from `#70` about unpaired sections,
so it buys no
 simplification where it matters.

## Ranking

B > C > A.

**B over C** because both need the same decision from `#70`,
both invalidate
 the same cache,
and both have to build pair objects for unpaired sections,
so
 C's smaller diff buys less than it looks like it should.
What separates them
 is that B inherits a general gap-placement rule and a test suite,
where C
 rests on a trailing-surplus assumption that holds on the single entry it was
 checked against.

The margin is narrower than it first appeared.
B's advantage was originally
 written as "wiring rather than new logic",
which was wrong:
the heading-to-
 chunk adapter is new code,
even though the correspondence it rests on is exact
 corpus-wide.
If the adapter turns out to be more than a few lines in practice,
 C deserves a fresh look rather than deference to this ranking.

**C over A** because A is the only option that produces a confidently wrong
 pairing.
C is never worse than index pairing,
and index pairing is already
 what 86 of 92 entries get.

## What has to be settled first

-   `#70`,
    unchanged:
    an unpaired source section needs a destination.
    Wiring
    the aligner does not answer it,
    it surfaces it explicitly instead of hiding
    it behind a merge.
-   Whether to restart the running pass for this.
    Alignment feeds every later
    stage,
    so the invalidation is total rather than partial,
    which is the
    strongest case for a restart seen so far.

## A note on how this was missed

The handover records the second productive question of the previous stretch as
 whether a built feature actually FIRES,
and names typography restoration as
 the case.
The same question applied to the aligner would have caught this one:
 `110fc3909` improved gap placement,
its tests passed,
and the improvement
 reached nothing.

A committed fix with a passing test suite is not evidence that the pipeline
 uses the code it fixed.
