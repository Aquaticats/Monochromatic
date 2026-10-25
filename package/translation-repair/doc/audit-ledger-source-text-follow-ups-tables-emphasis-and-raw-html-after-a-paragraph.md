# Audit ledger: the source text under a note, the sixth round, tables, emphasis and raw HTML after a paragraph

The sixth round of the follow-ups of B832 (the original of an untranslated passage written as literal text):
the fixes of the third re-check,
the footnote start indented four columns after a table row,
in emphasis and link text and after a backslash hard break,
the comment openers that cut a paragraph or stand in a footnote's fence,
the block of raw HTML that cuts a paragraph,
the pins and mutants of the round,
and the search,
the differentials,
the real-corpus counts and the full task of the final code.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.
The rounds before it are in
[the file of the fifth round](audit-ledger-source-text-follow-ups-the-block-comparison-and-the-openers-in-code.md),
whose passage on the red of three measurements is corrected there,
and [the file of the fourth round](audit-ledger-source-text-follow-ups-the-reading-comparison-and-the-real-corpus.md).

## B832 follow-ups, the sixth round

### The re-check and the owner's rulings

The third re-check of this work found no major finding,
five minor ones (N1 to N5),
three findings left partly fixed (F1,
 F7 and F8 of the fifth round),
and five nits and observations (N6 to N10).
Two rulings of the owner stand:
a run always ships (2026-09-27),
and "Add the comparison" (2026-10-25),
which means every page that reads differently from its source is reported,
no report blocks shipping,
and the classes are fixed at their source.

What each finding came to:

- N1,
  the regression of the join of `c547f29230c179b2542d2376ccdf6669dde3464f`:
  fixed,
  red first.
  The join reads its spans from the plain parse,
  which has no tables,
  so a table row holding a backtick that nothing closes,
  and a footnote line indented four columns after the row,
  were read as one paragraph with a code span across them,
  and the line was joined into the row.
  The page lost the closing pipe of the row,
  or took the tag of the line and did not parse,
  and the composition shipped the run without its original.
  `startsOffTables` (`page-check/src/corpus-run/source-ships-footnote-ranges.ts`) takes out of the starts the ones
  whose line follows a row of a table that the GFM reading finds,
  and `spansOf` hands the rest to both consumers,
  the paragraph cut and the line join,
  so they agree.
  A start so taken out is cut as any other,
  which is what the code before the join did.
  The brace and tag joins inside a table row,
  which the third re-check found already wrong before the join,
  are not changed and are listed with the residual classes.
- N2,
  emphasis,
  strong emphasis,
  the text of a link and the alternative text of an image cut by a footnote start indented four columns:
  fixed,
  red first,
  with a hard break written with a backslash,
  which the re-check named without listing it among the four.
  A line ending and the indentation after it are white space in all of them on any render,
  so the encoder joins the lines at a footnote start only,
  and a brace,
  a less-than sign or a fence mark that starts a line there stays escaped as text.
  The references (`linkReference`,
  `imageReference`) are joined as the inline forms are.
- N3,
  an opener that starts a line inside a code span,
  and an opener in a fence or display math held by a footnote definition:
  both fixed,
  red first.
  The first is one case of a wider class that the fix shows:
  a line that starts a block of raw HTML of the kinds that may interrupt a paragraph cuts that paragraph short for
  a reader of the source,
  and the page,
  which has no raw HTML,
  went on with the paragraph and paired a backtick with a line of the block.
  The encoder now doubles the line ending before such a block,
  in a block quote and a list item as at the top,
  unless a footnote definition holds the paragraph
  (`page-check/src/corpus-run/source-ships-html-interrupt.ts`).
  The second is read line by line from the indented code the plain reading sees under the definition
  (`page-check/src/corpus-run/source-ships-opener-footnote.ts`).
- N4,
  three mutants of `source-reading-blocks.ts` alive:
  pinned,
  each killed (see the mutants of the round).
- N5,
  `undecided` withholds a differing block that a range reaches:
  not narrowed,
  with the reason and the counts in the section on N5.
- F7,
  `source-reading-not-compared` unreachable from the composition:
  a page that parses and cannot be compared exists,
  and is pinned at both levels.
  The branch stays.
- F8 and the nits:
  the article openings of `source-reading-blocks.ts` are gone,
  as are two written in this round;
  the count of failing cases of three reds in the file of the fifth round is corrected there (N7);
  the sentence of the README and of the design commitments that offered the answer `same` for the block path says it
  is never `same` there (N8).
- N9,
  `blockDifferenceOf` grows faster than the square of the blocks:
  made quadratic with a small constant,
  with the measurement in the section on N9.
- N10,
  the offsets of a text with a byte order mark:
  a shape lands in the wrong answer,
  so it was red first and is fixed.

### What was red first, and what was not

