/**
 Tests for `arrayOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { arrayOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: arrayOrThrow.name,
  children: [
    it({
      name: 'returns the value when it is an array',
      fn: async () => {
        const arr = [1, 2, 3,];
        expect(arrayOrThrow(arr,),).toBe(arr,);
        const empty: readonly unknown[] = [];
        expect(arrayOrThrow(empty,),).toBe(empty,);
      },
    },),

    it({
      name: 'throws on every non-array value',
      fn: async () => {
        expect(() => arrayOrThrow('abc',)).toThrow('array',);
        expect(() => arrayOrThrow(42,)).toThrow('array',);
        expect(() => arrayOrThrow(null,)).toThrow('array',);
        expect(() => arrayOrThrow(undefined,)).toThrow('array',);
        expect(() => arrayOrThrow({ length: 0, },)).toThrow('array',);
        expect(() => arrayOrThrow(new Set(),)).toThrow('array',);
      },
    },),

    it({
      name: 'narrows unknown to readonly unknown[]',
      fn: async () => {
        const input: unknown = [1, 2,];
        const output = arrayOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<readonly unknown[]>();
      },
    },),

    it({
      name: 'narrows union to array branch',
      fn: async () => {
        const input: string | number[] = [1, 2,];
        const output = arrayOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<number[]>();
      },
    },),
  ],
},);
