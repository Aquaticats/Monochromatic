import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  BREAK,
  MAXIMAL_REPEAT_MINIMUM,
} from './repeat-groups-bounds.ts';

//region Repeat groups runs
// The finder of maximal runs over the wording stream.

/**
 Whether every occurrence reads the same wording character at one shift from
 its start, and that character is wording rather than a break or the end.

 @param stream - wording stream the occurrences index into

 @param positions - stream index where each occurrence starts

 @param shift - offset from each start, negative to read before it

 @returns Whether all occurrences agree on a wording character there

 @example
 ```ts
 const agrees = agreeAt({ stream, positions: [0, 12,], shift: 12, },);
 ```
 */
function agreeAt(
  {
    stream,
    positions,
    shift,
  }: {
    readonly stream: readonly string[];
    readonly positions: readonly number[];
    readonly shift: number;
  },
): boolean {
  /**
   Character each occurrence reads, a break where the read leaves the stream.
   */
  const reads = positions.map(function readAt(position,): string {
    /**
     Stream index read for this occurrence.
     */
    const index = position + shift;
    if ((index < 0) || (index >= stream.length))
      return BREAK;
    return nonNullishOrThrow(stream[index],);
  },);
  /**
   Character the first occurrence reads, which every other must match.
   */
  const [first,] = reads;
  return (first !== BREAK) && reads.every(function same(read,): boolean {
    return read === first;
  },);
}

/**
 Maximal runs of at least the minimum length that stand at two or more places,
 each with its occurrences as stream indices, the run extended as far as every
 occurrence agrees. Runs are listed in the order their first window of the
 minimum length appears in the stream.

 COST: one pass over the windows, then for each window group that is not the
 previous window's group moved right by one, an extension over its occurrences
 as long as the run, so a stretch copied whole is extended once and not once
 per window. The work is linear in the stream for prose and grows with the
 product of occurrences and run length only for text that repeats in
 nested stretches.

 @param stream - wording stream

 @returns Runs with their length and the stream index where each occurrence starts

 @example
 ```ts
 const runs = maximalRunsOf({ stream, },);
 ```
 */
export function maximalRunsOf(
  {
    stream,
  }: {
    readonly stream: readonly string[];
  },
): readonly {
  readonly length: number;
  readonly starts: readonly number[];
}[] {
  /**
   Stream indices at which each run of the minimum length begins.
   */
  const positionsOf = new Map<string, number[]>();
  for (let index = 0; (index + MAXIMAL_REPEAT_MINIMUM) <= stream.length; index++) {
    /**
     Run of the minimum length beginning at this index.
     */
    const run = stream.slice(
      index,
      index + MAXIMAL_REPEAT_MINIMUM,
    )
      .join('',);
    if (run.includes(BREAK,))
      continue;
    /**
     Positions of this run seen so far.
     */
    const positions = positionsOf.get(run,) ?? [];
    positions.push(index,);
    positionsOf.set(
      run,
      positions,
    );
  }

  /**
   Runs already reported, by length and start positions, since a run is found from each of its parts.
   */
  const reported = new Set<string>();

  /**
   Runs found, before the overlap pass.
   */
  const runs: {
    readonly length: number;
    readonly starts: readonly number[]
  }[] = [];
  for (const positions of positionsOf.values()) {
    if (positions.length < 2)
      continue;

    // A group whose every occurrence follows one more shared character, and
    // whose window one place earlier is held by exactly as many places, is
    // that earlier group moved right by one: the same run, reported from there.
    // Skipping it keeps the extension of a run from repeating for every window
    // of a long run, which grew with the square of the run length.
    if (agreeAt({
      stream,
      positions,
      shift: -1,
    },)) {
      /**
       Stream index one place before the group's first occurrence.
       */
      const before = nonNullishOrThrow(positions[0],) - 1;

      /**
       Window of the minimum length that begins there, as the map keys it.
       */
      const earlierWindow = stream.slice(
        before,
        before + MAXIMAL_REPEAT_MINIMUM,
      )
        .join('',);

      /**
       Places that hold the earlier window.
       */
      const earlierPlaces = positionsOf.get(earlierWindow,);
      if (earlierPlaces === undefined) {
        throw new Error(
          'unreachable: the window one place before a group whose occurrences all agree there holds a break, though the window of the group holds none',
        );
      }
      if (earlierPlaces.length === positions.length)
        continue;
    }

    /**
     Characters the occurrences agree on after the minimum, on the right.
     */
    let right = 0;
    while (agreeAt({
      stream,
      positions,
      shift: MAXIMAL_REPEAT_MINIMUM + right,
    },))
      right++;

    /**
     Characters the occurrences agree on before the run, on the left.
     */
    let left = 0;
    while (agreeAt({
      stream,
      positions,
      shift: -(left + 1),
    },))
      left++;

    /**
     Non-space length of the extended run.
     */
    const length = MAXIMAL_REPEAT_MINIMUM + left
      + right;
    /**
     Occurrence starts after extension, in order, keeping only those that do not overlap the one before.
     */
    const starts = positions
      .map(function extendLeft(position,): number {
        return position - left;
      },)
      .toSorted(function ascending(
        a,
        b,
      ): number {
        return a - b;
      },);
    /**
     Occurrences kept, none overlapping the one before.
     */
    const kept: number[] = [];
    /**
     End of the last kept occurrence, before which the next one overlaps.
     */
    let end = -1;
    for (const start of starts) {
      if (start < end)
        continue;
      kept.push(start,);
      end = start + length;
    }
    if (kept.length < 2)
      continue;
    /**
     Identity of this run for de-duplication.
     */
    const key = `${String(length,)}:${kept.join(',',)}`;
    if (reported.has(key,))
      continue;
    reported.add(key,);
    runs.push({
      length,
      starts: kept,
    },);
  }
  return runs;
}

//endregion Repeat groups runs
