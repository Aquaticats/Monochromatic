# Audit ledger: the consistency pass, stack depth, the floors fixed point and the page form

The record of the first two rounds that built the whole-page reading of the pass and made it consistent:
the parser's refusal at the edge of the stack,
the floors and the gain check settled together,
and the page that an arriving apostrophe takes its form from.
The seventh review and its fixes are filed in
[Audit ledger: the consistency pass, the page-wide proof, forms, floors and limits](audit-ledger-consistency-pass-page-wide-proof-forms-floors-and-limits.md),
and the eighth review and its fixes in
[Audit ledger: the consistency pass, one reading of every mark](audit-ledger-consistency-pass-one-reading-of-every-mark.md)
and
[Audit ledger: the consistency pass, the fixes of the eighth review and the oracle residue](audit-ledger-consistency-pass-fixes-of-the-eighth-review-and-the-oracle-residue.md).

Written on 2026-10-14 (UTC) from the hand-backs of n128 (`s185/handback-n128.md`) and n130 (`s187/handback-n130.md`),
the lead's notes of that day,
and the eighth review (n131,
`s188/review.md`).
The labels G1 to G12 of guards in these entries are the hand-backs' own and are no ledger ids.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B501: the parser's refusal raised the engine's `RangeError` when it was first built at the edge of the stack

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n128,
in the first whole-suite run on the tree where n127's work had been merged into n126's.
The run (`s185/step0-unit.log`) gave 2,277 `[PASS]` lines and 3 `[FAIL]` lines,
and the 3 were one case of `parse-mdx.unit.test.ts`:
the test expected `MdxParseError` and got `RangeError`.
That file alone passed (`s185/step0-parse-mdx.log`),
and the suite rerun passed (`s185/step0b-unit.log`,
2,277 and 0).
n128 read it as a load-dependent flake of a stack-depth test and changed nothing.
The lead decided on 2026-10-14 that it is a defect of the code,
and n130 diagnosed and fixed it.
The count of 2,277 is not explained beyond the union of the test files of the two merged trees (unverified).

What a reader of the page would have seen:
nothing in any run of the pass that was measured.
The pass catches every throw that is not an abort in `runPassConsistency` (`corpus-run/pass-consistency.ts`).
A raw engine `RangeError` from the parser reaches that catch,
the pass returns no consistency for the page with the finding "the consistency pass stopped",
and the page keeps its wording.
The eighth review found no input of the pass that reaches the stack edge before the nesting bound refuses it
(`s188/probes/p08-stack.out`),
and did not reproduce the bytecode flush that the stated residual of n130 needs (unverified).
The raw error reached by that path is B515.

The mechanism,
the measurements and the workaround are in `doc/troubleshooting/v8-first-call-compile-at-the-stack-edge.md`
at the repository root.
This entry holds only the facts of the ledger.

Fixed by n130.
`readyRefusals()` in the region "Refusals built before the stack is short" of `parse-mdx.ts` builds both refusals once at module load,
and the module calls it as a statement.
`STACK_OVERFLOW_MESSAGE` moved from `nesting-bound.ts` to the exported `nesting-vocabulary.ts`.
The new `parse-mdx-cold-refusal.unit.test.ts` starts a fresh child process per grammar through `runBuiltInChild`,
makes five stack-edge searches in each,
and asks for no raw `RangeError` out of the parser and at least three refusals.

Red evidence:
the test was written first and failed in 4 of 5 attempts for both grammars (`s187/red-cold.log`),
and the guard G11 (the module-load call removed) fails both cold cases (`s187/guard-11-red.log`).
After the fix the real fixture,
run 300 times per grammar in a cold process,
gave 299 refusals and 1 overflow inside the fixture's own recursion
(`s187/fixture-repeat-after-mdx.console`,
`s187/fixture-repeat-after-markdown.console`).
The fixture overflows in its own recursion about once in a hundred runs;
n130 did not loosen the test,
and it did not trip in either suite run.
The whole suite gave 2,287 `[PASS]` lines and 0 `[FAIL]` lines twice,
with exit status 0 in both units.

Stated residual of the author,
not reproduced:
bytecode flushing can make a long-idle process cold again,
and the last frames of the stack cannot run any function,
so a raw `RangeError` is still possible there.
The eighth review asked what that residual does to a page,
and the answer is the pending B515.

## B502: the floors and the gain check each assumed the other's result

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 in n128's list of what it did not build
("a floor that refuses a slice after the gain check is not seen by the gain check"),
and sent back to n130 by the lead.
The gain check of n128 (`withoutUnprovenGains`) assumed that every remaining edit of every slice is made.
The floors then ran per slice and could refuse a slice,
which keeps its wording and restores the apostrophe that the gain check had counted as gone.

What a reader of the page would have seen:
a leading apostrophe made on the premise that a closing mark stands nowhere after it,
while a refused slice restored exactly such a closing mark.
n130's red case is the round page of `corpus-run/consistency-round-page.test-fixture.ts`:
slice 0 `The folks ran ’round.`,
slice 1 `We went round the yard.` with the `’round` gain splice,
and slice 2 `Fill the parents’ bowl now.` whose splice to `the folks` the floors refuse through the declared name `parents`.
Before the fix `’round` was made while `parents’` stood restored.

