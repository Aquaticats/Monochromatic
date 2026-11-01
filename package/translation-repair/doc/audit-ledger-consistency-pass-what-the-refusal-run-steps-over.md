# Audit ledger: the consistency pass, what the refusal run steps over

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

The refusal run is the stretch of text that a gain looks across
to decide whether a kept mark stands next to the arriving one.
Its model is filed in
[Audit ledger: the consistency pass, the re-review and the refusal run](audit-ledger-consistency-pass-re-review-and-the-refusal-run.md).
This file holds what the three reviews after that file found in the run,
in what it reads as a break between two marks,
and in the page-wide refusal for a reference that is a mark,
with the rounds that fixed each finding:
the review of the fixes of n149 (n150) and n151,
the review of n151 (n153) and n154,
and the review of n154 (n155) and n156.
The review of n156 (n157) found one more doubled mark of the run,
and it is filed with the readers that parse a slice alone in
[Audit ledger: the consistency pass, the prose mask and slices parsed alone](audit-ledger-consistency-pass-the-prose-mask-and-slices-parsed-alone.md).
Every entry here has landed in the working tree,
and none of them is committed.

The findings about span edges are filed in
[Audit ledger: the consistency pass, span edges as whole units](audit-ledger-consistency-pass-span-edges-as-whole-units.md),
and those about elisions and pairing in
[Audit ledger: the consistency pass, elisions, quoted numbers and pairing](audit-ledger-consistency-pass-elisions-quoted-numbers-and-pairing.md).

Written on 2026-10-15 (UTC) from the three reviews
(`s217/review.md`,
`s221/review.md` and `s224/review.md`),
the hand-backs of n151 (`s219/handback-draft-items.md` and `s219/handback-n151-rest.md`),
n154 (`s222/handback-n154.md`) and n156 (`s225/handback-n156.md`),
and the lead's notes of 2026-10-14 and 2026-10-15.
The labels V1 to V7,
U1 to U8,
W1 to W9 and X1 to X11 are the reviews' own and are no ledger ids.
The labels of probe scenarios
(T2,
S5,
ZA,
B1 to B9,
E1 to E4b,
F0 to F4b,
G6,
K1 to K4,
ZE to ZS)
and the guard labels of the hand-backs are the sources' own too.
A path that starts with `src/` is under `package/module/translation-repair/` (moved on 2026-10-18 to the packages under
`package/translation-repair`).
Every function,
type and constant of the package named here was found declared in non-test source of the tree `w158`
by searches of declarations on the day of writing,
except those an entry says were removed.
The names that only a review or a hand-back gives
(test names,
guard names,
the helpers of test fixtures,
the names of probes and the names of logs)
were not searched.
No line numbers are cited,
since the tree is still changing.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B575: a reference that is the chosen words' own edge mark shipped a doubled mark

Found on 2026-10-14 by the review of n149 (n150,
finding V1).

Outcome written on 2026-10-15.
Fixed by n151;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
two marks.
The chosen place `He ate &#39;tis the cat now.` and the receiving place `He ate ’ the kitten now.`
shipped `He ate ’ &#39;tis the cat now.`,
and a kept reference before the receiving words (`He ate &#39;the kitten now`)
shipped `He ate &#39;&#39;tis the cat now`.
The same page with a literal `’tis` in the chosen words was refused,
and with nothing before the receiving words the gain was carried with one mark (T7,
T8,
T9).

The cause:
`planConsistencyEdit` in `src/consistency-splice.ts` asked the page check (`pageReference`)
only when the words brought an edge apostrophe (`bringsEdgeApostrophe`),
and that reading did not count a leading reference.
`gainsBesideKeptMark` returned early for words that bring no edge apostrophe,
so the refusal run was not consulted either.
A second gap was structural:
a slice with English that the page writes nothing for is read alone and holds no `page`,
so its text was in no `page.text`.

The lead ruled:
on a page whose English holds a quotation-like reference every splice is `unavailable`,
with the named reason
(`edge-reference-mark` of B564),
and the check reads the English of every slice of the page,
slices read alone included.

