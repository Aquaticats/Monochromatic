/**
 Workload model shared by the property files: one generated memoization
 scenario, run identically against this package's fork and upstream
 `p-memoize`, producing traces both implementations must agree on.
 
 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

//region Types

/**
 Domain of cache keys a workload passes; small on purpose so calls collide
 on keys and exercise in-flight sharing.
 */
export type WorkloadKey = 'a' | 'b' | 'c';

/**
 Behavior of the wrapped function for one key: resolve with its invocation
 ordinal, reject with a named failure, or throw synchronously before
 returning a promise.
 */
export type KeyBehavior = 'resolve' | 'reject' | 'throwSync';

/**
 How the cache key derives from the argument tuple.
 */
export type KeyMode = 'firstArgument' | 'json';

/**
 How the write predicate decides: allow every write, deny every write, or
 throw.
 */
export type ShouldCacheMode = 'always' | 'never' | 'throw';

/**
 One generated memoization scenario.
 */
export type Workload = {
  /**
   Wrapped-function behavior per key.
   */
  readonly behaviorByKey: Readonly<Record<WorkloadKey, KeyBehavior>>;
  /**
   Cache key derivation mode.
   */
  readonly keyMode: KeyMode;
  /**
   Write predicate mode.
   */
  readonly shouldCacheMode: ShouldCacheMode;
  /**
   Whether a cache storage backs the memoizer; `false` disables caching.
   */
  readonly cacheEnabled: boolean;
  /**
   Call batches; each batch launches all its calls synchronously before any
   of them settles, so in-flight sharing is observable.
   */
  readonly batches: readonly (readonly WorkloadKey[])[];
  /**
   Whether `pMemoizeClear` runs after each batch, by batch index.
   */
  readonly clearAfterBatch: readonly boolean[];
};

/**
 One call's observable outcome.
 */
export type CallOutcome = {
  /**
   Index of the first call whose promise object this call received; equal to
   this call's own index when its promise is unique.
   */
  readonly sharedWith: number;
  /**
   Whether the promise fulfilled or rejected.
   */
  readonly status: 'fulfilled' | 'rejected';
  /**
   Fulfilled invocation ordinal, or the rejection message.
   */
  readonly detail: number | string;
};

/**
 Observable trace of one workload run against one implementation.
 */
export type WorkloadTrace = {
  /**
   Per-call outcomes in scheduling order.
   */
  readonly outcomes: readonly CallOutcome[];
  /**
   Invocation ordinals in wrapped-function call order; ordinals skip cache
   hits, so the sequence proves which calls recomputed.
   */
  readonly invocations: readonly number[];
  /**
   Rejection messages in wrapped-function call order.
   */
  readonly failures: readonly string[];
  /**
   Cache-storage method calls as `method key value` strings, in order.
   */
  readonly cacheCalls: readonly string[];
  /**
   Final cache contents as sorted `key=value` strings.
   */
  readonly cacheContents: readonly string[];
  /**
   Write-predicate observations as `key value` strings, in order.
   */
  readonly shouldCacheCalls: readonly string[];
  /**
   Outcome text per `pMemoizeClear` run: empty on success, the failure
   message when the implementation's clearer refused.
   */
  readonly clearResults: readonly string[];
};

/**
 One memoized callable normalized over both implementations' call shapes.
 */
export type MemoizeAdapter = {
  /**
   Calls the memoized function with one key and returns its promise.
   */
  readonly call: (key: WorkloadKey,) => Promise<unknown>;
  /**
   Drops every cached value through the implementation's `pMemoizeClear`.
   
   @returns Empty string on success, failure message when refused.
   */
  readonly clear: () => string;
  /**
   Final cache contents as sorted `key=value` strings.
   */
  readonly cachedEntries: () => string[];
};

/**
 Recorders the wrapped function and the write predicate write into during
 one run.
 */
export type RunRecorders = {
  /**
   Invocation ordinals in wrapped-function call order.
   */
  readonly invocations: number[];
  /**
   Rejection messages in wrapped-function call order.
   */
  readonly failures: string[];
  /**
   Cache-storage method calls as `method key value` strings.
   */
  readonly cacheCalls: string[];
  /**
   Write-predicate observations as `key value` strings.
   */
  readonly shouldCacheCalls: string[];
};

//endregion Types

//region Fixtures

/**
 Failure message prefix for `reject` and `throwSync` behaviors.
 */
const FAILURE_PREFIX = 'boom-';

/**
 Builds the failure a rejecting or throwing wrapped function produces for
 one key, so both implementations surface identical messages.
 
 @param key - Cache key whose behavior fails.
 
 @returns Failure carrying the key in its message.
 
 @example
 ```ts
 failureFor('a',).message; // => 'boom-a'
 ```
 */
