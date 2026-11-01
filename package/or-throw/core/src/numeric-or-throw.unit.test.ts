/**
 Tests for `numericOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { numericOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: numericOrThrow.name,
  children: [
    it({
      name: 'returns primitive numbers unchanged',
      fn: async () => {
        expect(numericOrThrow(42,),).toBe(42,);
        expect(numericOrThrow(0,),).toBe(0,);
        expect(numericOrThrow(-1.5,),).toBe(-1.5,);
      },
    },),

    it({
      name: 'returns primitive bigints unchanged',
      fn: async () => {
        expect(numericOrThrow(42n,),).toBe(42n,);
        expect(numericOrThrow(0n,),).toBe(0n,);
      },
    },),

    it({
      name: 'throws on non-numeric values',
      fn: async () => {
        expect(() => numericOrThrow('42',)).toThrow('number or bigint',);
        expect(() => numericOrThrow(true,)).toThrow('number or bigint',);
        expect(() => numericOrThrow(null,)).toThrow('number or bigint',);
        expect(() => numericOrThrow(undefined,)).toThrow('number or bigint',);
        expect(() => numericOrThrow({},)).toThrow('number or bigint',);
      },
    },),

    it({
      name: 'narrows unknown to number | bigint',
      fn: async () => {
        const input: unknown = 42;
        const output = numericOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<number | bigint>();
      },
    },),
  ],
},);
