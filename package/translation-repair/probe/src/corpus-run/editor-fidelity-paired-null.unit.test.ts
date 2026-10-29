/**
 Tests for the simulation the paired rule's false-pick chance and power are measured with: the table of critical sums
 against the rule's own exact test, the order of the pool's trials and of a bench that walks them pass by pass, the fixes
 a simulated bench gives, and small seeded benches that must pick nothing under no difference and the better effort under
 a large one. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  pairedPermutationTest,
  unitDraws,
} from '../../dist/final/node/index.mjs';
import {
  type BenchScenario,
  type BenchUnit,
  criticalSum,
  criticalSumOf,
  pickShares,
  poolTrialOrder,
  shownBetter,
  simulatedFixes,
  simulatedPick,
  unitsOf,
} from './editor-fidelity-paired-null.test-fixture.ts';

/**
 Entries whose differences are drawn from a seed: each from minus a reach to plus a reach.

 @param seed - seed

 @param count - how many entries

 @param reach - most an entry may differ by, in trials

 @returns Difference of each entry

 @example
 ```ts
 const differences = differencesFrom({ seed: 'a', count: 20, reach: 2, },);
 ```
 */
function differencesFrom(
  {
    seed,
    count,
    reach,
  }: {
    readonly seed: string;
    readonly count: number;
    readonly reach: number;
  },
): readonly number[] {
  return unitDraws({
    seed,
    count,
  },).map(function differenceOf(draw,): number {
    return Math.floor(draw * ((2 * reach) + 1),) - reach;
  },);
}

/**
 The rule's own exact test read over entries that each hold as many trials as the reach.

 @param differences - for each entry, the effort's fixes minus the default's

 @param trials - trials each entry holds

 @returns Whether the rule's test is at most the level

 @example
 ```ts
 const shown = ruleShows({ differences: [1, 2, 0,], trials: 2, },);
 ```
 */
function ruleShows({ differences, trials, }: { readonly differences: readonly number[]; readonly trials: number; },): boolean {
  return pairedPermutationTest({
    clusters: differences.map(function clusterOf(
      difference,
      at,
    ) {
      return {
        entryId: `cat-${String(at,)}`,
        trials,
        fixed: [Math.max(difference, 0,), Math.max(-difference, 0,),],
      };
    },),
  },).atMostAlpha;
}

/**
 Whether every ask of an entry gave every arm the answer the entry's first ask gave it, for every entry.

 @param units - asks

 @param fixes - for each ask, whether the high, the low and the default effort fixed it

 @returns Whether the trials of each entry read alike

 @example
 ```ts
 const together = entriesAlike({ units, fixes, },);
 ```
 */
function entriesAlike(
  {
    units,
    fixes,
  }: {
    readonly units: readonly BenchUnit[];
    readonly fixes: readonly (readonly boolean[])[];
  },
): boolean {
  return units.every(function sameAsFirst(
    unit,
    at,
  ): boolean {
    /**
     The first ask of the entry.
     */
    const first = units.findIndex(function isFirst({ entryId, },): boolean {
      return entryId === unit.entryId;
    },);
    return JSON.stringify(fixes[at],) === JSON.stringify(fixes[first],);
  },);
}

/**
 The chances of a bench with the arms at rates far apart, so a fix reads the arm it came from.
 */
const FAR_APART: BenchScenario = { high: 0.9, low: 0.5, medium: 0.1, correlation: 0, share: 0, agreement: 0, };

/**
 The committed pool walked twice.

 @returns Asks of two passes, pass zero first

 @example
 ```ts
 const units = twoPasses();
 ```
 */
