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
  InvalidMaxAgeError,
  InvalidMaxSizeError,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: InvalidMaxSizeError.name,
      children: [
        it({
          name: 'extends TypeError so existing TypeError handlers keep working',
          fn: async () => {
            const error = new InvalidMaxSizeError();
            expect(error,).toBeInstanceOf(TypeError,);
            expect(error,).toBeInstanceOf(Error,);
          },
        },),

        it({
          name: 'carries upstream quick-lru message text',
          fn: async () => {
            const error = new InvalidMaxSizeError();
            expect(error.message,).toBe('`maxSize` must be a number greater than 0',);
          },
        },),

        it({
          name: 'names itself for readable stacks',
          fn: async () => {
            const error = new InvalidMaxSizeError();
            expect(error.name,).toBe('InvalidMaxSizeError',);
          },
        },),
      ],
    },),

    describe({
      name: InvalidMaxAgeError.name,
      children: [
        it({
          name: 'extends TypeError so existing TypeError handlers keep working',
          fn: async () => {
            const error = new InvalidMaxAgeError();
            expect(error,).toBeInstanceOf(TypeError,);
            expect(error,).toBeInstanceOf(Error,);
          },
        },),

        it({
          name: 'carries upstream quick-lru message text',
          fn: async () => {
            const error = new InvalidMaxAgeError();
            expect(error.message,).toBe('`maxAge` must be a number greater than 0',);
          },
        },),

        it({
          name: 'names itself for readable stacks',
          fn: async () => {
            const error = new InvalidMaxAgeError();
            expect(error.name,).toBe('InvalidMaxAgeError',);
          },
        },),
      ],
    },),
  ],
},);
