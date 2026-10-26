# Audit ledger: the source text under a note, the ninth round of follow-ups

The ninth round of the follow-ups of B832 (the original of an untranslated passage written as literal text):
lone carriage returns,
a wider sample of the generated sets and a full pass over them.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.
Rounds 1 to 8 are in
[the file of the follow-ups on parser gaps and step searches](audit-ledger-source-text-follow-ups-parser-gaps-and-step-searches.md),
and the entry they follow is in [the page grammar](audit-ledger-source-text-under-a-note-and-the-page-grammar.md).

## B832 follow-ups, the ninth round

This round follows the re-check of round 8.
It answers three findings of that re-check and four smaller ones.
The commits are in the order they were made;
each fix comes after a red commit,
and the failing lines of every red commit were kept as files apart from the commit message.

- `06630dabea8dc7c6f785f0c164438571926c65b9`:
  the pair and text readers of the sampled check move into a fixture.
- `19338352b89a6294a8096077fcd04c9679d38a5f`,
  red:
  a text with lone carriage returns for line endings (68 failing leaf cases,
  3 controls).
- `c72dc715f9c51cff425551edf6e99b1350b808eb`,
  the fix.
- `6fa76a85c0593b561dd02b370dd35ff050679f7f`,
  red:
  the walk fixture called through an object property,
  the module namespace and `Function.prototype.call`
  (7 failing leaf cases).
- `6dafae15297c41fbf9450bc2520a6d1ca15664f4`,
  red:
  a definition holding a brace before an indented fence,
  a caret label with white space,
  and a tab fence beside lone carriage returns (9 failing leaf cases).
- `f99b48691280f178cb81120dfb6468786e216622`,
  the fix of those.
- `13e53207b72e7dfc6f9bef886dd79770aed9d4d7`,
  the fix of the walk fixture.
- `b59a090e344f00183f2f22af1686101f5fe66fc3`,
  red:
  a code span across a footnote definition line in a block quote or a list item (4 failing leaf cases),
  and the pin of the limit on footnote labels with a brace.
- `693c4b67ca341a55d26b6cffaf637e8a8b36cc21`,
  the fix of those,
  and the source of a regression found by the full pass.
- `d6cdc13b5114623d8a489b297db2c41128ffd43d`:
  articles in TSDoc and lines over 120 columns.
- `e4871f05840ce82edef1eb2d4c6f6e1e64883cb1`,
  red:
  the regression (2 failing leaf cases);
  its three cases asked for reader equality that no encoding gives for
  a footnote in a container,
  which `83a717b45dca19419e4ce70e104cf1468682807a` corrects.
- `83a717b45dca19419e4ce70e104cf1468682807a`,
  red:
  the same three texts,
  asked for no live construct (1 failing leaf case).
- `99adc6093c69dd5eff714ee25e3f208822340adc`,
  the fix.
- `bed83bb5c3c8cd0ff8e7329f4b80e15b72a93040`:
  the sampled check,
  the full task and the invariant for step 3 of the composition.

### A lone carriage return

A carriage return that is no part of a Windows line ending is a line ending to the grammar and to the plain parse,
and the readers of the encoder split lines on the line feed alone.
The re-check found 18 text-level ids and 13 page-level ids that leaked a live expression,
and the shapes `$$CR{a}CR$$CR` and a fence that swallowed the page.
Two designs were open:
read a carriage return as a line ending in every reader that splits,
or write each one as a line feed where the text enters.
Ten modules split lines on the line feed
(`source-ships-nested.ts`,
`source-ships-open-flow.ts`,
`source-ships-readable.ts`,
`source-ships-lines.ts`,
`source-ships-fence.ts`,
`source-ships-fenced.ts`,
`source-ships-footnote.ts`,
`source-ships-addresses.ts`,
`source-ships-module-line.ts`
and `source-ships-lazy.ts`),
against one place where the text enters the encoder,
so the second was taken.
`foldLoneCarriageReturns` keeps the length,
so every offset the parse gives stands,
and it leaves a Windows line ending alone,
so a text of mixed endings keeps its Windows ones.
The commit message of `c72dc715f9c51cff425551edf6e99b1350b808eb` counts nine modules where the list has ten.

