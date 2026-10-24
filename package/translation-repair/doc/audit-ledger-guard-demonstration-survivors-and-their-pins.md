# Audit ledger: survivors of the guard demonstrations and the tests that pin them

Paths in this record are the layout after 2026-10-18,
under `package/translation-repair`.

The mutations of the guards of four rounds that every test passed (B674),
and the test round that closed the reachable ones.
B674 is filed in
[Audit ledger: test cases that cannot fail](audit-ledger-test-cases-that-cannot-fail.md),
with the three survivors that change what the code does.
The unreachable branches the same round met are B683,
filed in
[Audit ledger: branches no input reaches, kept as quiet answers](audit-ledger-unreachable-branches-kept-as-quiet-answers.md).

Written on 2026-10-19 (UTC) from the notes of the test round (`s245/w188-notes.md`).
The hashes here are those on the branch.
The notes give the hashes of the round's own worktree,
which is detached at `95a57f628d`.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B684: the test round that pinned the reachable survivors of the guard demonstrations

Run on 2026-10-18 (UTC),
after the demonstrations of n168,
n169,
n171 and n173 left 64 of 287 mutations passing every test (B674).

Outcome written on 2026-10-19 (UTC).
The round changed tests only,
and no production file.
It landed as 15 commits,
`4df68c685e` to `58b9d98583`,
in the seven packages `consistency`,
`corpus-pass`,
`provider`,
`stage`,
`provider-stream`,
`typography` and `artifact`.

Method:
each survivor was proved the same way.
The test was committed first.
Then the mutation of the demonstration's spec was applied
(or an equivalent by content,
where the file had moved),
the package and any consumer rebuilt,
and the named test run and required to show a `[FAIL]` line naming the new test.
Then the file was restored with `git checkout -- <file>`,
rebuilt,
and `git status --short` checked empty.
The proofs are kept outside the repository,
in six logs of the round's scratch folder.

The commits,
by what each pins:

- `4df68c685e`,
  consistency:
  the whole-number,
  once-counted and twice-shown readings of a label.
- `5a0fc8d58e`,
  consistency:
  the definitions of the final edge test,
  and the apostrophes of a slot.
- `b8b1f2eb01`,
  consistency:
  the label order,
  the word run past links,
  and the hidden stretches of a fold.
- `f6b787eac0` and `3b18ef4de5`,
  consistency:
  the tie-break of equal-length candidates,
  the mark before a match,
  and the longest reason.
  The second replaces the fixture of the first (M268).
- `197f5f3d50`,
  consistency:
  a slot is read in a span alone,
  with no definitions of its own.
- `fe70efbec5`,
  corpus-pass:
  the run-off key,
  stage,
  counts,
  log lines and findings of the carry choice.
- `36a887e41b`,
  corpus-pass:
  the page definitions on the English and block sides,
  and the worked page.
- `39dd48f4c2`,
  provider:
  the kind and cause of a refusal-shaped re-ask failure.
- `41dfc43d1f`,
  stage:
  the bound on the opening of the re-ask reply in the lost-voice line.
- `e6d073dead`,
  provider-stream:
  the counted characters on the cut that a drain throws and a transport masks.
- `4f1548ddd3`,
  typography:
  the whole-number clause of the landing error message.
- `5a0fe85281`,
  artifact:
  the run-off recount checks one at a time,
  and the reader of an unlocated reason.
  Its body says six guards for the nine mutations it pins (M267).
- `ce66f2cb07` and `58b9d98583`:
  type fixes in the new tests of consistency,
  and type fixes with a loop `await` in the run-off and page definitions tests of corpus-pass.
  `lint:types` and `lint` found them after the pins,
  and the affected proofs were run again.

The three survivors that change behaviour:
the decision of the round is that the current behaviour is right in the code and wrong in the mutation,
so the tests pin the code,
and no guard is disputed.
The evidence is the TSDoc and header comments named here,
and the dated ruling of 2026-10-15 in the header of `consistency-label-form.ts`.
The notes searched the package ledger and found no text on the slot apostrophes
and only the fold's own sentences on the final edges,
so the ledger neither supports nor contradicts these three.

