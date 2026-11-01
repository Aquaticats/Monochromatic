# Audit ledger: the consistency pass, span edges as whole units

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

The span the pass replaces is the words alone,
and its two edges are where a quote of the judges ends and starts.
This file holds the findings of the reviews of n150,
n153,
n155 and n157 about those edges:
an edge that cut a character reference or a backslash escape in two,
an edge that took in a reference-looking text,
and an edge that left half a word.
The rounds that fixed them are n151,
n154,
n156 and n159,
and the two findings of the review of n156 (X1 and X9) were built by n159
and landed on 2026-10-15 with the fixes of n161.
What the review of n159 found the word test still read wrongly is filed in
[Audit ledger: the consistency pass, what the word test reads as shown](audit-ledger-consistency-pass-what-the-word-test-reads-as-shown.md).
The edge marks themselves (apostrophes and quotation marks at the edge of a span) are filed in
[Audit ledger: the consistency pass, apostrophes and block floors](audit-ledger-consistency-pass-apostrophes-and-block-floors.md)
and in
[Audit ledger: the consistency pass, one reading of every mark](audit-ledger-consistency-pass-one-reading-of-every-mark.md).

The findings about the refusal run are filed in
[Audit ledger: the consistency pass, what the refusal run steps over](audit-ledger-consistency-pass-what-the-refusal-run-steps-over.md),
those about elisions and pairing in
[Audit ledger: the consistency pass, elisions, quoted numbers and pairing](audit-ledger-consistency-pass-elisions-quoted-numbers-and-pairing.md),
and those about the prose mask in
[Audit ledger: the consistency pass, the prose mask and slices parsed alone](audit-ledger-consistency-pass-the-prose-mask-and-slices-parsed-alone.md).

Written on 2026-10-15 (UTC) from the four reviews
(`s217/review.md`,
`s221/review.md`,
`s224/review.md` and `s227/review.md`),
the hand-backs of n151 (`s219/handback-draft-items.md`),
n154 (`s222/handback-n154.md`) and n156 (`s225/handback-n156.md`),
and the lead's notes of 2026-10-14 and 2026-10-15.
The landing text of B597 and B598 was added on 2026-10-15 from the review of n159 (`s231/review.md`),
the hand-back of n159 (`s230/handback-n159.md`) and the lead's notes of the landing.
The labels V3,
U1 to U3,
U8,
W2,
X1 and X9 are the reviews' own and are no ledger ids.
The labels of probe scenarios
(S0 to S8,
E1 to E4b,
F0 to F4b,
G1 to G8,
M1 to M15,
N1 to N9)
and the guard labels of the hand-backs are the sources' own too.
A path that starts with `src/` is under `package/module/translation-repair/` (moved on 2026-10-18 to the packages under
`package/translation-repair`).
Every function,
type and constant of the package named here was found declared in non-test source of the tree `w158`
by searches of declarations on the day of writing,
except those an entry says were removed;
`partStartAt`,
`keepsClosingAt`,
`ESCAPABLE_CLOSING` and `referenceStartIn` were searched in the four source files that held them and are gone.
The functions and constants named in the landing text of B597 and B598
(`transparentUnitsOf`,
`cutsShownWord`,
`shownRangesOf`,
`shownViewOf`,
`writtenBack`,
`restoreTypography`,
`isSpacedScriptWordCharacter`,
`SINGLE_MARK_LETTERS` and `JOINERS`)
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

## B590: the quote bounds cut the semicolon of a reference, and a broken reference shipped

Found on 2026-10-14 by the review of n149
(n150,
finding V3).

Outcome written on 2026-10-15.
Fixed by n151 and rewritten by n154 (B591);
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
the literal text `&#39` or a stray `;`,
where an apostrophe was.
The chosen words ending in a reference
(`The dogs&#39; bowl is full. The kittens bowl. A cat naps.`)
shipped `The dogs&#39; bowl is full. The dogs&#39 bowl. A dogs&#39 naps.`,
and receiving words ending in one
(`The cats bowl is full. The dogs&#39; bowl. A cat naps.`)
shipped `The cats; bowl.`.
The same held for `&rsquo;`,
`&amp;` and `&nbsp;`;
a literal `;` was fine.
Both shipped through the settled path with no refusal and no kept reason.

