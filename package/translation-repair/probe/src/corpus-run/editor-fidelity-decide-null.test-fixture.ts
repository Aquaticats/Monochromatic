import { countWithNoun, } from '../../dist/final/node/index.mjs';

//region Editor fidelity decision rule under no difference
// THE CHANCE THE DECISION RULE CALLS AN EFFORT DIFFERENT WHEN NONE IS, computed exactly and not simulated. The rule
// compares an effort's pooled clean fix count with the mean of the two medium arms against a threshold that is the
// larger of a floor and the two medium arms' own difference (`editor-fidelity-decide.ts`). This module enumerates every
// count the arms can show and sums the probability of the ones the rule calls, for arms that all fix a trial with the
// same chance.
//
// THE ARMS ARE INDEPENDENT AND THE TRIALS WITHIN AN ENTRY ARE NOT. The pool draws at most one insertion and one
// alteration from an entry, and the two sit in one page of one translation, so one arm that fixes one is likelier to
// fix the other. The counts are therefore clusters: an entry with both kinds adds 0, 1 or 2 to an arm's count, with a
// within-entry correlation from 0 (independent trials) to 1 (the two trials of an entry always read alike). Forty
// trials over twenty-four entries are not forty independent trials, and the effective size is nearer the entries than
// the trials. The same trials are asked of every arm, which makes the arms' counts agree more than independent arms
// would, so treating the arms as independent overstates how far they stray and the rate computed here is an upper
// bound.
//
// THE KIND VETO IS LEFT OUT. It can only turn a call into a keep, so the rate without it is an upper bound on the rate
// with it.

/**
 How the frozen pool's entries split by the kinds of defect they carry.

 @example
 ```ts
 const clusters: PoolClusters = { both: 16, insertionOnly: 6, alterationOnly: 2, };
 ```
 */
type PoolClusters = {
  /**
   Entries with an insertion trial and an alteration trial.
   */
  readonly both: number;

  /**
   Entries with an insertion trial only.
   */
  readonly insertionOnly: number;

  /**
   Entries with an alteration trial only.
   */
  readonly alterationOnly: number;
};

/**
 The split of the frozen pool of forty trials over twenty-four entries: sixteen entries with both kinds, six with an
 insertion only and two with an alteration only (insertion 22, alteration 18).
 */
export const POOL_CLUSTERS: PoolClusters = {
  both: 16,
  insertionOnly: 6,
  alterationOnly: 2,
};

/**
 The lowest success chance checked, in hundredths.
 */
const FIRST_RATE_PERCENT = 30;

/**
 How many success chances are checked, from the lowest in steps of one hundredth.
 */
const RATE_COUNT = 66;

/**
 Hundredths in one.
 */
const PERCENT = 100;

/**
 Steps between a correlation of none and one of always, a quarter each.
 */
const CORRELATION_STEPS = 4;

/**
 Success chances the no-difference rate is checked at: from 0.30 to 0.95 in steps of one hundredth, the range of rates a
 Sonnet editor card can plausibly have on this pool.
 */
export const NULL_RATES: readonly number[] = Array.from(
  { length: RATE_COUNT, },
  function rateAt(
    _unused,
    at,
  ): number {
    return (FIRST_RATE_PERCENT + at) / PERCENT;
  },
);

/**
 Within-entry correlations the no-difference rate is checked at, from independent trials to trials that always read
 alike.
 */
export const NULL_CORRELATIONS: readonly number[] = Array.from(
  { length: CORRELATION_STEPS + 1, },
  function correlationAt(
    _unused,
    at,
  ): number {
    return at / CORRELATION_STEPS;
  },
);

/**
 The largest chance of calling an effort different that the rule may have when none is.
 */
export const NULL_CALL_LIMIT = 0.05;

/**
 Convolves two distributions of counts.

 @param left - chance of each count of the first, from zero

 @param right - chance of each count of the second, from zero

 @returns The chance of each sum, from zero

 @example
 ```ts
 // yields [0.25, 0.5, 0.25]
 const sums = convolve({ left: [0.5, 0.5,], right: [0.5, 0.5,], },);
 ```
 */
