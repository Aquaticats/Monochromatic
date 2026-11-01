/**
 Tests for ordering a pairing map into the shape that gets written down.

 WHAT THESE PIN is that the recorded order is SORTED rather than inherited.
 Insertion order happens to be section order today, because the shell walks
 aligned sections in order, so a test that only fed an already-ordered map
 would pass against a function that did no sorting at all.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { sectionPairingsOf, } from '../dist/final/node/index.mjs';

await describe({
  name: sectionPairingsOf.name,
  children: [
    it({
      name:
        'ORDERS BY SECTION rather than by insertion, so two runs that agreed the same pairings record the '
          + 'same bytes however the sections happened to be asked',
      fn: async () => {
        expect(sectionPairingsOf({
          blockPairings: new Map([
            [
              2,
              [{
                source: 0,
                target: 1,
              },],
            ],
            [
              0,
              [{
                source: 1,
                target: 0,
              },],
            ],
          ],),
        },),).toEqual([
          {
            sectionIndex: 0,
            pairs: [{
              source: 1,
              target: 0,
            },],
          },
          {
            sectionIndex: 2,
            pairs: [{
              source: 0,
              target: 1,
            },],
          },
        ],);
      },
    },),
    it({
      name:
        'ACCEPTS a map naming nothing and answers with a list naming nothing, which is the recorded form of '
          + 'a roster asked about every section that committed to none of them',
      fn: async () => {
        expect(sectionPairingsOf({ blockPairings: new Map(), },),).toEqual([],);
      },
    },),
    it({
      name:
        'KEEPS a section whose pairs are empty, since the roster having been asked and agreed nothing is '
          + 'a different record from the section never having been asked at all',
      fn: async () => {
        expect(sectionPairingsOf({
          blockPairings: new Map([[
            1,
            [],
          ],],),
        },),).toEqual([{
          sectionIndex: 1,
          pairs: [],
        },],);
      },
    },),
  ],
},);
