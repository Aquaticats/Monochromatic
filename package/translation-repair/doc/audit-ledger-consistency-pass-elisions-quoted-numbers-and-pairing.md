# Audit ledger: the consistency pass, elisions, quoted numbers and pairing

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

The pass restores typography in the words it writes:
a straight mark on a curly page becomes `‘` or `’`,
and the pairing of single marks decides which.
This file holds the findings of the reviews of n150,
n153,
n155 and n157 about that pairing:
a listed elision (`'em`,
`'tis`,
`'90s`,
`'n'`) read as an opening mark inside a quotation,
the nesting of quotations,
an opening mark followed by markup,
and a quoted two-digit number.
The rounds that fixed them are n151,
n154,
n156 and n159.
The two findings of the review of n156 that were decided and not yet landed,
`'the '90s'` (B604) and the opening marks beside markup (B605),
were built by n159 and landed on 2026-10-15 with the fixes of n161.

The first rulings on elisions are filed in
[Audit ledger: the consistency pass, the tenth review](audit-ledger-consistency-pass-tenth-review.md)
(B563)
and in
[Audit ledger: the consistency pass, the re-review and the refusal run](audit-ledger-consistency-pass-re-review-and-the-refusal-run.md)
(B568 and B572).
The findings about the refusal run are filed in
[Audit ledger: the consistency pass, what the refusal run steps over](audit-ledger-consistency-pass-what-the-refusal-run-steps-over.md),
those about span edges in
[Audit ledger: the consistency pass, span edges as whole units](audit-ledger-consistency-pass-span-edges-as-whole-units.md),
and those about the prose mask in
[Audit ledger: the consistency pass, the prose mask and slices parsed alone](audit-ledger-consistency-pass-the-prose-mask-and-slices-parsed-alone.md).

Written on 2026-10-15 (UTC) from the four reviews
(`s217/review.md`,
`s221/review.md`,
`s224/review.md` and `s227/review.md`),
the hand-backs of n151 (`s219/handback-draft-items.md` and `s219/handback-n151-rest.md`),
n154 (`s222/handback-n154.md`) and n156 (`s225/handback-n156.md`),
and the lead's notes of 2026-10-14 and 2026-10-15.
The landing text of B604 and B605 was added on 2026-10-15 from the review of n159 (`s231/review.md`),
the hand-back of n159 (`s230/handback-n159.md`) and the lead's notes of the landing.
The functions and constants it names
(`restoreTypography`,
`LineState`,
`EMPTY_LINE`,
`withMark`,
`withQuotedNumber`,
`distinctDepths` and `bestPairsOf`)
were found declared in non-test source of the tree `w153` at `277145a862`.
The labels V2,
U5,
U6,
W3,
W7,
X7 and the item names of the notes are the reviews' own and are no ledger ids.
The inputs of the probes are quoted as the reviews print them.
A path that starts with `src/` is under `package/module/translation-repair/` (moved on 2026-10-18 to the packages under
`package/translation-repair`).
Every function,
type and constant of the package named here was found declared in non-test source of the tree `w158`
by searches of declarations on the day of writing.
The names that only a review or a hand-back gives
(test names,
guard names,
the names of probes and the names of logs)
were not searched.
No line numbers are cited,
since the tree is still changing.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B599: a listed elision inside a quotation took the place of the quotation's opening mark

Found on 2026-10-14 by the review of n149
(n150,
finding V2).

Outcome written on 2026-10-15.
Fixed by n151;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen,
on a curly page:
the real opening mark of the quotation left straight,
and the elision shown as `‘` (an opening mark) where an apostrophe `’` belongs.
`He said 'give 'em hell' to us.` shipped `He said 'give ‘em hell’ to us.`,
`She said 'sit 'round' and 'wait 'til' dawn.` shipped `She said 'sit ‘round’ and 'wait ‘til’ dawn.`,
and `'twas`,
`'til`,
`'cause`,
`'round` and `'90s` behaved the same.
A list such as `Mix: ..., '90s, 'n'.` gave `‘90s`.
A straight page was untouched.
Two shapes that were older and not the elision's:
`'Rock 'n' roll,' she said.` shipped `‘Rock ’n’ roll,' she said.`
(the closing mark after a comma stayed straight),
and `She said 'tis the dogs' and the cats' bowls,' and left.` left the final `,'` straight.