- `n169-locate-shown-definitions`:
  "CUTS THE FINAL EDGE BY THE REFERENCES THE PAGE DEFINES" in `consistency-fold.unit.test.ts` (`5a0fc8d58e`).
  The TSDoc of `locateWords` and the fold header say that the word test runs again on the final edges,
  reading the text a reader sees,
  which needs the page definitions.
  The mutation drops them at that call.
- `n171-B639-lf-last-digit`:
  "FINDS A LABEL BY THE WHOLE NUMBER IT ENDS IN" in `consistency-label-form.unit.test.ts` (`4df68c685e`),
  where `12` names `OCCURRENCE 12` and not the label that ends in `1`.
  The file header says that a name may be "only the number the label ends in",
  and `trailingNumberOf` is documented as the ASCII digits a label ends in.
- `n171-slot-apostrophes`:
  "GIVES THE SLOT AT ITS WORDS ALONE" in `consistency-slot-in-span.unit.test.ts` (`5a0fc8d58e`).
  The slot is a part of the located span,
  which already holds the apostrophes of the located words,
  and `locateWords` is called with `apostrophes: 'outside'` for it.

The other pinned survivors,
each shown to fail under its mutation:

- Reading order and joints:
  `n168-B638-nested` (the children of a nested node in the order a reader sees them),
  `n169-G2-runs-on-after` and `n169-G2-runs-back-before` (`cutsWord` with a stand inside a word,
  in `consistency-word-joints.unit.test.ts`),
  and `n169-fl-closing-first-after-last`,
  `n173-fl-first-after-last` and `n173-fl-no-unit`
  (folds built by hand with swapped and missing units,
  in `consistency-folded-links.unit.test.ts`).
- Labels:
  `n171-B639-lf-no-dedupe` and `n171-B639-lf-dedupe-both`
  (one label counted once when a name reaches it by its form and by its number,
  and a label the sheet showed twice counted as one;
  at unit level,
  since production sheets never trigger them).
- Span agreement:
  `n171-B645-mark-before` (words refused when a combining mark stands directly before them),
  `n171-B642-no-tie-break`,
  `n171-B645-utf16-tie-break` and `n171-B642-count-reason-from-shortest`.
- A slot read in a span alone:
  `n173-slot-reader-invents-definitions`.
  The mutation site is `slotInSpan`,
  which the recorded-slot reader of the artifact reaches through `asLocatedBySpan`,
  and the existing artifact test `n173-slot-reader-handed-page` is the reader's own pin.
- The carry choice and its run-off:
  `n171-B639-carry-ask-format`,
  `n171-B641-carry-ask-stage` and `n171-B641-carry-ask-key`,
  and `n171-B641-choose-count-all`,
  `-first-quorum`,
  `-log-quorum`,
  `-log-usable`,
  `-log-ending`,
  `-log-tie`,
  `-own-key`,
  `-record-quorum`,
  `-findings` and `n171-B641-finalists-below-quorum-asks`.
  The log lines are pinned exactly through the resumed-cache technique:
  run `choose` once,
  keep `scene.persisted`,
  edit the stored records by hand,
  and run again with `resumed`.
- Page definitions:
  `n173-al-english-side`,
  `n173-aw-page` and `n173-cc-block-side`.
- The re-ask and the drain:
  `n171-B640-reask-cause` and `n171-B640-reask-kind`,
  `n171-B640-stage-call-raw-unbounded`,
  `n171-B643-drain-passes-counts` and `n171-B643-transport-passes-counts`,
  and `n171-fle-message`.
