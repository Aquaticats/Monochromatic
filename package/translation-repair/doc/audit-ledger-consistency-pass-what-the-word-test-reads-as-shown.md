# Audit ledger: the consistency pass, what the word test reads as shown

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

The word test at a span edge decides whether a quote ends inside a word that a reader sees.
It reads the shown text:
what the page shows after the parse,
with the units that show nothing stepped over.
This file holds the findings of the review of n159 about what that reading stepped over wrongly,
or failed to step over:
a hidden stretch whose units were read one by one (B616),
markup that separates or annotates and was read as invisible (B617),
and inline code beside a text place (B618).
It also holds the rows of the twin check that the first fixes left (B619).
The first three are fixed by n161 and landed on 2026-10-15 with the tests `5fe3173cc5` and the code `277145a862`.
B619 is fixed by n166,
n168,
n169 and n173,
which landed on 2026-10-18:
n166,
n168 and n169 with n171 in the tests `555bb74aa0` and the code `29d6064708`,
and n173 in the tests `ff6be38eef` and the code `43428948cb`,
with the repair `f9169093e6` of that merge.
The same review's residue of padded inline code and raw elements is B633 and B634,
and the defect that n168 found in its own first version is B638.

The principle itself,
and the rounds that built it,
are filed in
[Audit ledger: the consistency pass, span edges as whole units](audit-ledger-consistency-pass-span-edges-as-whole-units.md)
(B597 and B598)
and in
[Audit ledger: the consistency pass, elisions, quoted numbers and pairing](audit-ledger-consistency-pass-elisions-quoted-numbers-and-pairing.md)
(B605).
The other findings of the same review are filed in
[Audit ledger: the consistency pass, definitions inside containers, fragment landings and the mask memory](audit-ledger-consistency-pass-definitions-landings-and-the-mask-memory.md)
(B620 to B622),
in
[Audit ledger: files a later run reads back](audit-ledger-files-a-later-run-reads-back.md)
(B624),
in
[Audit ledger: comments, TSDoc and source text](audit-ledger-comments-tsdoc-and-source-text.md)
(B625)
and in
[Audit ledger: test cases that cannot fail](audit-ledger-test-cases-that-cannot-fail.md)
(B626).

Written on 2026-10-15 (UTC) from the review of n159 (`s231/review.md`),
the hand-backs of n159 (`s230/handback-n159.md`) and n161 (`s232/handback-n161.md`)
and the lead's notes of that day.
The update of B619 on the same day is written from the review of n162 (`s233/review.md`),
the hand-backs of n166 (`s237/handback-n166.md`) and n168 (`s239/handback-n168.md`)
and the lead's notes of the rulings.
B633,
B634 and B638 were written on 2026-10-18 from the review of n162 (`s233/review.md`),
the hand-backs of n166 and n168,
and the commits that landed the rounds.
The landing of B619 is written from the commit messages,
the hand-back of n169 (`s240/handback-n169.md`)
and the probe outputs of n173 (`s244/`),
since n173 returned no hand-back file.
The labels Y1,
Y2 and Y4 are the review's own and are no ledger ids.
The labels of probe scenarios,
the names of probes and the names of logs are the sources' own too.
A path that starts with `src/` is under `package/module/translation-repair/` (moved on 2026-10-18 to the packages under
`package/translation-repair`).
Every function,
type and constant of the package named here was found declared in non-test source of the tree `w153`
at `277145a862`
by one search of declarations on the day of writing.
The names that only a review or a hand-back gives
(test names,
guard names,
the names of probes and the names of logs)
were not searched.
The functions that the update of B619 names (`apostropheJoinsAt`,
`closingJointsOf`,
`cutsShownWord`,
`locateWords` and `parseSliceBody`)
were found declared in non-test source of the tree `w163` at `e50786900d`,
which does not hold the work of n166 and n168.
No line numbers are cited,
since the tree is still changing.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B616: a reference, an escape or an astral character inside a hidden stretch was read as shown

