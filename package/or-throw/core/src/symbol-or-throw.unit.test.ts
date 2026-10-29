/**
 Tests for `symbolOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { symbolOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: symbolOrThrow.name,
  children: [
    it({
      name: 'returns symbols unchanged',
      fn: async () => {
        const s = Symbol('local test symbol returned unchanged',);
        expect(symbolOrThrow(s,),).toBe(s,);
        const registered = Symbol.for('global shared symbol returned unchanged',);
        expect(symbolOrThrow(registered,),).toBe(registered,);
      },
    },),

    it({
      name: 'throws on non-symbol values',
      fn: async () => {
        expect(() => symbolOrThrow('sym',)).toThrow('symbol',);
        expect(() => symbolOrThrow(0,)).toThrow('symbol',);
        expect(() => symbolOrThrow(null,)).toThrow('symbol',);
        expect(() => symbolOrThrow(undefined,)).toThrow('symbol',);
        expect(() => symbolOrThrow({},)).toThrow('symbol',);
      },
    },),

    it({
      name: 'narrows unknown to symbol',
      fn: async () => {
        const input: unknown = Symbol('unknown symbol value narrowed',);
        const output = symbolOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<symbol>();
      },
    },),
  ],
},);
