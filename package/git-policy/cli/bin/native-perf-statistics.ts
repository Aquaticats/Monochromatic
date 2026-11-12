/**
 Order statistics of the latency samples that `native-perf.ts` reports for each scenario.

 `SPEC.md` ("Benchmark method") asks for the median, the p95 and the median absolute deviation of every sample set.
 */

/**
 The fraction of samples at or below the reported high percentile.
 */
const HIGH_PERCENTILE = 0.95;

/**
 A statistic was asked of a sample set that cannot answer it, such as an empty one.
 */
export class SampleSetError extends Error {
  /**
   Stable class name that tells an unanswerable statistic apart from an unrelated error in a log.
   */
  override name = 'SampleSetError';
}

/**
 Sample at one index of an ascending set, which the median and percentile arithmetic keeps in range for a non-empty set.

 @param sorted - ascending samples

 @param index - position to read

 @returns the sample, so an empty set fails loudly instead of yielding `undefined`

 @throws SampleSetError when the index is outside the set
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
    throw new SampleSetError(`Sample index ${String(index)} is outside a set of ${String(sorted.length)} samples.`);
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

/**
 One successful sample: the round it was taken in and its elapsed milliseconds.
 */
export type RoundSample = {
  readonly sample: number;
  readonly elapsed: number;
};

/**
 Wrapped minus direct milliseconds for every round in which both sides succeeded, in round order.
 A round where either side failed contributes nothing, so a failure never shifts the pairing of the later rounds.

 @param direct - successful direct samples with their rounds

 @param wrapped - successful wrapped samples with their rounds

 @returns one delta per round that has both samples

 @example
 ```ts
 pairedDeltas({ direct: [{ sample: 0, elapsed: 4 }], wrapped: [{ sample: 0, elapsed: 9 }] });
 ```
 */
export function pairedDeltas({
  direct,
  wrapped,
}: {
  readonly direct: readonly RoundSample[];
  readonly wrapped: readonly RoundSample[];
}): readonly number[] {
  return pairedRounds({
    direct,
    wrapped,
  })
    .map(function delta(round) {
      return round.delta;
    });
}

/**
 Both sides' times for every round in which they both succeeded, in round order, so a bound that depends on the direct
 time of a round can be worked out for that round alone.
 A round where either side failed contributes nothing, so a failure never shifts the pairing of the later rounds.

 @param direct - successful direct samples with their rounds

 @param wrapped - successful wrapped samples with their rounds

 @returns one round per round that has both samples

 @example
 ```ts
 pairedRounds({ direct: [{ sample: 0, elapsed: 4 }], wrapped: [{ sample: 0, elapsed: 9 }] });
 ```
 */
export function pairedRounds({
  direct,
  wrapped,
}: {
  readonly direct: readonly RoundSample[];
  readonly wrapped: readonly RoundSample[];
}): readonly {
  readonly direct: number;
  readonly delta: number;
}[] {
  /**
   Direct elapsed milliseconds by round.
   */
  const directByRound = new Map(direct.map(function byRound(entry) {
    return [
      entry.sample,
      entry.elapsed,
    ] as const;
  }));
  return wrapped.flatMap(function roundOf(entry): readonly {
    readonly direct: number;
    readonly delta: number;
  }[] {
    /**
     Direct sample of the same round, absent when the direct side failed in it.
     */
    const partner = directByRound.get(entry.sample);
    return partner === undefined
      ? []
      : [
        {
          direct: partner,
          delta: entry.elapsed - partner,
        },
      ];
  });
}

/**
 Median, p95 and maximum of the paired deltas of a scenario, the shape the report stores for the commit measure.

 @param deltas - paired deltas in milliseconds, at least one

 @returns the three statistics; the maximum is reported because the owner's measure is about any one commit

 @example
 ```ts
 pairedDeltaSummary([1, 2, 3]);
 ```
 */
export function pairedDeltaSummary(deltas: readonly number[]): {
  readonly median: number;
  readonly p95: number;
  readonly maximum: number;
} {
  return {
    median: median(deltas),
    p95: percentile95(deltas),
    maximum: deltas.reduce(function larger(
      most,
      delta,
    ) {
      return Math.max(
        most,
        delta,
      );
    }),
  };
}
