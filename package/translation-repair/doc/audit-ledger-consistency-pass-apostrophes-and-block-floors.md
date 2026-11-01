# Audit ledger: the consistency pass, apostrophes and block floors

The record of the sixth independent review of the page-level consistency pass
and of the apostrophe round that followed it:
which apostrophes are attached to a word,
the apostrophes that need no guess and travel with their word,
and the blocks a splice carries in part.
The build of the pass is filed in
[Audit ledger: building the consistency pass, repeat groups and the carry](audit-ledger-consistency-pass-repeat-groups-and-the-carry.md),
the fourth and fifth reviews in
[Audit ledger: the consistency pass, the span and the marks beside it](audit-ledger-consistency-pass-span-and-marks-beside-it.md),
and the seventh review in
[Audit ledger: the consistency pass, the page-wide proof, forms, floors and limits](audit-ledger-consistency-pass-page-wide-proof-forms-floors-and-limits.md).
Findings of those reviews that belong to a named family are filed under that family.

Split on 2026-10-14 (UTC) out of `audit-ledger-tense-check-and-consistency-pass.md`,
which had reached 994 lines and held the tense check and the consistency pass in one file.
Entries keep their ids and their text.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B478: a splice could change which apostrophes are attached

Found on 2026-10-13 by the sixth review (G1).
The fix is uncommitted at the time of writing.
Three places held a repeated line.
The judges' quotes were `the cats` (English `Fill the cats' bowl.`),
`the **cat**` (English `Fill the **cat**'s bowl.`) and `the dogs`.
The bench chose the first place,
and the page read `Fill the cats's bowl.`
The same happened with `*Tom*'s`.
The chosen place held `markAfter: '` with a space beyond the mark,
and the receiving place held `markAfter: '` with an `s` beyond it.
The two records were equal,
and nothing looked at the character beyond the mark.
The locator did not catch it,
since its joint rule reads only stands that end in a Latin letter or digit.
The reviewer's grids built Englishes from 14 word shapes.
Of 300,000 pairs biased to equal beside marks,
148,408 gave a `replace` plan and 7,472 of those (5 percent) changed the loose marks of the page.
Double quotation marks,
`‘` and corner brackets never changed.
The grids show that the shape is cheap to build and say nothing about how often a corpus holds it.

Decided by the lead:
fix it in the planner by comparing the edge class of the incoming text with the receiving core's
on any side that holds a beside mark,
and leave the generation 17 record as it is.
Built:
`keepsMarksAttached` in `planConsistencyEdit` (`consistency-splice.ts`).
It knows three classes:
a letter or digit of any script,
any other character,
and an empty edge.
On the three grids of the reviewer the fixed build gave 108,779,
100,917 and 67,319 `replace` plans,
and none changed the loose marks
(the reviewer's counts were 19,142 of 148,702,
7,472 of 148,408 and 5,080 of 99,556).
39,923,
47,491 and 32,237 plans moved from `replace` to `unavailable`.
The refusal is wider than the damage on purpose,
since it also refuses where the far side is no letter.
The reader recomputes the plan,
so it refuses the same splice candidate,
and its message names the cause (`noSpliceReasonFor`).
A reader of the page sees such a place keep its own wording or take a writer's.
Two false sentences were corrected:
the claim that the result pairs words and marks the way both places already did,
in the headers of `consistency-splice.ts` and `consistency-loose-marks.ts`.

## B479: one place that code could not move withheld the splice from every other place of its block

Found on 2026-10-13 by the sixth review (G2).
The fix is uncommitted at the time of writing.
The block carry offered a splice only when every receiving place of the block had a `replace` plan
(`edits.length < receivers.length` in `corpus-run/pass-consistency-carry-offers.ts`).
The mark rule made `unavailable` common,
so one plural possessive in a table row,
or in any block no writer is asked about,
kept the other places of the block at their old wording although each had a valid splice.
In the reviewer's table row the first cell was `unavailable` and the second was `kept-current`,
and with no apostrophe both were `replaced`.
The recorded finding said only that the words stand in a table.

