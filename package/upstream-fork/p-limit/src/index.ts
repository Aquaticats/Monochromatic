/**
 TypeScript fork of [`p-limit`](https://github.com/sindresorhus/p-limit) by
 Sindre Sorhus (MIT): run promise-returning functions with limited
 concurrency.
 
 Rewritten to this repository's TypeScript standards with the upstream
 `p-limit` 7.3.3 scheduling semantics preserved: calls start asynchronously
 in FIFO order, at most `concurrency` run at once, and each call's promise
 settles with its function's result or failure. The only API-shape
 deviations are lint-mandated: calls pass an `args` tuple instead of a rest
 parameter, and `map` takes a destructured options object.
 
 @example
 ```ts
 import { pLimit, } from '\@monochromatic-dev/upstream-fork-p-limit';
 
 const limit = pLimit({ concurrency: 2, });
 const results = await Promise.all([
   limit({ fn: fetchUser, args: ['a'], }),
   limit({ fn: fetchUser, args: ['b'], }),
 ]);
 ```
 
 @packageDocumentation
 */

export {
  InvalidConcurrencyError,
  InvalidRejectOnClearError,
} from './errors.ts';

export { type LimitOptions, } from './limit-options.ts';

export {
  limitFunction,
  type LimitedFunction,
} from './limit-function.ts';

export {
  pLimit,
  type LimitFunction,
  type LimitMapOptions,
  type LimitMapper,
  type LimitTask,
} from './p-limit.ts';