function convolve(
  {
    left,
    right,
  }: {
    readonly left: readonly number[];
    readonly right: readonly number[];
  },
): readonly number[] {
  return Array.from(
    { length: (left.length + right.length) - 1, },
    function sumAt(
      _unused,
      total,
    ): number {
      return left.reduce(
        function add(
          chance: number,
          each,
          at,
        ): number {
          return chance + (each * (right[total - at] ?? 0));
        },
        0,
      );
    },
  );
}

/**
 The chance of each count of an arm over the pool's clusters.

 @param clusters - how the pool's entries split by kind

 @param successRate - chance that one trial is fixed

 @param correlation - how far the two trials of an entry read alike, from 0 to 1

 @returns The chance of each clean fix count, from zero to the pool's trials

 @example
 ```ts
 const chances = armCountChances({ clusters: POOL_CLUSTERS, successRate: 0.5, correlation: 1, },);
 ```
 */
export function armCountChances(
  {
    clusters,
    successRate,
    correlation,
  }: {
    readonly clusters: PoolClusters;
    readonly successRate: number;
    readonly correlation: number;
  },
): readonly number[] {
  /**
   Chance a trial is missed.
   */
  const miss = 1 - successRate;
  /**
   The chance both trials of an entry are fixed beyond what independent trials would give.
   */
  const together = correlation * successRate
    * miss;
  /**
   An entry with both kinds: neither fixed, one fixed, both fixed.
   */
  const pair: readonly number[] = [
    (miss * miss) + together,
    2 * successRate
      * miss
      * (1 - correlation),
    (successRate * successRate) + together,
  ];
  /**
   An entry with one kind: missed or fixed.
   */
  const single: readonly number[] = [
    miss,
    successRate,
  ];
  /**
   Every entry's chances, one after the other.
   */
  const entries: readonly (readonly number[])[] = [
    ...Array.from(
      { length: clusters.both, },
      function bothEntry(): readonly number[] {
        return pair;
      },
    ),
    ...Array.from(
      { length: clusters.insertionOnly + clusters.alterationOnly, },
      function oneEntry(): readonly number[] {
        return single;
      },
    ),
  ];
  return entries.reduce< readonly number[]>(
    function fold(
      chances: readonly number[],
      each,
    ): readonly number[] {
      return convolve({
        left: chances,
        right: each,
      },);
    },
    [1,],
  );
}

/**
 The chance that a count lies strictly less than a threshold away from a mean, with both doubled to stay whole.

 @param running - the chance of a count below each count, from zero, one entry longer than the counts

 @param sumTwice - the two medium counts added, which is twice their mean

 @param thresholdTwice - twice the threshold in trials

 @returns The chance of a count c with the absolute value of 2c minus the sum less than the doubled threshold

 @example
 ```ts
 const near = chanceInside({ running, sumTwice: 30, thresholdTwice: 20, },);
 ```
 */
function chanceInside(
  {
    running,
    sumTwice,
    thresholdTwice,
  }: {
    readonly running: readonly number[];
    readonly sumTwice: number;
    readonly thresholdTwice: number;
  },
): number {
  /**
   The largest count there is.
   */
  const top = running.length - 2;
  /**
   First count strictly above the lower edge, never below zero.
   */
  const from = Math.max(
    0,
    Math.floor((sumTwice - thresholdTwice) / 2,) + 1,
  );
  /**
   Last count strictly below the upper edge, never above the largest.
   */
  const to = Math.min(
    top,
    Math.ceil((sumTwice + thresholdTwice) / 2,) - 1,
  );
  return (to < from) ? 0 : (running[to + 1] ?? 0) - (running[from] ?? 0);
}

/**
 The chance that the rule calls at least one of two efforts different from the default when all four arms fix a trial
 with the same chance: the two medium arms and the two efforts compared with them are independent draws of one
 distribution.

 @param clusters - how the pool's entries split by kind

 @param successRate - chance that one trial is fixed, the same in every arm

 @param correlation - how far the two trials of an entry read alike, from 0 to 1

 @param floorTrials - the least pooled gap that can count, in trials

 @returns The chance at least one of the two efforts is called different

 @example
 ```ts
 const chance = falseCallRate({ clusters: POOL_CLUSTERS, successRate: 0.5, correlation: 1, floorTrials: 10, },);
 ```
 */
