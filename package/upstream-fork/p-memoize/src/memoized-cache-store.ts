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

//region Registration

/**
 What {@link memoizedCacheStore} remembers about one memoized function: the
 part of its cache storage that `pMemoizeClear` needs, or
 {@link CACHE_DISABLED} when caching is disabled for it.
 
 @example
 ```ts
 const registration: MemoizedCacheRegistration = {
   clear: function clear(): void {},
 };
 ```
 */
export type MemoizedCacheRegistration = {
  /**
   Drops every cache entry; absent on storages that cannot clear.
   */
  readonly clear?: () => unknown;
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
 memoizedCacheStore.set(memoized, cache,);
 ```
 */
export const memoizedCacheStore: MemoizedCacheStore = new WeakMap();

//endregion Store