The cause:
in `src/nested-single-quotes.ts` an opening shape let an elision opener replace the mark that was already waiting,
and `singleShape` called every listed elision after a space an opener
(only `'n'` was excluded).
The ruling of B568 weighed an elision that begins a quotation
and did not weigh one that stands inside a quotation or ends one.
The review inferred from the ledger that the build of n147 had paired these shapes by the outer mark;
it did not run that build.

The lead ruled:
an elision never displaces a waiting opener,
and inside a quotation it curls to `’`.
`'n'` is one unit whose marks never open or close a quotation,
and on a curly page a straight mark left unpaired in closing shape curls to `’`
(one glyph serves a closer and an apostrophe),
which fixes `‘Rock ’n’ roll,'` and `bowls,'`.

Fixed by n151.
`isElisionPairEnd` and `precedesClosingMark` in `src/quote-neighbours.ts`,
`src/nested-single-quotes.ts` and `src/restore-typography.ts` (`endsElisionPair`).
The closing-shape curl applies only where the page converts trailing marks (`convertTrailing`),
because an existing test needs an unpaired opening-shaped mark elsewhere to stop the curl
(`She hummed *Cat Chorus*'sad tune' twice.`),
and the lead accepted that.
The expectation for `'Tis the season, 'Em and me, and 'Round we go.'` changed to
`‘Tis the season, ’Em and me, and ’Round we go.’`.
The tests were red first.
The guard for the displacement and the one for the closing-shape curl failed,
and the guard for `isElisionPairEnd` stayed green until two stray-opener cases were added,
which need `endsElisionPair`:
with that guard off those two fail.

Checked by n153:
every changed line of the two typography probes was right
(11 lines in one,
10 in the other),
a straight page was unchanged in every case,
and `'Tis the cats' bowl` gives `‘Tis the cats’ bowl` as ruled (B572).
The corpus holds one original elision mark,
and the recount of the elisions is byte-identical (70 mismatches).

## B600: a quoted letter `'n'` shipped two closing marks

Found on 2026-10-14 by the review of n151
(n153,
finding U5).

Outcome written on 2026-10-15.
Fixed by n154;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen,
on a curly page:
`the letter 'n' is silent in autumn` shipped `the letter ’n’ is silent in autumn`,
`Press 'N' to continue` shipped `Press ’N’ to continue`,
and `The letters 'a', 'n' and 'e'` shipped `The letters ‘a’, ’n’ and ‘e’`,
a quoted letter that opens with a closing mark.
Quoted words from the same list (`'round'`,
`'em'`,
`'til'`,
`'cause'`,
`'tis`) paired correctly.

The cause:
`isElisionPair` in `src/quote-neighbours.ts` took any `'n'` as the `n` of `rock 'n' roll`.
A context test (a word directly before and after the marks) cannot tell `the letter 'n' is` from `rock 'n' roll`,
so a fix has to choose which to lose.

The lead ruled:
`'n'` is the elision unit only with a word directly before and after it,
and `the letter 'n' is` and `Press 'N' to` keep that reading,
an accepted limit,
since no context test separates them.

Fixed by n154.
The private `standsBetweenWords` in `src/quote-neighbours.ts`
(the letter,
a second straight mark,
one space and a word-binding character on each side)
is used by `isLeadingElision` for `n`,
and `isElisionPair` and `isElisionPairEnd` follow through it.
The tests were written first:
`The letters 'a', 'n' and 'e'` pairs all three,
`He said 'n'` and `'n' alone` pair,
`rock 'n' roll`,
`fish 'n' chips` and `She said 'rock 'n' roll rules' and left.` stay the unit,
two spaces around the unit,
or none,
give the pair or the apostrophes in words,
and a test pins the accepted limit.
The guard for the word context failed its test.
The corpus reach is none.

## B601: a straight mark stayed on a curly page when the same replacement held a mark that could not pair

Found on 2026-10-14 by the review of n151
(n153,
finding U6).

Outcome written on 2026-10-15.
Fixed by n154;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
a straight mark among curly ones on one line.
On a curly page these were unchanged,
both marks straight:
`the dogs' bowl and 'quote start`,
`He said *'hi'* and the dogs' bowl`,
`[the dogs' bowl](http://x.example/'q')`.
`He said ''tis the season' and left` shipped `He said '’tis the season’ and left`,
`''em'` shipped `'’em’`,
`He said 'she said 'hi' to me' and left` shipped the outer pair straight and the inner pair curly
(and `'a 'b 'c' d' e'` the same),
`He said 'O'Brien said 'hi'' to us` shipped `He said 'O’Brien said ‘hi’' to us`,
and `Back in '99, she left` was unchanged.

