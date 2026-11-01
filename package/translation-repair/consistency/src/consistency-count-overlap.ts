import type { ConsistencyRendering, } from './consistency-count-types.ts';

//region Consistency count overlap
// TWO PLACES OF ONE SLICE WHOSE LOCATED WORDS OVERLAP ARE NEITHER LOCATED. One run of English
// cannot be two renderings, and replacing the one would damage the other, so the counter
// (`consistency-count.ts`) takes the rendering of each such place and leaves it unlocated.
//
// THE COST IS A SORT PER SLICE, NOT A COMPARISON OF EVERY PAIR. A located span is never empty (its
// words show a reader something), so two spans of a slice overlap exactly when each starts before the
// other ends. With the spans of a slice in order of their starts, a span overlaps an earlier one when
// the furthest end read so far reaches past its start, and overlaps a later one when the next span
// starts before its end: no other span can overlap it, since every span between would overlap it too.
// A run of N places costs N log N for the sort and one more read for each place.

/**
 One located span of a slice with the position of the place it belongs to.
 */
type LocatedSpan = {
  /**
   Offset in the English where the words start.
   */
  readonly start: number;

  /**
   Exclusive offset in the English where the words end.
   */
  readonly end: number;

  /**
   Position of the place among the places the caller passed.
   */
  readonly position: number;
};

/**
 Takes the rendering of every place whose located words overlap another place
 of the same slice, since one run of English cannot be two renderings and
 replacing the one would damage the other.

 @param places - slice of each place, in page order, where it is known

 @param renderings - each place's rendering as located, in the same order

 @returns The renderings, a place that overlaps another unlocated

 @example
 ```ts
 const separated = unlessOverlapping({ places, renderings, },);
 ```
 */
export function unlessOverlapping(
  {
    places,
    renderings,
  }: {
    readonly places: readonly { readonly sliceIndex?: number; }[];
    readonly renderings: readonly ConsistencyRendering[];
  },
): readonly ConsistencyRendering[] {
  /**
   The located words of each slice the caller names, mutated only inside this function.
   */
  const spansBySlice = new Map<number, LocatedSpan[]>();
  for (const [position, rendering,] of renderings.entries()) {
    /**
     Slice this place stands in, where the caller knows it.
     */
    const sliceIndex = places[position]
      ?.sliceIndex;
    if ((rendering.kind !== 'located') || (sliceIndex === undefined))
      continue;
    /**
     Where the words of the place start and the words themselves.
     */
    const {
      start,
      core,
    } = rendering;
    /**
     The words of the place as their start, their end and the place's position.
     */
    const span: LocatedSpan = {
      start,
      end: start + core.length,
      position,
    };
    /**
     The spans of the slice so far.
     */
    const known = spansBySlice.get(sliceIndex,);
    if (known === undefined) {
      spansBySlice.set(
        sliceIndex,
        [span,],
      );
    }
    else {
      known.push(span,);
    }
  }
  /**
   Positions of the places whose words overlap those of another place of their slice, mutated only inside this function.
   */
  const overlapping = new Set<number>();
  for (const spans of spansBySlice.values()) {
    /**
     The slice's spans by where they start, so a span overlaps another exactly when an earlier one reaches past its start or
     the next one starts before its end.
     */
    const ordered = spans.toSorted(function byStart(
      left,
      right,
    ): number {
      return left.start - right.start;
    },);
    /**
     The furthest end of the spans read so far, mutated only inside this function.
     */
    const reach = { furthest: Number.NEGATIVE_INFINITY, };
    for (const [order, span,] of ordered.entries()) {
      /**
       The span that starts next, none for the last.
       */
      const next = ordered[order + 1];
      if ((span.start < reach.furthest) || ((next !== undefined) && (next.start < span.end)))
        overlapping.add(span.position,);
      reach.furthest = Math.max(
        reach.furthest,
        span.end,
      );
    }
  }
  return renderings.map(function separated(
    rendering,
    position,
  ): ConsistencyRendering {
    return overlapping.has(position,)
      ? {
        kind: 'unlocated',
        reason: 'overlaps-another',
      }
      : rendering;
  },);
}

//endregion Consistency count overlap
