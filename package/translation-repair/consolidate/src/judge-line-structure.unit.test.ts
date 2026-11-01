/**
 Guards ledger S15 on the lane contest and the consolidate gate: a judge
 weighing candidates over a line-structured original is told what the line
 rule asks, and only there.

 The producers and editors were told to keep every original line and the
 translate slate carried the line criterion, but the two sheets deciding which
 rendering ships carried no line rule and no bilingual clause, so a candidate
 merging the lines a producer had been told to keep could win on wording.

 THE KEY NAMES THE RULE, since the question now differs: two slices with
 identical texts in different chunks can differ in whether the rule governs,
 and ballots bought without the clause must not answer for a sheet with it.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildLaneContestMessages,
  laneContestSliceKey,
} from '@monochromatic-dev/translation-repair-lane-contest';
import { buildConsolidateGateMessages, } from '../dist/final/node/index.mjs';

/**
 Wording that opens the judge line clause.
 */
const LINE_CLAUSE = 'THE ORIGINAL IS LINE-STRUCTURED';

/**
 A short poem about a cat, two lines each side.
 */
const verse = {
  sourceText: '猫在窗边\n等一场雨',
  incumbentText: 'The cat by the window waits for the rain',
} as const;

/**
 Lane contest system text for the verse slice.

 @param lineStructured - whether the line rule governs the slice

 @returns System half of the sheet
 */
function laneSystem({ lineStructured, }: { readonly lineStructured: boolean; },): string {
  return buildLaneContestMessages({
    subject: {
      ...verse,
      repairText: 'The cat by the window\nwaits for the rain',
      translateText: 'The cat by the window waits for the rain',
      lineStructured,
    },
  },).at(0,)?.content ?? '';
}

/**
 Consolidate gate system text for the verse slice.

 @param lineStructured - whether the line rule governs the slice

 @returns System half of the sheet
 */
function gateSystem({ lineStructured, }: { readonly lineStructured: boolean; },): string {
  return buildConsolidateGateMessages({
    subject: {
      ...verse,
      consolidatedText: 'The cat by the window\nwaits for the rain',
      standingText: 'The cat by the window waits for the rain',
      lineStructured,
    },
  },).at(0,)?.content ?? '';
}

await describe({
  name: 'judges read the line rule where it governs (ledger S15)',
  children: [
    it({
      name: 'TELLS the lane contest what a line-structured original asks, with the bilingual clause',
      fn: async () => {
        expect(laneSystem({ lineStructured: true, },),).toContain(LINE_CLAUSE,);
        expect(laneSystem({ lineStructured: true, },),).toContain('is ONE line whose English is already',);
      },
    },),
    it({
      name: 'TELLS the consolidate gate the same',
      fn: async () => {
        expect(gateSystem({ lineStructured: true, },),).toContain(LINE_CLAUSE,);
        expect(gateSystem({ lineStructured: true, },),).toContain('is ONE line whose English is already',);
      },
    },),
    it({
      name: 'LEAVES prose sheets without it, where line breaks are the page\'s wrap',
      fn: async () => {
        expect(laneSystem({ lineStructured: false, },).includes(LINE_CLAUSE,),).toBe(false,);
        expect(gateSystem({ lineStructured: false, },).includes(LINE_CLAUSE,),).toBe(false,);
      },
    },),
    it({
      name: 'KEYS a governed slice apart from the same texts ungoverned',
      fn: async () => {
        /**
         Key material shared by both slices.
         */
        const material = {
          runShape: '[]',
          sourceText: verse.sourceText,
          incumbentText: verse.incumbentText,
          incumbentKind: 'present',
          repairText: 'The cat by the window\nwaits for the rain',
          translateText: 'The cat by the window waits for the rain',
        } as const;
        expect(laneContestSliceKey({ ...material, lineStructured: true, },),)
          .not.toBe(laneContestSliceKey({ ...material, lineStructured: false, },),);
      },
    },),
  ],
},);