- The run-off recount of the artifact:
  `n171-B641-recount-run-off-beside-majority`
  (the weak `includes('cat.carries[0].runOff')` assertions now assert the exact refusal text),
  `n171-B641-recount-quorum`,
  `n171-B641-recount-all-labels`,
  and the two checks that shadowed each other,
  `n171-B641-recount-finalists-count-redo` and `-labels-redo`.
  One fixture has finalists longer than the leaders,
  and only the count check refuses it.
  Another has the same count and other labels,
  and only the label check refuses it.
  Each mutation fails alone,
  in separate proofs.
- The reader of a reason:
  `n171-B642-reader-old-reading`,
  `-candidates-only`,
  `-any-span` and `-readable` (`artifact-two-lane-read-consistency-span.ts`).
  The second fixture uses a lone variation selector (U+FE0F) twice,
  which folds to something and yet `rendersAsNothing` is true.

Equivalent survivors,
with the reason the probes of the demonstration gave and no input found to tell the mutant from the code:

- `n168-f-gate`:
  the closing-bracket gate only skips a parse that finds no link in such a text,
  so the output never differs (0 of 21,870 probe rows).
- `n169-W8-apostrophe-before-stand`:
  the backward joint reading and the shown-word test answer the same rows,
  so the step over empty links in `apostropheJoinsAt` is shadowed (dead by shadowing,
  kept).
- `n171-B639-lf-shown-duplicates`:
  the final `Set` in `shownLabelOf` removes only duplicates that the first `Set` of that function already removes,
  so with it standing nothing differs.
  The pair is pinned through `n171-B639-lf-dedupe-both`.
  This describes the code as it stood before B679.
- `n171-B639-lf-isNumber`:
  `byNumber` holds digit strings only,
  so a name that is not all digits never matches in it.
- `n171-B639-lf-no-trailing-digits`:
  the empty number key is looked up only for an all-digit name,
  which cannot be empty.
- `n171-B640-bench-decide-kind`:
  only an asked verdict can hold a `runOff`,
  so the kind test restates the type (a dead guard,
  kept).
- `n171-B641-finalists-kind-test`:
  a counted choice has no `reason` field,
  so the reason test already excludes it (a dead guard,
  kept).
- `n171-B641-tied-majority-kept`:
  a label with a strict majority leaves no second label with the top count,
  so the lone-leader rule already returns none (a dead guard,
  kept).
- `n171-slot-blank`:
  words that fold to nothing are refused by later checks of `slotInSpan` (0 differences over 15 spans and slots).
- `n171-slot-once`:
  a slot standing twice makes the locator answer many,
  the slice becomes the whole span,
  and the all-of-the-span check refuses it.
- `n173-count-page-offset`:
  the moved offset reaches `slotInSpan`,
  whose `locateWords` call passes `apostrophes: 'outside'`,
  and `page.offset` is read only when proven apostrophes are taken in.
  It is equivalent today,
  and becomes live if that call ever switches proven apostrophes on.

Branches that cannot be reached:
four silently fell back to an empty list and none threw a named invariant error,
which the notes call a finding.
They are B683.

Not pinned:

- `n171-gl-number-passed`:
  the group number in the error text of the pass is unreachable through the pass,
  because no test makes the pass throw from the group line,
  and the error class and text are not exported for a unit test.
- `n169-LT-quadratic`:
  a work count (each link taken out rewrites the whole pending run),
  and the thousand-link test pins the result and not the work,
  so no assertion on output can fail.
  It is unpinned by design.

State at the end of the round:
`build`,
`lint:types`,
`lint` and `test:unit` exit 0 for all seven packages with no `[FAIL]` line,
and `git status --short` was empty.
Ported onto the branch of generation 19,
the round had four conflicts,
all of one kind,
each side having added whole tests at the same place of a test file,
and both sides were kept.
The `test:unit` task of the seven packages then had no `[FAIL]` line,
and the `source-scan` task,
which the round had not run,
found two problems in its tests (M266).

Recurrence:
`mistake-prevention-guards-searches-and-claims.md`,
"Guards that cannot fail".

## B861: the producer-list search read a builder in few shapes, and its chained-wrapper case could not fail

