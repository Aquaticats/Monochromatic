# Audit ledger: the consistency pass, the re-review and the refusal run

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

The independent re-review of the tenth review's fixes (n148) and the fixes that answered it in n149.
The re-review confirmed R1 (the screen) on every generator it wrote and on the three of the tenth review:
in 18,500 pages of four generators of its own the made set was the largest in all but two,
which the floors' whole-slice ruling explains,
every stated reason was true on the final page,
and the tallies of the tenth review reproduced to the unit.
It did not close R2 (the refusal run):
four ways to a doubled mark still shipped,
a character reference that is itself a mark,
a backslash escape,
the Unicode spaces other than three,
and a soft line break inside a paragraph.
It found that R8 (the elisions) left the closing mark of a quotation straight on a curly page
and read ordinary words on the list as elisions.
The replay read as expected.

The refusal run is the stretch of text that a gain looks across
to decide whether a kept mark stands next to the arriving one.
The lead's first ruling for it was a list of named characters
(three references,
three spaces,
"on the line"),
and each family around a name shipped a doubled mark (M208).
The ruling of this round states it by what the renderer shows between the two marks:
a reference that decodes to a quotation mark makes the page unavailable,
an escape is read as the character it escapes,
the white space is the proof's own,
and a single line break inside a paragraph shows as a space.
A fifth part of that model,
the container markers at the start of a line (B574),
was found by n149 and built by n151.

The fixes of n147 and the findings they answer are filed in
[Audit ledger: the consistency pass, the tenth review](audit-ledger-consistency-pass-tenth-review.md).
The rounds before are filed in
[Audit ledger: the consistency pass, the ninth review](audit-ledger-consistency-pass-ninth-review.md),
[Audit ledger: the consistency pass, the independent oracle and the rules for marks](audit-ledger-consistency-pass-independent-oracle-and-mark-rules.md)
and
[Audit ledger: the consistency pass, the fixes of the eighth review and the oracle residue](audit-ledger-consistency-pass-fixes-of-the-eighth-review-and-the-oracle-residue.md).

Written on 2026-10-14 (UTC) from the re-review (`s214/review.md`),
the hand-back of n149 (`s215/handback-n149.md`),
and the lead's notes of that day.
The labels Q1 to Q8,
the scenario labels of the probes (E1 to E8,
B1 to B8,
W1 to WA,
K1 to K7,
Z04,
A11c) and the guard labels A to D of n149 are the sources' own and are no ledger ids.
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
The container markers were built by n151.
The review of n149 (n150),
the round that answered it
and the rounds after them are filed in
[Audit ledger: the consistency pass, what the refusal run steps over](audit-ledger-consistency-pass-what-the-refusal-run-steps-over.md).

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B564: a character reference that is itself a single mark was stepped over, and a doubled mark shipped

Found on 2026-10-14 by the re-review (finding Q1).

Outcome written on 2026-10-14.
Fixed by n149;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
two marks.
The receiving place `The kittens&#39; bowl is empty.` with the chosen words `dogs’` was `replaced`,
and the page shipped `The dogs’&#39; bowl is empty.`.
The same held for `&apos;`,
`&rsquo;`,
`&lsquo;`,
`&#8217;`,
`&#x2019;`,
`&#x27;` and `&#039;`,
directly after the words and after a space,
and on the leading side
(`He ate &#39;the kitten now.` with the chosen `’tis the cat` shipped `&#39;’tis the cat`).
All 32 pages of the probe shipped it.
The package's own parse reads `dogs’&#39;` as `dogs’'`,
`dogs’&rsquo;` as `dogs’’` and `&#8217;’tis` as `’’tis`.
No rendered page was opened (unverified),
and the parse is the evidence.
The re-review did not run the old build,
so it claims no regression,
and it did not count references in the 92 pages.

The cause:
`commentAndReferenceUnitsOf` in `src/consistency-unread-units.ts` marked every `&name;`,
`&#n;` and `&#xh;` as an unread unit,
including a reference that decodes to a quotation mark.
`isRunUnit` in `src/consistency-closing-marks.ts` steps over every unread unit,
and the far end is read by `singleMarkAt`,
which reads one literal character.
The proof then parts the new mark on the `&` (`partsMarkFromWord`),
so nothing downstream objected.
No other reader decodes a reference either:
the review found no decoding of `&#39;` or `&rsquo;` outside `src/footnote-unpositioned-runs.ts`,
which decodes brackets only.
The step over a reference had been added for `&nbsp;`,
`&#160;` and `&#xA0;`,
and the same step hid a reference that is the mark.

