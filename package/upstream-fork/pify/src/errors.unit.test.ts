/**
 Tests for the input validation error class.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  InvalidInputError,
  pify,
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
            const error = new InvalidInputError('nope',);
            expect(error,).toBeInstanceOf(TypeError,);
            expect(error,).toBeInstanceOf(Error,);
          },
        },),

        it({
          name: 'names itself for readable stacks',
          fn: async () => {
            const error = new InvalidInputError('nope',);
            expect(error.name,).toBe('InvalidInputError',);
          },
        },),

        it({
          name: 'carries upstream pify message text for null input',
          fn: async () => {
            const error = new InvalidInputError(null,);
            expect(
              error.message,
            ).toBe('Expected `input` to be a `Function` or `Object`, got `null`',);
          },
        },),

        it({
          name: 'carries upstream pify message text for undefined input',
          fn: async () => {
            const error = new InvalidInputError(undefined,);
            expect(
              error.message,
            ).toBe('Expected `input` to be a `Function` or `Object`, got `undefined`',);
          },
        },),

        it({
          name: 'reports typeof strings for primitive input, like upstream',
          fn: async () => {
            /**
             Inputs paired with the `typeof` string upstream's message shows.
             */
            const cases: readonly { readonly input: unknown; readonly shown: string; }[] = [
              {
                input: '',
                shown: 'string',
              },
              {
                input: 3,
                shown: 'number',
              },
              {
                input: true,
                shown: 'boolean',
              },
              {
                input: 1n,
                shown: 'bigint',
              },
              {
                input: Symbol('sample member key for message text',),
                shown: 'symbol',
              },
            ];
            for (const { input, shown, } of cases) {
              const error = new InvalidInputError(input,);
              expect(
                error.message,
              ).toBe(`Expected \`input\` to be a \`Function\` or \`Object\`, got \`${shown}\``,);
            }
          },
        },),

        it({
          name: 'pify throws it for non-function, non-object input',
          fn: async () => {
            /**
             Last error thrown by `pify` on invalid input, captured for its
             message comparison.
             */
            let thrown: unknown;
            try {
              // @ts-expect-error -- upstream `pify` accepts invalid input at runtime and throws its TypeError there; the overload surface correctly rejects it
              pify({ input: null, },);
            }
            catch (error) {
              thrown = error;
            }
            expect(thrown,).toBeInstanceOf(InvalidInputError,);
            expect(
              (thrown as InvalidInputError).message,
            ).toBe('Expected `input` to be a `Function` or `Object`, got `null`',);
          },
        },),
      ],
    },),
  ],
},);
