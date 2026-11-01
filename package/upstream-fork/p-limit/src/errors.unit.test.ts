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
  InvalidConcurrencyError,
  InvalidRejectOnClearError,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: InvalidConcurrencyError.name,
      children: [
        it({
          name: 'extends TypeError so existing TypeError handlers keep working',
          fn: async () => {
            const error = new InvalidConcurrencyError();
            expect(error,).toBeInstanceOf(TypeError,);
            expect(error,).toBeInstanceOf(Error,);
          },
        },),

        it({
          name: 'carries upstream p-limit message text',
          fn: async () => {
            const error = new InvalidConcurrencyError();
            expect(error.message,).toBe('Expected `concurrency` to be a number from 1 and up',);
          },
        },),

        it({
          name: 'names itself for readable stacks',
          fn: async () => {
            const error = new InvalidConcurrencyError();
            expect(error.name,).toBe('InvalidConcurrencyError',);
          },
        },),
      ],
    },),

    describe({
      name: InvalidRejectOnClearError.name,
      children: [
        it({
          name: 'extends TypeError so existing TypeError handlers keep working',
          fn: async () => {
            const error = new InvalidRejectOnClearError();
            expect(error,).toBeInstanceOf(TypeError,);
            expect(error,).toBeInstanceOf(Error,);
          },
        },),

        it({
          name: 'carries upstream p-limit message text',
          fn: async () => {
            const error = new InvalidRejectOnClearError();
            expect(error.message,).toBe('Expected `rejectOnClear` to be a boolean',);
          },
        },),

        it({
          name: 'names itself for readable stacks',
          fn: async () => {
            const error = new InvalidRejectOnClearError();
            expect(error.name,).toBe('InvalidRejectOnClearError',);
          },
        },),
      ],
    },),
  ],
},);
