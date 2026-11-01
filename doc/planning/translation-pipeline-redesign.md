# Producing a translation rather than repairing one

Written 2026-08-13 for `#70`.
This is a PROPOSAL.
Nothing here is decided,
and it lives in `doc/planning/`
for that reason.

The goal it serves is decided:
 `doc/decision/translation-repair-output-goal.md` records that the pipeline
 yields a good translation of the ORIGINAL,
 even where the translation fed in does not make sense.

## Why the shape that exists cannot reach that goal

Every stage is defect-driven.
Critics find defects in the input translation,
 a panel adjudicates them,
 an editor rewrites the accepted ones through a deterministic apply gate,
 checkers confirm the rewrites.
The unit of work is a defect,
and a defect is a claim ABOUT EXISTING TEXT.

A passage with no translation has no existing text to make a claim about.
It can only enter as an omission filed against text that is not there,
 which is what the corpus already shows happening.

## What the corpus actually contains

`XingZ60`,
thirteen aligned section pairs,
rebuilt with the corrected parser:

```text
pair  3    62 blocks / 4641 chars    1 block  /    22 chars
pair 10    76 blocks / 3483 chars    5 blocks /   719 chars
pair  6     7 blocks /  763 chars    2 blocks /   105 chars
pair  7    18 blocks /  613 chars   20 blocks /  9551 chars
pair  5    20 blocks / 2908 chars   62 blocks / 14080 chars
```

Pair 3's entire English side is the string `## Memories by Friends`.
A heading,
with 4641 characters of memorial essay untranslated beneath it.
The current pipeline treats that as a translation with defects in it:
the
 critics compare a long original against a heading,
file omission after
 omission,
the panel accepts them,
and the editor writes English one accepted
 issue at a time,
with no stage aware it is translating rather than repairing.

Corpus-wide,
60 of 172 aligned pairs differ in block count.

The gap runs BOTH ways,
and that is the part earlier framings missed.
Pair 7 holds 613 characters of original against 9551 of translation.
Five to fifteen times is not English being wordier than Chinese,
so either those
 sections carry content the original does not,
or section alignment is pairing
 the wrong ones.
Any proposal that only answers "translate what is missing" answers pairs 3,
10
 and 6,
and says nothing about pairs 7 and 5.

### Pairs 7 and 5 are mispairing, and the real asymmetry is transcribed images

Settled 2026-08-13 by rebuilding the pairing outside the production fallback.
This section CORRECTS the paragraph before it,
and its own first version,
which
 claimed no entry carries unsupported content.
That claim was wrong.
What is
 true is narrower and more useful.

Pairs 7 and 5 do not exist under correct pairing.
Both are products of the
 proportional-by-character fallback that `XingZ60` triggers,
recorded as `#71`.
Pairing `XingZ60`'s sections by index instead leaves one real coverage gap,
 76 blocks against 14,
and two untranslated tail sections of 915 and 1459
 characters.
The five-to-fifteen ratio is an artifact of the aligner rather than
 a property of the translation.

So for pairs 7 and 5 the branch resolves to MISPAIRED.

#### Block counts, partitioned by whether the aligner trusted itself

Counting BODY blocks only,
since `nodes` includes the heading node and that +1
 on both sides compresses every ratio.
Partitioned on whether the entry emitted
 an alignment finding,
because a mispaired entry's block counts describe the
 aligner rather than the translation:

```text
                                  no finding    with finding
  entries                             85              7
  both-sides pairs                   251             33
  differ in body-block count          75             25
  target >= 1.5x source               10              6
  target >= 2x   source                2              4
  target >= 3x   source                0              2
  source >= 3x   target                0              4
```

Every extreme pair in the corpus,
in BOTH directions,
sits in one of the seven
 entries that emitted an alignment finding:
`Aniloviraw`,
`Hangmster`,
`XIEPT2`,
 `XingZ60`,
`interrgned`,
`noname`,
`yingying`.
Among the 85 cleanly-aligned
 entries the maximum is 2x,
reached twice.

The bias runs the helpful way.
Mispairing MANUFACTURES lopsided pairs,
which is
 visible in `XingZ60`'s production output pairing 20 blocks against 62.
