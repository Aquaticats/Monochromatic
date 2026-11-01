# Translation repair open decisions: the absence vote controls of 2026-08-23

The two measurements of 2026-08-23 under the question of whether a passage was ever translated:
the control showing the coverage wire can vote absence,
and the six-entry run showing the earlier null was one entry.

Part of [Decisions waiting on you, 2026-08-15 morning, still open on 2026-08-19](translation-repair-open-decisions.md),
which lists every file of this record.

## MEASURED 2026-08-23: the absence vote is reachable, so the block-scale null is about the corpus

The unanimity recorded for `mikaela_khara`,
ninety-six answers with not one vote for absence,
had two readings and nothing separated them.
Either the translation carries every passage,
or the coverage wire's evidence rule admits any non-empty quote,
which would make an absence verdict unreachable and the count meaningless.
Both produce the same reading,
and the second would undercut everything this question leans on.

Measured both,
on that same entry,
at the current wire.

FIRST,
the verdicts were graded deterministically,
no model in the loop,
over the sixteen block-scale answers from a fresh run:
no evidence span is offered for two different blocks,
no block's evidence sits out of document order relative to its neighbours,
and no evidence is too short to be rendering its source,
the thinnest being 2.08 English characters per source character against a spread running to 6.78.
Every detector was first shown to fire on a planted case,
and the reuse detector was also shown NOT to fire on several voters quoting one span for one block.

SECOND,
and this is what settles it,
the roster was asked about a passage,
THE SPANS IT ITSELF ANCHORED ON were deleted,
and it was asked again.
Two of three cases flipped from `carried` to `absent`,
with six of six and five of six voices calling it absent where none had before.
The third moved to `partly-carried`,
which is what a passage partly rendered elsewhere should produce.

THIRD,
the same cases were cut again by the same number of characters
taken from a part of the page the roster had not pointed at.
None of the three produced a single absence vote.
The deleted region was classified structurally rather than assumed to be ordinary:
one prose line of 111 characters,
not a heading,
list,
link or footer.
So the flip follows the rendering,
not the damage.

The targeted arm returned identical verdicts on two independent runs,
and the coverage path holds no cache,
so that agreement was bought rather than replayed.

WHAT THIS DOES AND DOES NOT LICENSE.
It establishes that the wire can say no,
which is what question 28's premise needed.
It does not establish that the corpus carries everything:
that is one entry and three damaged cases,
enough to settle reachability and not enough to make a claim about ninety-three pages.
The wiring itself stays where the owner left it,
`groupSourceFirst` unwired and `alignDocumentSections` emitting no insertion pairs,
because what the pipeline should DO with a reachable absence verdict is a separate decision.

Runner:
`mise run //package/module/translation-repair:coverage-control-probe -- --only <entry>` (moved on 2026-10-18 to
`//package/translation-repair/coverage-census:coverage-control-probe`).

## AND THE NULL ITSELF WAS ONE ENTRY, measured the same day

Ran the same control over six entries,
picked off a zero-quota census
that found 26 of the 92 pairs carry coverage candidates at all.

Three entries HELD,
meaning damage was detected and only the right damage:
`mikaela_khara`,
`TianqiChen666` and `Futajuhuacha` gave absence votes on 8 of 9 targeted cuts
and on 0 of 9 equally sized cuts taken where the roster had not pointed.

On the other three the control had NOTHING TO DAMAGE,
because the roster refused to call
a single candidate covered before anything was cut:

-   `shi_Yumiaoya`,
    8 of 8 candidates:
    seven `absent` with 4 to 6 of 6 voices,
    one `split`.
-   `Aniloviraw`,
    3 of 3:
    all `absent`,
    5 to 6 voices.
-   `XingZ60`,
    4 of 4:
    section 13 `absent` with 5 voices,
    the rest `partly-carried`.

Fifteen of fifteen undamaged candidates were not called covered,
and not one refusal anywhere in the run was an anchoring failure.
`XingZ60` also reproduced its recorded section-scale verdicts,
section 12 `partly-carried` and section 13 `absent`.

SO THE SENTENCE THIS QUESTION LEANS ON,
ninety-six answers with not one vote for absence,
IS A FACT ABOUT `mikaela_khara`.
It is not a fact about the wire,
which votes absence near-unanimously on other pages,
and it is not a fact about the corpus.
The reading that follows from it,
at most one of twenty-two and plausibly none,
inherits the same limit and needs re-deriving over a spread before it can carry
the weight the paragraph-scale refusal puts on it.

NOT ESTABLISHED:
whether those absence verdicts are CORRECT.
They are consistent with genuine omissions rather than merges,
since a merge puts the content somewhere and that is what `carried` reports while `absent`
claims it is nowhere,
but that is an argument and not a measurement.
Settling it means reading those passages against their source.

Still nothing wired,
and still the owner's call.
