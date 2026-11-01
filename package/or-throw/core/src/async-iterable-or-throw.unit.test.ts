/**
 Tests for `asyncIterableOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { asyncIterableOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: asyncIterableOrThrow.name,
  children: [
    it({
      name: 'returns async-iterable values unchanged',
      fn: async () => {
        const gen = (async function* yieldOne() {
          yield 1;
        })();
        expect(asyncIterableOrThrow(gen,),).toBe(gen,);
      },
    },),

    it({
      name: 'throws on sync-iterable-but-not-async values',
      fn: async () => {
        expect(() => asyncIterableOrThrow([1, 2,],)).toThrow('async iterable',);
        expect(() => asyncIterableOrThrow('abc',)).toThrow('async iterable',);
        expect(() => asyncIterableOrThrow(new Set(),)).toThrow('async iterable',);
      },
    },),

    it({
      name: 'throws on non-iterable values',
      fn: async () => {
        expect(() => asyncIterableOrThrow(42,)).toThrow('async iterable',);
        expect(() => asyncIterableOrThrow(null,)).toThrow('async iterable',);
        expect(() => asyncIterableOrThrow(undefined,)).toThrow('async iterable',);
        expect(() => asyncIterableOrThrow({},)).toThrow('async iterable',);
      },
    },),

    it({
      name: 'narrows unknown to AsyncIterable<unknown>',
      fn: async () => {
        const input: unknown = (async function* yieldOne() {
          yield 1;
        })();
        const output = asyncIterableOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<AsyncIterable<unknown>>();
      },
    },),
  ],
},);