Including
 the mispaired entries should therefore inflate the extreme counts,
and the
 cleanly-aligned population still returns zero.

`XIEPT2` is BOTH partial and mispaired,
so its individual figures are not
 quotable even though its empty sections are real:
target chunking does not
 depend on pairing,
but which source section each empty target sits against does.

#### Untranslated sections, counted properly

A section whose target body is empty is the untranslated-section signal,
and a
 ratio test cannot see it because the denominator is zero.
There are 10,
in
 three entries:
`XIEPT2` 6,
`shi_Yumiaoya` 3,
`XingZ60` 1.

`shi_Yumiaoya` emits no alignment finding,
so it is a clean partial translation.
Its English page carries headings with no bodies beneath them,
`## Experience`
 followed immediately by `## Departure`,
confirmed in the raw bytes.
`parseDocument` reads them correctly as `kind: "heading"` nodes;
the sections
 are empty because the translator left them empty.
That is the case `#69`
 already decided:
the pipeline must yield a good translation even when the
 translation fed in does not make sense.

#### The translation does carry content the source markdown lacks

Block counts are blind to this,
which is why the first version of this section
 missed it.
A section can hold one block that balloons.

`Zha_Ke` is one section,
4 body blocks against 6,
so every block test passes it.
In characters it is 256 against 4310,
a ratio of 16.84,
the corpus maximum.
The cause is legible in the block kinds:
the Chinese page presents a letter as
 an IMAGE,
`<PhotoScroll photos={[...letter.webp]} />`,
and the English page
 additionally transcribes and translates that letter into a 3625-character
 blockquote.
The image block itself is present on both sides.

This is a CLASS,
not an entry.
Comparing blockquote characters per side across
 all 92 entries,
with the image-block count identical on both sides in 91 of 92:

```text
  Mio               196 ->  4625     23.6x
  Zha_Ke              0 ->  3625      transcribed outright
  zheermao101        92 ->  3451     37.5x
  dogesir_          103 ->  1832     17.8x
  MizuharaNagisa      0 ->  1969      transcribed outright
  wangzihao980       41 ->  1208     29.5x
  mikaela_khara     242 ->  1397      5.8x
  shihai4h          933 ->  4416      4.7x
```

The 2.91 median expansion is what keeps this list short and honest.
Twenty
 entries have English blockquotes exceeding Chinese by 500 characters or more,
 but most of that excess is ORDINARY translation growth of a quote that exists
 on both sides:
`gqt` 1064 to 2761 is 2.6x,
`Dethelly` 638 to 1899 is 3.0x,
 `Toka_ls` 392 to 1013 is 2.6x.
Only entries above the 3.76 p90 are candidates,
 which is the 8 listed.

CAUSE CHECKED STRUCTURALLY,
and it holds for SIX of the eight rather than all.
`Zha_Ke` was read directly.
The others were then tested against a mechanical
 signature:
the largest one-sided English blockquote sitting within two blocks
 of an image,
with image counts identical on both sides and the Chinese
 blockquote near zero.

```text
  Zha_Ke           3625ch bq   image within 2 blocks   largest ZH bq    0
  MizuharaNagisa   1969ch bq   image within 2 blocks   largest ZH bq    0
  zheermao101      2115ch bq   image within 2 blocks   largest ZH bq   92
  dogesir_         1487ch bq   image within 2 blocks   largest ZH bq   54
  Mio              2052ch bq   image within 2 blocks   largest ZH bq  132
  wangzihao980     1098ch bq   image within 2 blocks   largest ZH bq   41

  mikaela_khara     741ch bq   NO image nearby         largest ZH bq  224
  shihai4h         1665ch bq   NO image nearby         largest ZH bq  330
```

`mikaela_khara` and `shihai4h` do NOT fit and are removed from the class.
Their
 excess is spread across several blockquotes rather than concentrated in one,
 and each already carries a substantial Chinese blockquote,
so their totals look
 closer to ordinary expansion of quotes present on both sides.

So the class is SIX entries with a verified signature,
not eight.
The
 corpus-wide figure of about 31 thousand excess characters counts all twenty
 entries above the 500-character cut and therefore OVERSTATES this class;
the
 six verified entries hold roughly 12 thousand characters between them.

