# Translation repair: where the archive's note says the English is the original

Decision of 2026-09-08,
the owner's,
asked after the twelfth hakureico pass rewrote Hanasaka's letter.
Landed as `439667ec3`.

## The question

The archive page of `hakureico` carries a translator note above the letter:
`这段话以下全部，包括结尾的两句祝愿，原文都是英文，中文是反向翻译的，请仅修可能造成误解或明显的非刻意语法错误，不大修`
(everything from here,
the two closing wishes included,
was written in English;
the Chinese is a back-translation;
fix only what misleads or is plainly unintended grammar;
no heavy revision).
The note reaches the slices (the sixth class's fix carries it with its position),
and the twelfth page still rewrote the English original in five places:
`I am never gone` became `I am never really gone`,
`see this` became `see this little poem`,
`And who’s by your side w` became `And no matter who’s by your side w`,
`I will always be with you` moved to the back-translation's position,
and the archive's two closing wishes became one sentence rendered from the source.
The two short quotes under the page's other note shipped verbatim.
Nothing in the pipeline made the archive the authority for a span whose note says the archive is the original.

Measured across the pinned corpus (`a41fc607`):
22 of the 93 archive pages carry a translator note,
and 2 say the English is the original:
`hakureico` (a span,
everything below the note),
and `cheonwoomaeng`,
whose note `这篇文章的原文即英文，作者的第一语言为英语，请翻译时不要动本篇。` says the whole page is the author's
own English and must not be touched in translation.

## The options

- Span authority from the note (recommended):
  where an archive note says the English is the original,
  the archive's text from the note to the next heading ships as it stands,
  as the front matter does,
  touched by no lane.
- Leave it:
  the models see the note and the twelfth's five changes are all defensible English.
- Copy-edit lane:
  the span goes to the writers as an English original with a copy-edit brief and is judged against the
  archive's text under a change budget.

## The decision

The owner chose span authority ("1"),
and added that for the whole-page case the pipeline should refuse to repair that entry.

## The rule

- A span note (the marks `以下`,
  `原文` and `英文` together) seals the archive from the note's end to the next heading or the end of the
  page.
  The sealed blocks and the originals paired with them reach no slice;
  they are recorded on the preparation as `archiveOriginalSpans`,
  named in an alignment finding,
  kept out of the block correction round,
  and written to the artifact (generation twelve).
  The publication guard `assertArchiveOriginalComplete` refuses a page that does not carry every sealed span
  byte for byte,
  the same shape as the front-matter guard.
  An original the source carries behind a sealed span (hakureico's footnote definitions) is written after the
  span,
  not folded into the passage before it.
- A whole-page note (the marks `原文即英文` or `不要动本篇`) declines the entry before any purchase:
  the pass writes `declined/<id>.json` beside the artifacts,
  prints `TALLY <id> status=DECLINED reason=archive-original`,
  the scheduler counts the id as done on every later pass,
  `verify-published` reports `declined=N` and flags a page standing for a declined entry,
  and the archive page stands as the output,
  untouched.
- Any other note (the quotes note among them) seals nothing and reaches the sheets as the advisory it is.

## What it keeps of the standing rules

`doc/decision/translation-repair-always-yields-output.md` says no stage may refuse to produce output for an
entry and that a decline must never be silent.
The whole-page decline keeps to its letter:
the output is the archive page as it stands,
the decline is recorded where the next pass reads it,
and the tally names it.

## What it does not settle

The note is recognized by its marks,
two wordings in the pinned corpus.
A third wording would seal nothing and be logged as advisory;
adding it is a one-line change to the marks.
A sealed span that cuts a container (a `<details>` half inside and half outside the seal) stops the entry at
the container assertion rather than shipping;
no pinned page has that shape.
