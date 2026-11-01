# Audit ledger: the consistency pass, the span and the marks beside it

The record of the fourth and fifth independent reviews of the page-level consistency pass:
what the located span covers,
which single marks beside it travel with it or refuse a splice,
and the joint rules that keep a stand from sitting inside a word.
The build of the pass is filed in
[Audit ledger: building the consistency pass, repeat groups and the carry](audit-ledger-consistency-pass-repeat-groups-and-the-carry.md),
the sixth review and the apostrophe round in
[Audit ledger: the consistency pass, apostrophes and block floors](audit-ledger-consistency-pass-apostrophes-and-block-floors.md),
and the seventh review in
[Audit ledger: the consistency pass, the page-wide proof, forms, floors and limits](audit-ledger-consistency-pass-page-wide-proof-forms-floors-and-limits.md).
Findings of those reviews that belong to a named family are filed under that family.

Split on 2026-10-14 (UTC) out of `audit-ledger-tense-check-and-consistency-pass.md`,
which had reached 994 lines and held the tense check and the consistency pass in one file.
Entries keep their ids and their text.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B463: the located span widened over a single mark that was guessed to be an apostrophe

Fix uncommitted at the time of writing;
commit:
pending.
The code of B463 to B491 is in the branch's working tree and in no commit.

Found on 2026-10-13 by the fourth independent review of the consistency pass
(its findings 1,
2,
3,
6,
7,
8 and 4b),
and reopened the same day by the fifth review (its findings F1 and F2).
The fix for the third review's quotation marks made the locator widen a located span
over a single mark it judged to be the apostrophe of the word at the span's edge
(`'Tis`,
`the cats'`,
`goin'`).
The judgment was a guess from the shape of nearby words.
The fourth review showed the guess deleting or stranding quotation marks:
`ol'` or `'n'` after a closed quotation,
a straight mark before a digit or a listed word,
an opener followed by a bracket,
a dash or emphasis,
and a mark inside a code span.
Over 1,020 generated texts it gave 194 wrong spans (`s160/b-gen.mjs`).
It also made two places that held the same words differ
when one stood before a possessive apostrophe,
so the splice deleted or added that apostrophe.

Decided by the lead on 2026-10-13:
the span is the words alone,
and no single mark beside them is ever inside it.
A guess that fails deletes a quotation mark from a published page,
and a span that is the words alone needs no guess.
`consistency-fold.ts` no longer reads what role a mark plays.

Withdrawn on 2026-10-13 after the fifth review.
The lead's note had said the only cost was an apostrophe left beside a replacement at a dropped-letter opening,
and had sized it by the one dropped opening in the pinned English.
That was wrong twice.
Every shape that puts a single mark at the edge of the words reaches the same code:
the header of the single-mark reader counted 33 plural possessives against that one opening
(the file is now `consistency-corner-brackets.ts` and keeps only the bracket test),
and years and `'em` reach it too.
And the mirror was missed:
the chosen words lose their own edge apostrophe when they are carried.
The fifth review ran four shapes (`s166/probes/p3-apostrophe.mjs`,
`p20-year.mjs`,
`p19-floor.ts`):

- chosen `He was goin' home.` onto `He was going home.` gave `He was goin home.`
- chosen `The cats' bowl is full.` onto `The cat's bowl is full.` gave `The cats bowl is full.`
- chosen `The cat` onto `The cats' bowl is full.` gave `The cat' bowl is full.`
- chosen `1990s cats` onto `The ’90s cats nap.` gave `The ’1990s cats nap.`

The first three went through `applyConsistency` and ended `replaced` with no finding.