Found on 2026-10-15 by the review of n159
(n160,
finding Y1).

Outcome written on 2026-10-15.
Fixed by n161;
red `5fe3173cc5`,
fixed in `277145a862`.

What a reader of the page would have seen:
a half word in one group of inputs,
and a repeated phrase left in its old wording in the other.
An emoji inside a hidden title or comment was kept as a shown character that is no word character,
so no cut was seen and the splice ran:
`The [kit](u "🐱")tens ate` and `The kit<!-- 🐱 -->tens ate` with `kit` shipped
`The [dogs](u "🐱")tens ate.` and `The dogs<!-- 🐱 -->tens ate.` with the outcome `replaced`,
which reads `dogstens`.
A reference or an astral letter inside a hidden destination,
title or attribute was read as shown,
so a whole word beside it was refused:
`foo [kit](u "a&#116;b") ate`,
`foo <a href="x&#116;y">kit</a> ate`,
`The [kit](u "𝓠") ate`,
`The [kit](https://x.example/𝓠𝓾𝓪𝓷) ate` and `The <a title="𝓠">kit</a> ate`,
each with `kit`,
gave `none` where the old build gave `once`,
and at page level `[kit](https://x.example/𝓠)` stayed `kit` with the place `unlocated`.
The ASCII controls (`[kit](u "x")tens` and `[kit](https://x.example/Q)`) behaved as the ruling says.
The twin check,
which compares the text with the invisible delimiters against the same text without them,
found 864 to 862 sequences per group (a reference in a destination,
a title or a tag attribute) that decide `once` in the twin and `none` with the hidden part.

The cause:
`cutsShownWord` in `src/consistency-shown-edge.ts` kept a unit unless it was one unit wide and hidden at its start,
so a reference or an escape (several units)
and an astral character (two units) inside a hidden stretch survived the filter.
`transparentUnitsOf` in `src/consistency-unread-units.ts` already flagged every unit of those stretches.
The header of `src/consistency-unread-units.ts` said that a character reference "is not here",
which only the filter could make true (B625).

Reach on the corpus:
none by the review.
It found no reference inside a destination,
a title or a tag,
and no astral character inside a destination,
a tag or a comment
(astral characters stand on 4 pages,
none in those places),
and each search was run against a control page holding a known hit for the pattern.
n161's corpus measure moved no decision on any of the 92 pages.

The lead ruled on 2026-10-15:
fix it,
and filter by the flag of the first unit for every unit of a hidden stretch.

Fixed by n161:
a unit is hidden by the kind of its first character,
whatever its width,
so a reference,
an escape or an astral character inside a link destination,
a title,
a tag attribute or a comment is hidden whole.
Tests:
`src/consistency-shown-edge-markup.unit.test.ts` (new,
with rows for B616,
B617 and B618);
the rows of this entry were 7 red on the build made before any source edit.
The guard that restores the width-one rule fails 9 rows.
Rerun of the review's probes on the final build:
`foo [kit](u "a&#116;b") ate` and `foo <a href="x&#116;y">kit</a> ate` with `kit` went from `none` to `once`,
the page-level row for a math letter in a destination went from `unlocated` to `replaced`,
and the rows for an emoji in a title or in a comment with half a word to the right went from `replaced` to `unlocated`,
as the ASCII rows already were.

## B617: tags and markup that separate or annotate were read as invisible, so words across them were one word

Found on 2026-10-15 by the review of n159
(n160,
finding Y2).

Outcome written on 2026-10-15.
Fixed by n161;
red `5fe3173cc5`,
fixed in `277145a862`.

The review framed it as a gap in the ruling and not as a slip of the author:
the ruling of B597 lists tags among the transparent units,
so stepping over a tag is what the ruling says,
and it did not tell a tag that wraps text from a tag that separates blocks or annotates text.
The harm was a repeated phrase left uncarried at those places,
never a half word.

