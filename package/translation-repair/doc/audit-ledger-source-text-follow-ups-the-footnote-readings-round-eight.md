# Audit ledger: the source text under a note, the footnote readings, round 8

The follow-ups of B832 (the original of an untranslated passage written as literal text)
that fix at source the residual classes the fourth re-check recorded,
fix the regressions of the containers re-check,
pin the tab path of the footnote container,
and make the placing of the block comparison linear:
an image whose alternative text is an angle autolink or a code span,
an indented code block before a list marker or a star,
indented display math after a line of a paragraph,
a footnote that opens with a tag alone,
a fence mark indented four columns after a line,
the twenty-nine minimal texts of the containers re-check,
and the time of `placedAfter`.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.
The record this round answers is
[the file on the plain reading with tables and the containers][plain-file],
whose sections "The residual classes" and "Stated limits after the re-check" name the classes fixed here.

## Round 8 at a glance

Every item had a red case that failed on `fd7f44c844` before its fix,
and each fix is one commit after its red commit.
The commits,
in the order they were made,
by item:

- B970,
  the image alternative text:
  `487cf98370` and `2e302754be`,
  then `ba5a5b569c` and `7cce751d38`.
- B971,
  the list marker after indented code:
  `08f93b475c` and `18924a613a`.
- B972,
  indented display math:
  `520f4426dc` and `646ccebe81`,
  `e9423b3c5a` and `61d608117f`,
  then `a9b77b764e` and `4a70e75162`.
- B973,
  the footnote that opens with a tag alone:
  `91ca64ab14` and `26d9014cca`,
  with the label-line commits `c710338f2f`,
  `68eb0a5c8b`,
  `5280225ddb` and `6d6e9ee363` of B975.
- B974,
  the fence mark indented four columns:
  `65e059ba9e` and `e9d5bf907f`,
  then `86f3e3e72d` and `31f872e176`.
- B975,
  the containers re-check:
  `78fc0af4cf`,
  `c710338f2f`,
  `12a2a3ca39`,
  `68eb0a5c8b`,
  `5280225ddb`,
  `6d6e9ee363`.
- B976,
  the tab pin:
  `db45d9f487`.
- B977,
  the placing:
  `3d49942409`.
- B979,
  the scans and the own tests:
  `3a7faed7af`.

The search table (132260 texts) went from 125025 that read the same,
1748 that differ,
184 undecided and 5303 not compared,
to 126659 that read the same,
130 that differ,
168 undecided and 5303 not compared.
No text that read the same before differs now (B978).

## B970: an image alternative text is written as inline content, and read as it stands between its brackets

`![<https://example.net/a>](u.png)` and ``![`<https://example.net/a>`](u.png)``,
1295 of the 1748 texts of the search that differed.
The plain parse holds the alternative text of an image as a string,
and the page shows the address with its brackets where the source shows it alone.

The encoder reads the alternative text between the brackets of the image as inline content,
writes the angle autolink and the code span as their own text,
and reads an alternative text that runs over several lines of a container as it stands.
The module is `source-ships-autolink.ts`,
and the cases are in `source-ships-image-alt.unit.test.ts`
(red `487cf98370` for the autolink and the code span,
red `ba5a5b569c` for several lines in a container).

In the search there were 1295 differing texts of this class at `fd7f44c844` and none at the head.
The pool text `![<https://example.net/a>](u.png)` at line 190 of `source-live-sample-pool.test-fixture.ts`
was deliberately left as it stands:
it is a text of the pool and not a case that asks the text to differ,
and the cases of the pool pass with it.

## B971: a list marker after indented code is written with a backslash

`    注\n2.`,
31 of the 1748 (19 in the record):
the encoder wrote a blank line after indented code,
and the page read the marker as an empty list item.

The marker of the line after an indented code block gets a backslash before the character that makes it a
marker (`markerEscapeEdits` in `source-ships-list-marker.ts`),
for a number followed by a dot or a bracket and for a star,
a plus and a minus.
`isIndentedCodeBlock` is the test of a block that is indented code.
Red `08f93b475c`,
fix `18924a613a`.

