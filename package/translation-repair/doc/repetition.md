# Repetition and the site's grammar

Part of [the family README](../README.md).

## Repetition the pipeline introduced

Every per-slice instrument in this package is structurally blind to a passage said twice,
because each works inside **one** slice and the duplication is inside no single slice.
Two checks run at assembly,
where the whole document is visible.
Neither spends quota:
both compare strings against the archive the artifact stores.

The archive is what makes this decidable.
Real writing repeats,
in refrains,
names and deliberate echoes,
so a standalone "says it twice" rule would fire on all of them.
Counting against the archive asks the only question worth asking,
whether the pipeline **added** a repetition,
and inherits the author's own judgement about acceptable repetition for free.

### Document scale, with a content gate

`findIntroducedRepetitions` compares phrase counts across the whole document.
Any two distant sentences may share ordinary phrasing,
so a phrase must carry at least two words of five or more letters to be reported.
Without that gate the check returns mostly noise.

### Adjacent slices, with no content gate

`findAdjacentRepetitions` asks a much narrower question:
did two **consecutive** slices ship the same wording,
which the archive did not repeat.

It has no content gate,
and it must not have one.
The duplication this package was built to catch carries no word of five letters at all,
so the document-scale check cannot see it at any setting.
Adjacency supplies the specificity the content gate supplies at document scale.
Measured over every settled artifact carrying a delivery ledger,
it fires once in twenty-two lane readings,
and that once is the known damage.

Both checks run in **both** lanes.
Writing a slice from its source rather than editing an incumbent does not stop a lane saying the same thing twice.

Findings carry the slice pair and the measurements and never the wording,
because a findings list travels into logs and artifacts where corpus text does not belong.

Two instruments are the exception,
by design,
and their standard output is an artifact rather than a summary:
`coverage-probe` prints its rows as JSON,
whose `evidence` is the document's own matched text,
and `judge-fidelity-probe` prints per-trial judge `reasons`,
which are model prose quoting candidates.
Redirect both to a file under the runs directory and never paste either into a log,
a commit,
or a chat;
both also persist their rows through the probe store,
so the redirect is a convenience rather than the record.

## The site's grammar is not this one

The corpus repository builds each `page.md` itself:
its `scripts/build.ts` rewrites `<!--` and `-->` into JSX comment delimiters
and `scripts/mdx.ts` compiles with `@mdx-js/mdx` under `remark-math` and `rehype-katex`,
with no GFM.
This package parses with `remark-mdx` plus `remark-gfm` after masking HTML comments to whitespace.
Verified with the site's own renderer on 2026-08-26:
a footnote reference compiles to literal text there and to structure here,
a `$...$` pair compiles to math there and to prose here.
Six source pages at the pin carry a math pair.
Whether to reconcile them is open;
nothing published changes either way,
since the text is preserved as written,
but a formula is unprotected structure until the strict grammar knows it.
