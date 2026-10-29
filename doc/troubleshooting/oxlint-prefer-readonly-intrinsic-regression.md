# `prefer-readonly-parameter-types` reports every ECMAScript intrinsic as opaque, 1,661 workspace findings

Status:
 root cause identified.
 The findings are the intended output of the catalog-free fail-closed architecture,
whose workspace-wide migration was never completed.
 Not an accidental regression,
 and not an upstream bug.

Last verified:
 2026-07-27,
 with `@oxlint/plugins` 1.75.0 and `typescript` 7.0.2.

## Symptom

Any package-scoped lint reports errors from the project-owned rule
`prefer-readonly-parameter-type(prefer-readonly-parameter-types)` on ordinary ECMAScript intrinsic calls.

For `package/toml-edit/core`,
 136 errors,
 every one from this rule.
 Verbatim:

```text
x prefer-readonly-parameter-type(prefer-readonly-parameter-types): The function input named "left" is used as the
object for these method calls: left.every [.../src/path-prefix.ts:101].

A method can change data stored inside its object or in the system that object controls, even when this code never
assigns a new value to the input.

This rule cannot inspect enough of those calls to know what they might change.
```

The same shape appears for `blocks.filter`,
 `path.slice`,
 `Object.entries`,
 `String`,
 and `Error.isError`.

It is not package-specific.
 `package/caught-value/core` is 38 lines whose only calls are `Error.isError` and
`String`,
 and it reports two errors of the same class.

Workspace-wide,
 from `mise run lint:oxlint` at the repository root on 2026-07-27:

```text
Found 3902 warnings and 2329 errors.
Finished in 530.7s on 2670 files with 479 rules using 16 threads.
```

Of those,
 1,661 findings are this rule.
 The next most frequent rule is `no-unsafe-member-access` at 754.

## Root cause

The rule was deliberately made catalog-free and fail-closed on 2026-07-22 and 2026-07-23.
`doc/audit/tech-prefer-readonly-native-effect-analysis-vet-2026-07-22.md:55-57` states the governing constraint:

> The requested architecture must not recover speed by trusting handwritten external-effect catalogs.
> An unresolved effect must be derived,
> contained by a verified isolation boundary,
> or reported as opaque.

`Array.prototype.every`,
 `String`,
 and `Error.isError` have no repository-owned implementation to derive from,
cannot be placed inside a verified isolation boundary at the call site,
and are no longer covered by a catalog.
 They are therefore reported as opaque,
 exactly as designed.

The same commit series recognised this and responded by exempting only one directory.
`package/config/oxlint/src/overrides.ts:255-270`,
 added by `32a06a75b` on 2026-07-23,
 says so in its own words:

```ts
/**
 * The effect rule cannot soundly use its own strict opacity policy to prove
 * ECMAScript collections, TypeScript semantic handles, or Oxlint's host context.
 * Self-application would require precisely the handwritten host authorities the
 * rule forbids. Other rules remain active for its implementation and tests.
 */
const readonlyEffectSelfHostingOverride = {
  files: [
    '**/oxlint-plugin/prefer-readonly-parameter-type/**',
  ],
  rules: {
    'prefer-readonly-parameter-type/prefer-readonly-parameter-types': 'off' as const,
  },
} satisfies OxlintOverride;
```

"Cannot soundly use its own strict opacity policy to prove ECMAScript collections" is the whole finding.
That statement is true of every package in the workspace,
 not only of the rule's own implementation.
The exemption was scoped to `**/oxlint-plugin/prefer-readonly-parameter-type/**` and nothing else,
so the remaining 142 workspace projects inherit the strict policy with no way to satisfy it.

The audit records the architecture decision and the performance gates but does not mention the
workspace-wide finding count,
 so this consequence is currently untracked.

## An earlier reading was wrong

`doc/planning/replace-prefer-readonly-parameter-types.md` records an acceptance gate dated 2026-07-14:

> It reported 3,792 warnings and 665 errors from existing non-readonly workspace findings.
> Its captured output contains zero occurrences of the replacement rule ID [...]

