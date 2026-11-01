# Two features were built and never wired, and what to do about the aligner

Measured 2026-08-13 against the current build,
at pin
 `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`.
This is a PROPOSAL.
Nothing here is decided,
and nothing has been landed.

## Verdict, in short

SUPERSEDED IN PART,
2026-08-13.
Read "The aligner cannot leave two headings
 unpaired,
and that breaks attempt five" before acting on anything here.
`alignHeadings` is NOT a working aligner:
pairing two headings that share
 nothing scores `0`,
while leaving both unpaired scores `-2 * GAP_PENALTY`,
so
 it can never withhold a pair on any evidence.
Attempt five's penalty is bounded
 by exactly that quantity and so cannot fix it.
The rest of this section is the
 original verdict,
kept because the unwired finding and the blast radius still
 hold.

`alignHeadings` is a working section aligner that nothing calls,
so `#71` is
 still live:
production merges source sections from the front and slides
 `XingZ60` by two.

The fix is a chunk-label adapter plus ONE new scoring term,
a penalty applied
 when exactly one side of a candidate pair is a preamble chunk,
valued strictly
 between zero and `-2 * GAP_PENALTY`.
`headingAffinity` itself stays untouched,
 which is what keeps the shared-name signal intact.

Validated across 92 corpus entries:
90 identical to production,
2 changed,
 0 regressed.
`XingZ60` becomes correct and `XIEPT2` becomes better than
 production.
Four of five invented cases pass;
the fifth,
a middle gap in a
 document sharing no evidence anywhere,
is undecidable rather than wrong.

Ranked B > C > A.
BLOCKED on `#70`:
the fix produces three unpaired sections
 corpus-wide that need a destination,
and it invalidates the slice cache
 totally.

The derivability probe is the OTHER unwired feature,
and it is blocked on
 nothing.
Wiring it is additive,
adding a third recall figure beside strict and
 lenient without changing either,
so it can be decided on its own and is the
 cheaper of the two to act on.

Everything below is supporting evidence,
including four prototype attempts
 that were wrong.
The attempt history is kept deliberately:
attempt three
 passed the entire corpus and was still wrong about its own mechanism,
which is
 the most transferable thing here.

## Contents

This record is kept as one file per subject,
each of at most 1,000 lines and named for what it holds.
The sections of the original single file moved whole and in their order.

- [The heading aligner plan: the finding, the options and the ranking](wire-the-heading-aligner-the-finding-options-and-ranking.md):
  The finding that `alignHeadings` is called by nothing,
  the `XingZ60` evidence side by side,
  the blast radius re-measured on the current build,
  the three options and their ranking,
  what has to be settled first,
  and a note on how the aligner was missed.
- [The heading aligner plan: unwired modules, the unpaired sections and the size of the change](wire-the-heading-aligner-unwired-modules-and-the-size-of-the-change.md):
  The same check run over every module,
  which found the derivability probe unwired and the unread fields of two central types,
  what the three unpaired sections actually are,
  how big the change actually is,
  and the verification notes.
- [The heading aligner plan: prototype attempts one to five](wire-the-heading-aligner-prototype-attempts-one-to-five.md):
  Attempts one to three of the heading-to-chunk adapter,
  the correction that attempt three is fragile and saturates the affinity,
  attempt four with its hard veto that is wrong about the corpus,
  and attempt five with its soft penalty,
  with what Option B then meant.
- [The heading aligner plan: the derivability probe and the block-count gap](wire-the-heading-aligner-derivability-probe-and-block-count-gap.md):
  Where the derivability probe would be wired and why it is not worth wiring at the current scale,
  and the block-count gap re-measured for `#71`.
- [The heading aligner plan: the aligner cannot leave two headings unpaired](wire-the-heading-aligner-the-unpaired-headings-defect.md):
  The arithmetic by which `alignHeadings` can never withhold a pair,
  why that breaks attempt five,
  the probe of the function itself,
  the shape a correct fix needs,
  the tests that would falsify the design,
  and the consequence for the ranking.
- [The heading aligner plan: attempt six, lexicographic scoring, its corrections and its prototype](wire-the-heading-aligner-attempt-six-lexicographic-scoring.md):
  Attempt six and its blast radius,
  the correction that the blast radius was measured against the wrong function,
  the asymmetric preamble,
  the affinity that saturates so that `TRUST` is vestigial,
  and the prototype kept in full.
- [The heading aligner plan: what the ambiguous sections cost and attempt seven](wire-the-heading-aligner-ambiguous-sections-and-attempt-seven.md):
  The cost of the ambiguous sections and the ranking of their destinations,
  attempt seven with the preamble as an empty-labelled unit,
  and why refusing `XIEPT2` costs nothing.
- [The heading aligner plan: the wiring attempt and the landing](wire-the-heading-aligner-wiring-and-landing.md):
  The wiring attempt reverted for a zero-pair risk,
  the landing re-measured against the whole corpus,
  and the block-count gap recomputed on the corrected pairing.
