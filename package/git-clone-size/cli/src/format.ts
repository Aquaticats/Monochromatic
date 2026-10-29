import { formatBytes, } from '@monochromatic-dev/module-numeric-format/ts';

import type {
  EstimateRange,
  RatioRange,
  Size,
} from './types.ts';

/**
 Percent scale factor for savings (ratio in [0, 1] -> percent in [0, 100]).
 */
const PERCENT = 100;

/**
 Decimal places retained for the unitless ratio.
 */
const RATIO_DECIMALS = 4;

/**
 Decimal places retained for the savings percentage.
 */
const SAVINGS_DECIMALS = 1;

/**
 Powers of ten for {@link roundTo}, indexed by decimal count, composed without
 magic literals beyond the exempt -2..2 range.
 */
const POW10: Readonly<Record<number, number>> = {
  1: 10,
  4: 10 * 10
    * 10
    * 10,
};

/**
 Rounds a number to a fixed number of decimal places.
 
 @param value - number to round
 
 @param decimals - decimal places to keep; must be a key of {@link POW10}
 
 @returns value rounded to `decimals` places
 
 @example
 ```ts
 roundTo({ value: 0.04421, decimals: 4 }); // 0.0442
 ```
 */
export function roundTo({
  value,
  decimals,
}: {
  readonly value: number;
  readonly decimals: number
},): number {
  /**
   Scale factor for the requested precision.
   */
  const scale = POW10[decimals] ?? 1;
  return Math.round(value * scale,) / scale;
}

/**
 Clamps a number into an inclusive range.
 
 @param value - number to clamp
 
 @param min - lower bound
 
 @param max - upper bound
 
 @returns value confined to `[min, max]`
 
 @example
 ```ts
 clamp({ value: 1.3, min: 0, max: 1 }); // 1
 ```
 */
export function clamp({
  value,
  min,
  max,
}: {
  readonly value: number;
  readonly min: number;
  readonly max: number
},): number {
  return Math.min(
    max,
    Math.max(
      min,
      value,
    ),
  );
}

/**
 Builds a {@link Size} from a raw byte count, attaching the human string via
 the shared IEC formatter. Negative or fractional inputs are floored to a
 non-negative integer first.
 
 @param bytes - raw byte count
 
 @returns size with `bytes` and `human` fields
 
 @example
 ```ts
 toSize(422); // { bytes: 422, human: '422 B' }
 ```
 */
export function toSize(bytes: number,): Size {
  /**
   Non-negative integer byte count for display and storage.
   */
  const safe = Math.max(
    0,
    Math.round(bytes,),
  );
  return {
    bytes: safe,
    human: formatBytes(safe,),
  };
}

/**
 Computes the shallow/full ratio range. The denominator's high end yields the
 ratio's low end and vice versa, since a larger full size means a smaller
 fraction. Ratio is clamped to `[0, 1]`.
 
 @param shallowBytes - exact shallow object-store bytes
 
 @param full - fused full-size interval
 
 @returns ratio point and bounds, clamped and rounded
 
 @example
 ```ts
 computeRatio({ shallowBytes: 4_404_019, full: { point: {...}, lo: {...}, hi: {...} } });
 ```
 */
export function computeRatio(
  {
    shallowBytes,
    full,
  }: {
    readonly shallowBytes: number;
    readonly full: EstimateRange
  },
): RatioRange {
  /**
   Ratio at a denominator, guarding division by a non-positive full size.
   
   @param denominatorBytes - full-size byte count used as the denominator
   
   @returns shallow/denominator ratio clamped to `[0, 1]`, or 1 when non-positive
   
   @example
   ```ts
   ratioAt(full.point.bytes);
   ```
   */
  function ratioAt(denominatorBytes: number,): number {
    if (denominatorBytes <= 0)
      return 1;
    return clamp({
      value: shallowBytes / denominatorBytes,
      min: 0,
      max: 1,
    },);
  }
  return {
    hi: roundTo({
      value: ratioAt(full.lo
        .bytes,),
      decimals: RATIO_DECIMALS,
    },),
    lo: roundTo({
      value: ratioAt(full.hi
        .bytes,),
      decimals: RATIO_DECIMALS,
    },),
    point: roundTo({
      value: ratioAt(full.point
        .bytes,),
      decimals: RATIO_DECIMALS,
    },),
  };
}

/**
 Computes the savings percentage range from a ratio range: `savings = 1 -
 ratio`, so the ratio's low end maps to the savings high end. Clamped to
 `[0, 100]`.
 
 @param ratio - shallow/full ratio range
 
 @returns savings percentage point and bounds
 
 @example
 ```ts
 computeSavings({ ratio: { point: 0.044, lo: 0.028, hi: 0.07 } });
 ```
 */
export function computeSavings({ ratio, }: { readonly ratio: RatioRange; },): RatioRange {
  /**
   Savings percent at a ratio value.
   
   @param ratioValue - shallow/full ratio in `[0, 1]`
   
   @returns savings percentage clamped to `[0, 100]`
   
   @example
   ```ts
   savingsAt(0.044);
   ```
   */
  function savingsAt(ratioValue: number,): number {
    return clamp({
      value: (1 - ratioValue) * PERCENT,
      min: 0,
      max: PERCENT,
    },);
  }
  return {
    hi: roundTo({
      value: savingsAt(ratio.lo,),
      decimals: SAVINGS_DECIMALS,
    },),
    lo: roundTo({
      value: savingsAt(ratio.hi,),
      decimals: SAVINGS_DECIMALS,
    },),
    point: roundTo({
      value: savingsAt(ratio.point,),
      decimals: SAVINGS_DECIMALS,
    },),
  };
}
