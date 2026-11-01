import {
  numbered,
  sharedLines,
} from './archive-composite-lines.ts';

//region Archive composite hunks
// THE LINE DIFF BEHIND THE COMPOSITE (ledger B803, zheermao22, 2026-10-21). The composite of disjoint revisions
// (ledger B712) matched a revision to the block line for line and so refused every revision that added or removed a
// line: the letter of that run drew a revision of 34 lines against the block's 36, and the email one of 23 against 24.
// A revision is now read as the hunks that turn the block into it, and the composite applies the hunks of several
// revisions together when none of them collides with another.
//
// WHAT A HUNK IS. The longest common subsequence of the block's lines and the revision's lines (a line is the text
// through its line ending, as `linesOf` cuts it) says which lines the revision kept. Every stretch between two kept
// lines in which the revision deleted lines, inserted lines or both is one hunk: it replaces the block's lines from
// `start` up to but not including `end` with `lines`. An insertion has `start === end` and sits in the gap before
// line `start`; a deletion has no `lines`.
//
// THE ALIGNMENT IS DETERMINISTIC. Blocks repeat lines (a blank quote line between every paragraph), so several
// alignments can be equally long, and the one chosen decides which of two identical lines a deletion is attributed to.
// The walk takes a kept line whenever the two next lines are equal, and otherwise deletes before it inserts when the
// two choices keep as many lines. The common prefix and suffix are kept first, which the same walk would also do.
//
// WHAT COLLIDES. Two hunks of different revisions collide when they share a block line, or when one is an insertion
// and the other an insertion at the same gap (which added line comes first is undetermined) or a hunk that spans the
// gap (the lines around it are replaced or removed). A hunk that ends where another begins does not collide: the
// block's lines on either side of the boundary are different lines.
//
// THE ALIGNMENT IS BOUNDED. After the common prefix and suffix are cut, the table holds at most
// `ALIGNMENT_CELL_LIMIT` cells (each 4 bytes), so the time and the space are bounded by that limit whatever the
// revision says. A stretch that would need more is not aligned: it is one hunk from the first line the texts differ at
// to the last, which only makes more revisions collide and never changes the text a revision stands for. A block of
// 36 lines against a revision of 36 lines fills at most 1,369 cells.

/**
 One stretch of the block a revision replaces.
 */
export type LineHunk = {
  /**
   Zero-based position of the first block line replaced, or of the gap an insertion goes into.
   */
  readonly start: number;
  /**
   Zero-based position after the last block line replaced; equal to `start` for an insertion.
   */
  readonly end: number;
  /**
   Lines that take the replaced lines' place, none for a deletion.
   */
  readonly lines: readonly string[];
};

/**
 The most table cells one alignment fills, after the common prefix and suffix are cut.
 */
export const ALIGNMENT_CELL_LIMIT = 4_000_000;

/**
 Where the block's lines and one revision's lines collide with another revision's.
 */
export type HunkCollisions = {
  /**
   Zero-based block lines both revisions replace.
   */
  readonly lines: readonly number[];
  /**
   Gaps, as the count of block lines before them, where an insertion of one revision meets a hunk of the other that
   inserts at it or spans it.
   */
  readonly gaps: readonly number[];
};

/**
 Table of how many lines the rest of the block and the rest of the revision keep, row by block line.

 @param base - block lines left after the common prefix and suffix

 @param revision - revision lines left after them

 @returns Row-major table of `(base.length + 1) * (revision.length + 1)` cells; the cell at a block position and a
 revision position holds the longest common subsequence of what follows them

 @example
 ```ts
 keptTable({ base: ['a\n', 'b\n',], revision: ['b\n',], },); // Uint32Array [1, 1, 1, 0, 0, 0]
 ```
 */
