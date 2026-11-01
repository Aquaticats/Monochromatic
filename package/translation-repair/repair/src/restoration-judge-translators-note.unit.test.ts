/**
 Guards the restoration judge against a note between translators (ledger B698,
 B702): a judge asked whether a repaired translation restores deleted
 sentences reads the repaired text, and a reader never sees an HTML comment, so
 a note in it may not stand as restored wording. The original's own comments
 stay in view. The judge's answers resolve through seed numbers, not quotes, so
 only what is shown is cut.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { userText, } from '@monochromatic-dev/translation-repair-provider/ts/chat-message-reading.test-fixture.ts';
import { buildRestorationJudgeMessages, } from '../dist/final/node/index.mjs';

/**
 Note the translators left each other beside the sentence it is about.
 */
const NOTE = '<!-- 前面这句猫话懂什么意思，先这么翻译，大家看一下 -->';

/**
 Comment the original itself carries.
 */
const ORIGINAL_COMMENT = '<!-- 猫窝：Cat Bed -->';

await describe({
  name: buildRestorationJudgeMessages.name,
  children: [
    it({
      name: 'SHOWS NO NOTE in the repaired translation, and keeps the original\'s own comment in both texts',
      fn: async () => {
        /**
         What the judge reads.
         */
        const sheet = userText({
          messages: buildRestorationJudgeMessages({
            sourceText: `猫睡在窗台上。${ORIGINAL_COMMENT}`,
            repairedText: `The cat slept on the windowsill. ${ORIGINAL_COMMENT}\n${NOTE}`,
            references: [{
              seedId: 'seed/omission-0',
              deletedText: 'The cat slept on the windowsill.',
            },],
          },).messages,
        },);
        expect(sheet,).not.toContain('大家看一下',);
        expect(sheet,).toContain('REFERENCE 1: The cat slept on the windowsill.',);
        expect(sheet.split(ORIGINAL_COMMENT,).length - 1,).toBe(2,);
      },
    },),
  ],
},);
