# The translate lane deletes English with no original, and deletion is the only decidable half

Measured 2026-08-18 across the pinned corpus and both settled artifact pools.
No corpus text is reproduced here:
the passages live in the corpus and the artifacts,
and this document carries counts and structure only.

## What goes wrong

The translate lane writes each slice fresh from its Chinese source.
Where a human translator added English the Chinese never had,
typically a transcript of an image,
that passage has nothing to produce it and the shipped slice simply does not contain it.

```text
dogesir_/translate       3493 -> 2042 characters   -41.5%
wangzihao980/translate   1952 -> 1091 characters   -44.1%
```

The repair lane retained the same passages,
so the content is retainable and only one lane destroys it.

## The guard that should have caught it did not

`assessSliceAlignment` refuses a replacement above `MAX_INCUMBENT_TO_SOURCE_RATIO`,
which is 16.

```text
dogesir_ slice 3        114 source code points, 1766 incumbent, ratio 15.49
wangzihao980 slice 4    141 source code points, 1228 incumbent, ratio  8.71
```

A near miss on one and nowhere near on the other.
A ratio cannot express this,
because the archive being long is not the problem;
the archive carrying something with no original is.

## Classifying target-only content is not cheaply decidable

The first attempt anchored on the source's own last block repeated in the archive,
and treated what followed as target-only.
Checked against every transcript now known in the corpus,
nine across six entries,
it reaches two.

```text
wangzihao980    caught cleanly
zheermao101 q2  caught, but sweeps six trailing paired blocks with it
dogesir_        caught only because the anchor comparison folds whitespace
Zha_Ke          MISSED: the transcript sits BEFORE the shared markup
MizuharaNagisa  MISSED: same inversion
zheermao101 q1  MISSED: interior, a later anchor in the same document supersedes it
Mio q1, q2      MISSED: same
shihai4h        INVISIBLE: merged into an already-paired block
```

Three distinct failure modes,
and a fourth reason the whole approach is unsound:
"has no source counterpart" cannot be decided by matching bytes,
because translation changes bytes by construction.
A corpus-wide scan for archive quotes with no exact source match returns 234 hits,
almost all of them ordinary translated quotations that ARE positionally paired.
Deciding correspondence properly is what the aligner does,
so a cheap structural rule is either the aligner or it is wrong.

## Deletion, on the other hand, is decidable from the two texts alone

Counting blockquote BLOCKS,
not lines,
so a lane may reflow a quotation freely:

```text
flagged pool   4 of 64 shipped replacements drop a whole quote block
               dogesir_/translate#3, lintong/translate#1, lintong/translate#3,
               wangzihao980/translate#4          all four in the translate lane
natural pool   0 of 69
```

Four hits,
every one a passage a reader would want back,
and nothing else across sixty-nine natural rows.
Reproduced through the built artifact rather than a copy of the logic.

THE COUNTER-CASE KEEPS THE GUARD NARROW.
Deleting is not always wrong.
On `saurikissa` the repair lane removed a paragraph of translator invention
with nine separate accepted findings against it,
and that was correct.
It was prose rather than a quotation,
so this guard would not have stopped it.

## The pipeline already knew, in one lane

On `wangzihao980` the repair lane's critics were asked whether the transcript was an improper addition
and rejected the claim 5 votes to 1.
On another entry the repair lane met a translator's own comment,
recognised it as an editorial question,
and recorded it as `needs-human` rather than acting.
The translate lane deleted both with no flag at all.

The harm is not that the pipeline cannot tell invention from archive content.
One lane can.
The harm is that the other lane never asks,
because a passage with no source span is invisible to a stage that reads only source spans.

## A separator bug found on the way

Of the 184 markdown files in the pinned corpus,
one uses CRLF throughout.
A block splitter looking for the two-byte sequence `\n\n` finds no boundary there,
reads the whole document as a single block,
and counts zero quotes.
That is worse than an error,
because a guard counting zero reports nothing wrong.
The shared splitter now folds carriage returns first.
