/**
 Integer and float value arbitraries for the fuzz generators.
 
 Integers cover decimal (signed, underscored), hexadecimal, octal, and binary
 spellings. Floats cover fractional, exponent, and the special `inf`/`nan`
 forms with both signs. Predicted values are computed with `Number(text)` (for
 the canonical decimal and exponent forms) or stated directly (for base and
 special forms), so they always match the IEEE-754 / two's-complement value
 the parser yields within the safe-integer range these arbitraries stay in.
 
 @module
 */

import {
  type Arbitrary,
  constantFrom,
  integer,
  nat,
  oneof,
  tuple,
} from 'fast-check';

import type { ValueSample, } from './arb-types.ts';

/**
 Exponent for the safe magnitude bound, kept well under 53 so every generated
 integer is exactly representable as a `Number`.
 */
const SAFE_MAGNITUDE_EXPONENT = 40;

/**
 Magnitude bound keeping generated integers inside `Number`'s exact range so
 the predicted value never disagrees with the parser over a rounding step.
 */
const SAFE_MAGNITUDE = 2 ** SAFE_MAGNITUDE_EXPONENT;

/**
 Decimal radix for `Number.prototype.toString`.
 */
const DECIMAL_RADIX = 10;

/**
 Digits per underscore group in a grouped decimal integer.
 */
const UNDERSCORE_GROUP_SIZE = 3;

/**
 A non-decimal integer spelling: its radix and TOML prefix.
 */
type RadixSpec = {
  readonly radix: number;
  readonly prefix: string;
};

/**
 Hexadecimal, octal, and binary spellings (no sign, per TOML).
 */
const RADIX_SPECS: readonly RadixSpec[] = [
  {
    radix: 16,
    prefix: '0x',
  },
  {
    radix: 8,
    prefix: '0o',
  },
  {
    radix: 2,
    prefix: '0b',
  },
];

/**
 Insert grouping underscores between every run of digits in `digits`.
 
 @returns Digit string with single interior underscores, valid TOML spacing.
 */
function underscoreGroups({ digits, }: { readonly digits: string; },): string {
  // `Array.from` with a mapper iterates code points without the string-spread
  // the linter rejects; the digits are ASCII, so either would do, but this
  // keeps the rule satisfied uniformly.
  return Array.from(
    digits,
    function each(
      digit,
      index,
    ) {
    /**
     Position from the right so groups of three align to the low digits.
     */
      const fromRight = digits.length - index;
    /**
     Whether a grouping underscore precedes this digit.
     */
      const needsUnderscore = (index > 0) && ((fromRight % UNDERSCORE_GROUP_SIZE) === 0);
      return needsUnderscore ? `_${digit}` : digit;
    },
  )
    .join('',);
}

/**
 Decimal integers, half of them grouped with underscores.
 */
const decimalIntegerArbitrary: Arbitrary<ValueSample> = tuple(
  integer({
    min: -SAFE_MAGNITUDE,
    max: SAFE_MAGNITUDE,
  },),
  constantFrom(
    true,
    false,
  ),
)
  .map(function build([value, grouped,],) {
  /**
   Sign prefix kept separate so underscores only land between magnitude digits.
   */
    const sign = value < 0 ? '-' : '';
  /**
   Unsigned magnitude digits before optional grouping.
   */
    const magnitude = Math.abs(value,)
      .toString(DECIMAL_RADIX,);
  /**
   Magnitude with grouping underscores applied when selected.
   */
    const body = grouped ? underscoreGroups({ digits: magnitude, },) : magnitude;
    return {
      text: `${sign}${body}`,
      value,
    };
  },);

/**
 Non-negative integers rendered in hex, octal, or binary (no sign, per spec).
 */
const radixIntegerArbitrary: Arbitrary<ValueSample> = tuple(
  nat({ max: SAFE_MAGNITUDE, },),
  constantFrom(...RADIX_SPECS,),
)
  .map(function build([value, {
    radix,
    prefix,
  },],) {
    return {
      text: `${prefix}${value.toString(radix,)}`,
      value,
    };
  },);

/**
 Deterministic integer examples spanning every spelling family.
 */
export const INTEGER_EXAMPLES: readonly ValueSample[] = [
  {
    text: '0',
    value: 0,
  },
  {
    text: '-42',
    value: -42,
  },
  {
    text: '1_000',
    value: 1_000,
  },
  {
    text: '0xDEAD_BEEF',
    value: 0xDE_AD_BE_EF,
  },
  {
    text: '0o755',
    value: 0o755,
  },
  {
    text: '0b1010',
    value: 0b1010,
  },
];

/**
 Integer value arbitrary across all spellings.
 */
export const integerSampleArbitrary: Arbitrary<ValueSample> = oneof(
  decimalIntegerArbitrary,
  radixIntegerArbitrary,
  constantFrom(...INTEGER_EXAMPLES,),
);

/**
 Finite fractional and exponent floats whose value is `Number(text)`.
 */
const finiteFloatArbitrary: Arbitrary<ValueSample> = tuple(
  integer({
    min: -9_999,
    max: 9_999,
  },),
  nat({ max: 999_999, },),
  constantFrom(
    '',
    'e3',
    'e-3',
    'E+2',
  ),
)
  .map(function build([intPart, frac, exponent,],) {
  /**
   Sign and magnitude split so the fractional dot always sits after a digit.
   */
    const sign = intPart < 0 ? '-' : '';
  /**
   Unsigned integer-part digits with no leading zeros.
   */
    const magnitude = Math.abs(intPart,)
      .toString(DECIMAL_RADIX,);
  /**
   Assembled float spelling, sign through optional exponent.
   */
    const text = `${sign}${magnitude}.${frac.toString(DECIMAL_RADIX,)}${exponent}`;
    return {
      text,
      value: Number(text,),
    };
  },);

/**
 Special float spellings: signed infinities and the three `nan` forms.
 */
const SPECIAL_FLOATS: readonly ValueSample[] = [
  {
    text: 'inf',
    value: Infinity,
  },
  {
    text: '+inf',
    value: Infinity,
  },
  {
    text: '-inf',
    value: -Infinity,
  },
  {
    text: 'nan',
    value: Number.NaN,
  },
  {
    text: '+nan',
    value: Number.NaN,
  },
  {
    text: '-nan',
    value: Number.NaN,
  },
];

/**
 Special float value arbitrary.
 */
const specialFloatArbitrary: Arbitrary<ValueSample> = constantFrom(...SPECIAL_FLOATS,);

/**
 Deterministic float examples spanning fractional, exponent, and special forms.
 */
export const FLOAT_EXAMPLES: readonly ValueSample[] = [
  {
    text: '3.14',
    value: 3.14,
  },
  {
    text: '-0.0',
    value: -0,
  },
  {
    text: '1e10',
    value: 1e10,
  },
  {
    text: '6.022e23',
    value: 6.022e23,
  },
  {
    text: 'inf',
    value: Infinity,
  },
  {
    text: '-inf',
    value: -Infinity,
  },
  {
    text: 'nan',
    value: Number.NaN,
  },
];

/**
 Float value arbitrary across fractional, exponent, and special spellings.
 */
export const floatSampleArbitrary: Arbitrary<ValueSample> = oneof(
  finiteFloatArbitrary,
  specialFloatArbitrary,
  constantFrom(...FLOAT_EXAMPLES,),
);