The cause:
`convertTrailing` in `src/restore-typography.ts` is decided per replacement text.
It holds when the page convention has a curly apostrophe
and `countOpeningSingles(...)` is 0,
that is,
when no opening-shaped straight mark is left without a partner after pairing;
when one is left,
every closing-shaped mark of that replacement stays straight too.
`OPENING_CONTEXT` in `src/nested-single-quotes.ts` lacked `*`,
so `*'hi'*` found no pair,
and the pairing took one level only.
The first row is the one that the author of n151 had reconciled with the existing test of `*Cat Chorus*'sad tune'`,
and the lead had accepted it as the cost of `convertTrailing`.

The lead ruled:
pairing is nested (a stack);
emphasis and link delimiters (`*`,
`_`,
`~`,
`[`,
`(`) are opening and closing context;
a doubled opener before an elision pairs;
and `'99`,
two digits and no third,
is an elision.
A replacement whose marks still cannot all pair keeps them straight:
decided,
no guessed glyph.

Fixed by n154.
`nestedSinglePairs` in `src/nested-single-quotes.ts` keeps a stack,
a closing mark pairs with the innermost opening mark,
a listed elision never joins the stack behind another opening mark,
and the end of a line clears it.
`opensAfterQuote` is new:
a run of `*`,
`_` and `~` is opening context only when what precedes the whole run is the start of the text,
white space or an opening mark,
so `*'hi'*` opens while `Chorus*'sad` and `)'s` do not.
`opensBeforeElision` is new:
a mark right before a leading elision with a word after it opens a quotation,
and the next mark is the apostrophe.
`YEAR_LENGTH` in `src/quote-neighbours.ts` makes two digits and no third an elision like `'90s`.
The headers of both files and of `src/restore-typography.ts` were updated.
Kept as decided:
the first row and the `Cat Chorus` row stay straight.
Reported and not changed then:
`[the dogs' bowl](http://x.example/'q')` stayed straight,
because the mask treated a link destination as prose;
the prose mask of B606 settled it
(the apostrophe of the label curls and the mark of the destination stays).
The tests were red first
(four new tests,
one each for the stack,
the emphasis delimiters,
the doubled opener and `'99`).
The guards failed:
the single slot instead of the stack,
no emphasis context,
no doubled opener,
and no two-digit year.
Every changed row of the probes was attributed.
The corpus reach of `'99` is none.

## B602: a quotation that opens with a non-word character shipped a straight opener and a curly closer

Found on 2026-10-15 by the review of n154
(n155,
finding W3).

Outcome written on 2026-10-15.
Fixed by n156;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen,
on a curly page where every other mark is curly:
a straight `'` and a curly `’` around one quotation.
`He said '(hi)’ to me`,
`He said '*hi*’ and ‘bye’ to me`,
and the same for `'_hi_'`,
`'**hi**'`,
`'~~hi~~'`,
`'[hi]'`,
`'“hi”'`,
a code span,
`'...hi'`,
`'` then U+2014 then `hi'`,
`'¡hi!'`,
`'@hi' and '#tag'`,
`'$5'` and a bracketed link.
`He said 'hi *there*' to me` (a word first) paired correctly.
The finding contradicts the decided rule that a replacement whose marks cannot all pair keeps them straight.
Whether it was new in n154 is unverified,
since no build from before the change was kept.

The cause:
in `src/nested-single-quotes.ts` a mark opened only when a word character followed it,
and `countOpeningSingles` in `src/restore-typography.ts` counted an opening-shaped mark only with a word after it,
so the unpaired opener was not counted,
`convertTrailing` stayed true,
and the closer curled alone.
n154 had fixed the mirror case,
emphasis before the opener.

The lead ruled:
an opening-shaped mark is one after white space,
the start of the text or an opening context,
and before any character that is not white space;
one mark of a pair is never curled while its partner stays straight.

