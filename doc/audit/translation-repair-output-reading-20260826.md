# Reading the pipeline's actual output (2026-08-26)

Opened on the owner's rejection of the readiness signal:
"Not yet.
You didn't even look at its actual output."
Every gate that signal rested on was a process gate.
This document is the reading of what the pipeline produces,
page by page,
against the Chinese source and the
archive English at the pinned corpus commit `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`.
It joins `doc/audit/eight-entries-read-against-the-original.md` (2026-08-20,
an older build).
It quotes a rendering only where the wording itself is the defect,
under the owner's standing decision that
corpus text in this repository is sanitized once at the end rather than avoided.

## Method

A page is read three ways:
the source `page.md`,
the archive `page.en.md`,
and the published
`<runs dir>/fixed/people/<entry>/page.en.md`,
sentence by sentence.
For each change the pipeline made,
the question is whether the change is right against the source;
for each sentence it kept,
whether keeping it inherited an archive error.
Rendering is read as a reader sees it:
grammar,
tense,
terminology within the page,
punctuation characters,
links,
front matter untouched.

Two rules for the fresh pass,
added at the checkpoint before it:

-   EVERY DEFECT IS TRACED INTO THE ARTIFACT before it is filed.
    The page cannot say whether a defect was
    inherited or introduced,
    nor which lane shipped the slice;
    the settled artifact records the recipe,
    the
    lane,
    and the ballots.
    A defect is filed as "slice N,
    lane L,
    ballots said X",
    which names a mechanism,
    not as "the page says Y",
    which names a symptom.
-   A SECOND READER.
    Each fresh page also goes to sol with the three whole files (source,
    archive,
    published)
    and one focused question,
    because this project's own record says agent grading drifts.
    Disagreements
    between the two readings are findings in their own right.

## Pages read so far, all from builds older than the current one

The newest published pages on disk come from single-entry verification runs of 2026-08-22 and 2026-08-24.
No multi-entry pass has run on the current build;
that pass is the next section.

### `gaoyanger` (2026-08-24, generation 4 build)

One paragraph changed.
The rendering of "lovely,
tender and kind" is closer to the source than the archive's
"pretty",
and the second sentence now says what the source says (a single-parent family,
poor living conditions,
always strong) where the archive had paraphrased it into being "raised in poverty".
Publishable;
an improvement.

### `Weideriche_` (2026-08-24 build)

The pronoun fabrication the 2026-08-20 reading found is gone:
the page now says she was happy to be called sister,
which is what the source says.
Three defects remain:

-   NAME.
    The first sentence names her by the handle `Weideriche_` where the source prose uses the alias (the
    front matter declares both,
    `name: Weideriche_` and `alias: Zihe`).
    The archive used the alias.
    A memorial
    sentence opening with a handle and a trailing underscore is a reader-facing regression,
    whatever rule about
    declared names produced it.
-   AMBIGUITY.
    伙伴 in the opening sentence became "partner",
    which in English reads as a romantic partner;
    the
    archive's "friend" was right.
    The same word later in the page ("with her partner") is the archive's own
    rendering and is left as it was.
-   TYPOGRAPHY.
    "non-binary" was written with U+2011 NON-BREAKING HYPHEN rather than the ASCII hyphen the archive
    used.
    Measured across every published page on disk (14 pages):
    the characters pages introduce relative to the
    archive are U+2019 (4,
    in two pages) and U+2011 (1,
    this page).
    Nothing normalizes punctuation at assembly.
    MEASURED CORPUS-WIDE at the pin,
    over all 92 archive `page.en.md` files,
    before deciding what that means:
    85
    pages carry typographic quotes (33 only typographic,
    52 mixed with straight,
    4 straight only,
    3 neither),
    with
    1173 U+2019,
    525 U+201C,
    528 U+201D,
    228 U+2014 and 86 U+2026 in total;
    U+2011 occurs 11 times and U+2013 7.
    So a U+2019 the pipeline writes is the archive's own majority convention,
    not a foreign character,
    and the
    earlier wording "the archive never used" was true of the pages read and false of the corpus.
    What remains a
    defect is the narrower class:
    a character whose difference from its plain counterpart a reader cannot see
    (U+2011 against the hyphen,
    and by the same reasoning U+00A0 and U+00AD) written where neither source nor
    archive page has it.
    That class is a normalization,
    not a house-style question for the owner.
    LANDED IN SOURCE (`8e8b7bd6e`,
    unbuilt while the arms hold `dist`):
    `foldInvisibleVariants` in
    `invisible-variants.ts` folds U+2011,
    U+00A0,
    U+202F,
    U+00AD,
    U+200B,
    U+2060 and U+FEFF at the point where
    each lane turns an answer into a candidate (edit and refine operations,
    translate and consolidate
    candidates),
    before any decider judges,
    and records `invisible-variant-folded (<code point> xN)` as a
    finding.
    Typographic quotes,
    dashes,
    the ellipsis and the emoji joiner pass through.

