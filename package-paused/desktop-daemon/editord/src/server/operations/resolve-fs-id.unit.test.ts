/**
 * Equivalence tests for the Windows `vol` serial-number parser.
 *
 * Capture the pre-refactor behavior of `parseVolumeSerial` so the
 * linear-pass rewrite stays behavior-identical: case-insensitive label
 * match, inline-whitespace skip (space and tab only) before the token,
 * token accumulation halting at any of the six ASCII whitespace
 * characters, the empty-token result when the label is absent or trails
 * into whitespace, a realistic multi-line `vol` block, and stack safety
 * on a long token or a long leading-whitespace run.
 *
 * @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import { parseVolumeSerial, } from './resolve-fs-id.ts';

/** Run length for the stack-safety cases; large enough to overflow a recursive cursor under V8, fast on a linear pass. */
const LONG_RUN = 50_000;

/** Whitespace characters that terminate the serial token; each must end accumulation at the same offset. */
const TERMINATORS = [
  ' ',
  '\t',
  '\n',
  '\r',
  '\f',
  '\v',
];

await describe({
  name: '',
  children: [
    describe({
      name: parseVolumeSerial.name,
      children: [
        //region Empty / no label -> empty string

        it({
          name: 'returns empty string for empty input',
          fn: async () => {
            expect(parseVolumeSerial('',),).toBe('',);
          },
        },),
        it({
          name: 'returns empty string when the label is absent',
          fn: async () => {
            expect(parseVolumeSerial(' Volume in drive C has no label.',),).toBe('',);
          },
        },),
        it({
          name: 'returns empty string when nothing follows the label',
          fn: async () => {
            expect(parseVolumeSerial('Serial Number is',),).toBe('',);
          },
        },),
        it({
          name: 'returns empty string when only whitespace follows the label',
          fn: async () => {
            expect(parseVolumeSerial('Serial Number is   \t',),).toBe('',);
          },
        },),

        //endregion Empty / no label -> empty string

        //region Token extraction

        it({
          name: 'extracts a serial token separated by a single space',
          fn: async () => {
            expect(parseVolumeSerial('Serial Number is 1A2B-3C4D',),).toBe('1A2B-3C4D',);
          },
        },),
        it({
          name: 'skips a tab between the label and the token',
          fn: async () => {
            expect(parseVolumeSerial('Serial Number is\t1A2B-3C4D',),).toBe('1A2B-3C4D',);
          },
        },),
        it({
          name: 'skips a mixed run of spaces and tabs before the token',
          fn: async () => {
            expect(parseVolumeSerial('Serial Number is  \t  1A2B-3C4D',),).toBe('1A2B-3C4D',);
          },
        },),

        //endregion Token extraction

        //region Case-insensitive label

        it({
          name: 'matches an all-lowercase label',
          fn: async () => {
            expect(parseVolumeSerial('serial number is ABCD',),).toBe('ABCD',);
          },
        },),
        it({
          name: 'matches an all-uppercase label',
          fn: async () => {
            expect(parseVolumeSerial('SERIAL NUMBER IS ABCD',),).toBe('ABCD',);
          },
        },),

        //endregion Case-insensitive label

        //region Token terminators

        ...TERMINATORS.map(function mapTerminator(term,) {
          /** Human-readable escape so the test name stays legible for control characters. */
          const label = JSON.stringify(term,);
          return it({
            name: `halts the token at the ${label} whitespace character`,
            fn: async () => {
              expect(parseVolumeSerial(`Serial Number is AB${term}CD`,),).toBe('AB',);
            },
          },);
        },),

        //endregion Token terminators

        //region Realistic multi-line vol output

        it({
          name: 'extracts the serial from a full vol output block',
          fn: async () => {
            expect(
              parseVolumeSerial(
                ' Volume in drive C has no label.\r\n Volume Serial Number is 1A2B-3C4D\r\n',
              ),
            ).toBe('1A2B-3C4D',);
          },
        },),

        //endregion Realistic multi-line vol output

        //region Stack safety on long runs

        it({
          name: 'accumulates a long token without overflow',
          fn: async () => {
            /** Long all-`A` token; the linear scan must reproduce it exactly. */
            const token = 'A'.repeat(LONG_RUN,);
            expect(parseVolumeSerial(`Serial Number is ${token}`,),).toBe(token,);
          },
        },),
        it({
          name: 'skips a long leading-whitespace run without overflow',
          fn: async () => {
            expect(
              parseVolumeSerial(`Serial Number is${' '.repeat(LONG_RUN,)}XY`,),
            ).toBe('XY',);
          },
        },),

        //endregion Stack safety on long runs
      ],
    },),
  ],
},);
