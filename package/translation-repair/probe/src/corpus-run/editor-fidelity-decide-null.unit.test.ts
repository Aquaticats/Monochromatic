/**
 Tests for the exact enumeration that fixes the decision rule's floor: the chance the rule calls an effort different
 when none is, at every fix rate from 0.30 to 0.95, for trials clustered by entry as the frozen pool draws them.

 The enumeration is checked against a plain loop over every outcome of a pool small enough to list, and the floor of
 twelve trials is pinned as the smallest that holds the chance under 0.05 at every rate and correlation, with eleven
 shown not to. Fixtures are invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  armCountChances,
  EDITOR_FIDELITY_POOL,
  falseCallRate,
  NULL_CALL_LIMIT,
  NULL_CORRELATIONS,
  NULL_RATES,
  POOL_CLUSTERS,
  POOLED_FLOOR_TRIALS,
  smallestFloor,
  worstFalseCall,
} from '../../dist/final/node/index.mjs';

/**
 The chance of each count of one entry's trials, written out outcome by outcome and not by convolution.

 @param kinds - how many trials the entry holds, one or two
 @param successRate - chance one trial is fixed
 @param correlation - within-entry correlation

 @returns For each outcome of the entry's trials, its count and its chance

 @example
 ```ts
 const outcomes = entryOutcomes({ kinds: 2, successRate: 0.4, correlation: 0.5, },);
 ```
 */
function entryOutcomes(
  {
    kinds,
    successRate,
    correlation,
  }: {
    readonly kinds: 1 | 2;
    readonly successRate: number;
    readonly correlation: number;
  },
): readonly (readonly [number, number])[] {
  /**
   Chance a trial is missed.
   */
  const miss = 1 - successRate;
  if (kinds === 1) {
    return [
      [0, miss,],
      [1, successRate,],
    ];
  }
  return [
    [0, (miss * miss) + (correlation * successRate * miss),],
    [1, successRate * miss * (1 - correlation),],
    [1, successRate * miss * (1 - correlation),],
    [2, (successRate * successRate) + (correlation * successRate * miss),],
  ];
}

/**
 The chance of each count of a pool of one paired entry and one single entry, by listing every outcome.

 @param successRate - chance one trial is fixed
 @param correlation - within-entry correlation

 @returns The chance of each count from zero to three

 @example
 ```ts
 const chances = listedChances({ successRate: 0.4, correlation: 0.5, },);
 ```
 */
function listedChances(
  {
    successRate,
    correlation,
  }: {
    readonly successRate: number;
    readonly correlation: number;
  },
): readonly number[] {
  /**
   The chance of each count, filled outcome by outcome.
   */
  const chances = [0, 0, 0, 0,];
  for (const [pairCount, pairChance,] of entryOutcomes({ kinds: 2, successRate, correlation, },)) {
    for (const [singleCount, singleChance,] of entryOutcomes({ kinds: 1, successRate, correlation, },))
      chances[pairCount + singleCount] = (chances[pairCount + singleCount] ?? 0) + (pairChance * singleChance);
  }
  return chances;
}

/**
 The chance of calling at least one of two efforts different, by looping over every count of the four arms and applying
 the rule with plain comparisons of means.

 @param chances - the chance of each count of an arm
 @param floorTrials - the least gap that can count

 @returns The chance, summed over every outcome

 @example
 ```ts
 const rate = listedFalseCall({ chances: [0.25, 0.5, 0.25,], floorTrials: 1, },);
 ```
 */
function listedFalseCall(
  {
    chances,
    floorTrials,
  }: {
    readonly chances: readonly number[];
    readonly floorTrials: number;
  },
): number {
  /**
   Running sum of the chance of the outcomes where an effort is called.
   */
  let called = 0;
  for (const [low, lowChance,] of chances.entries()) {
    for (const [high, highChance,] of chances.entries()) {
      for (const [firstMedium, firstChance,] of chances.entries()) {
        for (const [secondMedium, secondChance,] of chances.entries()) {
          /**
           Threshold in trials: the floor, or the mediums' own difference when larger.
           */
          const threshold = Math.max(floorTrials, Math.abs(firstMedium - secondMedium,),);
          /**
           Mean of the two mediums.
           */
          const mean = (firstMedium + secondMedium) / 2;
          if ((Math.abs(low - mean,) >= threshold) || (Math.abs(high - mean,) >= threshold))
            called += lowChance * highChance * firstChance * secondChance;
        }
      }
    }
  }
  return called;
}