Everything else on the page is faithful and in places more faithful than the archive (the added "it could be very
painful" in the quote is gone).

### `wangzihao980` (2026-08-22 build)

-   The opening quotes are rendered more faithfully than the archive (the second one keeps its question).
-   The sentence about the friend's statement kept the archive's wording and with it the archive's loss:
    the source
    links to the statement and says the friend logged into her account;
    the page has neither.
    A source hyperlink
    absent from the output is mechanically checkable and nothing checks it.
-   The page-only translated note (a transcription of an image the source shows) is kept,
    as the rule requires.
-   The closing sentence is now faithful ("whether you know her or not").

### `dogesir_` (2026-08-22 build)

-   GRAMMAR.
    The opening sentence is a comma splice ("a very cute sister,
    her cuteness is evident just from how she
    speaks");
    the archive's sentence was grammatical.
    The pipeline shipped worse English than it inherited.
-   TENSE.
    "She is also a sister with an exceptional talent" beside "She was also a lover of Minecraft" on the same
    page,
    about someone who has died.
    The 2026-08-20 reading recorded tense drift;
    `#152` made the English the
    authority for tense afterwards;
    whether the current build still drifts is for the fresh pass.
-   TERMINOLOGY.
    "the galactic train" and "the Galaxy Train" on one page,
    beside the quoted title *Night on the
    Galactic Railroad*.
-   The link to the companion's profile,
    which the archive dropped,
    is restored from the source.

## Defect classes so far

1.  English quality regressions:
    comma splices,
    present tense for the dead.
2.  In-page terminology inconsistency.
3.  Name rendering:
    handle where the source uses the alias.
4.  Lexical ambiguity introduced by a change (伙伴 as "partner").
5.  Invisible-variant punctuation introduced by models (U+2011 for the hyphen) with no normalization;
    U+2019 is
    the archive's own majority convention and stays.
6.  Source hyperlinks lost by inheriting an archive sentence.

## The fresh pass on the current build

The pass began on 2026-08-27 as matched overlap arms under the protocol in
`doc/planning/translation-repair-corpus-overlap-measurement.md`.
This lets the output-reading entries also answer the corpus overlap question without sharing artifacts,
published pages or slice cache between arms.
The hard cases remain `Toka_ls`,
whose editor once fabricated three lines;
`XIEPT2`,
whose pairing once collapsed to zero slices;
and a separate `XingZ60`,
whose sections once slid by two.
Eight short entries alone would answer whether the output is publishable on easy inputs,
which is not the owner's question.
Each published page is read the same way,
and `verify-published`,
the rendering audit and the damage probe run on the same pages
so their verdicts can be compared with the reading.

### `keyword233` on the current build, overlap-4 smoke arm

The arm settled two slices and published a complete page.
`verify-published` found one artifact,
one page,
every promised wording present,
expected page length and no missing source address.
The code-point census found no watched punctuation introduced.

The first reading and the independent sol reading agree that the page is publishable as-is.
It repairs the archive's invented timeframe,
ungrammatical plural of laughter,
dropped opening of the Telegram channel,
dropped heartfelt reflections,
dropped narrator opinion and ungrammatical closing.
Names,
pronouns,
date,
location and both links are right.
There is one minor introduced nuance:
`在我看来很棒` is rendered in past tense as `which I thought was wonderful`,
where `which I find wonderful` would preserve the source's present viewpoint
without the possible implication that the narrator changed their mind.

ARTIFACT TRACE.
This is slice 1.
The repair lane introduced the wording while repairing a larger region.
The lane contest settled on repair with 6 repair votes,
1 translate vote and 1 refusal.
All 10 consolidation slate verdicts were valid;
the final consolidation retained the phrase,
and its gate accepted the consolidation by 9 votes to 1.
The translate lane independently produced the same past-tense phrase.
This is an editorial-choice finding,
not an overlap mechanism finding:
ordering,
slice seams,
assembly and publication are intact,
and a two-slice stochastic pair cannot attribute wording to concurrency.

### `Toka_ls` on the current build, overlap-4 decision arm

Not publishable.
The arm settled 15 slices in 104.37 minutes over 11.82 hours of calls,
normalized `0.147`,
with both providers wet.
`verify-published` passed one artifact and one page,
but that process gate checked only that page carried what artifact promised.
Artifact promised one silent source gap.

BLOCKER,
inherited and knowingly left standing:
the entire factual paragraph after final quoted letter is absent.
The page omits that she wrote linked final letter on afternoon of October 9,
2024;
died in Shanghai at 4 p.m. after unsuccessful resuscitation for hemorrhagic shock;
and was 26.
It also loses source link.
These are central memorial facts,
so page cannot ship.

ARTIFACT TRACE.
Source-only slice 13 carries that exact paragraph and target placement with no incumbent.
Repair records `not-applicable`;
translation records `unfilled`,
reason `not-corroborated`,
with no model findings because no translation round was bought.
Both delivery ledgers say `gap-remains` and ship empty text.
Whole-page shortfall gate saw ordinary aggregate English length,
because verbose passages elsewhere masked local omission,
and treated likely merge as certainty.
That premise is false on this page.

Other findings from first and independent sol readings:

-   Source front matter designates Chinese alias while page front matter keeps `Nonamev`;
    source body supports that name,
    so this is minor metadata displacement rather than fabrication.
-   Poem shifts from past opening to present `brings`,
    and contains several inherited or revised
    unnatural phrases:
    companion `of the mortal world`,
    wings `from then on`,
    `narrow and long Big Dipper`,
    `dynamic musical notes` and `no wind by her ears`.
-   `life may be temporarily absent` adds uncertainty;
    `sprout and bloom` adds blooming.
-   Parallel poetic sequence mixes period and semicolon endings without source reason.
-   Biography says `a neat regulated meter is left behind`,
    which is conspicuously unnatural for writing regulated verse.
-   Final letter retains conspicuously unnatural English:
    remaining in memories,
    spending unknown time,
    being squeezed out,
    hearing everyone's thank you and working together `so much` rather than for so long.
-   One inherited U+FEFF ZERO WIDTH NO-BREAK SPACE remains.
    Archive had three and changed slices happened to remove two;
    candidate-only normalization could not touch unchanged archive bytes.

FIXED AFTER READING.
The corpus pass now refuses every unfilled translation before contest,
artifact and publication.
Production insertion admission now requires whole-document coverage verdict `absent`
plus either page shortfall or destination missing from target.
The destination comparison reuses package Markdown and bare-address readers.
Archive English is folded before preparation,
so deciders,
artifact and page share visible bytes.
Commits run from `c151e57ca` through `598401349`;
focused tests,
lint and guard-failure proofs are green.
A link-free local omission hidden by aggregate length remains unrepairable,
but now fails entry rather than publishing silently.
Current ten-model coverage roster passed live absence control on 2026-08-27.
Two targeted cuts flipped from `carried` to `absent` with 10 and 9 votes,
third moved to `partly-carried`,
and three equal-size decoys drew zero absence votes.
Both providers stayed wet.

### `Toka_ls` on fixed build, overlap 4 and 180-second grace

Not publishable under strict actual-output gate,
although omission blocker is fixed.
Run settled all 15 slices in 114.72 minutes over 14.49 call-hours,
normalized `0.132`,
with 72 voices unheard and both providers wet.
`verify-published` found 15 wordings,
zero silent,
expected length and zero missing.
Source destination count is one and page carries same one.
U+FEFF fell from archive three to page zero.
Restored final paragraph preserves date,
time,
place,
cause,
age and linked final note.

MAJOR,
inherited and left standing:
biography shifts from third person to first person in sentence about inspiration and regulated verse.
Source continues implicit third-person subject;
archive inserted first person.
Repair lane corrected it to third person;
translate lane retained archive.
Contest heard 8 of 10 and split 4 repair to 4 translate,
while 5 ballots independently called archive flawed.
Tie settled neither and therefore retained archive.
Contest artifact marks archive `declined`.
Consolidation then records `no-standing-text` and buys no third rendering,
even though final assembly revives archive as actual floor.
Slice 11 is stronger same contradiction:
all 10 contest ballots call archive flawed,
8 choose neither,
and consolidation still buys nothing before assembly restores archive.
This is not overlap ordering defect,
but known-flawed fallback is not semantic approval.
It exposes unresolved seam between contest standing text and publication fallback.

MINOR,
introduced:
letter opening lost trailing tilde source and archive both carry.
Other inherited awkwardness remains;
restored paragraph itself is faithful and complete.
An independent reviewer first called page publishable against regression-only baseline,
then strict-readiness review corrected verdict:
inherited identifies provenance,
not acceptability.

### `Toka_ls` on fixed build, overlap 4 and 300-second grace

Publishable as-is under strict actual-output gate.
Run settled 15 slices in 127.43 minutes over 15.15 call-hours,
normalized `0.140`,
with 35 voices unheard and both providers wet.
This recovered 37 voices against 180-second arm,
while wall rose 11.1 percent and normalized wall rose from `0.132` to `0.140`.
Different live ballots produced fewer streams and different issue inventory,
so one pair does not attribute those changes to grace.

First-person major is gone:
biography now uses passive third-person-compatible rendering for regulated verse.
Letter retains trailing tilde.
Final paragraph preserves date,
time,
place,
age and link.
It generalizes hemorrhagic shock to failed lifesaving efforts,
which follows primary corpus policy at pinned `CODE_OF_CONDUCT.md:14`:
suicide method detail is deliberately softened.
Independent reread found no blocker and no major.
Remaining findings are minor:
inherited poem tense and punctuation irregularity,
three English renderings of final-note term,
and source-supported alias choice at signature.

Declined-archive seam remains despite publishable page.
Slices 3 and 11 ended `settled-neither` with archive `declined`;
slice 11 had 9 of 10 archive-flawed ballots.
Consolidation bought nothing at both,
then assembly restored archive.
Favourable output does not repair structural path.
One 300-second observation also does not decide default grace.

## Tooling for the fresh pass, and the second reader's dry run

Two helpers in the session scratchpad (`page-read.py`,
`sol-read.py`),
owed as a package CLI (`#268`) so the
method survives the session:

-   `page-read.py <entry> <runs-dir>` writes the source,
    the archive and the published page beside a unified
    diff of archive against published and a census of the watched code points (typographic and invisible
    punctuation,
    archive count to published count,
    introduced ones marked),
    plus a bare-address count on both
    sides.
    Dry run on the older `wangzihao980` page:
    4 diff hunks,
    U+2019 9 to 11,
    no invisible variant,
    1 of 3
    source addresses absent from the page,
    which is the `#265` finding the reading made by hand.
-   `sol-read.py <entry> <runs-dir>` attaches the three whole files with `@file` arguments and asks sol for a
    numbered list of every place the page departs from the source,
    with the source words,
    whether the archive
    had the same defect,
    and a severity,
    ending with one sentence on publishability.
    Launched in the
    background,
    never polled.
    Dry run on the same page launched 16:52Z;
    it answered by 17:02Z,
    and is read
    against the first reading in the next section.

The pass itself:
`corpus-pass --only` over the ten entries into `~/temp/agent/fresh-read-20260826`,
then
`XingZ60` alone into the same directory,
at production defaults;
the launch lines are in the scratchpad
(`launch-fresh-pass.txt`) and in the corpus pass runbook.

## The second reader on the older `wangzihao980` page, against the first

sol returned 15 items on the same 2026-08-22 page this document's own reading found two remarks on.
Counted
by its own labels:
1 inherited blocker,
5 inherited majors,
5 inherited minors,
1 introduced major beside two
introduced findings it rated major and minor,
and one closing sentence:
not publishable as it stands.

WHAT THE FIRST READING MISSED,
all of it inherited from the archive and left standing by that build:

-   The front matter's `desc` field changes the tense of the source sentence,
    drops its opening word and adds
    a clause the source does not have.
    The first reading never looked at the front matter.
-   The day of death:
    the source places it in the early hours of THAT day,
    the page says the next day.
    The
    second reader rated this the page's one blocker.
    The first reading did not check dates against the
    source.
-   The friend's statement sentence:
    the first reading caught the lost hyperlink (`#265`) and missed that the
    same sentence also loses that the friend logged into the account and misstates what was published.
-   The English of the translated note (a page-only transcription of an image the source shows):
    six items
    of unnatural or wrong English,
    including a preposition wrong twice and right once on the same page,
    a
    garbled temporal clause that appears twice,
    and a point-of-view error in how the officiant is to refer to
    her.

WHERE THE TWO DISAGREE:
the closing sentence.
The first reading called it faithful (it is;
the archive had
dropped the clause);
the second reader called its English conspicuously unnatural and introduced.
Both are
right,
and the second is the one a reader of the page meets.
Faithfulness alone is not the reading.

WHAT THE SECOND READER ADDED THAT IS NEW AS A CLASS:
a restored link shipped inside a malformed sentence with
a determiner missing and a terminology split ("last words" beside "suicide note") the archive had kept
consistent.
That is class 2 (in-page terminology) and class 1 (English regressions) arriving together with
a repair,
which no gate reads.

WHAT THIS CHANGES FOR THE FRESH READING:

-   The front matter is part of the page.
    `desc` is translated prose and the pipeline never touches front
    matter (`#269`);
    every fresh page is read from its first line.
-   Dates,
    times,
    days and relationships are checked against the source as facts,
    not as wording.
-   Naturalness is graded beside faithfulness on every sentence.
-   Inherited defects are recorded even though the pipeline did not cause them,
    because the owner's question
    is whether the page is publishable,
    and the second reader's answer on a page carrying them was no.