The company check of the composition (`source-ships-company.ts`) reads the text of each run for an indented fence mark,
a declared name and a flow left open,
and it splits lines on the line feed too.
It reads the run folded in the same way,
and the page keeps the run as it was written.
The first version of the check left a run with a tab fence mark and lone carriage returns as written,
and the later run's brace stood outside the fence the mark opens;
the pair case in `source-ships-carriage-return.unit.test.ts` pins it.

The corpus reader folds CRLF and leaves a lone carriage return alone
(`slice-reading/src/line-endings.ts`),
so no page the corpus brings changes.
`doc/design-commitments-shipping-and-the-archive.md` states what the page does with one.

### The sampled check and the full pass

The sampled check guarded the class it was tuned on.
The pool now holds lone carriage returns,
footnote labels other than `1`,
tilde fences at the first column,
a footnote in a list item and in a block quote,
and a definition line after a footnote that is a tag alone.
The footnote product is 32256 texts.
The rate is one pair in four of the first seed:
a class of nine pairs among its 40000 draws is missed with chance 0.75 to the power 9,
so it is caught with chance 0.925,
where one in six catches it with chance 0.81.
The sampled case took 40.2 s and 51.4 s in the two measurements made,
for that one seed.
A task,
`test:live-full` of `translation-repair-page-check`,
runs `source-live-full.expensive.unit.test.ts`,
all 40000 pairs of each of the seeds 31415926 and 27182818 and all 32256 texts,
and took 293 s to 318 s in two measurements under a cap of three threads
(one seed alone,
143 s and about 360 MB).
The second fix round of this record corrects both figures:
the sampled case now reads both seeds,
and the full task is timed across the runs made before the second fix round (see the heading "The second fix round").

The invariant for step 3 of the composition read the nesting of the pair texts
with a function that splits lines on the line feed
and gave false alarms.
It now reads what the grammar refused of each page the composition tried,
and a page that ships without the original of a run is reported unless one refusal says the page nests too deeply.

The widened pool found,
in turn:

- A definition holding a brace before an indented fence,
  whose fence mark was joined to the definition line,
  which made the line no definition;
  the join is left out after a line that ends a definition.
- A link reference definition with a caret label that holds white space,
  which a footnote label cannot,
  read as a footnote definition in three places;
  `isFootnoteLabel` and `hasFootnoteLabel` now say it once.
- A code span across a footnote definition line in a block quote or a list item,
  one pair of the seed 31415926.
- The ten pairs of the two seeds that the fix of the code span across a footnote line made live,
  by cutting a paragraph of a container at a footnote of the container itself;
  the cut now takes only a definition that starts a line.
  The sampled case of that round did not keep any of the ten.

### The walk fixture

The fixture for a parse that returns a tree raising at the caller's first read told the grammar's own reads from the
caller's by looking for the function name in the call stack.
V8 writes a frame called with an object as its receiver as `Object.parseMdxBody`,
so a call through an object property,
the module namespace or `Function.prototype.call` with an object hid the name and the grammar's own read raised.
`parseMdxBody` and `parseMarkdownBody` now call a hook once they have made their own reads of the tree
(`document-structure/src/parse-end-hook.ts`),
which stands on the `Symbol` function under a registry symbol
and does nothing where no test sets it;
the fixture sets it for the span of one call and raises once it has fired for a tree.
The package of the fixture needs no import of the grammar's package,
and a source copy and a built copy of the module find the same hook.
The 14-site test of `translation-repair-artifact` passes with it.
Other parse sites (`parse-markdown-math.ts`,
`gfm-block-ranges.ts`,
`module-line-starts.ts`)
do not call the hook,
so the fixture never arms a tree they build and answers every read of it.

### Limits named

