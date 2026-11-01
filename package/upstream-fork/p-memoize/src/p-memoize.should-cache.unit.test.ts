/**
 Tests for the `shouldCache` write predicate: skipped writes, in-flight
 de-duplication, asynchronous predicates, and failure propagation.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { pMemoize, } from '../dist/final/neutral/index.mjs';

import {
  createDeferred,
  createSpyCache,
} from './test-support.ts';

await describe({
  name: 'shouldCache option',
  children: [
    it({
      name: 'skips cache.set when the predicate returns false',
      fn: async () => {
        /**
         Spy storage recording its method calls.
         */
        const spy = createSpyCache<string, number>();
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
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache: spy.cache,
            shouldCache: function denyWrites(): boolean {
              return false;
            },
          },
        },);

        expect(await memoized({
          args: ['a'],
        },),).toBe(0,);
        expect(spy.calls.some(function isSet(call,): boolean {
          return call.method === 'set';
        },),).toBe(false,);
        expect(await memoized({
          args: ['a'],
        },),).toBe(1,);
      },
    },),

    it({
      name: 'still de-duplicates in-flight calls when writes are skipped',
      fn: async () => {
        /**
         Deferred the wrapped call waits on, so both callers stay in flight.
         */
        const deferred = createDeferred<number>();
        /**
         Invocation counter for the wrapped function.
         */
        const state = {
          invocations: 0,
        };
        /**
         Wrapped function blocking on the deferred gate.
         */
        async function fixture(_key: string,): Promise<number> {
          state.invocations += 1;
          return await deferred.promise;
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            shouldCache: function denyWrites(): boolean {
              return false;
            },
          },
        },);

        const first = memoized({
          args: ['a'],
        },);
        const second = memoized({
          args: ['a'],
        },);
        expect(second,).toBe(first,);

        deferred.resolve(42,);
        expect(await first,).toBe(42,);
        expect(await second,).toBe(42,);
        expect(state.invocations,).toBe(1,);
      },
    },),

    it({
      name: 'is not called when caching is disabled',
      fn: async () => {
        /**
         Predicate call counter.
         */
        const state = {
          calls: 0,
        };
        /**
         Monotonic counter answered on every wrapped invocation.
         */
        const indexState = {
          index: 0,
        };
        /**
         Wrapped function counting invocations.
         */
        async function fixture(_key: string,): Promise<number> {
          return indexState.index++;
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache: false,
            shouldCache: function countCalls(): boolean {
              state.calls += 1;
              return true;
            },
          },
        },);

        await memoized({
          args: ['x'],
        },);
        await memoized({
          args: ['x'],
        },);
        expect(state.calls,).toBe(0,);
      },
    },),

    it({
      name: 'is not called on rejection and caches nothing',
      fn: async () => {
        /**
         Spy storage recording its method calls.
         */
        const spy = createSpyCache<string, string>();
        /**
         Predicate call counter.
         */
        const state = {
          calls: 0,
        };
        /**
         Wrapped function always failing.
         */
        async function fixture(_key: string,): Promise<string> {
          throw new Error('boom',);
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache: spy.cache,
            shouldCache: function countCalls(): boolean {
              state.calls += 1;
              return true;
            },
          },
        },);

        let caught: unknown;
        try {
          await memoized({
            args: ['x'],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect((caught as Error).message,).toBe('boom',);
        expect(state.calls,).toBe(0,);
        expect(spy.calls.some(function isSet(call,): boolean {
          return call.method === 'set';
        },),).toBe(false,);
      },
    },),

    it({
      name: 'supports an asynchronous predicate',
      fn: async () => {
        /**
         Spy storage recording its method calls.
         */
        const spy = createSpyCache<string, string>();
        /**
         Wrapped function resolving a constant.
         */
        async function fixture(_key: string,): Promise<string> {
          return 'ok';
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache: spy.cache,
            shouldCache: async function allowOk(value: string,): Promise<boolean> {
              await Promise.resolve();
              return value === 'ok';
            },
          },
        },);

        expect(await memoized({
          args: ['x'],
        },),).toBe('ok',);
        expect(spy.calls.filter(function isSet(call,): boolean {
          return call.method === 'set';
        },).length,).toBe(1,);
      },
    },),

    it({
      name: 'receives the derived key and argument tuple',
      fn: async () => {
        /**
         Context the predicate observed.
         */
        const state = {
          key: undefined as unknown,
          argumentsList: [] as readonly unknown[],
        };
        /**
         Wrapped function resolving its key.
         */
        async function fixture(_first: string | number,): Promise<string> {
          return 'v';
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cacheKey: function stringifyKey([firstArgument]: readonly unknown[],): string {
              return String(firstArgument,);
            },
            shouldCache: function recordContext(
              _value: string,
              context: {
                key: string;
                argumentsList: readonly unknown[];
              },
            ): boolean {
              state.key = context.key;
              state.argumentsList = context.argumentsList;
              return true;
            },
          },
        },);

        await memoized({
          args: [1],
        },);
        expect(state.key,).toBe('1',);
        expect(state.argumentsList,).toEqual([1],);
      },
    },),

    it({
      name: 'propagates a predicate failure and caches nothing',
      fn: async () => {
        /**
         Spy storage recording its method calls.
         */
        const spy = createSpyCache<string, string>();
        /**
         Wrapped function resolving a constant.
         */
        async function fixture(_key: string,): Promise<string> {
          return 'x';
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache: spy.cache,
            shouldCache: function brokenPredicate(): boolean {
              throw new Error('nope',);
            },
          },
        },);

        let caught: unknown;
        try {
          await memoized({
            args: ['k'],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect((caught as Error).message,).toBe('nope',);
        expect(spy.calls.some(function isSet(call,): boolean {
          return call.method === 'set';
        },),).toBe(false,);
      },
    },),

    it({
      name: 'lets external eviction recompute after a completed write',
      fn: async () => {
        /**
         Spy storage recording its method calls and allowing direct deletes.
         */
        const spy = createSpyCache<string, number>();
        /**
         Monotonic counter answered on every wrapped invocation.
         */
        const state = {
          index: 0,
        };
        /**
         Wrapped function counting invocations.
         */
        async function fixture(key: string,): Promise<number> {
          void key;
          return state.index++;
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache: spy.cache,
            shouldCache: function allowWrites(): boolean {
              return true;
            },
          },
        },);

        expect(await memoized({
          args: ['a'],
        },),).toBe(0,);
        spy.cache.delete('a',);
        expect(await memoized({
          args: ['a'],
        },),).toBe(1,);
      },
    },),
  ],
},);