Status:
fixed on 2026-10-23 (UTC),
found by the re-check of fix round 10 of the Claude route (its findings on what the search misses).
The design that landed is open to the owner's veto.

The defect:
`claude-schemas.unit.test.ts` keeps the list of answer-schema producers honest.
It searches the family's source (`claude-schema-sources.test-fixture.ts`) for files that write a `json_schema` format,
and for files that use a function that writes one (a builder) or a wrapper that returns a builder's result.
The search read a plain call,
a call that names type arguments,
a builder handed over by reference,
and a wrapper that returns the call after `return` or `=>`.
It missed the spread of a builder's result (the dots before the name read as a property),
a call through a namespace import or under an import alias,
a wrapper that returns an awaited call or a parenthesised call across lines,
a builder written as a function expression,
as the default export,
as a method of an object,
as an argument of a call,
or as an async arrow function with one bare parameter.
Its header said it would name the shapes it misses and named none.
The case for a wrapper of a wrapper passed on the search as it stood,
and the mutant that finds wrappers once and does not repeat passed every case,
while the mutant named for the closure removed every wrapper and so proved only that wrappers are read.
An `rg` of the names of the three shared builders over the family's non-test source on 2026-10-23
finds them only as plain calls,
import and export list entries and documentation,
so no producer was missing from the list.

The fix
(the red cases are `b7ba9306ba` and `9e8c7067f4`,
the fix is `76e07ed906`):
one case per shape,
with imaginary files
(`claude-schema-call-forms.test-fixture.ts`,
`claude-schema-builder-forms.test-fixture.ts`,
`claude-schema-wrapper-forms.test-fixture.ts`),
a search that reads each shape
(`claude-schema-code.test-fixture.ts`,
`claude-schema-text.test-fixture.ts`,
`claude-schema-member.test-fixture.ts`,
`claude-schema-import.test-fixture.ts`;
all four were removed on 2026-10-23 (UTC) by B863,
which reads each file's syntax tree in `claude-schema-tree.test-fixture.ts`,
`claude-schema-facts.test-fixture.ts`,
`claude-schema-declare.test-fixture.ts`,
`claude-schema-module.test-fixture.ts`,
`claude-schema-target.test-fixture.ts`,
`claude-schema-resolve.test-fixture.ts`
and `claude-schema-usage.test-fixture.ts`),
and a header in `claude-schema-code.test-fixture.ts` that lists the shapes it does not read:
a function declared inside another function or a class,
a wrapper that returns the call through a local variable,
a conditional,
an object,
an array
or a promise chain,
an optional call,
a function assigned to a second name,
taken out of an object
or re-exported under another name,
an anonymous default export
or one written `export default build;`,
a dynamic import
or `require`,
a default import through a module of another file name (a barrel),
an object member nested deeper than one level,
an accessor (a getter or a setter),
a member with a computed name,
and a member on the line that opens the object
(B862 added the barrel and the member limits to the list,
which first lacked them).
B863 reads,
of this list,
an optional call,
a function assigned to a second name,
taken out of an object by destructuring
or re-exported under another name,
`export default build;`,
a dynamic import
and `require` of a literal path,
a default import through a barrel,
and a member on the line that opens the object.
It still reads none of a function declared inside another function or a class,
a wrapper that returns the call through a local variable,
a conditional,
an object,
an array
or a promise chain around the returned call
(none of which is a wrapper),
an anonymous default export,
an object member nested deeper than one level,
an accessor,
a member with a computed name,
or a static method of a class.
Wrappers stay limited to calls returned at once.
Reading every function that calls a builder would make the three functions that ask the page consistency questions
wrappers,
and the producer list would demand every caller of each,
though they pass the builder's result to a request and return something else.
Mutating the search to read any call as a return fails three cases,
among them the case over the real sources.

