# Split receiver-structure from reachable-user-code in the readonly effect model

Status:
 accepted,
 implemented and measured.

Decided:
 2026-07-27.

Amends:
 `doc/audit/tech-prefer-readonly-native-effect-analysis-vet-2026-07-22.md`.

Evidence:
 `doc/troubleshooting/oxlint-prefer-readonly-intrinsic-regression.md`.

## Problem

`prefer-readonly-parameter-type/prefer-readonly-parameter-types` reports 1,661 findings across the workspace,
almost all on ordinary ECMAScript intrinsics.
 `package/caught-value/core` is 38 lines whose only calls are
`Error.isError` and `String`,
 and it reports two.

The audit requires that an unresolved effect be derived,
 contained by a verified isolation boundary,
or reported as opaque.
 Intrinsics can be neither derived nor isolated,
 so they are reported as opaque.
The commit that recognised this,
 `32a06a75b`,
 exempted only the rule's own directory,
though its stated reason applies to every package.

A recognizer that reads TypeScript's read-only collection views was built and reverted.
It is not that the recognizer was wrong;
 it is that the summary has nowhere to put its conclusion.

## Root cause in the model

`MutableEffectSummary` carries three dimensions per parameter:
 `mutated`,
 `invoked`,
 `opaque`.
A single `opaque` bit answers two different questions at once:

- Does the callee mutate the structure passed in?
- Can the callee cause user code to run with access to what the parameter reaches?

For `blocks.filter(predicate)` where `blocks: readonly Block[]` the sound answers differ.
The first is no,
 provably.
 The second is yes,
 and the interesting part is exactly what that user code does.
Collapsing both into one bit forces the analyzer to report the parameter as wholly unusable,
which is why idiomatic code that the rule itself pushes toward,
 `readonly T[]` plus `filter`,
produces findings no code change can satisfy.

## Decision

Split the question into two claims.
 A receiver is clean only when both are discharged.

### Claim A, receiver structure

Does the member mutate the collection or object identity passed as the receiver?

Discharged by membership of a default-library `Readonly*` interface,
 proved through the semantic project
with `isSourceFileDefaultLibrary`.
 TypeScript declares a read-only view beside each mutable collection and
places on it exactly the operations that remain available once the holder may not mutate the value.
Membership is therefore upstream's own assertion,
 read off the resolved declaration.

Verified against TypeScript 7.0.2:
 `ReadonlyArray`,
 `ReadonlyMap`,
 `ReadonlySet` and `ReadonlySetLike` are the
whole matching set,
 and none declares a mutator.
 The prefix is upstream's convention,
 so no member list is
authored here and a view added later is covered without an edit.

### Claim B, reachable user code

What user code can this call run,
 and what does it do to values reachable from the receiver?

Discharged only by analysis,
 never by assumption:

- When the caller supplies the observing function,
   that function is owned source with its own summary.
  Its effects propagate to the receiver through a relation derived from the member's own generic signature.

  Do not assume a parameter position,
   and do not read the observer's own annotations.
   Take the member's
  instantiated signature,
   find the callback type it declares for the observer's position,
   and select the
  parameters of that callback type whose types are the receiver or any of its type arguments.

  Two later corrections are folded in here.
   Reading only the first type argument takes the key rather than the
  value for `ReadonlyMap<K, V>`,
   so all of them count.
   Reading the observer's own parameter types works only
  when the observer is contextually typed by the member:
   a by-reference observer,
   `states.map(render)`,
  annotates its parameter independently,
   giving a structurally identical but distinct type that matches
  nothing,
   and discharging on that empty match silently dropped a real mutation.
   The member's signature is
  authoritative because the member,
   not the observer,
   decides what each argument position receives.

  Position varies by member:
  `forEach(callbackfn: (value: T, index: number, array: readonly T[]) => void)` carries the element at
  parameter 0,
   while
  `reduce<U>(callbackfn: (previousValue: U, currentValue: T, currentIndex: number, ...) => U, initialValue: U)`
  carries the accumulator at 0 and the element at 1.
   A comparator passed to `toSorted` carries it at both 0
  and 1.
   Matching by type rather than index handles all of them,
   and misreading `reduce` as element-at-0
  would map element flow onto the accumulator and silently discharge a real effect.

  Match against the receiver type as well as its element type.
   `forEach`'s third callback parameter is
  `array: readonly T[]`,
   the receiver itself,
   so a callback declaring it can reach receiver state without
  ever touching an element parameter.
   Matching elements alone would miss that entirely.

  The relation is read from the declaration's types,
   not asserted.
   Measured in TypeScript 7.0.2,
   the
  instantiated callback parameter types inside the member's signature are reference-identical to the
  receiver's type arguments,
   so identity comparison suffices there.

  A failed match must never by itself discharge.
   Within the member's signature an unmatched position genuinely
  receives no receiver state,
   but a position the signature does not describe at all,
   including an optional
  observer whose type is a union with `undefined` until unwrapped,
   proves nothing and leaves the call opaque.
