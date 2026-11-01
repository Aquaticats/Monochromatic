# Audit ledger: the consistency pass, the tenth review

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

The tenth independent review of the pass (n146) and the fixes that answered it in n147.
The review confirmed that L1 to L6 of the ninth review were fixed on the ninth review's inputs.
For L1,
12,000 random second-slice wordings for each of two page kinds
(a page with a curly convention and one with a straight convention)
gave no difference between the page the proof reads and the page that ships,
and a positive control (the raw English of the lane) differed on 30 curly and 18 straight pages.
L2 was left unchanged and decided unreachable.
L3 was fixed on the simple generator,
and the generator with connectors found R1.
L4 and L5 were fixed.
L6 was fixed,
with a cost of about 1.8 to 2.2 for each doubling of the pages,
from one run each,
so the spread of the probe was not measured.
L7 was answered with a grid that replaces 77,512 of 200,000 pairs.
L8 found one wrong example in the comments (R6).
L9 was not rerun by the review.
The corpus (92 pages,
1,329 slices) read as before:
27 proven trailing marks and 1 leading in the page,
25 and 1 in the shipped page,
and no mark read differently from n143's list.
The replay read as expected,
and 122 adversarial pages through the whole pass lost or invented no quotation mark outside the decided gains.

The answers to the four open questions of n144 are filed with their subjects:
the four pages of the oracle that stayed failing are the oracle's error (B549),
the 170 `pairs` reasons are real and false (B556),
the tie choice is deterministic but one header claimed "earliest" (B554,
B558),
and the reason `unproves` was not produced by the review (B554).
The review also counted the diff:
117 tracked files changed and 93 untracked,
with whitespace-only hunks in 14 files,
and it did not check whether those hunks reflow older text.
n147 compared them and restored the one pure reflow it found
(`src/insertion-separator.ts`,
the `@param fragments` line of the joiner's TSDoc,
back to the two-line wrapping that HEAD has);
every other whitespace-only difference is the re-indentation of a structural change of the same work,
so it stays.
The review's own breaches are filed as M203.

The fixes of n147 are filed here with the finding each answers.
The defects that the independent re-review of those fixes found are filed in
[Audit ledger: the consistency pass, the re-review and the refusal run](audit-ledger-consistency-pass-re-review-and-the-refusal-run.md).
The rounds before are filed in
[Audit ledger: the consistency pass, the ninth review](audit-ledger-consistency-pass-ninth-review.md),
[Audit ledger: the consistency pass, the independent oracle and the rules for marks](audit-ledger-consistency-pass-independent-oracle-and-mark-rules.md)
and
[Audit ledger: the consistency pass, the fixes of the eighth review and the oracle residue](audit-ledger-consistency-pass-fixes-of-the-eighth-review-and-the-oracle-residue.md).

Written on 2026-10-14 (UTC) from the review (`s210/review.md`),
the hand-back of n147 (`s212/handback-n147.md`),
the re-review (`s214/review.md`) where it measured n147's work,
and the lead's notes of that day.
The labels R1 to R8 and L1 to L9,
the scenario labels of the review's probes and the guard labels G1 to G7 of n147
are the sources' own and are no ledger ids.
The guard labels collide between hand-backs,
so each entry says whose guard it names.
A path that starts with `src/` is under `package/module/translation-repair/` (moved on 2026-10-18 to the packages under
`package/translation-repair`).
Every function,
type and constant of the package named here was found declared in non-test source of the merged tree `w158`
by searches of declarations on the day of writing,
except those an entry says were removed or renamed.
The names that only a hand-back or a review gives (test names,
guard names,
the helpers of test fixtures,
the names of probes and the names of logs) were not searched.
The tree changed after n147 (n149 followed),
so the entries of the re-review say what they changed.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B556: a gain was blocked because the mark of another gain read free, and the stated reason was false

Found on 2026-10-14 by the tenth review (finding R1).

Outcome written on 2026-10-14.
Fixed by n147;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
possessives left unrepaired,
a place that keeps `kittens` beside a repaired `dogs’`;
never a wrong mark.
The finding of the later gain said that it "would read as a quotation mark",
which was false on the final page,
and the made set of gains was not the largest.

