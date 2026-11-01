/**
 Tests for `mapOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { mapOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: mapOrThrow.name,
  children: [
    it({
      name: 'returns the value when it is a Map',
      fn: async () => {
        const map = new Map([['a', 1,], ['b', 2,],],);
        expect(mapOrThrow(map,),).toBe(map,);
      },
    },),

    it({
      name: 'throws on every non-Map value',
      fn: async () => {
        expect(() => mapOrThrow([],)).toThrow('Map',);
        expect(() => mapOrThrow(new Set(),)).toThrow('Map',);
        expect(() => mapOrThrow(new WeakMap(),)).toThrow('Map',);
        expect(() => mapOrThrow({},)).toThrow('Map',);
        expect(() => mapOrThrow('abc',)).toThrow('Map',);
        expect(() => mapOrThrow(null,)).toThrow('Map',);
      },
    },),

    it({
      name: 'narrows unknown to Map<unknown, unknown>',
      fn: async () => {
        const input: unknown = new Map();
        const output = mapOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<Map<unknown, unknown>>();
      },
    },),

    it({
      name: 'narrows union to Map branch',
      fn: async () => {
        const input: Map<string, number> | Set<string> = new Map();
        const output = mapOrThrow(input,);
        expectTypeOf(output,).toEqualTypeOf<Map<string, number>>();
      },
    },),
  ],
},);
