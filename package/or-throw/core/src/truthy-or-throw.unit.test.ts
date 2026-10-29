/**
 Tests for `truthyOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { truthyOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: truthyOrThrow.name,
  children: [
    it({
      name: 'returns the value when truthy',
      fn: async () => {
        expect(truthyOrThrow('hello',),).toBe('hello',);
        expect(truthyOrThrow(42,),).toBe(42,);
        expect(truthyOrThrow(true,),).toBe(true,);
        expect(truthyOrThrow(1n,),).toBe(1n,);
        const obj = { a: 1, };
        expect(truthyOrThrow(obj,),).toBe(obj,);
        const arr: number[] = [];
        expect(truthyOrThrow(arr,),).toBe(arr,);
      },
    },),

    it({
      name: 'throws on every falsy value',
      fn: async () => {
        expect(() => truthyOrThrow(false,)).toThrow('truthy',);
        expect(() => truthyOrThrow(0,)).toThrow('truthy',);
        expect(() => truthyOrThrow(0n,)).toThrow('truthy',);
        expect(() => truthyOrThrow('',)).toThrow('truthy',);
        expect(() => truthyOrThrow(null,)).toThrow('truthy',);
        expect(() => truthyOrThrow(undefined,)).toThrow('truthy',);
        expect(() => truthyOrThrow(Number.NaN,)).toThrow('truthy',);
      },
    },),

    it({
      name: 'narrows union return types to exclude falsy variants',
      fn: async () => {
        // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- narrowing test deliberately feeds a union with `undefined` to verify the return type excludes it
        const input: string | undefined = 'value';
        const output = truthyOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<string>();
      },
    },),

    it({
      name: 'preserves wide primitive types when falsy is the only literal in T',
      fn: async () => {
        const input: 0 | 5 = 5;
        const output = truthyOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<5>();
      },
    },),
  ],
},);
