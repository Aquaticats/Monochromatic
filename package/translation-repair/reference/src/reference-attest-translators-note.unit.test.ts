/**
 Guards the reference attestation sheets against a note between translators
 (ledger B698, B702): a model asked which details the archive rendering states
 that the original does not reads the archive's text, and a reader never sees
 an HTML comment, so a note may not stand as a detail the archive adds. The
 original's own comments stay in view. The quotes a model gives are verified
 against the stored archive text, so only what is shown is cut.

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
  buildReferenceAttestConfirmMessages,
  buildReferenceAttestMessages,
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
 Original with its own comment.
 */
const SOURCE = `猫睡在窗台上。${ORIGINAL_COMMENT}`;

/**
 Archive rendering ending with the original's comment and the note.
 */
const ARCHIVE = `The cat slept on the windowsill. ${ORIGINAL_COMMENT}\n${NOTE}`;

/**
 Reference line the sheets cite.
 */
const REFERENCES = '1. The windowsill faces the garden.';

await describe({
  name: 'a note between translators stays out of the reference attestation sheets',
  children: [
    describe({
      name: buildReferenceAttestMessages.name,
      children: [
        it({
          name: 'SHOWS NO NOTE in the archive rendering, and keeps the original\'s own comment in both texts',
          fn: async () => {
            /**
             What an attesting voice reads.
             */
            const sheet = userText({
              messages: buildReferenceAttestMessages({
                sourceText: SOURCE,
                archiveText: ARCHIVE,
                referenceContext: REFERENCES,
              },),
            },);
            expect(sheet,).not.toContain('大家看一下',);
            expect(sheet,).toContain('The cat slept on the windowsill.',);
            expect(sheet.split(ORIGINAL_COMMENT,).length - 1,).toBe(2,);
          },
        },),
      ],
    },),
    describe({
      name: buildReferenceAttestConfirmMessages.name,
      children: [
        it({
          name: 'SHOWS NO NOTE in the archive rendering, and keeps the original\'s own comment in both texts',
          fn: async () => {
            /**
             What a confirming voice reads.
             */
            const sheet = userText({
              messages: buildReferenceAttestConfirmMessages({
                sourceText: SOURCE,
                archiveText: ARCHIVE,
                referenceContext: REFERENCES,
                candidates: [{
                  archiveQuote: 'The cat slept on the windowsill.',
                  reference: 1,
                  referenceQuote: 'The windowsill faces the garden.',
                  voices: 2,
                  heard: 3,
                },],
              },),
            },);
            expect(sheet,).not.toContain('大家看一下',);
            expect(sheet,).toContain('The cat slept on the windowsill.',);
            expect(sheet.split(ORIGINAL_COMMENT,).length - 1,).toBe(2,);
          },
        },),
      ],
    },),
  ],
},);
