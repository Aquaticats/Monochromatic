/**
 Tests for `weakSetOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { weakSetOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: weakSetOrThrow.name,
  children: [
    it({
      name: 'returns the value when it is a WeakSet',
      fn: async () => {
        const ws = new WeakSet();
        expect(weakSetOrThrow(ws,),).toBe(ws,);
      },
    },),

    it({
      name: 'throws on every non-WeakSet value',
      fn: async () => {
        expect(() => weakSetOrThrow(new Set(),)).toThrow('WeakSet',);
        expect(() => weakSetOrThrow(new WeakMap(),)).toThrow('WeakSet',);
        expect(() => weakSetOrThrow([],)).toThrow('WeakSet',);
        expect(() => weakSetOrThrow({},)).toThrow('WeakSet',);
        expect(() => weakSetOrThrow(null,)).toThrow('WeakSet',);
      },
    },),

    it({
      name: 'narrows unknown to WeakSet<object>',
      fn: async () => {
        const input: unknown = new WeakSet();
        const output = weakSetOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<WeakSet<object>>();
      },
    },),
  ],
},);
