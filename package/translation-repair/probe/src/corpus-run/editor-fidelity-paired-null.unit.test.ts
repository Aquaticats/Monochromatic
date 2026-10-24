/**
 Tests for the simulation the paired rule's false-pick chance and power are measured with: the table of critical sums
 against the rule's own exact test, the order of the pool's trials, and small seeded benches that must pick nothing under no
 difference and the better effort under a large one. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  pairedPermutationTest,
  unitDraws,
} from '../../dist/final/node/index.mjs';
import {
  criticalSum,
  pickShares,
  poolTrialOrder,
  shownBetter,
  simulatedPick,
} from './editor-fidelity-paired-null.test-fixture.ts';

/**
 Entries whose differences are drawn from a seed: each from minus two to plus two.

 @param seed - the seed
 @param count - how many entries

 @returns The difference of each entry

 @example
 ```ts
 const differences = differencesFrom({ seed: 'a', count: 20, },);
 ```
 */
function differencesFrom({ seed, count, }: { readonly seed: string; readonly count: number; },): readonly number[] {
  return unitDraws({
    seed,
    count,
  },).map(function differenceOf(draw,): number {
    return Math.floor(draw * 5,) - 2;
  },);
}

await describe({
  name: 'editor fidelity paired test under a simulated bench',
  children: [
    describe({
      name: criticalSum.name,
      children: [
        it({
          name: 'FINDS SIX AS THE FEWEST WINNING ENTRIES THAT REACH THE LEVEL: five cannot, whatever their sum',
          fn: async () => {
            expect(criticalSum({ ones: 5, twos: 0, },),).toBe(Number.POSITIVE_INFINITY,);
            expect(criticalSum({ ones: 6, twos: 0, },),).toBe(5,);
            expect(criticalSum({ ones: 0, twos: 0, },),).toBe(Number.POSITIVE_INFINITY,);
          },
        },),

        it({
          name: 'AGREES WITH THE RULE\'S EXACT TEST on two hundred seeded sets of entries, shown or not shown',
          fn: async () => {
            for (let set = 0; set < 200; set += 1) {
              /**
               The differences of this set, from ten to sixty entries.
               */
              const differences = differencesFrom({
                seed: `set-${String(set,)}`,
                count: 10 + (set % 51),
              },);
              /**
               The same entries as the rule's test reads them.
               */
              const test = pairedPermutationTest({
                clusters: differences.map(function clusterOf(
                  difference,
                  at,
                ) {
                  return {
                    entryId: `cat-${String(at,)}`,
                    trials: 2,
                    fixed: [Math.max(difference, 0,), Math.max(-difference, 0,),],
                  };
                },),
              },);
              expect(shownBetter({ differences, },).shown,).toBe(test.atMostAlpha,);
            }
          },
        },),
      ],
    },),

    describe({
      name: poolTrialOrder.name,
      children: [
        it({
          name: 'LISTS THE COMMITTED POOL\'S EIGHTY-EIGHT TRIALS IN ITS ORDER, at most two to an entry',
          fn: async () => {
            /**
             The trials in order.
             */
            const trials = poolTrialOrder();
            expect(trials.length,).toBe(88,);
            expect(Map.groupBy(
              trials,
              function entryOf({ entryId, },): string {
                return entryId;
              },
            ).size,).toBe(53,);
            for (const group of Map.groupBy(
              trials,
              function entryOf({ entryId, },): string {
                return entryId;
              },
            ).values())
              expect(group.length,).toBeLessThanOrEqual(2,);
          },
        },),
      ],
    },),

    describe({
      name: simulatedPick.name,
      children: [
        it({
          name: 'PICKS THE SAME EFFORT FOR THE SAME SEED, and the better effort when it is far better',
          fn: async () => {
            /**
             The whole pool.
             */
            const trials = poolTrialOrder();
            /**
             High far above the rest, low and default alike.
             */
            const scenario = { high: 1, low: 0.5, medium: 0.5, correlation: 0.5, share: 0, };
            expect(simulatedPick({ trials, scenario, seed: 'a', },),).toBe(simulatedPick({ trials, scenario, seed: 'a', },),);
            expect(pickShares({ trials, scenario, replicates: 60, seed: 'far', },).high,).toBeGreaterThan(0.95,);
          },
        },),

        it({
          name: 'PICKS NOTHING MORE OFTEN THAN FIVE IN A HUNDRED when the arms are one card, at a rate of one half and '
            + 'trials that always read alike in an entry',
          fn: async () => {
            /**
             The whole pool.
             */
            const trials = poolTrialOrder();
            expect(pickShares({
              trials,
              scenario: { high: 0.5, low: 0.5, medium: 0.5, correlation: 1, share: 0, },
              replicates: 400,
              seed: 'null',
            },).any,).toBeLessThanOrEqual(0.07,);
          },
        },),
      ],
    },),
  ],
},);
