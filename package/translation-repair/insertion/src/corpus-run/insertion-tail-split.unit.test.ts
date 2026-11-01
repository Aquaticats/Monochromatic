/**
 Guards class sixty (XingZ610, 2026-09-19): a source-only passage standing in
 an untranslated tail the bound admits, whose coverage round split on one
 minority anchored claim, is admitted on the bound like its absent
 neighbours. The tail rule admitted only absent verdicts and unanchored
 splits; three tail slices (two paragraphs and a footnote definition, the
 definition split on every run) stayed unresolved on one voice quoting an
 archive line the pairing never assigned, and shipped as gaps. A verdict a
 majority carried still stays out. Cat-themed invention throughout; no
 corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readUntranslatedTail, } from '@monochromatic-dev/translation-repair-archive';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  classifyInsertionCoverage,
  type InsertionCoverageRow,
} from '../../dist/final/node/index.mjs';
import { insertion, } from '@monochromatic-dev/translation-repair-slice-reading/ts/corpus-run/insertion-only-slice.test-fixture.ts';

/**
 Source paragraph the archive translated, the last agreed pair.
 */
const PAIRED_SOURCE = '橘猫在窗台上睡了整个下午。';

/**
 Rendering of that paragraph, three code points per source code point.
 */
const PAIRED_TARGET = 'x'.repeat(39,);

/**
 One tail paragraph the archive never rendered.
 */
const TAIL_SOURCE = '猫在夜里回家了。';

/**
 Builds the one paired slice.

 @returns Slice paired with its rendering

 @example
 ```ts
 const slice = pairedSlice();
 ```
 */
function pairedSlice(): ChunkPair {
  return {
    source: {
      kind: 'content',
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: PAIRED_SOURCE.length,
      text: PAIRED_SOURCE,
    },
    target: {
      kind: 'content',
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: PAIRED_TARGET.length,
      text: PAIRED_TARGET,
    },
  };
}

/**
 Row for one tail slice, before its verdict is set.

 @param position - where the row stands

 @returns Row absent by every voice heard

 @example
 ```ts
 const row = rowAt({ position: 1, },);
 ```
 */
function rowAt({ position, }: { readonly position: number; },): InsertionCoverageRow {
  return {
    position,
    sliceIndex: position,
    sourceText: TAIL_SOURCE,
    frontMatter: false,
    verdictKind: 'absent',
    anchoredFull: 0,
    anchoredPartial: 0,
    absentCount: 2,
    heard: 2,
    asked: 2,
    missingDestinationCount: 0,
    coverageFinding: `insertion-coverage (slice ${String(position,)}, verdict absent, full 0, partial 0, absent 2, heard 2 of 2)`,
    stageFindings: [],
    coverageEvidence: [],
    destinationFindings: [],
  };
}

/**
 Classifies two tail rows behind one paired slice, a tail whose expectation
 at the page's own expansion exceeds the last pair's rendering.

 @param second - row for the second tail slice, under test

 @returns Classification of both tail candidates

 @example
 ```ts
 const state = classifyTail({ second: rowAt({ position: 2, },), },);
 ```
 */
function classifyTail({ second, }: { readonly second: InsertionCoverageRow; },) {
  /**
   Rows in slice order.
   */
  const rows = [
    rowAt({ position: 1, },),
    second,
  ];
  return classifyInsertionCoverage({
    candidates: rows.map(function toCandidate(row,) {
      return {
        position: row.position,
        sliceIndex: row.sliceIndex,
        sourceText: row.sourceText,
        frontMatter: false,
      };
    },),
    rows,
    frontMatterPositions: new Set(),
    sourceText: `${PAIRED_SOURCE}\n\n${TAIL_SOURCE}\n\n${TAIL_SOURCE}\n`,
    targetText: `${PAIRED_TARGET}\n`,
    tail: readUntranslatedTail({
      slices: [
        pairedSlice(),
        insertion({ sliceIndex: 1, source: TAIL_SOURCE, },),
        insertion({ sliceIndex: 2, source: TAIL_SOURCE, },),
      ],
    },),
  },);
}

await describe({
  name: 'a split verdict inside the admitted tail (class sixty, XingZ610)',
  children: [
    it({
      name: 'ADMITS the tail slice on the bound where one voice anchored a minority claim of coverage',
      fn: async () => {
        const state = classifyTail({
          second: {
            ...rowAt({ position: 2, },),
            verdictKind: 'split',
            anchoredPartial: 1,
            absentCount: 1,
          },
        },);
        expect([...state.positions,],).toEqual([
          1,
          2,
        ],);
        expect(state.unresolvedRows,).toEqual([],);
        expect(state.findings.join('\n',),).toContain('insertion-corroboration (slice 2, tail admitted',);
      },
    },),
    it({
      name: 'STILL LEAVES a tail slice a majority found partly carried out of the admission',
      fn: async () => {
        const state = classifyTail({
          second: {
            ...rowAt({ position: 2, },),
            verdictKind: 'partly-carried',
            anchoredPartial: 2,
            absentCount: 0,
          },
        },);
        expect([...state.positions,],).toEqual([1,],);
        expect(state.unresolvedRows.length,).toBe(1,);
      },
    },),
  ],
},);
