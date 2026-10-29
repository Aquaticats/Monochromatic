/**
 Tests for `emptyOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { emptyOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: emptyOrThrow.name,
  children: [
    it({
      name: 'returns the value for every recognized empty container',
      fn: async () => {
        expect(emptyOrThrow('',),).toBe('',);
        const arr: readonly unknown[] = [];
        expect(emptyOrThrow(arr,),).toBe(arr,);
        const set = new Set();
        expect(emptyOrThrow(set,),).toBe(set,);
        const map = new Map();
        expect(emptyOrThrow(map,),).toBe(map,);
        const obj = {};
        expect(emptyOrThrow(obj,),).toBe(obj,);
      },
    },),

    it({
      name: 'throws when a recognized container has nonzero size',
      fn: async () => {
        expect(() => emptyOrThrow('x',)).toThrow('size 1',);
        expect(() => emptyOrThrow([1,],)).toThrow('size 1',);
        expect(() => emptyOrThrow(new Set([1,],),)).toThrow('size 1',);
        expect(() => emptyOrThrow(new Map([['a', 1,],],),)).toThrow('size 1',);
        expect(() => emptyOrThrow({ a: 1, },)).toThrow('size 1',);
      },
    },),

    it({
      name: 'throws when value is not a recognized container',
      fn: async () => {
        expect(() => emptyOrThrow(null,)).toThrow('sized container',);
        expect(() => emptyOrThrow(undefined,)).toThrow('sized container',);
        expect(() => emptyOrThrow(42,)).toThrow('sized container',);
        expect(() => emptyOrThrow(true,)).toThrow('sized container',);
        expect(() => emptyOrThrow(1n,)).toThrow('sized container',);
        expect(() => emptyOrThrow(Symbol('non container symbol value',),)).toThrow('sized container',);
      },
    },),
  ],
},);