The lead ruled:
any character reference that decodes to a quotation-like character makes every gain of that page `unavailable`,
with a named reason.

Fixed.
The new `src/consistency-quote-references.ts` has `quoteReferencesIn`.
It knows 23 named references
(checked against the parser's table with `entity-table.mjs` in `s215`),
decimal references of up to 7 digits and hexadecimal ones of up to 6,
requires the `;`,
and counts only a reference that the prose view keeps:
not one in code,
a tag attribute,
a link destination or title,
and not one in an HTML comment.
`src/consistency-unread-units.ts` exports `referenceEndOf` and has a new `commentUnitsOf`.
`planConsistencyEdit` in `src/consistency-splice.ts` takes `pageReference`,
and a gain (chosen words that bring an edge apostrophe) is `unavailable` when the page holds one.
`src/corpus-run/pass-consistency-decide.ts` passes it,
from the page text and otherwise from the slice text.
`src/corpus-run/pass-consistency-edge-refusal.ts`
and `src/corpus-run/artifact-two-lane-consistency.ts` have a new kept reason,
`edge-reference-mark`,
in the closed set,
and the reader accepts it.
The reason is documented in `src/corpus-run/artifact-two-lane-read-consistency-span-shared.ts`.

Tests:
the page tests in `src/corpus-run/pass-consistency-shipped-marks.unit.test.ts`
for the references (E1 to E8 at four positions),
for the page-wide rule (P1 to P4)
and for plain references (`&amp;` and `&nbsp;` do not refuse),
the planner test in `src/consistency-splice.unit.test.ts`,
the tests of `src/corpus-run/pass-consistency-edge-refusal.unit.test.ts`,
and the new `src/consistency-quote-references.unit.test.ts`.
Red logs of n149 (`s215`):
a missing export,
the splice test,
the edge-refusal test,
and a page-test log with 58 failures across all the items of the round.
Guard A (the planner's reference check and the `edgeRefusalOf` branch removed) fails 3 edge-refusal tests,
the splice test and 36 children of the shipped-page test;
guard D (the prose and comment filters of `quoteReferencesIn` removed) fails 2 tests.

Evidence:
the re-run of the probe on the final build,
97 scenarios,
gave 46 identical and 51 changed,
with 51 doubled pages before and 2 after;
the 32 pages of E1 to E8 went from replacing places 2 to 4 to keeping them,
the page unchanged.
No page of the 92 holds a reference the planner counts.
A raw search finds one page with any character reference at all (`dogesir_`:
`&apos;` and `&#33;` twice),
inside a translator comment that the page does not show,
so it is correctly not counted;
the positive control finds `&apos;` and `&#39;` and not `&amp;` or `&nbsp;`.
The ruling does not cover a reference inside the chosen words themselves,
which are quoted from a place of the same page,
so the page-wide check should already see it;
the review of n149 is to confirm or break that (open).

## B565: a backslash escape ended the refusal run, and `dogs’\*’` shipped

Found on 2026-10-14 by the re-review (finding Q2).

Outcome written on 2026-10-14.
Fixed by n149;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
`dogs’*’`.
`The kittens\*’ bowl is empty.` (and the same with `\_`,
`\]`,
`\\` and `\~`) with the chosen `dogs’` shipped `The dogs’\*’ bowl is empty.`,
and a scenario with `\*kittens\*’` shipped `\*dogs’\*’`.
It is the shape of the refused literal `*` between two marks,
with the escape character in front of the `*`.
The package's parse gives the text `The dogs’*’ bowl`.

The cause:
`isRunUnit` in `src/consistency-closing-marks.ts` ended the run on `\`,
because the backslash is in none of the sets of the run.
The prose view keeps `\*` as text,
so the phrase of the ruling "every character the prose view does not read as words" does not name it,
and the reason of the ruling,
that the eye sees a second mark,
does.

The lead ruled:
the run reads `\x` as `x`,
and an escaped single mark is a mark.

Fixed.
The new `src/consistency-run-steps.ts` has `startsEscape`,
and `isRunUnit` steps forward over it.
`\*`,
`\_`,
`\~`,
`\]`,
`\)` and `\\` are stepped over,
and `\'` is a mark that ends the run.
Tests:
the escapes in the shipped-marks test,
the escapes in `src/consistency-closing-marks.unit.test.ts`,
and the new `src/consistency-run-steps.unit.test.ts`,
which n149 wrote after the code when the scan for tests of a module's own demanded it (M206).
Red:
58 of 78 cases of the probe differ on the old run (`run-cases-before.txt`),
and B1 to B5 and the scenario with the stars failed in the page test.
Guard B (the escape clause and the break clause removed from `isRunUnit`,
and `HORIZONTAL_SPACE` cut to five characters) fails the set test of the markup runs,
four children of the closing-marks test and 19 children of the shipped-page test.
The residual is B573.