- When the member observes elements with no caller-supplied function,
   nothing is discharged.
  `join` coerces elements through `String`,
   `toSorted()` without a comparator runs the default comparator,
  and `toLocaleString` likewise.
   These stay opaque.
- Deep readonly-ness of the element type discharges nothing on its own.
   That is the static plain-data
  exemption the catalog-free architecture removed,
   and it stays removed.

### Mechanism for claim B

The existing propagation machinery cannot carry this relation,
 so the split adds one.

`propagateEffects` skips any edge whose callee has no summary
(`effect-fixed-point-propagation.ts`,
 the `calleeSummary === undefined` guard),
 and a default-library member
never has one.
 `CallbackRelation` is also the wrong shape:
 it maps a callee's own parameter to a callback
that callee invokes,
 which `propagateCallbackRelations` then resolves in the caller.
 The relation needed here
runs from the receiver's *elements* to a callback the caller passes directly,
 with no owned callee anywhere.

So the caller's summary gains `elementApplications`,
 recorded when a call satisfies claim A:

```ts
export type ElementApplication = {
  readonly receiverParameterIndex: number;
  readonly callbackKey: string;
  readonly callbackParameterIndexes: readonly number[];
};
```

Its propagation step runs per summary rather than per call edge,
 resolves `callbackKey` against `summaries`,
and writes into the existing dimensions:
 a callback mutating any element parameter marks the receiver
`mutated`,
 and one leaving any element parameter opaque marks the receiver `opaque` and carries the
provenance.
 This mirrors `propagateCallbackRelations` exactly,
 which is the precedent for element mutation
surfacing as `mutated` rather than `opaque`.

Because it writes only to dimensions that already exist,
 `EFFECT_DIMENSION_COUNT` stays at three and the
fixed-point bit budget remains a correct termination bound.
 The field is JSON-safe,
 so serialization is
mechanical;
 a cache written before this change lacks the field,
 fails structural validation,
 and is
recomputed,
 which is the fail-closed outcome.

That rebuild was measured rather than assumed.
 Two consecutive `mise run lint:oxlint` runs over the same 2,694
files took 547.8 seconds and then 196.0 seconds,
 against an 11 MB cache directory written before the change.
The first run paid to recompute every summary the new field invalidated;
 the second reused them.

## Soundness defect, found 2026-07-27 after landing, since repaired

`map`,
 `filter`,
 `slice`,
 `concat` and `flat` are discharged unsoundly by this decision.
 They construct their
result through `ArraySpeciesCreate`,
 which reads `constructor[Symbol.species]` and calls whatever it returns.
That is a second user-code channel this decision does not account for,
 and it receives the receiver's own
elements.

Measured in Node against a subclass whose `Symbol.species` returns a hostile constructor backing the result
with a `Proxy`:

- `map` with an identity callback,
   `filter`,
   `slice`,
   `concat` and `flat` all pass the caller-owned element
  itself to the proxy traps.
- `map` with a callback returning a primitive passes only the primitive.
- `forEach`,
   `every`,
   `reduce`,
   `find`,
   `toReversed` and `with` pass nothing,
   and consult no species getter.

So an owned,
 effect-free observer is not sufficient to discharge a member that constructs a collection result.
Claim B is discharged on the observer alone,
 and the species channel bypasses it entirely.

Species use is not derivable from the declaration:
 `toReversed`,
 `with` and `toSpliced` return new arrays
without consulting species,
 while `slice`,
 `concat`,
 `flat`,
 `map` and `filter` do.
 The return type therefore
cannot distinguish them,
 which also rules out the shape-based inference rejected earlier in this document.

Both repairs below were applied together,
 in
`effect-readonly-view-application.ts`:

- Discharge only members whose return type is not a collection.
   Derivable and conservative,
   keeping `forEach`,
  `every`,
   `some`,
   `reduce`,
   `find` and `findIndex`,
   and losing `map`,
   `filter`,
   `slice`,
   `concat`,
   `flat` and
  `toSorted`.
   This would return most of the findings this decision cleared,
   including `map` at 164 and
  `filter` at 70.
