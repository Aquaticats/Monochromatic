# Authored `@mutates` names stop deciding which caller-owned values inherit a callee's effects

`prefer-readonly-parameter-type` propagates a callee's effects to a caller by mapping each
call argument to the caller parameters that argument packages.
Three defects in that mapping each produced a `readonly` offer for a parameter the callee
writes:
 two let a callee's own declarations decide the result,
 and one could not read a
value at all.
All three are fixed.
This document records what was measured,
 what the fixes cost,
 and what would recover the cost.

## What was wrong

The first two share one shape underneath them.
A single destructured object parameter gives every binding it introduces the same effect index.
A callee written as `function f({ run, target }: { run: () => void; target: Row })` has one
parameter,
 so `run` and `target` are both parameter zero,
and any fact recorded about either is recorded about the same index.
Effects are indexed by parameter,
 so the two cannot be told apart after the fact.
`ST9` requires this shape for every function taking more than one input,
which makes the collapse the ordinary case in this repository rather than a corner.

### The contract-name filter

`effect-owned-call-edge.ts` walked a caller's object-literal argument with only the property
names the callee's `@mutates` blocks listed,
 whenever the callee's parameter was a
destructuring pattern.
A caller-owned value sitting in a property the contract omitted contributed no origin,
so a write the callee performed through that property was attributed to nothing.

Measured on `directRestrictedRowEffect` in
`package/test-fixture/oxlint-no-restricted-syntax/src/readonly-result-provenance-invalid.ts`,
which involves no collection lookup and no result provenance:
the summary read `mutated=[]` and the rule emitted
`Parameter "row" should be readonly: property label is writable`
while the callee wrote `row.label`.

Three neighbours isolate the cause.
`fullContractLiteralEffect` names every mutated property in its contract,
`identifierParameterLiteralEffect` calls a callee whose parameter is a plain identifier,
and `directArgumentRestrictedEffect` hands the container over as a direct argument.
All three read `mutated=[0]` with the filter still in place.

The property names were never verified to mean anything.
Inside the callee every destructured binding maps to parameter zero,
so the staleness check resolves `@mutates run` and `@mutates target` to the same index
and finds both satisfied.
The names were load-bearing for caller-side propagation and checked nowhere.

### The invocation subtraction

`effect-fixed-point-propagation.ts` passed the callee's invoked set as indexes to skip while
propagating the callee's mutated set.
The intent was that invoking a caller-supplied callback should not claim the callback value is
mutated,
 since an owned callback's own effects arrive through the callback relation instead.
Because the index covers the whole destructured parameter,
a callee that invoked one property and wrote another carried both facts on index zero,
and the subtraction removed the write.

Measured on `invokedExclusionDirectEffect`,
 whose callee contract names the property it writes,
so the contract-name filter kept it:
the rule emitted `Parameter "row" should be readonly: property label is writable`
with the argument walk of the previous revision fully intact.
This defect predates every provenance change in this package.

The subtraction also made the answer depend on pass order.
Propagating mutation before invocation kept the write;
 propagating invocation first removed it.
A summary index built over one file in one order kept the write while the rule lost it,
which is why the fixture's diagnostic count is what detects this defect and the summary
assertions beside it only state the facts.

### The unread property forms

`parameterIndexes` enumerates the object-literal property forms whose value it can read
directly:
 assignments,
 shorthand and spreads.
An accessor has no such value.
The callee obtains one by reading the property,
 which runs the accessor body in the
caller's scope,
 so a parameter that body returns reached the callee while contributing no
origin at all.

Measured on `accessorPackagedEffect`,
 where `get unnamed() { return row; }` recorded
`mutated=[]` and the rule offered `row` as readonly while the callee wrote `row.label`.
The accessor body is now scanned for named bindings rather than evaluated,
which over-approximates in the direction that withholds an offer instead of making one.

Methods are deliberately excluded.
A callee has to call a method,
 which is the closure-capture category rather than the
packaging one,
 and including them changed exactly one summary across the 53-function
propagation fixture:
 `passedContainerClosureSemanticEffect` gained sound opacity where
the closure handling already records the write and no offer was ever at stake.

## Why the removal, and not a smaller change

Discharging a collection lookup's receiver opacity,
 added in commit `929c7e4e6`,
treated a value sitting in an object literal handed to a call as attributed,
on the grounds that the argument analysis walks such literals.
The contract-name filter is exactly the case where it does not.
Hardening that discharge alone would have left `directRestrictedRowEffect` unsound,
because no lookup is involved there at all.

`direct-effect-summary.ts` already states the governing policy for authored contracts:
they document known effects and never remove an unresolved implementation's opacity.
The filter contradicted it by letting a comment delete a recorded mutation,
and the subtraction contradicted it by letting one recorded fact cancel another.

Widening the walk is not conservative on its own,
 which is the reason both changes landed
together.
The widened argument set also feeds `propagateInvokedCapabilities`,
which adds every origin in it to the caller's invoked set,
and the subtraction then read that enlarged set to suppress a mutation one call further out.
`middleInvokedExclusionEffect` and `outerInvokedExclusionEffect` measure that chain:
with the walk widened and the subtraction still present,
the rule offered `readonly` for the outer parameter.
Removing the subtraction is what makes propagation add-only,
and add-only propagation is what makes the wider walk safe.

