/**
 Randomness primitives.

 Every draw comes from `crypto.getRandomValues` or `crypto.randomUUID`, both
 CSPRNG-backed in workerd. The Caddy original drew through Go's `math/rand`,
 which is not cryptographically secure, so this is a deliberate strengthening
 rather than a behaviour port; the observable charset and lengths are
 unchanged.

 Both draws use rejection sampling, because reducing a uniform source modulo a
 size that does not divide it over-represents the low values.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

//region Constants

/**
 Characters {@link randomAlphanumeric} draws from.

 Matches the 62 characters the Caddy original's `randAlphaNum` produced,
 measured against the live origin: all 62 of `0-9A-Za-z` appeared across 25
 requests of 64 characters.
 */
export const ALPHANUMERIC_CHARSET = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz';

/**
 Length the root path serves, and the longest length a `/N` route accepts.

 One constant because the Caddy original both capped its literal branches at
 64 and hard-coded 64 for the root route, so the two coincide.
 */
export const MAX_RANDOM_LENGTH = 64;

/**
 Largest byte value that reduces modulo the charset size without bias.

 248 is 62 times 4, the greatest multiple of the charset size that fits in a
 byte, so bytes from 248 upward are discarded and redrawn rather than mapped
 onto the first eight characters. Exported so the invariant is testable
 directly, since detecting the bias statistically would need more samples than
 a unit test should draw.
 */
export const UNBIASED_BYTE_LIMIT = 248;

/**
 Bits in one `crypto.getRandomValues` word.
 */
const WORD_BITS = 32;

/**
 Largest exponent whose power of two stays exactly representable as a double.
 */
const EXACT_POWER_LIMIT = 53;

//endregion Constants

//region Helpers

/**
 Bit count needed to represent `value`, and 0 for a value of 0.

 Computed by halving rather than through `Math.log2`, whose floating-point
 result can land on the wrong side of an exact power of two and so draw one
 bit too many or too few. Exported so the exact power-of-two boundaries are
 testable directly.

 @param value - non-negative integer to measure

 @returns count of significant bits

 @example
 ```ts
 bitLength(0); // 0
 bitLength(1); // 1
 bitLength(7); // 3
 bitLength(8); // 4
 ```
 */
export function bitLength(value: number,): number {
  /**
   Significant bits counted so far.
   */
  let bits = 0;
  /**
   Remaining value, halved each pass.
   */
  let remaining = value;
  while (remaining > 0) {
    bits += 1;
    remaining = Math.floor(remaining / 2,);
  }
  return bits;
}

/**
 Draw a uniformly distributed integer in the half-open range starting at 0 and
 ending at two raised to `bits`.

 Built from masked 32-bit words so the result stays exactly representable: the
 high word carries at most 21 bits when `bits` is at most
 {@link EXACT_POWER_LIMIT}, so both the shifted high word and the sum stay
 below two raised to 53.

 @param bits - width of the draw, from 1 to {@link EXACT_POWER_LIMIT}

 @returns drawn value, always below two raised to `bits`

 @throws Error when `bits` is outside the representable width this Worker draws

 @example
 ```ts
 drawBits(3); // an integer from 0 to 7
 ```
 */
export function drawBits(bits: number,): number {
  if ((bits < 1) || (bits > EXACT_POWER_LIMIT)) {
    throw new Error(`bit width must be between 1 and ${String(EXACT_POWER_LIMIT)}, got ${String(bits)}`,);
  }
  /**
   Random words this draw consumes; the second stays zero when one suffices.
   */
  const words = new Uint32Array(bits <= WORD_BITS ? 1 : 2,);
  crypto.getRandomValues(words,);
  /**
   Low word, used whole.
   */
  const low = nonNullishOrThrow(words[0],);
  if (bits <= WORD_BITS) {
    return low & ((2 ** bits) - 1);
  }
  /**
   Bits the high word contributes above the low word.
   */
  const highBits = bits - WORD_BITS;
  /**
   High word, masked to the bits the draw needs.
   */
  const high = nonNullishOrThrow(words[1],) & ((2 ** highBits) - 1);
  return (high * (2 ** WORD_BITS)) + low;
}

//endregion Helpers

//region Draws

/**
 Parameters for {@link randomAlphanumeric}.
 */
export type RandomAlphanumericParams = {
  /**
   How many characters to draw; the caller validates the bound.
   */
  readonly length: number;
};

/**
 Draw a string of characters from {@link ALPHANUMERIC_CHARSET}.

 @param length - how many characters to draw; the caller validates the bound

 @returns random string of exactly `length` characters

 @example
 ```ts
 randomAlphanumeric({ length: 4 }); // 'aZ3Q', for example
 ```
 */
export function randomAlphanumeric({ length, }: RandomAlphanumericParams,): string {
  /**
   Characters accepted so far.
   */
  const characters: string[] = [];
  while (characters.length < length) {
    /**
     Fresh bytes to reduce, oversized because roughly one byte in 32 is
     discarded as biased and a single pass should still usually suffice.
     */
    const bytes = new Uint8Array((length - characters.length) + 16,);
    crypto.getRandomValues(bytes,);
    for (const byte of bytes) {
      if (byte >= UNBIASED_BYTE_LIMIT) {
        continue;
      }
      /**
       Character this byte selects, guaranteed present because the byte is
       below the greatest multiple of the charset size.
       */
      const character = nonNullishOrThrow(ALPHANUMERIC_CHARSET[byte % ALPHANUMERIC_CHARSET.length],);
      characters.push(character,);
      if (characters.length === length) {
        break;
      }
    }
  }
  return characters.join('',);
}

/**
 Parameters for {@link randomIntInclusive}.
 */
export type RandomIntInclusiveParams = {
  /**
   Lower bound, included in the result.
   */
  readonly min: number;
  /**
   Upper bound, included in the result.
   */
  readonly max: number;
};

/**
 Draw an integer uniformly from the closed interval between `min` and `max`.

 Inclusive at both ends, which is the contract the owner asked for and the
 reason the Caddy original needed its `(int (add max 1))` wrapper around
 `randInt`, whose own upper bound is exclusive.

 @param min - lower bound, included; the caller validates the domain

 @param max - upper bound, included; the caller validates the domain and that
   the span is representable

 @returns drawn integer, between `min` and `max` inclusive

 @example
 ```ts
 randomIntInclusive({ min: 1, max: 6 }); // a die roll, 6 included
 randomIntInclusive({ min: 5, max: 5 }); // always 5
 ```
 */
export function randomIntInclusive({
  min,
  max,
}: RandomIntInclusiveParams,): number {
  /**
   Count of integers in the closed interval.
   */
  const span = (max - min) + 1;
  if (span === 1) {
    return min;
  }
  /**
   Draw width: the fewest bits whose range covers the span, so at most one
   drawn value in two is ever rejected.
   */
  const bits = bitLength(span - 1,);
  for (;;) {
    /**
     Candidate offset from `min`.
     */
    const offset = drawBits(bits,);
    if (offset < span) {
      return min + offset;
    }
  }
}

/**
 Draw a version 4 UUID.

 @returns lowercase canonical 36-character UUID

 @example
 ```ts
 randomUuidV4(); // '3f2504e0-4f89-41d3-9a0c-0305e82c3301', for example
 ```
 */
export function randomUuidV4(): string {
  return crypto.randomUUID();
}

//endregion Draws
