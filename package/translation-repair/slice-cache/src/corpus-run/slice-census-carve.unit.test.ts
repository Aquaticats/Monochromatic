/**
 Tests for the census line that says which carve each row's sizes describe.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { sliceCensusCarveLine, } from '../../dist/final/node/index.mjs';
import { censusRowOf, } from './slice-census-row.test-fixture.ts';

await describe({
  name: sliceCensusCarveLine.name,
  children: [
    it({
      name: 'COUNTS NOTHING for no rows and says "entries" for the zero complete recipes',
      fn: async () => {
        expect(sliceCensusCarveLine({
          rows: [],
          legacyCount: 0,
        },),).toBe(
          'CENSUS carve: 0 settled entries with a complete recipe, 0 settled with a defaulted half, '
            + '0 settled with a recorded block pairing that does not fit the text, carved by the deterministic '
            + 'aligner, 0 deterministic baseline (0 of those hold a legacy artifact)',
        );
      },
    },),
    it({
      name: 'SAYS "entry" for exactly one complete recipe and counts the other carves apart',
      fn: async () => {
        expect(sliceCensusCarveLine({
          rows: [
            censusRowOf({
              entryId: 'mochi',
              measures: { carve: 'settled-complete', },
            },),
            censusRowOf({
              entryId: 'nori',
              measures: { carve: 'deterministic', },
            },),
            censusRowOf({
              entryId: 'tama',
              measures: { carve: 'deterministic', },
            },),
          ],
          legacyCount: 1,
        },),).toBe(
          'CENSUS carve: 1 settled entry with a complete recipe, 0 settled with a defaulted half, '
            + '0 settled with a recorded block pairing that does not fit the text, carved by the deterministic '
            + 'aligner, 2 deterministic baseline (1 of those hold a legacy artifact)',
        );
      },
    },),
    it({
      name: 'SAYS "entries" for several complete recipes and counts the defaulted half and the moved pairing',
      fn: async () => {
        expect(sliceCensusCarveLine({
          rows: [
            censusRowOf({
              entryId: 'mochi',
              measures: { carve: 'settled-complete', },
            },),
            censusRowOf({
              entryId: 'nori',
              measures: { carve: 'settled-complete', },
            },),
            censusRowOf({
              entryId: 'tama',
              measures: { carve: 'settled-partial', },
            },),
            censusRowOf({
              entryId: 'yuzu',
              measures: { carve: 'settled-moved', },
            },),
            censusRowOf({
              entryId: 'kuro',
              measures: { carve: 'settled-moved', },
            },),
            censusRowOf({
              entryId: 'sora',
              measures: { carve: 'settled-moved', },
            },),
          ],
          legacyCount: 0,
        },),).toBe(
          'CENSUS carve: 2 settled entries with a complete recipe, 1 settled with a defaulted half, '
            + '3 settled with a recorded block pairing that does not fit the text, carved by the deterministic '
            + 'aligner, 0 deterministic baseline (0 of those hold a legacy artifact)',
        );
      },
    },),
  ],
},);