Fixed by n130.
`settledSlicesOf` in `corpus-run/pass-consistency-apply-plan.ts` takes the floors and the gain check to one fixed point:
round one plans every slice (`slicePlansOf`,
with the gain check over the page of all non-refused slices' edits),
asks the floors about each slice with something to make,
memoizes the answers by slice index and edit ids,
adds each newly refused slice to a map that holds its plan at the time of the refusal,
and plans again.
A refused slice keeps its wording,
so `slicePlansOf` gives it no edits in the gain check (the optional `refused` set).
The loop ends when a round refuses nothing new.
`applyConsistency` (`corpus-run/pass-consistency-apply.ts`) passes a `floorsOn` callback that wraps the existing floors,
and records outcomes and findings only from what `settledSlicesOf` returns.

The argument that the loop ends is not that edits only leave.
A gain dropped in one round can come back in the next when a refusal removes an edit that carried an opener.
What only grows is the refused set,
which the slice count bounds,
so there are at most as many rounds as slices plus one.

Red evidence:
the new `corpus-run/pass-consistency-apply-floor-refusal.unit.test.ts` (three cases:
a refusal restores the closer so `’round` is `kept-current` and slice 2 is `kept-floor`,
a control with no refusal,
and a refusal of a slice with no mark that keeps the gain)
failed with `’round` made while `parents’` stood restored (`s187/red-floors-apply.log`),
and the rewritten `pass-consistency-apply-plan.unit.test.ts` failed on the missing `settledSlicesOf` (`s187/red-floors-plan.log`).
Guards:
G3 (the gain verdict of `slicePlansOf` replaced by "keep all") fails in `slicePlansOf`,
`settledSlicesOf`,
`applyConsistency` and `runPassConsistency`;
G5 (the exclusion of refused slices removed) fails in `slicePlansOf`,
`settledSlicesOf` and `applyConsistency`.
Two guards were not shown to fail by a test that existed:
G4 (the `while` loop of `withoutUnprovenGains` made a single round) failed no test,
so n130 added a case after the code (two gains over the same words,
the first winning while it stands,
the page holding an opener for both,
expected kept `[]` and dropped `['a','b']`),
and the case fails without the loop (`s187/guard-4-red-b.log`);
G6 (the `while` loop of `settledSlicesOf` made a single extra round) failed no test and is not demonstrated (`s187/guard-6-red.log`).
n130 could not build a page that needs a third round from today's mark rules
(`s187/chain-gain-probe.out` shows which marks block which gain).
The lead decided on 2026-10-14 to keep the loop,
because the termination argument does not depend on the mark rules,
and to state the gap here:
no test notices if the loop is replaced by exactly two rounds.

What stays open:
the gain check still drops all unproven gains in one round,
so a gain that could stand on the final page can stay dropped (B513).
The suite gave 2,287 `[PASS]` lines and 0 `[FAIL]` lines twice,
the build,
`lint`,
`lint:types` and the 49 source scans were clean,
and the replay of the published page gave the same two groups at generation 15 as before.

## B503: an arriving apostrophe took its form from one slice, and not from the page

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 in n128's list of what it did not build
(`pageApostropheFormOf` still reads one slice),
and sent back to n130 by the lead.
B494 made an arriving apostrophe take the form that the page's prose writes,
and the code read the English of the one slice that holds the receiving words.

What a reader of the page would have seen:
a page that writes curly apostrophes in its other sections received a straight one in a section with no apostrophe of its own.

Fixed by n130.
`pageReadingOf` in `consistency-page-english.ts` validates the page and the slice's place in it
(it throws `ConsistencyInvariantError` for a page that does not hold the slice).
`proofViewOf` in `consistency-fold-apostrophes.ts` (the new type `ProofView`) gives the prose and the proven marks of the page.
`pageApostropheFormOf({ english, start, end, page, apart })` in `consistency-apostrophe-forms.ts` reads the page through that view
and leaves out the receiving span and the stretches named in `apart`.
`corpus-run/pass-consistency-decide.ts` passes `place.slice.page` and `apart`.

The decision that the lead accepted:
the chosen words' own span is left out of the page's reading.
Without that,
the arriving straight apostrophes vote for themselves,
and the existing `sharing` case contradicts the brief's red case.

Red evidence:
four new cases failed first (`s187/red-forms.log`):
a whole page,
a prose view that ignores code,
`apart` stretches,
and the refusal of a page that does not hold the slice;
in `corpus-run/pass-consistency-apostrophes.unit.test.ts` a receiving slice with no apostrophe and another slice that writes curly
make a straight chosen apostrophe arrive curly,
and a mixed page keeps the chosen form.
Guards:
G7 (the page not handed to `pageReadingOf` in `pageApostropheFormOf`) fails three `pageApostropheFormOf` cases and one in `runPassConsistency`;
G8 (`apart` not passed from `pass-consistency-decide.ts`) fails two cases in `runPassConsistency`.
The direct tests of `pageReadingOf` and of the move of `foldWithOffsets` were written after the code (M184).

Results:
the review (n131) read the same-slice rows of `s188/probes/p17-same-slice.out`:
the chosen mark arrives as written when it is the page's only apostrophe,
a curly mark elsewhere makes the arrival curly while the chosen place's own straight mark stays,
and the receiving words' own straight form wins over a curly page.
The arriving marks do not vote for themselves in any row.
Apostrophes after closing markup did not vote at all (B509).
