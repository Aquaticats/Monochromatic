//region Census spread
// Percentile reporting shared by the corpus censuses, split out because
// `slice-census.ts` sits at its line budget and a distribution reader is not
// part of what a census MEANS.

/**
 Middle of a sample.
 */
const HALF_PERCENTILE = 50;

/**
 Where a tail starts being worth naming.
 */
const NINETIETH_PERCENTILE = 90;

/**
 Far tail, which is where a per-call deadline is met.
 */
const NINETY_NINTH_PERCENTILE = 99;

/**
 Percentiles reported for every distribution here.
 */
export const REPORTED_PERCENTILES: readonly number[] = [
  HALF_PERCENTILE,
  NINETIETH_PERCENTILE,
  NINETY_NINTH_PERCENTILE,
];

/**
 Highest percentile denominator, so a rank lands inside the sample.
 */
const PERCENT_WHOLE = 100;

/**
 Reads one percentile from a sorted sample.

 @param sorted - values in ascending order

 @param percentile - percentile to read

 @returns Value at that rank, zero for an empty sample

 @throws RangeError where the percentile is below zero or not a number, which
 names no rank

 @example
 ```ts
 const p90 = percentileOf({ sorted, percentile: 90, },);
 ```
 */
export function percentileOf(
  {
    sorted,
    percentile,
  }: {
    readonly sorted: readonly number[];
    readonly percentile: number;
  },
): number {
  // A comparison with NaN is false either way, so this refuses it too.
  if (!(percentile >= 0))
    throw new RangeError(
      `percentile ${String(percentile,)} is not a number from zero up, `
        + 'so it names no rank in the sample',
    );
  if (sorted.length === 0)
    return 0;

  /**
   Rank of that percentile inside the sample.
   */
  const rank = Math.min(
    sorted.length - 1,
    Math.floor((percentile / PERCENT_WHOLE) * sorted.length,),
  );
  /**
   Value at that rank, which exists since the percentile is at least zero and
   the rank is clamped to the last index of a sample that holds one.
   */
  const value = sorted[rank];
  if (value === undefined)
    throw new Error(
      'unreachable: the rank of a percentile of zero or more, clamped to the last index of a non-empty '
        + 'sample, names no value',
    );
  return value;
}

/**
 Renders one distribution as a line.

 @param label - what the numbers describe

 @param values - sample

 @returns Line naming count, percentiles and maximum

 @example
 ```ts
 const line = describeSpread({ label: 'slice source chars', values, },);
 ```
 */
export function describeSpread(
  {
    label,
    values,
  }: {
    readonly label: string;
    readonly values: readonly number[];
  },
): string {
  /**
   Sample in ascending order.
   */
  const sorted = values.toSorted(function ascending(
    left,
    right,
  ) {
    return left - right;
  },);

  /**
   Percentile readings in the reported order.
   */
  const readings = REPORTED_PERCENTILES.map(function toReading(percentile,): string {
    return `p${String(percentile,)} ${
      String(percentileOf({
        sorted,
        percentile,
      },),)
    }`;
  },);
  return `${label}: n ${String(sorted.length,)}, ${readings.join(', ',)}, max ${
    String(sorted.at(-1,) ?? 0,)
  }`;
}

//endregion Census spread
