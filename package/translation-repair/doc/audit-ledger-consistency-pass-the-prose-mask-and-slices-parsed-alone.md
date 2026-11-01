# Audit ledger: the consistency pass, the prose mask and slices parsed alone

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

Typography may change only what the page parse shows as prose text.
It curls a mark,
restores the ellipsis,
places the guillemets and unifies the quote style of a page.
This file holds the finding of the review of n154 that made it so (W1),
the cache versions that moved with it,
and the findings of the review of n156 about the mask and about the readers that parse a slice alone:
a footnote or reference definition held in another slice,
an edit-region fragment that the grammar refuses alone,
a link whose label is an address,
two untrue sentences,
a refusal of the grammar that no one hears,
and a guillemet floor that stands aside.
W1 is fixed by n156 and X3 is landed by the lead;
X2,
X4,
X5,
X6,
X8,
X10 and X11 were decided and built by n159,
and landed on 2026-10-15 with the tests `5fe3173cc5` and the code `277145a862`.
The review of n159 found three more findings about the readers that parse a slice
(the definitions inside containers,
an out-of-range landing and the memory of the mask),
and n161 fixed two of them;
they are B620 to B622 and are filed in
[Audit ledger: the consistency pass, definitions inside containers, fragment landings and the mask memory](audit-ledger-consistency-pass-definitions-landings-and-the-mask-memory.md).
B623 here records the cache versions that n159 and n161 moved.

The findings about span edges are filed in
[Audit ledger: the consistency pass, span edges as whole units](audit-ledger-consistency-pass-span-edges-as-whole-units.md),
those about elisions and pairing in
[Audit ledger: the consistency pass, elisions, quoted numbers and pairing](audit-ledger-consistency-pass-elisions-quoted-numbers-and-pairing.md),
those about the refusal run in
[Audit ledger: the consistency pass, what the refusal run steps over](audit-ledger-consistency-pass-what-the-refusal-run-steps-over.md),
and those about what the word test reads as shown in
[Audit ledger: the consistency pass, what the word test reads as shown](audit-ledger-consistency-pass-what-the-word-test-reads-as-shown.md).

Written on 2026-10-15 (UTC) from the reviews of n154,
n156 and n159
(`s224/review.md`,
`s227/review.md` and `s231/review.md`),
the hand-backs of n156 (`s225/handback-n156.md`),
n159 (`s230/handback-n159.md`) and n161 (`s232/handback-n161.md`)
and the lead's notes of that day.
The labels W1 and X2 to X11 are the reviews' own and are no ledger ids.
The labels of probe scenarios and the names of probes and logs are the sources' own too.
A path that starts with `src/` is under `package/module/translation-repair/` (moved on 2026-10-18 to the packages under
`package/translation-repair`).
Every function,
type and constant of the package named here was found declared in non-test source of the tree `w158`
by searches of declarations on the day of writing.
The functions,
types and constants named in the landing text of B609 to B615
(`pageDefinitionsOf`,
`withDefinitions`,
`collectDefinitions`,
`destinationEndOf`,
`readingForRestoration`,
`proseMaskReading`,
`proseMaskBuilds`,
`PageReadings`,
`readingOf`,
`childrenAreProse`,
`labelIsAddress`,
`noteRefusedMask`,
`textNamed`,
`guillemetFindings` and `restoreTypography`)
were found declared in non-test source of the tree `w153` at `277145a862`.
The names that only a review or a hand-back gives
(test names,
guard names,
the names of probes and the names of logs)
were not searched.
No line numbers are cited,
since the tree is still changing.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B606: typography rewrote marks inside link destinations, titles, addresses, expressions, code, math and comments

Found on 2026-10-15 by the review of n154
(n155,
finding W1).

Outcome written on 2026-10-15.
Fixed by n156;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
a link to a wrong address
(`[Peter](https://en.wikipedia.org/wiki/Peter_O'Toole)` became `Peter_O’Toole`),
a lost link title
(`title` went from `"The dogs"` to `null`),
code samples with curly quotes that no longer run,
and math with a curly prime.
The page parse refused some of the curled text
(`{“a”}` and `alt=“d”` after a `>` inside an expression),
and the page-assembly guard would then withdraw the slice
(unverified,
the reviewer did not run the guard on these pages).
The page pass `unifyQuoteStyle` fires where curly prose marks outnumber straight ones,
and in one run it changed 36 marks,
among them the contents of a `~~~js` block.
The direct calls showed the same for an indented block,
double-backtick code,
`$f'(x)$`,
`$$` blocks and `<!-- it's "here" -->`.
Single-backtick code,
backtick fences,
four-backtick fences,
attributes of an ordinary tag and angle autolinks were left alone.