Read alone,
 that looks like proof of a regression,
 because the steady state was zero findings and it is now 1,661.
It is not.
 The gate predates the catalog-free work by eight days.
It certified the *replacement rule migration*,
 before the external-effect catalogs were retired
(`e2cda4e35`,
 2026-07-22) and before demand analysis began failing closed (`4bdbc0478`,
 2026-07-23).
Treating 2026-07-14 as a valid "before" for the current behaviour compares two different architectures.

## Verification

The version skew between the committed lockfile and the installed tree is real but irrelevant:

```text
installed:            @oxlint/plugins 1.75.0
committed lockfile:   @oxlint/plugins 1.74.0   (specifier '>=1.73.0')
uncommitted lockfile: @oxlint/plugins 1.75.0
```

Tested directly,
 in a worktree installed from the committed lockfile:

```bash
git worktree add "${HOME}/temp/agent/oxlint-bisect" HEAD
cd "${HOME}/temp/agent/oxlint-bisect"
pnpm install --no-frozen-lockfile   # resolves @oxlint/plugins 1.74.0
mise run //package/config/oxlint:build
cd package/caught-value/core
node ../../monorepo-task/core-exec/src/oxlint-wrapper.ts --type-aware
```

Result under 1.74.0:
 `Found 0 warnings and 2 errors`.
Result under 1.75.0 in the main checkout:
 `Found 0 warnings and 2 errors`.
The plugin version is not a factor.

Smallest reproduction:
 lint `package/caught-value/core`,
 whose entire source is two intrinsic calls.

## Attempted fix: derive receiver survival from read-only views, abandoned

The "finish the migration" option was built and measured,
 then reverted.
 Recording it so the next attempt
starts from the blocker rather than from the idea.

### The idea

TypeScript declares a read-only view beside each mutable collection and places on it exactly the operations
that survive when the holder may not mutate the value.
 Membership of that view is upstream's own assertion,
so a recognizer could read it off the resolved declaration without any handwritten effect catalog.

Measured facts supporting it:

- Every top receiver-side finding is such a member:
   `map` (164),
   `filter` (70),
   `slice` (46),
   `join` (44),
  `get` (35),
   `has` (27),
   `some` (25),
   `flatMap` (25),
   `every` (25),
   `forEach` (23),
   roughly 576 of the 857
  receiver-side findings.
- The calls resolve to `ReadonlyArray` directly,
   not to `Array`,
   because the parameters are already typed
  `readonly T[]`.
   The derivation would therefore only reward code already shaped the way the rule wants.
- `ReadonlyArray`,
   `ReadonlyMap`,
   `ReadonlySet` and `ReadonlySetLike` are the entire set of `Readonly`-prefixed
  default-library interfaces in TypeScript 7.0.2,
   and none declares a mutator,
   so a prefix test needs no
  enumeration.

With the recognizer wired to suppress receiver opacity only,
 `toml-edit` went from 136 findings to 78,
 and
`caught-value` stayed at 2 as predicted.

### Why it was reverted

The premise is that `Readonly*` membership proves the receiver survives the call.
 It proves less than the
opacity encodes.
 Two claims are involved:

- The member does not structurally mutate the receiver collection.
   Provable from membership.
- The call cannot run user code reachable from the receiver.
   What receiver opacity actually encodes.

The first does not imply the second.
 `join` coerces elements through `String`,
 `toSorted()` without a
comparator runs the default comparator,
 and `toLocaleString` likewise.
 Suppressing opacity for those reverses
the audited removal of static plain-data exemptions,
 caught by
`rejects static plain-data claims without runtime isolation`.

Narrowing to calls where the caller supplies the observing function (an argument whose type is definitely
callable) fixed that test and two others,
 but not the core objection.
 Three audited fixtures in
`package/test-fixture/oxlint-no-restricted-syntax/src/valid/typescript-sync-adapter.ts` pin exactly the
narrowed case as opaque:

```ts
export function arrayCallbackSemanticEffect(
  states: readonly { value: string; }[],
): void {
  states.forEach(function updateState(state,): void {
    state.value = 'changed';
  },);
}
```

