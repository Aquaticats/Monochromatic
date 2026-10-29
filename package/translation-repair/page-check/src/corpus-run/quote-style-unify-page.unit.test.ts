/**
 Tests that the page-wide quote style pass reads the page as a page (n159, X2, X5, X8 and X11). A reference or a footnote
 reference whose definition stands in another slice is curled at neither end, a slice's prose mask is read once for the tally and
 the rewrite, and a slice the grammar refuses is named in a warning instead of passing in silence.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass';
import { proseMaskBuilds, } from '@monochromatic-dev/translation-repair-typography';
import { unifyQuoteStyle, } from '../../dist/final/node/index.mjs';
import { warnLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import { pairAt, } from '@monochromatic-dev/translation-repair-slice-reading/ts/corpus-run/archive-slice-pair.test-fixture.ts';

/**
 A lane's text that writes the page's marks curly, which the archive slices stand beside.
 */
const CURLY_LANE = '“Purr,” said the cat. “Warm.” “Soft.” It’s the cat’s nap.';

/**
 What the pass writes for each slice of a page, the last slice replaced by a lane.

 @param archives - archive text of every slice but the last, in order

 @param lane - text a lane wrote for the last slice

 @returns Text the page carries at each slice after the pass

 @example
 ```ts
 const texts = pageAfter({ archives: ['The cat said "meow".',], lane: CURLY_LANE, },);
 ```
 */
function pageAfter(
  {
    archives,
    lane,
  }: {
    readonly archives: readonly string[];
    readonly lane: string;
  },
): readonly string[] {
  /**
   Where the next slice starts.
   */
  const cursor = { at: 0, };
  /**
   The prepared pairs, the last one a slice a lane replaces.
   */
  const slices = [...archives, 'The cat napped.',].map(function pairOf(
    target,
    sliceIndex,
  ) {
    /**
     The pair at this slice.
     */
    const pair = pairAt({
      sliceIndex,
      target,
      startOffset: cursor.at,
    },);
    cursor.at += target.length + 2;
    return pair;
  },);
  /**
   Lane rows: the last slice alone.
   */
  const replacements: readonly SliceReplacement[] = [{
    sliceIndex: archives.length,
    replacementText: lane,
  },];
  /**
   The page after the pass.
   */
  const unified = unifyQuoteStyle({
    slices,
    replacements,
    archiveOriginalSpans: [],
  },);
  return slices.map(function textOf(slice,): string {
    return unified.replacements
      .find(function isSlice(row,): boolean {
        return row.sliceIndex === slice.target
          .sliceIndex;
      },)
      ?.replacementText ?? slice.target
        .text;
  },);
}

await describe({
  name: 'unifyQuoteStyle reads the page as a page (n159)',
  // One case diverts the console, which is process-wide.
  concurrency: 1,
  children: [
    it({
      name: 'CURLS A REFERENCE AT NEITHER END WHEN ITS DEFINITION STANDS IN ANOTHER SLICE (X11): the label keeps its straight mark '
        + 'where the reference and the definition both stand, and the prose beside the reference curls',
      fn: async () => {
        expect(pageAfter({
          archives: ['See [the dogs][b\'s] and it\'s.', '[b\'s]: https://x.example/b',],
          lane: CURLY_LANE,
        },).slice(0, 2,),).toEqual(['See [the dogs][b\'s] and it’s.', '[b\'s]: https://x.example/b',],);
      },
    },),
    it({
      name: 'CURLS A FOOTNOTE REFERENCE AT NEITHER END WHEN ITS DEFINITION STANDS IN ANOTHER SLICE (X11)',
      fn: async () => {
        expect(pageAfter({
          archives: ['See the note[^o\'k] and it\'s.', '[^o\'k]: A note.',],
          lane: CURLY_LANE,
        },).slice(0, 2,),).toEqual(['See the note[^o\'k] and it’s.', '[^o\'k]: A note.',],);
      },
    },),
    it({
      name: 'CURLS A SHORTCUT REFERENCE AT NEITHER END WHEN ITS DEFINITION STANDS IN ANOTHER SLICE (X11)',
      fn: async () => {
        expect(pageAfter({
          archives: ['See [dogs\' bowl] and it\'s.', '[dogs\' bowl]: https://x.example/d',],
          lane: CURLY_LANE,
        },).slice(0, 2,),).toEqual(['See [dogs\' bowl] and it’s.', '[dogs\' bowl]: https://x.example/d',],);
      },
    },),
    it({
      name: 'READS THE PROSE MASK OF EACH SLICE ONCE FOR THE TALLY AND THE REWRITE (X5), also on a page of more slices than the '
        + 'memory of masks holds',
      fn: async () => {
        /**
         Archive slices with a straight apostrophe each, every text distinct so the memory of masks cannot answer for one.
         */
        const archives = Array.from(
          { length: 40, },
          function archiveOf(
            _unused,
            index,
          ): string {
            return `Slice ${String(index,)} says it's here.`;
          },
        );
        /**
         A lane text with many curly double marks, so the page's style is curly and every slice is rewritten.
         */
        const lane = '“Purr,” said the cat. '.repeat(50,);
        /**
         The masks built before the pass.
         */
        const before = proseMaskBuilds();
        pageAfter({
          archives,
          lane,
        },);
        expect(proseMaskBuilds() - before,).toBe(archives.length + 1,);
      },
    },),
    it({
      name: 'WARNS OF A SLICE THE GRAMMAR REFUSES, naming the slice, and leaves its marks as written (X8)',
      fn: async () => {
        /**
         The page after the pass and what the pass warned.
         */
        const {
          result,
          warned,
        } = await warnLinesDuring({
          run: async () => pageAfter({
            archives: ['Oops {brace it\'s here.', 'The cat said "meow" and it\'s fine.',],
            lane: CURLY_LANE,
          },),
        },);
        expect({
          first: nonNullishOrThrow(result[0],),
          warnedOfSliceZero: warned.filter(function namesSlice(line,): boolean {
            return line.includes('slice 0',);
          },).length,
          warnedOfSliceOne: warned.filter(function namesSlice(line,): boolean {
            return line.includes('slice 1',);
          },).length,
        },).toEqual({
          first: 'Oops {brace it\'s here.',
          warnedOfSliceZero: 1,
          warnedOfSliceOne: 0,
        },);
      },
    },),
  ],
},);
