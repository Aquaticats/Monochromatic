# Audit ledger: the consistency pass, the page-wide proof, forms, floors and limits

The record of the seventh independent review of the page-level consistency pass and the fixes that answered it:
the proof that an edge mark is an apostrophe,
read over the whole page,
the joints after closing marks,
the form of an arriving apostrophe,
the floors that keep a writer from changing words that must stay,
and the limits the lead accepted.
The sixth review and the apostrophe round are filed in
[Audit ledger: the consistency pass, apostrophes and block floors](audit-ledger-consistency-pass-apostrophes-and-block-floors.md),
the fourth and fifth reviews in
[Audit ledger: the consistency pass, the span and the marks beside it](audit-ledger-consistency-pass-span-and-marks-beside-it.md),
and the build of the pass in
[Audit ledger: building the consistency pass, repeat groups and the carry](audit-ledger-consistency-pass-repeat-groups-and-the-carry.md).
Findings of those reviews that belong to a named family are filed under that family.

Written on 2026-10-14 (UTC) in the round that split `audit-ledger-tense-check-and-consistency-pass.md`.
The entries of the seventh review have a file of their own,
so that the file of the sixth review keeps room for its later corrections.
The rounds that built the whole-page reading and answered the eighth review are filed in
[Audit ledger: the consistency pass, stack depth, the floors fixed point and the page form](audit-ledger-consistency-pass-stack-depth-floors-fixed-point-and-page-form.md),
[Audit ledger: the consistency pass, one reading of every mark](audit-ledger-consistency-pass-one-reading-of-every-mark.md)
and
[Audit ledger: the consistency pass, the fixes of the eighth review and the oracle residue](audit-ledger-consistency-pass-fixes-of-the-eighth-review-and-the-oracle-residue.md).

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B492: a quotation mark that opened in another paragraph was taken for a proven apostrophe and deleted

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the seventh review (n124,
`s179/review.md`,
finding H1).
The review reran the findings of the sixth against the build of the apostrophe round.
G1,
G2,
G4 and the loose-mark half of G7 were fixed,
G3 was fixed for its original input with a residue (B496),
G5,
G6 and G8 stood as the accepted limits of B482,
B483 and B485,
G9 was unchanged,
and G10 had turned into this defect.
Every rate the review gives is over inputs it built,
and none is a corpus rate.

The rule of B488 read "the paragraph" as the text between blank lines,
and counted four single marks as possible partners.
The review's probe `h1-apostrophes.mjs` showed a quotation mark deleted
(its case labels B1 to B5,
F1,
N1 to N4 and D1 to D4 belong to the probe and are no ledger ids):

- B1:
  the page `‘Fill the bowls.` and,
  after a blank line,
  `Then feed the kittens’.` with the chosen words `the dogs`
  became `‘Fill the bowls.` and `Then feed the dogs.`
  The closing mark of a quotation opened in the first paragraph was gone.
- B2:
  `He said, 'Fill the bowls for the kittens.` and,
  after a blank line,
  `Then feed them.'`
  lost the opening `'` and kept a stray closing one.
- B3 is B1 inside a block quote,
  and F1 (`'Tis ... cats'` across paragraphs) lost both edge marks.
- N1 to N4:
  `said‘fill the kittens’` and the same with `‚`,
  `‛` and `‹` as the opener lost the closing `’`,
  because the opener was not one of the four marks the rule counted as a partner.
- Controls B4 (block quote lines with no blank `>` line) and B5 (list items) were `unavailable`.

What a reader of the page saw:
a quotation that is opened and never closed,
or closed and never opened.
The words were right,
the punctuation wrong,
and no finding was recorded.
The corpus count of B488 (no such case in 92 pages) said the case is rare and not that it is impossible (M181).
The oracle of the apostrophe work could not see it:
it took its idea of a proven mark from the same definition as production,
and its grids were single-line texts with plain words around the mark (B499).

