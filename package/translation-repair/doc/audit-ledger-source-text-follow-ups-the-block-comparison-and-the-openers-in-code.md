# Audit ledger: the source text under a note, the block comparison and the openers in code

The fifth round of the follow-ups of B832 (the original of an untranslated passage written as literal text):
the fixes of the re-check of the third and fourth rounds,
the comparison decided for each top-level block,
the comments that never close taken out of code and math,
the pins and mutants of the round,
and the search,
the real-corpus counts and the full task of the final code.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.
The rounds before it are in
[the file of the fourth round](audit-ledger-source-text-follow-ups-the-reading-comparison-and-the-real-corpus.md),
whose passages on the indent-4 code span and on the comments that never close are corrected here.

## B832 follow-ups, the fifth round

### The re-check and the owner's rulings

An independent re-check of the third and fourth rounds found no major finding,
five minor ones (F1 to F5) and four nits (F6 to F9).
Two rulings of the owner stand:
a run always ships (2026-09-27),
and "Add the comparison" (2026-10-25),
which means every page that reads differently from its source is reported,
no report blocks shipping,
and the classes are fixed at their source.

What each finding came to:

- F1,
  the indent-4 code span and the comment that never closes:
  both fixed,
  red first.
  The ledger of the fourth round said no writing keeps both readings of a code span that an indent-four footnote
  start cuts,
  and counted 12 openers.
  Both statements were wrong.
  A line ending and the indentation after it are one space inside a code span,
  inline math and a link title on any render,
  so the encoder joins the lines (`e8a2a56ed0c1514b456f24ddc47e47cb62681591` red,
  `c547f29230c179b2542d2376ccdf6669dde3464f` fix).
  The opener of a comment that never closes was the cause of 236 differences,
  not 12,
  and 232 of them are fixed by taking what holds the opener out of its code or math
  (`af148afb1c493eeafb33de8f383b7a3b2ebb21c4` red,
  `0117118d994e798aacf1890e3a760ea0707c7b74` fix).
  The 4 left are listed in the section on the residual classes.
- F2,
  `undecided` withheld other differences:
  the comparison is decided for each top-level block,
  and a block left undecided ships a defect of its own,
  `source-reading-undecided`.
  Four reds and four fixes:
  `29f4deebdcd701816a1f270c0101f43aaf86367f` with `ea368918c35828f6f332d75cabee5d3335efdb81`,
  `e5f5231deaa0ef99954dc11b972e087291984ffd` with `04f6e4d033fe8d776f8be500510ca3d0f9fa6db2`,
  `456f0ad9c683c179edfe5276cd0d75e1a087689d` with `b1ddb3704024ac0d5670cb8b12d922f7103a5eff`,
  and `25aebef5a995620d4e003563146da4c9ec88f187` with `cb0bc554279b34871120820a4682d900af98f082`.
  The last three each answer a measurement of the search that the one before it left.
- F3,
  the three survivors:
  pinned by `e3ee3725ab298a6bbcb06589f474f2f27be547b9`,
  with a mutant of each shown to die (section on the mutants).
- F4,
  what was red first:
  recorded in the section on what was red first,
  and the comparison behaviour that no case failed without now has one
  (`17e405688f0bfe709f2fac9a96534aae1ea9eeed`).
- F5,
  the comparison and its defects in the places a reader looks:
  the design commitments of shipping and the archive,
  the README of `page-check`,
  the README of `translation-repair`,
  the README of `run` and the header of `publish-defects.ts`.
- F6 to F9,
  the nits:
  the gate rule of the README names `markup/src/nesting-*.ts`,
  the run figures are refreshed,
  27 TSDoc lines lose their article,
  ten capitalised run-in labels of the fourth-round file are headings,
  and the hook TSDoc names the parses it is not armed at.
- F7,
  a page that parses and cannot be compared:
  ships the defect `source-reading-not-compared` (`b8e339c879fedeb78ccfc4422ff7bb17654e52f5` red,
  `f983ecc7fcbb918825d4e1ed2a170de3bc82c7e7` fix).
  The throw is unreachable from the composition today (the re-check composed all 5303 texts that make the
  comparison throw,
  and every one ends in the dropped step with `page-source-unparseable`),
  so the case is pinned at `differenceDefectsOf`.

### The comparison now

