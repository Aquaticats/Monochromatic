/**
 Tests for `pMapSkip` result dropping in the concurrent map.
 
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
  name: 'pMapSkip handling',
  children: [
    it({
      name: 'carries a descriptive symbol description for readable traces',
      fn: async () => {
        expect(pMapSkip.description,).toBe('p-map mapper skip sentinel',);
      },
    },),

    it({
      name: 'drops one skipped input from the collected results',
      fn: async () => {
        const results = await pMap({
          iterable: [
            1,
            2,
            3,
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
        ],);
      },
    },),

    it({
      name: 'drops multiple skipped inputs from the collected results',
      fn: async () => {
        const results = await pMap({
          iterable: [
            1,
            2,
            3,
            4,
            5,
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
      name: 'resolves to an empty result when every input is skipped',
      fn: async () => {
        const results = await pMap({
          iterable: [
            1,
            2,
          ],
          mapper: function dropAll(): typeof pMapSkip {
            return pMapSkip;
          },
        },);
        expect(results,).toEqual([],);
      },
    },),

    it({
      name: 'drops skipped inputs the same way when stopOnError is false',
      fn: async () => {
        const results = await pMap({
          iterable: [
            1,
            2,
            3,
          ],
          mapper: function dropEvens(value: number,): number | typeof pMapSkip {
            return ((value % 2) === 0)
              ? pMapSkip
              : value;
          },
          options: {
            concurrency: 1,
            stopOnError: false,
          },
        },);
        expect(results,).toEqual([
          1,
          3,
        ],);
      },
    },),

    it({
      name: 'drops skipped inputs from an asynchronous source too',
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
          mapper: function dropFirst(value: number,): number | typeof pMapSkip {
            return (value === 1)
              ? pMapSkip
              : value;
          },
        },);
        expect(results,).toEqual([2],);
      },
    },),
  ],
},);
