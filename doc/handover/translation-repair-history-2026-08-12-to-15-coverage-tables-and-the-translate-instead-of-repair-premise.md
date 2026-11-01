# Translation repair history 2026-08-12 to 2026-08-15: the coverage tables and the end of the translate-instead-of-repair premise

The block-count gap re-measured under the forced aligner,
its correction as a formatting difference rather than coverage,
the coverage table calibrated against the corpus,
the expanded pairs resolved,
and the two ways the proposal to translate instead of repair lost its basis,
including the constructed-score lesson.

Part of [Translation repair history: 2026-08-12 to 2026-08-15](translation-repair-history-2026-08-12-to-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The block-count gap re-measured under the forced aligner

The 60-of-172 figure was computed while alignment distributed sections
 proportionally by character fraction.
`#71` established that this cannot
 express absence and so slid whole documents,
which made the number describe
 alignment quality as much as translation coverage.
`#69` and `#70` both rest
 on it,
so it needed recomputing before either could be decided.

Recomputed over all 92 entries carrying both sides,
with the shipped
 `alignDocumentSections`:

```text
aligned pairs                    275   (was 172)
differing in block count          85   30.9%   (was 60 of 172, 34.9%)
identical                        190
findings: structure-mismatch      21
findings: sections-merged          0   (was the mechanism behind the old number)
```

### What changed, and what did not

The RATE barely moved,
from 34.9% to 30.9%.
Sections whose translation covers a
 fraction of their source are real,
and the premise behind `#69` and `#70`
 survives in that sense.

The PAIR COUNT rose by more than half,
172 to 275,
and `sections-merged` fell to
 zero.
That is the whole difference in one line:
proportional merging welded
 several source sections into one pair,
so the old denominator counted welded
 blobs where the new one counts sections.

The headline example did not survive.
The recorded worst case was a section with
 76 source blocks against 5 target blocks,
cited as the clearest evidence that
 the pipeline is repairing something that was never translated.
Under correct
 pairing that section does not exist.
The largest gaps now are:

```text
shi_Yumiaoya   15 against 1
shi_Yumiaoya   14 against 1
shi_Yumiaoya   11 against 1
shi_Yumiaoya   12 against 5
mikaela_khara  18 against 14
Aniloviraw     24 against 21
```

Four of the six worst are one entry,
and its id begins `shi_`,
so verse
 formatting rather than missing translation is a live alternative reading for
 those.
Nobody has checked;
it is recorded here as the next thing to look at
 rather than as a conclusion.

### Limitation of this recompute, stated plainly

`alignDocumentSections` returns paired chunks,
so a section the aligner REFUSED
 appears in no pair and is not counted in either column.
The 21
 `structure-mismatch` findings are the only signal of refusal in this
 measurement,
and they count entries rather than sections.
A gap figure that
 accounted for refusals separately would need the aligner's steps,
not its
 pairs.

### Correction: the block-count rate is not a coverage measure at all

The recompute above concluded that the premise behind `#69` and `#70` survives,
 because the share of pairs differing in block count barely moved,
34.9% to
 30.9%.
That conclusion was wrong,
and the error was in the metric rather than
 in the arithmetic.

Block count conflates two unrelated things.
Reading the worst gaps by CHARACTER
 RATIO,
target characters over source characters,
separates them immediately:

```text
shi_Yumiaoya    695 ->   14   ratio 0.02
shi_Yumiaoya    988 ->   13   ratio 0.01
shi_Yumiaoya   1203 ->   12   ratio 0.01
mikaela_khara   517 -> 1731   ratio 3.35
Aniloviraw      751 -> 1498   ratio 1.99
```

The first three are stubs:
a dozen characters standing in for a thousand.
The
 last two are ordinary translations,
at or above the expansion a faithful
 zh-to-en rendering produces,
whose block counts differ only because paragraphs
 split differently.
Both shapes were being counted identically.

Measured corpus-wide over 254 pairs with a substantive source,
at least 80
 source characters:

```text
absent    ratio < 0.25        3   1.2%
partial   0.25 to 0.75        3   1.2%
covered   0.75 to 4         230  90.6%
expanded  ratio > 4          18   7.1%
```

And of the 84 pairs that differ in block count,
68 (81.0%) are fully covered by
 character ratio.
The block-count gap is overwhelmingly a formatting difference.

The three genuinely untranslated sections are all in ONE entry,
`shi_Yumiaoya`.

### What this does to `#70`

`#70` proposes re-designing the pipeline to PRODUCE a translation rather than
 repair one,
and its case was that a large share of sections are only partly
 translated.
On the corpus at the pinned commit that share is 2.4%,
absent plus
 partial,
concentrated in a single entry.
The corpus is essentially translated,
 and the pipeline is repairing translations that exist.

That does not settle `#70`,
which is the user's decision to make,
but it removes
 its evidentiary basis.
The honest framing for the proposal is now:
one entry
 needs translation rather than repair,
and a per-entry escape hatch would serve
 it without re-designing the pipeline everything else depends on.

The 18 pairs expanding beyond 4x are unexamined.
They could be translator
 additions,
which house policy keeps when accurate,
or a pairing artifact.
 Worth a look before anyone cites the coverage table as complete.

### The coverage table, calibrated against the corpus instead of a guess

The buckets in the table above used an assumed expansion band of 1.5 to 2.5,
 which was a guess about zh-to-en character growth and was wrong.
A Chinese
 character carries roughly an English word,
so faithful translation expands
 several-fold in characters.
Measured over the same 254 pairs,
the corpus states
 its own band,
and it is tight:

```text
p2    0.74      p50   2.94      p90   3.76
p5    1.55      p75   3.35      p98   4.55
p10   1.83
p25   2.48
```

Median expansion is 2.94x.
Half the median,
1.47x,
is a defensible line for
 "this section carries materially less than a translation should",
and it is
 derived rather than picked.

Below that line sit 10 pairs of 254:

```text
shi_Yumiaoya    1203 ->   12   0.01
shi_Yumiaoya     988 ->   13   0.01
shi_Yumiaoya     695 ->   14   0.02
cheonwoomaeng   1137 ->  529   0.47
Y1Ran            215 ->  150   0.70
shi_Yumiaoya     474 ->  350   0.74
aiyysk           116 ->  153   1.32
Considerate_cat  143 ->  190   1.33
Chinatsu_Suzuki  667 ->  948   1.42
noname3031      1191 -> 1694   1.42
```

The first six are the finding:
three stubs,
and three sections carrying under a
 quarter of the expected text.
The last four sit near the line and are ordinary
 variation.
So roughly 6 of 254 pairs,
2.4%,
are genuinely under-covered,
which
 is the same figure the arbitrary buckets produced.
The number was right before
 and is now right for a stated reason.

`cheonwoomaeng` at 1137 characters rendered as 529 is new,
and it is not in
 `shi_Yumiaoya`.
Under-translation is therefore not confined to one entry,
even
 though the extreme stubs are.

### The 18 expanded pairs, resolved

They are not translator additions and not a pairing artifact:
17 of 18 sit in
 entries with no `structure-mismatch` finding,
and several have identical block
 counts on both sides (`1->1`,
`6->6`,
`8->8`,
`21->21`).
Given p98 is 4.55,
a
 ratio just over 4 is simply the top of the normal distribution,
and the ">4"
 bucket was an artifact of the guessed band.

Five pairs do stand clear of the distribution and remain unexplained:

```text
Zha_Ke          256 ->  4310   16.8
Mio             250 ->  2622   10.5
shihai4h       1813 -> 17764    9.8
zheermao101     615 ->  4655    7.6
MizuharaNagisa  456 ->  3030    6.6
```

Those are the candidates for genuine translator addition,
which house policy
 keeps when accurate.
Nobody has read them.

## Two magic numbers in a row, the second one better disguised

`#70` proposed re-designing the pipeline to PRODUCE a translation rather than
 repair one.
It died twice in one session,
and the second death is the one worth
 keeping.

### First death: the evidence measured the aligner

The premise was that many sections are only partly translated,
resting on a
 block-count gap of 60 of 172 pairs.
Recomputed under the forced aligner it is
 85 of 275,
and 81% of those pairs carry a full translation by character ratio.
 Block count conflates a stub with a reformatted paragraph.

### Second death: the dichotomy is false

`accuracy/omission` is already a first-class issue category,
and the recall
 benchmark is built by DELETING sentences from the published English and judging
 whether they come back against the Chinese.
Repairing an omission already IS
 translating the missing part from the source.
There is no repair mode needing a
 translate mode bolted beside it;
there is one path.

### The part I got wrong twice

Asked what should replace `#70`,
I proposed a per-entry escape hatch keyed on a
 character ratio.
The user rejected it:
a ratio keyed to a magic number
 inherently misses cases.
Correct,
and worse than they said,
because a ratio
 measures VOLUME rather than COVERAGE.
A section can sit at a healthy 2.5x while
 omitting half its sentences and expanding the rest,
and at section granularity
 it cannot see a skipped paragraph inside an otherwise sound section.

I then reached for "the measured restoration rate" as the thing to improve.
That
 is the same error wearing a lab coat.
Unpacked,
0.60 means:

```text
MIN_SENTENCE_LENGTH        40    only sentences over 40 chars can seed
descending length order          so it measures the LONGEST sentences
RESTORATION_WORD_THRESHOLD 1/2   "restored" is half the vanished words
CONTENT_WORD_MIN_CHARS     4     what counts as a word at all
denominator                      policy-declined and non-derivable removed,
                                 exclusions I adjusted the same night
```

So it reads:
of artificially deleted long sentences,
60% got at least half their
 four-plus-letter words back,
over a population I curated.
That measures the
 instrument,
not the pipeline.

### The distinction to hold

Not every number is a magic number,
and collapsing them would be its own error.
 The panel-parity result is FORCED:
six voters produce even margins because that
 is arithmetic about integers,
with no constant to choose.
Counts of things that
 exist are the same.
What is suspect is a CONSTRUCTED SCORE:
a threshold grader
 over synthetic defects with a curated denominator.
Tonight produced one of each
 and I labelled both "measured".

### What replaces it

For "can this pipeline supply missing translation",
the non-constructed
 observation is to run it on a section that is missing translation and READ the
 output.
`shi_Yumiaoya` carries three sections at ratio 0.01 to 0.02,
a thousand
 characters of Chinese against a dozen of English.
Either the critics file
 omission claims covering that and the editor supplies Chinese-derived text,
or
 it invents.
Three sections read carefully answer it;
no aggregate can,
and `PRF`
 already says to judge the content rather than trust that a generator ran.

That is n=1 and should be reported as n=1.
