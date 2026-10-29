//region Editor fidelity arm repeat test
// THE EXACT TEST OF THE REPEAT WARNING. The warning asks whether the default effort repeats itself less than another arm,
// that is, whether its count of trials that read alike in every pass is low against the other arm's. The two counts are
// set out as a table of trials that read alike and trials that did not, for each of the two arms, and the one-sided
// Fisher exact test gives the chance, with the totals held, of a count as low as the default effort's. The chance is a
// sum of hypergeometric terms, summed in whole numbers so that a probability a float cannot hold is still decided at the
// level exactly, and the level is decided by cross-multiplying, so that a p-value of exactly one in twenty is at the
// level of 0.05 and no float difference moves it.
//
// THE LEVEL IS 0.05 DIVIDED BY THE NUMBER OF OTHER ARMS. The default effort is set against the other arm with the highest
// share, which is the largest gap and so the smallest p-value of the comparisons it could have made. Choosing the arm by
// the data would raise the chance of a warning above the level when every arm repeats itself alike (computed exactly for
// two other arms of 88 trials each, the better of two tests at 0.05 warned on up to 0.073 of benches of arms alike), so the
// level is divided by the number of other arms: the warning fires only if one of the tests at that level would, and by the
// union bound the chance of that, when every arm repeats itself alike, is at most 0.05.
//
// THE TRIALS OF AN ARM ARE TAKEN AS INDEPENDENT DRAWS. Two trials of one entry may read alike more than strangers do,
// and an arm and another arm are asked the same trials, so the test's validity is a model's, and its false alarm rate is
// stated from the simulation in the ledger and the README. A small number of trials asked again is no reason to refuse to
// test: the test needs a larger difference then, and the warning prints both counts.

/**
 The level of the warning as a fraction: the most the chance of it is, over all the comparisons the default effort is set to,
 when every arm repeats itself alike. A test against one other arm is made at this level, and against several, each at this
 level divided by their number. The warning line prints it.
 */
export const REPEAT_ALPHA = 0.05;

/**
 The same level as an exact fraction of the tables, the denominator of one in twenty, which the decision uses so that no float
 moves it. It and `REPEAT_ALPHA` say one thing twice; a case of the unit test holds them together.
 */
const REPEAT_ALPHA_DENOMINATOR = 20n;

/**
 Ten, the base of the scale a ratio of two large counts is printed at.
 */
const TEN = 10n;

/**
 Decimal places a ratio of two large counts is read to.
 */
const RATIO_DIGITS = 15n;

/**
 The scale a ratio of two large counts is read at.
 */
const RATIO_SCALE = TEN ** RATIO_DIGITS;

/**
 What the test reads of an arm: the trials it was asked in more than one pass, and those whose passes all read alike.
 */
export type RepeatCounts = {
  /**
   Trials the arm was asked in more than one pass.
   */
  readonly repeated: number;

  /**
   Those whose passes all read the same way.
   */
  readonly agreed: number;
};

/**
 What the exact test says of the default effort's count against another arm's.
 */
export type RepeatAgreementTest = {
  /**
   The one-sided exact p-value: the chance, with the totals held, of a count as low as the default effort's.
   */
  readonly p: number;

  /**
   Whether the p-value is at most 0.05 divided by the number of comparisons.
   */
  readonly atMostLevel: boolean;
};

/**
 The arm the default effort is under, and how.
 */
export type RepeatShortfall<Figure extends RepeatCounts> = {
  /**
   The other arm with the highest share, the first of the arms among equal shares.
   */
  readonly other: Figure;

  /**
   The one-sided exact p-value against it.
   */
  readonly p: number;

  /**
   The comparisons the level was divided by: the other arms that have trials asked again.
   */
  readonly comparisons: number;
};

/**
 The number of ways to choose some from a number, exact.

 @param from - how many there are to choose from

 @param pick - how many are chosen

 @returns Count, zero when the pick is not between nothing and all of them

 @example
 ```ts
 // yields 6n
 const ways = waysToChoose({ from: 4, pick: 2, },);
 ```
 */
