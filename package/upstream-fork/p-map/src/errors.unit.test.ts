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
  InvalidBackpressureError,
  InvalidConcurrencyError,
  InvalidInputError,
  InvalidKeyedInputError,
  MapperRequiredError,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: InvalidInputError.name,
      children: [
        it({
          name: 'extends TypeError so existing TypeError handlers keep working',
          fn: async () => {
            const error = new InvalidInputError(5,);
            expect(error,).toBeInstanceOf(TypeError,);
            expect(error,).toBeInstanceOf(Error,);
          },
        },),

        it({
          name: 'carries upstream p-map message text with the rejected runtime type',
          fn: async () => {
            const error = new InvalidInputError(5,);
            expect(error.message,).toBe('Expected `input` to be either an `Iterable` or `AsyncIterable`, got (number)',);
          },
        },),

        it({
          name: 'names itself for readable stacks',
          fn: async () => {
            const error = new InvalidInputError(5,);
            expect(error.name,).toBe('InvalidInputError',);
          },
        },),
      ],
    },),

    describe({
      name: MapperRequiredError.name,
      children: [
        it({
          name: 'extends TypeError so existing TypeError handlers keep working',
          fn: async () => {
            const error = new MapperRequiredError();
            expect(error,).toBeInstanceOf(TypeError,);
          },
        },),

        it({
          name: 'carries upstream p-map message text',
          fn: async () => {
            const error = new MapperRequiredError();
            expect(error.message,).toBe('Mapper function is required',);
          },
        },),

        it({
          name: 'names itself for readable stacks',
          fn: async () => {
            const error = new MapperRequiredError();
            expect(error.name,).toBe('MapperRequiredError',);
          },
        },),
      ],
    },),

    describe({
      name: InvalidConcurrencyError.name,
      children: [
        it({
          name: 'extends TypeError so existing TypeError handlers keep working',
          fn: async () => {
            const error = new InvalidConcurrencyError(0,);
            expect(error,).toBeInstanceOf(TypeError,);
          },
        },),

        it({
          name: 'carries upstream p-map message text with value and type interpolations',
          fn: async () => {
            const error = new InvalidConcurrencyError(0,);
            expect(error.message,).toBe('Expected `concurrency` to be an integer from 1 and up or `Infinity`, got `0` (number)',);
          },
        },),

        it({
          name: 'names itself for readable stacks',
          fn: async () => {
            const error = new InvalidConcurrencyError(0,);
            expect(error.name,).toBe('InvalidConcurrencyError',);
          },
        },),

        it({
          name: 'renders non-numeric values the way upstream templates them',
          fn: async () => {
            const error = new InvalidConcurrencyError('yes',);
            expect(error.message,).toBe('Expected `concurrency` to be an integer from 1 and up or `Infinity`, got `yes` (string)',);
          },
        },),

        it({
          name: 'throws TypeError while building the message for symbol values, like upstream',
          fn: async () => {
            /**
             Failure caught while constructing the error, exactly as
             upstream `p-map`'s message template fails.
             */
            let caught: unknown;
            try {
              caught = new InvalidConcurrencyError(Symbol('symbolic concurrency bound value',),);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(TypeError,);
            expect((caught as Error).message,).toBe('Cannot convert a Symbol value to a string',);
          },
        },),
      ],
    },),

    describe({
      name: InvalidBackpressureError.name,
      children: [
        it({
          name: 'extends TypeError so existing TypeError handlers keep working',
          fn: async () => {
            const error = new InvalidBackpressureError({
              backpressure: 2,
              concurrency: 4,
            },);
            expect(error,).toBeInstanceOf(TypeError,);
          },
        },),

        it({
          name: 'carries upstream p-map message text naming both bounds',
          fn: async () => {
            const error = new InvalidBackpressureError({
              backpressure: 2,
              concurrency: 4,
            },);
            expect(error.message,).toBe('Expected `backpressure` to be an integer from `concurrency` (4) and up or `Infinity`, got `2` (number)',);
          },
        },),

        it({
          name: 'names itself for readable stacks',
          fn: async () => {
            const error = new InvalidBackpressureError({
              backpressure: 2,
              concurrency: 4,
            },);
            expect(error.name,).toBe('InvalidBackpressureError',);
          },
        },),

        it({
          name: 'throws TypeError while building the message for symbol values, like upstream',
          fn: async () => {
            /**
             Failure caught while constructing the error, exactly as
             upstream `p-map`'s message template fails.
             */
            let caught: unknown;
            try {
              caught = new InvalidBackpressureError({
                backpressure: Symbol('symbolic backpressure bound value',),
                concurrency: 4,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(TypeError,);
            expect((caught as Error).message,).toBe('Cannot convert a Symbol value to a string',);
          },
        },),
      ],
    },),

    describe({
      name: InvalidKeyedInputError.name,
      children: [
        it({
          name: 'extends TypeError so existing TypeError handlers keep working',
          fn: async () => {
            const error = new InvalidKeyedInputError(5,);
            expect(error,).toBeInstanceOf(TypeError,);
            expect(error,).toBeInstanceOf(Error,);
          },
        },),

        it({
          name: 'carries upstream p-map message text with the rejected runtime type',
          fn: async () => {
            const error = new InvalidKeyedInputError(5,);
            expect(error.message,).toBe('Expected `input` to be an object, got (number)',);
          },
        },),

        it({
          name: 'names null explicitly, as its runtime type is object',
          fn: async () => {
            const error = new InvalidKeyedInputError(null,);
            expect(error.message,).toBe('Expected `input` to be an object, got (null)',);
          },
        },),

        it({
          name: 'names itself for readable stacks',
          fn: async () => {
            const error = new InvalidKeyedInputError(5,);
            expect(error.name,).toBe('InvalidKeyedInputError',);
          },
        },),
      ],
    },),
  ],
},);
