/**
 Tests for `iterableOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { iterableOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: iterableOrThrow.name,
  children: [
    it({
      name: 'returns sync-iterable values unchanged',
      fn: async () => {
        expect(iterableOrThrow('abc',),).toBe('abc',);
        const arr = [1, 2, 3,];
        expect(iterableOrThrow(arr,),).toBe(arr,);
        const set = new Set([1, 2,],);
        expect(iterableOrThrow(set,),).toBe(set,);
        const map = new Map();
        expect(iterableOrThrow(map,),).toBe(map,);
        const gen = (function* yieldOne() {
          yield 1;
        })();
        expect(iterableOrThrow(gen,),).toBe(gen,);
      },
    },),

    it({
      name: 'throws on non-iterable values',
      fn: async () => {
        expect(() => iterableOrThrow(42,)).toThrow('iterable',);
        expect(() => iterableOrThrow(true,)).toThrow('iterable',);
        expect(() => iterableOrThrow(null,)).toThrow('iterable',);
        expect(() => iterableOrThrow(undefined,)).toThrow('iterable',);
        expect(() => iterableOrThrow({ a: 1, },)).toThrow('iterable',);
      },
    },),

    it({
      name: 'narrows unknown to Iterable<unknown>',
      fn: async () => {
        const input: unknown = [1, 2,];
        const output = iterableOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<Iterable<unknown>>();
      },
    },),
  ],
},);
