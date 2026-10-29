# Derive mutable collection member effects from the paired read-only view

Status:
 accepted,
 implemented and measured.

Decided:
 2026-07-27.

Extends:
 `doc/decision/prefer-readonly-effect-model-split.md`.

Evidence:
 `doc/troubleshooting/oxlint-prefer-readonly-intrinsic-regression.md`.

## Problem

The model split derived receiver effects for `Readonly*` view receivers only.
 A mutable receiver stayed wholly
opaque,
 so `readonly T[]` iteration derived while the identical call on `T[]` did not.
 Measured across the
workspace before this change,
 mutable collection members still caused findings:
 `Map.set` 20 cause-mentions,
`Map.get` 11,
 `Map.delete` 4,
 `Set.has` 3,
 `Set.add` 1.

Worse,
 a genuine mutation and an unanalyzable call were reported identically.
 `values.clear()` is a certain
mutation of a caller-owned `Set`,
 but the rule could only say it did not know what the call did,
 which no
`@mutates` contract could satisfy.

## Decision

Pair each mutable collection interface with `Readonly` plus its own name,
 both proven default-library,
 and
read the structural claim off the difference between them.

TypeScript builds each read-only view by removing exactly the mutators,
 so a member the view also declares
preserves the receiver's structure and a member the view omits restructures it.
 The partition is upstream's,
not authored here.

Verified against TypeScript 7.0.2,
 the difference is exactly:

- `Set`:
   `add`,
   `clear`,
   `delete`.
- `Map`:
   `clear`,
   `delete`,
   `getOrInsert`,
   `getOrInsertComputed`,
   `set`.
- `Array`:
   `copyWithin`,
   `fill`,
   `pop`,
   `push`,
   `reverse`,
   `shift`,
   `sort`,
   `splice`,
   `unshift`.

No view declares a member its mutable interface lacks,
 so the partition is exact in both directions.
 A
collection with no paired view,
 `WeakMap`,
 `WeakSet`,
 a typed array,
 or any host interface,
 is unrecognized
and keeps failing closed.
 `SetLike` does not exist,
 so `ReadonlySetLike` is a view with no mutable
counterpart and is simply never looked up.

### The two claims stay independent

A member can restructure its receiver and run user code over it in the same call.
 `Map.getOrInsertComputed`
inserts and invokes a caller-supplied factory;
 `Array.sort(comparator)` reorders and invokes the comparator.
The structural claim therefore records its mutation,
 and the observer analysis from the model split runs
afterwards regardless.

Only a fully answered call discharges.
 A restructuring member whose reachable user code cannot be derived
reports its mutation and still falls through to the opaque boundary.
 A bare `Array.sort()` reorders and runs
the default comparator's string coercion,
 so it ends up both mutated and opaque rather than accepted.
 `push`
and `clear` likewise,
 since neither supplies an observer.

### Where the view member names come from

Scanning default-library files for `Readonly*` interface declarations,
 memoized per program snapshot.
Interfaces merge across library files,
 and `ReadonlyArray` alone is declared in `lib.es5.d.ts`,
`lib.es2015.core.d.ts`,
 `lib.es2015.iterable.d.ts` and more,
 so every default-library file must contribute:
a partial scan would misread a later-declared member as a mutator.

Filtering candidates by `lib.*.d.ts` basename before fetching cuts the cost from 262 to 116 milliseconds,
measured,
 with identical member counts.
 That is paid once per snapshot.

## What this does not achieve

This does not let the rule lint its own implementation,
 and does not unblock narrowing
`readonlyEffectSelfHostingOverride`.

Measured by enumerating every default-library call in three of the 37 plugin files that import no TypeScript
semantic API:
 `effect-element-application.ts` contains only `Map.get`,
 `ReadonlyMap.get` and `Set.has`;
`effect-callback-relation.ts` the same three;
 `effect-fixed-point-propagation.ts` adds `Array.forEach` and
`Array.reduce`,
 which this change derives,
 but also `ReadonlyMap.get` and `ReadonlyMap.values`,
 which it does
