import { unitDraws, } from '../../dist/final/node/index.mjs';
import { shownBetter, } from './editor-fidelity-paired-null-table.test-fixture.ts';
import type { BenchUnit, } from './editor-fidelity-paired-null-units.test-fixture.ts';

export {
  criticalSum,
  criticalSumOf,
  shownBetter,
} from './editor-fidelity-paired-null-table.test-fixture.ts';
export {
  type BenchUnit,
  poolTrialOrder,
  type PoolTrial,
  unitsOf,
} from './editor-fidelity-paired-null-units.test-fixture.ts';

//region Editor fidelity paired test under a simulated bench
// WHAT THE PAIRED RULE DOES WHEN THERE IS NOTHING TO FIND AND WHEN THERE IS SOMETHING, simulated once and seeded. The rule
// (`editor-fidelity-decide.ts`) tests the high and the low effort each against the default effort at 0.025, one-sided, with
// the entries as clusters, and picks the one shown better, the larger estimate when both are, the cheaper when the
// estimates are equal. This module plays the bench: three arms walk the leading asks of a pool, each ask fixed with a stated
// chance, and counts what the rule picks.
//
// THE MODEL OF AN ASK. An ask is fixed by an arm when a number in the unit interval, drawn for that arm, is under the arm's
// rate. The number is the ask's own draw shared by all arms with chance `share` (the arms then stray together, as arms asked
// the same trial do), else the arm's own draw. An arm also has a number for each entry, and an ask takes the entry's number
// with chance the square root of `correlation`, so that two trials of an entry read alike, for one arm, with chance
// `correlation` and are strangers otherwise, whatever the number of trials the entry holds. An ask in a later pass over the
// same trial gives the arm's first answer with chance `agreement`, else a number of its own. Every number is uniform, so the
// arm's chance of fixing any ask is its rate. The agreement of an arm with itself on one prompt is not measured; the
// simulation states the figures for several values and does not pick one.

/**
 Draws a trial needs: its own shared number, then for each of three arms the arm's own number, the choice of the shared
 number and the choice of copying the entry's first trial.
 */
const DRAWS_PER_TRIAL = 10;

/**
 Offset of the arm's own number among a trial's draws, after the shared number.
 */
const OWN_AT = 1;

/**
 Offset of the choice of the shared number among a trial's draws.
 */
const SHARE_AT = 4;

/**
 Offset of the choice of copying among a trial's draws.
 */
const COPY_AT = 7;

/**
 The place of the high effort among the arms' draws and counts.
 */
const HIGH = 0;

/**
 The place of the low effort among the arms' draws and counts.
 */
const LOW = 1;

/**
 The place of the default effort among the arms' draws and counts.
 */
const MEDIUM = 2;

/**
 The chances the simulated bench is played with.

 @example
 ```ts
 const scenario: BenchScenario = { high: 0.7, low: 0.5, medium: 0.5, correlation: 0.5, share: 0, agreement: 0.5, };
 ```
 */
export type BenchScenario = {
  /**
   The high effort's chance of fixing a trial.
   */
  readonly high: number;

  /**
   The low effort's chance of fixing a trial.
   */
  readonly low: number;

  /**
   The default effort's chance of fixing a trial.
   */
  readonly medium: number;

  /**
   The chance that the second trial of an entry reads like the first, for one arm.
   */
  readonly correlation: number;

  /**
   The chance that an arm takes the trial's shared number and not its own.
   */
  readonly share: number;

  /**
   The chance that an arm, asked a trial again, gives the answer it gave the first time and not a new one.
   */
  readonly agreement: number;
};

/**
 Which arms fixed each ask of a simulated bench.

 @param units - the asks, in order

 @param scenario - the chances

 @param seed - the seed of this bench

 @returns For each ask, whether the high, the low and the default effort fixed it

 @example
 ```ts
 const fixes = simulatedFixes({ units, scenario, seed: 'a', },);
 ```
 */
export function simulatedFixes(
  {
    units,
    scenario,
    seed,
  }: {
    readonly units: readonly BenchUnit[];
    readonly scenario: BenchScenario;
    readonly seed: string;
  },
): readonly (readonly boolean[])[] {
  return units.map(function nothingFixed(): readonly boolean[] {
    return [
      scenario.high < 0,
      scenario.low < 0,
      seed.length < 0,
    ];
  },);
}

