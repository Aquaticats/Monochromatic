import {
  ABSENT,
  createStore,
  type Store,
} from '@monochromatic-dev/kv-store-core/ts';

import { buildCacheKey, } from './cache-key.ts';
import {
  DEFAULT_MAX_CACHE_SIZE,
  type MemoizeAsyncNamedOptions,
  type MemoizedAsyncFunction,
  type MemoizedCallOptions,
} from './types.ts';

/**
 Wraps an async function with memoization using a {@link Store} backend, LRU eviction,
 per-call salt-based cache keys, and in-flight Promise deduplication.
 
 Salt is provided per-call via {@link MemoizedCallOptions}, enabling dynamic
 cache invalidation without recreating the memoized function. The Store, defaulted
 via {@link createStore} when the caller supplies none, is the single source of
 truth for cached values; an in-flight `Map` deduplicates concurrent calls with
 the same key onto a single computation. LRU eviction is handled by the Store when an
 eviction policy is configured.
 
 The `keyFn` option is required to compute cache keys from arguments. The `fn` parameter is
 typed with `this: void` to disallow method-style memoization, where `this` binding would
 cause incorrect caching. Cache misses are signalled by the {@link ABSENT} sentinel returned
 from the store, so any resolved value, including `undefined` or `null`, is distinguishable
 from a miss and is cached.
 
 @typeParam TArgs - tuple of function argument types
 
 @typeParam TReturn - resolved return type (not wrapped in Promise)
 
 @param options - function and memoization configuration
 
 @returns memoized async function exposing `.store`, `.clear()`, and `.delete()`
 
 @throws whatever `fn` rejects with; the rejection propagates, the in-flight entry is cleared,
 and nothing is cached, so the next call recomputes
 
 @example
 Basic usage:
 ```ts
 import { memoizeAsync } from '\@monochromatic-dev/kv-store-memoize';
 
 const memoized = await memoizeAsync({ fn: fetchUser, keyFn: (id) => id });
 await memoized({ args: ['user-1'], salt: 'v1' }); // fetched
 await memoized({ args: ['user-1'], salt: 'v1' }); // cached
 ```
 
 @example
 Dynamic salt for cache invalidation:
 ```ts
 const memoized = await memoizeAsync({ fn: expensiveFetch, keyFn: (url) => url });
 await memoized({ args: ['/api'], salt: 'v1' }); // fetched
 await memoized({ args: ['/api'], salt: 'v2' }); // refetched (salt changed)
 ```
 
 @example
 Custom Store backend:
 ```ts
 import { createStore } from '\@monochromatic-dev/kv-store-core';
 
 const store = await createStore({
   storeId: 'fetch-cache',
   eviction: [{ policy: 'lru', maxSize: 512 }],
 });
 const memoized = await memoizeAsync({ fn: fetchData, keyFn: (id) => id, store });
 ```
 */
export async function memoizeAsync<
  const TArgs extends readonly unknown[],
  const TReturn,
  >(
  options: MemoizeAsyncNamedOptions<TArgs, TReturn>,
  ): Promise<MemoizedAsyncFunction<TArgs, TReturn>> {
  /**
   Caller-provided function plus key derivation extracted from `options` for closure capture.
   */
  const {
    fn,
    keyFn,
  } = options;
  /**
   Cache backend; defaults to a per-instance LRU store so callers without one still get bounded memory.
   */
  const store: Store = options.store
    ?? await createStore({
      storeId: `memoize-${crypto.randomUUID()}`,
      eviction: [{
        policy: 'lru',
        maxSize: DEFAULT_MAX_CACHE_SIZE,
      },],
    },);

  /**
   In-flight promises keyed by cache key, deduplicating concurrent calls onto one computation.
   */
  const inflight = new Map<string, Promise<TReturn>>();

  /**
   Create a disposable that removes a key from the inflight map on dispose.
   
   @param cacheKey - key removed from inflight on disposal
   
   @returns disposable that cleans the inflight entry on scope exit
   
   @example
   ```ts
   using _guard = inflightGuard('my-key');
   ```
   */
  function inflightGuard(cacheKey: string,): Disposable {
    return {
      [Symbol.dispose](): void {
        inflight.delete(cacheKey,);
      },
    };
  }

  /**
   Core computation: check the store, then call `fn`, clearing the inflight entry on exit.
   
   @param cacheKey - full cache key
   
   @param args - original function arguments
   
   @returns cached or freshly computed result
   
   @throws whatever `fn` rejects with; `using` clears the inflight entry before the throw escapes
   */
  async function resolveValue({
    cacheKey,
    args,
  }: Readonly<{
    cacheKey: string;
    args: TArgs;
  }>,): Promise<TReturn> {
    /**
     Auto-disposer that clears the inflight entry on scope exit, even on throw.
     */
    using _guard = inflightGuard(cacheKey,);

    /**
     Cached value if present; `ABSENT` signals a miss and triggers recomputation.
     */
    const stored = await store.get<TReturn>(cacheKey,);
    if (stored !== ABSENT)
      return stored;

    /**
     Freshly computed value persisted to `store` so subsequent calls hit the cache.
     */
    const result = await fn(...args,);
    await store.set(
      cacheKey,
      result,
    );
    return result;
  }

  /**
   Dispatch a cache lookup plus compute for the given key via {@link resolveValue},
   sharing any existing in-flight promise.
   
   @param cacheKey - full cache key
   
   @param args - original function arguments
   
   @returns promise resolving to the cached or computed value
   */
  function dispatch({
    cacheKey,
    args,
  }: Readonly<{
    cacheKey: string;
    args: TArgs;
  }>,): Promise<TReturn> {
    /**
     In-flight promise for this key, if any; returning it dedupes concurrent callers.
     */
    const existing = inflight.get(cacheKey,);
    if (existing !== undefined)
      return existing;

    /**
     Newly started computation registered in `inflight` so concurrent callers share it.
     */
    const promise = resolveValue({
      cacheKey,
      args,
    },);
    inflight.set(
      cacheKey,
      promise,
    );
    return promise;
  }

  /**
   Memoized async wrapper, combining {@link buildCacheKey} with {@link dispatch}.
   Salt is provided per-call to enable dynamic cache invalidation.
   
   @param args - original function arguments spread into keyFn and fn
   
   @param salt - per-call cache-invalidation salt appended to the key
   
   @returns cached or freshly computed result
   */
  function memoized(
    {
      args,
      salt,
    }: MemoizedCallOptions<TArgs>,
  ): Promise<TReturn> {
    /**
     Composite cache key combining the argument-derived key with the per-call `salt`.
     */
    const cacheKey = buildCacheKey({
      argKey: keyFn(...args,),
      salt,
    },);
    return dispatch({
      cacheKey,
      args,
    },);
  }

  memoized.store = store;

  memoized.clear = async function clear(): Promise<void> {
    inflight.clear();
    await store.clear();
  };

  memoized.delete = async function deleteCacheEntry(key: string,): Promise<void> {
    inflight.delete(key,);
    await store.delete(key,);
  };

  return memoized as MemoizedAsyncFunction<TArgs, TReturn>;
}