In the search there were 31 at `fd7f44c844` and none at the head.

## B972: indented display math after a line of a paragraph is copied as it stands

`a\n    $$\n    {a}\n    $$\n`:
the indented `$$` after a line of text is no display math to the plain parse,
and the encoder wrote the lines as display math whose openers the page read.

The encoder copies indented display math after a paragraph line as it stands
(`646ccebe81`,
red `520f4426dc`),
splits an opener out of indented display math as a display math (`61d608117f`,
red `e9423b3c5a`),
and reads a part of a paragraph in a footnote that starts with `$$` without the lead that stands for the paragraph
(`4a70e75162`,
red `a9b77b764e`).
The last was found by the containers family of the search (seed 13):
two texts of the form `[r]:u\n    $$\n    <!--\n    $$\n[^1]:` read the same before the fix of the split and differed
after it.
The red case of that pair was made red by a temporary revert of the fix file,
since the fix of the split had already landed.

## B973: a footnote that opens with a tag alone is read with the lines of its raw HTML as text

The class has 173 texts (```` - [^1]:<b>\n      ``` ``` ````) of the search at `fd7f44c844` (176 in the record).
The raw HTML of a footnote that opens with a tag alone runs to the end mark of its kind,
and the lines in it that look like a fence or a display math were written as openers.

`source-ships-footnote-html-region.ts` finds the raw HTML region of a footnote that opens with a tag alone,
by the end mark of the kind of the tag (`endMarkOf`,
`regionEndOf`),
and the encoder writes the lines in the region as text (`26d9014cca`,
red `91ca64ab14`).
A GFM footnote ends its raw HTML at the closing mark for the first five kinds of raw HTML,
which the plain parse does not.
The label-line opener,
the table in the label line and the lazy-line rule that came from this work are in B975.

In the search there were 173 at `fd7f44c844` and 4 at the head,
and the 4 are tag-alone footnotes whose fence is also indented four after a line.

## B974: a fence mark indented four columns after a line of text

The class has 235 of the 1748 (```` [b]:x\n    ```\na``` ```` and ```` >\t```\n>     ``` ````).

The first character of an indented fence mark is written with a backslash
(`source-ships-fence-join.ts`,
`e9d5bf907f`,
red `65e059ba9e`),
and a literal backtick in a text the encoder writes is a character reference (`&#96;`,
`31f872e176`,
red `86f3e3e72d`):
micromark pairs the backticks of a code span without reading a backslash,
so a backslash does not write a literal backtick,
and a character reference does.

