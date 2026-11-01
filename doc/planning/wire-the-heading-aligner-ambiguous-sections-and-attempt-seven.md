# The heading aligner plan: what the ambiguous sections cost and attempt seven

Part of [Two features were built and never wired, and what to do about the aligner](wire-the-heading-aligner.md),
which lists every file of this record.

## What the ambiguous sections actually cost

The blocker has been stated as "unpaired sections need a destination" without a
 size attached,
which made it sound larger than it is.
Measured:

```text
  XingZ60       4 ambiguous, 11 paired,  5269 source chars
  interrgned    9 ambiguous,  0 paired,  4101 source chars
  noname        7 ambiguous,  0 paired,  1897 source chars

  entries with any ambiguity      3 of 92
  entries reduced to ZERO pairs   2
  ambiguous sections             20
  source characters they hold  11267
  corpus target characters    405915
  share of the corpus at stake  2.78%
```

So the destination decision governs 2.78% of the corpus,
concentrated in three
 entries,
two of which the pipeline would decline entirely.

That is small enough to change how blocked this is.
The three candidate
 destinations can be ranked on their own merits rather than deferred to `#70`:

-   REPORT ONLY,
    doing no critic work on an ambiguous section.
    Pros:
    never
    manufactures a false issue,
    which is `#71`'s stated requirement,
    and is the
    honest answer when the evidence genuinely does not determine a pairing.
    Cons:
    two entries get no processing at all,
    and a reader seeing an empty
    result cannot tell it from a clean one without reading findings.
-   ROUTE TO A TRANSLATE STAGE,
    treating an unpairable source section as
    content needing translation rather than repair.
    Pros:
    matches the decided
    goal,
    that the pipeline yields a good translation even when the input does
    not make sense,
    and turns the two zeroed entries into useful work.
    Cons:
    only coherent under a design that HAS a translate stage,
    so it presupposes
    the `#70` outcome rather than standing alone.
-   POSITIONAL FALLBACK,
    pairing by index when nothing is forced.
    Pros:
    keeps
    today's coverage exactly,
    changes nothing for the 89 unaffected entries.
    Cons:
    it is what production already does,
    and it is what produced `XingZ60`,
    so it reintroduces the defect being fixed on precisely the sections where
    the evidence is weakest.

Ranking:
ROUTE > REPORT > POSITIONAL.

ROUTE over REPORT because the goal document already decided that absent or
 broken input should still yield a translation,
and REPORT leaves two entries
 unserved when a translate stage would serve them.
The gap between them is
 conditional rather than absolute:
REPORT wins outright if no translate stage
 is built,
which is why this ranking is contingent on `#70` and the earlier
 blocking was not wrong,
only oversized.

REPORT over POSITIONAL because POSITIONAL reintroduces exactly the defect under
 repair.
It guesses hardest where evidence is weakest,
and `#71` recorded that a
 wrong pairing is worse than no pairing because it manufactures issues rather
 than skipping work.

## Attempt seven: the preamble as an empty-labelled unit, which now works

Prototyped 2026-08-13 after the asymmetric-preamble finding.
This SUPERSEDES
 the blast-radius figures in the correction of [attempt six](wire-the-heading-aligner-attempt-six-lexicographic-scoring.md),
which were measured with a
 naive offset adapter.

### The adapter

Build the alignable unit list as `(preamble ? [''] : []) ++ headings`.

That makes UNIT INDEX EQUAL CHUNK INDEX by construction,
because
 `chunkByHeadings` emits the preamble as chunk 0 and then one chunk per
 heading.
There is no offset arithmetic left to get wrong,
and offset
 arithmetic is exactly what the previous adapter got wrong.

The preamble carries an EMPTY label.
`headingAffinity` returns `0` against
 everything for it,
including against another empty label,
and it returns a
 finite `0` rather than dividing by a zero token count,
which was checked
 rather than assumed.

This is attempt TWO of [attempts one to five](wire-the-heading-aligner-prototype-attempts-one-to-five.md),
which failed under scoring
 that could not withhold a pairing.
Attempt six can,
and that is what makes the
 same adapter work now.

### Blast radius, against production

```text
  identical pairing to alignDocumentSections   90
  changed                                       2
  entries reduced to zero pairs                 1
  entries with ambiguity                        2, holding 21 units

  XIEPT2    units  8v9    prod  8 -> lex  0    17 ambiguous
  XingZ60   units 15v13   prod 13 -> lex 12     4 ambiguous
```

`Hangmster`,
`yingying`,
`interrgned` and `noname` now match production
 EXACTLY,
where the naive adapter reported all four as changed.
That is the
 adapter defect gone rather than a scoring change.

The reason is worth stating because it validates the design.
`interrgned` and
 `noname` have UNEQUAL heading counts,