- Additionally discharge a collection-returning member when the observer's own return type is primitive,
   which
  the measurement shows keeps the element out of the channel.
   Derivable from the observer's signature,
   and it
  preserves part of the gain,
   but it never rescues `filter`,
   `slice`,
   `concat` or `flat`,
   whose results carry
  elements regardless of any observer.

The gate reads the call's instantiated result type:
 a result holding any type that can carry mutable state
leaves the call underived.
 A result of `void`,
 `boolean`,
 a primitive,
 or an element union builds no collection
at all and passes.
 This over-restricts `sort`,
 which returns the receiver itself,
 and a `reduce` accumulating
into an array,
 which never constructs;
 neither is distinguishable from a species-consuming member here,
 and
both fail closed.

`objectArraySortCallbackEffect` returns to `opaque: [0]` and pins the gate:
 disabling it makes the fixture
clean again.
 `readonly-catalog-free-invalid.ts` moves from 18 diagnostics to 19,
 recovering a finding the
unsound discharge had removed.

Reproduction,
 run with `node`:

```js
// doc/decision/prefer-readonly-effect-model-split.md, species channel probe
function probe(name, invoke,) {
  const seen = [];
  class Tracked extends Array {
    static get [Symbol.species]() {
      return function Hostile(length,) {
        return new Proxy(new Array(length,), {
          defineProperty(target, key, descriptor,) {
            if (descriptor && ('value' in descriptor)) seen.push(descriptor.value,);
            return Reflect.defineProperty(target, key, descriptor,);
          },
        },);
      };
    }
  }
  const element = { secret: 'caller-owned', };
  const receiver = new Tracked();
  receiver.push(element,);
  invoke(receiver, element,);
  console.log(`${name}: user code saw the element = ${seen.includes(element,)}`,);
}

probe('map identity', (a,) => a.map((x,) => x,),);
probe('map primitive', (a,) => a.map(() => 1,),);
probe('filter', (a,) => a.filter(() => true,),);
probe('forEach', (a,) => a.forEach(() => {},),);
```

Prints `true` for `map identity` and `filter`,
 `false` for `map primitive` and `forEach`.

The gate this section installs is superseded in direction as of 2026-08-06.
It reads the result's type shape,
 which refuses a `map` whose owned callback allocates every object it
returns,
 a case no rewrite resolves and the one issue #414 reports.
`doc/decision/prefer-readonly-result-provenance.md` records the replacement and the measurement.
The semantics stated here stand until that lands.

## What this does not change

The four removals recorded in `doc/planning/replace-prefer-readonly-parameter-types.md` all stand:
handwritten catalogs,
 `@mutates` as a discharge mechanism,
 static plain-data exemptions,
and bodyless host authorities.
 Nothing here trusts a member list or a hand-authored effect.

Fail-closed remains the default.
 The split adds one derivable discharge path and one derivable propagation
path;
 every case that cannot use them reports exactly as it does now.

## Consequences, measured

`mise run lint:oxlint` over 2,694 files reports 1,364 findings for this rule after the change,
 against 1,661
recorded before it.

That pair is not a matched before-and-after and must not be quoted as a 297-finding improvement.
 The workspace
moved between the two runs:
 concurrent work added the `test-import` plugin,
 itself now reporting 697 findings,
along with the test files it lints.
 Establishing a matched baseline would mean reverting the rule in the shared
worktree while another session is working in it,
 which is not worth the disruption for a figure the decision
does not gate on.

The per-member counts fall the same way,
 and for the same reason cannot be read as a clean delta:
 a finding
lists every unresolved cause for one parameter,
 so discharging one cause removes the whole finding only when
every other cause clears too,
 and the remaining causes stop being counted with it.

What is verified directly is the mechanism,
 on real workspace code rather than fixtures.
 At
`package/toml-edit/core/src/emit-value.ts:437` two `map` calls sit in one expression.
 The outer
`body.map(...)`,
 whose receiver is `readonly TOMLKeyValue[]` and resolves to `ReadonlyArray.map`,
 no longer
appears as a cause.
 The inner `kv.key.keys.map(...)`,
 whose receiver is the mutable
`(TOMLBare | TOMLQuoted)[]` and resolves to `Array.map`,
 is the sole cause the diagnostic still names.
 Every
other surviving `map` cause sampled resolves to `Array` on a mutable receiver,
 including
`Readonly<{ ... }>[]`,
 which is a mutable array of readonly objects and correctly discharges nothing.

Argument-side findings are untouched by design.
 `String`,
 `Object.entries`,
 `JSON.stringify` and
`Error.isError` can each invoke getters,
 proxy traps or `toJSON`,
 so those findings are the rule being
