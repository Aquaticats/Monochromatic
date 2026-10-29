/**
 TypeScript fork of [`p-memoize`](https://github.com/sindresorhus/p-memoize) by
 Sindre Sorhus (MIT): memoize promise-returning and async functions.
 
 Rewritten to this repository's TypeScript standards with the upstream
 `p-memoize` 8.0.0 memoization semantics preserved (plus the 9.0.0 fix for
 synchronously throwing functions with caching disabled; see the package
 README for the 9.0.0 changes not yet ported): cache keys derive from
 the argument tuple, the cache holds only fulfilled values, concurrent
 calls on one key share one promise, `shouldCache` gates writes after
 fulfillment, and rejections are never cached. The only API-shape
 deviation is lint-mandated: a memoized call passes an `args` tuple instead
 of a rest parameter.
 
 @example
 ```ts
 import { pMemoize, } from '\@monochromatic-dev/upstream-fork-p-memoize';
 
 const memoized = pMemoize({
   fn: async function fetchUser(id: string): Promise<string> {
     return `user-${id}`;
   },
 },);
 
 const results = await Promise.all([
   memoized({ args: ['a'], },),
   memoized({ args: ['a'], },),
 ]);
 ```
 
 @packageDocumentation
 */

export {
  CacheDisabledError,
  NonMethodDecorationError,
  NotMemoizedError,
  PropertyDescriptorMissingError,
  PrivateMethodDecorationError,
  UnclearableCacheError,
} from './errors.ts';

export {
  type AnyAsyncFunction,
  type CacheStorage,
  type ShouldCache,
} from './cache-storage.ts';

export {
  type MemoizedCall,
  type MemoizedFunction,
  type MemoizeOptions,
  pMemoize,
} from './p-memoize.ts';

export {
  pMemoizeDecorator,
} from './p-memoize-decorator.ts';

export {
  pMemoizeClear,
} from './p-memoize-clear.ts';

// Exported for built-artifact tests: the function-identity copy's legal,
// skipped, and default-flag paths are only observable with hand-crafted
// source and target properties.
export {
  mimicFunction,
} from './mimic-function.ts';
