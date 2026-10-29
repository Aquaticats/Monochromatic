/**
 TypeScript fork of [`p-memoize`](https://github.com/sindresorhus/p-memoize) by
 Sindre Sorhus (MIT): memoize promise-returning and async functions.
 
 Derived from `p-memoize` 8.0.0 and tracking 9.0.0; see `LICENSES/MIT.txt`
 and this package's README for the full attribution. Memoization semantics
 follow upstream:
 cache keys derive from the argument tuple, the cache holds only fulfilled
 values, in-flight calls for one key share one promise, and `shouldCache`
 gates writes after fulfillment. The only call-shape deviation is
 lint-mandated: a memoized call passes an `args` tuple instead of a rest
 parameter.
 
 @module
 */

import {
  isCacheEnabled,
  type CacheStorage,
  type ShouldCache,
} from './cache-storage.ts';
import {
  CACHE_DISABLED,
  memoizedCacheStore,
  sharedCacheOf,
} from './memoized-cache-store.ts';
import { mimicFunction, } from './mimic-function.ts';

//region Types

/**
 Configuration accepted by `pMemoize`.
 
 Mirrors upstream `p-memoize`'s `Options` with the same defaults: a `Map`
 cache keyed by the first argument, every fulfilled value written, and
 rejections never cached.
 
 @typeParam TArgs - tuple of wrapped function argument types
 
 @typeParam TResult - wrapped function return type, awaited when thenable
 
 @typeParam CacheKeyType - key type produced by the `cacheKey` function
 
 @example
 ```ts
 const options: MemoizeOptions<[string], string, string> = {
   cacheKey: ([url],) => url,
 };
 ```
 */
export type MemoizeOptions<
  TArgs extends readonly unknown[],
  TResult,
  CacheKeyType,
  > = {
  /**
   Derives the cache key from a copy of the argument tuple. By default only
   the first argument is considered, which works for primitives and
   identity-comparable references. Return any type the chosen `cache`
   supports; for value-keyed arguments a serializer like `JSON.stringify`
   works. A call with no arguments shares a key with an explicit `undefined`
   argument. A function that throws makes the call reject, and a returned
   promise is rejected by the type because every promise is a fresh key.
   
   @defaultValue ([firstArgument],) => firstArgument
   
   @example ([url, options],) => JSON.stringify([url, options],)
   */
    readonly cacheKey?: (args: TArgs) => (CacheKeyType extends PromiseLike<unknown> ? never : CacheKeyType);
  /**
   Cache storage to read and write. Must implement `.has(key)`, `.get(key)`,
   `.set(key, value)`, and `.delete(key)`, and may implement `.clear()`;
   `has`, `get`, and `set` may return promises. Pass `false` to disable
   caching entirely so only concurrent calls share a result.
   
   @defaultValue new Map()
   
   @example new WeakMap()
   */
  // oxlint-disable-next-line no-restricted-syntax/no-optional-escape -- external-boundary mirror of upstream p-memoize's `Options.cache` union `CacheStorage | false`
    readonly cache?: CacheStorage<CacheKeyType, TResult> | false;
  /**
   Decides whether a fulfilled value is written to the cache. Runs after the
   function fulfills and before `cache.set`: omit to always write, return
   `false` to skip the write while clearing in-flight de-duplication, or
   throw to propagate the failure and skip caching. Reads are unaffected.
   
   @example (value,) => value !== undefined
   */
    readonly shouldCache?: ShouldCache<CacheKeyType, TResult, TArgs>;
  };

/**
 One memoized call: the argument tuple forwarded into the memoized
 function.
 
 The `args` tuple replaces upstream `p-memoize`'s `memoized(...arguments_)`
 rest parameter, which repository lint bans; the wrapped function itself
 still receives the tuple spread as real arguments.
 
 @typeParam TArgs - tuple of wrapped function argument types
 
 @example
 ```ts
 const call: MemoizedCall<[string]> = {
   args: ['https://sindresorhus.com'],
 };
 ```
 */
export type MemoizedCall<TArgs extends readonly unknown[]> = {
  /**
   Arguments spread into the memoized function when it runs.
   */
  readonly args: TArgs;
};

/**
 Memoized function: same settlement values as the wrapped function, with
 repeated keys answered from the cache and concurrent calls on one key
 sharing one promise.
 
 @typeParam TArgs - tuple of wrapped function argument types
 
 @typeParam TResult - wrapped function return type, awaited when thenable
 
 @example
 ```ts
 const memoized: MemoizedFunction<[string], string> = pMemoize({
   fn: fetchBody,
 },);
 const body = await memoized({ args: ['https://sindresorhus.com'], },);
 ```
 */
