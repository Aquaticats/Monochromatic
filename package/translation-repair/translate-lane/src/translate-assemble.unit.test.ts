/**
 Tests for the assembly's withdrawn and shipped index sets.

 A replacement can validate on its own and still break a relation between
 slices: a footnote reference settled apart from its definition. The guard
 withdraws it at assembly, and the two index sets the result carries must say
 so: the slice is withdrawn, not changed, and the document that ships is the
 archive.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source';
import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  assembleTranslation,
  type TranslateSliceRecord,
} from '../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';

/**
 Original with a footnote referenced in one section and defined at the end.
 */
const SOURCE_TEXT = `## 甲

猫猫在窗台上睡觉[^1]。

## 乙

胡须酱追着蝴蝶跑。

[^1]: 它最喜欢的地方。
`;

/**
 Archive with the same footnote.
 */
const TARGET_TEXT = `## Alpha

The cat sleeps on the windowsill[^1].

## Beta

Whiskers chases butterflies.

[^1]: The spot it likes best.
`;

/**
 Original of two paragraphs, for the repetition case.
 */
const TWO_PARAGRAPH_SOURCE = '猫每天早上数篱笆上的麻雀。\n\n后来她在炉子旁边睡到晚饭。\n';

/**
 First archive paragraph, carrying specific words.
 */
const COUNTING = 'Mittens counted the sparrows along the garden fence every morning.';

/**
 Second archive paragraph.
 */
const SLEEPING = 'Later she slept beside the kitchen stove until dinner.';

/**
 Wording of the referencing slice with its marker dropped.
 */
const DROPS_THE_MARKER = 'The cat sleeps on the windowsill.';

/**
 Archive carrying one paragraph and none of the disclosure block that follows it.
 */
const HALVES_TARGET = 'The cat naps on the windowsill.\n\n';

/**
 Opening half of the block the archive never carried: the slicer gives a
 container's opening tag to the first block inside it.
 */
const OPEN_HALF_SOURCE = '<details style="margin-top: 0.5rem;">\n<summary>猫的故事</summary>';

/**
 Closing half: the block's last paragraph and the closing tag.
 */
const CLOSE_HALF_SOURCE = '猫在夜里回家了。\n\n</details>';

/**
 Rendering of the opening half.
 */
const OPEN_HALF_TEXT = '<details style="margin-top: 0.5rem;">\n<summary>The cat\'s story</summary>\n\n';

/**
 Rendering of the closing half.
 */
const CLOSE_HALF_TEXT = 'The cat came home at night.\n\n</details>\n';

/**
 Preparation whose slice 0 is the archive paragraph and whose slices 1 and 2
 are the two halves of a block the archive never carried, both anchored
 after the paragraph.

 @returns Prepared pair with one content slice and two insertion slices

 @example
 ```ts
 const prepared = preparedWithHalves();
 ```
 */
function preparedWithHalves(): PreparedDocumentPair {
  /**
   Offset of the closing half in the source.
   */
  const closeStart = 20 + OPEN_HALF_SOURCE.length + 2;
  return {
    sourceText: `猫猫在窗台上打盹。\n\n${OPEN_HALF_SOURCE}\n\n${CLOSE_HALF_SOURCE}\n`,
    targetText: HALVES_TARGET,
    slices: [
      {
        source: {
          kind: 'content',
          sliceIndex: 0,
          nodes: [],
          startOffset: 0,
          endOffset: 9,
          text: '猫猫在窗台上打盹。',
        },
        target: {
          kind: 'content',
          sliceIndex: 0,
          nodes: [],
          startOffset: 0,
          endOffset: HALVES_TARGET.length,
          text: HALVES_TARGET,
        },
      },
      {
        source: {
          kind: 'content',
          sliceIndex: 1,
          nodes: [],
          startOffset: 20,
          endOffset: 20 + OPEN_HALF_SOURCE.length,
          text: OPEN_HALF_SOURCE,
        },
        target: makeInsertionChunk({ sliceIndex: 1, offset: HALVES_TARGET.length, },),
      },
      {
        source: {
          kind: 'content',
          sliceIndex: 2,
          nodes: [],
          startOffset: closeStart,
          endOffset: closeStart + CLOSE_HALF_SOURCE.length,
          text: CLOSE_HALF_SOURCE,
        },
        target: makeInsertionChunk({ sliceIndex: 2, offset: HALVES_TARGET.length, },),
      },
    ],
    lineStructuredSliceIndices: new Set(),
    declaredNames: [],
    alignmentFindings: [],
    unclaimedTargetBlocks: [],
    alignmentPairCount: 3,
  };
}

