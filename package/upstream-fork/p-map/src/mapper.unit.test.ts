/**
 Tests for the input-probe and mapper contracts, exercised through the
 public `pMap` and `pMapIterable` entry points (`mapper.ts` is
 package-internal).
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  MapperRequiredError,
  pMap,
  pMapIterable,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'mapper and probe contracts',
  children: [
    describe({
      name: 'mapper validation',
      children: [
        it({
          name: 'rejects a missing mapper with upstream p-map message text',
          fn: async () => {
            /**
             Failure observed from the run's promise.
             */
            let caught: unknown;
            try {
              await pMap({
                iterable: [1],
                mapper: undefined as never,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(MapperRequiredError,);
            expect((caught as Error).message,).toBe('Mapper function is required',);
          },
        },),

        it({
          name: 'rejects a non-function mapper',
          fn: async () => {
            /**
             Failure observed from the run's promise.
             */
            let caught: unknown;
            try {
              await pMap({
                iterable: [1],
                mapper: true as never,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(MapperRequiredError,);
          },
        },),

        it({
          name: 'accepts a function mapper and runs it',
          fn: async () => {
            const results = await pMap({
              iterable: [4],
              mapper: function identity(value: number,): number {
                return value;
              },
            },);
            expect(results,).toEqual([4],);
          },
        },),

        it({
          name: 'validates the mapper synchronously for the streaming map too',
          fn: async () => {
            /**
             Failure caught at the call site, matching upstream
             `pMapIterable`'s synchronous validation.
             */
            let caught: unknown;
            try {
              pMapIterable({
                iterable: [1],
                mapper: undefined as never,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(MapperRequiredError,);
          },
        },),
      ],
    },),

    describe({
      name: 'input probing',
      children: [
        it({
          name: 'walks a synchronous input through the iterator probe',
          fn: async () => {
            const results = await pMap({
              iterable: [
                1,
                2,
              ],
              mapper: function double(value: number,): number {
                return value * 2;
              },
            },);
            expect(results,).toEqual([
              2,
              4,
            ],);
          },
        },),

        it({
          name: 'walks an asynchronous input through the iterator probe',
          fn: async () => {
            /**
             Asynchronous source yielding one value.
             */
            const asyncIterable: AsyncIterable<number> = {
              [Symbol.asyncIterator]: function openAsyncIterable(): AsyncIterator<number> {
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
                      value: 7,
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
            expect(results,).toEqual([7],);
          },
        },),
      ],
    },),
  ],
},);