The cause:
the old mask in `src/typography-prose-mask.ts` toggled code per backtick character
(so double-backtick code was prose after the second backtick),
ended a tag at the first `>` (so `ok={a > b}` leaked the rest of the tag),
and masked only a backtick span and a tag.
Link destinations and titles,
fences by tilde,
expressions,
math and addresses were never masked.
The edits reached it through `src/corpus-run/quote-style-unify.ts`,
`src/corpus-run/page-slice-rewrite.ts` (every slice except front matter and sealed spans)
and `src/corpus-run/would-ship-text.ts` (`inArchiveTypography`,
every wording).
The finding predates n154.
The lead had asked for it as a check,
and the author's account of n154 listed only the case of the link destination and declined it,
which was the smallest of the set.

The lead ruled:
typography touches only what the page parse shows as prose text,
and one mask built from the parse is shared by every typography reader.
A raw search of the corpus found 23 pages that hold a link,
an expression or code with a quote mark in it.

Fixed by n156.
`proseMask` in `src/typography-prose-mask.ts` is rewritten to build the mask from the page parse:
front matter is no prose (a refusal of its split gives an all-false mask),
and the body goes through `parseSliceBody`.
A unit is prose when a text node of the tree covers it and the parser read it as written
(the written character equals the masked one,
so comments,
lone container tag lines and invisible-only lines are subtracted).
A link whose source does not start with `[`,
an autolink or a bare address,
makes its children no prose.
A text that the grammar refuses (`MdxParseError`) has an all-false mask,
so no design decision about prose is made on a page that the pipeline would not ship,
because the page assembly refuses it first (`src/corpus-run/page-assembly-guard.ts`).
The mask is memoised on the text (32 entries).
`src/parse-slice-body.ts` gains the type `InvisibleLinesReading` (`'blank'` or `'text'`),
the parameter `invisibleLines` (default `'blank'`,
the old behaviour) and the returned `masked` text.
`proseMask` is exported from `src/index.ts` so that the test reaches it.
Five readers call `proseMask({`:
`restoreTypography`,
`src/restore-ellipsis.ts`,
`straightenProseQuotes` in `src/quote-normalize.ts`,
`proseGuillemets` in `src/translate-guillemets.ts`
and `src/corpus-run/quote-style-unify.ts`;
a source scan in `typography-prose-mask.unit.test.ts` asserts that each of the five does,
so no reader keeps a scanner of its own.
Their header and TSDoc sentences now say that links,
tilde fences,
expressions,
math,
comments,
addresses and front matter are no prose.

Three places where the parse says something other than the ruling's list,
accepted by the lead:
an indented line is prose,
because the MDX grammar the site compiles has no indented code
(a fenced block of any kind is no prose);
an angle autolink such as `<https://x.example/O'Toole>` is a refusal in MDX,
so that text has no prose at all,
while bare and `www.` addresses are no prose and the marks after them are;
and a backslash-escaped mark in prose (`\'`) is flagged prose but stays as written.

The corpus measure of n156,
on the final build,
over the 92 pages and 1,237 slices (none refused):
`restoreTypography` as a curly page changed 279 characters,
none outside prose;
as a straight page it changed 179,
none outside (50 slices change length,
the ellipsis);
and the real `unifyQuoteStyle` on the page as it is changed 106,
none outside.
The positive control of ten W1 shapes changed 22 characters before,
17 of them outside prose,
and 5 after,
none outside.
So the reach of W1 on the corpus is zero changed characters:
the defect is real for synthetic inputs,
and the tests carry it.
Straight marks outside prose in the corpus:
456 in 53 pages in flow elements,
56 in 7 pages in text elements (already protected by the old tag mask),
and 14 in 2 pages in lines the mask removes.
No corpus page has a quote in a link destination or title,
an address,
code,
math or an expression.
The recount of the elisions moved from 70 to 115 mismatches
(`orig ’ -> out '` from 7 to 48 and `orig ‘ -> out '` new at 4):
the 45 added rows are all marks that the ruling makes no prose,
36 in comments,
12 in front matter and 4 in other non-prose
(an inline code span with a stray backtick and string literals in expressions),
no prose mark among them,
and the one original elision mark (`’round`) is still restored.
One page of the 92 (`ArtsEpiphany`,
120 characters) has no prose at all,
and none is refused for a reason that would hide prose.