Fixed by n126 (`s183/handback-n126.md`).
`provenApostrophesOf` (`consistency-proven-apostrophes.ts`) is one linear pass with running facts.
The backtick and `‘`,
`‚`,
`‛`,
`‹` and `›` count as an opener and a closer wherever they stand,
glued between letters included,
and `'`,
`’` and U+02BC count by kind.
A trailing mark is proven when no opener stands before it,
and a leading mark when no closer stands after it.
The proof reads the prose of the English (`proseTextOf` in `consistency-prose-text.ts`):
code,
JSX and HTML markup with its string attributes,
comments,
link destinations,
bare addresses and emphasis marks are blanked to spaces,
line breaks are kept and no offset moves,
and an English the grammar refuses is read whole.
Read as plain text instead,
six more trailing marks and one more leading mark of the pinned corpus drop out,
because a straight `'` inside a `<PhotoScroll photos={['…']} />` string counted as a partner.

Results:

- The probe's B1,
  B2,
  B3,
  F1 and N1 to N4 gave `replace` with a lost quotation mark before the fix and are `unavailable` after it;
  its controls still replace with no outside change
  (`s183/probes/h1-apostrophes.before.out` and `.after.out`).
- Fifteen hand fixtures for shapes the oracle cannot see
  (`s183/probes/hand-fixtures.after.out`):
  ten pages with a partner elsewhere
  (an opener in another paragraph,
  curly and straight,
  many paragraphs before,
  in a block quote,
  in a list item,
  in a heading,
  a prose backtick,
  a low single quotation mark,
  a glued opener,
  and a closer in a later paragraph)
  are refused and lose no quotation character;
  five controls
  (no partner,
  a partner only in inline code,
  in a link destination,
  in a component string,
  or only double quotes)
  carry the mark and lose exactly the proven apostrophe.
  No fixture is bad.
- Under the page rule 27 of the 28 trailing marks and 5 of the 5 leading marks of the pinned English stay proven
  (`s183/replay-why.out`,
  `s183/count-marks.out`).
  The one that drops is Kotori's `her friends’ excellent ideas`,
  with a prose backtick before it on the page.
  The production function agrees with an independent quadratic prototype on 368 views.
  The sentence of `design-commitments.md` that holds the counts of 2026-10-13 is left as it was.

Open when this was written:
the build read "the page" as the English of one slice,
the section handed to the locator,
so a quotation opening in another slice is unseen.
The lead decided on 2026-10-14 that the proof and the gain check read the whole page.
n128 builds it on the merged tree,
and that build is not in the code this entry was written from.
Until it lands,
a reader of the page sees the closing mark of a quotation that opened in another slice taken for an apostrophe.

Corrected on 2026-10-14 (the round that wrote B501 to B521):
the whole-page reading is built by n128 (`s185/handback-n128.md`).
`pageOfSlices` in `consistency-page-english.ts` joins the slices that carry English,
in page order,
with a blank line,
and blanks the YAML front matter at the start of the first slice with every offset kept.
The result,
`PageEnglish`,
is computed once in `consistencySlicesOf` (`corpus-run/pass-consistency-slices.ts`)
and reaches the counter (`countConsistency`),
the locator (`locateWords`) and the gain check
(`slicePlansOf` in `corpus-run/pass-consistency-apply-plan.ts`,
and `withoutUnprovenGains` over the page).
A quotation that opened in one slice and closed in another was invisible to the proof before this change.
Now the cross-slice case is refused and the page is unchanged
(`s188/probes/p04-pages.out`,
rows P1 and L1,
read by the eighth review);
the controls with other apostrophes only in code,
a link destination or mapping front matter are still replaced.
The new cases failed first on a build made before the code (`s185/red-page.log`,
28 failing lines).

Corpus counts of this change (92 pages,
1,329 slices,
`s185/replay-pagewide.stdout`):
the whole page gives 27 trailing and 5 leading proven marks,
the same marks as the whole-file unit of n126,
where each slice alone gave 28 and 5.
The one mark that changes is Kotori's `friends’` at offset 969.
Later rounds moved the leading count:
to 1 when an apostrophe after closing markup became attached (B504),
and to 2 when the letter test became one (B507).
The counts sentence of `design-commitments.md` is left as it was;
the lead rewrites it after the last code round.

