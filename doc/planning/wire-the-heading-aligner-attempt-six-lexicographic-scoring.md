# The heading aligner plan: attempt six, lexicographic scoring, its corrections and its prototype

Part of [Two features were built and never wired, and what to do about the aligner](wire-the-heading-aligner.md),
which lists every file of this record.

## Attempt six: lexicographic scoring, prototyped and validated

Built 2026-08-13 after the structural defect was found.
This is the first
 attempt that satisfies every case,
including the two that defeated attempts
 three and five.

### The algorithm

Three changes to `alignHeadings`,
none of them to `headingAffinity`.

FIRST,
the score becomes a lexicographic triple instead of one clamped scalar:

```text
  [ trustedAnchorAffinity   maximised
    gapCount                minimised
    softAffinity            maximised ]
```

A transition contributes `[trusted ? affinity : 0, 0, affinity]` when pairing
 and `[0, 1, 0]` when gapping.
Gaps never contribute negative heading evidence,
 which is what makes an affinity of `0` mean "no information" rather than
 "mismatch".

SECOND,
a TRUSTED anchor is a candidate at or above a threshold that is also the
 STRICT maximum of both its row and its column.
Strictness on both axes is what
 stops a name repeated across several headings from anchoring anything.

THIRD,
and this is the part that satisfies `#71`,
a pairing is emitted only when
 it is FORCED.
Run the table forwards and backwards,
then a transition lies on
 some optimal path exactly when `forward[cell] + transition + backward[next]`
 equals the optimal score.
A pair is emitted only when it is the sole partner
 for its source across every optimal path,
the sole partner for its target,
and
 neither side could instead be a gap.
Everything else returns as `ambiguous`.

The return type becomes a discriminated union of `paired`,
`source-only` and
 `target-only` rather than indices with `-1` sentinels.

### The cases that killed the earlier attempts

-   MIDDLE OMISSION with anchors on both sides.
    Anchors hold,
    and the
    zero-evidence middle returns ambiguous instead of sliding.
    Attempt five
    shifted it.
-   LONG DISPLACEMENT,
    an anchor needing three reciprocal gaps on each side.
    The anchor survives.
    Under the old scalar those six gaps cost `2.10` and
    would have crushed an affinity of `1.00`.
-   ALL-ZERO UNEQUAL sequences.
    No confident pair anywhere,
    where the current
    aligner confidently pairs by traceback order.
-   REPEATED NAME across several headings.
    Ties break strictness,
    so it never
    becomes a hard anchor.
-   EQUAL LENGTH with zero evidence anywhere.
    Still pairs by position,
    which is
    what keeps the corpus from moving.

### Blast radius, measured over all 92 entries

```text
  identical pairings to the current aligner   89
  changed                                      3
  entries with an ambiguous section            3, holding 20 sections
```

The three that change are exactly `XingZ60`,
`interrgned` and `noname`,
all of
 which already emit an alignment finding today.
Not one of the 85 cleanly
 aligned entries moves.

On `XingZ60` every essay section pairs correctly.
The three romanised-name
 anchors score `1.00` and pin the alignment;
the zero-affinity pairs ahead of
 them follow by monotonicity;
the untranslated tail sections and the one
 unmatched target heading come back ambiguous rather than being absorbed.
That
 is the outcome `#71` asked for.

### The consequence that still needs a decision

`interrgned` and `noname` go from 4 and 3 pairs to ZERO.
Both have unequal
 heading counts with no shared-name evidence anywhere,
so nothing is forced and
 everything is honestly ambiguous.

That is the stated requirement working as designed,
since `#71` holds that a
 wrong pairing is worse than no pairing.
It is also a coverage LOSS:
those two
 entries get critic work today,
wrongly paired,
and would get none.

So the blocker named earlier has grown rather than gone.
It is no longer three
 unpaired sections needing a destination,
it is 20 sections across 3 entries,
 and two entire entries that the pipeline would decline to process.
Whether
 ambiguous sections should fall back to positional pairing,
route to a
 translate stage,
or simply be reported is the decision that gates landing this,
 and it belongs with `#70` because option B answers it differently from A and C.

## Correction: the blast radius was measured against the wrong function again

The attempt-six section reported "89 of 92 identical to the current aligner".
That compared `lexAlign` against `alignHeadings`,
which production never calls.
It is the same conflation withdrawn in [the unpaired headings defect](wire-the-heading-aligner-the-unpaired-headings-defect.md),
made again one
 section later,
and it is withdrawn too.

Re-measured against `alignDocumentSections`,
the production path,
mapping
 heading indices to chunk indices:

