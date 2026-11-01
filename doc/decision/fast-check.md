# fast-check for the test-harness self-test

## Status

Accepted.
 Plan:
 `~/.claude/plans/setup-fast-check-for-packages-module-tes-smooth-minsky.md`.

## Package relocation

On 2026-10-04,
`format-error.ts` and `format-error.property.unit.test.ts` moved from `package/test-harness/core/src/`
to `package/test-harness/core-diagnostic/src/`.
The `fast-check` development dependency moved to that package too.
The decision and historical paths in this record remain unchanged.

## Context

`package/test-harness/core` is the workspace's self-contained test harness.
 Its self-tests under
`src/*.unit.test.ts` exercise the harness with the harness,
 using hand-picked example inputs.
Several internal behaviors have wide input domains where example fixtures miss edge cases,
 most
visibly `package/test-harness/core/src/format-error.ts`:
 cycle detection over `.cause` chains and
`AggregateError.errors`,
 recursion termination,
 and stack-frame filtering.
 The recent revert of
cycle detection to a shared `WeakSet` (commit `b4591c01`) confirms this is the fragile surface.

The goal is to fuzz those surfaces so the harness "behaves in every situation".
 fast-check is for
testing the harness only;
 it is not part of the published API and never reaches downstream
consumers.

## Decision

Add `fast-check` as a `devDependency` of `package/test-harness/core`,
 pinned in the pnpm catalog
(`pnpm-workspace.yaml`) at `>=4.8.0`.
 Use it for property-based self-tests,
 starting with
`package/test-harness/core/src/format-error.property.unit.test.ts`.
 Drive the source functions directly
(`formatErrorDeep`,
 `formatFailure`);
 do not re-export fast-check from `index.ts`.

The integration needs no wrapper:
 `await assert(asyncProperty(arbitrary, predicate), { numRuns })`
inside an `it` `fn` throws a shrunk counterexample on failure,
 which the harness records as a normal
FAIL.
 The conventions (named predicates,
 global `expect`,
 named `numRuns`,
 explicit `it` timeout,
random seed for reproducible failure output) are documented in `package/test-harness/core/README.md`.

## Tool choice

fast-check was specified by the user,
 so the alternative survey is moot;
 the open-source default and
constraint-fit checks still apply and were verified before adding it:

- Open source:
   MIT-licensed,
   published on npm,
   source at `github.com/dubzzz/fast-check`.
- Dependency weight:
   one transitive runtime dependency (`pure-rand`),
   no peer dependencies.
- Runtime fit:
   ships both ESM and CommonJS conditions and is pure JavaScript,
   so it runs under Node,
  which is how the harness self-test executes (`node <file>` per the `test:unit` task).
- Scope fit:
   needed only at test time,
   so it is a `devDependency` and stays out of the harness's
  runtime and public surface.

## Verification

The property tests were proven falsifiable,
 not just green:
 temporarily changing the cycle marker in
`package/test-harness/core/src/format-error.ts` made the cycle-termination property fail and fast-check
shrank to the minimal counterexample (`Counterexample: [1]`),
 then the source was reverted.
 A green
self-test alone does not prove a property exercises anything;
 this confirms it does.

## Outcome: a real bug found and fixed

Probing the no-throw surface showed `formatErrorDeep` was not total:
 an error object whose
`.message`/`.name`/`.cause`/`.stack`/`.errors` getter throws propagated the throw mid-walk,
 because
only `String()` conversion was guarded (via `safeString`),
 not the property reads.
 This contradicted
`safeString`'s own documented intent that trapped getters cannot derail logging.
 The reads now go
through a `readProperty` helper (`Reflect.get` inside `try`/`catch`),
 and the no-throw property
generates throwing-getter objects so it proves the fix.
 This is the property-testing payoff:
 the gap
was invisible to the example-based fixtures and surfaced only under generated adversarial input.
