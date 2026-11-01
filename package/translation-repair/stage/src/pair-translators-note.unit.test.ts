/**
 Guards the pairing sheets against a note between translators (ledger B698,
 B702): a model asked which original section or block each translation one
 renders reads the translation's text, and a reader never sees an HTML comment,
 so a note may neither make a block look like a rendering of anything nor
 make a section look as if it had a body. The original's own comments stay in
 view. Every index the sheet lists stays, the list keeps its length, and a
 block that held nothing but a note is listed empty, so the model's pairs still
 name the blocks the caller numbered.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { userText, } from '@monochromatic-dev/translation-repair-provider/ts/chat-message-reading.test-fixture.ts';
import {
  buildBlockPairingMessages,
  buildSectionPairingMessages,
} from '../dist/final/node/index.mjs';

/**
 Note the translators left each other beside the sentence it is about.
 */
const NOTE = '<!-- 前面这句猫话懂什么意思，先这么翻译，大家看一下 -->';

/**
 Comment the original itself carries.
 */
const ORIGINAL_COMMENT = '<!-- 猫窝：Cat Bed -->';

/**
 Words of the note, as a sheet that shows it prints them.
 */
const NOTE_WORDS = '大家看一下';

/**
 Original blocks, the first of which carries a comment of its own.
 */
const SOURCE_BLOCKS = [
  {
    index: 0,
    text: `猫睡在窗台上。${ORIGINAL_COMMENT}`,
  },
  {
    index: 1,
    text: '小猫呼噜了。',
  },
];

/**
 Translation blocks: a rendering that keeps the original's comment and ends with the note, a block that holds only the
 note, and a rendering with no note.
 */
const TARGET_BLOCKS = [
  {
    index: 0,
    text: `The cat slept on the windowsill. ${ORIGINAL_COMMENT}\n${NOTE}`,
  },
  {
    index: 1,
    text: NOTE,
  },
  {
    index: 2,
    text: 'The kitten purred.',
  },
];

await describe({
  name: 'a note between translators stays out of the pairing sheets',
  children: [
    describe({
      name: buildBlockPairingMessages.name,
      children: [
        it({
          name: 'SHOWS NO NOTE, keeps the original\'s own comment, and lists every index of both sides',
          fn: async () => {
            /**
             What the pairing model reads.
             */
            const sheet = userText({
              messages: buildBlockPairingMessages({
                sourceBlocks: SOURCE_BLOCKS,
                targetBlocks: TARGET_BLOCKS,
              },),
            },);
            expect(sheet,).not.toContain(NOTE_WORDS,);
            expect(sheet,).toContain(`The cat slept on the windowsill. ${ORIGINAL_COMMENT}`,);
            expect(sheet,).toContain('The kitten purred.',);
            for (const index of [0, 1, 2,]) {
              expect(sheet,).toContain(`[${String(index,)}]`,);
            }
          },
        },),
      ],
    },),
    describe({
      name: buildSectionPairingMessages.name,
      children: [
        it({
          name: 'SHOWS NO NOTE, keeps the original\'s own comment, and lists every index of both sides',
          fn: async () => {
            /**
             What the pairing model reads.
             */
            const sheet = userText({
              messages: buildSectionPairingMessages({
                sourceSections: [
                  {
                    index: 0,
                    text: `## 第一节\n\n猫睡在窗台上。${ORIGINAL_COMMENT}`,
                  },
                  {
                    index: 1,
                    text: '## 第二节\n\n小猫呼噜了。',
                  },
                ],
                targetSections: [
                  {
                    index: 0,
                    text: `## Naps\n\nThe cat slept on the windowsill. ${ORIGINAL_COMMENT}\n${NOTE}`,
                  },
                  {
                    index: 1,
                    text: `## Purrs\n\n${NOTE}`,
                  },
                ],
              },),
            },);
            expect(sheet,).not.toContain(NOTE_WORDS,);
            expect(sheet,).toContain(`The cat slept on the windowsill. ${ORIGINAL_COMMENT}`,);
            expect(sheet,).toContain('## Purrs',);
            for (const index of [0, 1,]) {
              expect(sheet.split(`[${String(index,)}]`,).length - 1,).toBe(2,);
            }
          },
        },),
      ],
    },),
  ],
},);