The rule that replaced the cost,
decided by the lead on 2026-10-13 after the fifth review:
the span stays the words.
The single marks that touch each edge of the span
(no space between mark and word;
`'`,
U+2019 and U+02BC)
are read at the chosen place and at the receiving place,
and a splice whose two places differ in a touching mark at an edge is unavailable,
since the replacement would write a pairing of words and mark that neither place wrote.
The writers' blocks stay the way such a place is carried.
A judge's edge marks choose between stands of the same words and are never span content.
Every single mark at a needle's edge is dropped from the needle,
paired or not.
B470,
B471 and B473 are the build of this rule.
What a reader of the page sees:
a place whose edge mark differs from the chosen place's keeps its own wording,
where the first rule gained or lost an apostrophe.

Evidence for the first rule:
five cases red on the old build (`s159/red-fold.log`)
and gate g91 green with 2,235 passes.

## B464 to B469: the rest of the fourth review

Decisions of the lead on 2026-10-13;
fixes uncommitted at the time of writing;
commit:
pending.

- B464,
  linear reading (finding 5):
  a quote made of emphasis marks stood at every mark of a long run and walked the run from each,
  12.6 s at 40,000 characters.
  `cutsWord` (`consistency-word-joints.ts`) now reads past emphasis only from an edge that can be joined
  (a letter or a digit);
  every row of `s159/probe-perf.log` is linear,
  6 to 18 ms at 40,000.
  No timing case was added,
  since the package keeps real-clock bounds out of tests;
  B475 records that the fifth review found the fix unpinned.
- B465,
  generation 16 artifacts (finding 4):
  no pass wrote one,
  since TianqiChen66623 wrote generation 15 and TianqiChen66622 wrote 14
  (`rg` over both run directories on 2026-10-13)
  and no pass ran between generation 16 landing and generation 17.
  The reviewer's two triggers
  (a ballot recount that differs by an edge apostrophe,
  a slot beside a hyphen variant)
  touch no file on disk.
  Decided:
  no locator gated by generation.
  The contract's comment for generation 16 (`corpus-run/artifact-two-lane-contract.ts`)
  says it was written by no pass and is read with the current locator.
- B466,
  cache version:
  `CONSISTENCY_CACHE_VERSION` stays 4.
  The reviewer traced the path:
  the key hashes the whole sheet (`corpus-run/pass-consistency-cache.ts`,
  `consistencyKey`),
  so changed sheet text is a new key,
  and the lane's namespace takes `pipelineDigest`,
  which any built change moves.
- B467,
  the post-pass and the name a place gave way to (findings 10 and 11):
  the post-pass on the splice arbitration was removed,
  and the verdict by exhaustion is in the hand-back at `s163/handback-n111.md`.
  The fifth review ran 119,000 random arrangements against the build that still had the post-pass
  (`s163/dist-with`) and found no difference in groups,
  outcomes,
  `gaveWayTo`,
  slices or findings (`s166/probes/p11-diff.ts`).
  Finding 11,
  decided by the lead:
  the reader accepts any located,
  replaced or refused place of another group in the same slice as the place given way to,
  two left-out places naming each other included.
  The record holds no span to check a name against,
  and the writer can produce the mutual case,
  so refusing it would make a record the writer writes unreadable.
  Three other choices stand:
  a reference to a place's own group is refused,
  the record module and the apply barrel stay,
  and the finding wording stays unless the removal took that finding away.
- B468,
  standing (finding 9):
  inline raw HTML nodes are read in the plain-Markdown fallback too,
  and a table or preformatted tag written with capitals is read as that tag in both parses
  (`consistency-carry-standing.ts`).
  Both choices were flagged and accepted by the lead.
  Measured on 2026-10-13 for the capitals:
  `scripts/mdx.ts` of the pinned corpus compiles pages with no component named for a table or preformatted tag,
  and none of the 276 Markdown pages under `people/` holds such a tag in any letter case
  (`rg --count-matches --ignore-case` over the tree;
  the same search without the tag pattern reaches all 276).
  The choice changes no corpus page today.
  A paragraph holding such a tag is read as a table,
  so no writer is asked about it and only a splice carries it.
