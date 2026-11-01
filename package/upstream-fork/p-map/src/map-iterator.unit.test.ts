/**
 Tests for source iterator selection and shutdown, exercised through the
 public `pMap` and `pMapIterable` entry points (iterator handling is
 package-internal).
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  InvalidInputError,
  pMap,
  pMapIterable,
} from '../dist/final/neutral/index.mjs';

import {
  createGate,
  yieldTurn,
} from './test-support.ts';

//region Fixtures

/**
 Telemetry recorded by the tracked iterable fixtures.
 */
type TrackingState = {
  /**
   Whether the source's `return` method ran.
   */
  returnCalled: boolean;
};

/**
 Builds a synchronous iterable of `1..count` whose `return` method records
 its invocation into the supplied telemetry.
 
 @param bounds - How many values to yield and where to record the close.
 
 @returns Iterable whose shutdown is observable.
 
 @example
 ```ts
 const iterable = countingIterable({
   count: 2,
   tracking: { returnCalled: false, },
 });
 ```
 */
function countingIterable(
  {
    count,
    tracking,
  }: {
    /**
     Number of values yielded before `done`.
     */
    readonly count: number;
    /**
     Telemetry the close request writes to.
     */
    readonly tracking: TrackingState;
  },
): Iterable<number> {
  return {
    [Symbol.iterator]: function openCountingIterator(): Iterator<number> {
      /**
       Pull cursor shared by this iterator's `next` calls.
       */
      const cursor = {
        position: 0,
      };
      return {
        next: function countingNext(): IteratorResult<number> {
          if (cursor.position >= count)
            return {
              done: true,
              value: undefined,
            };
          cursor.position += 1;
          return {
            done: false,
            value: cursor.position,
          };
        },
        return: function countingReturn(): IteratorResult<number> {
          tracking.returnCalled = true;
          return {
            done: true,
            value: undefined,
          };
        },
      };
    },
  };
}

/**
 Builds a source carrying both iterator slots with distinguishable values,
 so selection precedence is observable.
 
 @returns Input whose async slot yields `2` and sync slot yields `1`.
 
 @example
 ```ts
 const results = await pMap({ iterable: bothSlotsIterable(), mapper, });
 ```
 */
function bothSlotsIterable(): {
  readonly [Symbol.iterator]: () => Iterator<number>;
  readonly [Symbol.asyncIterator]: () => AsyncIterator<number>;
} {
  return {
    [Symbol.iterator]: function openSyncSlot(): Iterator<number> {
      const cursor = {
        position: 0,
      };
      return {
        next: function syncNext(): IteratorResult<number> {
          if (cursor.position >= 1)
            return {
              done: true,
              value: undefined,
            };
          cursor.position += 1;
          return {
            done: false,
            value: 1,
          };
        },
      };
    },
    [Symbol.asyncIterator]: function openAsyncSlot(): AsyncIterator<number> {
      const cursor = {
        position: 0,
      };
      return {
        next: async function asyncNext(): Promise<IteratorResult<number>> {
          if (cursor.position >= 1)
            return {
              done: true,
              value: undefined,
            };
          cursor.position += 1;
          return {
            done: false,
            value: 2,
          };
        },
      };
    },
  };
}

//endregion Fixtures

/**
 Builds a synchronous iterable of one value whose `return` method records
 its invocation and then throws, exercising the swallowed close failure.
 
 @param tracking - Telemetry the close request writes to.
 
 @returns Iterable whose shutdown fails loudly but harmlessly.
 
 @example
 ```ts
 const iterable = closeFailingIterable({ returnCalled: false, });
 ```
 */
function closeFailingIterable(tracking: TrackingState,): Iterable<number> {
  return {
    [Symbol.iterator]: function openCloseFailingIterator(): Iterator<number> {
      const cursor = {
        position: 0,
      };
      return {
        next: function closeFailingNext(): IteratorResult<number> {
          if (cursor.position >= 1)
            return {
              done: true,
              value: undefined,
            };
          cursor.position += 1;
          return {
            done: false,
            value: 1,
          };
        },
        return: function closeFailingReturn(): IteratorResult<number> {
          tracking.returnCalled = true;
          throw new Error('close failed',);
        },
      };
    },
  };
}

