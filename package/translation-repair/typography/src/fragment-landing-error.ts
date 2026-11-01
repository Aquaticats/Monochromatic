import { wordForCount, } from './count-word.ts';

//region Fragment landing error
// A LANDING THAT LIES OUTSIDE ITS SLICE, ENDS BEFORE IT STARTS, OR IS NO WHOLE NUMBER OF UNITS (n160, Y7; n166, Z8: `NaN` and a
// fraction fail every comparison of the range check, so the check asks for whole numbers first). A fragment of a slice is read in the slice with the
// fragment written over the region it replaces (`fragmentProseMask`, `typography-prose-mask.ts`). The region is the slice's own,
// found by `apply-patch.ts` only after the envelope's current text matched its base text, so no run reaches a bad landing; a mask
// built from one would be read as the mask of no prose at all, or as a refusal of the page grammar about a slice that parses, and a
// typography rule would then stay silent or log a wrong cause. Throwing names the broken invariant instead.

/**
 A landing of a fragment that lies outside its slice or ends before it starts.

 @example
 ```ts
 throw new FragmentLandingError({ slice: 24, start: 30, end: 31, },);
 ```
 */
export class FragmentLandingError extends Error {
  /**
   Message names offsets and lengths only, never text of the page.
   */
  readonly messageNamesOnly: true = true;

  /**
   @param slice - length of the slice the fragment lands in

   @param start - offset where the replaced region starts

   @param end - exclusive offset where it ends
   */
  constructor(
    {
      slice,
      start,
      end,
    }: {
      readonly slice: number;
      readonly start: number;
      readonly end: number;
    },
  ) {
    super(
      `unreachable: a fragment lands at ${String(start,)} to ${String(end,)} in a slice of ${
        String(slice,)
      } ${
        wordForCount({
          count: slice,
          one: 'unit',
          many: 'units',
        },)
      }, and a landing starts and ends at whole numbers, starts at 0 or later, ends at or after its start, and ends within the slice`,
    );
    this.name = 'FragmentLandingError';
  }
}

/**
 Throws when a landing lies outside its slice or ends before it starts.

 @param slice - length of the slice the fragment lands in

 @param start - offset where the replaced region starts

 @param end - exclusive offset where it ends

 @throws {@link FragmentLandingError} when an offset is not a whole number, or the region starts before the slice, ends before it
 starts, or ends past the slice

 @example
 ```ts
 assertLandingInside({ slice: 24, start: 8, end: 20, },);
 ```
 */
export function assertLandingInside(
  {
    slice,
    start,
    end,
  }: {
    readonly slice: number;
    readonly start: number;
    readonly end: number;
  },
): void {
  /**
   Whether an offset is a unit position at all: `NaN` and a fraction fail every comparison made after it, so neither is caught by them.
   */
  const wholeNumbers = Number.isInteger(start,) && Number.isInteger(end,);
  /**
   Whether the region reaches before the slice or past it.
   */
  const outside = (start < 0) || (end > slice);
  /**
   Whether the region ends before it starts, or is no unit position at all.
   */
  const unreadable = (!wholeNumbers) || (end < start);
  if (outside || unreadable) {
    throw new FragmentLandingError({
      slice,
      start,
      end,
    },);
  }
}

//endregion Fragment landing error