export type MemoizedFunction<
  TArgs extends readonly unknown[],
  TResult,
  > = (
    this: unknown,
    call: MemoizedCall<TArgs>,
  ) => Promise<TResult>;

//endregion Types

//region Memoizer

/**
 Registration state of one call: its promise (absent until the call body
 first suspends or settles) and whether the call is already over.

 @typeParam TResult - wrapped function return type, awaited when thenable
 */
type InFlightTracker<TResult> = {
  /**
   Promise of the call, assigned right after the call starts.
   */
  promise?: Promise<TResult>;
  /**
   Whether the call settled before its promise could be registered.
   */
  hasSettled: boolean;
};

/**
 Copies a function's identity onto the memoized wrapper on a best-effort
 basis, like upstream `p-memoize` 9.0.0: the copy is cosmetic, and a source
 function that makes it hard (a non-configurable own `toString`, a
 `toString` that throws, a proxy whose traps throw) is no reason to refuse
 to memoize it.

 @param to - Wrapper receiving the identity.

 @param from - Function whose identity is copied.

 @param ignoreNonConfigurable - Whether illegal property copies are skipped.

 @returns The wrapper, possibly only partly decorated when the copy failed.

 @example
 ```ts
 mimicBestEffort({ to: wrapper, from: fn, ignoreNonConfigurable: true, },);
 ```
 */
function mimicBestEffort<Target extends { readonly name: string; }>(
  {
    to,
    from,
    ignoreNonConfigurable,
  }: {
    readonly to: Target;
    readonly from: {
      readonly toString: () => string;
    };
    readonly ignoreNonConfigurable: boolean;
  },
): Target {
  try {
    return mimicFunction({
      to,
      from,
      ignoreNonConfigurable,
    },);
  }
  catch (error) {
    // The failure is cosmetic by design, so the wrapper stays as far as the
    // copy got.
    void error;
    return to;
  }
}

/**
 Wraps one promise-returning function with memoization.
 
 The wrapper reads and writes the cache exactly like upstream `p-memoize`:
 concurrent calls with one key share one promise, the shared promise is
 dropped from the in-flight map the moment it settles, fulfilled values are
 written through `shouldCache`, and rejections are never written.
 
 @param fn - Function to memoize; receives the tuple each call passes.
 
 @param options - Cache key, cache storage, and write predicate
 configuration.
 
 @returns Memoized function taking a `MemoizedCall` and settling like the
 wrapped function.
 
 @example
 ```ts
 import { pMemoize, } from '\@monochromatic-dev/upstream-fork-p-memoize';
 
 const memoized = pMemoize({
   fn: async function fetchUser(id: string): Promise<string> {
     return `user-${id}`;
   },
 },);
 await memoized({ args: ['7'], },); // runs
 await memoized({ args: ['7'], },); // cached
 ```
 */
