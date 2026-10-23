# Audit ledger: the source text under a note and the page grammar

Findings on making a page carry the original of an untranslated passage.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B832: the page could stop over an original the grammar refuses, and four texts said more than the code did

Status:
fixed on 2026-10-23 (UTC),
except the count of unendorsed standings,
which is a flagged choice.
The rulings of this entry are the lead's,
open to the owner's veto.
Follow-ups of 2026-10-23 (UTC) close the holes between the plain parse and the strict grammar that the encoder left,
add the wiring tests and settle the style findings (see the headings that start "Follow-up").

Commits that let the page ship over an original the grammar refuses:
reds `a024fdf1d0` (the default of the helper),
`4edf0025ab` (the finding text)
and `dbcf9aed27` (the grammar,
the text the page carries,
the case about the page passes);
fixes `0763fa31b8` (the finding text),
`449bfb369d` (the grammar,
the helper,
the text the page carries,
style findings,
the case about the page passes,
reworded in `fe2deb3d66`),
`93ecd78335` (the count of unendorsed standings,
the comment and a pin case),
`be1c689889` (style findings,
`run`),
`2eea925d80` (the finding text,
the log and the comment,
style findings in `translate-lane`)
and `0257149051` (the consistency pass over a page cut short,
a test only).

Commits that read the encoder from a parse and ship a page without the runs no writing makes parse:
`11ca30f13c` and `0b3611ebd8` (the plain reading with the site's math,
red and fix),
`c4e8d63098` and `21a74ece11` (the reader's account of the shipped page,
the encoder read from a parse,
red and fix),
`a1958cfced` and `a4582b3e42` (a comment that never closes,
red and fix),
`3b433336ad` and `a2853f7852` (a page shipped without runs,
red and fix),
`ba66757172` and `896e6c8346` (a page shipped without runs that parse alone,
red and fix),
`0e4ea9ffdd` and `3a3299d034` (the finding text,
red and fix),
`f400095ab1` (the case about the page passes)
and `3610d9ff39` (what takes steps 2 and 3,
style findings).

Commits of the follow-up:
`8cd54f4871` (main merged at `98395c8555`),
`b3f6b7ce51` and `95b8ae45f6` (the holes between the plain parse and the strict grammar,
red and fix),
`f7f20ef969` (the wiring tests and the cases for `includeSource: false`)
and `0c46a09eb0` (what takes steps 2 and 3,
the no-op sink,
style findings).

The red `dbcf9aed27` was wrong in two ways that its fix `449bfb369d` corrects:
it left the source-scan suite red (two top-level `describe` calls in one file),
and a case asserted that the warnings numbered exactly one where the composition's line is one of several;
the cases now filter the composition's own lines.

### The grammar can refuse the original

`assertPageParses` stopped the entry with `UnparseablePageError` when an original held a bare `{`,
an unclosed `{`,
an unclosed `<details>`,
a lone `</details>` or an indented `{x} <y>` line.
`composeSourcePage` ships each run as recorded when the page parses,
and otherwise:

1.  writes,
    as literal text,
    only the runs that do not parse as a fragment of their own;
2.  writes every run as literal text;
3.  ships the page without the source text of the slices and adds the defect `page-source-unparseable`.

Each step taken writes one warn line naming the slices and the step.

The step 1 text is not only the runs the grammar refuses:
a run that opens an HTML comment which never closes also goes to step 1,
because the package masks everything after such an opener and a page that "parses" with it hides the rest from the
package's own parser.
The opener is written `&lt;!--`;
a backslash before the `<` does not stop the mask,
which looks for the four characters
(a probe with `\<!--` still found the unterminated comment and hid the English after the run,
`&lt;!--` did not).
The grammar is acorn through `micromark-extension-mdxjs`,
which `remark-mdx` lists in its `package.json`;
the module comment of the encoder once said that no JavaScript parser stands behind the grammar,
which was wrong.

#### What is written, and where it is read from

