import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

//region Bench draw
// WHICH slices a bench runs, given the slices. Reading the corpus lives in
// `bench-sample.ts`; this half is pure so the draw can be tested at all.
//
// The properties that matter are both invisible in the output of one run: the
// draw must be identical between runs, or widths would be compared over
// different samples, and it must spread across the size range rather than
// clustering wherever the corpus is dense. Neither is checkable through a
// function that needs a pinned corpus checkout to execute.

/**
 Raised when no bench sample can be drawn from what was offered.

 @example
 ```ts
 throw new BenchDrawError({ message: 'a bench sample cannot be drawn from no slices', },);
 ```
 */
export class BenchDrawError extends Error {
  /**
   Builds refusal carrying what could not hold.

   @param message - what was offered in place of slices to draw from

   @example
   ```ts
   throw new BenchDrawError({ message: 'a bench sample cannot be drawn from no slices', },);
   ```
   */
  public constructor({ message, }: { readonly message: string; },) {
    super(message,);
    this.name = 'BenchDrawError';
  }
}

/**
 Midpoint of a stratum, so the draw takes representative slices rather than
 the corpus extremes.
 */
const HALF = 1 / 2;

/**
 What the draw needs of a slice: something to order by and something to break
 ties with.

 @example
 ```ts
 const slice: DrawableSlice = { entryId: 'Mittens', index: 3, sourceText, };
 ```
 */
export type DrawableSlice = {
  /**
   Entry this slice was cut from, the first tiebreak.
   */
  readonly entryId: string;

  /**
   Position within that entry, the second.
   */
  readonly index: number;

  /**
   Original passage, whose length orders the draw.
   */
  readonly sourceText: string;
};

/**
 Orders slices by source size, smallest first.

 Ties fall back to entry then position, so two slices of equal size never swap
 places between runs. `toSorted` is stable, but the input order is the corpus
 listing order, which is not a property worth depending on.

 @param slices - slices to order

 @returns Same slices, ordered

 @example
 ```ts
 const ordered = orderBySourceSize({ slices, },);
 ```
 */
export function orderBySourceSize<SliceT extends DrawableSlice,>(
  { slices, }: { readonly slices: readonly SliceT[]; },
): readonly SliceT[] {
  return slices.toSorted(function byLength(
    left,
    right,
  ): number {
    /**
     Source size of one side, which is what orders the draw.
     */
    const leftLength = left.sourceText
      .length;

    /**
     Same for the other side of this comparison.
     */
    const rightLength = right.sourceText
      .length;

    /**
     Size gap deciding almost every comparison.
     */
    const bySize = leftLength - rightLength;
    if (bySize !== 0)
      return bySize;

    /**
     Entry name by code point, so identical sizes never reorder between
     runs or between machines (ledger B95).
     */
    const byEntry = compareCodePoints({
      left: left.entryId,
      right: right.entryId,
    },);
    if (byEntry !== 0)
      return byEntry;

    return left.index - right.index;
  },);
}

/**
 Draws a sample spread evenly across the size range.

 Takes the MIDPOINT of each stratum rather than its first member. Taking the
 first starts the draw at the corpus minimum, and the smallest slice in this
 corpus is a 3-character source against a 226-character translation, which
 measures the aligner rather than the judges.

 @param slices - every candidate slice, in any order

 @param count - slices wanted; fewer come back only when fewer exist

 @returns Sample ordered by source size, smallest first

 @throws Error when there is nothing to draw from

 @example
 ```ts
 const sample = pickSpreadSample({ slices, count: 10, },);
 ```
 */
export function pickSpreadSample<SliceT extends DrawableSlice,>(
  {
    slices,
    count,
  }: {
    readonly slices: readonly SliceT[];
    readonly count: number;
  },
): readonly SliceT[] {
  if (slices.length === 0)
    throw new BenchDrawError({ message: 'a bench sample cannot be drawn from no slices', },);

  /**
   Slices ordered by source size.
   */
  const ordered = orderBySourceSize({ slices, },);

  /**
   Picks this draw can actually make.
   */
  const wanted = Math.min(
    count,
    ordered.length,
  );

  /**
   Stratum width that spreads the picks across the whole ordering.
   */
  const stride = ordered.length / wanted;

  return Array.from(
    { length: wanted, },
    function pick(
      _unused,
      position,
    ): SliceT {
      return nonNullishOrThrow(
        ordered[Math.floor((position + HALF) * stride,)],
      );
    },
  );
}

//endregion Bench draw
