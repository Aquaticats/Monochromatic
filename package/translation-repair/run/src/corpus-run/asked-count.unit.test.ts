/**
 Tests for reading how many units a bench or calibration was asked for.

 THE REFUSAL IS THE POINT, and it replaced four copies of a silent fallback.
 `editor-calibrate`, `producer-calibrate`, `roster-bench` and
 `editor-width-probe` each spelled this `Number(process.argv[2] ?? default)`
 and none checked the result. `Number('fourty')` is `NaN`, and
 `pickSpreadSample` with a count of `NaN` returns nothing, measured: `count
 NaN -> picked 0`. So a typo ran the whole calibration over an empty sample,
 printed its roster and its totals, and exited zero.

 WHY REFUSING BEATS FALLING BACK. A fallback also hides the typo, and it
 spends a roster while hiding it. The operator who typed `fourty` wanted forty
 slices, and would read a clean six-slice default as the forty they asked for.

 ZERO IS REFUSED, WHICH THE AUDIT'S `--cap 0` IS NOT. That cap reads a whole
 archive and buys nothing, which is a real use. A bench over zero slices asks
 nobody anything, so there is nothing for it to mean.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  readAskedCount,
  StatedRefusalError,
} from '@monochromatic-dev/translation-repair-glossary';
import { lineOf, } from './command-line.test-fixture.ts';

//region Asked count tests

/**
 Count a run does when nobody names one.
 */
const FALLBACK = 6;

/**
 Count one operator asked for instead.
 */
const ASKED_FOR = 40;

/**
 What the counted things are called, which the refusal has to name.
 */
const ASKS = 'slices';

/**
 A count typed the way a person mistypes one.
 */
const MISTYPED = 'fourty';

/**
 How the reader names the counts it takes: whole, in digits, and no larger
 than a double holds exactly.
 */
const WHOLE_NUMBER_RULE = `a whole number written in digits, at most ${String(Number.MAX_SAFE_INTEGER,)}`;

/**
 Reads a count off a bench's command line carrying only what was typed, read
 as `reportingRefusals` reads it (ledger B75).

 @param typed - what the operator wrote after the script path

 @param asks - what the counted things are called

 @returns Count the reader settled on

 @throws StatedRefusalError when the line or the reader refuses what was typed

 @example
 ```ts
 const wanted = countFrom({ typed: ['40',], asks: ASKS, },);
 ```
 */
function countFrom(
  {
    typed,
    asks,
  }: {
    readonly typed: readonly string[];
    readonly asks: string;
  },
): number {
  return readAskedCount({
    line: lineOf({
      command: 'editor-calibrate',
      typed,
    },),
    fallback: FALLBACK,
    asks,
  },);
}

await describe({
  name: readAskedCount.name,
  children: [
    it({
      name: 'FALLS BACK when nobody named a count',
      fn: async () => {
        expect(countFrom({
          typed: [],
          asks: ASKS,
        },),).toBe(FALLBACK,);
      },
    },),
    it({
      name: 'READS the count that was named',
      fn: async () => {
        expect(countFrom({
          typed: [String(ASKED_FOR,),],
          asks: ASKS,
        },),).toBe(ASKED_FOR,);
      },
    },),
    it({
      name: 'REFUSES a fractional count, and one written with an exponent, a radix, a sign or past the largest '
        + 'whole number a double holds exactly, rather than running a count nobody typed (ledger B73)',
      fn: async () => {
        /**
         Counts that are no whole number written in digits.
         */
        const counts = [
          '40.9',
          '4e1',
          '0x28',
          '+40',
          String(BigInt(Number.MAX_SAFE_INTEGER,) + 2n,),
        ];
        expect(counts.map(function refusalOf(count,): string {
          /**
           What the reader threw.
           */
          const refusal = caught(function readsCount(): void {
            countFrom({
              typed: [count,],
              asks: ASKS,
            },);
          },);
          expect(refusal,).toBeInstanceOf(StatedRefusalError,);
          return (refusal as Error).message;
        },),).toEqual(counts.map(function expectedOf(count,): string {
          return `${ASKS} must be ${WHOLE_NUMBER_RULE}, and ${JSON.stringify(count,)} is not one`;
        },),);
      },
    },),
    it({
      name: 'REFUSES a count that is not a number, instead of running over nothing',
      fn: async () => {
        expect(() => {
          countFrom({
            typed: [MISTYPED,],
            asks: ASKS,
          },);
        },).toThrow(StatedRefusalError,);
      },
    },),
    it({
      name: 'NAMES both the units and what was typed, so the operator can see the typo',
      fn: async () => {
        /**
         What the reader threw.
         */
        const refusal = caught(function readsWord(): void {
          countFrom({
            typed: [MISTYPED,],
            asks: ASKS,
          },);
        },);
        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect((refusal as Error).message,).toBe(
          `${ASKS} must be ${WHOLE_NUMBER_RULE}, and ${JSON.stringify(MISTYPED,)} is not one`,
        );
      },
    },),
    it({
      name: 'REFUSES a count of zero, which would ask nobody anything',
      fn: async () => {
        expect(() => {
          countFrom({
            typed: ['0',],
            asks: ASKS,
          },);
        },).toThrow(StatedRefusalError,);
      },
    },),
    it({
      name: 'REFUSES a negative count, which no sampler can take',
      fn: async () => {
        expect(() => {
          countFrom({
            typed: ['-3',],
            asks: ASKS,
          },);
        },).toThrow('slices must be at least 1, and -3 is not',);
      },
    },),
    it({
      name: 'REFUSES a count that is a number with something after it',
      fn: async () => {
        // `Number('40slices')` is `NaN`, so this lands on the same refusal as a
        // word. Held separately because it is the near miss a person actually
        // types, and because a reader built on `parseInt` would accept it as 40.
        expect(() => {
          countFrom({
            typed: ['40slices',],
            asks: ASKS,
          },);
        },).toThrow(StatedRefusalError,);
      },
    },),
    it({
      name: 'REFUSES an unbounded count, which truncates to itself and never lands',
      fn: async () => {
        expect(() => {
          countFrom({
            typed: ['Infinity',],
            asks: ASKS,
          },);
        },).toThrow(StatedRefusalError,);
      },
    },),
    it({
      name: 'REFUSES an argument written empty, as an empty flag value is refused, rather than running the '
        + 'default a script with an unset variable never asked for (ledger B75)',
      fn: async () => {
        /**
         What the reader threw.
         */
        const refusal = caught(function readsEmpty(): void {
          countFrom({
            typed: ['',],
            asks: ASKS,
          },);
        },);
        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect((refusal as Error).message,).toBe(`${ASKS} must be ${WHOLE_NUMBER_RULE}, and "" is not one`,);
      },
    },),
    it({
      name: 'ANSWERS in the caller\'s own words, so two runs do not share a noun',
      fn: async () => {
        /**
         What the reader threw.
         */
        const refusal = caught(function readsEntries(): void {
          countFrom({
            typed: [MISTYPED,],
            asks: 'entries',
          },);
        },);
        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect((refusal as Error).message,).toBe(
          `entries must be ${WHOLE_NUMBER_RULE}, and ${JSON.stringify(MISTYPED,)} is not one`,
        );
      },
    },),
  ],
},);

//endregion Asked count tests
