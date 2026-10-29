/**
 Tests for cache-key derivation: the default first-argument key and custom
 `cacheKey` functions.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { pMemoize, } from '../dist/final/neutral/index.mjs';

await describe({
  name: 'cacheKey option',
  children: [
    it({
      name: 'normalizes different argument types into one key',
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
        async function fixture(_key: unknown,): Promise<number> {
          return state.index++;
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cacheKey: function stringifyKey([firstArgument]: readonly unknown[],): string {
              return String(firstArgument,);
            },
          },
        });

        expect(await memoized({
          args: [1],
        },),).toBe(0,);
        expect(await memoized({
          args: [1],
        },),).toBe(0,);
        expect(await memoized({
          args: ['1'],
        },),).toBe(0,);
        expect(await memoized({
          args: ['2'],
        },),).toBe(1,);
        expect(await memoized({
          args: [2],
        },),).toBe(1,);
      },
    },),

    it({
      name: 'keys by all arguments with a JSON.stringify cacheKey',
      fn: async () => {
        /**
         Monotonic counter answered on every wrapped invocation.
         */
        const state = {
          index: 0,
        };
        /**
         Wrapped function ignoring its arguments and counting invocations.
         */
        async function fixture(
          _first: unknown,
          _second: unknown,
          _third?: unknown,
        ): Promise<number> {
          return state.index++;
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cacheKey: function stringifyAll(args: readonly unknown[],): string {
              return JSON.stringify(args,);
            },
          },
        });

        expect(await memoized({
          args: [
            {
              foo: true,
            },
            {
              bar: false,
            },
          ],
        },),).toBe(0,);
        expect(await memoized({
          args: [
            {
              foo: true,
            },
            {
              bar: false,
            },
          ],
        },),).toBe(0,);
        expect(await memoized({
          args: [
            {
              foo: true,
            },
            {
              bar: false,
            },
            {
              baz: true,
            },
          ],
        },),).toBe(1,);
      },
    },),

    it({
      name: 'receives the whole argument tuple',
      fn: async () => {
        /**
         Argument tuples the cacheKey function observed.
         */
        const state = {
          seen: [] as readonly (readonly unknown[])[],
        };
        /**
         Wrapped function echoing its key.
         */
        async function fixture(
          key: string,
          _ignored?: string,
        ): Promise<string> {
          return key;
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cacheKey: function recordKey(args: readonly unknown[],): string {
              state.seen = [
                ...state.seen,
                args,
              ];
              return String(args[0],);
            },
          },
        });

        await memoized({
          args: ['a', 'ignored'],
        },);
        expect(state.seen,).toEqual([
          ['a', 'ignored'],
        ],);
      },
    },),

    it({
      name: 'throws synchronously when the cacheKey function throws',
      fn: async () => {
        /**
         Wrapped function that must never run for a broken cacheKey.
         */
        async function fixture(_key: string,): Promise<string> {
          return 'ran';
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cacheKey: function brokenKey(_args: readonly unknown[],): string {
              throw new Error('key failure',);
            },
          },
        });

        let caught: unknown;
        try {
          void memoized({
            args: ['a'],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toBe('key failure',);
      },
    },),
  ],
},);
