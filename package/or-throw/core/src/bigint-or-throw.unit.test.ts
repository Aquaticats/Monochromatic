/**
 Tests for `bigintOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { bigintOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: bigintOrThrow.name,
  children: [
    it({
      name: 'returns primitive bigints unchanged',
      fn: async () => {
        expect(bigintOrThrow(0n,),).toBe(0n,);
        expect(bigintOrThrow(42n,),).toBe(42n,);
        expect(bigintOrThrow(-1n,),).toBe(-1n,);
      },
    },),

    it({
      name: 'throws on non-bigint values, including numeric numbers',
      fn: async () => {
        expect(() => bigintOrThrow(42,)).toThrow('bigint',);
        expect(() => bigintOrThrow('42',)).toThrow('bigint',);
        expect(() => bigintOrThrow(true,)).toThrow('bigint',);
        expect(() => bigintOrThrow(null,)).toThrow('bigint',);
        expect(() => bigintOrThrow(undefined,)).toThrow('bigint',);
      },
    },),

    it({
      name: 'narrows unknown to bigint',
      fn: async () => {
        const input: unknown = 42n;
        const output = bigintOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<bigint>();
      },
    },),
  ],
},);
