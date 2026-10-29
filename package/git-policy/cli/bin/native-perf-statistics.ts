/**
 Order statistics of the latency samples that `native-perf.ts` reports for each scenario.

 `SPEC.md` ("Benchmark method") asks for the median, the p95 and the median absolute deviation of every sample set.
 */
// The bin scripts sit outside the package tsconfig's include list, so type-aware lint checks them in an inferred
// program that loads no ambient Node types; this reference loads them, so `node:` imports stay typed in this file on
// its own instead of depending on a sibling module that carries the same reference.
// That inferred program also loads only the default ECMAScript library, which lacks `Array#toSorted`, so the second
// reference loads the library that declares it.
/// <reference types="node" />
/// <reference lib="es2023" />

/** The fraction of samples at or below the reported high percentile. */
const HIGH_PERCENTILE = 0.95;

/**
 Copy a sample set in ascending order, so the caller's samples keep the order they were measured in.

 @param {readonly number[]} samples - measured milliseconds
 @returns {readonly number[]} the same samples, ascending
 */
function ascending(samples) {
  return samples.toSorted(function ascendingOrder(
    left,
    right,
  ) {
    return left - right;
  });
}

/**
 Middle sample, or the mean of the two middle samples of an even-sized set.

 @param {readonly number[]} samples - measured milliseconds, at least one
 @returns {number} the median in milliseconds
 */
export function median(samples) {
  const sorted = ascending(samples);
  const middle = Math.floor(sorted.length / 2);
  return (sorted.length % 2) === 0 ? (sorted[middle - 1] + sorted[middle]) / 2 : sorted[middle];
}

/**
 Nearest-rank 95th percentile.

 @param {readonly number[]} samples - measured milliseconds, at least one
 @returns {number} the p95 in milliseconds
 */
export function percentile95(samples) {
  const sorted = ascending(samples);
  return sorted[Math.min(
    sorted.length - 1,
    Math.ceil(HIGH_PERCENTILE * sorted.length) - 1,
  )];
}

/**
 Median of each sample's distance from the median, a spread that one slow outlier cannot inflate.

 @param {readonly number[]} samples - measured milliseconds, at least one
 @returns {number} the median absolute deviation in milliseconds
 */
export function medianAbsoluteDeviation(samples) {
  const centre = median(samples);
  return median(samples.map(function distance(value) {
    return Math.abs(value - centre);
  }));
}

/**
 Median, p95 and median absolute deviation of one sample set, the shape the report stores.

 @param {readonly number[]} samples - measured milliseconds, at least one
 @returns {{ median: number, p95: number, medianAbsoluteDeviation: number }} the three statistics
 */
export function summary(samples) {
  return {
    median: median(samples),
    p95: percentile95(samples),
    medianAbsoluteDeviation: medianAbsoluteDeviation(samples),
  };
}
