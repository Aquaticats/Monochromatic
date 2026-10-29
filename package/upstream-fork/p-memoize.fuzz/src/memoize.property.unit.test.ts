/**
 Memoization invariant properties: outcomes follow the generated behavior,
 in-flight promises are shared per key, and cache writes respect the
 predicate.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  assert,
  asyncProperty,
} from 'fast-check';

import { fuzzRuns, } from './fuzz-budget.ts';
import { workloadArb, } from './memoize-arbitrary.ts';
import { createForkAdapter, } from './memoizer-adapters.ts';
import {
  failureFor,
  runWorkload,
  type WorkloadKey,
} from './workload.ts';

//region Helpers

/**
 Projects one workload key into its cache-key text under the workload's
 key mode.
 
 @param workload - Scenario whose key mode applies.
 
 @param key - Workload key to project.
 
 @returns Cache-key text both the cache and the predicate observe.
 
 @example
 ```ts
 cacheKeyText(workload, 'a',);
 ```
 */
function cacheKeyText(workload: {
  readonly keyMode: 'firstArgument' | 'json';
}, key: WorkloadKey,): string {
  if (workload.keyMode === 'json')
    return JSON.stringify([key],);
  return key;
}

/**
 Extracts the key half of one `method key value` cache-call record.
 
 @param cacheCall - Recorded cache call.
 
 @returns Key the call carried.
 
 @throws Error when the record carries no key.
 */
function cacheCallKey(cacheCall: string,): string {
  /**
   Space-separated record halves; the second half is the key.
   */
  const [, key] = cacheCall.split(' ',);

  if (key === undefined)
    throw new Error('cache call carried no key',);

  return key;
}

/**
 Extracts the stored value half of one `key=value` cache entry.
 
 @param entry - Final cache entry.
 
 @returns Stored value text.
 
 @throws Error when the entry carries no value.
 */
function cacheEntryValue(entry: string,): string {
  /**
   Equals-separated entry halves; the second half is the stored value.
   */
  const [, value] = entry.split('=',);

  if (value === undefined)
    throw new Error('cache entry carried no value',);

  return value;
}

//endregion Helpers

await describe({
  name: 'memoization invariants',
  children: [
    it({
      name: 'every call settles with its key behavior outcome',
      fn: async () => {
        await assert(
          asyncProperty(workloadArb, async (workload,) => {
            /**
             Trace of one fork run.
             */
            const trace = await runWorkload({
              createAdapter: createForkAdapter,
              workload,
            },);
            /**
             Flat key list matching the outcome order.
             */
            const keys = workload.batches.flat();

            for (const [
              index,
              outcome,
            ] of trace.outcomes.entries()) {
              /**
               Behavior this call's key was generated with.
               */
              const behavior = workload.behaviorByKey[keys[index] as WorkloadKey];

              if (behavior === 'resolve') {
                if ((workload.shouldCacheMode === 'throw') && workload.cacheEnabled) {
                  // A throwing write predicate rejects the call even though
                  // the wrapped function would have resolved; with caching
                  // disabled the predicate never runs.
                  expect(outcome.status,).toBe('rejected',);
                  /**
                   Predicate failure text the call must carry.
                   */
                  const detailText = String(outcome.detail,);
                  expect(detailText.startsWith('predicate-boom-',),).toBe(true,);
                }
                else {
                  expect(outcome.status,).toBe('fulfilled',);
                  expect(typeof outcome.detail,).toBe('number',);
                }
              }
              else {
                expect(outcome.status,).toBe('rejected',);
                expect(outcome.detail,).toBe(failureFor(keys[index] as WorkloadKey,).message,);
              }
            }
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'concurrent calls with one key share one promise',
      fn: async () => {
        await assert(
          asyncProperty(workloadArb, async (workload,) => {
            /**
             Trace of one fork run.
             */
            const trace = await runWorkload({
              createAdapter: createForkAdapter,
              workload,
            },);
            /**
             Running call index across batches.
             */
            const indexState = {
              next: 0,
            };

            for (const batch of workload.batches) {
              /**
               Owner index per key within this batch.
               */
              const ownerByKey = new Map<WorkloadKey, number>();

              for (const key of batch) {
                /**
                 Outcome of this call.
                 */
                const outcome = trace.outcomes[indexState.next];

                if (outcome === undefined)
                  throw new Error('trace is missing an outcome',);

                /**
                 Owner recorded for this key earlier in the batch.
                 */
                const owner = ownerByKey.get(key,);

                if (owner === undefined)
                  ownerByKey.set(
                    key,
                    outcome.sharedWith,
                  );
                else
                  expect(outcome.sharedWith,).toBe(owner,);

                indexState.next += 1;
              }
            }
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'invocation ordinals count real runs from zero without gaps',
      fn: async () => {
        await assert(
          asyncProperty(workloadArb, async (workload,) => {
            /**
             Trace of one fork run.
             */
            const trace = await runWorkload({
              createAdapter: createForkAdapter,
              workload,
            },);

            expect(trace.invocations,).toEqual(trace.invocations.map(function asOrdinal(
              _ordinal: number,
              index: number,
            ): number {
              return index;
            },),);
            expect(trace.invocations.length,).toBeLessThanOrEqual(trace.outcomes.length,);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'writes to the cache only for fulfilled resolve outcomes',
      fn: async () => {
        await assert(
          asyncProperty(workloadArb, async (workload,) => {
            /**
             Trace of one fork run.
             */
            const trace = await runWorkload({
              createAdapter: createForkAdapter,
              workload,
            },);

            for (const cacheCall of trace.cacheCalls) {
              if (!cacheCall.startsWith('set ',))
                continue;

              /**
               Key written by this cache call.
               */
              const writtenKey = cacheCallKey(cacheCall,);
              /**
               Workload key whose cache key matches the written key.
               */
              const writtenFor = workload.batches.flat().find(function matchesKey(key: WorkloadKey,): boolean {
                return cacheKeyText(workload, key,) === writtenKey;
              },);

              expect(writtenFor,).toBeDefined();
              expect(workload.behaviorByKey[writtenFor as WorkloadKey],).toBe('resolve',);
            }

            if (workload.shouldCacheMode !== 'always')
              expect(trace.cacheCalls.some(function isSet(cacheCall,): boolean {
                return cacheCall.startsWith('set ',);
              },),).toBe(false,);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),

    it({
      name: 'cached entries contain only fulfilled values',
      fn: async () => {
        await assert(
          asyncProperty(workloadArb, async (workload,) => {
            /**
             Trace of one fork run.
             */
            const trace = await runWorkload({
              createAdapter: createForkAdapter,
              workload,
            },);

            for (const entry of trace.cacheContents) {
              /**
               Stored value half of one `key=value` entry.
               */
              const storedValue = cacheEntryValue(entry,);
              /**
               Parsed numeric form of the stored value; fulfilled values are
               invocation ordinals.
               */
              const parsedValue = Number(storedValue,);
              expect(Number.isInteger(parsedValue,),).toBe(true,);
            }

            if (!workload.cacheEnabled)
              expect(trace.cacheContents,).toEqual([],);
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),
  ],
},);
