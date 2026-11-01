# Audit ledger: comments, TSDoc and source text

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

Defects in what the source says about itself:
TSDoc that documents nothing or the wrong thing,
comments citing what does not exist,
and characters in the source a reader cannot see.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B17: the TSDoc example scan closed a fence on any three backticks

Status:
fixed in `7cc4ac642`,
2026-09-28;
found by the full suite.
`exampleCodeOf` (`tsdoc-example-scan.ts`) took the next three backticks anywhere as the closing fence,
where CommonMark closes only on a line of backticks alone,
at least as long as the opener.
`longestRunOf`'s example,
whose string quoted a fence,
was cut short and read as leaving out `character`,
and the package-wide scan failed;
the merge (`304ae283d`) had passed its own tests,
and the suite that caught it ran once,
after the audit-area-six commits.
The scan now reads fences line by line;
a fixture quoting a fence passes,
the same fixture missing a key is still found,
and a mutant restoring the old close fails 2 assertions.
The suite after it:
0 FAIL,
1,301 PASS lines.

## B96: TSDoc blocks that document nothing

Found 2026-10-01 (UTC) by a census of every TSDoc block in the package
(`~/temp/agent/audit-glossary-fix/orphan-tsdoc-census-report.md`,
29,778 blocks read forward from each block to what follows it),
fixed in the commit adding this entry.
A TSDoc block documents the declaration directly after it.
30 stood where no declaration claimed them:
before a second TSDoc block,
before a `//` comment,
or before a statement,
so a reader at the declaration saw the other block or none,
and the tsdoc lint plugin never read them,
since every one of its rules walks from a declaration back to the nearest block
(`createTsdocVisitor`,
`package/oxlint-plugin/tsdoc/src/rule/tsdoc-visitors.ts`).
Twelve moved to the declaration they describe,
three of them merged with a shorter block already there;
eight that described statements became `//` comments;
five that repeated the declaration's own block went,
two after folding in a clause the survivor lacked;
and five pairs of a TSDoc block and a `//` comment became one block.
Seven file headers gained the `@module` tag the package's other headers carry.

37 more blocks were separated from their declaration by a `disable-next-line` lint suppression,
the shape the repository's rule forbids
(a suppression on a documented declaration opens before the TSDoc and closes after the declaration).
They were rewrapped in a change of their own,
which converted the one `eslint-disable-next-line` among them to the `oxlint` form
and merged two neighbouring suppressions in `cased-letters.ts` into one pair;
in each of the 27 files every rule a block disables is enabled again
(a count of the disable and enable comments per rule,
2026-10-01 UTC),
and the lint reported 0 warnings and 0 errors after it.

What enforces it:
`src/tsdoc-attachment.unit.test.ts`,
among the source scans,
reads forward from every TSDoc block and fails on one followed by anything but what it documents:
a declaration,
a class or object member,
an enum or union member,
or a function expression returned or passed;
a block holding `@module` is exempt.
Its fixture case finds five orphan shapes and none of the legitimate ones;
with one `disable-next-line` suppression put back between a TSDoc and its declaration in `renders-as-nothing.ts`,
its package case failed,
and passed once the line was removed.

Recurrence:
`mistake-prevention.md`,
"Lint and edits".

## B119: 16,856 lines ended in whitespace, which `.editorconfig` forbids and nothing checked

Red in `f6bf25ef2`,
fixed in `7f529468e`.
The repository's `.editorconfig` sets `trim_trailing_whitespace = true` for every file but HTML,
yet no configured check enforces it for TypeScript:
dprint's TypeScript plugin is disabled in `package/config/dprint/index.json`,
and the repository's stylistic oxlint plugin has no such rule.
At `f6bf25ef2` the package held 16,856 lines ending in spaces or tabs in 1,316 TypeScript files,
nearly all of them TSDoc blank lines written as a lone space:
at `6d8889c90`,
its 2,078 TypeScript files held 25,568 TSDoc blank lines,
16,851 holding whitespace and 8,717 empty,
and 285 files mixed both forms.
An editor honouring `.editorconfig` strips the lines it touches,
so every edit carried whitespace-only diff lines,
and the edit tool,
which writes a blank TSDoc line empty (M103),
left files mixing the two forms.

