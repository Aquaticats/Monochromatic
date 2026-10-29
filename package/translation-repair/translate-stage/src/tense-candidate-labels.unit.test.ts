/**
 Tests for the labels a judging sheet gives the texts on offer, which the sheet,
 the writer of the finding and its reader all spell.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CONTEST_TENSE_LABELS,
  shippedTenseLabelOf,
} from '../dist/final/node/index.mjs';

//region Tense candidate labels tests

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: shippedTenseLabelOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES a consolidation under the label the gate gave it and an incumbent that stood in under the archive\'s',
          fn: async () => {
            expect(shippedTenseLabelOf({ kind: 'consolidated', },),).toBe('CANDIDATE "consolidated"',);
            expect(shippedTenseLabelOf({ kind: 'incumbent', },),).toBe('ARCHIVE RENDERING',);
          },
        },),
      ],
    },),

    describe({
      name: 'CONTEST_TENSE_LABELS',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LISTS the three labels the lane contest sheet gives its texts, the archive first and the lanes after it',
          fn: async () => {
            expect(CONTEST_TENSE_LABELS,).toEqual([
              'ARCHIVE RENDERING',
              'CANDIDATE "repair"',
              'CANDIDATE "translate"',
            ],);
          },
        },),
      ],
    },),
  ],
},);

//endregion Tense candidate labels tests