export function pMemoize<
  const TArgs extends unknown[],
  TResult,
  CacheKeyType = TArgs[0],
  >(
  {
    fn,
    options,
  }: {
    /**
     Function to memoize; receives the tuple each call passes.
     */
    readonly fn: (...callArgs: TArgs) => TResult | PromiseLike<TResult>;
    /**
     Cache key, cache storage, and write predicate configuration.
     */
    readonly options?: MemoizeOptions<TArgs, TResult, CacheKeyType>;
  },
  ): MemoizedFunction<TArgs, TResult> {
  /**
   Default cache key: the first argument, matching upstream `p-memoize`'s
   `arguments_ => arguments_[0]`.
   
   @param args - Argument tuple of one call; its first element becomes the
   cache key.
   
   @returns First argument as the cache key.
   */
  function defaultCacheKey(
    args: TArgs,
  ): CacheKeyType {
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- upstream p-memoize keys by the first argument typed as the free `CacheKeyType`; the tuple slot can be no narrower
    return args[0] as CacheKeyType;
  }

  /**
   Effective key function, cache storage, and write predicate, defaulted
   exactly where upstream defaults them so `cache: false` survives
   destructuring. Read once, when the function is memoized, like upstream
   `p-memoize` 9.0.0: later changes to the options object have no effect.
   */
  const {
    cacheKey = defaultCacheKey,
    cache = new Map<CacheKeyType, TResult>(),
    shouldCache,
  } = options ?? {};

  /**
   Promises of calls currently in flight, keyed like the cache. Upstream
   keeps this map beside the memoizer: promises cannot live in `cache`
   itself because storages may hold only serializable values.
   */
  const promiseCache = new Map<CacheKeyType, Promise<TResult>>();

  /**
   Clear counter shared with every function over the same cache. A function
   with caching disabled reads and writes nothing, so it gets a counter of
   its own that nothing ever bumps.
   */
  const shared = isCacheEnabled(cache,)
    ? sharedCacheOf(cache,)
    : {
      generation: 0,
    };

  /**
   Drops the in-flight entry of one call, once the call is over.

   Only drops the entry when it is still this call's: a clear, or a later
   call for the same key, may have replaced it while this one was running.
   Marking the tracker settled also stops a call that settled before the
   caller could register it from being registered afterwards.

   @param key - Key of the call's in-flight entry.

   @param tracker - Registration state of the call.

   @example
   ```ts
   releaseInFlight({ key: '7', tracker, },);
   ```
   */
  function releaseInFlight(
    {
      key,
      tracker,
    }: {
      readonly key: CacheKeyType;
      readonly tracker: InFlightTracker<TResult>;
    },
  ): void {
    tracker.hasSettled = true;

    if (promiseCache.get(key,) === tracker.promise)
      promiseCache.delete(key,);
  }

  /**
   Runs one memoized call end to end: cache read, wrapped call, gated cache
   write, and in-flight cleanup.

   Every upstream `try`/`finally` becomes `try`/`catch` with the cleanup
   duplicated before each `return`, so a failure inside `fn` or a cache
   method still runs the in-flight cleanup before the call settles.

   With caching disabled the call yields one microtask before running `fn`,
   as upstream `p-memoize` 9.0.0 does: the caller registers the in-flight
   promise first, so the cleanup of a synchronously throwing `fn` always
   removes an entry that exists, and a later call with the same key
   recomputes instead of replaying the rejection.

   @param thisValue - Receiver forwarded into the wrapped function.

   @param args - Argument tuple forwarded into the wrapped function.

   @param key - Derived cache key shared by the call and its in-flight
   entry.

   @param generation - Clear counter value when the call started; a clear
   in between keeps the call from reading or writing the cleared cache.

   @param tracker - Registration state the caller fills in with this call's
   promise.

   @returns Promise settling with the cached value, or the wrapped
   function's result when it ran.

   @example
   ```ts
   await runCall({
     thisValue: undefined,
     args: ['7'],
     key: '7',
     generation: 0,
     tracker: { hasSettled: false, },
   });
   ```
   */
  async function runCall(
    {
      thisValue,
      args,
      key,
      generation,
      tracker,
    }: {
      readonly thisValue: unknown;
      readonly args: TArgs;
      readonly key: CacheKeyType;
      readonly generation: number;
      readonly tracker: InFlightTracker<TResult>;
    },
  ): Promise<TResult> {
    try {
      // Nothing above suspends when caching is disabled, so `fn` would run
      // before the caller registers this call as the in-flight one. Yielding
      // once lets the registration win, like upstream `p-memoize` 9.0.0.
      if (!isCacheEnabled(cache,))
        await Promise.resolve();

      if (isCacheEnabled(cache,)) {
        if (await cache.has(key,)) {
          /**
           Value already stored under this key; awaited so async storages
           resolve before the caller's promise does. A `has`-true read of
           `undefined` settles `undefined`, matching upstream's cast.
           */
          const cachedValue = await cache.get(key,);

          // A clear between the two reads empties the cache, so `get` may
          // hand back nothing though `has` said yes: treat that as a miss.
          if (shared.generation === generation) {
            releaseInFlight({
              key,
              tracker,
            },);
            // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- upstream `p-memoize` casts the cache-miss `undefined` possibility away the same way; `has` already reported the entry present
            return cachedValue as TResult;
          }
        }

        /**
         In-flight promise a re-entrant call started while `has` or `get`
         ran; `has` and `get` may call back into this function for the same
         key, in which case that call is already running it and joining
         anything else would mean awaiting ourselves.
         */
        const joinable = promiseCache.get(key,);

        if ((joinable !== undefined) && (joinable !== tracker.promise)) {
          /**
           Result of the call already running for this key. Awaited before
           the cleanup so the entry is not dropped while that call runs.
           */
          const joined = await joinable;
          releaseInFlight({
            key,
            tracker,
          },);
          return joined;
        }
      }

      /**
       Wrapped function result, awaited inline (not through a helper) so a
       synchronous throw from `fn` surfaces inside this `try` and runs the
       in-flight cleanup below.
       */
      const result = await fn.apply(
        thisValue,
        args,
      );

      if (isCacheEnabled(cache,)) {
        /**
         Whether the fulfilled value may be written; `shouldCache` was read
         once when the function was memoized.
         */
        const allow = shouldCache
          ? await shouldCache(
            result,
            {
              key,
              argumentsList: args,
            },
          )
          : true;

        // `shouldCache` may await, so a clear landing in that window would
        // otherwise write the result into the cache that was just cleared.
        // A write that already started is not undone.
        if (allow && (shared.generation === generation))
          await cache.set(
            key,
            result,
          );
      }

      releaseInFlight({
        key,
        tracker,
      },);
      return result;
    }
    catch (error) {
      // Rejections (and skipped writes caused by a throwing shouldCache) are
      // never cached; the in-flight entry goes with them.
      releaseInFlight({
        key,
        tracker,
      },);
      throw error;
    }
  }

  /**
   Starts one memoized call whose cache key is already derived: joins the
   in-flight call for the key when one exists, otherwise runs a new one and
   registers it.

   @param thisValue - Receiver forwarded into the wrapped function.

   @param args - Argument tuple forwarded into the wrapped function.

   @param key - Cache key derived from the tuple.

   @returns Promise of the call, shared by concurrent callers of one key.

   @example
   ```ts
   startCall({ thisValue: undefined, args: ['7'], key: '7', },);
   ```
   */
  function startCall(
    {
      thisValue,
      args,
      key,
    }: {
      readonly thisValue: unknown;
      readonly args: TArgs;
      readonly key: CacheKeyType;
    },
  ): Promise<TResult> {
    /**
     Clear counter value when this call started.
     */
    const { generation, } = shared;
    /**
     Promise of the call already in flight for this key, when one exists.
     */
    const inFlight = promiseCache.get(key,);

    if (inFlight !== undefined)
      return inFlight;

    /**
     Registration state of this call, filled in once its promise exists.
     */
    const tracker: InFlightTracker<TResult> = {
      hasSettled: false,
    };
    /**
     Promise of this call, registered below so concurrent callers join it.
     */
    const promise = runCall({
      thisValue,
      args,
      key,
      generation,
      tracker,
    },);
    tracker.promise = promise;

    // A `cache.has` that throws synchronously settles the call before this
    // point and its cleanup has already run; registering it now would keep
    // that failure as the in-flight entry for this key.
    if (!tracker.hasSettled)
      promiseCache.set(
        key,
        promise,
      );

    return promise;
  }

  /**
   Memoized call entry point: derives the key, then answers from the
   in-flight map when a call for the key is already running, otherwise
   starts one.

   @param call - Argument tuple forwarded into the wrapped function.

   @returns Promise settling with the wrapped function's result or failure;
   concurrent calls with one key receive the same promise.

   @example
   ```ts
   await memoized({ args: [input], },);
   ```
   */
  const memoized = mimicBestEffort({
    // The wrapper is named explicitly because bundlers strip expression
    // names; the wrapped `toString` marker reads this name before the source
    // function's own name replaces it, matching upstream `p-memoize`'s
    // `const memoized = function (...)` inference.
    to: Object.defineProperty(
      function memoized(
        this: unknown,
        {
          args,
        }: MemoizedCall<TArgs>,
      ): Promise<TResult> {
        try {
          return startCall({
            thisValue: this,
            args,
            // The key is derived from a copy of the tuple, recomputed on
            // every call like upstream: a key built by sorting the tuple in
            // place would otherwise change what the function receives.
            key: cacheKey([...args,],),
          },);
        }
        catch (error) {
          // `cacheKey` runs before the async part, so a throw would escape
          // synchronously while every other caller-supplied callback turns
          // its throw into a rejection; keep them consistent, like upstream
          // `p-memoize` 9.0.0. Nothing else in `startCall` can throw.
          // oxlint-disable-next-line prefer-promise-reject-errors -- the caller's own thrown value is propagated unchanged
          return Promise.reject(error,);
        }
      },
      'name',
      {
        value: 'memoized',
        configurable: true,
      },
    ),
    from: fn,
    // mutation-test-disable-next-line boolean -- the wrapper is always a fresh function whose own properties are a configurable `name` and the skipped `length`/`prototype`, so no copy can conflict; the flag's behavior is pinned by direct `mimicFunction` tests
    ignoreNonConfigurable: true,
  },);

  memoizedCacheStore.set(
    memoized,
    isCacheEnabled(cache,)
      ? {
        cache,
        promiseCache,
        shared,
      }
      : CACHE_DISABLED,
  );

  return memoized;
}

//endregion Memoizer
