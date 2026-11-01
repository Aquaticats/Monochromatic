/**
 Guards the archive's notices about its own completeness (XingZ60,
 2026-10-19). The archive opens with `**Come back later!**`, a warning that
 the version is not the formal one and will be updated after another version
 is finished, and a caution that it is unfinished and the reader should see
 another page. The original has none of them, and the archive translated only
 part of the original; the pass translated all of it, yet the shipped page kept
 all three. The archive-block review kept them: five judges called the notice
 "a faithful, plainly worded English notice about the page's own status",
 because nothing told them the page would carry a translation of every block
 of the original.

 A notice that the English page is unfinished is false on the page this pass
 publishes, so the archive's entry strip removes a paragraph that is nothing
 but one, as it removes a placeholder token, before any stage reads the
 archive. Only XingZ60 among the 92 pinned English archives carries one.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isStatusNoticeParagraph,
  stripStubMarkersWithOrigins,
} from '@monochromatic-dev/translation-repair-page-check';
import { passArchiveText, } from '../../dist/final/node/index.mjs';

/**
 An archive opening shaped as XingZ60's: front matter, three notices each a
 paragraph of its own, then the body.
 */
const NOTICE_OPENING = [
  '---',
  'name: Juzi Orange',
  '---',
  '',
  '**Come back later!**',
  '',
  '[!Warning] This version is NOT the formal version. Maintainers will update this after finishing the Siamese version.',
  '',
  '[!Caution] This is an unfinished version. Please refer to the Persian page.',
  '',
  '> The tabby said that napping is her right, not her duty.',
  '',
  'Her unfinished novel stayed in a drawer.',
  '',
].join('\n',);

/**
 The same opening with the three notices and their blank lines gone.
 */
const NOTICE_STRIPPED = [
  '---',
  'name: Juzi Orange',
  '---',
  '',
  '> The tabby said that napping is her right, not her duty.',
  '',
  'Her unfinished novel stayed in a drawer.',
  '',
].join('\n',);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: isStatusNoticeParagraph.name,
      children: [
        it({
          name: 'READS a paragraph made only of sentences that say come back later, that the page is not the finished '
            + 'one, who will update it, or which other page to see, in any case, wrapped in a callout label, emphasis '
            + 'or blockquote markers',
          fn: async () => {
            for (const paragraph of [
              '**Come back later!**',
              'Please check back later.',
              '[!Warning] This version is NOT the formal version. Maintainers will update this after finishing the '
                + 'Siamese version.',
              '> [!CAUTION] This is an unfinished version. Please refer to the Persian page.',
              'This page is under construction.',
              'The English version is incomplete! Translators will update this soon.',
              '_This translation is a work in progress._',
            ]) {
              expect(isStatusNoticeParagraph({ paragraph, },),).toBe(true,);
            }
          },
        },),
        it({
          name: 'LEAVES a sentence that merely says unfinished or under construction, a notice beside any other '
            + 'sentence, a pointer to a page in a story, and an empty paragraph',
          fn: async () => {
            for (const paragraph of [
              'Her unfinished novel stayed in a drawer.',
              'The cat house was under construction all spring.',
              'Come back later, she said, and the cat left.',
              'Come back later! The cat napped on.',
              'This is the unfinished business she kept returning to.',
              'Please refer to the diary she kept through the long and quiet winter of her last year.',
              '[!Warning]',
              '****',
              '',
            ]) {
              expect(isStatusNoticeParagraph({ paragraph, },),).toBe(false,);
            }
          },
        },),
      ],
    },),
    describe({
      name: stripStubMarkersWithOrigins.name,
      children: [
        it({
          name: 'REMOVES a paragraph that is nothing but a notice that the page is unfinished, with its blank line, '
            + 'and KEEPS the paragraphs around it, a sentence that merely says unfinished among them',
          fn: async () => {
            const { text, stripped, } = stripStubMarkersWithOrigins({ text: NOTICE_OPENING, },);
            expect(text,).toBe(NOTICE_STRIPPED,);
            expect(stripped.map(function lineOf({ lineNumber, },): number {
              return lineNumber;
            },),).toEqual([5, 7, 9,],);
          },
        },),
      ],
    },),
    describe({
      name: passArchiveText.name,
      children: [
        it({
          name: 'STRIPS the notices from the archive every stage reads',
          fn: async () => {
            expect(passArchiveText({
              text: NOTICE_OPENING,
              l: tagged({ tag: 'archive-status-notice-test', },),
            },),).toBe(NOTICE_STRIPPED,);
          },
        },),
      ],
    },),
  ],
},);
