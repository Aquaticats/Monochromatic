/**
 Tests for the sum the census lines share.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { sliceCensusTotal, } from '../../dist/final/node/index.mjs';
import { censusRowOf, } from './slice-census-row.test-fixture.ts';

await describe({
  name: sliceCensusTotal.name,
  children: [
    it({
      name: 'ADDS NOTHING over no rows',
      fn: async () => {
        expect(sliceCensusTotal({
          rows: [],
          field: 'targetOnlyBlocks',
        },),).toBe(0,);
      },
    },),
    it({
      name: 'READS THE ONE ROW as its own count',
      fn: async () => {
        expect(sliceCensusTotal({
          rows: [
            censusRowOf({
              entryId: 'mochi',
              measures: { targetOnlyBlocks: 4, },
            },),
          ],
          field: 'targetOnlyBlocks',
        },),).toBe(4,);
      },
    },),
    it({
      name: 'ADDS THE COUNT of every row, and only the count the case reads',
      fn: async () => {
        expect(sliceCensusTotal({
          rows: [
            censusRowOf({
              entryId: 'mochi',
              measures: {
                targetOnlyBlocks: 4,
                targetOnlyChars: 100,
              },
            },),
            censusRowOf({
              entryId: 'nori',
              measures: {
                targetOnlyBlocks: 1,
                targetOnlyChars: 20,
              },
            },),
          ],
          field: 'targetOnlyChars',
        },),).toBe(120,);
      },
    },),
  ],
},);
