//region Editor fidelity paired test table
// THE TABLE THE SIMULATED BENCH DECIDES THE PAIRED TEST BY. The exact p-value of `editor-fidelity-paired.ts` depends, for two
// arms, only on how large each entry's difference is and on the sum of the differences: the arms' labels are permuted within
// an entry, which flips the sign of its difference. The table of that dependence is built once for each set of magnitudes
// in doubles (a probability a double holds well enough for a simulation; the rule itself decides on whole numbers), and the
// unit test of this module checks the table against the rule on seeded random entries of up to six trials.

/**
 The level each effort is tested at, as a fraction.
 */
const LEVEL = 0.025;

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
 The least sum of entries' differences whose chance of being reached is at most the level, for entries that differ by any
 number of trials.

 @param magnitudes - for each entry, how many trials its two arms differ by, whatever the sign

 @returns The least sum that is at most the level, positive infinity when none is

 @example
 ```ts
 const least = criticalSumOf({ magnitudes: [3, 3, 3, 3, 3, 3,], },);
 ```
 */
export function criticalSumOf({ magnitudes, }: { readonly magnitudes: readonly number[]; },): number {
  return criticalSum({
    ones: magnitudes.filter(function isOne(magnitude,): boolean {
      return magnitude === 1;
    },)
      .length,
    twos: magnitudes.filter(function isTwo(magnitude,): boolean {
      return magnitude === 2;
    },)
      .length,
  },);
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


//endregion Editor fidelity paired test table
