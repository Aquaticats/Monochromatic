/**
 Tests for the clear-consistency behavior ported from upstream `p-memoize`
 9.0.0: a clear bumps a generation shared by every function over one cache,
 drops the cleared function's in-flight promises, and keeps calls that were
 running at clear time from refilling the cache.

 Each case mirrors an upstream test of the same behavior.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  pMemoize,
  pMemoizeClear,
} from '../dist/final/neutral/index.mjs';
import {
  createDeferred,
  yieldTurn,
} from './test-support.ts';

await describe({
  name: 'pMemoizeClear consistency with in-flight calls',
  children: [
    it({
      name: 'stops sharing the in-flight promise but keeps overlapping later calls de-duplicated',
      fn: async () => {
        const gate = createDeferred<number>();
        const state = {
          invocations: 0,
        };
        async function fixture(): Promise<number> {
          state.invocations += 1;
          return await gate.promise;
        }
        const memoized = pMemoize({
          fn: fixture,
        },);

        const before = memoized({
          args: [],
        },);
        pMemoizeClear(memoized,);
        const after = memoized({
          args: [],
        },);

        expect(after,).not.toBe(before,);

        gate.resolve(1,);
        expect(await before,).toBe(1,);
        expect(await after,).toBe(1,);
        // The two overlapping calls are still de-duplicated with each other:
        // the first finds the second registered once its cache read ends.
        expect(state.invocations,).toBe(1,);
      },
    },),

    it({
      name: 'discards the result of a call that was already in flight at clear time',
      fn: async () => {
        const gate = createDeferred<number>();
        const state = {
          invocations: 0,
        };
        async function fixture(): Promise<number> {
          state.invocations += 1;
          return await gate.promise;
        }
        const memoized = pMemoize({
          fn: fixture,
        },);

        const inFlight = memoized({
          args: [],
        },);
        pMemoizeClear(memoized,);
        gate.resolve(1,);
        expect(await inFlight,).toBe(1,);

        expect(await memoized({
          args: [],
        },),).toBe(1,);
        expect(state.invocations,).toBe(2,);
      },
    },),

    it({
      name: 'turns a hit into a miss when a clear lands between has and get',
      fn: async () => {
        const cache = new Map<string, string>();
        const state = {
          invocations: 0,
        };
        async function fixture(value: string,): Promise<string> {
          state.invocations += 1;
          return value;
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache,
          },
        },);

        await memoized({
          args: ['something'],
        },);

        // The clear runs while the second call sits between its reads.
        const pending = memoized({
          args: ['something'],
        },);
        pMemoizeClear(memoized,);

        expect(await pending,).toBe('something',);
        expect(state.invocations,).toBe(2,);
      },
    },),

    it({
      name: 'settles undefined when an entry is evicted between has and get without a clear (documented limitation)',
      fn: async () => {
        const cache = new Map<string, string>();
        const state = {
          invocations: 0,
        };
        async function fixture(value: string,): Promise<string> {
          state.invocations += 1;
          return value;
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache,
          },
        },);

        await memoized({
          args: ['something'],
        },);

        const pending = memoized({
          args: ['something'],
        },);
        cache.delete('something',);

        expect(await pending,).toBeUndefined();
        expect(state.invocations,).toBe(1,);
      },
    },),

    it({
      name: 'prevents the write when a clear lands while shouldCache is deciding',
      fn: async () => {
        const result = createDeferred<number>();
        const predicateStarted = createDeferred<void>();
        const releasePredicate = createDeferred<void>();
        const store = new Map<string, number>();
        const writes = {
          count: 0,
        };
        async function fixture(_value: string,): Promise<number> {
          return await result.promise;
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache: {
              has: (key: string,): boolean => store.has(key,),
              get: (key: string,) => store.get(key,),
              set: (key: string, value: number,): void => {
                writes.count += 1;
                store.set(key, value,);
              },
              clear: (): void => {
                store.clear();
              },
            },
            shouldCache: async (): Promise<boolean> => {
              predicateStarted.resolve();
              await releasePredicate.promise;
              return true;
            },
          },
        },);

        const inFlight = memoized({
          args: ['a'],
        },);
        result.resolve(1,);
        await predicateStarted.promise;

        pMemoizeClear(memoized,);
        releasePredicate.resolve();
        await inFlight;

        expect(store.size,).toBe(0,);
        // The write is skipped rather than issued, which matters for a
        // cache where every write costs a round trip.
        expect(writes.count,).toBe(0,);
      },
    },),

    it({
      name: 'does not undo a cache write that already started (documented limitation)',
      fn: async () => {
        const writeStarted = createDeferred<void>();
        const releaseWrite = createDeferred<void>();
        const store = new Map<string, number>();
        const state = {
          invocations: 0,
        };
        async function fixture(_key: string,): Promise<number> {
          state.invocations += 1;
          return state.invocations;
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache: {
              has: (key: string,): boolean => store.has(key,),
              get: (key: string,) => store.get(key,),
              set: async (key: string, value: number,): Promise<void> => {
                writeStarted.resolve();
                await releaseWrite.promise;
                store.set(key, value,);
              },
              clear: (): void => {
                store.clear();
              },
            },
          },
        },);

        const inFlight = memoized({
          args: ['k'],
        },);
        await writeStarted.promise;

        pMemoizeClear(memoized,);
        releaseWrite.resolve();
        await inFlight;
        await yieldTurn();

        expect(store.size,).toBe(1,);
        expect(await memoized({
          args: ['k'],
        },),).toBe(1,);
        expect(state.invocations,).toBe(1,);
      },
    },),

    it({
      name: 'does not let a settling call evict a newer in-flight call',
      fn: async () => {
        const results = [
          createDeferred<number>(),
          createDeferred<number>(),
        ];
        const started = [
          createDeferred<void>(),
          createDeferred<void>(),
        ];
        const state = {
          invocations: 0,
        };
        async function fixture(): Promise<number> {
          const index = state.invocations;
          state.invocations += 1;
          started[index]?.resolve();
          return await (results[index]?.promise ?? Promise.resolve(-1,));
        }
        const memoized = pMemoize({
          fn: fixture,
        },);

        const first = memoized({
          args: [],
        },);
        await started[0]?.promise;

        pMemoizeClear(memoized,);

        const second = memoized({
          args: [],
        },);
        await started[1]?.promise;

        results[0]?.resolve(1,);
        expect(await first,).toBe(1,);

        // `first` has settled, but `second` is still in flight and must stay
        // de-duplicated.
        const third = memoized({
          args: [],
        },);
        expect(third,).toBe(second,);
        expect(state.invocations,).toBe(2,);

        results[1]?.resolve(2,);
        expect(await second,).toBe(2,);
        expect(await third,).toBe(2,);
      },
    },),

    it({
      name: 'stops another memoized function from refilling a cache they share',
      fn: async () => {
        const cache = new Map<string, number>();
        async function betaFixture(_key: string,): Promise<number> {
          return 2;
        }
        async function alphaFixture(_key: string,): Promise<number> {
          return 1;
        }
        // The cleared function is created last on purpose: the clear must
        // reach the generation the other function holds.
        const beta = pMemoize({
          fn: betaFixture,
          options: {
            cache,
          },
        },);
        const alpha = pMemoize({
          fn: alphaFixture,
          options: {
            cache,
          },
        },);

        const inFlight = beta({
          args: ['a'],
        },);
        pMemoizeClear(alpha,);

        expect(await inFlight,).toBe(2,);
        expect(cache.size,).toBe(0,);
      },
    },),

    it({
      name: 'does not drop the in-flight call of another function over the same cache',
      fn: async () => {
        const cache = new Map<string, number>();
        const gate = createDeferred<number>();
        async function betaFixture(_key: string,): Promise<number> {
          return await gate.promise;
        }
        async function alphaFixture(_key: string,): Promise<number> {
          return 0;
        }
        const beta = pMemoize({
          fn: betaFixture,
          options: {
            cache,
          },
        },);
        const alpha = pMemoize({
          fn: alphaFixture,
          options: {
            cache,
          },
        },);

        const inFlight = beta({
          args: ['a'],
        },);
        pMemoizeClear(alpha,);

        const afterClear = beta({
          args: ['a'],
        },);
        expect(afterClear,).toBe(inFlight,);

        gate.resolve(1,);
        expect(await inFlight,).toBe(1,);
        expect(await afterClear,).toBe(1,);
        expect(cache.size,).toBe(0,);
      },
    },),

    it({
      name: 'shares entries and clears between two functions over one cache',
      fn: async () => {
        const cache = new Map<string, string>();
        async function alphaFixture(key: string,): Promise<string> {
          return `alpha:${key}`;
        }
        async function betaFixture(key: string,): Promise<string> {
          return `beta:${key}`;
        }
        const alpha = pMemoize({
          fn: alphaFixture,
          options: {
            cache,
          },
        },);
        const beta = pMemoize({
          fn: betaFixture,
          options: {
            cache,
          },
        },);

        expect(await alpha({
          args: ['x'],
        },),).toBe('alpha:x',);
        expect(await beta({
          args: ['x'],
        },),).toBe('alpha:x',);

        pMemoizeClear(alpha,);
        expect(await beta({
          args: ['x'],
        },),).toBe('beta:x',);
      },
    },),

    it({
      name: 'drops in-flight calls even when cache.clear throws',
      fn: async () => {
        const store = new Map<string, number>();
        const started = [
          createDeferred<void>(),
          createDeferred<void>(),
        ];
        const gate = createDeferred<void>();
        const state = {
          invocations: 0,
        };
        // Each execution resolves with its own invocation number, so the
        // value left in the cache says which of them wrote it.
        async function fixture(_value: string,): Promise<number> {
          started[state.invocations]?.resolve();
          state.invocations += 1;
          await gate.promise;
          return state.invocations;
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache: {
              has: (key: string,): boolean => store.has(key,),
              get: (key: string,) => store.get(key,),
              set: (key: string, value: number,): void => {
                store.set(key, value,);
              },
              clear: (): void => {
                store.clear();
                throw new Error('clear boom',);
              },
            },
          },
        },);

        const inFlight = memoized({
          args: ['a'],
        },);
        await started[0]?.promise;

        expect(() => {
          pMemoizeClear(memoized,);
        },).toThrow('clear boom',);

        // The bookkeeping happens before the cache is emptied, so a clear
        // that throws does not leave the old in-flight promise for this
        // same-key call to join.
        const afterClear = memoized({
          args: ['a'],
        },);
        await started[1]?.promise;
        expect(state.invocations,).toBe(2,);

        gate.resolve();
        await Promise.all([
          inFlight,
          afterClear,
        ],);

        // Only the call started after the clear may write.
        expect(store.get('a',),).toBe(2,);
      },
    },),

    it({
      name: 'does not leave an asynchronous clear rejection unhandled',
      fn: async () => {
        const failures: unknown[] = [];
        function onUnhandledRejection(reason: unknown,): void {
          failures.push(reason,);
        }
        async function fixture(_value: string,): Promise<string> {
          return 'value';
        }

        process.on('unhandledRejection', onUnhandledRejection,);

        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache: {
              has: (): boolean => false,
              get: (): undefined => undefined,
              set: (): undefined => undefined,
              clear: async (): Promise<void> => {
                await Promise.resolve();
                throw new Error('remote flush failed',);
              },
            },
          },
        },);
        await memoized({
          args: ['a'],
        },);

        // A synchronous clear still propagates to the caller; only the
        // promise it hands back is dropped, so this does not throw.
        pMemoizeClear(memoized,);

        await yieldTurn();
        process.off('unhandledRejection', onUnhandledRejection,);

        expect(failures,).toEqual([],);
      },
    },),

    it({
      name: 'settles every call when many clears race many in-flight calls',
      fn: async () => {
        const store = new Map<string, number>();
        const state = {
          invocations: 0,
        };
        async function fixture(_key: string,): Promise<number> {
          state.invocations += 1;
          return state.invocations;
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache: {
              has: (key: string,): boolean => store.has(key,),
              get: (key: string,) => store.get(key,),
              set: async (key: string, value: number,): Promise<void> => {
                await Promise.resolve();
                store.set(key, value,);
              },
              clear: (): void => {
                store.clear();
              },
            },
          },
        },);

        const calls: Promise<number>[] = Array.from(
          { length: 200, },
          function launch(_unused, index,): Promise<number> {
            const call = memoized({
              args: [`k${index % 10}`],
            },);

            if ((index % 3) === 0)
              pMemoizeClear(memoized,);

            return call;
          },
        );
        const settled = await Promise.allSettled(calls,);

        expect(settled.every(function isFulfilled(result,): boolean {
          return result.status === 'fulfilled';
        },),).toBe(true,);
        expect(state.invocations,).toBeLessThanOrEqual(200,);
      },
    },),

    it({
      name: 'lets a memoized function clear its own cache while running',
      fn: async () => {
        const cache = new Map<string, number>();
        const state = {
          invocations: 0,
        };
        async function fixture(_key: string,): Promise<number> {
          state.invocations += 1;
          if (state.invocations === 1)
            pMemoizeClear(memoized,);

          return state.invocations;
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
        expect(cache.size,).toBe(0,);
        expect(await memoized({
          args: ['a'],
        },),).toBe(2,);
      },
    },),

    it({
      name: 'lets shouldCache clear the cache while the call is running',
      fn: async () => {
        const cache = new Map<string, number>();
        const state = {
          invocations: 0,
          decisions: 0,
        };
        async function fixture(_key: string,): Promise<number> {
          state.invocations += 1;
          return state.invocations;
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache,
            shouldCache: (): boolean => {
              if (state.decisions === 0)
                pMemoizeClear(memoized,);

              state.decisions += 1;
              return true;
            },
          },
        },);

        expect(await memoized({
          args: ['a'],
        },),).toBe(1,);
        expect(cache.size,).toBe(0,);
        expect(await memoized({
          args: ['a'],
        },),).toBe(2,);
      },
    },),

    it({
      name: 'releases the memoized state of many keys on a clear',
      fn: async () => {
        const state = {
          invocations: 0,
        };
        async function fixture(_key: number,): Promise<number> {
          state.invocations += 1;
          return state.invocations;
        }
        const memoized = pMemoize({
          fn: fixture,
        },);

        for (let index = 0; index < 200; index += 1)
          // oxlint-disable-next-line eslint/no-await-in-loop -- sequential on purpose: the point is the volume of keys
          await memoized({
            args: [index],
          },);

        const before = state.invocations;
        pMemoizeClear(memoized,);
        await memoized({
          args: [0],
        },);
        expect(state.invocations,).toBe(before + 1,);
      },
    },),
  ],
},);