The cause:
`CLOSING_PUNCTUATION` in `src/consistency-quote-bounds.ts` lists `;`,
`isClosing` is true for it,
and `quoteBounds` takes the last character that is not closing as the end of the words,
with no question whether the `;` ends a reference.
The path is older than n149;
it mattered then because n149 made references a named concern and ruled the page-wide refusal for gains only (B575).
The corpus holds no page with a reference at the edge of a repeated line,
by the author's count.

The lead ruled:
a span edge never falls inside a character reference or a backslash escape.

Fixed by n151.
`src/consistency-quote-bounds.ts` kept a reference's semicolon and an escaped mark in the words
(`partStartAt` and `keepsClosingAt`),
`dropsAtEnd` in `src/consistency-edge-marks.ts` did the same at the fold level
(found at the shipped page and not in the brief),
and the new `src/consistency-whole-end.ts` had `wholeEndOf`,
which `src/consistency-count.ts` called,
so a quote that stops before the semicolon of `&amp;` or before the stop of `\.` names the span with it.
The tests were red first:
`quote-bounds`,
`edge-marks`,
`whole-end` and the shipped pages S0 to S8.
The guards for the reference branch of the bounds and for the `wholeEndOf` call failed their tests,
and the guard for `keepsClosingAt` failed unit tests only,
since `wholeEndOf` restores the semicolon on the page.
n153 found that the fold did not follow the bounds (B591).

## B591: a span edge still fell between a backslash and the bracket or mark it escapes

Found on 2026-10-14 by the review of n151
(n153,
finding U1).

Outcome written on 2026-10-15.
Fixed by n154;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
a literal backslash after `cats`.
With the judges quoting `the cats\)` and the receiving place `the kittens`,
`He said (the cats\) was here. He said the kittens was there. A cat naps.`
shipped `He said the cats\ was there. A the cats\ naps.`
(E1b;
`\]` in E3c and `the the cats\` in E1 shipped the same).
`\"` and `\'` ended `kept-current` for another reason,
but the cut was the same.
At the start edge,
`He said \*kittens* now.` with the quote `*kittens*` shipped `He said \dogs’ now.`
(F4;
`\(kittens)` was safe,
since the drop of the opening bracket leaves the escape whole).
The two span definitions disagreed on one quote:
`quoteBounds` kept `the cats\)` and the located core of the count was `the cats\`.

The cause:
only the `isClosing` branch of `dropsAtEnd` in `src/consistency-edge-marks.ts`
asked whether a backslash escapes the character.
The branches for the straight double mark,
for any single mark and for a closing bracket dropped it without asking,
and the set the escape test covers,
`ESCAPABLE_CLOSING` of `src/consistency-quote-bounds.ts` (`.,;:!?`),
is a subset of what `dropsAtEnd` can remove.
The header sentences that the two readers cut the same edges by the same test were untrue.
The author of n151 had written that the start side only moves right past a closing mark and so cannot land in an escape;
that is true of `dropsAtStart`,
but a span also starts wherever the quote's first character lands.
Reach,
measured on 200,000 random strings of 43 tokens:
the end of the words fell inside an escape in 25,744
(`\)` 6,518,
`\]` 6,480,
`\"` 5,918,
`\'` 6,597,
`\(` 131,
`\[` 100),
inside a real reference in none,
and the start fell inside one in none.

The lead ruled,
replacing the wording of B590:
a backslash escape and a character reference are each one unit with the character they show;
at a span edge the unit is kept or cut exactly as that character would be
(an escaped stop or bracket stays with the place like a plain one),
and never split;
a quote whose edge falls inside a unit widens to the whole unit;
and one edge definition serves `quoteBounds` and the fold.

