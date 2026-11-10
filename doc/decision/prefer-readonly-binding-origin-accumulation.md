# Accumulate every parameter origin per binding in `prefer-readonly-parameter-types`

Accepted and landed on 2026-07-27,
 after the user authorized the widening.
Scope:
 `package/oxlint-plugin/prefer-readonly-parameter-type`.
Raised while scoping result provenance for collection member calls,
then found to be a live unsound suggestion rather than a modelling enhancement.

## Problem

`bindingOriginBySymbolId` mapped one binding symbol to one parameter index.
A local alias reassigned from a second parameter therefore discarded the first origin,
and every effect recorded through that alias landed on whichever origin was written last.

The consequence was not an under-report.
The rule offered a `readonly` annotation for a parameter the body mutates,
and applying that annotation failed to compile.

### Evidence

Fixture,
 now `package/test-fixture/oxlint-no-restricted-syntax/src/readonly-binding-origin-invalid.ts`:

```ts
export function reassignedAliasEffect(
  first: Labelled[],
  second: Labelled[],
  flag: boolean,
): void {
  let cursor = second;
  if (flag)
    cursor = first;
  cursor.push({ label: 'appended', },);
}
```

Before the fix the rule emitted one diagnostic,
 naming the wrong parameter:

```text
Parameter "second" should be readonly: mutable Array has ReadonlyArray projection.
```

`first` was correctly withheld,
 because `cursor = first` was the last write to `cursor`'s
origin entry,
 so `cursor.push` recorded origin `0`.
`second` was offered because its own origin had been overwritten and never recorded.

Applying that suggestion,
 checked against TypeScript 7.0.2:

```text
applied-suggestion.ts(11,10): error TS2339: Property 'push' does not exist on type 'readonly Labelled[]'.
```

The same file with `second: Labelled[]` type-checks clean,
so the error was introduced by the rule's advice and not pre-existing.

## Decision

The map value became a set of origins,
 accumulating rather than replacing.

- `ParameterOrigins` (`ReadonlySet<number>`) and `NO_PARAMETER_ORIGIN` in
  `effect-summary-model.ts` replace the `number | PARAMETER_INDEX_UNAVAILABLE` shape on
  every origin-resolving path.
  Emptiness carries absence,
   so the sentinel is not joined into a union with a set that
  already distinguishes "no origin" from "some origin".
- `PARAMETER_INDEX_UNAVAILABLE` stays for the propagation paths that genuinely carry one
  callee index at a time,
   which is why `addEffectIndex` and `addEffectIndexes` are separate.
- `expressionOrigin` became `expressionOrigins`,
   and `parameterIndex` became
  `rootParameterOrigins`.
  Both names had become misleading once they returned a set.
- `expressionHasParameterOrigin` exists for the callers that only ask whether a root is
  parameter-derived,
   which keeps the existence test from reading as a chained member access.

### Why convergence improves

`registerBindingOrigin` reports progress as "did the set grow".
Under overwrite it reported `prior !== parameterIndex`,
so the fixture's alias flipped between origins `1` and `0` on every pass and reported
progress each time.
`changed` never settled,
 and `discoverAliasOrigins` terminated only when `pass` exceeded
its candidate-alias bound.
Monotone growth makes the fixed point settle on its own merit,
and the bound is a backstop again.

### Accepted cost, measured

Accumulation is flow-insensitive,
 and `flowInsensitiveAliasEffect` in the fixture pins what
that costs:

```ts
let cursor = shadowed;
cursor = reached;
cursor.push({ label: 'appended', },);
```

Only `reached` can be what the alias holds when the mutation runs,
yet both parameters are credited and `shadowed` loses a read-only offer it deserves.
Measured:
 neither parameter is offered,
 and under overwrite `shadowed` was offered.
So overwrite got this shape right,
 by keeping only the last write,
and got the branching shape wrong.

That trade is deliberate and asymmetric.
Withholding an offer costs a suggestion the author could have taken.
Making one for a mutated parameter emits an annotation that does not compile,
which is the defect this decision repairs.

## Sites that changed

Fifteen files mention `bindingOriginBySymbolId`;
most thread it through as a parameter and needed only the widened type.

- `effect-binding-origins.ts`,
   the accumulation itself,
   the progress signal,
   and
  `registerBindingOrigins`,
   which copied before registering so a self-assignment could
  not have one call iterating the set another call was inserting into.
  That copy stopped being the safety property when result provenance landed:
  `expressionValueOrigins` now always returns a freshly built set or a shared empty
  constant,
   so no caller ever holds the live stored set,
   and the spread remains only
  because `ReadonlySet` has no `reduce`.
- `effect-call-resolution.ts`,
   the root lookup and the object-shorthand lookup.
- `effect-opaque-boundary.ts`,
   one opaque record per receiver origin.
- `effect-call-analysis.ts`,
   one `directInvoked` entry and one callback relation per origin.
- `effect-readonly-view-application.ts`,
   one derivation per origin,
   accumulated before
  anything is recorded so a disagreement leaves the whole call to the opaque boundary
  rather than recording a partial answer.
- `effect-collection-member-effect.ts`,
   the mutated-receiver record.
- `effect-summary-cache.ts`,
   now a deep clone,
   matching `opaqueProvenanceByParameter`.
  A shallow copy would have shared each origin set with the cached summary,
   which is
  what cloning exists to prevent.

