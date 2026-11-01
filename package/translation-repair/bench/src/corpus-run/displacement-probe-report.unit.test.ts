/**
 Tests for the lines the displacement probe logs once every entry is read:
 the totals, the defaulted halves, and one block per entry that carries
 anything.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { reportLines, } from '../../dist/final/node/index.mjs';
import { rowOf, } from './displacement-probe-rows.test-fixture.ts';

/**
 Totals lines of a report over nothing, which every report carries after its
 first two lines.
 */
const ZERO_TOTALS = [
  'slices read: 0',
  'entries falling back to the corpus baseline: 0',
  'relocation candidates: 0',
  '  of which a transcription would also explain: 0',
  'untranslated slices: 0',
  'target-only slices: 0',
  'other imbalances: 0',
];

await describe({
  name: reportLines.name,
  children: [
    it({
      name: 'REPORTS zero everywhere for a runs directory with no artifact, in the plural',
      fn: async () => {
        expect(reportLines({
          rows: [],
          artifactCount: 0,
          defaulted: [],
        },),).toEqual([
          'settled entries carved: 0 of 0 artifacts',
          '  with a defaulted recipe half: 0',
          ...ZERO_TOTALS,
        ],);
      },
    },),
    it({
      name: 'SAYS artifact in the singular for one artifact and prints no block for an entry nothing named',
      fn: async () => {
        expect(reportLines({
          rows: [rowOf({ changes: { sliceCount: 2, }, },),],
          artifactCount: 1,
          defaulted: [],
        },),).toEqual([
          'settled entries carved: 1 of 1 artifact',
          '  with a defaulted recipe half: 0',
          'slices read: 2',
          ...ZERO_TOTALS.slice(1,),
        ],);
      },
    },),
    it({
      name: 'NAMES every entry whose recipe had a defaulted half beside the halves, counting artifacts that did not carve',
      fn: async () => {
        expect(reportLines({
          rows: [],
          artifactCount: 3,
          defaulted: [
            {
              entryId: 'Mittens',
              label: 'deterministic default for blockPairing',
            },
            {
              entryId: 'Tabby',
              label: 'deterministic default for sectionPairing, blockPairing',
            },
          ],
        },),).toEqual([
          'settled entries carved: 0 of 3 artifacts',
          '  with a defaulted recipe half: 2',
          '  Mittens: deterministic default for blockPairing',
          '  Tabby: deterministic default for sectionPairing, blockPairing',
          ...ZERO_TOTALS,
        ],);
      },
    },),
    it({
      name: 'PRINTS a block for each entry a class named, with a line for every class that holds positions and none for the rest',
      fn: async () => {
        expect(reportLines({
          rows: [
            rowOf({ changes: {
              entryId: 'Mittens',
              sliceCount: 8,
              baseline: 2.8571,
              relocationCandidates: [
                {
                  high: 5,
                  low: 4,
                  surplus: 448,
                  deficit: 472,
                },
                {
                  high: 7,
                  low: 6,
                  surplus: 130,
                  deficit: 90,
                },
              ],
              transcriptionSuspects: [5,],
              untranslated: [1, 2,],
              targetOnly: [3,],
              otherImbalances: [9,],
            }, },),
            rowOf({ changes: {
              entryId: 'Tabby',
              sliceCount: 4,
              baselineFrom: 'corpus-reference',
              otherImbalances: [2,],
            }, },),
            rowOf({ changes: {
              entryId: 'Whiskers',
              sliceCount: 1,
            }, },),
          ],
          artifactCount: 3,
          defaulted: [],
        },),).toEqual([
          'settled entries carved: 3 of 3 artifacts',
          '  with a defaulted recipe half: 0',
          'slices read: 13',
          'entries falling back to the corpus baseline: 1',
          'relocation candidates: 2',
          '  of which a transcription would also explain: 1',
          'untranslated slices: 2',
          'target-only slices: 1',
          'other imbalances: 2',
          '  Mittens: baseline 2.86 (document)',
          '    relocation: 5->4(+448/-472) 7->6(+130/-90)',
          '    untranslated: 1 2',
          '    target-only: 3',
          '    transcription suspect: 5',
          '    other imbalance: 9',
          '  Tabby: baseline 2.86 (corpus-reference)',
          '    other imbalance: 2',
        ],);
      },
    },),
  ],
},);
