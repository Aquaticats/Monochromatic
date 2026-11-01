//region Sorted number sets
// RENDERS SETS OF UNIT INDICES AS SORTED ARRAYS, for cases that compare
// partner or gap sets by value rather than by iteration order.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The heading-alignment and lane-slice
// tests kept their own copy of this rendering; both now import it from here.

/**
 Renders sets of unit indices as sorted arrays, so a case reads as what it
 claims.

 @param sets - sets in unit order

 @returns Same sets as sorted arrays

 @example
 ```ts
 const rendered = listed({ sets: [new Set([1, 0,],),], },);
 ```
 */
export function listed(
  { sets, }: { readonly sets: readonly ReadonlySet<number>[]; },
): readonly (readonly number[])[] {
  return sets.map(function toList(one,): readonly number[] {
    return [...one,].toSorted(function ascending(
      left,
      right,
    ): number {
      return left - right;
    },);
  },);
}

//endregion Sorted number sets
