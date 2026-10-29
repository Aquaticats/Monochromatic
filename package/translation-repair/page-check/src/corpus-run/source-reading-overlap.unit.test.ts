/**
 Tests for which stretches of a text share a character with any of a list.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  overlapIndexOf,
  reachesAny,
} from '../../dist/final/node/index.mjs';

await describe({
  name: overlapIndexOf.name,
  children: [
    it({
      name: 'SAYS WHETHER A STRETCH SHARES A CHARACTER WITH ANY OF A LIST, in any order of the list',
      fn: async () => {
        const index = overlapIndexOf({
          ranges: [
            { start: 20, end: 30, },
            { start: 0, end: 5, },
            { start: 9, end: 9, },
          ],
        },);
        expect(reachesAny({ index, range: { start: 4, end: 6, }, },),).toBe(true,);
        expect(reachesAny({ index, range: { start: 5, end: 8, }, },),).toBe(false,);
        expect(reachesAny({ index, range: { start: 8, end: 10, }, },),).toBe(true,);
        expect(reachesAny({ index, range: { start: 9, end: 12, }, },),).toBe(false,);
        expect(reachesAny({ index, range: { start: 29, end: 40, }, },),).toBe(true,);
        expect(reachesAny({ index, range: { start: 30, end: 40, }, },),).toBe(false,);
        expect(reachesAny({ index: overlapIndexOf({ ranges: [], },), range: { start: 0, end: 9, }, },),).toBe(false,);
      },
    },),
  ],
},);