Decided by the lead and built in a second round:
the spliced candidate is built from the places whose plan is `replace`,
a place whose plan is `unavailable` keeps its words,
and a new finding kind `consistency-partly-carried` says so.
The first round stopped,
since the generation 17 reader refused a splice candidate under an `unavailable` place.
A spike over the reviewer's 32 runs had the reader refuse the same eight as the unchanged build.
Accepted deviation from the lead's rule that such a place has no splice candidate of its own:
the bench votes once per block,
and every place of the block shares one record of candidates and ballots (`choose` in `corpus-run/pass-consistency-carry-choose.ts`),
so leaving the splice out of one place would make that place's ballots name a label its list lacks.
From generation 17 the reader accepts a splice candidate under an `unavailable` place
only when a place recorded under the same record has a `replace` plan
(`spliceBuiltForSomePlace` in `corpus-run/artifact-two-lane-read-consistency-span-shared.ts`),
and that place must be `kept-current` after a splice choice.
Generations 15 and 16 keep the old rule.
Weak point:
two blocks with byte-equal bench records would count as one,
which cannot happen for blocks of different text and could matter only for a forged file.
On the final build the reviewer's 32 runs gave 32 ok,
the 176 container runs gave 176 ok,
and 300 random pages with an apostrophe after about half the places gave 14 partly carried pages,
no stop and no reader refusal.
A reader of the page sees two cells of one row read two ways,
with the finding naming the place that kept its words.
Not done:
the cache version did not move,
so cached answers keyed by the old sheet are not reused for a partly carried block;
no case covers a floor refusing a partly carried block.

## B480 to B487: the rest of the sixth review

Found on 2026-10-13 by the sixth review (G3 to G10,
and the replay).
Fixes are uncommitted at the time of writing.

- B480,
  a writable paragraph was made unwritable by places that would never be spliced (G3).
  A place that crossed into the second paragraph and ended at an apostrophe was `unavailable`,
  yet it stopped the writers from being asked about a place inside that paragraph.
  The hand-back of the nesting fix gave a reason for not narrowing the rule:
  that it needs the bench result first.
  That reason is false for a place whose plan is `unavailable`,
  since the plan is known before any writer is asked.
  Built:
  `overlapsOutsideReceiver` (`corpus-run/pass-consistency-carry-units.ts`) ignores an outside place whose plan is not `replace`.
  Only the writer's chance to repair grammar around carried words was lost,
  so this was quality and not correctness.
- B481,
  a quote could be located inside a web or mail address (G4).
  `cats` in `Visit https://cats.example.org/page now.` was located,
  and a carry of `dogs` rewrote the address.
  The branch head did the same.
  Built:
  `consistency-addresses.ts`,
  used by the locator.
  A run of non-space characters is an address when it holds `://`,
  starts with `www.`,
  or holds `@` with a letter or digit on both sides and a `.` later in the run.
  A Markdown destination is caught by the same rule and the link text is still located.
  Accepted:
  a relative destination such as `(page.html)` is not an address and is still located.
- B482,
  `1990` is no stand inside `1990's` (G5).
  The joint rule cannot tell a decade from a height such as `5'2`.
  A frame whose slot is a bare year written `1980's` and `1990's` gets no slot in either place,
  so the carry is `unavailable`,
  where `1980s` onto `1990s` is `same`.
  Accepted and written in the header of `consistency-word-joints.ts`.
  A slot the judges quote whole as `1980's` is found and the plan is `same`;
  what judges quote in practice was not measured.
  A reader of the page sees both places keep their wording.
- B483,
  the slot inside a span gets no evidence from the judges' marks (G6).
  A slot that stands twice in its span is `many`,
  so the place has no slot and no code splice.
  It is never a wrong `once`.
  Accepted:
  evidence for the slot needs the slot's marks in the record,
  and the reader recounts the slot with no English.
- B484,
  straight and curly single quotation marks refused a splice (G7).
  A chosen place with `‘run’` could not be carried onto a place typed `'go'`,
  and a straight double mark could be written into a curly page.
  Decided by the lead:
  on `replace`,
  each loose mark of the incoming text is written in the receiving place's form,
  position by position.
  Built:
  `withLooseMarksOf` (`consistency-loose-marks.ts`),
  used only where marks are compared.
  `‘` still equals only itself.
  Words that differ only in the form of their loose marks are now `same`.
- B485,
  no reader can check recorded marks against English it does not hold (G8).
  The reviewer changed a recorded `markAfter` and the generation 17 reader accepted it
  while the outcome stayed what the recount expects.
  Accepted and stated in the header of `corpus-run/artifact-two-lane-read-consistency-span-located.ts`.
  A forged mark changes no page,
  since the record is an assertion.