```text
  identical pairing   86
  changed              6
  pairs removed 20, pairs added 8

  Hangmster    prod  1 -> lex  0
  XIEPT2       prod  8 -> lex  8   but all 8 differ
  XingZ60      prod 13 -> lex 12
  interrgned   prod  5 -> lex  0
  noname       prod  4 -> lex  0
  yingying     prod  1 -> lex  0
```

### And that measurement is ALSO mostly an artifact, which is the real finding

Five of those six changes are not disagreements about which sections match.
They are my adapter mishandling an ASYMMETRIC PREAMBLE:
one side has content
 before its first heading and the other does not,
so chunk index and heading
 index differ by one on one side only.

Counted across the corpus:

```text
  both sides have a preamble chunk    35
  neither side has one                52
  asymmetric, one side only            5
```

The five asymmetric entries are `Hangmster`,
`XIEPT2`,
`interrgned`,
`noname`
 and `yingying`.
Those are EXACTLY the five non-`XingZ60` movers,
name for name.

That is the headline,
and it reframes `#71`.
Five of the seven entries that fall
 back to proportional alignment do so because of an asymmetric preamble,
not
 because a section is missing.
`XingZ60` is the only entry whose fallback is
 caused by a genuine heading gap,
and `Aniloviraw` is neither.

So the aligner work splits in two,
and the halves are different sizes:

-   The SCORING fix,
    attempt six,
    addresses `XingZ60`.
    One entry.
-   The ADAPTER,
    deciding what a preamble on one side pairs with when the other
    side has none,
    addresses five.
    It is the larger half and this document has
    consistently treated it as the mechanical part.

`Hangmster` shows why it is not mechanical.
Its Chinese side is a single
 heading-led chunk and its English side is a single preamble chunk.
They are
 plainly the same content and production pairs them,
which is right.
A rule that
 pairs preamble only with preamble would refuse that pairing and lose the entry.

## The affinity function saturates, so `TRUST` is vestigial

Probed directly:

```text
  'alpha beta' vs 'alpha beta'          1.00
  'alpha beta' vs 'alpha'               1.00
  'alpha'      vs 'alpha beta'          1.00
  'alpha'      vs 'alpha gamma delta'   1.00
```

`headingAffinity` divides shared tokens by the SMALLER token count,
so a
 single-token heading scores `1.00` against any heading containing that token,
 however long.
There is no gradation above zero to threshold on.

Consequences for attempt six:

-   `TRUST = 0.5` never binds.
    Every candidate with any shared token clears it.
    UNIQUENESS on both axes is carrying the entire design,
    alone.
-   A bare romanised name can anchor the whole document on one coincidence,
    and
    it does so at full confidence.
    Probed:
    a lone `Mochi` heading anchors to a
    `Mochi and friends` heading and forces every other section to gap.
-   The repeated-name case passes for the right reason but by a narrow margin.
    When the same token appears in several headings the affinities TIE at
    `1.00`,
    strictness fails on both axes,
    and everything returns ambiguous.
    That is safe,
    but it is uniqueness doing the work,
    not the threshold.

Whether to fix this in `headingAffinity`,
by dividing by the LARGER token count
 so partial overlap scores below 1,
is a separate decision.
It would change what
 every existing test asserts,
and this document's rule has been that
 `headingAffinity` stays untouched.

MEASURED,
rather than left as an open hazard:
saturation causes no corpus-wide
 harm at this pin.
A false single-token anchor would pull an entry away from
 production's pairing,
and under attempt seven only `XIEPT2` and `XingZ60`
 differ,
both for reasons that are understood and neither involving a spurious
 anchor.
So the hazard is real in principle and absent in this corpus,
which
 makes it a reason to keep the uniqueness rule strict rather than a reason to
 change the affinity function now.

## The prototype, inlined so it survives

Kept verbatim rather than described,
because this session's repeated lesson is
 that subtle scoring bugs survive 92-entry validation.

