/**
 Tests for the page that already stands when a stopped entry ships its interim page (B1242 to B1246): an interim page
 never replaces a settled page, or an interim page with more settled slices, for the same entry in the same runs folder,
 and a stop before the admission of the source-only passages is read is shipped from the archive.

 Each case drives `shipInterimOnStop` over a scratch tree that holds what an earlier attempt left.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { readInterimEntries, } from '@monochromatic-dev/translation-repair-glossary';
import { UNTRANSLATED_PASSAGE_NOTE, } from '@monochromatic-dev/translation-repair-page-check';
import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import { startEntryProgress, } from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  consolidatedAt,
  dirsOf,
  ENTRY_ID,
  pageOf,
  preparation,
  SETTLED_FIRST,
  SETTLED_SECOND,
  shipOver,
  slicedProgress,
  SOURCE_DOC,
  standingPage,
  TARGET_DOC,
} from './pass-entry-stop.test-fixture.ts';

/**
 Archive that opens its sections with a stub marker and spells one word with a non-breaking hyphen, the two things the
 pass's own archive normalisation changes.
 */
const STUB_ARCHIVE = '## Section one\n\nThe cat sleeps on the sill.\n\n(To-Do)\n\n## Section two\n\n'
  + 'The cat has a non‑binary bowl.\n';

/**
 The same archive after that normalisation.
 */
const NORMALISED_ARCHIVE = '## Section one\n\nThe cat sleeps on the sill.\n\n## Section two\n\n'
  + 'The cat has a non-binary bowl.\n';

/**
 Progress of an entry sliced with its second slice a source-only passage the archive lacks.

 @param folded - whether the fold of carried passages had run when the entry stopped

 @returns Progress holding that slicing

 @example
 ```ts
 const progress = progressWithGap({ folded: false, },);
 ```
 */
function progressWithGap(
  { folded, }: { readonly folded: boolean; },
): ReturnType<typeof startEntryProgress> {
  /**
   The pair as the archive covers it whole.
   */
  const whole = preparation();
  const [covered, cut,] = whole.slices;
  if ((covered === undefined) || (cut === undefined))
    throw new Error('the page has two slices',);
  return {
    ...startEntryProgress(),
    prepared: {
      ...whole,
      slices: [
        covered,
        {
          source: cut.source,
          target: makeInsertionChunk({
            sliceIndex: cut.target.sliceIndex,
            offset: covered.target.endOffset,
          },),
        },
      ],
    },
    archiveText: whole.targetText.slice(
      0,
      covered.target.endOffset,
    ),
    ...(folded ? { folded: true as const, } : {}),
  };
}

