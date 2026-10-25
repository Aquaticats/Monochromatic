# Audit ledger: the source text under a note, the reading comparison and the real corpus

The fourth round of the follow-ups of B832 (the original of an untranslated passage written as literal text):
the comparison of what a reader of the page sees with what a reader of the source sees,
the count of real texts per residual class,
the fixes of the classes and the checks of the round.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.
The rounds before it are in
[the file of the ninth round](audit-ledger-source-text-follow-ups-lone-carriage-returns-and-the-full-pass.md),
where the third fix round ends with the search for a top-level difference and the owner's two-readings decision.

## B832 follow-ups, the fourth round

### The owner's decision to add the comparison, and the comparison

The coordinator ruled on 2026-10-25 that the page check compares the reading of the page
with the reading of the source,
and the owner then answered the same day "Add the comparison (Recommended)".
That is the owner's decision and no longer a ruling open to a veto.
It answers the question the third round left to the owner,
whether to add a reader comparison to the page check.

THE COMPARISON is `readingDifferenceOf` (`page-check/src/corpus-run/source-reading-difference.ts`).
It reduces the page,
read by the strict grammar,
and the source,
read as plain Markdown with GFM,
math and raw HTML,
to the text of their leaves,
as the oracle of the unit tests did (`source-reading-account.ts`,
moved out of the test fixture
`source-reader-text.test-fixture.ts` on 2026-10-25),
and answers `same`,
`differs` or `undecided`.
The composition runs it over every run it writes as literal text (`source-ships-differences.ts`)
and ships the page with one defect,
`source-reads-differently`,
which names the slices and no text of the page.
A run always ships,
so the defect never stops a page.
A run that ships as recorded is not compared,
since what the site compiles is what the original is upstream.
`parseGfmMathBody` moved from the test fixture `gfm-math-reading.test-fixture.ts`
to `document-structure/src/gfm-math-body.ts`
(the fixture is named in the ledger of the parser gaps,
which keeps its old text).

THE TWO-READINGS RULE decides what the source's reading is.
At the top of a text the footnote reading comes from the source,
so a difference there is reported.
Inside a block quote or a list item,
and after a footnote's last block,
it comes from the page.
The comparison cannot give the page's reading of a footnote,
so a difference between the accounts is not reported when the
source's footnote ranges and the page's differ at a place a block quote or a list holds
(`footnoteRangesDifferInContainer`).
After a footnote the page is made to read as the source does (the blank line of `separatingEdits`),
so no difference is left there.

THE CASES IT CANNOT DECIDE,
with the reason of each:

- A footnote in a block quote or a list item whose ranges the two readings disagree on,
  and whose accounts differ:
  `undecided`,
  because the ruling takes the page's reading and a reader of plain Markdown cannot give it.
  The function logs the run at the info level and reports nothing.
- A run the page does not carry as text:
  a text nested past the bound of the plain reading,
  which the strict grammar refuses.
  The comparison throws,
  the composition logs the run at the warn level and reports nothing,
  and such a run is left off the page by the third step of the composition,
  where `page-source-unparseable` names it.
- A run that ships as recorded:
  not asked.
  A text that parses as written ships as the site compiles it,
  so any of the texts of the search that parses as written
  is not compared in production,
  although the search put every one of them through `readingDifferenceOf` on its encoded form.

THREE LIMITS OF WHAT IS REPORTED.
The defect names the slices and no text of the page,
because `PublishDefect.message` carries ids,
counts and slice indices only,
so "the affected text named" is the slices.
`undecided` is keyed on whether a footnote range disagreement exists inside a container of the text,
not on whether it caused the difference,
so a difference of another kind in a text that also holds such a disagreement is withheld too.
Only runs written as literal text are compared.

THE SEARCH,
repeated on the final code with the same texts as the third round (132260,
both seeds,
the footnote product texts,
the shapes),
and with every text now put to the production comparison:
116341 texts read the same by the oracle and by the comparison,
and 0 of them is reported (the false-positive rate is 0 of 116341),
9442 read differently and are reported as `differs`,
1174 read differently and are `undecided`,
and 5303 are nested past the bound,
where the page does not parse and the comparison throws.
Of the 1407 texts of the third round that read differently with the page check silent,
956 now read the same and the other 451 are reported,
none of them undecided
(the first run of the search,
before the last two fixes,
read 36 and 1371).
The search over the generated texts ran in a capped unit,
`dx1f2-topdiff8`.

THE COMMITS:
`76d1c2e53a` is the red cases (`red-r21-failing-lines.txt`),
`dedb7886c0` moves `parseGfmMathBody` into `document-structure` and adds `footnoteRangesDifferInContainer`,
`9d0d9d5dc7` adds the comparison and the defect,
`2797c8f871` adds the unit tests of the modules and the findings of the source scans,
and `6ef07ed14a`,
`b1667e8903`,
`9f062811da` and `ad67735c42` are the two later fixes with their red cases.
The residual classes after them are in the section on the residual classes.
The two cases of `source-shipped-page.unit.test.ts` that hold a comment which never closes now expect the defect.
The unit tests of `dedb7886c0` were written with the code and not before it;
the comparison's behaviour was red first.

