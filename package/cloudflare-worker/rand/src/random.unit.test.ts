/**
 Tests for the randomness primitives: bit counting, raw bit draws, charset
 coverage and the unbiasedness invariant behind it, the inclusive integer
 interval, and UUID shape.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ALPHANUMERIC_CHARSET,
  bitLength,
  drawBits,
  MAX_RANDOM_LENGTH,
  randomAlphanumeric,
  randomIntInclusive,
  randomUuidV4,
  UNBIASED_BYTE_LIMIT,
} from '@monochromatic-dev/cloudflare-worker-rand';

import { isUuidV4, } from './fixture.uuid-v4-shape.ts';

/**
 Draws per case: enough that every outcome of a small domain appears, and that
 a one-sided assertion cannot pass by luck.
 */
const MANY_DRAWS = 400;

/**
 Bit widths the raw draw must honour: one word, the sign-sensitive 32-bit
 width, two words, and the widest exactly representable draw.
 */
const DRAW_WIDTHS = [
  1,
  31,
  32,
  33,
  53,
] as const;

/**
 Upper bound of an interval whose span needs exactly the 32-bit draw.
 */
const THIRTY_TWO_BIT_SPAN_MAX = 2 ** 31;

/**
 Interval offering a coin's two outcomes.
 */
const COIN_INTERVAL = {
  min: 1,
  max: 2,
} as const;

/**
 Interval offering three outcomes, all negative.
 */
const NEGATIVE_INTERVAL = {
  min: -3,
  max: -1,
} as const;

/**
 Upper bound of a span of exactly eight values, where rejection never occurs.
 */
const POWER_OF_TWO_SPAN_MAX = 7;