#### What this changes for the options

-   The claim that a proposal answering only "translate what is missing" says
    nothing about pairs 7 and 5 no longer holds.
    Pairs 7 and 5 are an aligner
    defect with its own fix in `#74`.
-   A THIRD category exists that none of the three options addresses:
    accurate
    English content with no source-markdown counterpart,
    because the source
    holds it in an image.
    Six entries carry a verified signature,
    holding
    roughly 12 thousand characters of one-sided blockquote between them.
-   This is a systematic FALSE-POSITIVE generator for the current pipeline.
    A
    critic comparing markdown against markdown sees thousands of characters of
    unsupported English and is correct to flag it on the evidence it was given.
-   It is a specific hazard for OPTION B.
    Translating every slice from the
    Chinese would DELETE these transcriptions,
    destroying accurate human work,
    and the selection step would have no source-side evidence to prefer keeping
    them.
    B needs an answer to this before it can be chosen.
-   The user's standing ruling covers the policy question:
    keep translator
    additions that are accurate.
    These are accurate.

#### On the earlier 60 of 172

Both sweeps report about a third of pairs differing,
33.6% here against 34.9%
 there,
but that agreement is weak evidence and the totals were never
 reconciled.
The two were produced by different code across a parser fix.
Carry
 this sweep forward because it names its method:
`alignDocumentSections` at the
 pinned corpus SHA,
body blocks only,
partitioned on alignment findings.

## The options

### Option A: route barely-covered sections to a translate stage

Keep the loop.
Add a classifier that measures coverage per aligned section and
 sends the sparse ones to a translator instead of to the critics.

Pros:
 smallest change;
 every existing measurement stays valid for the sections that still take the
 repair path;
 the translate stage is needed under every option,
so none of the work is
 wasted.

Cons:
 needs a coverage threshold,
which is a number someone has to defend,
and
 choosing one is the question the user rejected as an X/Y problem;
 the boundary is where the damage is worst,
since a section just above the line
 gets the omission-storm treatment;
 says nothing about pair 7.

### Option B: translate every slice, then select against the existing text

Every slice gets a fresh translation from the editor ensemble.
The existing translation joins as one more candidate.
The judges already built for candidate selection choose per slice.

Pros:
 no threshold anywhere,
so nothing has to be defended at a boundary;
 one mechanism covers absent,
partial,
wrong and good input alike,
which is
 exactly the decided goal;
 good human translation survives by WINNING selection rather than by not being
 touched,
so preservation becomes a measured outcome instead of an assumption;
 handles pair 7 without special-casing:
a fresh translation competes with an
 overlong one,
and the translate prompt already carries the user's decision that
 accurate translator additions are kept;
 selection,
judging and the apply gate already exist and are already measured.

Cons:
 most expensive by far,
a full translation per slice per editor model;
 risks discarding good human translation whenever judges are weak,
and judge
 quality has never been measured on this question;
 loses the minimal-edit property that makes a repair auditable,
since every
 slice is rewritten by construction;
 the introduced-defect differential needs a BEFORE text,
and a from-scratch
 translation has none,
so that instrument would need rebuilding.

### Option C: fill coverage gaps first, then repair the completed draft

Two phases.
A coverage check finds source content represented nowhere in the
 target and translates it into place.
The existing repair loop then runs over
 the completed draft.

Pros:
 the trigger is a question about TEXT,
not a ratio,
so no threshold has to be
 defended;
 the repair loop and every measurement built around it survive intact;
 degrades gracefully,
since a missed gap leaves a section exactly as it is
 today;
 the second phase sees a complete draft,
which is the state all its instruments
 assume.

Cons:
 the coverage check is the cross-lingual absence question,
and that is precisely
 what produced five of round three's eight false positives,
so the pipeline's
 correctness would rest on the hardest unsolved sub-problem in it;
 two model passes over everything;
 says nothing about pair 7.

## Ranking

B > C > A.

**B over C** because C builds on the sub-problem that has already defeated this
 project once.
Deciding whether source content is represented anywhere in a
 translation is the same judgement that made five accepted issues false
 positives,
and a coverage check that misses is invisible:
the section simply
 stays untranslated and nothing reports it.
B never asks that question.
It
 replaces detection with selection,
