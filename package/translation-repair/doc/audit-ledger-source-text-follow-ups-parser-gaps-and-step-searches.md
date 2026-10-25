# Audit ledger: the source text under a note, follow-ups on parser gaps and step searches

Follow-ups of B832 (the original of an untranslated passage written as literal text),
each found by reading the encoder against the strict grammar or by a seeded step search.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.
The entry they follow is in [the page grammar](audit-ledger-source-text-under-a-note-and-the-page-grammar.md).

## B832 follow-ups

### Follow-up: the holes between the plain parse and the strict grammar

Reading the encoder against the strict grammar,
run by run,
found texts it left refused,
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
reads the source in the flavour the package parses pages in,
CommonMark with GFM and the site's math,
no MDX
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

### Follow-up: the sixth round of fixes after the re-check of round 4

The re-check of round 4 (findings Minor 1 to 4 and nits N1 to N9) was rerun against the head of round 5 first,
and each finding was fixed red first where it was still open.
The merge of `origin/main` at `d8ac654323` (`a997614532`) had no conflict.

#### The findings

- Minor 1,
  the condition of step 2:
  restated in this entry,
  in the header of `source-ships-compose.unit.test.ts`,
  in the encoding module comment and in `design-commitments-shipping-and-the-archive.md`
  as the code takes it:
  step 2 is taken when the page of step 1 (the page as written when step 1 changed nothing) does not parse
  and writing every run as literal text gives a page unlike the one last tried.
  The older wording,
  "every run parses alone",
  was not the condition,
  since step 1 writes the runs that do not.
- Minor 2,
  a top-level indented block whose first line starts with a fence mark:
  round 5 had closed the visible symptom
  (indented code is copied as literal text there,
  so the page shows a paragraph and no backslash stands inside a fence).
  The remedy of the ruling,
  reading the block as a fence for the spans,
  was not implemented,
  and nothing needs it.
  Three reader cases pin the two shapes of the finding and a tilde variant (`14fe5243bc`).
- Minor 3,
  an opener of a comment that never closes,
  repeated:
  open at the head of round 5.
  Every `<!--` of such a run,
  outside a comment that closes,
  is written `&lt;!--`,
  and a backslash before it goes with it (`a1346f7565`,
  `bacb0a9b6b`).
- Minor 4,
  the label of the GFM and math reading:
  it is "the flavour the package parses pages in (CommonMark with GFM and the site's math,
  no MDX)"
  in `gfm-math-reading.test-fixture.ts`,
  in `source-reader-text.test-fixture.ts`
  and in the follow-up named "Follow-up:
  the holes between the plain parse and the strict grammar".
  Whether the site's own compiler,
  which has no GFM,
  accepts the encoder's output is not measured here.
- N1 to N6 and N9 are fixed in `9bddb06f37`:
  ragged tails of comment paragraphs,
  a blank line before a TSDoc that followed a statement at module level,
  the two pointers of this entry that named a position or a round,
  the articles that opened the `@param` and `@returns` descriptions the branch added,
  the two address shapes the plain parse cuts into several nodes (named in `source-ships-addresses.ts`),
  and the `@throws` of `parseGfmMathBody`.
  N2 needed nothing:
  every file the branch touches has as many region ends as region starts.
- N7,
  an angle autolink inside link text:
  written as its address alone,
  with the characters Markdown reads escaped,
  so the page shows the address and no brackets (`e6155d7e1b`,
  `50f1b18610`).
  The alternative text of an image stays a limit,
  since the plain parse holds it as a string with no autolink node in it;
  a test pins it and the encoding module comment names it.
- N8,
  the record of the step search:
  the script of this round writes every text with every warn line,
  uncut,
  one JSON line per text,
  and no sample cap.

#### A regression of round 5 found by rerunning the case sets

A fence in a nested list item had its closing counted as a deep mark of the content,
so the opening was lengthened and the closing was not.
The closing of a fence is now the last line,
a mark alone and at least as long as the opening,
within three columns of the opening (`5e1fffd44e`,
`2e6f975b66`).
No list of limits or findings named it before the case sets were rerun.

