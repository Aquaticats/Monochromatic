# Audit ledger: the consistency pass, the ninth review

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

The ninth independent review of the pass (n138) and the fixes that answered it in n140 and n141.
The review confirmed the fixes of the second round:
J1 to J5 were fixed on the inputs of the eighth review,
the re-admission of dropped gains was sound and maximal on 12,000 random pages
and the reasons of about 41,000 findings were true on the final page,
the grids of n133 reproduced byte for byte,
the corpus list of proven marks equalled n133's,
and the replay read as expected.
It found no input on which a wrong page is written.
The 262 two-edit failures of check b under the parse oracle all have one shape,
a leading gain mark in front of a letter or digit that stands directly after a straight `"`,
and they are the straight `"` rule that n133 decided and the lead accepted (M188),
so they are not a finding.

The second round is filed in
[Audit ledger: the consistency pass, parse boundaries, the letter test and the proof's page](audit-ledger-consistency-pass-parse-boundaries-letter-test-and-proof-page.md),
and the independent oracle that followed the ninth review in
[Audit ledger: the consistency pass, the independent oracle and the rules for marks](audit-ledger-consistency-pass-independent-oracle-and-mark-rules.md).
The entries that this round's fixes closed are filed in
[Audit ledger: the consistency pass, the fixes of the eighth review and the oracle residue](audit-ledger-consistency-pass-fixes-of-the-eighth-review-and-the-oracle-residue.md).
The tenth review of the pass is filed in
[Audit ledger: the consistency pass, the tenth review](audit-ledger-consistency-pass-tenth-review.md).

Written on 2026-10-14 (UTC) from the review (`s199/review.md`),
the hand-backs of n140 (`s202/handback-n140.md`) and n141 (`s203/handback-n141.md`),
and the lead's notes of that day.
The labels L1 to L9,
the guard labels (n01 to n09 are n141's,
e1 to e3 are n140's) and the item numbers are the sources' own and are no ledger ids.
L9 is a mistake of the lead's brief and is filed as M199.
A path that starts with `src/` is under `package/module/translation-repair/` (moved on 2026-10-18 to the packages under
`package/translation-repair`).
Every function,
type and constant of the package named here was found declared in non-test source of the merged tree `w158`
by searches of declarations on the day of writing,
except those an entry says were removed or renamed.
The names that only a hand-back gives (test names,
guard names,
the helpers of test fixtures and the names of logs) were not searched.
The tree changed after these entries were written (n144,
n147 and n149 followed),
and the entries of those rounds say what they changed.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B526: the proof read glyphs that are not the ones that ship

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the ninth review (finding L1,
most severe).

What a reader of the page would have seen:
not shown end to end.
The review built no input where a quotation mark is deleted or an apostrophe is written wrongly.
It showed the proof disagreeing with itself across the glyph change:
`restoreTypography` changes 49,048 of 200,000 random strings of quote marks,
letters,
spaces and `*`,
and in 363 of those the set of proven marks differs before and after.
`".....,"'s'` becomes `".....,"‘s’`,
and the trailing mark proven at offset 9 before is not proven after.
Two more rows are `)"'cats'‘` and `s,*tis’"'tis"""`.
Five natural sentences show no difference.
The direction of harm is a removal judged safe before typography
that deletes the closer of a pair which typography creates.

The cause:
the pass read each slice through `wouldShipTextFor` in `src/corpus-run/pass-consistency-slices.ts`,
while the publishers derive the page through `wouldShipTextPerSlice` in `src/would-ship-text.ts`,
which puts every wording the archive did not decide through `restoreTypography` (`inArchiveTypography`),
and the page-assembly passes (`unifyQuoteStyle` among them) rewrite the composed page after that.
The headers of `src/consistency-page-english.ts` and `src/corpus-run/pass-consistency-slices.ts` did not say so.

The lead decided:
the proof reads the glyphs that ship.
The place where the consistency edits enter the page is found,
the proof's page is that page built by that code,
and the headers say so.

Fixed by n141.
The edits enter in `wouldShipTextFor` (the consistency override ranks under the page-assembly override and over polish),
typography follows in `wouldShipTextPerSlice`,
and `settledPageArtifact` in `src/corpus-run/pass-page-assembly.ts` runs the page-assembly passes after the consistency pass,
through `guardPageAssembly`,
which can withdraw a slice so that it ships the archive text.
`PageEnglish`,
`PageLayout` and `ReadPage` gained a `shipped` text,
`shippedWriterOf` in `src/corpus-run/pass-consistency-shipped.ts` builds it (with a cache of 16 pages),
and `positionOnPage` in `src/consistency-page-english.ts` maps an offset between the written and the shipped text,
answering `NO_PLACE` where no mapping exists.
`unprovenOnPage` in `src/corpus-run/pass-consistency-apply-gains.ts` reads `page.shipped` and treats `NO_PLACE`
or a missing offset as unproven,
`src/consistency-apostrophe-forms.ts` and `src/consistency-fold-apostrophes.ts` use `positionOnPage`,
and `textAroundWorked` uses the shipped length.

Evidence:
the replay of 92 pages and 1,329 slices reads 27 trailing and 1 leading proven mark on the published page,
and 25 trailing and 1 leading on the shipped page.
Two marks are unmapped and stay unproven:
the `girls’` marks at offsets 1565 and 1660 of the page Jennife80677612,
which sit in a stretch that a page-assembly pass rewrote
(that they are the same two marks in both counts is an inference and is not verified).
On 58 of 92 pages the shipped text differs from the text the proof read before.
A page build takes a median of 37.6 ms,
p90 119.2 ms and at most 375.6 ms,
5,661 ms in total for the corpus.
The probes p04,
p10 and p15 are identical to n137's,
and p27 to the review's;
p25 and p26 measure `restoreTypography` alone and are identical by design.

An accepted consequence:
on a page whose straight marks typography curls,
an arriving apostrophe is written curly.
n141 changed two tests that expected a straight `dogs'`,
and the branch "the page's other apostrophes share no form,
so the chosen place keeps its own form" is no longer exercised by them
(an open item,
B554).

Red evidence:
n141's tests were written first for the typography rows,
the property over 120 seeds with a control and the cases of `unprovenOnPage`
(`s203/red-item-1a.log`),
and the mapping tests of `positionOnPage` and the case for a slice the page writes nothing for were written after the code.
They are red under n141's guards n01 (the `shipped` writer of the layout removed),
n02 (the shift of `positionOnPage` removed),
n03 (an unmappable mark not unproven) and n04 (a missing offset not unproven).
Under n04 no test failed at first,
and n141 added the case that leaves out a carried apostrophe of a slice the page writes nothing for,
which fails under the guard.
The guards ran on a build before the last test-only edits.

## B527: the stack-exhaustion conversion changed no outcome at the edge of the stack

Accepted as unreachable;
the text is corrected.
Uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the ninth review (finding L2).

What a reader of the page would have seen:
nothing.
The review swept about 1,500 caller depths over six attempts around the depth where `proseViewOf` exhausts the stack.
In every attempt it returned either the normal view or a raw `RangeError`
(at 35 to 40 depths per attempt),
and never a view converted by the refusal and read as text.
The work after `treeOf` needs more stack than the handler has left,
so the raw error reaches `src/corpus-run/pass-consistency.ts`,
which stops the pass for the page.
That is not a proof that no window exists,
since the compiler's frame sizes moved the edge between attempts.

The cause:
the header of `src/parse-mdx.ts` said that the page is read as text there and the pass goes on,
and `treeOf` in `src/consistency-prose-text.ts` was not parse-only
(its `try` wraps `parseSliceBody`,
which runs three masks and then the parse).

The lead decided:
accept as unreachable and correct the text.

Fixed by n140.
The header of `src/parse-mdx.ts`,
under "what holds at the stack edge",
says that a raw `RangeError` can still reach the pass and stop it for the page,
and states the measured figures:
the pass stands 42 frames deep at its first regex test and 51 at its deepest (`s202/probes/p33-pass-depth.out`),
the trivial body leaves room for 10,936 frames,
and the nesting bound needs 16 to 17 calls more.
The review's "a few hundred frames" was not reproduced.
The header claims no proof that no window exists.
The header of `treeOf` says that its `try` covers all of `parseSliceBody`,
which is reading a slice body,
and that the exhaustion is read as the refusal at the parse call only.
The test of `treeOf` that injected the exhaustion into a mask failed in n140's whole-suite run after B522's narrowing
and was rewritten around `withExhaustionFromParse`;
a new test pins that an exhaustion raised in a mask propagates.

## B528: the re-admission left fewer repairs than another order would have

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the ninth review (finding L3).

What a reader of the page would have seen:
possessives left unrepaired,
and never a wrong mark.
A brute force over every subset of the gain splices of 12,000 random pages
(4,000 plain,
8,000 with markup) showed that the made set was valid and maximal on all of them
and was not the largest valid set on 7:
plain seed 918 (made `[1,2]`,
best `[1,4,5]`) and six markup pages (seeds 1991 and 3262 among them).
That is about one page in 1,700 of the review's random pages.
On seed 3262 the pass repairs one place and the best order repairs two.

The cause:
page order admits the leading gain that comes first,
which pairs with the later trailing marks and blocks them.

The lead decided:
when a page has at most 12 dropped gains,
choose the largest valid set by exact search
(ties:
the set that page-order greed would make,
then the earliest in page order);
for more than 12 keep page order;
and measure the cost on the corpus and on `p06`.

Fixed by n141.
`src/corpus-run/pass-consistency-apply-largest.ts` holds `EXACT_SEARCH_LIMIT` (12),
`NO_LARGER_SET`,
`largestValidSetOf` and `putBackSet`,
and tries the combinations in lexicographic order with the sizes descending.
`madeWithSet` and `floorsWithSet` are in `src/corpus-run/pass-consistency-apply-trial.ts`,
`candidatesOf` and `clonedLedgerOf` in `src/corpus-run/pass-consistency-apply-ledger.ts`,
and `readmittedLedgerOf` in `src/corpus-run/pass-consistency-apply-readmit.ts`
runs the greedy loop on a clone,
searches exactly for a larger set than the greedy one when at most 12 candidates are left out (`leftOutCountOf`),
and lets the greedy loop supply the reasons.
`passOf`,
`refusalsOfBlocked` and `putBackInPageOrder` are in `src/corpus-run/pass-consistency-apply-order.ts`.

Evidence:
the random-page probe `p20` on the final build reads `ok:madeSetMaximum` on all 4,000 plain pages
(the review:
3,999 and one maximal but not maximum)
and on all 8,000 markup pages (7,994 and six).
The exact search builds up to 4,095 sets and one page for each,
a median of 37.6 ms a page,
and only where at most 12 candidates remain;
typical pages drop 0 to 3.
A page with exactly 12 dropped gains was not measured (a unit test with 12 candidates ran in seconds).
The dropped variant of `p06` reads 1,030 / 1,826 / 3,625 / 7,988 ms at 500,
1,000,
2,000 and 4,000 slices in the first of three runs,
and the runs differ by about 6 percent.

Red evidence:
seven pinned seeds (plain 918;
markup 1991,
2385,
3262,
3855,
4844 and 5621) and a tie case (`s203/red-item-2.log`).
n141's guards n05a (`EXACT_SEARCH_LIMIT` of 1) and n05b (of 100),
n06b (no exact result),
n08 (the pass counter advanced by two) and n09 (the reason `overlaps` disabled) are red.
Not shown red:
n06a (ascending sizes) is benign because the greedy loop extends the set,
and the tie rule has no page where it matters
(17 plain tie pages found,
none with greedy a maximum but not the earliest).
The tests for 12 and 13 left out,
the order tests and the overlap test were written after the code and are red only under guards (M195).

## B529: `„` and `‟` were quotation marks to the screen and to no other reader

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the ninth review (finding L4).

What a reader of the page would have seen:
a lone `„` disappears when its word is replaced.
With the receiving words `She said „hello to the kittens` and the chosen words `She said hello to the cats`,
the plan was `replace` and the page read `She said hello to the cats.`
No corpus page was shown to hold one;
the review did not count `„` in the corpus.
The curly closer of `„hello”` was already `unavailable`.

The cause:
the loose-mark sets of `src/consistency-loose-marks.ts` lacked `„` (U+201E) and `‟` (U+201F),
while the screen's list in `src/corpus-run/pass-consistency-apply-screen.ts` had them,
and the proof's set of partners has neither.

Fixed by n140.
`DOUBLE_MARKS` in `src/consistency-loose-marks.ts` holds both,
and its header says that there are five double marks and that `ʻ`,
`′` and `″` are a letter and unit marks and no loose mark at all.
In the probe `p31` row L1 changed from `replace` to `unavailable`
(`s202/final/diff-p31.txt`),
and the rows L3 to L5 (okina,
prime,
double prime) are unchanged.

Red evidence:
`s202/red-item1.log`.
The tests:
`looseMarksOf` reads `„ ‟` and not `ʻ ′ ″`,
adding,
deleting and moving `„ ‟` are `unavailable` in `src/consistency-splice.unit.test.ts`,
and matching `„` pairs still replace and write the receiving form.

## B530: a possessive mark after closing emphasis read free and stalled every later trailing gain

Fixes uncommitted at the time of writing;
commit:
pending.
Narrowed by n143 (B538).

Found on 2026-10-14 by the ninth review (finding L5).

What a reader of the page would have seen:
in `The *kittens*’ bowl is empty. A dogs’ naps.` the `’` after the closing `*` was neither joined nor proven,
so the proof of the page with the later gain read no trailing mark,
and the later gain was dropped with the finding that another quotation mark of the page could pair with it.
The later possessive stayed unrepaired.
The same mark followed by a letter (`*Mio*’s`) was joined and attached.
The doubling `*dogs’*’` of J1 was gone.
The direction is safe,
and the cost is repairs not made.

The lead decided:
a mark that follows a closing-markup run whose character before the run is a Latin letter
reads as a trailing edge mark (the run J1 walks),
so the proof can prove it,
and the leading mirror likewise.

Fixed by n140.
`MarksPastMarkup` has a `trailing` and a `leading` list,
`marksPastMarkupOf` in `src/consistency-marks-past-markup.ts` builds them from right to left in an `Int32Array` of run ends,
skipping a link destination in constant time,
and `src/consistency-markup-runs.ts` holds the characters of the runs (`EMPHASIS_CHARACTERS`,
`CLOSING_CURLY_DOUBLE`,
`TRAILING_RUN` and `LEADING_RUN`;
n143 rewrote the module and replaced the last two by `TRAILING_REFUSAL_RUN` and `LEADING_REFUSAL_RUN`,
so they are gone from `w158`,
B538).
`markReadingOf` names a free mark found past markup on exactly one side as trailing or leading,
leaves a mark named on both sides free,
and leaves a mark with a letter beside it as the letter says.
`provenApostrophesOf` and `markReadingsOf` take the marks past markup.

Evidence:
in the probe `q01-s1-pairs` the proven trailing list `[8]` became `[8,37]` in one row and `[8,37,60]` in another,
and `[]` became `[13,36]` and `[13,37]`.
In `p01-doubled` scenarios S1 and S2 the outcomes became `["chosen","kept-current","replaced","replaced"]`,
the page read `A dogs’ naps.` where it read `A cat naps.`,
and the two left-out findings were gone.
The corpus does not move (27 trailing,
1 leading).
`p16-beyond-markup` and `h1-apostrophes` are unchanged,
since neither holds a free mark past markup that a later gain would have met.
n140 also accepted that `Come (here)'` carries the mark,
because a closing character is in the run;
n143 reversed that acceptance (B538).

Red evidence:
`s202/red-item2.log` (the proof,
the form reader and the pass level),
and n140's guard e3 (the link skip of `marksPastMarkupOf` removed) fails the shape test and the 100,000-run test.
n140's first draft of the link test passed under that mutation,
because the letter before the final `)` named the mark anyway,
and the fixtures were changed to a destination ending in `/`.

## B531: the counter and the apply stage grew faster than the page

Fixes uncommitted at the time of writing;
commit:
pending.
The cost of the apply stage is accepted as linear.

Found on 2026-10-14 by the ninth review (finding L6).

What a reader of the page would have seen:
nothing wrong on the page.
The dropped variant of the apply stage took 49,
114,
317 and 990 ms at 500,
1,000,
2,000 and 4,000 slices,
which is 2.3,
2.8 and 3.1 per doubling,
where n134 had said about 2.
`countConsistency` with one group of N places grew 2.4 to 3.6 per doubling,
and its profile put the time in `namesShownOnce` (`src/consistency-wire.ts`),
`quotesOfPlace` (`src/consistency-majority.ts`) and `unlessOverlapping`.
A single group of thousands of places is not a corpus shape,
and with a repeat group in every slice the locate stage was linear.
The profile of the dropped variant was dominated by the fixture build,
so its cause was not isolated.

The lead decided:
make both linear and profile without the fixture build.

Fixed by n140 for the counter.
`namesShownOnce` uses a `Set`.
`quotesByLabelOf` and `keepsByLabelOf` in `src/consistency-majority.ts` index the places by label once per group,
and `countConsistency` builds both indexes once.
`unlessOverlapping` moved to `src/consistency-count-overlap.ts`
and became a sort by start per slice and a sweep.
In the shape "mark beside" of `p12-locate-cost` the cost of 4,000 slices over 500 (8.0 for a linear cost)
fell from 8.6,
5.0 and 9.9 to 3.4,
3.2 and 3.1;
in the shape "form thunk" from 23.4,
15.6 and 25.0 to 8.2,
8.3 and 8.3.
The "before" runs are the review's build (`w155`) and not a rebuild of the state before n140;
the two are equal on every probe n140 compared.

Measured by n141 for the apply stage,
with no code change.
A temporary counter in `consistencyFloorVerdict` (reverted,
file identical to git,
rebuilt) on dropped and refused `p06` pages of 200 slices counted 199 floors calls,
199 distinct,
199 slices,
so no floors verdict repeats for the same slice and edits.
The profile at 2,000 slices puts 76 percent of the time in `refusalsOfBlocked`,
`floorsWithSplice`,
`floorsOn` and `consistencyFloorVerdict`,
where the same texts are parsed several times per verdict
(`parseMdxBody` builds a new processor for each call;
`sourceCarryFindings` 35 percent,
`hanResidueFindings` 15,
`droppedAddressFindings` 14,
and the set-up of the grammar's extensions about 30).
The cost is about 2 ms a slice,
linear at 2.18,
1.92,
1.97 and 2.04.
The lead accepted that a shared processor or a per-verdict parse cache is not built,
since it would touch `src/parse-mdx.ts` and `src/translate-validate.ts`,
which the translate lane shares.

Red evidence:
n140's `s202/red-item3.log` (the two read-bound tests fail on `reads.count`),
its guard e1 (the sweep replaced by a scan of every rendering fails the read-bound test)
and e2 (the next-start clause removed fails both grid tests).
No explicit remove-rebuild-run-restore experiment was made for the majority read bound (M194).

## B532: the planner grid showed 36 more `replace` plans than n133 recorded

Accepted.
No defect shown,
and no code changed.
Uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the ninth review (finding L7).
The grid `h2-grid` read `replace` 74,134 against n133's 74,098 and `unavailable` 115,685 against 115,721,
with `same` equal at 10,181.
The planner files were identical to n133's final state and the `g2` and `claims` grids reproduced n133 byte for byte,
so the difference came from n133's recorded `h2-grid` run,
which n133 says ran before its last refactors.
The review could not attribute it,
because no build of n133's final state existed.

What a reader of the page would have seen:
nothing.
The oracle counts `wordsOutside`,
`outsideMarks`,
`attachedFlips` and `doubleChanged` were all 0.

The lead attributed it on 2026-10-14.
The grid run on n133's final tree (`w157`,
which is `s196/w148-before-merge` laid on `211bb37537`;
output `s201/h2/h2-grid-n133-final.out`) is byte-identical to the review's run on the merged tree,
so the 36 came from n133's last refactors,
which ran after its recorded grid run
(its hand-back says the grids were not rerun on them),
and the oracle finds all 36 safe.
Which refactor moved them was not traced.

Prevention:
the numbers of a hand-back come from the final build only
(`mistake-prevention-shell-edits-commits-and-agents.md`,
"Briefs and hand-backs").

## B533: comments, headers and terms that the ninth review found wrong

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the ninth review (finding L8).

What a reader of the code would have seen:
- The example of `readingsChangeOutside` in `src/consistency-splice-readings.ts` documented `true` for `end: 14`,
  which cuts the surrogate pair of the emoji,
  and the call returns false.
  With `end: 15` it returns true (`s199/probes/p32-doc-examples.out`).
- `src/consistency-proven-apostrophes.ts` said 1,200 single quotation characters in one place and 1,199 marks in another,
  and the difference of one was unexplained.
- "Standing" named two ideas:
  where a span stands among blocks (`consistency-carry-standing.ts`)
  and what the re-admission step knows of the page (`pass-consistency-apply-standing.ts`);
  the review gives both names without a directory and marks the paths unverified.
- "Screen",
  "trial",
  "stand",
  "joint" and "gain case" were defined only in some headers.
- The two headers of the page (L1) and `src/parse-mdx.ts` (B527) said otherwise.
- The reasons `unproves` and `overlaps` never occurred in the findings of the random-page probe.

Fixed by n140 and n141.
Both rounds changed the example to `end: 15`,
and n140 added an `@param end` note and a test that runs every `@example` of the file in `node:vm`
(red in `s202/red-item4-examples.log`:
the answers were `[true,'boolean',true,false]` against the commented `true`).
n140 reconciled the counts by measurement:
1,199 is `'` 65,
`’` 1,125,
U+02BC 0 and `‘` 9,
and 1,200 adds the one backtick that opens no code;
n141 repeated the count and found the same split.
n141 renamed the apply stage's "standing" to "ledger"
(`src/corpus-run/pass-consistency-apply-ledger.ts` was `-apply-standing.ts`),
defined "screen" and "trial" in the headers of the readmit,
screen and trial modules,
rewrote the readmit header (the loop ends because puts back only grow,
the passes number at most the left-out count plus one,
and a test builds a page that needs every pass),
and n140 defined "stand" and "joint" where they are first used in its files and pointed "gain case" to `src/consistency-splice.ts`.
The reason `overlaps` is reached through `applyConsistency` (its test is red under n141's guard n09).
The reason `unproves` is covered only by the hand-built test with two touching splices;
a whole-pass attempt ended in `gain-pairs`,
and it was neither proven unreachable nor removed (an open item,
B554).
After n143 the header of `src/consistency-proven-apostrophes.ts` reads 1,200 characters
where n143 counts 1,190 marks (an open item,
B555).

## B534: the edge reader and the proof read the run past markup differently

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n140's own grid run,
and not by its brief.

What a reader of the page would have seen:
a proven apostrophe could end as a free mark.
After the first three items of n140's brief the `g2` grid read single-edit check a at 16 pages under the parse oracle,
and two-edit a at 149 and b at 778.
A proven mark past markup (`_cats_’`,
`’**cats**`) stood at the edge of a span,
but `isEdgeApostropheAt` in `src/consistency-loose-marks.ts` still needed a letter directly inside the mark.
The span read the mark as a loose mark,
the plan was no gain case,
and a proven apostrophe could end as a free mark on the page.

Fixed by n140.
`isEdgeApostropheAt` steps over the same run (`TRAILING_RUN` and `LEADING_RUN`,
since replaced by n143's constants,
B538),
so the splice,
`bringsAtEdge`,
`src/corpus-run/pass-consistency-apply-gains.ts` and `src/corpus-run/artifact-two-lane-read-consistency-span-located.ts`
(the last two read `holdsEdgeApostrophe`) agree with the proof.
Two files of the other author changed behaviour with no edit.
Red evidence:
`s202/red-item2b.log` (`looseMarksOf` and `holdsEdgeApostrophe`).

After the fix n140 reported single-edit checks a to d at 0 on 144,227 pages of `g2`,
two-edit b at 221 (the review:
262),
and attributed every grid movement by planning the same pairs on the review's build and on its own.
In `h2-grid` the replace count rose from 74,134 to 78,790,
and the same pairs read 70,835 on the old planner:
the draw moved the count by minus 3,299 and the plan by plus 7,955
(7,929 `unavailable` to `replace`,
54 `same` to `replace`,
28 `replace` to `unavailable`,
11 `same` to `unavailable`).
In `g2` the same pairs read 117,889 against 144,227
(26,923 `unavailable` to `replace`,
585 `replace` to `unavailable`),
every moved pair holding a mark past markup.
In `claims` replace fell from 42,744 to 39,395,
and the same pairs read 41,658 against 39,395
(3,661 `replace` to `unavailable`,
of which 3,481 hold a mark past markup and 180 a Han word replaced by Latin words
that `readingsChangeOutside` catches).
These zeros were read under copies of the oracle and of `final-check.mjs` that the author had changed to the new rule,
so they check the author's reading against itself.
The review's own oracle,
unchanged,
still read single-edit check a at 16 at the intermediate state
(`s202/grid2/g2-grid-n132.n140.out`).
The independent oracle of n142 found what these zeros hid (B535 to B541,
M194).
