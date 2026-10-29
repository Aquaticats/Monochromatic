/**
 Tests for the registry behind `pMemoize` and `pMemoizeClear`: the clear
 counter shared by every memoized function over one cache storage.

 The registry is not part of the package API, so each case reaches it
 through `pMemoize` and `pMemoizeClear`, which is also how consumers depend
 on it.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CACHE_DISABLED,
  pMemoize,
  pMemoizeClear,
} from '../dist/final/neutral/index.mjs';
import { createDeferred, } from './test-support.ts';

await describe({
  name: 'memoizedCacheStore shared generation',
  children: [
    it({
      name: 'gives a memoized function a working counter when its cache is new',
      fn: async () => {
        const cache = new Map<string, number>();
        async function fixture(_key: string,): Promise<number> {
          return 1;
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache,
          },
        },);

        expect(await memoized({
          args: ['a'],
        },),).toBe(1,);
        expect(cache.get('a',),).toBe(1,);
      },
    },),

    it({
      name: 'lets a clear on one function stop an in-flight write of another over the same cache',
      fn: async () => {
        const cache = new Map<string, number>();
        const gate = createDeferred<number>();
        async function slowFixture(_key: string,): Promise<number> {
          return await gate.promise;
        }
        async function otherFixture(_key: string,): Promise<number> {
          return 0;
        }
        const slow = pMemoize({
          fn: slowFixture,
          options: {
            cache,
          },
        },);
        const other = pMemoize({
          fn: otherFixture,
          options: {
            cache,
          },
        },);

        const inFlight = slow({
          args: ['a'],
        },);
        pMemoizeClear(other,);
        gate.resolve(5,);

        expect(await inFlight,).toBe(5,);
        expect(cache.size,).toBe(0,);
      },
    },),

    it({
      name: 'keeps the counter of one cache independent of another cache',
      fn: async () => {
        const clearedCache = new Map<string, number>();
        const otherCache = new Map<string, number>();
        const gate = createDeferred<number>();
        async function slowFixture(_key: string,): Promise<number> {
          return await gate.promise;
        }
        async function clearedFixture(_key: string,): Promise<number> {
          return 0;
        }
        const slow = pMemoize({
          fn: slowFixture,
          options: {
            cache: otherCache,
          },
        },);
        const cleared = pMemoize({
          fn: clearedFixture,
          options: {
            cache: clearedCache,
          },
        },);

        const inFlight = slow({
          args: ['a'],
        },);
        pMemoizeClear(cleared,);
        gate.resolve(5,);

        expect(await inFlight,).toBe(5,);
        expect(otherCache.get('a',),).toBe(5,);
      },
    },),

    it({
      name: 'serves hits and writes for calls that start after a clear',
      fn: async () => {
        const cache = new Map<string, number>();
        const state = {
          calls: 0,
        };
        async function fixture(_key: string,): Promise<number> {
          state.calls += 1;
          return state.calls;
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache,
          },
        },);

        pMemoizeClear(memoized,);
        expect(await memoized({
          args: ['a'],
        },),).toBe(1,);
        expect(cache.get('a',),).toBe(1,);
        expect(await memoized({
          args: ['a'],
        },),).toBe(1,);
        expect(state.calls,).toBe(1,);
      },
    },),

    it({
      name: 'describes the caching-disabled marker for anyone who prints it',
      fn: async () => {
        expect(typeof CACHE_DISABLED,).toBe('symbol',);
        expect(CACHE_DISABLED.description,).toBe('memoized function caches nothing',);
      },
    },),
  ],
},);
