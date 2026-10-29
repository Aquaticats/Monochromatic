import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import { firstAtLeast, } from './source-reading-words.ts';
import type { TextRange, } from './source-ships-ranges.ts';

//region Source reading overlap
// WHICH STRETCHES OF A TEXT SHARE A CHARACTER WITH ANY OF A LIST, asked of each stretch of another list in the time of a
// search and not of a pass over the list: the stretches are put in the order of their starts with the furthest end any of
// them reaches so far, and a stretch overlaps one of them when the furthest end of those that start before it ends
// reaches past its start.

/**
 A list of stretches of a text in the order of their starts, with how far the ones up to each reach.
 */
export type OverlapIndex = {
  /**
   Where each stretch starts, in order.
   */
  readonly starts: readonly number[];

  /**
   The furthest end of the stretches up to and including the one at the same place of the list.
   */
  readonly furthestEnds: readonly number[];
};

/**
 Puts stretches of a text in the order of their starts.

 @param ranges - stretches, in any order

 @returns Index to ask

 @example
 ```ts
 const index = overlapIndexOf({ ranges: [{ start: 4, end: 9, }, { start: 0, end: 2, },], },);
 ```
 */
export function overlapIndexOf({ ranges, }: { readonly ranges: readonly TextRange[]; },): OverlapIndex {
  /**
   The stretches in the order of their starts.
   */
  const ordered = ranges.toSorted(function byStart(
    left,
    right,
  ): number {
    return left.start - right.start;
  },);
  /**
   The furthest end so far.
   */
  const seen = { furthest: Number.NEGATIVE_INFINITY, };
  return {
    starts: ordered.map(function startOf({ start, },): number {
      return start;
    },),
    furthestEnds: ordered.map(function furthestEndOf({ end, },): number {
      seen.furthest = Math.max(
        seen.furthest,
        end,
      );
      return seen.furthest;
    },),
  };
}

/**
 Whether a stretch shares a character with any stretch of a list.

 @param index - list, as {@link overlapIndexOf} puts it

 @param range - stretch

 @returns Whether some stretch of the list starts before the range ends and ends after it starts

 @example
 ```ts
 const reached = reachesAny({ index: overlapIndexOf({ ranges: [{ start: 0, end: 5, },], },), range: { start: 4, end: 9, }, },);
 ```
 */
export function reachesAny(
  {
    index,
    range,
  }: {
    readonly index: OverlapIndex;
    readonly range: TextRange;
  },
): boolean {
  /**
   How many stretches of the list start before the range ends.
   */
  const before = firstAtLeast({
    positions: index.starts,
    from: range.end,
  },);
  if (before === 0)
    return false;
  return nonNullishOrThrow(index.furthestEnds[before - 1],) > range.start;
}

//endregion Source reading overlap
