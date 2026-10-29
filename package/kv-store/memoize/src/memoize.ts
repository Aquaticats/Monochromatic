import {
  ABSENT,
  createSyncStore,
  type SyncStore,
} from '@monochromatic-dev/kv-store-core/ts';

import { buildCacheKey, } from './cache-key.ts';
import {
  DEFAULT_MAX_CACHE_SIZE,
  type MemoizedCallOptions,
  type MemoizedFunction,
  type MemoizeNamedOptions,
} from './types.ts';

/**
 Wraps a synchronous function with memoization using a {@link SyncStore} backend,
 LRU eviction, and per-call salt-based cache keys.
 
 Salt is provided per-call via {@link MemoizedCallOptions}, enabling dynamic
 cache invalidation without recreating the memoized function. The store
 defaults via {@link createSyncStore} when the caller supplies none.
 
 The `keyFn` option is required to compute cache keys from arguments; this
 prevents accidental memoization of variadic functions without explicit key derivation.
 The `fn` parameter is typed with `this: void` to disallow method-style memoization,
 where `this` binding would cause incorrect caching.
 
 Cache misses are signalled by the {@link ABSENT} sentinel returned from the store, so any
 stored value, including `undefined` or `null`, is distinguishable from a miss and is cached.
 
 @typeParam TArgs - tuple of function argument types
 
 @typeParam TReturn - function return type
 
 @param options - function and memoization configuration
 
 @returns memoized function exposing `.store`, `.clear()`, `.delete()`, and `.size`
 
 @throws whatever `fn` throws; the throw propagates and nothing is cached for that key
 
 @example
 Basic usage:
 ```ts
 import { memoize } from '\@monochromatic-dev/kv-store-memoize';
 
 const memoizedAdd = memoize({
   fn: (a: number, b: number) => a + b,
   keyFn: (a, b) => `${String(a)}:${String(b)}`,
 });
 memoizedAdd({ args: [1, 2], salt: 'v1' }); // computed: 3
 memoizedAdd({ args: [1, 2], salt: 'v1' }); // cached: 3
 ```
 
 @example
 Dynamic salt for cache invalidation:
 ```ts
 const memoized = memoize({ fn: expensiveCompute, keyFn: (input) => input });
 memoized({ args: ['data'], salt: 'v1' }); // computed
 memoized({ args: ['data'], salt: 'v1' }); // cached
 memoized({ args: ['data'], salt: 'v2' }); // recomputed (salt changed)
 ```
 
 @example
 Custom store with a different eviction size:
 ```ts
 import { createSyncStore } from '\@monochromatic-dev/kv-store-core';
 
 const store = createSyncStore({
   storeId: 'my-memo',
   eviction: [{ policy: 'lru', maxSize: 256 }],
 });
 const memoized = memoize({ fn: compute, keyFn: String, store });
 ```
 */
export function memoize<
  const TArgs extends readonly unknown[],
  const TReturn,
  >(
  options: MemoizeNamedOptions<TArgs, TReturn>,
  ): MemoizedFunction<TArgs, TReturn> {
  /**
   Function and key derivation destructured for repeated use inside the wrapper.
   */
  const {
    fn,
    keyFn,
  } = options;
  /**
   Backing cache, supplied by caller or freshly created with the default LRU policy.
   */
  const store: SyncStore = options.store
    ?? createSyncStore({
      storeId: `memoize-${crypto.randomUUID()}`,
      eviction: [{
        policy: 'lru',
        maxSize: DEFAULT_MAX_CACHE_SIZE,
      },],
    },);

  /**
   Memoized wrapper that checks the store before calling the original function,
   keyed via {@link buildCacheKey}. Salt is provided per-call to enable dynamic
   cache invalidation.
   
   @param args - original function arguments spread into keyFn and fn
   
   @param salt - per-call cache-invalidation salt appended to the key
   
   @returns cached or freshly computed result
   */
  function memoized(
    {
      args,
      salt,
    }: MemoizedCallOptions<TArgs>,
  ): TReturn {
    /**
     Salted cache key combining the keyFn output with the per-call salt.
     */
    const cacheKey = buildCacheKey({
      argKey: keyFn(...args,),
      salt,
    },);

    /**
     Previously memoized return value, or `ABSENT` when not yet stored (a miss).
     */
    const cached = store.get<TReturn>(cacheKey,);
    if (cached !== ABSENT)
      return cached;

    /**
     Freshly computed return value persisted into the store for future calls.
     */
    const result = fn(...args,);
    store.set(
      cacheKey,
      result,
    );
    return result;
  }

  memoized.store = store;

  memoized.clear = function clear(): void {
    store.clear();
  };

  memoized.delete = function deleteCacheEntry(key: string,): void {
    store.delete(key,);
  };

  Object.defineProperty(
    memoized,
    'size',
    {
      get(): number {
        return store.size;
      },
      enumerable: true,
      configurable: false,
    },
  );

  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- memoized function is extended with store/clear/delete/size properties to satisfy MemoizedFunction
  return memoized as MemoizedFunction<TArgs, TReturn>;
}