Fixed by n154.
The new `src/consistency-edge-units.ts` has `edgeUnitsOf`,
`textUnitsOf` and the type `EdgeUnit`,
and `ASCII_PUNCTUATION` moved there from `src/consistency-run-steps.ts`.
A backslash followed by an ASCII punctuation character as written
(the `originals` argument,
so `\’` is no escape)
is one unit shown as that character,
and an ampersand that starts a reference the parser decodes is one unit shown as the parser shows it.
The decoder is the parser itself:
`referenceShownAs` parses the lone reference with `parseMdxBody` and takes the text of the paragraph when it differs,
with a memo of 4,096 entries.
No table of names was usable,
because `decode-named-character-reference` resolves only through a stray install in an ancestor directory
and not from the package,
so it cannot be a dependency.
`src/consistency-quote-bounds.ts` is rewritten over units
(`partStartAt`,
`keepsClosingAt`,
`ESCAPABLE_CLOSING` and `referenceStartIn` are removed),
`edgeFreeRangeOf` and `edgeFreeWordsOf` in `src/consistency-edge-marks.ts` take optional `originals`,
and `src/consistency-whole-end.ts` has `wholeEndOf`,
`wholeStartOf` (widen only) and `wholeSpanOf` (widen both edges,
then trim the widened span).
`writtenPointsOf` in `src/consistency-fold-offsets.ts` is new,
`foldedWordsOf`,
`readQuote` and `locateWords` in `src/consistency-fold.ts` use the units,
and `src/consistency-count.ts` no longer calls `wholeEndOf`.
The tests were red first:
the new unit test,
the bounds,
edge-marks,
whole-end and fold tests,
and sixteen exact-page cases (`SPAN_EDGES`),
each asserting the shipped page and the four outcomes.
In the fuzz of 200,000 strings no end or start falls inside a unit
(25,744 before),
and the ends of `quoteBounds` and of the fold agree on mark-free tokens;
the start sides differ by design,
since `quoteBounds` cuts only space and invisibles at the start.
The guards failed:
the parser as decoder replaced by a shape (the edge-units tests and four pages),
the escape unit removed (15 assertions),
`originals` ignored (2),
and the widening and trimming of `wholeSpanOf` bypassed (11).

Checked by n155:
every one of the 2,125 named references of `character-entities` is one unit showing the decoded text,
and every decimal reference from `&#0;` to `&#196607;` matches the numeric decoder of micromark.
Odd shapes behave as the parser does
(`&#;`,
`&#xZ;`,
`&amp;amp;` is one unit and then text).
The memo has no stale answer:
its key is the reference text and its value a pure function of it.
Cost,
about 70 microseconds for a distinct reference;
5,000 distinct references (over the memo) take 215 to 372 ms a scan,
and `wholeStartOf` and `wholeEndOf` each scan the whole English,
so a located stand costs two scans.
A page English with more than a few thousand distinct references is not plausible,
so the reviewer called the cost a note and no finding.

## B592: an escaped full stop at a span end was part of the words, and the stop was lost or doubled

Found on 2026-10-14 by the review of n151
(n153,
finding U2).

Outcome written on 2026-10-15.
Fixed by n154;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen,
with the judges quoting through the escaped stop as the raw English shows it:
the control with a plain stop (F0) shipped `See dogs’. Then more.`,
the receiving place ending in `\.` (F2) shipped `See dogs’ Then more.`
(the sentence lost its full stop),
the chosen place ending in `\.` (F3) shipped `The dogs\.. A dogs\. naps.`
(two stops,
and a stop in the middle of a sentence),
and both ending in `\.` (F3b) shipped `The dogs\. Then. A dogs\. naps.`
A quote without the escaped stop (F2b) shipped `See dogs’\. Then more.`,
which is right.
Two judges who quoted `See cats\.` and `See cats` named different cores,
where `See cats.` and `See cats` agree:
the escape was treated as a word character and not as punctuation.

The cause:
the fix of B590 moved the pair `\.` into the words,
and the receiving span is then replaced as a whole,
so the stop was replaced with the words.
Cutting the pair as one unit at the end,
with both characters staying with the receiving place and out of the words,
keeps the ruling and the behaviour for a plain stop.

The lead ruled:
the rule of B591.

Fixed by n154 with the units of B591.
F2 and F5 keep the `\.` and the `\]`,
F3 and F3b ship no doubled stop,
and the pair of judges `See cats\.` and `See cats` is one core.

## B593: a semicolon after a word that only looks like a reference was taken into the words

Found on 2026-10-14 by the review of n151
(n153,
finding U3).

Outcome written on 2026-10-15.
Fixed by n154;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
the semicolon after `Q&A` disappearing from the receiving place
(`See Q&A; then more.` shipped `See dogs’ then more.`),
or a stray `;` inside the places the words go to
(`See R&D; then more.` as the chosen words shipped `The R&D; bowl. A R&D; naps.`).
`Call a&b; today.`,
`a&zzzz;` and an eleven-digit numeric `a&#00000000039;` behaved the same.
The parser treats none of them as a reference
(`Q&A;`,
`R&D;`,
`AT&T;`,
`a&b;` and `a&zzzz;` stay plain text nodes,
while `&amp;` and the seven-digit `&#0000039;` decode),
so the page shows the semicolon.

