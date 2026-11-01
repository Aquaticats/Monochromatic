/**
 Guards class fifty-nine (XingZ608, 2026-09-19): a footnote definition whose
 marker ships is admitted with the marker, whatever the whole-page shortfall
 budget has left. The budget is spent in document order and the definitions
 stand last on a page, so on an entry whose archive stops early the budget
 ran out 80 code points before the definitions; the assembly then withdrew
 every carrier of a marker with no definition, one of them holding the page's
 only rendering of a source link. Cat-themed invention throughout; no corpus
 content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  admitReferencedDefinitions,
  DEFINITION_ADMITTED_FINDING,
  type InsertionCoverageRow,
} from '../../dist/final/node/index.mjs';

/**
 Paragraph carrying a marker and no definition.
 */
const MARKER_PASSAGE = '猫在窗台晒太阳[^1]。';

/**
 Definition of the marker's label.
 */
const DEFINITION = '[^1]: 猫在夜里回家了。';

/**
 Definition of a label nothing references.
 */
const STRAY_DEFINITION = '[^2]: 猫不喜欢洗澡。';

/**
 Builds one source-only slice.

 @param sliceIndex - slice position and index alike

 @param text - source text of the slice

 @returns Prepared slice with no archive text

 @example
 ```ts
 const slice = sliceOf({ sliceIndex: 0, text: '猫。', },);
 ```
 */
function sliceOf(
  {
    sliceIndex,
    text,
  }: {
    readonly sliceIndex: number;
    readonly text: string;
  },
): ChunkPair {
  return {
    source: {
      kind: 'content',
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: text.length,
      text,
    },
    target: makeInsertionChunk({ sliceIndex, offset: 0, },),
  };
}

/**
 Builds one unresolved coverage row the roster found absent and the budget refused.

 @param position - position among the prepared slices

 @param sourceText - source text of the slice

 @returns Row neither admitted nor proven carried

 @example
 ```ts
 const row = rowOf({ position: 1, sourceText: DEFINITION, },);
 ```
 */
function rowOf(
  {
    position,
    sourceText,
  }: {
    readonly position: number;
    readonly sourceText: string;
  },
): InsertionCoverageRow {
  return {
    position,
    sliceIndex: position,
    sourceText,
    frontMatter: false,
    verdictKind: 'absent',
    anchoredFull: 0,
    anchoredPartial: 0,
    absentCount: 3,
    heard: 3,
    asked: 3,
    missingDestinationCount: 0,
    coverageFinding: 'scripted',
    stageFindings: [],
    coverageEvidence: [],
    destinationFindings: [],
  };
}

await describe({
  name: 'a definition whose marker ships (class fifty-nine, XingZ608)',
  children: [
    it({
      name: 'ADMITS THE DEFINITION BESIDE AN ADMITTED SLICE THAT REFERENCES ITS LABEL '
        + 'when the budget refused it, and names the referencing slice',
      fn: async () => {
        const admitted = admitReferencedDefinitions({
          slices: [
            sliceOf({ sliceIndex: 0, text: MARKER_PASSAGE, },),
            sliceOf({ sliceIndex: 1, text: DEFINITION, },),
          ],
          positions: new Set([0,],),
          unresolvedRows: [rowOf({ position: 1, sourceText: DEFINITION, },),],
          targetText: 'The cat sleeps.',
        },);
        expect([...admitted.positions,],).toEqual([
          0,
          1,
        ],);
        expect(admitted.unresolvedRows,).toEqual([],);
        expect(admitted.findings.length,).toBe(1,);
        expect(admitted.findings[0]?.startsWith(
          `${DEFINITION_ADMITTED_FINDING} (slice 1 defines 1, referenced by slice 0`,
        ),).toBe(true,);
      },
    },),
    it({
      name: 'ADMITS THE DEFINITION WHEN THE STANDING PAGE REFERENCES ITS LABEL and defines nothing under it',
      fn: async () => {
        const admitted = admitReferencedDefinitions({
          slices: [sliceOf({ sliceIndex: 0, text: DEFINITION, },),],
          positions: new Set(),
          unresolvedRows: [rowOf({ position: 0, sourceText: DEFINITION, },),],
          targetText: 'The cat suns itself on the sill[^1].',
        },);
        expect([...admitted.positions,],).toEqual([0,],);
        expect(admitted.findings[0]?.startsWith(
          `${DEFINITION_ADMITTED_FINDING} (slice 0 defines 1, referenced by the page`,
        ),).toBe(true,);
      },
    },),
    it({
      name: 'LEAVES A DEFINITION UNRESOLVED when the standing page already defines the label',
      fn: async () => {
        const admitted = admitReferencedDefinitions({
          slices: [sliceOf({ sliceIndex: 0, text: DEFINITION, },),],
          positions: new Set(),
          unresolvedRows: [rowOf({ position: 0, sourceText: DEFINITION, },),],
          targetText: 'The cat suns itself on the sill[^1].\n\n[^1]: The cat came home at night.',
        },);
        expect([...admitted.positions,],).toEqual([],);
        expect(admitted.unresolvedRows.length,).toBe(1,);
        expect(admitted.findings,).toEqual([],);
      },
    },),
    it({
      name: 'LEAVES A DEFINITION UNRESOLVED when nothing shipping references its label',
      fn: async () => {
        const admitted = admitReferencedDefinitions({
          slices: [
            sliceOf({ sliceIndex: 0, text: MARKER_PASSAGE, },),
            sliceOf({ sliceIndex: 1, text: STRAY_DEFINITION, },),
          ],
          positions: new Set([0,],),
          unresolvedRows: [rowOf({ position: 1, sourceText: STRAY_DEFINITION, },),],
          targetText: 'The cat sleeps.',
        },);
        expect([...admitted.positions,],).toEqual([0,],);
        expect(admitted.unresolvedRows.length,).toBe(1,);
        expect(admitted.findings,).toEqual([],);
      },
    },),
  ],
},);
