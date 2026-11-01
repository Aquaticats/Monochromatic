# Translation repair: the block floor accepts either rendering

Decision of 2026-09-07,
the owner's,
asked after the Huasheng relaunch stopped at its poem slice.

## The question

`compareBlocks` in `package/translation-repair/translate-floor/src/translate-validate.ts` requires a candidate to
carry the archive page's block sequence in order,
adding blocks only up to the larger of the page and the original.
The rule was written on 68 settled slice records
(48 the same,
11 archive more,
7 archive fewer)
so that an archive which merged the original's paragraphs is not undone,
and it was pinned by a case in which the archive's blockquote says a passage was left by someone
rather than written.

Huasheng's poem "To the Eternal Star" is two paragraphs whose lines end in `<br/>` in the source and
five paragraphs with soft breaks in the archive.
Every producer followed the source,
the contest winner and the consolidation standing both carried two paragraphs,
nothing was valid,
and the entry stopped under the 2026-09-04 decision that an ineligible standing stops rather than
reattempts.
Measured at the document level over the 92 pairs,
the archive carries more top-level blocks than its source on 34 entries,
the same on 40,
fewer on 18.

## The options

- A,
  either rendering at the block level:
  the floor also accepts a candidate whose block sequence is exactly the original's.
- C,
  hard-break equivalence:
  a source paragraph carrying hard breaks may be rendered as the page's run of paragraphs or as one
  paragraph carrying the same breaks.
- B,
  a sheet clause telling the producers to keep the page's split.
- D,
  A with a size guard,
  which needs a number.

Ranking A over C over B over D,
with the reasons in
[`translation-repair-openrouter-2026-09-03.md`](../planning/translation-repair-openrouter-2026-09-03.md)
under "The Huasheng relaunch stops at slice 21".

## The decision

A.
A candidate shaped exactly as the original,
kind for kind and detail for detail,
is a faithful rendering of it whatever shape the archive chose,
and passes beside a candidate shaped as the page.
The choice between two faithful shapes is the judges',
who see both texts.
What stays refused is a shape that is neither reference's,
which is what a dropped passage looks like.

## How it is implemented, and the one bound kept

The original's shape counts only where the page's surplus is more blocks of the original's own kinds,
a split.
A page whose extra blocks are of a kind the original lacks is carrying something a split cannot
explain,
and a candidate shaped as the original would drop it:
the sixth consolidation bed's page span read as a paragraph,
an html block and a blockquote against a one-paragraph original,
and the 164-character rendering that shipped there was shaped exactly as the original.
The suite `floor-holds-on-an-unparseable-page` pins that case and stays as it is;
the pinned blockquote case,
an archive quote saying a passage was left by someone,
stays refused for the same reason.

Measured at the document level over the 34 archive-more entries:
22 add only more blocks of the source's own kinds,
Huasheng among them,
and 12 add a kind
(a blockquote,
a heading,
a list,
a thematic break,
a footnote definition,
an expression).
The bound therefore covers the shape that stopped Huasheng and 21 entries like it,
and leaves the 12 under the page-as-floor rule.

## What it costs

A paragraph the archive added with no source counterpart,
of a kind the original has,
becomes droppable by a source-shaped candidate;
references and footnotes the archive added are still kept by the atom floor.
Accepted.

## Veto

Widening to the answer as asked,
every page and not only a split-only one,
is the removal of one condition in `compareBlocks` and the flip of the blockquote case;
reverting is the removal of the other.
The record stays either way.
