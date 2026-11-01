# Audit ledger: branches no input reaches, kept as quiet answers

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

Branches no input reaches that answered quietly where a loud refusal belongs,
most met when a coverage census asked for them to be removed.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B168: the persistence guard passed an artifact nothing reviewed, and four cases kept the branch that did it

Red in `8f0d7dd6f`,
fixed in `2e02fbd4e`.

Found on 2026-10-05 (UTC)
by the second review of the trial's added cases (its finding 3).
`assertFinalNaturalnessComplete` (`corpus-run/final-naturalness-completeness.ts`)
opened with a branch for a consolidation that never ran:
it returned without a word where the contest had not run either,
refused a contested body slice as short of the naturalness floor,
and passed contested front matter.
Its syntax map carried an empty arm for a contest that never ran.
No pass builds any of those pairings:
`persistSettledEntry` is the guard's one caller,
`pass-entry.ts` hands it only the result of `settledPageArtifact`,
and `settledEntryArtifact` writes `laneSelection` as `contested` and `consolidation` as `settled`,
literals that `buildSettledTwoLaneArtifact` copies unchanged.
The trial's four cases reached the branch through the cast in `artifactCarrying`,
and one pinned the silent pass:
an artifact whose contest and consolidation never ran,
so with no body slice reviewed,
was accepted,
against a TSDoc saying every absent review other than the two named exemptions fails closed.