Fixed by n156.
`opensBeforeMarkup` in `src/nested-single-quotes.ts` is exported:
a straight single mark opens when the character after it is not white space,
not a word character and not another straight single mark,
the character before it does not bind a word and is no double quote or dash,
and the mark may open a quotation (`opensAfterQuote`).
`singleShape` returns `'opening'` from it,
and `countOpeningSingles` counts a mark with a word after it or with `opensBeforeMarkup`,
so the pairing and the count agree.
The tests were red first:
seventeen rows (`OPENED_BY_MARKUP`),
a property test that no output pairs a curly mark with a straight one,
controls,
and the link row.
Every row of the review that was half-curled pairs both marks now.
The one row left half-curled is `' hi'`:
a mark before white space is not opening-shaped by the ruling's own words,
and its closer stays curled (accepted).
The guards for `opensBeforeMarkup` and for `closerOfQuotedNumber` (B603) were removed together and failed 21 assertions.

## B603: a quoted two-digit number inside a quotation took the outer opener

Found on 2026-10-15 by the review of n154
(n155,
finding W7),
as decided rules that combine to a wrong glyph.

Outcome written on 2026-10-15.
Fixed by n156;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
`He said 'he is '42' years old' today.` shipped `He said ‘he is ’42’ years old’ today.`,
with `’` as the inner opener.
A quoted number alone was right (`He said '99' and left.` gives `‘99’`).
`the 1990s and '90s and '90s' music` shipped `‘90s and ’90s’`,
and the first `'90s` curled as an opening mark.

The cause:
`nestedSinglePairs` never pushes a leading elision (`'42` reads as a year) behind an open mark,
and it then pairs the closing mark after `42` with the outer opener.

The lead ruled:
a two-digit elision directly followed by a closing straight mark (`'42'`,
`'90s'`) is a quoted number and not an elision.

Fixed by n156.
`closerOfQuotedNumber` in `src/nested-single-quotes.ts` finds a leading elision mark,
two digits,
an optional `s` and then a straight mark that no word follows,
and returns the closer as a list of one;
`nestedSinglePairs` pairs the opener with that closer at once and skips a closer already paired.
`'90s were fun'` keeps its elision reading.
The tests were red first,
and the old case `'the '90s'` of the restore-typography test changed to `'the ‘90s’`,
which is B604.
The guard returning no closer failed with that of B602.

## B604: `'the '90s'` ships its outer opener straight

Found on 2026-10-15 by the lead,
on the output of n156 (`'the '90s'` shipped `'the ‘90s’`),
and confirmed by the review of n156
(n157).

Decided on 2026-10-15.
Built by n159;
landed on 2026-10-15 with the tests `5fe3173cc5` and the code `277145a862`,
in one landing with the fixes of n161.

What a reader of the page would see:
the outer opener of a quotation that holds a quoted decade stays straight,
beside a curly pair.
Before n156 the same text shipped `‘the ’90s’`,
the wrong glyphs on the other mark.
The `'the` has no closing mark left and so stays straight,
which is not a pair curled on one side,
and the author of n156 recorded it as the result of the rule of B603.

The lead ruled:
between the elision reading and the quoted-number reading of `'NN`,
take the one that leaves fewer marks unpaired,
and a tie keeps the elision.

Built by n159,
and landed with the fixes of n161 and not as its own commit.
- The reading of a line is chosen in the new `src/nested-single-line-readings.ts`
  (pure arrays:
  `LineState`,
  `EMPTY_LINE`,
  `withMark`,
  `withQuotedNumber`,
  `distinctDepths` and `bestPairsOf`),
  which `src/nested-single-quotes.ts` uses,
  and the header of that file states the tie rule.
  `'the '90s'` gives `‘the ’90s’`,
  and `'a '42' b` gives `‘a ’42’ b`.
- Tests:
  `src/nested-single-line-readings.unit.test.ts` (new),
  `src/nested-single-quotes.unit.test.ts` and `src/restore-typography.unit.test.ts`.
  The guards that make equal pair counts keep the first reading,
  or that keep the quoted-number-only reading,
  fail 3 and 3 rows in the readings test,
  2 and 2 in the nested-quotes test,
  and 2 and 2 in the restore-typography test.
- Overridden pins,
  each a behaviour that the shown-text principle of B597 or the tie rule now overrides,
  flagged by the author and accepted by the lead:
  the elision row of `src/restore-typography.unit.test.ts`
  (`He said 'the '90s' and left.` is now `He said ‘the ’90s’ and left.`),
  its class-seventy row
  (`*Cat Chorus*'sad tune'` now curls both marks,
  as the plain-text twin `Cat Chorus'sad tune'` does),
  the U6 rows of `src/nested-single-quotes.unit.test.ts` for `*Cat Chorus*'sad tune'` and `_Cat Chorus_'sad tune'`
  (the earlier ruling that they stay read as before is overridden by B605,
  and the row name changed),
  and its W7 row 2:
  `the 1990s and '90s and '90s' music` is now `the 1990s and ‘90s and ’90s’ music`,
  since the literal tie rule keeps the elision reading and the old code always paired the quoted number.

