/**
 Tests for the page an entry stopped dry in the middle of its page ships (the owner's answer of 2026-10-29, "Ship
 interim, resumable").

 DRIVEN THROUGH THE REAL PUBLISHER, `publishFixedPage`, and a real preparation: the interim page is the page every
 settled entry gets with some slices marked unsettled, so the property under test is what the marking does to the
 reading and to the bytes written, and a stand-in for either would agree with whatever the marking did.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { existsSync, } from 'node:fs';
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
import { refusingNope, } from './source-shipped.test-fixture.ts';
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
 How many block quote markers the original of a slice is nested under, past the bound of 256 that the plain reading
 keeps, so no writing of it makes a page parse and the composition leaves it off the page.
 */
const NESTED_PAST_BOUND = 300;

/**
 Original of the second section nested past that bound.
 */
const NESTED_ORIGINAL = `${ '> '.repeat(NESTED_PAST_BOUND,) }猫猫有自己的碗。\n`;

/**
 Wording the grammar of these cases refuses (`refusingNope`), which the page-grammar fallback puts back to the archive.
 The assembly guard reads no fault in it, so the fallback is what reverts the slice.
 */
const REFUSED_WORDING = 'NOPE\n';

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

 @returns Record

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
      name: 'COUNTS A SLICE THE GRAMMAR PUT BACK TO THE ARCHIVE AS THE ARCHIVE\'S, not as settled (B1246): the counts '
        + 'are read off the page that was written, where they were read off the source before the fallback reverted '
        + 'the slice',
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
              text: REFUSED_WORDING,
            },),],
          },
          slices: prepared.slices,
          archiveText: prepared.targetText,
          sourceText: SOURCE_DOC,
          entryId: 'CatEntry1',
          publishDir: root.path,
          l: capturingLogger({ messages: [], },),
          readGrammar: refusingNope,
        },);
        expect({
          page: shipped.pageText,
          counts: [shipped.settledSlices, shipped.archiveSlices, shipped.sourceSlices,],
          kept: shipped.defects.map(function checkOf({ check, },): string {
            return check;
          },).includes('page-grammar-archive-kept',),
        },).toEqual({
          page: TARGET_DOC,
          counts: [0, prepared.slices.length, 0,],
          kept: true,
        },);
      },
    },),

    it({
      name: 'WRITES NOTHING OVER A SETTLED PAGE OR OVER AN INTERIM PAGE WITH MORE SETTLED SLICES (B1242, B1243), and '
        + 'says it wrote nothing; a standing page with as many settled slices, or none, is replaced',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'interim-page-', },);
        const prepared = prepareDocumentPair({
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
          includeFrontMatter: true,
          sealArchiveOriginal: true,
        },);
        /**
         Ships the page with one slice settled over the given standing page.

         @param standing - page that stands where this one would go
         */
        async function shipOver(
          { standing, }: { readonly standing?: Parameters<typeof publishInterimPage>[0]['standing']; },
        ): Promise<Awaited<ReturnType<typeof publishInterimPage>>> {
          return await publishInterimPage({
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
            ...((standing === undefined) ? {} : { standing, }),
          },);
        }
        /**
         Where the page goes.
         */
        const path = fixedPagePath({
          publishDir: root.path,
          entryId: 'CatEntry1',
        },);
        const settledStanding = await shipOver({ standing: { kind: 'settled', }, },);
        const existsAfterSettled = existsSync(path,);
        const fuller = await shipOver({
          standing: {
            kind: 'interim',
            settledSlices: 2,
          },
        },);
        const existsAfterFuller = existsSync(path,);
        const equal = await shipOver({
          standing: {
            kind: 'interim',
            settledSlices: 1,
          },
        },);
        expect({
          written: [settledStanding.written, fuller.written, equal.written,],
          existsAfter: [existsAfterSettled, existsAfterFuller,],
          page: await readFile(
            path,
            'utf8',
          ),
        },).toEqual({
          written: [false, false, true,],
          existsAfter: [false, false,],
          page: '## Section one\n\nThe cat naps on the sill.\n\n## Section two\n\nThe cat has a bowl.\n',
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

    it({
      name: 'COUNTS A SLICE WHOSE ORIGINAL THE COMPOSITION LEFT OFF THE PAGE AS THE ARCHIVE\'S, not as a slice that '
        + 'carries the original (B1300): an original no writing makes parse ships without it and with the defect '
        + '`page-source-unparseable`, and the counts of the record read the page that shipped',
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
        const covered = nonNullishOrThrow(whole.slices[0],);
        const cut = nonNullishOrThrow(whole.slices[1],);
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
              source: {
                ...cut.source,
                text: NESTED_ORIGINAL,
              },
              target: makeInsertionChunk({
                sliceIndex: 1,
                offset: covered.target
                  .endOffset,
              },),
            },
          ],
          archiveText,
          sourceText: SOURCE_DOC,
          entryId: 'CatEntryDropped',
          publishDir: root.path,
          l: capturingLogger({ messages: [], },),
        },);
        expect({
          page: shipped.pageText,
          counts: [shipped.settledSlices, shipped.archiveSlices, shipped.sourceSlices,],
          dropped: shipped.defects.some(function namesTheDrop({ check, },): boolean {
            return check === 'page-source-unparseable';
          },),
        },).toEqual({
          page: archiveText,
          counts: [0, 2, 0,],
          dropped: true,
        },);
      },
    },),
  ],
},);