await describe({
  name: 'input and iterator handling',
  children: [
    describe({
      name: 'input validation',
      children: [
        it({
          name: 'accepts a synchronous iterable',
          fn: async () => {
            const results = await pMap({
              iterable: [
                4,
                5,
              ],
              mapper: function identity(value: number,): number {
                return value;
              },
            },);
            expect(results,).toEqual([
              4,
              5,
            ],);
          },
        },),

        it({
          name: 'accepts an asynchronous iterable',
          fn: async () => {
            /**
             Asynchronous source yielding two values.
             */
            const asyncIterable: AsyncIterable<number> = {
              [Symbol.asyncIterator]: function openAsyncIterable(): AsyncIterator<number> {
                const cursor = {
                  position: 0,
                };
                return {
                  next: async function asyncNext(): Promise<IteratorResult<number>> {
                    if (cursor.position >= 2)
                      return {
                        done: true,
                        value: undefined,
                      };
                    cursor.position += 1;
                    return {
                      done: false,
                      value: cursor.position,
                    };
                  },
                };
              },
            };
            const results = await pMap({
              iterable: asyncIterable,
              mapper: function identity(value: number,): number {
                return value;
              },
            },);
            expect(results,).toEqual([
              1,
              2,
            ],);
          },
        },),

        it({
          name: 'rejects a non-iterable input with upstream p-map message text',
          fn: async () => {
            /**
             Failure observed from the run's promise.
             */
            let caught: unknown;
            try {
              await pMap({
                iterable: 5 as never,
                mapper: function identity(value: number,): number {
                  return value;
                },
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(InvalidInputError,);
            expect((caught as Error).message,).toBe('Expected `input` to be either an `Iterable` or `AsyncIterable`, got (number)',);
          },
        },),

        it({
          name: 'rejects a null input with the same message as any other wrong input, like upstream p-map 7.1.0',
          fn: async () => {
            /**
             Failure observed from the run's promise.
             */
            let caught: unknown;
            try {
              await pMap({
                iterable: null as never,
                mapper: function identity(value: number,): number {
                  return value;
                },
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(InvalidInputError,);
            expect((caught as Error).message,).toBe('Expected `input` to be either an `Iterable` or `AsyncIterable`, got (object)',);
          },
        },),

        it({
          name: 'rejects an undefined input with the same message as any other wrong input, like upstream p-map 7.1.0',
          fn: async () => {
            /**
             Failure observed from the run's promise.
             */
            let caught: unknown;
            try {
              await pMap({
                iterable: undefined as never,
                mapper: function identity(value: number,): number {
                  return value;
                },
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(InvalidInputError,);
            expect((caught as Error).message,).toBe('Expected `input` to be either an `Iterable` or `AsyncIterable`, got (undefined)',);
          },
        },),
      ],
    },),

    describe({
      name: 'iterator selection',
      children: [
        it({
          name: 'prefers the asynchronous slot when the input carries both',
          fn: async () => {
            const results = await pMap({
              iterable: bothSlotsIterable(),
              mapper: function identity(value: number,): number {
                return value;
              },
            },);
            expect(results,).toEqual([2],);
          },
        },),
      ],
    },),

    describe({
      name: 'iterator shutdown',
      children: [
        it({
          name: 'closes the source when the run rejects before the source is done',
          fn: async () => {
            /**
             Close telemetry for this source.
             */
            const tracking: TrackingState = {
              returnCalled: false,
            };
            /**
             Gate holding the single mapper call until the test releases it.
             */
            const gate = createGate();
            /**
             Run promise observed for its rejection.
             */
            const running = pMap({
              iterable: countingIterable({
                count: 2,
                tracking,
              },),
              mapper: async function gatedFailure(value: number,): Promise<never> {
                await gate.open;
                throw new Error(`mapper failed ${String(value,)}`,);
              },
              options: {
                concurrency: 1,
                stopOnError: true,
              },
            },);
            gate.release();
            /**
             Failure observed from the run's promise.
             */
            let caught: unknown;
            try {
              await running;
            }
            catch (error) {
              caught = error;
            }
            expect((caught as Error).message,).toBe('mapper failed 1',);
            await yieldTurn();
            expect(tracking.returnCalled,).toBe(true,);
          },
        },),

        it({
          name: 'leaves an exhausted source open rather than closing it twice',
          fn: async () => {
            /**
             Close telemetry for this source.
             */
            const tracking: TrackingState = {
              returnCalled: false,
            };
            await pMap({
              iterable: countingIterable({
                count: 2,
                tracking,
              },),
              mapper: function identity(value: number,): number {
                return value;
              },
            },);
            await yieldTurn();
            expect(tracking.returnCalled,).toBe(false,);
          },
        },),

        it({
          name: 'swallows a source close failure so the run rejection is untouched',
          fn: async () => {
            /**
             Close telemetry for this source; its `return` throws.
             */
            const tracking: TrackingState = {
              returnCalled: false,
            };
            /**
             Gate holding the single mapper call until the test releases it.
             */
            const gate = createGate();
            /**
             Run promise observed for its rejection.
             */
            const running = pMap({
              iterable: closeFailingIterable(tracking,),
              mapper: async function gatedFailure(): Promise<never> {
                await gate.open;
                throw new Error('mapper failed',);
              },
              options: { concurrency: 1, },
            },);
            gate.release();
            /**
             Failure observed from the run's promise.
             */
            let caught: unknown;
            try {
              await running;
            }
            catch (error) {
              caught = error;
            }
            expect((caught as Error).message,).toBe('mapper failed',);
            await yieldTurn();
            expect(tracking.returnCalled,).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
