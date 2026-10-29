/**
 Tests for the source puller behind both concurrent maps: a source that
 reported `done` is never pulled again, and a source answering with something
 that is not an iterator result fails where upstream `p-map` does.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  pMap,
  pMapIterable,
} from '../dist/final/neutral/index.mjs';

import { rejectionOf, } from './test-support.ts';

//region Fixtures

/**
 Pull counter written by the instrumented sources.
 */
type PullCount = {
  pulls: number;
};

/**
 Builds a synchronous source over fixed values that counts its pulls.

 @param values - Values yielded in order.

 @param count - Counter every pull increments.

 @returns Iterable whose iterator counts every `next()` call.

 @example
 ```ts
 const source = countedSyncSource([1,], { pulls: 0, },);
 ```
 */
function countedSyncSource(
  values: readonly number[],
  count: PullCount,
): Iterable<number> {
  return {
    [Symbol.iterator]: function openCountedIterator(): Iterator<number> {
      /**
       Cursor shared by this iterator's pulls.
       */
      const cursor = { position: 0, };
      return {
        next: function countedNext(): IteratorResult<number> {
          count.pulls += 1;
          if (cursor.position >= values.length)
            return {
              done: true,
              value: undefined,
            };
          /**
           Value at the cursor, proven present by the bound check above.
           */
          const value = values[cursor.position] ?? 0;
          cursor.position += 1;
          return {
            done: false,
            value,
          };
        },
      };
    },
  };
}

/**
 Builds a synchronous source whose only answer is not an iterator result.

 @param answer - What the source's `next()` hands back.

 @returns Iterable whose first pull answers with `answer`.

 @example
 ```ts
 const source = nonResultSource(null,);
 ```
 */
function nonResultSource(answer: unknown,): Iterable<number> {
  return {
    [Symbol.iterator]: function openNonResultIterator(): Iterator<number> {
      return {
        next: function nonResultNext(): IteratorResult<number> {
          return answer as unknown as IteratorResult<number>;
        },
      };
    },
  };
}

/**
 Maps a number to itself.

 @param value - Number to hand back.

 @returns The number.

 @example
 ```ts
 identity(1,); // => 1
 ```
 */
function identity(value: number,): number {
  return value;
}

/**
 Builds the test that a non-result answer fails like upstream `p-map`.

 @param answer - What the source's `next()` hands back.

 @returns Test over a run on that source.

 @example
 ```ts
 const test = nonResultTest(null,);
 ```
 */
function nonResultTest(answer: unknown,): ReturnType<typeof it> {
  return it({
    name: `fails with the property access error of ${String(answer,)}, like upstream p-map`,
    fn: async () => {
      /**
       Failure observed from the run's promise.
       */
      const caught = await rejectionOf(pMap({
        iterable: nonResultSource(answer,),
        mapper: identity,
      },),);

      expect(caught,).toBeInstanceOf(TypeError,);
      expect((caught as TypeError).message,).toContain('\'done\'',);
    },
  },);
}

//endregion Fixtures

await describe({
  name: 'source puller',
  children: [
    describe({
      name: 'a source that reported done',
      children: [
        it({
          name: 'is not pulled again by the concurrent map',
          fn: async () => {
            /**
             Pull counter for the source.
             */
            const count: PullCount = { pulls: 0, };
            /**
             Failure observed from the run's promise.
             */
            const caught = await rejectionOf(pMap({
              iterable: countedSyncSource(
                [1],
                count,
              ),
              mapper: function failing(): number {
                throw new Error('mapper failed',);
              },
              options: {
                concurrency: 2,
                stopOnError: false,
              },
            },),);

            // The run rejects with an `AggregateError`; only the pulls matter here.
            expect(caught,).toBeInstanceOf(AggregateError,);
            // One pull for the element, one for its `done`, and none after.
            expect(count.pulls,).toBe(2,);
          },
        },),

        it({
          name: 'is not pulled again by the streaming map',
          fn: async () => {
            /**
             Pull counter for the source.
             */
            const count: PullCount = { pulls: 0, };
            /**
             Results collected from the stream.
             */
            const results: number[] = [];
            for await (const result of pMapIterable({
              iterable: countedSyncSource(
                [
                  1,
                  2,
                ],
                count,
              ),
              mapper: identity,
              options: { concurrency: 4, },
            },))
              results.push(result,);

            expect(results,).toEqual([
              1,
              2,
            ],);
            // Two pulls for the elements, one for the `done`, and none after.
            expect(count.pulls,).toBe(3,);
          },
        },),
      ],
    },),

    describe({
      name: 'a source answering with something that is not an iterator result',
      children: [
        nonResultTest(null,),
        nonResultTest(undefined,),
      ],
    },),
  ],
},);
