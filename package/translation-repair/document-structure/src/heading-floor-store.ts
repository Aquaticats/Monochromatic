//region Heading floor store
// WHERE A SETEXT HEADING'S OWN LINES START, when the parser starts the heading earlier (`heading-floors.ts` says why),
// kept by the heading node, because a reader of the heading alone cannot see its siblings. The store is apart from the
// code that fills it so that the footnote reading, which asks, and the code that fills it, which reads the footnote
// reading's bounds, do not import each other.

/**
 The offset after the definitions that stand right before a heading and that the parser began the heading at.
 */
const HEADING_FLOORS = new WeakMap<object, number>();

/**
 Notes where the lines of a heading start.

 @param heading - heading node of a tree

 @param floor - offset after the last definition the parser started the heading at

 @example
 ```ts
 noteHeadingFloor({ heading, floor: 6, },);
 ```
 */
export function noteHeadingFloor(
  {
    heading,
    floor,
  }: {
    readonly heading: object;
    readonly floor: number;
  },
): void {
  HEADING_FLOORS.set(
    heading,
    floor,
  );
}

/**
 Where the lines of a heading start when the parser started the heading earlier, at the definitions before it.

 @param heading - heading node of a tree the floors were noted for

 @returns Offset after the last definition before the heading, or zero for a heading none overlaps, which no start can
 fall under

 @example
 ```ts
 const start = Math.max(heading.position.start.offset, headingFloorOf({ heading, },),);
 ```
 */
export function headingFloorOf({ heading, }: { readonly heading: object; }): number {
  return HEADING_FLOORS.get(heading,) ?? 0;
}

//endregion Heading floor store