await describe({
  name: 'random',
  children: [
    describe({
      name: bitLength.name,
      children: [
        it({
          name: 'is 0 for 0 and counts exact power-of-two boundaries without rounding',
          fn: async () => {
            expect(bitLength(0,),).toBe(0,);
            expect(bitLength(1,),).toBe(1,);
            expect(bitLength(2,),).toBe(2,);
            expect(bitLength(3,),).toBe(2,);
            expect(bitLength(4,),).toBe(3,);
            expect(bitLength(7,),).toBe(3,);
            expect(bitLength(8,),).toBe(4,);
            // The boundary a floating-point logarithm would get wrong: 2^53 is
            // exactly representable, so its bit length must be 54.
            expect(bitLength(2 ** 53,),).toBe(54,);
            expect(bitLength(Number.MAX_SAFE_INTEGER,),).toBe(53,);
          },
        },),
      ],
    },),
    describe({
      name: drawBits.name,
      children: [
        it({
          name: 'stays inside the requested width on both bounds at one word, the sign-sensitive 32-bit width, two words and the exact-integer limit',
          fn: async () => {
            /**
             Every width drawn once per attempt, flattened so each result is
             asserted once.
             */
            const draws = Array.from({ length: MANY_DRAWS, }, function drawEveryWidth() {
              return DRAW_WIDTHS.map((bits) => ({
                bits,
                drawn: drawBits(bits,),
              }),);
            },)
              .flat();
            for (const {
              bits,
              drawn,
            } of draws) {
              // The lower bound is the assertion a signed 32-bit reduction
              // fails: a negative draw still satisfies the upper bound.
              expect(drawn >= 0,).toBe(true,);
              expect(drawn < (2 ** bits),).toBe(true,);
              expect(Number.isSafeInteger(drawn,),).toBe(true,);
            }
          },
        },),
        it({
          name: 'refuses a width it cannot represent exactly',
          fn: async () => {
            expect(() => drawBits(0,),).toThrow('bit width',);
            expect(() => drawBits(-1,),).toThrow('bit width',);
            expect(() => drawBits(54,),).toThrow('bit width',);
          },
        },),
      ],
    },),
    describe({
      name: 'ALPHANUMERIC_CHARSET',
      children: [
        it({
          name: 'is the 62 characters measured on the Caddy origin',
          fn: async () => {
            expect(ALPHANUMERIC_CHARSET,).toBe('0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz',);
            expect(ALPHANUMERIC_CHARSET.length,).toBe(62,);
          },
        },),
      ],
    },),
    describe({
      name: 'UNBIASED_BYTE_LIMIT',
      children: [
        it({
          name: 'is the greatest multiple of the charset size that fits in a byte',
          fn: async () => {
            // Deterministic proof of the invariant rejection sampling depends
            // on: every accepted byte maps to a charset index the same number
            // of times, and no larger limit would.
            expect(UNBIASED_BYTE_LIMIT % ALPHANUMERIC_CHARSET.length,).toBe(0,);
            expect(UNBIASED_BYTE_LIMIT <= 256,).toBe(true,);
            expect((UNBIASED_BYTE_LIMIT + ALPHANUMERIC_CHARSET.length) > 256,).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: randomAlphanumeric.name,
      children: [
        it({
          name: 'returns the exact requested length and only charset characters',
          fn: async () => {
            expect(randomAlphanumeric({ length: 0, },),).toBe('',);
            expect(randomAlphanumeric({ length: 1, },),).toHaveLength(1,);
            for (const length of [
              1,
              7,
              32,
              MAX_RANDOM_LENGTH,
            ]) {
              /**
               One drawn string of this length.
               */
              const drawn = randomAlphanumeric({ length, },);
              expect(drawn,).toHaveLength(length,);
              for (const character of drawn) {
                expect(ALPHANUMERIC_CHARSET.includes(character,),).toBe(true,);
              }
            }
          },
        },),
        it({
          name: 'covers all 62 characters, so no index is unreachable',
          fn: async () => {
            /**
             One maximum-length draw per attempt.
             */
            const draws = Array.from({ length: MANY_DRAWS, }, function drawString() {
              return randomAlphanumeric({ length: MAX_RANDOM_LENGTH, },);
            },);
            /**
             Every character seen across those draws.
             */
            const seen = new Set(draws.join('',),);
            expect(seen.size,).toBe(ALPHANUMERIC_CHARSET.length,);
          },
        },),
      ],
    },),
    describe({
      name: randomIntInclusive.name,
      children: [
        it({
          name: 'returns the only value when the interval holds one',
          fn: async () => {
            /**
             Intervals holding one value each, drawn from more than once so the
             constant answer is not one lucky call.
             */
            const degenerate = [
              {
                min: 5,
                max: 5,
              },
              {
                min: 0,
                max: 0,
              },
              {
                min: -7,
                max: -7,
              },
            ] as const;
            for (const interval of degenerate) {
              /**
               Every draw from this interval.
               */
              const draws = Array.from({ length: MANY_DRAWS, }, function drawOnly() {
                return randomIntInclusive(interval,);
              },);
              for (const drawn of draws) {
                expect(drawn,).toBe(interval.min,);
              }
            }
          },
        },),
        it({
          name: 'includes the upper bound, the behaviour the Caddy origin lacked',
          fn: async () => {
            /**
             One draw per attempt for the interval 1 to 3.
             */
            const draws = Array.from({ length: MANY_DRAWS, }, function drawDie() {
              return randomIntInclusive({
                min: 1,
                max: 3,
              },);
            },);
            /**
             Every value seen for that interval.
             */
            const seen = new Set(draws,);
            expect([
              ...seen,
            ].toSorted((a, b) => a - b,),).toEqual([
              1,
              2,
              3,
            ],);
          },
        },),
        it({
          name: 'covers a two-value interval and a negative interval end to end',
          fn: async () => {
            /**
             Every coin outcome seen.
             */
            const coin = Array.from({ length: MANY_DRAWS, }, function flip() {
              return randomIntInclusive(COIN_INTERVAL,);
            },);
            /**
             Every draw taken from the negative interval.
             */
            const negative = Array.from({ length: MANY_DRAWS, }, function drawNegative() {
              return randomIntInclusive(NEGATIVE_INTERVAL,);
            },);
            for (const drawn of negative) {
              expect(drawn >= NEGATIVE_INTERVAL.min,).toBe(true,);
              expect(drawn <= NEGATIVE_INTERVAL.max,).toBe(true,);
            }
            expect(new Set(coin,).size,).toBe((COIN_INTERVAL.max - COIN_INTERVAL.min) + 1,);
            expect(new Set(negative,).size,).toBe((NEGATIVE_INTERVAL.max - NEGATIVE_INTERVAL.min) + 1,);
          },
        },),
        it({
          name: 'stays in bounds across a span that needs the full 53-bit draw',
          fn: async () => {
            /**
             Lower bound of the widest interval the validator accepts.
             */
            const min = -Number.MAX_SAFE_INTEGER;
            /**
             One draw per attempt across a span of exactly 2^53 values.
             */
            const draws = Array.from({ length: MANY_DRAWS, }, function drawWideSpan() {
              return randomIntInclusive({
                min,
                max: 0,
              },);
            },);
            for (const drawn of draws) {
              expect(Number.isSafeInteger(drawn,),).toBe(true,);
              expect(drawn >= min,).toBe(true,);
              expect(drawn <= 0,).toBe(true,);
            }
          },
        },),
        it({
          name: 'stays inside an interval whose span needs the 32-bit draw, where a signed reduction leaves it',
          fn: async () => {
            /**
             Every draw across a span of 2^31 + 1 values, so the draw width is
             exactly 32, the width a signed reduction turns negative.
             */
            const draws = Array.from({ length: MANY_DRAWS, }, function drawInInterval() {
              return randomIntInclusive({
                min: 0,
                max: THIRTY_TWO_BIT_SPAN_MAX,
              },);
            },);
            for (const drawn of draws) {
              expect(drawn >= 0,).toBe(true,);
              expect(drawn <= THIRTY_TWO_BIT_SPAN_MAX,).toBe(true,);
              expect(Number.isSafeInteger(drawn,),).toBe(true,);
            }
          },
        },),
        it({
          name: 'reaches both ends of a power-of-two span, which never rejects',
          fn: async () => {
            /**
             One draw per attempt from a span of exactly eight values.
             */
            const draws = Array.from({ length: MANY_DRAWS, }, function drawOctet() {
              return randomIntInclusive({
                min: 0,
                max: POWER_OF_TWO_SPAN_MAX,
              },);
            },);
            expect(new Set(draws,).size,).toBe(POWER_OF_TWO_SPAN_MAX + 1,);
          },
        },),
      ],
    },),
    describe({
      name: randomUuidV4.name,
      children: [
        it({
          name: 'is a lowercase canonical version 4 UUID',
          fn: async () => {
            /**
             One drawn UUID per attempt.
             */
            const uuids = Array.from({ length: MANY_DRAWS, }, function drawUuid() {
              return randomUuidV4();
            },);
            for (const uuid of uuids) {
              expect(uuid,).toHaveLength(36,);
              expect(isUuidV4(uuid,),).toBe(true,);
            }
          },
        },),
      ],
    },),
  ],
},);