function twoPasses(): readonly BenchUnit[] {
  return unitsOf({
    trials: poolTrialOrder(),
    passes: 2,
    units: 176,
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
                reach: 2,
              },);
              expect(shownBetter({ differences, },).shown,).toBe(ruleShows({
                differences,
                trials: 2,
              },),);
            }
          },
        },),
      ],
    },),

    describe({
      name: criticalSumOf.name,
      children: [
        it({
          name: 'FINDS THE LEAST SUM FOR ENTRIES OF ANY SIZE: six entries of three trials need thirteen, five cannot',
          fn: async () => {
            expect(criticalSumOf({ magnitudes: [3, 3, 3, 3, 3, 3,], },),).toBe(13,);
            expect(criticalSumOf({ magnitudes: [3, 3, 3, 3, 3,], },),).toBe(Number.POSITIVE_INFINITY,);
            expect(criticalSumOf({ magnitudes: [10, 1, 1, 1, 1, 1,], },),).toBe(14,);
            expect(criticalSumOf({ magnitudes: [2, 2, 2, 2, 2, 2,], },),).toBe(9,);
            expect(criticalSumOf({ magnitudes: [1, 1, 1, 1, 1, 1,], },),).toBe(criticalSum({ ones: 6, twos: 0, },),);
            expect(criticalSumOf({ magnitudes: [], },),).toBe(Number.POSITIVE_INFINITY,);
          },
        },),

        it({
          name: 'AGREES WITH THE RULE\'S EXACT TEST on two hundred seeded sets whose entries differ by up to six trials',
          fn: async () => {
            for (let set = 0; set < 200; set += 1) {
              /**
               The differences of this set, from ten to sixty entries.
               */
              const differences = differencesFrom({
                seed: `wide-${String(set,)}`,
                count: 10 + (set % 51),
                reach: 6,
              },);
              expect(shownBetter({ differences, },).shown,).toBe(ruleShows({
                differences,
                trials: 6,
              },),);
            }
          },
        },),
      ],
    },),

    describe({
      name: poolTrialOrder.name,
      children: [
        it({
          name: 'LISTS THE COMMITTED POOL\'S EIGHTY-EIGHT TRIALS IN ITS ORDER, at most two to an entry, each with its handle',
          fn: async () => {
            /**
             The trials in order.
             */
            const trials = poolTrialOrder();
            expect(trials.length,).toBe(88,);
            expect(new Set(trials.map(function handleOf({ trialId, },): string {
              return trialId;
            },),).size,).toBe(88,);
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
      name: unitsOf.name,
      children: [
        it({
          name: 'WALKS THE TRIALS PASS BY PASS, every trial once and then every trial again, up to the asks named',
          fn: async () => {
            /**
             Three trials of two entries.
             */
            const trials = [
              { entryId: 'tabby', trialId: 'tabby-s1/insertion', kind: 'insertion', },
              { entryId: 'tabby', trialId: 'tabby-s1/alteration', kind: 'alteration', },
              { entryId: 'ginger', trialId: 'ginger-s0/insertion', kind: 'insertion', },
            ];
            /**
             Five asks of two passes.
             */
            const five = unitsOf({
              trials,
              passes: 2,
              units: 5,
            },);
            expect(five.map(function handleOf({ trialId, pass, },): string {
              return `${trialId}#${String(pass,)}`;
            },),).toStrictEqual([
              'tabby-s1/insertion#0',
              'tabby-s1/alteration#0',
              'ginger-s0/insertion#0',
              'tabby-s1/insertion#1',
              'tabby-s1/alteration#1',
            ],);
            expect(unitsOf({
              trials,
              passes: 2,
              units: 99,
            },).length,).toBe(6,);
            expect(unitsOf({
              trials,
              passes: 1,
              units: 99,
            },).length,).toBe(3,);
          },
        },),
      ],
    },),

    describe({
      name: simulatedFixes.name,
      children: [
        it({
          name: 'GIVES EACH ARM ITS OWN RATE: the high, the low and the default effort fix about nine, five and one in ten',
          fn: async () => {
            /**
             The fixes of one pass over the pool.
             */
            const fixes = simulatedFixes({
              units: unitsOf({
                trials: poolTrialOrder(),
                passes: 1,
                units: 88,
              },),
              scenario: FAR_APART,
              seed: 'rates',
            },);
            expect(fixes.length,).toBe(88,);
            /**
             The share of the asks an arm fixed, for the high, the low and the default effort.
             */
            const shares = [0, 1, 2,].map(function shareOf(arm,): number {
              return fixes.filter(function fixed(row,): boolean {
                return row[arm] === true;
              },).length / fixes.length;
            },);
            expect(shares[0],).toBeGreaterThan(0.8,);
            expect(shares[1],).toBeGreaterThan(0.35,);
            expect(shares[1],).toBeLessThan(0.65,);
            expect(shares[2],).toBeLessThan(0.2,);
          },
        },),

        it({
          name: 'GIVES A REPLICATE ITS FIRST PASS\'S ANSWER when the agreement is one, and its own when it is zero',
          fn: async () => {
            /**
             The asks of two passes.
             */
            const units = twoPasses();
            expect(units.length,).toBe(176,);
            /**
             Fixes of a bench whose replicates repeat the first answer.
             */
            const repeated = simulatedFixes({
              units,
              scenario: { ...FAR_APART, high: 0.5, medium: 0.5, agreement: 1, },
              seed: 'agree',
            },);
            for (const [at, unit,] of units.entries()) {
              if (unit.pass === 0)
                continue;
              expect(repeated[at],).toStrictEqual(repeated[at - 88],);
            }
            /**
             Fixes of a bench whose replicates are asked afresh.
             */
            const fresh = simulatedFixes({
              units,
              scenario: { ...FAR_APART, high: 0.5, medium: 0.5, agreement: 0, },
              seed: 'agree',
            },);
            /**
             Cells where the replicate fixed what the first pass fixed.
             */
            const alike = fresh.slice(88,)
              .flatMap(function cellsOf(
                row,
                at,
              ): readonly boolean[] {
                return row.map(function same(
                  cell,
                  arm,
                ): boolean {
                  return cell === fresh[at]?.[arm];
                },);
              },);
            expect(alike.filter(Boolean,).length / alike.length,).toBeGreaterThan(0.35,);
            expect(alike.filter(Boolean,).length / alike.length,).toBeLessThan(0.65,);
          },
        },),

        it({
          name: 'GIVES EACH ARM ITS OWN AGREEMENT when the scenario names one for each: the high and the default effort '
            + 'repeat their first answer and the low effort is asked afresh',
          fn: async () => {
            /**
             The asks of two passes.
             */
            const units = twoPasses();
            /**
             Fixes of a bench in which only the low effort's replicates are asked afresh.
             */
            const fixes = simulatedFixes({
              units,
              scenario: { ...FAR_APART, high: 0.5, low: 0.5, medium: 0.5, agreement: 0.5, armAgreements: [1, 0, 1,], },
              seed: 'own-agreement',
            },);
            /**
             For an arm, how many of the second pass's asks give the answer the first pass gave.
             */
            const repeatedBy = (arm: number,): number => units.filter(function isRepeatSame(
              unit,
              at,
            ): boolean {
              return (unit.pass === 1) && (fixes[at]?.[arm] === fixes[at - 88]?.[arm]);
            },).length;
            expect(repeatedBy(0,),).toBe(88,);
            expect(repeatedBy(2,),).toBe(88,);
            expect(repeatedBy(1,),).toBeLessThan(80,);
            expect(repeatedBy(1,),).toBeGreaterThan(30,);
          },
        },),

        it({
          name: 'MAKES THE TRIALS OF AN ENTRY ALIKE when the correlation is one, and not when it is zero',
          fn: async () => {
            /**
             One pass over the pool.
             */
            const units = unitsOf({
              trials: poolTrialOrder(),
              passes: 1,
              units: 88,
            },);
            /**
             Fixes of a bench whose trials of an entry always read alike.
             */
            const alike = simulatedFixes({
              units,
              scenario: { ...FAR_APART, high: 0.5, low: 0.5, medium: 0.5, correlation: 1, },
              seed: 'alike',
            },);
            /**
             Fixes of a bench whose trials of an entry are strangers.
             */
            const strangers = simulatedFixes({
              units,
              scenario: { ...FAR_APART, high: 0.5, low: 0.5, medium: 0.5, correlation: 0, },
              seed: 'alike',
            },);
            expect(entriesAlike({ units, fixes: alike, },),).toBe(true,);
            expect(entriesAlike({ units, fixes: strangers, },),).toBe(false,);
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
            const units = unitsOf({
              trials: poolTrialOrder(),
              passes: 1,
              units: 88,
            },);
            /**
             High far above the rest, low and default alike.
             */
            const scenario = { high: 1, low: 0.5, medium: 0.5, correlation: 0.5, share: 0, agreement: 0, };
            expect(simulatedPick({ units, scenario, seed: 'a', },),).toBe(simulatedPick({ units, scenario, seed: 'a', },),);
            expect(pickShares({ units, scenario, replicates: 60, seed: 'far', },).high,).toBeGreaterThan(0.95,);
          },
        },),

        it({
          name: 'PICKS NOTHING MORE OFTEN THAN FIVE IN A HUNDRED when the arms are one card, at a rate of one half and '
            + 'trials that always read alike in an entry',
          fn: async () => {
            /**
             The whole pool.
             */
            const units = unitsOf({
              trials: poolTrialOrder(),
              passes: 1,
              units: 88,
            },);
            expect(pickShares({
              units,
              scenario: { high: 0.5, low: 0.5, medium: 0.5, correlation: 1, share: 0, agreement: 0, },
              replicates: 400,
              seed: 'null',
            },).any,).toBeLessThanOrEqual(0.07,);
          },
        },),

        it({
          name: 'PICKS ANOTHER EFFORT MORE OFTEN THAN SEVEN IN A HUNDRED when the default effort is asked afresh and the others '
            + 'repeat their first answer, over three passes at a rate of nine in ten and trials that read alike in an '
            + 'entry, and not when every arm repeats alike',
          fn: async () => {
            /**
             The whole pool walked three times.
             */
            const units = unitsOf({
              trials: poolTrialOrder(),
              passes: 3,
              units: 264,
            },);
            /**
             Every arm fixing nine in ten, the trials of an entry alike.
             */
            const arms = { high: 0.9, low: 0.9, medium: 0.9, correlation: 1, share: 0, agreement: 0, };
            expect(pickShares({
              units,
              scenario: { ...arms, armAgreements: [1, 1, 0,], },
              replicates: 1_000,
              seed: 'unequal',
            },).any,).toBeGreaterThan(0.07,);
            expect(pickShares({
              units,
              scenario: { ...arms, armAgreements: [1, 1, 1,], },
              replicates: 1_000,
              seed: 'unequal',
            },).any,).toBeLessThanOrEqual(0.05,);
          },
        },),

        it({
          name: 'PICKS THE HIGH EFFORT MORE OFTEN FROM TWO PASSES than from one when the replicates are asked afresh, '
            + 'and as often when they repeat the first answer',
          fn: async () => {
            /**
             High a fifth above the rest, trials of an entry strangers.
             */
            const apart = { high: 0.7, low: 0.5, medium: 0.5, correlation: 0, share: 0, };
            /**
             One pass over the pool.
             */
            const once = unitsOf({
              trials: poolTrialOrder(),
              passes: 1,
              units: 88,
            },);
            /**
             How often the high effort is picked from one pass.
             */
            const single = pickShares({
              units: once,
              scenario: { ...apart, agreement: 0, },
              replicates: 150,
              seed: 'passes',
            },).high;
            /**
             How often it is picked from two passes of fresh replicates.
             */
            const double = pickShares({
              units: twoPasses(),
              scenario: { ...apart, agreement: 0, },
              replicates: 150,
              seed: 'passes',
            },).high;
            expect(double,).toBeGreaterThan(single + 0.05,);
            for (let seed = 0; seed < 100; seed += 1) {
              /**
               A bench whose second pass repeats the first.
               */
              const repeats = { ...apart, agreement: 1, };
              expect(simulatedPick({
                units: twoPasses(),
                scenario: repeats,
                seed: `repeat-${String(seed,)}`,
              },),).toBe(simulatedPick({
                units: once,
                scenario: repeats,
                seed: `repeat-${String(seed,)}`,
              },),);
            }
          },
        },),
      ],
    },),
  ],
},);
