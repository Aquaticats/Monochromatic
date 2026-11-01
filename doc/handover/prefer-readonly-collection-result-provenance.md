# Handover: collection result provenance in `prefer-readonly-parameter-types`

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`.

Live document.
Updated as increments land,
 so the last section is the one to read first when picking this up.

Branch:
 `feat/readonly-collection-result-provenance`.
Worktree:
 `/home/user/worktrees/readonly-result-provenance`.
Issues:
 #414 the defect,
 #415 the trust baseline (closed,
 decided),
 #416 the shorthand provenance defect (closed,
 fixed).

## Why this work exists

Issue #414 reported that the rule fires on ordinary array methods over already-readonly parameters and prints
four remediations that fit none of them.
 The finding was intentional under the effect model,
 so the first
investigation recommended a better message.
 One measurement overturned that:
`slices.map(function toRow(slice) { return { chars: slice.targetChars }; })` is opaque although every object in
the result is freshly allocated inside an owned callback and holds one number.
 No rewrite fixes that finding.
The gate reads type shape where the claim is about provenance.

Two decisions followed,
 both accepted by the repository owner:

- `doc/decision/prefer-readonly-result-provenance.md`,
   "Adopted for issue #414":
   result provenance replaces
  the type-shape result gate.
- `doc/decision/prefer-readonly-member-channel-authority.md`,
   "The stated trust baseline":
   standard
  dispatch,
   indexed data properties,
   the standard iterator and default `Symbol.species` are trusted for a
  value typed as a collection view.
   Three of those four were already trusted,
   two of them silently.

## How to reproduce the measurements

Both scripts run against the built entry and write nothing into the tree,
 by supplying their own source text
over an existing fixture path.
 Build first:
 `mise run //package/oxlint-plugin/prefer-readonly-parameter-type:build`.

```js
// summaries for one overlay source, the shape every probe here uses
import {
  buildEffectSummaryIndex,
  NO_EFFECT_SUMMARY,
  openSemanticFile,
} from '<worktree>/package/oxlint-plugin/prefer-readonly-parameter-type/dist/final/node/index.mjs';

const FILE_NAME = '<repo>/package/test-fixture/oxlint-no-restricted-syntax/src/readonly-catalog-free-invalid.ts';
const session = openSemanticFile({ fileName: FILE_NAME, sourceText: SOURCE, hasBOM: false, },);
const index = buildEffectSummaryIndex({ project: session.project, activeSourceFile: session.sourceFile, },);
const summary = index.get(session.nodeAtOffset(SOURCE.indexOf('function name',) + 'function '.length,).parent,);
```

Baseline recorded on 2026-08-06,
 for a parameter typed `readonly Slice[]` whose element type is deeply
readonly:

- opaque:
   `filter`,
   `slice`,
   `entries`,
   `find`,
   and `map` whose callback returns an object.
- clean:
   `for...of`,
   spread,
   indexed read,
   destructuring,
   counter loop,
   `reduce` to a number,
   `map` to a
  number,
   `forEach`,
   `every`,
   and `filter` on `readonly string[]`.

The case that must flip is the fresh-object `map`.
Reaching zero is not the goal and would mean the guarantee had been abandoned.

## Plan

`doc/planning/prefer-readonly-collection-result-provenance.md` carries the eight increments and the six traps
two reviews found in the first draft.
 Read it before changing any gate.
 The short version:
 steps 1 to 4 change
no verdict,
 step 5 is the first verdict change,
 step 7 withdraws the type-shape gate last,
 step 8 is the
diagnostic #414 asked for.

## Working rules for this branch

- Commit through the worktree,
   which needs `git cli-git trust` once and the forbidden-strings binary at
  `package/cli/forbidden-strings/target/release/forbidden-strings`,
   gitignored and copied from the main
  worktree.
- `mise run //package/oxlint-plugin/prefer-readonly-parameter-type:test:unit` builds first,
   so it is the one
  command that covers a change.
   `lint:types` and `lint:oxlint` after every TypeScript edit.
- Every authority entry keeps its probe.
   An entry added without a passing identity probe is a defect by both
  authority documents' own standard.
- The two fixture diagnostic counts move together with the fixtures:
   `readonly-result-provenance-invalid.ts`
  is pinned at 13 messages in `prefer-readonly-parameter-type.unit.test.ts` with its offer list enumerated in
  full,
   and `readonly-catalog-free-invalid.ts` at 16.
   A count that drops is a report traded for something:
  check it was traded for an attribution and not for silence.

## Progress

### Landed