Every red fails on its parent for the stated reason,
none is a load error,
and the failing lines of each are in `dx1f4/red-*-failing-lines.txt` of the run directory,
outside the repository:

- `aefbe09ec897dba5b15c7f5a34bfc8efffd932e0` (N1):
  6 of 6 cases.
  Four fail with the page missing the closing pipe of the row
  (the page shows the row without its closing pipe),
  two with an `MdxParseError` from the strict grammar.
  All six read the same on the code of `ea368918c3`,
  the parent of the join,
  so the regression is the join's.
- `a18a68334ef53cd2cb46d9cdef25a2f10e5bb161` (N2):
  12 of 14 cases fail with the page showing the mark that opened the construct.
  The control indented three spaces,
  which both readings cut,
  and the case of emphasis holding a code span,
  which the join already covered,
  pass.
- `5f7fd2fe1109b3ec5dd262af0e39faeae28a8467` (N2,
  the backslash hard break):
  5 of 7 cases fail with the page showing the backslash;
  the control indented three spaces and the control with a break of two spaces pass.
- `e49bcea1922994faca3aea40885e564ebe7f9766` (N3,
  the opener that cuts a paragraph):
  5 of 8 cases fail with the page showing a code span or math the source lacks;
  the controls (no backtick,
  an opener after other text on its line,
  an opener after a blank line) pass.
