import { CloneSizeError, } from './errors.ts';
import type {
  Confidence,
  Estimate,
  FusionState,
} from './types.ts';

/**
 Numeric rank for each confidence level, for stepping up and down the ladder.
 */
const RANK_BY_CONFIDENCE: Record<Confidence, number> = {
  'low': 0,
  'medium': 1,
  'high': 2,
  'very high': 3,
};

/**
 Highest rank reachable by an agreement bump; agreement never fabricates the
 `very high` tier reserved for an exact local measurement.
 */
const MAX_BUMP_RANK = 2;

/**
 Relative fused half-width below which agreeing medium estimators are bumped
 to high confidence.
 */
const TIGHT_RELATIVE_WIDTH = 0.15;

/**
 Maps a numeric rank back to a confidence level without array indexing.
 
 @param rank - confidence rank (0 low to 3 very high)
 
 @returns confidence level for the clamped rank
 
 @example
 ```ts
 confidenceFromRank(2); // 'high'
 ```
 */
function confidenceFromRank(rank: number,): Confidence {
  if (rank >= RANK_BY_CONFIDENCE['very high'])
    return 'very high';
  if (rank === RANK_BY_CONFIDENCE.high)
    return 'high';
  if (rank === RANK_BY_CONFIDENCE.medium)
    return 'medium';
  return 'low';
}

/**
 Sums a numeric list.
 
 @param values - numbers to add
 
 @returns arithmetic sum
 
 @example
 ```ts
 sum([1, 2, 3]); // 6
 ```
 */
function sum(values: readonly number[],): number {
  return (function sumValues(): number {
    /**
     Arithmetic sum isolated inside local mutation scope.
     */
    let total = 0;
    for (const value of values)
      total += value;
    return total;
  })();
}

/**
 De-duplicates basis names, preserving first-seen order.
 
 @param estimates - contributing estimators
 
 @returns unique basis labels
 
 @example
 ```ts
 uniqueBasis([{ name: 'a' }, { name: 'a' }, { name: 'b' }] as Estimate[]); // ['a', 'b']
 ```
 */
function uniqueBasis(estimates: readonly Estimate[],): readonly string[] {
  return [...new Set(estimates.map(function name(estimate,) {
    return estimate.name;
  },),),];
}

/**
 Whether the intersection of every estimate interval is empty, the signal that
 estimators conflict rather than agree.
 
 @param estimates - contributing estimators
 
 @returns true when no single full size satisfies all intervals
 */
function intervalsConflict(estimates: readonly Estimate[],): boolean {
  /**
   Greatest lower bound across all intervals.
   */
  const maxLo = Math.max(...estimates.map(function lo(estimate,) {
    return estimate.lo;
  },),);
  /**
   Least upper bound across all intervals.
   */
  const minHi = Math.min(...estimates.map(function hi(estimate,) {
    return estimate.hi;
  },),);
  return maxLo > minHi;
}

/**
 Fuses the agreeing-case interval: a weighted-average band pulled toward the
 combined point by `1 / sqrt(count)`, so the interval tightens as independent
 agreeing estimators accumulate while a lone estimator is preserved exactly.
 
 @param estimates - contributing estimators (assumed non-conflicting)
 
 @param point - precision-weighted combined point
 
 @returns fused low and high bounds
 */
function fuseAgreeingBand(
  {
    estimates,
    point,
  }: {
    readonly estimates: readonly Estimate[];
    readonly point: number
  },
): {
  readonly lo: number;
  readonly hi: number
} {
  /**
   Sum of weights, the denominator for weighted averages.
   */
  const weightTotal = sum(estimates.map(function weight(estimate,) {
    return estimate.weight;
  },),);
  /**
   Weighted-average low bound.
   */
  const avgLo = sum(estimates.map(function weightedLo(estimate,) {
    return estimate.weight * estimate.lo;
  },),) / weightTotal;
  /**
   Weighted-average high bound.
   */
  const avgHi = sum(estimates.map(function weightedHi(estimate,) {
    return estimate.weight * estimate.hi;
  },),) / weightTotal;
  /**
   Estimators whose interval contains the combined point (mutually agreeing).
   */
  const agreeingCount = estimates.filter(function contains(estimate,) {
    return (estimate.lo <= point) && (point <= estimate.hi);
  },)
    .length;
  /**
   Tightening factor: 1 for a lone estimate, shrinking as agreement grows.
   */
  const tighten = 1 / Math.sqrt(Math.max(
    1,
    agreeingCount,
  ),);
  return {
    hi: point + ((avgHi - point)
      * tighten),
    lo: Math.max(
      0,
      point - ((point - avgLo)
        * tighten),
    ),
  };
}