not.

Every one of those files is blocked by the reachable-user-code claim on members that supply no observer.
`ReadonlyMap.get` already had its structural claim discharged before this change and still reported.

Discharging that claim is not derivable from types.
 `Map.prototype.get` runs no user code while
`Array.prototype.slice` consults `Symbol.species`,
 and nothing in either declaration says so:
 it is a fact
about ECMA-262.
 Inferring it from the return type or the parameter types would be a member-behaviour table
recovered from shape rather than read from a declaration,
 which is the handwritten catalog the audit closed
the door on,
 and which the passing `catalog-free effect architecture` test guards against.

So whether the rule can ever lint its own implementation is a policy question,
 not an engineering follow-up.
The options are to accept that it cannot,
 or to reopen the audited no-catalog constraint.
 Both are decisions
for the repository owner.

That question has since been decided,
 and the answer is recorded in
`doc/decision/prefer-readonly-member-channel-authority.md`:
 the constraint reopened for one probe-enforced table.
It did not unblock self-hosting.
 Remeasured on the same modules,
 `effect-element-application.ts` and
`effect-callback-relation.ts` now report one finding each and `effect-fixed-point-propagation.ts` reports two,
every one a `Map.get` whose value type carries state,
 which that decision keeps opaque deliberately.

One claim in this section did not survive the decision's own verification and is corrected there rather than
here:
 `Map.prototype.get` runs no user code,
 but the same was asserted of `Array.prototype.at` and
`Array.prototype.includes` and is false.
 Both invoke an indexed getter on the receiver.
 Read that document's
account of the two channels before reusing anything from this one about which members run nothing.

## Consequences, measured

- `readonly-catalog-free-invalid.ts` moves from 21 diagnostics to 18,
   and its contracts-cannot-discharge
  count from 13 to 11.
   All three losses were diffed against the pre-change build and verified individually:
  two are `map` and `toSorted` over a mutable `children` array with owned,
   pure observers,
   and the third is
  `clearReadonlyOverload`,
   whose declared `@mutates` now agrees with a derived mutation instead of an effect
  the rule could not prove.
   No `audited-call catalogue` message appears,
   which stays asserted.
- `crossFileSemanticEffect` changes from `opaque: [0]` alone to `mutated: [0]` and `opaque: [0]`.
   The helper
  it calls clears a `Set`,
   now a derived mutation.
- `package/caught-value/core` still reports its two argument-side findings,
   unchanged,
   as the control.
- Warm `//package/config/oxlint:lint:oxlint`:
   871 milliseconds over 14 files with no findings,
   against the
  939 milliseconds measured before this change.
   No regression despite the added snapshot scan.
- Workspace:
   1,300 findings for this rule over 2,696 files,
   against 1,364 over 2,694 before the change.
  Unlike the model split's headline,
   this pair is nearly matched,
   two files apart in tree state,
   so the
  reduction of roughly 64 findings is attributable to this work with modest uncertainty.
   Receiver-side
  findings fall from 557 to 516 and argument-side from 765 to 737.

  A first attempt at this measurement was discarded rather than reported:
   it overlapped the rebuilds used to
  verify the fixtures,
   so the run could have loaded a deliberately broken plugin.

## Which members actually run user code

Measured,
 not recalled.
 An earlier pass in this document's own history classified six members wrongly from
memory,
 which is why the table below exists and why the numbers after it were recomputed.

Probed by calling each member on an `Array`,
 `Map` or `Set` subclass whose `Symbol.species` getter records a
hit,
 holding an element whose `toString` and `valueOf` record hits:

- Consult `Symbol.species`,
   so they call a user-chosen constructor:
   `slice`,
   `concat`,
   `flat`,
   `filter`,
   `map`.
- Coerce elements,
   so they call user `toString` or `valueOf`:
   `join`,
   `toString`,
   `toLocaleString`,
   and
  `toSorted` when no comparator is supplied.
