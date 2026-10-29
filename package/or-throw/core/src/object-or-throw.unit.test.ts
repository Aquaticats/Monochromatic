/**
 Tests for `objectOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { objectOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: objectOrThrow.name,
  children: [
    it({
      name: 'returns plain objects, arrays, and class instances unchanged',
      fn: async () => {
        const obj = { a: 1, };
        expect(objectOrThrow(obj,),).toBe(obj,);
        const arr: unknown[] = [];
        expect(objectOrThrow(arr,),).toBe(arr,);
        const map = new Map();
        expect(objectOrThrow(map,),).toBe(map,);
      },
    },),

    it({
      name: 'throws on null',
      fn: async () => {
        expect(() => objectOrThrow(null,)).toThrow('non-null object',);
      },
    },),

    it({
      name: 'throws on primitives',
      fn: async () => {
        expect(() => objectOrThrow(0,)).toThrow('non-null object',);
        expect(() => objectOrThrow('abc',)).toThrow('non-null object',);
        expect(() => objectOrThrow(true,)).toThrow('non-null object',);
        expect(() => objectOrThrow(undefined,)).toThrow('non-null object',);
        expect(() => objectOrThrow(Symbol('non object symbol value',),)).toThrow('non-null object',);
        expect(() => objectOrThrow(1n,)).toThrow('non-null object',);
      },
    },),

    it({
      name: 'throws on functions (functions are typeof "function", not "object")',
      fn: async () => {
        function noop(): void {}
        expect(() => objectOrThrow(noop,)).toThrow('non-null object',);
      },
    },),

    it({
      name: 'narrows unknown to object',
      fn: async () => {
        const input: unknown = { a: 1, };
        const output = objectOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<object>();
      },
    },),
  ],
},);
