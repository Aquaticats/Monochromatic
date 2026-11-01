/**
 Tests that the translate lane's assembly GIVES EACH SLICE ITS OWN WORDING
 when it lists what the document carries, the twin of
 `repair-assemble-slice-match.unit.test.ts`.

 WHAT WAS MEASURED. On 2026-09-28 (audit area six), handing the translate
 lane's per-slice list no surviving rows at all, so every slice read as the
 archive, failed no test in this package: the repair lane's twin covered the
 shared reading and nothing covered this lane's call. The adjacent-repetition
 check then read the archive and stayed silent on a page that shipped one
 wording on two neighbours.

 THREE SLICES, for the reason the repair twin gives: a two-slice fixture
 cannot tell a correct match from an inverted one. The first case is the
 kill; the second is the control showing the silence is earned.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source';
import {
  assembleTranslation,
  type TranslateSliceRecord,
} from '../dist/final/node/index.mjs';

/**
 Logger for assembly under test.
 */
const l = tagged({ tag: 'translate-assemble-slice-match-test', },);

//region Fixtures

/**
 Original paragraphs, one per slice.
 */
const SOURCES: readonly string[] = [
  '小猫睡在窗台上。',
  '胡须看着小鸟。',
  '虎斑猫在暖气旁打盹。',
];

/**
 Archive wording of each slice, separators included the way the slicer keeps
 them.
 */
const ARCHIVES: readonly string[] = [
  'The kitten sleeps on the sill.\n\n',
  'Whiskers watches the birds.\n\n',
  'The tabby dozes by the heater.\n',
];

/**
 Wording the first slice settled on.
 */
const FIRST_WORDING = 'The kitten sleeps on the windowsill through the afternoon.\n\n';

/**
 Wording the second slice settled on, distinct from both its neighbours.
 */
const SECOND_WORDING = 'Whiskers watches the birds through the window glass.\n\n';

/**
 Wording the third slice settled on.
 */
const THIRD_WORDING = 'The tabby dozes beside the heater until the evening.\n';

/**
 Where each piece of a joined text starts.

 @param pieces - texts joined without separators

 @returns Offset of each piece in the joined text

 @example
 ```ts
 const starts = startsOf(['ab', 'c',],); // [0, 2]
 ```
 */
function startsOf(pieces: readonly string[],): readonly number[] {
  return pieces.map(function startOf(
    _piece,
    index,
  ): number {
    return pieces.slice(0, index,)
      .join('',)
      .length;
  },);
}

/**
 Preparation of three content slices whose archive and original both carry
 one paragraph per slice.

 @returns Prepared pair of three slices

 @example
 ```ts
 const prepared = preparedThreeSlices();
 ```
 */
function preparedThreeSlices(): PreparedDocumentPair {
  /**
   Source pieces with their paragraph separators.
   */
  const sourcePieces = SOURCES.map(function separated(text, index,): string {
    return (index === (SOURCES.length - 1)) ? `${text}\n` : `${text}\n\n`;
  },);

  /**
   Offsets of the source pieces.
   */
  const sourceStarts = startsOf(sourcePieces,);

  /**
   Offsets of the archive pieces.
   */
  const targetStarts = startsOf(ARCHIVES,);
  return {
    sourceText: sourcePieces.join('',),
    targetText: ARCHIVES.join('',),
    slices: ARCHIVES.map(function toSlice(text, sliceIndex,) {
      /**
       Source paragraph and where it starts.
       */
      const sourceText = SOURCES[sliceIndex] ?? '';
      const sourceStart = sourceStarts[sliceIndex] ?? 0;
      const targetStart = targetStarts[sliceIndex] ?? 0;
      return {
        source: {
          kind: 'content',
          sliceIndex,
          nodes: [],
          startOffset: sourceStart,
          endOffset: sourceStart + sourceText.length,
          text: sourceText,
        },
        target: {
          kind: 'content',
          sliceIndex,
          nodes: [],
          startOffset: targetStart,
          endOffset: targetStart + text.length,
          text,
        },
      };
    },),
    lineStructuredSliceIndices: new Set(),
    declaredNames: [],
    alignmentFindings: [],
    unclaimedTargetBlocks: [],
    alignmentPairCount: 3,
  } as unknown as PreparedDocumentPair;
}