/**
 Record for one slice, changed or kept.

 @param sliceIndex - slice this settles

 @param incumbentText - archive wording of the slice

 @param outputText - wording the lane settled on

 @returns Record as the lane writes it

 @example
 ```ts
 const record = recordFor({ sliceIndex: 0, incumbentText, outputText: incumbentText, },);
 ```
 */
function recordFor(
  {
    sliceIndex,
    incumbentText,
    outputText,
  }: {
    readonly sliceIndex: number;
    readonly incumbentText: string;
    readonly outputText: string;
  },
): TranslateSliceRecord {
  return {
    kind: 'translate-slice',
    schemaVersion: 1,
    sliceIndex,
    outputText,
    changed: outputText !== incumbentText,
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

await describe({
  name: assembleTranslation.name,
  children: [
    it({
      name: 'WITHHOLDS A CONTAINER HALF WHOSE PARTNER SHIPS NOTHING, naming the partner, instead of handing '
        + 'the guard a page with a closing tag and no opening (class fifty-seven, XingZ607: two blocks lost '
        + 'their summaries at admission, two closing tags shipped alone, and the guard withdrew all 88 slices)',
      fn: async () => {
        const prepared = preparedWithHalves();
        const said: string[] = [];
        const result = assembleTranslation({
          prepared,
          settled: [
            recordFor({ sliceIndex: 0, incumbentText: HALVES_TARGET, outputText: HALVES_TARGET, },),
            recordFor({ sliceIndex: 2, incumbentText: '', outputText: CLOSE_HALF_TEXT, },),
          ],
          unfilled: [{ sliceIndex: 1, reason: 'not-corroborated', findings: [], },],
          carriedChunkIndices: [],
          resumedSliceCount: 0,
          findings: [],
          l: capturingLogger({ messages: said, },),
        },);
        expect(result.withdrawnSliceIndices,).toEqual([2,],);
        expect(result.changedSliceIndices,).toEqual([],);
        expect(result.translatedText,).toBe(HALVES_TARGET,);
        expect(result.findings.some(function namesTheHalf(finding,): boolean {
          return finding.startsWith('assembly-container-half-withheld (slice 2 beside slice 1',);
        },),).toBe(true,);
        expect(result.findings.some(function blanket(finding,): boolean {
          return finding.startsWith('assembly-withdrew-every-replacement',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'WITHHOLDS THE OPENING HALF when the closing half ships nothing, the same pair from the other side: '
        + 'the page would carry an opening tag that never closes',
      fn: async () => {
        const prepared = preparedWithHalves();
        const said: string[] = [];
        const result = assembleTranslation({
          prepared,
          settled: [
            recordFor({ sliceIndex: 0, incumbentText: HALVES_TARGET, outputText: HALVES_TARGET, },),
            recordFor({ sliceIndex: 1, incumbentText: '', outputText: OPEN_HALF_TEXT, },),
          ],
          unfilled: [{ sliceIndex: 2, reason: 'not-corroborated', findings: [], },],
          carriedChunkIndices: [],
          resumedSliceCount: 0,
          findings: [],
          l: capturingLogger({ messages: said, },),
        },);
        expect(result.withdrawnSliceIndices,).toEqual([1,],);
        expect(result.changedSliceIndices,).toEqual([],);
        expect(result.translatedText,).toBe(HALVES_TARGET,);
        expect(result.findings.some(function namesTheHalf(finding,): boolean {
          return finding.startsWith('assembly-container-half-withheld (slice 1 beside slice 2',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'SHIPS BOTH HALVES when both settle, the page parsing as one block',
      fn: async () => {
        const prepared = preparedWithHalves();
        const said: string[] = [];
        const result = assembleTranslation({
          prepared,
          settled: [
            recordFor({ sliceIndex: 0, incumbentText: HALVES_TARGET, outputText: HALVES_TARGET, },),
            recordFor({ sliceIndex: 1, incumbentText: '', outputText: OPEN_HALF_TEXT, },),
            recordFor({ sliceIndex: 2, incumbentText: '', outputText: CLOSE_HALF_TEXT, },),
          ],
          unfilled: [],
          carriedChunkIndices: [],
          resumedSliceCount: 0,
          findings: [],
          l: capturingLogger({ messages: said, },),
        },);
        expect(result.withdrawnSliceIndices,).toEqual([],);
        expect(result.changedSliceIndices,).toEqual([1, 2,],);
        expect(result.translatedText,).toBe(`${HALVES_TARGET}${OPEN_HALF_TEXT}${CLOSE_HALF_TEXT}`,);
      },
    },),
    it({
      name: 'WITHDRAWS a replacement that drops a footnote marker at assembly and lists it as withdrawn, not '
        + 'changed, so the document that ships is the archive and the index sets say why',
      fn: async () => {
        /**
         Prepared pair, sliced by the pipeline.
         */
        const prepared = await prepareDocumentPair({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
        },);

        /**
         Slice carrying the reference, which the replacement damages.
         */
        const referencing = prepared.slices.find(function carriesMarker(slice,): boolean {
          return slice.target.text.includes('[^1].',);
        },);
        if (referencing === undefined)
          throw new Error('the fixture pair carries no slice with the reference',);

        /**
         Lines the assembly wrote.
         */
        const said: string[] = [];

        const result = assembleTranslation({
          prepared,
          settled: prepared.slices.map(function toRecord(slice,): TranslateSliceRecord {
            return recordFor({
              sliceIndex: slice.target.sliceIndex,
              incumbentText: slice.target.text,
              outputText: (slice.target.sliceIndex === referencing.target.sliceIndex)
                ? DROPS_THE_MARKER
                : slice.target.text,
            },);
          },),
          unfilled: [],
          resumedSliceCount: 0,
          findings: [],
          l: capturingLogger({ messages: said, },),
        },);

        expect(result.withdrawnSliceIndices,).toEqual([referencing.target.sliceIndex,],);
        expect(result.changedSliceIndices,).toEqual([],);
        expect(said.some(function mentionsWithdrawal(line,): boolean {
          return line.includes('withdrew 1 replacement at assembly',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'LOCATES A REPETITION in the log, naming the slices and the phrase, which the finding never does '
        + '(ledger L12)',
      fn: async () => {
        /**
         Prepared pair of two paragraphs, which the slicer keeps as one slice.
         */
        const prepared = await prepareDocumentPair({
          sourceText: TWO_PARAGRAPH_SOURCE,
          targetText: `${COUNTING}\n\n${SLEEPING}\n`,
        },);

        /**
         Lines the assembly wrote.
         */
        const said: string[] = [];

        assembleTranslation({
          prepared,
          settled: prepared.slices.map(function toRecord(slice,): TranslateSliceRecord {
            return recordFor({
              sliceIndex: slice.target.sliceIndex,
              incumbentText: slice.target.text,
              // The second paragraph repeats the first's specifics.
              outputText: slice.target.text.replace(
                SLEEPING,
                `${SLEEPING} Mittens counted the sparrows along the garden fence.`,
              ),
            },);
          },),
          unfilled: [],
          resumedSliceCount: 0,
          findings: [],
          l: capturingLogger({ messages: said, },),
        },);

        expect(said.some(function locates(line,): boolean {
          return line.includes('introduced repetition in slices 0 (',) && line.includes('sparrows along the garden',);
        },),).toBe(true,);
      },
    },),
  ],
},);