`readingDifferenceOf` still answers `same`,
`differs` or `undecided` for a run written as literal text,
and a text whose two accounts agree is `same` and says nothing.
Where they do not agree and no footnote range the two readings disagree on starts inside a block quote or a list
item,
the answer is `differs`,
as before.
Where such a range exists,
the blocks of the source it reaches (any block that overlaps the range of either reading) are set against the
page apart from the rest (`source-reading-blocks.ts`):
the words of every other block must be found in the words the page shows,
in order and wherever the page breaks them into blocks,
the places being chosen together so that every block can be placed;
the words between two of them are compared with what the blocks the range reaches show.
A block that cannot be placed is a difference,
a stretch that shows other words is undecided,
and a block that stands alone as a difference is reported whatever stands beside an undecided one.
The accounts of the blocks leave a link's target out,
as the account of a whole text does.

Three defects carry the answers,
each naming the slices of the run and no text of the page:
`source-reads-differently`,
`source-reading-undecided` and `source-reading-not-compared`,
in that order.
A run that differs and also holds an undecided block ships the first two.

### What was red first, and what was not

The reds of this round fail on their parent for the stated reason (the failing lines of the test run of each red,
which the entries of this section quote,
and which were read from scratch output files of the run;
that scratch output is not kept in the repository),
and none is a load error:

- `b8e339c879` (F7):
  1 case,
  `expected [] to deeply equal ['source-reading-not-compared']`.
- `29f4deebdc` (F2):
  4 cases across three files,
  each with an expected value the old answer does not give;
  a fifth case,
  which keeps a text undecided,
  passes on it.
- `e8a2a56ed0` (F1,
  the code span):
  14 of 15 cases;
  the fifteenth,
  indented three spaces,
  is a control that both readings cut and passes.
- `af148afb1c` (F1,
  the openers):
  32 of 33 cases;
  the case with the opener beside a code span passes.
  The exact written text of the case of the block quote was wrong in that commit (the text ends with the block
  quote and no closing mark) and is corrected in `0117118d99`.
- `e5f5231dea`,
  `456f0ad9c6` and `25aebef5a9` (F2,
  three measurements):
  one case,
  two cases and one case fail for the stated reason
  (the sixth round corrected "one case each",
  which was wrong for `456f0ad9c6`).
- The pins of F3,
  of F4 and of the line ending (`e3ee3725ab`,
  `17e405688f`,
  `ffbed7b5eb`) are regression pins on code that already passed.
  They were written to kill a mutant,
  and each was shown to fail on it.
- The unit cases of `containerFootnoteDisagreementsOf` (in `ea368918c3` and `04f6e4d033`),
  of `source-reading-blocks.ts`,
  of the new opener modules and of the two parts of the encoder that have their own files were written with the
  code or after it,
  not before it.
  The behaviour they serve was red first at the level of `readingDifferenceOf`,
  `differenceDefectsOf` and the encoder.

What was red first in the commits of the fourth round,
as the re-check measured it on a copy of each tree:

- `76d1c2e53a`:
  `source-reading-difference.unit.test.ts` did not load (the export `readingDifferenceOf` did not exist),
  so none of its five cases was red for its reason.
  Of the three cases of `source-reading-defect.unit.test.ts` one fails for the stated reason
  (`expected [] to deeply equal ['source-reads-differently']`) and two pass.
  The statement of the fourth round that the comparison's behaviour was red first holds for that one composition
  case.
- `dedb7886c0`:
  the tests of `container-footnote-ranges` and `gfm-math-body` were written with the code.
  Against the parent both files fail to load,
  so no case ran.
  Against the code,
  the mutants that drop the container test,
  keep quotes only,
  drop GFM and drop math die (2 cases each).
- `2797c8f871`:
  the tests of `source-reading-account`,
  `source-ships-differences` and `source-reading-difference`,
  put on the parent `b8c0b1dbff`,
  all pass (5,
  1 and 1 suites).
  They are regression pins and not reds.
  Of the mutants of the comparison,
  the one that reports `undecided` as `differs`,
  the one that reports syntax as `same`,
  the one that reports a throw as a difference and four more die;
  the one that keeps link targets on the page's side survived the comparison's own files and died only in
  `source-shipped-page.unit.test.ts` (4 cases),
  which `17e405688f` repairs;
  the one that drops the difference defects of the dropped step survived everything,
  which `e3ee3725ab` repairs.
