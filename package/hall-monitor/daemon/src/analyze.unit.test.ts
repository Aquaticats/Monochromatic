/**
 Equivalence tests for `skipSpacesAndTabs` and its consumer `parseVerdict`.
 
 Capture the pre-refactor behavior of the whitespace-skip cursor so the
 linear single-pass rewrite stays behavior-identical: a cursor at or past
 the end, no leading blank, runs of spaces and tabs, the deliberate
 exclusion of newline / carriage-return / form-feed / vertical-tab (only
 space and tab are skipped), and a long blank run that the prior recursive
 scan would overflow the stack on under engines without tail-call
 elimination. `parseVerdict` covers the same path at the public boundary.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseVerdict,
  skipSpacesAndTabs,
} from './analyze.ts';

/** Run length for the long blank-run case; large enough to overflow the prior recursion. */
const LONG_RUN = 100_000;

await describe({
  name: '',
  children: [
    describe({
      name: skipSpacesAndTabs.name,
      children: [
        it({
          name: 'returns `from` for empty input',
          fn: async () => {
            expect(skipSpacesAndTabs({
              s: '',
              from: 0,
            },),).toBe(0,);
          },
        },),

        it({
          name: 'returns `from` unchanged when it is already past the end',
          fn: async () => {
            expect(skipSpacesAndTabs({
              s: 'abc',
              from: 10,
            },),).toBe(10,);
          },
        },),

        it({
          name: 'returns `from` unchanged when it equals the length',
          fn: async () => {
            expect(skipSpacesAndTabs({
              s: 'abc',
              from: 3,
            },),).toBe(3,);
          },
        },),

        it({
          name: 'returns `from` unchanged when the cursor is on a non-blank char',
          fn: async () => {
            expect(skipSpacesAndTabs({
              s: 'abc',
              from: 0,
            },),).toBe(0,);
          },
        },),

        it({
          name: 'skips a run of spaces',
          fn: async () => {
            expect(skipSpacesAndTabs({
              s: '   x',
              from: 0,
            },),).toBe(3,);
          },
        },),

        it({
          name: 'skips a run of tabs',
          fn: async () => {
            expect(skipSpacesAndTabs({
              s: '\t\tx',
              from: 0,
            },),).toBe(2,);
          },
        },),

        it({
          name: 'skips an interleaved run of spaces and tabs',
          fn: async () => {
            expect(skipSpacesAndTabs({
              s: ' \t \tx',
              from: 0,
            },),).toBe(4,);
          },
        },),

        it({
          name: 'returns the length when the remainder is all blanks',
          fn: async () => {
            expect(skipSpacesAndTabs({
              s: '   ',
              from: 0,
            },),).toBe(3,);
          },
        },),

        it({
          name: 'starts skipping from the given offset, not the string start',
          fn: async () => {
            expect(skipSpacesAndTabs({
              s: 'ab   c',
              from: 2,
            },),).toBe(5,);
          },
        },),

        it({
          name: 'does not skip a newline, carriage return, form feed, or vertical tab',
          fn: async () => {
            expect(skipSpacesAndTabs({
              s: '\nx',
              from: 0,
            },),).toBe(0,);
            expect(skipSpacesAndTabs({
              s: '\rx',
              from: 0,
            },),).toBe(0,);
            expect(skipSpacesAndTabs({
              s: '\fx',
              from: 0,
            },),).toBe(0,);
            expect(skipSpacesAndTabs({
              s: '\vx',
              from: 0,
            },),).toBe(0,);
          },
        },),

        it({
          name: 'stops at the first non-blank after skipping spaces then tab',
          fn: async () => {
            expect(skipSpacesAndTabs({
              s: '  \t\n',
              from: 0,
            },),).toBe(3,);
          },
        },),

        it({
          name: 'skips a long blank run in one linear pass',
          fn: async () => {
            /** Long leading blank run; the prior recursion overflowed the stack here. */
            const s = `${' '.repeat(LONG_RUN,)}x`;
            expect(skipSpacesAndTabs({
              s,
              from: 0,
            },),).toBe(LONG_RUN,);
          },
        },),
      ],
    },),

    describe({
      name: parseVerdict.name,
      children: [
        it({
          name: 'reads the canonical verdict line after a single space',
          fn: async () => {
            expect(parseVerdict('VERDICT: PRODUCTIVE',),).toBe('PRODUCTIVE',);
            expect(parseVerdict('VERDICT: UNPRODUCTIVE',),).toBe('UNPRODUCTIVE',);
          },
        },),

        it({
          name: 'reads the verdict with no separating whitespace',
          fn: async () => {
            expect(parseVerdict('VERDICT:UNPRODUCTIVE',),).toBe('UNPRODUCTIVE',);
          },
        },),

        it({
          name: 'reads the verdict after multiple spaces and tabs',
          fn: async () => {
            expect(parseVerdict('VERDICT:   PRODUCTIVE',),).toBe('PRODUCTIVE',);
            expect(parseVerdict('VERDICT:\t\tUNPRODUCTIVE',),).toBe('UNPRODUCTIVE',);
          },
        },),

        it({
          name: 'matches the verdict line case-insensitively',
          fn: async () => {
            expect(parseVerdict('verdict: productive',),).toBe('PRODUCTIVE',);
          },
        },),

        it({
          name: 'does not read across a newline after the prefix, falling back to keyword scan',
          fn: async () => {
            expect(parseVerdict('VERDICT:\nPRODUCTIVE',),).toBe('PRODUCTIVE',);
          },
        },),

        it({
          name: 'falls back to the UNPRODUCTIVE keyword when no canonical line is present',
          fn: async () => {
            expect(parseVerdict('The user is clearly unproductive.',),).toBe('UNPRODUCTIVE',);
          },
        },),

        it({
          name: 'defaults to PRODUCTIVE when neither a line nor the keyword is present',
          fn: async () => {
            expect(parseVerdict('The user is focused and working.',),).toBe('PRODUCTIVE',);
          },
        },),

        it({
          name: 'reads the canonical line across a long inline blank run without overflowing',
          fn: async () => {
            /** Canonical line with a long blank gap; exercises the skip cursor at the public boundary. */
            const text = `VERDICT:${' '.repeat(LONG_RUN,)}PRODUCTIVE`;
            expect(parseVerdict(text,),).toBe('PRODUCTIVE',);
          },
        },),
      ],
    },),
  ],
},);