## What it costs

Workspace sweeps of `mise run lint:oxlint`,
 each on a clean tree:

- Before either change,
   1451 findings and 35 offers.
- After the discharge alone,
   1437 findings and 36 offers.
- After removing the contract-name filter,
   1834 findings and 25 offers.
- After removing the invocation subtraction as well,
   1832 findings and 23 offers.
- After the accessor scan,
   still 1832 findings and 23 offers.
- After attributing writes through a supplied callable's result,
   1849 findings and 23 offers.
- After aligning edges past an explicit `this` parameter and reading accessor
   shorthands,
   1850 findings and 23 offers.

The accessor scan leaving both counts untouched is a result rather than a null one.
It fixes a defect the fixture measures and no source in this repository currently hits,
and it was the check that mattered for a second reason:
the walk it widens also feeds `foreignArguments`,
whose consumer in `foreign-borrowed-propagation.ts` requires every packaged origin to be
a foreign candidate,
so a wider walk there can withdraw a foreign-borrowed conclusion and a withdrawn one
stops suppressing an offer.
An unchanged offer count is what rules that out.

Every sweep aborts one program with the same upstream panic,
`interface conversion: checker.TypeData is *checker.TypeReference, not *checker.TupleType`
raised inside `typescript-go`'s API while serializing a type response.
It predates all of this work,
 appears in the baseline sweep as well,
and costs the analysis of `package/test-harness/core`.
Because it is identical across every sweep,
 the comparisons above still hold.

The single offer the discharge added is at
`package/rolldown-plugin/import-attributes/src/transform-helpers.ts:170`,
on `collectStaticReplacements`.
It is correct.
That function only reads `source.start`,
 `source.end`,
 `source.value`,
 `code`,
and `attributes.at(-1)?.end`,
 and returns freshly built objects.
Annotating every property it reaches as deeply readonly type-checks,
at the declaration and at every call site.

Comparing the final sweep against the first,
 13 offers were withdrawn and one was added,
and the finding count rose by 381.
Withdrawal is the safe direction in every case,
and this measurement does not establish which of the 13 were unsound
rather than merely no longer provable.
Those withdrawals,
 and the added findings,
are the price of index-level granularity rather than of the removal itself.
Most of the added findings trace to one unresolved call.
`Object.entries` inside `@monochromatic-dev/toml-edit-core` and in
`package/pi-plugin/statusline/src/rate-limit-parse-helpers.ts` is not derived,
so those callees carry unproven reachability on the parameter they package it into,
and every caller-owned value in the argument literal now inherits it.
Under the filter,
 whether a value inherited that opacity depended on whether the callee's
author had listed its property name in a `@mutates` block,
which is not a proof of anything.

Each sweep after the first was compared by offer identity rather than by count,
which is the check that matters here:
one addition and one withdrawal cancel numerically.
Across all of them no offer was added and none withdrawn,
and the only offer new against the session baseline remains the verified-sound one on
`collectStaticReplacements`.

`narrowingPrecisionCostEffect` pins the precision loss as a measured number.
Its callee writes one property and only reads the other,
so only the first parameter is really mutated,
 and the summary now reads `mutated=[0,1]`.
The cost is always a withheld offer and never a wrong one.

## Remaining work

Per-property effect attribution is what recovers the precision.
Effects would be attributed to a parameter together with the property path reached through it,
rather than to the parameter alone,
so a callee invoking `run` and writing `target` records two separable facts,
and a caller's literal maps each property to the fact that belongs to it.
Attribution must survive transitive propagation,
 aliases,
 callbacks and nested patterns,
and anything it cannot attribute has to widen to the whole parameter rather than to nothing.
An authored contract may then be checked against the measured per-property facts,
which is the first point at which `@mutates` naming a destructured property means something
verifiable.

Deriving the `Object` readers is the larger single reduction available before that.
`Object.entries`,
 `Object.keys`,
 `Object.values` and `Object.hasOwn` read own enumerable
properties and build fresh containers.
They reach user code only through accessors,
and `readonly-classifier.ts` already classifies a type carrying accessors as an opaque
capability,
 so the channel argument the collection members use should transfer.
That would return the two live-source cases in
`package/oxlint-plugin/prefer-readonly-parameter-type/src/workspace-source-effect.unit.test.ts`
and the statusline case in `effect-summaries.unit.test.ts` to a clean reading on proof
instead of on an omitted contract name.

## Alternatives that were rejected

Reporting a callee whose body mutates a property its contract omits was rejected because a
diagnostic does not stop the offer,
 and identifying the omitted property needs the same
per-property machinery that would remove the need for the contract in propagation at all.

Recording the origins the filter dropped as opacity,
 rather than removing the filter,
was rejected on measurement.
It reaches the same reports through a longer route,
 since those origins become opaque anyway,
and it would add opacity even where the callee is fully resolved,
which the removal does not.
