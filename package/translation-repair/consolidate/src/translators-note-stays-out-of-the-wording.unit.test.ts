/**
 Guards the consolidation gate and the polish gate against a note between
 translators (XingZ60 slice 6, 2026-10-19). The consolidated text that gate
 judges carried the archive's comment asking the other translators about the
 Chinese before it, and two of three ballots wrote that both candidates
 "correctly keep the archive's editor comment". A reader never sees an HTML
 comment, so a candidate may neither win nor lose a vote for carrying one.

 The judges are never shown the note; the original's own comments stay in view.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  buildConsolidateGateMessages,
  buildConsolidationPolishGateMessages,
} from '../dist/final/node/index.mjs';

/**
 Note the translators left each other beside the sentence it is about.
 */
const TRANSLATORS_NOTE = '<!-- 前面这句猫话懂什么意思，先这么翻译，大家看一下 -->';

/**
 Comment the original itself carries, in Han, which the pipeline carries on.
 */
const ORIGINAL_COMMENT = '<!-- 猫窝：Cat Bed -->';

/**
 Original passage with its own comment.
 */
const SOURCE = `猫睡在窗台上。${ORIGINAL_COMMENT}`;

/**
 Joins what a judge reads.

 @param messages - exchange built for a judge

 @returns Every message's content in one string
 */
function shownIn(messages: readonly { readonly content: string; }[],): string {
  return messages
    .map(function contentOf(message,): string {
      return message.content;
    },)
    .join('\n',);
}

/**
 Gate sheet over a consolidation carrying the note against a standing text
 that does not.
 */
const gate = shownIn(buildConsolidateGateMessages({
  subject: {
    lineStructured: false,
    sourceText: SOURCE,
    incumbentText: `The cat slept on the sill. ${ORIGINAL_COMMENT} ${TRANSLATORS_NOTE}`,
    consolidatedText: `The cat slept on the windowsill. ${ORIGINAL_COMMENT} ${TRANSLATORS_NOTE}`,
    standingText: `The cat napped on the sill. ${ORIGINAL_COMMENT}`,
  },
},),);

/**
 Polish gate sheet over a polish carrying the note against a base that does not.
 */
const polish = shownIn(buildConsolidationPolishGateMessages({
  subject: {
    lineStructured: false,
    sourceText: SOURCE,
    archiveText: `The cat slept on the sill. ${ORIGINAL_COMMENT} ${TRANSLATORS_NOTE}`,
    baseText: `The cat napped on the sill. ${ORIGINAL_COMMENT}`,
    polishedText: `The cat slept on the windowsill. ${ORIGINAL_COMMENT} ${TRANSLATORS_NOTE}`,
    mode: { kind: 'comparative', },
  },
},),);

await describe({
  name: buildConsolidateGateMessages.name,
  children: [
    it({
      name: 'SHOWS NO NOTE BETWEEN TRANSLATORS in any text it compares',
      fn: async () => {
        expect(gate,).not.toContain('大家看一下',);
        expect(gate,).toContain('The cat slept on the windowsill.',);
        expect(gate,).toContain(ORIGINAL_COMMENT,);
      },
    },),
    describe({
      name: buildConsolidationPolishGateMessages.name,
      children: [
        it({
          name: 'SHOWS NO NOTE BETWEEN TRANSLATORS in any text it compares',
          fn: async () => {
            expect(polish,).not.toContain('大家看一下',);
            expect(polish,).toContain('The cat slept on the windowsill.',);
            expect(polish,).toContain(ORIGINAL_COMMENT,);
          },
        },),
      ],
    },),
  ],
},);
