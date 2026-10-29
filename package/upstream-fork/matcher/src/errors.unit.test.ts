/**
 Tests for the configuration error classes.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  InvalidInputsError,
  InvalidPatternsError,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: InvalidInputsError.name,
      children: [
        it({
          name: 'extends TypeError so existing TypeError handlers keep working',
          fn: async () => {
            const error = new InvalidInputsError({ value: 1, },);
            expect(error,).toBeInstanceOf(TypeError,);
            expect(error,).toBeInstanceOf(Error,);
          },
        },),

        it({
          name: 'carries upstream matcher inputs message text',
          fn: async () => {
            const error = new InvalidInputsError({ value: 1, },);
            expect(error.message,).toBe('Expected \'inputs\' to be a string or an array, but got a type of \'number\'',);
          },
        },),

        it({
          name: 'carries upstream matcher inputs element message text',
          fn: async () => {
            const error = new InvalidInputsError({ element: 0, },);
            expect(error.message,).toBe('Expected \'inputs\' to be an array of strings, but found a type of \'number\' in the array',);
          },
        },),

        it({
          name: 'names itself for readable stacks',
          fn: async () => {
            const error = new InvalidInputsError({ value: 1, },);
            expect(error.name,).toBe('InvalidInputsError',);
          },
        },),
      ],
    },),

    describe({
      name: InvalidPatternsError.name,
      children: [
        it({
          name: 'extends TypeError so existing TypeError handlers keep working',
          fn: async () => {
            const error = new InvalidPatternsError({ value: 1, },);
            expect(error,).toBeInstanceOf(TypeError,);
            expect(error,).toBeInstanceOf(Error,);
          },
        },),

        it({
          name: 'carries upstream matcher patterns message text',
          fn: async () => {
            const error = new InvalidPatternsError({ value: 1, },);
            expect(error.message,).toBe('Expected \'patterns\' to be a string or an array, but got a type of \'number\'',);
          },
        },),

        it({
          name: 'carries upstream matcher patterns element message text',
          fn: async () => {
            const error = new InvalidPatternsError({ element: null, },);
            expect(error.message,).toBe('Expected \'patterns\' to be an array of strings, but found a type of \'object\' in the array',);
          },
        },),

        it({
          name: 'names itself for readable stacks',
          fn: async () => {
            const error = new InvalidPatternsError({ value: 1, },);
            expect(error.name,).toBe('InvalidPatternsError',);
          },
        },),
      ],
    },),
  ],
},);
