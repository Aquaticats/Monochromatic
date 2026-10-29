/**
 Tests for `dateOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { dateOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: dateOrThrow.name,
  children: [
    it({
      name: 'returns native Date instances unchanged',
      fn: async () => {
        const d = new Date(0,);
        expect(dateOrThrow(d,),).toBe(d,);
      },
    },),

    it({
      name: 'returns invalid Date instances unchanged (type check only, not validity)',
      fn: async () => {
        const invalid = new Date('not-a-date',);
        expect(dateOrThrow(invalid,),).toBe(invalid,);
      },
    },),

    it({
      name: 'throws on non-Date values',
      fn: async () => {
        expect(() => dateOrThrow(0,)).toThrow('Date',);
        expect(() => dateOrThrow('2024-01-01',)).toThrow('Date',);
        expect(() => dateOrThrow(null,)).toThrow('Date',);
        expect(() => dateOrThrow(undefined,)).toThrow('Date',);
        expect(() => dateOrThrow({},)).toThrow('Date',);
      },
    },),

    it({
      name: 'narrows unknown to Date',
      fn: async () => {
        const input: unknown = new Date(0,);
        const output = dateOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<Date>();
      },
    },),
  ],
},);