`trailing-whitespace.unit.test.ts` fails on a line ending in anything `trimEnd` removes,
unless a template literal's text covers part of that run,
since a string or regular-expression literal cannot hold a line end;
none of the 16,856 sat in one.
A scratch normalizer that parses each file,
strips only outside literals,
and refuses a file whose literals change,
made `7f529468e`:
16,849 lines,
the other 7 having gone with the TSDoc blocks B118 deleted;
`git diff --ignore-all-space` over it is empty.
The same lines run through the whole repository:
30,859 more in 2,800 TypeScript files across 136 other packages,
every one of them whitespace-only;
issue 585 asks for a stylistic rule with an autofix,
after which this package's scan can go.

Recurrence:
`mistake-prevention.md`,
"Lint and edits".

## B333: case names and a comment cited clusters the ledger never names

Fixed in `e1c84094e`.

Found by the same cross-check.
Six case names in `fidelity-damage.unit.test.ts`,
`openrouter-cost.unit.test.ts` and `translate-skeleton.unit.test.ts`
ended citing "the fidelity cluster" of ledger T8 and the like,
and a comment of `group-run-anchor.ts`,
which the cross-check did not name,
cited "the group cluster";
no ledger entry names those clusters,
so no reader could open what they cited.

The fix:
the names say what they test and nothing more,
and the comment cites the ledger's T8 section alone.
The numbered-batch citations elsewhere stay,
since the ledger names each batch:
by the lead's count,
the sixth 9 times,
the ninth 7,
the twelfth 5,
the eighteenth 7 and the twenty-third 5.

Recurrence:
`mistake-prevention.md`,
"References in code and docs".

## B336: characters a reader cannot see stood literally in the source

Red in `2339cf5a6`,
fixed in `0b3996534`.

Found on 2026-10-06 (UTC):
M135 had inferred,
without measuring,
that the decoding which wrote agents' literal characters wrote older ones too.
`literal-unseen-characters.unit.test.ts` reads every TypeScript file under `src` code point by code point
and finds each character with the Unicode property `Default_Ignorable_Code_Point` or `White_Space`,
or of the general category `Cc`,
other than space,
tab,
line feed and carriage return,
written as itself rather than as its escape.
Its fixture makes every such character with `String.fromCodePoint`,
so the scan's own file holds none.
On the red tree it found 31 in 9 files:
byte order marks in the `duplicate-bodies` and `nesting-bound` tests;
ideographic spaces in the `parse-document`,
`image-reading-stage-padding` and `trailing-whitespace` tests
and in a set of `translate-address-original.ts`;
no-break and narrow no-break spaces in the `translator-answer-fold` and `corpus-run/emphasis-spans` tests
and in a set of `quote-normalize.ts`;
and a soft hyphen in the `translator-answer-fold` test.
In a test such a character is fixture data no reader of the file can see,
and in production code a member of a set no reader can see.

The fix:
each of the 31 is its `\u` escape,
written at the line and column the scan named by a script that refused any place holding another character.
Every site is a plain string or an untagged template,
the `parse-document` fixture's included,
so each value is the character it was.
The whole suite on `0b3996534`,
run in a side checkout:
2,019 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and 46 source scans passing.

Recurrence:
`mistake-prevention.md`,
"Escapes and the layer that reads them".

## B337: comments wrote backticks with a backslash before them

Red in `2339cf5a6`,
fixed in `0b3996534`.

Found on 2026-10-06 (UTC):
the twelve lines in five older files M137 recorded.
A comment is Markdown to every reader that renders it,
and Markdown keeps a backslash before a backtick inside a code span or a fenced block,
so a fence written with escaped backticks never opens
and a template literal escaped inside an example shows code that does not compile.
`comment-escaped-backticks.unit.test.ts` reads every comment the parser reports
and finds a backtick with a backslash before it,
unless the run of backslashes it ends has a backtick just before it,
which is a code span holding nothing but backslashes.
On the red tree it found 12 lines in 5 files:
the `artifact-probe-read` test 7,
`house-policy.ts` 2,
`refine-prompt.ts` 1,
the `local-program-failure` test 1
and the `text-accumulators` test 1.
The lead's first draft kept a span of one backslash only,
and so found the span of two in a comment of `credential-views.ts`,
the JSON escape of one backslash,
which renders as written (M138).