The evidence of the review:
on hand-built pages a first splice `kittens` to `dogs’` whose new mark stands beside 🐱,
©,
™,
the Cyrillic `Привет`,
the Greek `γάτα` or `→`,
and a second splice of `A cat naps.` to `dogs’`,
left both places `kept-current`.
Without the first splice the second is `replaced`,
and so it is with the first mark beside a letter `x`.
The connector generator of the fuzz showed it on whole pages:
of 8,000 markup pages 170 gave a `pairs` reason that was false on the final page
(equal to the 170 that n144 had seen and not investigated)
and 148 a made set that was not the largest;
of 16,000 pages 336 were false,
of 12,000 pages with touching connectors 364,
and the plain generator (4,000 pages) and the markup generator without connectors (8,000) gave none.
The smallest page was seed 4 with two splices:
`Bowl 1 of the family` to `Bowl 1 of the parentsʼ` and `Cat 1 of the family` to `Cat 1 of the parentsʼ`
in the slice `Then Bowl 1 of the family’Cat 1 of the family is empty.`.
Both were `kept-current`,
and the second alone was `replaced`.
The review examined the mechanism of seed 4 only;
the other 147 non-maximal pages were not classified.

The cause:
`screenSide` in `src/corpus-run/pass-consistency-apply-screen.ts`
builds one page with every screenable splice of a side made
and blocks every splice that is unproven there,
with the reason `pairs`.
`isScreenable` and `besideOf` asked only that the character beside the mark is no Latin letter or digit.
The proof (`kindOf` in `src/consistency-proven-apostrophes.ts`) reads the same mark as free,
a possible partner of every later mark,
when a symbol,
a pictograph,
a letter of another script or a single mark stands beside it,
and the screen read it as trailing.
The headers of the screen
(the page with all such splices made proves its mark exactly where the page with it alone would;
a splice blocked once stays blocked)
and of `src/corpus-run/pass-consistency-apply-readmit.ts` (each is true of the final page) were false by it.

The lead ruled:
the screen asks the proof's own test of whether a character parts a mark from a word,
and a splice whose mark would read free goes to a trial.

Fixed.
`isScreenable` ends by asking `partsMarkFromWord` (`src/consistency-mark-neighbours.ts`)
of every character beside the mark outside the words,
`besideOf` returns whole code points,
and the new `readsAsEdgeApostrophe` joins them.
The headers of the screen and of `pass-consistency-apply-readmit.ts` say what holds now.
n147 went one step beyond the ruling,
in the safe direction,
and the lead accepted it:
`readsAsEdgeApostrophe` also asks that the gain mark is apostrophe-like (`'`,
`’`,
`ʼ`) with a Latin letter or a digit directly inside it,
because a gain mark such as `‘`,
`‚` or `‹`,
or a mark with a space inside,
would otherwise be called screenable although the proof never reads it as an edge apostrophe,
and would block later splices.
One existing screen test,
the loose `‘` splice of B549,
changed its expectation:
the blocked list for a double and a single splice is now empty for both.

Tests:
`src/corpus-run/pass-consistency-apply-hand-built.unit.test.ts` is new
(rows for the emoji,
the copyright sign,
the trademark sign,
Cyrillic,
Greek and the arrow beside the mark,
a CJK row,
control rows,
and the page of seed 4 expecting the slice
`Then Bowl 1 of the family’Cat 1 of the parentsʼ is empty. He said 'quoted' twice.`
with the outcomes `kept-current` and `replaced`),
and `src/corpus-run/pass-consistency-apply-screen.unit.test.ts`
has the case of another script rewritten to the new truth,
a control that still blocks,
and a case of no Latin letter inside the mark.
Red logs of n147 (`s212`):
ten children of the hand-built file failed on the old build
(eight rows,
the CJK row and seed 4),
three cases of the screen file failed,
and the final hand-built file failed in 14 children.
The guards G2a (every mark screenable)
and G2b (any mark with a character inside reads as an edge apostrophe) of n147
are red in five children of the screen file,
the rows,
the CJK row and the page of seed 4.