/**
 What the rule picks for one simulated bench over the first asks of the pool.

 @param units - the asks the arms walk, in order

 @param scenario - the chances

 @param seed - the seed of this replicate

 @returns The effort picked

 @example
 ```ts
 const picked = simulatedPick({ units, scenario, seed: 'a', },);
 ```
 */
export function simulatedPick(
  {
    units,
    scenario,
    seed,
  }: {
    readonly units: readonly BenchUnit[];
    readonly scenario: BenchScenario;
    readonly seed: string;
  },
): 'high' | 'low' | 'medium' {
  /**
   The draws of the bench.
   */
  const draws = unitDraws({
    seed,
    count: DRAWS_PER_TRIAL * units.length,
  },);
  /**
   The rates by arm.
   */
  const rates = [
    scenario.high,
    scenario.low,
    scenario.medium,
  ];
  /**
   For each trial and arm, the number that decides it, in the order of the trials.
   */
  const numbers: (readonly number[])[] = [];
  /**
   The first trial of each entry seen so far, by entry.
   */
  const firstOf = new Map<string, number>();
  for (const [at, trial,] of units.entries()) {
    /**
     Where this trial's draws begin.
     */
    const base = at * DRAWS_PER_TRIAL;
    /**
     The first trial of this entry, absent when this is the first.
     */
    const first = firstOf.get(trial.entryId,);
    if (first === undefined)
      firstOf.set(
        trial.entryId,
        at,
      );
    numbers.push(rates.map(function numberFor(
      _unused,
      arm,
    ): number {
      /**
       The arm's number before copying: the shared one or its own.
       */
      const own = ((draws[base + SHARE_AT
        + arm] ?? 0) < scenario.share)
        ? (draws[base] ?? 0)
        : (draws[base + OWN_AT
          + arm] ?? 0);
      return ((first !== undefined) && ((draws[base + COPY_AT
        + arm] ?? 0) < scenario.correlation))
        ? (numbers[first]?.[arm] ?? own)
        : own;
    },),);
  }
  /**
   Fixes by arm in each entry, in the order the entries first appear.
   */
  const entries = new Map<string, readonly number[]>();
  for (const [at, trial,] of units.entries()) {
    entries.set(
      trial.entryId,
      rates.map(function fixesOf(
        rate,
        arm,
      ): number {
        return (entries.get(trial.entryId,)?.[arm] ?? 0) + (((numbers[at]?.[arm] ?? 1) < rate) ? 1 : 0);
      },),
    );
  }
  /**
   The paired differences of each effort against the default.
   */
  const against = [
    HIGH,
    LOW,
  ].map(function differencesOf(arm,): ReturnType<typeof shownBetter> {
    return shownBetter({
      differences: [...entries.values(),].map(function differenceOf(fixes,): number {
        return (fixes[arm] ?? 0) - (fixes[MEDIUM] ?? 0);
      },),
    },);
  },);
  /**
   The reading of the high and the low effort.
   */
  const [high, low,] = against;
  if ((high?.shown === true) && (low?.shown === true)) {
    return ((high.sum) > low.sum) ? 'high' : 'low';
  }
  if (high?.shown === true)
    return 'high';
  return (low?.shown === true) ? 'low' : 'medium';
}

/**
 The share of simulated benches in which the rule picks something other than the default effort, and the share in which it
 picks each.

 @param units - the asks the arms walk

 @param scenario - the chances

 @param replicates - how many benches

 @param seed - the seed of the whole run

 @returns The shares

 @example
 ```ts
 const shares = pickShares({ units, scenario, replicates: 500, seed: 'a', },);
 ```
 */
export function pickShares(
  {
    units,
    scenario,
    replicates,
    seed,
  }: {
    readonly units: readonly BenchUnit[];
    readonly scenario: BenchScenario;
    readonly replicates: number;
    readonly seed: string;
  },
): {
  readonly high: number;
  readonly low: number;
  readonly any: number
} {
  /**
   What each replicate picked.
   */
  const picked = Array.from(
    { length: replicates, },
    function pickAt(
      _unused,
      at,
    ): 'high' | 'low' | 'medium' {
      return simulatedPick({
        units,
        scenario,
        seed: `${seed}/${String(at,)}`,
      },);
    },
  );
  /**
   How many of the replicates picked each effort.
   */
  const counts = {
    high: 0,
    low: 0,
    medium: 0,
  };
  for (const each of picked)
    counts[each] += 1;
  return {
    high: counts.high / replicates,
    low: counts.low / replicates,
    any: 1 - (counts.medium / replicates),
  };
}


//endregion Editor fidelity paired test under a simulated bench