The tests were red first:
the new `typography-prose-mask.unit.test.ts`
(8 behavioural failures),
the new `restore-typography-prose.unit.test.ts` (41),
and the page cases of `quote-style-unify.unit.test.ts` (9),
with controls that pass before and after.
The guards failed:
a mask that is all true (12,
53 and 11 failures),
and a reader with a scanner of its own (the test that the mask is the one scanner of prose).

## B607: the cache versions of the three stages that store restored text moved

Found on 2026-10-15 by the author of n156,
as the step "Before a run launches" of `mistake-prevention.md` requires after B606.

Outcome written on 2026-10-15.
Moved by n156;
uncommitted at the time of writing;
commit:
pending.

The change alters text that a cached outcome stores,
so three versions moved:
`SLICE_CACHE_VERSION` from 34 to 35 (`src/repair-slice-key.ts`:
the outcome of a repair holds the replacement with `restoreTypography` applied in `src/apply-patch.ts`),
`REFINE_CACHE_VERSION` from 5 to 6 (`src/refine-slice-key.ts`:
the rewrite is stored restored),
and `TRANSLATE_SLICE_CACHE_VERSION` from 15 to 16 (`src/translate-document-contract.ts`:
the typography fold in `collapseKey` and the guillemet floor read the same mask).
Each has an account paragraph.
The consistency cache version did not move:
it stores ballots only and counting is not stored.
The two pinned hashes moved with the versions
(`repair-slice-key.unit.test.ts`,
red first,
and `LEGACY_WINDOWLESS_KEY` of `translate-slice-key.unit.test.ts`),
and `own-unit-tests.unit.test.ts` lost `src/typography-prose-mask.ts` from its allowlist,
since the file has a test of its own now.

The exception that the document allows,
that no cache file was written after the change,
could not be proven.
`cache-account-audit` finds no record under the worktree's own runs directory,
and a metadata `find` over the replay runs directory found no slice-cache file newer than the setting of 34,
but its control since January found nothing either,
so it proves no absence.
The author listed that `find` among its breaches (M216).
A bump costs nothing if no record exists and is required if one does.
The lead kept the bumps:
no record can be shown absent,
and the next run uses a fresh runs directory either way.

## B608: two cache versions whose own accounts name the changed readers did not move

Found on 2026-10-15 by the review of n156
(n157,
finding X3).

Outcome written on 2026-10-15.
Landed by the lead;
uncommitted at the time of writing;
commit:
pending.

`CONSOLIDATE_CACHE_VERSION` was 21 (`src/consolidate-key.ts`)
and `LANE_CONTEST_CACHE_VERSION` was 6 (`src/lane-contest-key.ts`).
The account of the first says that the consolidation slate collapses twins through `collapseKey`
and `straightenProseQuotes`
and that its validator refuses prose guillemets,
and the account of the second says that the lane eligibility validator refuses prose guillemets.
n156 changed both readers:
`wordingKey` now keeps apart two renderings that differ only by a curly against a straight mark
in a comment or in a link address
(they collapsed under the old mask),
and `guillemetFindings` returns no finding for a candidate that the grammar refuses.
The reason for moving the three versions of B607 holds for these two equally,
and `mistake-prevention.md` moves the version for "a change that alters what the stage asks or accepts".
What a reader sees:
nothing directly;
a stored slate or contest outcome written under the old reading would be served under the new one.
The lead's reason for not adding more,
that the next run uses a fresh runs directory,
lowers the urgency and not the argument of parity.
The reviewer did not read the lane-contest code that calls the validator,
and the finding rests on the two accounts.

The lead ruled and did it:
`CONSOLIDATE_CACHE_VERSION` from 21 to 22 and `LANE_CONTEST_CACHE_VERSION` from 6 to 7,
each with an account,
in the tree `w158`;
no test pins either version (a search of the tests).
The consistency cache (version 4) stores ballots only and counting is not stored,
so the word test of B596 moves no number.

## B609: a doubled mark ships when the footnote or reference definition is in another slice

Found on 2026-10-15 by the review of n156
(n157,
finding X2).

Decided on 2026-10-15.
Built by n159;
landed on 2026-10-15 with the tests `5fe3173cc5` and the code `277145a862`,
in one landing with the fixes of n161.

