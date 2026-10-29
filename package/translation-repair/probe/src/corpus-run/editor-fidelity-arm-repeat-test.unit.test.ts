/**
 Tests for the exact test the repeat warning of the read step is made of: whether the default effort's count of trials that
 read alike in every pass is low against another arm's, by a one-sided Fisher exact test of the two counts, at a level
 of 0.05 divided by the number of other arms so that the chance of a warning when every arm repeats itself alike stays under
 0.05. The p-values are the hypergeometric tail summed exactly, and the level is decided in whole numbers, so a p-value
 of exactly one in twenty is at the level and a float cannot move it.

 The p-values in the cases were computed twice, exactly with big integers and by log-gamma floats, and agree to six
 decimals. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  REPEAT_ALPHA,
  repeatAgreementTest,
  repeatShortfallsOf,
} from '../../dist/final/node/index.mjs';

/**
 One arm's counts, with a name so that the arm picked can be told.

 @param arm - name
 @param agreed - trials whose passes all read alike
 @param repeated - trials asked in more than one pass

 @returns Figure

 @example
 ```ts
 const figure = figureOf({ arm: 'high', agreed: 20, repeated: 20, },);
 ```
 */
function figureOf(
  {
    arm,
    agreed,
    repeated,
  }: {
    readonly arm: string;
    readonly agreed: number;
    readonly repeated: number;
  },
): {
  readonly arm: string;
  readonly agreed: number;
  readonly repeated: number
} {
  return {
    arm,
    agreed,
    repeated,
  };
}