export function falseCallRate(
  {
    clusters,
    successRate,
    correlation,
    floorTrials,
  }: {
    readonly clusters: PoolClusters;
    readonly successRate: number;
    readonly correlation: number;
    readonly floorTrials: number;
  },
): number {
  /**
   The chance of each count in every arm.
   */
  const chances = armCountChances({
    clusters,
    successRate,
    correlation,
  },);
  /**
   The chance of a count below each count, from zero, so a stretch of counts is a difference of two entries.
   */
  const running = Array.from(
    { length: chances.length + 1, },
    function runningTo(
      _unused,
      upTo,
    ): number {
      return chances.slice(
        0,
        upTo,
      )
        .reduce(
        function add(
          sum: number,
          each: number,
        ): number {
          return sum + each;
        },
        0,
        );
    },
  );
  /**
   The chance that neither effort is called, summed over the pairs of medium counts.
   */
  const neither = chances.reduce(
    function overFirst(
      outer: number,
      first,
      firstCount,
    ): number {
      return outer + (chances.reduce(
        function overSecond(
          inner: number,
          second,
          secondCount,
        ): number {
          /**
           Twice the threshold in trials, so the mean of the mediums stays a whole number.
           */
          const thresholdTwice = 2 * Math.max(
            floorTrials,
            Math.abs(firstCount - secondCount,),
          );
          /**
           The chance one effort's count lies inside the threshold of the mediums' mean.
           */
          const inside = chanceInside({
            running,
            sumTwice: firstCount + secondCount,
            thresholdTwice,
          },);
          return inner + (second * inside
            * inside);
        },
        0,
      ) * first);
    },
    0,
  );
  return 1 - neither;
}

/**
 The chance of calling an effort different at one success chance and correlation.

 @example
 ```ts
 const worst: WorstCase = { rate: 0.043, successRate: 0.5, correlation: 1, };
 ```
 */
type WorstCase = {
  /**
   The chance at least one effort is called different.
   */
  readonly rate: number;

  /**
   The success chance it falls at.
   */
  readonly successRate: number;

  /**
   The correlation it falls at.
   */
  readonly correlation: number;
};

/**
 The largest chance of calling an effort different, over the success chances and the correlations checked.

 @param clusters - how the pool's entries split by kind

 @param floorTrials - the least pooled gap that can count, in trials

 @returns The largest rate and where it falls

 @example
 ```ts
 const { rate, successRate, correlation, } = worstFalseCall({ clusters: POOL_CLUSTERS, floorTrials: 10, },);
 ```
 */
export function worstFalseCall(
  {
    clusters,
    floorTrials,
  }: {
    readonly clusters: PoolClusters;
    readonly floorTrials: number;
  },
): WorstCase {
  return NULL_CORRELATIONS.flatMap(function overCorrelation(correlation,) {
    return NULL_RATES.map(function overRate(successRate,) {
      return {
        rate: falseCallRate({
          clusters,
          successRate,
          correlation,
          floorTrials,
        },),
        successRate,
        correlation,
      };
    },);
  },)
    .reduce(function larger(
      worst: WorstCase,
      each: WorstCase,
    ): WorstCase {
      return (each.rate > worst.rate) ? each : worst;
    },);
}

/**
 The smallest floor, in whole trials, that keeps the chance of calling an effort different under the limit at every
 success chance and correlation checked.

 @param clusters - how the pool's entries split by kind

 @returns The floor in trials

 @throws {@link Error} when no floor up to the pool's trials is enough

 @example
 ```ts
 const floorTrials = smallestFloor({ clusters: POOL_CLUSTERS, },);
 ```
 */
export function smallestFloor({ clusters, }: { readonly clusters: PoolClusters; },): number {
  /**
   The trials the pool holds, the largest gap there is.
   */
  const trials = (2 * clusters.both) + clusters.insertionOnly
    + clusters.alterationOnly;
  for (let floorTrials = 0; floorTrials <= trials; floorTrials += 1) {
    if (worstFalseCall({
      clusters,
      floorTrials,
    },)
      .rate
      <= NULL_CALL_LIMIT)
      return floorTrials;
  }
  throw new Error(`no floor up to ${
    countWithNoun({
      count: trials,
      one: 'trial',
      many: 'trials',
    },)
  } keeps the chance of a false call under ${String(NULL_CALL_LIMIT,)}`,);
}

//endregion Editor fidelity decision rule under no difference
