/**
 Tests for `nonNullishOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: nonNullishOrThrow.name,
  children: [
    it({
      name: 'returns the value when non-nullish',
      fn: async () => {
        expect(nonNullishOrThrow('hello',),).toBe('hello',);
        expect(nonNullishOrThrow(42,),).toBe(42,);
        expect(nonNullishOrThrow(true,),).toBe(true,);
        expect(nonNullishOrThrow(false,),).toBe(false,);
        expect(nonNullishOrThrow(0,),).toBe(0,);
        expect(nonNullishOrThrow('',),).toBe('',);
        const obj = { a: 1, };
        expect(nonNullishOrThrow(obj,),).toBe(obj,);
        const arr: number[] = [];
        expect(nonNullishOrThrow(arr,),).toBe(arr,);
      },
    },),

    it({
      name: 'throws when value is null',
      fn: async () => {
        expect(() => nonNullishOrThrow(null,)).toThrow('non-nullish',);
      },
    },),

    it({
      name: 'throws when value is undefined',
      fn: async () => {
        expect(() => nonNullishOrThrow(undefined,)).toThrow('non-nullish',);
      },
    },),

    it({
      name: 'does not throw on falsy-but-non-nullish values',
      fn: async () => {
        expect(() => nonNullishOrThrow(0,)).not.toThrow();
        expect(() => nonNullishOrThrow('',)).not.toThrow();
        expect(() => nonNullishOrThrow(false,)).not.toThrow();
        expect(() => nonNullishOrThrow(Number.NaN,)).not.toThrow();
      },
    },),

    it({
      name: 'narrows the static return type to exclude null and undefined',
      fn: async () => {
        // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- narrowing test deliberately feeds a union with `undefined` to verify the return type excludes it
        const input: string | null | undefined = 'value';
        const output = nonNullishOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<string>();
      },
    },),
  ],
},);