One shape stays a stated limit:
a fence in a block quote opened after a tab or a two column opening,
with a closing mark indented four:
76 rows of the search,
26 shapes after reduction,
The form is ```` >\t```\n>     ````.
The closing mark of such a fence is counted from the column of the quote,
and a rule built from the columns of the quote alone made 125 texts of the containers family differ that read the same.
That rule was reverted (three cases removed),
and the shape stays a limit until the closing arithmetic of `fenceLengthEdits` knows the indentation relative to the
quote.

In the search there were 235 at `fd7f44c844` and 113 at the head
(112,
and one that is also a list marker).

## B975: the containers re-check, twenty-nine minimal texts

The re-check of the containers found 29 minimal texts whose page read differently from the source after the
round before,
reduced from the 1748 differing and 184 undecided texts of the search.
They are cases of `source-ships-container-regressions.unit.test.ts`
(red `78fc0af4cf`,
which asks `readingDifferenceOf(...).kind === 'same'` after `12a2a3ca39`).

The causes and their fixes:

- A footnote start on a label line that follows a definition or a quote opener
  was not read as an opener of a block of raw HTML under the label:
  `source` reaches `footnoteReadingsOf` and `rawHtmlRegionsOf`,
  and the region ends at `endMarkOf`
  (`c710338f2f`).
- A table in the label line of a footnote does not start a table on the page:
  `source-ships-label-line-table.ts` (`withoutLabelLineTable`,
  `bodyStartsOnLabelLine`),
  and a table opens a footnote paragraph (`PARAGRAPH_LIKE` in `source-ships-footnote.ts`),
  and `blockOpenerStarts` unions the plain and the GFM openers of raw HTML (`68eb0a5c8b`).
- A footnote of raw HTML ends at a line that carries none of the container's marks and is no lazy text
  (`carriesContainerIndentation` in `source-ships-footnote-container.ts`) (`5280225ddb`).
- A footnote that only the page reads is cut at the table that cuts its paragraph
  (`cutAtTable` with `separatingEdits`,
  `tableRightAfter`) (`6d6e9ee363`).
  The helpers of the nested reading moved to `source-ships-nested-text.ts`.

After the fixes 25 of the 29 minimal texts read the same.
Four stay limits:
a lazy code line after a footnote of a quote whose header is a table,
a quote followed by a definition and a footnote whose code line sits in the quote,
a list footnote body with a line that carries no indentation,
and a deep quote with a `>` that stays unescaped.
`[^2]:w` without the space after the colon differs at `fd7f44c844` without a table as well,
because the account of the label has another format,
and the case asks `[^2]: w`.
Of the 86 undecided minimal texts,
75 now read the same and 11 remain undecided.

Nine texts of 160000 of the containers family went from reading the same to undecided.
Three were reduced:
a list item `1. [^1]:x` with a comment line,
a fence of the item's own and a closing mark
(caused by the union of the openers of raw HTML),
a table row of a quote followed by a footnote and a tag,
and a list item with a star,
a footnote definition and display math that holds a comment opener.
They are undecided and not wrong,
and the page is shipped as recorded.

## B976: the tab path of the footnote container is pinned

`source-ships-footnote-container.ts` takes a space or a tab for one column of what the container puts before a line
(the condition `(character === ' ') || (character === '\t')` of the pattern walk).
`source-ships-footnote-container-tab.unit.test.ts` holds three cases.
The mutant that leaves the test of the tab alone (`else if (character === ' ')`) made all three fail
(`db45d9f487`),
and the file was restored with `git show HEAD:path`.

## B977: the placing of the block comparison is in the time of the places that matter

`placedAfter` grows by four for each doubling of the blocks:
for a page of blocks `x y` of which every fourth is reached,
1600 blocks took 0.13 s,
3200 blocks 0.37 s,
6400 blocks 1.5 s,
12800 blocks 7.2 s and 25600 blocks 36.8 s.

The places are walked in the order of the page with four prunings:
the lookahead (`source-reading-lookahead.ts`:
a place that costs as much as the least place plus the cost of the stretch
after it is dropped),
the bound (`source-reading-bound.ts`:
a cost some way is known to have,
found by a beam of the eight cheapest places
along two offsets,
and places that cost that or more are not written),
the overlap index (`source-reading-overlap.ts`,
which replaces the scan of every block against every range),
and the cheapest placing that combines them (`source-reading-cheapest.ts`).
`source-reading-words.ts` holds the words of the blocks.

The answers are the same:
`blockDifferenceOf` of the old and the new code gave the same answer for 60000 cases (seed 1) and
100000 cases (seed 2),
every case of the search set and of the re-check shapes with it,
and the unit oracles compare the cheapest placing with an enumeration (6000 small cases) and with a pair by pair
reading (4000 long cases).

On the time,
25600 blocks took about 24 ms,
51200 about 100 ms,
102400 about 190 ms and 204800 between 235 and 450 ms.
A page in which one reached block is missing took 27 ms at 12800 and 187 ms at 51200,
and two missing took 133 ms at 12800 and 608 ms at 102400.
The spread of seven runs on a machine with no other load:
25600 from 60 to 110 ms,
51200 from 99 to 148,
102400 from 185 to 248,
204800 from 451 to 674,
and the ratios of the least runs per doubling are 1.65,
1.87 and 2.44.

The growth case is not red first:
the smallest size that shows the growth takes minutes on `fd7f44c844`,
so the evidence that the old code grows by four per doubling is the old timings measured for this record,
listed in the paragraph on the time.
The case compares the least of 11 runs at 25600 and at 102400 blocks and fails when the larger takes eight times the
smaller or more:
a quadratic placing takes 16 times,
the new code on a quiet machine 3.1 times,
and 5.6 times once under the whole suite.

One regime stays quadratic:
a page with many changes of length gives the beam no bound,
and `placedAfter` then runs with the lookahead alone:
three to four times faster than the old code and still quadratic.

## B978: the measures

The search table,
132260 texts,
`fd7f44c844` and the head:

- Read the same:
  125025 and 126659.
- Differ:
  1748 and 130.
- Undecided:
  184 and 168.
- Not compared:
  5303 and 5303.
- From the same to differs:
  none;
  of the 5601 rows that did not read the same,
  the rows before and after the placing
  (B977) are equal.

The classes of the differing texts at `fd7f44c844`:
image alternative text 1295,
tag-alone footnote 173,
fence mark indented four 235 (it counts texts of other classes as well),
list marker 31,
unclassified 14.
At the head:
fence mark indented four 113,
tag-alone footnote with a fence 4,
unclassified 13.
The remaining 130 reduce to 26 shapes:
76 of the form ```` >\t```\n>     ```` (B974),
24 of the form ```` >[^1]:\n    ``` ```` (a table in the label line of a
quote footnote with a lazy code line),
and single shapes (a brace,
a footnote with a star,
a comment opener with a code span).

The families of the re-check,
`fd7f44c844` against the head,
40000 texts of the containers family a seed:
seed 7 gave 35266 that read the same before and after,
3320 that differed and read the same after,
1038 undecided that read the same after,
204 that differed before and after,
170 undecided before and after,
one from the same to undecided and one that throws before and after;
seeds 13,
21 and 33 gave none from the same to differs and 3,
1 and 4 from the same to undecided.
The tables family gave 51072 that read the same before and after.
Generated texts (20000 of seed 99 and 20000 of seed 5 with a byte order mark) all read the same before and after.

The real corpus has 276 files and 61 literal blocks of the pinned clone,
read by `git cat-file` and counted,
never copied:
61 of 61 literal blocks read the same,
7299 shipped as recorded,
and 1 would read differently encoded (the front matter).
The counts are the same at `fd7f44c844` and at the head.

## B979: the gates, the scans, the tools that failed and the slips

The gates:
`build`,
`lint:types` and `lint:oxlint` of `page-check` exit 0.
`test:unit` of `page-check` passes 256 to 258 cases and fails the 8 `polishConsolidation` cases and the
`publishFixedPage` case that start `monochromatic-lint`,
which passed before the merge of `origin/main`.
The source scans pass 54 and fail the three cases of the family "Markdown linter over the family's living docs".
Both failures are the tool lookup in this section.

On the tool lookup:
after the merge,
`monochromatic-lint --config
package/translation-repair/typography/src/semantic-wrap.config.jsonc --stdin --stdin-filename passage.md --fix` printed
`mise ERROR No version is set for shim: monochromatic-lint`,
and the scan reads `monochromatic-lint ENOENT`.
The tool is not installed by this branch and was not worked around.

On the cache versions,
`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' fd7f44c844..HEAD` prints nothing,
and its control on `c265e8df95~1..c265e8df95` prints `c265e8df95`.

The slips of the round:
edits of repo files through `node --eval` and `sed`,
one call that chained with `;`,
large outputs read in full,
evidence capped with `--max-count` and `--context`,
scratch worktrees added to the shared `.git`,
and a bisect by `git checkout --detach` in one of them.

[plain-file]: audit-ledger-source-text-follow-ups-the-plain-reading-with-tables-and-the-containers.md
