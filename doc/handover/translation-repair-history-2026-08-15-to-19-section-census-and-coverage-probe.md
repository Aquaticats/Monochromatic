# Translation repair history 2026-08-15 to 2026-08-19: the section census and the coverage probe

The section-level census that refuted the plan it was run to support,
four findings from the section review,
the body-token experiment that made alignment worse,
and the model-based coverage probe with its section-scale and block-scale results.
Its first paragraph picks up landings four and five,
and landing four's guard and grouper are recorded in `translation-repair-history-2026-08-15-to-19-block-grouper-and-the-pairing-measurement.md`.

Part of [Translation repair history: 2026-08-15 to 2026-08-19](translation-repair-history-2026-08-15-to-19.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The section census and the coverage probe

### The section-level census, which refuted the plan it was run to support

Landing five looked unblocked:
a source section with no target section is
stronger evidence than a single unpaired block,
so it could ship while landing
four waited.
That belief lasted until it was measured,
and the measurement is
the reason nothing was built on it.

WHAT THE CORPUS HOLDS:
92 entries,
of which 85 never reach the section matcher
at all,
because equal heading counts short circuit it (`#98`).
Of the seven that
do,
two produce unpaired source sections,
eleven in total.

EIGHT OF THE ELEVEN ARE FALSE.
One entry carries eight Chinese sections whose
English counterparts are plainly there under corresponding headings,
and the
matcher refused every one of them with reason `ambiguous`.
Its target side
carries a preamble chunk the source lacks,
which is `#74`'s asymmetric-preamble
finding arriving one level up.
Inserting on those eleven would have added about
seven thousand characters of duplicate translation to a document that is already
complete.
The three true ones are the tail sections of the entry `#71` is about.

THE REFUSAL REASON DOES NOT SEPARATE THEM:
all eleven say `ambiguous`,
so there
is no field a filter could read.
The matcher distinguishes PAIRED from UNPAIRED
and nothing else,
and an insertion needs a third verdict it never produces.

SO BOTH LANDINGS REST ON THE SAME MISSING THING,
and it is not a slicing
problem.
Question 28 in the decisions doc puts the four ways out,
and its
ranking changed because of this census:
the option that asks a model whether the
translation carries a passage AT ALL is the only one that does not consult the
pairing this measurement impeached.

WHAT THIS DOES NOT CHANGE:
everything landed so far stays.
The guards,
the
grouper,
the reflow rule,
the boundary value,
the delivery ledger and the
absent-incumbent lane are all correct and tested,
and none of them decides a
corpus until something wires the grouper up.

THE MEASUREMENTS ARE REPEATABLE:
`scratchpad/merge-census.mjs` for blocks and
`scratchpad/section-census.mjs` for sections,
both reading the pinned corpus and
spending no quota.
Neither prints corpus text into anything durable;
the hand
samples were read in the terminal only.

### Four findings from the section review, three fixed and one refuted

The review that blocked landing five also found defects in code that is already
running,
and they are independent of the decision it blocked.

THE CONTIGUITY CHECK COULD NOT SEE A CUT BLOCK.
It counted document nodes wholly
inside a span's range,
so a range stopping partway through a paragraph hid in
the gap between two facts:
the straddled block is not inside,
so it was not
counted,
and a span carrying nothing across half a paragraph agreed with itself.
It now reads every node the range touches and refuses a partial one by name.

THE SAME CHECK SKIPPED INSERTIONS ENTIRELY,
and the layout check cannot cover
them either:
an empty span starts where it ends,
so it never overlaps a
neighbour however wrong its offset is.
An anchor strictly inside a block would
have had assembly split that block around the inserted text.
Refused now,
and
both guards were shown to fail against the previous version.

THE FRAGMENT TRIM WAS DESCRIBED WRONGLY RATHER THAN WRITTEN WRONGLY.
Its comment
said it cut blank-line material and nothing else,
and the trailing side cuts
spaces too,
including the two that make a Markdown hard break.
Every caller
reaches it through `composeInsertion`,
which joins fragments with a blank line,
and a hard break before a blank line breaks nothing,
so the behaviour is right
and the claim was not.
The comment now names the condition that makes it safe
and a test pins both ends,
so a join that ever put two fragments on consecutive
lines fails there.

THE FLOATING-POINT CONCERN DOES NOT BITE,
and this is the one worth recording
because it was a plausible cause of a real symptom.
Lexicographic scores are
compared with strict equality while forward and backward path sums recombine the
same affinities in different orders,
so a genuinely optimal edge could fail the
comparison and manufacture an ambiguous refusal.
Measured:
comparing with a
tolerance of 1e-9 leaves all eleven unpaired source sections exactly as they
were,
and so does a tolerance of 0.05.
THE PROBE CAN SHOW A DIFFERENCE,
which is
what makes those nulls worth anything:
making every comparison return true moves
the same census from 11 unpaired source sections in 2 entries to 35 in 7.
So the refusals are the scorer's own judgement rather than numerical noise,
which strengthens rather than weakens what `#106` concluded.
The scorer was left
alone;
quantizing it can wait until something inserts on its verdicts.

### Body-token evidence for handle-free headings: measured, and it makes things worse

`#98` says the fast path can only be gated once heading scoring has a signal for
headings that share no Latin,
which is most of this corpus.
The obvious
candidate is the section BODY:
a memorial page carries names,
handles,
links and
dates that survive translation,
and a token appearing in exactly one section of
its own side identifies that section.

THE SIGNAL IS REALLY THERE.
Scoring each source section's distinctive body
tokens against each target section's pairs 其二：铃语 with Lingyu and 其四：无常
with Ann at 1.00,
which heading Latin alone cannot do,
and it leaves the two
sections the English never carried without a match.

AND FEEDING IT TO THE MATCHER MAKES ALIGNMENT WORSE.
Measured without changing
the library,
since the matcher scores Latin runs in whatever label it is handed:
appending each section's distinctive body tokens to its heading changes seven
entries and raises unpaired source sections from 11 to 18.
Two entries lose
correct pairings outright,
one of them turning a correctly paired section into a
refusal on both sides.

WHY,
and this is the part worth keeping:
with no Latin anywhere,
every pairing
scores zero,
the scorer has no preference,
and the lexicographic gap-count
component pairs by position,
which is right.
Body tokens give many pairings a
small non-zero score,
some of them spurious,
and a spurious strict row-and-column
maximum becomes a TRUSTED ANCHOR,
which outranks gap count and drags the rest of
the alignment into gaps around it.
Evidence that is weak and plentiful is worse
than none,
because the top of this scorer's order is designed to trust evidence.

WHAT WOULD HAVE TO BE TRUE for a second attempt:
body evidence entering BELOW
gap count,
as a tie-break among otherwise equal optima rather than as an anchor.
That is a fourth lexicographic component and a real change to the scorer,
and it
would fix at most one of the eight false refusals in `XIEPT2`,
whose sections
share no body tokens with their translations either.
It is not worth it.

SO THE DETERMINISTIC PATHS ARE EXHAUSTED for this corpus:
heading Latin,
section
length,
and body tokens have all now been measured,
and none of them can tell a
translated section from an absent one when the two sides share no characters.
What is left is semantic,
which is question 28's option A.

### The coverage probe corrected me, which is the strongest thing it could have done

`#106` says nothing produces a positive verdict that a passage is untranslated,
and question 28 asks what should.
Its stated default was to ask a model whether
the translation carries the passage AT ALL,
scoped to the whole translation
rather than to the neighbours an aligner chose.
That is built,
in
`coverage-wire.ts`,
`coverage-verdict.ts`,
`coverage-stage.ts`,
`coverage-candidates.ts` and the `coverage-probe` task,
and nothing calls any of
it:
no slicing,
no artifact and no lane reads a word of its output.

THE SHEET SEARCHES RATHER THAN TRANSLATES,
and every claim of coverage must
quote the English carrying the passage,
copied from what the model was shown.
A
claim whose quote is not in the document is DROPPED,
and is not counted for
absence either:
a bad quote is a voice that answered unusably,
and reading it as
agreement with "nothing carries this" would turn an invented quote into a reason
to insert text.

WHAT IT FOUND,
and it is not what I expected.
Asked about the eight unpaired
sections of `XIEPT2`,
six models answered `absent` on all eight,
near
unanimously.
I had labelled all eight CARRIED in the census,
because their
English headings plainly correspond:
经历 with Experience,
遇见 with Meeting,
and so on.
I checked after the probe disagreed with me.

THE ENGLISH DOCUMENT IS 1,218 CHARACTERS AGAINST 7,365 CHINESE.
Every section of
it except the last is a HEADING WITH NO BODY,
one block long,
seven to thirteen
characters.
The headings correspond and the translations do not exist.
My label
inferred body coverage from heading correspondence,
which is exactly the
reasoning this project keeps catching elsewhere,
and the probe caught it in me.

WHAT THAT CHANGES:
the section matcher's refusals were RIGHT IN OUTCOME on that
entry,
though for a reason it cannot state,
and the count of genuine insertion
candidates at section scale is higher than the corrected census said,
not lower.
Question 28's ranking moved with it.

AND IT EXPOSED A DESIGN DEFECT no amount of reading would have:
when a source
section's counterpart is a heading with no body,
the body belongs UNDER THAT
HEADING,
not inserted as a fresh section.
Landing five as designed inserts the
whole source section,
heading included,
at a boundary,
so on this entry it would
have produced eight duplicated headings.
The insertion unit has to be the
section BODY,
anchored after the existing target heading,
whenever a
corresponding heading is present.

### What the coverage probe measured, and the one thing anchoring does not prove

Eleven section candidates,
six voices each,
about two minutes and sixty-six
calls in total.
Verdicts against what the documents actually contain:

-   `XIEPT2`,
    eight sections:
    ABSENT on all eight,
    near unanimously.
    Correct.
    That entry's English is a set of headings with no bodies.
-   `XingZ60` section 12:
    CARRIED,
    four anchored and none absent.
    Correct,
    and I
    had it labelled missing.
    The source heading is 其九：空白,
    meaning blank,
    and
    the English heading is `### __`,
    which is the rendering.
    The matcher could
    not pair them because `__` carries no Latin run to score,
    and the bodies
    correspond underneath.
-   `XingZ60` sections 13 and 14:
    ABSENT.
    Correct;
    the English ends at the
    section that pairs with 12.

SO ELEVEN OF ELEVEN,
and my hand labels were wrong on nine of them.
That is the
result worth acting on:
the probe is not merely cheaper than reading,
it was
RIGHT where careful reading was wrong,
twice in opposite directions.

WHAT ANCHORING DOES NOT PROVE,
measured on the same run:
on section 14 two of
six voices claimed coverage and their quote WAS in the document,
a sentence
about helping people in marginalised groups that belongs to the introduction
rather than to the passage asked about.
An anchored quote proves the English
exists.
It does not prove the English renders THIS passage,
and the verdict
treats the two as the same thing.
Both voices agreed on the same irrelevant
sentence,
so agreement between voices does not separate them either.
The majority rule absorbed it here,
four to two.
It would not have on a roster
of three,
and it will not when the irrelevant sentence is the one most voices
reach for.

WHAT THAT COSTS AND WHAT WOULD FIX IT:
the failure direction is a false CARRIED,
which suppresses an insertion rather than causing one,
so it is the cheap
direction to be wrong in and nothing is at risk while nothing is wired.
The
straightforward fix is a second field:
the model names the SOURCE sentence its
quote renders,
and a verdict keeps only claims whose named source sentence is
actually in the passage asked about.
That is a wire change and a rerun,
not a
redesign.

### Block scale measured too, and it settles what landing four would have done

Twenty-two unpaired blocks,
from three entries,
six voices each:

    carried    18, most of them unanimous
    absent      1
    split       3, all with ZERO votes for absence: every voice believed the
                translation carried it and some could not quote it exactly

So at paragraph scale,
at most one of twenty-two passages the aligner refused is
a passage nobody translated.
Landing four inserts on all of them.
Corpus-wide
that is 95 refusals,
and this sample says the great majority already read
correctly in the archive.

THE SIX I HAD LABELLED BY HAND ALL AGREE with the probe,
including the three
consecutive lines one entry renders as a single English block,
the footnote
whose translation is paired with the following footnote,
and the blockquote
another entry renders as one paragraph.

HOW MUCH OF THIS IS THE MEASUREMENT AND HOW MUCH IS THE CLASS BALANCE,
which a
review asked and is the right question.
Within either set alone,
a constant
answer scores well:
always CARRIED gets 18 of 22 at block scale,
always ABSENT
gets 10 of 11 at section scale.
POOLED ACROSS BOTH,
thirty-three candidates
drawn the same way from the same corpus,
no constant answer beats 19 of 33.
The
probe agrees with every label I have,
in both directions,
which is what a
constant answer cannot do.

WHAT IS NOT ESTABLISHED,
and should not be claimed:
eleven of the thirty-three
sit in two entries and twenty-two in three,
so these are not thirty-three
independent documents.
Sixteen of the twenty-two blocks have no hand label at
all.
The labels were made by me,
after seeing the aligner's verdicts.
A real
accuracy number needs a preregistered sample,
blinded labels and a held-out set,
and `#106` says so rather than quoting a percentage.

THE VERDICT RULE WAS WRONG AND IS FIXED.
It took the majority over voices HEARD,
so one voice reporting it found nothing decided absence with five models lost
and quorum unmet,
while four fabricated quotes plus two such reports gave only
split:
silence was more dangerous than fabrication.
It now needs a majority of
every model ASKED,
an unmet quorum is inconclusive,
and partial coverage is its
own verdict rather than collapsing into carried.