- B486,
  smaller points (G10).
  The locator reads Latin letters and ASCII digits and the planner reads any letter or digit,
  so `猫'cats` is no joint for the locator and its apostrophe is not loose for the planner.
  The effect found was a refusal.
  Both headers now name their own set and the one shape where they differ.
  Equal words with different marks go through the writer and choice benches (`page-consistency-writers round: 5/5 heard`).
  Accepted:
  the rule treats them as not the same rendering,
  and the cost is model calls for words already equal.
  The single-mark file kept only the bracket test and was renamed `consistency-corner-brackets.ts`.
  Open and older than this work:
  `blockStandingOf` throws `RangeError` on 80,000 inline siblings after 16 s;
  the lead owns it for after launch.
  Open,
  cause not proven:
  `logger internal error: sink verification failed for entry 3: Timed out after 5000ms`
  is printed by `wording-key.unit.test.ts`,
  `corpus-run/dropped-destinations.unit.test.ts`
  and `corpus-run/pass-consistency-apply.unit.test.ts`.
  Running each of the 1,275 unit test files alone,
  twice,
  printed it from those three files only,
  and each of them holds a case that runs for 6.4,
  5.4 or 26.8 s.
  No test fails.
- B487,
  the replay never said what the reader read (the replay section of the sixth review).
  `verify-published` printed `matched=1` and a length check but not whether the artifact held a consistency section with groups.
  Built:
  `consistencyColumnOf` (`corpus-run/verify-published-entry.ts`) prints `consistency-groups=N read-at-generation=G`,
  and the replay of the stored run reads `consistency-groups=2 read-at-generation=15`.
  No end-to-end case covers an entry with groups other than 0;
  only `consistencyColumnOf` is tested for 1 and 12.

## B488: an edge apostrophe that needs no guess travels with its word

The owner asked on 2026-10-13 whether the apostrophe cases could be fixed deterministically.
The lead decided to do so wherever no guess is needed.
The fix is uncommitted at the time of writing.
A count came first.
The 92 English pages of the pinned corpus hold 1,199 single marks in prose.
The 28 trailing marks with no possible partner are all plural possessives after `s`,
and the 5 trailing marks with a partner all close a single quotation
(counts by class:
`design-commitments.md`,
"The consistency pass carries words and leaves every mark where it was").
The rule:
an edge single mark with no possible partner in its paragraph is a proven apostrophe.
It belongs to its word,
sits inside the span and travels with it.
Every other edge mark keeps the outside-span record and the refusal.
Built:
`provenApostrophesOf` (`consistency-proven-apostrophes.ts`),
the widening in `consistency-fold-apostrophes.ts`,
and the edge-apostrophe tests in the planner.
Carrying `the parents’` onto `the folks’` is `replace`.
Carrying `the parent` onto `the parents’ preference` is `replace`,
and the apostrophe leaves with the word.
A quotation closer still refuses.
Checked by the author of the build:
the production function over the 92 pages finds 28 trailing and 5 leading proven marks,
the same as the count;
three grids of the sixth review with an oracle written for this rule found no violation;
1,000 whole passes threw nothing and no reader refused a section.
Accepted limits:
the generation 17 reader holds no English and cannot verify that a recorded edge apostrophe was proven,
and a forged one changes no page;
a proven apostrophe is carried as the chosen place wrote it,
so straight is not made curly (as in `don't`),
and one page can mix the two.
The paragraph in the rule and this limit were both changed on 2026-10-14;
the note headed "Corrected on 2026-10-14" gives the reasons.

Corrected on 2026-10-14,
after the seventh review (B492 and B494).
Two statements of this entry did not hold.
The rule read "no possible partner in its paragraph".
The proof now reads the prose of the whole page,
and the possible partners are every single quotation character:
`‘`,
`‚`,
`‛`,
`‹`,
`›` and the backtick count as an opener and a closer wherever they stand,
and `'`,
`’` and U+02BC count by kind
(attached marks are inert,
a trailing mark is a closer,
a leading mark an opener,
a free mark both).
A trailing mark is proven when no possible opener stands before it in the page's prose,
and a leading mark when no possible closer stands after it.
The paragraph reading deleted a closing quotation mark whose opener stood in another paragraph (B492).
The corpus counts of this entry were read by paragraph;
the counts under the page rule are in B492.
The second statement is the accepted limit:
the lead accepted it from the author's description with no probe (M180),
and the seventh review measured 6,239 replaces out of 61,846 on its grid that put the two forms on one page
(a count over inputs the reviewer built,
not a corpus rate).
An arriving apostrophe now takes the form of the receiving place (B494).