The literal text is read off a CommonMark parse of the run with the site's math and no MDX
(`parseMarkdownMathBody` in `document-structure`),
by the position of each node,
and not off lines.
A backslash goes before each `{`,
`}` and `<` that the parse reads as text.
Fences wherever they stand (in a list item,
in a block quote),
code spans across lines,
math,
link destinations and titles,
bare addresses,
closed HTML comments and escapes the original wrote are copied as they are.
An angle autolink,
which MDX reads as a tag,
is written as a link with the same text and target.
A paragraph at the first column that starts `import ` or `export `,
which MDX reads as an ES module and refuses when it is not JavaScript,
has the first letter of its first word written as a character reference
(`&#101;xport to PDF`);
a valid `import Foo from "./foo.js"` ships as written.
`>` is not escaped,
because it is syntax only inside a tag and a block quote of the original would otherwise turn into a line starting
with the sign.
`<Foo />` and a balanced `{他说}` ship as written when the run parses,
since the original is a page of an upstream MDX site that compiles;
only a run the grammar refuses is escaped,
and there every brace and less-than sign in text is.
Known limits are in the module comment:
a backslash the original wrote before a character in indented code is read as an escape,
a raw HTML block that holds Markdown-looking lines is read as Markdown,
and a text nested past the bound of the plain reading is escaped as text throughout.
The bytes of the original differ from the artifact's only on the encoding paths.
Each case compares what a reader of the shipped page sees with what a reader of the source as plain Markdown sees
(`source-reader-text.test-fixture.ts`).

#### What takes steps 2 and 3

Two senses are kept apart in this section:
a step is taken when the composition writes its warn line and tries its page,
and it made the page parse when the page of that step is the one shipped.

Under the real grammar the hostile cases of `source-shipped-page.unit.test.ts`
(a bare `{` in a sentence,
an unclosed `{`,
an unclosed `<details>`,
a lone `</details>` and the indented `{x} <y>` line)
all take step 1 and step 1 made the page parse;
the control that parses ships as written;
a page with one parsing run and one failing run takes step 1 and encodes only the failing run's slice;
a `<details>` pair split across two slices of one run ships as written.

A seeded search of forty thousand pairs of runs,
built from a pool of fragments
(braces,
tags,
comment markers,
fences and fence marks indented four spaces,
module lines and setext headings that start with a module word,
autolinks,
bare addresses ended by a tag or holding a brace,
brackets,
footnote definitions CommonMark reads as link reference definitions and the blocks indented under them,
reference labels with a brace,
`$$` flows,
and block quote,
list and bracket markers nested 260 and 300 deep),
with English between the runs that parses,
gave these counts:

- 6887 pairs shipped as written;
- step 1 was taken for 31868 pairs and made the page parse for 26457 of them;
- step 2 was taken for 838 pairs and made the page parse for 32 of them;
- step 3 was taken for 6624 pairs.

Step 2 is taken when every run parses alone,
the page does not,
and writing every run as literal text changes some run.
It made the page parse for 32 pairs,
31 of them with a fence mark indented four spaces in both runs:
MDX has no indented code and reads such a mark as a fence,
the plain parse reads a paragraph or indented code there,
so a brace between two such marks is text to the strict grammar once the marks pair across the runs,
which writing every run so escapes.
The older statement that step 2 is taken by no text was wrong:
it was taken for 838 pairs,
for the most part a run nested past the bound beside a run holding a character that step 2 escapes,
and for the rest a pair of fence marks.

Step 3 is taken for these causes,
the counts being the pairs of the search that name each cause by the fragments they hold:

- a run nested past the bound of 256 that the plain reading keeps:
  6223 pairs,
  the 260 and the 300 deep alike,
  pinned by the 300-deep case in `source-shipped-page.unit.test.ts`;
- a fence mark indented four spaces or more:
  218 pairs;
- a raw HTML block that runs to a closing marker (`<script>`,
  a comment that never closes):
  52 pairs;
- a `$$` flow that one run opens and a later run closes,
  with the English between them falling inside it and a tag after the closing line:
  46 pairs.
  A fence does the same,
  which the fence case in `source-shipped-page.unit.test.ts` pins;
  the search found no pair of that shape among the fences,
  which is a finding about its pool and not about the shape;
- the remaining 85 pairs,
  which come down to the shapes the encoder does not meet,
  named in the next heading.

#### Shapes the encoder does not make parse

Found by encoding each run of the remaining pairs alone and reading what stayed refused.
Each is a limit of the module comment of `source-ships-encoding.ts` and is not fixed:

- A raw HTML block or indented code that holds `[https://example.net/a{ b](u)`:
  the plain parse reads one text and takes the address in it for a bare one,
  where the strict grammar reads a link and its text.
- A line CommonMark reads as a link reference definition right after a footnote definition:
  the strict grammar takes it for a line of the footnote.
- A bare address after a bracket nothing closes (`[ https://example.net/b{c`):
  the strict grammar does not read an address there,
  so the brace in it is syntax.
- A code span or inline math across lines of which a later line starts with `{` or `<`:
  an expression or a tag at the start of a line interrupts a paragraph in the strict grammar,
  so the span is cut there.
