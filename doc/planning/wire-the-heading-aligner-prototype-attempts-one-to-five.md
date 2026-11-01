# The heading aligner plan: prototype attempts one to five

Part of [Two features were built and never wired, and what to do about the aligner](wire-the-heading-aligner.md),
which lists every file of this record.

## Prototyped, and four of the five attempts were wrong

The ranking
(in [the finding, options and ranking](wire-the-heading-aligner-the-finding-options-and-ranking.md))
left one thing open:
whether the heading-to-chunk adapter is
 a few lines or enough code to change the answer.
It was built against the
 already-exported `chunkByHeadings` and `alignHeadings`,
with no package change
 and no rebuild,
and compared to `alignDocumentSections` across all 92 entries.

Comparing by chunk INDEX is useless here and was the first mistake:
a merged
 pair reports its first chunk's index,
so the merge that is the whole defect is
 invisible.
Every result below compares the first non-blank LINE of each side.

### Attempt one: pair preamble with preamble, then offset by one

Fixes `XingZ60`,
breaks FIVE entries.
When one side has a preamble and the
 other does not,
the separate preamble rule shifts everything after it,
so
 `XIEPT2`,
`interrgned` and `noname` all slide by one,
and `Hangmster` and
 `yingying` have a correct single pair split into two gaps.

### Attempt two: give the preamble an empty label and let the aligner place it

Fixes `XingZ60`,
breaks `XIEPT2`.
Better,
and still wrong,
for a reason worth
 keeping:
`headingAffinity` scores only shared Latin tokens of three or more
 characters,
so between a Chinese heading and its English translation the
 affinity is ZERO.
With every score zero the alignment is decided entirely by
 the gap-placement tiebreak,
and `110fc3909` put gaps at the end.
`XingZ60`
 needs its gap at the end.
`XIEPT2` needs one at the front,
because the English
 carries a `(To-Do)` preamble the Chinese has no counterpart for.

A tiebreak cannot be right for both.
The aligner needs a signal.

### Attempt three: let the label carry the structure

Heading DEPTH and preamble-ness are free,
deterministic,
and already present.
Feeding them in as a token the existing affinity can see:

```ts
const DEPTH_TOKEN = ['hdga', 'hdgb', 'hdgc', 'hdgd', 'hdge', 'hdgf',];
const PREAMBLE_TOKEN = 'pream';

// One label per chunk: a preamble chunk cannot look like a heading chunk, and
// two headings agree on structure only at the same depth.
function chunkLabels({ document, },) {
  return chunkByHeadings({ document, },).map(function toLabel(chunk,) {
    const first = firstNonBlankLine({ text: chunk.text, },);
    const depth = headingDepth({ text: first, },);
    return (depth === null)
      ? PREAMBLE_TOKEN
      : `${DEPTH_TOKEN[depth - 1]} ${headingTextOf({ line: first, },)}`;
  },);
}
```

Result over all 92 entries:
90 identical to production,
2 changed,
0 regressed.

```text
  XIEPT2
    production  ## 经历=(To-Do) | ## 遇见=## Meeting | ## 阴影=## Shadow | ...
    prototype   x=(To-Do) | ## 经历=## Experience | ## 遇见=## Meeting | ...

  XingZ60
    production  ### 其一：伊良子=## Engagement in Trans | ### 其三：绘都=### Irako | ...
    prototype   ### 其一：伊良子=### Irako | ### 其二：铃语=### Lingyu | ...
```

`XingZ60` becomes correct.
`XIEPT2` becomes MORE correct than production:
the
 English `(To-Do)` preamble is left unpaired instead of absorbing `经历`,
and
 `经历` pairs with `Experience` where it belongs.
Three unpaired sections are
 produced corpus-wide,
all of them real.

### What this changes about the options

Option B is confirmed as the right direction and is more than wiring,
in a way
 that is now measured rather than guessed:
it needs the chunk-label adapter
 above AND a structural component in the affinity.
Both are small,
and the
 combination is a strict improvement on every entry at this pin.

The structural token as prototyped rides inside the label string,
which works
 because affinity is shared-token based,
but is a hack.
A real change should
 give `headingAffinity` the depth explicitly rather than smuggling it through
 text.

Option C is now clearly worse than it looked.
Its trailing-surplus assumption
 is exactly what attempt two implemented,
and `XIEPT2` is the counterexample
 sitting in the corpus:
a gap that belongs at the FRONT.

So the ranking stands at B > C > A,
and the margin is wider than the earlier
 correction suggested,
not narrower.
The blocker is unchanged:
three unpaired
 sections still need the destination `#70` owes them.

## Correction: attempt three is fragile, and the "strict improvement" claim was wrong

Attempt three was tested only against the corpus,
which contains gaps at the
 front and at the end and nowhere else.
Run against invented cases covering
 positions the corpus does not have,
it fails:

```text
  gap at the END                          correct
  gap at the FRONT                        correct
  gap in the MIDDLE                       WRONG, gap slid to the end
  middle gap WITH a shared Latin name     WRONG, and it ignored the name:
                                          paired 老猫 with Mocha and left
                                          the real Mocha unpaired
  heading depth changed in translation    correct
```

The cause is the structural token itself.
`headingAffinity` divides shared
 tokens by the smaller token count,
so adding a depth token to every label
 saturates the score:

```text
  WITH the depth token          WITHOUT it
  hdgb 小猫  vs hdgb Kitten  1   小猫   vs Kitten  0
  hdgb 老猫  vs hdgb Mocha   1   老猫   vs Mocha   0
  hdgb Mocha vs hdgb Mocha  1   Mocha vs Mocha   1
  hdgb 老猫  vs hdgb Kitten  1   老猫   vs Kitten  0
```

