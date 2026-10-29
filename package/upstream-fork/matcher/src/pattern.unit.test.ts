/**
 Tests for wildcard pattern splitting and literal-part matching.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  compilePattern,
  matchParts,
  splitPattern,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: splitPattern.name,
      children: [
        it({
          name: 'returns one part when the pattern holds no wildcard',
          fn: async () => {
            expect(splitPattern('abc',),).toEqual([
              'abc',
            ],);
          },
        },),

        it({
          name: 'splits on unescaped wildcards',
          fn: async () => {
            expect(splitPattern('a*b',),).toEqual([
              'a',
              'b',
            ],);
            expect(splitPattern('*a*b*',),).toEqual([
              '',
              'a',
              'b',
              '',
            ],);
          },
        },),

        it({
          name: 'keeps escaped wildcards literal',
          fn: async () => {
            /**
             Backslash code unit for the escaped patterns below.
             */
            const escape = String.fromCodePoint(92,);
            expect(splitPattern(`a${escape}*b`,),).toEqual([
              'a*b',
            ],);
            expect(splitPattern(`*${escape}**`,),).toEqual([
              '',
              '*',
              '',
            ],);
          },
        },),

        it({
          name: 'escapes any character after a backslash',
          fn: async () => {
            /**
             Backslash code unit for the escaped patterns below.
             */
            const escape = String.fromCodePoint(92,);
            expect(splitPattern(`${escape}a${escape}b`,),).toEqual([
              'ab',
            ],);
          },
        },),

        it({
          name: 'keeps a trailing backslash literal',
          fn: async () => {
            /**
             Backslash code unit for the trailing-escape pattern below.
             */
            const escape = String.fromCodePoint(92,);
            expect(splitPattern(`test${escape}`,),).toEqual([
              `test${escape}`,
            ],);
          },
        },),
      ],
    },),

    describe({
      name: matchParts.name,
      children: [
        it({
          name: 'compares exactly when only one part exists',
          fn: async () => {
            expect(matchParts({
              input: 'abc',
              parts: ['abc'],
            },),).toBe(true,);
            expect(matchParts({
              input: 'abcd',
              parts: ['abc'],
            },),).toBe(false,);
          },
        },),

        it({
          name: 'matches prefix and suffix around one wildcard',
          fn: async () => {
            expect(matchParts({
              input: 'foobar',
              parts: [
                'foo',
                'bar',
              ],
            },),).toBe(true,);
            expect(matchParts({
              input: 'foobaz',
              parts: [
                'foo',
                'bar',
              ],
            },),).toBe(false,);
          },
        },),

        it({
          name: 'rejects a missing prefix or suffix',
          fn: async () => {
            expect(matchParts({
              input: 'xbar',
              parts: [
                'foo',
                'bar',
              ],
            },),).toBe(false,);
            expect(matchParts({
              input: 'foox',
              parts: [
                'foo',
                'bar',
              ],
            },),).toBe(false,);
          },
        },),

        it({
          name: 'rejects middle parts past the suffix room',
          fn: async () => {
            expect(matchParts({
              input: 'abc',
              parts: [
                '',
                'bc',
                'abc',
              ],
            },),).toBe(false,);
          },
        },),

        it({
          name: 'rejects overlapping prefix and suffix',
          fn: async () => {
            expect(matchParts({
              input: 'a',
              parts: [
                'a',
                'a',
              ],
            },),).toBe(false,);
            expect(matchParts({
              input: 'aa',
              parts: [
                'a',
                'a',
              ],
            },),).toBe(true,);
          },
        },),

        it({
          name: 'orders middle parts without overlap',
          fn: async () => {
            expect(matchParts({
              input: 'xaxbx',
              parts: [
                '',
                'a',
                'b',
                '',
              ],
            },),).toBe(true,);
            expect(matchParts({
              input: 'xbxax',
              parts: [
                '',
                'a',
                'b',
                '',
              ],
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: compilePattern.name,
      children: [
        it({
          name: 'marks leading-bang patterns as negated',
          fn: async () => {
            expect(compilePattern({
              pattern: '!foo',
              caseSensitive: false,
            },).negated,).toBe(true,);
            expect(compilePattern({
              pattern: 'foo',
              caseSensitive: false,
            },).negated,).toBe(false,);
          },
        },),

        it({
          name: 'tests folded inputs against folded parts',
          fn: async () => {
            /**
             Compiled case-insensitive pattern under test.
             */
            const compiled = compilePattern({
              pattern: 'uni*',
              caseSensitive: false,
            },);
            expect(compiled.test('UNICORN',),).toBe(true,);
            expect(compiled.test('rainbow',),).toBe(false,);
          },
        },),

        it({
          name: 'strips one negation level and keeps the remainder',
          fn: async () => {
            /**
             Backslash code unit for the escaped-bang pattern below.
             */
            const escape = String.fromCodePoint(92,);
            expect(compilePattern({
              pattern: '!!foo',
              caseSensitive: true,
            },).test('!foo',),).toBe(true,);
            expect(compilePattern({
              pattern: `${escape}!foo`,
              caseSensitive: true,
            },).test('!foo',),).toBe(true,);
            expect(compilePattern({
              pattern: 'a*b',
              caseSensitive: true,
            },).test('axb',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
