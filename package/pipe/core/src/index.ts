/**
 Type-safe left-to-right function composition.
 
 Four functions, two axes. The eager/deferred axis follows rambdax: `piped`/`pipedAsync` are
 eager (value-first, run immediately and return the result), while `pipe`/`pipeAsync` are
 deferred (point-free, return a reusable pipeline function). The sync/async axis adds
 sequential awaiting. Across both, running a value eagerly equals deferring then applying.
 
 Steps and the value travel as named keys (`fn1..fn9`) on one object, not an array: a classic
 variadic `pipe(value, ...fns)` cannot be typed safely and is doubly banned by the workspace
 (`no-rest-params`, `require-destructured-params`), and `noUncheckedIndexedAccess` would force
 a guard on every `fns[i]`. Arity is typed by hand-written 1..9 overloads; a gap, zero steps,
 or a tenth step is a compile error. The internal `run.ts`/`types.ts`/`errors.ts` are not part
 of the public surface.
 
 @example
 Eager, value-first ({@link piped}):
 ```ts
 import { piped } from '\@monochromatic-dev/pipe-core';
 
 piped({ value: 2, fn1: (x: number) => x + 1, fn2: (x: number) => `n=${x}` }); // 'n=3'
 ```
 
 @example
 Deferred, point-free ({@link pipe}):
 ```ts
 import { pipe } from '\@monochromatic-dev/pipe-core';
 
 const process = pipe({ fn1: (x: number) => x + 1, fn2: (x: number) => `n=${x}` });
 process(2); // 'n=3'
 ```
 
 @example
 Eager async ({@link pipedAsync}):
 ```ts
 import { pipedAsync } from '\@monochromatic-dev/pipe-core';
 
 await pipedAsync({ value: 2, fn1: async (x: number) => x + 1, fn2: (x: number) => `n=${x}` }); // 'n=3'
 ```
 
 @example
 Deferred async ({@link pipeAsync}):
 ```ts
 import { pipeAsync } from '\@monochromatic-dev/pipe-core';
 
 const process = pipeAsync({ fn1: async (x: number) => x + 1, fn2: (x: number) => `n=${x}` });
 await process(2); // 'n=3'
 ```
 
 @packageDocumentation
 */

export { pipe, } from './pipe.ts';
export { pipeAsync, } from './pipe-async.ts';
export { piped, } from './piped.ts';
export { pipedAsync, } from './piped-async.ts';
