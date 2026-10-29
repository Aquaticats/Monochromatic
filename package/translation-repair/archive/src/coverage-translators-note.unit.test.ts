/**
 Guards the coverage sheet against a note between translators (ledger B698,
 B702): a judge asked whether a translation carries a passage reads the whole
 translation, and a reader never sees an HTML comment, so a note in it may
 neither carry the passage nor stand as an unrendered remainder. The original's
 own comments stay in view, and so do the words of a prior verdict's evidence
 that are not a note.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { userText, } from '@monochromatic-dev/translation-repair-provider/ts/chat-message-reading.test-fixture.ts';
import { buildCoverageMessages, } from '../dist/final/node/index.mjs';

/**
 Note the translators left each other beside the sentence it is about.
 */
const NOTE = '<!-- 前面这句猫话懂什么意思，先这么翻译，大家看一下 -->';

/**
 Comment the original itself carries.
 */
const ORIGINAL_COMMENT = '<!-- 猫窝：Cat Bed -->';

/**
 Original passage with its own comment.
 */
const PASSAGE = `猫睡在窗台上。${ORIGINAL_COMMENT}`;

/**
 Translation ending with the original's comment and the note.
 */
const TRANSLATION = `The cat slept on the windowsill. ${ORIGINAL_COMMENT} ${NOTE}`;

await describe({
  name: buildCoverageMessages.name,
  children: [
    it({
      name: 'SHOWS NO NOTE in the translation, and keeps the original\'s own comment in the passage and the translation',
      fn: async () => {
        /**
         What a coverage judge reads.
         */
        const sheet = userText({
          messages: buildCoverageMessages({
            sourcePassage: PASSAGE,
            translationText: TRANSLATION,
          },).messages,
        },);
        expect(sheet,).not.toContain('大家看一下',);
        expect(sheet,).toContain('The cat slept on the windowsill.',);
        expect(sheet.split(ORIGINAL_COMMENT,).length - 1,).toBe(2,);
      },
    },),
    it({
      name: 'SHOWS NO NOTE in the evidence of a prior verdict',
      fn: async () => {
        /**
         What a coverage judge reads on the follow-up pass.
         */
        const sheet = userText({
          messages: buildCoverageMessages({
            sourcePassage: PASSAGE,
            translationText: TRANSLATION,
            followupEvidence: {
              verdictKind: 'split',
              anchoredFull: 1,
              anchoredPartial: 0,
              absent: 1,
              heard: 2,
              asked: 3,
              evidence: [`The cat slept on the windowsill. ${NOTE}`,],
              missingDestinationCount: 0,
              shortfallAdmitted: false,
            },
          },).messages,
        },);
        expect(sheet,).not.toContain('大家看一下',);
        expect(sheet.split('The cat slept on the windowsill.',).length - 1,).toBe(2,);
      },
    },),
  ],
},);
