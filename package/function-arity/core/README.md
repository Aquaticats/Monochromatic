# @monochromatic-dev/function-arity-core

Ready to publish.

Function arity wrappers for callbacks that receive more positional arguments than the wrapped function should see.

## Helpers

### `unary(fn)`

Returns a one-argument wrapper.
 Use it when a host API passes extra arguments that would change
wrapped-function behavior.

```ts
import { unary, } from '@monochromatic-dev/function-arity-core';

const parsed = ['10', '10', '10'].map(unary(Number.parseInt,));
// [10, 10, 10]
```

### `binary(fn)`

Returns a two-argument wrapper.
 Use it when value and index should reach callback logic,
 but source
collection should not.

```ts
import { binary, } from '@monochromatic-dev/function-arity-core';

const rendered = ['a', 'b'].map(binary(function render(
  value: string,
  index: number,
): string {
  return `${index}:${value}`;
},),);
// ['0:a', '1:b']
```

## Design decisions

- Source-only.
   No build step;
   consumers import directly from `src/index.ts`.
- Direct named exports.
   `unary` and `binary` are named for the arity they preserve.
- One file per helper.
   `unary.ts` and `binary.ts` hold implementations,
   and `index.ts` re-exports them.
- Family scope.
   This package owns callback arity wrappers.
   It does not own currying,
   partial application,
  memoization,
   or function composition.
- Extracted from `module-es`.
   These helpers previously lived under the `types.function.from.function`
  taxonomy.
   Promoting them to their own package makes direct consumers depend on the focused utility
  instead of the larger `module-es` package.
