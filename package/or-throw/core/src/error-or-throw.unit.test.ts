/**
 Tests for `errorOrThrow`.
 
 @module
 */

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { errorOrThrow, } from '@monochromatic-dev/or-throw-core';

await describe({
  name: errorOrThrow.name,
  children: [
    it({
      name: 'returns Error instances unchanged',
      fn: async () => {
        const e = new Error('boom',);
        expect(errorOrThrow(e,),).toBe(e,);
      },
    },),

    it({
      name: 'returns Error subclass instances unchanged',
      fn: async () => {
        const te = new TypeError('bad type',);
        expect(errorOrThrow(te,),).toBe(te,);
        const re = new RangeError('out of range',);
        expect(errorOrThrow(re,),).toBe(re,);
      },
    },),

    it({
      name: 'returns genuine errors from other realms unchanged',
      fn: async () => {
        const { runInNewContext, } = await import('node:vm');
        const foreign: unknown = runInNewContext('new TypeError("bad type",)',);
        expect(errorOrThrow(foreign,),).toBe(foreign,);
      },
    },),

    it({
      name: 'throws on prototype-only impostors and Error-shaped fakes',
      fn: async () => {
        expect(() => errorOrThrow(Object.create(Error.prototype,),),).toThrow('Error',);
        expect(() => errorOrThrow({ name: 'Error', message: 'boom', stack: '', },),).toThrow('Error',);
      },
    },),

    it({
      name: 'throws on non-Error values, including error-shaped plain objects',
      fn: async () => {
        expect(() => errorOrThrow('boom',)).toThrow('Error',);
        expect(() => errorOrThrow({ message: 'boom', },)).toThrow('Error',);
        expect(() => errorOrThrow(null,)).toThrow('Error',);
        expect(() => errorOrThrow(undefined,)).toThrow('Error',);
      },
    },),

    it({
      name: 'narrows unknown to Error',
      fn: async () => {
        const caught: unknown = new Error('boom',);
        const output = errorOrThrow(caught,);
        expectTypeOf(output,).toEqualTypeOf<Error>();
      },
    },),
  ],
},);