- `44b6cf767`,
   issue #416.
   Shorthand property provenance.
   `effect-expression-provenance.ts` resolved a
  shorthand property name with `getSymbolAtLocation`,
   which answers with the property rather than the local it
  reads,
   so a returned `{ row }` recorded no origin while `{ row: row }` recorded one and a caller writing
  through the returned holder kept its read-only offer.
   Fixture pair `packageRowShorthand` and
  `packageRowExplicit` with their writing callers,
   asserted in `effect-summaries.unit.test.ts`.
- `268a81632`.
   The increment plan and the traps.
- `873d444ea`,
   increment 1.
   The observer obligation is separate from the ambient channel.
   `OBSERVER_BEARING_MEMBER_NAMES` and `memberInvokesObserver` in
  `effect-member-channel-authority.ts`,
   consulted by `memberChannelIsVerifiedNarrow`,
   which now refuses an
  observer-bearing member whatever its ambient channel is.
   A new case in
  `effect-member-channel-authority.unit.test.ts` fails if any table entry ever names one,
   which is the guard
  the first draft of this work lacked.
   Verdicts unchanged,
   confirmed against the baseline above.
- `0ce8ede07`,
   increment 3.
   `find` and `findLast` carry the direct receiver-value relation on `Array` and
  `ReadonlyArray`,
   probed for identity against a sentinel with an accepting predicate,
   so the miss path
  cannot pass the comparison vacuously.
   `VERIFIED_RESULT_RELATION_COUNT` and the architecture guard's registry
  both move to 10.
   Verdicts unchanged,
   because both members invoke an observer and the composition from
  increment 1 still withholds their receiver claim.

- `4307d6fde`,
   increment 2,
   first half.
   The shared resolver prunes a successor that cannot carry mutable
  identity,
   so `{ named: row.label }` records no origin while `{ row }`,
   `{ row: row }` and `[row]`
  still do.
   Measured before and after on the same overlay sources.
   `packageCountFresh` in the
  result-provenance fixture pins it,
   with a deeply readonly parameter so it adds no diagnostic to the
  sibling counts.
   Whole suite unchanged,
   and the baseline above is unchanged.

### Deferred, with its reason

Increment 2,
 second half:
 the discriminated answer,
 proven-with-origins against unproven.
 `NO_SLOT_ORIGIN`
today means both "proven to carry nothing" and "could not be resolved",
 and discharging on an empty set
would repeat the defect the effect-model split already caught once,
 where an empty match silently dropped a
real mutation.
 `rows.map(row => unknownClone(row))` is the case that must stay unproven.

Deliberately deferred to increment 6 rather than done with the pruning:
 its only consumer is the observer-return
relation,
 nothing today discharges on a returned set,
 and changing the resolver's return type across every
extractor without a consumer would be a wide edit that proves nothing.
 The pruning half had a measurable
effect on its own and landed on its own.

- `449d3f945`,
   the container relation,
   which is increment 5's first half.
   `RESULT_RELATION_RECEIVER_ELEMENTS`
  for `filter` and `slice` on `Array` and `ReadonlyArray`,
   resolved by `callResultElementReceiver`
  rather than by `callResultReceiver`,
   which keeps meaning direct identity and now refuses any other
  relation explicitly.
   Probed in both halves,
   since either passes for the wrong value:
   the result is not
  the receiver,
   and an element of it is the sentinel.
   `FRESH_CONTAINER_MEMBER_NAMES` drops to five,
   each
  remaining name held back for a reason recorded beside it.
   Counts move to 14 in both places.
   No discharge
  consults the new query,
   so no verdict moves.
- `12ee85fb8`,
   the pin the facet has to move.
   `containerElementWriteEffect`,
   `filteredElementWriteEffect`
  and `containerGrowthEffect` in the result-provenance fixture,
   all three recording no mutation today,
   with
  the receiver opacity of an undischarged member doing the withholding.
   When the facet lands the two element
  writes must become `[0]` and the push must stay empty;
   a change moving all three,
   or neither,
   is the
  failure this pins.
   The fixture's pinned message count moves to 12,
   its offer list is unchanged at three.

- `f958b750b` and `e78aef28b`,
   increment 4.
   The element step is answered before the access layers are
  stripped,
   and only for a container whose relation is verified,
   so an unproven container still
  contributes nothing.
   `containerElementReceiver` follows local hops through declaration initializers,
   never
  through a later assignment,
   which over-attributes rather than under-attributes.
   All four spellings
  answer:
   element access,
   array pattern,
   `for...of`,
   and spread;
   an object pattern keeps asking the
  value question,
   since a container's properties are its own rather than its elements.
   Measured:
   all
  four writes attribute `[0]` and `containerGrowthEffect` stays empty.
   `expressionElementOrigins` lives in
  its own module so `effect-binding-origins.ts` stays inside the line limit,
   and the fixture's pinned
  message count moves to 15.

