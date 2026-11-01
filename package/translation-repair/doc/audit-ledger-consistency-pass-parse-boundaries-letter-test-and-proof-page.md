# Audit ledger: the consistency pass, parse boundaries, the letter test and the proof's page

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

The defects that the second fix round of the eighth review (n134) and the rounds after it (n136,
n137 and n139) found,
or were told to fix by the lead:
the stack exhaustion that was read as the parser's refusal wherever it came from,
the letter test that was widened for every caller of the package,
the reason a left-out gain states,
and the page the proof reads,
which was a reconstruction of the published page.
The bodies of B513 to B519,
which these rounds closed,
are in
[Audit ledger: the consistency pass, the fixes of the eighth review and the oracle residue](audit-ledger-consistency-pass-fixes-of-the-eighth-review-and-the-oracle-residue.md).
The ninth review and its fixes are filed in
[Audit ledger: the consistency pass, the ninth review](audit-ledger-consistency-pass-ninth-review.md),
and the independent oracle in
[Audit ledger: the consistency pass, the independent oracle and the rules for marks](audit-ledger-consistency-pass-independent-oracle-and-mark-rules.md).
The fixes before these are filed in
[Audit ledger: the consistency pass, one reading of every mark](audit-ledger-consistency-pass-one-reading-of-every-mark.md)
and
[Audit ledger: the consistency pass, stack depth, the floors fixed point and the page form](audit-ledger-consistency-pass-stack-depth-floors-fixed-point-and-page-form.md).

Written on 2026-10-14 (UTC) from the hand-backs of n134 (`s192/handback-n134.md`),
n136 (`s194/handback-n136.md`),
n137 (`s198/handback-n137.md`) and n139 (`s200/handback-n139.md`),
and the lead's notes of that day.
The labels J3 to J8,
the guard labels and the item numbers are the sources' own and are no ledger ids.
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
The tree changed after these entries were written (n144,
n147 and n149 followed),
and the entries of those rounds say what they changed.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B522: a stack exhaustion raised after the parse was read as the parser's refusal

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n134,
which reported that the conversion of B515 also reads an exhaustion raised by other code inside the same `try`
(reported,
not narrowed).
The lead decided the same day to narrow it.