5v4 and 4v3,
which is why aligning on
 headings alone refused everything.
Their CHUNK counts are equal,
5v5 and 4v4,
 because the preamble makes up the difference.
With equal unit counts and no
 evidence anywhere,
the aligner pairs by position,
which is what production
 does,
so they agree.

`Hangmster` is the case that a preamble-pairs-only-with-preamble rule would
 have lost:
one heading-led Chinese chunk against one preamble English chunk.
As units they are `['## ...']` against `['']`,
one against one,
so the pairing
 is forced and it is correct.

### What it still refuses, and whether that is right

`XIEPT2` is the whole remaining cost:
8 source units against 9 target units,
 zero shared Latin tokens anywhere,
so nothing is forced and all 17 units come
 back ambiguous.
Production pairs its 8 sections and,
per the inspection
 recorded in [the finding, options and ranking](wire-the-heading-aligner-the-finding-options-and-ranking.md),
pairs them CORRECTLY.

So the aligner refuses 8 pairings a human can verify are right.
The affinity
 function cannot see it,
because the headings are translations rather than
 shared names,
and the aligner is honest about that.

Whether the refusal is a loss depends on what `XIEPT2` is.
It is a PARTIAL
 TRANSLATION:
6 of its sections have an empty target body.
Pairing it
 "correctly" therefore hands the critics Chinese prose against bare English
 headings,
which produces omission claims for content that genuinely was never
 translated.
That is the work `#69` decided should be a TRANSLATION rather than
 a repair.

Read that way,
refusing to pair `XIEPT2` and routing it to a translate stage is
 the right outcome rather than a regression,
and it argues for ROUTE over
 REPORT in the destination ranking.

### Revised cost of the destination decision

The earlier sizing said 3 entries,
20 sections,
2 entries zeroed.
With the
 preamble adapter it is 2 entries,
21 units,
and only `XIEPT2` reduced to zero.
`XingZ60` keeps 12 of its 13 pairs and loses only the one that was wrong.

### `XIEPT2` settled: refusing it costs nothing

The cost of attempt seven was stated as "refuses 8 pairings production gets
 right",
hedged with "arguably".
The hedge is removable,
and this document has
 said two things about `XIEPT2` that needed reconciling:
that it "pairs
 correctly",
and that its figures are "not quotable".
Both are true and they
 are about different questions.

PRODUCTION PAIRS IT CORRECTLY.
Dumped pair by pair,
all eight are genuine
 translations of each other:
经历 with Experience,
遇见 with Meeting,
阴影 with
 Shadow,
事故 with Accident,
送行 with Farewell,
未来？ with Future?,
梦醒 with
 Fact,
and the postscript with Postscript.
The `sections-merged` finding folded
 the target preamble into pair 0,
and everything after it lines up.

AND EVERY ONE OF THOSE PAIRS IS EMPTY ON THE ENGLISH SIDE:

```text
  pair 0  经历      src 1600 ch / 17 blocks    tgt    7 ch / 1 block
  pair 1  遇见      src 1114 ch / 12 blocks    tgt    0 ch / 0 blocks
  pair 2  阴影      src 1160 ch / 23 blocks    tgt    0 ch / 0 blocks
  pair 3  事故      src  983 ch / 17 blocks    tgt    0 ch / 0 blocks
  pair 4  送行      src  576 ch / 14 blocks    tgt    0 ch / 0 blocks
  pair 5  未来？    src  592 ch / 13 blocks    tgt    0 ch / 0 blocks
  pair 6  梦醒      src  489 ch /  8 blocks    tgt    0 ch / 0 blocks
  pair 7  后记      src  479 ch / 17 blocks    tgt   75 ch / 1 block
```

The two pairs that technically carry target text carry 7 characters,
which is a
 to-do marker,
and 75,
which is a contributor credit.
The entry is 6994 source
 characters against 82 target characters.

So `XIEPT2` is not a translation with alignment trouble.
It is an UNTRANSLATED
 document with headings.
Every one of its eight correct pairings would hand the
 critics Chinese prose against nothing and collect omission claims for content
 that was never written,
which is exactly the work `#69` decided should be a
 translation rather than a repair.

REFUSING IT THEREFORE COSTS NOTHING.
The coverage the aligner gives up is
 coverage that produces noise.

That removes the last hedge from attempts six and seven together:

-   90 of 92 entries pair identically to production.
-   `XingZ60` keeps 12 of 13 pairs and loses the one that was wrong.
-   `XIEPT2` is refused,
    and it holds 82 characters of English.

The destination decision still stands,
because with no translate stage
 `XIEPT2` gets nothing rather than getting translated.
But the decision is now
 about ONE untranslated document,
not about losing correct work,
and it argues
 ROUTE over REPORT more strongly than before.
