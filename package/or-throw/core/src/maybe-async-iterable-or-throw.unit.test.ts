/**
 Tests for `maybeAsyncIterableOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { maybeAsyncIterableOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: maybeAsyncIterableOrThrow.name,
  children: [
    it({
      name: 'returns sync-iterable values unchanged',
      fn: async () => {
        expect(maybeAsyncIterableOrThrow('abc',),).toBe('abc',);
        const arr = [1, 2,];
        expect(maybeAsyncIterableOrThrow(arr,),).toBe(arr,);
        const set = new Set();
        expect(maybeAsyncIterableOrThrow(set,),).toBe(set,);
      },
    },),

    it({
      name: 'returns async-iterable values unchanged',
      fn: async () => {
        const gen = (async function* yieldOne() {
          yield 1;
        })();
        expect(maybeAsyncIterableOrThrow(gen,),).toBe(gen,);
      },
    },),

    it({
      name: 'throws on values that implement neither protocol',
      fn: async () => {
        expect(() => maybeAsyncIterableOrThrow(42,)).toThrow('iterable',);
        expect(() => maybeAsyncIterableOrThrow(null,)).toThrow('iterable',);
        expect(() => maybeAsyncIterableOrThrow(undefined,)).toThrow('iterable',);
        expect(() => maybeAsyncIterableOrThrow({ a: 1, },)).toThrow('iterable',);
      },
    },),
  ],
},);
