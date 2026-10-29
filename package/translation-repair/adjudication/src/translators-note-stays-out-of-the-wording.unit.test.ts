/**
 Guards every candidate slate against a note between translators (XingZ60
 slice 6, 2026-10-19). The translate, repair and archive-block slates all put
 their candidates and the archive's wording through one sheet and one typed
 question, so a note the archive carries beside a sentence reached each of
 them as part of the wording they compare. A reader never sees an HTML
 comment: a candidate may neither win nor lose a vote for carrying one.

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
  buildCandidateSelectMessages,
  selectDecision,
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
 The request every slate builds: the original, the archive carrying the note,
 and two candidates of which only one carries it.
 */
const REQUEST = {
  task: 'Pick one.',
  criteria: ['Faithful.',],
  evidence: [
    {
      label: 'ORIGINAL (Chinese)',
      text: `猫睡在窗台上。${ORIGINAL_COMMENT}`,
    },
    {
      label: 'CURRENT English translation',
      text: `The cat slept on the sill. ${ORIGINAL_COMMENT} ${TRANSLATORS_NOTE}`,
    },
  ],
  rendered: [
    `The cat slept on the windowsill. ${ORIGINAL_COMMENT} ${TRANSLATORS_NOTE}`,
    `The cat napped on the sill. ${ORIGINAL_COMMENT}`,
  ],
  sourceText: `猫睡在窗台上。${ORIGINAL_COMMENT}`,
};

/**
 What the chat sheet shows a judge.
 */
const chatSheet = buildCandidateSelectMessages(REQUEST,)
  .map(function contentOf(message,): string {
    return message.content;
  },)
  .join('\n',);

/**
 What the typed question shows a decision seat.
 */
const typedState = JSON.stringify(selectDecision(REQUEST,).state,);

await describe({
  name: buildCandidateSelectMessages.name,
  children: [
    it({
      name: 'SHOWS NO NOTE BETWEEN TRANSLATORS in the archive or in a candidate',
      fn: async () => {
        expect(chatSheet,).not.toContain('大家看一下',);
        expect(chatSheet,).toContain('The cat slept on the windowsill.',);
        expect(chatSheet,).toContain(ORIGINAL_COMMENT,);
      },
    },),
    describe({
      name: selectDecision.name,
      children: [
        it({
          name: 'SHOWS A DECISION SEAT NO NOTE BETWEEN TRANSLATORS either',
          fn: async () => {
            expect(typedState,).not.toContain('大家看一下',);
            expect(typedState,).toContain('The cat slept on the windowsill.',);
            expect(typedState,).toContain(ORIGINAL_COMMENT,);
          },
        },),
      ],
    },),
  ],
},);