export function failureFor(key: WorkloadKey,): Error {
  return new Error(`${FAILURE_PREFIX}${key}`,);
}

/**
 Builds the wrapped function one run executes: dispatching each key's
 generated behavior and recording every invocation.
 
 @param workload - Scenario whose per-key behaviors drive the function.
 
 @param recorders - Run recorders the function writes into.
 
 @returns Wrapped function handed to the memoizer under test.
 
 @example
 ```ts
 const fn = createScriptedFunction({ workload, recorders, });
 ```
 */
export function createScriptedFunction({
  workload,
  recorders,
}: {
  readonly workload: Workload;
  readonly recorders: RunRecorders;
},): (key: WorkloadKey,) => Promise<unknown> {
  /**
   Next invocation ordinal; also the resolved value of `resolve` calls.
   */
  const ordinalState = {
    next: 0,
  };
  /**
   Run recorders this script writes into, destructured for short chains.
   */
  const {
    invocations,
    failures,
  } = recorders;

  /**
   Runs one key's generated behavior, counting the invocation first so
   synchronous throws are counted too.
   */
  return function scripted(key: WorkloadKey,): Promise<unknown> {
    /**
     Ordinal of this invocation.
     */
    const ordinal = ordinalState.next;
    ordinalState.next += 1;
    invocations.push(ordinal,);

    /**
     Behavior map of the scenario.
     */
    const { behaviorByKey, } = workload;
    /**
     Behavior this key's calls take.
     */
    const behavior = behaviorByKey[key];

    if (behavior === 'throwSync')
      throw failureFor(key,);

    if (behavior === 'reject') {
      /**
       Failure message both implementations must surface for this key.
       */
      const { message, } = failureFor(key,);
      failures.push(message,);
      return Promise.reject(new Error(message,),);
    }

    return Promise.resolve(ordinal,);
  };
}

/**
 Builds a cache storage that records every method call and keeps its
 entries readable, so traces include the cache contract and not just
 results.
 
 @param recorders - Run recorders the storage writes into.
 
 @returns Storage to hand to the memoizer under test, plus its final
 contents projection.
 
 @example
 ```ts
 const { cache, contents, } = createRecordingCache(recorders,);
 ```
 */
export function createRecordingCache({
  recorders,
}: {
  readonly recorders: RunRecorders;
},): {
  readonly cache: {
    has: (key: unknown,) => Promise<boolean>;
    get: (key: unknown,) => Promise<unknown>;
    set: (
      key: unknown,
      value: unknown,
    ) => Promise<void>;
    delete: (key: unknown,) => boolean;
    clear: () => void;
  };
  readonly contents: () => string[];
} {
  /**
   Backing entries behind the recording storage.
   */
  const entries = new Map<unknown, unknown>();
  /**
   Cache-call recorder, destructured for short chains.
   */
  const { cacheCalls, } = recorders;

  /**
   Projects the backing entries as sorted `key=value` strings.
   
   @returns Sorted entry texts.
   */
  function contents(): string[] {
    return [
      ...entries.entries(),
    ]
      .map(function formatEntry([
        key,
        value,
      ],): string {
        return `${String(key,)}=${String(value,)}`;
      },)
      .toSorted();
  }

  /**
   Records and answers a presence check.
   
   @param key - Cache key to check.
   
   @returns Resolved presence flag.
   */
  async function has(key: unknown,): Promise<boolean> {
    cacheCalls.push(`has ${String(key,)}`,);
    return await Promise.resolve(entries.has(key,),);
  }

  /**
   Records and answers a read.
   
   @param key - Cache key to read.
   
   @returns Resolved stored value, or `undefined` for a miss.
   */
  async function get(key: unknown,): Promise<unknown> {
    cacheCalls.push(`get ${String(key,)}`,);
    return await Promise.resolve(entries.get(key,),);
  }

  /**
   Records and performs a delete.
   
   @param key - Cache key to delete.
   
   @returns Whether an entry was removed.
   */
  function remove(key: unknown,): boolean {
    cacheCalls.push(`delete ${String(key,)}`,);
    return entries.delete(key,);
  }

  /**
   Records and performs a full clear.
   */
  function clear(): void {
    cacheCalls.push('clear',);
    entries.clear();
  }

  return {
    contents,
    cache: {
      has,
      get,
      set: async function set(
        key: unknown,
        value: unknown,
      ): Promise<void> {
        cacheCalls.push(`set ${String(key,)} ${String(value,)}`,);
        entries.set(
          key,
          value,
        );
        await Promise.resolve();
      },
      delete: remove,
      clear,
    },
  };
}

//endregion Fixtures