await describe({
  name: 'editor fidelity decision rule under no difference',
  children: [
    describe({
      name: armCountChances.name,
      children: [
        it({
          name: 'ADDS UP TO ONE AND AGREES WITH A LISTING OF EVERY OUTCOME for a paired and a single entry, at '
            + 'correlations from independent to always alike',
          fn: async () => {
            for (const correlation of NULL_CORRELATIONS) {
              /**
               The enumeration over one paired and one single entry.
               */
              const chances = armCountChances({
                clusters: { both: 1, insertionOnly: 1, alterationOnly: 0, },
                successRate: 0.4,
                correlation,
              },);
              /**
               The same by listing outcomes.
               */
              const listed = listedChances({ successRate: 0.4, correlation, },);
              expect(chances.length,).toBe(4,);
              expect(chances.reduce((sum, each,) => sum + each, 0,),).toBeCloseTo(1, 12,);
              chances.forEach((each, count,) => {
                expect(each,).toBeCloseTo(listed[count] ?? Number.NaN, 12,);
              },);
            }
          },
        },),

        it({
          name: 'MAKES AN ARM OF TWO TRIALS PER ENTRY ALL OR NOTHING when the trials of an entry always read alike',
          fn: async () => {
            /**
             Four paired entries that always read alike: counts are even.
             */
            const chances = armCountChances({
              clusters: { both: 4, insertionOnly: 0, alterationOnly: 0, },
              successRate: 0.5,
              correlation: 1,
            },);
            expect(chances.length,).toBe(9,);
            expect(chances.filter((_each, count,) => count % 2 === 1,).every((each,) => each < 1e-12,),).toBe(true,);
            expect(chances[0],).toBeCloseTo(1 / 16, 12,);
            expect(chances[8],).toBeCloseTo(1 / 16, 12,);
          },
        },),
      ],
    },),

    describe({
      name: falseCallRate.name,
      children: [
        it({
          name: 'AGREES WITH A LOOP OVER EVERY OUTCOME OF THE FOUR ARMS for a small pool, at several floors and chances',
          fn: async () => {
            for (const floorTrials of [0, 1, 2, 3,]) {
              for (const successRate of [0.3, 0.5, 0.9,]) {
                for (const correlation of [0, 0.5, 1,]) {
                  expect(falseCallRate({
                    clusters: { both: 1, insertionOnly: 1, alterationOnly: 0, },
                    successRate,
                    correlation,
                    floorTrials,
                  },),).toBeCloseTo(listedFalseCall({
                    chances: listedChances({ successRate, correlation, },),
                    floorTrials,
                  },), 12,);
                }
              }
            }
          },
        },),

        it({
          name: 'CALLS LESS AS THE FLOOR GROWS, and never with a floor above every possible gap of the pool\'s forty '
            + 'trials',
          fn: async () => {
            /**
             The rate at each floor from none to the pool's trials plus one.
             */
            const rates = Array.from({ length: 42, }, (_unused, floorTrials,) => falseCallRate({
              clusters: POOL_CLUSTERS,
              successRate: 0.5,
              correlation: 1,
              floorTrials,
            },),);
            expect(rates[0],).toBeGreaterThan(0.5,);
            rates.slice(1,).forEach((rate, at,) => {
              expect(rate,).toBeLessThanOrEqual((rates[at] ?? 0) + 1e-12,);
            },);
            expect(rates[41],).toBeCloseTo(0, 12,);
          },
        },),
      ],
    },),

    describe({
      name: 'the pool',
      children: [
        it({
          name: 'IS THE FROZEN POOL: twenty-four entries, sixteen with both kinds, six with an insertion only and two with '
            + 'an alteration only, read off the committed specifications and not typed in',
          fn: async () => {
            /**
             The kinds of defect with a deterministic fix that each entry's specifications hold.
             */
            const kindsByEntry = new Map<string, Set<string>>();
            for (const spec of EDITOR_FIDELITY_POOL) {
              /**
               This entry's kinds so far.
               */
              const kinds = kindsByEntry.get(spec.entryId,) ?? new Set<string>();
              for (const damage of spec.damages) {
                if (damage.kind !== 'deletion')
                  kinds.add(damage.kind,);
              }
              kindsByEntry.set(spec.entryId, kinds,);
            }
            /**
             The entries by how many kinds and which.
             */
            const split = {
              both: 0,
              insertionOnly: 0,
              alterationOnly: 0,
            };
            for (const kinds of kindsByEntry.values()) {
              if (kinds.size === 2)
                split.both += 1;
              else if (kinds.has('insertion',))
                split.insertionOnly += 1;
              else
                split.alterationOnly += 1;
            }
            expect(split,).toStrictEqual(POOL_CLUSTERS,);
            expect(kindsByEntry.size,).toBe(24,);
          },
        },),
      ],
    },),

    describe({
      name: smallestFloor.name,
      children: [
        it({
          name: 'FINDS TWELVE TRIALS as the smallest floor that keeps the chance of calling an effort different under 0.05 '
            + 'at every fix rate from 0.30 to 0.95 and every correlation, and shows eleven does not, so the constant '
            + 'the rule prints is the enumeration\'s and not a guess',
          fn: async () => {
            expect(NULL_RATES[0],).toBe(0.3,);
            expect(NULL_RATES.at(-1,),).toBe(0.95,);
            expect(NULL_CALL_LIMIT,).toBe(0.05,);
            expect(smallestFloor({ clusters: POOL_CLUSTERS, },),).toBe(12,);
            expect(POOLED_FLOOR_TRIALS,).toBe(12,);
            /**
             The worst case at the printed floor and one below.
             */
            const atFloor = worstFalseCall({ clusters: POOL_CLUSTERS, floorTrials: POOLED_FLOOR_TRIALS, },);
            const below = worstFalseCall({ clusters: POOL_CLUSTERS, floorTrials: POOLED_FLOOR_TRIALS - 1, },);
            expect(atFloor.rate,).toBeLessThanOrEqual(NULL_CALL_LIMIT,);
            expect(below.rate,).toBeGreaterThan(NULL_CALL_LIMIT,);
            expect(atFloor.correlation,).toBe(1,);
            expect(atFloor.successRate,).toBe(0.5,);
          },
        },),
      ],
    },),
  ],
},);
