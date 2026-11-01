# Translation repair history 2026-08-12 to 2026-08-15: the invisible character that welded paragraphs and the mispaired sections

How a byte-order mark line welded two paragraphs and shifted every later pairing,
with the correction of the first description,
the drift figure re-measured,
one entry's section alignment laid out,
and the finding that the entry is mispaired end to end rather than partly untranslated.

Part of [Translation repair history: 2026-08-12 to 2026-08-15](translation-repair-history-2026-08-12-to-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The invisible character that misaligned a document, and the entry mispaired end to end

### An invisible character was misaligning a document, and no stage could see it

Found by READING the drawn damage sheet,
not by any measurement.

`people/Toka_ls/page.en.md` carries three lines holding U+FEFF and nothing else.
Verified with `parseDocument`:
such a line becomes its own paragraph node,
so
the translation has a block the original lacks and every paragraph after it
pairs with the WRONG source paragraph.

The drawn item shows what follows.
The source slice began at 期盼中 while the
 target slice began two paragraphs earlier,
and the editor,
mapping them
 positionally,
replaced a correct rendering of 期盼中，她看见光穿透暗影 with a
 faithful rendering of 尽管前路漫布荆棘,
a sentence three lines away.

Nothing downstream can catch that.
Both texts are fluent,
both translate
 something the source really says,
and the critics,
the checkers and the probe
 are all comparing against whatever paragraph the misalignment handed them.
A
 source-anchored probe does not help,
because the source it is anchored to is
 the wrong paragraph.

Fixed in `buildDocumentNodes`:
a block whose every character is whitespace or
 invisible is dropped.
Filtering runs AFTER the map so a surviving node keeps
 the index it had among the parser's children,
because accepted issues anchor to
 `block/N` and renumbering would repoint every claim recorded against an earlier
 parse.

Guard proven by removal:
with the filter disabled the fixture parses to three
 nodes instead of two and ids read `block/0, block/1, block/2` instead of
 `block/0, block/2`.
Restored and green.

Scope:
one entry of the corpus carries the character,
and that entry was drawn
 three times into the twenty-item sample,
so up to three of those items sat on a
 misaligned pairing.
The sample is being redrawn.

### Correction: the invisible character WELDS paragraphs, it does not split them

The entry above this one described a lone U+FEFF becoming its own paragraph.
That is what a fixture with blank lines either side does,
and it is not the
shape the corpus contains.
The corpus shape has ordinary sentences directly
above and below with no blank line,
and a line carrying a byte-order mark is not
blank to CommonMark,
so it reads as a CONTINUATION and merges the two
paragraphs into one block.

```text
REAL shape   (mark line, no blank lines around it)   1 block   two paragraphs MERGED
fixture I wrote (blank lines around it)              2 blocks
plain blank line                                     2 blocks
```

So that translation parsed to 29 blocks against the original's 33.
They track
 one-to-one to index 9,
and after the first weld every English block pairs with
 the wrong Chinese one,
which is how a correct rendering of 期盼中，她看见光穿透暗影
 was rewritten into a faithful rendering of 尽管前路漫布荆棘,
two blocks away.

The first fix dropped invisible-ONLY blocks,
a shape the corpus never produces.
It has been removed rather than kept:
a guard that cannot fire reads as
protection that is not there.
The real fix blanks such a line to spaces before
parsing,
preserving length exactly as `maskHtmlComments` does,
because node
text,
quotes,
hashes and claim anchors are all sliced by absolute offset.

After the fix that document reads 33 against 32 and the slice pairs correctly:

```text
期盼中，她看见光穿透暗影…      In her anticipation, she saw the light…
在心灵最深处…                  In the deepest recesses of the soul…
尽管前路漫布荆棘…              Despite the thorns that litter the path…
或许她的世界里…                Perhaps in her world, the light is…
```

### Two traps worth keeping

ECMAScript counts U+FEFF as WHITESPACE,
so `'\u{FEFF}'.trim()` is empty and a
 whitespace-first check skips the character it is hunting.
The first draft of
 the masker blanked nothing for exactly that reason.

A guard proven by removal proves the guard RUNS.
It does not prove the guard
 addresses the defect,
because the fixture came from the hypothesis rather than
 from the corpus.
Both the removal proof and the test passed while the corpus
 case went untouched.

### The drift figure re-measured

`#69` was recorded against the broken parser and flagged as suspect.
Re-measured
 with the fix:
60 of 172 aligned pairs differ in block count,
against 61 before,
 so the welding accounted for exactly ONE pair.
The premise stands and the
 largest gaps are unchanged,
including a section with 76 source blocks against
 5 target blocks.

### What one entry's section alignment actually looks like

`XingZ60`,
rebuilt with the corrected parser.
Thirteen aligned section pairs,
 source against target,
in blocks and characters:

```text
pair  0    10 blocks /  558 chars    5 blocks /  1024 chars
pair  1     5 blocks /  223 chars    8 blocks /  1042 chars
pair  2     2 blocks /   89 chars    4 blocks /   612 chars
pair  3    62 blocks / 4641 chars    1 block  /    22 chars   <- heading only
pair  4     2 blocks /   73 chars    2 blocks /   137 chars
pair  5    20 blocks / 2908 chars   62 blocks / 14080 chars
pair  6     7 blocks /  763 chars    2 blocks /   105 chars
pair  7    18 blocks /  613 chars   20 blocks /  9551 chars
pair  8     5 blocks /  255 chars    6 blocks /  1712 chars
pair  9     9 blocks /  643 chars   18 blocks /  1434 chars
pair 10    76 blocks / 3483 chars    5 blocks /   719 chars
pair 11     6 blocks /  915 chars    9 blocks /  1931 chars
pair 12    23 blocks / 1459 chars   14 blocks /   933 chars
```

Pair 3 is the sharpest case in the corpus so far.
Its entire English side is the
 string `## Memories by Friends`:
a heading with 4641 characters of memorial
 essay untranslated beneath it.
The pipeline treats that as a translation with
 defects in it.

The gap runs BOTH ways,
which the earlier framing missed.
Pair 7 holds 613
 characters of original against 9551 of translation,
and pair 5 holds 2908
 against 14080.
A translation five to fifteen times its original is not
 explained by English being wordier than Chinese,
so either those sections carry
 content the original does not,
or section alignment is pairing the wrong ones.
Neither has been established,
and the artifacts cannot settle it because they
 were written before the parser fix.

This matters for `#70` beyond the untranslated case:
a re-design premised on
 "translate what is missing" answers pair 3 and pair 10,
and says nothing about
 pair 7.
Whatever shape is proposed has to state what it does when the
 translation carries far MORE than the original.

### XingZ60 is not partly untranslated. It is mispaired end to end

Correcting what this document said earlier tonight.
`alignDocumentSections`
 pairs every section of that entry with the WRONG one,
shifted by two:

```text
### 其一：伊良子      || ## Engagement in Trans Aid
### 其二：铃语        || ## Memories by Friends
### 其三：绘都        || ### Irako
### 其四：无常        || ### Lingyu
### 其五：东云        || ### HiYku
### 其六：Mikä       || ### Ann
### 其七：wing       || ### Shinonome
### 其八：白毛 suki   || ### Mikä
### 其九：空白        || ### wing
### 其十：锦心        || ### Baimao suki
## 致曾划过夜空的流星   || ### __
```

The correct pairing is legible from the headings themselves:
其一：伊良子 with
 Irako,
其二：铃语 with Lingyu,
其六：Mikä with Mikä,
其七：wing with wing,
 其八：白毛 suki with Baimao suki.
The English headings carry the romanised
 names outright.

Cause:
the Chinese has 14 headings and the English 12,
missing two sections at
 the END.
Facing 15 source chunks against 13 target chunks,
the aligner reports
 `structure-mismatch` and falls back to aligning PROPORTIONALLY BY CHARACTER
 FRACTION,
which slides everything by two.

### What that invalidates

The section I called the worst untranslated case is not untranslated.
`其二：铃语`
 has a full English rendering in the same file under `### Lingyu`,
four
 occurrences of the name included.
The translate prototype rendered a section
 that was already translated,
so its output demonstrates capability and NOT a
 gap being filled.

For this entry every critic call compared the wrong original against the wrong
 translation,
so every issue filed against it is noise,
every repair damaged
 correct text,
and the probe agreed because it was handed the same wrong source.

The signal existed the whole time.
The artifact carries
 `alignment structure-mismatch` and two `sections-merged` findings,
and an
 earlier measurement in this document counted 5 of 56 entries carrying an
 alignment finding.
Nothing reads them.

### What it means for `#69` and `#70`

Both were premised on sections whose translation covers a fraction of their
 source.
Some of that is real,
and some of it is this.
The block-count gaps
 measured across the corpus,
60 of 172 pairs,
were computed from the same
 possibly-shifted pairing,
so the figure describes alignment quality as much as
 translation coverage.

Fixing alignment now outranks both.
A pipeline that pairs sections wrongly
 cannot be repaired into correctness by any later stage,
and cannot be
 re-designed around either.