The fix:
the 12 lines write their backticks bare:
the test's two example fences open and close,
the TSDoc examples of `house-policy.ts` and `refine-prompt.ts` show code that compiles
(the refine prompt's example had escaped its substitution too),
and the `text-accumulators` test's span,
whose code holds backticks of its own,
is delimited by two backticks.

Recurrence:
`mistake-prevention.md`,
"Escapes and the layer that reads them".

## B356: declarations without TSDoc and parameter lines that said what, not why

Changed in `ce2654b2c`.

Found on 2026-10-06 (UTC) by the review of the input-order change's second pass.
Four destructurings and a loop's binding in the combinator scan had no TSDoc,
the example of `memberKeyOf` passed a value of another type than the node it takes,
and `@param` and `@returns` lines in the scan and in `corpus-run/bench-sample.ts` said what rather than why.
They now say why,
the example passes a `TreeNode`,
and the TSDoc of `printSkipLine` names `skippedAt` as the builder of its line.

Recurrence:
`mistake-prevention.md`,
"Lint and edits".

## B357: the prevention doc recorded neither the combinator scan nor the gather round's lost failure

Changed in `ce2654b2c`.

Found on 2026-10-06 (UTC) by the review of the input-order change's second pass.
The agent's report says no section of `mistake-prevention.md` covered concurrent work,
and it wrote a new one,
"Outcomes decided by which call ends first",
where "Output decided by arrival order" already held the family (B253,
B316,
B317).
At the merge the lead folded the new section into the existing one:
the bench draw's and the gather round's facts,
the rule that a combinator call outside the helper is listed with its reason and what holds it,
and the scan that enforces it,
rather than a second section on one rule.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B377: TSDoc and wording defects in the runner observations change

Red in `bb2a8b9ad1e164974e54d2750432a25b267ef0de`,
fixed in `cfac5b981`.

Found on 2026-10-06 (UTC) by the review of the runner observations change,
its eleventh finding.
`countAmong`'s `@returns` named ids for a count;
`@param` and `@returns` lines of `countAmong`,
`classifyCorpusReadFailure`,
`kindOfFailure`,
`corpusReadRefusal`,
`codePointLength` and `roundWasJudged` said what and not why;
a test helper's name opened with an article;
the TSDoc of `MISSING_OBJECT_PHRASES` and of the `other` kind
described a message language `LC_ALL=C` no longer lets through;
and the off-roster note of `noJudgedRoundLine` read "1 of these artifacts name".
By the agent's run on the port build,
the case at one off-roster artifact fails on that note;
the TSDoc has nothing a case can fail.

The fix:
each is corrected,
and the off-roster sentence goes through `wordForCount`.

Recurrence:
`mistake-prevention.md`,
"Lint and edits" and "Counts in printed text".

## B625: header sentences that the code did not make true

Found on 2026-10-15 by the review of n159
(n160,
finding Y6,
with the sentences of Y1,
Y4 and Y7 that the code did not make true).

Outcome written on 2026-10-15.
Fixed by n161;
red `5fe3173cc5`,
fixed in `277145a862`.

Two header sentences were untrue,
and a third and fourth rested on code that B616 and B621 changed:
- The header of `src/consistency-shown-edge.ts` said that the first or last character of the quoted words
  "is a Latin letter or digit".
  The code uses `isSpacedScriptWordCharacter`,
  any script that spaces its words,
  which n159 changed for B598.
  The Greek and Cyrillic rows of the review's probes follow the code.
- The header of `src/consistency-unread-units.ts`,
  and the account of n159,
  said that the inline-code floor refuses a word of code beside a word of text,
  which it does not (B618).
- The same header said that a character reference "is not here",
  which only the filter of `cutsShownWord` made true while B616 stood.
- The TSDoc of `fragmentProseMask` said one flag per unit of the fragment,
  which an out-of-range landing broke (B621).
The review ran the examples of the TSDoc of `transparentUnitsOf`,
`unreadUnitsOf`,
`cutsShownWord`,
`pageDefinitionsOf`,
`withDefinitions`,
`shownViewOf`,
`writtenBack`,
`textNamed`,
`fragmentProseMask` and `isSpacedScriptWordCharacter`,
and they return what their comments say.
It did not read the headers of the other changed files line by line.

Fixed by n161:
the header of `src/consistency-shown-edge.ts` now says "word character of a script that sets its words apart by spaces",
and the header of `src/consistency-unread-units.ts` no longer says
that the inline-code floor refuses a word of code beside a word of text.
Test:
`src/consistency-header-claims.unit.test.ts` (new),
2 red on the build made before any source edit,
and the guard that restores the old sentence fails 2 rows.

## B632: the account paragraphs of five cache versions stated a fact that no read supported

Found on 2026-10-15 by the review of n162 (`s233/review.md`,
finding Z5).

Outcome written on 2026-10-18.
Fixed by n166 in `29d6064708`;
no test pins the sentence.

What the review found:
each account paragraph that n161 wrote for a cache version said "no record has been written at" its number
(23 in `src/consolidate-key.ts`,
8 in `src/lane-contest-key.ts`,
36 in `src/repair-slice-key.ts`,
7 in `src/refine-slice-key.ts`
and 17 in `src/translate-document-contract.ts`),
while the paragraph of n159 beside it said that the check that would show it
(no slice-cache record written after the change,
beside a control that finds records)
"was not made,
since no runs directory is read in this round".
The claim holds only if no run launched at those versions after n159 set them,
and nothing in the round showed that.
If it were false,
a record written at a number n159 set and before the sheets of n161 would be served under the new sheet text.
The review did not read the runs directories and measured nothing.
It checked that the audit of n161 (`s228/cache-audit-n161.txt`)
put both commits of n161 inside the consistency and pairing versions,
which agrees with the key reads
(the consistency key hashes the sheet text,
and the pairing sheets carry no tense rule),
and that is no finding.

Fixed by n166.
The account sentence in each moved version's file now states its evidence and no guess:
no record had been written at that number when the paragraph was written;
the numbers n159 set were first committed in `277145a862` together with the quoted-tense rule,
and the audit of that commit found 0 slice-cache records under the keyless worktree's runs directory
(`s228/cache-audit-n161.txt`,
line 992).
The same round moved six versions for the sheet text of B637:
consolidate 23 to 24,
refine 7 to 8,
slice 36 to 37,
translate slice 17 to 18,
lane contest 8 to 9
and page title 2 to 3.
The consistency version (4) and the pairing version (3) rode,
since the consistency key hashes the sheet text and the pairing sheet carries no tense rule.
Each new paragraph says that a record at the old number may exist,
because the launch of `TianqiChen66625` started at `5f6c8593d7` and ran at the old numbers,
and that this was not measured;
no record can exist at a new number,
since the number is first set there.
The consistency version moved from 4 to 5 in `29d6064708` for the work of B639 to B642.

Still open:
the `cache-account-audit` task refuses a version that is not in any commit,
so it could not run in the round
(exit 6,
"CONSOLIDATE_CACHE_VERSION = 24 ... is not in any commit;
commit it,
then run the audit again").
The sources of this entry hold no result of that audit after the landing.

Recurrence:
`mistake-prevention-stored-files-and-cached-decisions.md`,
"Cached decisions".

## B681: the header of the span agreement named the wrong direction of its evidence

Found on 2026-10-18 (UTC) by the sol review of the production diff of five landed rounds
(`s245/sol-review/answer-gpt61.md`,
finding 8),
and partly reproduced.

Outcome written on 2026-10-19 (UTC).
Fixed in `ae0b9e28e9`,
a comment only,
with no test.

What was wrong:
the header of `consistency/src/consistency-span-agreement.ts`
said that a quote is evidence for every quote that contains it.
The code runs the other way:
a quote is evidence for every quote it contains,
so the longer `wanted to stay` supports the shorter candidate `to stay`,
and the shorter supports no longer one.
The header now says so with that example,
and says that the count is over every usable ballot and not over the quotes of the place.

What did not reproduce:
the second half of the finding,
that the denominator was unstated.
The TSDoc of `containedQuoteWordsOf` already states it as all the usable ballots.

Prevention:
the notes name none for a header whose direction of evidence was written backwards.