function keptTable(
  {
    base,
    revision,
  }: {
    readonly base: readonly string[];
    readonly revision: readonly string[];
  },
): Uint32Array {
  /**
   Cells per row.
   */
  const width = revision.length + 1;
  /**
   Table whose last row and last column stay zero.
   */
  const table = new Uint32Array((base.length + 1) * width,);
  for (let row = base.length - 1; row >= 0; row -= 1) {
    for (let column = revision.length - 1; column >= 0; column -= 1) {
      table[(row * width) + column] = base[row] === revision[column]
        ? (table[((row + 1) * width) + column
          + 1] ?? 0) + 1
        : Math.max(
          table[((row + 1) * width) + column] ?? 0,
          table[(row * width) + column
            + 1] ?? 0,
        );
    }
  }
  return table;
}

/**
 The hunk a stretch of the walk closes, when it changed anything.

 @param revision - the revision's lines between the common prefix and suffix

 @param anchor - positions in the block's and the revision's lines after the last kept line pair

 @param here - positions the walk has reached in each

 @param prefix - count of lines both texts start with, which positions in the block are counted from

 @returns The one hunk from the anchor to here, none when both stayed where they were

 @example
 ```ts
 closedHunk({ revision: ['x\n',], anchor: { row: 0, column: 0, }, here: { row: 1, column: 1, }, prefix: 2, },); // [{ start: 2, end: 3, lines: ['x\n'] }]
 ```
 */
function closedHunk(
  {
    revision,
    anchor,
    here,
    prefix,
  }: {
    readonly revision: readonly string[];
    readonly anchor: {
      readonly row: number;
      readonly column: number
    };
    readonly here: {
      readonly row: number;
      readonly column: number
    };
    readonly prefix: number;
  },
): readonly LineHunk[] {
  return ((here.row === anchor.row) && (here.column === anchor.column)) ? [] : [{
    start: prefix + anchor.row,
    end: prefix + here.row,
    lines: revision.slice(
      anchor.column,
      here.column,
    ),
  },];
}

/**
 Walks the table from the start of both texts, keeping equal lines and collecting the stretches between them.

 @param base - the block's lines between the common prefix and suffix

 @param revision - the revision's lines between them

 @param prefix - count of lines both texts start with

 @returns The hunks in block order

 @example
 ```ts
 walkedHunks({ base: ['a\n',], revision: ['b\n',], prefix: 0, },); // [{ start: 0, end: 1, lines: ['b\n'] }]
 ```
 */
function walkedHunks(
  {
    base,
    revision,
    prefix,
  }: {
    readonly base: readonly string[];
    readonly revision: readonly string[];
    readonly prefix: number;
  },
): readonly LineHunk[] {
  /**
   What the rest of each text keeps, read forward to choose each step.
   */
  const table = keptTable({
    base,
    revision,
  },);
  /**
   Cells per row of the table.
   */
  const width = revision.length + 1;
  /**
   Hunks closed so far.
   */
  const hunks: LineHunk[] = [];
  /**
   Position in the block's lines.
   */
  let row = 0;
  /**
   Position in the revision's lines.
   */
  let column = 0;
  /**
   Block position after the last kept line pair.
   */
  let anchorRow = 0;
  /**
   Revision position after the last kept line pair.
   */
  let anchorColumn = 0;
  while ((row < base.length) || (column < revision.length)) {
    if ((row < base.length) && (column < revision.length)
      && (base[row] === revision[column])) {
      hunks.push(...closedHunk({
        revision,
        anchor: {
          row: anchorRow,
          column: anchorColumn,
        },
        here: {
          row,
          column,
        },
        prefix,
      },),);
      row += 1;
      column += 1;
      anchorRow = row;
      anchorColumn = column;
    }
    else if (
      (column >= revision.length)
        || ((row < base.length)
          && ((table[((row + 1) * width) + column] ?? 0) >= (table[(row * width) + column
            + 1] ?? 0)))
    ) {
      // A tie deletes before it inserts, and deleting is the only step once the revision's lines run out.
      row += 1;
    }
    else
      column += 1;
  }
  hunks.push(...closedHunk({
    revision,
    anchor: {
      row: anchorRow,
      column: anchorColumn,
    },
    here: {
      row,
      column,
    },
    prefix,
  },),);
  return hunks;
}

