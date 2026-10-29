/**
 Registry mapping each memoized function to the cache it writes.
 
 Upstream `p-memoize` keeps this `WeakMap` beside its memoizer, storing
 `false` for caching-disabled functions; the fork splits it out so `pMemoize`
 (which records) and `pMemoizeClear` (which reads) share one owned
 definition, and stores a `CACHE_DISABLED` symbol instead of `false` because
 repository lint bans falsy sentinels.
 
 @module
 */

import type {
  AnyAsyncFunction,
} from './cache-storage.ts';

//region Sentinels

/**
 Registry value for a memoized function created with caching disabled.
 
 @example
 ```ts
 memoizedCacheStore.set(memoized, CACHE_DISABLED,);
 ```
 */
export const CACHE_DISABLED: unique symbol = Symbol(
  // mutation-test-disable-next-line string -- the description is debugging metadata only; consumers narrow by symbol identity
  'memoized function caches nothing',
);

//endregion Sentinels

//region Shared generation

/**
 Clear counter shared by every memoized function over one cache storage.

 A `pMemoizeClear` bumps `generation`, so a call that started before the
 clear can tell its result no longer belongs in the cache. The counter hangs
 off the cache rather than off one function because two functions over one
 cache share its entries.

 @example
 ```ts
 const shared: SharedCache = { generation: 0, };
 ```
 */
export type SharedCache = {
  /**
   Number of clears the cache has seen so far.
   */
  generation: number;
};

/**
 Generation counters keyed by cache storage; weak keys let a counter live
 exactly as long as its cache does.
 */
const sharedCaches = new WeakMap<object, SharedCache>();

/**
 Finds or creates the generation counter of one cache storage.

 @param cache - Cache storage whose counter is wanted.

 @returns Counter shared by every memoized function over that storage.

 @example
 ```ts
 const shared = sharedCacheOf(new Map(),);
 ```
 */
export function sharedCacheOf(cache: object,): SharedCache {
  /**
   Counter already created for this storage, when one exists.
   */
  const existing = sharedCaches.get(cache,);

  if (existing !== undefined)
    return existing;

  /**
   Fresh counter for a storage seen for the first time.
   */
  const created: SharedCache = {
    generation: 0,
  };
  sharedCaches.set(
    cache,
    created,
  );
  return created;
}

//endregion Shared generation

//region Registration

/**
 What {@link memoizedCacheStore} remembers about one memoized function:
 everything `pMemoizeClear` must reach, including the in-flight promises,
 which live outside the cache but are part of the memoized state.

 @example
 ```ts
 const registration: MemoizedCacheRegistration = {
   cache: new Map(),
   promiseCache: new Map(),
   shared: { generation: 0, },
 };
 ```
 */
export type MemoizedCacheRegistration = {
  /**
   Cache storage the function reads and writes; `clear` is absent on
   storages that cannot clear. Held as the storage itself so `clear` is
   called as a method.
   */
  readonly cache: {
    readonly clear?: () => unknown;
  };
  /**
   In-flight promises of this function, dropped on a clear so a later call
   starts fresh instead of joining one that began before it.
   */
  readonly promiseCache: {
    readonly clear: () => void;
  };
  /**
   Generation counter shared with every function over the same cache.
   */
  readonly shared: SharedCache;
};

//endregion Registration

//region Store

/**
 Registry type: weak map from memoized function to its cache registration.
 */
export type MemoizedCacheStore = WeakMap<AnyAsyncFunction, MemoizedCacheRegistration | typeof CACHE_DISABLED>;

/**
 Weak map from memoized function to its cache registration.

 Weak keys let a memoized function and its cache be collected together once
 every caller drops the function.

 @example
 ```ts
 memoizedCacheStore.set(memoized, registration,);
 ```
 */
export const memoizedCacheStore: MemoizedCacheStore = new WeakMap();

//endregion Store
