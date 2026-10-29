/**
 Equivalence tests for `expandEscapes`.
 
 Capture the pre-refactor behavior of the desktop-entry escape expander
 so the linear-pass rewrite stays behavior-identical: each escape map
 entry, unknown escapes passing through as the literal next char, a
 trailing lone backslash emitted verbatim, and a long repeated run.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { expandEscapes, } from '../dist/final/node/launch.mjs';

/** Iteration count for the long-run case; large enough to exercise the linear scan, fast to compare. */
const LONG_RUN = 100_000;

await describe({
  name: '',
  children: [
    describe({
      name: expandEscapes.name,
      children: [
        it({
          name: 'returns empty string for empty input',
          fn: async () => {
            expect(expandEscapes({ s: '', },),).toBe('',);
          },
        },),

        it({
          name: 'passes through text with no backslash unchanged',
          fn: async () => {
            expect(expandEscapes({ s: 'hello world', },),).toBe('hello world',);
          },
        },),

        it({
          name: 'passes through all-whitespace input unchanged',
          fn: async () => {
            expect(expandEscapes({ s: '   ', },),).toBe('   ',);
          },
        },),

        it({
          name: String.raw`expands the documented hello\sworld example`,
          fn: async () => {
            expect(expandEscapes({ s: String.raw`hello\sworld`, },),).toBe('hello world',);
          },
        },),

        it({
          name: String.raw`maps \s to a space`,
          fn: async () => {
            expect(expandEscapes({ s: String.raw`\s`, },),).toBe(' ',);
          },
        },),

        it({
          name: String.raw`maps \n to a newline`,
          fn: async () => {
            expect(expandEscapes({ s: String.raw`\n`, },),).toBe('\n',);
          },
        },),

        it({
          name: String.raw`maps \t to a tab`,
          fn: async () => {
            expect(expandEscapes({ s: String.raw`\t`, },),).toBe('\t',);
          },
        },),

        it({
          name: String.raw`maps \r to a carriage return`,
          fn: async () => {
            expect(expandEscapes({ s: String.raw`\r`, },),).toBe('\r',);
          },
        },),

        it({
          name: 'collapses an escaped backslash to a single backslash',
          fn: async () => {
            expect(expandEscapes({ s: String.raw`\\`, },),).toBe('\\',);
          },
        },),

        it({
          name: 'expands every escape map entry in sequence',
          fn: async () => {
            expect(expandEscapes({ s: String.raw`\s\n\t\r\\`, },),).toBe(' \n\t\r\\',);
          },
        },),

        it({
          name: 'passes an unknown escape through as the literal next char',
          fn: async () => {
            expect(expandEscapes({ s: String.raw`\x`, },),).toBe('x',);
            expect(expandEscapes({ s: String.raw`\9`, },),).toBe('9',);
          },
        },),

        it({
          name: 'emits a trailing lone backslash verbatim',
          fn: async () => {
            expect(expandEscapes({ s: 'abc\\', },),).toBe('abc\\',);
          },
        },),

        it({
          name: 'emits a single lone backslash verbatim',
          fn: async () => {
            expect(expandEscapes({ s: '\\', },),).toBe('\\',);
          },
        },),

        it({
          name: 'expands multiple interspersed escapes',
          fn: async () => {
            expect(expandEscapes({ s: String.raw`a\sb\sc`, },),).toBe('a b c',);
          },
        },),

        it({
          name: 'expands a long repeated run in one linear pass',
          fn: async () => {
            expect(expandEscapes({ s: String.raw`\s`.repeat(LONG_RUN,), },),).toBe(' '.repeat(LONG_RUN,),);
          },
        },),
      ],
    },),
  ],
},);