The fix:
any pairing other than a contested selection over a settled consolidation throws,
its message opening `unreachable:` and naming both records found and the writer that records them otherwise;
the syntax map reads the contested slices with no second arm.
Narrowing `SettledArtifact` to the one pairing was measured first and set aside
(the fixing agent's measurement):
`buildSettledTwoLaneArtifact` takes the whole unions,
and with its parameters narrowed twenty-eight type errors follow
in five test files and one fixture that build pending artifacts for the readers.
One case replaces the four:
it builds each of the three other pairings through `buildSettledTwoLaneArtifact`,
with no cast,
and asserts the three whole messages.
The front-matter case refuses under `unsafe-baseline`,
the skip a body slice ships with,
so only the front-matter rule can be what refused;
it and the absent-review case assert the class and the whole message.

The same pairing stood in a neighbouring file's fixture.
`declinedArtifact` in `corpus-run/pass-readiness-boundaries.unit.test.ts` built it by cast,
and its case went red under the fix (the fixing agent's measurement);
the fixture now records a settled consolidation whose slice carries no polish,
which the guard refuses with the error the case asserts.

Open to the owner's veto:
the throw in place of a narrowed type.

The same review's other findings changed test files only,
merged in `8f0d7dd6f` and green before the fix and after it:

- `preservation-check.unit.test.ts` asserts the whole verdict for an empty licensed quote,
  where the old assertion held with the guard deleted;
- `corpus-run/probe-relabel-control.unit.test.ts` had two cases that passed under three of four mutants of the gatherer;
  it now holds one case for each of the four rules,
  its header states them as the code has them,
  and its dog fixtures are cat fixtures;
- `corpus-run/canadian-forms.unit.test.ts` refuses a year with its ordinal suffix,
  an input the refusal alone leaves unchanged;
- `coverage-wire.unit.test.ts` refuses two replies under a coverage degree that exists,
  and accepts the reply carrying all three fields;
- `corpus-run/model-catalog-compare.unit.test.ts` asserts the report line for line,
  the whole comparison,
  and both parse refusals whole;
- `grade-sheet-read.unit.test.ts`,
  `model-card-derive.unit.test.ts`,
  `synthetic-quota.unit.test.ts`,
  `synthetic-transport.unit.test.ts` and `resolution-wire.unit.test.ts`
  assert whole values and whole messages where they asserted a fragment or a class,
  and the last feeds a numeric string that the whole-number test alone would pass.

Each rewritten assertion fails under the deletion it names
(the fixing agent's mutant runs,
the gatherer's on rebuilt bundles and the others on type-stripped copies of the source).

Recurrence:
`mistake-prevention.md`,
"Guards a census wants gone".

## B176: the corroboration's two empty-group refusals had lost their words

Fixed in `6d60ce1a2`,
with no red case,
since no input reaches the throw.

Found on 2026-10-05 (UTC),
reading the MiMo trial's production diff once the trial was merged.
`corroborate` and `corroborateByOverlap` (`rendering-audit-corroborate.ts`) each read the first member of a group
behind a `RenderingAuditInvariantError` that named the group and why it cannot be empty,
and the trial's T8 change (`18f0b3a85`) replaced both with `nonNullishOrThrow(members.at(0,),)`,
whose whole message is `Expected non-nullish value, got undefined`.
No claim list reaches either:
`corroborate` opens every group with the claim it is keyed on,
and `corroborateByOverlap` grows every group from its seed.

The fix:
both throws are back with their class and their words,
and each function's TSDoc names the refusal.
No case drives them,
since no input builds an empty group;
`stretchReadingOf` reads both as invariant throws,
where it reads the `nonNullishOrThrow` line as cold (the fixing agent's probe).

Recurrence:
`mistake-prevention.md`,
"Guards a census wants gone".

## B177: the substitute break floor took lists of different lengths without a word

Fixed in `6d60ce1a2`,
with no red case,
since no input reaches the throw.

Found on 2026-10-05 (UTC),
reading the same diff.
`substituteBreakFindings` (`source-only-breaks.ts`) takes each side as two parallel lists,
block shapes and break counts,
and read a count at a block's index with `?? 0`,
which the trial's T8 change (`250da1333`) made `nonNullishOrThrow`.
The proof the trial gave holds for production:
`readSliceSkeleton` reads both lists off one parse's top-level blocks,
and `validateTranslatedSlice` is the one caller.
But the conversion refused only some broken inputs and named none.
On the file as it stood at `7f069adc9` (the fixing agent's run of it),
a break list one entry long on either side returned no finding,
a break list one entry short returned no finding where the uncounted block was of a kind the page holds,
and otherwise threw `Expected non-nullish value, got undefined`.

The fix:
the function refuses each side at its entry where the two lists differ in length,
with a throw whose message opens `unreachable:` and names the side.
No case drives it:
the function is not exported,
and 40,009 seeded texts through `readSliceSkeleton` gave no skeleton whose lists differ
(the fixing agent's measurement).
The 55 test files that reach the function through `validateTranslatedSlice` print no failure.

Open to the owner's veto:
the entry check itself,
where the trial's three reads alone already refuse every input production builds.

The same reading left the trial's other edits to these modules standing,
each on the fixing agent's evidence.
Five string reads in `preservation-tokens.ts`,
one in `retry-stated-wait.ts` and one split read in `rendering-audit-screen.ts`
are bounded by their own string's length:
60,016 texts through each reader threw nothing,
where a control read one past the end threw 7,594 times.
The branch removed from `benchRotation` (`stage-fanout-window.ts`) rests on `ChatMessage.content` being a string,
with no cast in production code.
The two OpenRouter extractions return what the inline code at `28303c42a` returned,
over 280 and 14 inputs.
The package index exports 28 more value names than at `28303c42a` and none fewer,
and no added name is imported by nothing.

Recurrence:
`mistake-prevention.md`,
"Guards a census wants gone":
when a fallback becomes a refusal,
the inputs the fallback covered are refused at the function's entry,
in words.

## B199: an artifact whose contest never ran read as nothing to report

Red in `6f61ba5dd`,
fixed in `72f5be638`.

Found on 2026-10-05 (UTC),
as an item ledger M113 left open.
`finalSelectionFindings` (`corpus-run/final-selection-completeness.ts`) read a lane selection that was not `contested`
as an empty list of contests.
No production input reaches that arm:
`pass-entry-persist.ts` holds the function's one caller,
`settledEntryArtifact` writes the kind as a literal,
and `assertFinalNaturalnessComplete` already refuses the same state out loud (ledger B168).

The fix:
the arm throws,
its message opening `unreachable:` and naming the kind found and the writer that records it as contested.
The case that asserted the empty list asserts the whole message.

Open when this entry was written,
closed in B250 (`df807a8b5`):
the sibling arm of the same function,
for a consolidation that never ran,
has the same shape and was left,
since every fixture default of its test file builds that state.

Recurrence:
`mistake-prevention.md`,
"Guards a census wants gone".

## B200: a parsed list item without offsets dropped its whole list without a word

Fixed in `72f5be638`,
its case in the same commit,
since it calls the reader the fix adds.

Found on 2026-10-05 (UTC),
as an item ledger M113 left open.
`readLists` (`corpus-run/list-spread-restore.ts`) returned no list at all when one item carried no start or end offset,
which would move every later list's position in the slice.
The installed parser sets a position on every node it builds
(`mdast-util-from-markdown` 2.0.3,
`enter` and `exit` in its `lib/index.js`),
and none of 54 list items on the 184 pinned pages,
nor of 31 on 13 invented lists,
lacked one
(the fixing agent's reading and counts).

The fix:
`listItemExtent` reads an item's offsets and throws `unreachable:` for an item with none;
it is exported through the barrel so that its case can hand it an item built without a position.
No input reached the old arm,
so the case had no red run on it:
on the tree of `6f61ba5dd` it failed on the missing export.

Recurrence:
`mistake-prevention.md`,
"Guards a census wants gone".

## B223: the width contest answered `none` for a composite whose text is neither seat's

Red in `81ad0add4`,
fixed in `0b687fbdf`.

Found on 2026-10-05 (UTC)
by the first review of the trial's added cases (its finding 4),
behind a case pinning the answer.
`seatThatWon` (`corpus-run/editor-width-contest.ts`) reads which seat won
from the shipped producer and the shipped text,
and closed with `return 'none'` where a composite's text matched neither seat.
`contest`,
its one caller,
seats two composites,
each carrying its arm's patch as the candidate's value,
and a stage that reports a composite ships one candidate's value
(the agent's reading of `contest` and of `selectChunkPatch` in `editor-ensemble.ts`),
so no run produces that state.
`none` is also the answer where no candidate won,
so the sentinel could not be told from an honest reading.

The fix:
the branch is `throw new Error('unreachable: …')`,
naming the composite whose text is neither seat's and why the slate cannot hold one,
with a `@throws` line.
The case that pinned `none` is replaced by
"REFUSES A COMPOSITE WHOSE SHIPPED TEXT IS NEITHER SEAT'S,
since the two seats are the only composites on the slate and a winner ships one of their patches",
which asserts the class and the whole message
and failed on the lead's red tree.
The whole suite's run on `f7653073b` is recorded in B222.

Open to the owner's veto:
the throw,
where deleting the line and keeping `seatThatWon` total over two seats was the other way.

Recurrence:
`mistake-prevention.md`,
"Guards a census wants gone".

## B224: the refine eligibility boundary kept a verdict for a paragraph no parse emits

Fixed in `0b687fbdf`,
with no red case,
since the state cannot be built without a cast.

Found on 2026-10-05 (UTC)
by the first review of the trial's added cases (its finding 3),
behind a case that reached the arm through an `as unknown as` cast.
`judgeParagraph` (`refine-eligibility.ts`) skipped a paragraph standing outside the body zone,
with the reason `not-body-zone`.
`parseDocument` gives a node the zone `footnote-definition` exactly where its kind is `footnoteDefinition`
(`document-node.ts`,
the zone's one writer by the agent's reading),
and the kind test refuses that kind first,
so the arm answered for a node no parse emits.
The trial's fifteenth T8 cluster had recorded as much and cased the arm all the same,
handing the boundary a document built for it.

The fix:
the arm is `throw new Error('unreachable: …')`,
naming the paragraph outside the body zone and the zone's one source,
with a `@throws` line;
`not-body-zone` leaves `IneligibleReason`,
which only the module and its test named on the red commit's tree (`git grep`);
and the case is deleted.
The whole suite's run on `f7653073b` is recorded in B222.

Recurrence:
`mistake-prevention.md`,
"Guards a census wants gone":
no verdict is kept for a fixture production cannot build.

## B231: fourteen chain tails refusing an unhandled member read to the census as code no test ran

Changed in `94de8258e`,
a refactor with no red case.

Found on 2026-10-06 (UTC)
in the whole-suite coverage census taken at `65bd3a645` (`census-5RzoUC`).
B227 closed fourteen branch chains over a union with a call to a helper,
`refuseUnhandledMember`,
which threw a plain `Error`.
The census counts a cold stretch apart from untested code only where it holds nothing but
`throw new Error('unreachable: …')` or a throw of a class whose name ends in `InvariantError`
(`corpus-run/coverage-invariant-throw.ts`).
A call to a helper is neither,
so each tail was listed as cold code:
fourteen cold stretches over 80 lines,
one in each of the fourteen caller files
(the lead's count over the census file,
by a script kept in the session's scratch).
The prevention doc already held that an invariant is written in a shape the census reads (ledger M115);
the helper's shape was not read against that at B227's merge.

The fix:
the helper is the class `UnhandledMemberInvariantError` (`unhandled-member.ts`),
thrown at the chain's end.
Its message is the helper's,
unchanged,
and its constructor keeps the `never` parameter,
so a widened union still stops the build at the throw site;
the thrown value's class name changes from `Error` to the class's own.
The fourteen `@throws` lines name the class.
The class stays unmarked,
as the helper's `Error` was,
since its discriminant is read off a value the types say cannot exist.
The inventory scan refuses an unmarked class that writes its own sentence without a recorded reason,
failed on that at the merge,
and now holds the reason in its withheld list.
The test builds the class through `Reflect.construct`,
since the constructor takes `never`,
and compares the whole text of each refusal.

The whole suite ran on `94de8258e` in a side checkout:
1,621 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 36 source scans passing.

Not measured:
that the census now counts the fourteen apart.
That is read off the census's rule,
and the recount is owed to the next whole-suite census.

Recurrence:
`mistake-prevention.md`,
"Guards a census wants gone":
an invariant is written in a shape the census reads,
and a helper that throws for its caller is not one.

## B250: the final-selection findings read a consolidation that never ran as nothing to report

Red in `3d4558d9c`,
fixed in `df807a8b5`.

Found on 2026-10-06 (UTC),
closing the item B199 left open.
`finalSelectionFindings` read a consolidation recorded as not run as an empty list,
where `settledEntryArtifact`,
its one writer,
records every consolidation as settled.

The fix:
it throws `unreachable:` naming the kind found and that writer.
No input reaches it:
the writer sets the kind as a literal and `assertFinalNaturalnessComplete` already refuses it.
The case is "REFUSES an artifact whose consolidation has not run,
since settledEntryArtifact records every consolidation it hands persistence
and no polish can stand on a stage that never ran".

Recurrence:
`mistake-prevention.md`,
"Guards a census wants gone".

## B630: a group line named a writer's text for any chosen label the record did not explain

Found on 2026-10-15 by the review of n162 (`s233/review.md`,
finding Z2),
in the line that B628 added.

Outcome written on 2026-10-18.
Red `555bb74aa0`,
fixed in `29d6064708`;
the fix is the round n166.

What a reader of the log would have seen:
two records the code cannot write printed `because=carry:chose-writer-text`.
In the probe (`s233/probes/q2-groupline.ts`)
row A was a carry decision `chosen` whose label matched no candidate of the record,
and row B was a writer candidate chosen on a place that ended `kept-current`.
A third record,
row G,
was a verdict that named no winner beside a place that ended `replaced`,
and printed `because=no-winner:no-quorum` beside `replaced=1`.
Each is a damaged record,
and the line reported a decision the record did not hold.

The cause:
`keptReasonOf` took the first candidate whose label matched,
and with none fell through to `chosen?.kind === 'splice' ? ... : 'carry:chose-writer-text'`.
By the code a writer's text that is chosen replaces every receiving place of its block
(`src/corpus-run/pass-consistency-carry-choose.ts`,
lines 141 to 155 at `277145a862`),
so on a `kept-current` place the state is unreachable,
and the fall-through was a guess where the repository's rule is to throw on an unreachable branch.

Fixed by n166.
`groupLine` declares `@throws ConsistencyInvariantError` and throws,
naming the group and the place,
in three cases:
the carry chose a candidate that the carry did not record,
a writer's text was chosen on a place that ended `kept-current`,
and a group whose bench named no winner has a place that ended `replaced`.
The reason `chose-writer-text` is gone from the line.
Rows E,
F and H of the probe
(unlocated only,
kept-floor only,
untouched under a winner)
print no `because=` clause,
as the header says,
and the counts are then the only account.

Test:
the 3 tests for the throws in `src/corpus-run/pass-consistency-group-line.unit.test.ts`
were red against the build the round started from
("Expected ... to throw,
but it returned"),
and red again with the three throws replaced by returns (`s237/mut-a.log`).
The rule breaches of the round are M231.

Recurrence:
`mistake-prevention.md`,
"Guards a census wants gone".

## B683: four branches of the markup ranges answered with an empty list for a node the parser had not placed

Found on 2026-10-18 (UTC) by the test round `w188`,
which went through the survivors of the guard demonstrations of n168,
n169,
n171 and n173 (B674) looking for a named invariant error behind each unreachable branch,
and found none.

Outcome written on 2026-10-19 (UTC).
Red `74f0593e20`,
fixed in `71e4880707`,
with `5c1d1aef1b` clearing the source-scan findings of these tests and of the ported ones (M268).

What the code did:
four branches of `consistency/src/consistency-markup-ranges.ts` fell back to `[]`,
so a link's resource or a stretch was left out without a word:

- `labelEnd > end` in `linkResourcesOf`,
  a label ending after its link.
- The last child of a label with no position.
- A link with no start or end offset in `linkResourcesOf`.
- A node with no start in the walk of `labelledLinksOf`,
  which runs after `linkResourcesOf`.

The fallback of `stretchesOfNodes` (a node with no `position?.start.offset`) was of the same kind,
and was changed with them.
None was reachable:
the parser places every link and every node it reads (0 of 21,870 probe rows),
and `markupRangesOf` was not exported,
so no test could reach them.

Fixed:
all of them throw `ConsistencyInvariantError`,
the class that the consistency package already throws for unreachable states.
The messages name the node type and the offsets and never page text:

- A node of a stretch kind with no start or end offset (`stretchesOfNodes`,
  reached by `stretchesOfTypes` and `stretchesOfLabelsShowing`):
  "the `<type>` node has no start or no end offset,
  and the parser places every node it reads".
- A link with no start or end offset in `linkResourcesOf`:
  the same sentence,
  from one helper,
  `placeOf`.
- A label that ends in a child with no end offset:
  "the label of a `<type>` node ends in a child with no end offset,
  and the parser places every child of a label".
- A label ending after its link:
  "the label of a `<type>` node ends at 12,
  after the link ends at 8,
  and a label ends inside its link".
- The branch for a node with no start in the walk of `labelledLinksOf` is gone.
  `labelledLinksOf` now takes its start and its resource from `placeOf` and `resourceOfLink`,
  which throw.
  It could not be tested apart from `linkResourcesOf`,
  which runs first in `markupRangesOf` and throws first.

`markupRangesOf` is now exported by `index.ts`,
as the package may export a small helper so that a test of the built artifact reaches it.
The tests build their trees by hand in `consistency-markup-ranges-unplaced.unit.test.ts`.

Test,
red then green:
the tests were committed first.
Four of the five failed with `Expected reading to throw, but it returned`
(no node placed,
no start or end,
an unplaced last child,
a label past the end).
The fifth is the control that a placed link reads its resource,
and it passes at both heads.
After the fix all five pass,
and so does the existing `consistency-markup-ranges.unit.test.ts`.

Corpus check (read-only,
run before the throws and again after):
the pinned corpus `~/one-among-us/data` (git head `a41fc607ea`),
every `page.md` and `page.en.md`,
184 files.
Before the throws,
a mirror walk of the parse (14,547 nodes) counted no node of a range kind without a position,
no link without offsets,
no label ending in an unplaced child,
and no label past the end of its link.
After the throws,
`proseViewOf` over the same 184 texts threw on none of them.
All 184 parsed,
so none was skipped.

Recurrence:
`mistake-prevention-guards-searches-and-claims.md`,
"Guards a census wants gone".

## B824: the decision seat's run-ending branch is reached by a test and not by the router

Status:
kept and documented on 2026-10-22 (UTC) in `9d6095ae7e`,
found by note N1 of the review of the XingZ6018 fixes.

`stage/src/stage-decision-call.ts` rethrows a failure that `endsTheRun` names,
and its TSDoc promised `@throws EveryProviderDryError`.
The decisions route (`provider/src/provider-router-decide.ts`) never raises that class.
When OpenRouter is dry it raises `NoProviderForModelError` ("the decisions endpoint reads dry"),
and only `chooseProvider`,
`routeProviderFor` and `readBudgetsPastHolds` raise the dry error,
all three on chat calls.
`stage-decision-call.unit.test.ts` reaches the branch by throwing the class from a scripted `decide`,
which proves the branch and not that production gets there.
In production a decision seat on a fully dry system is a lost voice (`unreachable`),
the shape the run-ending change meant to remove for chat seats.

The decision:
the branch stays,
because a test reaches it and the pass guard wraps `decide`,
and its TSDoc now says the shipped route raises `NoProviderForModelError` and not the dry error.
Whether a dry decisions route alone ends the run was left open by that review.
The ruling of 2026-10-22 on model-level dryness answers it:
a dry OpenRouter with other providers wet is not run-wide dryness,
so the decision seat is a lost voice,
which is what the router does (B822).

Cache versions:
none.

Recurrence:
`mistake-prevention-sheets-quorums-and-outcome-rules.md`,
"A refusal that ends the run":
a branch written for an error is shown reachable from the router that raises it,
and a test that throws the error itself shows the branch and not the route.

## B825: the re-ask elsewhere catches every failure, and the run-ending refusal cannot reach that catch

Status:
recorded on 2026-10-22 (UTC) as a comment in `provider/src/nudged-reask.ts` (`reaskElsewhereNudged`),
no code change and no test,
found as the fourth catch listed by the review's S2 (B818),
where the reviewer had not traced whether the error could arrive.

The catch logs any failure of the re-ask as "no re-ask elsewhere" and keeps the first answer,
which is defensible because an answer exists.
The trace:
the re-ask is hinted away from the provider that served the first answer (`otherThan`),
so the router serves it through `routedTextElsewhere`,
which reads the budgets once (`secondOpinionsFrom`) and raises only `NoProviderForModelError`.
The one path that raises `EveryProviderDryError` is `routedText`,
and a hinted ask never takes it.
Under the pass guard the shared signal is already aborted when any error reaches the catch,
and the catch rethrows on `request.signal.aborted`.
No input reaches the refusal there,
so no test could,
and none was written;
the comment says so,
so a later reader does not add a check that nothing exercises.

Cache versions:
none.

Recurrence:
`mistake-prevention-sheets-quorums-and-outcome-rules.md`,
"A refusal that ends the run".
