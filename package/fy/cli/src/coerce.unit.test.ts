import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { coerceArg, } from './coerce.ts';

await describe({
  name: coerceArg.name,
  children: [
    //region Numeric coercion

    it({
      name: 'coerces integer string to number',
      fn: async () => {
        expect(coerceArg({ arg: '42', },),).toBe(42,);
      },
    },),

    it({
      name: 'coerces negative integer string to number',
      fn: async () => {
        expect(coerceArg({ arg: '-7', },),).toBe(-7,);
      },
    },),

    it({
      name: 'coerces float string to number',
      fn: async () => {
        expect(coerceArg({ arg: '3.14', },),).toBe(3.14,);
      },
    },),

    it({
      name: 'coerces zero to number',
      fn: async () => {
        expect(coerceArg({ arg: '0', },),).toBe(0,);
      },
    },),

    //endregion Numeric coercion

    //region Boolean and null coercion

    it({
      name: 'coerces "true" to boolean true',
      fn: async () => {
        expect(coerceArg({ arg: 'true', },),).toBe(true,);
      },
    },),

    it({
      name: 'coerces "false" to boolean false',
      fn: async () => {
        expect(coerceArg({ arg: 'false', },),).toBe(false,);
      },
    },),

    it({
      name: 'coerces "null" to null',
      fn: async () => {
        expect(coerceArg({ arg: 'null', },),).toBeNull();
      },
    },),

    //endregion Boolean and null coercion

    //region Structured value coercion

    it({
      name: 'coerces JSON array string to array',
      fn: async () => {
        expect(coerceArg({ arg: '[1,2,3]', },),).toEqual([1, 2, 3,],);
      },
    },),

    it({
      name: 'coerces JSON object string to object',
      fn: async () => {
        expect(coerceArg({ arg: '{"a":1}', },),).toEqual({ a: 1, },);
      },
    },),

    it({
      name: 'coerces JSON string literal to string value',
      fn: async () => {
        expect(coerceArg({ arg: '"hello"', },),).toBe('hello',);
      },
    },),

    //endregion Structured value coercion

    //region String fallback

    it({
      name: 'falls back to raw string for non-JSON text',
      fn: async () => {
        expect(coerceArg({ arg: 'hello', },),).toBe('hello',);
      },
    },),

    it({
      name: 'falls back to raw string for path-like values',
      fn: async () => {
        expect(coerceArg({ arg: '/tmp/test', },),).toBe('/tmp/test',);
      },
    },),

    it({
      name: 'falls back to raw string for incomplete JSON',
      fn: async () => {
        expect(coerceArg({ arg: '{broken', },),).toBe('{broken',);
      },
    },),

    it({
      name: 'preserves empty string',
      fn: async () => {
        expect(coerceArg({ arg: '', },),).toBe('',);
      },
    },),
    //endregion String fallback
  ],
},);
