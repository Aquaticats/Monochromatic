/**
 Guards class forty-eight (shi_Yumiaoya2, 2026-09-17): a source-only passage
 whose coverage round split with no anchored claim of coverage at all, only
 absent votes and a quote nobody could find on the page, joins the absent
 passages before the whole-page shortfall corroboration instead of shipping
 as a silent gap. A split that carries an anchored claim stays unresolved.
 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readUntranslatedTail, } from '@monochromatic-dev/translation-repair-archive';
import {
  classifyInsertionCoverage,
  type InsertionCoverageRow,
} from '../../dist/final/node/index.mjs';

/**
 Original passage the archive never rendered.
 */
const PASSAGE = '猫的胶片相机交给了阿花，那台老相机将替猫继续看这个世界。猫的数码相机由前主人收藏。猫的裙子和手机由小雪继承。猫的花交给了小由。';

/**
 Row shape shared by every case.
 */
const ROW: InsertionCoverageRow = {
  position: 3,
  sliceIndex: 13,
  sourceText: PASSAGE,
  frontMatter: false,
  verdictKind: 'split',
  anchoredFull: 0,
  anchoredPartial: 0,
  absentCount: 1,
  heard: 2,
  asked: 2,
  missingDestinationCount: 0,
  coverageFinding: 'insertion-coverage (slice 13, verdict split, full 0, partial 0, absent 1, heard 2 of 2)',
  stageFindings: [],
  coverageEvidence: [],
  destinationFindings: [],
};

/**
 Classifies one row against a skeleton page that has room for the passage.

 @param row - coverage row under test

 @returns Classification of that one candidate

 @example
 ```ts
 const state = classify({ row: ROW, },);
 ```
 */
function classify({ row, }: { readonly row: InsertionCoverageRow; },) {
  return classifyInsertionCoverage({
    candidates: [{
      position: row.position,
      sliceIndex: row.sliceIndex,
      sourceText: row.sourceText,
      frontMatter: false,
    },],
    rows: [row,],
    frontMatterPositions: new Set(),
    sourceText: `## 猫\n\n${PASSAGE}\n`,
    targetText: '## Cat\n',
    tail: readUntranslatedTail({ slices: [], },),
  },);
}

await describe({
  name: 'a split coverage verdict with no anchored claim (class forty-eight)',
  children: [
    it({
      name: 'ADMITS the passage by the shortfall when only absent votes and unanchorable quotes were heard',
      fn: async () => {
        const state = classify({ row: ROW, },);
        expect([...state.positions,],).toEqual([3,],);
        expect(state.unresolvedRows,).toEqual([],);
        expect(state.findings.join('\n',),).toContain('slice 13',);
      },
    },),
    it({
      name: 'LEAVES a split unresolved where one voice anchored a claim of coverage',
      fn: async () => {
        const state = classify({ row: {
          ...ROW,
          anchoredPartial: 1,
        }, },);
        expect([...state.positions,],).toEqual([],);
        expect(state.unresolvedRows.length,).toBe(1,);
      },
    },),
    it({
      name: 'LEAVES a split unresolved where no voice voted absent either',
      fn: async () => {
        const state = classify({ row: {
          ...ROW,
          absentCount: 0,
        }, },);
        expect([...state.positions,],).toEqual([],);
      },
    },),
  ],
},);
