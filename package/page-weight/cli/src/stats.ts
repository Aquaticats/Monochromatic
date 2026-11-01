/**
 Distribution summary for a finite sample of numbers.
 
 Reports the four stats the user cares about: min, max, mean, median.
 Standard deviation is intentionally omitted; it adds noise to a short
 CLI report and rarely drives action.
 */
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

/**
 Aggregate description of a numeric sample.
 */
export type Stats = {
  /**
   Number of observations.
   */
  count: number;
  /**
   Smallest observation.
   */
  min: number;
  /**
   Largest observation.
   */
  max: number;
  /**
   Arithmetic mean (sum / count).
   */
  mean: number;
  /**
   Median; for even samples, the average of the two middle values.
   */
  median: number;
  /**
   Sum of all observations.
   */
  sum: number;
};

/**
 Computes the median of an already-sorted-ascending array of numbers.
 
 @param sorted - ascending-sorted observations (non-empty)
 
 @returns middle value for odd-length input, average of the two middle values for even-length
 
 @example
 ```ts
 medianOfSorted([1, 2, 3]); // 2
 medianOfSorted([1, 2, 3, 4]); // 2.5
 ```
 */
function medianOfSorted(sorted: readonly number[],): number {
  if (sorted.length
    === 0)
    throw new Error('cannot compute median of empty sample',);
  /**
   Index of the upper-middle element; for odd lengths this is the median position.
   */
  const mid = Math.floor(sorted.length
    / 2,);
  if ((sorted.length
    % 2) === 1)
    return nonNullishOrThrow(sorted[mid],);
  /**
   Lower of the two central values for even-length samples.
   */
  const low = nonNullishOrThrow(sorted[mid - 1],);
  /**
   Upper of the two central values for even-length samples.
   */
  const high = nonNullishOrThrow(sorted[mid],);
  return (low + high) / 2;
}

/**
 Computes min, max, mean, median, sum for a sample.
 
 Throws when given an empty sample; an empty distribution has no
 meaningful summary, and silently returning zeros would mask bugs
 in the caller's data pipeline.
 
 @param sample - observations; must contain at least one value
 
 @returns aggregate description
 
 @throws when `sample.length` is zero
 
 @example
 ```ts
 summarize([1, 2, 3, 4]);
 // { count: 4, min: 1, max: 4, mean: 2.5, median: 2.5, sum: 10 }
 ```
 */
export function summarize(sample: readonly number[],): Stats {
  if (sample.length
    === 0)
    throw new Error('cannot summarize an empty sample',);
  /**
   Ascending-sorted copy; needed for median computation and for `min`/`max` lookup.
   */
  const sorted = sample.toSorted(function ascending(
    a,
    b,
  ) {
    return a - b;
  },);
  /**
   Sum of every observation; reused for both the `sum` output and the `mean` computation.
   */
  const sum = sorted.reduce(
    function add(
      acc,
      value,
    ) {
      return acc + value;
    },
    0,
  );
  return {
    count: sorted.length,
    min: nonNullishOrThrow(sorted[0],),
    max: nonNullishOrThrow(sorted.at(-1,),),
    mean: sum / sorted
      .length,
    median: medianOfSorted(sorted,),
    sum,
  };
}
