/**
 Tests for cache storage behavior: default `Map` caching, custom storages,
 caching disabled, and the cache method contract.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { pMemoize, } from '../dist/final/neutral/index.mjs';

import { createSpyCache, } from './test-support.ts';

await describe({
  name: 'cache option',
  children: [
    it({
      name: 'creates a fresh Map cache per memoized function',
      fn: async () => {
        /**
         Monotonic counter answered on every wrapped invocation.
         */
        const state = {
          index: 0,
        };
        /**
         Wrapped function ignoring its key and counting invocations.
         */
        async function fixture(_key: string,): Promise<number> {
          return state.index++;
        }
        const first = pMemoize({
          fn: fixture,
        },);
        const second = pMemoize({
          fn: fixture,
        },);

        expect(await first({
          args: ['k'],
        },),).toBe(0,);
        expect(await second({
          args: ['k'],
        },),).toBe(1,);
      },
    },),

    it({
      name: 'reads a cache hit through has then get without calling the function',
      fn: async () => {
        /**
         Spy storage recording its method calls.
         */
        const spy = createSpyCache<string, string>();
        /**
         Invocation counter for the wrapped function.
         */
        const state = {
          invocations: 0,
        };
        /**
         Wrapped function counting invocations.
         */
        async function fixture(key: string,): Promise<string> {
          state.invocations += 1;
          return `value-${key}`;
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache: spy.cache,
          },
        },);

        expect(await memoized({
          args: ['k'],
        },),).toBe('value-k',);
        expect(await memoized({
          args: ['k'],
        },),).toBe('value-k',);
        expect(state.invocations,).toBe(1,);
        expect(spy.calls.map(function callMethod(call,): string {
          return call.method;
        },),).toEqual([
          'has',
          'set',
          'has',
          'get',
        ],);
      },
    },),

    it({
      name: 'settles undefined when has reports a hit but get misses',
      fn: async () => {
        /**
         Storage reporting every key present but reading `undefined`.
         */
        const alwaysPresentCache = {
          has: function has(_key: string,): boolean {
            return true;
          },
          get: function get(_key: string,): undefined {
            return undefined;
          },
          set: function set(_key: string, _value: string,): void {},
          delete: function remove(_key: string,): boolean {
            return false;
          },
        } as const;
        /**
         Wrapped function that must not run for a reported cache hit.
         */
        async function fixture(key: string,): Promise<string> {
          return `value-${key}`;
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache: alwaysPresentCache,
          },
        },);

        expect(await memoized({
          args: ['k'],
        },),).toBeUndefined();
      },
    },),

    it({
      name: 'accepts a WeakMap cache keyed by identity',
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
        async function fixture(key: object,): Promise<number> {
          void key;
          return state.index++;
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache: new WeakMap<object, number>(),
            cacheKey: function identityKey([firstArgument]: readonly unknown[],): object {
              return firstArgument as object;
            },
          },
        },);
        /**
         Identity-keyed argument reused across calls.
         */
        const foo = {};
        /**
         Second identity-keyed argument with its own cache entry.
         */
        const bar = {};

        expect(await memoized({
          args: [foo],
        },),).toBe(0,);
        expect(await memoized({
          args: [foo],
        },),).toBe(0,);
        expect(await memoized({
          args: [bar],
        },),).toBe(1,);
        expect(await memoized({
          args: [bar],
        },),).toBe(1,);
      },
    },),

    it({
      name: 'recomputes after the cache entry is deleted externally',
      fn: async () => {
        /**
         Monotonic counter answered on every wrapped invocation.
         */
        const state = {
          index: 0,
        };
        /**
         Shared cache the test deletes from directly.
         */
        const cache = new Map<object, number>();
        /**
         Wrapped function counting invocations.
         */
        async function fixture(key: object,): Promise<number> {
          void key;
          return state.index++;
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache,
            cacheKey: function identityKey([firstArgument]: readonly unknown[],): object {
              return firstArgument as object;
            },
          },
        },);
        /**
         Identity-keyed argument whose entry the test removes.
         */
        const foo = {};

        expect(await memoized({
          args: [foo],
        },),).toBe(0,);
        expect(await memoized({
          args: [foo],
        },),).toBe(0,);
        cache.delete(foo,);
        expect(await memoized({
          args: [foo],
        },),).toBe(1,);
        expect(await memoized({
          args: [foo],
        },),).toBe(1,);
      },
    },),

    it({
      name: 'disables caching while still de-duplicating concurrent calls',
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
        async function fixture(): Promise<number> {
          return state.index++;
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache: false,
          },
        },);

        expect(await memoized({
          args: [],
        },),).toBe(0,);
        expect(await memoized({
          args: [],
        },),).toBe(1,);
        expect(await memoized({
          args: [],
        },),).toBe(2,);
        expect(await Promise.all([
          memoized({
            args: [],
          },),
          memoized({
            args: [],
          },),
        ],),).toEqual([
          3,
          3,
        ],);
      },
    },),
  ],
},);