//region Runner

/**
 Projects one settled result into its comparable detail: the invocation
 ordinal for fulfillments, the failure text for rejections.
 
 @param result - Settled result of one launched call.
 
 @returns Ordinal or failure text both implementations must agree on.
 
 @example
 ```ts
 settledDetail({ status: 'fulfilled', value: 3, },); // => 3
 ```
 */
function settledDetail(result: PromiseSettledResult<unknown>,): number | string {
  if (result.status === 'rejected')
    return caughtValueText(result.reason,);

  if ((typeof result.value) === 'number')
    return result.value;

  return String(result.value,);
}

/**
 Runs one workload against one implementation and records its trace.
 
 Batches run strictly in order: each batch launches its calls synchronously,
 settles them, and optionally clears, exactly as the scenario describes.
 
 @param createAdapter - Adapter factory for the implementation under test.
 
 @param workload - Scenario to run.
 
 @returns Trace of call outcomes, invocations, failures, and cache traffic.
 
 @example
 ```ts
 const trace = await runWorkload({
   createAdapter,
   workload,
 });
 ```
 */
export async function runWorkload({
  createAdapter,
  workload,
}: {
  readonly createAdapter: (spec: {
    readonly fn: (key: WorkloadKey,) => Promise<unknown>;
    readonly workload: Workload;
    readonly recorders: RunRecorders;
  },) => MemoizeAdapter;
  readonly workload: Workload;
},): Promise<WorkloadTrace> {
  /**
   Run recorders shared by the wrapped function, the predicate, and the
   cache storage.
   */
  const recorders: RunRecorders = {
    invocations: [],
    failures: [],
    cacheCalls: [],
    shouldCacheCalls: [],
  };
  /**
   Adapter driving the implementation under test.
   */
  const adapter = createAdapter({
    fn: createScriptedFunction({
      workload,
      recorders,
    },),
    workload,
    recorders,
  });
  /**
   Promise object to first-call index, so shared promises are visible.
   */
  const promiseOwners = new Map<unknown, number>();
  /**
   Per-call outcomes in scheduling order.
   */
  const outcomes: CallOutcome[] = [];
  /**
   Outcome text per clear run.
   */
  const clearResults: string[] = [];
  /**
   Running index across batches, used as each promise's owner identity.
   */
  const callIndexState = {
    next: 0,
  };

  /**
   Clear flags of the scenario, destructured for short chains.
   */
  const { clearAfterBatch, } = workload;
  /**
   Batches of the scenario, destructured for short chains.
   */
  const { batches, } = workload;

  /**
   Launches one batch's calls synchronously, settles them, and optionally
   clears the cache; folds over batches so their order is the only possible
   order.
   */
  await batches.reduce(
    async function runBatch(
      settledSoFar: Promise<void>,
      batch: readonly WorkloadKey[],
      batchIndex: number,
    ): Promise<void> {
      await settledSoFar;

      /**
       Promises launched together in this batch.
       */
      const launched: Promise<unknown>[] = [];
      /**
       Owner index per launched position, captured before any awaiting.
       */
      const owners: number[] = [];

      for (const key of batch) {
        /**
         Promise this call received, before any awaiting so shared objects
         are detectable.
         */
        const promise = adapter.call(key,);
        /**
         Index of this call across the whole run.
         */
        const callIndex = callIndexState.next;
        callIndexState.next += 1;
        /**
         Index of the first call holding the same promise object.
         */
        const owner = promiseOwners.get(promise,);

        if (owner === undefined)
          promiseOwners.set(
            promise,
            callIndex,
          );

        launched.push(promise,);
        owners.push(owner ?? callIndex,);
      }

      /**
       Settlement of every launched call, in batch order.
       */
      const settled = await Promise.allSettled(launched,);
      /**
       Outcomes of this batch's calls, mapped in batch order.
       */
      const batchOutcomes = settled.map(function toOutcome(
        result: PromiseSettledResult<unknown>,
        position: number,
      ): CallOutcome {
        return {
          sharedWith: owners[position] ?? position,
          status: result.status === 'fulfilled'
            ? 'fulfilled'
            : 'rejected',
          detail: settledDetail(result,),
        };
      },);
      outcomes.push(...batchOutcomes,);

      if (clearAfterBatch[batchIndex] === true)
        clearResults.push(adapter.clear(),);
    },
    Promise.resolve(),
  );

  return {
    outcomes,
    invocations: recorders.invocations,
    failures: recorders.failures,
    cacheCalls: recorders.cacheCalls,
    shouldCacheCalls: recorders.shouldCacheCalls,
    cacheContents: adapter.cachedEntries(),
    clearResults,
  };
}

//endregion Runner
