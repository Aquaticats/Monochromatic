# The heading aligner plan: the wiring attempt and the landing

Part of [Two features were built and never wired, and what to do about the aligner](wire-the-heading-aligner.md),
which lists every file of this record.

## The wiring attempt, and the risk the blast radius did not measure

Attempted 2026-08-13 and reverted deliberately.
The scorer itself landed as
 `alignHeadingsForced`;
what follows is about connecting it.

Replacing the proportional fallback in `alignDocumentSections` broke three
 `chunk-document.unit.test.ts` cases.
One break is correct and two are a
 warning.

THE CORRECT ONE:
a leading-kind mismatch with equal counts now PAIRS rather
 than reporting `structure-mismatch`.
That is the spurious finding measured on
 `Aniloviraw`,
whose report read "source 1 chunks,
target 1 chunks" while
 calling the structures different.
With one chunk on each side the old
 `mirrored` test could only fail on leading node kinds,
so it was reporting on
 an asymmetric preamble and calling it structure.

THE WARNING:
two cases produced ZERO pairs for the whole document.
Their
 fixtures are synthetic and their headings share no tokens across the two
 sides,
so nothing is ever forced,
the aligner refuses everything,
and under the
 ratified destination the entire document gets no critic work.

`90 of 92 identical` does not cover this,
and it is worth being precise about
 why.
That comparison ran over real corpus entries,
whose headings carry
 romanised names and therefore share tokens;
it also compared PAIRINGS rather
 than counting entries that ended with no pairing at all.
A document with
 genuinely disjoint heading vocabulary,
pure Chinese headings against pure
 English ones with no names anywhere,
is the untested case.
Its failure mode is
 the worst kind:
the entry settles,
reports findings,
and has done nothing.

So wiring needs a FLOOR before it lands.
Either the aligner forcing nothing at
 all falls back to the old path,
or the entry blocks loudly.
Silently settling
 an entry with zero pairs is the one outcome that must not be possible,
and
 choosing between those two is a decision rather than an implementation detail.

The 92-entry comparison should also be re-run reporting zero-pair entries per
 side,
which the original did not.

## Landed, and re-measured against the whole corpus rather than inherited

Wired 2026-08-13.
The zero-pair risk raised when the first attempt was reverted
 is REAL AND BOUNDED,
and this is a fresh measurement of the shipped code rather
 than the earlier 90-of-92 figure,
which was taken with a different adapter and
 never counted entries that ended with no pairing at all.

Over every corpus entry carrying both sides at the pinned commit:

```text
  entries with both sides                     92
  pairs produced                             275
  sections refused                            21
  ZERO-PAIR entries                            1   XIEPT2
  entries pairing WITH some refusal            1   XingZ60
```

So 90 of 92 entries pair completely and refuse nothing.
`XingZ60`,
the defect
 that started this,
now pairs by name and refuses only what it should.
 `XIEPT2` is the single entry that refuses everything,
and that is the case the
 record already settled:
it is a partial translation with six empty target
 bodies,
so its "correct" pairing feeds critics Chinese prose against bare
 English headings and manufactures omission claims for content nobody
 translated.

Under `doc/decision/translation-repair-always-yields-output.md`,
`XIEPT2` still
 settles,
with its translation unchanged and a finding per refused section.
It
 does no repair work,
which is the honest outcome until a translate stage
 exists,
and it never blocks.

The concern was worth raising and the answer was worth measuring:
had disjoint
 heading vocabularies been common rather than singular,
the same change would
 have quietly emptied a large part of the corpus.

## The block-count gap, recomputed on the corrected pairing

Measured 2026-08-13 on the shipped aligner,
over all 92 entries carrying both
 sides:

```text
  pairs whose source and target block counts differ    85 of 275   30.9%
```

NOT DIRECTLY COMPARABLE with the 60 of 172 in [the derivability probe and block-count gap](wire-the-heading-aligner-derivability-probe-and-block-count-gap.md),
and saying so matters more
 than the number.
Those denominators describe different populations:
172 pairs
 came from a smaller settled set under the old proportional pairing,
while 275
 come from every corpus entry under forced pairing.
Reading a drop from 34.9% to
 30.9% out of them would be comparing two different questions.

What can be said is narrower and sounder.
On pairings the aligner committed to
 only when they were forced,
roughly three in ten pairs still differ in block
 count.
That is now a statement about translation coverage rather than about
 alignment,
which is what `#69` and `#70` needed and what the old figure could
 not provide.

ONE BIAS TO CARRY FORWARD:
this population is self-selected toward confident
 pairings,
because ambiguous sections are refused rather than paired.
If
 ambiguity correlates with uneven coverage,
and it plausibly does,
the true gap
 is somewhat higher than 30.9%.
A comparable figure would restrict both
 measurements to the same entries.
