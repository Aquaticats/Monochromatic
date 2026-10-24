//region Editor fidelity paired test
// THE PAIRED TEST THE EDITOR BENCH IS READ BY, as a skeleton: the types and the names the tests reach. The red commit of
// the paired test holds this file so that its cases fail by assertion and not by a missing export; the commit after it
// builds the test.

/**
 The level the test is run at for each effort against the default effort, one-sided, as a fraction: 0.025.
 */
export const ALPHA = 0.025;

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
   @param entryId - the entry whose counts do not line up

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
 Runs the paired exact permutation test of the first arm against the others.

 @param clusters - the entries, each with the clean fixes of every arm in its shared trials

 @returns The estimate, the one-sided exact p-value and whether it is at most the level

 @throws {@link PairedShapeError} when the entries state different numbers of arms, or fewer than two

 @example
 ```ts
 const test = pairedPermutationTest({ clusters, },);
 ```
 */
export function pairedPermutationTest({ clusters, }: { readonly clusters: readonly PairedCluster[]; },): PairedTest {
  return {
    clusters: clusters.length,
    trials: 0,
    estimate: 0,
    p: 1,
    atMostAlpha: false,
  };
}

/**
 The interval of the paired estimate by a seeded bootstrap over the entries.

 @param clusters - the entries, as for the test

 @param seed - the seed of the resampling

 @param replicates - how many resamples to make

 @returns The centred interval of the resampled estimates

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
  return {
    low: Number((clusters.length > 0) && (seed !== '')
      && (replicates > 0)) - 1,
    high: 1,
  };
}

//endregion Editor fidelity paired test
