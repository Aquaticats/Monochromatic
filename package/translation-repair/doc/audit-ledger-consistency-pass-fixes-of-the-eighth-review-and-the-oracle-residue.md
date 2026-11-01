# Audit ledger: the consistency pass, the fixes of the eighth review and the oracle residue

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

The findings of the eighth review (n131) that the second fix round (n134) took in hand,
the flags of the first fix round (n133) that the lead decided on 2026-10-14 to fix after the merge of the two rounds,
and the residue that the rerun of the independent oracle and n143's attention list left for n144.
Each entry was written when the decision was made,
and its outcome was added on 2026-10-14:
B513 to B519 when the fixes of n134,
n136,
n137,
n139,
n140 and n141 had landed in the working trees,
and B549 to B555 when the fixes of n144 had landed in them.
Every entry here has landed,
and none of them is committed.
The file was named for the fixes decided and not yet landed,
and carries the name of what it holds now.
The one fix of this work that was still not landed when it was written,
the container markers inside the refusal run,
was built by n151 and is filed in
[Audit ledger: the consistency pass, the re-review and the refusal run](audit-ledger-consistency-pass-re-review-and-the-refusal-run.md).
The entries of the eighth review that n133 closed,
and the two limits it left accepted,
are filed in
[Audit ledger: the consistency pass, one reading of every mark](audit-ledger-consistency-pass-one-reading-of-every-mark.md).
The stack depth,
the floors fixed point and the page form are filed in
[Audit ledger: the consistency pass, stack depth, the floors fixed point and the page form](audit-ledger-consistency-pass-stack-depth-floors-fixed-point-and-page-form.md).
The defects that the later rounds found are filed in
[Audit ledger: the consistency pass, parse boundaries, the letter test and the proof's page](audit-ledger-consistency-pass-parse-boundaries-letter-test-and-proof-page.md),
[Audit ledger: the consistency pass, the ninth review](audit-ledger-consistency-pass-ninth-review.md),
[Audit ledger: the consistency pass, the independent oracle and the rules for marks](audit-ledger-consistency-pass-independent-oracle-and-mark-rules.md),
[Audit ledger: the consistency pass, the tenth review](audit-ledger-consistency-pass-tenth-review.md)
and
[Audit ledger: the consistency pass, the re-review and the refusal run](audit-ledger-consistency-pass-re-review-and-the-refusal-run.md).

Written on 2026-10-14 (UTC) from the eighth review (`s188/review.md`),
the hand-backs of n133 (`s191/handback-n133.md`),
n134 (`s192/handback-n134.md`),
n136 (`s194/handback-n136.md`),
n137 (`s198/handback-n137.md`),
n139 (`s200/handback-n139.md`),
n141 (`s203/handback-n141.md`),
n143 (`s206/handback-n143.md`),
n144 (`s208/handback-n144.md`),
n147 (`s212/handback-n147.md`) and n149 (`s215/handback-n149.md`),
the tenth review (`s210/review.md`),
and the lead's notes of that day.
The labels J3 to J7 and the letters of flags are the review's and the hand-back's own and are no ledger ids.
The guard labels collide between hand-backs,
so each entry says whose guard it names.
A path that starts with `src/` is under `package/module/translation-repair/` (moved on 2026-10-18 to the packages under
`package/translation-repair`).
Every function,
type and constant of the package named here was found declared in non-test source of the merged tree `w158`
by searches of declarations on the day of writing,
except those an entry says were removed or renamed.
The names that only a hand-back gives (test names,
guard names,
the helpers of test fixtures and the names of logs) were not searched.
The tree changed after n144 (n147 and n149 followed),
so an outcome of B549 to B555 names a function as n144 left it,
and the entries of the later rounds say what they changed.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B513: a gain that could stand on the final page stays dropped

Found on 2026-10-14 by the eighth review (n131,
finding J3,
`s188/review.md`).

Outcome written on 2026-10-14.
Fixed by n134 and tightened by n137 and n141;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
a place that kept its words although its gain could stand on the final page.
In the probe `p15-overdrop` row X1 (two places with a leading and a trailing gain),
the baseline left both places `kept-current` and changed nothing;
row X2 (the first place refused by the floors) left both `kept-current` too,
although the first slice's mark is not on the final page and the second gain could stand there.

The cause:
the floors loop dropped a gain on the page as it was when the gain was judged,
and nothing asked again once other drops and refusals had changed the page.

Fixed.
`readmittedSlicesOf` in `src/corpus-run/pass-consistency-apply-readmit.ts`
runs passes of screens (`screenSide`) and trials (`trialOf`)
in page order until a pass admits none,
and `settledSlicesOf` calls it after the floors loop.
An admitted splice is never dropped again and the admitted set only grows,
so there are at most D+1 passes for D dropped splices.
`gainReasons` and `gainFindingOf` state the reason that holds on the final page (B524).
n141 added the exact search that makes the admitted set the largest (B528).
n134 merged into `w148` with 34 files copied,
2 merged and no conflicts;
the one semantic conflict was the screen's import of the any-script letter test that n133 had removed,
and the lead switched it to `continuesLatinWord` and later to `continuesLatinWordOrCompatibilityForm` (B523).

Red evidence:
in `p15-overdrop` X1 now reads `["replaced","kept-current"]` with the page `Then ’Bowl 1 of the folks is empty.`,
and X2 `["kept-current","replaced"]` with `Then Bowl 2 of the parents’ is empty.`;
the control X3 is unchanged.
The check `droppedGainCouldStand` of the random-page probe `p10-settle-fuzz` (3,000 pages) read 33 and reads 0.
The 33 flagged gains and the 32 re-admitted ones did not match one for one;
n137 traced the remaining one (seed 2815,
slice 2):
slice 1's leading gain is re-admitted first in page order,
so the two marks pair as a quotation and neither is proven,
and leaving the second out is the right outcome.
The control `p10-control` still fires:
every injected break of its modes A0 and C1 is caught,
and its mode 3 catches 63 of 750 where it caught 58.
n134's guard 1 (the plan returns without `readmittedSlicesOf`) fails 5 lines of the readmit test,
and its guard 2 (`screenSide` returns before reading) fails 5 lines of the screen test
and takes the dropped variant of `p06` to 14,703 ms at 500 slices and 60,836 ms at 1,000
against 76 and 197 ms with the screen.
n134 did not show the conditions of `isScreenable` red;
n137 did,
one by one:
the guards G3 (the count test),
G4 (the quote taken out),
G5 (the touching replacement),
G6 (`!stays`) and G7 (the letter beside the mark)
each fail a named screen test,
G1b fails the loose-mark test,
G7b (any letter) fails only the new Greek-letter test,
and two conditions whose removal failed no test (G1 and G2) were deleted as redundant.

## B514: the proof does not read the author's English in sealed spans

Found on 2026-10-14 by the eighth review (n131,
finding J4,
`s188/review.md`).

Outcome written on 2026-10-14.
Fixed by n134 and rebuilt by n137 and n141;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
a place replaced although the opening of a quotation stood in a sealed span,
which is the author's English that the pass keeps as it is,
and its closer stood in the next slice.
The proof had not read the sealed English,
so it could not see the opener as the closer's partner.
In the probe `p04-pages` rows P4 and P4b (the opener in a sealed span,
the closer in the next slice) the baseline said `replaced`,
and the page now keeps the place (`kept-current`) with the page unchanged;
the control P4c (no opener) is `replaced` before and after.

The cause:
the page the proof read held only the slices' English,
and the text between the slices (sealed spans,
front matter,
anchors) was missing from it.

Fixed.
n134 built `pageOfSlices({slices, between})` in `src/consistency-page-english.ts`,
with the text between the slices taken from `prepared.targetText`
(`textBetweenSlices`,
`SliceSpan`) in `src/corpus-run/pass-consistency-slices.ts`,
and an invariant error when the pieces do not number the slices plus one.
n137 replaced that reconstruction with the assembler that publishing uses (B525),
and removed `textBetweenSlices` and `SliceSpan`.
n141 made the proof read the glyphs that ship (B526).

Red evidence:
n134's guard 3 (`pass-consistency-slices.ts` passes no text between) fails 2 cases of the slices test
and the sealed-span case of the apostrophes test,
and its guard 5 (the count check in `pageOfSlices` removed) fails 3 cases of the page-english test.
`p05-sealed` is byte-identical to its baseline,
since it reads the slicer only.
The corpus replay of 92 pages reads 27 trailing and 1 leading proven marks on both the old page and the published one,
with no mark whose proof changes and no mark between slices,
so the corpus holds no sealed-span opener that affects a mark;
the change is shown by the P4 probes and not by the corpus.
The limit that B492's correction named (sealed spans) is closed by this fix.

## B515: a raw stack-exhaustion `RangeError` from the parser stops the pass for the page

Found on 2026-10-14 by the eighth review (n131,
finding J7,
`s188/review.md`).

Outcome written on 2026-10-14.
Fixed by n134,
narrowed by n136,
n139 and n140,
and accepted as unreachable at the edge of the stack by the ninth review (B527);
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
a raw `RangeError` from the parser's stack exhaustion that stopped the pass for the page,
instead of a refusal the pass reads and goes on from.

Fixed.
n134 made `requireMdxRefusal` build an `MdxParseError` (with `droppedColumns: 0`)
and `requireMarkdownRefusal` build a `MarkdownParseError` with the reading `stack-exhausted`
for the engine's stack-exhaustion `RangeError`,
and rethrow every other value unchanged.
`src/refusal-text.ts` says in plain words that the parse was refused because the engine ran out of call stack space.
The boundaries that read the refusal are the parse boundaries of the package (21 listed in `s192/refusal-sites.txt`).
The test B100 of `src/parse-mdx.unit.test.ts` asserted that a stack exhaustion is rethrown;
n134 reversed it,
since that is the behaviour this item changes,
and the lead accepted it.
The conversion read an exhaustion raised by code other than the parse (B522),
which n136,
n139 and n140 narrowed.

Red evidence:
n134's guard 4 (the conversion removed from both readers) fails 5 lines of the `parse-mdx` tests
and 3 of the `consistency-prose-text` tests,
and the refusal-text test,
which does not depend on the conversion,
shows its 2 passes and no failure.
The probe `p08-stack` does not reach the conversion path,
because the engine never exhausts the stack at the depths it builds,
so it is no evidence for this item;
its only differences from the review's output are the stack-frame measurements,
which move with the build and the machine.

## B516: the apply stage grows with the square of the page

Found on 2026-10-14 by the eighth review (n131,
finding J6,
the apply-stage part,
`s188/review.md`).
The locate and form stages of the same finding are B512.

Outcome written on 2026-10-14.
Fixed by n134;
the cost was measured again by the ninth review (B531),
n137 and n141;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
nothing wrong on the page.
The time of the apply stage grew with the square of the page.

Fixed.
n134 added `src/corpus-run/pass-consistency-apply-index.ts`:
`settledIndexOf` reads the groups in one pass,
`sliceEditsOf` in `src/corpus-run/pass-consistency-apply-record.ts` and `editsChangingLocatedWords` in
`src/corpus-run/pass-consistency-apply-located.ts` look an edit up by slice,
and `workedPageOf` in `src/corpus-run/pass-consistency-apply-worked.ts` builds the index and the worked list once,
called once for each `settledSlicesOf`.
`slicePlansOf` was removed as unreached.

Measured with the cost probe `p06` (`apply_ms` for 125,
250,
500,
1,000 and 2,000 slices):
- n134,
  one run each:
  the kept variant 279,
  479,
  884,
  1,707 and 4,559 ms (the last ratio 2.67 comes partly from a low point at 1,000),
  the dropped variant 24,
  38,
  76,
  197 and 431 ms (1.58 to 2.59 per doubling),
  and the refused variant 212,
  424,
  874,
  1,706 and 3,332 ms (1.95 to 2.06).
  The review's figures were 1.68 to 2.40,
  1.9 to 4.6 and 2.07 to 2.61.
  n134 did not remeasure the run-to-run spread.
- The ninth review measured the dropped variant at 2.3,
  2.8 and 3.1 per doubling (B531).
- n137 measured the kept variant three times at 1,000,
  2,000 and 4,000 slices (medians 1,917,
  3,459 and 7,088 ms before its changes and 1,958,
  3,504 and 7,162 ms after,
  growth 1.80 and 2.05 against 1.79 and 2.04),
  so the change is inside about 2 percent.
  Its dropped variant became about nine times n134's,
  linear at 2.00 to 2.13 per doubling (B524).
- n141 found about 2 ms a slice and linear growth for all three variants,
  and a cost that sits in the parse of the floors verdict (B531).

Red evidence:
n134 did not show the indexing of `settledIndexOf` red;
the poison-proxy tests and the `p06` figures cover it.
n137 did:
its guard G8 (an extra read) fails the test that reads each group once and never again
(12 reads built,
156 with the variant).

## B517: a place left as it was under a `replace` plan records no reason

Decided on 2026-10-14;
fixed by n137 on the same day;
uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n133 (flag a).
`mayBeLeftAsItWas` in `corpus-run/artifact-two-lane-read-consistency-span-shared.ts` now takes `marks`,
and accepts the outcome `kept-current` for a place whose plan is `replace` and whose splice was chosen,
only in generation 17 (`marks === 'compared'`).
Generation 16 (`'unrecorded'`) still refuses it.
`corpus-run/artifact-two-lane-read-consistency-span.ts` was changed to fit.
The pass leaves such a place as it was when a gain is dropped on the final page or a splice is refused at the edge,
and in the probe `p01` (row S1) the second place is `kept-current` and the reader accepts it.

What a reader of the page sees:
nothing,
since the record is not the page.
What is lost is a check:
the record holds no English around the splice,
so the reader cannot check why the pass left the place alone,
and accepts `kept-current` under `replace` for any reason or none.
This is weaker validation than before,
and the family of B485 and B500.

The lead decided to tighten,
and not to accept:
a place left `kept-current` under a `replace` plan records why,
from a closed set
(a gain dropped on the final page,
a splice refused at the edge),
and the record reader allows `kept-current` under `replace` only with a recorded reason.
Generation 17 is unreleased,
so its record changes.
The fix touches the files of both fix rounds and is made after the merge.

Evidence of the present state:
guard G9 of n133 (the refusal for `marks === 'unrecorded'` removed)
fails the case that expects a throw for `kept-current`
under a `replace` plan in a section that recorded no marks beside the words (it returned instead).

Outcome written on 2026-10-14.
Fixed by n137.
The reasons are a closed set,
`ConsistencyKeptReason`:
`gain-pairs`,
`gain-unproves`,
`gain-floors`,
`gain-overlaps`,
`gain-refused`,
`edge-second-mark` and `edge-reading-changes`.
An occurrence carries the field `keptBecause` only in generation 17 and only on `kept-current`,
generation 16 keeps refusing it by exact keys,
and `assertCarriedOutcomes` in `src/corpus-run/artifact-two-lane-read-consistency-span-shared.ts` requires it where
`mayBeLeftAsItWas` allows `kept-current`.
`edgeRefusalOf` in the new `src/corpus-run/pass-consistency-edge-refusal.ts` returns an `EdgeVerdict`
with the sentinel `'unrefused'`,
and `leftBecause` is carried through `settleCarried` and `settleAsked` in the settle,
decide,
types,
record and settled modules.
`keptReasonOfGain` and `keptReasonExpected` give the reason and the check of it.

Red evidence:
`s198/red-items-1-2.log`.
n137's guard G9 (the reader requires the reason),
G12 (the record check of `edgeRefusalOf`),
G13 and G13c (the occurrence checks),
G18 (generation 16 refusing `keptBecause`) and G19 and G20 (the `leftBecause` plumbing) each fail a named test.
These guard experiments ran on the build before its last refactors
(the `EdgeVerdict` sentinel,
the split of `splice-slice-places.ts`,
`FragmentPlace.fragment`,
lint fixes and the removal of `composeInsertion`),
which is why `guards/original.sha256` shows 3 files unchanged and 9 changed against the final tree;
the restores were verified at the time,
and the final tree is covered by the lint,
the scans and the suite.

## B518: mathematical letters are not Latin to the one letter test

Decided on 2026-10-14;
fixed by n136 and n139,
and changed again by n143;
uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n133 (flag c),
as a consequence of B507.
The one corpus mark whose kind changes is `𝓠𝓾𝓪𝓷’s` of mikaela_khara at offset 4703:
the letters before the mark are mathematical script letters,
which are not Latin to `continuesLatinWord`,
so the mark went from attached to a proven leading mark.
No page whose output changes was shown.

The lead decided:
a neighbour whose NFKC form is a Latin letter or an ASCII digit counts as Latin
(mathematical alphanumerics and fullwidth Latin letters),
so `𝓠𝓾𝓪𝓷’s` reads attached again.
The fix is made after the merge.

Outcome written on 2026-10-14.
n136 built the decision inside `continuesLatinWord`,
and the widening reached every caller in the package (B523).
n139 moved it into `continuesLatinWordOrCompatibilityForm` in `src/latin-letters.ts`
and put `continuesLatinWord` back to its body from before n136;
the consistency readers and the screen use the new predicate.
n143 added a gate that the compatibility form must be a letter or a decimal digit (categories L and Nd),
which makes `™`,
`²` and `№` no letters as decision D3 asks,
and also makes `①` and `Ⅻ` no letters (B546).
The corpus:
`𝓠𝓾𝓪𝓷’s` of mikaela_khara at offset 4703 reads attached again,
and the proven marks are 27 trailing and 1 leading.
A census of the 92 pages finds only `𝓠 𝓾 𝓪 𝓷` (7 each) and `₂` (2,
in the front matter `name` of Aniloviraw)
among the characters that the widened test admits and the old one did not.

Red evidence:
n136's `s194/red-item1.log` holds 13 failing lines and 18 passing lines over four test files
(`latin letters`,
`looseMarksOf`,
`edgeClassOf`,
`markReadingsOf` with compatibility letters,
and the cut of a word by the parse),
with the controls passing before and after.
The case that proves a mark beside a mathematical or fullwidth letter,
and the change of a Fraktur letter to a Deseret one in an existing proven-apostrophes case,
were written after the baseline build was replaced,
so their red is n136's guard G1 (`s194/guards/g1-red.log`) and not the log.
n139's reds are in B523.

## B519: four helpers of the mark readings have no test of their own

Decided on 2026-10-14;
fixed by n136 and n137 with n139;
uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n133 (reported under "Requirements not met").
`readProof` and `isPartOfAWord` in `consistency-fold-apostrophes.ts`
and `bringsAtEdge` in `consistency-splice-readings.ts`
are private helpers that the exported callers reach
(`proofViewOf`,
`markReadingsInEnglish`,
`bringsEdgeApostrophe` and `gainsBesideKeptMark`),
and those callers have tests.
`isLetterOrDigitOfAnyScript` is unchanged code and has no direct test.
No page effect was shown.
A branch of a helper that no case of a caller reaches has no test,
and which branches those are is not measured (unverified):
no branch coverage was run.

The lead decided:
export the four from their modules,
not from the barrel,
and test each branch directly.
The fix is made after the merge.

Outcome written on 2026-10-14.
n136 exported `readProof` with its type `ProofReading` and `isPartOfAWord` from `src/consistency-fold-apostrophes.ts`,
and `bringsAtEdge` from `src/consistency-splice-readings.ts`,
each marked `@internal`.
It did not follow the decision to keep them off the barrel,
because the lint rule `test-import(require-eventual-artifact)` rejects a test that imports a module's source
and tells the author to export the symbol from the package entry marked `@internal`,
which rule XPT of `AGENTS.md` allows.
The helpers are re-exported through one new barrel,
`src/built-artifact-barrel.ts`,
and the tests import the built entry.
`isLetterOrDigitOfAnyScript` is tested in the test file of its own module,
because the scan for tests of a module's own maps a test to a module by name.
Its describe cannot be red before a code change,
since the function already existed,
and n136's guard G1 shows it fail.
The tests of `readProof` and `isPartOfAWord` are in `src/consistency-fold-apostrophes.unit.test.ts`,
of `bringsAtEdge` in `src/consistency-splice-readings.unit.test.ts`,
and of `isLetterOrDigitOfAnyScript` in `src/consistency-loose-marks.unit.test.ts`.

n137 added direct tests for every new helper of n134:
`editedStartOf`,
`gainedBy` and `madeIn` (gains),
`idsOf` and `pageFrameOf` (standing,
now the ledger),
`madeWithSplice` and `floorsWithSplice` (trial),
`contentWrite` and `placementsOf`,
`fragmentCut` and `composedInsertion`,
`placeSlices`,
the `leftBecause` plumbing of `settleCarried` and `settleAsked`,
`edgeRefusalOf`,
`keptReasonOfGain` and `keptReasonExpected`.
n139 added tests for `closingJointsOf` and `joinedMarksOf`,
`edgeFreeWordsOf`,
`bringsEdgeApostrophe`,
`readingsChangeOutside`,
`bringsAtEdge` and `pageApostropheFormOf`.
Branch coverage was not measured,
so which branches of these helpers no case reaches stays unverified.

Red evidence:
n136's `s194/red-item3.log` (the first version imported the source:
"does not provide an export named" `isPartOfAWord` and `bringsAtEdge`),
and its guard G1,
which makes several mutations at once and shows each by its own failures
(the hand-back says five and lists six):
the compatibility clause of `continuesLatinWord` removed,
`isLetterOrDigitOfAnyScript` without the decimal digits,
`disjointSpans` keeping the last span's end,
`bringsAtEdge` ignoring the receiving words,
`readProof` returning no joined marks,
and `isPartOfAWord` false for attached and joint marks.
n137's tests were written after their code and are red by mutation (G17,
G19 and G20) or by the missing export (`s198/red-item-7.log`).
n139's tests of `bringsEdgeApostrophe`,
`readingsChangeOutside`,
`bringsAtEdge` and `pageApostropheFormOf` are red only under its guard G1 (all four fail),
since they were written after the readers were switched (M190,
M191,
M193).

