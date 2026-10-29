/**
 Tests for the census lines about blocks only the translation carries.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { sliceCensusTargetOnlyLines, } from '../../dist/final/node/index.mjs';
import { censusRowOf, } from './slice-census-row.test-fixture.ts';

await describe({
  name: sliceCensusTargetOnlyLines.name,
  children: [
    it({
      name: 'PRINTS ZERO TOTALS and an empty spread where no row carries a target-only block',
      fn: async () => {
        expect(sliceCensusTargetOnlyLines({
          rows: [
            censusRowOf({ entryId: 'mochi', },),
          ],
        },),).toEqual([
          'CENSUS target-only blocks: 0; entries: 0; chars: 0',
          'CENSUS target-only block chars: n 0, p50 0, p90 0, p99 0, max 0',
        ],);
      },
    },),
    it({
      name: 'PRINTS ZERO TOTALS for no rows',
      fn: async () => {
        expect(sliceCensusTargetOnlyLines({ rows: [], },),).toEqual([
          'CENSUS target-only blocks: 0; entries: 0; chars: 0',
          'CENSUS target-only block chars: n 0, p50 0, p90 0, p99 0, max 0',
        ],);
      },
    },),
    it({
      name: 'NAMES THE ONE ENTRY with its blocks and the spread of their sizes, leaving out an entry with none',
      fn: async () => {
        expect(sliceCensusTargetOnlyLines({
          rows: [
            censusRowOf({ entryId: 'mochi', },),
            censusRowOf({
              entryId: 'nori',
              measures: {
                targetOnlyBlocks: 1,
                targetOnlyChars: 44,
                targetOnlyBlockChars: [44,],
              },
            },),
          ],
        },),).toEqual([
          'CENSUS target-only blocks: 1; entries: 1; chars: 44',
          'CENSUS   nori: blocks 1, chars 44',
          'CENSUS target-only block chars: n 1, p50 44, p90 44, p99 44, max 44',
        ],);
      },
    },),
    it({
      name: 'NAMES ONLY THE THREE ENTRIES holding the most target-only characters, in that order, while the totals '
        + 'and the spread count all four',
      fn: async () => {
        expect(sliceCensusTargetOnlyLines({
          rows: [
            censusRowOf({
              entryId: 'mochi',
              measures: {
                targetOnlyBlocks: 1,
                targetOnlyChars: 10,
                targetOnlyBlockChars: [10,],
              },
            },),
            censusRowOf({
              entryId: 'nori',
              measures: {
                targetOnlyBlocks: 2,
                targetOnlyChars: 330,
                targetOnlyBlockChars: [30, 300,],
              },
            },),
            censusRowOf({
              entryId: 'tama',
              measures: {
                targetOnlyBlocks: 1,
                targetOnlyChars: 20,
                targetOnlyBlockChars: [20,],
              },
            },),
            censusRowOf({
              entryId: 'yuzu',
              measures: {
                targetOnlyBlocks: 1,
                targetOnlyChars: 40,
                targetOnlyBlockChars: [40,],
              },
            },),
          ],
        },),).toEqual([
          'CENSUS target-only blocks: 5; entries: 4; chars: 400',
          'CENSUS   nori: blocks 2, chars 330',
          'CENSUS   yuzu: blocks 1, chars 40',
          'CENSUS   tama: blocks 1, chars 20',
          'CENSUS target-only block chars: n 5, p50 30, p90 300, p99 300, max 300',
        ],);
      },
    },),
  ],
},);
