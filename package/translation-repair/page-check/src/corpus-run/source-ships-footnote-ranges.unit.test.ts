/**
 Tests the join of the footnote ranges of a reader of the source with those only the page starts.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { withPageOnlyStarts, } from '../../dist/final/node/index.mjs';

await describe({
  name: withPageOnlyStarts.name,
  children: [
    it({
      name: 'ADDS A PAGE RANGE THAT STARTS WHERE NO RANGE OF THE SOURCE STANDS, in text order',
      fn: async () => {
        expect(withPageOnlyStarts({
          source: [
            { start: 0, end: 5, },
            { start: 20, end: 30, },
          ],
          page: [
            { start: 9, end: 15, },
          ],
        },),).toEqual([
          { start: 0, end: 5, },
          { start: 9, end: 15, },
          { start: 20, end: 30, },
        ],);
      },
    },),
    it({
      name: 'KEEPS THE SOURCE\'S RANGE WHERE THE PAGE STARTS ONE AT THE SAME PLACE OR INSIDE IT',
      fn: async () => {
        expect(withPageOnlyStarts({
          source: [
            { start: 0, end: 5, },
          ],
          page: [
            { start: 0, end: 12, },
            { start: 3, end: 4, },
          ],
        },),).toEqual([
          { start: 0, end: 5, },
        ],);
      },
    },),
    it({
      name: 'LEAVES THE SOURCE\'S RANGES AS THEY ARE WHERE THE PAGE HAS NONE, and the page\'s where the source has none',
      fn: async () => {
        expect(withPageOnlyStarts({ source: [{ start: 1, end: 2, },], page: [], },),).toEqual([{ start: 1, end: 2, },],);
        expect(withPageOnlyStarts({ source: [], page: [{ start: 9, end: 23, },], },),).toEqual([{ start: 9, end: 23, },],);
        expect(withPageOnlyStarts({ source: [], page: [], },),).toEqual([],);
      },
    },),
  ],
},);
