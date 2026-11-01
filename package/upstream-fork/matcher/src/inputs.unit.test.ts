/**
 Tests for input and pattern list normalization, exercised through the
 public `matcher` and `isMatch` entry points.
 
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
  isMatch,
  matcher,
} from '../dist/final/neutral/index.mjs';

/**
 Inputs values the matcher must reject, each paired with its upstream
 message noun.
 */
const INVALID_INPUTS: readonly unknown[] = [
  0,
  null,
  false,
];

await describe({
  name: 'matcher input normalization',
  children: [
    //region Accepted shapes

    it({
      name: 'accepts a single input string',
      fn: async () => {
        expect(matcher({
          inputs: 'moo',
          patterns: ['*oo'],
        },),).toEqual([
          'moo',
        ],);
      },
    },),

    it({
      name: 'accepts a single pattern string',
      fn: async () => {
        expect(matcher({
          inputs: [
            'foo',
            'bar',
          ],
          patterns: 'foo',
        },),).toEqual([
          'foo',
        ],);
      },
    },),

    it({
      name: 'treats undefined inputs and patterns as empty lists',
      fn: async () => {
        expect(matcher({
          inputs: undefined as never,
          patterns: ['*'],
        },),).toEqual([],);
        expect(matcher({
          inputs: ['phoenix'],
          patterns: undefined as never,
        },),).toEqual([],);
      },
    },),

    it({
      name: 'drops undefined entries and holes from both lists',
      fn: async () => {
        /**
         Inputs with an explicit undefined entry.
         */
        const inputs: readonly string[] = [
          'phoenix',
          undefined as never,
        ];
        /**
         Patterns with an explicit undefined entry.
         */
        const patterns: readonly string[] = [
          undefined as never,
          '',
        ];
        expect(matcher({
          inputs,
          patterns: ['bar'],
        },),).toEqual([],);
        expect(matcher({
          inputs: [''],
          patterns,
        },),).toEqual([
          '',
        ],);
      },
    },),

    it({
      name: 'ignores holes in sparse arrays',
      fn: async () => {
        /**
         Inputs with a hole at index 3.
         */
        const inputs: string[] = [
          'a',
          'b',
        ];
        inputs[3] = 'c';
        /**
         Patterns with a hole at index 5.
         */
        const patterns: string[] = [
          'a',
          'c',
        ];
        patterns[5] = 'b';
        expect(matcher({
          inputs,
          patterns: '*',
        },),).toEqual([
          'a',
          'b',
          'c',
        ],);
        expect(matcher({
          inputs: [
            'a',
            'b',
            'c',
          ],
          patterns,
        },),).toEqual([
          'a',
          'b',
          'c',
        ],);
      },
    },),

    //endregion Accepted shapes

    //region Rejected shapes

    ...INVALID_INPUTS.map(function mapInvalidInputs(value: unknown,) {
      return it({
        name: `rejects inputs ${String(value,)} with InvalidInputsError`,
        fn: async () => {
          let caught: unknown;
          try {
            matcher({
              inputs: value as never,
              patterns: ['bar'],
            },);
          }
          catch (error) {
            caught = error;
          }
          expect(caught,).toBeInstanceOf(InvalidInputsError,);
          expect((caught as Error).message,).toBe(`Expected 'inputs' to be a string or an array, but got a type of '${typeof value}'`,);
        },
      },);
    },),

    ...INVALID_INPUTS.map(function mapInvalidPatterns(value: unknown,) {
      return it({
        name: `rejects patterns ${String(value,)} with InvalidPatternsError`,
        fn: async () => {
          let caught: unknown;
          try {
            matcher({
              inputs: ['phoenix'],
              patterns: value as never,
            },);
          }
          catch (error) {
            caught = error;
          }
          expect(caught,).toBeInstanceOf(InvalidPatternsError,);
          expect((caught as Error).message,).toBe(`Expected 'patterns' to be a string or an array, but got a type of '${typeof value}'`,);
        },
      },);
    },),

    it({
      name: 'rejects a non-string inputs entry with InvalidInputsError',
      fn: async () => {
        let caught: unknown;
        try {
          matcher({
            inputs: [0] as never,
            patterns: ['bar'],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(InvalidInputsError,);
        expect((caught as Error).message,).toBe('Expected \'inputs\' to be an array of strings, but found a type of \'number\' in the array',);
      },
    },),

    it({
      name: 'rejects a non-string patterns entry with InvalidPatternsError',
      fn: async () => {
        let caught: unknown;
        try {
          matcher({
            inputs: ['phoenix'],
            patterns: [null] as never,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(InvalidPatternsError,);
        expect((caught as Error).message,).toBe('Expected \'patterns\' to be an array of strings, but found a type of \'object\' in the array',);
      },
    },),

    it({
      name: 'validates inputs before patterns, matching upstream precedence',
      fn: async () => {
        let caught: unknown;
        try {
          isMatch({
            inputs: 1 as never,
            patterns: 1 as never,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(InvalidInputsError,);
      },
    },),

    //endregion Rejected shapes
  ],
},);
