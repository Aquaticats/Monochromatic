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
 Convolves the chances of the sums so far with one more entry, which adds or takes its magnitude with equal chance.

 @param held - chance of each sum, least sum first, middle being zero

 @param magnitude - how many trials the entry's two arms differ by

 @returns Chances once the entry is in

 @example
 ```ts
 const next = withEntry({ held: [0, 1, 0,], magnitude: 1, },);
 ```
 */
function withEntry(
  {
    held,
    magnitude,
  }: {
    readonly held: readonly number[];
    readonly magnitude: number;
  },
): readonly number[] {
  return held.map(function sumAt(
    _unused,
    at,
  ): number {
    return ((held[at - magnitude] ?? 0) + (held[at + magnitude] ?? 0)) / 2;
  },);
}

/**
 The least sum of entries' differences whose chance of being reached is at most the level, for entries that differ by any
 number of trials.

 @param magnitudes - for each entry, how many trials its two arms differ by, whatever the sign

 @returns Least sum that is at most the level, positive infinity when none is

 @example
 ```ts
 const least = criticalSumOf({ magnitudes: [3, 3, 3, 3, 3, 3,], },);
 ```
 */
export function criticalSumOf({ magnitudes, }: { readonly magnitudes: readonly number[]; },): number {
  /**
   The largest sum there can be.
   */
  const top = magnitudes.reduce(
    function add(
      total: number,
      magnitude,
    ): number {
      return total + magnitude;
    },
    0,
  );
  /**
   The chance of each sum, from the least up, the least being minus the top.
   */
  const chances = magnitudes.reduce(
    function spread(
      held: readonly number[],
      magnitude: number,
    ): readonly number[] {
      return withEntry({
        held,
        magnitude,
      },);
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
   The running chance of reaching the sum at the place the walk from the top has come to, and the least place whose chance
   is at most the level, minus one while none is.
   */
  const walk = {
    reached: 0,
    at: -1,
  };
  for (const from of Array.from(
    { length: chances.length, },
    function downFrom(
      _unused,
      step,
    ): number {
      return chances.length - (step + 1);
    },
  )) {
    walk.reached += chances[from] ?? 0;
    if (walk.reached > LEVEL)
      break;
    walk.at = from;
  }
  return (walk.at === (-1)) ? Number.POSITIVE_INFINITY : (walk.at - top);
}

/**
 The least sum of entries that differ by one trial or by two.

 @param ones - entries that differ by one trial

 @param twos - entries that differ by two trials

 @returns Least sum that is at most the level, positive infinity when none is

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
  return criticalSumOf({
    magnitudes: [
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
    ],
  },);
}

/**
 Memo of the critical sums by the sorted magnitudes they were made for.
 */
const CRITICAL = new Map<string, number>();

/**
 The critical sum of the entries' differences, from the table, made once for each set of magnitudes.

 @param magnitudes - for each entry that differs, how many trials its two arms differ by

 @returns Least sum at most the level

 @example
 ```ts
 const least = criticalSumFor({ magnitudes: [1, 1, 2,], },);
 ```
 */
function criticalSumFor({ magnitudes, }: { readonly magnitudes: readonly number[]; },): number {
  /**
   The memo key.
   */
  const key = magnitudes.toSorted(function ascending(
    left,
    right,
  ): number {
    return left - right;
  },)
    .join(',',);
  /**
   The sum made before, if any.
   */
  const made = CRITICAL.get(key,);
  if (made !== undefined)
    return made;
  /**
   The sum made now.
   */
  const fresh = criticalSumOf({ magnitudes, },);
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
      magnitudes: differences.map(function sizeOf(difference,): number {
        return Math.abs(difference,);
      },)
        .filter(function differs(magnitude,): boolean {
          return magnitude > 0;
        },),
    },),
    sum,
  };
}

//endregion Editor fidelity paired test table
