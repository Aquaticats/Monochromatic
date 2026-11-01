/**
 Tests for `pMapSkip` staging inside the concurrent map: which mapper
 results reach the collected results and which are dropped before the run
 resolves (`pMapSkip` itself is package-internal).
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  pMap,
  pMapSkip,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'pMap skip staging',
  children: [
    it({
      name: 'resolves the staged results directly when nothing is skipped',
      fn: async () => {
        const results = await pMap({
          iterable: [
            1,
            2,
          ],
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
      name: 'drops the staged skip and keeps its neighbours in order',
      fn: async () => {
        const results = await pMap({
          iterable: [
            1,
            2,
            3,
          ],
          mapper: function dropMiddle(value: number,): number | typeof pMapSkip {
            return (value === 2)
              ? pMapSkip
              : value;
          },
        },);
        expect(results,).toEqual([
          1,
          3,
        ],);
      },
    },),

    it({
      name: 'keeps only the unskipped values when skips outnumber results',
      fn: async () => {
        const results = await pMap({
          iterable: [
            1,
            2,
            3,
            4,
            5,
            6,
          ],
          mapper: function dropEvens(value: number,): number | typeof pMapSkip {
            return ((value % 2) === 0)
              ? pMapSkip
              : value;
          },
        },);
        expect(results,).toEqual([
          1,
          3,
          5,
        ],);
      },
    },),

    it({
      name: 'resolves an empty result when every input is skipped',
      fn: async () => {
        const results = await pMap({
          iterable: [
            1,
            2,
            3,
          ],
          mapper: function skipAll(): typeof pMapSkip {
            return pMapSkip;
          },
        },);
        expect(results,).toEqual([],);
      },
    },),

    it({
      name: 'drops a single skipped input',
      fn: async () => {
        const results = await pMap({
          iterable: [1],
          mapper: function skipOne(): typeof pMapSkip {
            return pMapSkip;
          },
        },);
        expect(results,).toEqual([],);
      },
    },),

    it({
      name: 'never stages a skip for a rejected mapper call',
      fn: async () => {
        /**
         Failure observed from the run's promise.
         */
        let caught: unknown;
        try {
          await pMap({
            iterable: [
              1,
              2,
            ],
            mapper: function skipThenFail(value: number,): number | typeof pMapSkip {
              if (value === 1)
                return pMapSkip;
              throw new Error('mapper failed',);
            },
          },);
        }
        catch (error) {
          caught = error;
        }
        expect((caught as Error).message,).toBe('mapper failed',);
      },
    },),
  ],
},);
