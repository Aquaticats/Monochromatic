/**
 Memoizer adapters exposing one uniform call surface over this package's
 fork and upstream `p-memoize`, so the same generated workload can drive
 both.
 
 @module
 */

import pMemoizeUpstream, {
  pMemoizeClear as pMemoizeClearUpstream,
} from 'p-memoize';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import {
  pMemoize,
  pMemoizeClear,
} from '@monochromatic-dev/upstream-fork-p-memoize/ts';

import {
  type KeyMode,
  type MemoizeAdapter,
  type RunRecorders,
  type ShouldCacheMode,
  type Workload,
  type WorkloadKey,
  createRecordingCache,
} from './workload.ts';

//region Options mapping

/**
 Stringifies the argument tuple for the JSON key mode.
 
 @param args - Argument tuple of one call.
 
 @returns JSON form of the argument tuple.
 */
function stringifyArguments(args: readonly unknown[],): string {
  return JSON.stringify(args,);
}

/**
 Builds the cache-key option one key mode needs: absent for the default
 first-argument key, present for the JSON key.
 
 @param keyMode - Generated key derivation mode.
 
 @returns Option object carrying `cacheKey` only when the mode overrides
 the default.
 
 @example
 ```ts
 keyOption('json',);
 ```
 */
function keyOption(keyMode: KeyMode,): {
  readonly cacheKey?: (args: readonly unknown[],) => string;
} {
  if (keyMode === 'firstArgument')
    return {};

  return {
    cacheKey: stringifyArguments,
  };
}

/**
 Builds the write-predicate option one predicate mode needs: absent for the
 always-write default, present for deny and throw.
 
 @param mode - Generated predicate mode.
 
 @param recorders - Run recorders the predicate writes into.
 
 @returns Option object carrying `shouldCache` only when the mode overrides
 the default.
 
 @example
 ```ts
 predicateOption({ mode: 'never', recorders, });
 ```
 */
function predicateOption({
  mode,
  recorders,
}: {
  readonly mode: ShouldCacheMode;
  readonly recorders: RunRecorders;
},): {
  readonly shouldCache?: (
    value: unknown,
    context: {
      key: unknown;
      argumentsList: readonly unknown[];
    },
  ) => boolean;
} {
  if (mode === 'always')
    return {};

  /**
   Predicate-observation recorder, destructured for short chains.
   */
  const { shouldCacheCalls, } = recorders;

  return {
    shouldCache: function generatedPredicate(
      value: unknown,
      context: {
        key: unknown;
        argumentsList: readonly unknown[];
      },
    ): boolean {
      shouldCacheCalls.push(`${String(context.key,)} ${String(value,)}`,);
      if (mode === 'throw')
        throw new Error(`predicate-boom-${String(context.key,)}`);
      return false;
    },
  };
}

/**
 Builds the full generated option object both implementations receive.
 
 @param workload - Scenario fields deciding cache, key, and predicate.
 
 @param cache - Recording cache to use when the scenario enables caching.
 
 @param recorders - Run recorders the predicate writes into.
 
 @returns Option object with the cache, key, and predicate decisions folded
 in.
 
 @example
 ```ts
 generatedOptions({ workload, cache, recorders, });
 ```
 */
function generatedOptions({
  workload,
  cache,
  recorders,
}: {
  readonly workload: Workload;
  readonly cache: ReturnType<typeof createRecordingCache>['cache'];
  readonly recorders: RunRecorders;
},): Record<string, unknown> {
  return {
    cache: workload.cacheEnabled
      ? cache
      : false,
    ...keyOption(workload.keyMode,),
    ...predicateOption({
      mode: workload.shouldCacheMode,
      recorders,
    },),
  };
}

//endregion Options mapping

//region Adapters

/**
 Adapts this package's fork to the workload call surface.
 
 @param fn - Wrapped function whose calls the adapter serves.
 
 @param workload - Scenario fields deciding cache, key, and predicate.
 
 @param recorders - Run recorders shared with the workload runner.
 
 @returns Adapter driving the fork's memoized function.
 
 @example
 ```ts
 const adapter = createForkAdapter({ fn, workload, recorders, });
 ```
 */
export function createForkAdapter({
  fn,
  workload,
  recorders,
}: {
  readonly fn: (key: WorkloadKey,) => Promise<unknown>;
  readonly workload: Workload;
  readonly recorders: RunRecorders;
},): MemoizeAdapter {
  /**
   Recording cache the fork memoizer reads and writes when enabled.
   */
  const recording = createRecordingCache({ recorders, },);
  /**
   Generated option object bridging both implementations' option typings.
   */
  const options = generatedOptions({
    workload,
    cache: recording.cache,
    recorders,
  },);
  /* oxlint-disable typescript/no-unsafe-type-assertion -- the generated option record bridges the fork's `MemoizeOptions` typing and upstream's `Options` typing at this adapter seam */
  /**
   Generated options asserted at the adapter seam into the fork's option
   slot.
   */
  const forkOptions = options as never;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  /**
   Fork memoized function under test.
   */
  const memoized = pMemoize({
    fn,
    options: forkOptions,
  },);

  return {
    call: function callFork(key: WorkloadKey,): Promise<unknown> {
      return memoized({
        args: [key],
      },);
    },
    clear: function clearFork(): string {
      try {
        pMemoizeClear(memoized,);
        return '';
      }
      catch (error) {
        return caughtValueText(error,);
      }
    },
    cachedEntries: recording.contents,
  };
}

/**
 Adapts upstream `p-memoize` to the workload call surface.
 
 @param fn - Wrapped function whose calls the adapter serves.
 
 @param workload - Scenario fields deciding cache, key, and predicate.
 
 @param recorders - Run recorders shared with the workload runner.
 
 @returns Adapter driving an upstream memoized function.
 
 @example
 ```ts
 const adapter = createUpstreamAdapter({ fn, workload, recorders, });
 ```
 */
export function createUpstreamAdapter({
  fn,
  workload,
  recorders,
}: {
  readonly fn: (key: WorkloadKey,) => Promise<unknown>;
  readonly workload: Workload;
  readonly recorders: RunRecorders;
},): MemoizeAdapter {
  /**
   Recording cache the upstream memoizer reads and writes when enabled.
   */
  const recording = createRecordingCache({ recorders, },);
  /**
   Generated option object bridging both implementations' option typings.
   */
  const options = generatedOptions({
    workload,
    cache: recording.cache,
    recorders,
  },);
  /* oxlint-disable typescript/no-unsafe-type-assertion -- the generated option record bridges upstream's `Options` typing and the fork's `MemoizeOptions` typing at this adapter seam */
  /**
   Generated options asserted at the adapter seam into upstream's option
   slot.
   */
  const upstreamOptions = options as never;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  /**
   Upstream memoized function under comparison.
   */
  const memoized = pMemoizeUpstream(
    fn,
    upstreamOptions,
  );

  return {
    call: function callUpstream(key: WorkloadKey,): Promise<unknown> {
      return memoized(key,);
    },
    clear: function clearUpstream(): string {
      try {
        pMemoizeClearUpstream(memoized,);
        return '';
      }
      catch (error) {
        return caughtValueText(error,);
      }
    },
    cachedEntries: recording.contents,
  };
}

//endregion Adapters
