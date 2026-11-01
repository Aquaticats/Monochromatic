/**
 Tests for the refusals `scoreGradeAgreement` makes over a pre-grade file that
 names a sheet position more than once.

 The cases that score and compare whole draws live beside the sheet reader in
 `grade-sheet-read.unit.test.ts`; this file holds the repeated-position
 refusals, which that file's fixtures cannot build because they number
 positions from the array they are handed.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type GradedItem,
  scoreGradeAgreement,
} from '../dist/final/node/index.mjs';
import { statedRefusalMessage, } from '@monochromatic-dev/translation-repair-glossary/ts/stated-refusal-message.test-fixture.ts';

/**
 Builds one graded item with no note.

 @param index - sheet position

 @param verdict - verdict recorded at it

 @returns Graded item

 @example
 ```ts
 const item = gradedAt({ index: 1, verdict: 'real-defect', },);
 ```
 */
function gradedAt(
  {
    index,
    verdict,
  }: {
    readonly index: number;
    readonly verdict: GradedItem['verdict'];
  },
): GradedItem {
  return {
    index,
    verdict,
    note: '',
  };
}

await describe({
  name: scoreGradeAgreement.name,
  children: [
    it({
      name: 'REFUSES A PRE-GRADE FILE THAT NAMES ONE SHEET POSITION TWICE, naming the position, even when the '
        + 'file also carries one row too many so that its distinct positions match the sheet',
      fn: async () => {
        expect(statedRefusalMessage({
          read: function scoresRepeatedPosition(): unknown {
            return scoreGradeAgreement({
              agent: [
                gradedAt({ index: 1, verdict: 'real-defect', },),
                gradedAt({ index: 2, verdict: 'false-positive', },),
                gradedAt({ index: 2, verdict: 'real-defect', },),
              ],
              human: [
                gradedAt({ index: 1, verdict: 'real-defect', },),
                gradedAt({ index: 2, verdict: 'real-defect', },),
              ],
            },);
          },
        },),).toBe(
          'pre-grades name sheet position 2 more than once, so which verdict belongs to it cannot be read',
        );
      },
    },),
  ],
},);