and selection is built,
measured,
and
 auditable through its own telemetry.

**C over A** because C's trigger is a question about text and A's is a number.
 A's threshold has to be defended at exactly the boundary where misrouting costs
 most,
and the user has already rejected reasoning in those terms.
C also fails
 softly where A fails loudly:
a missed gap under C leaves today's behaviour,
 while a misclassified section under A sends a mostly-good translation into a
 stage that rewrites it wholesale.

## What has to be settled before any of this is built

-   Judge quality on the new question.
    B stakes everything on judges preferring
    a good human translation over a fluent machine one.
    Nothing has measured
    that,
    and #31's judge crosscheck was deferred all the way back in
    milestone three.
-   What replaces the introduced-defect differential.
    It compares before against
    after,
    and B removes the before.
    The source-anchored damage question built
    on 2026-08-12 is the natural replacement,
    since it never needed the before
    text except to establish that the edit caused the change.
-   ~~What pairs 7 and 5 actually are.~~
    ANSWERED 2026-08-13:
    mispairing.
    Details in "Pairs 7 and 5 are mispairing,
    and the real asymmetry is
    transcribed images".
    This no longer gates the decision,
    but it RAISED a new
    one:
    the transcribed-image class has no answer in any of the three options,
    and it is a specific hazard for B.
-   ~~Cost.~~
    MEASURED 2026-08-13,
    and it is smaller than this bullet assumed.
    Details in "What option B actually costs".
    B raises editor CALLS by 1.56x
    and editor OUTPUT volume by 3.9x,
    because the editor already runs on 64% of
    slices.
    It does not multiply generation over the whole pipeline.

## What the prototype did, run against pair 3

`translate-probe` was run over the section whose entire English side is the
 heading.
All three editor models rendered it:

```text
source section              4641 chars, 62 blocks
existing English              22 chars, 1 block  ("## Memories by Friends")

hf:moonshotai/Kimi-K3      14177 chars produced
hf:zai-org/GLM-5.2         14275 chars produced
hf:zai-org/GLM-4.7-Flash   14534 chars produced
```

The output is a memorial essay in fluent English that preserves the structure
 the original carries:
the `其二：铃语` heading,
`<h3 align = "center">`,
 `<p style="text-align: center;">`,
`<Hexagon>`,
`<details>`,
`<summary>`,
and
 `<DottedNumber n="二"/>` with its Chinese numeral intact as a component prop.
Three voices of three were heard,
and they agree closely on length.

That is the thing the repair loop cannot produce at all.
It is not evidence the
 output is GOOD:
nobody has read it against the original,
the three renderings
 differ (`铃语` came back as both "Lingyu" and "The Bell's Whisper"),
and one
 sample proves nothing about the corpus.

### Two findings the run produced by failing first

The FIRST attempt asked one call for the whole section and lost two voices of
 three,
one to a six-minute timeout and one to schema-invalid output,
then died
 on an uncaught deadline.
Editors in this pipeline work on regions of median 75
 characters and at most 562,
so that call was eight times larger than anything
 the stage has ever been asked for.
The second attempt succeeded on the same
 size,
so the limit is load-dependent rather than hard,
and a translate stage
 has to treat a large unit as a risk to manage rather than a size to assume.

More structurally:
`subdivideChunkPair` returned ONE slice for this section.
Slicing is driven by the pairing,
and with a single 22-character target block
 there is nothing to subdivide against,
so the whole 4641-character section
 arrives as one unit precisely where the work is largest.
A translate stage
 needs subdivision driven by the SOURCE side.
This is a concrete requirement
 under every option,
and it is invisible from the repair path because a section
 with a real translation always subdivides.

### Read against the Chinese

The output was checked rather than admired.
What holds up:

-   Structure survives intact,
    including the JSX components the archive uses.
-   The classical idiom pair 一家之言，姑妄听之；兼听则明，偏信则暗 came back as
    "What one person says,
    take with a grain of salt;
    listen to many and you see
    clearly,
    trust one side and you stay in the dark."
