/**
 Tests for the displacement probe's walk over the settled entries: what it
 logs for an artifact it skips, for a re-carve that moved and for the
 entries it reads, and the rows document it writes, over scripted carves.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { SettledCarve, } from '@monochromatic-dev/translation-repair-slice-cache';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { probeDisplacement, } from '../../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';

/**
 Original with one section.
 */
const SOURCE_PAGE = '## 第一节\n\n猫猫在窗台上睡觉。\n';

/**
 Translation of the same shape.
 */
const TARGET_PAGE = '## Section one\n\nThe cat sleeps on the sill.\n';

/**
 Carve of the fixture page through a recipe, as a settled entry gives it.

 @param unrecorded - halves of the recipe the artifact lacks

 @param reproduction - whether the re-carve is the run's own

 @returns Carve

 @example
 ```ts
 const carve = settledCarve({ unrecorded: [], reproduction: { kind: 'reproduced', }, },);
 ```
 */
function settledCarve(
  {
    unrecorded,
    reproduction,
  }: {
    readonly unrecorded: readonly ('sectionPairing' | 'blockPairing')[];
    readonly reproduction: Extract<SettledCarve, { readonly kind: 'settled'; }>['reproduction'];
  },
): SettledCarve {
  return {
    kind: 'settled',
    corpusSha: 'a'.repeat(40,),
    sourceText: SOURCE_PAGE,
    targetText: TARGET_PAGE,
    prepared: prepareDocumentPair({
      sourceText: SOURCE_PAGE,
      targetText: TARGET_PAGE,
    },),
    recipe: { unrecorded, },
    reproduction,
  };
}

/**
 Walks entries whose carves are scripted by id.

 @param carves - carve of each entry id, which is also the list

 @returns Lines logged with their level, and what was written out

 @example
 ```ts
 const walked = await walkOver({ carves: { Mittens: { kind: 'unsettled', }, }, },);
 ```
 */
async function walkOver(
  { carves, }: { readonly carves: Readonly<Record<string, SettledCarve>>; },
): Promise<{
  readonly lines: readonly string[];
  readonly written: readonly string[];
}> {
  /**
   Lines logged, each behind its level.
   */
  const lines: string[] = [];

  /**
   Texts written out.
   */
  const written: string[] = [];
  await probeDisplacement({
    log: levelCapturingLogger({ lines, },),
    listEntryIds: function listEntries(): Promise<readonly string[]> {
      return Promise.resolve(Object.keys(carves,),);
    },
    carve: function carveEntry(entryId,): Promise<SettledCarve> {
      return Promise.resolve(carves[entryId] ?? { kind: 'unsettled', },);
    },
    writeOut: function keep(text,): void {
      written.push(text,);
    },
  },);
  return {
    lines,
    written,
  };
}

/**
 Rows document an entry the fixture page gives.

 @param entryId - entry the row is for

 @returns Text the probe writes for that one entry

 @example
 ```ts
 const text = rowsDocumentOf({ entryId: 'Mittens', },);
 ```
 */
function rowsDocumentOf({ entryId, }: { readonly entryId: string; },): string {
  return `{\n  "rows": [\n    {\n      "entryId": "${entryId}",\n      "sliceCount": 1,\n`
    + '      "baseline": 2.86,\n      "baselineFrom": "corpus-reference",\n      "untranslated": [],\n'
    + '      "targetOnly": [],\n      "relocationCandidates": [],\n      "transcriptionSuspects": [],\n'
    + '      "markupDonors": [],\n      "otherImbalances": []\n    }\n  ]\n}\n';
}

