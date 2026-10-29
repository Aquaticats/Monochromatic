/**
 Tests for the page an entry stopped dry in the middle of its page ships (the owner's answer of 2026-10-29, "Ship
 interim, resumable").

 DRIVEN THROUGH THE REAL PUBLISHER, `publishFixedPage`, and a real preparation: the interim page is the page every
 settled entry gets with some slices marked unsettled, so the property under test is what the marking does to the
 reading and to the bytes written, and a stand-in for either would agree with whatever the marking did.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { readFile, } from 'node:fs/promises';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { ArtifactConsolidateSlice, } from '@monochromatic-dev/translation-repair-artifact';
import {
  fixedPagePath,
  publishInterimPage,
  UNTRANSLATED_PASSAGE_NOTE,
} from '../../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Original with two sections.
 */
const SOURCE_DOC = '## 第一节\n\n猫猫在窗台上睡觉。\n\n## 第二节\n\n猫猫有自己的碗。\n';

/**
 Archive English of the same shape.
 */
const TARGET_DOC = '## Section one\n\nThe cat sleeps on the sill.\n\n## Section two\n\nThe cat has a bowl.\n';

/**
 Wording a consolidation record settled for the first section.
 */
const SETTLED_FIRST = '## Section one\n\nThe cat naps on the sill.\n';

/**
 What every consolidation record of these cases shares: a consolidated slice no gate was asked about.
 */
const UNGATED_RECORD = {
  terminal: 'consolidated',
  shipped: {
    kind: 'consolidated',
    text: '',
  },
  rewrapped: false,
  demoted: false,
  verdicts: [],
  gate: { kind: 'not-asked', },
} as const;

/**
 Record the consolidation left at a slice, shipping the wording it consolidated.

 @param sliceIndex - slice the record answers

 @param text - wording it consolidated

 @returns The record

 @example
 ```ts
 const record = consolidatedAt({ sliceIndex: 0, text: SETTLED_FIRST, },);
 ```
 */
function consolidatedAt(
  {
    sliceIndex,
    text,
  }: {
    readonly sliceIndex: number;
    readonly text: string;
  },
): ArtifactConsolidateSlice {
  return {
    ...UNGATED_RECORD,
    sliceIndex,
    shipped: {
      kind: 'consolidated',
      text,
    },
  };
}

await describe({
  name: publishInterimPage.name,
  children: [
    it({
      name: 'SHIPS THE ARCHIVE AS IT STANDS when nothing is settled and neither lane returned, counting every slice as '
        + 'the archive\'s',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'interim-page-', },);
        const prepared = prepareDocumentPair({
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
          includeFrontMatter: true,
          sealArchiveOriginal: true,
        },);
        const shipped = await publishInterimPage({
          stop: { consolidateSlices: [], },
          slices: prepared.slices,
          archiveText: prepared.targetText,
          sourceText: SOURCE_DOC,
          entryId: 'CatEntry1',
          publishDir: root.path,
          l: capturingLogger({ messages: [], },),
        },);
        expect({
          page: await readFile(
            fixedPagePath({
              publishDir: root.path,
              entryId: 'CatEntry1',
            },),
            'utf8',
          ),
          returned: shipped.pageText,
          counts: [shipped.settledSlices, shipped.archiveSlices, shipped.sourceSlices,],
        },).toEqual({
          page: TARGET_DOC,
          returned: TARGET_DOC,
          counts: [0, prepared.slices.length, 0,],
        },);
      },
    },),

    it({
      name: 'SHIPS THE SETTLED WORDING OF A SLICE THAT HAS A CONSOLIDATION RECORD and the archive for the slice that has '
        + 'none, and counts them apart',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'interim-page-', },);
        const prepared = prepareDocumentPair({
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
          includeFrontMatter: true,
          sealArchiveOriginal: true,
        },);
        const shipped = await publishInterimPage({
          stop: {
            consolidateSlices: [consolidatedAt({
              sliceIndex: 0,
              text: SETTLED_FIRST,
            },),],
          },
          slices: prepared.slices,
          archiveText: prepared.targetText,
          sourceText: SOURCE_DOC,
          entryId: 'CatEntry1',
          publishDir: root.path,
          l: capturingLogger({ messages: [], },),
        },);
        expect({
          page: shipped.pageText,
          counts: [shipped.settledSlices, shipped.archiveSlices, shipped.sourceSlices,],
        },).toEqual({
          page: '## Section one\n\nThe cat naps on the sill.\n\n## Section two\n\nThe cat has a bowl.\n',
          counts: [1, prepared.slices.length - 1, 0,],
        },);
      },
    },),

    it({
      name: 'SHIPS THE ORIGINAL UNDER ITS NOTE for a slice the archive lacks, beside the archive text of the slice it '
        + 'has, and counts it as source text',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'interim-page-', },);
        /**
         The pair as the archive covers it whole.
         */
        const whole = prepareDocumentPair({
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
          includeFrontMatter: true,
          sealArchiveOriginal: true,
        },);
        /**
         The slice of the first section, which the archive keeps.
         */
        const covered = nonNullishOrThrow(whole.slices[0],);
        /**
         The slice of the second section, which the archive loses.
         */
        const cut = nonNullishOrThrow(whole.slices[1],);
        /**
         The archive up to where it stops, after the first section.
         */
        const archiveText = whole.targetText.slice(
          0,
          covered.target
            .endOffset,
        );
        const shipped = await publishInterimPage({
          stop: { consolidateSlices: [], },
          slices: [
            covered,
            {
              source: cut.source,
              target: makeInsertionChunk({
                sliceIndex: 1,
                offset: covered.target
                  .endOffset,
              },),
            },
          ],
          archiveText,
          sourceText: SOURCE_DOC,
          entryId: 'CatEntryGap',
          publishDir: root.path,
          l: capturingLogger({ messages: [], },),
        },);
        expect(shipped.pageText,).toContain('The cat sleeps on the sill.',);
        expect(shipped.pageText,).toContain(`${UNTRANSLATED_PASSAGE_NOTE}\n\n`,);
        expect(shipped.pageText,).toContain('猫猫有自己的碗。',);
        expect([shipped.settledSlices, shipped.archiveSlices, shipped.sourceSlices,],).toEqual([0, 1, 1,],);
      },
    },),
  ],
},);