-   PRONOUNS,
    which is the highest-stakes question this corpus has.
    The subject
    had at least three personas with different gender identities and the source
    uses the neutral `TA` throughout.
    Every gendered pronoun in all three
    renderings,
    5 in Kimi-K3,
    4 in GLM-5.2 and 3 in GLM-4.7-Flash,
    refers to a
    DIFFERENT friend described in the same passage,
    never to the subject.
    Neutral forms carry the subject in every one:
    45 in Kimi-K3,
    11 in each GLM.
    Misgendering a trans person in their own memorial is the worst error this
    pipeline could make,
    and none of the three made it.

What does not hold up:

-   `铃语` is the essay's AUTHOR,
    named in the heading and again in the byline
    `<p style="text-align: center;">铃语</p>`.
    Kimi-K3 and GLM-5.2 render it
    "Lingyu";
    GLM-4.7-Flash renders it "The Bell's Whisper",
    translating a
    person's name as prose.
    The identity block cannot help,
    since it declares
    the subject's names and not a contributor's.
-   GLM-4.7-Flash leaves `TA` untranslated 13 times.
    Faithful to the source's
    deliberate choice and opaque to an English reader;
    whether that is right is
    a policy question this proposal does not settle.

So the prototype clears the bar that matters most and fails on a name,
which is
 the same class the declared-names rule already handles for subjects and does
 not yet handle for contributors.

## The section aligner turned out to need this decision too

Added 2026-08-12,
after `#71`'s aligner was measured against the pin.

The aligner works.
On `XingZ60` it pairs every section correctly,
anchored by
 three headings whose Latin names match outright,
and it reports the two
 genuinely untranslated sections as UNPAIRED rather than placing them somewhere.
Today's proportional fallback slides all of that by two.

Wiring it in is nonetheless blocked,
and on this document's question rather than
 on any remaining code.
The pairable sections improve unambiguously.
The unpaired ones need a destination,
and both destinations available today are
 ones the pipeline handles badly:

-   Pairing an unpaired section with an EMPTY target survives.
    `subdivideChunkPair` was run on exactly that and returned ONE slice carrying
    915 characters of original against nothing.
    That is the pathological shape recorded already:
    slicing is driven by the
    pairing,
    so a section with no translation cannot be subdivided and arrives
    whole,
    precisely where the work is largest,
    and the critics then file
    omission after omission against text that is not there.
-   Skipping the section is honest about what is known,
    and means source content
    reaches no stage at all.
    That contradicts `doc/decision/translation-repair-output-goal.md`,
    which
    decided the pipeline yields a good translation of the ORIGINAL even where
    the translation fed in does not make sense.

Under option B both disappear,
because an unpaired section is simply a slice
 whose existing translation is empty,
and every slice is translated regardless.
Under option C the coverage check is what fills it.
Under option A the section is exactly the sparse case the classifier routes.

So the aligner is a reason to settle this document sooner,
not a separate
 decision.
The measurement is durable either way:
it is recorded in `#71` and the
 tie-break defect it exposed is fixed and tested.

## What exists already

`translate-wire.ts` and the `translate-probe` task are committed,
called by
 nothing,
and carry both policies the user settled on 2026-08-12:
accurate
 translator additions are kept,
and declared names are authoritative.
The prompt
 offers the existing translation as evidence and as wording worth keeping where
 it is right,
never as the thing being corrected,
which is the anchoring the
 introduced-defect probe was moved to the same day.

## The contributor-name gap, sized

Added 2026-08-13.
The prototype above fails on `铃语`,
the essay's author,
 because the identity block declares the SUBJECT's names and not a
 contributor's.
That gap is worth sizing before it is designed around.

Searching the corpus for the pattern the prototype hit:

```text
  entries with contributor-named headings (其N：NAME)   1 of 92, 10 headings
  entries with centred byline paragraphs                2, 9 paragraphs
```

The detection is pattern-specific,
so a contributor named some other way would
 not be counted.
Taken as a lower bound,
the exposure is one or two entries and
 roughly ten names.

So the gap is real and NARROW.
It does not block option B:
extending the
 declared-names block to contributors,
or a rule that a heading of the form
 `其N：NAME` names a person rather than a topic,
would cover the observed cases.

## `XingZ60` is carrying most of the hard cases at once

Worth stating because it cuts both ways.
That single entry is:

-   the ONLY entry whose sections mispair,
    out of 92;
