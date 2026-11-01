# Translation repair history 2026-08-15 to 2026-08-19: quote anchoring and the coverage reruns

The correct quote refused over a line break and the locator pass that fixed it,
the reruns of the section and block sets,
the cache invalidation nobody would have reported,
the verdict guards shown to fail,
and the second reviewer on the anchoring change.
Its first paragraph continues the coverage probe recorded in `translation-repair-history-2026-08-15-to-19-section-census-and-coverage-probe.md`,
whose split candidates it opens with.

Part of [Translation repair history: 2026-08-15 to 2026-08-19](translation-repair-history-2026-08-15-to-19.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Quote anchoring and the coverage reruns

### A correct quote was being refused over a line break, in both lanes

The three coverage candidates that came back SPLIT had something in common:
no
voice said the passage was absent.
Every one believed the translation carried
it,
and some could not point at it.
The new rows keep the quotes that failed to
anchor,
so the reason is now measurable rather than guessable.

TEN OF ELEVEN FAILED ANCHORS ARE A SOFT LINE WRAP AND NOTHING ELSE.
A model
copying a sentence out of a wrapped paragraph writes it on one line;
the
document holds the same characters with a newline in the middle;
the locator
searched byte-exact,
then with punctuation normalised,
and refused.
It then
NAMED the cause in the failure reason,
`[line-break-collapsible]`,
having
computed the collapsed match to say so.

MEASURED BEFORE CHANGING ANYTHING,
across every stored run artifact:
844
`quote-not-found` failures,
of which 45 carry that suffix.
So for critic claims
this is a small correction,
consistent with what `#72` estimated,
and for the
coverage question,
whose quotes are whole sentences rather than fragments,
it
was almost the whole failure mode.

THE FIX IS A THIRD PASS,
collapsing soft line breaks on both sides.
Both
fallbacks are length-preserving,
so offsets still index the stored document and
anchors still carry its canonical bytes.
A blank line still separates:
a
paragraph break carries two line endings where a space-joined quote carries one
space,
so nothing can be joined across a boundary the document keeps.
The
diagnostic is deleted,
because nothing can emit it now,
and the two tests that
pinned the refusal now pin the location and the ambiguity.
Shown to fail with
the pass disabled,
then restored;
the whole package suite is green.

### The eleven sections rerun under all three changes, and what the rerun can and cannot attribute

The section set was measured once before the v2 sheet,
the roster threshold and
the line-wrap pass landed,
and once after.
Nine of the eleven verdicts are
identical.
Two moved,
and only one of the two can be attributed.

    XIEPT2 sections 0 to 6   absent, both runs
    XIEPT2 section 7         absent  ->  split
    XingZ60 section 12       carried ->  partly-carried
    XingZ60 sections 13, 14  absent, both runs

WHAT `#106` RESTS ON IS UNCHANGED,
and it is worth saying plainly because the
rerun was run to try to break it:
NO CANDIDATE IN EITHER RUN REPORTS FULL
COVERAGE.
Nine of eleven are absent by a majority of the entire roster with all
six models heard,
and the two that moved both moved AWAY from coverage,
not
toward it.
The sections I had labelled as plainly translated are still reported
as carrying nothing.

SECTION 12 IS ATTRIBUTABLE,
AND IT IS NOT THE MODELS CHANGING THEIR MINDS.
The
v2 tallies are 0 full and 5 partial.
The verdict shape before this landing
counted any claim of coverage as one anchored vote,
so those same five votes
printed as `carried (anchored 5)`.
The move is the partial-from-full separation
arriving,
which is exactly what a review said it would do to this candidate,
and
`partly-carried` forbids inserting the passage whole just as `carried` did.

SECTION 7 IS NOT ATTRIBUTABLE,
and claiming otherwise would be the failure this
document exists to prevent.
The VOTES moved,
5 absent and 1 anchored in the
first run against 3 absent and 2 partial in the second,
and votes of that shape
read as `split` under either threshold rule.
So the cause is either the rewritten
sheet or ordinary run-to-run variance between two samples of six stochastic
models,
and ONE RUN CANNOT SEPARATE THEM:
the run-to-run band for this stage has
never been measured,
so a single move smaller than an unmeasured band is not
evidence of anything.

THE LINE-WRAP FIX DID NOT SHOW UP HERE,
AND WAS NOT EXPECTED TO.
Unanchored
quotes across the section set went from 4 to 6,
the wrong direction for a fix
that makes anchoring strictly easier.
That is not a contradiction:
the two runs
quote different sentences from different replies,
so the comparison is
uncontrolled,
and the wrap diagnosis was made on the BLOCK set,
where 10 of 11
unanchored quotes were soft wraps.
The controlled test is a rerun of
`mikaela_khara`,
whose three v1 splits carried 3,
3 and 4 unanchored quotes with
ZERO absent votes,
which is the wrap signature exactly.

### The locator fix invalidated the repair cache, and nothing would have reported it

Found by a reviewer reading the landing rather than by anything in the code.
`locateQuote` gained the collapsing pass,
and `critic-wire.ts` DROPS a claim it
cannot anchor.
So a critic quote copied out of a wrapped paragraph now survives
where it used to be discarded,
the surviving issue set for a slice changes,
and
with it the patch and the settled text.

WHAT MAKES IT INVISIBLE is that the cache key holds the slice texts,
the
governance flag and the run shape,
and the fix changes NONE of them.
The same
key answers differently before and after,
so a resumed corpus pass would mix
records from both generations and report nothing.

IT IS NOT THE CASE THAT LET VERSION 25 STAND.
That record could only overclaim a
change,
and `sliceRecordAgrees` catches an overclaim on resume at the cost of one
recomputed slice.
This one can differ either way and leaves no contradiction
behind:
a slice settled before the fix with a dropped wrapped quote reads exactly
like a slice where the critic found nothing.
`SLICE_CACHE_VERSION` is 27,
and the
pinned key hash moved with it.
`TRANSLATE_SLICE_CACHE_VERSION` deliberately did
not:
anchoring reaches the repair lane through `repair-stages.ts` alone,
and the
translate lane never asks a critic to quote anything.

### Both verdict guards shown to fail, which they had not been

The roster threshold and the quorum gate arrived together with a signature
change,
so the old behaviour was unreachable and neither guard had ever been
watched to fail.
One probe covers both:
compute the majority over `voices.length`
and drop the `quorumMet` gate.
Four tests fail,
three on the threshold and one on
the gate,
`'absent'` where `'inconclusive'` is required.
Restored,
rebuilt,
green.

WORTH RECORDING ABOUT THE GATE'S REACH,
since it is not obvious from reading it:
quorum needs `ceil(n / 2)` and a majority needs `floor(n / 2) + 1`,
so reaching a
majority ALWAYS implies quorum.
The gate can therefore only ever convert a
`split` into `inconclusive`,
which is the case the failing test pins,
and it can
never overturn a decided side.

### The block set rerun, which is where the wrap fix was diagnosed and where it shows

The section rerun could not speak to the line-wrap fix,
so `mikaela_khara` was
run again:
the same sixteen candidates that produced the three splits,
under the
same roster.

ALL THREE SPLITS ARE NOW CARRIED.

    pair 1 block 3   split (3 anchored, 3 unanchored)  ->  carried (4 full, 2 partial, 0 unanchored)
    pair 1 block 4   split (3 anchored, 3 unanchored)  ->  carried (5 full, 0 partial, 1 unanchored)
    pair 2 block 16  split (2 anchored, 4 unanchored)  ->  carried (5 full, 0 partial, 1 unanchored)

Unanchored quotes across the sixteen fell from 12 to 5.
That number alone does
not attribute,
because the sheet also changed and its copy-exactly rule pushes
the same direction,
and the replies are fresh samples either way.

WHAT DOES ATTRIBUTE IS INTERNAL TO THE NEW RUN,
and it is the check worth
keeping:
of the five quotes still unanchored,
ZERO are wrap-collapsible,
against
10 of 11 before the fix.
Four are English the model composed rather than copied,
which is precisely what the anchoring check exists to refuse,
and one is the
single word `September`,
which occurs twice in that document and is refused as
`ambiguous-quote (target)`.
Verified by locating it directly rather than assumed
from the classification.
So the wrap class is not merely smaller,
it is empty,
and the remaining refusals are the two failure modes that SHOULD refuse.

ONE VERDICT MOVED THE OTHER WAY,
`pair 1 block 5` from carried to
`partly-carried` at 3 full and 3 partial,
which is the same partial-from-full
separation that moved XingZ60's section 12 and not a change of votes.

WHAT THIS DOES TO ITEM 28's BLOCK-SCALE FINDING is strengthen it.
Across both
runs of this entry,
ninety-six voice answers,
NOT ONE VOTE FOR ABSENCE was cast.
Sixteen passages the block aligner refuses to pair,
and the roster says every one
of them is already carried by the translation.
Landing four would have inserted
sixteen renderings of text that is already there.

THE SINGLE-WORD QUOTE IS ALSO EVIDENCE FOR A REVIEW FINDING nobody has acted on:
the wire guard accepts any non-empty quote,
so `September` was admissible
evidence and only the locator's ambiguity check stopped it.
`#106` records the
identifying-evidence constraint as open.

### A second reviewer on the anchoring change: two real holes, two refusals, one still open

The locator change was sent for review with the four files it touches.
Six
findings came back.
What matters is that they split three ways,
and the split
was decided by reading this repository rather than by the reviewer's confidence.

TWO WERE REAL AND ARE FIXED.

The first is the one worth remembering:
ANCHORING WAS A CHAIN OF PASSES,
strict
to loose,
and each pass checked ambiguity only within its own class before
returning on its first hit.
A document holding `bad\nword` early and `bad word`
late answered the quote `bad word` with the LATE one,
unique among byte-exact
matches,
while the earlier occurrence was just as valid under the wrapping rule
the next pass would have applied.
A model normalizes whitespace and punctuation
when it copies,
so neither says which occurrence it read.
The fix judges
uniqueness over the broadest accepted form,
which REMOVES two passes rather than
adding a fourth.

MEASURED BEFORE CHANGING IT,
because a stricter rule that refuses real evidence
is a regression:
over three corpus passes,
16,479 anchored quotes checked against
the slice each was anchored in,
NOT ONE is refused by the stricter rule.
The
first version of that probe counted against whole pages and reported 566,
which
is the wrong scope,
since `repair-chunk.ts` parses the chunk pair and claims
anchor against that.
The page figure survives as the positive control:
same
counter,
same needles,
and it can see ambiguity when the scope allows it,
so the
zero is a measurement rather than a broken probe.

The second is the coverage `evidence` field,
which promised text a reader could
check against the translation and stored what the model sent.
Those read the same
until a fallback does the matching,
which is exactly when the submitted text does
not occur.
It now reads the located region back out of the document.
`unanchoredQuotes` still keeps what was sent,
since there the submitted text IS
the finding.

TWO WERE REFUTED BY THE SOURCE,
and both are worth recording because they look
right until you open the file.

CRLF was said to break offsets,
since mapping a two-unit line ending onto one
space shortens the text.
`quote-normalize.ts` maps `\r` and `\n` INDIVIDUALLY,
so
CRLF becomes two spaces and length is preserved.
A CRLF document then fails to
match a one-space quote,
which is a refusal rather than a wrong anchor.

A zero-width span carrying an empty quote was said to bypass the anchorless
guard,
since equal offsets are not inverted and the document slices to empty
there.
That shape is the INSERTION ANCHOR:
it is how an omission claim names
where missing content belongs,
and `validateIssueClaim` admits it deliberately.
A
guard against it was written,
and it broke the omission fixtures,
which is how
the design announced itself.
Reverted.

ONE IS REAL,
PARTLY FIXED,
AND THE REST IS RECORDED IN `#106`.
Collapsing every
line break turned a blank line into two spaces,
so a quote carrying two spaces
matched straight across a paragraph boundary.
The old note called that safe,
which held only while every model joined lines with exactly one space,
and a
critic quote is an untrusted input.
Matching now collapses SOLE line breaks and
leaves a run alone.
What remains unprotected needs the parse rather than the
characters:
boundaries carried by a single line break,
inside fenced code,
between list items,
between table rows,
and Markdown hard breaks.

CHECKED THAT THE NARROWING DID NOT UNDO THE RESCUE,
since a fix that refuses the
quotes it was built to accept is worse than the bug:
of 40 space-joined quotes
spanning a lone wrap taken from four corpus translations,
35 anchor,
and the 5
refusals are `quote-outside-blocks` in front matter,
which is not quotable
content.

### The section set run a third time, under unchanged code, and it reproduces exactly

The v2 run left one sentence unsupported:
that XIEPT2's section 7 moved from
`absent` to `split` for a reason a single run cannot separate from ordinary
variance between two samples of six stochastic models.
So the same eleven
candidates were run again with nothing changed.

ALL ELEVEN VERDICTS ARE IDENTICAL to v2,
section 7 included.
It splits again,
on
almost the same tallies.

WHAT THAT BUYS.
Section 7's split is REPRODUCIBLE rather than a fluke,
so the
version of the sentence that says "this might just be noise" is no longer the
honest one.
It also gives the stage a variance floor it never had:
two
independent samples of six models over eleven passages agreed on every verdict,
which is worth more than the individual results,
since every earlier number from
this stage was a single sample.

WHAT IT STILL DOES NOT SETTLE.
Attribution of the v1 to v2 move needs a run under
the OLD sheet,
which was not kept.
Low variance under v2 makes the sheet the more
likely cause than sampling,
and that is an inference from two runs of one
configuration,
not a measurement of the other.

ALSO WORTH SEEING:
three voices were lost to the sixty-second grace in this run,
and no verdict moved.
That is the roster threshold behaving as intended,
since a
majority of the whole roster was still reached and silence could not lower the
bar.
