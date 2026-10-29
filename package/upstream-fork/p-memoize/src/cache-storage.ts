/**
 Cache storage contracts shared by memoization, write predicates, and cache
 clearing.
 
 Mirrors upstream `p-memoize`'s `CacheStorage` and `ShouldCache` types with
 the same method sets and the same defaults, minus upstream's `type-fest`
 dependency.
 
 @module
 */

//region Function shapes

/**
 Any promise-returning function the fork can memoize.
 
 Declared with `never` arguments so every concrete async function type
 satisfies the constraint without `any`: parameters stay contravariant while
 `Parameters<FunctionToMemoize>` still resolves to the concrete tuple.
 
 @example
 ```ts
 const fn: AnyAsyncFunction = async function fetchUser(id: string): Promise<string> {
   return `user-${id}`;
 };
 ```
 */
export type AnyAsyncFunction = (...callArgs: never[]) => PromiseLike<unknown>;

//endregion Function shapes

//region Cache storage

/* oxlint-disable no-restricted-syntax/no-optional-escape, typescript/no-redundant-type-constituents -- external-boundary mirror of upstream p-memoize's `CacheStorage.set` return union `Promise<unknown> | unknown` */
/**
 Return of a cache write.
 
 Mirrors upstream `p-memoize`'s `CacheStorage.set` return union: a promise
 when the write is asynchronous, any other value when it is synchronous.
 */
export type CacheWriteResult = Promise<unknown> | unknown;
/* oxlint-enable no-restricted-syntax/no-optional-escape, typescript/no-redundant-type-constituents */

/**
 Storage contract a memoization cache must implement.
 
 `.has(key)`, `.get(key)`, and `.set(key, value)` may run asynchronously by
 returning a promise, so database-backed caches work as well as `Map`.
 `delete(key)` is required and `clear()` is optional; `pMemoizeClear`
 reports a missing `clear` instead of calling it.
 
 @typeParam KeyType - cache key type produced by the `cacheKey` function
 
 @typeParam ValueType - settled function result type stored under each key
 
 @example
 ```ts
 const cache: CacheStorage<string, number> = {
   has: function has(key: string): boolean {
     return false;
   },
   get: function get(key: string): number | undefined {
     return undefined;
   },
   set: function set(key: string, value: number): void {},
   delete: function remove(key: string): void {},
 };
 ```
 */
export type CacheStorage<KeyType, ValueType> = {
  /**
   Reports whether a key is present; awaiting the result is supported.
   */
  has: (key: KeyType) => Promise<boolean> | boolean;
  /**
   Reads the value under a key; awaiting the result is supported, and a
   cache miss reads `undefined`, mirroring upstream `p-memoize`'s
   `CacheStorage.get`.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- external-boundary mirror of upstream p-memoize's `CacheStorage.get`, whose miss value is `undefined`
  get: (key: KeyType) => Promise<ValueType | undefined> | ValueType | undefined;
  /**
   Writes a settled value under a key; awaiting the result is supported, and
   the return value mirrors upstream `p-memoize`'s `Promise<unknown> |
   unknown` (a promise for asynchronous writes, anything else for
   synchronous ones).
   */
  set: (
    key: KeyType,
    value: ValueType,
  ) => CacheWriteResult;
  /**
   Removes the entry under a key.
   */
  delete: (key: KeyType) => unknown;
  /**
   Drops every entry; absent on storages like `WeakMap` that cannot clear.
   */
  clear?: () => unknown;
};

//endregion Cache storage

//region Write predicate

/**
 Decides whether a fulfilled value should be written to the cache.
 
 Runs after the memoized function fulfills and before `cache.set`, so
 selective caching can skip writes while in-flight de-duplication still
 works. Returning `false` skips the write; throwing or rejecting propagates
 the failure to the caller and skips caching.
 
 @typeParam KeyType - cache key type produced by the `cacheKey` function
 
 @typeParam ValueType - settled function result type being written
 
 @typeParam ArgumentsList - argument tuple passed to the memoized function
 
 @example
 ```ts
 const shouldCache: ShouldCache<string, number | undefined, [string]> = (
   value,
   context,
 ) => value !== undefined;
 ```
 */
export type ShouldCache<
  KeyType,
  ValueType,
  ArgumentsList extends readonly unknown[] = readonly unknown[],
  > = (
    value: ValueType,
    context: {
      key: KeyType;
      argumentsList: ArgumentsList;
    },
  ) => boolean | Promise<boolean>;

//endregion Write predicate

//region Cache guard

/**
 Reports whether a configured cache value enables caching.
 
 Upstream `p-memoize` gates cache access on truthiness (`if (cache && ...)`),
 so this guard keeps falsy untyped `cache` values disabling caching exactly
 like upstream while narrowing `cache: CacheStorage | false` for the type
 system.
 
 @typeParam KeyType - cache key type produced by the `cacheKey` function
 
 @typeParam ValueType - settled function result type stored under each key
 
 @param cache - Configured cache storage or the disabled `false` sentinel.
 
 @returns `true` when cache reads and writes should run.
 
 @example
 ```ts
 isCacheEnabled(new Map(),); // => true
 isCacheEnabled(false,); // => false
 ```
 */
export function isCacheEnabled<KeyType, ValueType>(
  // oxlint-disable-next-line no-restricted-syntax/no-optional-escape -- external-boundary mirror of upstream p-memoize's `Options.cache` union `CacheStorage | false`, whose truthiness this guard evaluates
  cache: CacheStorage<KeyType, ValueType> | false,
): cache is CacheStorage<KeyType, ValueType> {
  return Boolean(cache);
}

//endregion Cache guard
