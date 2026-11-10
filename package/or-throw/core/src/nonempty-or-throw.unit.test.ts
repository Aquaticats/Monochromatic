/**
 Tests for `nonemptyOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { nonemptyOrThrow, } from '@monochromatic-dev/or-throw-core';

import { createCallerOwnedRecord, } from './caller-owned-record.test-fixture.ts';

await describe({
  name: nonemptyOrThrow.name,
  children: [
    it({
      name: 'returns the value for every recognized nonempty container',
      fn: async () => {
        expect(nonemptyOrThrow('x',),).toBe('x',);
        const arr = [1,];
        expect(nonemptyOrThrow(arr,),).toBe(arr,);
        const set = new Set([1,],);
        expect(nonemptyOrThrow(set,),).toBe(set,);
        const map = new Map([['a', 1,],],);
        expect(nonemptyOrThrow(map,),).toBe(map,);
        const obj = { a: 1, };
        expect(nonemptyOrThrow(obj,),).toBe(obj,);
      },
    },),

    it({
      name: 'throws when a recognized container has zero size',
      fn: async () => {
        expect(() => nonemptyOrThrow('',)).toThrow('size 0',);
        expect(() => nonemptyOrThrow([],)).toThrow('size 0',);
        expect(() => nonemptyOrThrow(new Set(),)).toThrow('size 0',);
        expect(() => nonemptyOrThrow(new Map(),)).toThrow('size 0',);
        expect(() => nonemptyOrThrow({},)).toThrow('size 0',);
      },
    },),

    it({
      name: 'throws when value is not a recognized container',
      fn: async () => {
        expect(() => nonemptyOrThrow(null,)).toThrow('sized container',);
        expect(() => nonemptyOrThrow(undefined,)).toThrow('sized container',);
        expect(() => nonemptyOrThrow(42,)).toThrow('sized container',);
        expect(() => nonemptyOrThrow(true,)).toThrow('sized container',);
      },
    },),

    it({
      name: 'throws when an object is not a plain object',
      fn: async () => {
        expect(() => nonemptyOrThrow(new WeakSet(),)).toThrow('sized container',);
        expect(() => nonemptyOrThrow(new WeakMap(),)).toThrow('sized container',);
        expect(() => nonemptyOrThrow(new Date(),)).toThrow('sized container',);
        expect(() => nonemptyOrThrow(new Object('ab',),)).toThrow('sized container',);
        expect(() => nonemptyOrThrow(createCallerOwnedRecord(),)).toThrow('sized container',);
      },
    },),
  ],
},);
