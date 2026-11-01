/**
 Tests for the robustness behavior ported from upstream `p-memoize` 9.0.0:
 re-entrant calls join the running call, a failing cache method leaves no
 stuck in-flight entry, a failing function-identity copy does not prevent
 memoizing, options are read once, and thenables are awaited.

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
  rejectionOf,
  yieldTurn,
} from './test-support.ts';

await describe({
  name: 'pMemoize robustness',
  children: [
    describe({
      name: 're-entrant calls',
      children: [
        it({
          name: 'joins the in-flight promise instead of running again',
          fn: async () => {
            const state: {
              invocations: number;
              nested?: Promise<number>;
            } = {
              invocations: 0,
            };
            async function fixture(): Promise<number> {
              state.invocations += 1;
              state.nested ??= memoized({
                args: [],
              },);
              return state.invocations;
            }
            const memoized = pMemoize({
              fn: fixture,
              options: {
                cache: false,
              },
            },);

            expect(await memoized({
              args: [],
            },),).toBe(1,);
            expect(await state.nested,).toBe(1,);
            expect(state.invocations,).toBe(1,);
          },
        },),

        it({
          name: 'joins the in-flight promise when the call comes from cache.has',
          fn: async () => {
            const store = new Map<unknown, unknown>();
            const state: {
              invocations: number;
              shouldReenter: boolean;
              reentered?: Promise<number>;
            } = {
              invocations: 0,
              shouldReenter: false,
            };
            async function fixture(_value: string,): Promise<number> {
              state.invocations += 1;
              return state.invocations;
            }
            const memoized = pMemoize({
              fn: fixture,
              options: {
                cache: {
                  has: (key: unknown,): boolean => {
                    if (state.shouldReenter) {
                      state.shouldReenter = false;
                      state.reentered = memoized({
                        args: ['a'],
                      },);
                    }

                    return store.has(key,);
                  },
                  get: (key: unknown,): unknown => store.get(key,),
                  set: (key: unknown, value: unknown,): void => {
                    store.set(key, value,);
                  },
                },
              } as never,
            },);

            state.shouldReenter = true;
            expect(await memoized({
              args: ['a'],
            },),).toBe(1,);
            expect(await state.reentered,).toBe(1,);
            expect(state.invocations,).toBe(1,);
          },
        },),

        it({
          name: 'joins one execution for several re-entrant calls',
          fn: async () => {
            const state: {
              invocations: number;
              pending: number;
              reentered: Promise<number>[];
            } = {
              invocations: 0,
              pending: 2,
              reentered: [],
            };
            async function fixture(): Promise<number> {
              state.invocations += 1;

              while (state.pending > 0) {
                state.pending -= 1;
                state.reentered.push(memoized({
                  args: [],
                },),);
              }

              return state.invocations;
            }
            const memoized = pMemoize({
              fn: fixture,
              options: {
                cache: false,
              },
            },);

            expect(await memoized({
              args: [],
            },),).toBe(1,);
            expect(await Promise.all(state.reentered,),).toEqual([
              1,
              1,
            ],);
            expect(state.invocations,).toBe(1,);
          },
        },),

        it({
          name: 'joins one execution for two re-entrant calls from the cache reads',
          fn: async () => {
            const store = new Map<string, number>();
            const state: {
              invocations: number;
              pending: number;
              reentered: Promise<number>[];
            } = {
              invocations: 0,
              pending: 2,
              reentered: [],
            };
            /**
             Starts a re-entrant call while any are still pending.
             */
            function reenterWhilePending(): void {
              if (state.pending > 0) {
                state.pending -= 1;
                state.reentered.push(memoized({
                  args: ['a'],
                },),);
              }
            }
            async function fixture(_key: string,): Promise<number> {
              state.invocations += 1;
              return state.invocations;
            }
            const memoized = pMemoize({
              fn: fixture,
              options: {
                cache: {
                  has: (key: string,): boolean => {
                    reenterWhilePending();
                    return store.has(key,);
                  },
                  get: (key: string,) => {
                    reenterWhilePending();
                    return store.get(key,);
                  },
                  set: (key: string, value: number,): void => {
                    store.set(key, value,);
                  },
                },
              },
            },);

            expect(await memoized({
              args: ['a'],
            },),).toBe(1,);
            expect(await Promise.all(state.reentered,),).toEqual([
              1,
              1,
            ],);
            expect(state.invocations,).toBe(1,);
          },
        },),

        it({
          name: 're-enters itself for a different key',
          fn: async () => {
            const state = {
              invocations: 0,
            };
            async function fixture(key: string,): Promise<string> {
              state.invocations += 1;
              if (key === 'a')
                await memoized({
                  args: ['b'],
                },);

              return key;
            }
            const memoized = pMemoize({
              fn: fixture,
            },);

            expect(await memoized({
              args: ['a'],
            },),).toBe('a',);
            expect(state.invocations,).toBe(2,);
          },
        },),
      ],
    },),

    describe({
      name: 'failing cache methods',
      children: [
        it({
          name: 'runs the function when get rejects for an absent key',
          fn: async () => {
            const state = {
              invocations: 0,
            };
            async function fixture(_value: string,): Promise<number> {
              state.invocations += 1;
              return state.invocations;
            }
            const memoized = pMemoize({
              fn: fixture,
              options: {
                cache: {
                  has: (): boolean => false,
                  get: async (): Promise<undefined> => {
                    throw new Error('get boom',);
                  },
                  set: (): undefined => undefined,
                },
              },
            },);

            expect(await memoized({
              args: ['a'],
            },),).toBe(1,);
            expect(state.invocations,).toBe(1,);
          },
        },),

        it({
          name: 'fails the call when get rejects on a hit',
          fn: async () => {
            async function fixture(_value: string,): Promise<string> {
              return 'value';
            }
            const memoized = pMemoize({
              fn: fixture,
              options: {
                cache: {
                  has: (): boolean => true,
                  get: async (): Promise<undefined> => {
                    throw new Error('get boom',);
                  },
                  set: (): undefined => undefined,
                },
              },
            },);

            const reason = await rejectionOf(memoized({
              args: ['a'],
            },),);
            expect((reason as Error).message,).toBe('get boom',);
          },
        },),

        it({
          name: 'leaves no unhandled rejection behind when has rejects',
          fn: async () => {
            const failures: unknown[] = [];
            function onUnhandledRejection(reason: unknown,): void {
              failures.push(reason,);
            }
            async function fixture(_value: string,): Promise<number> {
              return 1;
            }

            process.on('unhandledRejection', onUnhandledRejection,);

            const memoized = pMemoize({
              fn: fixture,
              options: {
                cache: {
                  has: async (): Promise<boolean> => {
                    throw new Error('has boom',);
                  },
                  get: (): undefined => undefined,
                  set: (): undefined => undefined,
                },
              },
            },);
            const reason = await rejectionOf(memoized({
              args: ['a'],
            },),);
            await yieldTurn();
            process.off('unhandledRejection', onUnhandledRejection,);

            expect((reason as Error).message,).toBe('has boom',);
            expect(failures,).toEqual([],);
          },
        },),

        it({
          name: 'runs the function when get throws synchronously for an absent key',
          fn: async () => {
            const store = new Map<string, number>();
            const state = {
              invocations: 0,
            };
            async function fixture(_value: string,): Promise<number> {
              state.invocations += 1;
              return state.invocations;
            }
            const memoized = pMemoize({
              fn: fixture,
              options: {
                cache: {
                  has: (key: string,): boolean => store.has(key,),
                  get: (key: string,) => {
                    if (!store.has(key,))
                      throw new Error('missing',);

                    return store.get(key,);
                  },
                  set: (key: string, value: number,): void => {
                    store.set(key, value,);
                  },
                },
              },
            },);

            expect(await memoized({
              args: ['a'],
            },),).toBe(1,);
            expect(await memoized({
              args: ['a'],
            },),).toBe(1,);
            expect(state.invocations,).toBe(1,);
          },
        },),

        it({
          name: 'does not poison the key when has throws synchronously once',
          fn: async () => {
            // A synchronous throw from `has` settles the call before it is
            // registered as the in-flight one. Registering it anyway would
            // replay the failure for every later call.
            const state = {
              invocations: 0,
              shouldThrow: true,
            };
            async function fixture(_value: string,): Promise<number> {
              state.invocations += 1;
              return state.invocations;
            }
            const memoized = pMemoize({
              fn: fixture,
              options: {
                cache: {
                  has: (): boolean => {
                    if (state.shouldThrow) {
                      state.shouldThrow = false;
                      throw new Error('has boom',);
                    }

                    return false;
                  },
                  get: (): undefined => undefined,
                  set: (): undefined => undefined,
                },
              },
            },);

            const reason = await rejectionOf(memoized({
              args: ['a'],
            },),);
            expect((reason as Error).message,).toBe('has boom',);
            expect(await memoized({
              args: ['a'],
            },),).toBe(1,);
            expect(state.invocations,).toBe(1,);
          },
        },),
      ],
    },),

    describe({
      name: 'function identity copy failures',
      children: [
        it({
          name: 'memoizes a function whose own toString cannot be redefined',
          fn: async () => {
            const state = {
              calls: 0,
            };
            const fixture: (value: number) => Promise<number> = Object.defineProperty(
              async (_value: number,): Promise<number> => {
                state.calls += 1;
                return state.calls;
              },
              'toString',
              {
                value: (): string => 'fixture',
                configurable: false,
              },
            );
            const memoized = pMemoize({
              fn: fixture,
            },);

            expect(await memoized({
              args: [1],
            },),).toBe(1,);
            expect(await memoized({
              args: [1],
            },),).toBe(1,);
            expect(state.calls,).toBe(1,);
          },
        },),

        it({
          name: 'memoizes a function whose toString throws',
          fn: async () => {
            const state = {
              calls: 0,
            };
            async function fixture(_value: number,): Promise<number> {
              state.calls += 1;
              return state.calls;
            }
            Object.defineProperty(
              fixture,
              'toString',
              {
                value: (): string => {
                  throw new Error('toString exploded',);
                },
              },
            );
            const memoized = pMemoize({
              fn: fixture,
            },);

            expect(await memoized({
              args: [1],
            },),).toBe(1,);
            expect(await memoized({
              args: [1],
            },),).toBe(1,);
            expect(state.calls,).toBe(1,);
          },
        },),

        it({
          name: 'memoizes a proxy that throws from ownKeys',
          fn: async () => {
            const state = {
              calls: 0,
            };
            async function target(_value: number,): Promise<number> {
              state.calls += 1;
              return state.calls;
            }
            const fixture: (value: number) => Promise<number> = new Proxy(
              target,
              {
                ownKeys: (): never => {
                  throw new Error('ownKeys exploded',);
                },
              },
            );
            const memoized = pMemoize({
              fn: fixture,
            },);

            expect(await memoized({
              args: [1],
            },),).toBe(1,);
            expect(await memoized({
              args: [1],
            },),).toBe(1,);
            expect(state.calls,).toBe(1,);
          },
        },),

        it({
          name: 'still clears a memoized function whose identity copy failed',
          fn: async () => {
            const fixture: () => Promise<number> = Object.defineProperty(
              async (): Promise<number> => 1,
              'toString',
              {
                value: (): string => 'fixture',
                configurable: false,
              },
            );
            const memoized = pMemoize({
              fn: fixture,
            },);

            await memoized({
              args: [],
            },);
            pMemoizeClear(memoized,);
            expect(await memoized({
              args: [],
            },),).toBe(1,);
          },
        },),
      ],
    },),

    describe({
      name: 'options and values',
      children: [
        it({
          name: 'reads shouldCache once, when the function is memoized',
          fn: async () => {
            const cache = new Map<string, number>();
            const state = {
              calls: 0,
            };
            const options: {
              cache: Map<string, number>;
              shouldCache: () => boolean;
            } = {
              cache,
              shouldCache: (): boolean => true,
            };
            async function fixture(_value: string,): Promise<number> {
              state.calls += 1;
              return state.calls;
            }
            const memoized = pMemoize({
              fn: fixture,
              options,
            },);

            await memoized({
              args: ['a'],
            },);
            expect(cache.size,).toBe(1,);

            // `cache` and `cacheKey` are captured when the function is
            // memoized, so `shouldCache` is as well: editing the options
            // object afterwards must not change how writes happen.
            options.shouldCache = (): boolean => false;
            await memoized({
              args: ['b'],
            },);
            expect(cache.size,).toBe(2,);
            expect(state.calls,).toBe(2,);
          },
        },),

        it({
          name: 'calls cacheKey for every call, including cache hits',
          fn: async () => {
            const state = {
              keyCalls: 0,
            };
            async function fixture(value: string,): Promise<string> {
              return value;
            }
            const memoized = pMemoize({
              fn: fixture,
              options: {
                cacheKey: ([first]: readonly string[],): string => {
                  state.keyCalls += 1;
                  return first ?? '';
                },
              },
            },);

            await memoized({
              args: ['a'],
            },);
            await memoized({
              args: ['a'],
            },);
            expect(state.keyCalls,).toBe(2,);
          },
        },),

        it({
          name: 'gives shouldCache the argument list of the call it decides about',
          fn: async () => {
            const seen: (readonly unknown[])[] = [];
            async function fixture(first: string, second: string,): Promise<string> {
              return `${first}${second}`;
            }
            const memoized = pMemoize({
              fn: fixture,
              options: {
                shouldCache: (
                  _value: string,
                  context: { argumentsList: readonly unknown[]; },
                ): boolean => {
                  seen.push(context.argumentsList,);
                  return true;
                },
              },
            },);

            await memoized({
              args: [
                'a',
                'b',
              ],
            },);
            expect(seen,).toEqual([
              [
                'a',
                'b',
              ],
            ],);
          },
        },),

        it({
          name: 'caches and serves back a value of undefined',
          fn: async () => {
            const state = {
              calls: 0,
            };
            async function fixture(_value: string,): Promise<undefined> {
              state.calls += 1;
              return undefined;
            }
            const memoized = pMemoize({
              fn: fixture,
            },);

            expect(await memoized({
              args: ['a'],
            },),).toBeUndefined();
            expect(await memoized({
              args: ['a'],
            },),).toBeUndefined();
            expect(state.calls,).toBe(1,);
          },
        },),

        it({
          name: 'awaits and caches a thenable returned by the memoized function',
          fn: async () => {
            const state = {
              invocations: 0,
            };
            /**
             Plain function returning a thenable that is not a promise.
             */
            function fixture(): PromiseLike<string> {
              state.invocations += 1;
              return {
                // oxlint-disable-next-line unicorn/no-thenable -- a thenable that is not a promise is the point of this fixture
                then: function then(
                  onFulfilled?: (value: string) => unknown,
                ): void {
                  onFulfilled?.('from thenable',);
                },
              } as PromiseLike<string>;
            }
            const memoized = pMemoize({
              fn: fixture,
            },);

            expect(await memoized({
              args: [],
            },),).toBe('from thenable',);
            expect(await memoized({
              args: [],
            },),).toBe('from thenable',);
            expect(state.invocations,).toBe(1,);
          },
        },),

        it({
          name: 'accepts a cache that hands back thenables instead of promises',
          fn: async () => {
            const store = new Map<string, string>();
            const state = {
              invocations: 0,
            };
            /**
             Builds a thenable that is not a promise, resolving later.

             @param value - Value the thenable fulfills with.

             @returns Thenable fulfilling with the value.
             */
            function makeThenable(value: string,): PromiseLike<string> {
              return {
                // oxlint-disable-next-line unicorn/no-thenable -- a thenable that is not a promise is the point of this fixture
                then: function then(
                  onFulfilled?: (result: string) => unknown,
                ): void {
                  queueMicrotask(function settle(): void {
                    onFulfilled?.(value,);
                  },);
                },
              } as PromiseLike<string>;
            }
            async function fixture(key: string,): Promise<string> {
              state.invocations += 1;
              return `computed:${key}:${state.invocations}`;
            }
            const memoized = pMemoize({
              fn: fixture,
              options: {
                cache: {
                  has: (key: string,): boolean => store.has(key,),
                  get: (key: string,) => store.has(key,)
                    ? makeThenable(store.get(key,) ?? '',)
                    : undefined,
                  set: (key: string, value: string,): PromiseLike<string> => {
                    store.set(key, value,);
                    return makeThenable('',);
                  },
                },
              } as never,
            },);

            expect(await memoized({
              args: ['a'],
            },),).toBe('computed:a:1',);
            expect(await memoized({
              args: ['a'],
            },),).toBe('computed:a:1',);
            expect(state.invocations,).toBe(1,);
          },
        },),
      ],
    },),
  ],
},);
