/**
 Guards the sheet lines for a Han name the archive spells inside its
 sentences. The judges of a polish, a selection and a panel were shown the
 archive's link texts, signatures and headings and nothing for a cited
 author's handle written in a sentence, so a rule that asks for a regrouping
 and a gloss of "a handle with no archive rendering" read as if it applied.
 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  pageIdentityLines,
  pageNameLines,
  pairedPageNames,
} from '../dist/final/node/index.mjs';

/**
 Original naming a cited author in a sentence.
 */
const SOURCE_TEXT = '在她离世后，雪团子为她写下了[悼文](https://example.invalid/memorial)。';

/**
 Archive spelling that author in its sentence.
 */
const ARCHIVE_TEXT = 'After her departure, Xue Tuanzi wrote an [epitaph](https://example.invalid/memorial) for her.';

/**
 Heading of the block that names the archive's spellings inside its sentences.
 */
const PROSE_HEADING = 'NAMES THE ARCHIVE SPELLS IN LATIN LETTERS INSIDE ITS SENTENCES ON THIS PAGE (the archive\'s spelling of '
  + 'a name is its rendering, so the rule for a handle with no declared, archive or corpus rendering does not apply '
  + 'to it: no regrouping and no meaning in parentheses is asked for, and a rewrite keeps the spelling as the archive '
  + 'writes it):';

await describe({
  name: 'a page names the people its sentences spell as the archive spells them',
  children: [
    describe({
      name: pageNameLines.name,
      children: [
        it({
          name: 'LISTS a Han name the archive spells in a sentence, under a heading that says its spelling is the rendering',
          fn: async () => {
            const lines = pageNameLines({ sourceText: SOURCE_TEXT, targetText: ARCHIVE_TEXT, },);
            expect(lines,).toEqual([
              'NAMES AND LINKED TEXT THE ARCHIVE RENDERS ON THIS PAGE (render the same person or title the same way everywhere; a declared name inside a title takes its declared form):',
              '- 悼文 (link text, https://example.invalid/memorial): "epitaph"',
              PROSE_HEADING,
              '- 雪团子: "Xue Tuanzi"',
            ],);
          },
        },),
        it({
          name: 'LISTS it with no other block when the page pairs no link, signature or heading',
          fn: async () => {
            expect(pageNameLines({
              sourceText: '雪团子来了。',
              targetText: 'Xue Tuanzi came.',
            },),).toEqual([PROSE_HEADING, '- 雪团子: "Xue Tuanzi"',],);
          },
        },),
        it({
          name: 'LEAVES OUT a name another block already pairs, and a name the front matter declares',
          fn: async () => {
            expect(pageNameLines({
              sourceText: '雪团子来了。\n\n——雪团子, 2024 年 12 月 17 日\n',
              targetText: 'Xue Tuanzi came.\n\n——Xue Tuanzi, December 17, 2024\n',
            },),).toEqual([
              'NAMES AND LINKED TEXT THE ARCHIVE RENDERS ON THIS PAGE (render the same person or title the same way everywhere; a declared name inside a title takes its declared form):',
              '- 雪团子 (signature): "Xue Tuanzi"',
            ],);
            expect(pageNameLines({
              sourceText: SOURCE_TEXT,
              targetText: ARCHIVE_TEXT,
              declared: [{ source: '雪团子', rendering: 'Snow Dumpling', },],
            },),).toEqual([
              'NAMES AND LINKED TEXT THE ARCHIVE RENDERS ON THIS PAGE (render the same person or title the same way everywhere; a declared name inside a title takes its declared form):',
              '- 悼文 (link text, https://example.invalid/memorial): "epitaph"',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: pairedPageNames.name,
      children: [
        it({
          name: 'LEAVES a name spelled in a sentence out of the names the title stage leaves to the block',
          fn: async () => {
            expect([...pairedPageNames({ sourceText: SOURCE_TEXT, targetText: ARCHIVE_TEXT, },),],).toEqual(['悼文',],);
          },
        },),
      ],
    },),
    describe({
      name: pageIdentityLines.name,
      children: [
        it({
          name: 'CARRIES the block into the identity lines every sheet reads',
          fn: async () => {
            const lines = pageIdentityLines({
              sourceDocument: parseDocument({ text: SOURCE_TEXT, },),
              targetDocument: parseDocument({ text: ARCHIVE_TEXT, },),
              sourceText: SOURCE_TEXT,
              targetText: ARCHIVE_TEXT,
              contextLines: [],
            },);
            expect(lines,).toContain('- 雪团子: "Xue Tuanzi"',);
          },
        },),
      ],
    },),
  ],
},);