Fixed.
The gate on a gain is gone from `planConsistencyEdit`:
once the words differ from the receiving words the page check is asked.
`src/corpus-run/pass-consistency-slices.ts` has a new fact,
`wholePageReferences`,
built from every present slice,
and `pageReference` in `src/corpus-run/pass-consistency-decide.ts` unions it with the references of the page.
Each test was red first:
the planner,
the slices,
the decision,
the edge refusal,
and the shipped pages T2,
T4 to T7 and a lone slice.
The guards were removed once each and failed their named tests or pages.

Checked by n153:
the author's 195 scenarios and 87 extra ones were rerun.
A reference in a heading,
link text,
a table,
a footnote definition,
another section or the chosen words ends every place `kept-current`,
and the receiving words' own reference too.
A reference in an image's alt text,
a link title,
inline code,
a comment or behind a backslash is carried.
The corpus holds no reference page (n151's count:
0 pages).
Not checked:
a reference reached through an MDX expression or a JSX attribute value.

## B576: zero-width and format characters, and the line and paragraph separators, ended the refusal run

Found on 2026-10-14 by the review of n149 (n150,
finding V4).

Outcome written on 2026-10-15.
Fixed by n151;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
`dogs’  ’ bowl`,
the shape of the page that the run refuses when a space stands there.
`The kittens <U+200B>’ bowl is empty.` shipped `The dogs’ <U+200B>’ bowl is empty.`
for U+200B,
U+200C,
U+200D,
U+200E,
U+2060,
U+00AD,
U+034F and U+180E,
and U+2028 and U+2029 shipped `The dogs’<separator>’ bowl` even with no space.
With the zero-width character directly after the word the pass refused for another rule.

The cause:
`isRunUnit` in `src/consistency-closing-marks.ts` stepped over `HORIZONTAL_SPACE`
(`src/consistency-markup-runs.ts`)
and over nothing else that is invisible,
so the run ended at the zero-width character.
The header said that the run ends there,
so the code did what its text said;
the ruling for B566 said that the run's white space is the proof's own
and did not weigh characters that a reader does not see at all.

The lead ruled:
the run steps over every `Default_Ignorable_Code_Point`,
and U+2028,
U+2029 and U+0085 count as a single line break.

Fixed.
`isDefaultIgnorable` in `src/renders-as-nothing.ts`,
`BREAK_UNITS` and `ignorableWidthAt` in `src/consistency-run-steps.ts`,
used by `src/consistency-closing-marks.ts`.
The tests were red first.
Two pages the review expected to change (U+034F directly beside the words) were red and went green for another reason:
the combining mark joins the letter,
so the place ends `unlocated` before and after.
They stay as regression pages,
and the step over U+034F is guarded at unit level.

Checked by n153:
against the property over every code point from U+0000 to U+10FFFF,
no default-ignorable code point is left unstepped.
Forty others are stepped without being default-ignorable
(white space,
`*`,
`_`,
`~`,
the backtick,
`"`,
and brackets by side).
Counting U+0085,
U+2028 and U+2029 as breaks led to B580.

## B577: a reference behind a backslash was counted, and a comment stretch differed from the parser's

Found on 2026-10-14 by the review of n149 (n150,
finding V5).

Outcome written on 2026-10-15.
Fixed by n151;
uncommitted at the time of writing;
commit:
pending.

