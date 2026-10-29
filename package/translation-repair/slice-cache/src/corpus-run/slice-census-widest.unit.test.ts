/**
 Tests for the census lines about the largest slices.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { sliceCensusWidestLines, } from '../../dist/final/node/index.mjs';
import { censusRowOf, } from './slice-census-row.test-fixture.ts';

await describe({
  name: sliceCensusWidestLines.name,
  children: [
    it({
      name: 'PRINTS ZERO OF ZERO and no entry for no rows',
      fn: async () => {
        expect(sliceCensusWidestLines({ rows: [], },),).toEqual([
          'CENSUS slices over 4641 target chars: 0 of 0',
        ],);
      },
    },),
    it({
      name: 'COUNTS ONLY TARGET SLICES LARGER than 4641 characters, so a slice of exactly 4641 is not over, and '
        + 'reads the larger side of an entry for its widest slice',
      fn: async () => {
        expect(sliceCensusWidestLines({
          rows: [
            censusRowOf({
              entryId: 'mochi',
              measures: {
                sliceSourceChars: [9_000, 20,],
                sliceTargetChars: [4_641, 4_642,],
              },
            },),
          ],
        },),).toEqual([
          'CENSUS slices over 4641 target chars: 1 of 2',
          'CENSUS   widest mochi: chars in one slice: 9000',
        ],);
      },
    },),
    it({
      name: 'READS AN ENTRY WITH NO SLICE as zero wide',
      fn: async () => {
        expect(sliceCensusWidestLines({
          rows: [
            censusRowOf({ entryId: 'nori', },),
          ],
        },),).toEqual([
          'CENSUS slices over 4641 target chars: 0 of 0',
          'CENSUS   widest nori: chars in one slice: 0',
        ],);
      },
    },),
    it({
      name: 'NAMES ONLY THE THREE WIDEST ENTRIES, widest first, while the count covers all four',
      fn: async () => {
        expect(sliceCensusWidestLines({
          rows: [
            censusRowOf({
              entryId: 'mochi',
              measures: {
                sliceTargetChars: [300,],
              },
            },),
            censusRowOf({
              entryId: 'nori',
              measures: {
                sliceTargetChars: [100,],
              },
            },),
            censusRowOf({
              entryId: 'tama',
              measures: {
                sliceTargetChars: [5_000,],
              },
            },),
            censusRowOf({
              entryId: 'yuzu',
              measures: {
                sliceTargetChars: [200,],
              },
            },),
          ],
        },),).toEqual([
          'CENSUS slices over 4641 target chars: 1 of 4',
          'CENSUS   widest tama: chars in one slice: 5000',
          'CENSUS   widest mochi: chars in one slice: 300',
          'CENSUS   widest yuzu: chars in one slice: 200',
        ],);
      },
    },),
  ],
},);