What a reader of the page would have seen:
separate lines,
cells,
blocks,
or a ruby base with its annotation,
where the word test read one word.
`cats<br/>dogs` with `dogs`,
`cats<br />dogs` with `cats`,
`cats![pic](u)dogs`,
`<ruby>kan<rt>ji</rt></ruby>`,
`| cats | dogs |` (the cell pipes and the padding are hidden units),
`<p>The kit</p><p>tens ate</p>`,
`<ul><li>cats</li><li>dogs</li></ul>`,
a table of `<td>` cells and `<details><summary>The kit</summary>tens ate</details>`
gave `none` where the old build located the word.
`cats<br>dogs` with no slash still located,
so one line break was read two ways.
At page level `The dogs bowl.<br/>kit<br/>A cat naps.` left `kit` (`unlocated`),
and so did a table cell `kit`.

The cause:
`transparentUnitsOf` in `src/consistency-unread-units.ts` set the hidden flag
for every unit outside a text node except line breaks,
which took in the cell pipes,
the padding inside a table row,
every tag and the `![](u)` of an image,
and `cutsShownWord` then stepped over all of them.

Reach on the corpus:
one page.
The ruby annotation `<ruby>reload a save file<rt>OVERLOAD</rt></ruby>` on `windward0032`,
slice 24,
put `file` and `OVERLOAD` among the 23 word-test decisions that n159 changed (B597),
though the author's account called all 23 "each now cutting like its plain-text equivalent".
Every other tag of the corpus
(`p`,
`br`,
`details`,
`summary`,
`h3`,
`div`,
`blockquote` and components)
never has a letter or a digit on both sides;
the only run of tags with a word character on both sides in 92 pages is the `<rt>` one.
The corpus holds no table and no image,
and `<br/>` touches a letter on one side twice on `XingZ60` and never on both.

The lead ruled on 2026-10-15:
fix it.
Separating and annotating markup is a word boundary and not transparent:
`<br/>` with or without the slash alike,
block tags,
table pipes and padding,
images and `<rt>`.
The ruling changes 2 decisions on `windward0032`.

Fixed by n161:
- `src/consistency-shown-kinds.ts` (new) holds `unitKindsOf`,
  `UnitKind` and `BOUNDARY_TAG_NAMES`.
  A break with or without a slash,
  the CommonMark block tag names,
  `hr`,
  `img`,
  `rt`,
  `rp`,
  images and table rows (pipes and padding) read as a space.
  Inline wrappers (`b`,
  `i`,
  `em`,
  `strong`,
  `span`,
  `a`,
  `u`,
  `s`,
  `sub`,
  `sup`,
  `ruby`,
  `mark`,
  `small` and `code`) stay transparent.
- `src/consistency-markup-ranges.ts` (new) holds `markupRangesOf` and `NO_MARKUP_RANGES`.
  `ProseView` in `src/consistency-prose-text.ts` gained `images`,
  `tableRows` and `codeContents`,
  and `src/consistency-reading-barrel.ts` exports `BOUNDARY_TAG_NAMES` and `unitKindsOf`.
- The block tag list is typed by hand from the list in `micromark-util-html-tag-name@2.0.1` (plus `br`,
  `hr`,
  `img`,
  `rt` and `rp`),
  because that package is no declared dependency of this one and adding a dependency was outside the brief.
  `src/consistency-shown-kinds.unit.test.ts` reads the installed parser's list through the `remark-parse` chain
  and fails when a name it holds is missing here.
  The lead accepted the hand-typed list under that test.
- Tests:
  the rows of this entry in `src/consistency-shown-edge-markup.unit.test.ts`
  (21 red on the build made before any source edit),
  `src/consistency-shown-kinds.unit.test.ts` (3 red by missing exports),
  and `src/consistency-markup-ranges.unit.test.ts`.
  The guards that stop a tag,
  an image or a table row from being a boundary fail 19 and 4 rows (tags),
  4 and 3 (images),
  and 4 and 3 (table rows).
  The first guard written for this finding stopped reading a boundary unit as a space and changed nothing a reader sees,
  because a boundary's own characters (`<`,
  `b`,
  `r`) are no word characters either;
  it was replaced by these three before it counted.