What stays open:
the page the proof reads is the slices that carry English,
so text no slice holds is not read.
The eighth review showed that a sealed span holds the author's English with quotation marks,
and the fix is in progress (B514).

## B493: a stand was located inside a word when closing marks stood between it and the apostrophe

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the seventh review (finding H2).
The probes `h6-joint-examples.mjs` and `h6-straight-quote-s.mjs` located `cat` in each of
`Fill the "cat"'s bowl.`,
`Fill the “cat”’s bowl.`,
`Fill the (grey cat)'s bowl.`,
`Fill the [grey cat]'s bowl.` and
`[the cat](x)'s bowl` (with the words `the cat`),
and a plural chosen onto it gave `Fill the "cats"'s bowl.` and `Fill the “cats”’s bowl.`
The outside marks and the attached flags did not change (0 flips),
so this was a defect of words and not of marks.
What a reader of the page saw:
a plural word followed by a singular possessive `'s`.
`**cat**'s` and `*Yorushika*’s` were already no joint and stayed so.

The header of `consistency-word-joints.ts` said four things that were false.
A straight `"` before an apostrophe "is left to the refusal of a splice",
though no mark is recorded beside the words in that case and nothing refuses.
`"hi"` and `hi` were said to be none inside `“hi”’s`,
and the text of a link none inside `[the cat](x)'s`,
though both were found once.
The clauses for `”`,
`)` and `]` fired only when the stand itself ended in that character,
and the joint rule stepped back only over emphasis marks.
B489 had stated the closing-bracket half as built.

Fixed by n127 (`s184/handback-n127.md`).
`closingMarksThenApostrophe` (`consistency-closing-marks.ts`,
split out of `consistency-word-joints.ts` when it reached the line limit)
reads forward from the end of a stand.
It steps over closing marks:
an emphasis mark `*`,
`_` or `~` with a character that is no white space before it,
a straight `"` the same,
a closing `”`,
a closing `)` or `]`,
and the `(...)` destination right after a `]`.
An apostrophe that is no corner bracket,
followed by a Latin letter or an ASCII digit,
then makes the stand inside a word.
A link destination with white space in it (a title) or with no closing bracket is not read.
A curly opening mark and the corner brackets are no closing marks.
A stand that begins right after the apostrophe (`s bowl` in `"cat"'s bowl`) is still located,
because the backward reading does not step over a straight `"`,
which in `"'Tis"` opens a quotation.

The build is bounded:
it steps over at most 8 closing marks (`CLOSING_RUN_LIMIT`)
and reads at most 512 characters of one link destination (`LINK_DESTINATION_LIMIT`),
both in `consistency-closing-marks.ts` (lines 76 and 81 at the time of writing).
The pinned test of the linear reading allows 16 reads per character,
and a walk of any length for each position of a run of marks did not fit;
the author measured 13.99 reads per character on 4,000 emphasis marks,
and about 246 for a link opened and never closed.
Past the bound the defect returns.
The lead decided on 2026-10-14 to replace the bound with one right-to-left pass per text that marks every position
whose closing run ends in a joining apostrophe,
so that the check is linear with no cap.
It is built after the merge,
because it touches the locator that n126 also touches.
n128 builds it,
and the bounds still stand in the code this entry was written from.

Results:
in `h6-joint-examples` rows 3 to 11 and 15 were `once` and are `none`;
in `h6-straight-quote-s` three rows were replaced and all five are unlocated;
`g14-attached` is byte-identical to the review's file (24 of 24).
The new cases were red first (`s184/red-word-joints.log`),
and with both limits set to 1,000,000 three cases fail
(`s184/mutant-word-joints.log`:
the limits case,
the pinned linear case and the doubling case).
The merge of n127 into n126's tree copied 11 files,
merged 3 line by line (`pass-consistency-apply.ts`,
`pass-consistency-carry-units.ts` and `pass-consistency-types.ts`) and had 0 conflicts
(`s182/merge-report.json`).