```js
/** Affinity at or above which a unique candidate may anchor. */
const TRUST = 0.5;

/** Lexicographic compare: trusted desc, gaps asc, soft desc. */
function better(a, b) {
  if (a[0] !== b[0]) return a[0] - b[0];
  if (a[1] !== b[1]) return b[1] - a[1];
  return a[2] - b[2];
}

const add = (a, b) => [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
const eq = (a, b) => a[0] === b[0] && a[1] === b[1] && a[2] === b[2];
const NEG = [-Infinity, Infinity, -Infinity];

export function lexAlign({ sourceHeadings, targetHeadings, }) {
  const n = sourceHeadings.length, m = targetHeadings.length;

  const A = Array.from({ length: n, }, (_, i) => Array.from({ length: m, }, (_, j) =>
    headingAffinity({ source: sourceHeadings[i], target: targetHeadings[j], },)));

  // Trusted: at or above threshold AND the strict maximum of both its row and
  // its column, so a name repeated across headings never anchors.
  const trusted = Array.from({ length: n, }, () => Array(m,).fill(false,));
  for (let i = 0; i < n; i += 1) {
    for (let j = 0; j < m; j += 1) {
      if (A[i][j] < TRUST) continue;
      const rowMax = A[i].every((v, k) => k === j || v < A[i][j]);
      const colMax = A.every((row, k) => k === i || row[j] < A[i][j]);
      trusted[i][j] = rowMax && colMax;
    }
  }

  const pairCost = (i, j) => [trusted[i][j] ? A[i][j] : 0, 0, A[i][j]];
  const GAP = [0, 1, 0];

  const F = Array.from({ length: n + 1, }, () => Array(m + 1,).fill(NEG,));
  F[0][0] = [0, 0, 0];
  for (let i = 0; i <= n; i += 1) {
    for (let j = 0; j <= m; j += 1) {
      if (i === 0 && j === 0) continue;
      let best = NEG;
      if (i > 0 && j > 0) {
        const c = add(F[i - 1][j - 1], pairCost(i - 1, j - 1,),);
        if (better(c, best,) > 0) best = c;
      }
      if (i > 0) { const c = add(F[i - 1][j], GAP,); if (better(c, best,) > 0) best = c; }
      if (j > 0) { const c = add(F[i][j - 1], GAP,); if (better(c, best,) > 0) best = c; }
      F[i][j] = best;
    }
  }

  const B = Array.from({ length: n + 1, }, () => Array(m + 1,).fill(NEG,));
  B[n][m] = [0, 0, 0];
  for (let i = n; i >= 0; i -= 1) {
    for (let j = m; j >= 0; j -= 1) {
      if (i === n && j === m) continue;
      let best = NEG;
      if (i < n && j < m) {
        const c = add(pairCost(i, j,), B[i + 1][j + 1],);
        if (better(c, best,) > 0) best = c;
      }
      if (i < n) { const c = add(GAP, B[i + 1][j],); if (better(c, best,) > 0) best = c; }
      if (j < m) { const c = add(GAP, B[i][j + 1],); if (better(c, best,) > 0) best = c; }
      B[i][j] = best;
    }
  }

  const optimal = F[n][m];

  // A transition lies on SOME optimal path when forward + it + backward equals
  // the optimum. A pair is emitted only when it is forced on EVERY such path.
  const partnersOfSource = Array.from({ length: n, }, () => new Set(),);
  const partnersOfTarget = Array.from({ length: m, }, () => new Set(),);
  const sourceCanGap = Array(n,).fill(false,);
  const targetCanGap = Array(m,).fill(false,);
  for (let i = 0; i <= n; i += 1) {
    for (let j = 0; j <= m; j += 1) {
      if (eq(F[i][j], NEG,)) continue;
      if (i < n && j < m && eq(add(add(F[i][j], pairCost(i, j,),), B[i + 1][j + 1],), optimal,)) {
        partnersOfSource[i].add(j,); partnersOfTarget[j].add(i,);
      }
      if (i < n && eq(add(add(F[i][j], GAP,), B[i + 1][j],), optimal,)) sourceCanGap[i] = true;
      if (j < m && eq(add(add(F[i][j], GAP,), B[i][j + 1],), optimal,)) targetCanGap[j] = true;
    }
  }

  const steps = [];
  for (let i = 0; i < n; i += 1) {
    const p = partnersOfSource[i];
    if (p.size === 1 && !sourceCanGap[i]) {
      const j = [...p,][0];
      if (partnersOfTarget[j].size === 1 && !targetCanGap[j]) {
        steps.push({ kind: 'paired', sourceIndex: i, targetIndex: j, affinity: A[i][j], },);
        continue;
      }
    }
    steps.push({ kind: 'source-only', sourceIndex: i, reason: p.size === 0 ? 'forced-gap' : 'ambiguous', },);
  }
  for (let j = 0; j < m; j += 1) {
    if (steps.some((s) => s.kind === 'paired' && s.targetIndex === j)) continue;
    steps.push({ kind: 'target-only', targetIndex: j, reason: partnersOfTarget[j].size === 0 ? 'forced-gap' : 'ambiguous', },);
  }
  return steps;
}
```