What a reader of the page would see:
`dogs’¹’ bowl`,
a mark,
a footnote number and a second mark.
The English of one slice is `The dogs’ bowl is full. The kittens[^1]’ bowl is empty. A cat naps.`
with the definition `[^1]: A note.` in the next section,
the usual layout,
and the chosen words are `dogs’`.
The page ships `The dogs’[^1]’ bowl is empty` (`replaced`),
and so do `kittens[^1] ’`,
`kittens[^1][^1]’` and `kittens[x][r]’` with `[r]: https://x.example/r` in the next section.
The same text with the definition in the same slice is kept (`kept-current`).
The comments of `src/consistency-closing-marks.ts` call exactly `dogs’[^1] ’`
a second mark for the eye that the run must not hide.

The cause,
by the reviewer's best reading,
the deciding function not traced:
`textBlocksOf` in `src/consistency-paragraphs.ts` parses the English of the slice alone,
where a reference without its definition is plain text `[^1]`,
so there are visible characters between the marks.
Only a definition inside the slice makes the label an unread unit.
It has the same root as B615:
a slice parsed alone cannot resolve a label whose definition lives elsewhere.
`src/refine-envelope.ts` states the same limit
("a paragraph parsed alone cannot resolve a reference")
and collects the page's definitions for its gate,
which is the precedent for the fix.
The finding predates n156;
it is the doubled mark that still ships of the fifth item of the review of n154,
in the area that B589 touched.

The lead ruled:
every reader that parses a slice
(the paragraphs and units of the run,
the prose mask)
parses it with the page's definitions,
as `src/refine-envelope.ts` collects them,
through one shared reader of the page's definitions.

Reach,
measured by the lead:
none on the corpus.
No definition label holds a mark and no reference is followed by a mark,
and the search reached all 92 pages.

Built by n159,
for B609 and B615 together,
and landed with the fixes of n161 and not as its own commit.
- `src/page-definitions.ts` (new) holds `pageDefinitionsOf` and `withDefinitions`,
  and the definitions are threaded through `src/typography-prose-mask.ts`,
  `src/consistency-prose-text.ts`,
  `src/consistency-paragraphs.ts`,
  `src/consistency-run-steps.ts`,
  `src/consistency-closing-marks.ts`,
  `src/consistency-shown-edge.ts`,
  `src/consistency-fold.ts`,
  `src/consistency-count.ts`,
  `src/corpus-run/quote-style-unify.ts`
  and `src/refine-stage-replies.ts`,
  which hands `definitions` on to `restoreTypography`.
  `destinationEndOf` moved from `src/consistency-closing-marks.ts` to `src/consistency-markup-runs.ts`.
- A shortcut or collapsed reference such as `[dogs' bowl]` reads as no prose when a definition resolves it,
  since its text is also the definition's label.
- Tests:
  `src/page-definitions.unit.test.ts` and `src/corpus-run/quote-style-unify-page.unit.test.ts` (new),
  and the `DEFINED_ELSEWHERE` rows of `src/corpus-run/pass-consistency-shipped-marks.unit.test.ts`
  (the reference-link row now expects `unlocated` and an unchanged page).
  The doubled mark `dogs’[x][r]’` of the four footnote probes F0 to F3 was shown on the build before the round,
  the shipped-marks test was red,
  and the page-pass test was red by a failed load only.
  With `withDefinitions` removed,
  the definitions test fails 4 rows,
  the shipped-marks test 5 and the page-pass test 4;
  with the shortcut-reference rule removed,
  the definitions test fails 2 rows
  (the link-text test alone passed,
  so the guard was unguarded until the definitions test was added to its run).
- The review of n159 (n160) reran the page-level probe:
  the doubled mark is gone,
  `The kittens[^1]’ bowl` stays unchanged and `[x][r]` is `unlocated`.
  Appending the definitions after the slice shifts no offset,
  and the mask,
  the visible flags and the transparent flags keep the slice's length.
  It invented no definition from a fence,
  a comment,
  an indented code block,
  an unterminated fence or a title with marks.

Corpus effect:
none,
as the lead measured.
Typography changed 0 characters on the 92 pages,
and the prose flag of every straight mark changed 0 times with the definitions on (21 pages hold definitions);
the positive controls,
four texts that must move,
changed 6 marks,
so the probe could show a difference.

Limits that stand:
a call site with no page at hand gets no definitions
(`src/apply-patch.ts`,
which passes the slice as the landing text,
`src/archive-replacement-candidates.ts` and `src/corpus-run/would-ship-text.ts`),
and `collectDefinitions` in `src/refine-envelope.ts` joins definition lines with a single newline,
so a link definition directly after a footnote definition is read as a continuation of the footnote.
The definitions inside containers and that single-newline join are B620.

