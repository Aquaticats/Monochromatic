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
add the wiring tests and settle the style findings (see the headings that start "Follow-up"),
and a second round closes the limits the first follow-up recorded and the shapes a new search found
(see "Follow-up:
the limits of the first follow-up closed,
and what the search found next").

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

At the first follow-up a seeded search of forty thousand pairs of runs,
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
gave these counts,
which the fixes of the second round changed
(the counts after them are under
"Follow-up:
the limits of the first follow-up closed,
and what the search found next"):

- 6887 pairs shipped as written;
- step 1 was taken for 31868 pairs and made the page parse for 26457 of them;
- step 2 was taken for 838 pairs and made the page parse for 32 of them;
- step 3 was taken for 6624 pairs.

Step 2 is taken when every run parses alone,
the page does not,
and writing every run as literal text changes some run.
It made the page parse for 32 pairs.
One of them is two runs that import the same name:
each parses alone,
the strict grammar refuses the second declaration of the page,
and writing every run so gives the first letter of each module line as a character reference.
The other 31 hold a fence mark indented four spaces in both runs:
MDX has no indented code and reads such a mark as a fence,
the plain parse reads a paragraph or indented code there,
so a brace between two such marks is text to the strict grammar once the marks pair across the runs,
which writing every run so escapes.
The older statement that step 2 is taken by no text was wrong:
it was taken for 838 pairs,
for the most part a run nested past the bound beside a run holding a character that step 2 escapes,
and for the rest a fence mark indented four spaces,
a raw HTML block that runs to a closing marker,
a `$$` flow and the shapes named under "Shapes the encoder did not make parse at the first follow-up".

At the first follow-up step 3 was taken for these causes,
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
  which come down to the shapes the encoder did not meet then,
  named in the next heading.

#### Shapes the encoder did not make parse at the first follow-up

Found by encoding each run of the remaining pairs alone and reading what stayed refused.
Each was a limit of the module comment of `source-ships-encoding.ts` then,
and the second round closed each of them
(see "Follow-up:
the limits of the first follow-up closed,
and what the search found next"):

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

### Follow-up: the limits of the first follow-up closed, and what the search found next

The lead's ruling for this round asked that each limit the first follow-up recorded be closed,
red first,
with reader cases under the real grammar,
and that the two shapes only step 2 made parse be closed at step 1 if they could be.
The goal of the ruling stays what it was:
a reader of the shipped page sees the recorded source text and the page parses,
and dropping a run is the last resort.
Five more shapes,
found by the search of this round and listed under "What the search found next",
go beyond the limits the ruling named;
they were taken under its clause that the goal stays,
and they are open to the owner's veto like the rest of the ruling.

Commits,
oldest first,
each red before its fix:
`dc7fd7dca1` (main merged at `e8ba946aed`;
one conflict,
in `audit-ledger-entries-by-id.md`,
kept both added lines);
`c46bd5315f` and `02cae5f93f` (a byte order mark line before a setext heading that starts `import`);
`1ae0f45a05` and `57f9d8c7ea` (a definition line after a footnote definition);
`08258dbbd0`,
`da0ac7a655` and `f841a1bb88` (a bare address after a bracket nothing closes;
`da0ac7a655` gives a new module its own unit test,
which `57f9d8c7ea` had left out);
`3f26bce66d` and `bdebc89412` (a code span,
inline math or a title across lines);
`de2594974c` and `847031ae5e` (a raw HTML block or indented code);
`66e3a059f2` and `bc1487032f` (a fence mark indented four spaces and a repeated import,
at step 1);
`1b260ba2a7` and `22e6cc8ca2`,
`c39d91d454` and `e5dcfe7c0c`,
`9ee315816d` and `c513e15b29`,
`04e3e9b32e` and `96be3fb8f8`,
`a55189e20c` and `964a8f733c` (the five shapes of the search);
`fcfa656d64` (the comments that state the causes).
The body of `66e3a059f2` says the fence pair ships without its runs;
it took step 2,
as the repeated import did,
and the body of `bc1487032f` corrects it.
The body of `57f9d8c7ea` says the source-scan suite passed;
it failed on the new module's missing test until `da0ac7a655`.
Neither commit is amended;
this paragraph and the bodies named are the corrective comment.

#### The five limits, closed

Each has reader cases in `page-check/src/corpus-run/source-ships-encoding.unit.test.ts`:
the page parses under the real grammar and a reader of it sees the source text.

- A raw HTML block or indented code holding `[https://example.net/a{ b](u)`:
  both are paragraphs to the strict grammar,
  so every character Markdown reads as syntax in them is escaped and the reader sees the characters of the source
  (`source-ships-lines.ts`,
  `contentStretches`).
  The cases of the literal group,
  the first of which is "a raw HTML block holding a link whose text is an address with a brace",
  compare with the targets of links left out,
  since the strict grammar's autolink transform links an address in that text.
- A line CommonMark reads as a definition right after a footnote definition:
  a lazy line of the footnote's paragraph,
  written as text (`isLazyDefinition` in `source-ships-footnote.ts`).
  The cases of the reader group named "a definition line ... after a footnote definition" cover one line,
  two lines,
  a line holding an address and a line after a blank line.
- A bare address after a bracket nothing closes:
  the tokenizer reads none after an unbalanced `[` or `![` of the same run of inline content,
  so the brace in it is escaped (`source-ships-addresses.ts`).
  An address that a less-than sign ends is still written as a link.
  The cases of the reader group that name a bracket nothing closes cover text,
  an image opener,
  a www address,
  an address ended by a tag,
  brackets that hold a link or a pair of brackets,
  a bracket a later one closes and a later paragraph.
- A code span or inline math across lines whose later line starts with `{` or `<`:
  the line ending and the marks before that line are written as one space,
  which a code span and math already read as a space (`source-ships-lines.ts`,
  `joinedLines`).
  The same is done for the destination and the title of a link,
  an image and a definition.
- A byte order mark line before a setext heading that starts `import`:
  the line after a line of invisible characters is written with its first letter as a character reference
  (`source-ships-module-line.ts`).
  The measurement is the one the module comment records:
  a byte order mark,
  a no-break space,
  an ideographic space,
  a zero-width space,
  a word joiner and a zero-width joiner end the paragraph for the package's reading,
  and a soft hyphen,
  a left-to-right mark and a variation selector do not.

#### The two shapes that only step 2 made parse, closed at step 1

A fence mark indented four spaces or more is a fence in the strict grammar and text in the plain reading.
The closing fence of micromark takes its line prefix with no bound where indented code is off
(`micromark-core-commonmark@2.0.4`,
`lib/code-fenced.js`,
the closing fence reads its prefix with `codeIndented` disabled);
the opening and the interruption of a paragraph were measured by the reader cases.
So a mark that the plain reading keeps as text opens a code block in the page,
which swallows what follows up to a mark in the page that closes it.
`source-ships-fence.ts` finds such a mark,
the encoder escapes each character of one in text,
`joinedLines` joins a code span,
math or a title across a line that starts with one,
and a fenced block that holds one is written with a longer fence (see "What the search found next").

Two runs that each import one name parse alone and not in one page.
`source-ships-declared.ts` reads the declared names from the `estree` of each module block of the strict parse,
with comments and lines of format characters blanked as the package does for a page,
and `source-ships-company.ts` takes the runs that hold such a mark
and the runs that repeat a name an earlier run keeps as written.
The first step writes those runs as literal text with the runs that do not parse alone,
so the first run keeps its module and the later ones show their text.
The page-level cases are in `source-shipped-page.unit.test.ts`:
the fence pair and the repeated import take step 1 of 3,
and two different imports take no step.

#### What the search found next

The search of this round (the pool extended by every shape of this follow-up,
two seeds,
forty thousand pairs each)
found texts that reached step 3 for five more reasons.
Each became red cases and a fix,
and each is beyond the limits the ruling named.

- An `import` or `export` line after a blank line or a line of format characters in a list item,
  a block quote or a raw HTML block:
  a line at the first column ends every container,
  and the plain reading keeps a raw HTML block whole across blank lines.
  `laterModuleLines` reads any paragraph,
  setext heading or raw HTML block at any depth (`1b260ba2a7`,
  `22e6cc8ca2`).
- A definition line right after a block quote or a list whose last block is a raw HTML block,
  or right after indented code:
  to the strict grammar the block is a paragraph and the line is its lazy line (`c39d91d454`,
  `e5dcfe7c0c`;
  `source-ships-literal-end.ts`).
- A fenced block holding a mark indented four spaces:
  the fence and its closing are written one character longer than the longest such mark,
  so the strict grammar cannot close it there and the code is unchanged (`9ee315816d`,
  `c513e15b29`;
  `source-ships-fenced.ts`).
- A raw HTML block that follows a paragraph with a bracket nothing closes:
  the strict grammar keeps its lines in the paragraph,
  so no address is read in them either,
  and a blank line before the block starts a paragraph of its own (`04e3e9b32e`,
  `96be3fb8f8`).
- The names a run declares,
  read with comments and lines of format characters blanked:
  a second run holding a comment was refused for it and read no name,
  so a repeated import fell through to step 2 (`a55189e20c`,
  `964a8f733c`).

The counts after the fixes,
the same script and pool for both seeds
(the pool of the first follow-up was extended by these shapes,
and the sample cap changed,
so the counts are not those of the first follow-up):

- Seed 20261023:
  8102 pairs shipped as written;
  step 1 was taken for 31160 and made the page parse for 26404;
  step 2 was taken for 486 and made the page parse for 1;
  step 3 was taken for 5493.
- Seed 20270101:
  8112 pairs shipped as written;
  step 1 was taken for 31140 and made the page parse for 26361;
  step 2 was taken for 467 and made the page parse for none;
  step 3 was taken for 5527.

Of the pairs that took step 3,
5440 and 5471 hold a run nested past the bound of the plain reading,
which the pool holds on purpose and the 300-deep case pins.
The other 53 and 56 are the limits that stay:

- A `$$` flow that one run opens and a later run closes
  (41 and 40 pairs that took step 3,
  and one pair of the first seed that step 2 made parse;
  a fence does the same,
  1 and 1;
  each of the 5 and 7 pairs that also hold a fence mark indented four spaces holds such a flow):
  an unclosed flow runs to the end of the page in the strict grammar,
  taking the English between the runs,
  and closing it would take a line the original does not hold.
  The fence case in `source-shipped-page.unit.test.ts` pins the behaviour.
- A comment that never closes beside a line the plain reading takes for a definition
  (2 and 3 pairs):
  the opener is written as a character reference,
  and the line after it is then a line of a paragraph where the reading blanked the opener and saw a definition.
- A run that starts with a byte order mark and whose first line then starts a block
  (3 and 3 pairs):
  the encoder reads the run as a document start,
  where the mark is dropped,
  and in the page the mark stands in the middle of a line.
- A definition line right after a block quote that holds a footnote definition
  (1 and 1 pair).
- A footnote definition followed by a line of `*` (1 pair of the second seed),
  which was not diagnosed.

No cache version moved:
`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' dc7fd7dca1..HEAD` names no commit,
and the same form with `fenceLengthEdits` names `c513e15b29`.
The encoder's output lives only in the pages on disk,
and no stage keyed by a prompt or a sheet changed.

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