The review of n159 (n160) tried the rule on handpicked lines and on every string of up to eight units over `'`,
`a`,
`9` and a space (87,381 strings):
12 differ from the build before n159,
all of the shape `'<words> '<digits>'` where that build left the first mark straight
(`'a '99'` was `'a ‘99’` and is now `‘a ’99’`).
Two ties keep the outer pair:
`'99 '99'` gives `‘99 ’99’` where the old build gave `’99 ‘99’`,
and `the '70s and '80s'` gives `the ‘70s and ’80s’`.
The ruling can be read either way for those two,
and both outputs pair every mark.
No new output curls one mark of a pair and leaves its partner straight.

Corpus effect:
typography changed 0 characters on the 92 pages.
The elision recount is the same on the build before n159 and on the new one
(115 mismatches:
63 `'` to `’`,
48 `’` to `'`,
4 `‘` to `'`,
and the one original elision mark),
with no newly refused site,
and the 50 run sites are unchanged.
The count of 70 of the earlier recount differs from both,
since the mask of n156 left more marks straight.

## B605: opening marks beside markup stay straight, and a quoted `n` between emphasis marks takes the wrong glyph

Found on 2026-10-15 by the review of n156
(n157,
finding X7).

Decided on 2026-10-15.
Built by n159 and corrected by n161 (B616 to B618);
landed on 2026-10-15 with the tests `5fe3173cc5` and the code `277145a862`,
in one landing with the fixes of n161.

What a reader of the page would see,
on a curly page,
from a generator of 137,592 pairs and 66 free rows
(the review could not run it on the build from before n156,
so it cannot say which of these predate n156):
`<b>'hi'</b>` ships with both marks straight,
while `*'hi'*` curls,
because a mark right after the end of a tag is not opening-shaped (`opensAfterQuote`),
and no pair is split.
`<b>'*hi$x$'</b>` and `"'*hi$x$'` curl the first mark as a closing apostrophe and leave the second straight
(not a shape that a translated page is likely to carry).
`rock *'n'* roll` ships `rock *‘n’* roll`
and `rock **'n'** roll` ships `rock **‘n’** roll`:
the elision pair is read as a quotation when markup stands before it,
and the glyphs should be `’n’`.
`rock '*n*' roll` ships `‘*n*’`.
Of 15,542 mixed outputs,
14,592 are an opener directly after `/`,
`@`,
`#` or `$`,
which the ruling's own definition excludes,
as it does `' hi'`;
the review lists that as a count and not as a finding.
`'tis` after punctuation,
a possessive before `*` and `'90s` are right.

The lead ruled,
as one principle with B597:
the pairing context of typography reads shown text,
and the units that the prose view does not read as words are transparent.
This fixes `<b>'hi'</b>` and `rock *'n'* roll` (`’n’`).

Built by n159,
with B597,
and landed with the fixes of n161 and not as its own commit.
`restoreTypography` in `src/restore-typography.ts` reads the shown text of the replacement
through `src/typography-shown-view.ts`
and writes the curled marks back (its header gained a paragraph),
so emphasis,
strike and code delimiters,
link brackets with destination and title,
tags,
comments and expressions are transparent to the pairing.
`src/restore-ellipsis.ts` and `src/quote-normalize.ts` changed for B613 only.
The tests are those of B597,
with the same red and guard counts for the shown view.
Probe `p-opening-marks`:
the mixed outputs went from 15,542 to 13,757,
the `<b>` prefix class from 38 to 0,
`rock *'n'* roll` now ships `’n’`,
`'the '90s'` ships `‘the ’90s’` (B604),
and `<b>'hi'</b>` is curled.
The mixed classes that remain (an opener after `"`,
`/`,
`@`,
`#`,
`$` or `\*`) predate the round,
and their counts went down.
The review of n159 found the plain forms `/'hi'` and `@'hi'` fully straight on both builds,
and `/'*hi*'` improved (the old build curled only the closer).
Its twin probe found no case where invisible delimiters change the curling:
0 against 4,360 on the build before n159.

Corpus effect:
typography changed 0 characters on the 92 pages.
No separate corpus count was measured for the shapes of this finding.
