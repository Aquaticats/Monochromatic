# Audit ledger: the consistency pass, one reading of every mark

The record of the work that gave a single mark one meaning wherever the pass reads it:
the counter,
the locator,
the proof that an edge mark is an apostrophe,
the reader of the page's form,
and the reader of marks inside the words.
It holds the apostrophe after closing markup,
the three findings of the independent grid check (n132),
and the findings of the eighth review (n131) that the fix round n133 closed,
with the two limits that round left accepted.
The stack depth,
the floors fixed point and the page form are filed in
[Audit ledger: the consistency pass, stack depth, the floors fixed point and the page form](audit-ledger-consistency-pass-stack-depth-floors-fixed-point-and-page-form.md),
and the findings of the eighth review that the later rounds closed,
with the residue that followed them,
in
[Audit ledger: the consistency pass, the fixes of the eighth review and the oracle residue](audit-ledger-consistency-pass-fixes-of-the-eighth-review-and-the-oracle-residue.md).
The rounds after them are filed in
[Audit ledger: the consistency pass, parse boundaries, the letter test and the proof's page](audit-ledger-consistency-pass-parse-boundaries-letter-test-and-proof-page.md),
[Audit ledger: the consistency pass, the ninth review](audit-ledger-consistency-pass-ninth-review.md)
and
[Audit ledger: the consistency pass, the independent oracle and the rules for marks](audit-ledger-consistency-pass-independent-oracle-and-mark-rules.md).
The seventh review is filed in
[Audit ledger: the consistency pass, the page-wide proof, forms, floors and limits](audit-ledger-consistency-pass-page-wide-proof-forms-floors-and-limits.md).

Written on 2026-10-14 (UTC) from the hand-backs of n130 (`s187/handback-n130.md`),
n132 (`s189/handback-n132.md`) and n133 (`s191/handback-n133.md`),
the eighth review (`s188/review.md`) and the lead's notes of that day.
The labels J1 to J8 and K1 to K3 are the review's and the check's own findings,
and the labels of guards are the hand-backs' own;
none is a ledger id.
The guards G1,
G9 and G10 named in B504 are n130's,
and every other guard label in this file is n133's.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B504: an apostrophe after closing markup had two readings, one in the proof and one in the cut of a word

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n128 and sent back to n130 by the lead.
The joint rule of B493 treated the apostrophe of `**cat**'s` as joining the letter after it,
so no stand is cut inside that word.
The proof of B492 read the same apostrophe as a possible opener.
n128 met the difference when its page-wide reading failed an existing pass case:
a `dogs'` in a slice after `**cat**'s` was not proven,
so it was not carried.
n128 reordered the case so that `dogs'` comes first and added a case for the other order
(outcomes `chosen`,
`unlocated`,
`kept-current` and nothing changed).
It named the change as a changed expectation of an existing test,
and both readings as safe,
since a mark is kept and never dropped.

What a reader of the page would have seen:
a plural possessive left unrepaired whenever an apostrophe after closing markup stood before it on the page.

Fixed by n130.
`joinedMarksOf({ english })` in `consistency-closing-marks.ts` gives the marks that `closingJointsOf` joins.
`provenApostrophesOf({ text, attached })` in `consistency-proven-apostrophes.ts` reads a mark in `attached` as kind `attached`,
which is inert,
and `proofViewOf` passes `attached: joinedMarksOf(...)`.
`foldWithOffsets` moved to the new `consistency-fold-offsets.ts` to avoid an import cycle,
and its type is named `OffsetFoldedText`,
because `FoldedText` collided with the type of `invisible-variants.ts` (TS2308 in `index.ts`).
The n128 case is back in its original order with new expectations
(outcomes `chosen`,
`unlocated` and `replaced`,
one slice changed),
and the reordered case stays as its own case.

Red evidence:
eight shapes of closing markup before a possessive
(`**cat**'s`,
`_cat_'s`,
`"cat"'s`,
`“cat”'s`,
`(grey cat)'s`,
`[the cat]'s`,
`[the cat](x)'s` and `**cat**’s`),
each followed by a `dogs'` that must be proven,
in `consistency-fold-apostrophes.unit.test.ts`;
the controls `*'Tis*`,
`**cats**'` and `"'cat"` stay unproven.
Three cases failed and the control passed (`s187/red-inert.log`),
and the new `joinedMarksOf` cases and `consistency-fold-offsets.unit.test.ts` failed on missing imports (`s187/red-joined.log`).
Guards:
G9 (the `attached.has` branch of `provenApostrophesOf` removed) fails two cases of `provenApostrophesInEnglish` and one of `runPassConsistency`;
G10 (`attached: joinedMarksOf(...)` not passed from `proofViewOf`) fails the same three;
G1 (the `closesAfter(previous)` call in the apostrophe branch of `closingJointsOf` removed) fails 43 lines of cases in the suites of
`closingJointsOf`,
`joinedMarksOf`,
`cutsWord`,
`provenApostrophesInEnglish`,
`slicePlansOf`,
`settledSlicesOf`,
`applyConsistency`,
`withoutUnprovenGains` and `locateWords`.
The `consistency-fold-offsets.ts` extraction was written after the code (M184).

Results on the corpus (92 pages,
1,329 slices,
`s187/replay-pagewide.stdout`):
before this change 27 trailing and 5 leading marks were proven,
and after it 27 trailing and 1 leading.
The four leading marks that became attached:
`*Yorushika*’s music` (Susiethegamer,
offset 722),
`(…SS3B_0016/)’s Iveco van` (a2581911655,
offset 1049),
`*Arcaea*’s *Last | Moment*` (lxyddice,
offset 1995),
and `[…](https://twitter.com/yqua_)’s Chronicle` (mikaela_khara,
offset 9387).
No trailing mark changed,
and Kotori's `friends’` (offset 969) stays the one mark that the page reading proves differently from the slice reading.

Results on the grids,
which moved from n128's output only through this change (ablation with the G10 edit reproduces n128 byte for byte):
`claims` (150,000 pairs) dropped 2,663 gains where n128 dropped 2,556;
`g2-grid2` (300,000 pairs) kept 45,655 gains where n128 kept 49,033,
dropped 7,003 where n128 dropped 3,625,
and counted 3,378 over-drops where n128 counted none.
Over-drops are gains left out,
which is the safe direction.
The 68 under-drops of the two-edit pages are pairs where the leading gain is now left out,
so the trailing gain no longer has it as a partner and stays.
The grid oracle still held n128's reading,
so those 68 were not independently checked in n130.
n132 checked them (B505 to B507):
all 68 leave a safe final page.

The mismatch that n130 named and accepted:
the joint rule asks for a Latin letter or an ASCII digit after the mark,
and the proof asked for a letter or digit of any script.
B507 makes it one test.

## B505: a splice changed how a mark outside the carried span reads

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n132 (finding K1),
the independent check of the gain decisions under the joint reading.
The check read the final pages of the grids with a test that no mark outside the carried spans
changes between inert (attached or joint) and loose,
or between a plain letter or digit on both sides and not.
It failed on 460 single-edit and 5 two-edit pages of the 300,000-pair grid,
and on 588 pages of the 150,000-pair `claims` run.
The grid pages have one shape:
the receiving words end in an emoji and an apostrophe stands before a Latin letter or digit (`🐱's`),
and the incoming words end in closing markup,
so after the splice the apostrophe is a joint.
The `claims` pages have three:
414 pages with `”’猫` where a receiving word starting in a non-Latin letter and incoming words starting with a Latin letter or digit
turn a leading mark into a joint (332 with a letter after and 82 with a digit),
77 in the other direction,
and 97 where the incoming words end in `**` or a straight `"` and a loose leading mark follows another mark.

What a reader of the page would have seen:
no quotation mark lost or added (check a passed on every page),
and no kept gain that depended on the change.
A mark beside the replaced words read as an apostrophe of a word where it had read as a quotation mark,
or the reverse.
That could matter if a gain elsewhere relied on the changed mark.
The corpus counts of n130 (4 marks after closing markup and before `s`) say the shape is rare.

Cause:
the edge test of a splice (letter or digit,
another character,
nothing) is coarser than the joint reading (a Latin letter or ASCII digit,
closing markup,
another character).

Fixed by n133.
The new `consistency-splice-readings.ts` holds `readingsChangeOutside`,
`gainsBesideKeptMark` and `bringsEdgeApostrophe` (moved there from `consistency-splice.ts`).
`planConsistencyEdit` in `consistency-splice.ts` takes an optional `english` and calls them,
and `corpus-run/pass-consistency-decide.ts` passes it.
After a splice is built,
every single mark of the block outside the span is read before and after with the reading the proof uses,
and any change makes the place `unavailable`.

Red evidence:
the new `consistency-splice-readings.unit.test.ts`,
and guards G1 (the planner call removed) and G2 (the `gainsBesideKeptMark` call removed),
which fail the planner case for a mark outside the span and three splice cases.
On the final build the oracle of n132 gave no failure of its four checks on any single-edit plan of the grid (134,956 pages),
and none on `claims` (all three variants of the check).
The grids and the corpus replay were not rerun on the code after the last refactors of n133
(the memo,
the `offsets` rename and the `ReadMark` type);
the 2,299 passing cases of the suite cover those,
and the grids do not (unverified).

## B506: `*`, `_` and `~` counted as closing markup by their shape

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n132 (finding K2).
The rule says an apostrophe after closing emphasis joins its word.
Production counted `*`,
`_` and `~` as closing by shape,
and a parse of the page counts only the end of an emphasis or strikethrough node.
In 1,713 single-edit plans a lone asterisk stood directly before the apostrophe and a letter (`猫*'cats`),
and production left the gain out where the parse keeps it.
The straight double quotation mark was the second pattern (1,665 single-edit plans):
the rule text says "a closing straight double mark" and gives no way to tell an opening one from a closing one.

What a reader of the page would have seen:
a possessive left unrepaired.
The page keeps the receiving words,
which is the safe direction.

The lead decided on 2026-10-14:
`*`,
`_` and `~` count as closing markup only when the parse makes them the closing delimiter of an emphasis or strikethrough node,
the same delimiters that the prose view blanks,
so a lone `*` is a character.
A straight `"` keeps the shape test
(a character that is no white space and no opening mark before it makes it close),
and the rule is now written in the headers of `consistency-word-joints.ts` and `consistency-closing-marks.ts`,
since the text of the rule was silent.

Fixed by n133.
`proseViewOf` in `consistency-prose-text.ts` records `EmphasisDelimiters.offsets`
(renamed from `.all`,
because the scan of Promise combinators flags `.all`),
which `consistency-fold-offsets.ts`,
`consistency-fold.ts` and `consistency-word-joints.ts` read.
`proseTextOf` is removed from production and from the barrel by the dead-code scan,
and `consistency-prose-text.unit.test.ts` holds a local copy over `proseViewOf`.
B492 keeps its original `proseTextOf` sentences as the history of that round.

Red evidence:
guard G3 (closing emphasis read by shape again) fails in the suites of the closing marks and of the word joints for a lone asterisk,
and G3b (the opening exception of a straight `"` dropped) fails the case that reads a straight quotation mark as closing.
Results (`s191/probes-n132/g2.console`,
300,000 pairs and 100,000 two-edit draws):
under the parse-based oracle P of n132 the single-edit checks a to d are all 0 (134,956 pages),
and the two-edit pages have check b at 262,
all of them straight `"` gains where the decided shape test and the parity of P disagree.
The lead accepted those 262 as the stated rule of this entry.
Under the textual oracle T the single-edit pages have check c at 72 and check b at 2,119,
because T reads a lone `*` as emphasis;
they cannot reach 0 without undoing the decision.
The brief had asked for 0 under T (M188).

## B507: the letter next to a mark was tested in two ways

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 as the named mismatch of n130 and measured by n132 (finding K3):
the joint rule asked for a Latin letter or an ASCII digit after the mark,
and the proof asked for a letter or digit of any script;
n133 found the same any-script test in the reader of marks inside words and in the splice.
In the `claims` run 414 pages flipped one way and 77 the other,
which are the pages of B505 where the incoming words start in another script than the receiving words.

What a reader of the page would have seen:
a mark beside a Chinese,
Japanese or Korean character that the locator cut as a joint and the proof read as attached,
and a splice that changed which of the two it was.

The lead decided on 2026-10-14:
one test for every reading,
a Latin letter (accents included) or an ASCII digit,
because apostrophes belong to Latin-script prose and a mark beside a CJK character reads as loose,
which is the safe direction.
The corpus marks whose kind changes were to be measured and named.

Fixed by n133.
`continuesLatinWord` in `consistency-word-joints.ts` is the one neighbour test.
`consistency-proven-apostrophes.ts` (`markReadingsOf`,
`provenApostrophesOf`),
`consistency-loose-marks.ts`,
`consistency-closing-marks.ts` and `consistency-splice.ts` use it.
`isLetterOrDigitOfAnyScript` stays,
unchanged,
for `consistency-addresses.ts` and `corpus-run/pass-consistency-carry-crossing.ts`.
It has no direct test (B519).

Red evidence:
guard G4 (the any-script test back in the loose-mark reader) fails the case that leaves out a single mark with a Latin letter beside it.
Results on the corpus (`s191/corpus/replay-new.out` and `kinds.out`):
of 1,199 marks `' ’ ʼ ‘` exactly 1 changes kind,
`𝓠𝓾𝓪𝓷’s` of mikaela_khara at offset 4703,
whose mathematical script letters are not Latin to this test,
from attached to a proven leading mark.
The proven marks are 27 trailing and 2 leading.
No angle mark exists in the corpus,
and the 9 marks `‘` are not between letters.
The probe rows `H2` and `H3` of `h1-apostrophes` (`cats’猫`) now replace with the mark read as trailing (B520),
and `H1` (`猫’`) is now `unavailable`,
because a CJK character is not Latin to this test.
The decision on the mathematical letters is B518.

## B508: an arriving edge apostrophe was written beside a mark that the receiving place kept one markup character away

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the eighth review (n131,
finding J1),
the most severe finding of that review.
With the chosen place `dogs’` and the receiving place `The *kittens*’ bowl`
the pass wrote `The *dogs’*’ bowl is empty.`,
with the place recorded `replaced` and no finding
(`s188/probes/p01-doubled.out`,
row S1).
The same doubling appeared at planner level for `**kittens**’`,
`"kittens"’`,
`“kittens”’`,
`(kittens)’`,
`[kittens](url)’` and `~~kittens~~’`,
and a straight kept mark gave the mixed pair `*dogs’*'`.
The leading side has the mirror:
702 of 76,543 `replace` plans of the 200,000-pair grid gave a page with a single mark,
one or two markup or closing characters,
and a single mark that the receiving page had not held
(`He said '*5 cats` became `He said '*'Come (here)`).
The count is of the shape and not of defects.

What a reader of the page saw:
two apostrophes for one possessive,
one inside the emphasis and one after it.
The stray free mark also unproved the later gains of the same group:
two other possessives were left unrepaired with the finding "left out because the apostrophe it carries in would read as a quotation mark there".

Cause:
the counter records a mark only at the character directly outside the span,
so a mark one markup character farther was not recorded,
and the planner saw no mark on the receiving side and planned the gain case.
The gain check asks whether the carried mark is proven,
and a trailing mark is proven unless an opener stands before it,
which a kept mark after it never is.
The path was opened by the build of the gain case (B490):
the rows `E3` and `G3` of `h1-apostrophes` were `unavailable` at the seventh review and `replace` afterwards.

Fixed by n133.
`markPastRunOf` in `consistency-closing-marks.ts` reads the mark past a run of markup and closing characters
on the trailing side or of markup and opening characters on the leading side,
and a backtick is in both runs.
`consistency-count.ts` and `consistency-count-types.ts` carry the fields `markPastBefore` and `markPastAfter`
(not recorded in the artifact),
and `consistency-splice.ts` plans `unavailable` for a gain whose side holds such a mark.

Red evidence:
guard G10 (the backtick removed from both runs) fails two `markPastRunOf` cases,
and G11 (the two fields not copied into the rendering) fails the count test and two `applyConsistencyEdits` splice tests.
Results:
the shape count of `h2-grid-j1` fell from 702 to 0;
at planner level every markup row of `p16-beyond-markup` is `unavailable`,
the control "opener before" included,
and only "mark directly after" replaces;
the S1 page no longer holds `*dogs’*’`,
and its second place is `kept-current`,
which the reader accepts.
The rows `E3` and `G3` are `unavailable` again.

The review also asked where the rise of `replace` plans came from
(76,543 against 61,846 at the seventh review on the same seed).
n133 reproduced the review's counts on its baseline
(76,543 `replace`,
10,222 `same`,
113,235 `unavailable`),
and found 13,303 plans of the gain case:
12,269 with no kept mark on the gain side,
which is the gain case of B490 by design,
and 1,034 beside a kept mark at the start.
The page check keeps 6,416 and drops 6,887.
Those 13,303 are 90.5 percent of the rise of 14,697.
A residual of 1,394 is not attributed:
no build of the seventh review exists to bisect,
the seed spread is about 220,
and the entry set differs (51,408 located plans against 52,101).
The lead accepted this on the evidence that the oracle finds no defect among all `replace` plans
(`wordsOutside`,
`outsideMarks`,
`attachedFlips` and `doubleChanged` all 0).
On the new build the grid gives 74,098 `replace`,
10,181 `same` and 115,721 `unavailable`,
and 11,570 plans of the gain case (657 beside a kept mark).
The count `straightCurlyMixed` with the form thunk is 6,970 on the baseline and 6,522 now:
2,801 are receiving words that hold their own apostrophe form,
which wins by rule,
and 3,721 are pages with no shared form to follow.
None is a known page form left unapplied.
Without the thunk the count is 10,347 and then 9,866.

## B509: the page's form ignored apostrophes after closing markup

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the eighth review (finding J2),
which reopens B494 for the shape "after markup".
With the chosen place `dog's bowl` (straight) and a receiving place with no apostrophe
(`s188/probes/p11-forms.out`):
a page that writes only `*Mio*’s` (row F2) or only `[Mio](url)’s` (row F3) received a straight `dog's`,
a page that writes `Mio’s` (row F1) received the curly `dog’s`,
and a page with a curly mark between letters and a straight one after emphasis (row F6) received the curly form
and ignored the straight mark.

What a reader of the page saw:
a page that writes curly apostrophes everywhere got one straight apostrophe.

Cause:
the proof reads a mark after closing markup as attached,
so it is in no list of the proof,
and the form reader decided "attached" from the prose neighbours alone,
which had the markup blanked.
The header and the TSDoc of `consistency-apostrophe-forms.ts` said "the apostrophes of the page's prose",
which a reader takes to include them.

Fixed by n133.
`ProofView.joined` and `wordApostrophes` in `consistency-fold-apostrophes.ts` give the form reader the joined marks,
and `pageApostropheFormOf` in `consistency-apostrophe-forms.ts` counts them.

Red evidence:
guard G5 (the form reader not counting joined marks) fails cases in `consistency-apostrophe-forms.unit.test.ts` and `consistency-fold-apostrophes.unit.test.ts`,
and the new cases for `wordApostrophes` failed first (`s191/red-word-apostrophes.log`).
Results:
rows F2 and F3 now arrive curly,
and row F6 (a mixed page) keeps the chosen form,
because a mixed page offers no one form to follow.
The `wordApostrophes` reader was written after its tests were red-run but not strictly first (M187).

## B510: angle quotes, guillemets and the low and reversed single marks inside the words were removed or added by a replace

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the eighth review (finding J5).
At planner level (`s188/probes/p03-marks-carry.out`):
`She said ‹hello› to the kittens.` with the chosen words `She said hello to the cats.` gave `replace`,
and the page lost `‹` and `›` (row A1);
the reverse added them (row A2),
and `«hello»` was removed (row A3).
The rows with `‚hello‘`,
`‛hello’` and curly double marks were `unavailable`.

What a reader of the page would have seen:
a quotation mark removed or added by a replace.
The review rated it low,
because the guillemet note of `translate-guillemets.ts` says that no pinned original or archive carries one,
and the review did not recount that.
n133 found no angle mark in the corpus.

Cause:
the proof treats `‚ ‛ ‹ ›` and the backtick as partners wherever they stand,
and the reader of marks inside the words knew four single marks,
the double marks and the corner brackets only (`consistency-loose-marks.ts`).

Fixed by n133.
In `consistency-loose-marks.ts` the marks `‚ ‛ ‹ › « »` are loose marks,
so a carry that changes their count or order is `unavailable`,
as for the double marks.

Red evidence:
guard G6 (the angle marks removed from the reader) fails the case that counts the low,
reversed,
angle and guillemet marks.
Results:
rows A1,
A2 and A3 of the probe `p03` went from `replace` to `PLAN unavailable`.

## B511: a mark between letters had three readings, and comments said what the code no longer did

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the eighth review (finding J8 and item 2),
the marks side of J8.
The mark `‘` between letters (`the cat‘s bowl`) was read three ways:
the locator treats it as a joint and refuses the words,
the reader of marks inside words treats it as attached and not as an apostrophe,
and the proof treats it as an opener anywhere.
The header of `consistency-proven-apostrophes.ts` said "none of them is an apostrophe in any use",
which the header of `consistency-word-joints.ts` contradicted.
The review found no page consequence,
because the locator refuses the words and no carry is attempted.
Other comments the review listed:
the header of `pass-consistency-slices.ts` says the proof looks "anywhere on the page",
true of the English of the slices and silent about sealed spans (B514);
the header and the TSDoc of `consistency-apostrophe-forms.ts` omit the apostrophes that the proof reads as attached (B509);
the headers of `consistency-count.ts` and `consistency-splice.ts` say that a mark directly beside the span is kept
and say nothing of a mark one markup character beyond (B508).
The terms "stand",
"joint",
"gain case",
"floors",
"carried in part" and "settled" were each defined in some header and not where first used.
The review's own comment pass is partial:
it read the body of about 12 of the 37 files the hand-backs name and the header of 10 more,
and nothing of the rest.

The lead decided on 2026-10-14:
the loose-mark reader counts `‘` between letters as a quotation mark,
so a carry that changes it is refused,
the locator keeps refusing the words,
and both headers say so;
terms are defined where first used or pointed to by file.

Fixed by n133 in its files:
`consistency-loose-marks.ts` and `consistency-proven-apostrophes.ts` for the reading,
and the comments of `consistency-splice.ts`,
`consistency-count.ts`,
`consistency-word-joints.ts` (the terms "stand" and "joint"),
`consistency-fold.ts`,
`consistency-prose-text.ts`,
`consistency-proven-apostrophes.ts`,
`consistency-splice-readings.ts`,
`consistency-apostrophe-forms.ts` and `consistency-loose-marks.ts`,
with the gain case defined at its first use in `consistency-splice.ts`.
The comments of the files that n134 changes (the apply stage) are in progress there and have no entry yet.

Red evidence:
guard G7 (`‘` between letters read as attached) fails the case that counts the opening curly single mark between two letters as a quotation mark.
Results:
in the probe `p02` (`s191/probes/p02.diff`) `‘` between letters and `‚ ‛ ‹ ›` everywhere are now loose marks,
`αβ'γδ` and a digit on both sides of `‘` became loose (B507 and this entry),
and the locator still refuses `cat‘s`.

## B512: locating words beside a mark and reading the page's form cost the square of the page

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the eighth review (finding J6,
locate and form).
Locating words with a single mark beside them read and parsed the whole page once per place:
130 s at 2,000 slices.
The page-form reading did the same once per receiving place:
128 s at 2,000 slices.
With no mark beside the words the same count took 8 to 329 ms,
so the cost was the proof of the page per place.
The review's times are one run each,
partly beside other probes,
so a difference under about 15 percent is noise;
the spread on unchanged input was not measured.

What a reader of the page would have seen:
a page of 2,000 slices that waits over two minutes in each of two stages of the pass.
At corpus size it is not a problem:
the largest English page (33,451 characters,
cut into 54 pseudo-slices) took 335 ms in `applyConsistency`,
most of it floors,
and a group of 10 places with marks beside them costs about 0.2 s.

Fixed by n133.
`proofViewOf` in `consistency-fold-apostrophes.ts` is memoised (four views),
and `ProofView.wordApostrophes` (a list of `{ at, character }`,
the exported type `WordApostrophe`) is read once per page in `readProofView`,
built on the internal `readProof`.
`pageApostropheFormOf` filters that list,
and `markReadingsInEnglish` reads without the memo.
The memo alone gave 1,771 ms at 2,000 slices with a quadratic tail,
and the per-page list removed the tail.

Red evidence:
guard G8 (the memo bypassed) fails the case that reads a page once,
and G12 (the filter of `wordApostrophes` outside the unread stretches removed) fails two `pageApostropheFormOf` cases.
Results (`s191/probes/out/p12-locate-cost.out`,
one run per size):
at 125,
250,
500,
1,000 and 2,000 slices the count with a mark beside the words took 43,
33,
78,
154 and 345 ms,
and the plan with the form thunk took 47,
91,
160,
319 and 612 ms:
linear growth.
The memo and the reader change were written after their tests were first red-run (M187).
The cost of the apply stage is B516.

## B520: a plural possessive mark beside a CJK character reads as trailing

Accepted limit.
Uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n133 (flag e) as a consequence of B507.
In the probe `h1-apostrophes`,
rows `H2` and `H3` (`cats’猫`) now replace,
with the mark read as trailing and proven when no opener stands before it,
and `H1` (`猫’`) is `unavailable`.
The hand-back says the replacement removes or keeps the mark;
which of the two happens on a given page was not separately probed (unverified).
The lead accepted the limit on 2026-10-14.

What a reader of the page sees:
a mark that stands between a Latin word and a CJK character is treated as the closing mark of a quotation,
and not as the apostrophe of the word.
The words that arrive are right,
and the mark follows the rule for a trailing mark.

Corrected on 2026-10-14 (the original sentences stay).
Decision D3 of the independent oracle reversed this accepted limit:
the acceptance went against the text of K3 itself,
which says that a mark beside CJK reads as loose (M200),
and it rested on a hand-back that said the page outcome was not separately probed.
`cats’猫` now reads free.
The `’` stays outside the words as `markAfter` and the plan is `unavailable`;
in `h1-apostrophes` the rows H2 and H3 (`Fill the cats’猫 bowl.`) no longer replace,
where the baseline read the mark as trailing,
took it into the core,
and replaced it (`Fill the dogs猫 bowl.` and `Fill the dogs’猫 bowl.`).
Built by n143 (B537).

## B521: a word wrapped in backticks before a closing mark is not carried

Accepted limit.
Uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n133 (flag h) as a consequence of B508.
A backtick is now a character of the runs that `markPastRunOf` reads,
so a place whose words are wrapped in backticks and followed by a closing apostrophe
(the rows `G3` of `h1-apostrophes` and `S2` of `p01-doubled`) is `unavailable`.
The lead accepted this on 2026-10-14 as the safe side.

What a reader of the page sees:
a place wrapped in backticks with a closing mark after it keeps its own words.
The page is not wrong;
the place stays as the page wrote it.