Two shapes,
both rare.
`kittens\&#39; x` counted `&#39;`,
though the parser shows the literal text `&#39;`,
so every gain on such a page was refused:
the safe direction,
a repair withheld.
`x <!-- y <!--> &#39; z --> w.` returned no reference,
while the parser's text holds `'`:
the comment ends at the dashes that the second opener shares,
so the reference is text,
and a gain was carried on a page whose reference is shown.

The cause:
`commentUnitsOf` in `src/consistency-unread-units.ts` took a stretch from every opener,
including openers inside a comment,
while `src/mask-html-comments.ts` scans without overlap.
The divergence was inherited from the rewrite of B571 and not caused by it:
the rewrite matched the old literal reading on 60,000 random strings,
and the old quadratic search had the same union of stretches.
Against the parser's mask there were no mismatches in 80,000 strings built from well-formed pieces or loose dashes,
and 8,389 of 40,000 once `<!-->` or `<!--->` is a piece.

The lead ruled:
an escaped `&` starts no reference,
and the comment stretches are the parser mask's,
one definition.

Fixed.
`commentRegionsOf` in `src/mask-html-comments.ts` is shared by `maskHtmlComments` and by `commentUnitsOf`,
`isEscapedAt` in `src/consistency-unread-units.ts`
and the check for an escaped `&` in `src/consistency-quote-references.ts`.
The tests were red first.
The guards for `commentUnitsOf` and for the escape check were removed once each and failed.

Checked by n153:
the third family reads no mismatch now (8,389 before).
That check compares two readers of one set of regions,
so it proves that the units follow the regions and not that the regions match a Markdown renderer.

## B578: a line break inside a setext heading ended the run, and why a heading builds no group

Found on 2026-10-14 by the review of n149 (n150,
finding V6).

Outcome written on 2026-10-15.
Fixed by n151;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen,
by CommonMark and not run:
a heading whose text holds a soft break,
shown as a space,
so a gain would give `The dogs’ ’ x` inside one heading.
`markPastRunOf` returned no mark for `The kittens` and a break and `’ x` and a break and `===`
(also `---`,
two spaces and a break,
and a backslash and a break),
and for the leading side `He ate ’` and a break and `the kitten now` and a break and `===`.

The cause:
`src/consistency-paragraphs.ts` collected only paragraph nodes,
so `insideParagraphAt` was false inside a heading.
At the pass level every heading shape built no consistency group in the review's harness,
so the finding could not ship a mark.
It was a divergence at unit level from the rule "a line break inside a paragraph",
and the review did not establish why headings build no group.

The lead ruled:
the run continues across a single break inside any block whose inline content spans lines
(a paragraph or a setext heading),
and the author states why headings build no group.

Fixed.
`src/consistency-paragraphs.ts` reads the text range of a heading,
with tests in its unit test and in the closing-marks test (setext).
The author's account of the group:
an English with a heading that the source lacks has three heading chunks against the source's two
(a heading node starts a chunk in `src/chunk-document.ts`),
so the chunk counts differ and `alignDocumentSections` sends the sections to the forced aligner,
which refuses both.
Every occurrence is then `in-no-slice` (`src/repeat-groups.ts`),
and the group is built with no placed place and the verdict `not-asked` for `too-few-placed`,
so the pass never reaches the splice.
That is the harness's page shape:
a heading present in both languages can be carried.
A control page with a paragraph has two slices and is asked.
n153 reran the unit probe:
four setext shapes now refuse.

## B579: header and TSDoc sentences about the run that the code did not make true

Found on 2026-10-14 by the review of n149 (n150,
finding V7).

Outcome written on 2026-10-15.
Fixed by n151;
uncommitted at the time of writing;
commit:
pending.

Four sentences:
the header of `src/consistency-markup-runs.ts` said that the run ends at "a zero-width character",
though U+FEFF is in `HORIZONTAL_SPACE` and the run steps over it,
and the TSDoc of `HORIZONTAL_SPACE` lists the byte order mark correctly.
`src/consistency-quote-references.ts` said that "a reference counts only where the page shows it as text",
and `\&#39;` counted (B577).
It also said that it "refuses every gain on such a page,
wherever the reference stands",
which is true for gains and left a reference carried as the chosen words' own edge mark unrefused (B575).
`src/consistency-splice.ts` repeated the statement about gains,
accurately,
and it hid B575.

Fixed.
n151 rewrote the headers and TSDoc of `src/consistency-markup-runs.ts`,
`src/consistency-closing-marks.ts`,
`src/consistency-quote-references.ts`,
`src/consistency-quote-bounds.ts`,
`src/consistency-edge-marks.ts`,
`src/consistency-fold.ts` (one TSDoc),
`src/consistency-count.ts`,
`src/corpus-run/pass-consistency-edge-refusal.ts`,
`src/consistency-splice.ts`,
`src/nested-single-quotes.ts`,
`src/quote-neighbours.ts` and `src/mask-html-comments.ts`.
n153 ran 131 single-call examples of the exported helpers in eighteen files and every one held;
the sentences it found untrue after the round are filed with their findings
(B591 for the edge sentences,
B583 for the version,
B584 for the sentence about a refused English).