A fact about the probes:
bare `node` resolves `monochromatic-lint` 0.1.1 on the path while the pin is 0.1.2,
and every carry then fails as `consistency-carry-failed`,
since the version check refuses as designed.
Probes and the launch go through `mise`,
and the launch step must be read for this.
The first before-runs of n127 under bare `node` were discarded.

Corrected on 2026-10-14 (the round that wrote B501 to B521):
the pass with no cap is built by n128 (`s185/handback-n128.md`).
`closingJointsOf` in `consistency-closing-marks.ts` replaces the bounded walk.
It is one right-to-left pass over a text that returns the set of positions
whose run of closing marks ends in an apostrophe before a letter or digit,
a `(...)` destination after a `]` being part of the run,
and it reads only `length`,
`charAt` and `codePointAt`.
`closingMarksThenApostrophe`,
`CLOSING_RUN_LIMIT` and `LINK_DESTINATION_LIMIT` no longer exist.
`cutsWord` in `consistency-word-joints.ts` takes the set as a required `joints` argument,
and `apartStartsOf` in `consistency-fold.ts` builds it once per text and only when the text has stands.
The defect no longer returns past a bound:
the new cases cut a stand behind 9 and behind 10,000 closing marks of `)`,
`"` and `*`,
and behind a link destination of 512 and of 100,000 characters.
The new cases failed first (`s185/red-joints.log`,
6 failing lines),
the pinned linear case and the doubling case of n127 are kept,
and every row of `consistency-word-joints.unit.test.ts` and `consistency-closing-marks.unit.test.ts` keeps its old expectation.
The eighth review measured the reading (`s188/probes/p07-closing.out`):
100,000 closing marks took 6.1 ms for `)`,
5.4 ms for `*`,
6.6 ms for `"` and 3.2 ms for chains of links,
and the ratios per doubling between 12,500 and 200,000 marks were between 1.1 and 2.4 with no superlinear trend.
n130 guard G1 (the `closesAfter(previous)` call in the apostrophe branch removed) fails the suites that read a joint (B504).
Two readings of the apostrophe in `**cat**'s` followed (B504),
and the letter test changed in B507.

## B494: arriving apostrophes kept the chosen place's form, so a page could mix straight and curly

Fix uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the seventh review (finding H3).
It reverses the limit of B488 ("carried as the chosen place wrote it"),
which the lead had accepted from the author's description with no probe (M180).
The probe `h1-apostrophes.mjs` showed four cases:

- D1:
  a straight chosen place onto a curly page gave `Fill the dogs' bowl.`
- D2:
  an interior apostrophe,
  `the kitten’s bowl` became `the dog's bowl`.
- D3:
  U+02BC arrived on a curly page.
- D4:
  the reverse of D1.

On the review's grid `straightCurlyMixed` counted 6,239 of 61,846 replaces
(inputs the reviewer built,
not a corpus rate).
What a reader of the page saw:
two apostrophe forms on one page.
Loose marks had been fixed in B484;
apostrophes took another path (`consistency-splice.ts` and `consistency-loose-marks.ts`).
The review did not check whether a later typography step repairs the mix,
and no source of this entry did.

Fixed by n126.
`consistency-apostrophe-forms.ts` (`pageApostropheFormOf`,
`inReceivingForm`) and,
in `consistency-loose-marks.ts`,
`apostropheMarksOf` and `withApostropheForm`.
An arriving apostrophe is written in the form the receiving words share when they share one,
otherwise in the form that the page's prose apostrophes outside the span share when they share one,
otherwise as the chosen place wrote it.
U+02BC is a form of its own.
`planConsistencyEdit` takes an optional `pageApostrophe` thunk,
and `corpus-run/pass-consistency-decide.ts` passes one.
The reader plans without the thunk,
and the plan kind never depends on it,
since words already equal after loose marks are `same` before any form is applied.
The reader check was answered by a case:
the `sharing` case of `corpus-run/pass-consistency-apostrophes.unit.test.ts` writes `the dogs’`
from a straight chosen place through the thunk and the section reads back at generation 17.

