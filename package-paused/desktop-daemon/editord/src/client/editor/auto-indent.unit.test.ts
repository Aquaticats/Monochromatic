/**
 * Equivalence tests for the editor auto-indent helpers.
 *
 * Capture the pre-refactor behavior of `leadingSpaces` so the
 * linear-pass rewrite stays behavior-identical: only ASCII space
 * counts (tabs are not leading indentation), the run stops at the first
 * non-space, all-space and empty inputs round-trip, and a long space
 * run is stack-safe. `computeIndent` exercises the same helper through
 * the public surface plus the opening-bracket deepen rule.
 *
 * @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import {
  computeIndent,
  leadingSpaces,
} from './auto-indent.ts';

/** Run length for the stack-safety case; large enough to overflow a recursive cursor under V8, fast on a linear pass. */
const LONG_RUN = 50_000;

await describe({
  name: '',
  children: [
    describe({
      name: leadingSpaces.name,
      children: [
        it({
          name: 'returns empty string for empty input',
          fn: async () => {
            expect(leadingSpaces('',),).toBe('',);
          },
        },),
        it({
          name: 'returns empty string when no leading space is present',
          fn: async () => {
            expect(leadingSpaces('abc',),).toBe('',);
          },
        },),
        it({
          name: 'returns the leading space run before text',
          fn: async () => {
            expect(leadingSpaces('  abc',),).toBe('  ',);
          },
        },),
        it({
          name: 'returns the whole string when it is all spaces',
          fn: async () => {
            expect(leadingSpaces('    ',),).toBe('    ',);
          },
        },),
        it({
          name: 'treats a leading tab as a non-space and returns empty',
          fn: async () => {
            expect(leadingSpaces('\tabc',),).toBe('',);
          },
        },),
        it({
          name: 'stops the run at the first tab after spaces',
          fn: async () => {
            expect(leadingSpaces(' \t abc',),).toBe(' ',);
          },
        },),
        it({
          name: 'collects a long leading-space run without overflow',
          fn: async () => {
            /** Long all-space run followed by a non-space sentinel. */
            const spaces = ' '.repeat(LONG_RUN,);
            expect(leadingSpaces(`${spaces}x`,),).toBe(spaces,);
          },
        },),
      ],
    },),

    describe({
      name: computeIndent.name,
      children: [
        it({
          name: 'matches the previous indentation when no bracket opens',
          fn: async () => {
            expect(computeIndent({ lineText: '  foo', },),).toBe('  ',);
          },
        },),
        it({
          name: 'returns empty indentation for an empty line',
          fn: async () => {
            expect(computeIndent({ lineText: '', },),).toBe('',);
          },
        },),
        it({
          name: 'deepens by one unit after a trailing opening brace',
          fn: async () => {
            expect(computeIndent({ lineText: '  if (true) {', },),).toBe('    ',);
          },
        },),
        it({
          name: 'deepens after a trailing opening parenthesis',
          fn: async () => {
            expect(computeIndent({ lineText: '    call(', },),).toBe('      ',);
          },
        },),
        it({
          name: 'deepens after a trailing opening bracket',
          fn: async () => {
            expect(computeIndent({ lineText: '  arr[', },),).toBe('    ',);
          },
        },),
        it({
          name: 'ignores trailing whitespace when inspecting the last character',
          fn: async () => {
            expect(computeIndent({ lineText: '  block {   ', },),).toBe('    ',);
          },
        },),
        it({
          name: 'deepens from zero indentation when a bracket opens at column zero',
          fn: async () => {
            expect(computeIndent({ lineText: 'top {', },),).toBe('  ',);
          },
        },),
      ],
    },),
  ],
},);