- B469,
  tests (finding 12):
  the two assertions that could not fail (they sliced constants) are gone,
  and the replacement cases build the replaced page from the offsets the locator returns
  (`replacedIn` in `consistency-fold.unit.test.ts`).
  A property case builds 3,240 Englishes from every combination of the words,
  marks and neighbours it lists
  and holds the span to exactly the words in each.
  Mutants on the splice arbitration:
  24 of 26 killed at first,
  and the two survivors are killed by the cases in the hand-back at `s162/handback-n110.md`.

## B470 to B477: the fifth independent review, finding by finding

Found on 2026-10-13 by the fifth review (findings F1 to F9).
F1 and F2 are filed under B463.
The fixes in these entries are uncommitted at the time of writing.

- B470,
  the edge reading's guess still decided what was inside the span (F3).
  The English `He said ‘the cats’ bowl is full.’` with the quote `‘the cats’ bowl`
  paired the opening mark with the plural possessive after `cats`.
  The mark stayed in the needle,
  and a replacement gave `He said The kittens’ bowls is full.’`.
  The branch head gave the same span,
  so this was no regression,
  but it broke the reason B463 gave for accepting its cost.
  Built:
  `consistency-edge-marks.ts` takes any single mark off either edge of a needle.
  The pairing reader and its seven cases are gone.
  The sixth review reran the input and got the span `the cats’ bowl` with the opening mark outside it.
- B471,
  a mark inside the quote that pairs with a mark outside it (F4).
  The chosen `He said, "So she was kind," she said.` carried onto `Then "She was kind," she said.`
  gave `Then ""So she was kind," she said.`,
  and the converse deleted a mark.
  The floors accepted both.
  Built:
  the planner refuses a splice when the loose marks inside the two sets of words differ in sequence
  (`looseMarksOf` in `consistency-loose-marks.ts`,
  `planConsistencyEdit` in `consistency-splice.ts`).
  The locator still takes an interior mark into the span,
  by design,
  and the planner is the guard.
  Before the build the planner's property case found 3,232 of 5,184 pairs changing the receiving place's loose marks.
- B472,
  two block replacements of one group could nest (F5).
  `blockStandingOf` (`consistency-carry-standing.ts`) gave a writable block to a span that crosses a paragraph boundary
  inside a block quote or list,
  and another place of the group in a paragraph of the same container gave a block inside it.
  `applyConsistency` then recorded the later place `overlapped` and named its own group,
  which the generation 17 reader refuses,
  so the next resume would have refused the artifact the pass had just written.
  The reviewer ran this with synthetic edits and not through a whole pass.
  Built in three rounds.
  First,
  a span that no single paragraph or heading holds gets no block when the top-level block is a container,
  and `sliceEditsOf` (`corpus-run/pass-consistency-apply-record.ts`) throws `ConsistencyInvariantError`
  before any outcome is recorded when two replacements of a group overlap.
  The throw was still reachable.
  A place that crosses a paragraph boundary is carried by a splice,
  and a place of the group inside the second paragraph was carried by a block over that paragraph.
  Real edits from `carryAll` threw in `applyConsistency`,
  and the scripted pass stopped with the page keeping its wording.
  Second,
  `unitsOf` (`corpus-run/pass-consistency-carry-units.ts`) makes a block unit unwritable
  when a receiving place of the group outside it overlaps its block.
  The counter already keeps two located places of one slice from overlapping (`unlessOverlapping` in `consistency-count.ts`).
  Third,
  the reason a unit is unwritable is recorded in `CarryUnit.unwritableBecause`
  (`no-single-block` or `overlaps-outside-receiver`) and no longer inferred from the extent of its block.
  The sixth review ran 3,576 scripted passes over hand-shaped and random pages.
  The invariant never fired and no section was refused,
  but its detector had no positive control at pass level.
  At the arbitration level the throw fires as designed.
  A reader of a page sees such a place carried by splice or kept as it was,
  with the finding `consistency-writers-not-asked` saying why no writer was asked.