await describe({
  name: 'the exact test of the repeat agreement',
  children: [
    describe({
      name: repeatAgreementTest.name,
      children: [
        it({
          name: 'SUMS THE HYPERGEOMETRIC TAIL EXACTLY: 15 of 20 trials against 20 of 20 is 0.0236, 16 of 20 is 0.0530, 9 of 12 '
            + 'against 12 of 12 is 0.1087, 38 of 44 against 44 of 44 is 0.0130 and 30 of 44 against 38 of 44 is 0.0367',
          fn: async () => {
            expect(repeatAgreementTest({ low: { agreed: 15, repeated: 20, }, high: { agreed: 20, repeated: 20, }, },).p,).toBeCloseTo(0.023562, 5,);
            expect(repeatAgreementTest({ low: { agreed: 16, repeated: 20, }, high: { agreed: 20, repeated: 20, }, },).p,).toBeCloseTo(0.053015, 5,);
            expect(repeatAgreementTest({ low: { agreed: 9, repeated: 12, }, high: { agreed: 12, repeated: 12, }, },).p,).toBeCloseTo(0.108696, 5,);
            expect(repeatAgreementTest({ low: { agreed: 38, repeated: 44, }, high: { agreed: 44, repeated: 44, }, },).p,).toBeCloseTo(0.013026, 5,);
            expect(repeatAgreementTest({ low: { agreed: 30, repeated: 44, }, high: { agreed: 38, repeated: 44, }, },).p,).toBeCloseTo(0.036689, 5,);
          },
        },),

        it({
          name: 'DECIDES THE LEVEL IN WHOLE NUMBERS: 0 of 3 trials against 3 of 3 has a p-value of exactly one in twenty, which is '
            + 'at the level of 0.05 and not at 0.05 divided by two',
          fn: async () => {
            const low = { agreed: 0, repeated: 3, };
            const high = { agreed: 3, repeated: 3, };
            expect(repeatAgreementTest({ low, high, },).atMostLevel,).toBe(true,);
            expect(repeatAgreementTest({ low, high, comparisons: 1, },).atMostLevel,).toBe(true,);
            expect(repeatAgreementTest({ low, high, comparisons: 2, },).atMostLevel,).toBe(false,);
            expect(repeatAgreementTest({ low, high, },).p,).toBeCloseTo(0.05, 10,);
          },
        },),

        it({
          name: 'DECIDES AT THE LEVEL THE WARNING PRINTS: REPEAT_ALPHA is 0.05, and the decision is made at one in twenty, '
            + 'which the two spellings of the level must keep saying together, so 0 of 3 against 3 of 3 (exactly 0.05) '
            + 'and 15 of 20 against 20 of 20 (0.0236) are at it and 16 of 20 against 20 of 20 (0.0530) is not',
          fn: async () => {
            expect(REPEAT_ALPHA,).toBe(0.05,);
            expect(repeatAgreementTest({ low: { agreed: 0, repeated: 3, }, high: { agreed: 3, repeated: 3, }, },).atMostLevel,).toBe(true,);
            expect(repeatAgreementTest({ low: { agreed: 15, repeated: 20, }, high: { agreed: 20, repeated: 20, }, },).atMostLevel,).toBe(true,);
            expect(repeatAgreementTest({ low: { agreed: 16, repeated: 20, }, high: { agreed: 20, repeated: 20, }, },).atMostLevel,).toBe(false,);
          },
        },),

        it({
          name: 'DIVIDES THE LEVEL BY THE NUMBER OF COMPARISONS: 15 of 20 against 20 of 20 (0.0236) is at the level for one and two '
            + 'comparisons (0.05 and 0.025) and over it for three (0.0167), and 16 of 20 (0.0530) is over it for one',
          fn: async () => {
            const low = { agreed: 15, repeated: 20, };
            const high = { agreed: 20, repeated: 20, };
            expect(repeatAgreementTest({ low, high, comparisons: 1, },).atMostLevel,).toBe(true,);
            expect(repeatAgreementTest({ low, high, comparisons: 2, },).atMostLevel,).toBe(true,);
            expect(repeatAgreementTest({ low, high, comparisons: 3, },).atMostLevel,).toBe(false,);
            expect(repeatAgreementTest({ low: { agreed: 16, repeated: 20, }, high, comparisons: 1, },).atMostLevel,).toBe(false,);
          },
        },),

        it({
          name: 'TESTS ONE SIDE ONLY: an arm that repeats itself more than the one it is set against, and arms alike, are not at '
            + 'the level however far apart they are',
          fn: async () => {
            expect(repeatAgreementTest({ low: { agreed: 44, repeated: 44, }, high: { agreed: 10, repeated: 44, }, },).atMostLevel,).toBe(false,);
            expect(repeatAgreementTest({ low: { agreed: 68, repeated: 88, }, high: { agreed: 68, repeated: 88, }, },).atMostLevel,).toBe(false,);
            expect(repeatAgreementTest({ low: { agreed: 1, repeated: 1, }, high: { agreed: 1, repeated: 1, }, },).p,).toBe(1,);
          },
        },),

        it({
          name: 'NEEDS A LARGER DIFFERENCE WITH FEWER TRIALS ASKED AGAIN and has no floor on them: 9 of 12 against 12 of 12 '
            + '(25 points) and 8 of 12 (33 points) are over the level for two comparisons, 7 of 12 (42 points) is at it, '
            + '0 of 4 against 4 of 4 is at the level for one, and 0 of 2 against 2 of 2 is not',
          fn: async () => {
            const twelve = { agreed: 12, repeated: 12, };
            expect(repeatAgreementTest({ low: { agreed: 9, repeated: 12, }, high: twelve, comparisons: 2, },).atMostLevel,).toBe(false,);
            expect(repeatAgreementTest({ low: { agreed: 8, repeated: 12, }, high: twelve, comparisons: 2, },).atMostLevel,).toBe(false,);
            expect(repeatAgreementTest({ low: { agreed: 7, repeated: 12, }, high: twelve, comparisons: 2, },).atMostLevel,).toBe(true,);
            expect(repeatAgreementTest({ low: { agreed: 0, repeated: 4, }, high: { agreed: 4, repeated: 4, }, },).atMostLevel,).toBe(true,);
            expect(repeatAgreementTest({ low: { agreed: 0, repeated: 2, }, high: { agreed: 2, repeated: 2, }, },).atMostLevel,).toBe(false,);
          },
        },),

        it({
          name: 'HOLDS BIG COUNTS WITHOUT LOSING THE TAIL: 68 of 88 trials against 88 of 88 is far under one in a million, and the '
            + 'level is decided exactly',
          fn: async () => {
            const test = repeatAgreementTest({ low: { agreed: 68, repeated: 88, }, high: { agreed: 88, repeated: 88, }, },);
            expect(test.atMostLevel,).toBe(true,);
            expect(test.p,).toBeLessThan(0.000001,);
          },
        },),
      ],
    },),

    describe({
      name: repeatShortfallsOf.name,
      children: [
        it({
          name: 'SETS THE DEFAULT EFFORT AGAINST THE OTHER ARM WITH THE HIGHEST SHARE, the first of the arms among equals, and '
            + 'names it with the p-value',
          fn: async () => {
            const figure = figureOf({ arm: 'medium', agreed: 20, repeated: 44, },);
            const others = [
              figureOf({ arm: 'low', agreed: 40, repeated: 44, },),
              figureOf({ arm: 'high', agreed: 44, repeated: 44, },),
              figureOf({ arm: 'medium-b', agreed: 44, repeated: 44, },),
            ];
            const [shortfall,] = repeatShortfallsOf({ figure, others, },);
            expect(shortfall?.other.arm,).toBe('high',);
            expect(shortfall?.p,).toBeLessThan(0.000001,);
            expect(shortfall?.comparisons,).toBe(3,);
          },
        },),

        it({
          name: 'NAMES THE ARM THAT COMES FIRST AMONG OTHER ARMS OF ONE SHARE, whatever order they are given in, and sets '
            + 'a share of 3 of 3 level with 6 of 6',
          fn: async () => {
            const figure = figureOf({ arm: 'medium', agreed: 0, repeated: 6, },);
            const others = [
              figureOf({ arm: 'high', agreed: 3, repeated: 3, },),
              figureOf({ arm: 'low', agreed: 6, repeated: 6, },),
            ];
            expect(repeatShortfallsOf({ figure, others: others.toReversed(), },)[0]?.other.arm,).toBe('low',);
            expect(repeatShortfallsOf({ figure, others, },)[0]?.other.arm,).toBe('high',);
          },
        },),

        it({
          name: 'DIVIDES THE LEVEL BY THE OTHER ARMS: 15 of 20 against two arms at 20 of 20 and 19 of 20 is at 0.025 and is '
            + 'named, and against three arms is over 0.0167 and is not',
          fn: async () => {
            const figure = figureOf({ arm: 'medium', agreed: 15, repeated: 20, },);
            const high = figureOf({ arm: 'high', agreed: 20, repeated: 20, },);
            const low = figureOf({ arm: 'low', agreed: 19, repeated: 20, },);
            expect(repeatShortfallsOf({ figure, others: [high, low,], },)[0]?.other.arm,).toBe('high',);
            expect(repeatShortfallsOf({ figure, others: [high, low, figureOf({ arm: 'medium-b', agreed: 19, repeated: 20, },),], },),).toStrictEqual([],);
            expect(repeatShortfallsOf({ figure, others: [high,], },)[0]?.comparisons,).toBe(1,);
          },
        },),

        it({
          name: 'SAYS NOTHING WHEN THE DEFAULT EFFORT IS NOT UNDER ANY OTHER ARM AT THE LEVEL, and when there is no other arm',
          fn: async () => {
            const figure = figureOf({ arm: 'medium', agreed: 9, repeated: 12, },);
            expect(repeatShortfallsOf({ figure, others: [figureOf({ arm: 'high', agreed: 12, repeated: 12, },),], },),).toStrictEqual([],);
            expect(repeatShortfallsOf({ figure, others: [], },),).toStrictEqual([],);
          },
        },),
      ],
    },),
  ],
},);