-   the source of both untranslated sections the aligner would leave unpaired,
    915 and 1459 characters;
-   the only entry with contributor-named headings,
    all ten of them;
-   the source of the two largest block-count gaps cited in
    `doc/decision/translation-repair-output-goal.md`,
    76 against 5 and 62
    against 1.

A fix aimed at `XingZ60` therefore addresses a disproportionate share of the
 known hard cases,
which makes it attractive.

The same fact is the warning:
a design validated ONLY against `XingZ60` is
 fitted to one document.
`XIEPT2` is the useful second case,
since it pairs
 correctly while still carrying genuine coverage gaps of 24 against 1,
18
 against 1 and 18 against 2,
so it exercises the translate path without the
 alignment defect confounding it.

## What option B actually costs

Measured 2026-08-13 over the 10 entries `pass13` had settled,
by re-deriving
 slices with `subdivideChunkPair` at the pinned corpus SHA and reading the
 editor envelopes out of the artifacts.

The premise that needed checking is in option B's con list,
"most expensive by
 far,
a full translation per slice per editor model",
and in the settle-first
 bullet,
"B multiplies generation by the editor roster over every slice".
Both
 read as though the editor is idle today.
It is not.

```text
  slices                              97
  slices the editor fires on today    62   (64%)
  share of all slice TEXT they hold        75%

  editor CALLS      today  3 x 62 =  186
  editor CALLS      in B   3 x 97 =  291      1.56x

  distinct envelopes today           116, holding 8365 characters
  target characters in all slices  32361

  editor OUTPUT     today  ~25095 characters
  editor OUTPUT     in B   ~97083 characters   3.9x

  mean slice target   334 characters
  mean repair envelope 72 characters
```

So B costs 1.56 times the editor calls,
each producing about 4.6 times more
 text,
for 3.9 times the editor output volume.
That is a real increase and it is
 nothing like a multiplication of the run.

The reason is that the current pipeline is ALREADY mostly-rewriting.
The editor
 fires on 64% of slices,
and those slices hold 75% of all slice text,
so B's
 "every slice" adds 35 slices carrying a quarter of the text.

The 3.9x output figure is where B's real cost sits,
and it comes from the SIZE
 of what is generated rather than from how often:
a full slice averages 334
 characters against a repair envelope's 72.

### What this does not measure

Only the editor stage.
The critic stage fans out to a six-model roster on every
 slice,
which is 582 calls over the same 97 slices,
against the editor's 186.
Panel,
refiner and probe calls were not counted.
So the editor is a MINORITY of
 pipeline calls,
and adding 117 of them is a small fraction of total traffic
 rather than 67% of it.

The 60% figure comes from 10 settled entries,
which skew small:
`Dethelly` at 24
 slices is the largest and the corpus contains much larger documents.
The slice
 counts are exact,
the ratio is a sample.

### An unasked question that could invert the ranking

Option B says the judges "choose per slice" among candidates,
with the existing
 translation as one candidate.
If selection decides quality per slice,
it is not
 obvious the CRITIC stage is still needed at all:
critics exist to find defects
 for an editor to repair,
and B repairs nothing.

Dropping critics would remove 582 calls and add 105,
making B CHEAPER than the
 pipeline it replaces rather than more expensive.
The redesign note does not say
 whether critics survive B,
and the answer moves B from "most expensive by far"
 to "least expensive".
This should be settled before the cost con is weighed.

It is a genuine open question rather than a recommendation:
critics may still be
 wanted to inform the judges,
and the transcribed-image class is a reason to keep
 a stage that reasons about the source,
since selection alone would have no
 evidence for preferring a translation that carries image-borne content.

## The premise this note was written on does not hold

Written 2026-08-14,
after calibrating the coverage measure.
This section CORRECTS "The options" and "Ranking",
 which were argued from a corpus that was believed to be widely untranslated.

The block-count gap that opened this note counts a stub and a reformatted
 paragraph identically.
Of the 84 block-count gaps,
68 carry a full translation,
so the metric was
 measuring formatting as much as coverage.

Character ratio replaces it,
and the corpus supplies its own band rather than
 taking the assumed zh-to-en range of 1.5 to 2.5,
