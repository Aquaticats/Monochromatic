/**
 Cache clearing for memoized functions.
 
 Derived from upstream `p-memoize` 9.0.0's `pMemoizeClear` with the same
 error precedence: unknown function first, disabled cache second, and a cache
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
 Reports whether a value handed back by a cache method is something to
 await rather than a plain value.

 @param value - Result of a cache method call.

 @returns `true` when the value has a callable `then`.

 @example
 ```ts
 isThenable(Promise.resolve(),); // => true
 ```
 */
function isThenable(value: unknown,): value is PromiseLike<unknown> {
  return (((typeof value) === 'object') || ((typeof value) === 'function'))
    && (value !== null)
    && ('then' in value)
    && ((typeof value.then) === 'function');
}

/**
 Awaits a pending cache `clear()` and settles with its failure instead of
 rejecting.

 `pMemoizeClear` is synchronous, so a rejection from an asynchronous
 `clear()` has nowhere to go; left unobserved it would take the process down
 as an unhandled rejection.

 @param pending - Thenable returned by the cache's `clear()`.

 @returns The rejection reason, or `undefined` when the clear fulfilled;
 callers drop it by design.

 @example
 ```ts
 await observeAndDrop(cache.clear(),);
 ```
 */
async function observeAndDrop(pending: PromiseLike<unknown>,): Promise<unknown> {
  try {
    await pending;
    return undefined;
  }
  catch (error) {
    return error;
  }
}

/**
 Drops every cached value of one memoized function, so its next call
 recomputes.

 What the clear reaches:

 - The cache is emptied for every memoized function sharing it, and a call
 still running on any of them will not write its result back into it.
 - Only this function's in-flight calls are dropped, so a later call starts
 fresh instead of joining one that began before the clear. Another memoized
 function over the same cache keeps its own in-flight calls until they
 settle.

 A `cache.set` that already started when the clear runs is not undone, so
 that value can still land in the cleared cache.

 This function is synchronous: a `clear()` that returns a promise finishes
 after it returns, and a rejection from it is dropped rather than reported.
 A `clear()` that throws synchronously still propagates to the caller.

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
   Registration recorded when the function was memoized; `CACHE_DISABLED`
   marks a caching-disabled function.
   */
  const registration = memoizedCacheStore.get(fn,);

  if ((registration === undefined) || (registration === CACHE_DISABLED))
    throw new CacheDisabledError();

  /**
   Cache storage, in-flight map, and shared generation counter of the
   function.
   */
  const {
    cache,
    promiseCache,
    shared,
  } = registration;

  if ((typeof cache.clear) !== 'function')
    throw new UnclearableCacheError();

  // Bumping the shared generation stops the calls still running on any
  // function over this cache from writing into it. Done before emptying the
  // cache and not conditional on it succeeding: a `clear()` that fails
  // halfway must not leave those calls free to refill it.
  shared.generation += 1;

  // Drop this function's in-flight promises so a later call starts fresh
  // instead of joining one that was already running at clear time.
  promiseCache.clear();

  // Called as a method so custom storages whose `clear` reads `this` keep
  // working, matching upstream `p-memoize`'s `cache.clear()` call.
  /**
   Whatever the storage's `clear` handed back: a promise for asynchronous
   storages, anything else for synchronous ones.
   */
  const cleared = cache.clear();

  if (isThenable(cleared,))
    void observeAndDrop(cleared,);
}

//endregion Clearing