## B610: typography goes silent on an edit-region fragment that the grammar refuses alone

Found on 2026-10-15 by the review of n156
(n157,
finding X4).

Decided on 2026-10-15.
Built by n159;
landed on 2026-10-15 with the tests `5fe3173cc5` and the code `277145a862`,
in one landing with the fixes of n161.

What a reader of the page would see:
a region that the editor rewrote keeps straight marks on a curly page
when the region cuts an inline element.
`restoreTypography` with the fragment `He said <b>it's here` (an unclosed inline tag),
`it's here</b> and 'x'`,
`<Note>It's here`,
`It's here</Note>`,
`It's {here`,
`It's a<b and 'x'`,
`It's <3 and 'x'` or `<A><B>It's</B>`
gets an all-false mask and ships unchanged,
where the old scanner curled the prose marks (`<b>It’s here`).
The other direction has the same cause:
a fragment that parses alone but begins inside markup is read as prose,
and `{a} b="it's" /> It's` ships `{a} b=“it’s” /> It’s`,
a mark changed outside prose.
That direction was true of the old scanner as well,
so it is no regression.

The cause:
`src/apply-patch.ts` passes the replacement of one envelope,
`operation.newText`,
and the envelopes are merged intervals of the spans that accepted issues claim
(`deriveEditableEnvelopes` in `src/patch-model.ts`).
The reviewer found no snapping of an interval to a block or word edge in the part it read,
and the author's own text says that the regions run to a median of 75 characters,
so a region that cuts an inline element is plausible;
there is no corpus count of it.
The header claim of `src/typography-prose-mask.ts` that a replacement the grammar refuses
"is refused by the lane before it ships" has no cited support:
the page-assembly guard speaks of a slice that the lanes chose and not of an envelope fragment
or an archive slice that no lane touched (unverified for both).

The lead ruled:
an edit-region fragment gets its mask from the slice it lands in,
with the fragment in place,
and never from the fragment alone.
This fixes the straight marks after `<b>It's here` and the curled attribute tail.

Reach:
the lead measured it at run time only.

Built by n159,
and landed with the fixes of n161 and not as its own commit.
`src/apply-patch.ts` passes a landing (the slice,
the start and the end) to `restoreTypography`,
and `src/restore-typography.ts` reads the fragment as part of its slice
(`readingForRestoration`,
with the new optional parameters `reading`,
`landing` and `definitions`).
Tests:
the `LANDINGS` rows of `src/typography-transparent-markup.unit.test.ts`,
and a row of `src/apply-patch.unit.test.ts` ("READS THE REGION AS PART OF ITS SLICE",
the tail of a tag with an attribute),
whose failure is shown by the guard-removal run (3 rows fail);
the landing row was changed after its red log to the base behaviour found by the review's probe.
The review of n159 (n160) found the landing built from the target text and the envelope's offsets
after the checks that the current text equals the base text and that the hashes agree,
so its range is never stale or out of bounds there,
and it can be out of range only through a direct call (B621).
Its reach on the corpus is 0 pages:
typography changed 0 characters on the 92 pages.
The limit that stands:
no definitions reach `src/apply-patch.ts`,
so a fragment holding `[^1]` or `[x][r]` whose definition is in another slice is read there as plain text
(reasoned by the review from the trailing-apostrophe rule,
not probed).

## B611: two sentences of the mask file are not true, and the page pass parses each slice about twice

Found on 2026-10-15 by the review of n156
(n157,
finding X5).

Decided on 2026-10-15.
Built by n159;
landed on 2026-10-15 with the tests `5fe3173cc5` and the code `277145a862`,
in one landing with the fixes of n161.

The header of `src/typography-prose-mask.ts` says that the memory of 32 texts answers nearly every question
because the page pass reads each slice twice,
for its tally and its rewrite.
The tally walks every slice before the rewrite starts (`src/corpus-run/quote-style-unify.ts`),
so the rewrite of slice N finds the memory holding only the last 32 slices.
Measured by the reviewer:
`unifyQuoteStyle` costs about 2.2 parses per slice
(16,000 slices of four paragraphs:
the mask pass alone 9,687 ms,
the whole pass 21,792 ms;
1,000 slices of one paragraph:
the new mask 228 ms against 1 ms for the old scanner).
The memory serves only the repeated reads inside one slice.
The example in the TSDoc of `proseMask` and of `buildMask`,
`proseMask({ text: 'a <b c="d"> e' })`,
is refused by the grammar (an unclosed `<b>`),
so the result is all false,
including the `a` and the `e` that the description calls prose.