- A footnote label that holds a brace or a less-than sign (`[^{a}]`,
  `[^a}]`,
  `[^<b>]`):
  the reference is written with a backslash and the definition keeps its label,
  so the page holds no expression and the reference shows as text.
  `source-ships-encoding.unit.test.ts` pins three labels,
  and the module comment of `source-ships-encoding.ts` names the limit.
  Writing the same backslash into both labels would make them agree again.
- A footnote definition inside a block quote or a list item,
  a tag alone on the next line and a definition line with a brace:
  the page parses and holds no expression,
  and the definition line shows as text where the source reads a definition.
  This is the reading difference round 8 named for a tag alone and a definition in a container.

### Mutants

Run once each on a scratch tree of `bed83bb5c3c8cd0ff8e7329f4b80e15b72a93040`,
with the sampled case and the unmutated tree as the control (it passes):

- The JSX leak,
  a less-than sign before `Foo` left unescaped:
  the sampled case fails.
- The ES-module leak,
  module lines left as they are for a text with a quote mark and an import after a blank line:
  the sampled case fails.
- The revert of `f97ac62bcf6843d1a3907ae53428691b68538075`
  (`source-ships-nested.ts` as it was before it):
  the sampled case fails.
- The fold of lone carriage returns removed from the encoder,
  the caret label rule reverted,
  and the join after a definition restored:
  the sampled case fails for each.
- The cut of a paragraph of a container reverted,
  and the fold removed from the company check:
  the sampled case passes,
  and the full task fails for each.

### Checks

`build`,
`test:unit` (147 `[PASS]` lines and no `[FAIL]` for the page-check package,
60 for document-structure,
7 for test-fixture,
121 for artifact),
`lint:types` and `lint:oxlint` of the packages touched,
and `test:unit` (68) and `source-scans` (53) of `translation-repair-source-scan`,
run one at a time on the code of `1c7918806f6f2ae46586bceb622c2f61802591e5`,
the commit that ended the round.
No cache version moved:
`git log --oneline --extended-regexp -G '_CACHE_VERSION = [0-9]+' bcff23b8ce..HEAD` names no commit,
and the same command over `c265e8df95~1..c265e8df95` names `c265e8df95`.
The fold is in the encoder of the page,
and no package that declares a cache version imports `translation-repair-page-check`;
the hook of the grammar returns the tree it was given and changes no value.

### The second fix round

This round answers the re-check of the first fix round (`dx1f-recheck.md` in the working directory of the review,
finding ids R1 to R11),
and the full task it asked for.
A red commit holds a failing case alone,
and its failing lines were kept in `dx1f2/red-*-failing-lines.txt` of that directory;
each fix names the red commit it answers.

#### Findings of the re-check

- R2,
  a code span across a footnote line that carries the marks or the hanging indent of its container:
  red `f7165c5e494295f08e188a33c21d26e582867510` (`red-r2`,
  `red-r2b`),
  fix `549d97ddf4461b830ab09b8d78fc4bdba48dee72`,
  which cut at the character before the footnote and was replaced;
  red `dee820cf6613af0044e77af28d353284d5715d3b` (the pool of runs and the pairs it finds as cases),
  fix `3ef2fd61ccf622655f1e7d70d30f2ea11b02a9ff`,
  which cuts a container paragraph at every footnote line
  and reads each part by whether its line stays in the container.
- R4,
  the label rules of `micromark-extension-gfm-footnote@2.1.0`:
  red `e9a4920bcf28952fcd47ecf86d3a4032c58d2821` (`red-r4`) and `84c1bec06825a7f60580a65595b92376295d6803` (`red-r4b`),
  fix `bb0d27d04a21ecdc58996dd616f04dd6b5a54e9e`.
  The predicate of `markup` and the one of `page-check` meant the same,
  so there is one,
  in `markup`,
  and `page-check` reads a definition's label from its source.
- R5,
  the step 3 invariant excuses a drop only by the refusal for depth that run's own page recorded:
  red `b5fa8c92cc67dfeec565331d3eb3549bc2ebf5b6` (`red-r5`),
  fix `b79df2282574c34e59c42ce344faf8c276020bcb`.
