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
- `measureTwoSizeRatio` and `expectLinearTimeGrowth`,
  the two-size timing measurement and its linear-growth bound,
  so a test proves work over unbounded input stays linear
  instead of trusting a complexity comment
  (`RG2`).
  The caller owns the exposing shape:
  an input whose early exit avoids the suspected path
  passes a quadratic function,
  so land the shape with a test that proves the path runs.

Chai and Sinon remain implementation dependencies.
Importing this package does not execute a test or install the runner's rejection observer.

## Use

```ts
// consumer.ts
import { expect, expectTypeOf, } from '@monochromatic-dev/test-harness-core-expect/ts';

expect([1, 1,],).toAllBe();
expectTypeOf<string>().toEqualTypeOf<string>();
```

```ts
// linear-growth-proof.ts
import { expectLinearTimeGrowth, } from '@monochromatic-dev/test-harness-core-expect/ts';

expectLinearTimeGrowth({
  grow: (n,) => `z${'  a b'.repeat(n,)}`,
  run: (row,) => parseBinList(row,),
},);
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
