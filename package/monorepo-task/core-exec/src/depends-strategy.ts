/**
 Time strategy definitions for task-depends staleness detection.
 
 Defines builtin aggregation strategies (newest, oldest, mean, median)
 for reducing multiple timestamps to a single representative value.
 
 @module
 */

//region Types

/**
 Aggregation strategy for reducing multiple timestamps to a single value.
 
 Builtin strategies:
 - `newest`: `Math.max`: most recent timestamp wins
 - `oldest`: `Math.min`: least recent timestamp wins
 - `mean`: arithmetic mean; average timestamp across all items
 - `median`: middle value; central timestamp, robust to outliers
 
 Custom strategies use `sh:` prefix: the command receives all resolved
 timestamps as space-separated arguments and must output a single timestamp.
 
 @example
 ```ts
 const builtin: BuiltinTimeStrategy = 'newest';
 ```
 */
export type BuiltinTimeStrategy = 'newest' | 'oldest' | 'mean' | 'median';

/**
 Time aggregation strategy: a builtin name or a `sh:` command.
 
 @example
 ```ts
 const s: TimeStrategy = 'newest';
 const c: TimeStrategy = 'sh:custom-aggregator';
 ```
 */
export type TimeStrategy = BuiltinTimeStrategy | `sh:${string}`;

//endregion Types

//region Strategy functions

/**
 Sums numeric values in one linear pass.
 
 @param values - Numeric values to total.
 
 @returns arithmetic total.
 
 @example
 ```ts
 sumValues([1, 2, 3]); // 6
 ```
 */
function sumValues(values: readonly number[],): number {
  /**
   Running arithmetic total.
   */
  let sum = 0;
  for (const value of values)
    sum += value;
  return sum;
}

/**
 Computes the arithmetic mean of an array of numbers.
 
 @param values - Non-empty array of timestamps
 
 @returns Arithmetic mean
 
 @example
 ```ts
 computeMean([1, 2, 3]) // 2
 ```
 */
function computeMean(values: readonly number[],): number {
  return sumValues(values,) / values
    .length;
}

/**
 Computes the median of an array of numbers.
 
 For even-length arrays, returns the lower of the two middle values
 to avoid fractional timestamps.
 
 @param values - Non-empty array of timestamps
 
 @returns Median value
 
 @example
 ```ts
 computeMedian([3, 1, 2]) // 2
 computeMedian([4, 1, 3, 2]) // 2
 ```
 */
function computeMedian(values: readonly number[],): number {
  /**
   Ascending copy of the input so the original ordering is preserved for the caller.
   */
  const sorted = [...values,].toSorted(function ascending(
    a,
    b,
  ) {
    return a - b;
  },);
  /**
   Index of the upper middle element; for even-length arrays the lower middle (`mid - 1`) is used to avoid fractional values.
   */
  const mid = Math.floor(sorted.length
    / 2,);
  // Even length: use lower middle to avoid fractional timestamps
  if ((sorted.length
    % 2) === 0)
    return sorted[mid - 1]
      ?? 0;
  return sorted[mid]
    ?? 0;
}

/**
 Maps strategy names to their aggregation functions, including {@link computeMean}
 and {@link computeMedian} for the `mean`/`median` strategies.
 
 Empty arrays return `-Infinity` ("no information available") regardless
 of strategy. This means empty sources = "nothing to trigger on" (fresh)
 and empty outputs = "nothing exists yet" (stale), because
 `-Infinity \> x` is always false and `x \> -Infinity` is always true.
 
 @example
 ```ts
 builtinStrategies.newest([1, 2, 3]) // 3
 builtinStrategies.oldest([1, 2, 3]) // 1
 builtinStrategies.mean([1, 2, 3]) // 2
 builtinStrategies.newest([]) // -Infinity
 ```
 */
export const builtinStrategies: Readonly<
  Record<BuiltinTimeStrategy, (values: readonly number[],) => number>
  > = {
    newest: function newest(values,) {
      return values.length
        === 0 ? -Infinity : Math.max(...values,);
    },
    oldest: function oldest(values,) {
      return values.length
        === 0 ? -Infinity : Math.min(...values,);
    },
    mean: function mean(values,) {
      return values.length
        === 0 ? -Infinity : computeMean(values,);
    },
    median: function median(values,) {
      return values.length
        === 0 ? -Infinity : computeMedian(values,);
    },
  };

//endregion Strategy functions
