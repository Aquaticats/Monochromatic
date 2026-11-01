/**
 Tests the one shape test the block pairing reader, the section pairing
 reader and the slice cache share for a list of correspondences (audit area
 six).

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isIndexPairList, } from '../dist/final/node/index.mjs';

await describe({
  name: isIndexPairList.name,
  children: [
    it({
      name: 'ACCEPTS A LIST OF INTEGER PAIRS, the empty list included, and leaves range checks to the readers',
      fn: async () => {
        expect(isIndexPairList([{ source: 0, target: 0, }, { source: 2, target: 1, },],),).toBe(true,);
        expect(isIndexPairList([],),).toBe(true,);
        expect(isIndexPairList([{ source: -1, target: 9, },],),).toBe(true,);
      },
    },),
    it({
      name: 'REFUSES anything that is not a list, and any entry that is not an object naming two integers',
      fn: async () => {
        expect([
          undefined,
          null,
          { pairs: [], },
          [null,],
          [{ source: 0, },],
          [{ target: 0, },],
          [{ source: 0, target: 1.5, },],
          [{ source: '0', target: 0, },],
          [{ source: Number.NaN, target: 0, },],
          [{ source: 0, target: 0, }, 'cat',],
        ].map(function reads(value,): boolean {
          return isIndexPairList(value,);
        },),).toEqual(Array.from({ length: 10, }, function refused(): boolean {
          return false;
        },),);
      },
    },),
  ],
},);