## B489 to B491: the rest of the apostrophe round

Fixes and decisions are uncommitted at the time of writing.

- B489,
  an apostrophe after closing emphasis or a closing bracket and before a letter joins the word.
  No stand sits inside `**cat**'s`,
  so such a place is unlocated and keeps its words.
  `*'Tis the season*`,
  `('em)` and `"'Tis"` are not joints.
  The corpus holds four such marks,
  all `’` before `s`.
  Corrected on 2026-10-14:
  the build read back only over emphasis marks,
  so the closing-bracket half of this sentence did not hold
  (`(grey cat)'s` and `[the cat](x)'s` were located;
  B493).
- B490,
  the gain case was left refused here and is built (see the note headed "Corrected on 2026-10-14" in this entry).
  Carrying `the parents’` onto `the family` (no mark) was `unavailable`.
  Whether the carried mark stays proven depends on the receiving paragraph's other marks;
  where the paragraph holds a leading `‘` before the place,
  the carried mark is no longer proven and becomes a loose mark.
  Deciding it needed a field the record does not hold.
  The seventh review was asked whether the pass can decide it from text it already holds.
  A reader of the page then saw the plural possessive stay as it was.

  Corrected on 2026-10-14,
  after the seventh review answered that question.
  The pass can decide it from text it already holds,
  though not place by place:
  a leading gain `’round the family` and a trailing gain `the kittens’` in one paragraph are each proven alone
  and both unproven together (the review's `h3-gain.mjs`).
  The decision therefore runs on the slice with every edit of the pass applied.
  Built by n126 (`s183/handback-n126.md`):
  `planConsistencyEdit` (`consistency-splice.ts`) no longer refuses a splice because the chosen words carry an edge apostrophe
  that the receiving words lack,
  and `bringsEdgeApostrophe` names such plans,
  which the pass must check on the page.
  `withoutUnprovenGains` (`corpus-run/pass-consistency-apply-gains.ts`) applies the edits,
  reads the proven apostrophes of the resulting page,
  drops every code splice whose carried edge apostrophe is not proven there (all unproven ones in one round),
  and repeats until none drops.
  Dropping an edit can only make the remaining ones more proven,
  so the loop converges.
  A dropped edit leaves the place's words,
  records the outcome `kept-current`,
  and puts this finding on the slice:
  "slice N:
  the consistency replacement of group G OCCURRENCE K was left out because the apostrophe it carries in would read as a quotation mark there,
  since another quotation mark of the slice could pair with it;
  the place keeps its words".
  No record field was added.
  The generation 17 reader allows `kept-current` for a place whose plan is `replace`,
  whose splice carries an edge apostrophe in and whose carry chose a splice
  (`CarriedPlace.carriesApostrophe` in `corpus-run/artifact-two-lane-read-consistency-span-shared.ts`).
  It still cannot check that the mark was proven on the page,
  since it holds no English (the limit of B485 and of B488).
  Generation 17 is unreleased,
  so this is a design change and not a compatibility break.
  Checks by the author:
  two gains proven alone and neither together are both dropped (a unit case);
  over the three grids of the apostrophe work (n122),
  with a page-wide oracle that shares no code with the pass,
  the gain plans were 50,922,
  61,862 and 41,374,
  of which 3,600,
  565 and 374 were dropped,
  with 0 over-dropped and 0 under-dropped,
  and 40,792,
  48,289 and 48,613 pages of two edits also showed 0 of either.
  A reader of the page sees the carried apostrophe arrive where the page proves it one,
  and the place keep its words,
  with the finding,
  where another quotation mark of the slice could pair with it.
  Fix uncommitted at the time of writing;
  commit:
  pending.
- B491,
  a writer's paragraph is withheld when it no longer holds,
  at the block's edge,
  the words of an `unavailable` place that runs over that edge.
  Built:
  `corpus-run/pass-consistency-carry-crossing.ts`,
  the finding `consistency-candidate-refused`,
  and a line in the writer sheet's list of words to keep.
  Before it a writer could rewrite the half of a crossing place that lay in its paragraph
  while the record said the place kept its words (B479).
  Open when this was written:
  a chosen place,
  kept context and another group's words that cross a block edge have the same gap.
  Closed on 2026-10-14 by B495:
  the group's own chosen and kept places are listed,
  and another group's places are covered where the edits are applied.
  The cache version did not move,
  since the writers' key hashes the sheet text.