- `6ef07ed14a`:
  four cases fail on the parent.
  With the oracle of the fix commit (`readerOfSource` now blanks the comments that close when another does not)
  and the parent's encoder,
  three still fail;
  the case with a closed comment before the open one passes.
  That case was red because of the oracle the same commit changes and not because of the encoder.

### The mutants of the round

Each was made on a copy of the source in the working tree,
built,
run against the test file named,
and removed.
All of these die:

- `inContainer` passed as false at the two sites of `source-ships-spans.ts` (F3):
  3 cases of `source-ships-container-blank-line.unit.test.ts`.
- The difference defects of the dropped step removed (F3):
  1 case of `source-reading-defect.unit.test.ts`.
- The direction of `container-footnote-ranges.ts` where the page has a range the source lacks removed (F3):
  1 case on the function of that commit,
  and 4 cases of `container-footnote-ranges.unit.test.ts` on the final function.
- A disagreement placed in no block taken as placed (`unplaced`):
  1 case of `source-reading-blocks.unit.test.ts`.
- A block reached only where it holds the first character of a range:
  1 case of the same file.
- The span kept as code whatever stands before the opener:
  2 cases of `source-ships-opener-in-code.unit.test.ts`.
- Display math not looked for:
  3 cases.
  The marks of the container taken as the indentation of a fence at the top:
  1 case.
  The lengthened opening mark of a fence ignored:
  1 case.
- The lines added around an opener written with a line feed alone:
  survived every case until `ffbed7b5eb` pinned the written text of a fence in a text of Windows line endings,
  and dies on it (1 case).
- Every footnote start the page reads taken as one the source does not (`source-ships-spans.ts`):
  1 case of `source-ships-footnote-code-span.unit.test.ts`,
  the control indented three spaces.
- The two mutants of `source-reading-blocks.ts` (a disagreement placed in no block taken as placed;
  a block reached only where it holds the first character of a range) died first on the version of the file of
  `b1ddb37040`,
  which placed each block at the first place its words stand on the page.
  The file was rewritten in `cb0bc55427`,
  so both were made again on that final source and each died (the case of the unplaced disagreement,
  and the case of the range that runs over two blocks).
- The page's account keeping the targets of links in `readingDifferenceOf`:
  1 case of `source-reading-difference.unit.test.ts`.

### The search on the final code

The texts are those of the third and fourth rounds (132260:
every run of both seeds' 40000 pairs,
the footnote product texts and the shapes),
each put to the production comparison by a scratch script that asked the oracle and the comparison about every text
and counted the answers
(the script and its output are not kept in the repository,
and the counts of this section are what its last run printed;
the unit was capped with `MemoryMax=8G` and `CPUQuota=800%`,
and the search ran five times:
after each of the fixes that the search itself called for,
and a last time on the head `d57b80ea0ab4c4c2541ed43d2a87f2db4642aeaf`,
whose code is that of `4f1ba3055b8d288e320bfed893d06889ff487927`;
the run before it,
on `cb0bc554279b34871120820a4682d900af98f082`,
gave the same counts and the same texts,
since `4f1ba3055b` only removed a function and an unused helper and made one filter call another).

The answers of the final code:

- the oracle and the comparison agree that the text reads the same:
  117000 texts,
  none of them reported (116341 before;
  the other 659 are 658 texts that were `differs` and 1 that was `undecided`);
- `differs`:
  8838 texts (9442 before),
  of which 54 also hold a block left undecided;
- `undecided`:
  1119 texts (1174 before),
  each reported by `source-reading-undecided` where it was only logged before;
- could not be compared:
  5303 texts (the same 5303 as before),
  whose page does not parse because the text nests past the bound;
  the composition leaves them off the page with `page-source-unparseable`,
  and a text that parses and cannot be compared ships `source-reading-not-compared`;
- texts that read the same before and are reported now:
  0.

Of the 1174 undecided texts of the fourth round,
1 now read the same,
54 are `differs` (the footnote was not the whole difference) and the rest stay `undecided`.

