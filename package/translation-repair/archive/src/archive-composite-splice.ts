import type { LineHunk, } from './archive-composite-hunks.ts';

//region Archive composite splice
// THE HUNKS OF SEVERAL REVISIONS APPLIED TO THE BLOCK TOGETHER (ledger B803). The caller has already refused hunks that
// collide (`hunkCollisions`), so the hunks taken together are ordered by where they start in the block, an insertion
// before a hunk that begins at its gap, and the block's lines between them are copied once.

/**
 The block with every revision's hunks applied.
 */
export type SplicedBlock = {
  /**
   Lines of the combined text.
   */
  readonly lines: readonly string[];
  /**
   For each revision, in the order given, the zero-based positions of the combined text's lines it put there: the
   replacement lines of its hunks, and for a hunk that only removes lines, the line that now follows the cut (the last
   line when the cut was at the end).
   */
  readonly positions: readonly (readonly number[])[];
};

/**
 One hunk with the revision it came from.
 */
type OwnedHunk = {
  /**
   Position of the revision among those given.
   */
  readonly owner: number;
  /**
   The hunk.
   */
  readonly hunk: LineHunk;
};

/**
 Positions a revision put, settled to exist in the combined text.

 @param positions - positions recorded while the hunks were applied, a removal's being where the next line would go

 @param length - count of lines in the combined text

 @returns Ascending positions without repeats, a removal's cut at the end falling on the last line

 @example
 ```ts
 settledPositions({ positions: [3, 3, 1,], length: 3, },); // [1, 2]
 ```
 */
function settledPositions(
  {
    positions,
    length,
  }: {
    readonly positions: readonly number[];
    readonly length: number;
  },
): readonly number[] {
  return [
    ...new Set(
      positions
        .map(function withinText(position,): number {
          return Math.min(
            position,
            length - 1,
          );
        },)
        .filter(function exists(position,): boolean {
          return position >= 0;
        },),
    ),
  ].toSorted(function ascending(
    first,
    second,
  ): number {
    return first - second;
  },);
}

/**
 Applies the hunks of several revisions that do not collide to the block's lines.

 @param base - lines of the block, each with the line ending that closes it

 @param revisions - each revision's hunks, none colliding with another revision's

 @returns The combined lines, and the combined lines each revision put there

 @example
 ```ts
 spliceHunks({ base: ['a\n', 'b',], revisions: [[{ start: 0, end: 1, lines: ['c\n',], },],],},); // lines ['c\n', 'b']
 ```
 */
export function spliceHunks(
  {
    base,
    revisions,
  }: {
    readonly base: readonly string[];
    readonly revisions: readonly (readonly LineHunk[])[];
  },
): SplicedBlock {
  /**
   Every hunk with the revision it came from, in block order: by start, and an insertion ahead of a hunk that begins
   at its gap.
   */
  const ordered = revisions
    .flatMap(function ownedHunks(
      hunks,
      owner,
    ): readonly OwnedHunk[] {
      return hunks.map(function owned(hunk,): OwnedHunk {
        return {
          owner,
          hunk,
        };
      },);
    },)
    .toSorted(function byPlace(
      first,
      second,
    ): number {
      return (first.hunk
        .start
        - second.hunk
          .start) || (first.hunk
            .end
            - second.hunk
              .end);
    },);
  /**
   Combined lines so far.
   */
  const lines: string[] = [];
  /**
   Positions each revision put, before they are settled.
   */
  const placed: number[][] = revisions.map(function noPositions(): number[] {
    return [];
  },);
  /**
   First block line not yet copied or replaced.
   */
  let cursor = 0;
  for (const {
    owner,
    hunk,
  } of ordered) {
    for (const line of base.slice(
      cursor,
      hunk.start,
    ))
      lines.push(line,);
    if (hunk.lines
      .length
      === 0)
      placed[owner]
        ?.push(lines.length,);
    for (const line of hunk.lines) {
      placed[owner]
        ?.push(lines.length,);
      lines.push(line,);
    }
    cursor = hunk.end;
  }
  for (const line of base.slice(cursor,))
    lines.push(line,);
  /**
   The combined text with the positions each revision put.
   */
  const block: SplicedBlock = {
    lines,
    positions: placed.map(function settled(positions: readonly number[],): readonly number[] {
      return settledPositions({
        positions,
        length: lines.length,
      },);
    },),
  };
  return block;
}

//endregion Archive composite splice