/**
 Picks the fused confidence: the best contributing tier, bumped one step when
 two or more medium-or-better estimators agree within a tight band, and never
 exceeding high through agreement alone.
 
 @param estimates - contributing estimators
 
 @param point - combined point
 
 @param halfWidth - fused half-width
 
 @returns fused confidence level
 */
function fuseConfidence(
  {
    estimates,
    point,
    halfWidth,
  }: {
    readonly estimates: readonly Estimate[];
    readonly point: number;
    readonly halfWidth: number
  },
): Confidence {
  /**
   Best confidence rank among contributors.
   */
  const baseRank = Math.max(...estimates.map(function rank(estimate,) {
    return RANK_BY_CONFIDENCE[estimate.confidence];
  },),);
  /**
   Count of medium-or-better estimators whose interval contains the point.
   */
  const meaningfulAgree = estimates.filter(function agrees(estimate,) {
    return (RANK_BY_CONFIDENCE[estimate.confidence] >= RANK_BY_CONFIDENCE.medium)
      && (estimate.lo <= point)
      && (point <= estimate.hi);
  },)
    .length;
  /**
   Whether the fused band is tight relative to the point.
   */
  const tight = (point > 0) && ((halfWidth / point) < TIGHT_RELATIVE_WIDTH);
  if ((meaningfulAgree >= 2) && tight
    && (baseRank < MAX_BUMP_RANK))
    return confidenceFromRank(baseRank + 1,);
  return confidenceFromRank(baseRank,);
}

/**
 Fuses a set of estimators into a single full-size belief. An exact local
 measurement (`very high`) collapses the result to its tiny band and dominates.
 Otherwise the point is precision-weighted, the band tightens when independent
 estimators agree, and conflicting intervals widen to their union and drop
 confidence one step. Called after each new signal lands, so the snapshot is
 always the best fusion of signals seen so far.
 
 @param estimates - all estimators gathered so far (at least one)
 
 @returns fused point, interval, confidence, and basis labels
 
 @throws {@link CloneSizeError} when called with no estimators
 
 @example
 ```ts
 const fused = combineEstimates({ estimates: [priorEstimate({ c1Bytes: 4_000_000 })] });
 ```
 */
export function combineEstimates({ estimates, }: { readonly estimates: readonly Estimate[]; },): FusionState {
  if (estimates.length === 0)
    throw new CloneSizeError({ message: 'combineEstimates requires at least one estimate', },);

  /**
   Exact local measurements, which dominate when present.
   */
  const exact = estimates.filter(function isExact(estimate,) {
    return estimate.confidence === 'very high';
  },);
  if (exact.length > 0) {
    /**
     Highest-weight exact estimate.
     */
    const best = exact.reduce(function heavier(
      winner,
      candidate,
    ) {
      return candidate.weight > winner.weight ? candidate : winner;
    },);
    return {
      basis: uniqueBasis(exact,),
      confidence: 'very high',
      hi: best.hi,
      lo: best.lo,
      point: best.point,
    };
  }

  /**
   Sum of weights for the precision-weighted point.
   */
  const weightTotal = sum(estimates.map(function weight(estimate,) {
    return estimate.weight;
  },),);
  /**
   Precision-weighted combined point.
   */
  const point = sum(estimates.map(function weightedPoint(estimate,) {
    return estimate.weight * estimate.point;
  },),) / weightTotal;

  if (intervalsConflict(estimates,)) {
    /**
     Best contributing confidence rank, downgraded one step for the conflict.
     */
    const downgraded = Math.max(
      0,
      Math.max(...estimates.map(function rank(estimate,) {
        return RANK_BY_CONFIDENCE[estimate.confidence];
      },),) - 1,
    );
    return {
      basis: uniqueBasis(estimates,),
      confidence: confidenceFromRank(downgraded,),
      hi: Math.max(...estimates.map(function hi(estimate,) {
        return estimate.hi;
      },),),
      lo: Math.max(
        0,
        Math.min(...estimates.map(function lo(estimate,) {
          return estimate.lo;
        },),),
      ),
      point,
    };
  }

  /**
   Tightened agreeing-case band.
   */
  const band = fuseAgreeingBand({
    estimates,
    point,
  },);
  return {
    basis: uniqueBasis(estimates,),
    confidence: fuseConfidence({
      estimates,
      point,
      halfWidth: (band.hi - band.lo) / 2,
    },),
    hi: band.hi,
    lo: band.lo,
    point,
  };
}
