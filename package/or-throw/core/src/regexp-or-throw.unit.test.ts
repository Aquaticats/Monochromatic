/**
 Tests for `regExpOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { regExpOrThrow, } from '@monochromatic-dev/or-throw-core';

/* oxlint-disable no-restricted-syntax/no-regex -- this file tests regExpOrThrow; every test must construct a regex literal or RegExp instance as input. The regex sites here ARE the test fixtures. */

await describe({
  name: regExpOrThrow.name,
  children: [
    it({
      name: 'returns RegExp instances unchanged',
      fn: async () => {
        const re = /abc/u;
        expect(regExpOrThrow(re,),).toBe(re,);
        // oxlint-disable-next-line eslint/prefer-regex-literals -- this fixture deliberately constructs via `new RegExp` to verify constructor-built instances pass through unchanged; a literal would collapse it into the `re` case above
        const ctor = new RegExp('xyz', 'u',);
        expect(regExpOrThrow(ctor,),).toBe(ctor,);
      },
    },),

    it({
      name: 'throws on non-RegExp values',
      fn: async () => {
        expect(() => regExpOrThrow('abc',)).toThrow('RegExp',);
        expect(() =>
          regExpOrThrow({
            test() {
              return true;
            },
          },)
        )
          .toThrow('RegExp',);
        expect(() => regExpOrThrow(null,)).toThrow('RegExp',);
        expect(() => regExpOrThrow(undefined,)).toThrow('RegExp',);
      },
    },),

    it({
      name: 'narrows unknown to RegExp',
      fn: async () => {
        const input: unknown = /abc/u;
        const output = regExpOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<RegExp>();
      },
    },),
  ],
},);

/* oxlint-enable no-restricted-syntax/no-regex */
