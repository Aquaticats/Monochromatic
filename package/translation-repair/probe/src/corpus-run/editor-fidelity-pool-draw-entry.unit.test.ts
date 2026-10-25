/**
 Tests for the draw of one entry of the editor fidelity pool: which sections it takes for the insertion and the
 alteration, and why it leaves an entry out. The pages are invented; no clone is read.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  drawPoolEntry,
  type FidelityReferenceSpec,
} from '../../dist/final/node/index.mjs';
import {
  DRAW_CHINESE,
  DRAW_ENGLISH,
  DRAW_PIN,
  readInventedEntry,
} from '../editor-fidelity-pool.test-fixture.ts';

/**
 Draws the invented entry under an id.

 @param entryId - id the entry is drawn under

 @returns What the draw took and why it left things out

 @example
 ```ts
 const drawn = await drawnAs({ entryId: 'tabby', },);
 ```
 */
async function drawnAs({ entryId, }: { readonly entryId: string; },): ReturnType<typeof drawPoolEntry> {
  const pages = readInventedEntry({ entryId, },);
  if ('unreadable' in pages)
    throw new Error('the invented entry is unreadable',);
  return drawPoolEntry({
    entryId,
    zh: pages.zh,
    en: pages.en,
    corpusSha: DRAW_PIN.commitSha,
    reviewedOn: '2026-10-23',
  },);
}

await describe({
  name: drawPoolEntry.name,
  children: [
    it({
      name: 'TAKES THE FIRST SECTION WHOSE BORROWED-SENTENCE TWIN BUILDS FOR THE INSERTION and the first other section '
        + 'whose altered-number twin builds for the alteration, each as a specification with the damage it holds',
      fn: async () => {
        const { specs, } = await drawnAs({ entryId: 'tabby', },);
        expect(specs.map(function summary(spec: FidelityReferenceSpec,): string {
          return `${spec.id}:${spec.damages.map(function kindOf(damage,): string {
            return damage.kind;
          },).join('+',)}`;
        },),).toStrictEqual(['tabby-s0:insertion', 'tabby-s1:alteration',],);
      },
    },),

    it({
      name: 'BORROWS FROM THE FURTHEST OTHER SECTION: the donor range of the first section is the last section\'s',
      fn: async () => {
        const { specs, } = await drawnAs({ entryId: 'tabby', },);
        const [first,] = specs;
        expect(DRAW_ENGLISH.slice(
          first?.donor.startOffset,
          first?.donor.endOffset,
        ).startsWith('## Three',),).toBe(true,);
      },
    },),

    it({
      name: 'KEEPS ONLY IDS, OFFSETS AND HASHES: no field of a specification holds a passage of either page',
      fn: async () => {
        const { specs, } = await drawnAs({ entryId: 'tabby', },);
        const written = JSON.stringify(specs,);
        expect(written.includes('grey cat',),).toBe(false,);
        /**
         Opening of the Chinese page.
         */
        const opening = DRAW_CHINESE.slice(
          0,
          12,
        );
        expect(written.includes(opening,),).toBe(false,);
        for (const spec of specs)
          expect(spec.corpusSha,).toBe(DRAW_PIN.commitSha,);
      },
    },),

    it({
      name: 'LEAVES AN ENTRY OUT WHEN ITS PAGES ARE NOT OF EQUAL SHAPE, saying so, because the aligner would have to '
        + 'guess which section answers which',
      fn: async () => {
        const { specs, skipped, } = await drawnAs({ entryId: 'lopsided', },);
        expect(specs,).toStrictEqual([],);
        expect(skipped,).toStrictEqual(['entry not equal shape',],);
      },
    },),

    it({
      name: 'SAYS HOW BIG EACH SHEET IS, so the ceiling the probe prints can be checked against the draw',
      fn: async () => {
        const { sheetChars, } = await drawnAs({ entryId: 'tabby', },);
        expect(sheetChars.length,).toBe(2,);
        for (const chars of sheetChars)
          expect(chars,).toBeGreaterThan(DRAW_ENGLISH.length,);
      },
    },),
  ],
},);
