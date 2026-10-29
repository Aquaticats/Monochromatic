/**
 Tests for one entry's reading in the displacement probe: what the size
 screen makes of a slicing the lanes saw, over real preparations whose
 paragraphs a case sizes.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  type EntryDisplacement,
  readEntry,
} from '../../dist/final/node/index.mjs';

/**
 One paragraph of a fixture page: its original and its translation.
 */
type Paragraph = readonly [string, string];

/**
 Original of `count` characters.

 @param count - characters wanted

 @returns That many characters of an invented original

 @example
 ```ts
 const text = original({ count: 100, },);
 ```
 */
function original({ count, }: { readonly count: number; },): string {
  return '猫'.repeat(count,);
}

/**
 Translation of `count` characters.

 @param count - characters wanted

 @returns That many characters of an invented translation

 @example
 ```ts
 const text = translation({ count: 286, },);
 ```
 */
function translation({ count, }: { readonly count: number; },): string {
  return 'c'.repeat(count,);
}

/**
 Paragraph translated at the baseline expansion.
 */
const ORDINARY: Paragraph = [original({ count: 100, },), translation({ count: 286, },),];

/**
 Image both sides of a paragraph carry.
 */
const IMAGE = '![](cat.png)';

/**
 Original made of image lines only, the markup a donor slice can be.
 */
const IMAGE_LINES = Array.from(
  { length: 10, },
  function imageLine(
    _unused,
    index,
  ): string {
    return `![](photos/cat${String(index,)}.png)`;
  },
).join('\n',);

/**
 Reads the entry of a page made of the paragraphs, one slice each.

 @param paragraphs - paragraphs in order, under one heading

 @returns Row the probe reads off them

 @example
 ```ts
 const row = readParagraphs({ paragraphs: [ORDINARY,], },);
 ```
 */
function readParagraphs(
  { paragraphs, }: { readonly paragraphs: readonly Paragraph[]; },
): EntryDisplacement {
  return readEntry({
    entryId: 'Mittens',
    prepared: prepareDocumentPair({
      sourceText: `## 节\n\n${paragraphs.map(function sourceOf(paragraph,): string {
        return paragraph[0];
      },).join('\n\n',)}\n`,
      targetText: `## S\n\n${paragraphs.map(function targetOf(paragraph,): string {
        return paragraph[1];
      },).join('\n\n',)}\n`,
      sliceCharBudget: 100,
    },),
  },);
}

await describe({
  name: readEntry.name,
  children: [
    it({
      name: 'READS a page translated at one density as nothing to report, against its own baseline',
      fn: async () => {
        expect(readParagraphs({ paragraphs: [ORDINARY, ORDINARY, ORDINARY, ORDINARY,], },),).toEqual({
          entryId: 'Mittens',
          sliceCount: 5,
          baseline: 2.86,
          baselineFrom: 'document',
          untranslated: [],
          targetOnly: [],
          relocationCandidates: [],
          transcriptionSuspects: [],
          markupDonors: [],
          otherImbalances: [],
        },);
      },
    },),
    it({
      name: 'READS a low slice beside a high one as one relocation candidate, with no suspect and no donor',
      fn: async () => {
        expect(readParagraphs({
          paragraphs: [
            ORDINARY,
            ORDINARY,
            ORDINARY,
            [original({ count: 200, },), translation({ count: 100, },),],
            [original({ count: 100, },), translation({ count: 758, },),],
            ORDINARY,
            ORDINARY,
          ],
        },),).toEqual({
          entryId: 'Mittens',
          sliceCount: 8,
          baseline: 2.86,
          baselineFrom: 'document',
          untranslated: [],
          targetOnly: [],
          relocationCandidates: [{
            high: 5,
            low: 4,
            surplus: 472,
            deficit: 472,
          },],
          transcriptionSuspects: [],
          markupDonors: [],
          otherImbalances: [],
        },);
      },
    },),
    it({
      name: 'NAMES the high slice a transcription suspect when it embeds the same image on both sides',
      fn: async () => {
        expect(readParagraphs({
          paragraphs: [
            ORDINARY,
            ORDINARY,
            ORDINARY,
            [original({ count: 200, },), translation({ count: 100, },),],
            [`${original({ count: 100, },)} ${IMAGE}`, `${translation({ count: 758, },)} ${IMAGE}`,],
            ORDINARY,
            ORDINARY,
          ],
        },),).toEqual({
          entryId: 'Mittens',
          sliceCount: 8,
          baseline: 2.86,
          baselineFrom: 'document',
          untranslated: [],
          targetOnly: [],
          relocationCandidates: [{
            high: 5,
            low: 4,
            surplus: 448,
            deficit: 472,
          },],
          transcriptionSuspects: [5,],
          markupDonors: [],
          otherImbalances: [],
        },);
      },
    },),
    it({
      name: 'NAMES the low slice a markup donor when its original is image lines and no prose',
      fn: async () => {
        expect(readParagraphs({
          paragraphs: [
            ORDINARY,
            ORDINARY,
            ORDINARY,
            [IMAGE_LINES, translation({ count: 100, },),],
            [original({ count: 100, },), translation({ count: 758, },),],
            ORDINARY,
            ORDINARY,
          ],
        },),).toEqual({
          entryId: 'Mittens',
          sliceCount: 8,
          baseline: 2.86,
          baselineFrom: 'document',
          untranslated: [],
          targetOnly: [],
          relocationCandidates: [{
            high: 5,
            low: 4,
            surplus: 472,
            deficit: 498,
          },],
          transcriptionSuspects: [],
          markupDonors: [4,],
          otherImbalances: [],
        },);
      },
    },),
    it({
      name: 'PASSES the untranslated, target-only and one-ended classes through as the classification gave them',
      fn: async () => {
        expect(readParagraphs({
          paragraphs: [
            ORDINARY,
            ORDINARY,
            [original({ count: 200, },), translation({ count: 20, },),],
            ORDINARY,
            [original({ count: 40, },), translation({ count: 500, },),],
            ORDINARY,
            ORDINARY,
            [original({ count: 100, },), translation({ count: 758, },),],
            ORDINARY,
            ORDINARY,
          ],
        },),).toEqual({
          entryId: 'Mittens',
          sliceCount: 11,
          baseline: 2.86,
          baselineFrom: 'document',
          untranslated: [3,],
          targetOnly: [5,],
          relocationCandidates: [],
          transcriptionSuspects: [],
          markupDonors: [],
          otherImbalances: [8,],
        },);
      },
    },),
  ],
},);
