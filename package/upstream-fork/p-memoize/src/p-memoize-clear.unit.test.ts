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
import { yieldTurn, } from './test-support.ts';

/**
 Result of a fake cache `clear()` whose `then` answers come from a script,
 plus the observations a test asserts on.
 */
type ScriptedThenable = {
  /**
   Value the fake `clear()` returns.
   */
  readonly value: object;
  /**
   Number of times `then` was read from the value.
   */
  readonly reads: () => number;
  /**
   Number of times a callable `then` was invoked.
   */
  readonly invocations: () => number;
};

/**
 Builds a value whose `then` property answers each read from a script, so
 a test can tell how often and in what order `then` is read and whether the
 answer was ever called.

 @param answers - Successive answers to reads of `then`; the last repeats.

 @param base - Object or function that carries the `then` accessor.

 @returns The value and counters over its `then` reads and calls.

 @example
 ```ts
 const scripted = createScriptedThenable({ answers: [1], base: {}, },);
 ```
 */
function createScriptedThenable(
  {
    answers,
    base,
  }: {
    readonly answers: readonly unknown[];
    readonly base: object;
  },
): ScriptedThenable {
  /**
   Mutable counters behind the accessors.
   */
  const counts = {
    reads: 0,
    invocations: 0,
  };
  Object.defineProperty(
    base,
    // oxlint-disable-next-line unicorn/no-thenable -- a thenable whose `then` reads are scripted is the point of this fixture
    'then',
    {
      configurable: true,
      get: function readThen(): unknown {
        /**
         Answer scripted for this read.
         */
        const answer = answers[Math.min(
          counts.reads,
          answers.length - 1,
        )];
        counts.reads += 1;
        if ((typeof answer) !== 'function')
          return answer;

        return function then(): void {
          counts.invocations += 1;
        };
      },
    },
  );

  return {
    value: base,
    reads: (): number => counts.reads,
    invocations: (): number => counts.invocations,
  };
}

/**
 Memoizes a function over a cache whose `clear()` hands back the given value,
 then clears it.

 @param cleared - Value the cache's `clear()` returns.

 @example
 ```ts
 await clearWithResult({ cleared: Promise.resolve(), },);
 ```
 */
async function clearWithResult({ cleared, }: { readonly cleared: unknown; },): Promise<void> {
  async function fixture(_key: string,): Promise<number> {
    return 1;
  }
  const memoized = pMemoize({
    fn: fixture,
    options: {
      cache: {
        has: (): boolean => false,
        get: (): undefined => undefined,
        set: (): undefined => undefined,
        clear: (): unknown => cleared,
      },
    },
  },);
  await memoized({
    args: ['a'],
  },);
  pMemoizeClear(memoized,);
  await yieldTurn();
}

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

    it({
      name: 'awaits a function-valued thenable returned by clear',
      fn: async () => {
        /**
         Function carrying a callable `then`, as a thenable class constructor would.
         */
        function thenableFunction(): void {}
        const scripted = createScriptedThenable({
          answers: [function answer(): void {}],
          base: thenableFunction,
        },);

        await clearWithResult({
          cleared: scripted.value,
        },);

        expect(scripted.invocations(),).toBe(1,);
      },
    },),

    it({
      name: 'awaits an object thenable returned by clear',
      fn: async () => {
        const scripted = createScriptedThenable({
          answers: [function answer(): void {}],
          base: {},
        },);

        await clearWithResult({
          cleared: scripted.value,
        },);

        expect(scripted.invocations(),).toBe(1,);
      },
    },),

    it({
      name: 'does not await a clear result whose then is not callable',
      fn: async () => {
        // The first read answers with a number and later reads with a
        // function: a result that were awaited anyway would read again and
        // call that function.
        const scripted = createScriptedThenable({
          answers: [1, function answer(): void {}],
          base: {},
        },);

        await clearWithResult({
          cleared: scripted.value,
        },);

        expect(scripted.reads(),).toBe(1,);
        expect(scripted.invocations(),).toBe(0,);
      },
    },),

    it({
      name: 'decides from a single read of then whether a clear result is awaited',
      fn: async () => {
        // The first read answers with a function and later reads with a
        // number: the decision to await was taken on the first read, and the
        // await's own read finds nothing to call.
        const scripted = createScriptedThenable({
          answers: [function answer(): void {}, 1],
          base: {},
        },);

        await clearWithResult({
          cleared: scripted.value,
        },);

        expect(scripted.reads(),).toBe(2,);
        expect(scripted.invocations(),).toBe(0,);
      },
    },),
  ],
},);