Corpus effect:
the two `windward0032` slice 24 decisions are restored,
so the word-test changes of n159 go from 23 on 9 pages to 21 on 8 pages and nothing else moves.
A direct check of the slice (379 units) gives `once[191,195)` for `file` and `once[199,207)` for `OVERLOAD`,
as the build before n159 does.
The twin check's `br` group went from 442 to 2,542 mismatches,
the new 2,100 all `none` to `once`
(`kitkitkit<br/>kit` is now `once` where the text without the delimiter says `none`).
That rise is the change by design:
the twin's definition,
delete the invisible delimiter,
no longer describes a break.

## B618: inline code beside a text place still shipped a half word, and the headers said a floor refused it

Found on 2026-10-15 by the review of n159
(n160,
finding Y4).

Outcome written on 2026-10-15.
Fixed by n161;
red `5fe3173cc5`,
fixed in `277145a862`.

What a reader of the page would have seen:
a code word fused with a text word.
`` The `kit`tens ate. `` with the place `tens` shipped `` The `kit`dogs ate. ``,
and `` The kit`tens` ate. `` with the place `kit` shipped `` The dogs`tens` ate. ``,
both with the outcome `replaced`,
which reads `kitdogs` and `dogstens`.
The build before n159 did the same,
so the shape predates the round.
The author's account and the header of `src/consistency-unread-units.ts`
said that the inline-code floor refuses this by its own rule.
The floor refuses only a place inside the code,
and not a text place beside it.

The cause:
`transparentUnitsOf` in `src/consistency-unread-units.ts` hid the code content with its backticks,
so `src/consistency-shown-edge.ts` saw nothing of the code word.
The typography side reads the code content as shown (`src/typography-shown-view.ts`),
so the two readers disagreed about one unit.

Reach on the corpus:
none.
Backticks occur on 3 pages and no letter or digit touches a code span
(the search found 0 lines,
and its control page gave 2).

The lead ruled on 2026-10-15:
fix it.
A text place beside inline code reads the code word as shown,
in agreement with `src/typography-shown-view.ts`.

Fixed by n161:
`transparentUnitsOf` now shows the code contents of the view,
and its header is rewritten (B625).
The rows of this entry in `src/consistency-shown-edge-markup.unit.test.ts`
were 2 red on the build made before any source edit,
and the guard that removes the code contents fails 4 rows.
Rows 7 and 8 of the `p1-edges` probe went from `once` to `none`,
and the two page-level rows (code before the word and code after it) went from `replaced` to `unlocated`,
with the page unchanged there.

## B619: a hidden reference then an apostrophe before the word decided differently from the plain text

Found on 2026-10-15 by the rerun of the twin check in n161,
after the fixes of B616 to B618.

Fixed by n166 and n168 on 2026-10-15,
finished by n169 and n173,
and landed on 2026-10-18 (see "Landing" at the end of this entry).
The review of n162 reproduced the rows and measured their reach,
and the lead ruled on the work of n166 and n168 the same day.
The entry first recorded the rows as open,
reported by the author and not fixed.

The twin check compares each input with the same input without its invisible delimiters,
and reports where the two decide differently.
After B616 the groups for a reference inside a destination and inside a title went from 982 to 144 mismatches,
and the group for a reference inside a tag attribute from 984 to 122.
In each of the two groups,
`link-ref-in-dest` and `link-ref-in-title`,
26 rows of the kind `once` to `none` remain:
the plain-text twin locates the word once,
and the text with the hidden part refuses it.
All 26 have the shape `X[](u&#116;)'kit`,
a hidden reference and then an apostrophe before the word.
They stood inside the 982 before the round,
and the author could not explain them by the changes of the round.

What the author could not say was whether the refusal was right,
since an apostrophe before the word may be a mark the word test reads,
or a wrong refusal where the twin is right.

