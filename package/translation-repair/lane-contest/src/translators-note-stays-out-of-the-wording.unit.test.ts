/**
 Guards the lane contest's candidate texts against a note between translators
 (XingZ60 slice 6, 2026-10-19). The archive's English ended a sentence with an
 HTML comment asking the other translators whether anyone understood the
 Chinese before it. The repair lane kept the comment as markup, the translate
 lane wrote a cleaner rendering without it, and all three contest judges
 chose repair, one of them writing that the translate candidate "drops the
 editor's comment note from the archive rendering, which is page apparatus".
 A reader never sees an HTML comment, so the vote was decided by text no
 reader reads, and the rendering that read better lost it.

 Sheet carve-outs do not move judges (`house-rule/src/declared-name-survival.ts`),
 so the judges are never shown the note: it can neither win nor lose a vote.
 The original's own comments are markup the pipeline carries and stay in view.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { buildLaneContestMessages, } from '../dist/final/node/index.mjs';

/**
 Note the translators left each other beside the sentence it is about.
 */
const TRANSLATORS_NOTE = '<!-- 前面这句猫话懂什么意思，先这么翻译，大家看一下 -->';

/**
 Comment the original itself carries, in Han, which the pipeline carries on.
 */
const ORIGINAL_COMMENT = '<!-- 猫窝：Cat Bed -->';

/**
 What a contest judge is shown for a slice whose archive ends in the note.
 */
const SUBJECT = {
  lineStructured: false,
  sourceText: `猫睡在窗台上。${ORIGINAL_COMMENT}`,
  incumbentText: `The cat slept on the sill. ${ORIGINAL_COMMENT} ${TRANSLATORS_NOTE}`,
  repairText: `The cat slept on the windowsill. ${ORIGINAL_COMMENT} ${TRANSLATORS_NOTE}`,
  translateText: `The cat napped on the sill. ${ORIGINAL_COMMENT}`,
};

/**
 Everything the judge reads, as one string.
 */
const shown = buildLaneContestMessages({ subject: SUBJECT, },)
  .map(function contentOf(message,): string {
    return message.content;
  },)
  .join('\n',);

await describe({
  name: buildLaneContestMessages.name,
  children: [
    it({
      name: 'SHOWS NO NOTE BETWEEN TRANSLATORS in any text it compares, so the note can neither win nor lose a vote',
      fn: async () => {
        expect(shown,).not.toContain('大家看一下',);
        expect(shown,).not.toContain(TRANSLATORS_NOTE,);
      },
    },),
    it({
      name: 'KEEPS the wording around the note and the original\'s own comment in view',
      fn: async () => {
        expect(shown,).toContain('The cat slept on the windowsill.',);
        expect(shown,).toContain('The cat napped on the sill.',);
        expect(shown,).toContain(ORIGINAL_COMMENT,);
      },
    },),
  ],
},);