- R10,
  the pins of a tab,
  a carriage return and a line feed in a caret label and of CRLF in the fence-mark join,
  and the N3 case named for what it checks:
  `f073fe69b5cff831caade709f06b1a0ec19bbecd`.
  The two fail-safe mutants `mx-tab` and `mx-crlf` fail 3 cases each (`mut-tab`,
  `mut-crlf`).
- R1,
  R3 and R6:
  the sampled case reads both seeds at one in four and computes the reach of its carriage return guard on its own
  (`9136a1e690f98e0adfde70e1c485ff58fc773a80`),
  the README of `page-check` names the full task and when it runs,
  and its header says what it guards (`874cc10c257b76e9d9eeb8b62f3225bab0110a71`),
  and the ledger drops its positional wording (`0dbc55a48825118217ed76169fd5f9e3ce4f4c93`).
  The chance of catching nine violators among the 80000 draws is 0.920 reading both seeds at one in four,
  and 0.695 reading the first seed alone;
  for seventeen it is 0.992 and 0.892
  (`r1-sim.txt`,
  a simulation against the real kept set).
- R7,
  `Symbol` among the roots of the global writes scan,
  with a synchronous set and dispose under `using` allowed:
  red `57a01038eecc001ba00ab20ca006b57cf41638a3` (`red-r7`),
  fix `396af2b1f0c826d69f0f4592b3b1ca18b8d0b7d3`.
- R8 and R9,
  the TSDoc of the parse-end hook names the parses that call it and those that do not,
  and three TSDoc lines lose their article:
  `83e67534fb81ec3ee0833dd1f2db1b71be513383` and `0a4f92ba212ffcd213ccd9e34af6d4571ba00eac`.
  `4e4065e48fc40e17d9ce8ad296a4635fc66f0747` removes an export no file imports.
- R11,
  the dead declarations scan follows only the imports of test files and fixtures,
  and a built entry under `dist/` is left alone:
  red `7458179d3761668dc11500a467ae3bd9a065ca31` (`red-r11`),
  fix `5a3b6f7085bcef4e4d8afb42b0d527cd504c9c0a`.
- The scratch trees of the first fix round (`dx1f/mut/`) were removed,
  the links by `rm` on the link itself.
- The probe grid of the re-check (80 texts:
  a code span or a plain paragraph across a footnote line indented 0 to 4,
  at the top,
  in a block quote marked or lazy,
  and in a list item,
  with a tag or a brace after the colon) found 16 refused or live pages on `bcff23b8ce`,
  14 on `1c7918806f`,
  and 2 refused and 2 live on the head of this round.
  The four that remain are the indent-4 probes named in the limits of this round.
  The 24 lines of the shapes of the ruling (marked,
  lazy and list,
  indent 0 to 3,
  a tag and a brace) parse with no live construct and read as the source does.
  The 10 pairs that the scans of the first fix round listed (6 and 4),
  replayed on this head,
  have no finding.

#### The first full task, and what it found

`test:live-full` ran once on `0dbc55a48825118217ed76169fd5f9e3ce4f4c93` as a capped unit
(4 GB of memory,
300% of CPU).
It failed,
and a scan of all 80000 pairs on the same code found 15 findings and one crash:
8 findings in the seed 31415926,
7 findings and a `MarkdownParseError` in the seed 27182818.
The pairs fail on `1c7918806f` too,
so none came in with this round.
Draw ids are the comments of `source-live-full-pass-pairs.test-fixture.ts`.

- The measure of nesting split lines on the line feed alone,
  so a text whose lines end in a lone carriage return hid fences and brackets from it
  (draw 11096 of the seed 27182818 threw):
  red `0412d71d8344ce03c76667948d61b88086fa49fa` (`red-r12`,
  2 failing cases),
  fix `0fda29e1d0562203ee799d9c0b93fd04ec302401`,
  the lines cut at a line feed,
  a carriage return and both;
  `4b8d84aa361fdb979b5ac2047acaa3c943445ac5` names the function `linesAtEveryEnding`,
  since `linesOf` is a name `archive` exports.