await describe({
  name: probeDisplacement.name,
  children: [
    it({
      name: 'WRITES an empty rows document and logs the zero totals when no artifact is settled',
      fn: async () => {
        expect(await walkOver({ carves: {}, },),).toEqual({
          lines: [
            'info settled entries carved: 0 of 0 artifacts',
            'info   with a defaulted recipe half: 0',
            'info slices read: 0',
            'info entries falling back to the corpus baseline: 0',
            'info relocation candidates: 0',
            'info   of which a transcription would also explain: 0',
            'info untranslated slices: 0',
            'info target-only slices: 0',
            'info other imbalances: 0',
          ],
          written: ['{\n  "rows": []\n}\n',],
        },);
      },
    },),
    it({
      name: 'SKIPS an artifact that records no recipe, naming its kind, whether legacy or unsettled, and still counts it',
      fn: async () => {
        expect(await walkOver({ carves: {
          Mittens: { kind: 'legacy', },
          Tabby: { kind: 'unsettled', },
        }, },),).toEqual({
          lines: [
            'info Mittens: skipped, legacy artifact records no recipe',
            'info Tabby: skipped, unsettled artifact records no recipe',
            'info settled entries carved: 0 of 2 artifacts',
            'info   with a defaulted recipe half: 0',
            'info slices read: 0',
            'info entries falling back to the corpus baseline: 0',
            'info relocation candidates: 0',
            'info   of which a transcription would also explain: 0',
            'info untranslated slices: 0',
            'info target-only slices: 0',
            'info other imbalances: 0',
          ],
          written: ['{\n  "rows": []\n}\n',],
        },);
      },
    },),
    it({
      name: 'READS a settled entry whose re-carve is the run\'s own, without a warning, and writes its row',
      fn: async () => {
        expect(await walkOver({ carves: { Mittens: settledCarve({
          unrecorded: [],
          reproduction: { kind: 'reproduced', },
        },), }, },),).toEqual({
          lines: [
            'info settled entries carved: 1 of 1 artifact',
            'info   with a defaulted recipe half: 0',
            'info slices read: 1',
            'info entries falling back to the corpus baseline: 1',
            'info relocation candidates: 0',
            'info   of which a transcription would also explain: 0',
            'info untranslated slices: 0',
            'info target-only slices: 0',
            'info other imbalances: 0',
          ],
          written: [rowsDocumentOf({ entryId: 'Mittens', },),],
        },);
      },
    },),
    it({
      name: 'WARNS that a re-carve that moved measures other slices, still reads it, and names its defaulted halves',
      fn: async () => {
        expect(await walkOver({ carves: { Mittens: settledCarve({
          unrecorded: ['blockPairing',],
          reproduction: {
            kind: 'moved',
            detail: 'row 2 differs',
          },
        },), }, },),).toEqual({
          lines: [
            'warn Mittens: re-carve is not the run\'s (row 2 differs); its readings measure other slices',
            'info settled entries carved: 1 of 1 artifact',
            'info   with a defaulted recipe half: 1',
            'info   Mittens: deterministic default for blockPairing',
            'info slices read: 1',
            'info entries falling back to the corpus baseline: 1',
            'info relocation candidates: 0',
            'info   of which a transcription would also explain: 0',
            'info untranslated slices: 0',
            'info target-only slices: 0',
            'info other imbalances: 0',
          ],
          written: [rowsDocumentOf({ entryId: 'Mittens', },),],
        },);
      },
    },),
    it({
      name: 'LOGS EACH ENTRY IT SKIPS IN THE ORDER THE ENTRIES ARE LISTED when the later entry\'s carve ends first, '
        + 'so two walks over one archive log the same lines',
      fn: async () => {
        /**
         Opened by the later entry's carve as it ends, which the earlier
         entry's carve waits for.
         */
        const laterCarved = Promise.withResolvers<undefined>();
        /**
         Lines logged, each behind its level.
         */
        const lines: string[] = [];
        await probeDisplacement({
          log: levelCapturingLogger({ lines, },),
          listEntryIds: function listEntries(): Promise<readonly string[]> {
            return Promise.resolve([
              'Mittens',
              'Tabby',
            ],);
          },
          carve: async function carvesTabbyFirst(entryId,): Promise<SettledCarve> {
            if (entryId === 'Tabby') {
              laterCarved.resolve(undefined,);
              return { kind: 'unsettled', };
            }
            await laterCarved.promise;
            return { kind: 'legacy', };
          },
          writeOut: function keep(): void {},
        },);
        expect(lines.slice(
          0,
          2,
        ),).toEqual([
          'info Mittens: skipped, legacy artifact records no recipe',
          'info Tabby: skipped, unsettled artifact records no recipe',
        ],);
      },
    },),
    it({
      name: 'REFUSES with the first listed entry\'s refusal when two entries are refused and the later one is '
        + 'refused first',
      fn: async () => {
        /**
         Opened by the later entry's carve as it refuses, which the earlier
         entry's carve waits for.
         */
        const laterRefused = Promise.withResolvers<undefined>();
        /**
         What the walk refused with.
         */
        const refusal = await rejectionOf({
          promise: probeDisplacement({
            log: levelCapturingLogger({ lines: [], },),
            listEntryIds: function listEntries(): Promise<readonly string[]> {
              return Promise.resolve([
                'Mittens',
                'Tabby',
              ],);
            },
            carve: async function refusesTabbyFirst(entryId,): Promise<SettledCarve> {
              if (entryId === 'Tabby') {
                laterRefused.resolve(undefined,);
                throw new Error('Tabby was refused',);
              }
              await laterRefused.promise;
              throw new Error('Mittens was refused',);
            },
            writeOut: function keep(): void {},
          },),
        },);
        expect(refusal,).toBeInstanceOf(Error,);
        expect(String(refusal,),).toBe('Error: Mittens was refused',);
      },
    },),
  ],
},);
