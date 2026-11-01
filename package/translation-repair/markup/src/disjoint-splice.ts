//region Disjoint splice
// Writes several edits into one text in a single pass (ledger B70). The
// callers used to splice their edits one at a time, last first so earlier
// offsets held, which copies the whole text once per edit. Every edit here
// is placed by offsets into the text as given, so one pass in start order
// copies each untouched run once and joins.
//
// OVERLAP IS REFUSED, not resolved. Each caller's edits are disjoint by
// construction (patch envelopes are checked apart, a list's gaps lie between
// its items, and the title pass drops a quoted span that overlaps one it
// keeps), so two edits sharing a unit or a start mean a caller built its
// edits wrong, and no order of writing them would say what the text should be.

/**
 One edit: the text that replaces a range of the original.
 */
export type SpliceEdit = {
  /**
   First unit the edit replaces, an offset into the original text.
   */
  readonly start: number;

  /**
   First unit after the range, so `start === end` inserts.
   */
  readonly end: number;

  /**
   Text written over the range.
   */
  readonly text: string;
};

/**
 Range of the original text an edit covers, as a refusal names it.
 */
type EditRange = {
  /**
   First unit the edit covers.
   */
  readonly start: number;

  /**
   First unit after it.
   */
  readonly end: number;
};

/**
 Thrown when two edits share part of the original text, or start at one offset.
 */
export class OverlappingEditsError extends Error {
  /**
   Declares this message safe to forward: it names four offsets and no text.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the refusal from the two ranges that meet.

   @param earlier - range starting first

   @param later - range starting inside it

   @example
   ```ts
   throw new OverlappingEditsError({ earlier: { start: 0, end: 4, }, later: { start: 2, end: 6, }, },);
   ```
   */
  constructor(
    {
      earlier,
      later,
    }: {
      readonly earlier: EditRange;
      readonly later: EditRange;
    },
  ) {
    super(
      `edits ${String(earlier.start,)}-${String(earlier.end,)} and ${String(later.start,)}-${
        String(later.end,)
      } overlap or start at one offset`,
    );
    this.name = 'OverlappingEditsError';
  }
}

/**
 Writes disjoint edits into a text in one pass.

 @param text - original text every edit's offsets point into

 @param edits - edits in any order; one may begin where another ends, but two
 never share a unit or a start

 @returns Text with every edit written

 @throws {@link OverlappingEditsError} when two edits share a unit or a start,
 since the caller's edits were built to be disjoint

 @example
 ```ts
 spliceDisjointEdits({ text: 'a cat sat', edits: [ { start: 6, end: 9, text: 'napped', }, { start: 2, end: 5, text: 'kitten', }, ], },);
 // 'a kitten napped'
 ```
 */
export function spliceDisjointEdits(
  {
    text,
    edits,
  }: {
    readonly text: string;
    readonly edits: readonly SpliceEdit[];
  },
): string {
  /**
   Edits in start order.
   */
  const ordered = edits.toSorted(function byStart(
    left,
    right,
  ): number {
    return left.start - right.start;
  },);
  /**
   Pieces of the result: each untouched run, then each edit's text.
   */
  const pieces: string[] = [];
  /**
   First unit of the original not yet copied.
   */
  const cursor = { at: 0, };
  for (const [index, edit,] of ordered.entries()) {
    /**
     The edit before this one in start order, absent for the first.
     */
    const earlier = ordered[index - 1];
    // TWO EDITS AT ONE START have no order the offsets could give them, so
    // they are refused with the overlaps.
    if ((earlier !== undefined) && ((edit.start < earlier.end) || (edit.start === earlier.start))) {
      throw new OverlappingEditsError({
        earlier,
        later: edit,
      },);
    }
    pieces.push(
      text.slice(
        cursor.at,
        edit.start,
      ),
      edit.text,
    );
    cursor.at = edit.end;
  }
  pieces.push(text.slice(cursor.at,),);
  return pieces.join('',);
}

//endregion Disjoint splice