The cause:
`partStartAt` asked `referenceStartIn`,
which asked `referenceEndOf` in `src/consistency-unread-units.ts`,
and that function reads by the shape of the characters and not by a table of names.
This is the safe direction for the refusal run,
where an unread unit is stepped over,
and the unsafe direction for a span edge,
where "inside a reference" decides what is replaced.

The lead ruled:
a span edge counts as a reference only what the parser decodes,
by its name table and its numeric limits,
and the refusal run keeps the wider reading by shape.

Fixed by n154.
`referenceUnitAt` in `src/consistency-edge-units.ts` asks the parser.
`a&b;`,
`a&zzzz;` and the eleven-digit reference are plain text,
and `AT&amp;` is a unit.
The pages F1 (the `;` kept) and F1b (`R&D` carries no `;`) were red first,
and the control `AT&amp;` is unchanged.
Guard G1,
the parser replaced by a shape,
fails the edge-units tests and four pages.

## B594: a quote that starts inside a reference, and what its reach turned out to be

Found on 2026-10-14 by the review of n151
(n153,
finding U8),
as the measured reach of the class that the lead had ruled for the next round.

Outcome written on 2026-10-15.
Fixed by n154 and corrected by n156 (B596);
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
with the receiving quote `amp; cats` on `The dogs’ bowl is full. See dogs&amp; cats now.`,
`See dogs&dogs’ now.`
(E4;
the `&` is now literal text),
and with the chosen quote `amp; cats` (E4b)
`The amp; cats bowl is full. A amp; cats naps.`
Nothing refused either,
since `&amp;` is not quotation-like.
The escape twin is the start edge of B591 (F4).
The author of n151 had written that no judge's quote of whole words starts inside a reference,
and had not handled it.

The lead ruled,
with B591:
a span edge never falls inside a reference or an escape,
which covers both edges.

Fixed by n154 with the units:
a quote whose edge falls inside a unit widens to the whole unit.
E4 then shipped `See dogsdogs’ now`
(the reference whole,
the word still split),
which is B596.

## B595: an escaped single mark at an edge is cut as the mark it shows (accepted)

Found on 2026-10-15 by the author of n154,
and flagged to the lead as a consequence of the edge unit and not a requested change.

The lead accepted it on 2026-10-15.
An escaped `\'` at an edge changed from `kept-current` to `replaced`:
the escape is a unit that shows `'`,
so it is cut like the mark,
and the readers of marks do not see an escaped mark.
`&nbsp;` and `&#39;` at an edge cut like a space and a single mark,
and `AT&amp;T` with the quote `Call AT&` widens to `Call AT&amp;`.

Checked by n155,
on the shipped page:
with the chosen `the cats\'` and the receiving `the kittens`,
the receiving places ship `the cats`,
so the mark is neither carried nor doubled;
with the receiving `kittens\'` and the mark outside the quote,
the page ships `dogs\'`,
one mark kept in place;
with the chosen curly `dogs’` against a receiving `kittens\'`,
every place is `kept-current`.
No mark is lost from a place that had one.

## B596: the pass spliced inside a word whenever the joint was a reference, a plain ampersand or a zero-width space

Found on 2026-10-15 by the review of n154
(n155,
finding W2),
as the general hole of which the E4 of B594 is one case.

Outcome written on 2026-10-15.
Fixed by n156;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen,
with the chosen place `dogs’` and the rest receiving:
a glued or truncated word.
`See kit&shy;tens now` with the quote `tens` shipped `kit&shy;dogs’` (M3),
`See k&#105;ttens now` with `ttens` shipped `k&#105;dogs’` (M4),
`See kitt&shy;ens now` with `kitt` shipped `dogs’&shy;ens` (M5),
`kitten&#115;` with `kitten` shipped `dogs’&#115;`,
which reads as `dogs’s` (M13),
`Call AT&amp;T now` with `AT` shipped `dogs’&amp;T` (N1),
`Our R&D team` with `R` shipped `Our dogs’&D team` (N5),
`See Tom&amp; Jerry` with `amp; Jerry` shipped `See Tomdogs’ now` (M8),
and a zero-width space between `kit` and `tens` shipped `kit` and the space and `dogs’` (M15).
On the chosen side a quote starting after `k&#105;` or inside `Tom&amp;` was carried whole into the other places
(`The ttens bowl ate`).
Plain mid-word quotes,
a raw soft hyphen,
`so` in `so-called` and an escaped star were refused,
as intended.