On false positives:
a `differs` reported by the new block rule is the only new kind of report that a text reading the same could
receive,
since the other answers are fixed by the oracle of the whole text.
There are 54 such texts.
Asked block by block on its own (encoded alone),
34 of them hold a block that differs by itself.
The other 20 were read by their accounts,
outside the stretch of the footnote:
15 show the quote mark of a math block in a block quote as text with the lone carriage returns of the text,
3 show a footnote label with a brace written with a backslash in the references and not in the definition (a
limit recorded since the ninth round),
and 2 show a definition label of the source as text.
Each is a difference of the page from its source.
No false positive was found:
0 of 54.

In the other direction,
7 of the 1119 undecided texts hold a block that differs when it is asked alone;
the block is one a range reaches,
which the ruling puts out of the comparison's reach,
so the text stays undecided and is reported.

### The real corpus

The pinned clone `one-among-us/data` at `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`,
read with `git cat-file` and never written, by a scratch script that listed the blocks of each file and counted the
openers and the comparison's answers
(the script and its output are not kept in the repository;
the unit was capped with `MemoryMax=4G` and `CPUQuota=200%`).
Counts only;
no text of the corpus is written here.
It ran twice,
on `cb0bc55427` and on the head `d57b80ea0a`,
and the two outputs are identical.

- 276 files `page.md`,
  `page.en.md` and `page.zh_hant.md`,
  274 distinct texts,
  8423 top-level blocks and 7360 distinct ones.
- 55 files hold a comment opener and 68 hold a footnote label opener;
  118 distinct blocks hold the first and 296 the second.
- None holds the lexical trigger of a class:
  a comment that never closes,
  an image whose alternative text is an angle autolink,
  an indent-4 footnote start,
  a footnote that opens with a tag alone,
  an indented fence mark or an indented line after a definition.
  1 file holds a carriage return.
- The two readings of a footnote disagree inside a container in 0 files and 0 blocks.
- 61 of the 7360 blocks ship as literal text,
  and the comparison answers `same` for all 61:
  61 same,
  0 differs,
  0 undecided,
  0 could not be compared.
  The other 7299 ship as recorded and are not compared;
  of those,
  the encoded form of 7298 would read the same and one would not (the front matter of one entry,
  which ships as recorded).

### The residual classes

- The indent-4 footnote start that cuts a code span:
  fixed.
  No text of the search holds it as a cause of a difference any more.
- A comment that never closes:
  232 of the 236 texts fixed.
  The 4 left hold the opener in a code span that only the reading of a footnote in a block quote pairs;
  the plain parse that the opener rewrite reads has a text node there and no code span (the chain from the root to
  the opener is `root>blockquote>paragraph>text` in all 4),
  so the rewrite does not see the span.
  The comparison reports them as `differs`.
  Fixing them means running the rewrite on the nested reads of `spansOf`,
  which is not done.
  Real corpus:
  0.
- An image whose alternative text is an angle autolink:
  1283 texts of the search read differently because of it and no other cause (replacing the angle brackets makes
  them equal),
  12 more are undecided.
  Known since the parser gaps ledger and reported as before.
  Real corpus:
  0.
- A line indented four columns or more after a definition that begins with a heading mark,
  a list marker or a quote mark:
  reported,
  not fixed,
  as in the fourth round.
  Real corpus:
  0 files.
- A footnote label with a brace,
  written with a backslash in the references and not in the definition:
  reported,
  a limit since the ninth round.
- A lone list marker after indented code reads differently in the two parses (observation F10 of the re-check):
  reported,
  not fixed.
  The comparison is blind to structure (a footnote definition against text,
  one quote against two,
  link targets and titles):
  unchanged.

### Cache versions

`git log --oneline --extended-regexp -G '_CACHE_VERSION = [0-9]+' 56514f37f1..HEAD` names no commit,
and the same command over `1c7918806f..HEAD` names none either;
over `c265e8df95~1..c265e8df95` it names `c265e8df95`.

