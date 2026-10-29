/**
 Tests for `promiseOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { promiseOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: promiseOrThrow.name,
  children: [
    it({
      name: 'returns native promises unchanged',
      fn: async () => {
        const p = Promise.resolve(1,);
        expect(promiseOrThrow(p,),).toBe(p,);
      },
    },),

    it({
      name: 'throws on non-promise values',
      fn: async () => {
        expect(() => promiseOrThrow(42,)).toThrow('Promise',);
        expect(() => promiseOrThrow(null,)).toThrow('Promise',);
        expect(() => promiseOrThrow(undefined,)).toThrow('Promise',);
        expect(() => promiseOrThrow([],)).toThrow('Promise',);
        expect(() => promiseOrThrow('not a promise',)).toThrow('Promise',);
      },
    },),

    it({
      name: 'narrows unknown to Promise<unknown>',
      fn: async () => {
        const input: unknown = Promise.resolve(1,);
        const output = promiseOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<Promise<unknown>>();
      },
    },),
  ],
},);