function waysToChoose(
  {
    from,
    pick,
  }: {
    readonly from: number;
    readonly pick: number;
  },
): bigint {
  if ((pick < 0) || (pick > from))
    return 0n;
  /**
   The steps of the product, counted from one.
   */
  const steps = Array.from(
    { length: pick, },
    function stepAt(
      _unused,
      at,
    ): number {
      return at + 1;
    },
  );
  return steps.reduce(
    function grow(
      ways: bigint,
      step,
    ): bigint {
      // Each step keeps the running product a whole binomial coefficient, so the division is exact.
      return (ways * BigInt((from - pick) + step,)) / BigInt(step,);
    },
    1n,
  );
}

/**
 Runs the one-sided Fisher exact test that the default effort's count of trials that read alike in every pass is low against
 another arm's.

 @param low - default effort's counts

 @param high - counts of the arm it is set against

 @param comparisons - how many arms the default effort is set against, the level being 0.05 divided by it; one when omitted

 @returns Exact p-value and whether it is at the level

 @example
 ```ts
 const test = repeatAgreementTest({ low: { agreed: 15, repeated: 20, }, high: { agreed: 20, repeated: 20, }, },);
 ```
 */
export function repeatAgreementTest(
  {
    low,
    high,
    comparisons = 1,
  }: {
    readonly low: RepeatCounts;
    readonly high: RepeatCounts;
    readonly comparisons?: number;
  },
): RepeatAgreementTest {
  /**
   The trials of both arms.
   */
  const total = low.repeated + high.repeated;
  /**
   The trials that read alike in both arms.
   */
  const alike = low.agreed + high.agreed;
  /**
   The least count the default effort could have with these totals.
   */
  const least = Math.max(
    0,
    low.repeated - (total - alike),
  );
  /**
   The tables for each count the default effort could have had up to its own, in whole numbers.
   */
  const tables = Array.from(
    { length: (low.agreed - least) + 1, },
    function countAt(
      _unused,
      at,
    ): bigint {
      /**
       A count the default effort could have had.
       */
      const count = least + at;
      return waysToChoose({
        from: alike,
        pick: count,
      },) * waysToChoose({
        from: total - alike,
        pick: low.repeated - count,
      },);
    },
  );
  /**
   The tables whose count for the default effort is no higher than its own.
   */
  const reaching = tables.reduce(
    function sum(
      ways: bigint,
      each,
    ): bigint {
      return ways + each;
    },
    0n,
  );
  /**
   All the tables with these totals.
   */
  const all = waysToChoose({
    from: total,
    pick: low.repeated,
  },);
  /**
   The tables at the level, in the same whole numbers: the level of one in twenty over the comparisons.
   */
  const levelled = reaching * REPEAT_ALPHA_DENOMINATOR;
  return {
    p: Number((reaching * RATIO_SCALE) / all,) / Number(RATIO_SCALE,),
    atMostLevel: (levelled * BigInt(comparisons,)) <= all,
  };
}

/**
 Finds the other arm the default effort repeats itself less than, when there is one at the level.

 @param figure - default effort's counts, with at least one trial asked again

 @param others - other arms that have trials asked again, in the order they ran

 @returns One entry for the other arm with the highest share (the first among equals), with the p-value against it and the
 number of comparisons the level was divided by; none when the default effort is not under it at that level, or there is
 no other arm

 @example
 ```ts
 const [shortfall,] = repeatShortfallsOf({ figure, others, },);
 ```
 */
export function repeatShortfallsOf<Figure extends RepeatCounts>(
  {
    figure,
    others,
  }: {
    readonly figure: Figure;
    readonly others: readonly Figure[];
  },
): readonly RepeatShortfall<Figure>[] {
  /**
   The other arm with the highest share, when there is one: shares compared by cross-multiplying the counts, so that two
   counts of one share are equal and the arm that ran first stays first.
   */
  const [highest,] = others.toSorted(function byShareDescending(
    a,
    b,
  ): number {
    return (b.agreed * a.repeated) - (a.agreed * b.repeated);
  },);
  if (highest === undefined)
    return [];
  /**
   The exact test against it, at the level shared among the other arms.
   */
  const test = repeatAgreementTest({
    low: figure,
    high: highest,
    comparisons: others.length,
  },);
  if (!test.atMostLevel)
    return [];
  return [
    {
      other: highest,
      p: test.p,
      comparisons: others.length,
    },
  ];
}

//endregion Editor fidelity arm repeat test
