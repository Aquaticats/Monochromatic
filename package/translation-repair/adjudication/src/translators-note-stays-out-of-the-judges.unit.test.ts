/**
 Guards the judges of the repair lane and the candidate selection slates
 against a note between translators (ledger B698, B702): the critic, the panel,
 the introduced-defect prober and the resolution checker each read the archive's
 text, and a reader never sees an HTML comment, so none may raise, weigh or
 excuse one. The critic is where it starts: it is told to report Han left in an
 English text as untranslated, and an accepted issue becomes an envelope an
 editor renders, so a note put into English there is a comment the carry then
 puts back beside.

 The judges are never shown the note; the original's own comments stay in view.
 Every answer is located in the stored text, which the cut never touches.

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
  buildAdjudicationMessages,
  buildCandidateSelectMessages,
  buildCriticMessages,
  buildIntroducedDefectMessages,
  buildResolutionMessages,
  selectDecision,
} from '../dist/final/node/index.mjs';

/**
 Note the translators left each other beside the sentence it is about.
 */
const TRANSLATORS_NOTE = '<!-- 前面这句猫话懂什么意思，先这么翻译，大家看一下 -->';

/**
 Words of that note, as a sheet that shows it prints them.
 */
const NOTE_WORDS = '大家看一下';

/**
 Comment the original itself carries, in Han, which the pipeline carries on.
 */
const ORIGINAL_COMMENT = '<!-- 猫窝：Cat Bed -->';

/**
 Original passage with its own comment.
 */
const SOURCE = `猫睡在窗台上。${ORIGINAL_COMMENT}`;

/**
 Archive text carrying the original's comment and the note.
 */
const ARCHIVE = `The cat slept on the windowsill. ${ORIGINAL_COMMENT} ${TRANSLATORS_NOTE}`;

/**
 Archive text of the passage next door, ending with a note of its own.
 */
const NEXT_DOOR = `The kitten purred. ${TRANSLATORS_NOTE}`;

/**
 Original of the passage next door.
 */
const NEXT_DOOR_SOURCE = '小猫呼噜了。';

/**
 Label of evidence that holds the current English and merely names the
 original in its own words, as the objection correction's slate labels it.
 */
const CURRENT_LABEL = 'CURRENT English translation, which ships unchanged unless a candidate resolves an objection the '
  + 'ORIGINAL supports';

/**
 Label of evidence that holds the surrounding existing translation and names
 the original in its own words, as the translate slate labels it.
 */
const SURROUNDING_LABEL = 'SURROUNDING EXISTING TRANSLATION (English), context only: wording that belongs to the '
  + 'passages either side of this one. Where wording the ORIGINAL of THIS passage calls for is already sitting here, '
  + 'the archive carried it across a boundary.';

/**
 Asserts that a sheet shows the text without the note and keeps the original's
 own comment.

 @param sheet - what the judge reads

 @example
 ```ts
 showsWithoutNote({ sheet, },);
 ```
 */
function showsWithoutNote({ sheet, }: { readonly sheet: string; },): void {
  expect(sheet,).not.toContain(NOTE_WORDS,);
  expect(sheet,).toContain('The cat slept on the windowsill.',);
  expect(sheet,).toContain(ORIGINAL_COMMENT,);
}

