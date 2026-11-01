/**
 Test-only helpers shared by this package's unit tests.
 
 Deferreds and gates make call timing deterministic: each wrapped call
 blocks on its own gate and the test releases gates in a chosen order, so
 in-flight overlap and settle order are decided by the test instead of by
 scheduler timing. The spy cache records every storage call so tests can
 assert the cache contract, not just its final contents.
 
 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import type {
  CacheStorage,
} from '../dist/final/neutral/index.mjs';

//region Deferreds and gates

/**
 One deferred promise plus its settlement functions.
 
 @typeParam T - settled value type
 
 @example
 ```ts
 const deferred = createDeferred<boolean>();
 deferred.resolve(true,);
 ```
 */
export type Deferred<T> = PromiseWithResolvers<T>;

/**
 Creates one deferred promise pair.
 
 @returns Deferred whose promise settles when `resolve` or `reject` runs.
 
 @example
 ```ts
 const deferred = createDeferred<number>();
 deferred.resolve(1,);
 await deferred.promise;
 ```
 */
export function createDeferred<T>(): Deferred<T> {
  return Promise.withResolvers<T>();
}

/**
 One controllable completion gate: a blocked call awaits `open`, the test
 finishes it with `release`.
 
 @example
 ```ts
 const gate = createGate();
 await gate.open;
 ```
 */
export type Gate = {
  /**
   Promise settled by {@link Gate.release}.
   */
  readonly open: Promise<void>;
  /**
   Settles {@link Gate.open}, letting the blocked call continue.
   */
  readonly release: () => void;
};

/**
 Creates one openable gate.
 
 @returns Gate whose promise is pending until released.
 
 @example
 ```ts
 const gate = createGate();
 gate.release();
 await gate.open;
 ```
 */
export function createGate(): Gate {
  /**
   Deferred pair backing the gate's promise and its release.
   */
  const deferred = Promise.withResolvers<void>();
  return {
    open: deferred.promise,
    release: function release(): void {
      deferred.resolve();
    },
  };
}

//endregion Deferreds and gates

//region Turns and promise state

/**
 Yields one macrotask turn.
 
 A `setTimeout` turn runs after every queued microtask, so awaiting this
 guarantees all pending continuations of earlier settles have run.
 
 @example
 ```ts
 await yieldTurn();
 ```
 */
export async function yieldTurn(): Promise<void> {
  await wait(
    0,
  );
}

/**
 Observable settlement state of a promise, mirroring `p-state`'s result
 shape used by upstream `p-memoize`'s tests.
 */
export type PromiseState = 'pending' | 'fulfilled' | 'rejected';

/**
 One promise observation: its state plus the rejection reason when it
 rejected.
 
 @example
 ```ts
 const observation: PromiseObservation = {
   state: 'pending',
 };
 ```
 */
export type PromiseObservation = {
  /**
   Settlement state observed at the end of the observation window.
   */
  readonly state: PromiseState;
  /**
   Rejection reason, present only when {@link PromiseObservation.state} is
   `rejected`.
   */
  readonly reason?: unknown;
};

/**
 Observes one promise without ever rejecting, resolving the verdict with
 whichever comes first: settlement, or a full macrotask turn.
 
 @param verdict - Deferred receiving the first-of observation.
 
 @param promise - Promise to observe.
 
 @example
 ```ts
 const verdict = Promise.withResolvers<PromiseObservation>();
 void observePromiseState({
   verdict,
   promise,
 });
 ```
 */
async function observePromiseState(
  {
    verdict,
    promise,
  }: {
    readonly verdict: PromiseWithResolvers<PromiseObservation>;
    readonly promise: PromiseLike<unknown>;
  },
): Promise<void> {
  try {
    await promise;
    verdict.resolve({
      state: 'fulfilled',
    },);
  }
  catch (error) {
    // The rejection is the observation itself, so it is consumed here and
    // reported through the verdict instead of rethrown.
    verdict.resolve({
      state: 'rejected',
      reason: error,
    },);
  }
}

/**
 Reports whether a promise is still pending after one macrotask turn, or
 how it settled.
 
 @param promise - Promise whose state is observed.
 
 @returns Settlement state observed at the end of the turn.
 
 @example
 ```ts
 expect(await promiseState(promise,),).toBe('pending',);
 ```
 */