Mutants shown to fail a case:
the repeat of the search for wrappers (`one-level-wrappers`),
the reading of wrappers at all (`wrappers-unread`),
the spread,
the property rule,
the namespace names,
the alias names,
the default import names,
`return await`,
`=> await`,
`=> (`,
`return (`,
the closing up of `( `,
the function expression,
the default export,
the object members,
the call argument,
the async parameter,
the alias of a wrapper,
and the rule that a wrapper returns the call.
A run with the fixtures copied unchanged passes.
The mutation of `=> await ` first passed every case,
which added the two arrow cases of `claude-schema-wrapper-forms.test-fixture.ts`.
The claim that every mutation of the search's reading was shown to fail a case did not hold for nine readings (B862).

Prevention rule:
a search that stands in for a parser has one case per shape it reads,
a header that lists the shapes it does not read,
and every mutation of its reading is shown to fail a case.

Cache versions:
none moved.
The change is to test fixtures.

Recurrence:
`mistake-prevention-guards-searches-and-claims.md`,
"Guards that cannot fail".

## B862: nine mutations of the producer-list search passed every case, and a header sentence on strings had none

Status:
fixed on 2026-10-23 (UTC),
found by the re-check of fix round 11 of the Claude route
(its findings on the survivors of the search and on its header).
The design that landed is open to the owner's veto.

The defect:
B861 said every mutation of the search's reading is shown to fail a case.
Eight did not fail one,
and a ninth showed the header's sentence on strings had no case behind it.
Dropping the opening `=> ` of a wrapper,
the follower `<` of a call,
the opening `export default async function` of a builder,
or the reading of a value that starts with type parameters,
each passed every case of the search,
though the header and the TSDoc of `returnsCallOf` claimed all four.
No case wrapped a builder written as a method of an object or as a default export,
and a search that follows no wrapper of either also passed,
though the header of `claude-schemas.unit.test.ts` claimed a wrapper of any builder form.
The header said a name inside a string is no use.
The rule is a quote directly against the name,
a name inside a longer string (`'call build() here'`) is read as a use,
and so is a name in a comment that follows code on its line.
Dropping either quote rule passed every case,
and so did a search that cut the contents of strings out of the code.
The list of shapes the search does not read lacked a default import through a barrel
and the limits on object members,
and B861's copy of the list lacked three more of the header's
(`export default build;`,
`require`
and the setter).
A search of the family's non-test source for the names of the three shared builders
found no use of any of these shapes,
so no producer was missing from the list.

The fix:
six cases for the readings that had none
(`claude-schema-wrapped-builder-forms.test-fixture.ts`,
`claude-schema-builder-forms.test-fixture.ts`):
a wrapper written as an arrow function with the call as its body,
a wrapper that returns a call naming a type argument,
a wrapper of a method builder,
a wrapper of a default-export builder,
a generic arrow builder
and an `export default async function` builder,
and two cases in `claude-schema-wrapper-forms.test-fixture.ts`:
a name that holds a whole string or starts or ends it is no use,
and a name inside a longer string or in a comment after code is read as a use.
The scan did not change.
The longer string is pinned as the over-read the header states,
rather than fixed by cutting string contents out of the code,
which would be a change in behaviour that has to handle template literals that span lines and escaped quotes.
The headers were meant to say what the cases hold,
but the next re-check ran 37 further mutations of their claims
and 15 passed every case (B863).
The list of shapes not read matches B861's.
Each of the nine mutations,
applied to a scratch copy of the tree,
passed every case before the cases and fails the case named for its shape after them,
and the unmutated copy passes.
Wrappers of the other builder forms (an arrow function,
a function expression)
have no case of their own,
and the header says so.

Prevention rule:
a sentence of a header that names a shape the search reads is a claim,
and each such sentence has a case that a mutation dropping the shape fails,
or it is narrowed to what the cases hold.

Cache versions:
none moved.
The change is to test fixtures.

Recurrence:
`mistake-prevention-guards-searches-and-claims.md`,
"Guards that cannot fail".

## B863: the producer-list search read source text, and four re-checks in a row found shapes it misread