Evidence on the final source:
the six rows of the review's `r13` page and the CJK row now give the first gain `kept-current`
and the second one `replaced`.
The control with a doubled straight mark outside (`kittens'`)
changed from `kept-current` to `replaced` for the second place,
which the ruling did not list but follows from it,
since the proof reads that `'` as the trailing mark of `kittens` on its own.
The three generators of `r02` read,
on markup with the maximality check (8,000 pages):
no false `pairs` reason (170 before),
no non-maximal set (148),
a maximum set on 8,000 pages (7,852),
`replaced` 4,872 against 4,702 and `kept-current` 27,301 against 27,471,
and every other tally equal.
On 16,000 pages the false reasons are gone (336) with 336 more `replaced`;
on 12,000 touching pages they are gone (364) with 365 more `replaced`,
and the extra page is a `kept-current` page whose reason was `pairs`
with the mark proven alone and unproven on the final page.
Nobody isolated that page,
so its cause is not shown,
and its direction (one more splice made,
none fewer) matches the other 364.
The fuzz of the re-admission on plain and markup pages was byte for byte unchanged,
since its fragments hold no mark beside a non-letter.
The cost probe on dropped splices differed between two runs of the same build by up to 8%,
and its rise was clear only at 500 slices (about 26 to 30%),
where the build time of the probe,
which the change does not touch,
rose more still;
n147 claims no change of cost either way.
The long runs (the fuzz generators,
the cost probe and the adversarial pages) were not repeated on the last build,
and n147 argues from the checksum of the source files that it equals the build they ran on.
The re-review (n148) wrote four generators of its own (18,500 pages in all)
and found the made set maximum on all but two,
with every stated reason true on the final page.
Those two hold a Han character in both splices of a slice,
the floors refuse the slice with both and pass it with the first,
and the readmit header says a refused slice keeps its wording whole.
The screen's wider quote set (`QUOTE_CHARACTERS`) can only send a splice to a trial,
and a hand-built page with `Hawaiʻi `,
`5′ `,
`"x" ` or `don’t ` in front of `dogs’` made both of its gains.

## B557: a gain was carried while the kept mark stood past a footnote reference, a tag, a link or a space

Found on 2026-10-14 by the tenth review (finding R2).

Outcome written on 2026-10-14.
Fixed by n147 for the shapes it named,
and widened by n149 (B564 to B567);
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
a doubled mark,
`dogs’’` or `dogs’¹’` where the source had one.
The proof calls the new mark proven and the old one free,
so nothing downstream objected.
This is the finding that wrote a wrong mark.

