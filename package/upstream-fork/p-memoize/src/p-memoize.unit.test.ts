/**
 Tests for core memoization semantics: keying, in-flight sharing, rejection
 handling, argument forwarding, and receiver forwarding.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type MemoizedCall,
  type MemoizedFunction,
  pMemoize,
} from '../dist/final/neutral/index.mjs';

import {
  createDeferred,
  promiseState,
} from './test-support.ts';

await describe({
  name: 'p-memoize',
  children: [
    it({
      name: 'keys the cache by the first argument across value types',
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
        async function fixture(
          _first?: unknown,
          _second?: unknown,
        ): Promise<number> {
          return state.index++;
        }
        const memoized = pMemoize({
          fn: fixture,
        });

        expect(await memoized({
          args: [],
        },),).toBe(0,);
        expect(await memoized({
          args: [],
        },),).toBe(0,);
        expect(await memoized({
          args: [undefined],
        },),).toBe(0,);
        expect(await memoized({
          args: ['foo'],
        },),).toBe(1,);
        expect(await memoized({
          args: ['foo', 'bar'],
        },),).toBe(1,);
        expect(await memoized({
          args: [1],
        },),).toBe(2,);
        expect(await memoized({
          args: [null],
        },),).toBe(3,);
        expect(await memoized({
          args: [fixture],
        },),).toBe(4,);
        expect(await memoized({
          args: [fixture],
        },),).toBe(4,);
        expect(await memoized({
          args: [true],
        },),).toBe(5,);
      },
    },),

    it({
      name: 'stores functions by reference, not by their string form',
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
        async function fixture(_key?: unknown,): Promise<number> {
          return state.index++;
        }
        const memoized = pMemoize({
          fn: fixture,
        });

        expect(await memoized({
          args: [function firstKey(): void {}],
        },),).toBe(0,);
        expect(await memoized({
          args: [function secondKey(): void {}],
        },),).toBe(1,);
      },
    },),

    it({
      name: 'shares one in-flight promise between concurrent calls',
      fn: async () => {
        /**
         Deferred the wrapped call waits on, so both callers stay in flight.
         */
        const deferred = createDeferred<boolean>();
        /**
         Invocation counter for the wrapped function.
         */
        const state = {
          invocations: 0,
        };
        /**
         Wrapped function blocking on the deferred gate.
         */
        async function fixture(_key: string,): Promise<boolean> {
          state.invocations += 1;
          return await deferred.promise;
        }
        const memoized = pMemoize({
          fn: fixture,
        });

        const first = memoized({
          args: ['k'],
        },);
        expect(await promiseState(first,),).toBe('pending',);

        const second = memoized({
          args: ['k'],
        },);
        expect(second,).toBe(first,);
        expect(state.invocations,).toBe(1,);

        deferred.resolve(true,);
        expect(await first,).toBe(true,);
        expect(await second,).toBe(true,);
      },
    },),

    it({
      name: 'starts the wrapped function only after the cache lookup settles',
      fn: async () => {
        /**
         Whether the wrapped function has started.
         */
        const state = {
          started: false,
        };
        /**
         Wrapped function marking its own start.
         */
        async function fixture(): Promise<string> {
          state.started = true;
          return 'done';
        }
        const memoized = pMemoize({
          fn: fixture,
        });

        const promise = memoized({
          args: [],
        },);
        expect(state.started,).toBe(false,);
        expect(await promise,).toBe('done',);
        expect(state.started,).toBe(true,);
      },
    },),

    it({
      name: 'starts the wrapped function one microtask after the call when caching is disabled, like upstream 9.0.0',
      fn: async () => {
        /**
         Whether the wrapped function has started.
         */
        const state = {
          started: false,
        };
        /**
         Wrapped function marking its own start.
         */
        async function fixture(): Promise<string> {
          state.started = true;
          return 'done';
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache: false,
          },
        });

        const promise = memoized({
          args: [],
        },);
        expect(state.started,).toBe(false,);
        expect(await promise,).toBe('done',);
        expect(state.started,).toBe(true,);
      },
    },),

    it({
      name: 'does not cache rejections',
      fn: async () => {
        /**
         Attempt counter flipping from failing to succeeding.
         */
        const state = {
          attempts: 0,
        };
        /**
         Wrapped function failing only on its first invocation.
         */
        async function fixture(): Promise<string> {
          state.attempts += 1;
          if (state.attempts === 1)
            throw new Error('boom',);
          return 'ok';
        }
        const memoized = pMemoize({
          fn: fixture,
        });

        let caught: unknown;
        try {
          await memoized({
            args: [],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toBe('boom',);

        expect(await memoized({
          args: [],
        },),).toBe('ok',);
        expect(state.attempts,).toBe(2,);
      },
    },),

    it({
      name: 'shares one in-flight rejection for a synchronous throw when caching is disabled, then recomputes, like upstream 9.0.0',
      fn: async () => {
        /**
         Attempt counter proving when the wrapped function runs.
         */
        const state = {
          attempts: 0,
        };
        /**
         Wrapped function throwing before it can return a promise.
         */
        function fixture(): Promise<string> {
          state.attempts += 1;
          throw new Error('sync boom',);
        }
        const memoized = pMemoize({
          fn: fixture,
          options: {
            cache: false,
          },
        });

        const first = memoized({
          args: [],
        },);
        const second = memoized({
          args: [],
        },);
        expect(second,).toBe(first,);
        // The disabled-cache call yields one microtask before running the
        // wrapped function, so nothing ran during the synchronous launch.
        expect(state.attempts,).toBe(0,);

        let caught: unknown;
        try {
          await second;
        }
        catch (error) {
          caught = error;
        }
        expect((caught as Error).message,).toBe('sync boom',);
        expect(state.attempts,).toBe(1,);

        /**
         Call after the first settled: the in-flight entry is gone, so it
         recomputes instead of replaying the earlier rejection.
         */
        const third = memoized({
          args: [],
        },);
        expect(third,).not.toBe(first,);
        let thirdCaught: unknown;
        try {
          await third;
        }
        catch (error) {
          thirdCaught = error;
        }
        expect((thirdCaught as Error).message,).toBe('sync boom',);
        expect(state.attempts,).toBe(2,);
      },
    },),

    it({
      name: 'spreads the args tuple into the wrapped function',
      fn: async () => {
        /**
         Wrapped function joining its two arguments.
         */
        async function fixture(left: string, right: string,): Promise<string> {
          return `${left}-${right}`;
        }
        const memoized = pMemoize({
          fn: fixture,
        });

        expect(await memoized({
          args: [
            'a',
            'b',
          ],
        },),).toBe('a-b',);
      },
    },),

    it({
      name: 'forwards the receiver into the wrapped function shared across receivers',
      fn: async () => {
        /**
         Wrapped method before memoization: reads and bumps the receiver's
         counter.
         */
        async function unwrappedFoo(this: {
          index: number;
        }, _key: string,): Promise<number> {
          return this.index++;
        }
        /**
         Memoized method shared by both receivers below.
         */
        const memoizedMethod = pMemoize({
          fn: unwrappedFoo,
        },);
        /**
         First receiver sharing the memoized method.
         */
        const alpha = {
          index: 0,
          foo: memoizedMethod,
        };
        /**
         Second receiver whose own counter the shared method must read.
         */
        const beta = {
          index: 100,
          foo: memoizedMethod,
        };

        expect(await alpha.foo({
          args: ['alpha'],
        },),).toBe(0,);
        expect(await alpha.foo({
          args: ['alpha'],
        },),).toBe(0,);
        expect(await beta.foo({
          args: ['beta'],
        },),).toBe(100,);
        expect(await beta.foo({
          args: ['beta'],
        },),).toBe(100,);
      },
    },),

    it({
      name: 'keeps the wrapper toString marker and copies the source name',
      fn: async () => {
        /**
         Wrapped function whose identity the wrapper adopts.
         */
        async function fixture(): Promise<number> {
          return 1;
        }
        const memoized = pMemoize({
          fn: fixture,
        },);

        expect(memoized.toString(),).toBe(`/* Wrapped with memoized() */\n${fixture.toString()}`,);
        expect(memoized.name,).toBe('fixture',);
      },
    },),

    it({
      name: 'types the memoized function as MemoizedFunction and calls as MemoizedCall',
      fn: async () => {
        /**
         Wrapped function measuring its key's length.
         */
        async function fixture(key: string,): Promise<number> {
          return key.length;
        }
        const memoized: MemoizedFunction<[string], number> = pMemoize({
          fn: fixture,
        });
        /**
         Typed call handing the wrapped function its argument tuple.
         */
        const call: MemoizedCall<[string]> = {
          args: ['ab'],
        };
        expect(await memoized(call,),).toBe(2,);
      },
    },),
  ],
},);
