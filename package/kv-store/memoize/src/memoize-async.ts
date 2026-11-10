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
 In-flight computation registered in the deduplication map.
 
 The token identifies this computation among all computations for the same cache key: a
 wipe invalidates the token instead of the map entry, so the computation it belongs to
 stays invalidated after the map entry is gone.
 */
type InflightEntry<TReturn> = {
  readonly promise: Promise<TReturn>;
  readonly token: symbol;
};

/**
 Store write already started by an in-flight computation, tracked so a wipe can await it.
 */
type PendingWrite = {
  readonly cacheKey: string;
  readonly promise: Promise<unknown>;
};

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
 
 `clear` and `delete` fence computations that are already running: those still resolve for
 their own caller, but their results are not cached, and each method waits for cache writes
 already in progress before wiping, so a pre-wipe value never reappears afterwards.
 
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
   In-flight computations keyed by cache key, deduplicating concurrent calls onto one
   computation. Each entry carries the token a concurrent wipe invalidates.
   */
  const inflight = new Map<string, InflightEntry<TReturn>>();

  /**
   Tokens of computations a wipe invalidated. A computation whose token is here still
   resolves for its own caller but never writes its result to the store.
   */
  const invalidatedTokens = new Set<symbol>();

  /**
   Store writes already started; a wipe awaits the writes it is about to invalidate.
   */
  const pendingWrites = new Set<PendingWrite>();

  /**
   Create a disposable that retires the token and the inflight registration of a
   computation on dispose.
   
   The registration is removed only while this computation still owns it, so an
   invalidated computation cannot drop the entry a later call registered for that key.
   
   @param cacheKey - key whose registration is removed on disposal
   
   @param token - token identifying this computation among the key's registrations
   
   @returns disposable that retires the token and inflight entry on scope exit
   
   @example
   ```ts
   using _guard = inflightGuard({ cacheKey: 'my-key', token });
   ```
   */
  function inflightGuard({
    cacheKey,
    token,
  }: Readonly<{
    cacheKey: string;
    token: symbol;
  }>,): Disposable {
    return {
      [Symbol.dispose](): void {
        invalidatedTokens.delete(token,);
        /**
         Registration still owned by this computation, if no wipe or later call replaced it.
         */
        const registered = inflight.get(cacheKey,);
        if (registered?.token === token)
          inflight.delete(cacheKey,);
      },
    };
  }

  /**
   Create a disposable that stops tracking a completed store write on dispose.
   
   @param write - write removed from the pending set on disposal
   
   @returns disposable that drops the pending write on scope exit
   
   @example
   ```ts
   using _guard = pendingWriteGuard(write);
   ```
   */
  function pendingWriteGuard(write: PendingWrite,): Disposable {
    return {
      [Symbol.dispose](): void {
        pendingWrites.delete(write,);
      },
    };
  }

  /**
   Persist a computed value, tracking the write so a concurrent wipe waits for it.
   
   The write is registered in the same synchronous step that starts it, so a wipe that
   begins while the store is still writing sees the write and clears the store only after
   it has landed.
   
   @param cacheKey - full cache key to write
   
   @param value - freshly computed value
   */
  async function persist({
    cacheKey,
    value,
  }: Readonly<{
    cacheKey: string;
    value: TReturn;
  }>,): Promise<void> {
    /**
     Store write, registered for wipe tracking before the first await.
     */
    const write: PendingWrite = {
      cacheKey,
      promise: store.set(
        cacheKey,
        value,
      ),
    };
    pendingWrites.add(write,);
    /**
     Auto-disposer that stops tracking the settled write on scope exit, even on throw.
     */
    using _guard = pendingWriteGuard(write,);
    await write.promise;
  }

  /**
   Await store writes that a wipe is about to invalidate.
   
   A rejected write stays reported to the call it belongs to; the wipe only needs it settled,
   so one failure does not prevent the remaining writes from being awaited.
   
   @param writes - writes captured before the wipe
   */
  async function settleWrites(writes: readonly PendingWrite[],): Promise<void> {
    await Promise.allSettled(
      writes.map(function pendingPromise(write,) {
        return write.promise;
      },),
    );
  }

  /**
   Core computation: check the store, then call `fn`, clearing the inflight entry on exit.
   
   @param cacheKey - full cache key
   
   @param args - original function arguments
   
   @param token - identity a wipe invalidates while `fn` runs
   
   @returns cached or freshly computed result
   
   @throws whatever `fn` rejects with; `using` clears the inflight entry before the throw escapes
   */
  async function resolveValue({
    cacheKey,
    args,
    token,
  }: Readonly<{
    cacheKey: string;
    args: TArgs;
    token: symbol;
  }>,): Promise<TReturn> {
    /**
     Auto-disposer that clears the inflight entry on scope exit, even on throw.
     */
    using _guard = inflightGuard({
      cacheKey,
      token,
    },);

    /**
     Cached value if present; `ABSENT` signals a miss and triggers recomputation.
     */
    const stored = await store.get<TReturn>(cacheKey,);
    if (stored !== ABSENT)
      return stored;

    /**
     Freshly computed value, persisted only while no concurrent wipe invalidated its token.
     */
    const result = await fn(...args,);
    if (!invalidatedTokens.has(token,))
      await persist({
        cacheKey,
        value: result,
      },);
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
     In-flight entry for this key, if any; returning its promise dedupes concurrent callers.
     */
    const existing = inflight.get(cacheKey,);
    if (existing !== undefined)
      return existing.promise;

    /**
     Identity a wipe invalidates when it reaches this computation, unique across every call.
     */
    const token = Symbol('memoize async in-flight computation',);
    /**
     Newly started computation, registered in `inflight` so concurrent callers share it.
     */
    const promise = resolveValue({
      cacheKey,
      args,
      token,
    },);
    inflight.set(
      cacheKey,
      {
        promise,
        token,
      },
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
    /**
     Computations still running, invalidated before the map is emptied; their tokens carry
     the invalidation, so a value that resolves afterwards is returned but never cached.
     */
    for (const entry of inflight.values())
      invalidatedTokens.add(entry.token,);
    inflight.clear();
    /**
     Writes that already left their computation, awaited so none lands after the wipe.
     */
    await settleWrites([...pendingWrites,],);
    await store.clear();
  };

  memoized.delete = async function deleteCacheEntry(key: string,): Promise<void> {
    /**
     Computation still running for this key; the deleted value must not be restored by it.
     */
    const invalidated = inflight.get(key,);
    if (invalidated !== undefined) {
      invalidatedTokens.add(invalidated.token,);
      inflight.delete(key,);
    }
    /**
     Writes already started for this key, awaited so none lands after the delete.
     */
    await settleWrites(
      [...pendingWrites,].filter(function belongsToKey(write,) {
        return write.cacheKey === key;
      },),
    );
    await store.delete(key,);
  };

  return memoized as MemoizedAsyncFunction<TArgs, TReturn>;
}