The markup label rule and the nesting measure feed no cache key.
The files that define or read a cache version are the key modules of `translate-lane`,
`repair`,
`lane-contest`,
`artifact`,
`slice-reading`,
`consistency`,
`shared-contract` and `corpus-pass`;
of the packages whose names they import,
they take from `document-structure` only `hashContent`,
`isJsonArray` and `isJsonRecord`,
and from `markup` only `codePointCount` (`translate-alignment.ts`).
The label rule (`footnote-label.ts`) and the nesting measure (`firstNestingExcess`,
`NESTING_BOUND`) are read outside `markup` by `parse-mdx.ts`,
`parse-markdown-math.ts`,
`gfm-block-ranges.ts` and `module-line-starts.ts` of `document-structure`,
by `patch-nesting.ts` of `shared-contract` (the refusal of an editor's patch),
by `apply-patch.ts`,
`footnote-closure-input.ts` and `title-reference-scope.ts` of `page-pass` (the preservation check of a patch),
and by `page-check`.
All of them judge an input or an answer after the key is computed,
so a changed rule changes which texts are refused,
not what a key hashes for a text.
No version is bumped.

### The full task, the merges, the gates and the slips

The merges of `origin/main` conflicted with nothing (`git merge-tree --write-tree` exited 0 each time):
`b6927f7ba93f5139fa0605ef160c6f44c60fa32d` at the start of the round
(`d92e03f53944aa219ac133e1f11203416321332b`) and `6457f1f09c85c79c7b1b1f87d2b31314dfe4ec4e` before the gates
(`5bc9c77c0b339f23827abc10cad7883d5a9ee0fb`).
Neither brought a change to `page-check`,
`document-structure`,
`markup`,
`source-scan` or `run`.

The full task ran once,
on `d782255a2cdc753ad25d21ccc99e3a7a0602bdd2` (the last commit that changes code is
`4f1ba3055b8d288e320bfed893d06889ff487927`;
the commits after it are Markdown and the comment header of one test file),
in its own capped unit,
`dx1f3-livefull`,
with `MemoryMax=8G` and `CPUQuota=800%` and a stderr path never used before.
It passed:
the case that holds the three names ran in 656.9 s,
the unit reported success after a wall clock time of 10 min 58.707 s,
12 min 55.130 s of CPU time and a memory peak of 742.6 MB with 13.7 MB swapped.
The one-minute load average of the host was 11.17 when the unit started and 23.82 when it ended.
The stderr file holds three lines:
the two preamble lines of the task and the logger message that a sink could not be verified within 5000 ms.
The unit was started without `--wait`,
so its summary was read from the journal.

The gates ran one at a time,
from the worktree,
on the final code,
and every one passed:

- `document-structure`:
  `build`,
  `lint:types`,
  `lint:oxlint` (0 warnings and 0 errors),
  `test:unit` (65 `[PASS]` lines,
  no `[FAIL]`).
- `markup`:
  the same four (51 `[PASS]` lines);
  its only change is the opening of one TSDoc line.
- `page-check`:
  the same four (186 `[PASS]` lines,
  6 more than the fourth round's 180).
- `source-scan`:
  `test:unit` (68) and `source-scans` (53),
  and `lint:types` and `lint:oxlint` for the file it gained an entry in.
  `run` changed its README only,
  so no gate of it ran.

The first `source-scans` run on the code of this round failed these,
all fixed by `4f1ba3055b` and the Markdown commit:
two modules with no unit test of their own,
a function nothing reaches (`joinedAccounts`) and an export nothing reaches (`footnoteRangesDifferInContainer`),
a byte order mark written literally in a test,
two test files with more than one top-level awaited describe,
three blankness tests not listed,
a function body written twice,
two TSDoc comments before a statement,
a dead link to the file of this round before it existed,
and Markdown lines the lint breaks (a break is wanted after every comma,
semicolon,
colon and full stop).
The second run found one phrase pointing by position and one more TSDoc before a statement.

The commit-message slips of the third and fourth rounds that the re-check found beyond the list of the fourth
round,
none corrected on the remote and nothing posted:

- `e9a4920bcf`,
  `bb0d27d04a` and `5a3b6f7085` are multi-package commits whose bodies use paragraphs headed
  `translation-repair-<package>:` and not the shape `<type>(<package>): <what>`,
  a blank line and the why.
- `cd61a365fd` has a body sentence in capitals.

Slips of the commit messages of this round:

- `f983ecc7fcbb918825d4e1ed2a170de3bc82c7e7` ends "The case of the preceding commit passes",
  a positional phrase.
- `ea368918c35828f6f332d75cabee5d3335efdb81` says the other 180 suites pass with 2 new suite files;
  the run reads 184 suites,
  a count that includes the new files.
- `af148afb1c493eeafb33de8f383b7a3b2ebb21c4` splits the 147 texts whose opener stands in a fence at the top into
  115 and 23 and leaves 9 out.