/**
 Record for one slice the lane settled.

 @param sliceIndex - slice this settles

 @param outputText - wording the lane settled on

 @returns Record as the lane writes it

 @example
 ```ts
 const record = recordFor({ sliceIndex: 0, outputText: FIRST_WORDING, },);
 ```
 */
function recordFor(
  {
    sliceIndex,
    outputText,
  }: {
    readonly sliceIndex: number;
    readonly outputText: string;
  },
): TranslateSliceRecord {
  return {
    kind: 'translate-slice',
    schemaVersion: 1,
    sliceIndex,
    outputText,
    changed: true,
    disposition: 'stage-result',
    findings: [],
    droppedDeclaredNames: [],
    alignment: {
      kind: 'incumbent-dominates-source',
      sourceCodePoints: 11,
      incumbentCodePoints: 33,
      minProtectedIncumbent: 20,
      maxRatio: 2,
    },
    stageResult: {
      text: outputText,
      origin: 'fresh',
      decision: 'judged',
      voteWeight: 1,
      ballots: [],
      heardTranslators: 2,
      candidateCount: 2,
      slate: [],
      perCandidate: [],
      findings: [],
    },
  } as unknown as TranslateSliceRecord;
}

/**
 Adjacent-repetition findings of an assembly whose middle slice shipped the
 given wording.

 @param second - wording the middle slice shipped

 @returns Findings naming an adjacent repetition, and the changed slices

 @example
 ```ts
 const found = assembleWith({ second: FIRST_WORDING, },);
 ```
 */
function assembleWith(
  { second, }: { readonly second: string; },
): {
  readonly adjacent: readonly string[];
  readonly changedSliceIndices: readonly number[];
} {
  /**
   Assembly of the three settled slices.
   */
  const result = assembleTranslation({
    prepared: preparedThreeSlices(),
    settled: [
      recordFor({ sliceIndex: 0, outputText: FIRST_WORDING, },),
      recordFor({ sliceIndex: 1, outputText: second, },),
      recordFor({ sliceIndex: 2, outputText: THIRD_WORDING, },),
    ],
    unfilled: [],
    carriedChunkIndices: [],
    resumedSliceCount: 0,
    findings: [],
    l,
  },);
  return {
    adjacent: result.findings.filter(function isAdjacent(finding,): boolean {
      return finding.startsWith('adjacent-repetition',);
    },),
    changedSliceIndices: result.changedSliceIndices,
  };
}

//endregion Fixtures

await describe({
  name: assembleTranslation.name,
  children: [
    it({
      name: 'NAMES THE TWO NEIGHBOURS that really did ship the same wording, read off the rows the guard '
        + 'let stand rather than the archive',
      fn: async () => {
        /**
         Assembly where the middle slice shipped its neighbour's wording.
         */
        const found = assembleWith({ second: FIRST_WORDING, },);

        expect(found.adjacent.length,).toBe(1,);
        expect(found.adjacent[0],).toContain('adjacent-repetition (slices 0 and 1',);
      },
    },),
    it({
      name: 'STAYS SILENT when three neighbours ship three different wordings, all three counted as changed',
      fn: async () => {
        /**
         Assembly where every slice shipped its own distinct wording.
         */
        const found = assembleWith({ second: SECOND_WORDING, },);

        expect(found.adjacent,).toStrictEqual([],);
        expect(found.changedSliceIndices,).toStrictEqual([
          0,
          1,
          2,
        ],);
      },
    },),
  ],
},);
