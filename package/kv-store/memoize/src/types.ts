import type {
  Store,
  SyncStore,
} from '@monochromatic-dev/kv-store-core/ts';

/**
 Maximum cache entries before LRU eviction in default memoize stores.
 */
export const DEFAULT_MAX_CACHE_SIZE = 1_024;

/**
 Options for sync memoization via {@link MemoizedFunction}.
 
 @typeParam TArgs - tuple of memoized function argument types
 
 @example
 ```ts
 const options: MemoizeOptions<[number, number]> = {
   keyFn: (a, b) => `${String(a)}:${String(b)}`,
 };
 ```
 */
export type MemoizeOptions<
  TArgs extends readonly unknown[],
  > = {
  /**
   Computes cache key from arguments; must be deterministic.
   Required because variadic argument hashing is error-prone, so the caller owns key derivation.
   */
    readonly keyFn: (
      this: void,
      ...args: TArgs
    ) => string;
  /**
   Sync store backend for cache persistence.
   Defaults to a fresh in-memory SyncStore with LRU eviction at {@link DEFAULT_MAX_CACHE_SIZE}.
   */
    readonly store?: SyncStore;
  };

/**
 Options for async memoization via {@link MemoizedAsyncFunction}.
 
 @typeParam TArgs - tuple of memoized function argument types
 
 @example
 ```ts
 const options: MemoizeAsyncOptions<[string]> = {
   keyFn: (url) => url,
 };
 ```
 */
export type MemoizeAsyncOptions<
  TArgs extends readonly unknown[],
  > = {
  /**
   Computes cache key from arguments; must be deterministic.
   Required because variadic argument hashing is error-prone, so the caller owns key derivation.
   */
    readonly keyFn: (
      this: void,
      ...args: TArgs
    ) => string;
  /**
   Async store backend for cache persistence.
   Defaults to a fresh in-memory Store with LRU eviction at {@link DEFAULT_MAX_CACHE_SIZE}.
   */
    readonly store?: Store;
  };

/**
 Call-site options passed to a memoized function on each invocation.
 
 @typeParam TArgs - tuple of original function argument types
 
 @example
 ```ts
 memoized({ args: [1, 2], salt: 'v1' });
 ```
 */
export type MemoizedCallOptions<
  TArgs extends readonly unknown[],
  > = {
  /**
   Original function arguments as a tuple, spread into both `keyFn` and the wrapped function.
   */
    readonly args: TArgs;
  /**
   Salt appended to cache key; change it to invalidate the cache
   (e.g. `String(time % 3600000)` for hourly expiry) without recreating the memoized function.
   */
    readonly salt: string;
  };

/**
 Memoized synchronous function with cache management methods.
 
 @typeParam TArgs - tuple of function argument types
 
 @typeParam TReturn - function return type
 
 @example
 ```ts
 const memoized: MemoizedFunction<[number], number> = memoize({ fn: square, keyFn: String });
 memoized({ args: [42], salt: 'v1' }); // computed
 memoized({ args: [42], salt: 'v1' }); // cached
 memoized.clear();
 ```
 */
export type MemoizedFunction<
  TArgs extends readonly unknown[],
  TReturn,
  > = {
    (
      this: void,
      options: MemoizedCallOptions<TArgs>,
    ): TReturn;
  /**
   Underlying SyncStore, exposed for inspection and direct manipulation.
   */
    readonly store: SyncStore;
  /**
   Wipe all cached entries.
   */
    readonly clear: () => void;
  /**
   Remove a specific entry by its full cache key (keyFn result plus salt).
   */
    readonly delete: (key: string,) => void;
  /**
   Current number of cached entries.
   */
    readonly size: number;
  };

/**
 Memoized asynchronous function with cache management methods.
 
 @typeParam TArgs - tuple of function argument types
 
 @typeParam TReturn - resolved return type (not wrapped in Promise)
 
 @example
 ```ts
 const memoized: MemoizedAsyncFunction<[string], User> = await memoizeAsync({ fn: fetchUser, keyFn: (id) => id });
 await memoized({ args: ['user-1'], salt: 'v1' }); // fetched
 await memoized({ args: ['user-1'], salt: 'v1' }); // cached
 await memoized.clear();
 ```
 */
export type MemoizedAsyncFunction<
  TArgs extends readonly unknown[],
  TReturn,
  > = {
    (
      this: void,
      options: MemoizedCallOptions<TArgs>,
    ): Promise<TReturn>;
  /**
   Underlying Store, exposed for inspection and direct manipulation.
   */
    readonly store: Store;
  /**
   Wipe all cached entries and drop any in-flight deduplication state.
   */
    readonly clear: () => Promise<void>;
  /**
   Remove a specific entry by its full cache key (keyFn result plus salt).
   */
    readonly delete: (key: string,) => Promise<void>;
  };

/**
 Named-parameter options for sync memoization, extending {@link MemoizeOptions}
 by bundling the function to memoize with its configuration.
 
 @typeParam TArgs - tuple of function argument types
 
 @typeParam TReturn - function return type
 
 @example
 ```ts
 const opts: MemoizeNamedOptions<[number], number> = {
   fn: (x) => x * 2,
   keyFn: String,
 };
 ```
 */
export type MemoizeNamedOptions<
  TArgs extends readonly unknown[],
  TReturn,
  > = MemoizeOptions<TArgs> & {
  /**
   Pure synchronous function to memoize; impure functions produce incorrect cached results.
   */
    readonly fn: (
      this: void,
      ...args: TArgs
    ) => TReturn;
  };

/**
 Named-parameter options for async memoization, extending {@link MemoizeAsyncOptions}
 by bundling the function to memoize with its configuration.
 
 @typeParam TArgs - tuple of function argument types
 
 @typeParam TReturn - resolved return type (not wrapped in Promise)
 
 @example
 ```ts
 const opts: MemoizeAsyncNamedOptions<[string], User> = {
   fn: fetchUser,
   keyFn: (id) => id,
 };
 ```
 */
export type MemoizeAsyncNamedOptions<
  TArgs extends readonly unknown[],
  TReturn,
  > = MemoizeAsyncOptions<TArgs> & {
  /**
   Pure async function to memoize; impure functions produce incorrect cached results.
   */
    readonly fn: (
      this: void,
      ...args: TArgs
    ) => Promise<TReturn>;
  };