Results:
D1 to D4 now give `’`,
`’`,
`’` and `'`,
each in the page's form
(before:
straight,
straight,
U+02BC and a curly one mixed in).
A page whose own apostrophes mix forms keeps the chosen form (a unit case).
On the three grids of the apostrophe work an independent form check ran 58,457 to 87,395 times per grid with 0 violations
(`s183/probes/n122/oracle2.mjs`).
The review's crude metric rose to 10,291 with no thunk and 6,924 with it;
the metric also counts closing curly quotation marks as apostrophes,
and the rise is the gain plans that now replace (B490) and plans that arrive in the chosen form when no thunk is given.

## B495: the crossing-place floor covered only places that code could not move

Fix uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the seventh review (finding H4,
and its answer to the question of B491).
The probe `h4-crossing.ts` showed that the floor of B491 kept only receiving places whose plan is `unavailable`
(`corpus-run/pass-consistency-carry-crossing.ts` lines 18 to 23 and 173 to 184,
and `corpus-run/pass-consistency-carry-units.ts` lines 44 to 50):

- X1:
  a receiver that crosses the block edge with plan `unavailable`:
  the writer's paragraph is refused.
  X2,
  the control,
  passes.
- X3:
  a place kept for its context that crosses (`keep: ['OCCURRENCE 1']`):
  the writer wins,
  the page holds `to nap‘ said she.` where it held `to sleep‘`,
  and the place is recorded `kept-context` with no finding.
- X4:
  a chosen place that crosses:
  the writer wins,
  the page holds `to nap said she.` and the place is recorded `chosen`.

What a reader of the page saw:
the words of a chosen or kept place changed although the record said they were not.
That closes the open item of B491.