Status:
fixed on 2026-10-23 (UTC),
found by the re-check of fix round 12 of the Claude route
(its findings on header claims with no case,
and its probes of strings,
comments,
destructured imports,
promise chains
and a counting function).
The design that landed is open to the owner's veto.

Superseded in part by B864:
the sentences of this entry that the search follows "a dynamic `import()` or `require` of a literal path"
(it follows an `import()` only under `await`,
a `require`,
and the same loads bound inside a function or read in place),
that the scan keeps three over-reads
(it keeps eight),
and that the installed TypeScript has no `createSourceFile` and no `createProgram`
(true of the root export only).
This entry is otherwise left as it was written.

The defect:
B861 and B862 widened a search over the text of the family's source one shape at a time,
and each re-check found shapes the search read wrongly or claimed to read.
The re-check of B862's fix ran 37 further mutations of the header's claims:
22 failed a case and 15 passed every case.
Its probes found a function that maps a builder over items and returns a count read as a wrapper,
a promise chain or a fallback after a returned call read as a wrapper,
a destructured dynamic `import()` or `require` and an optional call read as uses,
though the header listed them as unread,
and a name against a quote of another kind inside a longer string not read,
though the header said it was.
An `rg` of the names of the three shared builders over the family's non-test source
finds them only as plain calls,
import and export list entries and documentation,
so no producer was missing from the list.

The fix
(the red cases are `ac5808a20b`,
the fix is `2bb6429e28`,
and `9987a0cd48` and `0c8c2a838c` follow it):
the search parses each file and reads the syntax tree,
with the parser that `source-scan` wraps
(oxc through rolldown;
the TypeScript 7.1.0-beta installed here has no `createSourceFile` and no `createProgram`,
so the compiler API the plan named does not exist),
and an own resolver for which file a specifier names and which function a name stands for.
A builder is a declared function whose body holds an object literal with the property `type: 'json_schema'`,
a wrapper is a function that returns the call of a builder,
and a use is a name or a chain of members where a value sits,
followed through imports,
aliases,
namespace imports,
re-exports,
default exports,
destructuring,
and a dynamic `import()` or `require` of a literal path.
A reading of the import graph was measured and rejected:
it reaches 629 files where the search found 33,
so 600 files would each need a named exception,
26 of them barrels.
44 cases state the shapes the re-check named,
and 19 of them failed against the text reading before the rewrite.
The header of `claude-schema-sources.test-fixture.ts` states the search in 22 sentences,
each held by named cases,
and 69 mutants of the scan,
one run each in a scratch copy,
each fail at least one case,
with the unmutated copy passing.
Two further cases came from mutants that survived the first cases:
a function that calls a builder in a statement of its own and returns something else is no wrapper,
and the file that declares a builder and the producer list are not its callers.
One mutant of the object-literal check survived and was equivalent
(an object pattern cannot list a string value),
so the check was removed rather than pinned.
One existing case changed its answer:
a builder's name inside a longer string or in a comment that trails code
listed two files and now lists none.
The scan keeps three over-reads,
each held by a case:
a name the file neither declares nor imports,
a name imported from a specifier that names no file of the sources,
and a parameter that shares the builder's name.
`source-scan`'s own scans flagged the first version of the new fixtures
(nine exports nothing read and one TSDoc example without its required key),
which `9987a0cd48` fixed after the fix commit was made without running them.
The test file takes 1.93 s before and 2.67 s after,
one run each,
with 44 more cases in the second.

Prevention rule:
a scan that stands in for a parser reads the parse tree where the repository has a parser,
a header sentence is a claim,
and each sentence has a case and a mutant that fails it,
listed one against the other.

Cache versions:
none moved.
The change is to test fixtures and one dev dependency.
The search for a moved cache version over the commits of this fix prints nothing:

```sh
# package/translation-repair, over ac5808a20b~1..HEAD
git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' ac5808a20b~1..HEAD -- package/translation-repair
```

