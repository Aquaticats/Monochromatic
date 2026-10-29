/**
 Tests for the paired exact permutation test and the seeded interval the editor bench is read by: the p-value against a
 brute-force count of every way to assign the arms' labels within the entries, the boundary of the level, the estimate
 and the interval. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ALPHA,
  BOOTSTRAP_REPLICATES,
  BOOTSTRAP_SEED,
  clusterBootstrapInterval,
  INTERVAL_LEVEL,
  type PairedCluster,
  pairedPermutationTest,
  PairedShapeError,
} from '../../dist/final/node/index.mjs';

/**
 An entry of the given shared trials with the given clean fixes for each arm.

 @param entryId - entry
 @param trials - shared trials in it
 @param fixed - clean fixes in them for each arm, the effort under test first

 @returns Entry

 @example
 ```ts
 const cluster = entryOf({ entryId: 'tabby', trials: 2, fixed: [2, 0,], },);
 ```
 */
function entryOf(
  {
    entryId,
    trials,
    fixed,
  }: {
    readonly entryId: string;
    readonly trials: number;
    readonly fixed: readonly number[];
  },
): PairedCluster {
  return {
    entryId,
    trials,
    fixed,
  };
}

/**
 The one-sided p-value counted the slow way: every assignment of the arms' labels within every entry, each equally likely,
 the statistic being the first arm's fixes minus the mean of the others', summed over the entries.

 @param clusters - entries

 @returns Share of assignments whose statistic reaches the observed one

 @example
 ```ts
 const p = bruteForceP({ clusters, },);
 ```
 */
function bruteForceP({ clusters, }: { readonly clusters: readonly PairedCluster[]; },): number {
  /**
   Arms in each entry.
   */
  const arms = clusters[0]?.fixed.length ?? 2;
  /**
   The statistic of an assignment in doubled-and-scaled integers: arms times the chosen arm's fixes minus the entry's total.
   */
  const scaled = (assignment: readonly number[]): number =>
    clusters.reduce(
      function add(
        total: number,
        cluster,
        index,
      ): number {
        /**
         The arm chosen to stand as the effort in this entry.
         */
        const chosen = assignment[index] ?? 0;
        return (total + (arms * (cluster.fixed[chosen] ?? 0))) - cluster.fixed.reduce(
          function sum(
            all: number,
            count,
          ): number {
            return all + count;
          },
          0,
        );
      },
      0,
    )
    ;
  /**
   The observed statistic: arm zero everywhere.
   */
  const observed = scaled(clusters.map(function zero(): number {
    return 0;
  },),);
  /**
   Assignments counted as reaching it, and all assignments.
   */
  const tally = { reaching: 0, all: 0, };
  for (let code = 0; code < (arms ** clusters.length); code += 1) {
    /**
     The label chosen in each entry, read from the code in base arms.
     */
    const assignment = clusters.map(function digitOf(
      _,
      index,
    ): number {
      return Math.floor(code / (arms ** index)) % arms;
    },);
    tally.all += 1;
    if (scaled(assignment,) >= observed)
      tally.reaching += 1;
  }
  return tally.reaching / tally.all;
}