The cause,
as the review of n162 traced it and n166 corrected it:
the review reran the `p5-wordtwin` probe and counted the same rows
(`link-ref-in-dest` and `link-ref-in-title` each 26 of `once` to `none`,
of 3,782 comparable rows,
and 118 of `none` to `many`).
The word `kit` after `[](u)'` and `[](u&#116;)'`,
with or without the reference,
was refused by `locateWords`,
while its twin with a space in place of the link was located once.
The review found `cutsShownWord` false on those rows,
so the refusal was earlier,
and it named the branch of `closingJointsOf` in `src/consistency-closing-marks.ts`
that continues the run past a destination as the place where the apostrophe became a joint.
It said that it had read that branch and had not instrumented it.
The brief of n166 carried that pointer.
n166 instrumented a copy before it fixed anything:
refusing an empty label in that branch changed nothing.
The cause was the last return of `apostropheJoinsAt` in `src/consistency-word-joints.ts`,
which took any closing bracket before an apostrophe as a joint
without asking which bracket it closed.

Reach on the corpus:
none.
The review of n162 searched the 92 pages for `\[\]\(` and for `\[\]\([^)]*\)['’]` and found no match in either,
and the control `\]\(` found links on many pages.
The inputs are constructed.
The corpus measures of n166 and n168 found no decision or character change on any page.

The lead's rulings of 2026-10-15:
- The fix of n166 cleared the 26 rows of each group that went from `once` to `none`,
  and moved two rows of each group from `none` to `many` (118 to 120):
  `kit[](u)'kit` and its twin with the reference.
  The lead ruled that wrong,
  since a reader sees `kit'kit`:
  the joint is decided from the character before the empty link,
  as with no link there (n168).
- The documented limit of n166's helper,
  that it did not read a round bracket inside a destination or a title,
  nor a square bracket inside a label,
  was ruled a mistake and not a limit:
  n168 reads the link by the parser's rules (micromark,
  through `parseSliceBody`),
  and the backward scanner is gone.
- The lead accepted three decisions of n168:
  the plain-text twin is the oracle,
  a label of marks only (`[ ](u)`,
  `[-](u)`,
  `[_](u)`) gives no joint,
  except that one whose last shown character is a closing round or square bracket
  or a closing curly double quotation mark gives the joint,
  as the plain text does,
  and an English that the page grammar refuses has no parse,
  so no link is read in it and an apostrophe after `)` stays a joint.

The state of the fixes:
n166 left `once` to `none` at 0 in both groups,
and its suite was green at 2,445 passes and 0 failures on its second whole run
(the first was red on seven tests that pinned cache keys and seat order the sheet text of another finding moved).
n168 left `kit[](u)'kit` and `kit[](u&#116;)'kit` answering `none` like `kit'kit`,
`once` to `none` at 0 and `none` to `many` back at 118 in each group,
and its suite was green at 2,446 passes and 0 failures,
with lint,
types and the source scans clean,
the replays as expected
and no change on any page of the corpus.
Four divergences from the twin that n168 measured and left to other layers were ruled to be fixed in n169:
the forward reading over empty links,
the joints of closing marks across links,
images with an empty alt text in the shown layer,
and a line break inside a destination in the shown layer.
The same round took the page definitions for a reference with an empty label,
and the question whether the consistency cache can hold values derived from the fold.
n169 handed back green on 2026-10-15
(suite 2,450 pass and 0 fail,
lint,
types and the source scans clean,
the replays as expected,
no real change on any page of the corpus,
every new guard shown to fail),
and found that no cache version needs to move,
since the sheet text is in the consistency key and the pipeline digest covers the build
(`s240/handback-n169.md`).
It left links with a label that is not empty,
which still read apart from their twin,
and four callers that fold without the page's definitions;
the lead ruled both fixed,
in round n173.