## B549: a single mark at the edge of the chosen words that the replaced words lack is not counted as a gain

Found on 2026-10-14 by the rerun of n142's oracle on n143's build (`s207`).

Outcome written on 2026-10-14.
Fixed by n144;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
a gain mark carried without a proof,
for instance an apostrophe written after `_cats_` in a place whose emphasis the conversion then broke.
All five of the oracle's examples were planned `replace` and kept on n143's build
(`before/item1-repro.txt` in `s208`).

The cause:
the planner (`planConsistencyEdit` in `src/consistency-splice.ts`)
compared the loose marks of the two words (`sameLooseMarks`)
before `inReceivingForm` wrote the apostrophe in the receiving place's form.
In `_cats_’` the underscores are real emphasis,
so the `’` is an edge apostrophe past a closing run and is no loose mark.
The conversion then wrote U+02BC,
a letter to the Markdown parse,
so `_cats_ʼ` has no closing underscore and its mark reads free and loose.
The comparison had read the text before the conversion and saw no loose mark arrive.
`holdsEdgeApostrophe` (a letter or digit directly inside the mark) was then false on the written words,
so `bringsAtEdge` and `gainedSidesOf` did not call it a gain,
and the mark arrived unproven.
The package parse gives `_cats_’` the closing delimiter at 5 and no loose mark,
and `_cats_ʼ` no closing delimiter and the loose mark `'`.