What a reader of the page would have seen:
no page was shown to change.
A function that parses and then reads the tree could run out of stack after the parse had returned,
and answer as it does for a refused parse:
`splitTargetOnlyRun` returned the whole run and `tenseCandidateGroups` returned no candidates
(n139's red run).

The cause:
the conversion of B515 sat in `requireMdxRefusal` and `requireMarkdownRefusal`,
which the catch of each parse boundary called.
A catch cannot tell where an exhaustion came from,
and 21 boundaries had a `try` that held more than the parse.

The lead decided:
a boundary converts only an exhaustion raised by the parse call itself,
and any other code's `RangeError` propagates.
For consistency the lead added that `source-break-display.ts` calls `sourceBreakSpans` directly
and that `proseBodyTree` drops its redundant outer wrapper.

Fixed in three steps.
n136 added `withMdxExhaustionRefused` and `withMarkdownExhaustionRefused` to `src/parse-mdx.ts`,
each of which calls the parse and converts only its exhaustion,
and sent `requireMdxRefusal` and `requireMarkdownRefusal` back to refusal-only.
The 21 boundaries now wrap their parse call and keep their `try`
(among them `blockBreaksIn`,
`readSliceSkeleton`,
`foldSoftBreaks`,
`introducedStructuralRegressions`,
`archiveContributorNameForms`,
`fragmentReadingOf` and `activeFootnoteMarkers`).
`parseDocument` now raises `MarkdownParseError` for the exhaustion of its plain fallback where it raised a raw `RangeError`,
and so does `assembly-regressions` for the incumbent's exhaustion;
the lead accepted both.
`disjointSpans` moved to `src/footnote-span-union.ts`,
because `src/footnote-parsed-spans.ts` reached the 300-line limit.
n139 made `parseSliceBody` wrap its own parse,
so `splitTargetOnlyRun` and `tenseCandidateGroups` call their helper directly
and let the later walk's exhaustion propagate;
the brief had not asked for this production change,
and the hand-back reports it for the lead to accept,
which the lead did.
n140 made `sourceBreakDisplay` call `sourceBreakSpans` directly
and `proseBodyTree` call `parseBodyTolerant` directly,
and narrowed seven more wrappers whose `parse` argument did more than parse,
in `src/consistency-prose-text.ts` (`treeOf`),
`src/translate-skeleton.ts`,
`src/line-structure.ts`,
`src/translate-address-drop.ts`,
`src/consistency-block-breaks.ts`,
`src/soft-break-fold.ts` and `src/assembly-regressions.ts`.
`s202/wrapper-calls-final.txt` lists the 12 calls that remain,
and each `parse` argument is `parseMdxBody` or `parseMarkdownBody` and nothing else.

Red evidence:
n136's guard G2 (the converting catch put back) fails 13 site tests and the 2 reversed `parse-mdx` tests,
and its guard G3 (both wrappers rethrow) fails every site's parse-side test,
the 4 wrapper tests,
the `consistency-prose-text` exhaustion test and the `parseDocument` test.
n139's two propagation tests failed before the production change with "Expected walk to throw,
but it returned"
(`s200/item2-run1.log`),
and its guard G2 (`parseSliceBody` unwrapped,
and the wrapper removed from `parseStrictly` in `inspect-paragraph.ts` and from `grammarAcceptsDocument`) fails three tests.
n140's test that `sourceBreakDisplay` lets the exhaustion of its walk propagate failed first with the same message
(`s202/red-item5.log`).
Not demonstrated by n136:
the wrappers at `inspect-paragraph` and in the tag-name fixture,
and the non-parse red of `proseBodyTree`,
for which no seam exists;
n139 added direct tests for `inspectParagraph` and `grammarAcceptsDocument`,
which cannot be red before a code change and are red only under its guard G2.

## B523: the letter test was widened for every caller in the package

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 from the list of other callers in the hand-back of n136.
The brief had asked n136 to make `continuesLatinWord` accept the letters whose compatibility form is Latin (B518).

What a reader of the page would have seen:
nothing on the corpus.
`continuesLatinWord` is also the word-boundary test of the glossary and phrase boundaries (`word-bounds.ts`),
of quote locating (`locate-quote.ts`),
of alignment,
of preservation tokens,
of ballots and of archive name runs.
Each of those would have read a compatibility letter as part of a word at a boundary.
n136 measured the corpus and found two characters that the widened test admits and the old one did not:
`𝓠 𝓾 𝓪 𝓷` (7 each,
on the page of mikaela_khara) and `₂` (2,
in the front matter `name` of Aniloviraw).
The glossary and alignment passes were not run on mikaela_khara.

The cause:
the brief widened a shared predicate for the consistency pass without a list of its callers and a measurement of each.
The widening also admitted `™`,
`²` and `№` as word letters,
and it opened an asymmetry:
`isLatinWordCharacter` opens a run and refuses `𝓠`,
while `continuesLatinWord` continued through it.
The lead recorded the brief's mistake (M198).

The lead decided on 2026-10-14:
the consistency pass gets its own neighbour test,
used by every consistency reader and by the screen,
and `continuesLatinWord` goes back to its behaviour for every other caller.

Fixed by n139.
`continuesLatinWordOrCompatibilityForm` in `src/latin-letters.ts` carries the widening,
and `continuesLatinWord` is `isLatinWordCharacter` or a combining mark again.
The readers switched are `isEdgeApostropheAt`,
`looseMarkPointsOf` and `edgeClassOf` in `src/consistency-loose-marks.ts`,
the two readers that classify a mark in `src/consistency-proven-apostrophes.ts`,
the six calls of `src/consistency-word-joints.ts`,
`apostropheBeforeLetter` in `src/consistency-closing-marks.ts`,
and `edgeTablesOf` in `src/consistency-edge-marks.ts`,
which the brief did not name and which calls the test for a mark's neighbour.
The screen in `src/corpus-run/pass-consistency-apply-screen.ts` was left on the old test on purpose,
and the lead switched it at the merge of the two trees.
Tests and headers that said "one letter test" now say "the neighbour test of the consistency pass".

Red evidence:
`s200/red-a.log` shows the new refusal test failing on a build without the change
(`continuesLatinWord` returned true for a mathematical script capital Q),
and `s200/red-readers.log` shows eight reader tests failing when the predicate is added but the readers are not switched.
n139's guard G1 (the new predicate made equal to `continuesLatinWord`) fails 13 tests across 9 files.
A census over every code point,
the empty string and each lone surrogate (1,114,113 inputs) finds `continuesLatinWord` equal to
`isLatinWordCharacter` or a combining mark for all of them,
and a census of the 92 pages (431,558 characters) finds no character whose result differs from before n136.
The corpus is unchanged by the confinement:
27 trailing and 1 leading proven marks,
and `mikaela_khara` at offset 4703 attached.
Four tests (`bringsEdgeApostrophe`,
`readingsChangeOutside`,
`bringsAtEdge` and `pageApostropheFormOf`) were written after the readers were switched
and are red only under guard G1 (M193).

## B524: a gain left out stated the wrong reason when the floors refused its slice

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n134,
whose design note says that in its probe row X2 the reason of slice 1's gain is `pairs` and not `floors` as the brief expected.
The lead decided the same day that the floors' refusal comes first.

What a reader of the page would have seen:
the finding for a place that kept its words said
that the apostrophe it carries in would read as a quotation mark there,
since another quotation mark of the page could pair with it.
It was true on the final page,
and it left out the reason that holds whatever the other gains do,
which is that the floors refuse the slice with the replacement.
After the fix the finding reads
that the consistency replacement was left out because the floors refuse the slice with it,
naming the refusal `translate-refused-declared-name`
(archive text carries a name and the replacement does not;
the archive text is kept),
and that the place keeps its words.

The decision:
a gain whose slice is refused on the final page states that refusal first,
and the other reasons follow in the order overlaps,
pairs,
unproves.

Fixed by n137.
`trialOf` in `src/corpus-run/pass-consistency-apply-trial.ts` checks the reasons in that order,
with `floorsWithSplice` and `madeWithSplice` exported,
and `refusalsOfBlocked` gives the floors' reason to a splice that the screen blocked
(it moved to `src/corpus-run/pass-consistency-apply-order.ts` in n141).
The readmit test asserts the exact finding text of probe row X2 of `p15-findings`.

The cost:
the `dropped` variant of the cost probe `p06` went from 24,
38,
76,
197 and 431 ms to 211,
423,
862,
1,767 and 3,758 ms (125 to 2,000 slices),
about nine times,
because the floors now run once for each dropped gain.
The growth stays linear,
with 2.00 to 2.13 per doubling in single runs,
and the lead decided to keep the order.
n141 profiled it and found no floors verdict repeated for the same slice and edits
(B531).

Red evidence:
n137's red log `s198/red-items-1-2.log`,
its guard G10 (the trial order),
G11 (`refusalsOfBlocked`) and G17 (`madeWithSplice` overlaps).
The pairs-to-floors shift in the random-page probe `p20` of n141 (plain pages 13,179 to 8,979 for `pairs`,
and 320 to 4,519 for `floors`) is a reclassification:
the three reason counts together change only by the one place that the exact search of B528 re-admits.

## B525: the proof read a reconstruction of the published page

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n134,
which documented two approximations in its `pageOfSlices`:
line endings at the edges of a span,
and the separators of anchor insertions.
The lead decided the same day that the proof's page is built by the code that assembles the published page.

What a reader of the page would have seen:
no page whose marks read differently.
The ninth review's probes of the differences (extra newlines at a span's edges,
CRLF pages that mixed a lone `\n` with `\r\n`) found the proven sets equal modulo the shift of offsets.
The risk was a difference nobody had listed,
and the ninth review found one (the glyphs of B526).