#### Limits of round 5 that were not structural, closed

- A comment opener beside a line the plain reading takes for a definition (`4d68fd59f8`,
  `124be8dd2b`):
  the text the parse is given holds a stand-in that starts with an ampersand,
  where it held four spaces that made the next line a definition.
- A run that starts with a byte order mark (`079a66e58b`,
  `d9782af3a2`):
  the page carries the run after a note and a blank line,
  so the mark stands mid-page and starts no block;
  the parse is given a letter in its place.
- A definition line after a block quote that holds a footnote definition (`0759da4a22`,
  `f8cf6ad08d`):
  the footnote state is kept for each container,
  and a lazy definition line counts in a root,
  a block quote and a list item.
- A footnote definition followed by a line of `*` (`c1dce5b237`,
  `16b9ee1c3e`),
  diagnosed:
  the strict grammar keeps the footnote definition open for the line after it and tries a new container there as if no
  paragraph were open (`micromark@4.0.3`,
  `lib/initialize/document.js`,
  `checkNewContainers`,
  where the interrupt flag is set only when every open container continued;
  `micromark-core-commonmark@2.0.4`,
  `lib/list.js`,
  where a marker with nothing after it starts a list,
  and so does an ordered marker that is not 1,
  when the flag is unset).
  A bullet,
  a dash,
  a plus or a number with a dot or a parenthesis alone on the line is then an empty list item,
  and the line after it starts a block,
  an ES module among them.
  The plain reading keeps the marker as a line of the paragraph.
  The module line after such a marker is written with a character reference.

#### What the step search found next, and what stays

The script is the one of round 5 with a pool extended by the shapes of this round
(a comment opener beside a definition,
a byte order mark before a block,
a definition after a block quote that holds one,
a marker alone after a footnote definition,
an autolink in link text,
a fence in a nested list item,
two openers),
run with both seeds,
forty thousand pairs each,
each seed in its own `systemd-run` unit with a memory and a CPU cap.
Because the pool is larger,
the counts are not those of the step search in the follow-up on the limits of the first follow-up.
The first run of the final head showed three more shapes,
each fixed red first:

- A footnote definition that starts inside a paragraph of the plain reading
  (the plain reading has no footnotes,
  so the line stays in the paragraph),
  then a list marker alone and an `import` or `export` line (`93961f4aae`,
  `5236e713d6`).
  A definition indented four spaces or more is a definition to MDX and a line of the paragraph to CommonMark with GFM,
  so the page parses and the reader of the page and of the source differ in the marker;
  the encoding module comment names it.
- A line of invisible characters,
  then a list marker alone and a module line:
  the invisible line ends the paragraph for the strict grammar (`e21a75ef6f`,
  `eb28a07167`).
- A code span that starts with a run of three backticks indented four spaces or more below a paragraph line,
  in a block quote or not:
  to the plain reading the run opens the span and no backslash can be written into it,
  and the strict grammar takes it for a fence that cuts the paragraph.
  The line ending before the span is written as one space,
  as a later line that starts so already was (`e21a75ef6f`,
  `eb28a07167`).

The counts at the final head:

- Seed 20261023:
  7017 pairs shipped as written;
  step 1 was taken for 32279 and made the page parse for 27974;
  step 2 was taken for 402 and made the page parse for none;
  step 3 was taken for 5009.
- Seed 20270101:
  7180 pairs shipped as written;
  step 1 was taken for 32171 and made the page parse for 27899;
  step 2 was taken for 407 and made the page parse for 1 (a pair of `$$` flows);
  step 3 was taken for 4920.

Of the pairs that took step 3,
4969 and 4882 hold a run nested past the bound of the plain reading.
The other 40 and 38 are one structural limit:
a `$$` flow or a fence that one run opens and no line of the run closes,
and a later run closes.
Each run of each such pair parses alone.
The grammar leaves the flow open to the end of the page
(`micromark-extension-math@3.1.0`,
`lib/math-flow.js`,
`beforeContentChunk`,
where the end of the input ends the flow;
`micromark-core-commonmark@2.0.4`,
`lib/code-fenced.js`,
the same for a fence),
so it takes the English between the runs,
where a reader of the run alone sees it end with the run.
Closing it would add a line the original does not hold.
The other structural limit,
nesting past 256,
is refused by the package's own bound before any parser reads the text
(`NESTING_BOUND` in `markup/src/nesting-bound.ts`),
and a parse that ran out of stack is answered as the same refusal
(`withMdxExhaustionRefused` in `document-structure/src/parse-mdx.ts`).