/**
 Reads the hunks that turn the block's lines into a revision's lines.

 @param base - lines of the block, each with the line ending that closes it

 @param revision - lines of the revision, cut the same way

 @param cellLimit - most table cells to fill after the common prefix and suffix are cut; a larger stretch is one hunk

 @returns The hunks in block order, empty when the lines are the same

 @example
 ```ts
 hunksBetween({ base: ['a\n', 'b\n',], revision: ['a\n', 'c\n', 'b\n',], },); // [{ start: 1, end: 1, lines: ['c\n'] }]
 ```
 */
export function hunksBetween(
  {
    base,
    revision,
    cellLimit = ALIGNMENT_CELL_LIMIT,
  }: {
    readonly base: readonly string[];
    readonly revision: readonly string[];
    readonly cellLimit?: number;
  },
): readonly LineHunk[] {
  /**
   Lines both texts start with: up to the first block line the revision does not repeat in place.
   */
  const firstDifference = base.findIndex(function differs(
    line,
    index,
  ): boolean {
    return line !== revision[index];
  },);
  /**
   Count of those lines.
   */
  const prefix = firstDifference === (-1) ? base.length : firstDifference;
  /**
   Block lines after the prefix.
   */
  const baseTail = base.slice(prefix,);
  /**
   Revision lines after the prefix.
   */
  const revisionTail = revision.slice(prefix,);
  /**
   Revision lines after the prefix, last first.
   */
  const revisionTailReversed = revisionTail.toReversed();
  /**
   Where, counting back from the end, the block and the revision first differ.
   */
  const lastDifference = baseTail
    .toReversed()
    .findIndex(function differs(
      line,
      index,
    ): boolean {
      return line !== revisionTailReversed[index];
    },);
  /**
   Count of lines both texts end with, after the prefix.
   */
  const suffix = lastDifference === (-1) ? baseTail.length : lastDifference;
  /**
   Block lines between the prefix and the suffix.
   */
  const middleBase = baseTail.slice(
    0,
    baseTail.length - suffix,
  );
  /**
   Revision lines between them.
   */
  const middleRevision = revisionTail.slice(
    0,
    revisionTail.length - suffix,
  );
  if ((middleBase.length === 0) && (middleRevision.length === 0))
    return [];
  // With nothing on one side, or a stretch the table could not hold, the whole stretch is one hunk: a coarser
  // alignment than the longest common subsequence, which can only make more revisions collide.
  if (
    (middleBase.length === 0)
      || (middleRevision.length === 0)
      || (((middleBase.length + 1) * (middleRevision.length + 1)) > cellLimit)
  ) {
    return [{
      start: prefix,
      end: prefix + middleBase.length,
      lines: middleRevision,
    },];
  }
  return walkedHunks({
    base: middleBase,
    revision: middleRevision,
    prefix,
  },);
}

/**
 Block lines a revision's hunks replace.

 @param hunks - one revision's hunks

 @returns Zero-based positions, ascending

 @example
 ```ts
 replacedLines({ hunks: [{ start: 2, end: 4, lines: [], },], },); // [2, 3]
 ```
 */
export function replacedLines({ hunks, }: { readonly hunks: readonly LineHunk[]; },): readonly number[] {
  return hunks.flatMap(function positionsOf({
    start,
    end,
  },): readonly number[] {
    return Array.from(
      { length: end - start, },
      function position(
        _unused,
        offset,
      ): number {
        return start + offset;
      },
    );
  },);
}

/**
 Gaps where an insertion of one revision meets a hunk of another that inserts at it or spans it.

 @param inserting - hunks whose insertions are tested

 @param meeting - hunks of the other revision

 @returns Gaps, as the count of block lines before them

 @example
 ```ts
 meetingGaps({ inserting: [{ start: 2, end: 2, lines: ['x\n',], },], meeting: [{ start: 1, end: 3, lines: [], },], },); // [2]
 ```
 */
function meetingGaps(
  {
    inserting,
    meeting,
  }: {
    readonly inserting: readonly LineHunk[];
    readonly meeting: readonly LineHunk[];
  },
): readonly number[] {
  return inserting
    .filter(function isInsertion({
      start,
      end,
    },): boolean {
      return start === end;
    },)
    .map(function gapOf({ start, },): number {
      return start;
    },)
    .filter(function metBy(gap,): boolean {
      return meeting.some(function holdsGap(hunk,): boolean {
        return hunk.start === hunk.end ? hunk.start === gap : (hunk.start < gap) && (gap < hunk.end);
      },);
    },);
}