Increments 4 and 5 swapped order,
 for the reason that deferred the discriminated answer.
 Enumerated rather
than assumed:
 no binding can hold element-only origins today,
 because `callResultReceiver` answers only for
the direct-value relation and every entry in the result-provenance authority is one.
 So the facet would have
had no consumer and no test that fails without it.

The order is now the container relation first,
 exposed but consulted by no discharge,
 then the facet that
consumes it,
 then the discharge that needs both.
 The first two are landed.

- `43e65ed7b`,
   the container discharge,
   and the first verdict change.
   `slice` joins the channel authority
  under `MEMBER_CHANNEL_RECEIVER_INDEX_AND_SPECIES`,
   and a verified container relation licenses the same
  discharge a direct one does on the same escape condition.
   The escape test is asked once,
   before either
  relation,
   because it follows the call's result to whatever holds it and is relation-agnostic.

  Two consumers had to learn the element question first,
   and measurement found both where an audit had
  not.
   An argument that is a fresh container hands the callee everything it holds,
   and
  `unresolvedSink(rows.slice())` reported nothing at all until `parameterIndexes` asked;
   an unresolved call
  on a container receiver reaches the same values,
   and `tree.children.slice().filter(observer)` lost its
  finding entirely until the opaque boundary asked.
   Both were reports traded for silence,
   which is the one
  outcome the discharge may not produce.

  Measured across six escape shapes:
   a local read discharges,
   an element write becomes an attribution,
  and a return,
   a store,
   an argument and an escaping element all keep reporting.
   The species tripwire's
  control moves from `slice` to `concat`,
   since a control has to name a member the authority still
  excludes.
   `readonly-result-provenance-invalid.ts` moves to 13 messages with the first offer this work
  produced,
   `containerGrowthEffect`,
   which copies its parameter and writes only the copy.
   The
  catalog-free fixture stays at 16.

- `84a29cfd1`,
   `178dd6ae7`,
   `c6d4c54c7` and `eaeff893d`,
   the rest of the plan.
   An observer handing its element back
  propagates as receiver opacity;
   `map` and `flatMap` carry the observer-return relation,
   probed
  against a marker the receiver never held;
   the type-shape gate is replaced by `viewResultUnaccounted`,
  which requires a relation and,
   for a container,
   that the result stay inside the callable;
   and a
  collection-only finding gets a message whose every remediation is a measured behaviour of the rule.

  Measured after all four:
   the fresh-object `map` is clean,
   which is what issue #414 turns on;
   a
  locally read `filter` is clean;
   an escaping `filter`,
   a foreign observer,
   an identity `map` and a
  `reduce` accumulator all still report;
   a mutating observer attributes rather than reports.
- `2b03eaaad`,
   the matched measurement.
   302 errors and 206 from this rule before,
   231 and 135 after,
   on
  source `git diff` reports as identical,
   which reproduces this issue's recorded numbers and attributes
  the whole delta to the rule.
   Recorded in `doc/decision/prefer-readonly-result-provenance.md`,
  "Consequences of the provenance replacement,
   measured".
   The after-count read 232 when taken from the
  feature worktree,
   which has no `tsgolint`,
   so its lint could not run the semantic rule on every file;
   231 is
  the figure from the built main worktree and is the one to quote.
- `c6e99c4e8`,
   the merge to `main`,
   dirty worktree and no pull request,
   as instructed.

### Next

Nothing outstanding for issue #414.
 Its scope is implemented,
 measured,
 documented and merged.

Issue #417 is a follow-up this work created rather than anything the issue asked for:
 a fold whose
receiver is a container `filter` built reports,
 in both the chained and the bound spelling.
 Precision
rather than soundness.

The measurement that was owed here has since been run,
 and it retracted what this section used to
say.
 The regression recorded against the first attempt does not reproduce:
 the two fixtures it was
blamed for,
 `iteratedContainerWriteEffect` and `spreadContainerWriteEffect`,
 already report `rows.slice`
on merged `main`,
 and swapping `recordReadonlyViewApplications` to `expressionElementOrigins` leaves every
container fixture byte-identical and the whole unit suite green.
 The issue title was wrong too:
 the bound
form is not clean,
 it reports `kept.reduce` where the chained form reports the whole chain,
 so binding
changes the spelling of the cause and nothing else.

The swap is the fix,
 and the reason is the one the element facet was built for.
 For `kept.reduce(fold, 0)`
no parameter is the value `kept` holds,
 so `rootParameterOrigins` answers empty,
 `recordReadonlyViewApplications`