#### Checks

Five guards were mutated in the working tree and each mutation failed the suite,
with the patch undone by `git restore` after each:
the closing reach of a fence (1 test),
the stand-in for a leading byte order mark (13 [FAIL] lines),
the openers of an unclosed comment (7),
the list-marker rule of the module line (19),
and the choice between a link and its address for an autolink in link text (15).
No cache version moved:
`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' a997614532..HEAD` names no commit,
the same form without the range names 110,
and `-G closingMarksOf` over the range names the fix of the fence closing.

### Follow-up: the seventh round, the module lines from the tokenizer and the footnote reading

This round follows the re-check of round 6.
Each fix has a red commit that names the failing lines.
The commits are in the order they were made.

#### The module lines are the tokenizer's

Every module line the encoder guards was found by a rule per shape,
and the next shape was always missing (`746b01f9be`,
red for the line after a setext underline;
`3c35d28d17`).
The lines now come from the strict grammar's own tokenizer,
run on the page as it will be written
(`document-structure/src/module-line-starts.ts`),
so no list of shapes stands between the encoder and the lines the page holds.
The guard stays a guard on the shapes the search has run,
not a proof about every text:
the search of this round shows no pair whose cause is a module line,
on two seeds and one pool.

#### Findings of the re-check

- A fence mark indented under a footnote definition (`0062eacfe1`,
  `fc06211a3c`):
  the lines indented under a footnote are read as blocks of the footnote,
  so a fence among them is copied and its braces are code.
- A dollar sign in the text of an angle autolink written as a link (`77d13ed420`,
  `5ef191dfc3`):
  it is written with a backslash.
- An angle autolink in a table cell that holds a pipe (`a24a59843f`,
  `b7aaf2efd6`):
  a table cuts a cell at every pipe,
  so the autolink is written as text there.
- A definition line after a blank line in a comment that never closes (`54a4da317d`):
  classified against the named limit of an unclosed comment and pinned.
- A code span or inline math across a line of invisible characters (`041ca5760e`,
  `941406e8de`):
  the mask of `parseDocument` blanks that line before the strict parse and ends the paragraph there,
  so a module word in the span is joined to the line before it.
- A footnote definition after a setext heading that follows a link reference definition (`3f11ebd734`,
  `120edff021`):
  the step search had recorded a `FootnoteRewriteError` for a pair (seed 31415926,
  pair 13815);
  the heading run now starts after the definitions the parser began it at.
  `readPageGrammar` should not answer "refused" for that class:
  the error is a fault in the boundary between the code and the parser,
  not a property of the text,
  and `page-grammar.ts` rethrows everything but the two parse errors on purpose.
  A refusal would hide the fault and ship the page without the original.
- The lines after an unclosed fence indented under a footnote (`74da98d1b3`,
  `3097db022f`):
  a footnote body ends where a parse with GFM ends the footnote
  (`document-structure/src/gfm-block-ranges.ts`,
  renamed from `table-ranges.ts`),
  so the lines after it are text of the page.
- A footnote's lazy lines and a footnote that starts on a later line of a paragraph (`141406c1d1`,
  `811e045e5f`):
  every unindented line after a footnote's first line is a line of its paragraph to the strict grammar,
  a lazy line is no setext underline,
  and a definition line cannot interrupt a paragraph.
  A code span across a later-line footnote start used to leave a brace live on the page;
  the paragraph is now cut at the footnote and each part is read as a text of its own.

#### What the step search found, at `214660a830`

Two searches of forty thousand pairs each,
seeds 31415926 and 27182818,
each in its own `systemd-run` unit with a memory and a CPU cap.
The counts and the recorded pairs are the same at `94c8ddb620`,
the head before the last style commit.
The pool is the pool of the previous follow-up.