Fixed.
`holdsEdgeMark` in `src/consistency-loose-marks.ts` is true when the whole edge character is one of the single marks,
whatever stands inside it.
`bringsAtEdge` in `src/consistency-splice-readings.ts`
asks `holdsEdgeMark` of the text and `holdsEdgeApostrophe` of the core,
and is true when the first holds and the second does not.
`gainedSidesOf` in `src/corpus-run/pass-consistency-apply-gains.ts` calls `bringsAtEdge`,
so the planner,
the apply stage and the record reader have one definition of a gain.
After the form conversion the planner compares the loose marks again,
the written words against the chosen core,
and a mismatch makes the place `unavailable`.
`holdsEdgeApostrophe` is unchanged,
since a wide change would kill the record reader's guard.

The lead accepted one consequence:
the screen test now reads a loose `‘` splice as a gain,
which only hand-built edits reach,
since the planner never plans that splice.
The first suite run after the change had one failure for it,
in `src/corpus-run/pass-consistency-apply-screen.unit.test.ts`.
The test now keeps its intent with a double mark,
which blocks nothing,
and asserts for `‘` that both splices are blocked and the trial decides.
n147 changed that expectation again (B556).

Red evidence (logs under `s208`):
the tests are a five-case planner test with `english` and the page's form as the pass gives them,
with the control `*cats*’` still planned,
in `src/consistency-splice.unit.test.ts`;
rows for `bringsAtEdge` and `bringsEdgeApostrophe` in `src/consistency-splice-readings.unit.test.ts`;
a test of `holdsEdgeMark` in `src/consistency-loose-marks.unit.test.ts`;
and a page test in `src/corpus-run/pass-consistency-apply-gains.unit.test.ts`
(seven pages dropped,
`gainedBy` `['end']`,
the control `*cats*ʼ` kept).
Against the unchanged build they fail in `red/splice-red.log`,
`red/readings-red.log`,
`red/gains-red.log` and `red/loose-marks-red.log`;
the last is a missing-export red only,
since `holdsEdgeMark` did not exist.
The guard A (the planner's check after the conversion removed) is red on exactly the five-case planner test,
and the guard B (`bringsAtEdge` back to `holdsEdgeApostrophe`) on the gains page test,
the readings rows and the `bringsEdgeApostrophe` rows;
both restores are shown by checksum (`guards/A-restore.log`,
`guards/B-restore.log`).
The five-case planner test was not run on the baseline build,
and guard A is its red.

The two-edit pages of the oracle:
`He said 5 ***cats**’)猫 now. He said 猫**cats**’) now.` and its shapes.
Of the 59 two-edit pages of the oracle's grid,
55 had the shape of this entry and are fixed by it,
and 4 pages in 3 keys follow real closing strong emphasis.
The package parse gives the first page the delimiter offsets 11,
12,
17 and 18 and the second 37,
38,
43 and 44,
the `*` at 10 is no delimiter,
and both marks (at 19 and 45) read trailing and proven
(`s208/before/item2-parse.txt`).
n144 added a regression test that keeps both splices of a paragraph whose two marks follow real closing strong emphasis
(`src/corpus-run/pass-consistency-apply-gains.unit.test.ts`,
green at the baseline by design),
and could not say why the oracle reads the first edit's mark free on those four;
its guess,
that the oracle does not take `**` directly after a Han letter for an opener,
was unverified.
The lead's rerun of the oracle on `w158` (`s211`) gave:
single-edit grids a to d 0,
two-edit grid a 4 and b to d 0,
claims 0.
The tenth review (`s210/review.md`,
part 5,
item 4a) decided the four:
micromark 4.0.3 gives `<strong>` for both pages,
the package's closing delimiters are 44,
43,
18 and 17 and its proven trailing marks 19 and 45,
so the oracle is wrong on the four and production proves the marks correctly.
No code changed.
The reviewer wrote the flanking-rule reasoning from memory,
since the CommonMark text was not installed,
so that wording is unverified and the behaviour of the parse is checked.
The rerun on n149's build (`s216`) read the same:
122,499 single-edit pages with a to d 0,
45,389 two-edit pages with a 4 (the file of failing patterns is byte for byte the one of `s213`),
and 42,412 claims pages with 0.

n144's last whole-suite run read 2,365 passes and no failures (exit 0,
n143's last green was 2,340),
lint 0 warnings and 0 errors,
types clean,
the source scans clean,
and the replay lines as expected.
That holds for B549 to B555.

## B550: a gain with a literal `*`, `_` or `~` between two marks is not refused

Found on 2026-10-14 by n143's attention list (the planner grid `h2-grid-j1`).

Outcome written on 2026-10-14.
Fixed by n144;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
a doubled mark.
The decision D1 refuses a gain when a single mark stands one run away from the span on the gain side,
and the refusal run of n143 took `*`,
`_` and `~` into the run only where the parse made them delimiters.
A literal one ended the run,
and 139 shapes of the grid `h2-grid-j1` (200,000 pairs) were planned `replace` with a mark on each side of it.

Fixed.
`isRunUnit` in `src/consistency-closing-marks.ts` is now `characters.has(unit) || EMPHASIS_CHARACTERS.has(unit)`:
the run counts `*`,
`_` and `~` whatever the parse says,
and the `emphasis` parameter of `isRunUnit` and `markPastRunOf` is gone.
The headers of `src/consistency-closing-marks.ts` and `src/consistency-markup-runs.ts` say so.
The tests are in `src/consistency-closing-marks.unit.test.ts`,
`src/consistency-count.unit.test.ts` and `src/consistency-splice.unit.test.ts` (five shapes `unavailable`),
with reds in `red/closing-marks-red.log`,
`red/count-red.log` and `red/splice-red.log`.
n144's guard G3 (the emphasis characters out of `isRunUnit`) is red in `guards/C-test-closing.log`,
`guards/C-test-count.log` and `guards/C-test-splice.log`.
n147 rewrote the run again (B557).

Evidence on the final build:
`final/h2-grid-j1.out` reads `{"pairs":200000,"replace":77512,"shapeNew":0}`,
n143's grid read 77,817 and 139.
The tenth review reproduced 77,512.
Of the 305 fewer replacements,
139 are the shapes of this entry,
and the other 166 are not enumerated
(the probe has no pair list of n143's;
which change removed them is unverified).
Two experiments on a mutated build are labelled as such and are no final numbers:
without the check of B549 the grid replaces 77,550 (that check alone removes 38),
and without the wide run it replaces 77,848 with 179 new shapes,
but that build also stops the run at real delimiters,
so it is not n143's behaviour and the two effects do not add up.
The grid holds no `‚ ‛ ‹ ›`,
so B551 does not reach it.
The rest is the wider `bringsAtEdge` of B549 and this entry on pages without the shape;
no change allows a splice that was refused before.

## B551: the generation 17 record does not store `‚ ‛ ‹ ›`, so decision D1 cannot see them

Found on 2026-10-14 by n143's attention list.

Outcome written on 2026-10-14.
Fixed by n144;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
no page was shown.
The record stored four single marks,
so a place with one of the other four beside its words looked markless to D1.

Fixed.
`isSingleMark` stays at four marks.
`isCountedSingleMark` in `src/consistency-loose-marks.ts` (new,
the eight over `isNeighbouringSingleMark`) is used by `singleMarkAt`
(`markBefore`,
`markAfter` and `markPast`),
by `requireSingleMark` in `src/corpus-run/artifact-two-lane-read-consistency-rendering.ts`
and by `holdsTwoMarksAtEdge`.
The locator and `edgeMarksContradictedBySpan`
in `src/corpus-run/artifact-two-lane-read-consistency-span-located.ts` stay at four,
so a judge's quote `‹cats›` is still spanned with the marks inside.
The contract TSDoc in `src/corpus-run/artifact-two-lane-contract.ts` names the eight.
Tests:
`isCountedSingleMark` and `singleMarkAt` in the loose-marks test,
a gain `unavailable` beside each of `‚ ‛ ‹ ›` with a control `»` in the splice test,
each of the eight read back and `«` and `»` refused in the rendering test,
the preservation test of the span (words that start or end with `‹ › ‚ ‛` stay accepted),
and pair rows in the readings test.
Reds against the unchanged build:
`red/rendering-red.log` and `red/splice-red.log`.
The guards G4 (`singleMarkAt`),
G4c (`requireSingleMark`),
G5 (`holdsTwoMarksAtEdge`) and G9 (`edgeMarksContradictedBySpan` widened) are each red in
`guards/C-test-loose.log`,
`guards/C-test-rendering.log`,
`guards/C-test-readings.log` and `guards/C-test-span.log`,
restored by `guards/C-restore.log`.
`holdsEdgeMark` still knew four of the eight until n147 (B560).

## B552: the test of the page `”’**cats**).` says pending where the lead accepted the outcome

Found on 2026-10-14 by the lead's ruling on n143's attention list.

Outcome written on 2026-10-14.
Fixed by n144;
uncommitted at the time of writing;
commit:
pending.

The test in `src/consistency-splice.unit.test.ts` is renamed to an accepted outcome
that cites the lead's ruling on the case c3 of D7,
and asserts the readings `18:free` and `24:attached`.
It is green.
The outcome itself is B544.

## B553: the header of `consistency-marks-reading.test-fixture.ts` still describes the leading mirror

Found on 2026-10-14 by n143's attention list.

Outcome written on 2026-10-14.
Fixed by n144 in the header only;
uncommitted at the time of writing;
commit:
pending.

The module header of `src/consistency-marks-reading.test-fixture.ts` is rewritten to D1 to D9 as they hold.
It lists where this independent reader is coarser than the rules:
it has no parse,
it keeps a leading mirror,
it has a wider closing run,
and it has no parting by a pictograph or a symbol (D3).
The code of the fixture is not changed.
No later round touched it either,
so the fixture still reads those marks more coarsely than the rules do,
and the header says where.

## B554: the follow-ups that n141 left: the fixture with straight marks, the reason `unproves` and the tie rule

Found on 2026-10-14 by n141's hand-back.

Outcome written on 2026-10-14.
The straight-marks fixture and the tie rule were fixed by n144,
the reason `unproves` was settled by n147,
and its finding text was corrected by n149;
uncommitted at the time of writing;
commit:
pending.

The straight-marks fixture:
a test in `src/corpus-run/pass-consistency-apostrophes.unit.test.ts`
keeps the straight form that the chosen place wrote on a page whose archive keeps its straight marks.
It is green at the baseline (`red/apostrophes-before.log`).
The guard G7 of n144 (`inReceivingForm` always writes a form,
curly by default) is red in `guards/C-test-apostrophes.log` and in the splice test,
restored by checksum (`guards/C-restore.log`).

The reason `unproves`:
n144 neither built nor removed it and did not prove it unreachable.
A random search (3,000 plain,
8,000 markup and 8,000 connector pages) found none,
on a build made before the last items of that round,
and it was not rerun on the final build.
n144's argument is that pairing through marks alone is symmetric,
so any mark that unproves a standing splice also unproves the splice's own mark,
and the own-mark check in `src/consistency-proven-apostrophes.ts` fires first as `pairs`;
the channels left (touching spans,
parse effects) need a third splice to survive the first round's fixed point.
The hand-built test of `readmittedSlicesOf` stayed the only coverage.
The tenth review found none either (4,000 plain,
8,000 markup,
16,000 connector and 12,000 touching-connector pages) and did not prove it unreachable.
n147 then built a page by hand that produces it:
the slice `A cat naps. The kittens, cats nap.` with three splices,
`cat` to `‘cat`,
`kittens` to `dogs’` and `, cats` to `xcats’`,
the last two touching and the second changing the comma beside the first's mark into a letter.
The outcomes are `kept-current`,
`replaced` and `kept-current`.
The reason is therefore reachable and stays,
with its type member.
The test is in `src/corpus-run/pass-consistency-apply-hand-built.unit.test.ts`
(it passes on the old build too,
so the guard G7 of n147 is what shows it can fail),
and a header paragraph is added to `src/corpus-run/pass-consistency-apply-trial.ts`.
The re-review confirmed the page and found the finding text untrue:
the first mark reads attached,
not as a quotation mark (B570).
n149 changed the text to "lose its proof".
The 170 `pairs` reasons that n144 saw in its connector fuzz
while the splice alone was proven were real and false (B556).

The tie rule:
the seed tests now assert that the made set equals the first maximum set found.
No existing seed discriminated it.
A search over 8,000 markup and 8,000 plain pages
under a mutation that reverses the order of the sets found plain seed 7322,
which holds two maximum sets (`[1,2,3,4]` and `[1,2,3,5]`) decided by the tie rule.
It is added to the seed list in `src/corpus-run/pass-consistency-apply-largest.unit.test.ts`.
The guard G8 (the order of the sets tried reversed in `largestValidSetOf`,
rebuilt) is red on exactly seed 7322,
and the restore is checked by checksum (`guards/G8-test.log`,
`guards/G8-restore.log`).
The assertion is a pin of each seed and no invariant:
a crafted page (`s208/tie-probe.ts`) makes a maximum set that is not the earliest,
because the check stage lets the later of two overlapping gains of one slice stand before the exact search runs.
The lead decided that the tie choice need not be the earliest set,
that a maximum set chosen deterministically is accepted,
and that no header may claim "earliest".
The tenth review confirmed the choice deterministic
(its tie probe run twice is byte for byte equal)
and found one header that still claimed it (B558).

## B555: the corpus count in the header of `consistency-proven-apostrophes.ts` disagrees with n143's count

Found on 2026-10-14 in the lead's notes on n143's hand-back.

Outcome written on 2026-10-14.
Fixed by n144 in the header only;
uncommitted at the time of writing;
commit:
pending.

n144 recounted on the final build with `count-single.mjs` over the 92 pages,
and the output is byte for byte n143's (`final/count-single.out` in `s208`):
`'` 65,
`’` 1,125,
U+02BC 0,
`‘` 9,
backtick 1 and prime `′` 1.
That is 1,199 marks of the four forms.
The readings are attached 1,148,
trailing 33,
leading 1,
free 4 and joint 4,
which sum to 1,190,
the marks `'` and `’`;
proven trailing 27 and leading 1.
The header said 1,200,
and the difference of 10 is exactly the 9 marks `‘` and the one backtick,
which are single quotation characters and no apostrophe.
The header "What the corpus holds" of `src/consistency-proven-apostrophes.ts` now states the 1,190,
the split,
the 10 and how the count was taken (the script,
the 92 pages,
the prose view).
The tenth review reconciled its own count the same way (1,199 marks of the four forms against the header).