export async function promiseState(promise: PromiseLike<unknown>,): Promise<PromiseState> {
  /**
   First-of verdict between the promise settling and the turn ending.
   */
  const verdict = Promise.withResolvers<PromiseObservation>();

  setTimeout(
    function markStillPending(): void {
      verdict.resolve({
        state: 'pending',
      },);
    },
    0,
  );
  void observePromiseState({
    verdict,
    promise,
  },);

  /**
   Observation from the first-of verdict.
   */
  const observation = await verdict.promise;
  return observation.state;
}

/**
 Awaits a promise expected to reject and hands back its reason.

 @param promise - Promise the test expects to reject.

 @returns The rejection reason.

 @throws Error when the promise fulfills instead.

 @example
 ```ts
 const reason = await rejectionOf(Promise.reject(new Error('boom',),),);
 ```
 */
export async function rejectionOf(promise: PromiseLike<unknown>,): Promise<unknown> {
  try {
    await promise;
  }
  catch (error) {
    return error;
  }

  throw new Error('expected the promise to reject, but it fulfilled',);
}

//endregion Turns and promise state

//region Spy cache

/**
 One recorded cache-storage method call.
 */
export type SpyCacheCall = {
  /**
   Storage method that was called.
   */
  readonly method: 'has' | 'get' | 'set' | 'delete' | 'clear';
  /**
   Key passed to the method, absent for `clear`.
   */
  readonly key?: unknown;
  /**
   Value passed to `set`, absent for every other method.
   */
  readonly value?: unknown;
};

/**
 Spy cache storage: an in-memory `Map` that records `has`, `get`, `set`,
 `delete`, and `clear` calls in order. `has`, `get`, and `set` return
 promises like an asynchronous cache, while `delete` and `clear` stay
 synchronous like `Map`'s own methods.
 
 @typeParam KeyType - cache key type the storage accepts
 
 @typeParam ValueType - stored value type
 
 @example
 ```ts
 const spy = createSpyCache<string, number>();
 await spy.cache.set('k', 1,);
 expect(spy.calls.length,).toBe(1,);
 ```
 */
export type SpyCache<KeyType, ValueType> = {
  /**
   Storage to hand to a memoizer.
   */
  readonly cache: CacheStorage<KeyType, ValueType> & {
    /**
     Removes one entry; always present on the spy.
     */
    readonly delete: (key: KeyType) => unknown;
    /**
     Drops every entry; always present on the spy.
     */
    readonly clear: () => unknown;
  };
  /**
   Recorded calls in invocation order.
   */
  readonly calls: SpyCacheCall[];
  /**
   Backing entries, readable for content assertions.
   */
  readonly entries: Map<KeyType, ValueType>;
};

/**
 Creates one recording cache storage.
 
 @typeParam KeyType - cache key type the storage accepts
 
 @typeParam ValueType - stored value type
 
 @returns Spy cache whose `cache` records every method call.
 
 @example
 ```ts
 const spy = createSpyCache<string, number>();
 ```
 */
export function createSpyCache<KeyType, ValueType>(): SpyCache<KeyType, ValueType> {
  /**
   Backing store behind the spy's methods.
   */
  const entries = new Map<KeyType, ValueType>();
  /**
   Recorded calls in invocation order.
   */
  const calls: SpyCacheCall[] = [];

  return {
    entries,
    calls,
    cache: {
      has: function spyHas(key: KeyType,): Promise<boolean> {
        calls.push({
          method: 'has',
          key,
        },);
        return Promise.resolve(entries.has(key,),);
      },
      // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors `CacheStorage.get` documented cache-miss `undefined` value
      get: function spyGet(key: KeyType,): Promise<ValueType | undefined> {
        calls.push({
          method: 'get',
          key,
        },);
        return Promise.resolve(entries.get(key,),);
      },
      set: function spySet(
        key: KeyType,
        value: ValueType,
      ): Promise<void> {
        calls.push({
          method: 'set',
          key,
          value,
        },);
        entries.set(
          key,
          value,
        );
        return Promise.resolve();
      },
      delete: function spyDelete(key: KeyType,): boolean {
        calls.push({
          method: 'delete',
          key,
        },);
        return entries.delete(key,);
      },
      clear: function spyClear(): void {
        calls.push({
          method: 'clear',
        },);
        entries.clear();
      },
    },
  };
}

//endregion Spy cache
