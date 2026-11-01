import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  headingAffinity,
  tableCell,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Heading alignment grid
// Scoring primitives for the forced aligner: the lexicographic score, its
// arithmetic, and the affinity and trust grids. Split from the aligner only
// because the two together exceed one file's line budget.

/**
 Affinity at or above which a unique candidate may anchor.

 Vestigial in practice and kept for the case it was written for.
 `headingAffinity` divides by the smaller token count, so a single-token
 heading scores 1.00 against any heading containing that token, and the
 threshold never binds. UNIQUENESS carries this design, not the threshold.
 */
const TRUST = 0.5;

/**
 Lexicographic score: trusted anchors, gap count, soft affinity.
 */
export type LexScore = readonly [
  number,
  number,
  number
];

/**
 Score no path can reach, so an unreachable cell never wins a comparison.
 */
export const UNREACHABLE: LexScore = [
  Number.NEGATIVE_INFINITY,
  Number.POSITIVE_INFINITY,
  Number.NEGATIVE_INFINITY,
];

/**
 Cost of leaving one heading unpaired.
 */
export const GAP: LexScore = [
  0,
  1,
  0
];

/**
 Adds two lexicographic scores component by component.

 @param left - first score

 @param right - second score

 @returns Sum

 @example
 ```ts
 const total = addScore({ left, right, },);
 ```
 */
export function addScore(
  {
    left,
    right,
  }: {
    readonly left: LexScore;
    readonly right: LexScore;
  },
): LexScore {
  return [
    left[0] + right[0],
    left[1] + right[1],
    left[2] + right[2],
  ];
}

/**
 Reports whether one score beats another.

 Trusted anchors dominate, then FEWER gaps, then soft affinity. The middle
 term inverts, which is the whole reason a gap can win here and cannot win in
 the shipped scorer.

 @param candidate - score under test

 @param incumbent - score to beat

 @returns True when the candidate is strictly better

 @example
 ```ts
 const wins = beats({ candidate, incumbent, },);
 ```
 */
export function beats(
  {
    candidate,
    incumbent,
  }: {
    readonly candidate: LexScore;
    readonly incumbent: LexScore;
  },
): boolean {
  if (candidate[0] !== incumbent[0])
    return candidate[0] > incumbent[0];
  if (candidate[1] !== incumbent[1])
    return candidate[1] < incumbent[1];
  return candidate[2] > incumbent[2];
}

/**
 Reports whether two scores are identical.

 @param left - first score

 @param right - second score

 @returns True when every component matches

 @example
 ```ts
 const same = sameScore({ left, right, },);
 ```
 */
export function sameScore(
  {
    left,
    right,
  }: {
    readonly left: LexScore;
    readonly right: LexScore;
  },
): boolean {
  return (left[0] === right[0]) && (left[1] === right[1])
    && (left[2] === right[2]);
}

/**
 Everything the affinity grid says about two heading sequences.
 */
export type Grid = {
  /**
   Affinity of every source and target pairing.
   */
  readonly affinity: readonly (readonly number[])[];

  /**
   Pairings at or above threshold that are the strict maximum of both their
   row and their column, so a name repeated across headings never anchors.
   */
  readonly trusted: readonly (readonly boolean[])[];
};

/**
 Scores every possible pairing and marks the trustworthy ones.

 @param sourceHeadings - original-side unit labels

 @param targetHeadings - translation-side unit labels

 @returns Affinity and trust grids

 @example
 ```ts
 const grid = buildGrid({ sourceHeadings, targetHeadings, },);
 ```
 */
export function buildGrid(
  {
    sourceHeadings,
    targetHeadings,
  }: {
    readonly sourceHeadings: readonly string[];
    readonly targetHeadings: readonly string[];
  },
): Grid {
  /**
   Affinity of every pairing.
   */
  const affinity = sourceHeadings.map(function scoreRow(source,): readonly number[] {
    return targetHeadings.map(function scoreCell(target,): number {
      return headingAffinity({
        source,
        target,
      },);
    },);
  },);

  return {
    affinity,
    trusted: affinity.map(function markRow(
      row,
      sourceIndex,
    ): readonly boolean[] {
      return row.map(function markCell(
        value,
        targetIndex,
      ): boolean {
        if (value < TRUST)
          return false;

        return row.every(function isRowMax(
          other,
          index,
        ): boolean {
          return (index === targetIndex) || (other < value);
        },)
          && affinity.every(function isColumnMax(
            otherRow,
            index,
          ): boolean {
            // Every row scores every target, so the column holds a cell in
            // each.
            return (index === sourceIndex) || (nonNullishOrThrow(otherRow[targetIndex],) < value);
          },);
      },);
    },),
  };
}

/**
 Scores pairing one source unit with one target unit.

 @param grid - affinity and trust

 @param sourceIndex - source unit

 @param targetIndex - target unit

 @returns Lexicographic cost of that pairing

 @example
 ```ts
 const cost = pairScore({ grid, sourceIndex: 0, targetIndex: 0, },);
 ```
 */
export function pairScore(
  {
    grid,
    sourceIndex,
    targetIndex,
  }: {
    readonly grid: Grid;
    readonly sourceIndex: number;
    readonly targetIndex: number;
  },
): LexScore {
  /**
   Affinity of this pairing, a cell the grid holds for every unit on both
   sides.
   */
  const value = tableCell({
    table: grid.affinity,
    row: sourceIndex,
    column: targetIndex,
  },);
  /**
   Whether the pairing may anchor.
   */
  const trusted = tableCell({
    table: grid.trusted,
    row: sourceIndex,
    column: targetIndex,
  },);
  return [
    trusted ? value : 0,
    0,
    value,
  ];
}


//endregion Heading alignment grid