await describe({
  name: 'a note between translators stays out of the judges',
  children: [
    describe({
      name: buildCriticMessages.name,
      children: [
        it({
          name: 'SHOWS NO NOTE in the translation or in the existing translation next door, and keeps the original\'s '
            + 'own comment and the original next door whole',
          fn: async () => {
            /**
             What the critic reads.
             */
            const sheet = userText({
              messages: buildCriticMessages({
                sourceText: SOURCE,
                targetText: ARCHIVE,
                neighbouringSourceText: NEXT_DOOR_SOURCE,
                neighbouringIncumbentText: NEXT_DOOR,
              },),
            },);
            showsWithoutNote({ sheet, },);
            expect(sheet,).toContain('The kitten purred.',);
            expect(sheet,).toContain(NEXT_DOOR_SOURCE,);
          },
        },),
      ],
    },),
    describe({
      name: buildAdjudicationMessages.name,
      children: [
        it({
          name: 'SHOWS NO NOTE in the translation or in the existing translation next door',
          fn: async () => {
            /**
             What a panelist reads.
             */
            const sheet = userText({
              messages: buildAdjudicationMessages({
                sourceText: SOURCE,
                targetText: ARCHIVE,
                clusters: [],
                neighbouringSourceText: NEXT_DOOR_SOURCE,
                neighbouringIncumbentText: NEXT_DOOR,
              },).messages,
            },);
            showsWithoutNote({ sheet, },);
            expect(sheet,).toContain('The kitten purred.',);
          },
        },),
      ],
    },),
    describe({
      name: buildIntroducedDefectMessages.name,
      children: [
        it({
          name: 'SHOWS NO NOTE in the baseline, in the existing translation next door or in either text of a region',
          fn: async () => {
            /**
             What a prober reads.
             */
            const sheet = userText({
              messages: buildIntroducedDefectMessages({
                sourceText: SOURCE,
                baselineText: ARCHIVE,
                regions: [{
                  envelopeId: 'envelope/tail',
                  issueIds: [],
                  before: `The cat slept. ${TRANSLATORS_NOTE}`,
                  editorAfter: `The cat napped on the windowsill. ${TRANSLATORS_NOTE}`,
                },],
                issues: [],
                neighbouringSourceText: NEXT_DOOR_SOURCE,
                neighbouringIncumbentText: NEXT_DOOR,
              },).messages,
            },);
            showsWithoutNote({ sheet, },);
            expect(sheet,).toContain('The cat napped on the windowsill.',);
            expect(sheet,).toContain('The kitten purred.',);
          },
        },),
      ],
    },),
    describe({
      name: buildResolutionMessages.name,
      children: [
        it({
          name: 'SHOWS NO NOTE in the revised translation',
          fn: async () => {
            /**
             What a checker reads.
             */
            const sheet = userText({
              messages: buildResolutionMessages({
                sourceText: SOURCE,
                patchedText: ARCHIVE,
                issues: [],
              },).messages,
            },);
            showsWithoutNote({ sheet, },);
          },
        },),
      ],
    },),
    describe({
      name: buildCandidateSelectMessages.name,
      children: [
        it({
          name: 'CUTS THE NOTE FROM EVIDENCE WHOSE HEADING ONLY NAMES THE ORIGINAL, and from the candidates beside it',
          fn: async () => {
            for (const label of [CURRENT_LABEL, SURROUNDING_LABEL,]) {
              /**
               What a judge reads: the original, one block of English whose heading names the original in its own
               words, and two candidates ending with the same note.
               */
              const sheet = userText({
                messages: buildCandidateSelectMessages({
                  task: 'Each candidate revises the English.',
                  criteria: ['Faithful to the Chinese ORIGINAL.',],
                  evidence: [
                    {
                      label: 'ORIGINAL (Chinese)',
                      text: SOURCE,
                    },
                    {
                      label,
                      text: ARCHIVE,
                    },
                  ],
                  rendered: [
                    ARCHIVE,
                    `The cat napped on the windowsill. ${ORIGINAL_COMMENT} ${TRANSLATORS_NOTE}`,
                  ],
                  sourceText: SOURCE,
                },),
              },);
              expect(sheet,).not.toContain(NOTE_WORDS,);
              expect(sheet,).toContain('The cat napped on the windowsill.',);
              expect(sheet,).toContain(ORIGINAL_COMMENT,);
            }
          },
        },),
        it({
          name: 'KEEPS WHOLE EVIDENCE WHOSE HEADING NAMES IT THE ORIGINAL, however the heading goes on',
          fn: async () => {
            for (
              const label of [
                'ORIGINAL (Chinese)',
                'ORIGINAL (Chinese), the standard',
                'SURROUNDING ORIGINAL (Chinese), context only: the candidates are not expected to render this',
                'EXPECTED ORIGINAL SECTION',
                'NEARBY ORIGINAL, source evidence for the current passage rather than additional coverage',
                'FULL ORIGINAL DOCUMENT, factual evidence only: use it to check current claims',
              ]
            ) {
              /**
               What a judge reads when the original itself holds a comment in Han that no candidate repeats.
               */
              const sheet = userText({
                messages: buildCandidateSelectMessages({
                  task: 'Each candidate revises the English.',
                  criteria: ['Faithful to the Chinese ORIGINAL.',],
                  evidence: [{
                    label,
                    text: `猫睡了。${TRANSLATORS_NOTE}`,
                  },],
                  rendered: ['The cat slept.',],
                },),
              },);
              expect(sheet,).toContain(NOTE_WORDS,);
            }
          },
        },),
      ],
    },),
    describe({
      name: selectDecision.name,
      children: [
        it({
          name: 'CUTS THE NOTE FROM EVIDENCE WHOSE HEADING ONLY NAMES THE ORIGINAL, in the state a decision seat reads',
          fn: async () => {
            /**
             The state and question a decision seat reads.
             */
            const decision = selectDecision({
              task: 'Each candidate revises the English.',
              criteria: ['Faithful to the Chinese ORIGINAL.',],
              evidence: [
                {
                  label: 'ORIGINAL (Chinese)',
                  text: SOURCE,
                },
                {
                  label: SURROUNDING_LABEL,
                  text: NEXT_DOOR,
                },
              ],
              rendered: [
                ARCHIVE,
                `The cat napped on the windowsill. ${TRANSLATORS_NOTE}`,
              ],
              sourceText: SOURCE,
            },);
            /**
             Everything the seat reads, as JSON.
             */
            const read = JSON.stringify({
              state: decision.state,
              questions: decision.questions,
            },);
            expect(read,).not.toContain(NOTE_WORDS,);
            expect(read,).toContain('The kitten purred.',);
          },
        },),
      ],
    },),
  ],
},);