The cause:
`src/word-bounds.ts` and `src/consistency-fold.ts`
accept an edge when the written neighbour is not a Latin letter or digit,
and the neighbour was `;`,
`&` or a zero-width space.
`cutsWord` in `src/consistency-word-joints.ts` knows hyphens,
apostrophes and a stop between digits as joints,
and not a reference that shows a letter,
a plain `&` between letters or a zero-width space.
For E4 and M8 the widening comes after the word test:
`wholeSpanOf` widened the stand that had already passed `cutsWord`,
and nothing asked again.
A raw U+00AD was refused because the fold drops it.

The lead ruled:
the word test reads shown text
(a unit as its shown character,
default-ignorable characters transparent,
and `&` between two word characters joins them)
and runs on the final,
widened edges.

Fixed by n156.
The new `src/consistency-shown-edge.ts` exports `cutsShownWord`
(also through `src/consistency-barrel.ts`).
For each edge it takes the shown characters outside the span,
nearest first,
with the units of `textUnitsOf` as their shown character and the default-ignorables skipped,
and the shown characters inside.
An edge cuts a word when the outside continues a word
(the nearest is a word character,
or a joiner,
`&`,
`-`,
`'` or the right single quotation mark with a word character beyond it)
and the edge is a word edge.
`locateWords` in `src/consistency-fold.ts` calls it on the final edges,
after `wholeSpanOf` and before `withProvenApostrophes`,
and a cut gives `{ kind: 'none' }`.
On a page,
a receiving place whose edge cuts a shown word is `unlocated` and keeps its words,
and a chosen place that cuts is refused,
so the whole group is `untouched`.
Counting is by the folded text and is not stored,
so `standsOf` is unchanged and no ballot count moves.
The tests were red first
(the new unit test failed as a load error for the missing export,
and the page rows M3,
M4,
M5,
M8,
M13,
M15,
N1,
N3 to N6 and their controls).
E4 now ships `See dogs&amp; cats now.` unchanged and `A dogs’ naps.`,
with the places `chosen`,
`unlocated`,
`replaced` and `replaced`,
and E4b ships the unchanged page with all four `untouched`.
N2 (the quote ends inside the reference of `AT&amp;T`) went from `kept-current` to `unlocated`.
Of the fifteen cases that changed in the probes,
fourteen had shipped half a word or half a reference.
The guard that disables the call failed 3 assertions of the fold test and page rows.

Corpus exposure,
measured by n156:
nine sites on the 92 pages hold a reference,
an escape,
a raw ampersand or a default-ignorable beside a word character
(two escapes in `AmbeR\_the\_anpa`,
a raw ampersand in a URL,
three zero-width spaces and three references).
No newly refused place was observed,
and the replay re-reads the artifacts without running the consistency pass,
so this is exposure and not a count of refusals.
Not checked:
whether the recorded span of a published artifact has an edge at one of the nine.

## B597: markup that shows nothing between the pieces of one word lets an edge ship half a word

Found on 2026-10-15 by the review of n156 (n157,
finding X1).

Decided on 2026-10-15.
Built by n159 and corrected by n161 (B616 to B618);
landed on 2026-10-15 with the tests `5fe3173cc5` and the code `277145a862`,
in one landing with the fixes of n161 and not as its own commit.

What a reader of the page would have seen:
`dogstens`,
`kitdogs` and `dogstens` in bold,
a word that is neither rendering.
With the chosen place `dogs`,
`The kit*tens* ate.` with `kit` as the span shipped `The dogs*tens* ate.` (M1),
with `tens` shipped `The kit*dogs* ate.` (M2),
and `The **kit**tens ate.`,
`The [kit](u)tens ate.` and `The <b>kit</b>tens ate.` shipped the same shape (M3,
M4,
M6).
M5,
with inline code,
was kept only by the inline-code floor.
In M8 the chosen place itself is half a word (`dogs*s*`),
and the pass carried `dogs` to the other places as if it were the word the reader sees
(`dogss`),
which is against the author's stated contract that a chosen place that cuts is refused.

