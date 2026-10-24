import {
  EDITOR_DAMAGE_KINDS,
  EDITOR_FIDELITY_POOL,
  unitDraws,
} from '../../dist/final/node/index.mjs';

//region Editor fidelity paired test under a simulated bench
// WHAT THE PAIRED RULE DOES WHEN THERE IS NOTHING TO FIND AND WHEN THERE IS SOMETHING, simulated once and seeded. The rule
// (`editor-fidelity-decide.ts`) tests the high and the low effort each against the default effort at 0.025, one-sided, with
// the entries as clusters, and picks the one shown better, the larger estimate when both are, the cheaper when the
// estimates are equal. This module plays the bench: three arms walk the first n trials of the committed pool in its order,
// each trial fixed with a stated chance, and counts what the rule picks.
//
// THE MODEL OF A TRIAL. A trial is fixed by an arm when a number in the unit interval, drawn for that arm, is under the arm's
// rate. The number is the trial's own draw shared by all arms with chance `share` (the arms then stray together, as arms
// asked the same trial do), else the arm's own draw. The second trial of an entry takes the first trial's number for that
// arm with chance `correlation` (the two then read alike), else a number of its own. Every number is uniform, so the arm's
// chance of fixing any trial is its rate, and the correlation of two trials of an entry is the stated one.
//
// THE TEST IS DECIDED BY A TABLE. The exact p-value of `editor-fidelity-paired.ts` depends, for two arms, only on how many
// entries differ by one trial, how many by two, and the sum of the differences. The table of that dependence is built once
// for each pair of counts in doubles (a probability a double holds well enough for a simulation; the rule itself decides
// on whole numbers), and the unit test of this module checks the table against the rule on seeded random entries.

/**
 The level each effort is tested at, as a fraction.
 */
const LEVEL = 0.025;

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
 One trial of the pool in its order: the entry and the kind of defect.

 @example
 ```ts
 const trial: PoolTrial = { entryId: 'tabby', kind: 'insertion', };
 ```
 */
type PoolTrial = {
  /**
   The entry the trial comes from.
   */
  readonly entryId: string;

  /**
   The kind of defect.
   */
  readonly kind: string;
};

/**
 The committed pool's trials in the order every arm walks them.

 @returns The trials, a reference's damages in the order the specification lists them

 @example
 ```ts
 const order = poolTrialOrder();
 ```
 */
export function poolTrialOrder(): readonly PoolTrial[] {
  return EDITOR_FIDELITY_POOL.flatMap(function trialsOf(spec,): readonly PoolTrial[] {
    return spec.damages
      .filter(function builds(damage,): boolean {
        return EDITOR_DAMAGE_KINDS.some(function is(kind,): boolean {
          return kind === damage.kind;
        },);
      },)
      .map(function trialOf(damage,): PoolTrial {
        return {
          entryId: spec.entryId,
          kind: damage.kind,
        };
      },);
  },);
}

/**
 The chance that a sum of independent signed magnitudes reaches each value, and the least sum whose chance of being reached
 is at most the level.

 @param ones - entries that differ by one trial

 @param twos - entries that differ by two trials

 @returns The least sum that is at most the level, positive infinity when none is

 @example
 ```ts
 const least = criticalSum({ ones: 6, twos: 0, },);
 ```
 */
export function criticalSum({
  ones,
  twos,
}: {
  readonly ones: number;
  readonly twos: number
},): number {
  /**
   The largest sum there can be.
   */
  const top = ones + (2 * twos);
  /**
   The chance of each sum, from the least up, the least being minus the top.
   */
  const chances = [
    ...Array.from(
      { length: ones, },
      function one(): number {
        return 1;
      },
    ),
    ...Array.from(
      { length: twos, },
      function two(): number {
        return 2;
      },
    ),
  ].reduce(
    function spread(
      held: readonly number[],
      size: number,
    ): readonly number[] {
      return Array.from(
        { length: (2 * top) + 1, },
        function sumAt(
          _unused,
          at,
        ): number {
          return ((held[at - size] ?? 0) + (held[at + size] ?? 0)) / 2;
        },
      );
    },
    Array.from(
      { length: (2 * top) + 1, },
      function startAt(
        _unused,
        at,
      ): number {
        return (at === top) ? 1 : 0;
      },
    ),
  );
  /**
   The chance of reaching each sum, from the top down.
   */
  const reach = chances.map(function reachFrom(
    _unused,
    from,
  ): number {
    return chances.slice(from,)
      .reduce(
      function add(
        total: number,
        chance,
      ): number {
        return total + chance;
      },
      0,
      );
  },);
  /**
   Where the chance first falls to the level.
   */
  const at = reach.findIndex(function atMost(chance,): boolean {
    return chance <= LEVEL;
  },);
  return (at === (-1)) ? Number.POSITIVE_INFINITY : (at - top);
}