## B566: the Unicode spaces other than three ended the refusal run, and `dogs’ ’` shipped

Found on 2026-10-14 by the re-review (finding Q3).

Outcome written on 2026-10-14.
Fixed by n149;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
`dogs’ ’`,
the page that the ruling of the tenth review refused,
with a narrower or wider space.
`The kittens<X>’ bowl is empty.` with X one of U+2009,
U+2003,
U+202F,
U+205F,
U+3000,
U+1680,
U+2002,
U+200A,
U+2007 and U+FEFF,
and the chosen `dogs’`,
was `replaced` and shipped `dogs’<X>’` for all ten.
U+0085 and the zero-width characters were refused,
and not by the run;
the reviewer's reading is that the proof refuses them because `\s` does not match them,
which it did not isolate with a probe.

The cause:
`HORIZONTAL_SPACE` in `src/consistency-markup-runs.ts` held three characters
(the space,
the tab and the no-break space),
against the ruling's "horizontal white space on the line" and against the proof's own parting test,
the white space of JavaScript's `\s`,
which holds all of these.
The hand-back of n147 said that the thin and ideographic spaces end the run on purpose,
and the ruling does not say so.

The lead ruled:
the white space of the run is the proof's,
`\s` without the line terminators.

Fixed.
`HORIZONTAL_SPACE` is now the 21 characters that JavaScript trims as white space,
less the line terminators,
and the header is rewritten.
The test in `src/consistency-markup-runs.unit.test.ts` compares the set with one derived from `trim()`
(red in `red-markup-runs.log`),
and the shipped pages W1 to W9 and WA,
with variants that put the space before the words,
are in the page test.
Four of them (U+202F and U+FEFF,
before and after) run with a new option of the fixture,
`silentWriters` (`Case.silentWriters` in `src/corpus-run/consistency-shipped-page.test-fixture.ts`):
the scripted writer of the shared fixture echoes the paragraph the sheet shows,
and `src/invisible-variants.ts` maps U+202F to a space and drops U+FEFF in the sheet,
so with the writers answering,
the writer lane replaces the place.
That rewriting is existing behaviour outside this ruling,
and the lead left it as it is.
The run itself returns `’` for every one of the spaces (`render-debug.txt`),
and the probe whose writers throw shows W3 and WA kept.

## B567: a single line break inside a paragraph ended the refusal run, and `dogs’` then a mark on the next line shipped

Found on 2026-10-14 by the re-review (finding Q4).

Outcome written on 2026-10-14.
Fixed by n149 for the line breaks,
and the container markers at a line start are B574;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
`The kittens` and a new line and `’ bowl is empty.` in one paragraph,
and the same with two spaces before the break,
a backslash before it,
or a space after it,
with the chosen `dogs’`,
was `replaced` and shipped `The dogs’`,
a new line and `’ bowl is empty.`.
In HTML a soft break renders as a space,
so a reader sees `dogs’ ’`,
the page the ruling refused
(an inference from the CommonMark rule;
no page was rendered).
A hard break shows the mark at the start of the next line,
which is weaker,
but the ruling refuses the same visible break written as a tag (`<br/>’`),
so the same break shipped or was refused by how it is spelled.
A blank line starts a new paragraph and was correctly no case.

The cause:
`isRunUnit` ended the run at a line break,
as the ruling of the tenth review says ("on the line").
It was a decided rule,
and the re-review reported it under the instruction
to flag a decided rule that harms a reader in a way the ruling did not weigh:
the line of a paragraph in the source is not the line a reader sees.

The lead ruled,
amending the wording "on the line":
a single line break inside a paragraph renders as a space,
so the run continues across it,
and a blank line ends it.