- Seed 31415926:
  7101 pairs shipped as written;
  step 1 was taken for 32268 and made the page parse for 27976;
  step 2 was taken for 383 and made the page parse for none;
  step 3 was taken for 4923.
- Seed 27182818:
  7202 pairs shipped as written;
  step 1 was taken for 32142 and made the page parse for 27683;
  step 2 was taken for 412 and made the page parse for none;
  step 3 was taken for 5115.

Of the pairs that took step 3,
4876 and 5070 hold a run nested past the bound.
The other 47 and 45 are not nested.
For 44 and 40 of them each run parses alone and one swallows the English that follows it,
which is the structural limit of the previous follow-up.
The other 3 and 5 are two named limits,
and nothing else:

- A tag alone on a line between a footnote's paragraph and a definition line:
  3 pairs in each run
  (seed 31415926 pairs 15275,
  22277 and 33035;
  seed 27182818 pairs 18545,
  29475 and 37568).
  With the tags read as letters the same pairs take step 1 alone.
- A byte order mark before a fence mark:
  2 pairs in the second run (7839 and 18575).
  With the marks removed the same pairs take step 1 alone.

The first run of this round,
at `941406e8de`,
showed ten pairs of an unclosed fence indented under a footnote,
which `3097db022f` closed.
That run predates the footnote fixes of this round and is not the record of the final head.

#### Limits named

Two of the limits named here,
the tag alone on a line and the byte order mark before a fence mark,
and the structural limit of a fence or display math that one run opens,
are closed in the follow-up "The eighth round,
the whole-page check and the limits it closed".

- A tag alone on a line between a footnote's paragraph and a definition line.
  The parse of the original reads the tag as a raw HTML block and ends the footnote there,
  where the page writes the tag as text and goes on with the paragraph.
  The definition line is then a lazy line of the paragraph,
  and its braces are live:
  the page parses and holds an expression the source reader hides.
  Reading each tag as a letter before the footnote ranges are taken gave the page's structure for this shape,
  and turned 96 of the generated footnote texts from equal to unequal;
  a second form that merged the two ends turned 648 unequal.
  Both were removed.
  The encoding test pins the text.
- A byte order mark before a fence mark,
  seen in 2 pairs of one run.
- The same limits as in the previous follow-ups for a footnote in a list item or a block quote,
  a table's bare address that a pipe ends,
  and an angle autolink holding a pipe written with a backslash.

#### Checks

The probe sets of the footnote reading were run before and after the final fix:
409 combined texts equal,
2 refused,
97 equal without link targets and 11 unequal,
the same counts before and after;
6636 generated footnote texts equal and 4164 unequal,
the same before and after,
all the unequal ones in a container.
No cache version moved:
`git log --oneline --extended-regexp -G '_CACHE_VERSION = [0-9]+' ec812f269b..HEAD` names no commit,
and the same command over `c265e8df95~1..c265e8df95` names `c265e8df95`.

### Follow-up: the eighth round, the whole-page check and the limits it closed

This round follows the re-check of round 7.
The commits are in the order they were made;
each fix comes after a red commit that names the failing lines.

- `54107958fa5c284e805899d4b581fff2002cc4d2`,
  red:
  the whole-page check and 11 failing cases.
- `6c1651044db14569f9f5dbfc3fb5a797ad79b7bf`,
  the fix of those 11:
  a fence or display math a text leaves open is closed,
  and a footnote that ends in a tag alone on its line ends before a definition line that follows.
- `1c105209aa0b1ea283941127280530d344f2e773`,
  red:
  a footnote that is a block of its own,
  then a definition line (10 failing lines).
- `f97ac62bcf6843d1a3907ae53428691b68538075`,
  the fix of those.

#### The whole-page check

The limit of round 7 for a tag alone on a line was a defect and not a limit.
The text `[^1]: x`,
then a line `<a href="x">`,
then `[r]: u{b}` shipped a live `{b}` expression:
a page that parses and holds code the original lacks.
Pinning a text that does this made the pin pass for the wrong reason,
so the check is now a function and not a case list.