## B580: a line of only a separator or some spaces was read as a blank line, and a doubled mark shipped

Found on 2026-10-14 by the review of n151 (n153,
finding U4).

Outcome written on 2026-10-15.
Fixed by n154;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
`The dogs’`,
then a line holding only a space or an invisible character,
then `’ bowl`,
in one paragraph:
two marks in a row for the eye.
The shapes that shipped them:
a line of one en space between two line feeds (B1),
a line of one line separator (B2),
two paragraph separators in a row (B3) and a line separator with a paragraph separator (B3b),
a line of one no-break space between a line feed and a carriage return (B4),
a line of one U+FEFF between two carriage returns (B5),
and a line of one U+0085 (B6).
A single line separator (B7) was refused as ruled,
and a real blank line (B0) shipped a separate paragraph,
which is right.

The cause:
`isSingleBreakAt` in `src/consistency-run-steps.ts` called a break with horizontal white space beside it
and another break past that white space a blank line,
where horizontal white space is the 21 characters of `HORIZONTAL_SPACE`,
wider than the space and tab that end a CommonMark paragraph.
`BREAK_UNITS` counted U+0085,
U+2028 and U+2029 as breaks since B576,
so two of them in a row,
or one beside a carriage return or line feed,
looked like a break,
white space and a break.
`maskInvisibleLines` splits on the line feed only,
so a line made with lone carriage returns is never masked and the grammar keeps one paragraph.
Over 150,000 random strings of invisible characters and markup,
20 strings were missed with the separators left out and 1,488 with them in.

The lead ruled:
whether two marks share a paragraph or a setext heading is read from the pass's own parse
(the chain of `parseSliceBody`),
and the run's own character test for a blank line goes.

Fixed.
`isSingleBreakAt` is now a break unit that `insideParagraphAt` finds inside a paragraph,
`horizontalEndOf` and the character test are gone,
and an English that the grammar refuses steps every break,
blank lines included:
the conservative direction.
The tests were red first
(the separator expectations of the run-steps test flipped,
the shipped pages B1 to B6,
B3b and B7,
and the carried controls B0,
B8 and B9).
The guard that asks the parse (`|| true` in its place) failed 12 assertions.
The seven pages that doubled the mark now keep `kittens`.
In the grid of random strings the missed count went from 1,511 to 1,
and the overstepped count from 18,748 to 19,220:
all of them hold markup that the run takes on purpose (`*`,
`_`,
`~`,
a backtick,
a tag),
and the rows with white space,
invisible characters and breaks alone read 0 and 0.
The one row missed is B586.
B8 and B9 were moved by B589.

## B581: controls other than the breaks, and the Braille blank, were not stepped by the run

Found on 2026-10-14 by the review of n151 (n153,
lower finding).

Outcome written on 2026-10-15.
Fixed by n154;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen,
if a renderer shows nothing for the control:
a gain beside a kept mark past `kittens` and U+0001 would double the mark for the eye.
The run stepped neither U+0000 to U+0008,
U+000E to U+001F and U+007F,
nor U+2800.
This was shown with `markPastRunOf` only,
not at the pass level.

The lead ruled:
the C0 controls other than the breaks and U+007F are stepped,
U+0000 is shown as U+FFFD and ends the run,
and U+2800 is stepped as white space.

Fixed.
`CONTROLS_THAT_SHOW_NOTHING`,
`BRAILLE_BLANK` and the private `showsNothing` in `src/consistency-run-steps.ts`,
used by `ignorableWidthAt`.
`HORIZONTAL_SPACE` is unchanged on purpose,
because `consistency-markup-runs.unit.test.ts` pins it equal to the white space of JavaScript.
The shipped pages ZE to ZJ cover U+0001,
U+0008,
U+000E,
U+001F,
U+007F and U+2800 at three positions each
(red first),
and the page with U+0000 (ZK) is kept by another guard,
so its end of the run is pinned at unit level.
U+001C now steps.
The guard that reduces `showsNothing` to the default-ignorables failed 7 assertions.
Two later findings corrected what this round said:
B585 (the C1 controls) and B587 (what the C0 controls show).

## B582: `<!-->` and `<!--->` are read as unterminated by the shared mask (accepted)

