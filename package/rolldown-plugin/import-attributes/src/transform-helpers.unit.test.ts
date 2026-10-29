/**
 Equivalence tests for `skipWithClauseWhitespace`.
 
 Capture the pre-refactor behavior of the with-clause whitespace skipper
 so the linear-pass rewrite stays behavior-identical: empty input, an
 all-whitespace run, an immediate non-whitespace char (no match), a
 cursor at or past `s.length`, each of the four permitted whitespace
 chars, a char outside that set (form feed, which `\s` would match but
 this scanner must not), a mid-string scan after a quoted path, trailing
 whitespace with no following token, and a long repeated run that would
 overflow the prior recursive scan's stack.
 
 The generic delimiter edge cases (unmatched delimiter, both path
 separators) do not apply: this function scans a whitespace prefix and
 has no delimiter or path concept.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { skipWithClauseWhitespace, } from './transform-helpers.ts';

/** Whitespace-run length for the long-run case; large enough to overflow the prior recursive scan, fast to compare. */
const LONG_RUN = 100_000;

await describe({
  name: '',
  children: [
    describe({
      name: skipWithClauseWhitespace.name,
      children: [
        it({
          name: 'returns the starting index for empty input',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: '', idx: 0, },),).toBe(0,);
          },
        },),

        it({
          name: 'skips an all-whitespace run to end of string',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: '    ', idx: 0, },),).toBe(4,);
          },
        },),

        it({
          name: 'returns the starting index when the cursor is already on a non-whitespace char',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: 'with', idx: 0, },),).toBe(0,);
          },
        },),

        it({
          name: 'returns the index unchanged when the cursor equals s.length',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: 'ab', idx: 2, },),).toBe(2,);
          },
        },),

        it({
          name: 'returns the index unchanged when the cursor is past s.length',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: 'ab', idx: 5, },),).toBe(5,);
          },
        },),

        it({
          name: 'advances past a single space',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: ' x', idx: 0, },),).toBe(1,);
          },
        },),

        it({
          name: 'advances past a single tab',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: '\tx', idx: 0, },),).toBe(1,);
          },
        },),

        it({
          name: 'advances past a single newline',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: '\nx', idx: 0, },),).toBe(1,);
          },
        },),

        it({
          name: 'advances past a single carriage return',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: '\rx', idx: 0, },),).toBe(1,);
          },
        },),

        it({
          name: 'advances past a mixed run of all four whitespace chars',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: ' \t\n\rwith', idx: 0, },),).toBe(4,);
          },
        },),

        it({
          name: String.raw`stops at a form feed (outside the four-char set that \s would match)`,
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: '\fwith', idx: 0, },),).toBe(0,);
          },
        },),

        it({
          name: 'scans the whitespace gap mid-string after a quoted path',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: '\'./x.sql\'   with', idx: 9, },),).toBe(12,);
          },
        },),

        it({
          name: 'skips trailing whitespace with no following token to end of string',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: 'with   ', idx: 4, },),).toBe(7,);
          },
        },),

        it({
          name: 'skips a long whitespace run in one linear pass without stack overflow',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: ' '.repeat(LONG_RUN,), idx: 0, },),).toBe(LONG_RUN,);
          },
        },),

        it({
          name: 'finds the first non-whitespace char after a long whitespace run',
          fn: async () => {
            expect(
              skipWithClauseWhitespace({ s: `${' '.repeat(LONG_RUN,)}with`, idx: 0, },),
            ).toBe(LONG_RUN,);
          },
        },),
      ],
    },),
  ],
},);