`liveConstructsOf` (`page-check/src/corpus-run/source-live-constructs.test-fixture.ts`)
masks the comments of a page as the package does,
parses it with the strict grammar,
and lists every flow expression,
text expression,
flow element,
text element and ES module in the tree.
An attribute belongs to its element.
The original read with the package's own grammar (GFM and the site's math,
no MDX) builds none of these nodes,
so one in the text of an encoded slice is a construct the original lacks.
It is called from the reader,
literal and shown cases of `source-ships-encoding.unit.test.ts`
and from the page-level cases of `source-shipped-page.unit.test.ts`.
Over the generated sets and the step search it runs once per code state in a driver that is not in the suite,
because the search takes about two minutes per seed:
the driver finds the text of each slice the composition wrote as literal text in the page
and counts a construct that starts inside it.

At the head before this round it found 5 of the 11398 texts of the sets with a live construct
(all 5 the tag-alone class)
and 4 and 1 pairs of the step search (seeds 31415926 and 27182818).
At the final head it finds none in either:
11398 texts with no live construct,
and 40000 pairs per seed with 31534 and 31396 encoded pages and no violation.

#### Why the two letter-reading attempts moved texts

Round 7 read each tag as a letter before taking the footnote ranges,
and 96 of the generated footnote texts went from equal to unequal;
a second form that merged the two ends moved 648.
The cause is that a letter in place of a tag changes the parse of the lines after it,
and the encoder's other readings (spans,
joins of lines in a paragraph) were made against the parse with the tag.
For `[^1]: <b>`,
then `abc`,
then two lines of four spaces and a fence mark,
the original has `abc` and the two indented marks as a paragraph of the top with a code span across them,
and the encoder joins those lines with a space.
With the letter,
the footnote's paragraph is `x`,
`abc` and the marks are a fence of the footnote,
the encoder escaped the marks,
and the page showed them.
Any text of that shape changed reading.

The fix keeps the parse of the original as it is.
Where the footnote ends is already taken from the GFM parse of the original (`footnoteDefinitionRangesOf`).
The page writes a tag as text,
so the footnote's paragraph goes on where the original's footnote ended,
and only the line ending that ends the footnote needs to change:
`footnoteBodySpans` (`page-check/src/corpus-run/source-ships-nested.ts`)
writes it twice when the next line directly follows and the first block of the lines up to the first blank one,
read on their own,
is a link reference definition.
No other line is read differently.
The page then has the definition where the original has it,
and the reader of the page equals the reader of the source for the five cases of the red commit.
Over the sets no text went from equal to unequal;
6 went from unequal to equal and 2 from refused to equal.

The second pair of commits covers the form in which the plain reading's block ends where the footnote does
(`[^1]: <b>` is a link reference definition to the plain reading,
and the code a footnote ends with is the same),
which the first fix did not reach.
The seeds 31415926 and 27182818 each took step 3 for three pairs of it
(a footnote that is a tag alone,
then `[r]: u{b "t {"`),
which now take step 1.

#### A fence or display math a run opens and no line of it closes

This is the class the whole-page check found beyond the tag-alone class.
The strict grammar ends such a flow where the page ends
(`micromark-extension-math@3.1.0` `lib/math-flow.js`,
`micromark-core-commonmark@2.0.4` `lib/code-fenced.js`),
so the English between two runs fell inside it,
and the first line of the next run that looked like its closing line ended it
and left the text after that line outside it,
where a brace or a tag is code.
Round 7 named this a structural limit for 44 and 40 pairs and shipped the page without the originals.
The four violations of seed 31415926 were an open display math beside a later run,
and the one of seed 27182818 was a byte order mark case:
the mark sits in the middle of the page,
where the page reads the run's first fence as an ordinary line
and a later fence of the run as open.

