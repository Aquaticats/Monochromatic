# Translation repair open decisions: the transcribed-image class (question 2)

Question 2 on blockquotes that transcribe letters held as images in the Chinese pages,
the overnight census of target-only blocks,
the options,
and the ruling that built the two-reader image reading and refused the paired-quote ratio guard.

Part of [Decisions waiting on you, 2026-08-15 morning, still open on 2026-08-19](translation-repair-open-decisions.md),
which lists every file of this record.

## Question 2: the transcribed-image class

BLOCKS nothing mechanically,
and is the largest known quality risk in the new
shape.
The class is now enumerated rather than estimated:
8 blockquotes over 6
entries,
15299 characters,
sitting inside a wider target-only population of 132
blocks and 44731 characters that also holds translator apparatus and alignment
slop.

WHICH NUMBER COUNTS WHAT,
since three have been in circulation and they are not
the same population.
44731 is every block the translation carries that no
source block partnered,
apparatus and slop included.
16249 is the blockquote
part of it.
15299 is the 8 blockquotes over 1000 characters,
which is the
transcription class itself.
The handover's older "roughly 31 thousand
characters,
6 entries verified" reproduces from none of these,
so it should not
be carried forward:
it appears to have counted whole English blockquotes in the
named entries rather than the part with no Chinese counterpart.

Chinese pages hold letters and documents as IMAGES.
English pages transcribe
and translate them.
So the English carries text with no counterpart in the
Chinese markdown at all.

Under the repair shape this was safe:
nothing asked a model to produce the
English from scratch,
so the transcription simply survived.
Under the translate shape it is in danger twice over.
A translator working from
the source has no source for it,
so its candidate omits it.
A judge comparing
candidates against the source cannot tell that omission from a correct one,
and
the structural validator cannot either,
since the source genuinely has no
footnote,
link or block for that text.

Your standing ruling is that accurate translator additions are kept.
Nothing in
the lane yet makes that happen.

### Measured overnight, and it changes the options

Four measurements,
all from the pinned corpus,
none spending quota.

A CORRECTION FIRST.
An earlier draft of this section said the image is not in
the markdown at all,
and drew conclusions from it.
That was wrong,
and it was
wrong because the search was:
it looked for Markdown image syntax and for the
string `img`.
The corpus writes images as an MDX component instead,
and
`<PhotoScroll ...>` appears in 50 of 92 entries,
matching on both sides in 49
of them.
The one exception is `shi_Yumiaoya`,
whose Chinese page carries a
photo block the English page does not.
Everything the earlier draft concluded
from the absence is struck.

WHAT THE 132 TARGET-ONLY BLOCKS ARE MADE OF.
This is the measurement the
options actually turn on,
and no earlier draft had it:

    blockquote            17 blocks   16249 chars
    paragraph             87 blocks   14370 chars
    mdxFlowExpression      7 blocks   11179 chars
    footnoteDefinition    16 blocks    2746 chars
    list                   1 block       160 chars
    heading                2 blocks       21 chars
    thematicBreak          2 blocks        6 chars

Three populations,
not one.
The blockquotes are the transcriptions.
The
`mdxFlowExpression` blocks are translator apparatus:
the largest single
target-only block in the corpus,
10737 characters in `shihai4h`,
is a
commented-out block of localization notes,
and the footnote definitions are the
citation apparatus the translator added to go with them.
The 87 paragraphs are
mostly ordinary alignment slop,
a sentence moved or a paragraph split,
at 165
characters each on average.

THE TRANSCRIPTIONS ARE EXACTLY THE BIG TARGET-ONLY BLOCKQUOTES,
and they match
the entries `doc/planning/translation-pipeline-redesign.md` named by hand:

    Zha_Ke          3625     zheermao101   2115 + 1071
    Mio      2052 + 1882     MizuharaNagisa      1969
    dogesir_        1487     wangzihao980        1098

That is 8 blocks over 6 entries,
and it answers the question the earlier draft
could not:
the class does land in the target-only population,
so a structural
rule can reach it.

ONE TRANSCRIPTION IS NOT TARGET-ONLY,
and it is the reason a size test alone is
not enough.
Across the corpus there are 210 aligned blockquote pairs,
and their
growth band is narrow:
p50 2.71,
p90 3.68.
Exactly one pair sits outside it,
`shihai4h` at 102 characters against 1665,
a ratio of 16.3.
That is a letter
transcribed INTO a quote the Chinese also has,
so it is aligned as an ordinary
pair and no target-only rule will ever see it.
It is one block in one entry,
and the ratio band says a paired quote over five times its source is the test
that finds it without catching ordinary growth.

### Options

A.
 Protect the class structurally:
keep every target-only block out of
    translation and splice it back unchanged,
and add the paired-quote ratio
    guard for the one merged case.
    Pros:
the transcription cannot be lost by any model decision;
cheap;
no
    model has to be told anything;
now measurable rather than hypothetical,
    since the population it protects is enumerated above.
    Cons:
rests on the aligner,
the component with the worst track record in
    this pipeline,
and the aligner is being rebuilt under `#74`.
It also
    protects the 87 alignment-slop paragraphs,
which simply are not
    retranslated;
at 165 characters each that is a small amount of text left
    exactly as it stands.

B.
 Supply the image to the translators and judges,
so the text has a source.
    Pros:
the only option where the translation of that text can be CHECKED
    rather than preserved;
would also catch a bad existing transcription;
and
    the component naming the image file is right there in the markdown,
so
    finding the asset is a path resolution rather than a new corpus reader.
    Cons:
needs image transport and models that read images,
which this
    provider roster may not have;
and OCR of handwritten Chinese letters is its
    own failure surface.

C.
 Licence it as evidence:
pass the incumbent's target-only blocks to the
    judges as "verified additions the archive keeps",
without asking anyone to
    reproduce them.
    Pros:
no new machinery;
judges stop reading the omission as correct.
    Cons:
tells the judges what to believe rather than letting them check,
and
    the incumbent's additions are exactly what nobody has verified.

RANKING:
A > C > B,
unchanged by the measurements,
though A is now a smaller
and better-specified piece of work than when it was ranked.

A over C because A cannot be talked out of by a model,
while C depends on every
judge weighing an instruction the same way.
C over B because B needs capabilities the roster may not have,
and its OCR
failure mode replaces a known-good human transcription with a machine guess,
which is the one outcome worse than losing it.

WHAT I WOULD DO WITHOUT AN ANSWER:
build A for target-only blocks,
since it is
structural and needs no decision from you,
and leave the paired-quote guard
alone until you have ruled,
because that one is a threshold and thresholds are
yours to set.

B,
but best effort,
fallback to A whenever an image's OCR doesn't make sense.

RULED,
AND HALF-REFUSED ON MEASUREMENT.
B is built:
two blind readers per
picture,
compared on character trigrams,
with an uncorroborated reading refused
rather than caveated.
A's first half,
keeping target-only runs out of
translation,
is built and wired into `translate-slice.ts`.

A's second half,
the paired-quote ratio guard,
is REFUSED.
Measured over all 211
aligned blockquote pairs it fires nine times:
once on `shihai4h/14`,
which two
guards already hold,
and eight times on ordinary text,
five of those on source
quotes of 10 to 16 characters where a ratio is arithmetic on almost nothing.
It
never reaches the slice it was meant for.
`shihai4h/3` carries no blockquote on
either side and sits at ratio 4.1,
so both the shape test and the threshold miss
it.
The reading covers that slice instead:
it names `photo3.webp`,
18550 bytes,
inside both readers' allowances.
Evidence in
`doc/audit/reading-a-picture-at-the-user-boundary.md`.
