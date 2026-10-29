/**
 Order statistics of the latency samples that `native-perf.ts` reports for each scenario.

 `SPEC.md` ("Benchmark method") asks for the median, the p95 and the median absolute deviation of every sample set.
 */

/**
 The fraction of samples at or below the reported high percentile.
 */
const HIGH_PERCENTILE = 0.95;

/**
 Sample at one index of an ascending set, which the median and percentile arithmetic keeps in range for a non-empty set.

 @param sorted - ascending samples

 @param index - position to read

 @returns the sample, so an empty set fails loudly instead of yielding `undefined`

 @throws RangeError when the index is outside the set
 */
function sampleAt({
  sorted,
  index,
}: {
  readonly sorted: readonly number[];
  readonly index: number;
}): number {
  /**
   Sample at the index, absent only when the index is outside the set.
   */
  const sample = sorted[index];
  if (sample === undefined)
    throw new RangeError(`Sample index ${String(index)} is outside a set of ${String(sorted.length)} samples.`);
  return sample;
}

/**
 Copy a sample set in ascending order, so the caller's samples keep the order they were measured in.

 @param samples - measured milliseconds

 @returns the same samples, ascending
 */
function ascending(samples: readonly number[]): readonly number[] {
  return samples.toSorted(function ascendingOrder(
    left,
    right,
  ) {
    return left - right;
  });
}

/**
 Middle sample, or the mean of the two middle samples of an even-sized set.

 @param samples - measured milliseconds, at least one

 @returns the median in milliseconds

 @example
 ```ts
 median([3, 1, 2]);
 ```
 */
export function median(samples: readonly number[]): number {
  /**
   The samples in ascending order, which the middle position is read from.
   */
  const sorted = ascending(samples);
  /**
   Index of the upper middle sample.
   */
  const middle = Math.floor(sorted.length / 2);
  if ((sorted.length % 2) === 0)
    return (sampleAt({
      sorted,
      index: middle - 1,
    }) + sampleAt({
      sorted,
      index: middle,
    })) / 2;
  return sampleAt({
    sorted,
    index: middle,
  });
}

/**
 Nearest-rank 95th percentile.

 @param samples - measured milliseconds, at least one

 @returns the p95 in milliseconds

 @example
 ```ts
 percentile95([1, 2, 3, 4, 5]);
 ```
 */
export function percentile95(samples: readonly number[]): number {
  /**
   The samples in ascending order, which the rank is read from.
   */
  const sorted = ascending(samples);
  return sampleAt({
    sorted,
    index: Math.min(
      sorted.length - 1,
      Math.ceil(HIGH_PERCENTILE * sorted.length) - 1,
    ),
  });
}

/**
 Median of each sample's distance from the median, a spread that one slow outlier cannot inflate.

 @param samples - measured milliseconds, at least one

 @returns the median absolute deviation in milliseconds

 @example
 ```ts
 medianAbsoluteDeviation([1, 2, 3, 100]);
 ```
 */
export function medianAbsoluteDeviation(samples: readonly number[]): number {
  /**
   The median every sample's distance is measured from.
   */
  const centre = median(samples);
  return median(samples.map(function distance(value) {
    return Math.abs(value - centre);
  }));
}

/**
 Median, p95 and median absolute deviation of one sample set, the shape the report stores.

 @param samples - measured milliseconds, at least one

 @returns the three statistics

 @example
 ```ts
 summary([1, 2, 3]);
 ```
 */
export function summary(samples: readonly number[]): {
  readonly median: number;
  readonly p95: number;
  readonly medianAbsoluteDeviation: number;
} {
  return {
    median: median(samples),
    p95: percentile95(samples),
    medianAbsoluteDeviation: medianAbsoluteDeviation(samples),
  };
}
