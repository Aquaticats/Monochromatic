/**
 Tests for `stringOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { stringOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: stringOrThrow.name,
  children: [
    it({
      name: 'returns primitive strings unchanged',
      fn: async () => {
        expect(stringOrThrow('hello',),).toBe('hello',);
        expect(stringOrThrow('',),).toBe('',);
      },
    },),

    it({
      name: 'throws on non-string values',
      fn: async () => {
        expect(() => stringOrThrow(42,)).toThrow('string',);
        expect(() => stringOrThrow(true,)).toThrow('string',);
        expect(() => stringOrThrow(1n,)).toThrow('string',);
        expect(() => stringOrThrow(Symbol('non string symbol value',),)).toThrow('string',);
        expect(() => stringOrThrow(null,)).toThrow('string',);
        expect(() => stringOrThrow(undefined,)).toThrow('string',);
        expect(() => stringOrThrow({},)).toThrow('string',);
        expect(() => stringOrThrow([],)).toThrow('string',);
      },
    },),

    it({
      name: 'does not invoke caller-owned coercion hooks while formatting failures',
      fn: async () => {
        /**
         Number of user coercion hook calls.
         */
        let coercionCount = 0;
        /**
         Rejected reference carrying observable coercion hook.
         */
        const value = {
          toString(): string {
            coercionCount++;
            return 'coerced';
          },
        };
        expect(() => stringOrThrow(value,)).toThrow('[object]',);
        expect(coercionCount,).toBe(0,);
      },
    },),

    it({
      name: 'narrows unknown to string',
      fn: async () => {
        const input: unknown = 'hello';
        const output = stringOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<string>();
      },
    },),

    it({
      name: 'narrows union to string branch',
      fn: async () => {
        const input: string | number = 'hello';
        const output = stringOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<string>();
      },
    },),
  ],
},);