await describe({
  name: 'the page that stands when a stopped entry ships',
  children: [
    it({
      name: 'KEEPS AN INTERIM PAGE WITH MORE SETTLED SLICES when a later attempt of the same entry stops earlier and '
        + 'settled fewer (B1242), and leaves its record as it was',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-standing-', },);
        const dirs = dirsOf({ root: root.path, },);
        await shipOver({
          dirs,
          error: new RangeError('scripted transport failure',),
          progress: {
            ...slicedProgress(),
            consolidated: [consolidatedAt({
              sliceIndex: 0,
              text: SETTLED_FIRST,
            },),],
          },
          messages: [],
        },);
        /**
         The page and the record the first attempt left.
         */
        const first = {
          page: await pageOf({ dirs, },),
          record: (await readInterimEntries({ interimDir: dirs.interimDir, },)).get(ENTRY_ID,),
        };
        /**
         Log lines of the second attempt's ship.
         */
        const messages: string[] = [];
        await shipOver({
          dirs,
          error: new RangeError('scripted transport failure',),
          progress: slicedProgress(),
          messages,
        },);

        expect(first.page,).toBe(`${SETTLED_FIRST}\n## Section two\n\nThe cat has a bowl.\n`,);
        expect(await pageOf({ dirs, },),).toBe(first.page,);
        expect((await readInterimEntries({ interimDir: dirs.interimDir, },)).get(ENTRY_ID,),).toEqual(first.record,);
        expect(messages.some(function namesTheKeptPage(message,): boolean {
          return message.includes('interim page with more settled slices',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'REPLACES AN INTERIM PAGE WITH ONE THAT SETTLED MORE (the case of the same number follows), the control: the '
        + 'guard keeps a fuller page and nothing else',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-standing-', },);
        const dirs = dirsOf({ root: root.path, },);
        await shipOver({
          dirs,
          error: new RangeError('scripted transport failure',),
          progress: {
            ...slicedProgress(),
            consolidated: [consolidatedAt({
              sliceIndex: 0,
              text: SETTLED_FIRST,
            },),],
          },
          messages: [],
        },);
        await shipOver({
          dirs,
          error: new RangeError('scripted transport failure',),
          progress: {
            ...slicedProgress(),
            consolidated: [
              consolidatedAt({
                sliceIndex: 0,
                text: SETTLED_FIRST,
              },),
              consolidatedAt({
                sliceIndex: 1,
                text: SETTLED_SECOND,
              },),
            ],
          },
          messages: [],
        },);
        expect(await pageOf({ dirs, },),).toBe(`${SETTLED_FIRST}${SETTLED_SECOND}`,);
        expect((await readInterimEntries({ interimDir: dirs.interimDir, },)).get(ENTRY_ID,)?.settledSlices,).toBe(2,);
      },
    },),

    it({
      name: 'REPLACES AN INTERIM PAGE WITH ONE THAT SETTLED THE SAME NUMBER OF SLICES (B1242), the equal case: the later '
        + 'attempt\'s page is the fresher, so its wording stands and its record replaces the earlier one',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-standing-', },);
        const dirs = dirsOf({ root: root.path, },);
        /**
         Another wording of the first section, which the later attempt settled.
         */
        const fresher = '## Section one\n\nThe cat dozes on the sill.\n';
        await shipOver({
          dirs,
          error: new RangeError('scripted transport failure',),
          progress: {
            ...slicedProgress(),
            consolidated: [consolidatedAt({
              sliceIndex: 0,
              text: SETTLED_FIRST,
            },),],
          },
          messages: [],
        },);
        /**
         The record the first attempt left.
         */
        const first = (await readInterimEntries({ interimDir: dirs.interimDir, },)).get(ENTRY_ID,);
        await shipOver({
          dirs,
          error: new RangeError('scripted transport failure',),
          progress: {
            ...slicedProgress(),
            consolidated: [consolidatedAt({
              sliceIndex: 0,
              text: fresher,
            },),],
          },
          messages: [],
        },);
        /**
         The record the second attempt left.
         */
        const second = (await readInterimEntries({ interimDir: dirs.interimDir, },)).get(ENTRY_ID,);

        expect(await pageOf({ dirs, },),).toBe(`${fresher}\n## Section two\n\nThe cat has a bowl.\n`,);
        expect([first?.settledSlices, second?.settledSlices,],).toEqual([1, 1,],);
        expect(second?.pageSha256,).not.toBe(first?.pageSha256,);
      },
    },),

    it({
      name: 'KEEPS A SETTLED PAGE WHOSE ARTIFACT WRITE FAILED when the next attempt of the entry stops, though that '
        + 'attempt never wrote the settled page itself (B1243): a page with no interim record and no artifact is the '
        + 'settled page of an earlier attempt, and no record is written beside it',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-standing-', },);
        const dirs = dirsOf({ root: root.path, },);
        await standingPage({
          dirs,
          text: 'the settled page\n',
        },);
        /**
         Log lines of the ship.
         */
        const messages: string[] = [];
        await shipOver({
          dirs,
          error: new RangeError('scripted transport failure',),
          progress: slicedProgress(),
          messages,
        },);
        expect(await pageOf({ dirs, },),).toBe('the settled page\n',);
        expect([...await readInterimEntries({ interimDir: dirs.interimDir, },),],).toEqual([],);
        expect(messages.some(function namesTheSettledPage(message,): boolean {
          return message.includes('settled page',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'SHIPS WHEN NO PAGE STANDS, the control: the first stop of an entry in a runs folder writes its page and '
        + 'its record as before',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-standing-', },);
        const dirs = dirsOf({ root: root.path, },);
        await shipOver({
          dirs,
          error: new RangeError('scripted transport failure',),
          progress: slicedProgress(),
          messages: [],
        },);
        expect(await pageOf({ dirs, },),).toBe(TARGET_DOC,);
        expect((await readInterimEntries({ interimDir: dirs.interimDir, },)).get(ENTRY_ID,)?.settledSlices,).toBe(0,);
      },
    },),

    it({
      name: 'SHIPS NO NOTE FOR A SOURCE-ONLY PASSAGE BEFORE THE FOLD, since no admission has said whether the archive '
        + 'already renders it (B1244); the passage reads as the archive has it, which is nothing, and counts as the '
        + 'archive\'s',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-standing-', },);
        const dirs = dirsOf({ root: root.path, },);
        await shipOver({
          dirs,
          error: new RangeError('scripted transport failure',),
          progress: progressWithGap({ folded: false, },),
          messages: [],
        },);
        /**
         The page and the record the stop left.
         */
        const page = await pageOf({ dirs, },);
        const record = (await readInterimEntries({ interimDir: dirs.interimDir, },)).get(ENTRY_ID,);
        expect(page.includes(UNTRANSLATED_PASSAGE_NOTE,),).toBe(false,);
        expect(page.includes('猫猫有自己的碗',),).toBe(false,);
        expect([record?.settledSlices, record?.archiveSlices, record?.sourceSlices,],).toEqual([0, 2, 0,],);
      },
    },),

    it({
      name: 'SHIPS THE ORIGINAL UNDER ITS NOTE FOR A SOURCE-ONLY PASSAGE AFTER THE FOLD, the control: a passage the fold '
        + 'left a gap is one the lanes still owe, and the page says it has not been translated yet',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-standing-', },);
        const dirs = dirsOf({ root: root.path, },);
        await shipOver({
          dirs,
          error: new RangeError('scripted transport failure',),
          progress: progressWithGap({ folded: true, },),
          messages: [],
        },);
        /**
         The record the stop left.
         */
        const record = (await readInterimEntries({ interimDir: dirs.interimDir, },)).get(ENTRY_ID,);
        expect((await pageOf({ dirs, },)).includes(UNTRANSLATED_PASSAGE_NOTE,),).toBe(true,);
        expect([record?.settledSlices, record?.archiveSlices, record?.sourceSlices,],).toEqual([0, 1, 1,],);
      },
    },),

    it({
      name: 'CUTS THE SLICING OF A STOP BEFORE THE PREPARATION FROM THE ARCHIVE AS THE PASS NORMALISES IT (B1245): a '
        + 'stub marker is gone and an invisible variant is folded, as the real preparation reads the same bytes',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-standing-', },);
        const dirs = dirsOf({ root: root.path, },);
        await shipOver({
          dirs,
          error: new RangeError('scripted transport failure',),
          progress: startEntryProgress(),
          messages: [],
          targetText: STUB_ARCHIVE,
        },);
        expect(await pageOf({ dirs, },),).toBe(NORMALISED_ARCHIVE,);
      },
    },),

    it({
      name: 'CUTS THE SLICING UNDER THE FRONT MATTER AUTHORITY THE PASS DECIDES (B1245): an archive that translated its '
        + 'own front matter has no metadata slice for a lane to render, in the real preparation or here',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-standing-', },);
        const dirs = dirsOf({ root: root.path, },);
        /**
         Original and archive that both declare a name, the archive's rendered.
         */
        const source = `---\nname: 猫\n---\n\n${SOURCE_DOC}`;
        const target = `---\nname: Mittens\n---\n\n${TARGET_DOC}`;
        await shipOver({
          dirs,
          error: new RangeError('scripted transport failure',),
          progress: startEntryProgress(),
          messages: [],
          sourceText: source,
          targetText: target,
        },);
        /**
         Slices of the preparation under the authority the archive's translated name gives.
         */
        const expected = prepareDocumentPair({
          sourceText: source,
          targetText: target,
          frontMatterAuthority: 'archive',
          sealArchiveOriginal: true,
        },).slices.length;
        const record = (await readInterimEntries({ interimDir: dirs.interimDir, },)).get(ENTRY_ID,);
        expect(record?.archiveSlices,).toBe(expected,);
        expect(await pageOf({ dirs, },),).toBe(target,);
      },
    },),
  ],
},);