/**
 Memo of the critical sums by the counts they were made for.
 */
const CRITICAL = new Map<string, number>();

/**
 The critical sum of the entries' differences, from the table, made once for each pair of counts.

 @param ones - entries that differ by one trial

 @param twos - entries that differ by two trials

 @returns The least sum at most the level

 @example
 ```ts
 const least = criticalSumFor({ ones: 6, twos: 0, },);
 ```
 */
function criticalSumFor({
  ones,
  twos,
}: {
  readonly ones: number;
  readonly twos: number
},): number {
  /**
   The memo key.
   */
  const key = `${String(ones,)}/${String(twos,)}`;
  /**
   The sum made before, if any.
   */
  const made = CRITICAL.get(key,);
  if (made !== undefined)
    return made;
  /**
   The sum made now.
   */
  const fresh = criticalSum({
    ones,
    twos,
  },);
  CRITICAL.set(
    key,
    fresh,
  );
  return fresh;
}

/**
 Whether the first arm is shown better than the second over entries, by the table.

 @param differences - for each entry, the first arm's fixes minus the second's

 @returns Whether the one-sided exact test is at most the level, and the sum of the differences

 @example
 ```ts
 const { shown, sum, } = shownBetter({ differences: [1, 1, 0, 2, -1,], },);
 ```
 */
export function shownBetter(
  { differences, }: { readonly differences: readonly number[]; },
): {
  readonly shown: boolean;
  readonly sum: number
} {
  /**
   Entries that differ by one trial.
   */
  const ones = differences.filter(function isOne(difference,): boolean {
    return Math.abs(difference,) === 1;
  },)
    .length;
  /**
   Entries that differ by two trials.
   */
  const twos = differences.filter(function isTwo(difference,): boolean {
    return Math.abs(difference,) === 2;
  },)
    .length;
  /**
   The sum of the differences.
   */
  const sum = differences.reduce(
    function add(
      total: number,
      difference,
    ): number {
      return total + difference;
    },
    0,
  );
  return {
    shown: sum >= criticalSumFor({
      ones,
      twos,
    },),
    sum,
  };
}

/**
 The chances the simulated bench is played with.
 */
type BenchScenario = {
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
};

/**
 What the rule picks for one simulated bench over the first trials of the pool.

 @param trials - the trials the arms walk, in order

 @param scenario - the chances

 @param seed - the seed of this replicate

 @returns The effort picked

 @example
 ```ts
 const picked = simulatedPick({ trials, scenario, seed: 'a', },);
 ```
 */
export function simulatedPick(
  {
    trials,
    scenario,
    seed,
  }: {
    readonly trials: readonly PoolTrial[];
    readonly scenario: BenchScenario;
    readonly seed: string;
  },
): 'high' | 'low' | 'medium' {
  /**
   The draws of the bench.
   */
  const draws = unitDraws({
    seed,
    count: DRAWS_PER_TRIAL * trials.length,
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
  for (const [at, trial,] of trials.entries()) {
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
  for (const [at, trial,] of trials.entries()) {
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

 @param trials - the trials the arms walk

 @param scenario - the chances

 @param replicates - how many benches

 @param seed - the seed of the whole run

 @returns The shares

 @example
 ```ts
 const shares = pickShares({ trials, scenario, replicates: 500, seed: 'a', },);
 ```
 */
export function pickShares(
  {
    trials,
    scenario,
    replicates,
    seed,
  }: {
    readonly trials: readonly PoolTrial[];
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
        trials,
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
