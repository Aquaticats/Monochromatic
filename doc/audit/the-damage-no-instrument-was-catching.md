# The damage no instrument was catching

Date:
2026-08-20.
Zero quota:
settled artifacts and the pinned corpus.
No corpus text:
counts only.

## What happened

On `saurikissa` slice 4 the baseline repair lane replaced a hundred and thirty-six words
with twenty-one.
What it deleted was the concrete detail of a life:
what she wore,
what she collected,
how she kept it.
This is a memorial page.

EVERY INSTRUMENT IN THIS PACKAGE MISSED IT.
The document lost seventy characters out of four thousand four hundred and seventy-nine,
because the editor wrote generic prose where the detail had been.
So the length looks fine,
the structure is intact,
nothing repeats,
nothing is severed,
no footnote breaks,
and the page has simply stopped being about a particular person.

## Measuring it properly

Hand-picking eight words to search for proves nothing,
so:

DISTINCTIVE WORD = at least six letters,
used at most twice in the archive.
Rare long words carry specifics:
objects,
places,
names.
Common words survive any rewrite and say nothing about whether meaning did.

`saurikissa`,
216 distinctive words in the archive:

```text
REPAIR lane          kept   share    lost
  baseline            119   55.1%      97
  window              197   91.2%      19
  +quote fix          193   89.4%      23
  +panel evidence     181   83.8%      35
```

The baseline repair lane loses ninety-seven of them.
The relocation window cuts that to nineteen.

`lintong`,
35 distinctive words,
same direction and much smaller:

```text
  baseline             25   71.4%      10
  window               27   77.1%       8
  +quote fix           30   85.7%       5
  +panel evidence      30   85.7%       5
```

## What this settles

`#107` IS VINDICATED,
and not marginally.
The window exists to stop the pipeline deleting content on the theory that it is an
unsupported addition,
and on the metric that measures exactly that it moves the repair
lane from 55% to 91% retention.

The earlier reading,
that the window merely "halves repairs at no visible benefit",
was made with instruments that cannot see this damage class.
Half those repairs were deletions of things worth keeping.

## What it costs, stated against the same measurement

Both rules added on top of the window made retention WORSE:

```text
window            91.2%
+quote fix        89.4%
+panel evidence   83.8%
```

The panel-evidence rule also lowered panel support further
(`lintong` 72% to 65%,
`saurikissa` 52% to 51%)
having been added to raise it.
It fails on both metrics it could be judged by,
so it is reverted rather than kept on the argument that made it seem reasonable.

ONE RUN PER ARM.
The direction agrees on two metrics and two entries,
which is why this is acted on;
the exact percentages are not to be quoted as stable.

## A caveat that matters for the lane question

The TRANSLATE lane retains 46% to 54% on `saurikissa` in every arm.
That is NOT comparable to the repair lane's number and must not be read as worse damage.
The translate lane writes a fresh rendering from the Chinese,
so it may express the same detail in different English words,
and a rare-word test cannot tell re-wording from deletion.

The repair lane edits the incumbent,
so a lost distinctive word there
IS a detail removed or blurred.
The instrument is sound for the repair lane and only suggestive for the translate lane.
`doc/planning/which-lane-ships.md` must not use these numbers to rank the lanes.

## How widespread it is

The instrument found the damage on one entry.
Run over every settled artifact from the PRE-WINDOW runs,
repair lane:

```text
  saurikissa         119 /  216   55.1%   lost 97
  GLaDOSister         64 /  108   59.3%   lost 44
  lintong              25 /  35   71.4%   lost 10
  Anilovr              61 /  82   74.4%   lost 21
  Arita               205 / 252   81.3%   lost 47
  Acheron              51 /  61   83.6%   lost 10
  AmbeR_the_anpa       87 /  98   88.8%   lost 11
  dogesir_             95 / 100   95.0%   lost  5
  wangzihao980         46 /  48   95.8%   lost  2
  Chinatsu_Suzuki     210 / 218   96.3%   lost  8
  AkiraComplex         30 /  30  100.0%   lost  0

  POOLED              993 / 1248  79.6%   lost 255
```

Four of eleven entries lose more than a fifth of their specifics,
two of them more than forty percent.
This is a corpus-wide behaviour of the pre-window repair lane,
not one bad entry.

Since the window moved `saurikissa` from 55.1% to 91.2%,
the same change is expected across the pool,
and that is the strongest production-readiness argument for `#107` there is.

### What the number is not

AN UPPER BOUND ON DAMAGE,
not a count of it.

A distinctive word can leave the document innocently:
a repair may reword a phrase correctly and drop a rare word while keeping the meaning,
and a corrected misspelling leaves as a lost word while improving the page.
Nothing here separates those from deletions.

What makes the top of this list real rather than arithmetic
is that `saurikissa`'s ninety-seven were checked by reading them:
one slice went from a hundred and thirty-six words to twenty-one,
and what left was what she wore and what she collected.

So treat a HIGH retention figure as reassuring and a LOW one as a place to look,
never as a verdict on its own.

## CORRECTION: this measures against the archive, which is not the standard

Written the same day,
after the owner rejected a lane question built on the same mistake.

`doc/decision/translation-repair-output-goal.md`,
taken 2026-08-12,
decides:

> the output is judged against the ORIGINAL,
> not against how far it improved on the
> input translation.
> An input translation is EVIDENCE about what the original says and
> a starting point worth preserving where it is right,
> never the standard the result is
> measured by.

EVERYTHING ABOVE COUNTS ARCHIVE WORDS.
"Distinctive archive words kept" makes the incumbent English the yardstick,
which is the framing that record supersedes.
So the numbers are a PROXY and the language around them was wrong:
retention is not quality,
and a lane retaining more is not thereby better.

### Checking the headline case against the actual standard

`saurikissa` slice 4,
where the baseline replaced 136 words with 21.
The Chinese for that slice is one short line,
and the English the baseline shipped
renders it closely.
On the slice alone the deletion looks CORRECT.

Reading the whole Chinese document changes it again:

```text
磁带  cassette      present in the source
硬盘  hard drive    present in the source
贴纸  sticker       present in the source
手机  phone         present in the source
水手服 sailor uniform  NOT in the source
丝带  ribbon          NOT in the source
```

So the archive's slice-4 text was two things at once:
content the Chinese carries ELSEWHERE in the document,
and detail the translator added with no source behind it.

The baseline deleted both,
and only one of eight probe terms survives anywhere in its
finished document.
Content the ORIGINAL supports left the page.
That is damage on the
decided standard,
not merely on the archive proxy,
and it is precisely the relocation
defect `#107`'s window exists to prevent.

The translator's additions are covered by the second decision of 2026-08-12:
accurate detail a translator added is KEPT,
not stripped to match the original.

### What stands and what does not

STANDS:
the baseline repair lane lost source-supported content,
the window arm kept it,
and `#107` is right for that reason.

DOES NOT STAND:
"distinctive archive words kept" as a measure of quality,
and every
sentence here that treated a higher figure as a better document.
A lane could score
well by preserving translator additions the source never had,
and badly by correctly
removing them.

WHAT THE INSTRUMENT SHOULD MEASURE is whether content the ORIGINAL carries has some
English representation in the output.
That is a cross-language question,
and the
archive-word check is a cheap proxy for it that will mislead wherever the archive and
the original disagree,
which is the interesting case every time.
