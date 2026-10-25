/**
 Tests for the heading sections the editor fidelity pool draw takes its trials from: how an entry's two pages pair up
 through the production aligner, and that an entry whose pages are not of equal shape has none. Pages are cat-themed
 invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { sectionsOf, } from '../../dist/final/node/index.mjs';
import {
  DRAW_CHINESE,
  DRAW_ENGLISH,
  readInventedEntry,
} from '../editor-fidelity-pool.test-fixture.ts';

await describe({
  name: sectionsOf.name,
  children: [
    it({
      name: 'PAIRS EACH HEADING SECTION OF THE ORIGINAL WITH THE ONE OF THE ARCHIVE by position, giving each its ranges '
        + 'in both pages and the archive text it covers byte for byte',
      fn: async () => {
        const sections = sectionsOf({
          zh: DRAW_CHINESE,
          en: DRAW_ENGLISH,
        },);
        expect(sections.map(function indexOf(section,): number {
          return section.index;
        },),).toStrictEqual([0, 1, 2,],);
        for (const section of sections) {
          expect(DRAW_ENGLISH.slice(
            section.archive.startOffset,
            section.archive.endOffset,
          ),).toBe(section.archiveRaw,);
          expect(DRAW_CHINESE.slice(
            section.source.startOffset,
            section.source.endOffset,
          ),).toBe(section.sourceRaw,);
        }
        expect(sections[1]?.archiveRaw.startsWith('## Two',),).toBe(true,);
      },
    },),

    it({
      name: 'HOLDS NO SECTION OF AN ENTRY WHOSE PAGES ARE NOT OF EQUAL SHAPE, since the aligner would have to guess '
        + 'which section answers which',
      fn: async () => {
        const pages = readInventedEntry({ entryId: 'lopsided', },);
        if ('unreadable' in pages)
          throw new Error('the invented entry is unreadable',);
        expect(sectionsOf({
          zh: pages.zh,
          en: pages.en,
        },),).toStrictEqual([],);
      },
    },),
  ],
},);
