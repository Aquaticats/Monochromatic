# Audit ledger: the consistency pass, the independent oracle and the rules for marks

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

The record of what an oracle written by someone else from the rule text found in the tree that n140 left,
and of the rules for marks that the lead decided in answer (D1 to D9) and n143 built.
The grids had read zero under the oracle that n140 had changed to its own rule.
Under the oracle of n142,
which imports nothing and was written from the text of the rules alone (76 hand cases,
four of them failing-first controls for the checks a to d),
n140's tree failed the final-page checks on 3,555 of 144,227 single-edit pages of the `g2` grid
(a 2,883,
b 598,
c 81),
on 2,208 of 49,811 two-edit pages (a 2,176,
b 32,
c 26),
and on 1,092 of 39,395 pages of the `claims` grid (a 22,
b 249,
c 23,
d 861).
The author's zeros had been the author's reading checked against itself (M194,
M196).
The oracle stays with its author,
who updated it to D1 to D9 (111 cases pass,
16 old expectations changed,
each tied to a decision),
and the code went to n143.
The rerun of the grids on n143's build (`s207`,
with n142's command lines) reads single-edit a 76,
b 0,
c 0,
d 0 on `g2`,
two-edit a 59,
b 0,
c 0,
d 0,
and a to d all 0 on `claims`.
Every page left is check a,
and it is the residue B549,
which n144 fixed apart from four two-edit pages that are the oracle's error (B549).

