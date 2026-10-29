import { createHash, } from 'node:crypto';

//region Editor fidelity paired test
// THE PAIRED TEST THE EDITOR BENCH IS READ BY. Every arm walks the same trials, so an effort and the default effort are
// compared trial by trial and not count against count. The trials of one entry sit in one page of one translation and read
// alike more often than strangers do, so the unit of the test is the entry: a cluster of at most two trials.
//
// THE TEST IS EXACT AND CONDITIONAL. Under the hypothesis that the arms are one card, the arms' clean fixes in an entry
// could have come out in any assignment of the arms to the labels, each equally likely, whatever the entry's difficulty and
// however alike its trials read. The statistic is the effort's fixes minus the mean of the other arms' fixes, summed over
// the entries, and the p-value is the share of assignments, taken within each entry independently, whose statistic reaches
// the observed one. That share is counted over the whole distribution by convolving the entries one at a time, never by
// listing the assignments, and in whole numbers, so that a probability a float cannot hold (two to the minus sixty) is
// still decided at the level exactly. With two arms the assignments of an entry are two, the sign of its difference.
//
// THE INTERVAL IS A SEEDED CLUSTER BOOTSTRAP. The entries are resampled with replacement, the estimate is taken again on
// each resample, and the interval is the centre of the resampled estimates. It is the interval of the estimate and not
// the inversion of the test, so it can include zero when the test is at its level and the reverse; the rule says which it
// prints. The draws of the resampling are the bytes of sha256 over the seed and a block number, so one seed gives one
// interval in any runtime.

/**
 The level the test is run at for each effort against the default effort, one-sided, as a fraction.
 */
export const ALPHA = 0.025;

/**
 The level as an exact fraction of the assignments: the denominator of one in forty.
 */
const ALPHA_DENOMINATOR = 40n;

/**
 How many resamples of the entries the interval is made from.
 */
export const BOOTSTRAP_REPLICATES = 10_000;

/**
 The share of the resampled estimates the interval covers, centred.
 */
export const INTERVAL_LEVEL = 0.95;

/**
 The seed the interval's resampling is made under, so a log read twice gives one interval.
 */
export const BOOTSTRAP_SEED = 'editor-fidelity-bench/interval/v1';

/**
 Ten, the base of the scale a ratio of two large counts is printed at.
 */
const TEN = 10n;

/**
 Decimal places a ratio of two large counts is printed to.
 */
const RATIO_DIGITS = 15n;

/**
 The scale a ratio of two large counts is printed at.
 */
const RATIO_SCALE = TEN ** RATIO_DIGITS;

/**
 The arms a test is made of at least: the effort and one to compare it with.
 */
const ARMS_LEAST = 2;

/**
 Bytes of a hash block that make one draw.
 */
const BYTES_PER_DRAW = 6;

/**
 Draws one hash block holds: a sha256 digest is thirty-two bytes.
 */
const DRAWS_PER_BLOCK = 5;

/**
 Bits in a byte.
 */
const BITS_PER_BYTE = 8;

/**
 Two, the base the draw's range is a power of.
 */
const BINARY_BASE = 2;

/**
 The number of values a draw takes, so that dividing by it gives a number in the unit interval.
 */
const DRAW_RANGE = BINARY_BASE ** (BYTES_PER_DRAW * BITS_PER_BYTE);

/**
 One entry's share of the shared trials: how many trials it holds and how many each arm fixed cleanly in them.

 @example
 ```ts
 const cluster: PairedCluster = { entryId: 'tabby', trials: 2, fixed: [2, 1,], };
 ```
 */
export type PairedCluster = {
  /**
   The entry the trials come from.
   */
  readonly entryId: string;

  /**
   Shared trials of the entry.
   */
  readonly trials: number;

  /**
   Clean fixes in them for each arm: the effort under test first, then the arms it is compared with.
   */
  readonly fixed: readonly number[];
};

/**
 What the exact test says of one effort against the arms it is compared with.

 @example
 ```ts
 const test: PairedTest = pairedPermutationTest({ clusters, },);
 ```
 */
export type PairedTest = {
  /**
   Entries tested.
   */
  readonly clusters: number;

  /**
   Shared trials in them.
   */
  readonly trials: number;

  /**
   The mean paired difference per shared trial, as a fraction: the effort's fixes minus the mean of the other arms'.
   */
  readonly estimate: number;

  /**
   The one-sided exact p-value.
   */
  readonly p: number;

  /**
   Whether the p-value is at most the level, decided on the exact counts and not on the printed p-value.
   */
  readonly atMostAlpha: boolean;
};

/**
 The interval around an estimate.

 @example
 ```ts
 const interval: PairedInterval = { low: -0.02, high: 0.31, };
 ```
 */
export type PairedInterval = {
  /**
   Lower end, as a fraction of the shared trials.
   */
  readonly low: number;

  /**
   Upper end, as a fraction of the shared trials.
   */
  readonly high: number;
};