- 11 of the 15 are a footnote followed by a line of raw HTML.
  A reader of the source in GFM ends the footnote at the tag,
  and the page,
  which writes every less-than sign as text,
  goes on with the footnote's paragraph over it,
  so a code span pairs across the lines and a definition line is a lazy line.
  Red `2927ecec0eec8068dafb9d8ec3d25424a8411b7b` (`red-r13`,
  1 failing case),
  `29b90243c88ffc29c91c526912d7297447bb104a` read the ranges with raw HTML off for every caller.
- A lazy line with a fence mark indented four spaces after a footnote in a container
  (draws 9633 and 33130),
  a footnote in a container followed by blocks of the top,
  and a block that two readers reach and so put the same edit twice.
  Red `a830a40e8a3ce1da81cb941cf0240bb99389809c` (`red-r14`):
  the 16 pairs and 5 shapes,
  13 failing cases.
- Braces that the measure of nesting counted open in front of a fence whose information string holds
  a long line of brackets,
  which the measure then counted as nested:
  a brace inside a quoted string of an open expression
  (red `53a0548d9eb7bbd7a7562a1762b576e7a7627116`,
  `red-r15`,
  2 failing cases;
  fix `ebdd01317365886efe1799e19a259a264d09f57d`),
  and the braces of a link reference definition,
  which read a brace as a character and open no expression
  (red `f878ffbed6619ef972edf3b9d5dc6c71e19091a3`,
  `red-r16`,
  1 failing case;
  fix `286e7979b4c70aca929b17a4e56262e1ee46acef`).
  Draw 21529 of the seed 27182818 was refused for a depth it did not have.

#### Two readings of a footnote, and the choice between them

`cd61a365fdc67aa115efe119fa0d7578f0052e9c` closed the 16 pairs by reading the ranges of the page everywhere.
It turned 16 existing leaf cases red:
the reader equality cases of `source-ships-encoding.unit.test.ts` (13),
whose oracle reads the source in GFM with raw HTML on,
and one case each of the container,
pending and separation files.
Where the oracle sees a definition,
that version shows the line as text.
Its commit body says so,
and it was not kept as the end state.

The end state is the owner's decision,
answered on 2026-10-25 as "Two readings (Recommended)",
and it is no longer open to a veto.
`footnoteDefinitionRangesOf` reads the source's ranges with raw HTML on,
and `pageFootnoteDefinitionRangesOf` the page's with it off
(`1ab47ca1f5dbb8790764eb7d1352044fc22afeb3`).
Only two places use the page's ranges:
a footnote in a container,
and the blank line after a footnote's last block,
which is put before a paragraph as well as a definition where the page goes on over a line of raw HTML.
That keeps the top of a text as the oracle reads it,
and the page differs from the source by one blank line where the footnote ends at a tag.
The alternative the owner declined is the page's ranges everywhere with the oracle changed to read the page,
which the state of `cd61a365fd` reaches with no finding;
it moves the claim "the page shows the source text" to "the page shows the page's reading of the source text".
`24cdb8ac739d0f9b3b4ded019727621797c5376f` carries the page-check side,
and also makes the encoder make an edit once where a block is reached twice.

#### Result of the full task on the final code

