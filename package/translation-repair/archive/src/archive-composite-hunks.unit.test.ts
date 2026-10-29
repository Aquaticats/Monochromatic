/**
 Tests the line diff behind the composite candidate of the archive block review (ledger B803): the hunks a revision
 makes to a block, where the hunks of two revisions collide, how they are worded in a log line, and how several that
 do not collide are applied together.

 Fixtures are invented lines about a cat; no corpus text appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ALIGNMENT_CELL_LIMIT,
  describeCollisions,
  describeHunk,
  hunkCollisions,
  hunksBetween,
  type LineHunk,
  placed,
  replacedLines,
  spliceHunks,
} from '../dist/final/node/index.mjs';

/**
 Lines of a block with their endings, the last without one, as a block is cut.

 @param names - lines' words

 @returns Lines

 @example
 ```ts
 linesNamed({ names: ['a', 'b',], },); // ['a\n', 'b']
 ```
 */
function linesNamed({ names, }: { readonly names: readonly string[]; },): readonly string[] {
  return names.map(function ended(name, index,): string {
    return index === (names.length - 1) ? name : `${name}\n`;
  },);
}

/**
 The most lines both texts keep, counted by the plain table, which the diff's own alignment is held against.

 @param left - first lines

 @param right - second lines

 @returns Length of the longest common subsequence

 @example
 ```ts
 keptCount({ left: ['a', 'b',], right: ['b',], },); // 1
 ```
 */
function keptCount(
  {
    left,
    right,
  }: {
    readonly left: readonly string[];
    readonly right: readonly string[];
  },
): number {
  /**
   Row of counts for the left lines read so far.
   */
  const rows = left.reduce(
    function nextRow(previous: readonly number[], line,): readonly number[] {
      return right.reduce(
        function nextCell(row: readonly number[], other, index,): readonly number[] {
          return [
            ...row,
            line === other
              ? (previous[index] ?? 0) + 1
              : Math.max(previous[index + 1] ?? 0, row[index] ?? 0,),
          ];
        },
        [0,],
      );
    },
    [
      ...right.map(function zero(): number {
        return 0;
      },),
      0,
    ],
  );
  return rows.at(-1,) ?? 0;
}

/**
 A pseudo-random sequence of lines from a small alphabet, the same for the same seed.

 @param seed - starting state

 @param length - how many lines

 @returns Lines, each ending in a line feed

 @example
 ```ts
 randomLines({ seed: 7, length: 3, },); // three lines of 'a\n', 'b\n' or 'c\n'
 ```
 */
function randomLines({ seed, length, }: { readonly seed: number; readonly length: number; },): readonly string[] {
  return Array.from({ length, }, function pick(_unused, index,): string {
    return `${'abc'[Math.floor((((seed * 7_919) + (index * 104_729) + (seed * index * 31)) % 97) / 33)] ?? 'a'}\n`;
  },);
}

