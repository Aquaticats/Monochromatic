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

/**
 Draws taken where a case needs every outcome of a small domain to appear.
 */
const MANY_DRAWS = 400;

/**
 Whether `text` is a lowercase canonical version 4 UUID.

 Checked by index scan rather than pattern match: the shape is fully
 expressible as character positions, and the repository bans regex literals
 without a scoped justification.

 @param text - value to classify

 @returns whether every position matches the version 4 UUID layout
 */
function isUuidV4(text: string,): boolean {
  if (text.length !== 36) {
    return false;
  }
  /**
   Characters a hexadecimal position may hold.
   */
  const hex = '0123456789abcdef';
  for (let index = 0; index < 36; index += 1) {
    /**
     Character at this position.
     */
    const character = text.charAt(index,);
    if ((index === 8) || (index === 13) || (index === 18) || (index === 23)) {
      if (character !== '-') {
        return false;
      }
      continue;
    }
    if (!hex.includes(character,)) {
      return false;
    }
  }
  // Position 14 is the version nibble and position 19 the variant nibble.
  return (text.charAt(14,) === '4') && '89ab'.includes(text.charAt(19,),);
}

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
          name: 'stays inside the requested width at one word, two words and the exact-integer limit',
          fn: async () => {
            for (let attempt = 0; attempt < 200; attempt += 1) {
              expect(drawBits(1,) < 2,).toBe(true,);
              expect(drawBits(32,) < (2 ** 32),).toBe(true,);
              expect(drawBits(33,) < (2 ** 33),).toBe(true,);
              /**
               Widest draw this Worker supports, which must stay exactly
               representable so the modulo-free comparison is sound.
               */
              const wide = drawBits(53,);
              expect(wide < (2 ** 53),).toBe(true,);
              expect(Number.isSafeInteger(wide,),).toBe(true,);
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
               Every character seen across many draws.
               */
            const seen = new Set<string>();
            for (let attempt = 0; attempt < 60; attempt += 1) {
              for (const character of randomAlphanumeric({ length: MAX_RANDOM_LENGTH, },)) {
                seen.add(character,);
              }
            }
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
            for (let attempt = 0; attempt < 20; attempt += 1) {
              expect(randomIntInclusive({ min: 5, max: 5, },),).toBe(5,);
              expect(randomIntInclusive({ min: 0, max: 0, },),).toBe(0,);
              expect(randomIntInclusive({
                min: -7,
                max: -7,
              },),).toBe(-7,);
            }
          },
        },),
        it({
          name: 'includes the upper bound, the behaviour the Caddy origin lacked',
          fn: async () => {
            /**
               Every value seen for the interval 1 to 3.
               */
            const seen = new Set<number>();
            for (let attempt = 0; attempt < MANY_DRAWS; attempt += 1) {
              seen.add(randomIntInclusive({ min: 1, max: 3, },),);
            }
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
               Values seen for 1 to 2.
               */
            const coin = new Set<number>();
            /**
             Values seen for -3 to -1.
             */
            const negative = new Set<number>();
            for (let attempt = 0; attempt < MANY_DRAWS; attempt += 1) {
              coin.add(randomIntInclusive({ min: 1, max: 2, },),);
              /**
               One draw from the negative interval.
               */
              const drawn = randomIntInclusive({ min: -3, max: -1, },);
              expect(drawn >= (-3),).toBe(true,);
              expect(drawn <= (-1),).toBe(true,);
              negative.add(drawn,);
            }
            expect(coin.size,).toBe(2,);
            expect(negative.size,).toBe(3,);
          },
        },),
        it({
          name: 'stays in bounds across a span that needs the full 53-bit draw',
          fn: async () => {
            /**
             Lower bound of the widest interval the validator accepts.
             */
            const min = -Number.MAX_SAFE_INTEGER;
            for (let attempt = 0; attempt < 200; attempt += 1) {
              /**
               One draw across a span of exactly 2^53 values.
               */
              const drawn = randomIntInclusive({ min, max: 0, },);
              expect(Number.isSafeInteger(drawn,),).toBe(true,);
              expect(drawn >= min,).toBe(true,);
              expect(drawn <= 0,).toBe(true,);
            }
          },
        },),
        it({
          name: 'reaches both ends of a power-of-two span, which never rejects',
          fn: async () => {
            /**
             Values seen for 0 to 7, a span of exactly 8.
             */
            const seen = new Set<number>();
            for (let attempt = 0; attempt < MANY_DRAWS; attempt += 1) {
              seen.add(randomIntInclusive({ min: 0, max: 7, },),);
            }
            expect(seen.size,).toBe(8,);
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
            for (let attempt = 0; attempt < 20; attempt += 1) {
              /**
               One drawn UUID.
               */
              const uuid = randomUuidV4();
              expect(uuid,).toHaveLength(36,);
              expect(isUuidV4(uuid,),).toBe(true,);
            }
          },
        },),
      ],
    },),
  ],
},);
