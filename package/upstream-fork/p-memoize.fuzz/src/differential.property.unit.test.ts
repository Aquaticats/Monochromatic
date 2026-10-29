/**
 Differential property tests: the fork must produce the same observable
 traces as the installed upstream `p-memoize` (the floating catalog
 entry, currently 9.0.0) on the same generated workload.
 
 The oracle is upstream itself: cache keying, in-flight promise sharing,
 cache-contract traffic, write-predicate gating, clearing, and the
 in-flight handling of synchronous throws are all checked against the
 upstream package this fork was derived from.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import pMemoizeUpstream, {
  pMemoizeClear as pMemoizeClearUpstream,
} from 'p-memoize';
import {
  assert,
  asyncProperty,
} from 'fast-check';

import {
  pMemoize,
  pMemoizeClear,
} from '@monochromatic-dev/upstream-fork-p-memoize/ts';

import { fuzzRuns, } from './fuzz-budget.ts';
import { workloadArb, } from './memoize-arbitrary.ts';
import {
  createForkAdapter,
  createUpstreamAdapter,
} from './memoizer-adapters.ts';
import {
  runWorkload,
  type WorkloadTrace,
} from './workload.ts';

//region Helpers

/**
 Projects one trace into its comparable shape: every recorded surface must
 match across implementations, so no normalization is needed beyond the
 trace itself.
 
 @param trace - Trace from one workload run.
 
 @returns Comparable projection of the trace.
 
 @example
 ```ts
 expect(comparable(forkTrace,),).toEqual(comparable(upstreamTrace,),);
 ```
 */
function comparable(trace: WorkloadTrace,): Record<string, unknown> {
  return {
    outcomes: trace.outcomes.map(function projectOutcome(outcome,): Record<string, unknown> {
      return {
        sharedWith: outcome.sharedWith,
        status: outcome.status,
        detail: outcome.detail,
      };
    },),
    invocations: [...trace.invocations,],
    failures: [...trace.failures,],
    cacheCalls: [...trace.cacheCalls,],
    cacheContents: [...trace.cacheContents,],
    shouldCacheCalls: [...trace.shouldCacheCalls,],
    clearResults: [...trace.clearResults,],
  };
}

//endregion Helpers

