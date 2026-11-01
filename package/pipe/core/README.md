# module-pipe

Ready to publish.

Type-safe left-to-right function composition.
 Four functions across two axes:
eager versus deferred,
 and synchronous versus asynchronous.
 The eager forms (`piped`,
`pipedAsync`) take a value and run it through the steps immediately.
 The deferred forms
(`pipe`,
 `pipeAsync`) take only the steps and return a reusable pipeline function.
 Runtime-neutral.

For sequential asynchronous transformations,
 this module crowds out [p-waterfall](https://github.com/sindresorhus/p-waterfall)
 in this workspace.
 Use `pipedAsync` to run steps immediately or `pipeAsync` to build a reusable pipeline.
 Unlike p-waterfall's iterable of tasks,
 these pipelines use named step keys and require an initial value when run.

## Concepts

Steps and the value travel as named keys on a single object (`{ value, fn1, fn2, ... }` for the
eager forms,
 `{ fn1, fn2, ... }` for the deferred forms),
 not as an array or rest arguments.
Composition runs left to right:
 `fn1` first,
 then `fn2` on its result,
 and so on.
 The eager and
deferred forms are mirrors of each other:
 running a value eagerly through `piped` matches
deferring the same steps with `pipe` and then calling the pipeline on that value (and likewise
for the async pair).

One to nine steps type-check.
 Zero steps,
 a gap (a present step after an absent one),
 or a tenth
step is a compile error.
 Beyond nine steps,
 compose two pipes.

Each function accepts an optional `l` logger.
 When supplied,
 the function wraps it with its own
tag and the core re-tags again,
 so messages read `[piped] [runPipe] ...`.
 When omitted,
 the
module-level singleton logger is used.

## Eager, value-first

```ts
import { piped } from '@monochromatic-dev/pipe-core';

piped({
  value: 2,
  fn1: (x: number) => x + 1,
  fn2: (x: number) => `n=${x}`,
}); // 'n=3'
```

Step parameter types are inferred from `value` without annotation.
 The result type is the last
step's return type.

## Deferred, point-free

```ts
import { pipe } from '@monochromatic-dev/pipe-core';

const process = pipe({
  fn1: (x: number) => x + 1,
  fn2: (x: number) => `n=${x}`,
});

process(2); // 'n=3'
process(9); // 'n=10'
```

No `value` key.
 The input type is inferred from `fn1`'s declared parameter,
 so annotate that
parameter.
 The returned function is reusable across inputs.

## Eager async

```ts
import { pipedAsync } from '@monochromatic-dev/pipe-core';

await pipedAsync({
  value: 2,
  fn1: async (x: number) => x + 1,
  fn2: (x: number) => `n=${x}`,
}); // 'n=3'
```

Steps run sequentially:
 the initial value is awaited first,
 then each intermediate result is
awaited before the next step.
 A step may return a plain value or a promise.
 The value itself may
be a `T` or a `Promise<T>`.
 The call resolves to a promise of the last step's awaited result.

## Deferred async

```ts
import { pipeAsync } from '@monochromatic-dev/pipe-core';

const process = pipeAsync({
  fn1: async (x: number) => x + 1,
  fn2: (x: number) => `n=${x}`,
});

await process(2);                    // 'n=3'
await process(Promise.resolve(9));   // 'n=10'
```

The returned pipeline accepts a value or a promise of that value,
 since it awaits the input first.

## Passing a logger

```ts
import { pipedAsync } from '@monochromatic-dev/pipe-core';
import { tagged } from '@monochromatic-dev/logger-core/tagged';

await pipedAsync({
  value: 2,
  fn1: async (x: number) => x + 1,
  fn2: (x: number) => x * 10,
  l: tagged({ tag: 'checkout' }),
}); // logs under [checkout] [pipedAsync] [runPipeAsync]
```

## Design decisions

- Named keys,
   not an array or rest arguments.
   A classic variadic `pipe(value, ...fns)` cannot be
  typed safely,
   and the workspace doubly bans it:
   `no-rest-params` and `require-destructured-params`
  both carry un-disableable meta-bans.
   Named keys (`fn1..fn9`) are typed properties,
   so there is no
  index access.
   With `noUncheckedIndexedAccess` on,
   an array `fns[i]` would be `T | undefined` and
  force a null guard on every access;
   named keys avoid that entirely.

- Arity by hand-written overloads,
   capped at nine.
   Each function declares one overload per arity
  (1 to 9),
   with the step input and output types threaded through `TStep1..TStep9`.
   Each smaller
  overload forbids the higher `fnN` keys with optional-`never` tail properties,
   so a gap or an
  explicit `undefined` step fails to type-check for both fresh object literals and pre-built
  variables.
   Zero steps and a tenth step are compile errors too.

- Hand-written if-chain,
   not `reduce`.
   The core dispatches on which `fnN` is the first absent key
  and applies the present steps as explicit nested calls.
   No array indexing,
   no recursion.

- Async awaits sequentially and accepts `T | Promise<T>`.
   Steps never run in parallel;
   each result
  is awaited before the next step receives it,
   and the initial value is awaited first.

- Optional logger with deep tag composition.
   Each function wraps the caller's logger with its own
  tag before the work runs;
   the core re-tags again.
   For the deferred forms,
   wrapping happens inside
  the returned pipeline,
   so each call is tagged at call time rather than once at definition time.

- Runtime-neutral build.
   The package ships a tsdown-built `.` entry (neutral and node variants).

- Defense in depth at runtime.
   Callers that bypass the types (plain JavaScript,
   or an `as` cast)
  and pass a gapped or overflowing object get a thrown error rather than silently dropped steps.

## Source escape hatch

The package ships built output for its `.` entry.
 Source modules are reachable through the `./ts`
and `./ts/*` exports for exceptional source-level imports.
 The internal core (`run.ts`),
 shared
types (`types.ts`),
 and the error classes (`errors.ts`) are not part of the public surface.