- A byte order mark line before a setext heading that starts `import`.

#### Step 3 and the page on disk

Step 3 ships the page without the runs no writing makes parse
(the runs the grammar refuses even as literal text),
and without every run where that does not parse or no such run is found.
`composeSourcePage` returns the artifact the page is weighed against (`weighed`),
which is the artifact without the originals of the slices left off
(`withoutSourceRecordAt`),
so the agreement check at publish time does not report them.
A page on disk is judged by the same composition run over what the artifact records
(`composeOverTiling`,
the archive taken as the slices' archive wordings end to end):
a page that disagrees as recorded and that the composition would ship without runs is judged against the artifact
without those originals,
and reads as agreed and of the right weight;
`verify-published` prints one line naming the entry,
`page-source-unparseable` and the slices.
A page that never carried originals still disagrees wherever the composition keeps them,
so a pass publishes it again;
a first attempt,
a bare fallback to every run dropped,
failed that control.
The residual is stated in the TSDoc of `pageAgreement`:
the composition runs over the archive the slices tile and not over the archive itself,
and a page that lost every original it should carry,
where the composition would also ship without them,
reads as shipped without them and not as lost.
The first design had `pageAgreement` compute the expected page through the composition;
it has no slice offsets,
so it runs the composition over the tiling.

#### The original is never rewritten by a page pass

Every page pass on the real publish path reads `shippableReplacements` with `includeSource: false`
(`page-assembly-guard.ts`,
`page-footnote-integrity.ts`
and `carried-insertion-completeness.ts`),
and `publishFixedPage` runs no pass after the composition,
so no pass rewrites a shipped original and no exemption was needed.
A case with a Windows line ending and `March 5th, 2020` ships the original byte for byte,
and fails when `guardPageAssembly` is made to carry the original
(under the flip the guard trims the slice with a finding for the folded line ending and the rewritten date);
the older footnote case fails under the same flip.

### Smaller fixes of the entry

- The text the page carries:
  the docs say that the page carries each slice's text as the artifact records it,
  joined by one blank line,
  and that text between slices is not carried;
  the test is renamed,
  and a case with a single newline between slices is added.
- The default of the helper:
  `shippableReplacements` defaults `includeSource` to `false`;
  the guards and the consistency pass cannot carry the original,
  and only `publishFixedPage` passes `true`.
- The finding text:
  the finding `source-passage-unfilled` and its TSDoc say the page carries the source text under a note
  unless the source recorded for the slice renders as nothing or the artifact records none,
  because the only caller is in `corpus-pass/src/corpus-run/pass-entry.ts`,
  which that fix does not edit,
  so the reading cannot be passed in.
  The comment and the warn in `translate-lane/src/translate-slice-buy.ts` say the same.
- Style findings:
  lines over 120 characters,
  article-opening `@param` and `@returns`,
  a ragged tail,
  an empty suite name and a double cast.
- The case that shows the original stays out of the page passes:
  the first case (English-looking words and straight quotes) passed under the flip that makes the guard carry the
  original,
  so it guarded nothing,
  and its text claimed otherwise.
  The case now holds an original with a Windows line ending and `March 5th, 2020`,
  which the line-ending fold and the Canadian-form pass rewrite under the flip,
  and its name no longer narrates a measurement.
  It passes against the default and fails under the flip.
- The consistency pass over a page cut short:
  a test over the cat page with its last section cut from the archive shows that the consistency pass reads that
  slice as carrying no English,
  writes nothing there,
  leaves the refrain place there unlocated,
  and carries neither the original nor the note into the slices it rewrites (the pass wrote two,
  the positive control).
  It was not shown to fail,
  because making the pass read the original takes an edit to the fenced pass source.

### The count of unendorsed standings: a flagged choice, not implemented

The ruling asked that `standingUnendorsed` leave out the slices that carry the
`ineligible-standing-replaced-by-incumbent` finding.
The artifact cannot support that:
`ArtifactConsolidateSlice` has no `findings` field,
the finding text is in no field of the XingZ6020 artifact,
and the only trace is the shipped kind `incumbent`,
which also marks the archive-dispute stand-in that the stage warns about.
The options:

- Add the finding to the consolidation record.
  Pros:
  exact.
  Cons:
  a change to what the generation writes,
  which is a generation question for the owner.
- Leave out every slice of shipped kind `incumbent`.
  Pros:
  needs no new field.
  Cons:
  the count on XingZ6020 would drop by up to the 6 slices of that kind and then contradict the stage's lines for
  the archive-dispute ones,
  which B831 matched.
- Reword the comment and keep the count.
  Pros:
  true to the artifact and no behaviour change.
  Cons:
  the count can exceed the lines.

Ranking:
reword the comment and keep the count > add the finding to the record > leave out the shipped kind,
because the first is true today and changes nothing a reader relies on,
the second is exact but moves a persisted shape on a decision that is the owner's,
and the third trades a documented count for a different wrong one.
The first was taken (`93ecd78335`),
and a case pins that an `incumbent` slice is counted.

Recurrence:
a comment that says a count "asks the same question the log did" is checked against what the artifact records.
A claim about a page is checked against what a reader of the page sees.

### Follow-up: the holes between the plain parse and the strict grammar

A review of the encoder found texts it left refused,
and texts that parse and read unlike the source.
Each became a reader case in `page-check/src/corpus-run/source-ships-encoding.unit.test.ts`:
the reader text of the encoded run under the real grammar equals the source read as plain Markdown,
and the page parses.
The cases were written first and failed;
the encoder then closed each hole.

- A footnote definition that CommonMark reads as a link reference definition
  (`[^1]: {`,
  `[^1]: <b>`,
  `[^1]: <https://example.net/x>`,
  `[^1]: a{b`
  and `[^1]: 注 ({)`):
  its body is encoded as the strict grammar reads it,
  with footnotes on,
  so `[^1]: <https://example.net/x>` ships as a footnote whose text and link target equal the source's.
  The blocks indented under a footnote definition are read the same way,
  and a fenced block in a footnote body keeps its braces without a visible backslash
  (`source-ships-footnote.ts`,
  `source-ships-spans.ts`).
- A setext heading at the top level that starts `import ` or `export `
  (`import data from the old system` over `===`):
  MDX reads it as an ES module,
  and it has its first letter written as a character reference like a paragraph does
  (`source-ships-module-line.ts`).
- A bare address that a less-than sign ends (`https://example.net/a<b> {`):
  a backslash before the sign is read as part of the address,
  and so is an entity written for it,
  because GFM ends an address at an entity only when nothing follows it.
  Writing `&lt;` was the first idea;
  it does not give the same text,
  so the address is written as a link with the same text and target and the sign after it is escaped
  (`source-ships-addresses.ts`,
  `source-ships-links.ts`).
  A bare address in the text of a link (`[https://example.net/a{ b](u)`)
  and a protocol with nothing after it (`https://{x`) are left alone,
  since GFM reads no address there.
- An angle autolink in the text of a link (`[<https://example.net/a>](u)`) is written as text,
  since a link cannot hold a link,
  and a reference label with a brace (`[a {b][]` with `[a {b]: u`) is written the same way where it is used and
  where it is defined,
  so the reference still resolves.
- The encoder asks whether a text is nested past the bound of the plain reading
  (`firstNestingExcess` of `translation-repair-markup`)
  and no longer catches the parse error for it,
  so no caught value goes unused.

The reader of the source text in the cases (`page-check/src/corpus-run/source-reader-text.test-fixture.ts`)
reads the source on the site's flavour,
CommonMark with GFM and the site's math
(`document-structure/src/gfm-math-reading.test-fixture.ts`),
and no longer re-wraps a math node in `$`.
`parseMarkdownMathBody` has no GFM,
so a footnote definition made of one address was a link reference definition there and a footnote on the page;
the cases would have compared different texts.

The search of steps 2 and 3 was run again after the fixes,
with the pool extended by every shape named in this follow-up,
and the shapes it still finds are recorded under "What takes steps 2 and 3"
and "Shapes the encoder does not make parse".
No cache version moved:
the encoder's output lives only in the pages on disk,
and no stage keyed by a prompt or a sheet changed.

### Follow-up: wiring tests that fail when the wiring goes

Each case passes on the code as it stands and was shown to fail under a mutation of the wiring it guards.
The mutations were made in the working tree and undone,
and each failing line is quoted.

- `run/src/corpus-run/verify-published-entry.unit.test.ts` writes a page the composition ships without the original
  of its last slice,
  with its artifact (`sourceDroppedRunOf` in `page-check/src/corpus-run/published-run.test-fixture.ts`),
  and expects the line that names the entry,
  `page-source-unparseable` and the slice.
  With the spread of `sourceDroppedLinesOf` deleted from `judgePublishedEntry` it fails:
  the expected line `CatEntry1: page-source-unparseable, the page ships without the source text of slices [1]` is
  only in the expected lines.
- `page-check/src/corpus-run/page-republish.unit.test.ts` expects `republishSettledPages` to leave that page as agreed.
  With `pageAgreement` made to return the verdict as recorded,
  ignoring `sourceDropped`,
  it fails at the republish:
  the outcome is a rewrite for a disagreement,
  where `{ kind: 'agreed' }` was expected.
- `page-check/src/corpus-run/page-footnote-integrity.unit.test.ts` and
  `page-check/src/corpus-run/carried-insertion-completeness.unit.test.ts`
  hold a case each for the original that an unfilled anchor ships.
  With `includeSource: true` in place of `false` in the call of `shippableReplacements`
  in `page-footnote-integrity.ts` and in `carried-insertion-completeness.ts`,
  the footnote case throws where none is expected and the carried case does not throw where one is.

### Follow-up: style findings

Lines over 120 characters,
article-opening `@returns`,
one-word tails of paragraphs,
a spelling,
a hand-written silent logger in `source-ships-tiling.ts` (now the no-op sink of `module-logger`),
and statements in this ledger that pointed to a round or a time were corrected.
The checks that found them were run over the added lines of the whole branch.

## Corrections to A8, B827, B830 and B831

### A8

The header and the finding text of `publish-completeness.ts` say that the page carries the source text
only where the artifact records a source that renders as something,
and that a page the grammar refuses with the source on it ships without it (ledger B832).

### B827

- The text of B827 that says the passes read the page with `includeSource: false` is now the default
  of `shippableReplacements`,
  which moved to `page-check/src/corpus-run/shippable-replacements.ts`;
  only `publishFixedPage` passes `includeSource: true`.
  The consistency pass cannot carry the original,
  for two reasons that the new test in
  `corpus-pass/src/corpus-run/pass-consistency-source-ships.unit.test.ts` shows:
  the artifact it composes has no `lanes`,
  so it reads the slice as carrying no English,
  and the helper it shares with the guards leaves the source text out unless asked.
- The two citations for "a slice it leaves unfilled is written to no cache" are
  `slice-cache/src/corpus-run/entry-reattempt.ts` and `translate-lane/src/translate-slice-buy.ts` (lines 168 to 191
  at the fix of this follow-up),
  where the slice returns `unfilled` and the warn says it is not cached.
- "The source text follows as written" overstated what the page carries.
  The page carries each slice's text as the artifact records it,
  the slices of one run joined by one blank line,
  and no text between slices (an HTML comment the original holds between two blocks is not on the page).
  The docs now say so,
  and a test keeps each slice's text and the join.
- The page grammar can refuse the original (a bare `{`,
  an unclosed `{`,
  an unclosed `<details>`,
  a lone `</details>`,
  an indented `{x} <y>` line),
  and `assertPageParses` then stopped the whole entry.
  The ruling:
  the page ships each run as recorded whenever it parses;
  otherwise it takes up to three steps,
  parsing after each (the runs that do not parse alone as literal text,
  then every run as literal text,
  then the page without the source text and the defect `page-source-unparseable`),
  with one warn line for each step taken,
  and the entry never stops on this path.
  The bytes of the original differ from the artifact's only on those paths.
  Measured against the real grammar,
  every hostile case of `source-shipped-page.unit.test.ts` takes the first step;
  the claim that no text takes the second or the third was wrong for both
  (see "What takes steps 2 and 3" in B832).

### B830

The subject of the fix commit says "structure and section-pairing",
and the code logs every finding that begins `alignment ` (every kind the aligner writes) and every
`section-pairing` one,
so the subject names fewer kinds than the code logs.
The commit is not amended;
this paragraph is the corrective comment.

### B831

The comment at the call said the stage writes the line only for a valid standing,
so the count asked the same question.
The artifact does not record that.
The stage's findings are not projected per slice (`describeConsolidateSlice`),
and a shipped kind `incumbent` is written both where an ineligible standing was replaced by the incumbent
(a different warn,
no "remains retryable" line) and where an archive dispute made the incumbent stand in over a valid
standing (the line is written).
So `standingValid` is `true` for every kept standing,
the count can exceed the lines by the first kind,
and the comment and the TSDoc say so.
On the XingZ6020 artifact the consolidation record holds 49 slices ending `slate-declined-standing/unchanged`,
6 ending `slate-declined-standing/incumbent`,
7 `consolidated` and 1 `gate-kept-standing/unchanged`
(a read-only script over `XingZ60.json`).