## The persistent cache is unaffected

Two separate reasons,
 and both were checked,
 because an external review of this change
raised the cache as its highest finding:
 a stale entry could otherwise keep serving the
pre-fix `directMutated` and therefore the suggestion that does not compile.

`effect-summary-serialization.ts:220` rebuilds the field as `new Map()` when a summary is
deserialized,
 because symbol ids are process-local.
Origins never round-trip,
 so widening the value type is not a schema change and
`EFFECT_CACHE_SCHEMA` did not move.

Stale entries cannot survive the change either.
`analyzerDigest` in `effect-summary-cache-identity.ts` streams the exact implementation
bytes into a SHA-256 digest,
 the published bundle when running from `dist` and every
analyzer source file when running through `/ts`,
 alongside `EFFECT_CACHE_SCHEMA` and the
TypeScript version.
`effect-summary-persistent-cache.ts:131` makes that digest part of the cache address,
so editing `effect-binding-origins.ts` invalidates every entry without a schema bump.
That is why the schema constant is not the invalidation mechanism here and must not be
bumped for behavior changes.

## Enforcement

`credits a reassigned alias with every parameter it can hold` in
`prefer-readonly-parameter-type.unit.test.ts` pins the whole expected message set.

The mutation test:
 reverting the accumulator lookup in `registerBindingOrigin` to a fresh
set,
 so origins overwrite again,
 must fail that assertion.
Measured,
 with overwrite restored and the package rebuilt,
 the fixture emits three messages
where accumulation emits one:

```text
Parameter "values" should be readonly: mutable Array has ReadonlyArray projection.
Parameter "shadowed" should be readonly: mutable Array has ReadonlyArray projection.
Parameter "second" should be readonly: mutable Array has ReadonlyArray projection.
```

The `second` offer is the unsound one this decision removes.
The `shadowed` offer is the precision overwrite happened to have and accumulation gives up.
The `values` offer is the control,
 which survives both branches,
 as it must to be a control.

### Both measurement scopes now run built artifacts

While measuring this fix,
 the fixture harness ran analyzer **source**:
`.oxlintrc.readonly.fixture.json` named
`package/oxlint-plugin/prefer-readonly-parameter-type/src/index.ts` in its `jsPlugins`,
so a source edit took effect with no build,
while the workspace sweep ran a built sidecar
(`package/config/oxlint/dist/final/node/plugin-prefer-readonly-parameter-type.mjs`,
rebuilt by `ensureOxlintConfig` in `mise.no-env.toml:417` at every `lint:oxlint`).
Two scopes,
 two artifacts,
 and rebuilding the plugin package updated only one of them.

The fixture config now names
`../../oxlint-plugin/prefer-readonly-parameter-type/dist/final/node/index.mjs`,
the package's own `.` export,
 so both scopes exercise built output.
This matches what the package's tests already did,
 several of which import
`../dist/final/node/index.mjs` directly,
 and what `test-import(require-eventual-artifact)`
exists to require.

The consequence for anyone measuring this rule:
 a fixture run now reflects the last build,
not the working tree.
Verified two ways,
 because a passing suite proves neither on its own.
Mutating `registerBindingOrigin` without rebuilding left the fixture unaffected,
which is how to know the config resolves to `dist` rather than silently falling back to
source.
Moving one plugin's built entry aside made its suite fail with
`Failed to load JS plugin`,
 which is how to know `dist` is required rather than optional.

Naming built artifacts also means a cold tree has no plugin to load,
so `test:unit` in each of the five oxlint plugin packages now carries
`depends = ["build"]`.
Two of those packages already had test files importing `dist` with no such dependency,
so their cold runs already rested on a prior build;
this closes that gap rather than only the one the fixture change opened.
Verified by deleting every plugin `dist` and running all five suites,
 which build and pass.

`readAliasEffect` is that control.
Every other claim in the case is that some parameter is **not** offered,
which a fixture nothing linted would satisfy equally well,
so one parameter that must be offered is what proves the file reached the rule.
This was not hypothetical:
 the first version of the assertions passed against a fixture
emitting nothing at all.

## Workspace effect, measured both ways

No workspace diagnostic moved.

Both sweeps report 1,405 findings for this rule,
 splitting identically into 35 read-only
offers,
 499 `is used by these calls`,
 and 588 `is used as the object`.
The pre-fix sweep was taken with the overwrite mutation in the tree,
 so the sidecar it
built carried the old behavior,
 which is what makes it a real before-measurement rather
than a rerun.

Matching counts alone would not settle it,
 because accumulation pushes in two directions
at once:
 added mutation records suppress offers,
 while added opacity records create
diagnostics,
 so a net-zero total could hide compensating moves.
The sorted diagnostic sets were therefore diffed and are byte-identical,
 1,405 lines each.

The conclusion is about this repository,
 not about the fix:
 no code here reassigns a local
across two parameters and then mutates through it.
The defect was real and the fixture proves it;
 nothing in the workspace happened to hit it.

## Relationship to result provenance

Modelling result provenance for collection member calls propagates through the same core
and is now unblocked.
The two stayed separable on purpose:
 this was a bug fix in one map's value type,
so its measurement is attributable on its own.