which was wrong:
a Chinese
 character carries roughly an English word.
Measured over 254 aligned pairs with at least 80 source characters:

```text
  p2 0.74   p5 1.55   p10 1.83   p25 2.48
  p50 2.94  p75 3.35  p90 3.76   p98 4.55
```

### What is actually under-covered

Six pairs of 254,
which is 2.4%:

```text
  shi_Yumiaoya    1203 ->   12   0.01   stub
  shi_Yumiaoya     988 ->   13   0.01   stub
  shi_Yumiaoya     695 ->   14   0.02   stub
  cheonwoomaeng   1137 ->  529   0.47
  Y1Ran            215 ->  150   0.70
  shi_Yumiaoya     474 ->  350   0.74
```

The extreme stubs are one entry,
but `cheonwoomaeng` is not,
so this is not a
 single bad page.
The next four pairs sit at 1.32 to 1.42 and read as ordinary variation.

That leaves a MEASURED GAP between 0.74 and 1.32 with nothing in it,
which is
 what turns the threshold from a tuned number into a described one:
a ratio of
 1.0 says the translation carries fewer characters than the original,
and for a
 language pair whose median expansion is 2.94 that is a statement about text
 rather than a knob.
Half the median,
1.47,
would have been the natural line and is the wrong one:
 it sits above four pairs that are fine.

The expanded tail resolved too.
Seventeen of the 18 pairs above 4x are in entries with no structure-mismatch
 finding,
several with identical block counts on both sides,
so the ">4" bucket
 was an artifact of the guessed band.
Five pairs do stand clear and remain unread:
`Zha_Ke` 16.8x,
`Mio` 10.5x,
 `shihai4h` 9.8x,
`zheermao101` 7.6x,
`MizuharaNagisa` 6.6x.

### Revised ranking: A > C > B

The ranking inverts because the thing being served shrank from "most of the
 corpus" to 2.4% of it,
while every option's cost stayed where it was.

**A over C** because A's trigger is arithmetic on two character counts,
computed
 at alignment time before any model call,
and C's trigger is the cross-lingual
 absence question that produced five of round three's eight false positives.
A costs nothing and cannot be wrong about a section it does not route;
C spends
 a model pass over the whole corpus to find six sections,
and a coverage check
 that misses is invisible,
since the section simply stays untranslated.
A's original con was that its threshold is a number someone has to defend.
The measured gap answers it:
any line between 0.74 and 1.32 routes exactly the
 same six sections,
so the choice inside that interval changes nothing.

**C over B** because C keeps the repair loop and every instrument built around
 it,
while B removes the BEFORE text that the introduced-defect differential
 needs and stakes the whole corpus on judges preferring good human translation
 to fluent machine translation,
which nothing has measured.
Retranslating 248 sections that are already translated to serve 6 is the wrong
 trade at this ratio,
and it is the trade B makes by construction.

B keeps one advantage worth naming:
it is the only option that answers a section
 whose translation is present,
fluent and WRONG,
since detection by ratio and
 detection by absence both see such a section as covered.
Nothing in the corpus has been shown to be that,
so it is a hazard rather than a
 finding.

### What A actually is, concretely

-   At alignment time,
    for each paired section,
    divide target characters by
    source characters.
    Both counts already exist;
    nothing is called.
-   Below the line,
    send the SOURCE section to the translate lane that
    `translate-probe` already prototyped,
    and skip the critics for it entirely.
    A section with no translation has nothing to file a defect against,
    which is
    why the omission storms happen.
-   Above the line,
    nothing changes.
    Every measurement built on the repair path
    stays valid for the 248 sections that keep taking it.
-   Record the ratio and the routing decision in the artifact,
    so the boundary
    can be audited later rather than argued about now.

### What to re-measure before building it

The percentiles were computed under the forced aligner on 2026-08-13,
and the
 aligner has changed since:
`#74` corrected its scoring and its handling of
 asymmetric preambles.
Section pairing decides which characters are compared,
so the band and the six
 under-covered pairs should be recomputed against the current pipeline before
 the line is written into code.
The shape of the finding is unlikely to move,
since the stubs are stubs under
 any pairing,
but the four near-line pairs are exactly the ones a pairing change
 could shift across it.