`effect-summaries.unit.test.ts` expects `opaque: [0]` for it,
 and the same for
`objectArraySortCallbackEffect`,
 which passes an explicit comparator over deeply readonly elements.
`workspace-source-effect.unit.test.ts` expects `opaque: [0]` for `applyCargoPlan`,
 whose only relevant calls
are `plan.blocks.filter(...)` and `plan.enforcements.reduce(...)`.

So the audited design already answers the question:
 a read-only view member that hands caller-owned element
identities to a callback stays opaque,
 because the callback invocation is itself an unresolved boundary.
`readonly T[]` does not make elements readonly,
 and the analyzer will not assume the callback is harmless.

### What this means for the option

"Finish the migration" is not blocked on a recognizer.
 It is blocked on an effect-model change:
 the summary
carries one opacity flag per parameter,
 conflating structural mutation of the receiver with reachability of
user code from the receiver.
 Separating those two would let `Readonly*` membership discharge the first while
the second keeps propagating,
 which is what the audited tests require.

That is a change to what counts as proof,
 so it belongs in the audit
(`doc/audit/tech-prefer-readonly-native-effect-analysis-vet-2026-07-22.md`),
 not in a drive-by patch.

This diagnosis held.
 The model split was drafted as a decision,
 amended four times as probing contradicted the
draft,
 then implemented.
 The recognizer described here was reinstated unchanged in substance;
 what unblocked it
was the model change,
 exactly as this section predicted.

## What does not work

- Treating the findings as defects in the linted package.
   `caught-value` reports two errors over 38 lines whose
  only calls are `Error.isError` and `String`;
   there is no code change that satisfies the rule there.
- Blaming the `@oxlint/plugins` bump.
   Measured above:
   1.74.0 and 1.75.0 both report two.
- Blaming TypeScript configuration drift.
   `tsconfig.json` and `package/config/typescript/` have not changed since
  the acceptance gate except for the 2026-07-15 directory rename (`ece5b7553`).
- Following the rule's own remediation list.
   It offers four paths:
   include the implementation in the nearest
  `tsconfig.json`,
   pass only primitives or a verified isolated snapshot,
   remove the call,
   or mark the input as a
  foreign host capability with an audited `@mutates` contract.
   For `Array.prototype.every` the first is impossible,
  the third means not using array methods,
   and `AGENTS.md` `JCH` forbids the fourth for absent effects.
- Bisecting by materialising historical plugin source into a current worktree.
   Attempted and abandoned:
  substituting `package/oxlint-plugin/prefer-readonly-parameter-type/src` at an older commit while the rest of the
  tree stays current mixes two architectures and produced misleading results.
   A full historical checkout is the
  only sound bisect,
   and one attempt hit `spawn ENOMEM` on a host whose 15 GiB swap was fully consumed.

## Resolution

The third option was chosen and implemented.
 `doc/decision/prefer-readonly-effect-model-split.md` records the
decision,
 the derivation verified against TypeScript 7.0.2,
 and the measured results.

The model now carries two claims instead of one opacity flag.
 Receiver structure is discharged by membership of
a default-library `Readonly*` interface;
 reachable user code is discharged only by analyzing the
caller-supplied observer,
 related to the receiver through the member's own instantiated signature.
 Summaries
gained an `elementApplications` relation,
 because the existing call-edge propagation skips callees without
summaries and a default-library member never has one.

Measured over 2,694 files afterwards:
 1,364 findings for this rule.
 That is not a matched before-and-after
against the 1,661 in this document's title,
 since concurrent work changed the workspace between the runs;
 the
decision document explains why and records the mechanism verification that does hold on real code.

`caught-value` still reports its two errors.
 Those are argument-side findings on `String` and `Error.isError`,
which can invoke getters,
 proxy traps or `toJSON`,
 so they are the rule being correct rather than the
regression this document describes.

One question this resolution does not settle:
 `readonlyEffectSelfHostingOverride` in
`package/config/oxlint/src/overrides.ts` still exempts only the rule's own directory.
 Whether it should now
