# Semantic readonly migration must propagate foreign ownership from boundaries

## Symptom

An early `package/toml-edit/core` migration wrapped many parser AST descendants and callback parameters in
`ForeignBorrowed<T>`.
That reduced diagnostics,
but it repeated an ownership claim at every traversal step and made it difficult to see where foreign ownership entered.
It could also hide a real owned call path into a helper.

Removing those descendant wrappers exposed another problem:
the semantic rule recognized the marker only on the directly annotated parameter.
Property access,
element access,
destructuring,
callbacks,
iteration,
and owned helper calls lost the foreign origin and produced replacement-rule diagnostics again.

## Root cause

`ForeignBorrowed<T>` describes provenance,
not a recursive type projection.
The marker belongs where caller-owned or third-party-owned mutable state enters or is deliberately retained.
Type identity alone cannot answer whether a descendant is still wholly foreign-owned because a helper may receive values
from several call sites.

A sound exemption requires two independent facts:

- the value's reachable mutable origin is covered by a real `ForeignBorrowed` boundary;
- every owned inbound call to the callee supplies foreign-covered mutable state for that parameter.

One owned mutable inbound path must remove the guarantee.
Likewise,
a fresh object containing one foreign field must not make an unrelated owned mutable sibling foreign.
Primitive and soundly readonly siblings do not introduce mutable ownership and can be ignored for this coverage check.

Nested closures add a separate ownership boundary in the analyzer.
Call edges belonging to an active inner closure must not be attributed to the outer callable.

## Verified resolution

The semantic rule now separates exact marker identity from dataflow propagation:

- `foreign-borrowed-identity.ts` recognizes only the project-owned marker package and declaration;
- `foreign-borrowed-classifier.ts` determines whether an expression's reachable mutable state is wholly covered by
  foreign provenance;
- `foreign-borrowed-propagation.ts` computes guaranteed foreign parameters across owned call edges to a greatest fixed
  point;
- `effect-binding-origins.ts` carries origins through properties,
  elements,
  aliases,
  nested destructuring,
  and `for...of` element bindings;
- call-edge facts keep `foreignArguments`,
  `directForeignArguments`,
  and `foreignInbound` separate from mutation summaries.

The propagation rule is deliberately conjunctive.
A callee parameter becomes foreign only when every relevant owned inbound path agrees.
`readonly-foreign-provenance-invalid.ts` supplies one foreign call and one owned call to prove that the helper remains
under ordinary readonly enforcement.

TOML now marks ownership at actual boundaries:

- parser ingress from `safeParse`;
- conformance parser ingress;
- retained clean-origin AST storage;
- exported AST-emission seams that receive a foreign AST root.

Descendant `AST.*` parameters remain ordinary types.
The removed `BorrowedTomlAst` alias must not return.

Workspace packages import the marker through its `/ts` subpath.
A working-tree `rg` scan over TypeScript file extensions found 74 package-root imports and 105 `/ts` imports before
correction;
the same scan now finds 179 `/ts` imports and no marker root import.
Commit `300ccac29` records 58 standalone rewrites,
while corrected imports embedded in the current plugin and TOML changes remain with those changes.
`AGENTS.md` rule `ST3` records the workspace-wide source-subpath requirement.

## Default-library observer callbacks remain a provenance gap

Issue #427 reproduced foreign receiver elements losing ownership when a default-library collection member invokes an
inline observer.
`effect-readonly-view-application.ts` records an `ElementApplication`,
and `propagateElementApplications` transfers mutation and opacity from the observer summary.
The foreign-ownership graph has no corresponding receiver-element edge:
`foreignBorrowedOwnershipSeed` initializes `elementApplications` to an empty list,
and `foreignBorrowedDirectSummary` scans ordinary owned calls only.

The gap affects `map`,
`forEach`,
`filter`,
`find`,
`findLast`,
`every`,
`some`,
`flatMap`,
`reduce`,
and `reduceRight`.
Verified controls distinguish it from a general marker failure:

- explicitly marking the callback formal suppresses the finding,
  but repeats the ownership claim at a descendant and is not the intended migration;
- passing `children[0]` through an ordinary owned helper propagates foreign ownership;
- an inline collection observer remains unproved;
- adding one ordinary owned inbound correctly removes the guarantee.

The required relation is position-aware.
For ordinary observers,
the receiver element reaches one callback parameter.
For folds,
the accumulator,
receiver element,
index,
and collection positions are distinct even when TypeScript instantiates them with the same type.
Broad effect reachability cannot prove which one carries foreign ownership.

The implementation should record a foreign observer application beside `ElementApplication`,
including receiver identity and exact callback position,
then consume it in the same conjunctive inbound proof as ordinary call edges.
No type-name allow-list or member-wide exemption is sound.

## Callback capability distinction

Foreign provenance is independent of callback effects.
Invoking a callback is caller-observable capability use,
but invocation does not prove mutation of the function object or every captured value.

The effect model now keeps:

- invoked-capability indexes for contract verification and propagation;
- referent-mutation indexes for readonly-soundy checks;
- a combined affected-input set for complete `@mutates` contracts.

