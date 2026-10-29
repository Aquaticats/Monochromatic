/**
 Tests for `setOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { setOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: setOrThrow.name,
  children: [
    it({
      name: 'returns the value when it is a Set',
      fn: async () => {
        const set = new Set([1, 2, 3,],);
        expect(setOrThrow(set,),).toBe(set,);
      },
    },),

    it({
      name: 'throws on every non-Set value',
      fn: async () => {
        expect(() => setOrThrow([],)).toThrow('Set',);
        expect(() => setOrThrow(new Map(),)).toThrow('Set',);
        expect(() => setOrThrow(new WeakSet(),)).toThrow('Set',);
        expect(() => setOrThrow({},)).toThrow('Set',);
        expect(() => setOrThrow('abc',)).toThrow('Set',);
        expect(() => setOrThrow(null,)).toThrow('Set',);
      },
    },),

    it({
      name: 'narrows unknown to Set<unknown>',
      fn: async () => {
        const input: unknown = new Set([1,],);
        const output = setOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<Set<unknown>>();
      },
    },),

    it({
      name: 'narrows union to Set branch',
      fn: async () => {
        const input: Set<string> | string[] = new Set(['a',],);
        const output = setOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<Set<string>>();
      },
    },),
  ],
},);