await describe({
  name: 'upstream p-memoize parity',
  children: [
    it({
      name: 'memoization traces match upstream p-memoize on every generated workload',
      fn: async () => {
        await assert(
          asyncProperty(workloadArb, async (workload,) => {
            /**
             Trace of the fork run.
             */
            const forkTrace = await runWorkload({
              createAdapter: createForkAdapter,
              workload,
            },);
            /**
             Trace of the upstream run over the same workload.
             */
            const upstreamTrace = await runWorkload({
              createAdapter: createUpstreamAdapter,
              workload,
            },);
            expect(comparable(forkTrace,),).toEqual(comparable(upstreamTrace,),);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'pMemoizeClear error messages match upstream p-memoize',
      fn: async () => {
        /**
         Wrapped function never memoized by either implementation.
         */
        async function plain(): Promise<number> {
          return 1;
        }
        /**
         Error message the fork threw, if any.
         */
        let forkMessage = '';
        try {
          pMemoizeClear(plain,);
        }
        catch (error) {
          forkMessage = (error as Error).message;
        }
        /**
         Error message the upstream clearer threw, if any.
         */
        let upstreamMessage = '';
        try {
          pMemoizeClearUpstream(plain,);
        }
        catch (error) {
          upstreamMessage = (error as Error).message;
        }
        expect(forkMessage,).toBe(upstreamMessage,);
        expect(forkMessage,).toBe('Can\'t clear a function that was not memoized!',);
      },
    },),

    it({
      name: 'cached-disabled and unclearable-cache clear errors match upstream p-memoize',
      fn: async () => {
        /**
         Wrapped function memoized with caching disabled.
         */
        async function uncachedTarget(key: string,): Promise<string> {
          return key;
        }
        /**
         Fork memoized function with caching disabled.
         */
        const forkUncached = pMemoize({
          fn: uncachedTarget,
          options: {
            cache: false,
          },
        },);
        /**
         Upstream memoized function with caching disabled.
         */
        const upstreamUncached = pMemoizeUpstream(uncachedTarget, {
          cache: false,
        },);

        /**
         Error message the fork threw, if any.
         */
        let forkMessage = '';
        try {
          pMemoizeClear(forkUncached,);
        }
        catch (error) {
          forkMessage = (error as Error).message;
        }
        /**
         Error message the upstream clearer threw, if any.
         */
        let upstreamMessage = '';
        try {
          pMemoizeClearUpstream(upstreamUncached,);
        }
        catch (error) {
          upstreamMessage = (error as Error).message;
        }
        expect(forkMessage,).toBe(upstreamMessage,);
        expect(forkMessage,).toBe('Can\'t clear a function that doesn\'t use a cache!',);

        /**
         Wrapped function memoized over a cache without `clear`.
         */
        async function weakTarget(key: object,): Promise<object> {
          return key;
        }
        /**
         Identity cache key shared by both implementations' WeakMap caches.
         */
        function weakKey([firstArgument]: readonly unknown[],): object {
          return firstArgument as object;
        }
        /**
         Fork memoized function over a WeakMap cache.
         */
        const forkWeak = pMemoize({
          fn: weakTarget,
          options: {
            cache: new WeakMap<object, object>(),
            cacheKey: weakKey,
          },
        },);
        /**
         Upstream memoized function over a WeakMap cache.
         */
        const upstreamWeak = pMemoizeUpstream(weakTarget, {
          cache: new WeakMap<object, object>(),
          cacheKey: weakKey,
        },);

        forkMessage = '';
        try {
          pMemoizeClear(forkWeak,);
        }
        catch (error) {
          forkMessage = (error as Error).message;
        }
        upstreamMessage = '';
        try {
          pMemoizeClearUpstream(upstreamWeak,);
        }
        catch (error) {
          upstreamMessage = (error as Error).message;
        }
        expect(forkMessage,).toBe(upstreamMessage,);
        expect(forkMessage,).toBe('The cache can\'t be cleared!',);
      },
    },),

    it({
      name: 'synchronous-throw in-flight sharing and rejection messages match upstream p-memoize',
      fn: async () => {
        /**
         Wrapped function throwing synchronously on every call.
         */
        function syncThrower(_key: string,): Promise<string> {
          throw new Error('sync-boom',);
        }
        /**
         Fork memoized function with caching disabled.
         */
        const forkMemoized = pMemoize({
          fn: syncThrower,
          options: {
            cache: false,
          },
        },);
        /**
         Upstream memoized function with caching disabled.
         */
        const upstreamMemoized = pMemoizeUpstream(syncThrower, {
          cache: false,
        },);

        /**
         First fork call's promise.
         */
        const forkFirst = forkMemoized({
          args: ['k'],
        },);
        /**
         Second fork call must join the first's in-flight entry.
         */
        const forkSecond = forkMemoized({
          args: ['k'],
        },);
        /**
         First upstream call's promise.
         */
        const upstreamFirst = upstreamMemoized('k',);
        /**
         Second upstream call must join the first's in-flight entry.
         */
        const upstreamSecond = upstreamMemoized('k',);

        expect(forkSecond,).toBe(forkFirst,);
        expect(upstreamSecond,).toBe(upstreamFirst,);

        /**
         Rejection message the fork produced.
         */
        let forkMessage = '';
        try {
          await forkFirst;
        }
        catch (error) {
          forkMessage = (error as Error).message;
        }
        /**
         Rejection message the upstream call produced.
         */
        let upstreamMessage = '';
        try {
          await upstreamFirst;
        }
        catch (error) {
          upstreamMessage = (error as Error).message;
        }
        expect(forkMessage,).toBe(upstreamMessage,);
        expect(forkMessage,).toBe('sync-boom',);
      },
    },),
  ],
},);
