/**
 Equivalence tests for `firstNonWhitespaceToken`.
 
 Lock in the pre-refactor behavior of the leading-token scanner so the
 recursion-to-linear-pass rewrite stays behavior-identical: empty and
 all-whitespace input, the no-whitespace whole-string case, leading-
 whitespace skipping, trailing-whitespace trimming, each ASCII whitespace
 character acting as a boundary, the ASCII-only set (a non-breaking space is
 not a boundary), and long repeated runs that a per-character recursion would
 overflow on a V8 target.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { firstNonWhitespaceToken, } from './html.ts';

/** Run length for the long-input cases; large enough to overflow a per-character recursion on V8, trivial for a linear scan. */
const LONG_RUN = 200_000;

await describe({
  name: '',
  children: [
    describe({
      name: firstNonWhitespaceToken.name,
      children: [
        it({
          name: 'returns an empty token for the empty string',
          fn: async () => {
            expect(firstNonWhitespaceToken('',),).toBe('',);
          },
        },),

        it({
          name: 'returns an empty token for an all-whitespace line',
          fn: async () => {
            expect(firstNonWhitespaceToken('   ',),).toBe('',);
          },
        },),

        it({
          name: 'returns an empty token for a line of every ASCII whitespace character',
          fn: async () => {
            expect(firstNonWhitespaceToken(' \t\n\r\f\v',),).toBe('',);
          },
        },),

        it({
          name: 'returns the whole string when there is no whitespace',
          fn: async () => {
            expect(firstNonWhitespaceToken('a.jpg',),).toBe('a.jpg',);
          },
        },),

        it({
          name: 'returns the leading token up to the first space',
          fn: async () => {
            expect(firstNonWhitespaceToken('a.jpg 2x',),).toBe('a.jpg',);
          },
        },),

        it({
          name: 'skips leading whitespace before the token',
          fn: async () => {
            expect(firstNonWhitespaceToken('   a.jpg 2x',),).toBe('a.jpg',);
          },
        },),

        it({
          name: 'skips leading whitespace and ignores trailing whitespace',
          fn: async () => {
            expect(firstNonWhitespaceToken('  a.jpg  ',),).toBe('a.jpg',);
          },
        },),

        it({
          name: 'treats a tab as a token boundary',
          fn: async () => {
            expect(firstNonWhitespaceToken('a\tb',),).toBe('a',);
          },
        },),

        it({
          name: 'treats a newline as a token boundary',
          fn: async () => {
            expect(firstNonWhitespaceToken('a\nb',),).toBe('a',);
          },
        },),

        it({
          name: 'treats a carriage return as a token boundary',
          fn: async () => {
            expect(firstNonWhitespaceToken('a\rb',),).toBe('a',);
          },
        },),

        it({
          name: 'treats a form feed as a token boundary',
          fn: async () => {
            expect(firstNonWhitespaceToken('a\fb',),).toBe('a',);
          },
        },),

        it({
          name: 'treats a vertical tab as a token boundary',
          fn: async () => {
            expect(firstNonWhitespaceToken('a\vb',),).toBe('a',);
          },
        },),

        it({
          name: 'does not treat a non-breaking space as a boundary (ASCII-only set)',
          fn: async () => {
            expect(firstNonWhitespaceToken(`a${String.fromCodePoint(160,)}b`,),).toBe(`a${String.fromCodePoint(160,)}b`,);
          },
        },),

        it({
          name: 'skips a long leading-whitespace run without overflowing the stack',
          fn: async () => {
            expect(firstNonWhitespaceToken(`${' '.repeat(LONG_RUN,)}x`,),).toBe('x',);
          },
        },),

        it({
          name: 'scans a long token run without overflowing the stack',
          fn: async () => {
            expect(
              firstNonWhitespaceToken('a'.repeat(LONG_RUN,),),
            ).toBe('a'.repeat(LONG_RUN,),);
          },
        },),
      ],
    },),
  ],
},);