On `6ebbd6677650d44aed8bf8a3cfe62dc688fcda08` the task passed:
the case that holds the three names in 762.4 s,
`systemd-run --wait` reporting a service runtime of 12 min 48.955 s,
12 min 46.327 s of CPU time and a memory peak of 687.1 MB with 320.3 MB swapped.
The host load average was 58.75 when it started and 26.15 when it ended.
The first run took 308.3 s,
with a memory peak of 746.7 MB.
Both differ from the 535 to 559 MB of the measurements made before this round,
whose method is not recorded here;
these are the peaks of the unit's cgroup.
A scan of every draw of both seeds on the code with the repeated-edit filter written inline,
before it was named `withoutRepeatedEdits` and before `linesOf` was renamed,
found 0 findings and 0 throws
(seed 31415926:
40000 pairs,
32121 with literal pages;
seed 27182818:
40000 pairs,
32018 with literal pages).
The stderr file of the final run holds 100 lines stamped 03:18Z that are the findings of the first failed run.
Both runs wrote their stderr to the same path with `StandardError=file:`,
which `systemd.exec` (the man page under `StandardOutput=`) says opens a file
"for writing at the beginning of the file,
but without truncating it",
so the short stderr of the final run
(two preamble lines and a logger message that a sink could not be verified within 5000 ms)
overwrote only the start of the longer stderr of the first run,
and the rest of that file is the tail of the first run's.
The exit status is 0,
and the PASS line is stamped 05:04Z in the stdout file of the final run.

#### Limits and open findings of this round

