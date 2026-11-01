import { randomUUID, } from 'node:crypto';
import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { resolveCachedRealGit, } from '../dist/final/node/resolution-cache.mjs';

//region Cache test fixtures

/**
 Successful entry count retained by real-Git cache.
 */
const CACHE_CAPACITY = 16;

/**
 One additional key that forces least-recently-used eviction.
 */
const OVERFLOW_KEY_INDEX = CACHE_CAPACITY;

/**
 Mutable observation state scoped to one cache test.
 */
type ScanState = {
  /**
   Fresh scanner invocations observed through injected callback.
   */
  count: number;
};

/**
 Resolves test value after asynchronous filesystem operation.
 
 @param cacheKey - Unique cache identity and returned test value.
 
 @param state - Observable fresh-scan counter.
 
 @returns Cache key after asynchronous work keeps first resolution in flight.
 
 @example
 ```ts
 await observedResolution({
   cacheKey: 'case-1',
   state: { count: 0 },
 });
 ```
 */
async function observedResolution({
  cacheKey,
  state,
}: {
  readonly cacheKey: string;
  readonly state: ScanState;
},): Promise<string> {
  state.count += 1;
  await readFile(import.meta.filename,);
  return cacheKey;
}

//endregion Cache test fixtures

await describe({
  name: resolveCachedRealGit.name,
  concurrency: 1,
  children: [
    it({
      name: 'invokes one scanner for concurrent equal cache misses',
      fn: async function invokesOneConcurrentScanner(): Promise<void> {
        /**
         Unique key isolating this test from process-level cache entries.
         */
        const cacheKey = `concurrent-${randomUUID()}`;
        /**
         Fresh-scan observation shared by equal callers.
         */
        const state: ScanState = { count: 0, };
        /**
         Concurrent equal results sharing one in-flight scanner.
         */
        const results = await Promise.all([
          resolveCachedRealGit({
            cacheKey,
            resolve: async function resolveFirstCall(): Promise<string> {
              return await observedResolution({ cacheKey, state, },);
            },
          },),
          resolveCachedRealGit({
            cacheKey,
            resolve: async function resolveSecondCall(): Promise<string> {
              return await observedResolution({ cacheKey, state, },);
            },
          },),
        ],);

        expect(results,).toEqual([cacheKey, cacheKey,],);
        expect(state.count,).toBe(1,);
      },
    },),
    it({
      name: 'retains recently reused success and evicts untouched oldest success',
      fn: async function retainsRecentlyUsedSuccess(): Promise<void> {
        /**
         Unique prefix isolating LRU sequence from other cache tests.
         */
        const keyPrefix = `lru-${randomUUID()}`;
        /**
         Fresh-scan observation across fill,
         refresh,
         overflow,
         and reprobe.
         */
        const state: ScanState = { count: 0, };
        /**
         Initial keys filling cache exactly to capacity.
         */
        const initialKeys = Array.from(
          { length: CACHE_CAPACITY, },
          function initialCacheKey(_unused, index,) {
            return `${keyPrefix}-${String(index,)}`;
          },
        );

        for (const cacheKey of initialKeys) {
          // oxlint-disable-next-line no-await-in-loop -- deterministic insertion order establishes LRU fixture
          await resolveCachedRealGit({
            cacheKey,
            resolve: async function resolveInitialKey(): Promise<string> {
              return await observedResolution({ cacheKey, state, },);
            },
          },);
        }

        /**
         Oldest key refreshed to most recently used before overflow.
         */
        const [refreshedKey, evictedKey,] = initialKeys;
        if ((refreshedKey === undefined) || (evictedKey === undefined))
          throw new Error('LRU fixture did not create refresh and eviction keys.',);
        await resolveCachedRealGit({
          cacheKey: refreshedKey,
          resolve: async function shouldNotResolveRefreshedKey(): Promise<string> {
            return await observedResolution({ cacheKey: refreshedKey, state, },);
          },
        },);

        /**
         New key forcing one least-recently-used eviction.
         */
        const overflowKey = `${keyPrefix}-${String(OVERFLOW_KEY_INDEX,)}`;
        await resolveCachedRealGit({
          cacheKey: overflowKey,
          resolve: async function resolveOverflowKey(): Promise<string> {
            return await observedResolution({ cacheKey: overflowKey, state, },);
          },
        },);
        await resolveCachedRealGit({
          cacheKey: refreshedKey,
          resolve: async function shouldStillNotResolveRefreshedKey(): Promise<string> {
            return await observedResolution({ cacheKey: refreshedKey, state, },);
          },
        },);
        await resolveCachedRealGit({
          cacheKey: evictedKey,
          resolve: async function resolveEvictedKeyAgain(): Promise<string> {
            return await observedResolution({ cacheKey: evictedKey, state, },);
          },
        },);

        expect(state.count,).toBe(CACHE_CAPACITY + 2,);
      },
    },),
  ],
},);