The cause:
`nearestShown` in `src/consistency-shown-edge.ts` takes the nearest shown character outside the span,
and for `kit*tens*` that is `*`,
which is no word character and no joiner,
so `continuesWord` says no.
The same holds for `]`,
`>` and the backtick.
The header says that the test reads what a reader sees,
and it reads references,
escapes and default-ignorables,
not delimiters that render as nothing between two pieces of one word.
`src/consistency-unread-units.ts` already names the delimiters of emphasis,
code and a component's markup as units that a reader does not read as words,
and `cutsShownWord` does not use it.
The finding predates n156,
but the ruling of B596 is that the word test reads shown text.
The review did not trace whether `textUnitsOf` or the call is the place for the fix.

The lead ruled,
as one principle with X7 (B605):
the word test at a span edge and the pairing context of typography read shown text,
and every unit that the prose view does not read as words is transparent
(emphasis,
strike and code delimiters,
link brackets with destination and title,
tags,
comments and expressions),
as references,
escapes and default-ignorables already are.
This fixes `dogs*tens*`,
`**dogs**tens` and `[dogs](u)tens`,
and a chosen place that is half a word is refused.

Reach,
measured by the lead:
37 places on 16 pages of the 92,
where a letter touches an emphasis,
link or tag delimiter,
and a span edge must fall there to ship half a word.

The lead's plan was to build it after the launch and land it as its own commit,
because the checks that measure what ships had not moved in four builds (the rule of M219).
It was built as n159 in the tree `w153` and landed with the fixes of n161.

Built by n159:
- `transparentUnitsOf` in `src/consistency-unread-units.ts` flags the units that show nothing,
  and `cutsShownWord` in `src/consistency-shown-edge.ts` steps over them,
  so an edge inside a word that a reader sees is refused,
  and an edge beside markup that shows nothing is read as beside the word.
  `src/consistency-prose-text.ts`,
  `src/consistency-count.ts` and `src/consistency-fold.ts` changed with it,
  and the header of `src/consistency-shown-edge.ts` was updated.
- Typography reads the same shown text through the new `src/typography-shown-view.ts`
  (`shownRangesOf`,
  `shownViewOf` and `writtenBack`):
  `restoreTypography` reads the shown text of the replacement and writes the curled marks back (B605).
- Tests:
  `src/typography-transparent-markup.unit.test.ts` and `src/typography-shown-view.unit.test.ts` (new),
  `src/consistency-shown-edge.unit.test.ts`,
  and rows X1a to X1h of `src/corpus-run/pass-consistency-shipped-marks.unit.test.ts`.
  Red on the build before the round:
  9 failures in the transparent-markup test and 10 in the shipped-marks test
  (the controls X1g and X1h pass).
  With the single-unit filter removed,
  `src/consistency-shown-edge.unit.test.ts` fails 2 rows and the shipped-marks test 8
  (the transparent-markup test passes,
  since it does not exercise the filter).
  With the shown view of `src/restore-typography.ts` removed,
  the transparent-markup test fails 4 rows and `src/restore-typography.unit.test.ts` 2.
- Probe `p-intraword`:
  M1 to M4,
  M6 and M8 are now `unlocated` or `untouched`;
  M5 (the inline-code case) has place 2 `unlocated` and place 3 replaced.

The review of n159 (n160) held the principle for the delimiters it was built for:
a twin probe over 189 thousand short sequences (24 thousand of them comparable)
found no case where a string with invisible delimiters curls differently from the same string without them,
and the same probe on the build before n159 found 4,360.
What it found wrong in how the principle decides which units show nothing is filed as B616,
B617 and B618.

Corpus effect:
- Of the 23 word-test decisions that n159 changed on 9 pages,
  10 were from this finding on 5 pages
  (`*Yorushika*’s` on `Susiethegamer`,
  `*Arcaea*’s` on `lxyddice`,
  `[Yantian](...)’s` on `a2581911655`,
  `[𝓠𝓾𝓪𝓷](...)’s` on `mikaela_khara`,
  and the `<rt>` annotation on `windward0032`).
  The 37 places of the lead's measure counted every letter that touches a delimiter,
  and the decisions count only those that change.
- n161 restored the two `<rt>` decisions (B617),
  so as landed this finding changes 8 decisions on 4 pages,
  and the word test changes 21 decisions on 8 pages in all
  (13 from the `_` joiner of B598 and 8 from this finding).
