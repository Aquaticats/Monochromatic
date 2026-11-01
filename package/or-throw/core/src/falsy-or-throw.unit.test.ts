/**
 Tests for `falsyOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { falsyOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: falsyOrThrow.name,
  children: [
    it({
      name: 'returns the value when falsy',
      fn: async () => {
        expect(falsyOrThrow(false,),).toBe(false,);
        expect(falsyOrThrow(0,),).toBe(0,);
        expect(falsyOrThrow(0n,),).toBe(0n,);
        expect(falsyOrThrow('',),).toBe('',);
        expect(falsyOrThrow(null,),).toBe(null,);
        expect(falsyOrThrow(undefined,),).toBe(undefined,);
        expect(
          Number.isNaN(falsyOrThrow(Number.NaN,),),
        ).toBe(true,);
      },
    },),

    it({
      name: 'throws on truthy primitives',
      fn: async () => {
        expect(() => falsyOrThrow('hello',)).toThrow('falsy',);
        expect(() => falsyOrThrow(1,)).toThrow('falsy',);
        expect(() => falsyOrThrow(true,)).toThrow('falsy',);
        expect(() => falsyOrThrow(1n,)).toThrow('falsy',);
      },
    },),

    it({
      name: 'throws on truthy reference types',
      fn: async () => {
        expect(() => falsyOrThrow({},)).toThrow('falsy',);
        expect(() => falsyOrThrow([],)).toThrow('falsy',);
        expect(() => falsyOrThrow(() => 0)).toThrow('falsy',);
      },
    },),

    it({
      name: 'narrows union return types to the falsy side',
      fn: async () => {
        // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- narrowing test deliberately feeds a union with `undefined` to verify the return type narrows to the falsy side
        const input: string | undefined = undefined;
        const output = falsyOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<undefined>();
      },
    },),
  ],
},);