The lead ruled:
the page pass reads each slice's mask once and hands it to its tally and its rewrite,
and the memo sentence and the two refused TSDoc examples become true.

Built by n159,
and landed with the fixes of n161 and not as its own commit.
`unifyQuoteStyle` in `src/corpus-run/quote-style-unify.ts` reads each slice once
(`PageReadings` and `readingOf`),
and `src/typography-prose-mask.ts` exports the seam `proseMaskBuilds`,
which `src/dead-code.unit.test.ts` lists among the package seams with its reason.
`proseMask` left the package API,
since it was dead in production once every reader had moved to `proseMaskReading`;
the package is unpublished,
and the lead accepted its removal.
The tests that used it read the new fixture `src/quote-flags.test-fixture.ts` (`quoteFlags`).
The TSDoc examples of `src/typography-prose-mask.ts` now use `a <b c="d">cat</b> e`,
which the grammar accepts.
Test:
`src/corpus-run/quote-style-unify-page.unit.test.ts`,
red by a failed load,
and 2 rows fail with the guard removed.
`proseMaskBuilds` is guarded only through that test.
The review of n159 (n160) did not re-measure the single read per slice
(`readingOf` keys by the carried text,
which is right,
since the reading is a function of the text and the definitions),
and the author's guard-removal run is the only evidence.

## B612: a link whose label is an address has its label curled

Found on 2026-10-15 by the review of n156
(n157,
finding X6).

Decided on 2026-10-15.
Built by n159;
landed on 2026-10-15 with the tests `5fe3173cc5` and the code `277145a862`,
in one landing with the fixes of n161.

What a reader of the page would see:
the address shown on the page is not the address that opens.
`See [https://x.example/O'Toole](https://x.example/O'Toole) now.` ships the label `O’Toole`
and the destination `O'Toole`,
and `Mail [a'b@c.example](mailto:a'b@c.example) now.` ships `a’b@c.example` over `a'b@c.example`.
A reader who copies the label gets a different address.
The label is a text node of a bracketed link,
so `writesLabel` in `src/typography-prose-mask.ts` says prose.
The review called it small and a decision for a ruling.

The lead ruled:
the text of a link whose label is itself an address is no prose,
where the reading of autolink literals finds an address that spans the label.

Built by n159,
and landed with the fixes of n161 and not as its own commit.
The new `src/typography-link-text.ts` holds `childrenAreProse`,
the internal `writesLabel`,
`namesItsDefinition` and `labelIsAddress`
(which checks that the label is one token with no white space),
and `src/typography-prose-mask.ts` uses them.
Tests:
`src/typography-link-text.unit.test.ts`,
renamed from `src/typography-prose-mask-links.unit.test.ts` with `mv`,
which was 3 red before the rename and fails 4 rows with the guard removed.
The review of n159 (n160) checked `[https://x.example/O'Toole](…)` and `[a'b@c.example](mailto:…)`
(no prose in either),
and the one address label of the corpus,
`[info@one-among.us](`,
which holds no mark;
a label with words keeps its prose.
Probe `p-mask-more`:
the address label and the email label are now no prose.
Corpus effect:
none,
since the one address label holds no mark.

## B613: a refusal of the grammar for a bound is silent for typography

Found on 2026-10-15 by the review of n156
(n157,
finding X8).

Decided on 2026-10-15.
Built by n159;
landed on 2026-10-15 with the tests `5fe3173cc5` and the code `277145a862`,
in one landing with the fixes of n161.

The grammar refuses a block with more than 1,024 emphasis delimiters or more than 256 levels of nesting
(`src/nesting-vocabulary.ts`).
A paragraph of ordinary prose of 13,890 characters with 400 repeats of `*bowl* ... **full**` is refused,
and so are 400 unmatched `[`.
The mask is then all false,
so `unifyQuoteStyle` skips the slice and says nothing.
The pipeline's own parse refuses the same texts,
so for the output of a lane the page is withdrawn first;
whether anything withdraws an archive slice that no lane replaced is unverified.

The lead ruled:
a typography reader that gets an all-false mask from a refusal logs a finding that names the slice.

Reach:
none on the corpus,
since the 1,237 slices of the author's measure all parse;
this is exposure.

