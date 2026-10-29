/**
 Tests for the package entry point's export surface.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  InvalidBackpressureError,
  InvalidConcurrencyError,
  InvalidInputError,
  MapperRequiredError,
  pMap,
  pMapIterable,
  pMapSkip,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: '',
  children: [
    it({
      name: 'exports both mapper factories, the skip sentinel, and all four error classes',
      fn: async () => {
        expect(typeof pMap,).toBe('function',);
        expect(typeof pMapIterable,).toBe('function',);
        expect(typeof pMapSkip,).toBe('symbol',);
        expect(typeof InvalidBackpressureError,).toBe('function',);
        expect(typeof InvalidConcurrencyError,).toBe('function',);
        expect(typeof InvalidInputError,).toBe('function',);
        expect(typeof MapperRequiredError,).toBe('function',);
      },
    },),

    it({
      name: 'runs a working map through the package entry point',
      fn: async () => {
        const results = await pMap({
          iterable: [
            'a',
            'b',
          ],
          mapper: function greet(name: string,): string {
            return `hello ${name}`;
          },
        },);
        expect(results,).toEqual([
          'hello a',
          'hello b',
        ],);
      },
    },),

    it({
      name: 'streams a working map through the package entry point',
      fn: async () => {
        /**
         Streamed mapper results in yield order.
         */
        const streamed: number[] = [];
        for await (const mapped of pMapIterable({
          iterable: [
            1,
            2,
          ],
          mapper: function double(value: number,): number {
            return value * 2;
          },
        }))
          streamed.push(mapped,);
        expect(streamed,).toEqual([
          2,
          4,
        ],);
      },
    },),
  ],
},);