The same search over `c265e8df95~1..c265e8df95` prints one line,
so the search can print.

Recurrence:
`mistake-prevention-guards-searches-and-claims.md`,
"Guards that cannot fail".

## B864: the producer-list search missed shapes its header claimed, and nothing named the shapes it leaves unread

Status:
fixed on 2026-10-24 (UTC),
found by the re-check of fix round 13 of the Claude route
(re-check 12:
its surviving mutants of the header's claims,
its probes of loads inside functions,
loads read in place and call methods,
and its list of over-reads).
The design that landed is open to the owner's veto.

The defect:
five re-checks in a row found shapes that the header of `claude-schema-sources.test-fixture.ts` claims the search reads
and it does not,
and each round closed the shapes named and the next re-check found more.
Re-check 12 ran 39 mutations of the header's sentences:
2 changed nothing on any input,
17 of the 37 effective ones failed a case,
and 20 passed every case.
Its probes found that a dynamic `import()` or `require` bound to a name inside a function body,
a member read off a load in place
(`(await import(path)).name(...)`,
`require(path).name(...)`)
and `.call`,
`.apply` and `.bind` read off a builder were not followed,
so a file that reaches a builder that way was not listed,
though B863 and the header said a dynamic `import()` and `require` of a literal path were read.
It also found five over-reads the header did not name:
a top-level class or constant that is no function and has the builder's name,
a local constant of that name,
a `catch` parameter of that name,
a function with a second `return` that holds the call,
and a constant that holds the result of mapping a builder over items.
Cases for the shapes named so far cannot end that,
because a shape nobody wrote a case for outruns the header.
An `rg` of the names of the three shared builders over the family's non-test source
still finds them only as plain calls,
import and export list entries and documentation,
so no producer was missing from the list.

The fix
(the cases for the survivors are `df6934f822`;
the red cases for the parser message are `af02390820` and the fix is `0132780b9c`;
the red cases for the loads and call methods are `0bd5ab4428` and the fix is `a23335df23`;
the red cases for the check are `9324804e42` and the check is `69115bfd87`;
the style pass is `e0afa3df57`,
the header is `aa81a922c9`,
and the cases that the first run of the round's own mutants called for are `7c19efc06d` (red) and `0b43d4f849` (fix),
with the header clause `0c4eecc1ed`):

- A check over the real sources.
  `blindSpotsOf` reads the 1579 non-test sources of the family that the search reads
  and reports,
  by file and line,
  each place that holds a shape the search leaves unread.
  It is a net and not a second search:
  it reports a place whether or not a producer hides there.
  Seven shapes:
  a load whose path is a name,
  a joined path or a template with a substitution;
  a load held un-awaited,
  chained with `.then` or handed to a call;
  a builder stored as the value of an object;
  a builder handed to a call that a constant holds,
  as in `memoize(build)`;
  a builder's result returned through a local;
  a builder called in a namespace,
  an enum initializer or a parameter property;
  and a function that holds a format and that no declared function holds
  (a static method,
  a getter,
  a computed or nested member,
  an anonymous default export).
  Ten cases,
  one scratch fixture file per shape,
  failed against a stub at the red commit
  and pass at the check;
  the case over the real sources expects none and passes,
  so no source holds an unread shape today,
  and a source that gains one fails that case with the file and the line.
  It costs about 0.8 s over the real sources.
  The first mutants of the check itself showed what its cases lacked:
  no case said that a followed load is reported as nothing,
  and when one was added it failed,
  because a property of a destructuring pattern was reported as a builder stored in an object.
  The check now leaves the properties of patterns out,
  and that case,
  one for a file named as no stage schema
  and one for a returned `await` of a local hold the check.
- Shapes now followed:
  a name a declaration inside a function or a block binds to a loaded module or to another name,
  the first binding of a name in the text being kept and the top of the file winning over a function;
  a member read off a load in place;
  and `.call`,
  `.apply` and `.bind` read off a builder,
  or off a name the file neither declares nor imports,
  as the builder,
  while a member named `call` of a declared object stays that member.
  A load counts only when it yields the module itself
  (a `require`,
  or an `import()` under `await`),
  so an `import()` held un-awaited is followed to no name,
  which also corrects a reading in which a pending `import()` bound to a constant was taken as the module.