The decision:
no reconstruction.
The proof reads the page that `placeSlices` lays out.

Fixed by n137.
`laidOutPage` in `src/consistency-page-english.ts` calls `placeSlices` in `src/splice-slices.ts`,
which is the assembler publishing uses:
`spliceSlices` is its `.text`,
and `src/corpus-run/publish-fixed.ts` calls it with the replacements from `shippableReplacements`.
`pageOfSlices` returns a `ReadPage`.
`src/splice-slice-places.ts` is new (`contentWrite`,
`placementsOf`),
split out because `src/splice-slices.ts` passed the line limit,
and `fragmentCut` and `composedInsertion` in `src/insertion-separator.ts` return the text and its places.
`composeInsertion`,
`textBetweenSlices` and `SliceSpan` were removed as unreached.

Red evidence:
`s198/red-item-3.log`.
The property test in `src/consistency-page-english-assembled.unit.test.ts` builds 400 seeded pages
(sealed text,
anchors,
edge line endings,
silent slices,
front matter,
CRLF) and asserts that the proof's page equals the text of `spliceSlices` character for character with the front matter blanked,
with a coverage control.
n137's guards G14 (`layout.silent`),
G15 (`placementsOf` growth) and G16 (the cut start of `composedInsertion`) fail named tests.
The whole suite found one failing test,
which built a preparation that production cannot make (a slice dropped without renumbering),
and `assertSliceIndexing` in `placeSlices` stopped the pass on it;
n137 renumbered the slices in that test and ran its file alone green.
n137 ran its guard experiments on the build before its last refactors,
and says so;
the final tree is covered by the lint,
the scans and the suite.