Pure and throwing owned callbacks are summarized from their own bodies and do not taint captured readonly values.
Unknown and external callbacks remain fail-closed and require sound invocation contracts.

## Verification

Focused fixture coverage includes:

- foreign property and element descendants;
- nested destructuring and aliases;
- arrays packaged in fresh objects;
- callback elements explicitly marked at a boundary;
- owned helper calls;
- synchronous `for...of` bindings;
- a mixed foreign and owned inbound call;
- pure and throwing owned callbacks.

The current plugin commands are:

```sh
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:lint:types
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:lint:oxlint
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:build:js:node
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:test:unit
```

Type lint,
Oxlint,
build,
and the complete unit suite pass for the current implementation.
The first unit run after the callback distinction produced 11 expected invalid-fixture diagnostics while its assertion
still expected 12.
Updating the assertion to 11 restored the suite;
the rerun also passed bridge lifecycle,
intrinsic,
Pi catalogue,
readonly classifier,
declaration publication,
and external-consumer checks.

The TOML package commands are:

```sh
mise run //package/toml-edit/core:lint:types
mise run //package/toml-edit/core:lint:oxlint
```

Type lint passes after the `/ts` import correction.
The package began this continuation with 105 replacement-rule diagnostics.
The 2026-07-13 compaction-boundary run completed with status `1` and exactly 28 replacement-rule errors,
with no additional lint category in that run.
Status `1` is expected while findings remain and does not indicate a semantic-bridge crash.

The remaining TOML findings concern recursive value hooks,
internal mutation propagation,
`path.map` callback uncertainty,
and a fast-check `Arbitrary.chain` audit.
Package completion is not claimed.

## What does not work

### Repeat `ForeignBorrowed` on descendants

This erases the boundary decision,
creates annotation churn,
and can hide an owned inbound path.

### Treat one foreign object field as covering the whole object

An unrelated owned mutable sibling remains caller-owned by the current function's caller and must still be checked.

### Use possible instead of guaranteed provenance

A helper called once with a foreign value and once with an owned value is not globally foreign-borrowed.
The propagation meet requires all owned inbound paths to agree.

### Mix provenance into mutation summaries

Ownership answers who controls reachable state.
Mutation summaries answer what execution can affect.
Conflating them can either hide effects or create false mutation claims.

### Treat callback invocation as referent mutation

That makes a pure local callback appear to mutate its function object and readonly captures.
Invocation needs a contract without inventing referent mutation.

### Infer asynchronous iteration from synchronous iteration

The current verified propagation handles synchronous `for...of` bindings.
Async-iterator consumption still needs a distinct effect decision before it can be treated as equivalent.

### Treat all collection callback positions as receiver elements

`reduce` and `reduceRight` pass fold state and receiver elements through separate positions.
A shared instantiated type does not make those values share ownership.

### Reuse effect reachability as ownership proof

Effect propagation intentionally over-approximates what execution may reach.
Foreign ownership needs exact value provenance and a conjunctive inbound guarantee,
so broad reachability would suppress findings on unproved values.

## Upstream filing artifact

### Upstream filing decision

1. **Is it really upstream's fault?**
    No. Oxlint and TypeScript do not define this repository's ownership marker or
   effect semantics.
2. **Can upstream fix it?**
    Not without adopting project-specific provenance and contract rules.
3. **Are they supporting this use case?**
    Oxlint supports JavaScript plugins,
   and TypeScript 7 exposes the semantic graph used by the bridge,
   but neither promises ownership inference.
4. **Would the repo welcome our contribution?**
    No general upstream defect was identified.
5. **Will they likely fix it?**
    Not applicable because the behavior is implemented in the project-owned plugin.
6. **Have we prototyped a minimal fix compatible with their architecture?**
    The identity classifier,
   fixed-point propagation,
   call-edge facts,
   and focused fixtures are the implemented prototype and migration path.

Nothing should be filed upstream.

## Source audit boundary

The investigation followed the marker declaration,
semantic identity and provenance modules,
call-edge summary model,
fixture projects,
and TOML parser and traversal sources.
A repository search found related authoring-identity,
ESTree,
and Pi method-effect documents;
none covers guaranteed descendant provenance across owned call edges.
A root `.out-of-scope` search for readonly,
Oxlint,
TSDoc,
and foreign-ownership topics found no applicable entry.

## Traversal-narrowing era changes (2026-07-15)

The rule contract changed with
[the traversal-narrowing refactor](../planning/prefer-readonly-traversal-narrowing.md):

- traversal of statically plain data is no longer a caller-observable effect,
  so hook-class findings on TOML-like and JSON-like values disappear;
- workspace package calls analyze live repository source;
  the workspace effect catalog and its commit-plus-sha evidence are gone;
- a boundary function's complete `@mutates` contract absorbs the
  uncertainty for its callers;
  transitive callers no longer add their own contracts;
- a `ForeignBorrowed` marker whose underlying type is already deeply
  readonly is reported as inert with a removal instruction.