An original is closed where its slice ends,
so a text written as literal text now ends with a line of the closing mark
(`closerOfWritten` and `closedWith`,
`page-check/src/corpus-run/source-ships-open-flow.ts`),
read from the text as the page holds it:
comments blanked,
a byte order mark that opens the text an ordinary character
(`source-ships-readable.ts`),
the mark as long as the one that opened the flow.
The closing line shows nothing,
since a reader of a fence or a display math reads it the same with or without it.
A run that ends inside a flow is also taken at the first step
(`leavesFlowOpen`,
`source-ships-company.ts`),
since a neighbour left as written could close it.
The step search before the change had 47 and 45 pairs that were not nested,
and none is left at the final head.
The fence case of `source-shipped-page.unit.test.ts` used to pin that the page ships without both runs;
it now pins that it ships both,
and the lead may veto this change.

#### What the step search found at `f97ac62bcf6843d1a3907ae53428691b68538075`

Two searches of forty thousand pairs each,
seeds 31415926 and 27182818,
each in its own `systemd-run` unit with a memory and a CPU cap,
on the pool of the seventh round (the file is unchanged).

- Seed 31415926:
  7101 pairs shipped as written;
  step 1 was taken for 32363 and made the page parse for 28012 (27976 at round 7);
  step 2 was taken for 365 (383) and made the page parse for none;
  step 3 was taken for 4887 (4923).
- Seed 27182818:
  7202 pairs shipped as written;
  step 1 was taken for 32244 and made the page parse for 27717 (27683);
  step 2 was taken for 403 (412) and made the page parse for none;
  step 3 was taken for 5081 (5115).

Every pair that took step 3 holds a run nested past the bound,
4887 of 4887 and 5081 of 5081,
so no step-3 pair is left that the classification calls unexplained or structural of another kind.
The seventh round listed 3 and 5 unexplained pairs.

A byte order mark in a run:
5605 pairs of seed 31415926 and 5681 of seed 27182818 hold one,
3944 and 3940 of them with a fence or display math mark.
Of those,
2823 and 2906 took step 1,
724 and 666 shipped as written,
and 397 and 368 took step 3,
every one for a run nested past the bound.
None had a violation.
(The count of these pairs came from a rerun of the composition on those pairs only,
on the same code.)

#### Reading differences that stay

Over the sets of the round (11398 texts):
7115 equal,
99 equal without link targets,
4181 unequal and 3 refused.
At the head before this round the same sets held 7107 equal,
99,
4187 and 5.
The 10800 generated footnote texts are as they were before this round:
6636 equal and 4164 unequal,
all 4164 in a list item or a block quote.
Of the 598 texts that are not generated,
17 are unequal and 3 refused:

- A footnote in a list item or a block quote with an indented fenced block
  (the marks are written with a backslash and the page shows them):
  6 unequal.
- A footnote definition indented four spaces in a paragraph:
  2 unequal.
- An angle autolink in the alternative text of an image:
  3 unequal.
- A bare address with emphasis marks beside a brace:
  1 unequal;
  a bare address with two dollar signs beside an angle bracket:
  1 refused.
- A table cell with an angle autolink holding a pipe written with a backslash,
  or a bare address that a pipe ends:
  2 unequal;
  a code span holding a pipe:
  1 refused.
- A comment that never closes before a definition line:
  2 unequal.
- A text nested 300 deep:
  1 refused.
- A list item that opens with a tag,
  then a definition line with a title:
  1 unequal.

None is new:
each has the verdict it had at the head before this round,
and the 6 unequal and 2 refused texts that became equal are not in this list.
The module comment of `page-check/src/corpus-run/source-ships-encoding.ts` names the rest of the smaller limits.

#### Checks

`build`,
`test:unit` (142 `[PASS]` lines and no `[FAIL]`),
`lint:types` and `lint:oxlint` of `translation-repair-page-check`,
and `test:unit` (67) and `source-scans` (52) of `translation-repair-source-scan`,
run in turn and on the code of the last fix.
No cache version moved:
`git log --oneline --extended-regexp -G '_CACHE_VERSION = [0-9]+' 75c33a64946a2be1e872d40b0a4a6e9c3b398084..HEAD`
names no commit,
and the same command over `c265e8df95~1..c265e8df95` names `c265e8df95`.
