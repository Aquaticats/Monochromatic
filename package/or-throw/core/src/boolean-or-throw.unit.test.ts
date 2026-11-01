/**
 Tests for `booleanOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { booleanOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: booleanOrThrow.name,
  children: [
    it({
      name: 'returns primitive booleans unchanged',
      fn: async () => {
        expect(booleanOrThrow(true,),).toBe(true,);
        expect(booleanOrThrow(false,),).toBe(false,);
      },
    },),

    it({
      name: 'throws on truthy/falsy values that are not literal booleans',
      fn: async () => {
        expect(() => booleanOrThrow(1,)).toThrow('boolean',);
        expect(() => booleanOrThrow(0,)).toThrow('boolean',);
        expect(() => booleanOrThrow('true',)).toThrow('boolean',);
        expect(() => booleanOrThrow('',)).toThrow('boolean',);
        expect(() => booleanOrThrow(null,)).toThrow('boolean',);
        expect(() => booleanOrThrow(undefined,)).toThrow('boolean',);
      },
    },),

    it({
      name: 'narrows unknown to boolean',
      fn: async () => {
        const input: unknown = true;
        const output = booleanOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<boolean>();
      },
    },),
  ],
},);