The rounds before this one are filed in
[Audit ledger: the consistency pass, parse boundaries, the letter test and the proof's page](audit-ledger-consistency-pass-parse-boundaries-letter-test-and-proof-page.md)
and
[Audit ledger: the consistency pass, the ninth review](audit-ledger-consistency-pass-ninth-review.md).
The accepted limits of n133,
one of which D3 reversed,
are filed in
[Audit ledger: the consistency pass, one reading of every mark](audit-ledger-consistency-pass-one-reading-of-every-mark.md).
The residue that n143's notes handed to n144 is filed in
[Audit ledger: the consistency pass, the fixes of the eighth review and the oracle residue](audit-ledger-consistency-pass-fixes-of-the-eighth-review-and-the-oracle-residue.md),
and the tenth review and the re-review that followed in
[Audit ledger: the consistency pass, the tenth review](audit-ledger-consistency-pass-tenth-review.md)
and
[Audit ledger: the consistency pass, the re-review and the refusal run](audit-ledger-consistency-pass-re-review-and-the-refusal-run.md).

Written on 2026-10-14 (UTC) from the hand-backs of n142 (`s205/handback-n142.md`) and n143 (`s206/handback-n143.md`),
the rerun of the grids (`s207`) and the lead's notes of that day.
The labels D1 to D9,
K1 to K3,
the check letters a to d,
the case labels (S02,
R04,
P19,
c1 to c3) and the guard names (d1a,
d2,
d4 and others) are the sources' own and are no ledger ids.
A path that starts with `src/` is under `package/module/translation-repair/` (moved on 2026-10-18 to the packages under
`package/translation-repair`).
Every function,
type and constant of the package named here was found declared in non-test source of the merged tree `w158`
by searches of declarations on the day of writing,
except those an entry says were removed or renamed.
The names that only a hand-back gives (test names,
guard names,
the helpers of test fixtures and the names of logs) were not searched.
The tree changed after n143 (n144,
n147 and n149 followed),
so the entries of the later rounds say what they changed.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B535: a gained edge mark was carried directly beside the mark the receiving place keeps

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n142 (check d on 861 pages of `claims`,
decision D1).

What a reader of the page would have seen:
a doubled mark.
`He said ’猫 now.` became `He said ’'cats now.`,
and `She, (猫’”.` became `She, (cats’’”.`
n140 had reported J1 closed (its grid read no new shape),
and this was J1 stated in the rule text:
a gained mark arrives beside a mark that the receiving place keeps.

The cause:
the chosen page held a doubled mark before its words (`''`,
`’'`,
`ʼʼ` or `’ʼ`) and the target held a single mark directly before the span.
For `He said ''cats` the outer `'` read free,
because nothing letter-like stood beside it,
and the inner `'` read leading,
because a letter followed and none preceded.
A leading mark never closes and stood after the last possible closer,
so it was proven,
and `withProvenApostrophes` widened the span over the proven leading mark directly before the words.
The core became `'cats` and the outer `'` stayed outside as `markBefore`.
The `g2` grid showed no case because its chosen pages carry single marks only,
and the `claims` list of doubled marks reached it.
The author's oracle had checked the author's rule (M194).

The lead decided (D1):
a gain is `unavailable` when any single mark stands directly outside the span on the gain side,
or one wide run away,
whatever that mark's reading,
and when the chosen words carry two marks at that edge.

Fixed by n143.
`gainsBesideKeptMark` in `src/consistency-splice-readings.ts` is true when the chosen words carry two single marks
at an edge where the receiving words carry none (the new exported `holdsTwoMarksAtEdge`),
and when the words bring an edge apostrophe on a side where the target has `markBefore` or `markAfter`,
or `markPastBefore` or `markPastAfter`,
whatever that mark reads as.
`bringsEdgeApostrophe` is still reached from it;
the source scan failed once when it was not.
D1 sees the four single marks the generation 17 record stores
(`'`,
`‘`,
`’` and U+02BC),
and not the four that the record refuses to store (B551).

Evidence:
the baseline gave `He said ’'cats now.` for `He said ’猫 now.` with the chosen words `''cats`
(`s206/before/repro-probe.txt`),
and the plan is now `unavailable` (`s206/final/repro-probe.txt`);
`’’cats` too;
and `She, ''**cats**s now.` into `He said ’猫s now.` gives `He said ’**cats**s now.`,
one mark and not two.
The rerun reads check d at 0 on `claims`.
The planner grid `h2-grid-j1` moved from 78,790 replaced pairs to 77,817,
and its J1 shape count from 0 to 139 (B550).

Red evidence:
`s206/red/item6-red.log` (two tests fail),
and the guards d1a (the marks directly beside dropped) and d1b (the pair rule dropped).
The tests build their located renderings by hand (`locatedWith`).

## B536: `marksPastMarkupOf` read a lone `*`, `_` or `~` as closing markup by its shape

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n142 (decision D2).
It is the shape test that K2 had rejected in B506,
and n140's `marksPastMarkupOf` brought it back (B530).

What a reader of the page would have seen:
the underscores print as underscores
(or as italics under a renderer that reads CJK-friendly emphasis),
and a bare quote mark sits before or after the word with no partner.
`He said 猫_cats_’ now.` and `He said '_cats_s now.` are no emphasis for the parse,
yet the mark was listed past markup and proven.
425 single-edit and 234 two-edit pages of check a had no single choice that made them pass,
and every recorded example held `_word_`.
Through the built package the prose view and the proof view blanked nothing for the underscore,
and `marksPastMarkupOf` still listed the mark.
`a*’s bowl` listed the mark past markup as trailing and proved it as leading.

The lead decided (D2):
every run walker counts `*`,
`_` and `~` only where the parse makes them a delimiter of emphasis,
strong emphasis or strikethrough.

Fixed by n143.
`markPastRunOf` in `src/consistency-closing-marks.ts` takes an optional `emphasis` and calls a private `isRunUnit`.
`closingRunMarksOf` in `src/consistency-markup-runs.ts` steps only over the offsets the parse names,
`src/consistency-marks-past-markup.ts` reads the parse,
and `isEdgeApostropheAt` and `endsReadingRun` in `src/consistency-loose-marks.ts` read the parse of the span's words.
`emphasisDelimitersOf` in `src/consistency-prose-text.ts` got a memo of four entries (`rememberedDelimiters`),
and `readProof` hands `emphasis` on.

Evidence:
n143's probe `prose-probe` (no output to compare with) reads `猫_cats_’`,
`'_cats_s`,
`the kittens*’ bowl and *`,
`the kittens_’` and `~’` as naming no mark past markup,
and `猫*cats*’`,
`猫**cats**’` and `5 “_cats_’` as naming the `’` past real closing delimiters and proving it.

Red evidence:
`s206/red/item1-red.log`.
The guard d2 (the parse gate removed in `closingRunMarksOf`) fails the lone-emphasis tests of `holdsEdgeApostrophe`,
`marksPastMarkupOf` and `markReadingsInEnglish`,
and the guard d2b (removed in `isRunUnit`) fails the tests of `markPastRunOf` and `countConsistency`.

## B537: a mark beside another script, a symbol or a pictograph was read as an apostrophe

Fixes uncommitted at the time of writing;
commit:
pending.
It reverses the accepted limit of B520.

Found on 2026-10-14 by n142 (decision D3).
The oracle read K3 ("a mark beside CJK reads as loose") as "free",
and production had followed the other reading,
in which `猫'cats` is a leading edge mark by the letter on its own side.
The lead decided that production deviated from K3 as decided,
and recorded a mistake of its own:
n133's flag e (`cats’猫` trailing) had been accepted against the text of K3 (M200).

What a reader of the page would have seen:
a quote mark with no partner after a CJK character,
before a Latin word,
carried as if it were an apostrophe.
`He said 猫cats…'s猫 now.` with the carried words `'Come (here)` gave `He said 猫'Come (here)…'s猫 now.`,
`He said 猫cats 🐱"s now.` with `’rock'n'roll` gave `He said 猫’rock'n'roll"s now.`,
and `He said 5 (Cat's'猫 now.` kept a stray `Cat's'`.
The oracle's own flip showed 2,854 of the 3,555 single-edit failures and 1,830 of the 2,208 two-edit failures
disappear under the literal reading,
so the choice carries most of the count.
A careful reader of mixed Latin and CJK prose mostly still reads `cats’猫` as a possessive
(the picks of n142 mark `He said **cats**’猫 now.` and `He said 猫_cats_' now.` doubtful),
and the lead chose the stricter reading as the safe direction.

The lead decided (D3):
a mark reads trailing only when the character after it is white space,
punctuation other than a single mark,
or the text's edge,
and leading likewise mirrored.
A letter or digit of another script,
a symbol or a pictograph beside the mark makes it free.

Fixed by n143.
`src/consistency-mark-neighbours.ts` is new:
`isNeighbouringSingleMark` knows the eight single marks,
and `partsMarkFromWord` knows the edge,
white space and punctuation that is no single mark
(one regular expression,
with a justified `no-restricted-syntax/no-regex` disable).
`kindOf` and `markReadingOf` in `src/consistency-proven-apostrophes.ts` were rewritten:
a mark is attached if a letter stands on both sides,
trailing only if what follows parts it,
leading only if what precedes parts it,
a mark past markup is trailing only if what follows parts it,
and everything else is free.
`src/latin-letters.ts` gained the category gate (a letter or a decimal digit) in the compatibility branch of
`continuesLatinWordOrCompatibilityForm`,
so `™`,
`²` and `№` are no letters,
and `isLetterOrDigitOfAnyScript` and its expression moved there,
re-exported by `src/consistency-loose-marks.ts`.
The brief's reds all read free now:
`the cats’猫 sat`,
`猫’rock`,
`the 🐱's bowl`,
`the cat™’s bowl`,
`He said ''cats now.`,
`the kittens’’ bowl` and `the dogsʼ’ bowl`;
`the kittens’ bowl` and `go home ’til dawn` stay proven.

Evidence:
in `h1-apostrophes` the rows H2 and H3 (`Fill the cats’猫 bowl.`) replaced the `’`,
which had read trailing and was part of the core
(`Fill the dogs猫 bowl.` and `Fill the dogs’猫 bowl.`);
the `’` now reads free,
stays outside as `markAfter`,
and the plan is `unavailable`.
The corpus of 1,190 marks in 92 pages does not change:
none of the marks differs from a simulation of the old rule text,
and five invented strings in the same file differ as expected,
which is the positive control.

Red evidence:
`s206/red/item3-red.log` (the sign-character test of `latin-letters`,
the compatibility-letter test and three new tests named for the character beside the mark),
and the guards "parting removed" (four tests),
"run-end parting removed" (`guard-tail.log`) and "category gate removed" (`guard-latin-gate.log`).

## B538: the run a mark is read past was too wide, on both sides

Fixes uncommitted at the time of writing;
commit:
pending.
It narrows B530 and reverses an acceptance of n140.

Found on 2026-10-14 by n142 (decision D4).

What a reader of the page would have seen:
a closing quotation mark taken for an apostrophe.
Of 30 marks that production proved past markup,
a careful reader agreed that 5 are apostrophes,
read 5 as doubtful and 20 as no apostrophe,
and of 30 gains carried past markup the reader agreed with none (7 doubtful,
23 not an apostrophe).
The agreed ones all have one shape:
a plural noun,
closing markup,
the mark and a space.
The others include `He said cats are "ok"' now.` (a mark after a closing double quote),
`He said Come (here)’ now.`,
`He said ’**cats** now.` (an unclosed opening quote) and `He said ''**cats** now.`.
n140 had accepted that `Come (here)'` carries the mark,
since a closing character is in the run.
The cause was the width of the run:
it held `"`,
`”` and `)` as well as closing markup,
and had a leading mirror.

The lead decided (D4):
the run a mark is read past holds only closing emphasis,
strong or strikethrough delimiters from the parse and a link's closing `](…)`,
on the trailing side only.
`"`,
`”` and `)` leave it,
because a single mark after them is most often a closing quotation mark.
The leading mirror is removed.
The refusal run of J1 stays wide.
D2 governs how a mark reads,
not when a gain is refused.

Fixed by n143.
`src/consistency-markup-runs.ts` was rewritten with two runs:
the reading run (closing delimiters from the parse and a link's closing `](…)`)
and the refusal run (`TRAILING_REFUSAL_RUN`,
`LEADING_REFUSAL_RUN` and `EMPHASIS_CHARACTERS`),
and `closingRunMarksOf` is the new walker.
`src/consistency-marks-past-markup.ts` is thin and trailing only,
and `MarksPastMarkup` is `{ trailing }`.
By the text of D4,
`*cat*’*dog*` now reads trailing (no leading mirror),
and a link's tail is read by its shape `](dest)`.

Evidence:
`p01-doubled` scenario S2 (a pair of backticks around `kittens`,
then `’ bowl`):
the baseline replaced the third place (`A dogs’ naps.`),
and the build now keeps the third and fourth places (`A cat naps.`) with two findings (B547).

Red evidence:
`s206/red/item2-red.log`,
and the guard d4 (`)` and `"` added to the reading run) fails the tests of `closingRunMarksOf` and `holdsEdgeApostrophe`.
The leading mirror has no code left to mutate;
its tests (`’**cats**` and `'_cats_` read free) stay in `src/consistency-marks-past-markup.unit.test.ts`
and `src/consistency-proven-apostrophes.unit.test.ts`.

## B539: a single mark directly beside another single mark read as an apostrophe

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n142 (decision D5),
from the picks P11,
P13,
P16 and P30:
on an existing page both the oracle and production proved a mark inside a doubled pair (`''`,
`’’`),
and a reader sees no apostrophe.
Check d covers gains only,
so the rule text left the reading of an existing doubled mark open.

What a reader of the page would have seen:
`He said ''**cats** now.`,
`He said cats are "ok"'' now.`,
`He said **cats**'' now.` and `He said ’’**cats** now.` carry a proven mark that is no apostrophe.

The lead decided (D5):
a single mark directly beside another single mark (`''`,
`’’`,
`’ʼ`) reads free,
both marks.

Fixed by n143,
with no line of its own:
a single mark never parts a mark from its word (B537),
so adjacent marks read free on both,
with a letter on either side of the pair or none.
n143 first wrote an explicit adjacency line,
and its guard run showed the line could not fail (`s206/red/guard-d5.log` was green),
because the parting test already made the tests pass,
so the line was deleted.
`He said ''cats now.`,
`the kittens’’ bowl` and `the dogsʼ’ bowl` read free in the tests of B537.

## B540: a leading mark of any apostrophe-like form and a later trailing mark are possible partners

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n142 (decision D6).
The oracle's flip that stops marks from pairing moved 239 single-edit and 443 two-edit pages,
and a reader sees `"'cats 🐱'ss` as a nested quote pair.

What a reader of the page would have seen:
in `He said 'cats and the dogs' bowl` the first mark may open a quotation that the second closes,
so neither is an apostrophe by the other's absence.

The lead decided (D6):
neither mark is proven by the other's absence.

Fixed by n143,
with no change to the production reading:
the case was not red at the baseline,
since `He said 'cats and the dogs' bowl` already proved neither
(`s206/before/trace.txt`).
n143 added a green regression test
(a straight leading mark or a later trailing mark is not proven by the other's absence)
and showed it red under a mutation in which a leading mark no longer sets the opener:
five tests fail (`s206/red/guard-d6.log`).
Corrected on 2026-10-14 (the wording of D6 is corrected where it stood).
D6 had been worded for the straight mark only,
and the hand-backs of n142 and n143 disagreed on a curly leading mark:
n143's build reads `it was ’tis the dogs’ bowl` as proving neither mark,
and n142's updated oracle keeps the curly leading `’` proven,
because in its reading a curly mark cannot open.
The tenth review (n146,
`s210/review.md`) answered it,
in its finding R4 and in the answer to its item 3 (one mark,
one meaning):
the readings of that page are `7:leading` and `20:trailing`,
and the proof proves none;
the straight `'tis the dogs'` and `ʼtis` read the same.
The code path is `provenApostrophesOf` in `src/consistency-proven-apostrophes.ts`:
`APOSTROPHE_LIKE` treats `'`,
`’` and U+02BC alike,
a leading mark sets `seen.opener`,
a trailing mark is proven only while no opener has been seen,
and a leading mark is proven only when it stands after the last closer.
The code follows its own header,
which names a straight or closing curly mark or U+02BC that is leading or free as a possible opener.
The lead ruled that the code stays:
a leading `’` of any apostrophe-like form is a possible opener,
which is the safe direction,
and D6 is read that way.
The narrower choice of the oracle (a curly `’` cannot open) is the oracle's own.
What a reader of the page sees:
no mark lost and none gained.
With a curly `’tis` early on a page,
every later gain is blocked and the receiving place's own `dogs’` stays unproven,
and with it after the gains only the gains after it are blocked;
a curly leading `’` is the usual glyph of an elision,
so the code withholds more repairs than a narrower rule would (B559).

## B541: a splice removed or added a quotation mark whose partner stayed

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n142 (check c on 81 single-edit pages of `g2`,
26 two-edit pages and 23 pages of `claims`,
decision D7),
with three examples for the lead to find what the grid's c rows slip through.

What a reader of the page would have seen:
a closing quotation mark with no opener.
In c1 and c2,
`He said 'cats 🐱'5 now.` and `He said 'cats 🐱'ss now.` with the carried words `cats 🐱`,
the page became `He said cats 🐱'5 now.`:
a closing quote with no opener in front of `5`.

The cause (c1 and c2):
the second `'` read leading,
because the pictograph is no letter and a digit or a letter follows it.
A leading mark never closes,
so both marks were proven,
and the locator widened the span over the opener (the core was `'cats 🐱`).
The replacement dropped an opening quotation mark and left its closer.

The lead decided (D7):
a splice that removes or adds a mark that is a quotation mark on the page,
free or partnered,
is refused,
and the lead asked what the grid's c rows slip through.

Fixed by n143,
through D3.
The second mark now reads free,
because a pictograph does not part it,
so the opener is no longer proven,
the core is `cats 🐱` with `markBefore` and `markAfter` both `'`,
and both plans are `unavailable`.
The rerun reads check c at 0 on `g2` (both runs) and `claims`.
The third example is c3 (B544).

Red evidence:
the test `planConsistencyEdit` beside a quotation mark that no reader takes for an apostrophe,
in `src/consistency-splice.unit.test.ts`.
It was red under a mutation that reads the mark back by letters only (`s206/red/item7-red-guard1.log`:
the first test fails).
The test for c3 passes under that mutation,
since c3's cause was D4,
and it is a green regression test.

## B542: the oracle and the counter disagreed on U+02BC

Decided without a code change.
Uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n142 (decision D8).
The oracle had read U+02BC as a mark that can never pair,
always proven and no letter,
and its flip to an inert character moved 395 single-edit,
2 two-edit and 351 `claims` pages.
`gainDefinitionDiffers` reads 0 on all 144,227 and 39,395 plans,
so the grids' own gain definition agrees with the oracle's default.

What a reader of the page would have seen:
nothing.
U+02BC cannot be a quotation mark,
and the 241 pages of n142's b group in which an outside U+02BC changes from leading to free after `cats 🐱` arrives
show no visible change.

The lead decided (D8):
the oracle follows production.
U+02BC is a single mark like `’`,
in the counter's set,
as decided.
n143 changed no code for D8,
and D1 sees U+02BC among the four single marks the record stores.

## B543: a bare leading mark with no possible closer reads as an elision

Accepted limit.
Uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n142 (decision D9).

What a reader of the page sees:
a bare leading mark that no later mark could close is read as an elision (`'tis`) and is carried with its word.
Where the word is not an elision,
the reader sees an unclosed opening quotation mark.
n142's picks on n140's tree show it:
`He said ’**cats** now.`,
`He said '_cats_ now.` and `He said '**cats** now.` are proven by the rule that proves `'tis`,
and a reader sees an opening quote,
since the word is not an elision;
the rule cannot know that.
D4 then removed the leading mirror past markup,
so those exact strings read free in n143's build.
The shapes that remain proven are the bare ones:
n143's probe reads `a*’s bowl` as proving its leading `’`,
and the oracle's cases R04 and R05 (`go home ’til dawn`,
`go home 'til dawn`) are proven leading marks.
No probe of a carried page on the final code that shows an unclosed quotation was made (unverified).

The lead accepted this on 2026-10-14 as a stated limit.
n143 rewrote the headers of `src/consistency-proven-apostrophes.ts`,
`src/consistency-markup-runs.ts`,
`src/consistency-marks-past-markup.ts`,
`src/consistency-loose-marks.ts` and `src/consistency-word-joints.ts` to state D1 to D9 as they hold,
D9 as an accepted limit,
and corrected stale text in `src/consistency-closing-marks.ts`,
`src/consistency-fold-apostrophes.ts`,
`src/consistency-splice.ts` and `src/consistency-splice-readings.ts`.

## B544: carried words add an attached apostrophe beside a mark that stays

Accepted limit.
Uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n143 (its attention list,
the third example c3 of D7).

What a reader of the page sees:
on a page whose paragraphs are `‘Go on.` and `He said ”’**cats**).`,
with the carried words `cat’s` for the emphasis,
the plan is still `replace`,
and the second paragraph becomes `He said ”’**cat’s**).`
The `’` before the emphasis now reads free and stays where it was,
and the carried words add an attached apostrophe inside `cat’s`.
The stray mark after the closing quote is as the page wrote it.
Under D4 the mark read leading through the old mirror and was proven before,
so the cause of the change was D4 and not D3.

The lead ruled on 2026-10-14:
accepted.
An attached `’` that the carried words add inside a word is a contraction,
and the claims run of the oracle reads check c at 0.
The test of this page keeps the page as a regression lock,
and its wording that it stands pending the lead's reading changes to accepted (an open item,
B552).

## B545: a lone `~` leaves a mark free where a lone `*` or `_` parts it

Accepted limit.
Uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n143 (its attention list),
as a consequence of D3.

What a reader of the page sees:
`*` and `_` are punctuation and part a mark from its word,
and `~` is a symbol and does not.
`a*’s bowl` reads its `’` as leading and proves it,
and `a~’s bowl` reads it as free.
Real `~~` delimiters are blanked by the prose view before the proof reads them,
so strikethrough itself is unaffected.
The lead ruled on 2026-10-14 that `~` leaves a mark free where `*` and `_` part it,
as D3 states.

## B546: circled and roman numerals are no letters for the neighbour test

Accepted limit.
Uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n143 (its attention list),
as a consequence of the category gate of B537.

What a reader of the page sees:
the neighbour test now requires a letter or a decimal digit
(category L or Nd),
which is what makes `™` and `²` no letters.
It also makes `①` (category No) and `Ⅻ` (category Nl) no letters,
which B518's first fix had made letters.
A mark beside them reads free,
and the place is left as it was.
Fullwidth digits,
mathematical letters and ligatures stay letters.
Tests and headers that named circled digits changed.
The lead ruled on 2026-10-14 that this is accepted as the safe direction.

## B547: a mark after inline code reads free and blocks a later gain

Accepted limit.
Uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by n143 (probe `p01-doubled`,
scenario S2),
as a consequence of D4.

What a reader of the page sees:
a backtick is not in the reading run,
so the `’` after inline code reads free (offset 37 in the trace) and is a possible partner for a later `dogs’`.
In scenario S2 the third and fourth places keep their words (`A cat naps.`),
and two findings say that the replacement was left out
because the arriving apostrophe would read as a quotation mark.
The lead ruled on 2026-10-14 that this is accepted as D4's consequence.
The direction is safe,
and the cost is repairs not made.

## B548: a control row of n141 no longer changed the proven set after the merge

Fixes uncommitted at the time of writing;
commit:
pending.

Found on 2026-10-14 by the tests after the merge of `w148` (n140 and n143) into `w158` against `s204/base`
(report `s207/merge-report.json`:
38 copied,
5 merged,
one conflicting hunk).
The hunk was the corpus-count text of `src/consistency-proven-apostrophes.ts`,
and the lead resolved it to `w148`'s text.

What a reader of the page would have seen:
nothing.
The test broke:
n141's control row `)"'cats'‘` of the typography cases was meant to change the proven set across the glyph change,
and after D5 its `'‘` reads free whichever way the glyphs fall,
so the row no longer changes anything.
The test file is `src/corpus-run/pass-consistency-shipped.unit.test.ts`,
where n141 put its typography rows;
the lead checked it,
and its control list `PROOF_CHANGING_ROWS` is there.

The cause:
a control was built on a row whose result depended on a rule that D5 changed,
and the merge was the first run that held both.

Fixed by the lead at the merge.
The control uses the two rows that still change the proven set,
and all three rows stay in the layout tests.
The merged tree read 2,360 passes and no failures in the whole suite (exit 0),
lint 0 warnings and 0 errors,
types clean,
49 scans passing and none failing,
and the replay as expected.