await describe({
  name: 'editor fidelity paired test',
  children: [
    describe({
      name: 'the constants',
      children: [
        it({
          name: 'FIX THE LEVEL AT 0.025 FOR EACH EFFORT, the interval at 95 percent over 10000 resamples and a named seed, '
            + 'the figures the printed rule states',
          fn: async () => {
            expect(ALPHA,).toBe(0.025,);
            expect(INTERVAL_LEVEL,).toBe(0.95,);
            expect(BOOTSTRAP_REPLICATES,).toBe(10_000,);
            expect(BOOTSTRAP_SEED,).toBe('editor-fidelity-bench/interval/v1',);
          },
        },),
      ],
    },),

    describe({
      name: pairedPermutationTest.name,
      children: [
        it({
          name: 'GIVES THE BRUTE-FORCE P-VALUE for the effort against one other arm, over entries of one trial and of two, '
            + 'ties and reversals among them',
          fn: async () => {
            /**
             Seven entries: gains, losses, ties and a gain of two.
             */
            const clusters = [
              entryOf({ entryId: 'a', trials: 1, fixed: [1, 0,], },),
              entryOf({ entryId: 'b', trials: 2, fixed: [2, 0,], },),
              entryOf({ entryId: 'c', trials: 1, fixed: [1, 1,], },),
              entryOf({ entryId: 'd', trials: 2, fixed: [1, 2,], },),
              entryOf({ entryId: 'e', trials: 1, fixed: [0, 1,], },),
              entryOf({ entryId: 'f', trials: 2, fixed: [2, 1,], },),
              entryOf({ entryId: 'g', trials: 1, fixed: [1, 0,], },),
            ];
            /**
             What the test says.
             */
            const test = pairedPermutationTest({ clusters, },);
            expect(test.p,).toBeCloseTo(bruteForceP({ clusters, },), 9,);
            expect(test.clusters,).toBe(7,);
            expect(test.trials,).toBe(10,);
          },
        },),

        it({
          name: 'GIVES THE BRUTE-FORCE P-VALUE for the effort against two other arms, the labels permuted among all three',
          fn: async () => {
            /**
             Six entries with three arms each.
             */
            const clusters = [
              entryOf({ entryId: 'a', trials: 2, fixed: [2, 1, 0,], },),
              entryOf({ entryId: 'b', trials: 1, fixed: [1, 0, 0,], },),
              entryOf({ entryId: 'c', trials: 2, fixed: [1, 1, 2,], },),
              entryOf({ entryId: 'd', trials: 1, fixed: [1, 1, 1,], },),
              entryOf({ entryId: 'e', trials: 2, fixed: [2, 0, 1,], },),
              entryOf({ entryId: 'f', trials: 1, fixed: [0, 1, 1,], },),
            ];
            expect(pairedPermutationTest({ clusters, },).p,).toBeCloseTo(bruteForceP({ clusters, },), 9,);
          },
        },),

        it({
          name: 'REACHES THE LEVEL AT SIX WINNING ENTRIES AND NOT AT FIVE: one in 64 is under 0.025 and one in 32 is not',
          fn: async () => {
            /**
             Entries of one trial the effort fixed and the other arm did not.
             */
            const winning = (count: number): readonly PairedCluster[] =>
              Array.from(
                { length: count, },
                function won(
                  _,
                  index,
                ): PairedCluster {
                  return entryOf({ entryId: `cat-${String(index,)}`, trials: 1, fixed: [1, 0,], },);
                },
              )
              ;
            expect(pairedPermutationTest({ clusters: winning(6,), },).p,).toBeCloseTo(1 / 64, 9,);
            expect(pairedPermutationTest({ clusters: winning(6,), },).atMostAlpha,).toBe(true,);
            expect(pairedPermutationTest({ clusters: winning(5,), },).p,).toBeCloseTo(1 / 32, 9,);
            expect(pairedPermutationTest({ clusters: winning(5,), },).atMostAlpha,).toBe(false,);
          },
        },),

        it({
          name: 'DECIDES THE LEVEL ON THE EXACT COUNTS for an entry count whose probability a float cannot hold, and keeps '
            + 'the printed p-value apart from the decision',
          fn: async () => {
            /**
             Sixty entries won: the probability is 2 to the minus 60.
             */
            const clusters = Array.from(
              { length: 60, },
              function won(
                _,
                index,
              ): PairedCluster {
                return entryOf({ entryId: `cat-${String(index,)}`, trials: 1, fixed: [1, 0,], },);
              },
            );
            /**
             What the test says of them.
             */
            const test = pairedPermutationTest({ clusters, },);
            expect(test.atMostAlpha,).toBe(true,);
            expect(test.p,).toBeLessThan(1e-12,);
            expect(test.estimate,).toBe(1,);
          },
        },),

        it({
          name: 'GIVES P OF ONE AND AN ESTIMATE OF ZERO when the arms tie in every entry, and for no entries at all',
          fn: async () => {
            /**
             Every arm fixes the same in every entry.
             */
            const tied = pairedPermutationTest({
              clusters: [
                entryOf({ entryId: 'a', trials: 2, fixed: [1, 1,], },),
                entryOf({ entryId: 'b', trials: 1, fixed: [0, 0,], },),
              ],
            },);
            expect(tied.p,).toBe(1,);
            expect(tied.estimate,).toBe(0,);
            expect(tied.atMostAlpha,).toBe(false,);
            /**
             Nothing shared.
             */
            const empty = pairedPermutationTest({ clusters: [], },);
            expect(empty.p,).toBe(1,);
            expect(empty.trials,).toBe(0,);
            expect(empty.estimate,).toBe(0,);
          },
        },),

        it({
          name: 'ESTIMATES THE MEAN PAIRED DIFFERENCE PER SHARED TRIAL against the mean of the other arms: 3 trials of '
            + 'the effort against 1 and 2 over 4 shared trials is a quarter',
          fn: async () => {
            /**
             Two entries, four trials, and the effort fixes 3 where the others fix 1 and 2.
             */
            const test = pairedPermutationTest({
              clusters: [
                entryOf({ entryId: 'a', trials: 2, fixed: [2, 1, 1,], },),
                entryOf({ entryId: 'b', trials: 2, fixed: [1, 0, 1,], },),
              ],
            },);
            expect(test.estimate,).toBeCloseTo((3 - 1.5) / 4, 12,);
          },
        },),

        it({
          name: 'COUNTS SIXTY-ONE ENTRIES OF TWO ARMS IN A BLINK, so the pool of the bench is tested in the read step',
          fn: async () => {
            /**
             Sixty-one mixed entries.
             */
            const clusters = Array.from(
              { length: 61, },
              function mixed(
                _,
                index,
              ): PairedCluster {
                return entryOf({
                  entryId: `cat-${String(index,)}`,
                  trials: 2,
                  fixed: [index % 3, (index + 1) % 3,],
                },);
              },
            );
            /**
             What the test says.
             */
            const test = pairedPermutationTest({ clusters, },);
            expect(test.p,).toBeGreaterThan(0,);
            expect(test.p,).toBeLessThanOrEqual(1,);
          },
        },),

        it({
          name: 'REFUSES ENTRIES THAT STATE DIFFERENT NUMBERS OF ARMS, and fewer than two',
          fn: async () => {
            expect(function mismatched(): void {
              pairedPermutationTest({
                clusters: [
                  entryOf({ entryId: 'a', trials: 1, fixed: [1, 0,], },),
                  entryOf({ entryId: 'b', trials: 1, fixed: [1, 0, 0,], },),
                ],
              },);
            },).toThrow(PairedShapeError,);
            expect(function lone(): void {
              pairedPermutationTest({ clusters: [entryOf({ entryId: 'a', trials: 1, fixed: [1,], },),], },);
            },).toThrow(PairedShapeError,);
          },
        },),
      ],
    },),

    describe({
      name: clusterBootstrapInterval.name,
      children: [
        it({
          name: 'GIVES ONE INTERVAL FOR ONE SEED, another for another, and brackets the estimate of a mixed set',
          fn: async () => {
            /**
             Thirty entries, the effort ahead in a third of them and behind in a tenth.
             */
            const clusters = Array.from(
              { length: 30, },
              function mixed(
                _,
                index,
              ): PairedCluster {
                return entryOf({
                  entryId: `cat-${String(index,)}`,
                  trials: 2,
                  fixed: ((index % 3) === 0) ? [2, 0,] : ((index % 10) === 1) ? [0, 1,] : [1, 1,],
                },);
              },
            );
            /**
             The estimate of the set.
             */
            const { estimate, } = pairedPermutationTest({ clusters, },);
            /**
             The interval under the bench's seed, twice.
             */
            const first = clusterBootstrapInterval({ clusters, seed: BOOTSTRAP_SEED, replicates: 2_000, },);
            const second = clusterBootstrapInterval({ clusters, seed: BOOTSTRAP_SEED, replicates: 2_000, },);
            expect(second,).toEqual(first,);
            // Nine resamples leave the extremes of nine estimates, which two seeds do not share.
            expect(clusterBootstrapInterval({ clusters, seed: 'another seed', replicates: 9, },),).not.toEqual(
              clusterBootstrapInterval({ clusters, seed: BOOTSTRAP_SEED, replicates: 9, },),
            );
            expect(first.low,).toBeLessThan(estimate,);
            expect(first.high,).toBeGreaterThan(estimate,);
            expect(first.low,).toBeLessThan(first.high,);
          },
        },),

        it({
          name: 'GIVES A POINT FOR A SET WHERE EVERY ENTRY SAYS THE SAME: ten entries all one fix ahead',
          fn: async () => {
            /**
             Ten entries of one trial, the effort ahead in each.
             */
            const clusters = Array.from(
              { length: 10, },
              function ahead(
                _,
                index,
              ): PairedCluster {
                return entryOf({ entryId: `cat-${String(index,)}`, trials: 1, fixed: [1, 0,], },);
              },
            );
            expect(clusterBootstrapInterval({ clusters, seed: BOOTSTRAP_SEED, replicates: 500, },),).toEqual({
              low: 1,
              high: 1,
            },);
          },
        },),

        it({
          name: 'REFUSES ENTRIES THAT STATE DIFFERENT NUMBERS OF ARMS',
          fn: async () => {
            expect(function mismatched(): void {
              clusterBootstrapInterval({
                clusters: [
                  entryOf({ entryId: 'a', trials: 1, fixed: [1, 0,], },),
                  entryOf({ entryId: 'b', trials: 1, fixed: [1,], },),
                ],
                seed: BOOTSTRAP_SEED,
                replicates: 10,
              },);
            },).toThrow(PairedShapeError,);
          },
        },),
      ],
    },),
  ],
},);