correct.
 `package/caught-value/core` still reports its two,
 unchanged,
 and served as the control.

Reaching zero is not the goal and would indicate the guarantee had been abandoned.

## Acceptance

Every position rule above carries a fixture that fails when that rule alone is dropped,
 verified by mutating
the matcher and observing the expected failure rather than by assuming coverage:
`reduceElementParameterEffect` for element-at-1,
 `forEachWholeArrayEffect` for the receiver position,
`readonlyMapCallbackEffect` for type arguments past the first and for a view other than `ReadonlyArray`,
 and
`referencedObserverEffect` for a by-reference observer.
 All four expect `mutated: [0]`.

The audited fixtures decide correctness,
 and all must keep their current expectations:

- `arrayCallbackSemanticEffect` keeps reporting an effect on parameter 0,
   but the dimension changes from
  `opaque: [0]` to `mutated: [0]`.
   Its callback assigns `state.value`,
   which is a proven mutation rather
  than an unresolved call,
   and `propagateCallbackRelations` already routes exactly this shape to `mutated`.
  The sibling fixture `aliasedCallbackSemanticEffect` is the same mutation reached through an owned callee
  and expects `mutated: [0]` today,
   so the split makes the two agree instead of splitting them by
  coincidence of routing.
   Its `@mutates states` contract holds either way.

  This corrects an error in the first draft of this decision,
   which asserted the fixture would stay
  `opaque: [0]`.
   Under the split,
   keeping it opaque would mean the analyzer had failed to resolve a callback
  it demonstrably resolved.
- `objectArraySortCallbackEffect` goes clean,
   `mutated: []` and `opaque: []`,
   and so does the sibling
  `primitiveArraySortObservationEffect`.

  This corrects a second drafting error,
   which claimed the first would stay `opaque: [0]` because its
  comparator calls `localeCompare`.
   It does,
   but on `left.value`,
   a `string` extracted from the element
  rather than the element itself,
   and the analyzer already treats a primitive receiver as unable to carry
  mutable state.
   Both comparators only read element properties and operate on the resulting primitives,
   so
  the previous opaque marks came from the coarse receiver bit and nothing else.
   The bare-comparator siblings
  are the control and are unchanged:
   `plainArrayDefaultSortObservationEffect`,
  `plainArrayOptionalSortObservationEffect`,
   `hookedArrayDefaultSortOpaqueEffect` and
  `objectArrayUndefinedSortOpaqueEffect` all stay opaque,
   because `toSorted` without a comparator supplies
  no observer to analyze.
- `readonly-static-plain-data-invalid.ts` keeps four diagnostics.
   `join` and `toSorted()` supply no observer.
- `readonly-catalog-free-invalid.ts` keeps 21.

`applyCargoPlan` in `workspace-source-effect.unit.test.ts` keeps `opaque: [0]`,
 and its test needs no edit.
This retracts a third drafting error,
 which expected it to go clean.

The measured provenance names exactly one surviving cause,
`plan.enforcements.reduce [package/file-enforcer/core/src/cargo/apply-plan.ts:154]`.
 Its sibling
`plan.blocks.filter(...)` discharged as predicted.
 The difference is `reduce`'s second argument:
 the seed
`original` carries mutable state,
 and a non-observer argument carrying state leaves the call underived,
because the element-flow relation describes element and receiver positions and says nothing about how a seed
flows into the accumulator.
 Confirmed on an isolated fixture where the same fold discharges with a primitive
seed and stays opaque with an object seed.

Discharging it would require deriving accumulator flow as well,
 which is a larger generalization of the
signature-reading step than this decision authorizes.
 It stays out of scope and remains a known conservative
gap alongside `get`,
 `has`,
 `slice` and `at`.

Performance must not regress.
 The 10-second cold-run figure in
`doc/troubleshooting/oxlint-prefer-readonly-incremental-cache.md` is that incident's open acceptance target,
not a gate this change inherits:
 the same document measures the cold run at 62.9 seconds and the warm run at
1.0 seconds.
 Measured here before any edit,
 `//package/config/oxlint:lint:oxlint` runs warm in 923
milliseconds over 14 files with no findings,
 and `//package/caught-value/core:lint:oxlint` reports two
errors in 1.2 seconds.
 Those are the numbers this change is held to;
 closing the cold-path gap remains the
other incident's work.

Measured after the change:
 `//package/config/oxlint:lint:oxlint` runs warm in 1.0 seconds over the same 14
files with no findings,
 and `//package/caught-value/core:lint:oxlint` still reports two errors,
 in 739
milliseconds.
 No regression.
