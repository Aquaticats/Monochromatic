# module-test architecture improvement plan

This plan deepens `@monochromatic-dev/test-harness-core` by concentrating failure reporting in a runner-owned Module.
It records decisions from the architecture review.
It keeps the package focused on suite and assertion primitives.

## Decisions already made

- `module-test` remains a suite primitive.
  File discovery,
  per-file process isolation,
  and aggregate execution belong to the root `mise` `test:unit` template.
  This is recorded in `doc/adr/0001-module-test-suite-primitive.md`.
- Scheduling stays numeric through `describe({ concurrency })`.
  Node method replacement through `ctx.sinon` has attempt-owned isolation under issue #481.
  Other shared-state meaning remains caller-owned documentation and test structure.
- Matcher declarations stay explicit across `MatcherSet`,
  matcher builders,
  key lists,
  wrappers,
  docs,
  and tests.
- Existing negative-path primitives stay:
  `fails`,
  `.rejects`,
  and `.resolves` remain the compatibility surface.
- The accepted deepening is a runner-owned failure reporter Module.

## Context-owned method replacement

The accepted #481 implementation keeps the existing test syntax and numeric scheduling API.
`it-attempt.ts` owns a fresh sandbox generation for each body attempt,
including repeats and timed-out bodies.
It irreversibly completes ownership before cleanup.

`sandbox.ts` exposes the test capability and retains runner-owned restoration separately.
On 2026-10-04 this file moved to `package/test-harness/core-sandbox/src/sandbox.ts`;
`it-attempt.ts` remains in the runner.
The operation adapter delegates actual fake creation to Sinon on private method facades.
A realm-shared registry selects those facades using the existing Node async execution context.
The target's getter returns the current reader's fake or its original method;
it does not dispatch to another fake at invocation time.

Only own,
configurable,
writable data methods receive contextual isolation.
Other mutation families retain ordinary semantics with collision preflight and lifetime guards.
Browsers retain ordinary Sinon method replacement rather than a claimed browser async-context implementation.

The supported behavior and deliberate reflection/assignment boundaries are documented in
[`README.md`](README.md#stubs-and-spies).
This addition does not implement or supersede the failure-reporter work in this plan.

## Accepted deepening: failure reporter Module

### Files involved

- `package/test-harness/core/src/it.ts`
- `package/test-harness/core/src/describe.ts`
- `package/test-harness/core/src/format-error.ts`.
  Moved on 2026-10-04 to `package/test-harness/core-diagnostic/src/format-error.ts`.
- `package/test-harness/core/src/assertion-source.ts`.
  Moved on 2026-10-04 to `package/test-harness/core-diagnostic/src/assertion-source.ts`.
- `package/test-harness/core/src/harness-frames.ts`.
  Moved on 2026-10-04 to `package/test-harness/core-diagnostic/src/harness-frames.ts`.
- `package/test-harness/core/src/index.ts`
- `package/test-harness/core/src/*.unit.test.ts`
- `package/test-harness/core/README.md`

### Problem

`format-error.ts` is already deep for formatting,
but the runner paths are still shallow.
`it.ts` and `describe.ts` each decide when to log,
how to summarize,
what synthetic cause to create,
and what to throw.
Every branch must remember both diagnostics and throw-shape policy.

Deletion test:
deleting `formatFailure` would push stack and cause formatting back into runner code,
but deleting the runner failure branches would still leave failure policy scattered.
The deeper Module should own the whole failure outcome:
log useful diagnostics,
then preserve the correct throw contract.

### Target shape

Add an exported,
underscored,
testable internal Module,
likely `src/failure-reporter.ts`:

```ts
// package/test-harness/core/src/failure-reporter.ts
export async function _reportItFailure(input: _ReportItFailureInput): Promise<never>;
export async function _reportDescribeFailure(input: _ReportDescribeFailureInput): Promise<never>;

export type _ReportItFailureInput = ...;
export type _ReportDescribeFailureInput = ...;
```

The reporter functions throw inside the reporter after logging.
The runner should not construct the final error or remember to throw it.

The reporter owns these failure outcomes:

- unexpected `it` throw or timeout,
  wrapping as `Error(name, { cause })`
- `fails` test that unexpectedly passed,
  with a synthetic cause created by the reporter
- assertion-count mismatch,
  with a synthetic cause created by the reporter
- `hasAssertions()` mismatch,
  with a synthetic cause created by the reporter
- suite child failure,
  preserving current single-cause versus `AggregateError` behavior
- empty-name suite child failure,
  preserving direct cause rethrow
- suite timeout,
  preserving current raw timeout rethrow

`format-error.ts`,
`assertion-source.ts`,
and `harness-frames.ts` remain implementation helpers behind the reporter seam.
Their current property tests stay useful,
but runner-level reporter tests become the main test surface for failure behavior.

## Implementation plan

1. Add `src/failure-reporter.ts` with `_reportItFailure`,
   `_reportDescribeFailure`,
   and their input types.
2. Move log-and-throw policy from `runIt` into `_reportItFailure`.
3. Move suite failure and timeout log-and-throw policy from `runDescribe` into `_reportDescribeFailure`.
4. Keep `formatFailure` focused on rendering.
   Do not make callers format summaries directly in runner branches.
5. Export the underscored functions and input types from `src/index.ts` for package self-tests.
6. Add focused reporter unit tests that use a logger adapter capturing `error` calls.
7. Keep integration tests through `it` and `describe` to verify the public suite Interface still behaves the same.
8. Update `README.md` only where it documents failure output or internal self-test conventions.

## Test plan

- Reporter unit tests:
  - assert each reporter variant logs a `FAIL` line with formatted cause detail
  - assert each reporter variant rejects with the current throw shape
  - assert synthetic causes are created only by reporter-owned synthetic variants
- Runner integration tests:
  - `it` unexpected throw still rejects with `Error(testName, { cause })`
  - `fails: true` unexpected pass still rejects with `Error(testName, { cause })`
  - assertion-count failures still reject with `Error(testName, { cause })`
  - non-empty `describe` still wraps child failures with suite name
  - empty-name `describe` still rethrows the child cause directly
  - suite timeout still rethrows the raw timeout error
- Formatting property tests remain in `format-error.property.unit.test.ts`.
- Verification command:
  `mise run //package/test-harness/core:buildAndTest`.

## Non-goals

- Do not add a file-level runner to `module-test`.
- Do not replace numeric `concurrency` with shared-state declarations.
- Do not introduce a metadata-driven matcher catalog.
- Do not replace `fails`,
  `.rejects`,
  or `.resolves` with a stricter expected-outcome Interface in this plan.
- Do not change public `describe`,
  `it`,
  or `expect` behavior.

## Acceptance criteria

- Failure reporting has one locality point in the reporter Module.
- `it.ts` and `describe.ts` no longer construct synthetic failure causes or format failure logs inline.
- The existing throw contracts are preserved.
- The log stream alone remains sufficient for diagnosis.
- `mise run //package/test-harness/core:buildAndTest` passes.