- Run nothing:
   `at`,
   `includes`,
   `indexOf`,
   `lastIndexOf`,
   `with`,
   `toReversed`,
   `toSpliced`,
   `keys`,
   `values`,
  `entries`,
   `Map.get`,
   `Map.has`,
   `Set.has`,
   and the `Map` and `Set` iterator members.

The third bullet is wrong and is kept only so the correction has something to point at.
 Do not reuse it.
 The
probe behind it instrumented species and element coercion and nothing else,
 so it could not observe the channel
that actually matters for the `Array` half.
 Re-probed with an own accessor installed at index 0,
 `at`,
`includes`,
 `indexOf`,
 `lastIndexOf`,
 `toReversed`,
 `toSpliced` and `pop` all invoke the getter,
 and a getter
that pushes restructures the receiver during `includes`.
 Only the `Map` and `Set` entries survive as running
nothing.
 The corrected classification,
 as two named channels,
 is in
`doc/decision/prefer-readonly-member-channel-authority.md` and is enforced by a test rather than written down.

The six corrected from recall are `includes`,
 `indexOf`,
 `lastIndexOf`,
 `with`,
 `toReversed` and `toSpliced`.
That correction moved them out of the coercion bullet,
 which was right,
 and into the run-nothing bullet,
 which
was wrong for a different reason nobody was probing for at the time.

Two consequences.
 `map` and `filter` needed the species repair recorded in
`doc/decision/prefer-readonly-effect-model-split.md`.
 And the split between findings the rule reports
correctly and findings it reports only from conservatism is not what a first pass over member names suggested,
so any policy decision resting on that split has to use this table.

## The share that stays unfixable, remeasured

Measured after the species repair,
 using the verified member classification above rather than member names read
from memory.

Workspace:
 1,450 findings for this rule,
 against 1,300 before the repair.
 The repair added 150 by re-reporting
`map`,
 `filter`,
 `slice`,
 `concat` and `flat` over non-primitive element types,
 which is the cost of closing
the species channel and was expected.

The 658 receiver-side findings divide three ways:

- 265 name a collection member that genuinely runs user code,
   so the rule is correct and a code change can
  resolve them.
- 112 name only collection members verified to run nothing.
   This is the conservative residue,
   7.7 percent of
  all findings for this rule,
   across 38 packages.
   Its members are `get` 29,
   `has` 27,
   `at` 13,
   `includes` 13,
  `entries` 12,
   `push` 8,
   `set` 8,
   then a short tail.
- 276 name calls that are not default-library collection members at all:
   host,
   package and project methods
  whose implementations cannot be inspected.
   These are the original catalog-free consequence and are outside
  this decision.

The remaining 746 argument-side and 31 shape findings are unaffected and already classified as the rule being
correct.

## The policy question, restated after remeasurement

Separating the residue into its own diagnostic class,
 which an earlier draft of this work recommended,
 does not
survive the corrected numbers.
 To categorise a finding as belonging to the residue,
 the analyzer must first
know that every member it reached runs no user code,
 and that knowledge is a member list.
 A catalog-free
version of that idea can only key on something coarser,
 such as whether the member takes an observer,
 which
was measured to cover 222 findings of which the majority are the rule being correct.
 So the idea either
mis-targets badly or requires the catalog it was meant to avoid.

That leaves two options.

Accept the residue.
 112 findings across 38 packages stay permanently unactionable,
 and any package wanting a
clean lint has the same claim to an exemption that this plugin already has,
 so the allowlist grows with the
workspace.

Reopen the no-catalog constraint for exactly the verified-inert member set.
 This is narrower than it sounds and
differs from what the audit removed.
 The audit rejected hand-authored effect catalogs,
 whose entries were
unverified assertions.
 The set here was produced by probing a real engine,
 and it can be regenerated and
enforced by a test that fails when a member starts dispatching,
 which no hand-authored table could do.
 It
targets exactly the 112 and hides nothing the rule reports correctly.

A probe is evidence rather than proof:
 absence of dispatch under one set of inputs does not establish absence
for all inputs.
 A test-enforced table is therefore stronger than an authored one and weaker than a derivation,
and choosing it means accepting that middle ground deliberately.