Fixed.
`isSingleBreakAt` in the new `src/consistency-run-steps.ts`
and `insideParagraphAt` in the new `src/consistency-paragraphs.ts`
(the paragraph ranges of the parse;
an English that the grammar refuses counts every offset as inside).
A soft break,
two spaces and a break,
a backslash and a break,
and a carriage return with a line feed continue the run,
and a blank line ends it.
The pages K1 to K4 failed and now pass,
and K7 (the blank line) stays carried.
Tests:
the breaks in the shipped-marks test,
the closing-marks tests,
and the new `src/consistency-paragraphs.unit.test.ts` and `src/consistency-run-steps.unit.test.ts`.
Red logs:
`red-closing-marks-paragraph.log`,
`red-paragraphs.log` and the page-test log.
Guard C (the paragraph check of `isSingleBreakAt` removed) fails the run-steps and the closing-marks tests.

## B568: a quotation that began with a listed elision shipped its opening mark as `’` and its closing mark straight

Found on 2026-10-14 by the re-review (finding Q5).

Outcome written on 2026-10-14.
Fixed by n149;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen,
with a curly convention (`It’s fine.`) in the probe:
`'Tis the season,' she said` became `’Tis the season,' she said`,
a closing straight mark on a curly page.
`'Em and me,' they said`,
`'Round and round,' went the song.`,
`'90s kids,' he said`,
`'Til the end,' she said`,
`'Twas the night,' he read.` and `'Cause I said so,' she said.` gave the same.
`'Em and me' they said` became `’Em and me’` and `He said '90s kids' twice.` became `’90s kids’`,
a quotation opened with a mark of the closing form.
`She said 'Round things are best' and left.` became `’Round things are best’`,
an ordinary quotation read as an elision because the list matches the whole word with the case folded.
`She said "he said 'Tis the season,' too."` ended in a straight mark again.
Right already:
`“'Tis the season,” she said`,
`the '60s and '00s`,
`rock 'n' roll`,
the controls `'tisane'`,
`'emperor'`,
`'rounders'`,
`'1990s'` and `'Mio'`,
and a straight page (the convention `It's fine.`),
unchanged for all 36 inputs.
The tests of n147 pinned no quotation that begins with a listed word.
The corpus count of n147 found one newly curled mark and none of this shape.

The cause:
the elision branch of `restoreTypography` (`src/restore-typography.ts`) wrote `’` for the leading mark,
and `singleShape` in `src/nested-single-quotes.ts` returned `neither` for it,
so the closing mark of that quotation had no opener to pair with,
and `restoreTypography` leaves such a mark straight unless a word character stands before it.
Before n147 `singleShape` returned `opening` for any such mark,
so the pair curled as `‘Tis … season,’`
(read from the source of HEAD;
the old build was not run).

The lead ruled,
reversing the acceptance of the option that n147 chose (M207):
the elision curls to `’` only when it is unpaired,
and a pair curls as a quotation as before.

Fixed.
`isElisionPair` in `src/quote-neighbours.ts` is new,
and the comment there no longer names a review.
`src/nested-single-quotes.ts` lets the elision openers pair again,
and `'n'` never opens.
`countOpeningSingles` in `src/restore-typography.ts` counts the unpaired openers that are no elision,
and its TSDoc says what happens to a quotation that begins with a listed word.
The probe against the re-review's outputs shows the quotations
that begin with a listed word and have a later straight mark curling as a pair
(`'Tis the season,' she said` becomes `‘Tis the season,’ she said`,
and `'Em`,
`'Round`,
`'90s`,
`'Til`,
`'Twas`,
`'Cause`,
`'tis` and `'TIS` likewise),
where the re-review had them half curled.
The unpaired `’round`,
a `’tis` in mid-sentence,
`’60s` and `rock 'n' roll` are unchanged.
Red:
`red-restore-typography.log`.
The guard on `countOpeningSingles` (the check of pairing removed) passed with no test,
so n149 added `She said 'Mio' and the cats' bowl was full.` to `src/restore-typography.unit.test.ts`,
and the guard then fails.
Guard C also removes `isElisionPair`,
which fails the restore test.
The corpus recount is byte for byte equal to n147's:
92 pages,
1,631 single marks flattened,
70 mismatches (63 straight to curly and 7 curly to straight),
and the one elision of the corpus (`’round` in `Susiethegamer`) curls to `’`.
The consequence that the lead accepted is B572.

## B569: the wider refusal run withholds repairs where no mark would be doubled (accepted)

Found on 2026-10-14 by the re-review (finding Q6).

The lead accepted it on 2026-10-14:
the direction is safe,
and the cost is repairs not made,
as decision D1 allows.