Built by n159,
and landed with the fixes of n161 and not as its own commit.
The new `src/typography-mask-refusal.ts` holds `noteRefusedMask` and `textNamed`,
and every reader that refuses a mask calls the first
(`src/restore-typography.ts`,
`src/restore-ellipsis.ts`,
`src/quote-normalize.ts`,
`src/translate-guillemets.ts` and `src/corpus-run/quote-style-unify.ts`).
The finding names the text by its first 48 characters.
Test:
`src/typography-mask-refusal.unit.test.ts` (new,
with no separate red log),
which fails 2 rows with the guard removed.
The review of n159 (n160) found that `src/restore-typography.ts` logs only when it built the reading itself,
since a reading it was given is the caller's to log (`src/corpus-run/quote-style-unify.ts`),
and that the warning can be wrong for an out-of-range landing,
which B621 closes.

## B614: the guillemet floor stands aside for a candidate that the grammar refuses

Found on 2026-10-15 by the review of n156
(n157,
finding X10).

Decided on 2026-10-15.
Built by n159;
landed on 2026-10-15 with the tests `5fe3173cc5` and the code `277145a862`,
in one landing with the fixes of n161.

`guillemetFindings({ candidateText: 'The cat said «I am hungry.» and {' })` returns no finding,
and with the grammar satisfied it returns one
(`src/translate-guillemets.ts`).
That is safe only if another floor refuses an unparseable candidate before the shipping gate,
and the reviewer did not trace the order of the floors.

The lead ruled:
trace which floor refuses an unparseable candidate before the guillemet floor stands aside,
and if none does,
the guillemet floor refuses it.
To check,
and to fix if needed.

Built by n159,
and landed with the fixes of n161 and not as its own commit.
By the author's account of the order,
the floor of `src/translate-source-carry.ts` runs at an earlier step of `src/translate-validate.ts`,
and the refusal of an unparseable candidate comes later and so never speaks first.
`guillemetFindings` in `src/translate-guillemets.ts` therefore now answers for a candidate that the grammar refuses:
it carries its guillemets outside comments as a finding.
Test:
`src/translate-guillemets-refused.unit.test.ts` (new),
whose refused-candidate row was red with 0 findings against the 1 expected,
and which fails 2 rows with the guard removed.
Probe `p-readers`:
the guillemet floor went from 0 to 1 finding for the refused-parse candidates.
The review of n159 (n160) found that the floor stands aside on `carriesGuillemet` of the source or the page,
which does not read the parse,
so counting every guillemet of the refused parse cannot change whether it stands aside.

## B615: a reference label with a straight mark is curled at the reference and left straight at its definition

Found on 2026-10-15 by the review of n156
(n157,
finding X11).

Decided on 2026-10-15.
Built by n159;
landed on 2026-10-15 with the tests `5fe3173cc5` and the code `277145a862`,
in one landing with the fixes of n161.

What a reader of the page would see:
the reference `[b’s]` no longer matches the definition `[b's]`,
so the link shows as the literal text `[the dogs][b’s]`,
and a footnote reference `[^o’k]` has no definition
(whether the assembly guard catches the footnote was not run).
Slice one reads `See [the dogs][b's] and it's.`
and another slice,
in the usual layout with the definitions at the end,
holds `[b's]: https://x.example/b`.
The same holds for `See [dogs' bowl] and it's.` with `[dogs' bowl]: https://x.example/d`,
and for `See the note[^o'k] and it's.` with `[^o'k]: A note.`
The slice parsed alone has no definition,
so `[b's]` is plain text and the mask says prose:
the output is `See [the dogs][b’s] and it’s.`
The slice that holds the definition parses alone as a definition,
whose label is no prose,
and keeps `[b's]:` straight.
Parsed together,
the label is no prose at all,
which is what the page is.
The old scanner curled both ends,
so they still matched.
The review ranked it between B609 and B608,
as a link or footnote that stops working.

The cause:
`src/typography-prose-mask.ts` builds the mask from one text alone,
and its callers hand it one slice or one fragment
(`src/corpus-run/quote-style-unify.ts` and `src/apply-patch.ts`).
It has the same root as B609,
and `src/refine-envelope.ts` is again the precedent.

The lead ruled:
the rule of B609,
one shared reader of the page's definitions for every reader that parses a slice.

Reach,
measured by the lead:
none on the corpus (0 pages;
no definition label holds a mark;
the search reached 92 pages).
The author's own corpus measure had not counted labels with a mark.
Labels are mostly numeric.

Built by n159 with B609,
by the page's definitions that every reader of a slice now has,
and landed with the fixes of n161 and not as its own commit.
The tests,
the red and the guard counts are those of B609.
Probe `p-refs` shows the shortcut reference label no longer half curled,
with its definition unchanged,
and probe `p-mask-kinds` counts 2 more marks that are no prose in that case (66 to 68).
Corpus effect:
none,
as the lead measured:
typography changed 0 characters on the 92 pages,
and no straight mark changed its prose flag.
The limit that stands for a fragment is that of B610.

