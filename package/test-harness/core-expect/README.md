# @monochromatic-dev/test-harness-core-expect

Runtime assertions and type assertions extracted from the test harness.
The runner in [module-test](../core/README.md) still re-exports the existing interface.

## Responsibilities

- Jest-style `expect` matchers,
  including negation,
  asynchronous matchers,
  asymmetric matchers,
  and Sinon-spy assertions.
- `createScopedExpect`,
  which supplies a fresh assertion tracker for the runner to validate.
- `expectTypeOf`,
  re-exported from `expect-type`.

Chai and Sinon remain implementation dependencies.
Importing this package does not execute a test or install the runner's rejection observer.

## Use

```ts
// consumer.ts
import { expect, expectTypeOf, } from '@monochromatic-dev/test-harness-core-expect/ts';

expect([1, 1,],).toAllBe();
expectTypeOf<string>().toEqualTypeOf<string>();
```

Workspace consumers use `/ts`.
The root package entry resolves to the built neutral artifact.
For matcher semantics,
see the [harness assertion reference](../core/README.md#expectactual).

## Verification

```bash
# From the repository root.
mise run //package/test-harness/core-expect:buildAndTest
mise run //package/test-harness/core-expect:lint
```

Unit tests exercise this package's built artifact.
Runner integration retains assertion-count verdict checks in `module-test`.