/**
 Where the hunks of two revisions collide.

 @param left - first revision's hunks

 @param right - second revision's hunks

 @returns The block lines both replace and the gaps an insertion of either meets, each ascending and without repeats

 @example
 ```ts
 hunkCollisions({ left: [{ start: 1, end: 2, lines: ['x\n',], },], right: [{ start: 1, end: 3, lines: [], },], },); // { lines: [1], gaps: [] }
 ```
 */
export function hunkCollisions(
  {
    left,
    right,
  }: {
    readonly left: readonly LineHunk[];
    readonly right: readonly LineHunk[];
  },
): HunkCollisions {
  return {
    lines: sharedLines({
      left: replacedLines({ hunks: left, },),
      right: replacedLines({ hunks: right, },),
    },),
    gaps: [
      ...new Set([
        ...meetingGaps({
          inserting: left,
          meeting: right,
        },),
        ...meetingGaps({
          inserting: right,
          meeting: left,
        },),
      ],),
    ].toSorted(function ascending(
      first,
      second,
    ): number {
      return first - second;
    },),
  };
}

/**
 The place an insertion goes, as a log line says it.

 @param gap - count of block lines before the gap

 @returns "after line 3", or "before line 1" at the start

 @example
 ```ts
 placed({ gap: 3, },); // 'after line 3'
 ```
 */
export function placed({ gap, }: { readonly gap: number; },): string {
  return gap === 0 ? 'before line 1' : `after line ${String(gap,)}`;
}

/**
 Block lines as a log line says a stretch of them, counting from one.

 @param start - zero-based first line

 @param end - zero-based position after the last line

 @returns "line 4" for one line, "lines 3 to 5" for several

 @example
 ```ts
 stretchOf({ start: 2, end: 5, },); // 'lines 3 to 5'
 ```
 */
function stretchOf({
  start,
  end,
}: {
  readonly start: number;
  readonly end: number
},): string {
  return (end - start) === 1 ? `line ${String(start + 1,)}` : `lines ${String(start + 1,)} to ${String(end,)}`;
}

/**
 One hunk as a log line says what it does to the block.

 @param hunk - hunk of a revision

 @returns "removes line 3", "adds 2 lines after line 5" or "replaces line 3 with 1 line"

 @example
 ```ts
 describeHunk({ hunk: { start: 2, end: 2, lines: ['a\n',], }, },); // 'adds 1 line after line 2'
 ```
 */
export function describeHunk({ hunk, }: { readonly hunk: LineHunk; },): string {
  /**
   The replacement's size, as a count of lines.
   */
  const size = `${String(hunk.lines
    .length,)} ${hunk.lines
      .length
      === 1 ? 'line' : 'lines'}`;
  if (hunk.lines
    .length
    === 0)
    return `removes ${stretchOf(hunk,)}`;
  if (hunk.start === hunk.end)
    return `adds ${size} ${placed({ gap: hunk.start, },)}`;
  return `replaces ${stretchOf(hunk,)} with ${size}`;
}

/**
 Collisions of a pair as a log line says them.

 @param collisions - lines and gaps two revisions collide on

 @returns "line 4", "lines 2, 3 and the place after line 6", or the empty string when they do not collide

 @example
 ```ts
 describeCollisions({ collisions: { lines: [1,], gaps: [3,], }, },); // 'line 2 and the place after line 3'
 ```
 */
export function describeCollisions({ collisions, }: { readonly collisions: HunkCollisions; },): string {
  return [
    ...(collisions.lines
      .length
      === 0 ? [] : [numbered({ indexes: collisions.lines, },),]),
    ...collisions.gaps
      .map(function gapWords(gap,): string {
        return `the place ${placed({ gap, },)}`;
      },),
  ].join(' and ',);
}

//endregion Archive composite hunks
