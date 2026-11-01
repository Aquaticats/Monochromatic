# Translation repair history 2026-08-15 to 2026-08-19: the displacement screen, the wider window, the screen rebuilt and the second review of the rebuild

The displacement screen built and run,
the wider-window trial,
the finding that the screen counted four different things and its rebuild into a classification,
the corrections to the numbers the handover gave,
and the second review of the rebuild with what was deferred on purpose.

Part of [Translation repair history: 2026-08-15 to 2026-08-19](translation-repair-history-2026-08-15-to-19.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The displacement screen built, the wider window and the screen rebuilt

### The displacement screen, built and run: 6.4 percent of slices

WHAT IT READS AND WHY THAT WORKS.
Chinese-to-English expansion is roughly
threefold and steady WITHIN one document,
because one translator worked at one
density.
So a slice far above its own document's median took text on,
and a
neighbour below it gave text up,
and the PAIR is the evidence:
one anomaly alone
is a loose paraphrase,
while a high beside a low is a passage that moved.
Comparing each document to itself is what makes one threshold usable across a
corpus of different registers.

`src/displacement-ratio.ts` and the `displacement-probe` task.
ZERO QUOTA,
nothing cached,
nothing written,
no lane reads it.

    complete pairs                        92
    slices read                         1198
    high slices                           41
    moved pairs                           44
    entries carrying at least one         22   (24 percent of entries)
    slices involved in a moved pair       77   (6.4 percent of slices)

THE SCREEN WAS VALIDATED BEFORE ITS OUTPUT WAS BELIEVED.
Its tests carry a
positive control (a slice at three times the median beside one below it reads as
moved) and two nulls that must stay silent (an evenly expanding document flags
nothing;
a high slice with NO low neighbour is reported as high rather than
moved).
Slices under twenty source characters are skipped so a bare heading
cannot set the median.

AND THEN THREE FLAGGED PAIRS WERE READ BY HAND,
on both sides,
which is what
turns a count into a finding:

- `Dethelly` 0 to 1:
  a true relocation,
  the one that started this.
- `lintong` 3 to 2:
  a true relocation.
  Slice 3's English carries slice 2's
  Chinese almost sentence for sentence,
  and slice 2's English is reduced to an
  attribution line.
- `dogesir_` 3 to 2:
  A DIFFERENT CLASS through the same screen.
  The English adds
  "English transcript of her self-description above" plus the transcript,
  which
  the Chinese carries only as an IMAGE.
  That is Question 2's transcribed-image
  class,
  found here without looking for it.
  Its low neighbour is pure
  `PhotoScroll` markup at ratio 1.00,
  which cannot expand.

SO SAY THE PRECISION IN TERMS OF THE QUESTION ASKED.
For "a per-slice judge will
condemn the archive here",
three of three.
For "a passage moved between
sections",
two of three.
Both classes cause the same judging failure,
which is
what Question 5 needs;
only the remedy differs.

6.4 PERCENT IS A FLOOR,
not an estimate.
The screen reads SIZE,
so a passage
swapped for one of similar length reads as perfectly ordinary,
and a markup-only
slice at ratio 1.00 will pair with any high neighbour without being a donor.
Excluding markup-only slices from the low side would sharpen the count and cannot
create false highs.

### The wider window: the judges were right, and the window was the problem

Question 5's option E asked whether the roster's declines on `Dethelly/0` came
from the JUDGEMENT or from the WINDOW.
The alteration fixture was rerun on the
same four slices with the neighbouring source sections added as evidence,
labelled
"context only:
the candidates are not expected to render this".
Ground truth
unchanged,
damage unchanged,
roster unchanged.

    alteration, narrow window        12 of 16 trials chose the complete text
    alteration, wider window         15 of 16
    ballots for the DAMAGED text      0 of 96, both arms

All four narrow-window misses were `Dethelly/0` declines.
Three became correct
choices.
THE WINDOW WAS THE PROBLEM,
not the judgement.

THE PAIRED COMPARISON IS SOUND,
which had to be checked rather than assumed.
The
`sharedNumber` whole-run fix landed BETWEEN the two runs and can only remove
picks,
so a moved pick would have made the arms differ in more than the window.
The picks are recoverable from the judges' quoted reasons,
and they are identical
per entry:
`AmbeR_the_anpa/1` 2004 to 2005,
`Arita/4` 2027 to 2028,
`Chinatsu_Suzuki/1` and `Dethelly/0` 2023 to 2024.
Nothing moved.

NO REGRESSION ON THE OTHER THREE ENTRIES:
12 of 12 in both arms.
Widening did not
buy the Dethelly flip with new failures elsewhere,
which was the cost to watch
for,
since a context block a judge mistakes for required content would show up as
completeness complaints on slices that were previously clean.

THE ONE REMAINING DECLINE IS THE FINDING,
NOT THE FAILURE.
With the neighbouring
Chinese visible,
the roster declines `Dethelly/0` (replace) on this reasoning:
both candidates OMIT that Sugar is a girl from Chongqing,
and both ADD material
the slice's own original does not carry.
Both halves are true,
and together they
are the relocation stated precisely.
The judges are not confused here;
they are
reporting that the archive laid this passage out differently,
and no per-slice
candidate can satisfy them.
`#107` is the ticket for that,
and this is its
sharpest evidence.

THE WIDER WINDOW DID NOT MOVE THE CHRONIC DECLINERS,
which is a roster finding
rather than a window one.
Declines out of 16 trials per arm,
in order
deletion / insertion / alteration-narrow / alteration-wide:

    NVIDIA-Nemotron-3-Super       16  15  15  14
    openai/gpt-oss-120b            9  11  12  12
    zai-org/GLM-5.2                4   3   4   4
    moonshotai/Kimi-K3             0   1   3   3
    zai-org/GLM-4.7-Flash          0   1   3   1
    Qwen/Qwen3.6-27B               1   0   4   1

Nemotron declines 87 to 100 percent of trials that have a plainly correct answer,
and gpt-oss 56 to 75 percent.
Neither is reading the fixture wrong;
both are
refusing to choose.
The other four sit between 0 and 25 percent.
Question 6 reads
this.

CORRECTION TO A NUMBER I PUBLISHED.
The decline triples previously recorded in
Question 5 (`gpt-oss` 11,
11,
12 and `Nemotron` 11,
15, 16) mixed the SUPERSEDED
marked-deletion arm into the sequence;
both models declined 11 in that arm,
which
is where both leading elevens came from.
The three current arms give `gpt-oss`
9,
11,
12 and `Nemotron` 16,
15, 15.
Corrected in place.

### The displacement screen measures four different things, and 6.4 percent is not a floor

The screen's output was believed too quickly.
Its precision was hand-checked on
three pairs drawn from entries whose median expansion was ORDINARY (3.31,
2.88,
2.85),
and it was never checked on entries whose median is anomalous.
Four
entries carry a median far outside any plausible Chinese-to-English band,
and
reading their per-slice numbers shows the single `movedPairs` bucket is
conflating four distinct phenomena.

`shi_Yumiaoya`,
median 0.76,
is the one that shows the mechanism:

    slice 0   zh   165   en   300   ratio 1.82   <- flagged HIGH
    slice 1   zh   162   en   436   ratio 2.69   <- flagged HIGH
    slice 2   zh   715   en    14   ratio 0.02
    slice 3   zh  1016   en    13   ratio 0.01
    slice 4   zh  1313   en    12   ratio 0.01

Slices 2,
3 and 4 are UNTRANSLATED:
the English is a bare heading.
They drag the
document's median to 0.76,
which drops the HIGH threshold to 1.51,
which flags
two ORDINARY translations as anomalies.
THE STATISTIC IS CONTAMINATED BY THE
THING IT IS MEANT TO DETECT.
That is a design fault,
not a threshold to tune.

The other three each fail differently:

- `noname3031`,
  median 1.35:
  the HIGH slice is TWENTY-THREE Chinese characters,
  one over the floor.
  Both its pairs are arithmetic noise.
  Its neighbours sit at
  ratio 1.00 and 1.17,
  and one is 287 characters against 287,
  which is the shape
  of an untranslated verbatim block rather than a translation.
- `Zha_Ke`,
  median 7.51:
  slice 1 is zh 41 against en 3652,
  a ratio of 89.
  A long
  letter exists ONLY in English;
  the whole Chinese page is 615 characters.
- `wangzihao980`,
  median 1.55:
  slice 4 is zh 141 against en 1228,
  ratio 8.71.
  Same class as `Zha_Ke`.

SO THE FOUR CLASSES,
which want four different remedies:

1.  RELOCATION.
    A passage moved across a boundary.
    High on one side,
    low on the
    neighbour,
    both sides substantial.
    `Dethelly/0`,
    `lintong/3`.
    This is the
    per-slice judging hazard and the only one the screen was built for.
2.  UNTRANSLATED SECTION.
    Ratio near zero,
    large original,
    negligible
    translation.
    This is `#106`'s subject,
    and the screen produces exactly the
    positive verdict `#106` says nothing produces:
    a ratio of 0.01 is not
    ambiguous.
3.  SOURCE-ABSENT CONTENT.
    Ratio far above any translation density with a tiny
    original.
    Question 2's transcribed-image class,
    at document scale.
4.  NOISE.
    A slice too short for a ratio to carry information.

WHAT THIS DOES TO THE PUBLISHED NUMBER.
44 moved pairs across 22 entries is an
UPPER BOUND on a mixture,
not a floor on relocation.
The seven pairs contributed
by the four anomalous-median entries are classes 2,
3 and 4,
not class 1.
The
earlier "6.4 percent is a floor" wording is withdrawn:
it is a floor only for
"slices where a per-slice judge will misjudge the archive",
which is the union of
all four classes and is the quantity Question 5 actually needs.
It is NOT a floor
for relocation.

NEXT:
`readDisplacement` should return a classification rather than one bucket,
so each class is separately actionable,
and the median should be computed over
slices that are plausibly translated at all.
A second reviewer was asked for the
type and the thresholds before this is rebuilt.

### The screen rebuilt, and the corrected corpus numbers

`src/displacement-ratio.ts` now holds size primitives and
`src/displacement-class.ts` the classification,
gated by tests built from the
labelled cases rather than from invented shapes.

    complete pairs                                     92
    slices read                                      1260
    entries falling back to the corpus baseline         12
    relocation candidates                              22
      of which a transcription would also explain        2
    untranslated slices                                 4
    target-only slices                                 12
    other imbalances                                   24

AGAINST THE OLD 44 MOVED PAIRS,
which is the comparison worth reading:
roughly
half of that number was relocation and the rest was three other phenomena plus
arithmetic.
The count did not shrink because the screen got stricter in general;
it shrank because three of the four things it was counting now have their own
names.

WHAT CHANGED IN THE INSTRUMENT,
each with a test that fails without it:

- THE DOCUMENT MEDIAN IS GONE,
  replaced by an aggregate over slices that are
  plausibly translated.
  A median is contaminated by the untranslated sections it
  exists to find.
  An aggregate is also invariant under relocation,
  since moving
  text between slices leaves both document totals alone,
  so the baseline cannot
  absorb the phenomenon being measured.
- THE CORPUS REFERENCE IS MEASURED:
  2.86,
  the median of 91 document aggregates,
  where the first version said "roughly threefold" from recall.
  Twelve entries
  fall outside the believable band and borrow it.
- `median` TOOK THE HIGH MIDDLE on an even count while documenting the low one,
  so `median([1, 2, 9, 10,])` answered nine.
  On a threshold that is a multiple of
  the median,
  that bias raises the bar and hides anomalies.
- A MEDIAN OF ZERO MADE EVERY SLICE HIGH,
  since twice zero is zero.
  A document
  whose sections are mostly untranslated hit this exactly.
- A SOURCE-LENGTH FLOOR DISCARDED THE BEST EVIDENCE.
  `Zha_Ke`'s slice carries 41
  original characters against 3652 translated and was dropped before anything
  looked at it.
- `readPair` CAUGHT EVERY ERROR as a missing translation,
  so a permission or
  decoding fault would have quietly shrunk the corpus-wide counts.

TWO THINGS THE ACCEPTANCE GATE CAUGHT IN MY OWN DESIGN,
which is what it was for:

- `target-only` AS A SLICE CLASS SILENTLY ATE THE FOUNDING CASE.
  `Dethelly/0` is
  35 original characters against 403 translated,
  and so is the shape of
  `Zha_Ke/1`.
  Size cannot tell a relocation from English-only content;
  only the
  neighbour's deficit can.
  It is now decided AFTER the neighbour test.
- A LENGTH FLOOR ON THE DONOR SIDE DROPPED `lintong`.
  Its verified donor carries
  43 original characters against 25 translated.
  What makes a slice evidence is
  its residual,
  not its length.

CONSERVATION IS ASYMMETRIC,
measured rather than assumed.
Both verified
relocations run a deficit near HALF the surplus:
`Dethelly` 297 against 121,
`lintong` 281 against 99.
The slice that gave text up still renders its own
original,
while the slice that took it on carries the expanded English of both.
A reviewer proposed requiring "reasonably similar magnitudes",
which would have
rejected both cases the instrument exists for.

AND ONE MORE CLASS FOUND BY HAND-CHECKING THE BOUNDARY.
`wangzihao980/4->3` sat
just over the conservation floor at 232/816.
Reading both sides:
the Chinese
embeds a `PhotoScroll` of a note,
and the English embeds the SAME image plus a
full transcription and translation of what it holds.
The neighbour's deficit is
independent condensation that happens to sit next door.
Screening all 22
candidates for a media component present on both sides flags exactly the two
entries already verified by hand as transcriptions,
`dogesir_/3` and
`wangzihao980/4`,
and leaves both verified relocations clean.
That check reads
TEXT rather than sizes,
so it lives beside the probe in
`src/corpus-run/transcription-suspect.ts`.

SO THE HONEST READING OF 22.
Two are transcriptions on the probe's own evidence.
Two are verified relocations.
The remaining eighteen are unchecked,
and the class
is named `relocationCandidates` for that reason.
What the number supports is
"slices where a per-slice judge will misjudge the archive",
which is what
Question 5 needs;
it does not yet support "22 passages moved".

WHAT WAS ADOPTED FROM THE SECOND REVIEWER AND WHAT WAS NOT.
Adopted:
candidates
rather than verdicts in the naming,
the fifth bucket for one-ended surpluses,
absolute floors beside relative ones,
the aggregate baseline,
all four defects
above.
Rejected:
a twenty-five threshold configuration table and leave-one-out
corpus learning,
neither of which 92 documents and seven labelled slices can
calibrate;
and the symmetric conservation test,
which the measurement refuted.

## Corrections to the numbers the handover gave, and the second review of the rebuild

### Three corrections to what this document said an hour ago

A reviewer reading the shipped state caught three numbers,
and all three were
wrong in the same way:
stated confidently,
never recomputed after the thing they
described changed.

THE UNION WAS NOT 62 SLICES,
IT IS 80 OF 1260,
6.3 percent.
Adding the four class
totals mixes units:
the relocation figure counts PAIRS while the other three
count slices,
and pairs share slices,
since `lintong` reports 3 to 2 and 3 to 4
across three slices rather than four.
Counted as unique slices,
with both ends of
every pair included because a per-slice judge misjudges the archive at both,
the
answer is 80.
Corrected in Question 5.

THE CONSERVATION RATIOS IN THE CODE COMMENT WERE FROM THE OLD BASELINE.
It cited
0.51 and 0.44,
measured before the eligibility filter existed.
Against the
baseline the shipped code computes they are 0.41 and 0.35.
The margin above the
quarter floor is thinner than the comment implied,
and the corrected numbers say
something the old ones hid:
BOTH candidates verified by hand as transcriptions
sit at 0.28,
and both verified relocations at 0.35 and 0.41.
Five points is not
grounds to retune a threshold.
It is grounds to hand-check a near-floor candidate
rather than count it,
which `#108` now says to do.

THE TEST COUNT WAS WRONG and the numeral is gone rather than corrected,
per WR4.

AND A FOURTH INSTANCE OF THE SLICE-ALONE MISTAKE,
found by the same reviewer
after reading the closing observation above and applying it to the one class the
rebuild exempted.
`untranslated` is decided from the slice alone,
and its own
discriminator can sit in the neighbour:
where a translator rendered a WHOLE
section inside its neighbour,
the emptied slice reads untranslated,
the taker
reads high,
and the guard that refuses an untranslated donor suppresses exactly
that pair.

IT WAS MEASURED RATHER THAN FIXED,
and the measurement is why.
At the pinned
commit,
ZERO untranslated slices sit beside a flagged high,
so the gap costs
nothing today.
Changing the rule on no evidence would trade a measured zero for
an unmeasured guess.
It is recorded in the class's own TSDoc and in `#108` as a
reading instruction:
an untranslated slice beside a high one is a possible
whole-section move,
so hand-check it rather than trusting the class.

### The second reviewer on the rebuild, and what was deferred on purpose

A reviewer reading the rebuilt instrument found four design problems and four
false claims.
The claims were corrected in place;
the design changes went to
`#432` rather than being made the same night as the numbers that depend on them,
which is a judgement worth stating rather than hiding:
the count had already been
corrected twice in a few hours,
and a third change at that hour is how the third
error gets introduced.

THE ONE THAT MATTERS MOST,
and it was verified rather than relayed.
The comment
claimed the aggregate baseline is INVARIANT UNDER RELOCATION.
It is not,
as
implemented.
A total over ALL slices would be,
but this one is taken over
ELIGIBLE slices,
and relocation can carry text across that boundary.
`Dethelly`
is the case:
its 35-character recipient is excluded by the length floor while its
129-character donor is included.
Measured,
the deficit over surplus reads

    shipped baseline (>=80 filter)      Dethelly 0.406    lintong 0.352
    leave-one-pair-out                  Dethelly 0.442    lintong 0.352
    all slices                          Dethelly 0.513    lintong 0.438

THE CONCLUSION SURVIVES ALL THREE.
The deficit is the smaller side under every
baseline,
so the asymmetry finding stands and so does the refusal of a symmetric
conservation test.
What does not survive is the word "invariant",
and the
estimator being endogenous to the thing it measures is the SAME error as the
median contamination the rebuild was built to fix.
`#432` carries the
leave-one-pair-out estimator.

THREE MORE FALSE CLAIMS,
all corrected.
`median` returned the lower middle rather
than a conventional median,
and once the aggregate replaced it nothing used it at
all,
so it is deleted rather than renamed.
`readPair`'s comment said it separated
an absent path from a failure,
but `CorpusReadError` is thrown on ANY non-zero
git exit,
so a bad pin and a broken invocation still read as a missing
translation;
narrowing the catch was an improvement and not the fix.
`CorpusTotals.relocationCandidates` counts ADJACENCIES while its documentation
said slices.

AND THE 91 AGAINST 92:
both are right.
`XIEPT2` has both files and no source
text,
so it can carry no aggregate while still being a complete pair.

WHAT IS ON `#432` AND UNFIXED:
the leave-one-pair-out baseline;
a HIGH gate that
is relative and so misses a large absolute surplus on a long slice,
which is a
false negative and those cost more here;
classes that are exclusive when the
evidence is not,
which `wangzihao980` demonstrates by having a real transcription
pushed out of `target-only` by an unrelated neighbouring deficit;
and sizes that
count markup as translation,
which is exactly the class the media cases live in.