### The real corpus

The corpus is the pinned clone `one-among-us/data` under the home directory,
at `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`,
read with `git cat-file` and never written,
and the settled pages of the artifacts the passes wrote.
The units are the 274 files `page.md`,
`page.en.md` and `page.zh_hant.md` of the entries (each text once),
their 1026 heading chunks,
their 7360 top-level blocks,
and the 56 settled English pages of the artifacts of the
`translation-repair` worktree.
A text counts as shipped as literal text when the page grammar refuses it as recorded
or it holds a comment that never closes
(the first step of the composition);
the others ship as recorded and are not compared.

- 61 of the 7360 blocks would ship as literal text,
  and 0 of the 61 read differently;
  no file and no chunk would.
- 1 text reads differently by the oracle and is not compared:
  the front matter of `people/Kotori/page.en.md`,
  which holds
  comment lines of the YAML that plain Markdown reads as headings.
  It counts once in each of four units (the file,
  the chunk,
  the block and the settled page that holds it) and ships as recorded.
- Per residual class,
  the count of real texts is 0 for each:
  a comment that never closes,
  an image whose alternative text is an
  angle autolink,
  an indent-4 footnote start in a paragraph,
  a footnote that opens with a tag alone,
  an indented fence mark beside a
  footnote,
  and any other class.
  No text of any unit holds the lexical trigger of any of them.
  Comment openers occur (118 blocks,
  77 chunks,
  55 files,
  12 settled pages) and every one closes;
  footnote label openers occur in 296 blocks,
  135 chunks,
  68 files and 13 settled pages.
- None of the 56 settled pages carries the note of an untranslated passage,
  since the artifacts on this machine are of the
  repair stage and hold no record of the lanes.
  The pages the source-ships rule wrote are not here,
  so a count over them is not made.

The script is `corpus-measure.ts` in the run directory,
outside the repository,
and ran in the capped unit `dx1f2-corpus1`.

### The residual classes

FIXED,
red first.
A footnote that opens with a tag alone (`d887680218` red,
7 cases in `red-r22-failing-lines.txt`;
`bdbbd2b0a3` fix):
`rootWithFreshTails` parses the text from the line after a top-level footnote's end,
with the lines before it blank,
where a paragraph of the plain parse starts at that line;
a tag alone there is raw HTML to a reader of the source.
34 of the 37 texts of the class read the same now,
and the other 3 are of the indented fence mark class.
The blank line in a block quote or a list item (`9afaf60506` red,
10 cases in `red-r23-failing-lines.txt`;
`1f40b76acd` fix):
`afterLiteralBlockEdits` takes `inContainer` and writes the blank line with the marks of the next line,
so the named limit of
the third round is closed.
Two cases of `source-ships-encoding.unit.test.ts` that pinned the page's text in a container
now compare what a reader sees.
A comment that never closes,
opened at the start of a line (`6ef07ed14a` red,
4 failing cases and 2 controls in
`red-r24-failing-lines.txt`;
`b1667e8903` fix):
a reader of the source takes it for raw HTML to the end of the text,
so the rest is text there;
the parse is given such an opener as it stands,
reads the one raw HTML block,
and the rest is written as the text of such a block
is,
with the opener itself still a character reference
and the four characters of every opener cut out of the raw-text stretches.
`readerOfSource` now blanks the comments that close when another does not,
as the page's reading blanks them.
913 of the 925 texts of the class read the same now.
An indented fence mark in the paragraph a footnote start cuts (`9f062811da` red,
4 cases in `red-r25-failing-lines.txt`;
`ad67735c42` fix):
a paragraph cut at a footnote start was read again alone and lost the indentation of its first line,
so a code span that
opens with an indented fence mark read as a fence that never closes
and the page showed the rest of the text as its code.
`nestedSpans` reads such a piece behind the lead `x ` and moves the spans back (`source-ships-paragraph-lead.ts`).
All 7 texts of the class read the same now.
The earlier text of this section,
in `2725c1280f`,
called the comment class intrinsic to the owner's ruling and the fence class
a matter of context the cutting module does not keep;
both were wrong,
and both are fixed by the commits named in this section.

REPORTED,
NOT CHANGED,
each reported by the comparison:

- An indent-4 footnote start that cuts a code span (384 of the 1407).
  The strict grammar starts the footnote at the indented line and the page shows the backticks as characters;
  the source pairs them across the lazy line.
  No writing keeps both:
  a backslash before the label shows inside a code span,
  and no character can be added.
  The encoder makes the page safe (no live construct),
  and the comparison reports it.