The pages (labels are the review's):
`The dogs’[^1]’ bowl`;
a straight `'` after `[^1]` that becomes `dogs’[^1]’`;
`*dogs’*[^1]’`;
`<b>dogs’</b>’`,
`<i>dogs’</i>’` and `<span class="x">dogs’</span>’`;
the reference link `[dogs’][1]’`;
the titled link `[dogs’](https://cats.example/k "the cats")’`;
`dogs’<br/>’`;
and `dogs’ ’` with a no-break space or `&nbsp;`.
Correctly refused already:
a literal `*`,
`**`,
`_` and `~~`,
an untitled link destination,
`)`,
`"` and `”`,
invisible characters and a comment.

The cause:
the refusal run was `TRAILING_REFUSAL_RUN` in `src/consistency-markup-runs.ts`
(the backtick,
`)`,
`]`,
`"`,
`”`,
`」`,
`』`,
and `*`,
`_` and `~`,
with a link's destination through `markPastRunOf`),
used by `gainsBesideKeptMark` in `src/consistency-splice-readings.ts`.
It had no end of a tag,
no footnote or reference label,
and no title in a link destination,
and the header of the module said otherwise.
The review did not count how often the shapes occur in the corpus (unverified).

The lead ruled:
the refusal run holds every character the prose view does not read as words,
the characters it holds now,
and horizontal white space on the line.

Fixed.
`isRunUnit` in `src/consistency-closing-marks.ts` also asks the unread units and `HORIZONTAL_SPACE`,
and `markPastRunOf` (same signature) steps over `](` destinations,
unread units and horizontal white space.
A line break,
a letter,
a digit,
a zero-width character or any other character ends the run,
and the far-end mark is read by `singleMarkAt`,
so it is one of the eight counted marks whatever it reads as.
`HORIZONTAL_SPACE` in `src/consistency-markup-runs.ts` is new and holds the space,
the tab and the no-break space.
`unreadUnitsOf` in the new `src/consistency-unread-units.ts` returns every unit that `proseViewOf` blanks,
and the units of an HTML comment and of a character reference
(n147 named `&nbsp;`,
`&#160;` and `&#xA0;`),
never white space,
and it remembers the last four English texts.
It was split out of `src/consistency-prose-text.ts`,
which had reached the `max-lines` rule,
and `rememberedViewOf` is exported from there.
One definition serves every caller:
`src/consistency-count.ts` calls it for `markPastBefore` and `markPastAfter`,
and `gainsBesideKeptMark` consumes them.
n147 chose the three references and the three spaces for the cases the ruling and the review showed.
The families around them are the findings B564 to B567,
and M208 records why.

Tests:
`src/consistency-closing-marks.unit.test.ts` (three cases of `markPastRunOf`),
the new `src/consistency-unread-units.unit.test.ts`,
`src/consistency-markup-runs.unit.test.ts` (`HORIZONTAL_SPACE`),
and the new `src/corpus-run/pass-consistency-shipped-marks.unit.test.ts`
with `src/corpus-run/consistency-shipped-page.test-fixture.ts`,
which holds the twelve doubled pages as page-level tests expecting `kept-current` and the sentence kept,
the refused controls and a plain-gain control.
Red logs of n147 (`s212`):
39 `[FAIL]` and 24 `[PASS]` lines in the first;
the twelve pages came out `replaced`,
the three new cases failed with empty marks,
and the prose-text file failed to import `unreadUnitsOf`;
a second log shows the missing export `HORIZONTAL_SPACE`.
The refused controls and the plain gain passed in the red run,
as controls must.
One expectation was corrected after the build:
a straight `'` after `[^1]` ships `’`,
because `restoreTypography` curls a straight mark after a closed span,
so the case carries the shipped sentence and still checks that exactly one mark stands.
Its test name still said that the mark is the one the archive held,
which is true in count and not in glyph;
n147 left the name to avoid another whole-suite run,
and n149 renamed it (B570).
Guards of n147:
G1 (`isRunUnit` back to the old characters) is red in the three cases and all twelve pages,
G5a and G5b (no unread units) in two cases,
and G6 (`HORIZONTAL_SPACE` holds the space only) in one case.

The corpus count:
`count-run-sites.mjs` asked,
at every word-edge offset (145,308) of every slice English of the 92 pages (1,329 slices),
which mark `markPastRunOf` names.
It is a proxy for the gains the wider run refuses.
The old run named 4 sites and the new one 50,
and the 4 are unchanged.
The 46 newly named sites are on 23 pages,
with the mark `’` in 43 and `‘` in 3:
32 leading sites where the word before ends in a kept mark and a space stands between
(`the parents’ |preference`,
`girls’ |clothes`),
4 trailing sites
(a space and a kept opening mark after the word,
and the elision `snow| ’round`),
and 10 trailing sites inside a link destination,
where no span can end,
so they never refuse a real gain.
So 36 of the 46 are at an offset where a span can end.
The lead recorded that effect as accepted.

The review's scenarios,
compared by n147 with its baselines (125 scenarios):
A06,
B05 and B16 (a spaced mark,
a literal `*` beside a mark) were `replaced` and are `kept-current`;
the twelve doubled pages are `kept-current` with the added finding `consistency-partly-carried`;
one scenario with a comment kept its outcomes and changed its finding,
since the comment is now a run unit and the refusal is reported as an edge refusal;
and the repairs of B556 are made.
The corpus replay rows are byte for byte unchanged,
and the proven marks are the same (27 trailing and 1 leading,
25 and 1 shipped).

## B558: a header still claimed that the re-admission keeps the earliest splices

Found on 2026-10-14 by the tenth review (finding R3).

Outcome written on 2026-10-14.
Fixed by n147;
uncommitted at the time of writing;
commit:
pending.

`src/corpus-run/pass-consistency-apply-readmit.ts` said
that among larger sets of one size the first found holds the earliest splices of the page.
The lead had decided (B554) that no header may claim "earliest",
since a crafted page of n144 makes a maximum set that is not the earliest.
The headers of `src/corpus-run/pass-consistency-apply-largest.ts` and of its test were already qualified.

Fixed:
the header of `pass-consistency-apply-readmit.ts` now says that the made set is a maximum set picked deterministically,
not promised to hold the earliest splices.
There is no red log,
since only prose changed;
n147 checked the wording against the code and against the maximality of the fuzz (8,000 of 8,000 pages).
The re-review found the wording accurate,
except that with more splices than `EXACT_SEARCH_LIMIT` (12)
the made set follows the order of the page and is not promised to be maximum,
which the last sentence of the same paragraph says.

## B559: the proof treats a leading `’` of any apostrophe-like form as a possible opener

Found on 2026-10-14 by the tenth review (finding R4).

Decision recorded on 2026-10-14;
no code change.

What a reader of the page sees:
no mark lost and none gained.
In `it was ’tis the dogs’ bowl` the readings are `7:leading` and `20:trailing`,
and the proof proves none;
the straight `'tis the dogs'` and `ʼtis` read the same.
With a curly `’tis` early on a page,
every later gain is blocked and the receiving place's own `dogs’` stays unproven;
with it after the gains,
only the gains after it are blocked.
A curly leading `’` is the usual glyph of an elision,
so the code withholds more repairs than a narrower rule would.

The code path:
`provenApostrophesOf` in `src/consistency-proven-apostrophes.ts`.
`APOSTROPHE_LIKE` treats `'`,
`’` and U+02BC alike,
a leading mark sets `seen.opener`,
a trailing mark is proven only while no opener has been seen,
and a leading mark is proven only when it stands after the last closer.
The code follows its own header,
which names a straight or closing curly mark or U+02BC that is leading or free as a possible opener.
The wording of D6 in the ledger (B540) and the oracle,
whose author left open that a curly `’` cannot open,
were narrower.

The lead decided:
keep the code,
because the header's rule is the safe direction.
The ledger wording of D6 is corrected in B540,
and the oracle's narrower choice is the oracle's own.
The shipped glyphs of an elision are B563,
B568 and B572.

## B560: the apply stage's gain check knew four single marks where the counter and the record reader knew eight

Found on 2026-10-14 by the tenth review (finding R5).

Outcome written on 2026-10-14.
Fixed by n147;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
nothing from the planner,
which never plans such a splice.
On hand-built splices ending `dogs‚` or `dogs‛` the planner said `unavailable`,
and the apply stage said `replaced` with the mark unproven.

The cause:
`gainedSidesOf` in `src/corpus-run/pass-consistency-apply-gains.ts` uses `bringsAtEdge`,
which used `holdsEdgeMark`,
which asked `isSingleMark` (four marks).
The counter uses `isCountedSingleMark` (eight,
with `‚ ‛ ‹ ›`),
and `requireSingleMark` in `src/corpus-run/artifact-two-lane-read-consistency-rendering.ts`
accepts exactly those eight.

Fixed.
`holdsEdgeMark` in `src/consistency-loose-marks.ts` asks `isCountedSingleMark`,
and its TSDoc and the header say eight.
The loose-marks test flipped its case of the low and angle marks to `true` and has eight rows,
and the hand-built test has eight rows in which the outcome is never `replaced` and the page is unchanged.
The old build failed them (`dogs‚`,
`dogs‛`,
`‚dogs` and `‛dogs` came out `replaced`);
the rows of `‹` and `›` already ended `kept-floor` on the old build,
so n147 logged them as controls.
The matrix probe moved four of its 32 lines:
the columns `bringsAtEdge`,
`holdsEdgeMark` and `holdsEdgeApostrophe` went from false,
false,
false to true,
true,
false,
and the outcomes of `‚ ‛` (`replaced`) and `‹ ›` (`kept-floor`) to `kept-current`,
since the splice is refused for its edge mark before the floors run.
The guard G3 of n147 (`holdsEdgeMark` back to four marks) is red in the flipped case and the four rows.

The review's table of the readers
(its part 4 and the re-review's third item)
found no input on which two readers disagree and a wrong mark ships.
The nearest is the wider quote set of the screen,
which can only send a splice to a trial.
One input is worth a note:
the receiving place `The kittens′ bowl is empty.` with the chosen `dogs’` ships `The dogs’′ bowl is empty.`,
because the prime is none of the eight;
it looks like `'` in many fonts,
and the re-review calls it a point about glyphs and no finding,
since the eight marks are decided.

## B561: a TSDoc example of `joinedMarksOf` was off by one

Found on 2026-10-14 by the tenth review (finding R6).

Outcome written on 2026-10-14.
Fixed by n147;
uncommitted at the time of writing;
commit:
pending.

The example in `src/consistency-closing-marks.ts` for `joinedMarksOf` on `Fill the **cat**'s bowl.` said it holds 15,
and it returns `[16]`.
n147 changed the example to `// holds 16` and checked it by a probe
(`probe-joined.mjs`,
output `[16]`),
and the re-review ran the example and found it holds.

## B562: a header cited a file that no longer exists

Found on 2026-10-14 by the tenth review (finding R7).

Outcome written on 2026-10-14.
Fixed by n147,
and corrected again by n149 (B570);
uncommitted at the time of writing;
commit:
pending.

`src/artifact-key-vocabulary.ts` cited `artifact-build.ts`,
which is now `corpus-run/artifact-two-lane-build.ts`.
The line predates the work of the consistency pass.
n147 changed it to say that `corpus-run/artifact-two-lane-build.ts` writes the keys out,
and dropped the clause that the file names this table in a comment,
because `rg` finds no mention of the table in it.
The re-review then found that the file writes none of the table's four keys,
so the cited file was still not where they are written.

## B563: a leading straight elision shipped as an opening quotation mark

Found on 2026-10-14 by the tenth review (finding R8,
observations outside the work).

Outcome written on 2026-10-14.
Fixed by n147 with a ruling that the re-review reversed in part (B568);
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
`restoreTypography` turned a leading straight `'tis` into `‘tis`,
an opening quotation mark where an elision stands.
The review also saw that page assembly moves the full stop in `“bye”.` to `“bye.”`.
The lead accepted the full stop as the package's punctuation style,
and the proof read the shipped page in both cases.
The review noted,
as limits of its harness and no findings of the code,
that headings inside a paragraph formed no group,
and that no "cannot happen" throw fired in about 60,000 apply-stage pages,
while whether one can fire at all is unverified.

The lead ruled:
a closed list of elisions and a decade (`'90s`) curl to `’`,
and every other leading mark stays as it was.

Fixed.
`isLeadingElision` in `src/quote-neighbours.ts` matches the closed list `ELISION_WORDS`
(`tis`,
`twas`,
`til`,
`em`,
`cause`,
`round`,
the `n` of `'n'` when a straight mark follows,
and a decade of two digits and an `s`)
as a whole word with the case folded.
`'bout`,
`'twere` and `'nother` are not on it.
`singleShape` in `src/nested-single-quotes.ts` returns `neither` for it,
and `countOpeningSingles` in `src/restore-typography.ts` skips it
while the main scan writes `’`.
n147 chose,
and the lead accepted,
the option that on a curly page a leading straight mark before a listed elision curls to `’`
whether or not it is paired
(M207 records that this was accepted without a probe of a paired quotation).
That flipped the existing expectation for an unpaired `'90s` to `’90s`,
and the test that pinned `'Tis the season` straight now uses `'Nap time, said the cat.`.
The re-review showed what the option does to a quotation that begins with a listed word (B568).

Tests:
`src/restore-typography.unit.test.ts` (the list,
paired and unpaired,
and the controls `'Mio'`,
`'tisane'`,
`'emperor'`,
`'rounders'`,
`'1990s'` and a straight page left alone)
and `src/nested-single-quotes.unit.test.ts`.
Both fail on the old build,
and the guard G4 of n147 (`isLeadingElision` always false) is red in the elision case and the row of `'90s`.
The corpus count flattens each curly single mark of the 92 pages (1,631 marks) to a straight one,
runs `restoreTypography` with the flattened page as the replacement and the original as the convention,
and compares each mark with the original,
because the pages hold almost no straight elisions.
No mark would curl to `‘` before an elision word,
before or after.
One unpaired elision newly curls,
`Guard her from snow ’round the unforgiving stars`,
which was left straight and is now `’`,
as the page holds it.
Everything else is equal:
71 marks differ from the original before (63 straight to `’` and 8 `’` to `'`)
and 70 after (63 and 7).
In the review's scenarios `'tis` now ships `’tis`
(it shipped `‘tis` in one of them).