All of these were `kept-current`:
`The kittens ‘normal’ bowl`,
a space and then an opening single quotation mark
(`dogs’ ‘normal’` is ordinary typography;
three of the four trailing sites that n147 counted as newly refused on the corpus are of this shape,
`as| ‘normal'`,
`say| ‘it’s okay'` and `Jensen’s| ‘Gradiva’`);
the same with a straight `'normal'`,
which the shipping step curls as a pair,
while `“normal”` is allowed;
inline code,
a bare address and an autolink between the marks
(the code and the address are text a reader reads,
so `dogs’ abc def ’` has words between the marks,
and the reader would not call it a doubled mark);
and a table row,
where the run reaches across the pipe between two cells
(the reviewer's reading;
not isolated).
No wrong mark is written,
and the re-review rated it low.

## B570: documentation that the code did not make true

Found on 2026-10-14 by the re-review (finding Q7).

Outcome written on 2026-10-14.
Fixed by n149;
uncommitted at the time of writing;
commit:
pending.

The re-review listed six sentences:
the header of `src/consistency-markup-runs.ts` said that `unreadUnitsOf` is in `src/consistency-prose-text.ts`,
which n147 had moved to `src/consistency-unread-units.ts`,
and that a line break,
a letter,
a digit and a single mark end the run,
while every other character ends it too;
the finding text of the reason `unproves` said
that the apostrophe another replacement carries in "would read as a quotation mark",
and on the author's page the first mark reads `attached` (`The dogs’xcats’ nap.` gives `20:attached`),
a contraction,
so the condition (the first mark loses its proof) was true and the wording was not;
the comment in `src/artifact-key-vocabulary.ts` said
that `corpus-run/artifact-two-lane-build.ts` writes the table's keys out,
and that file names none of `changedSliceIndices`,
`withdrawnSliceIndices`,
`sliceCritics` or `sliceIndex`;
a comment in `src/quote-neighbours.ts` named "tenth review,
R8",
a process label with no file in the repository;
and the TSDoc of `restoreTypography` said nothing of the closing mark of a quotation that began with an elision.
Besides these,
the name of the S15 test of n147 still said that the mark is the one the archive held (B557).

Fixed by n149.
The header of `src/consistency-markup-runs.ts` is rewritten.
The finding text in `src/corpus-run/pass-consistency-apply-settled.ts` now says
that the apostrophe another replacement carries in would lose its proof,
with the tests in the files for the settled outcomes and the hand-built pages
(red in `red-settled.log` and `red-hand-built.log`).
`src/artifact-key-vocabulary.ts` names the files `repair-result.ts`,
`slice-delivery.ts`,
`translate-assemble.ts` and `repair-assemble.ts`,
each checked with `rg`.
The comment of `quote-neighbours.ts` and the TSDoc of `restore-typography.ts` are rewritten,
and the S15 name and the generic names of the shipped-page tests say
that the shipped page holds exactly one mark there.

## B571: the units of repeated comment openers were found in quadratic time

Found on 2026-10-14 by the re-review (finding Q8).

Outcome written on 2026-10-14.
Fixed by n149;
uncommitted at the time of writing;
commit:
pending.

`commentAndReferenceUnitsOf` in `src/consistency-unread-units.ts`
searched the nearest `-->` for each `<!--` and added the whole stretch.
A text of n openers and one closer cost,
for `markPastRunOf` end to end,
6 ms for 500 openers (2,022 characters),
24 ms for 1,000,
96 ms for 2,000
and 482 ms for 4,000 (16,022 characters).
The scan for `&` was linear (20,000 to 80,000 pairs `&a`:
60,
82 and 181 ms).
Only an adversarial English reaches it,
and the memo means it is paid once for an English.
The lead ruled that it becomes one linear pass.

Fixed.
`commentUnitsOf` makes one pass,
keeps the nearest closer in a cache,
and adds only the part of a stretch that is new.
The median cost for openers `<!--` repeated n times and one `-->`
(`q8-cost.mjs`,
`q8-cost-before.txt` and `q8-cost-after.txt` in `s215`):
500 openers 6.2 ms to 0.5,
1,000 28.6 to 0.5,
2,000 130.7 to 1.2,
4,000 529.2 to 1.6
and 8,000 2,331.6 to 3.4.
The test in `src/consistency-unread-units.unit.test.ts`
compares `commentUnitsOf` with a copy of the old search for 13 Englishes
(many openers and one closer,
overlap,
no closer,
new lines and none),
red in `red-unread-units.log`.
Guard C removes the cache of the closer,
and the unread-units test fails.
The oracle of that test is a copy of the old search and no reading of the rule text,
so the test shows that the new pass equals the old one on those 13 texts and nothing more.

