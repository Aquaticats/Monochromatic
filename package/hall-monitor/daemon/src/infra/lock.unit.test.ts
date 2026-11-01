/**
 Equivalence tests for `splitOnWhitespace`.
 
 Capture the pre-refactor behavior of the whitespace splitter so the
 linear single-pass rewrite stays behavior-identical: empty input, all
 whitespace, a single token, leading and trailing separators dropped,
 consecutive separators collapsed, every whitespace kind treated as a
 break, the realistic `/proc/net/unix` field-extraction case, and long
 repeated runs that the prior recursive O(n) stack and O(n^2) array-spread
 walker could not handle.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { splitOnWhitespace, } from './lock.ts';

/** Run length for the long-input cases; large enough to overflow the prior recursion, fast to compare. */
const LONG_RUN = 100_000;

/** Field index the production caller reads from a split `/proc/net/unix` row. */
const INODE_FIELD_INDEX = 6;

await describe({
  name: '',
  children: [
    describe({
      name: splitOnWhitespace.name,
      children: [
        it({
          name: 'returns an empty array for empty input',
          fn: async () => {
            expect(splitOnWhitespace('',),).toEqual([],);
          },
        },),

        it({
          name: 'returns an empty array for all-spaces input',
          fn: async () => {
            expect(splitOnWhitespace('     ',),).toEqual([],);
          },
        },),

        it({
          name: 'returns an empty array for mixed all-whitespace input',
          fn: async () => {
            expect(splitOnWhitespace(' \t\n\r\f\v ',),).toEqual([],);
          },
        },),

        it({
          name: 'returns a single token when there is no whitespace',
          fn: async () => {
            expect(splitOnWhitespace('abc',),).toEqual(['abc',],);
          },
        },),

        it({
          name: 'splits two space-separated tokens',
          fn: async () => {
            expect(splitOnWhitespace('a b',),).toEqual([
              'a',
              'b',
            ],);
          },
        },),

        it({
          name: 'drops a leading separator',
          fn: async () => {
            expect(splitOnWhitespace('  a b',),).toEqual([
              'a',
              'b',
            ],);
          },
        },),

        it({
          name: 'drops a trailing separator',
          fn: async () => {
            expect(splitOnWhitespace('a b  ',),).toEqual([
              'a',
              'b',
            ],);
          },
        },),

        it({
          name: 'drops both leading and trailing separators',
          fn: async () => {
            expect(splitOnWhitespace('   a b   ',),).toEqual([
              'a',
              'b',
            ],);
          },
        },),

        it({
          name: 'collapses consecutive internal separators into no empty tokens',
          fn: async () => {
            expect(splitOnWhitespace('a    b\t\tc',),).toEqual([
              'a',
              'b',
              'c',
            ],);
          },
        },),

        it({
          name: 'treats every whitespace kind as a token break',
          fn: async () => {
            expect(splitOnWhitespace('a b\tc\nd\re\ff\vg',),).toEqual([
              'a',
              'b',
              'c',
              'd',
              'e',
              'f',
              'g',
            ],);
          },
        },),

        it({
          name: 'extracts the inode field from a realistic /proc/net/unix row',
          fn: async () => {
            /** Sample socket row whose trimmed columns mirror the production layout. */
            const row =
              '0000000000000000: 00000002 00000000 00010000 0001 01 54321 @hall-monitor';
            expect(splitOnWhitespace(row.trim(),)[INODE_FIELD_INDEX],).toBe('54321',);
          },
        },),

        it({
          name: 'returns a single token for one whitespace-bounded character',
          fn: async () => {
            expect(splitOnWhitespace('  x  ',),).toEqual(['x',],);
          },
        },),

        it({
          name: 'handles a long single token in one linear pass',
          fn: async () => {
            /** Long unbroken token; the prior char-by-char recursion overflowed the stack here. */
            const token = 'x'.repeat(LONG_RUN,);
            /** Result for the long single token; expected to be the token verbatim in a one-element array. */
            const result = splitOnWhitespace(token,);
            expect(result.length,).toBe(1,);
            expect(result[0],).toBe(token,);
          },
        },),

        it({
          name: 'handles a long run of many tokens without quadratic blowup',
          fn: async () => {
            /** Many single-character tokens separated by single spaces, plus a trailing space. */
            const result = splitOnWhitespace('a '.repeat(LONG_RUN,),);
            expect(result.length,).toBe(LONG_RUN,);
            expect(result[0],).toBe('a',);
            expect(result[LONG_RUN - 1],).toBe('a',);
          },
        },),
      ],
    },),
  ],
},);
