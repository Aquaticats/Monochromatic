/**
 Tests for `pMemoizeClear`: cache dropping and the three failure shapes.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type MemoizedFunction,
  pMemoize,
  pMemoizeClear,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: pMemoizeClear.name,
  children: [
    it({
      name: 'drops every cached value so the next call recomputes',
      fn: async () => {
        /**
         Monotonic counter answered on every wrapped invocation.
         */
        const state = {
          index: 0,
        };
        /**
         Wrapped function counting invocations.
         */
        async function fixture(_key: string,): Promise<number> {
          return state.index++;
        };
        const memoized = pMemoize({
          fn: fixture,
        },);

        expect(await memoized({
          args: ['k'],
        },),).toBe(0,);
        expect(await memoized({
          args: ['k'],
        },),).toBe(0,);
        pMemoizeClear(memoized,);
        expect(await memoized({
          args: ['k'],
        },),).toBe(1,);
        expect(await memoized({
          args: ['k'],
        },),).toBe(1,);
      },
    },),

    it({
      name: 'calls clear as a method on the cache storage',
      fn: async () => {
        /**
         Receiver name read by `clear` through `this`.
         */
        const state = {
          seen: '',
        };
        /**
         Cache storage whose `clear` depends on its own receiver.
         */
        const cache = {
          marker: 'receiver',
          has: function has(_key: string,): boolean {
            return false;
          },
          get: function get(_key: string,): undefined {
            return undefined;
          },
          set: function set(_key: string, _value: number,): void {},
          delete: function remove(_key: string,): boolean {
            return false;
          },
          clear: function clear(this: {
            readonly marker: string;
          },): void {
            state.seen = this.marker;
          },
        } as const;
        /**
         Wrapped function resolving a constant.
         */
        async function fixture(_key: string,): Promise<number> {
          return 1;
        };
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache,
          },
        },);

        pMemoizeClear(memoized,);
        expect(state.seen,).toBe('receiver',);
      },
    },),

    it({
      name: 'throws NotMemoizedError for a function that was never memoized',
      fn: async () => {
        /**
         Plain function that was never memoized.
         */
        async function plain(): Promise<number> {
          return 1;
        };

        let caught: unknown;
        try {
          pMemoizeClear(plain,);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(TypeError,);
        expect((caught as Error).name,).toBe('NotMemoizedError',);
        expect((caught as Error).message,).toBe('Can\'t clear a function that was not memoized!',);
      },
    },),

    it({
      name: 'throws CacheDisabledError for a caching-disabled memoized function',
      fn: async () => {
        /**
         Wrapped function resolving a constant.
         */
        async function fixture(): Promise<number> {
          return 1;
        };
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache: false,
          },
        },);

        let caught: unknown;
        try {
          pMemoizeClear(memoized,);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(TypeError,);
        expect((caught as Error).name,).toBe('CacheDisabledError',);
        expect((caught as Error).message,).toBe('Can\'t clear a function that doesn\'t use a cache!',);
      },
    },),

    it({
      name: 'throws UnclearableCacheError for a cache storage without clear',
      fn: async () => {
        /**
         Wrapped function resolving a constant.
         */
        async function fixture(key: object,): Promise<number> {
          void key;
          return 1;
        };
        const memoized: MemoizedFunction<[object], number> = pMemoize({
          fn: fixture,
          options: {
            cache: new WeakMap<object, number>(),
            cacheKey: function identityKey([firstArgument]: readonly unknown[],): object {
              return firstArgument as object;
            },
          },
        },);

        let caught: unknown;
        try {
          pMemoizeClear(memoized,);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(TypeError,);
        expect((caught as Error).name,).toBe('UnclearableCacheError',);
        expect((caught as Error).message,).toBe('The cache can\'t be cleared!',);
      },
    },),
  ],
},);