returns on `receiverOrigins.size === 0` before deriving anything,
 and the call falls to a receiver claim that
cannot answer for a member carrying an observer.
 Parameter 0 is where the receiver's elements came from,
which is the question the observer derivation needs,
 and once it is asked the derivation discharges on its
merits.
 The reduce-escape control,
 `observerAccumulatorEscapeEffect`,
 still reports under the change,
 because
what catches it is the result gate on the reduce result type rather than the receiver resolution.

Superseded,
 and left here because the reasoning that produced it was the error:
 the claim that
for a parameter receiver,
 element
origins and value origins agree,
 so resolving the receiver through `expressionElementOrigins` should have
changed nothing,
 and it changed two of the three container fixtures.
 The first half was right and the second half was
never checked;
 the fixtures already read that way before anything was touched.
 The lesson is the one
`AGENTS.md` already states under `QF1`:
 a verdict change is measurable,
 and a measurable fact gets measured
rather than recalled from what a failing test run seemed to say.

Landed as `ab8dda1f1`,
 merged as `d59f7618b`,
 and issue #417 is closed.
 Workspace matched pair in one
worktree:
 3378 errors and 1722 rule findings before,
 3345 and 1689 after,
 warnings and semantic-failure
counts identical across both runs,
 and comparing findings by parameter and location rather than by message,
no parameter became opaque that was not opaque before.
 Recorded in
`doc/decision/prefer-readonly-result-provenance.md`,
 "The receiver question was the value question,
 and it
should have been the element question".

The full write-up is on issue #417.
 The change is the one-line receiver swap,
 two fixtures in `valid/typescript-sync-adapter.ts`,
 and a test
pinning both spellings clean against two controls that must stay opaque.
 The test fails on the revert with
`expected [ 0 ] to deeply equal []`,
 checked rather than assumed.
 The worktree and branch are removed.

## A second arc followed, on iterator members

Picked up after issue #414 closed,
 chosen by the repository owner from a list of open directions.
 Complete
and merged;
 this section exists so the reasoning survives with the commits.

The decision record's "Remaining work" said iterator members were unproven and named a finding that no longer
existed.
 Measuring first was what made the arc tractable,
 and it changed the target:
 the channel was already
proven,
 and what was unproven was the returned iterator's contents.
 Of 1689 findings,
 62 named an iterator
member and 16 named nothing else,
 14 of them the one idiom `for (const [index, item,] of items.entries())`.

Landed,
 each with its own matched pair and none of them changing the read-only offer count of 33:

- `d5e522ad4`,
   `values` under the existing element relation.
   The identity probe had to learn to drain a
  non-array result.
- `b0e560f90`,
   a defect the previous commit exposed.
   The element relation compared the result's type
  argument at the receiver's position,
   so `Map.values` was inert on arrival.
   The relation now carries its
  own `resultTypeArgumentIndex`.
- `0d88c4d2e`,
   `RESULT_RELATION_RECEIVER_ELEMENTS_PAIRED` for `entries`.
- `2527eee1c` and `c29df4e9b`,
   the escape walk admitting the iterated position and then ascending a spread
  to the literal carrying it.
   The only commits here that moved verdicts.
- `7577f123b`,
   `keys` for `Map` and `ReadonlyMap`.

Result:
 iterator-only findings 0 from 16,
 the rule 1682 from 1689,
 semantic failures steady at 4.

### What to carry forward, which is not the code

Two decisions in this arc were committed and then overturned by measurement,
 and one revert was itself
mistaken.
 The failure was identical each time:
 reading the state before a change and concluding something
about the state after it.
 It is now `AGENTS.md` rule `QAB`,
 and the retractions are kept beside their
replacements in `doc/planning/prefer-readonly-iterator-member-provenance.md` rather than tidied away.

The other lesson is cheaper to apply:
 twice I spent a round of reasoning where one instrumented run
answered the question immediately.
 `receiverClaimAnswerable` and `resultEscapesCallable` both take well to a
temporary `console.error` per gate,
 and the gate that refuses is usually not the one the argument predicts.

### Still open, none of it started

- Attributing a write through an iterated binding is the unlocking step for anything further here,
   with
  `readonly-tuple-exposure-invalid.ts` as the control that must stay green throughout.
- `Set` and `ReadonlySet` have no result-relation entries.
   Nothing needs them,
   and `receiverHolding` builds
  only a `Map` or an array,
   so adding one means probe work for no measured effect.
- `keys` for `Array` and `ReadonlyArray` is deliberately absent:
   indices are not the receiver's elements,
  and the exposure test already answers before any relation is consulted.
