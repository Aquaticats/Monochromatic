/**
 Constant-time comparison of two strings.

 String equality returns at the first differing character, so the time it takes
 says how many leading characters a candidate matched. That is enough to recover
 a secret one character at a time over enough requests (CWE-208). Token and digest
 comparisons therefore go through this module instead of `===`.

 The cost is deliberately independent of where the values differ: the comparison
 runs over the longer value and folds the length difference in, so neither a
 mismatch position nor a shared prefix can shorten it.

 @module
 */

/**
 Values to compare.
 */
export type ConstantTimeEqualsParams = {
  /**
   First value.
   */
  readonly left: string;
  /**
   Second value.
   */
  readonly right: string;
};

/**
 Parameters for {@link foldedDifference}.
 */
type FoldedDifferenceParams = {
  /**
   First value.
   */
  readonly left: string;
  /**
   Second value.
   */
  readonly right: string;
  /**
   How many positions to compare, which is the longer of the two lengths.
   */
  readonly positions: number;
};

/**
 Read one code unit of a value, treating a position past its end as zero.

 @param value - value to read

 @param index - position to read

 @returns the code unit, or `0` past the end of the value
 */
function codeUnitAt({
  value,
  index,
}: {
  readonly value: string;
  readonly index: number
},): number {
  return index < value.length ? (value.codePointAt(index,) ?? 0) : 0;
}

/**
 Fold every code-unit difference of two values, and their length difference, into
 one number.

 @param left - first value

 @param right - second value

 @param positions - how many positions to compare

 @returns zero when both values hold the same code units and the same length
 */
function foldedDifference({
  left,
  right,
  positions,
}: FoldedDifferenceParams,): number {
  /**
   Folded difference; a length difference alone already sets it.
   */
  let difference = left.length ^ right.length;
  for (let index = 0; index < positions; index += 1) {
    difference |= codeUnitAt({
      value: left,
      index,
    },) ^ codeUnitAt({
      value: right,
      index,
    },);
  }
  return difference;
}

/**
 Whether two strings are equal, in time that does not depend on where they differ.

 @param left - first value

 @param right - second value

 @returns `true` when both hold the same characters in the same order

 @example
 ```ts
 constantTimeEquals({ left: 'abc', right: 'abc' }); // true
 constantTimeEquals({ left: 'abc', right: 'abd' }); // false
 ```
 */
export function constantTimeEquals({
  left,
  right,
}: ConstantTimeEqualsParams,): boolean {
  /**
   Length the loop runs, so a shorter value never ends it early.
   */
  const positions = Math.max(
    left.length,
    right.length,
  );
  return foldedDifference({
    left,
    right,
    positions,
  },) === 0;
}