- Typography changed 0 characters on the 92 pages.
- The claims grid located 28,137 texts on the build before n159 and 27,873 on the new one;
  the 264 lost texts are all attributed to the shapes of this finding
  (3 for each of seven plain words in the shape `*cats*s`,
  and 243 for the `**cats**` word followed by `s` or `*s`,
  or with a quote mark before it).
  The final-page failures of the oracle's column `b` went from 9 to 4,
  all one class (a free mark before `*`).
  The second grid located 22,524 texts before and 22,298 after,
  and the review's count over that grid's word list attributes all 226 lost texts to the two rulings
  (emphasis closed directly before a letter or digit,
  and the literal `_cats_`).

Accepted by the lead:
the cost of `cutsShownWord` rose up to about 2.2 times on long slices
(7.3 to 13.1 ms,
32.6 to 118 ms and 168.6 to 370.9 ms on English slices of 37 thousand,
379 thousand and 1.16 million characters;
a repeat showed a run-to-run spread of about 15 percent),
and the corpus slices are far shorter than the longest probe.
The overridden pins are listed in B604 (four of them) and in B598 (one).

## B598: words in a script other than Latin are not word characters at an edge

Found on 2026-10-15 by the review of n156 (n157,
finding X9).

Decided on 2026-10-15.
Built by n159;
landed on 2026-10-15 with the tests `5fe3173cc5` and the code `277145a862`,
in one landing with the fixes of n161.

`Привет мир` with the span `Прив` and `Γειά σου` with `Γει` return `once`,
so a cut inside a Cyrillic or Greek word ships,
and `dogs_cats` with `dogs` does too.
The word test of `src/consistency-shown-edge.ts` is `continuesLatinWordOrCompatibilityForm`.
The finding predates n156,
and English pages with Cyrillic names are rare.
The reviewer found no false refusal:
contractions,
possessives,
hyphenated names,
CJK beside Latin,
digits,
a trailing apostrophe in the words and references that show a space,
a comma or a dash all behave.

The lead ruled:
a word character at an edge is a letter,
a mark or a digit of any script,
and `_` between two word characters joins them.
No corpus reach was measured for it before the build.

Built by n159:
- `isSpacedScriptWordCharacter` in `src/latin-letters.ts` accepts a letter,
  a mark or a decimal digit of any script except Han,
  Hiragana and Katakana,
  which keep the Latin-style reading
  (an unspaced script has no word edge to cut,
  and `看kittens好` stays a control with no cut),
  and except U+02BC,
  which the module reads as a single mark (`SINGLE_MARK_LETTERS`;
  failing tests found it).
  The lead accepted both exceptions.
  `JOINERS` in `src/consistency-shown-edge.ts` gained `_`,
  and the header of that file was updated.
- Tests:
  `src/consistency-shown-edge.unit.test.ts` and `src/consistency-fold.unit.test.ts`,
  rows X9a (`kit_tens`) and X9b (a Greek letter) of `src/corpus-run/pass-consistency-shipped-marks.unit.test.ts`,
  and a direct test in `src/latin-letters.unit.test.ts`,
  which needed `isSpacedScriptWordCharacter` exported through `src/text-barrel.ts`.
  The guard run first showed the function with no direct test and no page-level row
  (2 rows failed in the shown-edge test and 3 in the fold test,
  with the latin-letters test and the shipped-marks test passing),
  and the `_` joiner with no page-level row;
  both were added,
  and with the guards removed the latin-letters test fails 2 rows,
  the shown-edge test 2,
  the fold test 3 and the shipped-marks test 2 for the letters,
  and 2 and 2 for the joiner.
- One existing pin changed:
  `catω naps` with `cat` is now `none`,
  since a Greek letter cuts,
  and the name of the case was updated.
- The review of n159 ran the function:
  true for `П`,
  a combining acute and `٣`;
  false for `猫`,
  `ʼ` and `²`.
  Its header sentence "a Latin letter or digit" was stale and is B625.

Corpus effect:
13 word-test decisions on 4 pages changed because of the `_` joiner,
all of them handles that a reader sees whole:
`AmbeR_the_anpa` on its own page (3),
`@Cang_Shan_Jing_Ye` on `GLaDOSister` (4),
`@qiandao_shi` on `dogesir_` (2),
`@awxs_2081573566` on `luxuanwen3` (2)
and `[Collapsar_ndn]` on `luxuanwen3` (2).
The Greek,
Cyrillic and other letters changed no corpus decision.
