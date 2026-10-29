/**
 Cache clearing for memoized functions.
 
 Derived from upstream `p-memoize`'s `pMemoizeClear` with the same error
 precedence: unknown function first, disabled cache second, and a cache
 storage without `clear` third.
 
 @module
 */

import type {
  AnyAsyncFunction,
} from './cache-storage.ts';
import {
  CacheDisabledError,
  NotMemoizedError,
  UnclearableCacheError,
} from './errors.ts';
import {
  CACHE_DISABLED,
  memoizedCacheStore,
} from './memoized-cache-store.ts';

//region Clearing

/**
 Drops every cached value of one memoized function, so its next call
 recomputes. In-flight calls are unaffected: their results settle and write
 the cache as usual.
 
 @param fn - Memoized function whose cache is cleared.
 
 @throws NotMemoizedError when the function was never memoized.
 
 @throws CacheDisabledError when the function was memoized with caching
 disabled.
 
 @throws UnclearableCacheError when the function's cache storage exposes no
 `clear` method.
 
 @example
 ```ts
 import { pMemoize, pMemoizeClear, } from '\@monochromatic-dev/upstream-fork-p-memoize';
 
 const memoized = pMemoize({ fn: compute, },);
 pMemoizeClear(memoized,);
 ```
 */
export function pMemoizeClear(fn: AnyAsyncFunction): void {
  if (!memoizedCacheStore.has(fn,))
    throw new NotMemoizedError();

  /**
   Cache registration recorded when the function was memoized;
   `CACHE_DISABLED` marks a caching-disabled function.
   */
  const cache = memoizedCacheStore.get(fn,);

  if ((cache === undefined) || (cache === CACHE_DISABLED))
    throw new CacheDisabledError();

  if ((typeof cache.clear) !== 'function')
    throw new UnclearableCacheError();

  // Called as a method so custom storages whose `clear` reads `this` keep
  // working, matching upstream `p-memoize`'s `cache.clear()` call.
  cache.clear();
}

//endregion Clearing