Found on 2026-10-14 by the review of n151 (n153,
item 4).

The lead accepted it on 2026-10-14.
The shared mask reads `<!-->` and `<!--->` as comments that never end and masks to the end of the text,
while CommonMark 0.31 calls both complete.
No page of the corpus holds either,
and the search reached all 92 pages (a positive control).
The mask has fourteen production call sites in thirteen files,
so the pass and the page parser agree with each other.
What is open is what the published site's pipeline does with the text after such a comment,
and no renderer was run.

The same review measured the cost of the comment readers:
best of three at 16,000 to 256,000 openers is close to linear
(16 times the openers cost 15 to 29 times as much),
and `unreadUnitsOf` and `markPastRunOf` take 8 s at 32,000 repetitions of `\&#39; ` because `parseMdxBody`
takes 7.3 s on the same text:
the cost is the parser's.

## B583: a header named a Unicode version that the runtime does not ship

Found on 2026-10-14 by the review of n151 (n153,
finding U7).

Outcome written on 2026-10-15.
Fixed by n154;
uncommitted at the time of writing;
commit:
pending.

`src/renders-as-nothing.ts` said that its set is `Default_Ignorable_Code_Point` from
`DerivedCoreProperties.txt` of Unicode 18.0.0.
The set is made by `/^\p{Default_Ignorable_Code_Point}$/u`,
which uses the runtime's tables,
and the runtime of the review ships Unicode 17.0.
Equality with the property holds by construction,
so the harm was the claim and not the behaviour.

The lead ruled:
say that the file reads the runtime's tables.

Fixed.
The header says that it reads the runtime's own Unicode tables and applies no version.
A test in `renders-as-nothing.unit.test.ts` reads the source,
red first,
and the guard that puts a version back in the header failed it.
The file also named a class by its position once,
which the source scan found,
and it now names `SHOWS_NOTHING` and `IS_DEFAULT_IGNORABLE`.

## B584: a header said that a grammar-refused English never carries a mark, and no run showed it

Found on 2026-10-14 by the review of n151 (n153,
item 7),
asked of n154 by the lead as a check.

Outcome written on 2026-10-15.
Settled by n154;
uncommitted at the time of writing;
commit:
pending.

The header of `src/consistency-paragraphs.ts` said that an English the grammar refuses
"keeps a mark from being carried beside a mark and never carries one".
`markPastRunOf` alone does not make that true:
on a refused English the run steps three quotation markers and refuses nothing at a list,
a heading or a blank line.
Whether the pass splices on such a page was not checked.

n154 ran the pass.
With a real grammar refusal in the first English
(an unclosed brace,
a tag with a character that no name holds,
an unclosed tag,
a closing tag that closes nothing;
`a < b` is no refusal)
the pass keeps the receiving words of that English whatever stands beside them,
even where no mark does,
and the same English parsed carries the words.
So the pass never carries a mark there,
and the run's reading is not the reason:
the whole English is withheld.

Fixed.
The sentence now says that the run holds at least what any parse would hold
and that the receiving places of such an English keep their words at the pass,
with the test file named.
Fourteen shipped pages pin it
(`REFUSED_ENGLISH`:
four refusals in three spellings between the receiving words and the rest,
and two parsed controls).
They characterize behaviour that existed,
so no red or guard applies.

## B585: the C1 controls ended the run, and a doubled mark shipped

Found on 2026-10-15 by the review of n154 (n155,
finding W4).

Outcome written on 2026-10-15.
Fixed by n156;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
`See kittens <C>’ bowl is empty`,
with a space before a C1 control and the chosen words `dogs’`,
shipped `See dogs’ <C>’ bowl is empty` for U+0080 and U+009F.
With U+0001,
U+007F,
U+200B and U+2800 in the same four shapes the pass kept the receiving words.
A C1 control directly after `kittens`,
with no space,
is kept by another guard.
In headless Chromium the controls U+0080 to U+009F take no width,
the page parser keeps them as text,
and the run returned no mark for all 31 of them
(U+0085 is a break).

The cause:
`CONTROLS_THAT_SHOW_NOTHING` in `src/consistency-run-steps.ts` held the C0 controls and U+007F only,
and the ruling of B581 had named the C0 controls.

