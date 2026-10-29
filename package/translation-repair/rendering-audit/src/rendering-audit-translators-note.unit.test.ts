/**
 Guards the rendering audit against a note between translators (ledger B698,
 B702): an auditor asked what a rendering gets wrong against the original reads
 the candidate, and a reader never sees an HTML comment, so a note in it may not
 be filed as untranslated text, an addition or a mistake. The original's own
 comments stay in view. The audit's answers are screened against the stored
 texts, so only what is shown is cut.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { messageText, } from '@monochromatic-dev/translation-repair-provider';
import { buildRenderingAuditMessages, } from '../dist/final/node/index.mjs';

/**
 Note the translators left each other beside the sentence it is about.
 */
const NOTE = '<!-- 前面这句猫话懂什么意思，先这么翻译，大家看一下 -->';

/**
 Comment the original itself carries.
 */
const ORIGINAL_COMMENT = '<!-- 猫窝：Cat Bed -->';

await describe({
  name: buildRenderingAuditMessages.name,
  children: [
    it({
      name: 'SHOWS NO NOTE in the candidate, and keeps the original\'s own comment in both texts',
      fn: async () => {
        /**
         What an auditor reads.
         */
        const sheet = buildRenderingAuditMessages({
          subject: {
            sourceText: `猫睡在窗台上。${ORIGINAL_COMMENT}`,
            candidateText: `The cat slept on the windowsill. ${ORIGINAL_COMMENT}\n${NOTE}`,
          },
        },)
          .map(function content(message,): string {
            return messageText({ message, },);
          },)
          .join('\n',);
        expect(sheet,).not.toContain('大家看一下',);
        expect(sheet,).toContain('The cat slept on the windowsill.',);
        expect(sheet.split(ORIGINAL_COMMENT,).length - 1,).toBe(2,);
      },
    },),
  ],
},);