await describe({
  name: 'archive composite hunks',
  children: [
    it({
      name: 'READS NO HUNK BETWEEN THE SAME LINES, and one hunk between a block and a revision of one of its lines',
      fn: async () => {
        const base = linesNamed({ names: ['cat', 'dog', 'bird',], },);
        expect(hunksBetween({ base, revision: base, },),).toEqual([],);
        expect(hunksBetween({
          base,
          revision: linesNamed({ names: ['cat', 'hound', 'bird',], },),
        },),).toEqual([{ start: 1, end: 2, lines: ['hound\n',], },],);
      },
    },),
    it({
      name: 'READS AN ADDED LINE AS AN INSERTION AT A GAP, at the start, between lines and after the last line (which '
        + 'has no ending, so it is a replacement of that line)',
      fn: async () => {
        const base = linesNamed({ names: ['cat', 'dog', 'bird',], },);
        expect(hunksBetween({
          base,
          revision: linesNamed({ names: ['fish', 'cat', 'dog', 'bird',], },),
        },),).toEqual([{ start: 0, end: 0, lines: ['fish\n',], },],);
        expect(hunksBetween({
          base,
          revision: linesNamed({ names: ['cat', 'fish', 'dog', 'bird',], },),
        },),).toEqual([{ start: 1, end: 1, lines: ['fish\n',], },],);
        expect(hunksBetween({
          base,
          revision: linesNamed({ names: ['cat', 'dog', 'bird', 'fish',], },),
        },),).toEqual([{ start: 2, end: 3, lines: ['bird\n', 'fish',], },],);
      },
    },),
    it({
      name: 'READS A REMOVED LINE AS A HUNK WITH NO LINES, in the middle and at the end',
      fn: async () => {
        const base = linesNamed({ names: ['cat', 'dog', 'bird', 'fish',], },);
        expect(hunksBetween({
          base,
          revision: linesNamed({ names: ['cat', 'bird', 'fish',], },),
        },),).toEqual([{ start: 1, end: 2, lines: [], },],);
        expect(hunksBetween({
          base,
          revision: ['cat\n', 'dog\n', 'bird\n',],
        },),).toEqual([{ start: 3, end: 4, lines: [], },],);
      },
    },),
    it({
      name: 'READS ONE REVISION THAT ADDS AND REMOVES AS SEVERAL HUNKS in block order, each between lines it keeps',
      fn: async () => {
        const base = linesNamed({ names: ['cat', 'dog', 'bird', 'fish',], },);
        expect(hunksBetween({
          base,
          revision: linesNamed({ names: ['cat', 'bird', 'owl', 'fish',], },),
        },),).toEqual([
          { start: 1, end: 2, lines: [], },
          { start: 3, end: 3, lines: ['owl\n',], },
        ],);
      },
    },),
    it({
      name: 'DELETES BEFORE IT INSERTS WHEN BOTH KEEP AS MANY LINES: a block turned into different lines is one '
        + 'replacement, and a moved line is a removal and an insertion',
      fn: async () => {
        expect(hunksBetween({
          base: ['a\n', 'b\n',],
          revision: ['c\n', 'd\n',],
        },),).toEqual([{ start: 0, end: 2, lines: ['c\n', 'd\n',], },],);
        expect(hunksBetween({
          base: ['a\n', 'b\n', 'c\n',],
          revision: ['a\n', 'c\n', 'b\n',],
        },),).toEqual([
          { start: 1, end: 2, lines: [], },
          { start: 3, end: 3, lines: ['b\n',], },
        ],);
      },
    },),
    it({
      name: 'ATTRIBUTES A DELETION AMONG REPEATED BLANK LINES THE SAME WAY EVERY TIME: lines both texts share at the '
        + 'start are kept first, so removing a paragraph with a blank line beside it removes the blank line after it',
      fn: async () => {
        /** Three paragraphs with a blank quote line between each. */
        const base = ['>a\n', '>\n', '>b\n', '>\n', '>c',];
        const revision = ['>a\n', '>\n', '>c',];
        expect(hunksBetween({ base, revision, },),).toEqual([{ start: 2, end: 4, lines: [], },],);
        expect(hunksBetween({ base, revision, },),).toEqual(hunksBetween({ base, revision, },),);
      },
    },),
    it({
      name: 'ROUND-TRIPS: applying the hunks of a revision to the block gives the revision, and the lines the hunks '
        + 'leave are as many as the plain longest-common-subsequence table keeps, over generated lines',
      fn: async () => {
        for (let seed = 1; seed <= 120; seed += 1) {
          const base = randomLines({ seed, length: 1 + (seed % 9), },);
          const revision = randomLines({ seed: seed * 3, length: seed % 8, },);
          const hunks = hunksBetween({ base, revision, },);
          expect(spliceHunks({ base, revisions: [hunks,], },).lines.join('',),).toBe(revision.join('',),);
          const replaced = replacedLines({ hunks, },).length;
          expect(base.length - replaced,).toBe(keptCount({ left: base, right: revision, },),);
        }
      },
    },),
    it({
      name: 'MAKES ONE HUNK OF A STRETCH THE TABLE COULD NOT HOLD, from the first line the texts differ at to the '
        + 'last, which still gives the revision back and collides with more',
      fn: async () => {
        const base = linesNamed({ names: ['a', 'b', 'c', 'd', 'e',], },);
        const revision = linesNamed({ names: ['a', 'X', 'c', 'Y', 'e',], },);
        expect(hunksBetween({ base, revision, },),).toEqual([
          { start: 1, end: 2, lines: ['X\n',], },
          { start: 3, end: 4, lines: ['Y\n',], },
        ],);
        /** Hunks read with a table too small for the stretch. */
        const coarse = hunksBetween({ base, revision, cellLimit: 4, },);
        expect(coarse,).toEqual([{ start: 1, end: 4, lines: ['X\n', 'c\n', 'Y\n',], },],);
        expect(spliceHunks({ base, revisions: [coarse,], },).lines.join('',),).toBe(revision.join('',),);
      },
    },),
    it({
      name: `BOUNDS THE TABLE: a block of 1,500 lines that differ in every other line is aligned into one hunk for `
        + `each differing line, and one of 2,100 is cut to a single hunk, the cell limit being ${
         String(ALIGNMENT_CELL_LIMIT,)}`,
      fn: async () => {
        /**
         Lines named by position, the odd ones carrying the given word.
         */
        function alternating({ count, odd, }: { readonly count: number; readonly odd: string; }): readonly string[] {
          return Array.from({ length: count, }, function line(_unused, index,): string {
            return `${(index % 2) === 0 ? 'old' : odd} ${String(index,)}\n`;
          },);
        }
        /** Hunks of a block that fits the table: one for each of the 750 changed lines. */
        const within = hunksBetween({
          base: alternating({ count: 1_500, odd: 'old', }),
          revision: alternating({ count: 1_500, odd: 'new', }),
        },);
        expect(within,).toHaveLength(750,);
        /** Hunks of a block that needs more cells than the limit allows. */
        const beyond = hunksBetween({
          base: alternating({ count: 2_100, odd: 'old', }),
          revision: alternating({ count: 2_100, odd: 'new', }),
        },);
        expect(beyond,).toHaveLength(1,);
        expect(beyond[0]?.start,).toBe(1,);
        expect(beyond[0]?.end,).toBe(2_100,);
        expect(spliceHunks({
          base: alternating({ count: 2_100, odd: 'old', }),
          revisions: [beyond,],
        },).lines,).toEqual(alternating({ count: 2_100, odd: 'new', }),);
      },
    },),
    it({
      name: 'FINDS NO COLLISION BETWEEN HUNKS THAT ONLY TOUCH: a hunk that ends where another begins, and an '
        + 'insertion at the edge of a hunk, share no block line and no gap inside either',
      fn: async () => {
        /** Replacement of lines 2 and 3. */
        const middle: LineHunk = { start: 1, end: 3, lines: ['x\n',], };
        expect(hunkCollisions({
          left: [middle,],
          right: [{ start: 3, end: 4, lines: [], },],
        },),).toEqual({ lines: [], gaps: [], },);
        expect(hunkCollisions({
          left: [middle,],
          right: [{ start: 1, end: 1, lines: ['a\n',], },],
        },),).toEqual({ lines: [], gaps: [], },);
        expect(hunkCollisions({
          left: [middle,],
          right: [{ start: 3, end: 3, lines: ['a\n',], },],
        },),).toEqual({ lines: [], gaps: [], },);
      },
    },),
    it({
      name: 'FINDS THE BLOCK LINES TWO REVISIONS BOTH REPLACE, even when both replace them with the same text',
      fn: async () => {
        expect(hunkCollisions({
          left: [{ start: 1, end: 4, lines: ['x\n',], },],
          right: [{ start: 3, end: 6, lines: [], },],
        },),).toEqual({ lines: [3,], gaps: [], },);
        expect(hunkCollisions({
          left: [{ start: 2, end: 3, lines: ['x\n',], },],
          right: [{ start: 2, end: 3, lines: ['x\n',], },],
        },),).toEqual({ lines: [2,], gaps: [], },);
      },
    },),
    it({
      name: 'FINDS THE GAP TWO INSERTIONS SHARE AND THE GAP A HUNK SPANS, whichever revision inserts',
      fn: async () => {
        expect(hunkCollisions({
          left: [{ start: 2, end: 2, lines: ['a\n',], },],
          right: [{ start: 2, end: 2, lines: ['b\n',], },],
        },),).toEqual({ lines: [], gaps: [2,], },);
        expect(hunkCollisions({
          left: [{ start: 1, end: 3, lines: [], },],
          right: [{ start: 2, end: 2, lines: ['b\n',], },],
        },),).toEqual({ lines: [], gaps: [2,], },);
        expect(hunkCollisions({
          left: [{ start: 2, end: 2, lines: ['b\n',], },],
          right: [{ start: 1, end: 3, lines: ['y\n',], },],
        },),).toEqual({ lines: [], gaps: [2,], },);
      },
    },),
    it({
      name: 'WORDS A HUNK AND A COLLISION AS A LOG LINE SAYS THEM, counting lines from one',
      fn: async () => {
        expect(describeHunk({ hunk: { start: 2, end: 3, lines: [], }, },),).toBe('removes line 3',);
        expect(describeHunk({ hunk: { start: 2, end: 5, lines: [], }, },),).toBe('removes lines 3 to 5',);
        expect(describeHunk({ hunk: { start: 0, end: 0, lines: ['a\n', 'b\n',], }, },),).toBe(
          'adds 2 lines before line 1',
        );
        expect(describeHunk({ hunk: { start: 4, end: 4, lines: ['a\n',], }, },),).toBe('adds 1 line after line 4',);
        expect(describeHunk({ hunk: { start: 1, end: 3, lines: ['a\n',], }, },),).toBe(
          'replaces lines 2 to 3 with 1 line',
        );
        expect(placed({ gap: 0, },),).toBe('before line 1',);
        expect(describeCollisions({ collisions: { lines: [1, 2,], gaps: [5,], }, },),).toBe(
          'lines 2, 3 and the place after line 5',
        );
        expect(describeCollisions({ collisions: { lines: [], gaps: [], }, },),).toBe('',);
      },
    },),
  ],
},);