## B572: `It was 'tis the cats' bowl.` ships `‘tis the cats’ bowl` (accepted)

Found on 2026-10-14 by n149 (scenario A11c of the review's adversarial pages).

The lead accepted it on 2026-10-14.
It follows from the ruling of B568:
a later straight mark pairs with the elision,
and no reader tells a possessive from a closing mark.
The page shipped `’tis the cats’ bowl` after n147,
and `‘tis the cats’ bowl` before the work of the tenth review
(the page `A11c` of that review),
so the shipped glyph is what shipped before this work.
A reader sees an opening quotation mark in front of `tis` in a sentence with a possessive after it.
Of the scenarios of the tenth review,
n149 changed two against the baseline of n147,
this one and the escaped stars of B565;
the other 123 of 125 are identical.

## B573: `kittens\’` is not an escape, and a visible backslash parts the marks (accepted)

Found on 2026-10-14 by n149 (scenario B8,
the residual of B565).

The lead accepted it on 2026-10-14.
`’` is not ASCII punctuation,
so `\’` is not an escape:
the backslash is visible and parts the two marks,
and the run ending there is right.
The tally of the probe still lists the page as doubled,
unchanged from the re-review,
and the other page that stays carried is the blank line of B567 (K7),
which the ruling keeps.

## B574: container markers at the start of a line inside the refusal run were not unread

Found on 2026-10-14 by n149 (its residual for the line breaks),
decided by the lead the same day.

Fix decided on 2026-10-14;
landed by n151;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would see:
a blockquote `>` or a list continuation indent after a single break inside a paragraph renders nothing,
so `> The kittens` and a new line and `> ’ bowl` shipped with a doubled mark.
The run of B567 continues across the break and then reads the `>` or the `*` of a list marker as prose characters,
so the run ends there and the gain is not refused.

The lead decided:
container markers and indentation at the start of a line inside the paragraph are unread by the run.

Outcome written on 2026-10-15.
Built by n151.
`src/consistency-paragraphs.ts` gives its blocks a `quoteDepth` and has a new `quoteDepthAt`.
`src/consistency-run-steps.ts` has `markersAfterBreakOf` and `containerStepAt`
(with the private `lineBreakBeforeMarkersAt`),
and `markPastRunOf` in `src/consistency-closing-marks.ts` steps the quotation markers (`>`) after one break
and,
on the leading side,
jumps back to the break.
The marker steps live in `containerStepAt` and not in `src/consistency-closing-marks.ts`,
which had passed the limit of 300 lines (303).
The tests were red first:
the pages C1 to C5 of the shipped-marks test,
and the unit tests of the paragraphs,
the closing marks and the run steps;
C6 to C9 were green already,
kept by another guard,
and stay as regression pages.
`markersAfterBreakOf`,
`containerStepAt` and `ignorableWidthAt` have no unit test of their own,
and are covered through `markPastRunOf`.
The guards for the forward step and for the backward jump were removed once each and failed.
In the container probes eight cases now name the mark `’`.

Checked by n153:
113,764 two-line shapes
(a quotation or list piece before each line,
zero to three pieces from sixteen kinds,
seven separators,
twenty-seven context prefixes and ten suffixes,
the trailing side with `kittens` and the leading side with `He ate ’`)
were read through `markPastRunOf` and through the page grammar.
None was missed,
and none was stepped where the reader sees separate blocks.
The grammar shows the marks adjacent on 11,628 pages and not on 102,136,
so both classes are populated and the probe can fail.
The whole pass was run on 34 pages:
every page that should refuse is `kept-current`,
and the pages that should ship
(a list marker,
a `1.` marker,
a deeper `>`,
a zero-width space before the marker,
a blank quotation line,
a blank line and an indented paragraph,
a thematic break,
a footnote definition line)
ship a separate block.
Six pages differ from what the reviewer first wrote and are right:
`2.` does not interrupt a paragraph and its digits stand between the marks,
a table row `| ’ |` has `|` between the marks,
a four-space `> ` is text,
a comment line between is a separate block,
and a heading marker or a fence line builds no group (B578).
Eight pages with the repeated paragraph second in a quotation,
a list item,
a quotation in a list item,
a second list item or a footnote definition were all refused,
so the text of a slice keeps the context that the depth needs.
The run steps `*`,
`_`,
`~`,
a backtick and tags on purpose,
and the review did not count that as an overstep.
