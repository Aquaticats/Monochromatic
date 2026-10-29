/**
 Tests for reply settlement: result restoration and error reconstruction.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { getResult, } from '../dist/final/neutral/index.mjs';

await describe({
  name: getResult.name,
  children: [
    it({
      name: 'returns the output of a success reply',
      fn: async () => {
        expect(getResult({
          id: 1,
          output: 42,
        },),).toBe(42,);
      },
    },),

    it({
      name: 'returns falsy outputs instead of throwing',
      fn: async () => {
        expect(getResult({
          id: 2,
          output: 0,
        },),).toBe(0,);
      },
    },),

    it({
      name: 'throws the reported failure value verbatim',
      fn: async () => {
        let caught: unknown;
        try {
          getResult({
            error: 'unicorn',
            id: 3,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBe('unicorn',);
      },
    },),

    it({
      name: 'throws a falsy reported failure instead of returning output',
      fn: async () => {
        let caught: unknown;
        try {
          getResult({
            error: 0,
            id: 4,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBe(0,);
      },
    },),

    it({
      name: 'defines a traveling custom error name onto the cloned error',
      fn: async () => {
        /**
         Cloned error missing its custom name.
         */
        const error = new Error('unicorn',);
        let caught: unknown;
        try {
          getResult({
            error,
            errorName: 'UnicornError',
            id: 5,
          },);
        }
        catch (failure) {
          caught = failure;
        }
        expect(caught,).toBe(error,);
        expect((caught as Error).name,).toBe('UnicornError',);
      },
    },),

    it({
      name: 'leaves a built-in error without an own name property',
      fn: async () => {
        /**
         Cloned built-in error already carrying its name.
         */
        const error = new TypeError('unicorn',);
        let caught: unknown;
        try {
          getResult({
            error,
            errorName: 'TypeError',
            id: 6,
          },);
        }
        catch (failure) {
          caught = failure;
        }
        expect(caught,).toBe(error,);
        expect(Object.hasOwn(
          caught as object,
          'name',
        ),).toBe(false,);
      },
    },),

    it({
      name: 'skips property restoration when no extra properties travel',
      fn: async () => {
        /**
         Cloned error traveling without extra properties.
         */
        const error = new Error('unicorn',);
        let caught: unknown;
        try {
          getResult({
            error,
            id: 9,
          },);
        }
        catch (failure) {
          caught = failure;
        }
        expect(caught,).toBe(error,);
        expect(Object.hasOwn(
          caught as object,
          'code',
        ),).toBe(false,);
      },
    },),

    it({
      name: 'defines traveling extra error properties onto the cloned error',
      fn: async () => {
        /**
         Cloned error missing its extra properties.
         */
        const error = new Error('unicorn',);
        let caught: unknown;
        try {
          getResult({
            error,
            errorProperties: { code: 'ENOENT', },
            id: 7,
          },);
        }
        catch (failure) {
          caught = failure;
        }
        expect((caught as { readonly code?: string; }).code,).toBe('ENOENT',);
      },
    },),
  ],
},);