- An image whose alternative text is an angle autolink (55):
  a known finding,
  recorded in `audit-ledger-source-text-follow-ups-parser-gaps-and-step-searches.md`
  ("An angle autolink in the alternative text of an image"),
  left reported.
- A comment that never closes and does not start a block,
  or sits where the plain parse reads no raw HTML block (12):
  an opener inside a display math that is open,
  a fence that a byte order mark hides from the source's reader,
  and the like.
  The page shows the character reference as characters there.
  Not attempted;
  the count in the real corpus is 0.
- A line indented four columns or more right after a link reference definition that begins with a heading mark,
  a list marker or a
  quote mark.
  The source's reader takes it for a paragraph and the page for a heading,
  a list or a quote.
  It is outside the cut paragraph and the 1407,
  found while fixing the fence marks,
  and it is not fixed.
  The comparison reports it,
  and no file of the pinned corpus holds the shape (276 files read).

Of the 1407,
956 now read the same and 451 are reported (384 indent-4 footnote starts,
55 images,
12 openers).

### The full task, the merge, the checks and the slips

The merge of `origin/main` at `a50c2875a2` conflicted with nothing (`git merge-tree --write-tree` exited 0)
and is `b8c0b1dbff7f1c0391fe798536747af4694d62d2`.
It brought no change to `page-check`,
`document-structure` or `markup`.

The full task ran once on `2725c1280fd8d1dc0f8358f972d80bd096ee55fb`,
the head that holds the last code and the ledger,
as the unit `dx1f2-livefull4` with `MemoryMax=4G` and `CPUQuota=300%`,
its stderr on a path never used before.
`systemd-run --wait` reported success,
a service runtime of 9 min 43.561 s,
11 min 24.328 s of CPU time and a memory peak of
749.2 MB with 19.7 MB swapped.
The case that holds the three names passed in 581.8 s.
The one-minute load average of the host was 13.04 when the unit started and 8.91 when it ended.
The stderr file holds three lines:
the two preamble lines of the task and the logger message that a sink could not be verified
within 5000 ms.

The checks ran one at a time on the final code:
`build`,
`lint:types`,
`lint:oxlint` (0 warnings and 0 errors) and `test:unit` of `document-structure` (65 `[PASS]` lines)
and of `page-check` (170),
both with no `[FAIL]`;
`markup` is untouched in this round.
`test:unit` of `translation-repair-source-scan` ran 68 and `source-scans` 53,
with no `[FAIL]`.
The first `source-scans` run on the new code found these in the new files,
all fixed by `2797c8f871`:
two modules with no unit test of their own,
three suites that copied one case body and a helper written twice,
a caught error's message handed on as text,
two blankness tests of the account,
a class with the marker and no inventory line,
a record check in the GFM reading,
and two top-level suites in one file.
The first `test:unit` of `translation-repair-source-scan` after the merge of the third round
failed on a stale built `run` package;
the task `source-scans` builds before it reads.

No cache version moved:
`git log --oneline --extended-regexp -G '_CACHE_VERSION = [0-9]+' 1c7918806f..HEAD` names no commit,
and the same command over `c265e8df95~1..c265e8df95` names `c265e8df95`.
No cached stage changes its output:
the encoder,
the composition and the comparison run when a page is published,
no cache key reads them,
and no package but `page-check` and `document-structure` calls them
(a search of `package` for `encodeAsLiteralMdx`,
`composeSourcePage`,
`footnoteDefinitionRangesOf` and the other names of these
two packages outside them finds only their own tests).
A page published before this round and compared with the page expected now may differ,
which a later pass republishes.
No new B id was taken.

COMMIT-MESSAGE SLIPS of the third and fourth rounds,
none corrected on the remote and nothing posted:

- Positional words in the body:
  `5c874bbe52c55f8b8eaab94b2d35b8db49bdc2f5` ("above"),
  `45b25ac221ed098caa3831822c636ea3f5173baa` ("follows",
  "above"),
  `f4f533493a0a474e2ac1188408e47e74952ffa5b`,
  `cc7aa64640a8137e7b8b760c3560995bfa70d9bb` and
  `a5faefbb710596da17da5fc7b0c39ce4a2d97041` ("follows" in the subject),
  and
  `1f40b76acdaac69d48e86339d48a5cbf18bbfb31` ("next").
- `1f40b76acdaac69d48e86339d48a5cbf18bbfb31` ends on a garbled reference,
  "a8f5d3f...",
  where `f4f533493a` named the limit.
- `873e6ac8ad2ebad5979afd17793700dfb2336f5b` says `document-structure` holds 62 suites;
  the run reads 63.
- `2797c8f8714569cdf347fd02d58a8f2d1078486f` has the type `test` and changes source files
  (the reason of `readingDifferenceOf`,
  `parseGfmMathBody`,
  the marker of `PageSyntaxError`).