/**
 Refuses entries whose arm counts do not line up.

 @example
 ```ts
 throw new PairedShapeError({ entryId: 'tabby', },);
 ```
 */
export class PairedShapeError extends Error {
  /**
   Only an entry id enters the message.
   */
  readonly messageNamesOnly: true = true;

  /**
   @param entryId - entry whose counts do not line up

   @example
   ```ts
   new PairedShapeError({ entryId: 'tabby', },);
   ```
   */
  public constructor({ entryId, }: { readonly entryId: string; },) {
    super(`entry ${entryId} does not state the same arms as the others`,);
    this.name = 'PairedShapeError';
  }
}

/**
 Reads how many arms the entries state, the same in every entry.

 @param clusters - entries

 @returns Number of arms, two when there are no entries

 @throws {@link PairedShapeError} when the entries state different numbers of arms, or fewer than two

 @example
 ```ts
 const arms = armCountOf({ clusters, },);
 ```
 */
function armCountOf({ clusters, }: { readonly clusters: readonly PairedCluster[]; },): number {
  /**
   The first entry, which the others are held to.
   */
  const [first,] = clusters;
  if (first === undefined)
    return ARMS_LEAST;
  for (const cluster of clusters) {
    if ((cluster.fixed
      .length
      !== first.fixed
        .length) || (cluster.fixed
          .length
          < ARMS_LEAST))
      throw new PairedShapeError({ entryId: cluster.entryId, },);
  }
  return first.fixed
    .length;
}

/**
 Adds counts.

 @param counts - counts

 @returns Their sum

 @example
 ```ts
 // yields 3
 const sum = sumOf({ counts: [1, 2,], },);
 ```
 */
function sumOf({ counts, }: { readonly counts: readonly number[]; },): number {
  return counts.reduce(
    function add(
      total: number,
      count: number,
    ): number {
      return total + count;
    },
    0,
  );
}

/**
 The statistic one entry adds when each arm in turn stands as the effort, in whole numbers: the number of arms times the
 chosen arm's fixes, minus all the arms' fixes. Divided by the arms less one it is the chosen arm's fixes minus the mean of
 the others'.

 @param cluster - entry

 @returns One number for each arm, the first being the observed one

 @example
 ```ts
 // yields [1, -1]
 const contributions = contributionsOf({ cluster: { entryId: 'tabby', trials: 1, fixed: [1, 0,], }, },);
 ```
 */
function contributionsOf({ cluster, }: { readonly cluster: PairedCluster; },): readonly number[] {
  /**
   All the arms' fixes in the entry.
   */
  const all = sumOf({ counts: cluster.fixed, },);
  return cluster.fixed
    .map(function scaled(count,): number {
      return (cluster.fixed
        .length
        * count) - all;
    },);
}

/**
 The distribution of the statistic over every assignment, as the number of assignments for each value, by convolving the
 entries one at a time.

 @param clusters - entries

 @returns Assignments counted for each sum of the entries' contributions

 @example
 ```ts
 const ways = distributionOf({ clusters, },);
 ```
 */
function distributionOf({ clusters, }: { readonly clusters: readonly PairedCluster[]; },): ReadonlyMap<number, bigint> {
  return clusters.reduce(
    function spread(
      ways: ReadonlyMap<number, bigint>,
      cluster,
    ): ReadonlyMap<number, bigint> {
      /**
       The distribution once this entry is in.
       */
      const next = new Map<number, bigint>();
      for (const [sum, count,] of ways) {
        for (const contribution of contributionsOf({ cluster, },)) {
          /**
           The statistic when this entry takes this assignment.
           */
          const reached = sum + contribution;
          next.set(
            reached,
            (next.get(reached,) ?? 0n) + count,
          );
        }
      }
      return next;
    },
    new Map<number, bigint>([[
      0,
      1n,
    ],],),
  );
}

/**
 Runs the paired exact permutation test of the first arm against the others.

 @param clusters - entries, each with the clean fixes of every arm in its shared trials

 @returns Estimate, one-sided exact p-value and whether it is at most the level

 @throws {@link PairedShapeError} when the entries state different numbers of arms, or fewer than two

 @example
 ```ts
 const test = pairedPermutationTest({ clusters, },);
 ```
 */
