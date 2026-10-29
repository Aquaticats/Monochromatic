/**
 Tests for the package entry point's export surface.
 
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

await describe({
  name: '',
  children: [
    it({
      name: 'exports the matcher entry points and both error classes',
      fn: async () => {
        expect(typeof matcher,).toBe('function',);
        expect(typeof isMatch,).toBe('function',);
        expect(typeof InvalidInputsError,).toBe('function',);
        expect(typeof InvalidPatternsError,).toBe('function',);
      },
    },),

    it({
      name: 'filters inputs through the package entry point',
      fn: async () => {
        expect(matcher({
          inputs: [
            'foo',
            'bar',
            'moo',
          ],
          patterns: [
            '*oo',
            '!foo',
          ],
        },),).toEqual([
          'moo',
        ],);
      },
    },),

    it({
      name: 'answers through the package entry point',
      fn: async () => {
        expect(isMatch({
          inputs: 'unicorn',
          patterns: 'uni*',
        },),).toBe(true,);
      },
    },),

    it({
      name: 'matches when any input matches without allPatterns',
      fn: async () => {
        expect(isMatch({
          inputs: [
            'foo',
            'zoo',
          ],
          patterns: [
            'f*',
            'b*',
          ],
        },),).toBe(true,);
      },
    },),

    it({
      name: 'requires every input under allPatterns with only negations',
      fn: async () => {
        expect(isMatch({
          inputs: [
            'foo',
            'bar',
          ],
          patterns: [
            '!bar',
            '!baz',
          ],
          options: { allPatterns: true, },
        },),).toBe(false,);
        expect(isMatch({
          inputs: [
            'foo',
            'qux',
          ],
          patterns: [
            '!bar',
            '!baz',
          ],
          options: { allPatterns: true, },
        },),).toBe(true,);
        expect(isMatch({
          inputs: [],
          patterns: [
            '!bar',
            '!baz',
          ],
          options: { allPatterns: true, },
        },),).toBe(false,);
      },
    },),
  ],
},);