The lead ruled:
step them.

Fixed.
The set gains U+0080 to U+0084 and U+0086 to U+009F,
as named constants,
and U+0085 stays a line break.
The unit test (`C1_CODES`) and four shipped pages (ZP U+0080,
ZQ U+0084,
ZR U+0086 and ZS U+009F) were red first.
The pages that doubled the mark now keep the group.
In the grid of random strings the overstepped count rose from 19,220 to 19,732,
which the author attributes to the C1 controls the run now steps
(the safe direction),
by cause and not code point by code point.
Other renderers were not run;
Firefox may draw controls as boxes.

## B586: a lone container tag line followed by a no-break space left one paragraph that the run read as two

Found on 2026-10-15 by the author of n154 (the one miss of its grid),
cause found on 2026-10-15 by the review of n154 (n155,
finding W5).

Outcome written on 2026-10-15.
Fixed by n156;
uncommitted at the time of writing;
commit:
pending.

The page:
`The kittens`,
a line break,
`<Note>`,
a no-break space,
a carriage return and line feed,
a soft hyphen,
`’ x`.
The pass's own parse holds one paragraph,
and `markPastRunOf` returned no mark.
The author guessed that the run misses a step over a tag;
that was wrong,
and the run is not the place to fix it.

The cause:
`maskLoneContainerTags` read the line with `line.trimEnd()`,
which drops the white space of JavaScript including U+00A0,
so `<Note>` and the no-break space counted as a lone tag line,
and the mask blanked only the trimmed length,
so the no-break space survived.
A line of spaces and one no-break space is not blank to CommonMark,
so the parse kept one paragraph while the run read the tag line as a separator.

The lead ruled:
the lone-tag test uses Markdown white space (space and tab),
as the MDX parser does.

Fixed.
`LINE_END_SPACE` (space,
tab,
carriage return) and `trimLineEnd` in `src/mask-container-tags.ts` replace `trimEnd()`
in `tagOnLine` and in the trailing check of `multiLineTagEnd`.
No-break space,
U+2003,
U+3000,
vertical tab,
form feed and U+FEFF no longer let a tag stand alone.
The tests were red first,
and the shipped page W5 was added.
The old and the new mask were compared on the 92 pages as written,
with comments masked,
and with comments and invisible lines masked
(276 inputs):
no difference,
because the corpus holds no lone container tag at all.
The positive controls differ:
a no-break space or an ideographic space after a lone tag gave one tag before and none now,
while a space,
a tab,
nothing and a closing tag do not differ.
The callers of the mask are `src/parse-slice-body.ts`,
`src/container-half-pairs.ts`,
`src/corpus-run/insertion-container-blocks.ts` and `src/corpus-run/prose-ranges.ts`.
The missed count of the grid went from 1 to 0.
The guard that puts `trimEnd` back failed 2 assertions.

## B587: a header said that the C0 controls render nothing, and a browser draws a gap

Found on 2026-10-15 by the review of n154 (n155,
finding W6).

Outcome written on 2026-10-15.
Fixed by n156;
uncommitted at the time of writing;
commit:
pending.

The header and a TSDoc of `src/consistency-run-steps.ts` said that U+0001 to U+0008 and U+000E to U+001F render nothing.
In headless Chromium each takes 15.55 px at 20 px serif,
a blank gap wider than a space;
only U+007F and U+0080 to U+009F take no width,
and U+2800 takes 12 px.
The Hangul fillers U+115F,
U+1160,
U+3164 and U+FFA0 are default-ignorable yet are drawn at 18.39 px,
and the run steps them.
The direction is the safe one (a gain is refused across a gap),
so this was a wording fault and no shipped defect.

Fixed.
The header says that the C0 controls show a blank gap or nothing,
that the default-ignorable code points show nothing except the Hangul fillers,
and that stepping over a control is the safe direction:
the run then reaches the mark beyond it,
so a mark is never carried beside a mark,
while stepping too little would be the unsafe one.
The TSDoc of `CONTROLS_THAT_SHOW_NOTHING`,
`showsNothing` and the step-length function say "a blank gap or nothing".
A test (`headerSuite`) reads the file and asserts the new wording and the absence of the old sentence.
Firefox was not run.