narrow further,
 widen,
 or stay is a policy decision under `LN7` and needs approval.

## Options as they stood

`AGENTS.md` `LN7` forbids loosening lint rules without prior approval,
 and all three plausible resolutions were
policy decisions rather than mechanical cleanups.

- Widen the exemption.
   The rationale already written into `readonlyEffectSelfHostingOverride` applies verbatim to
  every package.
   Broadening `files` to the workspace turns the rule off in practice,
   which is sound about the
  current information limit but abandons the guarantee the rule exists to provide.
- Restore a minimal ECMAScript-intrinsic authority.
   Directly contradicts the audited constraint that no handwritten
  external-effect catalog may be trusted,
   so it reopens the decision the audit closed.
- Finish the migration.
   Give the rule a derivation path for default-library declarations,
   so intrinsics resolve
  without a handwritten catalog.
   Largest option,
   and the only one that keeps both the guarantee and the audit.
  This is the one that was taken.

## Upstream filing decision

Nothing to file.
 The 6-constraint check does not apply.

Constraint 1 (is it really upstream's fault?
) fails outright:
 the behaviour originates in the repository-owned
`package/oxlint-plugin/prefer-readonly-parameter-type` and a repository-owned architecture decision recorded in
`doc/audit/tech-prefer-readonly-native-effect-analysis-vet-2026-07-22.md`.
 `@oxlint/plugins` was tested at two
versions and is not implicated.
 There is no upstream claim to make,
 so constraints 2 through 6 are not reached and
no duplicate search was run.

`.out-of-scope/` was checked and holds no oxlint or oxc exemption;
 it did not need to gate anything here.

## Related defect fixed during this investigation

Separate from the intrinsic question,
 and worth distinguishing because it *suppressed* findings rather than
creating them.

`nearestOwnedCallable` walked parent pointers to find a call's enclosing callable,
 stopping only on a self-parented
node.
 A source file reports no parent at all rather than parenting itself,
 despite the non-optional `parent` in
TypeScript's node types,
 so the walk assigned an absent parent to the cursor and the next predicate read `.kind`
off it:

```text
[prefer-readonly-parameter-types] [Program] semantic rule failed: TypeError: Cannot read properties of undefined
(reading 'kind')
    at isFunctionLikeDeclaration (.../typescript@7.0.2/.../ast/is.generated.js:730:23)
    at isEffectCallableDeclaration (.../plugin-prefer-readonly-parameter-type.mjs:3:10951)
    at nearestOwnedCallable (.../plugin-prefer-readonly-parameter-type.mjs:6:35262)
    at completeForeignBorrowedGraph (.../plugin-prefer-readonly-parameter-type.mjs:6:37637)
```

The trigger is a callable in the inbound closure that is itself invoked at module top level,
 such as `await main();`
at the end of `package/toml-edit/core/src/conformance/decode.ts`.
 The walk from that call passes no callable
before reaching the source file.

Two properties made it hard to see:

- It reproduces only on a cold or invalidated effect-summary cache
  (`node_modules/.cache/prefer-readonly-parameter-type`),
   and vanishes on a warm re-run.
- The rule logged `String(error)`,
   which reduces the thrown `TypeError` to its message and drops every frame,
   so
  the crash site could not be located from lint output.

Both are fixed.
 The walk stops on an absent parent (`01f85074a`,
 reshaped in `a2a7e7f5a` to avoid a nullish union),
and the log uses `caughtValueStack` from `@monochromatic-dev/caught-value-core` so frames survive (`5c8a624a9`).
A regression test covers the top-level-invocation shape;
 reverting the guard fails it with the original `TypeError`.

Fixing the crash raised `toml-edit` from 133 to 139 reported errors,
 because the crashed file previously emitted a
single `semanticBridgeUnavailable` diagnostic in place of its real findings.
That diagnostic no longer exists:
 an internal failure now logs a warning rather than
reporting,
 so the same situation today shows the findings missing and a warning naming the
file,
 with no diagnostic standing in for them.
 The count is 136 after the
duplicate-predicate refactors landed.