Every same-depth pair scores 1.00,
so a genuine name match is worth exactly as
 much as an unrelated heading at the same depth.
The signal the aligner was
 supposed to gain is destroyed by the mechanism meant to deliver it.

Without the token the same middle-gap case aligns correctly,
gapping the middle
 source section and pinning the tail on the real name match at affinity 1.00.

So attempt three's corpus wins were not earned.
`XingZ60` needs its gap at the
 end,
which is where the tiebreak already puts it,
and `XIEPT2` came out right
 only because saturation happened to make heading-to-heading pairing beat
 heading-to-preamble pairing.
Neither depended on structure being informative.

### Why this cannot be fixed by feeding labels

`XIEPT2` needs the aligner to refuse to pair a preamble chunk with a heading
 chunk.
Affinity returns a value between zero and one,
and a zero-affinity
 PAIRING still scores better than a gap,
which costs `GAP_PENALTY`.
So a label
 can never express "these must not pair",
only "these are unrelated",
which is
 what every cross-language heading pair already scores.

Saturating the good pairs to 1.00 is the only way to express the preference
 through labels,
and that is precisely what destroys the name signal.

The real change is therefore to `headingAffinity` or to the aligner:
structural
 incompatibility has to be expressible,
as a mismatch penalty or a hard
 constraint,
separately from textual similarity.

### What this does to the options

Option B is still the right direction and is now clearly NOT small.
It needs
 the chunk-label adapter,
plus a structural term in the scoring that does not
 ride on the shared-token count.
That is a design change to the affinity
 function,
not wiring.

The ranking B > C > A is unchanged,
because C is still the trailing-surplus
 assumption that `XIEPT2` refutes and A is still confidently wrong on
 `XingZ60`.
What changes is the cost of B,
which should not be presented as
 cheap.

The lesson is the one this document keeps repeating in a different form.
A
 change measured only against the corpus is measured against the cases the
 corpus happens to contain,
and a prototype that passes there can still be
 wrong about the mechanism it claims to use.

## Attempt five resolves it: structure as a soft penalty, not a veto

Two more attempts,
and the fourth is the answer.

### Attempt four, a hard veto, is wrong about the corpus

Refusing outright to pair a preamble chunk with a heading chunk fixes `XingZ60`
 and `XIEPT2` and REGRESSES four entries:
`Hangmster`,
`interrgned`,
`noname`
 and `yingying`.

All four have the same shape,
and it is a shape the corpus has every right to:
 one side keeps content in a preamble where the other gives it a heading.
On
 `interrgned` the Chinese opens with prose about tangled threads and the
 English titles that section `## Line`.
Pairing them is CORRECT.

So "preamble and heading are structurally incompatible" is simply false here.
A translation may add or drop a heading for the same content.

### Attempt five, a soft penalty, threads every case

The distinction is not that the pairing is forbidden,
it is that a better
 global alignment may exist.
That is what a penalty expresses and a veto
 cannot.

The working range follows from the algorithm rather than from tuning.
Pairing
 across a structure boundary costs the penalty;
refusing costs two gaps,
one on
 each side.
So the penalty must sit strictly between zero and
 `-2 * GAP_PENALTY`:

-   above `-0.7`,
    `Hangmster` still pairs its lone heading with the lone
    English preamble instead of dropping both;
-   below zero,
    `XIEPT2` prefers gapping its `(To-Do)` preamble and matching
    all eight headings straight across.

Swept from `-0.1` to `-0.69`,
every value gives the same corpus result:

```text
  92 entries: 90 identical to production, 2 changed, 0 regressed
    XingZ60  now correct
    XIEPT2   now better than production: the (To-Do) preamble is left
             unpaired and 经历 pairs with Experience
```

And on the invented cases that broke attempt three:

```text
  gap at the END                          correct
  gap at the FRONT                        correct
  gap in the MIDDLE, no evidence          slides to the end, see below
  gap in the MIDDLE, name pins the tail   CORRECT: 老猫 unpaired, Mocha pinned
  heading depth changed in translation    correct, still paired in order
```

The name signal survives because heading-to-heading scoring is untouched.
Only
 the structure boundary carries the new term,
so an exact shared name still
 outscores everything around it.

### The one case that stays unsolved, and why that is acceptable

A gap in the middle of a document whose headings share NO evidence cannot be
 placed.
Every arrangement scores identically,
so the tiebreak decides and puts
 it at the end.
That is not a defect of this design;
it is the absence of
 information.
Placing it correctly would need a signal nobody has yet,
such as
 comparing section bodies rather than headings.

Worth stating plainly because the earlier claim that Option B "handles a gap
 ANYWHERE" is still too strong.
It handles a gap anywhere EVIDENCE reaches,
and
 falls back to the end-of-document prior otherwise.

### What Option B now means, concretely

-   A chunk-label adapter,
    since `alignHeadings` speaks heading indices and
    `alignDocumentSections` speaks chunk objects.
    The correspondence is exact
    on all 184 documents.
-   One new term in the scoring:
    a penalty between zero and `-2 * GAP_PENALTY`
    when exactly one side of a candidate pair is a preamble chunk.
-   No change to `headingAffinity` itself,
    which is what keeps the name signal
    intact.

That is small,
it is bounded,
and it is now measured against 92 real entries
 plus five invented ones rather than argued for.
The blocker is unchanged:
 three unpaired sections corpus-wide still need the destination `#70` owes
 them.