Round n173 (a copy of the tree after n169;
no hand-back file is among the sources):
- A link or reference whose label shows letters,
  digits or marks shows a reader its label,
  and its opening bracket and its resource show nothing,
  so `kit[x](u)'kit` reads as `kitx'kit`.
  The word joints step over those two stretches as they step over a link that shows nothing,
  and the parse lists them.
  The rerun of the probe `p6` after the round printed `rows differing from the twin: 0`
  (`s244/p6-a.out`),
  where n169 had left 90 rows per group.
  The tests are `src/consistency-labelled-links.unit.test.ts` (new in `ff6be38eef`)
  and rows in the tests of the folded links,
  the markup ranges and the word joints.
- A reference defined on the page can change the answer of `emphasisDelimitersOf`
  (the probe rows of `s244/item2-a.out`;
  the corpus measure's control `[*a][r] kit*-kit` moves from `none` to `many`),
  so the definitions are passed there too.
  The other callers are B647.
- The corpus measure over the 92 pages (1,237 slices,
  21 pages with definitions,
  55,605 `locateWords` comparisons,
  24,017 of them with the page)
  shows no real change:
  its output holds only control rows,
  and every control moved (`s244/measure-n173.out`).
- The message of `43428948cb` says that n173 demonstrated its guards in its own worktree.

Landing on 2026-10-18.
- n166,
  n168,
  n169 and n171 landed together in the tests `555bb74aa0` and the code `29d6064708`.
- n173 landed in the tests `ff6be38eef` and the code `43428948cb`,
  merged onto n171.
  The merge kept two declarations of the folded links in `apartStartsOf`,
  so `43428948cb` did not build,
  and `f9169093e6` keeps one (M238).
- The gate then failed one scan on a test of n171 (B646),
  and `94ebefa1e3` fixed it:
  the suite passed 2,482 cases,
  lint 0 warnings and 0 errors,
  and the type check was clean.

The entry is fixed.

## B633: inline code padded by a space still read as two words

Found on 2026-10-15 by the review of n162 (`s233/review.md`,
finding Z6,
the residue of the review's label Y4).

Outcome written on 2026-10-18.
Red `555bb74aa0`,
fixed in `29d6064708`;
the fix is the round n166.

What a reader of the page would have seen:
the input is ``The ` kit `tens ate.`` with the place `tens`.
`locateWords` answered `once` at the offsets 11 to 15.
CommonMark strips one padding space from each side of a code span,
so a reader sees `kittens`,
and the code content was read with its padding as shown text,
so a space stood between `kit` and `tens`.
The repeated phrase `tens` would have been carried into a word that reads `kittens`:
a half word ships.
The rows with double backticks (`` ``kit``tens `` and `` kit``tens ``) already answered `none`.

Reach:
none on the corpus.
The census of the 92 pages holds five single-backtick spans on three pages and none is padded
(the control is the same run counting those five),
and the probe for code beside `*` or `_` printed nothing.
The shape is constructed.
The header of `src/consistency-unread-units.ts` said
a word of code beside a word of text reads as the one word it shows,
which this shape broke.

Fixed by n166.
`codeContentOf` in `src/consistency-markup-ranges.ts` strips one padding space on each side
when both ends are a space (U+0020) and the content holds a character that is no space,
which is the CommonMark rule;
a line ending is not read as a space.
The header of `src/consistency-unread-units.ts` now says the typography side keeps the padding as shown.

Test:
`PADDED_CODE_ROWS` in `src/consistency-shown-edge-markup.unit.test.ts`
and one `locateWords` row in `src/consistency-fold.unit.test.ts`.
Four padded rows were red against the build the round started from
(two with single backticks and two with double),
and the same rows and the fold row are red again with the strip removed (`s237/mut-b.log`).
The corpus measure over the 92 pages showed no decision or character change,
and its control for a padded code word moved from `false` to `true`.

Recurrence:
`mistake-prevention-text-comparison-and-structure.md`,
"Text that shows nothing".

Revised on 2026-10-19 (UTC):
the clause "a line ending is not read as a space" was wrong for CommonMark,
which turns each line ending in inline code into a space before it strips the padding.
B677 holds the correction,
and the recount of an artifact written before it reads the padding by the generation the artifact records (B682).

## B634: raw HTML elements did not end a word

Found on 2026-10-15 by the review of n162 (`s233/review.md`,
finding Z7).

Outcome written on 2026-10-18.
Red `555bb74aa0`,
fixed in `29d6064708`;
the fix is the round n166.

What a reader of the page would have seen:
`cats<pre>dogs</pre>` and `cats<textarea></textarea>dogs`,
with the place `cats`,
answered `none`,
where the tree before n159 answered `once` at the offsets 0 to 4.
The tag was read as a wrapper,
so `cats` and `dogs` read as one word.
`BOUNDARY_TAG_NAMES` in `src/consistency-shown-kinds.ts` held the parser's block names only,
and `pre`,
`textarea`,
`script` and `style` are the CommonMark openers of a raw block.
The test of the list read `htmlBlockNames` only,
so it could not see them.
A phrase beside such an element is left uncarried,
the safe direction,
and never cut.

Not findings:
the elements that the parser lists in neither place
(`button`,
`video`,
`audio`,
`svg`,
`canvas`,
`object`,
`select`,
`hgroup`)
stay transparent by the lead's ruling "and the like",
and the review reported them as what n161 decided.

Reach:
none on the corpus.
The tag census holds `br`,
`div`,
`details`,
`summary`,
`blockquote`,
`span`,
`p`,
`h3`,
`ruby`,
`rt` and custom components,
and none is `pre` or `textarea`.

Fixed by n166.
`pre`,
`script`,
`style` and `textarea` are in `BOUNDARY_TAG_NAMES`,
and the list test reads both parser lists,
`htmlBlockNames` (62 names) and `htmlRawNames`
of `micromark-util-html-tag-name` 2.0.1.

Test:
`RAW_ELEMENT_ROWS` (10 rows) in `src/consistency-shown-edge-markup.unit.test.ts`,
the list test in `src/consistency-shown-kinds.unit.test.ts`
and a row in `src/consistency-fold.unit.test.ts`,
red against the build the round started from
(`s237/red-z6z7-kinds.log`)
and red again with four raw names removed from the list (`s237/mut-b.log`).
The probe of the shapes (32 new rows,
the four names times eight tag shapes) answered as expected.

Recurrence:
`mistake-prevention-text-comparison-and-structure.md`,
"Structure read off the parse".

## B638: the first reading of a link's label took its nodes in reverse

Found on 2026-10-15 by the author of n168 at its own review,
before the round landed,
in the code the round was writing for B619.

Outcome written on 2026-10-18.
Never in a commit:
the landed code (`29d6064708`) has the fix.

What a reader of a run would have seen,
had it shipped:
a link whose label holds several nodes was judged by its first shown character and not its last,
so a label of marks that ends in a closing bracket,
written with a code span or emphasis among its nodes,
answered `once` where its plain-text twin answers `none`.
The probe rows showed it against the build that had it
(`s239/probes/p4-label-order.before.out`:
all six rows differed there).

The cause:
the first version of `linkLabelKindOf` filled its work stack in child order and popped it,
which reads the children last to first.

Fixed by n168.
The pushes in `labelTextOf` in `src/consistency-link-label.ts` are reversed,
and every check was rerun on the fixed source.
The round's own account lists this as no rule breach (M233).

Test:
"READS THE NODES OF A LABEL IN THE ORDER A READER SEES THEM" in `src/consistency-link-label.unit.test.ts`,
the marks rows of `src/consistency-markup-ranges.unit.test.ts`
and the controls of `src/consistency-fold.unit.test.ts`.
They were red against the build with the defect (7 FAIL lines in 3 files,
`s239/red-order.log`),
and red again with the two reversed pushes put back (`s239/mut-j.log`).
The probe after the fix answered `SAME` on all six rows.

Recurrence:
`mistake-prevention-text-comparison-and-structure.md`,
"Structure read off the parse".

## B677: inline code padded by line endings was read as unpadded

Found on 2026-10-18 (UTC) by the sol review of the production diff of five landed rounds
(`s245/sol-review/answer-gpt61.md`,
finding 5),
and reproduced before the fix.
It reopens B633,
whose fix wrote that a line ending is not read as a space.

Outcome written on 2026-10-19 (UTC).
Red `4aafb65895`,
fixed in `2cfefcd2e1`,
with a layout follow-up `840b65d249`.

What a reader of a page would have seen:
CommonMark turns each line ending in inline code into a space,
then strips one space from each side when the content begins and ends with a space and is not all spaces.
`codeContentOf` (`consistency/src/consistency-markup-ranges.ts`) stripped literal spaces only.
A code span of `tens` between two line endings,
written right after `kit`,
kept both line endings as separators,
and `kit` stood as a whole word although the page shows `kittens`.
On real pages this reaches only a word directly before or after inline code padded by line endings.

Fixed:
`codeContentOf` reads a line feed,
a carriage return or the two together as a space.
`transparentUnitsOf` (`consistency-unread-units.ts`) hides the line ending directly outside a content stretch.
Without that,
the stripped line ending stayed a shown separator,
and the stand of `kit` before that code span would still have been found.

Test:
a case in `consistency-fold.unit.test.ts`.
`locateWords` finds no `kit` in the text with the code span between two line feeds,
between two carriage return and line feed pairs,
and between a line feed and a space.
It finds `kit` once where only one side holds a line ending,
and where the content is a line ending alone.

Stored artifacts:
the stricter reading would refuse a stored artifact that was right under the reading its pass used,
so the recount of an artifact reads the padding by the generation it records (B682).

Recurrence:
`mistake-prevention-text-comparison-and-structure.md`,
"Text that shows nothing".

## B678: two review findings about link labels did not reproduce, and their cases are pinned

Found on 2026-10-18 (UTC) by the same sol review
(`s245/sol-review/answer-gpt61.md`,
findings 3 and 4).
Neither reproduces on the code at `bc9b2db70d`.

Outcome written on 2026-10-19 (UTC).
Both cases are pinned in `64707f0e48`,
a test-only commit,
in `consistency-fold.unit.test.ts`.

Finding 3 said that a labelled link was read as its label for closing marks.
The case is `kit["](u)'s`,
a word before a link whose label is a straight quotation mark,
and then an apostrophe.
The label is classed `marks` (`linkLabelKindOf`,
`consistency-link-label.ts`),
and it also ends in an open mark (`linkLabelEndsOpenOf`).
`foldedLinksOf` therefore records the stretches of that link in `openLabelStretches`,
`closingJointsOf` (`consistency-closing-links.ts`) sees that the set is not empty and builds the twin,
and `linkTwinOf` writes the link as its label.
The twin `kit"'s` closes,
so the stand is refused,
as the review asks.

Finding 4 said that a raw `)` still joins an apostrophe after a label ending in white space.
The case is `[cat ](u)'kit`.
The existing tests read the forward direction (`[kit ](u)'s bowl`,
in `consistency-closing-links.unit.test.ts`),
and the backward direction after a labelled link had no case,
so it is kept.
`afterBracket` in `consistency-word-joints.ts` excludes `openLabelEnds`,
and a label ending in white space ends open (`linkLabelEndsOpenOf`),
so the word after the apostrophe is a stand,
as it is in the plain text `cat 'kit`.

Control:
each case compares the answer for the linked text with the answer for its plain twin and pins the twin's answer.
A control run showed that the locator answers differently for a label that ends in a letter:
`[cat](u)'kit` is refused and `[cat ](u)'kit` is a stand,
so the comparison can fail.

Recurrence:
`mistake-prevention-guards-searches-and-claims.md`,
"Guards that cannot fail".
