/**
 Tests that the overlap dial REFUSES rather than falling back.

 WHY A FALLBACK WOULD BE THE WORST OUTCOME. This dial exists so two runs can
 differ in exactly one value and nothing else. A value it cannot read that
 quietly became `1` would produce two sequential runs, a comparison showing no
 difference, and a recorded conclusion that overlapping units does nothing.
 That is not a failed measurement, it is a wrong one, and it would be believed.

 THE DEFAULT IS STILL A FALLBACK, deliberately: an UNSET, empty or blank
 variable is an invoker who did not ask for overlap, which is a different
 thing from one who asked for something unreadable.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  CALIBRATION_OVERLAP,
  readOverlap,
  readOverlapSetting,
} from '../../dist/final/node/index.mjs';
import { dialSaying, } from './slice-overlap-dial.test-fixture.ts';

//region Fixtures

/**
 Environment variable naming how many slices may run at once.
 */
const OVERLAP_VAR = 'TRANSLATION_REPAIR_SLICE_OVERLAP';

//endregion Fixtures

await describe({
  name: readOverlap.name,
  // ONE AT A TIME. The case showing the reader takes the environment it is
  // handed points the process-wide variable elsewhere, and `describe` runs its
  // children concurrently by default.
  concurrency: 1,
  children: [
    it({
      name: 'RETURNS THE CALLER\'S FALLBACK when nobody asked, so a driver decided to run sequentially '
        + 'still does and the control arm of any comparison is the same program',
      fn: async () => {
        expect(readOverlap({ fallback: 1, env: {}, },),).toBe(1,);
        expect(readOverlapSetting({ fallback: 1, env: {}, },),).toEqual({
          overlap: 1,
          source: 'fallback',
        },);
      },
    },),

    it({
      name: 'READS THE ENVIRONMENT IT IS HANDED, not the process\'s own variable, whose unreadable value would '
        + 'refuse the read',
      fn: async () => {
        using dial = dialSaying({ says: 'several', },);

        expect(readOverlapSetting({
          fallback: 1,
          env: { [OVERLAP_VAR]: '2', },
        },),).toEqual({
          overlap: 2,
          source: OVERLAP_VAR,
        },);
      },
    },),

    it({
      name: 'READS a whole number, which is the case the dial exists for',
      fn: async () => {
        /**
         Environment the reader is handed.
         */
        const env = { [OVERLAP_VAR]: '4', };

        expect(readOverlap({ fallback: 1, env, },),).toBe(4,);
        expect(readOverlapSetting({ fallback: 1, env, },),).toEqual({
          overlap: 4,
          source: OVERLAP_VAR,
        },);
      },
    },),

    it({
      name: 'READS a whole number written with a leading zero as the number it names, by the one rule every '
        + 'count reader in the package now shares (ledger B73)',
      fn: async () => {
        /**
         Environment the reader is handed.
         */
        const env = { [OVERLAP_VAR]: '04', };

        expect(readOverlap({ fallback: 1, env, },),).toBe(4,);
      },
    },),

    it({
      name: 'RETURNS THE CALLER\'S FALLBACK for a variable set to blanks, as it does for one set empty and as the '
        + 'grace, writer grace, cap, spend ceiling, credit and request pace dials do, since blanks ask for no '
        + 'overlap at all; it read them as an unreadable value and refused (ledger B73)',
      fn: async () => {
        /**
         Environment the reader is handed.
         */
        const env = { [OVERLAP_VAR]: '  ', };

        expect(readOverlapSetting({ fallback: 1, env, },),).toEqual({
          overlap: 1,
          source: 'fallback',
        },);
      },
    },),

    it({
      name: 'REFUSES a value that is not a number rather than falling back to one, because a typo '
        + 'that quietly became sequential would produce a comparison of two identical runs and a '
        + 'recorded conclusion that overlapping units changes nothing',
      fn: async () => {
        /**
         Environment the reader is handed.
         */
        const env = { [OVERLAP_VAR]: 'four', };

        /**
         What the reader threw on a value nothing could read.
         */
        const refusal = caught(function readsProse() {
          readOverlap({ fallback: 1, env, },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect((refusal as Error).message,).toContain(OVERLAP_VAR,);
        expect((refusal as Error).message,).toContain('four',);
      },
    },),

    it({
      name: 'REFUSES a fractional value rather than truncating it into another arm, '
        + 'because overlap 1.5 is neither overlap 1 nor overlap 2',
      fn: async () => {
        /**
         Environment the reader is handed.
         */
        const env = { [OVERLAP_VAR]: '1.5', };

        const refusal = caught(function readsFraction() {
          readOverlap({ fallback: 1, env, },);
        },);
        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect((refusal as Error).message,).toContain('whole number',);
        expect((refusal as Error).message,).toContain('1.5',);
      },
    },),

    it({
      name: 'REFUSES what is no whole number written in digits rather than silently turning hexadecimal, '
        + 'exponent, decimal-point, signed, padded or minus-zero input into another arm (ledger B73)',
      fn: async () => {
        /**
         Numeric strings JavaScript would otherwise read as a number.
         */
        const spellings = [
          '0x2',
          '1e1',
          '2.0',
          ' 2',
          '+2',
          '-0',
        ];
        for (const spelling of spellings) {
          const refusal = caught(function readsNotDigits() {
            readOverlap({
              fallback: 1,
              env: { [OVERLAP_VAR]: spelling, },
            },);
          },);
          expect(refusal,).toBeInstanceOf(StatedRefusalError,);
          expect((refusal as Error).message,).toBe(
            `${OVERLAP_VAR} must be a whole number written in digits, at most `
              + `${String(Number.MAX_SAFE_INTEGER,)}, and ${spelling} is not one`,
          );
        }
      },
    },),

    it({
      name: 'REFUSES zero, which is not a smaller amount of work but no work at all: a limit of zero '
        + 'admits nothing and the run would wait forever having said nothing about why',
      fn: async () => {
        /**
         Environment the reader is handed.
         */
        const env = { [OVERLAP_VAR]: '0', };

        /**
         What the reader threw on a limit that admits nothing.
         */
        const refusal = caught(function readsZero() {
          readOverlap({ fallback: 1, env, },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect((refusal as Error).message,).toContain('at least 1',);
      },
    },),

    it({
      name: 'GIVES THE CALIBRATION FOUR when nobody asked, which is the owner\'s decision of 2026-08-26 '
        + 'on arm B against arms A and A2, and lets the variable bring it back down to one',
      fn: async () => {
        expect(CALIBRATION_OVERLAP,).toBe(4,);
        expect(readOverlap({ fallback: CALIBRATION_OVERLAP, env: {}, },),).toBe(4,);
      },
    },),

    it({
      name: 'LETS A LAUNCH BRING THE CALIBRATION BACK TO ONE, so the sequential arm of a comparison is '
        + 'still one variable away',
      fn: async () => {
        /**
         Environment the reader is handed.
         */
        const env = { [OVERLAP_VAR]: '1', };

        expect(readOverlap({ fallback: CALIBRATION_OVERLAP, env, },),).toBe(1,);
      },
    },),
  ],
},);
