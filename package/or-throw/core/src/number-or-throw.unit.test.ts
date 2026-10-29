/**
 Tests for `numberOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { numberOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: numberOrThrow.name,
  children: [
    it({
      name: 'returns primitive numbers unchanged',
      fn: async () => {
        expect(numberOrThrow(0,),).toBe(0,);
        expect(numberOrThrow(42,),).toBe(42,);
        expect(numberOrThrow(-1.5,),).toBe(-1.5,);
      },
    },),

    it({
      name: 'accepts NaN and Infinity as numbers',
      fn: async () => {
        expect(
          Number.isNaN(numberOrThrow(Number.NaN,),),
        ).toBe(true,);
        expect(numberOrThrow(Number.POSITIVE_INFINITY,),).toBe(Number.POSITIVE_INFINITY,);
      },
    },),

    it({
      name: 'throws on non-number primitives',
      fn: async () => {
        expect(() => numberOrThrow('42',)).toThrow('number',);
        expect(() => numberOrThrow(true,)).toThrow('number',);
        expect(() => numberOrThrow(1n,)).toThrow('number',);
        expect(() => numberOrThrow(null,)).toThrow('number',);
        expect(() => numberOrThrow(undefined,)).toThrow('number',);
      },
    },),

    it({
      name: 'narrows unknown to number',
      fn: async () => {
        const input: unknown = 42;
        const output = numberOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<number>();
      },
    },),
  ],
},);