- `aed90bd742678da24a7f861f12510f82e70685b5` (N3,
  the opener in a footnote's fence):
  7 of 8 cases fail with the character reference shown inside the code;
  the control with the opener in a paragraph after a closed fence passes.
- `501f49f5c0fa027ff8a48a94dd850ef6aaea0609` (N10):
  1 case,
  `expected 'undecided' to equal 'differs'`,
  on the code of `75faaefa33`,
  whose `source-reading-difference.ts` is the parent's.
- `4103fc14f60eb817fb01a270f87ed1e5718f8d98` (the block of raw HTML that cuts a paragraph):
  11 of 15 cases;
  the four controls (no backtick,
  a blank line before the block,
  an inline element,
  a paragraph that a footnote definition holds) pass.
- The pins of this round are regression pins on code that already passed:
  `75faaefa3359805efa85dbcaa7acd85032e64a77` (N4,
  3 cases),
  `9a6c03da360a1deef3fa0a37e19662429ba94708` (F7,
  2 cases),
  and `8b04b6caed7673d6f9f95c626a9ed9ef2c6329e8` (3 texts).
  The three texts of the pin were red in an intermediate state of the fix of the block of raw HTML,
  which was never committed:
  the first form cut the paragraph on the page but left the block in the run of text of the paragraph it cuts for the
  reader of bare addresses.
  The search of the 132260 texts reported them (3 texts that read the same before),
  and the fix adds the block to the stream starts.
- The case of 800 blocks in `source-reading-blocks.unit.test.ts` (with the speed-up
  `86d98c82842baed0843ef9b189817f81081fce5b`) passes on both codes.
  The answer is the same;
  only the time differs,
  and a timing is no test.

### The mutants of the round

Mutants were applied one at a time to a scratch copy of the tree,
never to the worktree,
after the unmutated suites ran green.
Each line gives the commit the tree was made from.

On `75faaefa33` (N4),
all three die,
one case each:
the order rule of `placedAfter` replaced by `false`;
`Math.min` replaced by `Math.max`;
the skip of a block that shows nothing and no range reaches removed.

On `77273d9ca3` (N1,
N2,
N3 and N10):

- `startsOffTables` keeps every start (6 cases fail),
  and its table end made exclusive (the same 6).
- Emphasis not joined (6 cases of the suite fail);
  strong not joined (1 case);
  the references not joined (2);
  the text of a link not joined (3);
  the stretch of a link or image kept to its destination (3).
- The backslash break never joined (5 cases);
  its stretch ending at the start instead of past it (4).
- The blank line before the opener removed (5 cases);
  the footnote flows not read (7);
  the closing line of a flow never recognised (1);
  a flow left open ignored (1);
  the tab indentation not read (1);
  the footnote range requirement inverted (7).
- The ranges of a text that opens with a byte order mark read from the text with the mark dropped (1 case).

On `8b04b6caed` (the block of raw HTML),
the blank line removed (11 cases fail),
the containers cut down to the top (2),
and the block not added to the stream starts (3) die.

Survivors,
judged equivalent for the reader of the page,
whose account is the text of the leaves with white space folded,
so a blank line or a line ending written another way shows nothing:
the footnote guard of the opener and of the block of raw HTML turned off
(no text tried reads differently without it;
it keeps the arrangement of the page for a footnote that goes on over a line of raw HTML,
which the two-readings ruling takes);
a line ending of a text with Windows endings written as a line feed;
the blank line written without the marks of the container (the quote ends and starts again);
the blank line written also after a block that is no paragraph or after a blank line;
and joining a brace line inside emphasis,
which a backslash escapes without a join.
No survivor changes what the page shows.

### The search on the final code

The search of the fifth round,
132260 texts (every run of both seeds' 40000 pairs,
the footnote product texts and the shape grid),
ran in a unit capped at 8G and 800 percent of CPU on the final code:

- same (oracle equal,
  comparison same),
  reported 0: 117000.
- differs:
  8838,
  of which 54 also hold an undecided block.
- undecided:
  1119,
  each shipping `source-reading-undecided`.
- could not compare (page nested past the bound,
  comparison throws,
  composition ships `dropped`): 5303.
- Row for row,
  text,
  class and undecided count,
  equal to the search of the fifth round;
  0 texts that read the same before are reported now.

By answer,
with the false positives:

- same:
  117000,
  and no text the oracle finds unequal is called the same (0 false negatives).
- differs:
  8838,
  all of which the oracle finds unequal,
  so 0 are reported for a page that reads the same.
  8678 hold a block that differs when asked alone;
  160 hold none,
  and are the same texts as in the fifth round,
  whose hand reading stands,
  and which this round did not re-read.
- undecided:
  1119,
  all of which the oracle finds unequal.
  7 hold a block that differs alone inside a reached range (see the section on N5),
  and 0 hold one outside every range.
- could not compare:
  5303,
  all of them pages that refuse to parse (a nest past the bound),
  which ship without their original and with the defect `page-source-unparseable`.

The pool holds none of the shapes of N1,
N2 and the openers (the re-check said so of N1),
so three differentials of generated texts stand beside it,
each run as the code of `242d4d14fd` (the head the re-check read) and the final code,
with `ea368918c3` for the first two:

- 6000 texts built around indented footnote lines,
  spans,
  tables,
  fences,
  quotes and lists (seed 99,
  the re-check's generator):
  against `ea368918c3`,
  same to throw 0 and same to differs 0 (19 and 2 at `242d4d14fd`);
  the answers for the final code are 5858 same,
  104 undecided,
  38 differs.
- 30000 texts with the leads widened to emphasis,
  links,
  images,
  tables and references (seed 11):
  against `ea368918c3`,
  same to differs 0,
  same to throw 0;
  against `242d4d14fd`,
  53 throws to same,
  343 differs to same,
  1093 undecided to same,
  and nothing that read the same before is reported.
- 30000 texts of a paragraph,
  a block of raw HTML and a tail (seed 5):
  against `242d4d14fd`,
  same to differs 0,
  same to throw 0,
  differs to same 1229,
  undecided to same 30.

### The real corpus

Pinned clone `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`,
read with `git cat-file` only,
in a capped unit:
276 files,
274 distinct texts,
8423 top-level blocks of which 7360 are distinct.
61 blocks ship as literal text and all 61 read the same.
7299 ship as recorded and are not compared;
1 of them would read differently encoded (the front matter,
which the fourth and fifth rounds name).
No file holds a comment that never closes,
an image whose alternative text is an angle autolink,
a footnote that opens with a tag alone,
a footnote start indented four columns,
or a fence mark indented four columns,
and no file holds a table row followed by a footnote start indented four columns.
Nine files hold a comment opener first on a line after a line of text;
every one of those comments closes,
so none is the class.
No footnote range the two readings disagree on starts inside a quote or a list item,
in any file or block.

### N5, what is not narrowed

Over the 1119 undecided texts of the search,
asking every block of the source alone:
0 hold a block that differs alone outside every range that reaches blocks,
and 7 hold one inside such a range.
The first count is the one the brief asks for,
and it is zero:
a block that differs outside every reached range is reported,
because the blocks no range reaches are anchors,
and an anchor that the page does not show where it comes is a difference.
The 7 are texts whose quote holds a footnote that the page ends further down than the source does.
Four of them show a code span that the page cuts at the start of the footnote inside the quote,
which the source reads across it.
The block that holds the start differs alone,
but the difference exists only through the footnote:
with the caret taken out,
all four read the same.
The two-readings ruling takes the page's reading of a footnote inside a quote,
so that difference is the ruling's own choice and stays undecided.
The other three show an image whose alternative text is an angle autolink,
a known class that differs with the caret taken out too;
two stand in the tail the page's footnote runs over and one inside the quote's own block.
Such a block is reached,
and the comparison cannot ask its words apart from the footnote that holds them on the page:
asking it alone differs,
but a difference alone is no proof of a difference in its place,
and the encoder writes the block in the nested reading of the footnote in context.
The run ships `source-reading-undecided` for all seven,
so no page is silent.
No such text is in the pinned corpus.

### F7, which branch

A page that parses and cannot be compared exists.
Both readings decode the target of a link with `decodeURI` (`leavesOf` in `source-reading-account.ts`),
which throws a `URIError` for a percent escape that is no text in UTF-8,
such as `%E0%A4`,
while the strict grammar parses the page.
A run holding `[t](%E0%A4)` and a bare brace is written as literal text,
the page parses,
and the comparison throws:
the composition ships the step `encoded-failing-runs` with the one defect `source-reading-not-compared`
(`9a6c03da360a1deef3fa0a37e19662429ba94708`,
at the level of `differenceDefectsOf` and of the composition).
The branch is therefore kept.
The commit message of the fifth round and its ledger named such a page while the one test drove a page that does not
parse (300 nested quotes);
this round supplies the page.
The targets are dropped before any comparison (`withoutLinkTargets`),
so a tolerant decode would make the branch unreachable again without changing a single answer;
that is the owner's call and is not made here.

### N9, the work of the block comparison

The shape of the re-check,
every block the same two words and every fourth block reached,
one CPU:

- Before:
  6 ms at 50 blocks,
  8 ms at 100,
  25 ms at 200,
  391 ms at 400,
  4219 ms at 800,
  78591 ms at 1600.
- After (`86d98c82842baed0843ef9b189817f81081fce5b`):
  2 ms,
  2 ms,
  5 ms,
  14 ms,
  56 ms,
  242 ms.

The cost of a way to a place is the cost of the place the blocks before ended at plus the blocks a range reaches,
unless the words between are what those blocks show;
only the place exactly that many words back can avoid that,
so the cheapest way to a place is read off the places in order.
The work is the places the words stand at and the places the blocks before can end at,
once for each block,
which grows with the square of the blocks and a small constant.
223000 random configurations of blocks,
ranges and page blocks (words from a vocabulary of three,
so words repeat) answer identically on the old and the new code.

### The residual classes

What reads differently from its source after this round,
reported by the comparison and not fixed:

- An image whose alternative text is an angle autolink (known,
  `audit-ledger-source-text-follow-ups-parser-gaps-and-step-searches.md`).
- The brace and tag joins inside a table row,
  and any other construct the plain reading pairs across rows
  (a row holding a backtick that nothing closes,
  then a line that starts with a brace):
  the plain reading has no GFM tables,
  so a span is paired across rows that the strict grammar cuts.
  The proper fix reads the spans with the grammar the page uses,
  tables included,
  which is a change to `parse-markdown-math.ts` that every consumer of the plain parse would feel.
  A block of raw HTML line after a table row with a backtick in the row is the same cause.
- A footnote start indented four columns right after a link reference definition (`[r]: u` then `    [^1]: y`):
  the source reads indented code,
  the page a footnote.
  88 of the 30000 widened texts.
  No file of the pinned corpus holds an indented footnote start.
- A comment that never closes in a code span that only the reading of a footnote in a block quote pairs (4 texts of the
  search,
  from the fifth round),
  and one in a block quote or a list item at the start of a line,
  which is text to both readers.
- The container footnotes the two-readings ruling decides (278 of the 30000 widened texts,
  1119 of the search),
  each shipping `source-reading-undecided`.

### The full task, the merge and the gates

The full task,
`mise run //package/translation-repair/page-check:test:live-full`,
ran on the merge commit `08789214b1885c50d4ec060abdb120bac84f21cd` in the unit `dx1f4-livefull`
(8G,
800 percent of CPU,
`--wait`),
with its output in fresh files:
success,
case 696.2 s,
service runtime 11 min 38.120 s,
CPU time 13 min 34.234 s,
memory peak 749.1M,
swap 0B.

`origin/main` moved three times during the round
(`1362f15c40c72d721e13a6beab11c12e334e03e7`,
then `495bf2f39e5b7ebbeacec33b5cdfb239a2649c5a`,
then `cff94d8247df758de839d7dc839e68dbd5df18bf`),
and was merged at the second and at the third,
each by a merge that `git merge-tree` showed clean.
The second merge changes no file of `page-check`,
`document-structure`,
`markup` or `run`,
and the third changes one handover note.
Main's own commits in the merged range bump cache versions of other stages;
this round's commits (`242d4d14fd..af5e52e29a`) name none.

Each gate ran by itself from the worktree on the final code:
page-check `build`,
`lint:types`,
`lint:oxlint` (0 warnings,
0 errors),
and `test:unit` (195 PASS,
0 FAIL);
source-scan `test:unit` (68 PASS) and `source-scans` (53 PASS).
The packages `document-structure` and `markup` are not touched by this round.

### Cache versions

`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+'` over `242d4d14fd..af5e52e29a`,
the commits of this round before the merge,
and over `56514f37f1..af5e52e29a` names no commit;
the control over `c265e8df95~1..c265e8df95` names `c265e8df95`.
The round changes `page-check` only,
which no key module of the cached stages imports.
No version is bumped.