- Fixed in the third round (the section on the owner's ruling and the indent-4 probes):
  the indent-4 probes `span-top-ind4-tag`,
  `span-top-ind4-brace`,
  `span-quote-lazy-ind4-tag` and `span-quote-lazy-ind4-brace` failed on the head of this round:
  a code span across a footnote line indented four spaces.
  The strict grammar disables indented code,
  so a footnote definition may start at any indent there,
  while CommonMark reads the line as a lazy line of the paragraph.
  The ruling named indents 0 to 3,
  the generators of the pools write none at 4,
  and the four fail on the base too.
  The fix reads a footnote start at any indent in the strict grammar.
- The source scan of the own-test ratchet gained two own tests (`without-raw-html.unit.test.ts` and
  `source-ships-separation.unit.test.ts`),
  and `withoutRawHtml` and `separatingEdits` are exported for them.

#### Checks of the second fix round

`build`,
`test:unit` (155 `[PASS]` lines and no `[FAIL]` for `page-check`,
62 for `document-structure`,
51 for `markup`),
`lint:types` and `lint:oxlint` (0 warnings and 0 errors) of the three packages,
and `test:unit` (68) and `source-scans` (53) of `translation-repair-source-scan`,
run one at a time.
No cache version moved:
`git log --oneline --extended-regexp -G '_CACHE_VERSION = [0-9]+' 1c7918806f..HEAD` names no commit,
and the same command over `c265e8df95~1..c265e8df95` names `c265e8df95`.
The nesting measure feeds the markup atom scan,
whose result enters `markupDelta` in the page pass (`apply-patch.ts`),
and no cache key reads it,
so the change of the bound is no cache key either.
No new B id was taken:
the next free one is B842.

#### Third round: the owner's ruling and the indent-4 probes

The owner answered on 2026-10-25 "Two readings (Recommended)".
That is the decision recorded in the section on the two readings of a footnote:
`cd61a365fd` with its 16 red leaf cases is the alternative the owner declined,
and it is not the end state.
The choice rests on one condition,
that the page check catches a page which reads differently from the source.
The search of this round tests it,
and the answer is that it does not hold for every text.

`5c874bbe52` adds `source-ships-indent-four.unit.test.ts`,
eight cases that all failed:
the four probes `span-top-ind4-tag`,
`span-top-ind4-brace`,
`span-quote-lazy-ind4-tag` and `span-quote-lazy-ind4-brace`,
and four more at six spaces,
a tab,
four spaces after the marks of a block quote and four spaces under the text of a list item
(the failing lines are in `red-r17-failing-lines.txt`).
`6bb3065bd0` adds the case of `pageFootnoteDefinitionRangesOf` that the fix needed,
which reads a footnote start at a line indented four spaces or more
(`red-r18-failing-lines.txt`).
The fix is `873e6ac8ad` in `document-structure` and `fb82a8f537` in `page-check`.
The strict grammar disables indented code,
so the page's ranges are now the ranges with raw HTML off,
plus the starts of a second parse with indented code off as well,
where that start lies outside every range of the first.
A parse with indented code off alone was tried and regressed the sampled pairs:
a fence mark indented four spaces opens a fence there,
which hides the footnotes of a container after it,
and the encoder does not leave such a mark as a fence.
At the top of a text,
`withPageOnlyStarts` joins the source's ranges with the page's starts that the source does not have,
so a code span the original pairs across the line is cut where the page cuts it.
The 80-probe grid reads 0 refused and 0 live on the final code,
where the head of the second round read 2 and 2,
and the 24 shapes of the ruling parse,
hold no live construct and are reader-equal.
The 10 pairs of the two c10 scan files that listed findings read 0 findings on the final code.

#### Third round: the search for a top-level reading difference

The search is a script of the run directory (`topdiff.ts`,
not part of the repository).
Its texts are every first and second run of the 40000 pairs of each seed (31415926 and 27182818),
the texts of `footnoteProductTexts`,
and a grid of shapes:
six footnote openers,
nine tags or raw HTML lines and 23 following blocks,
each with and without the final line ending,
and a footnote start at indents 0 to 6 after nine leads.
That is 132260 distinct texts,
96192 of them with a label opener.
A text is in scope when the source's ranges and the page's differ,
and the plain parse of the text holds no block quote and no list at all.
For each text in scope the search encodes the text,
reads the page with the strict grammar,
and compares the reader of the page with the reader of the source (`readerOfPage` and `readerOfSource`,
link targets aside).
It then asks the page check (`readText` of `source-live-pairs.test-fixture.ts`)
whether it finds a live construct,
a refusal or a drop.
It also takes a control for each different text:
the same text with every `[^` written `[`,
so that no footnote is read.

The page check compares no reading:
it finds live constructs,
refusals and unforced drops,
and that is all.
So a page that reads differently from the source is caught only when it is also live or refused.

On `1a5f953284`,
before the third round,
10856 texts were in scope:
9678 read the same,
6 were refused,
and 1172 read differently with the page check silent
(1042 from the pairs and 130 from the shapes).
Two causes were fixable and are fixed red first.

The first is the blank line after the end of a footnote,
where the page goes on with the footnote's paragraph over a line of raw HTML.
`separatingEdits` put it only before a definition or a paragraph;
a heading underlined by equals signs,
a table or a definition after a tag that the source ends the footnote at
showed as lines of the footnote's paragraph on the page.
The second is the same on a block of the top that the page writes as a paragraph:
raw HTML of the kinds that end at a line of their own
(a processing instruction,
a declaration,
a CDATA section,
a script,
a pre or a style element),
and indented code,
which the page has none of.
The line right after it was a lazy line of that paragraph.
`45b25ac221` adds ten failing cases (`red-r19-failing-lines.txt`) and `f4f533493a` makes them pass:
`separatingEdits` applies wherever the page goes on,
and `afterLiteralBlockEdits` (first named `afterHtmlBlockEdits`) puts the blank line after a raw HTML block.
`cc7aa64640` adds three failing cases for indented code (`red-r20-failing-lines.txt`),
and `a5faefbb71` makes them pass.
That commit also moves two cases of `source-ships-encoding.unit.test.ts`,
which pinned a definition line right after indented code at the top as shown text,
to the cases that compare what a reader sees.
In a block quote or a list item the same blank line is not written,
because it needs the marks of the container.
That is a named limit,
closed in the fourth round.

On the final code,
`a5faefbb71`,
11423 texts are in scope,
because the page's ranges now also hold the starts at indent 4 and 19789 texts have ranges that differ,
8366 more being in or beside a container and out of scope.
10016 read the same,
none is refused,
and 1407 read differently with the page check silent.
They fall in classes that the third round did not fix:

- 925 texts hold a comment that never closes.
  The reader of the source reads the rest of the text as one raw HTML block,
  where the encoder writes `&lt;!--` on purpose and the page reads Markdown.
  851 of them read differently with the label openers removed,
  so the footnote ranges are not the cause.
  The other 74 differ only because a footnote sits after the opener.
- 57 texts hold an image whose alternative text is an angle autolink:
  the page shows the angle brackets and the source shows the address alone.
  This is an open finding of the first rounds and differs without any footnote
  (`![<https://example.net/a>](u.png)` alone reads differently).
- 384 texts hold a footnote start indented four spaces or more after a paragraph with a code span across the lines.
  The page's grammar reads the footnote,
  so the code span is cut and the page shows the backticks as characters,
  where the source shows the code.
  No escape can keep both,
  and the encoder only makes the page safe (no live construct).
  Before the fix of the indent-4 probes the encoder did not cut the span there,
  and the page held live constructs (the four probes);
  now the page is safe and differs by the backticks.
- 37 texts hold a footnote whose first line is a tag alone,
  followed by a tag alone and Markdown syntax
  (`[^1]: <b>` then `<q>` then a code span).
  The source's reading ends the footnote after its first line and reads the rest as a raw HTML block;
  the plain parse of the encoder reads a paragraph,
  so the encoder leaves the code span a code span on the page.
  A fix would parse the text after a footnote's end as a text of its own.
  It is not made here.
- 4 texts mix an indent-4 fence mark with a footnote.

The answer to the condition is therefore that the page check does not catch every page that reads differently.
The count is 1407 of 11423 texts in scope,
12.3 percent,
in the draws and the shapes named,
outside a block quote and a list item.
Whether to add a reader comparison to the page check,
which would refuse or drop those pages,
is a choice for the owner;
the owner decided on 2026-10-25 to add it,
and the fourth round did.

#### Third round: the full task, the merge and the checks

The merge of `origin/main` at `d91f415253` conflicted with nothing (`git merge-tree --write-tree` exited 0),
and is `68018d2f6d36e8897c88f4c1c8f04e58f8de9c33`.
It brought no change to `page-check`,
`document-structure` or `markup`.

The full task ran once on `68018d2f6d36e8897c88f4c1c8f04e58f8de9c33`,
as the unit `dx1f2-livefull3` with `MemoryMax=4G` and `CPUQuota=300%`.
Its stderr went to a path never used before,
because `StandardError=file:` opens a file without truncating it.
`systemd-run --wait` reported success,
a service runtime of 12 min 8.162 s,
12 min 45.618 s of CPU time and a memory peak of 708.2 MB with 301.1 MB swapped.
The case that holds the three names passed in 719.2 s.
The one-minute load average of the host was 36.64 when the unit started
and 49.71 when it ended.
The stderr file holds three lines:
the two preamble lines of the task
and the logger message that a sink could not be verified within 5000 ms.
After that run the head moved only by `4fd5c77878`,
which changes two test files and no source.

The checks run one at a time on that head:
`build`,
`lint:types`,
`lint:oxlint` (0 warnings and 0 errors) and `test:unit` of `page-check`
(159 `[PASS]` lines and no `[FAIL]`) and `document-structure` (63).
`markup` is untouched in this round.
The first `test:unit` of `translation-repair-source-scan` after the merge had 10 `[FAIL]` lines and six failing cases:
3 for `COMMAND_LINES`,
because the built `run` package was older than the merge and the task `source-scans` builds before it reads,
and 3 findings of my own,
which the third round fixed in `4fd5c77878` and in the wording of this file:
two suites awaited at the top of the separation test file,
a case of `without-indented-code.unit.test.ts` that copied one of `without-raw-html.unit.test.ts`,
and a word in this file that points by position.
After the fix `source-scans` ran 53 `[PASS]` lines and `test:unit` 68,
with no `[FAIL]`.
No cache version moved:
`git log --oneline --extended-regexp -G '_CACHE_VERSION = [0-9]+' 1c7918806f..HEAD` names no commit,
and the same command over `c265e8df95~1..c265e8df95` names `c265e8df95`.
No new B id was taken:
the next free one is B842.

The fourth round is in
[its own file](audit-ledger-source-text-follow-ups-the-reading-comparison-and-the-real-corpus.md):
the comparison of the page's reading with the source's,
the real corpus,
the residual classes,
the full task of that round
and the slips of the third and fourth rounds.