- B473,
  places that the branch head located once became ambiguous,
  and the record of marks (F6).
  The English `Fill the cats' bowl. Then the cats nap.` with the quote `the cats'`
  was `once` at the branch head and `many` under the span rule.
  A place's English is the whole slice,
  so two spellings of the words in one slice are not rare.
  Built in three parts.
  The locator takes the single marks the judges copied beside the words and uses them only when the words stand more than once.
  `markEvidenceOf` (`consistency-majority.ts`) says a side holds a mark when more than half of all usable ballots copied it,
  and a ballot that copied no mark or quoted other words stays in the count.
  Generation 17 records `markBefore` and `markAfter` on a located place (`corpus-run/artifact-two-lane-consistency.ts`).
  The planner compares them,
  and generations 15 and 16 read as before.
  The pass wrote no marks at first,
  so two places with equal words and different marks were `unavailable` for the pass and `same` for the reader,
  and the pass would have written a section its next resume refuses.
  `recordedRendering` (`corpus-run/pass-consistency-apply-record.ts`) now stores them,
  and a round-trip case runs the pass and the generation 17 reader.
  The sixth review ran the majorities:
  three of three,
  two of three and three of four ballots locate the place;
  one of three,
  two of four,
  and two of three with one more ballot that omits the place stay ambiguous.
  The brief for this part gave an example that the reader accepts without the fix (M167).
  No reader holds a place's English,
  so recorded marks are checked for shape only (B485).
- B474,
  quote characters outside the four single marks stay in the span (F7).
  U+02BC was named in the rule and stayed in the span;
  it is fixed.
  The backtick,
  U+00B4,
  U+2032 and the low and angle quotation marks still stay inside a span at a quote's edge.
  Accepted:
  the rule names four marks.
  Two judges who copy the same words with and without one of these marks name different spans,
  and a replacement replaces the mark along with the span.
- B475,
  no test pinned the linear reading,
  and two emphasis paths survived mutation (F8).
  A copy of `cutsWord` without both `canBeJoined` guards agreed with the real one on all 29 input pairs of its test
  and took 6,479 ms against 4 ms on 40,000 emphasis marks.
  Two mutants of the emphasis reading differed on none of 33 suite inputs.
  Built:
  a case that counts reads through `charAt` and `codePointAt` instead of timing,
  allows 16 reads per character on 4,000 emphasis marks,
  and measured 10.99,
  flat from 1,000 to 8,000 marks;
  a positive control that `a **so**-called nap` reads more than `a so-called nap`;
  and two inputs that kill the mutants.
  The sixth review ran eleven mutants of `cutsWord`,
  and each fails a case of the right kind.
- B476,
  a hyphen or separator beside a digit was no joint (F9).
  `22` was found inside `Catch-22`,
  `10` inside `2026-10-13`,
  `30` inside `10:30` and `cat` inside `cat-5`.
  The rule now in `consistency-word-joints.ts`:
  a hyphen (ASCII,
  U+2010,
  U+2011 or the soft hyphen) joins when each side is a letter or a digit;
  an apostrophe (straight or U+02BC,
  never a corner bracket) joins a letter or digit to a letter or digit;
  `.` and `,` need a digit on both sides,
  and `:` and `/` join the same way;
  a dash between clauses is no joint;
  the backtick,
  U+00B4 and U+2032 join nothing.
  Letters here are Latin and digits are ASCII.
  The old case that expected `the cat-9 nap` to be no joint pinned the rule then in force,
  and no commit message gives a reason against the new one.
  A quote that is only a number inside a range,
  a date,
  a clock time or a fraction is now refused as a place,
  and `1990` is no stand inside `1990's` (B482).
- B477,
  corner brackets (the follow-up to F4).
  `looseMarksOf` counts each of the four corner brackets as a loose mark of its own and never as attached.
  A bracket directly beside a span is not recorded,
  since a generation 17 record stores one of the four single marks and recording a bracket would widen it.
  A replacement never reaches outside the span,
  so such a bracket stays where it is.
  Accepted.