Fixed by n127.
`crossingOfUnit` takes `kept` and lists,
with the `unavailable` receivers,
the stretch of every place of the group whose words must stay:
the chosen place,
a place kept for its context,
and a place that already reads as the chosen place does.
`KeptWords` (`corpus-run/pass-consistency-carry-units.ts`) gets an optional `ofOtherGroup: true`
that `keptWordsOf` sets on the located words of other groups.
The existing finding wording and the existing keep list on the writer sheet are reused.
The other groups' places are not listed on purpose.
n127 read and then probed the apply stage (`corpus-run/pass-consistency-apply-located.ts`):
it skips the replacement's own group,
so X3 and X4 never reached it,
but another group's crossing place does
(`s184/probes/x-two-groups.ts`:
the place of the group that edits it ends `overlapped` and names the other group's place).

Results:
the page keeps `to sleep‘` (X3) and `to sleep` (X4),
with two findings
("the paragraph no longer holds,
at the edge of the block,
the words of a place that runs over it and must stay as they are"),
and the controls are structurally as before.
In X4 the places beside it stay `kept-current`,
because the splice for them is refused by the block-count floor
and the writer is withheld;
before,
the writer's paragraph had won them.
The new whole-pass cases (`corpus-run/pass-consistency-crossing-kept.unit.test.ts`) ran red against a build made before the code
(`s184/red-crossing.log`),
and with `kept: []` at the one call the whole-pass case fails
(`s184/mutant-crossing.log`).
The code was written before its tests (M182);
the mutation run is what shows the cases bind.

## B496: a splice the floors always refuse made the next paragraph unwritable

Fix uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the seventh review (finding H5),
the residue of B480.
The probe `h6-g3-residue.ts` rows 1 and 2:
a receiver with plan `replace` whose chosen words hold no blank line is refused by the floor
("The PAGE AS IT STANDS is 4 blocks ... your translation is 3"),
because a splice there can never pass.
The place of the next paragraph then received
"consistency-writers-not-asked (OCCURRENCE 2):
... overlaps the words of another receiving place ... outside it",
though no splice for the first place can exist.
Row 3,
with a `‘` mark,
was writable,
so the original input of B480 was fixed.
What a reader of the page saw:
a paragraph left unrepaired,
and a reason that names a splice that does not exist.
The header of `corpus-run/pass-consistency-carry-units.ts` (lines 37 to 47)
and `UnwritableReason` in `corpus-run/pass-consistency-types.ts` were false in this case.

Fixed by n126.
`consistency-block-breaks.ts` (`blockBreaksIn`) counts the top-level blocks of the words less one,
and the blank-line runs when the grammar refuses.
`planConsistencyEdit` gives `unavailable` when the receiving and the chosen words differ in block breaks.
Only marks are compared,
so records of generations 15 and 16 read as before.
After the fix rows 1 and 2 have the shape of row 3:
the outcomes are `kept-current`,
`replaced`,
`replaced` and `chosen`,
writers are offered,
and the reader accepts at generation 17
(`s183/probes/h6-g3-residue.final.out`).
The header text and the `UnwritableReason` text are made true.

## B497: the group that gives way in a shared block stays inconsistent

Accepted limit.
Written in the header of `corpus-run/pass-consistency-apply.ts` by n127;
uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the seventh review (finding H6).
The probes `h5-two-groups.ts` and `h5-two-detail.ts` ran six variants of two groups sharing one block;
the reader accepted all and there were 0 stops.
When one group's partial splice or writer block covers the paragraph,
the places of the other group are recorded `overlapped`,
which is the designed arbitration,
and the page keeps its wording for that group.
A caveat of the review:
its harness writer does not substitute the numbered label for single-receiver sheets,
so the rows where a writer won show only an incidental edit.
What a reader of the page sees:
one group rendered one way and the other still rendered several ways.
This is a cost and not a wrong page.
The lead accepted it on 2026-10-14 and had it documented.
The sentence in the header says that the group that gives way keeps its places' words as they were,
so the page stays inconsistent for that group,
and that this is the cost of settling the groups together over the English as it stood when the pass began.
Not tried:
two groups with a partial carry and a writer beyond the six variants,
and the timing of `overlappingInOneGroup`.

## B498: headers and TSDoc that were no longer true, or that cited paths a reader cannot open

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the seventh review (finding H7).

- `consistency-splice.ts` lines 40 to 69 said "the three tests" and "Why the third test" while four tests were listed,
  and its TSDoc at lines 472 to 485 said "All four false".
  n126 named and counted the four everywhere.
- `consistency-proven-apostrophes.ts` lines 36 to 38 cited scratch paths of one machine
  (a counts file,
  a hand-back and the scratch folder),
  which a reader of the code cannot open,
  so the figure of 0 cases in 92 pages was not checkable from the repository.
  n126 replaced them with the bullet "The consistency pass carries words and leaves every mark where it was"
  in `design-commitments.md`,
  and no file it wrote holds a scratch path (grepped).
- `consistency-word-joints.ts` lines 46 to 51:
  B493.
- `corpus-run/pass-consistency-carry-units.ts` lines 37 to 47 and `UnwritableReason`:
  B496.
  n127 also made the TSDoc of `CarryUnit.crossing` true,
  which said "no splice" where a crossing place can now be a chosen or kept one.

Two claims the review did not probe were probed by n126 (`s183/probes/claims.mjs`):
the header of `consistency-splice.ts` ("leaves every single mark beside the span attached or loose as it was")
held over ten contrived pairs of neighbouring marks on each side
(`’’`,
`''`,
`‘’`,
`’‘`,
`'’`,
`”’`,
`’”`,
`’'`,
`ʼʼ` and `’ʼ`):
50,016 replace plans of 150,000 pairs,
0 words changed outside,
0 outside marks changed,
0 attached flips,
and 0 flips among the gain plans that were kept.
The TSDoc of `looseMarksOf` ("a span never holds any other single mark at an edge") held over 4,494 single-mark edges
of every located rendering of that grid,
0 of them no edge apostrophe.
Both now say how far they hold:
the second for spans the locator makes,
and it names `edgeMarksContradictedBySpan` for recorded words.
The review's own probe `g9-random.mjs` had a counter,
`blockEdits`,
that was never incremented,
so its g9 run said nothing about edits to blocks;
that defect is the reviewer's and is in a scratch probe.

## B499: the property oracle could not see a mark wrongly proven and then deleted

Accepted limit,
with answers by hand fixtures.
Uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the seventh review (finding H8).
The oracle of the apostrophe round (`s178/probes/oracle.mjs` lines 48 to 54,
and `grid-run.mjs` line 43):

- shared the definition of a proven mark with production,
  so a mark wrongly proven and then deleted was invisible (B492);
- left proven marks out of the comparison of loose marks on both pages;
- ran on single-line texts with plain words before and after,
  so it held no distant mark,
  no paragraph break and no `’tis` neighbour;
- never compared the typography of edge marks (B494);
- ignored straight `'` quotation marks in `hardSeq`.

The review's own check (`h-oracle.mjs` with the grid `h2-grid.mjs`) covers distant marks,
paragraph breaks and `’tis` neighbours:
200,000 pairs gave 61,846 replace,
9,897 same and 128,257 unavailable,
with 0 words changed outside the span,
0 outside marks,
0 attached flips and 0 double marks changed.
It sees a deleted mark only inside the span and cannot tell an apostrophe from a quotation mark,
so B492 needed hand fixtures.

What was done.
n126 wrote a page-wide oracle that shares no code with the pass (`s183/probes/n122/oracle2.mjs`,
quadratic,
with the runner `grid-run2.mjs`),
a cross-check of the gain case on single edits,
a fixpoint cross-check over two edits in one page,
and an expected-form check;
on the three grids of the apostrophe work every violation count is 0
(`loose_b`,
`attachedFlip_c`,
`hard`,
`outside_a`,
`overDropped`,
`underDropped`,
`planKindChangedByThunk` and `form_e`),
and the pages of two edits were 40,792,
48,289 and 48,613 with 0 over-dropped and 0 under-dropped.
The fifteen fixtures of B492 answer role truth for the shapes they hold.
The hand-back states no planted violation that `oracle2.mjs` was shown to detect;
that control is unverified.
The accepted limit:
no property oracle tells an apostrophe from a quotation mark,
since that is a judgment about the English that the oracle can only derive again.
What a reader of the page could still see:
a quotation mark deleted or stranded on a page shape that no fixture holds.

## B500: records the pass never writes that the reader accepts

Accepted limit,
the family of B485.
Uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the seventh review (finding H9).
The probe `h5-forge.ts` forged records on the bases ONE_BLOCK and THREE_BLOCKS.
Its forgeries are numbered M1 to M14,
and those numbers are the probe's and no ledger ids.
The reader refused:
M1 (an `unavailable` place recorded `replaced`),
M4 (all receiving places `unavailable` and the splice kept),
M5 in ONE_BLOCK,
M8 (`overlapped` with no `gaveWayTo`),
M9,
M10,
M11,
M12 and M14,
and the readers of generations 15 and 16 refuse the new keys as designed.
It accepted:

- THREE_BLOCKS M7:
  a copy of another block's record under a place whose block was offered no splice,
  with the outcome `kept-current`.
  This is the weak point of B479 (blocks with byte-equal records count as one;
  `corpus-run/artifact-two-lane-read-consistency-span-shared.ts` lines 100 to 116),
  and the pass never writes it.
- THREE_BLOCKS M5:
  an `unavailable` place regains equal marks but stays `kept-current` after a block that offered no splice.
  The reader cannot know the floors of a block.
- M13:
  a quote ending in a letter-adjacent `’`,
  which is the limit of B485 and B488.
- M6 and M7 in ONE_BLOCK are no-ops,
  because `candidates` is a shared reference after `structuredClone`;
  that is an artifact of the probe and no result of the pass.

The honest record of a partly carried unit (B479) has the outcomes `kept-current`,
`replaced`,
`replaced` and `chosen`,
the finding `consistency-partly-carried`,
and the reader accepts it.
The fixes of the seventh review widened what the generation 17 reader accepts by one rule only:
`kept-current` for a place whose splice carries an edge apostrophe in (B490).
What a reader of the page sees:
nothing,
since a forged record changes no page;
what is lost is the check that a record is the one the pass wrote.
The lead stated it as a limit of the family of B485 on 2026-10-14.