export function pairedPermutationTest({ clusters, }: { readonly clusters: readonly PairedCluster[]; },): PairedTest {
  /**
   The arms every entry states.
   */
  const arms = armCountOf({ clusters, },);
  /**
   The observed statistic: the first arm standing as the effort in every entry.
   */
  const observed = sumOf({
    counts: clusters.map(function firstOf(cluster,): number {
      return contributionsOf({ cluster, },)[0] ?? 0;
    },),
  },);
  /**
   Assignments whose statistic reaches the observed one.
   */
  const tally = { reaching: 0n, };
  for (const [sum, count,] of distributionOf({ clusters, },)) {
    if (sum >= observed)
      tally.reaching += count;
  }
  /**
   All the assignments: every arm in turn in every entry.
   */
  const all = BigInt(arms,) ** BigInt(clusters.length,);
  /**
   Shared trials in the entries.
   */
  const trials = sumOf({
    counts: clusters.map(function trialsOf({ trials: shared, },): number {
      return shared;
    },),
  },);
  return {
    clusters: clusters.length,
    trials,
    estimate: (trials === 0) ? 0 : (observed / (arms - 1)
      / trials),
    p: Number((tally.reaching * RATIO_SCALE) / all,) / Number(RATIO_SCALE,),
    atMostAlpha: (tally.reaching * ALPHA_DENOMINATOR) <= all,
  };
}

/**
 Numbers in the unit interval that one seed always gives: the bytes of sha256 over the seed and a block number, six bytes to
 a draw.

 @param seed - seed

 @param count - how many draws

 @returns Draws, each at least zero and under one

 @example
 ```ts
 const draws = unitDraws({ seed: 'a', count: 12, },);
 ```
 */
export function unitDraws({
  seed,
  count,
}: {
  readonly seed: string;
  readonly count: number
},): readonly number[] {
  return Array.from(
    { length: Math.ceil(count / DRAWS_PER_BLOCK,), },
    function blockAt(
      _,
      block,
    ): readonly number[] {
      /**
       The hash of the seed and this block.
       */
      const digest = createHash('sha256',)
        .update(`${seed}\u0000${String(block,)}`,)
        .digest();
      return Array.from(
        { length: DRAWS_PER_BLOCK, },
        function drawAt(
          __,
          slot,
        ): number {
          return digest.readUIntBE(
            slot * BYTES_PER_DRAW,
            BYTES_PER_DRAW,
          ) / DRAW_RANGE;
        },
      );
    },
  )
    .flat()
    .slice(
    0,
    count,
    );
}

/**
 The interval of the paired estimate by a seeded bootstrap over the entries.

 @param clusters - entries, as for the test

 @param seed - seed of the resampling

 @param replicates - how many resamples to make

 @returns Centred interval of the resampled estimates, a point at zero for no entries

 @throws {@link PairedShapeError} when the entries state different numbers of arms, or fewer than two

 @example
 ```ts
 const interval = clusterBootstrapInterval({ clusters, seed: BOOTSTRAP_SEED, replicates: BOOTSTRAP_REPLICATES, },);
 ```
 */
export function clusterBootstrapInterval(
  {
    clusters,
    seed,
    replicates,
  }: {
    readonly clusters: readonly PairedCluster[];
    readonly seed: string;
    readonly replicates: number;
  },
): PairedInterval {
  /**
   The arms every entry states.
   */
  const arms = armCountOf({ clusters, },);
  if (clusters.length === 0) {
    return {
      low: 0,
      high: 0,
    };
  }
  /**
   Each entry's paired difference in trials and its shared trials.
   */
  const entries = clusters.map(function differenceOf(cluster,): {
    readonly difference: number;
    readonly trials: number;
  } {
    return {
      difference: (contributionsOf({ cluster, },)[0] ?? 0) / (arms - 1),
      trials: cluster.trials,
    };
  },);
  /**
   The draws of every resample, as many for each as there are entries.
   */
  const draws = unitDraws({
    seed,
    count: replicates * entries.length,
  },);
  /**
   The estimate of each resample, smallest first.
   */
  const estimates = Array.from(
    { length: replicates, },
    function resample(
      _,
      replicate,
    ): number {
      /**
       The entries one resample picks, as many as there are, with replacement.
       */
      const picked = entries.map(function pickAt(
        __,
        drawn,
      ): {
        readonly difference: number;
        readonly trials: number
      } {
        /**
         Which entry the draw picks.
         */
        const at = Math.floor((draws[(replicate * entries.length) + drawn] ?? 0) * entries.length,);
        return entries[at] ?? {
          difference: 0,
          trials: 0,
        };
      },);
      /**
       Shared trials in the resample.
       */
      const trials = sumOf({
        counts: picked.map(function trialsOf(entry,): number {
          return entry.trials;
        },),
      },);
      return (trials === 0)
        ? 0
        : (sumOf({
          counts: picked.map(function differenceOf(entry,): number {
            return entry.difference;
          },),
        },) / trials);
    },
  )
    .toSorted(function ascending(
      left,
      right,
    ): number {
      return left - right;
    },);
  /**
   How many estimates are left out at each end.
   */
  const tail = Math.min(
    Math.round(((1 - INTERVAL_LEVEL) / 2) * replicates,),
    Math.floor((replicates - 1) / 2,),
  );
  return {
    low: estimates[tail] ?? 0,
    high: estimates[replicates - 1
      - tail] ?? 0,
  };
}

//endregion Editor fidelity paired test
