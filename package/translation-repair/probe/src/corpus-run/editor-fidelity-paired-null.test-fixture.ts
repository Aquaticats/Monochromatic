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
//
// THE TEST IS DECIDED BY A TABLE (`editor-fidelity-paired-null-table.test-fixture.ts`), built from the magnitudes of the
// entries' differences, and the unit test checks it against the rule's own exact test.

/**
 Arms the bench plays, the high, the low and the default effort; each entry draws one number for each before any ask.
 */
const ARMS = 3;

/**
 Draws an ask needs: its own shared number, then for each of three arms the arm's own number, the choice of the shared
 number, the choice of the entry's number, the choice of repeating the first answer and a fresh number for a later pass.
 */
const DRAWS_PER_UNIT = 16;

/**
 Offset of the arm's own number among an ask's draws, after the shared number.
 */
const OWN_AT = 1;

/**
 Offset of the choice of the shared number among an ask's draws.
 */
const SHARE_AT = 4;

/**
 Offset of the choice of the entry's number among an ask's draws.
 */
const ANCHOR_AT = 7;

/**
 Offset of the choice of repeating the first answer among an ask's draws.
 */
const REPEAT_AT = 10;

/**
 Offset of the fresh number of a later pass among an ask's draws.
 */
const FRESH_AT = 13;

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
 Every arm's place, the high effort first.
 */
const ALL_ARMS = [
  HIGH,
  LOW,
  MEDIUM,
];

/**
 Reads one draw of an ask.

 @param draws - draws of the bench

 @param base - where the ask's draws begin

 @param offset - which kind of draw, among the ask's

 @param arm - place of the arm among the arms

 @returns Draw, number in the unit interval

 @example
 ```ts
 const chosen = drawOf({ draws, base: 160, offset: SHARE_AT, arm: HIGH, },);
 ```
 */
function drawOf(
  {
    draws,
    base,
    offset,
    arm,
  }: {
    readonly draws: readonly number[];
    readonly base: number;
    readonly offset: number;
    readonly arm: number;
  },
): number {
  return draws[base + (offset + arm)] ?? 0;
}

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
   The chance that two trials of an entry read alike, for one arm.
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

  /**
   The agreement of each arm, the high, the low and the default effort in that order, in place of `agreement` for arms
   that repeat themselves unlike one another.
   */
  readonly armAgreements?: readonly [
    number,
    number,
    number
  ];
};

/**
 The number that decides one arm's answer to one ask, before any later pass is considered.

 @param draws - draws of the bench

 @param base - where the ask's draws begin

 @param entryAt - where the entry's draws begin

 @param arm - place of the arm among the arms

 @param scenario - chances

 @returns Number in the unit interval

 @example
 ```ts
 const first = firstNumber({ draws, base: 160, entryAt: 0, arm: HIGH, scenario, },);
 ```
 */
function firstNumber(
  {
    draws,
    base,
    entryAt,
    arm,
    scenario,
  }: {
    readonly draws: readonly number[];
    readonly base: number;
    readonly entryAt: number;
    readonly arm: number;
    readonly scenario: BenchScenario;
  },
): number {
  /**
   The ask's number before the entry is considered: the shared one or the arm's own.
   */
  const own = (drawOf({
    draws,
    base,
    offset: SHARE_AT,
    arm,
  },) < scenario.share)
    ? (draws[base] ?? 0)
    : drawOf({
      draws,
      base,
      offset: OWN_AT,
      arm,
    },);
  return (drawOf({
    draws,
    base,
    offset: ANCHOR_AT,
    arm,
  },) < Math.sqrt(scenario.correlation,))
    ? (draws[entryAt + arm] ?? own)
    : own;
}

/**
 Which arms fixed each ask of a simulated bench.

 @param units - asks, in order

 @param scenario - chances

 @param seed - seed of this bench

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
  /**
   The entries, in the order they first appear.
   */
  const entries = [...new Set(units.map(function entryOf({ entryId, },): string {
    return entryId;
  },),),];
  /**
   The draws of the bench: the entries' numbers, then the draws of each ask.
   */
  const draws = unitDraws({
    seed,
    count: (ARMS * entries.length) + (DRAWS_PER_UNIT * units.length),
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
   The numbers of each ask by arm, in the order of the asks.
   */
  const numbers: (readonly number[])[] = [];
  /**
   The place of the first ask of each trial.
   */
  const firstAskOf = new Map<string, number>();
  for (const [at, unit,] of units.entries()) {
    /**
     Where this ask's draws begin.
     */
    const base = (ARMS * entries.length) + (at * DRAWS_PER_UNIT);
    /**
     The ask of this trial in the first pass, absent when this is it.
     */
    const first = firstAskOf.get(unit.trialId,);
    if (first === undefined)
      firstAskOf.set(
        unit.trialId,
        at,
      );
    numbers.push(rates.map(function numberFor(
      _unused,
      arm,
    ): number {
      if (first === undefined) {
        return firstNumber({
          draws,
          base,
          entryAt: ARMS * entries.indexOf(unit.entryId,),
          arm,
          scenario,
        },);
      }
      // A later ask of the trial repeats the arm's first answer with the stated agreement, else gives a number of its own.
      return (drawOf({
        draws,
        base,
        offset: REPEAT_AT,
        arm,
      },) < (scenario.armAgreements?.[arm] ?? scenario.agreement))
        ? (numbers[first]?.[arm] ?? 0)
        : drawOf({
          draws,
          base,
          offset: FRESH_AT,
          arm,
        },);
    },),);
  }
  return numbers.map(function fixedBy(row,): readonly boolean[] {
    return row.map(function underRate(
      number,
      arm,
    ): boolean {
      return number < (rates[arm] ?? 0);
    },);
  },);
}

/**
 What the rule picks for one simulated bench over the first asks of the pool.

 @param units - asks the arms walk, in order

 @param scenario - chances

 @param seed - seed of this replicate

 @returns Effort picked

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
   Fixes by arm for each ask.
   */
  const fixes = simulatedFixes({
    units,
    scenario,
    seed,
  },);
  /**
   Fixes by arm in each entry, in the order the entries first appear.
   */
  const entries = new Map<string, readonly number[]>();
  for (const [at, unit,] of units.entries()) {
    entries.set(
      unit.entryId,
      ALL_ARMS.map(function fixesOf(arm,): number {
        return (entries.get(unit.entryId,)?.[arm] ?? 0) + ((fixes[at]?.[arm] === true) ? 1 : 0);
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
      differences: [...entries.values(),].map(function differenceOf(counts,): number {
        return (counts[arm] ?? 0) - (counts[MEDIUM] ?? 0);
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

 @param units - asks the arms walk

 @param scenario - chances

 @param replicates - how many benches

 @param seed - seed of the whole run

 @returns Shares

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