## B588: which guard keeps the places whose edge is `&#39;` or `&rsquo;` (stated, no code changed)

Found on 2026-10-15 by the review of n154 (n155,
finding W8),
as units of one kind that the ruling treats alike but the pass treats differently:
a chosen place ending in `&#39;` or `&rsquo;`,
and a receiving place ending in `&#39;`,
keep every place,
while the escaped `\'` is replaced.
The review did not find the guard.

Outcome written on 2026-10-15.
Stated by n156;
uncommitted at the time of writing;
commit:
pending.

n156 bisected it.
The guard is the `pageReference` test of `planConsistencyEdit` in `src/consistency-splice.ts`:
once the words that go in differ from the receiving words
and the page holds any character reference that decodes to a quotation mark,
the plan is `unavailable`,
wherever the reference stands on the page.
The pass records the place `kept-current` in `src/corpus-run/pass-consistency-decide.ts`
with the reason `edge-reference-mark` of `src/corpus-run/pass-consistency-edge-refusal.ts`.
The same chosen words in curly marks and no reference carry to every place.
With a `&#39;` or a `&rsquo;` far from every place,
in an unrelated sentence,
every place is kept alike,
and with a far `&amp;`,
which is no quotation mark,
nothing is kept.
So the stated reason is true,
the guard is page-wide and not about a place's own edge,
and a refusal is the safe direction.

Fixed in words only.
The header of `src/corpus-run/pass-consistency-edge-refusal.ts` names the guard
and says that a place whose own edge is the reference and a place far from it are kept alike
(its line was over 120 characters and was reflowed).
Three shipped pages pin it:
W8a (a chosen place that ends in the reference:
every place `kept-current`),
W8b (a far reference:
every place `kept-current`)
and W8c (the `&amp;` control:
every place carried).
They pass on the unchanged code,
as they must.

## B589: a line of only invisible or non-ASCII space characters is inside the paragraph for the run

Found on 2026-10-14 by the review of n151 (n153,
the pages B8 and B9),
decided as a blank line,
and restated as a harm on 2026-10-15 by the review of n154 (n155,
finding W9).

Outcome written on 2026-10-15.
Fixed by n156;
uncommitted at the time of writing;
commit:
pending.

What a reader of the page would have seen:
`The dogs’`,
a line holding one ideographic space or one zero-width space,
and `’ bowl`,
shipped carried.
For a line of one U+3000,
U+200B,
U+FEFF,
U+00A0,
U+2060,
U+180E,
U+202F,
U+2007 or U+200C
the raw page parse holds one paragraph,
the parse after `maskInvisibleLines` holds two,
and the run returned no mark.
The header of `src/mask-invisible-lines.ts` says that CommonMark merges these lines,
so a reader of the raw page sees `dogs’ ’ bowl`.
Lines of an en space,
U+00AD,
U+200E,
U+2800 and U+0001 are one paragraph in both readings and were refused.
The ruling of B580 had followed the pass's chain,
which blanks such lines,
and had not weighed this harm.
The ruling reverses that part of it.

The lead ruled:
the run reads a line made only of invisible or non-ASCII space characters as inside the paragraph,
and refuses the gain.

Fixed.
`textBlocksOf` in `src/consistency-paragraphs.ts` calls `parseSliceBody`
with `invisibleLines: 'text'`
(the new type `InvisibleLinesReading` and the parameter of `src/parse-slice-body.ts`,
default `'blank'`),
so the paragraph goes on through such a line as CommonMark and the renderer read it.
The page assembly still turns that line into spaces and ends the paragraph,
and the run is the wider reading on purpose.
The tests were red first:
the blank-line test of the run steps was split
(a space or tab line stays blank),
nine characters were added to the unit tests,
and the shipped pages B8 and B9 moved from carried to refused,
with BA to BF for a no-break space,
U+FEFF,
U+2060,
U+202F,
U+2007 and U+200C.
In the probe,
twelve line shapes went from no mark to the mark `’`.
The guards for the C1 ranges and for `invisibleLines: 'text'` were removed together and failed
4 assertions in the closing-marks test,
3 in the run-steps test and the page rows.