## B623: the cache versions that n159 and n161 moved, and the two that stayed

Written on 2026-10-15,
from the accounts of n159 and n161,
the review of n159 and the lead's notes.

Red `5fe3173cc5`,
fixed in `277145a862`.

The earlier moves of this family are B607 and B608.
The rule is the same:
a cache needs a move when its stored output could hold text written under the old rule
and its key does not hash the sheet.
Each version has an account paragraph in its own file.

Moved by n159,
because a stored or derived outcome was decided by a reader that the round changed
(quote restoration,
the shown-word test,
the prose mask or the guillemet floor):
- `CONSOLIDATE_CACHE_VERSION` from 22 to 23 (`src/consolidate-key.ts`).
- `LANE_CONTEST_CACHE_VERSION` from 7 to 8 (`src/lane-contest-key.ts`).
- `SLICE_CACHE_VERSION` from 35 to 36 (`src/repair-slice-key.ts`).
- `REFINE_CACHE_VERSION` from 6 to 7 (`src/refine-slice-key.ts`).
  No test pinned this version before,
  and the guard run showed it unguarded,
  so `src/refine-slice-key.unit.test.ts` gained a pin;
  with the version reverted,
  3 rows fail.
- `TRANSLATE_SLICE_CACHE_VERSION` from 16 to 17 (`src/translate-document-contract.ts`).
  The pinned `LEGACY_WINDOWLESS_KEY` of `src/translate-slice-key.unit.test.ts` moved with it,
  as did the pins of the consolidation,
  lane contest and repair slice tests.
  With each of the four reverted,
  2 rows fail.

Moved by n161:
- The account paragraphs of the five versions that n159 moved gained the quoted-tense rule (B627),
  and each now names both rounds.
- `PAGE_TITLE_CACHE_VERSION` from 1 to 2 (`src/page-title-cache-version.ts`,
  with a new paragraph for n161).
  Its key does not hash the sheet,
  and the page title lexicon sheet carries the rule,
  so a reading cached under version one was written without it.
  The review of n159 had judged version 1 right for that round,
  since `src/page-title-lexicon-stage.ts` imports `straightenQuotes`,
  which n159 did not touch.
  A pin case,
  "STANDS AT VERSION TWO",
  in `src/corpus-run/pass-page-titles.unit.test.ts` fails 3 rows with the version put back to 1.
  The lead accepted the move.

Stayed:
- `CONSISTENCY_CACHE_VERSION` at 4.
  Its key hashes the whole sheet messages with the roster and the stage (`src/corpus-run/pass-consistency-cache.ts`),
  so a changed sheet is a different key,
  the stored answer is the heard voices,
  counting is not stored (`src/consistency-cache-version.ts`),
  and no consistency path calls a typography rule.
  The word test of B597 moves no number for that reason.
- `PAIRING_CACHE_VERSION` at 3,
  since the pairing sheets carry no house tense rule.

The control that the document allows an exception for,
a cache record written after the change,
was not made by either round,
since no runs directory is read except by the read-only replay.
The review of n159 did not check which key covers the stored replies of the archive review,
which restores typography (`src/archive-replacement-candidates.ts`),
and `wording-key.ts` uses `straightenProseQuotes`,
whose mask changed for address labels and shortcut references;
the replay (24 wordings,
none missing) and the corpus (one address label without a mark) show no change.
The account paragraphs are documentation text that no test guards,
and only the cache account audit reads them.

The audit:
`cache-account-audit` run by n161 on the uncommitted tree rebuilt the package,
read 1,615 source files and 8 versions,
accepted `CONSISTENCY_CACHE_VERSION` 4 (set in `e91da9ea8`) and `PAIRING_CACHE_VERSION` 3 (set in `d614a0c1d`),
and stopped with exit status 6:
`CONSOLIDATE_CACHE_VERSION = 23` was in no commit,
and the audit reads committed history.
After the landing the lead ran it again,
and by the lead's notes of that run
`CONSOLIDATE` 23,
`LANE_CONTEST` 8,
`PAGE_TITLE` 2,
`REFINE` 7,
`SLICE` 36 and `TRANSLATE_SLICE` 17 are set in `277145a86`,
and `CONSISTENCY` 4 and `PAIRING` 3 ride.