- Shapes named and not followed:
  the seven of the check.
  Each is a sentence of the header,
  a case that states what the search lists of it,
  and an entry of the check.
- Over-reads:
  the header now names the five of the defect beside the three of B863
  (a name the file neither declares nor imports,
  a name imported from a specifier that names no file of the sources,
  a parameter of the builder's name),
  each with a case in `claude-schema-over-read-forms.test-fixture.ts` or `claude-schema-binding-forms.test-fixture.ts`.
- Cases for the survivors:
  each surviving mutant of re-check 12 has a case that states the answer the search gives,
  including two cases for the cycle guards
  (`export *` between two files,
  `const a = b; const b = a;`).
  The mutant S02b is near-equivalent
  (a type annotation that names the builder's exact name)
  and has no case.
- `parseSource` reports the `message` of each error the parser returns,
  where it printed `[object Object]`.
- The sentence of B863 on the TypeScript API is corrected:
  the root export of `typescript` 7.1.0-beta,
  `./lib/version.cjs`,
  holds `version` and `versionMajorMinor` and nothing else.
  `typescript/ast/factory` exports a function named `createSourceFile`
  (`dist/ast/factory.generated.d.ts:372`),
  but it assembles a node from statements that are already parsed,
  and `typescript/ast/scanner` has `createScanner`,
  which reads tokens.
  `typescript/sync` and `typescript/async` have the client methods `createSourceFile(fileName, sourceText, options)`
  (`dist/api/sync/api.d.ts:144`,
  `dist/api/async/api.d.ts:119`)
  and `createProgram`
  (`dist/api/sync/api.d.ts:236`,
  `dist/api/async/api.d.ts:167`),
  whose declarations describe a source file created and retained "remotely"
  (`dist/api/sync/api.d.ts:141-147`)
  and a program created from the file system
  (`dist/api/sync/api.d.ts:235`),
  so the one method of the package that takes a text is a client call,
  and no subpath exposes a parser that runs in the process that asks.
  Those methods were read in their declarations and not run here.
  The oxc parser through `source-scan` stays the reader:
  it is in process,
  it yields plain ESTree objects,
  and `source-scan` already wraps it.
- Declared before use:
  `buildersIn` and `familyOf` precede their callers,
  and the four resolver functions that call one another keep one call before its declaration,
  with a comment that says so.
  `@param` and `@returns` lines carry no articles,
  and no line of the claude-schema fixtures passes 120 characters.
- The header states 41 sentences,
  each held by named cases and by mutants,
  listed one against the other in the report of the round:
  147 mutants in all,
  of which 145 changed what the code does and 144 fail at least one case.
  The one that does not is S02b,
  which is near-equivalent;
  the other two changed nothing on any input.

The test file takes 2.75 to 3.31 s before and 4.16 to 4.56 s after,
three runs each,
alternating.
The check over the real sources accounts for about 0.8 s of the difference;
the rest is the added cases and the larger reading of each file,
which was not measured on its own.

Prevention rule:
a header that lists shapes as unread is paired with a check over the real sources that fails,
naming the file and line,
when any of them holds one,
so an unread shape can enter the family only with a loud failure;
and a shape cheap to follow is followed rather than named.

Cache versions:
none moved.
The change is to test fixtures and to one scan fixture.
The search for a moved cache version over the commits of this fix prints nothing:

```sh
# package/translation-repair, over df6934f822~1..HEAD
git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' df6934f822~1..HEAD -- package/translation-repair
```

The same search over `c265e8df95~1..c265e8df95` prints one line,
so the search can print.

Recurrence:
`mistake-prevention-guards-searches-and-claims.md`,
"Guards that cannot fail".
