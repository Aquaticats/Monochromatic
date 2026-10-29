/**
 Tests absolute naturalness reviewer prompt and reply consistency guard.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { messageText, } from '@monochromatic-dev/translation-repair-provider';
import {
  type AbsoluteNaturalnessReviewSubject,
  type AbsoluteNaturalnessReviewWire,
  buildAbsoluteNaturalnessReviewMessages,
  isAbsoluteNaturalnessReviewWire,
} from '../dist/final/node/index.mjs';

/**
 Otherwise-valid reply each structural-malformation case corrupts in one field.
 */
const VALID_REVIEW: AbsoluteNaturalnessReviewWire = {
  isAcceptable: false,
  findings: [{ paragraph: 2, problem: 'Avoid literal word-for-word order.', },],
  assessment: 'retains source phrasing',
};

/**
 Note the archive's translators left each other, ending a sentence.
 */
const SENTENCE_NOTE = '<!-- 前面这句中文懂什么意思，先这么翻译，大家看一下 -->';

/**
 Note alone on a line, in JSX comment form.
 */
const LINE_NOTE = '{/* 猫窝在哪里 */}';

/**
 Note on a quoted line, with a `>` among its words.
 */
const QUOTED_NOTE = '{/* 猫 > 窝 */}';

/**
 Comment in English, which no translator wrote for another.
 */
const ENGLISH_COMMENT = '<!-- check the cat\'s name -->';

/**
 Times a text occurs in another.

 @param text - text searched

 @param needle - text looked for

 @returns Count of non-overlapping occurrences
 */
function occurrencesOf({ text, needle, }: { readonly text: string; readonly needle: string; },): number {
  return text.split(needle,)
    .length - 1;
}

/**
 The user message of the review sheet for a subject, in either responsibility.

 @param subject - what the review is asked about

 @returns Sheet's user message for each responsibility
 */
function userSheetsOf(
  { subject, }: { readonly subject: AbsoluteNaturalnessReviewSubject; },
): readonly string[] {
  return (['defect-discovery', 'acceptance-challenge',] as const).map(function sheetOf(perspective,): string {
    return messageText({
      message: nonNullishOrThrow(buildAbsoluteNaturalnessReviewMessages({
        subject,
        perspective,
      },)[1],),
    },);
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: isAbsoluteNaturalnessReviewWire.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REQUIRES FINDINGS exactly for unacceptable verdict',
          fn: async () => {
            expect(isAbsoluteNaturalnessReviewWire({
              isAcceptable: true,
              findings: [],
              assessment: 'ready',
            },),).toBe(true,);
            expect(isAbsoluteNaturalnessReviewWire({
              isAcceptable: false,
              findings: [{ paragraph: 1, problem: 'Replace stiff syntax.', },],
              assessment: 'translationese remains',
            },),).toBe(true,);
            expect(isAbsoluteNaturalnessReviewWire({
              isAcceptable: true,
              findings: [{ paragraph: 1, problem: 'Optional preference.', },],
              assessment: 'contradictory',
            },),).toBe(false,);
            expect(isAbsoluteNaturalnessReviewWire({
              isAcceptable: false,
              findings: [],
              assessment: 'contradictory',
            },),).toBe(false,);
          },
        },),

        it({
          name: 'ACCEPTS the otherwise-valid reply each refusal case corrupts in one field, so each refusal is '
            + 'that field\'s',
          fn: async () => {
            expect(isAbsoluteNaturalnessReviewWire(VALID_REVIEW,),).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES A NON-BOOLEAN isAcceptable field',
          fn: async () => {
            expect(isAbsoluteNaturalnessReviewWire({ ...VALID_REVIEW, isAcceptable: 'false', },),)
              .toBe(false,);
          },
        },),

        it({
          name: 'REFUSES A REPLY MISSING findings',
          fn: async () => {
            expect(isAbsoluteNaturalnessReviewWire({
              isAcceptable: false,
              assessment: 'retains source phrasing',
            },),).toBe(false,);
          },
        },),

        it({
          name: 'REFUSES A NON-ARRAY findings field',
          fn: async () => {
            expect(isAbsoluteNaturalnessReviewWire({ ...VALID_REVIEW, findings: 'many', },),)
              .toBe(false,);
          },
        },),

        it({
          name: 'REFUSES A FINDING THAT IS NOT A RECORD',
          fn: async () => {
            expect(isAbsoluteNaturalnessReviewWire({
              ...VALID_REVIEW,
              findings: [['paragraph', 1,],],
            },),).toBe(false,);
          },
        },),

        it({
          name: 'REFUSES A FINDING WITH A NON-NUMBER paragraph',
          fn: async () => {
            expect(isAbsoluteNaturalnessReviewWire({
              ...VALID_REVIEW,
              findings: [{ paragraph: '2', problem: 'Avoid literal word-for-word order.', },],
            },),).toBe(false,);
          },
        },),

        it({
          name: 'REFUSES A FINDING WITH A NON-INTEGER paragraph',
          fn: async () => {
            expect(isAbsoluteNaturalnessReviewWire({
              ...VALID_REVIEW,
              findings: [{ paragraph: 1.5, problem: 'Avoid literal word-for-word order.', },],
            },),).toBe(false,);
          },
        },),

        it({
          name: 'REFUSES A FINDING WITH paragraph BELOW ONE',
          fn: async () => {
            expect(isAbsoluteNaturalnessReviewWire({
              ...VALID_REVIEW,
              findings: [{ paragraph: 0, problem: 'Avoid literal word-for-word order.', },],
            },),).toBe(false,);
          },
        },),

        it({
          name: 'REFUSES A FINDING WITH A NON-STRING problem',
          fn: async () => {
            expect(isAbsoluteNaturalnessReviewWire({
              ...VALID_REVIEW,
              findings: [{ paragraph: 2, problem: 7, },],
            },),).toBe(false,);
          },
        },),

        it({
          name: 'REFUSES A FINDING WITH AN EMPTY problem',
          fn: async () => {
            // The one finding-level refusal decided by the finding check's last
            // question, whether the problem shows a reader anything, rather
            // than by a type or range check; it refuses the reply through the
            // same `every` as the other finding-level cases.
            expect(isAbsoluteNaturalnessReviewWire({
              ...VALID_REVIEW,
              findings: [{ paragraph: 2, problem: '', },],
            },),).toBe(false,);
          },
        },),

        it({
          name: 'REFUSES A FINDING WHOSE problem SHOWS A READER NOTHING, as it refuses an empty one: spaces, a '
            + 'zero-width space, a Hangul filler, or both among spaces',
          fn: async () => {
            expect([
              '   ',
              '\u{200B}',
              '\u{3164}',
              ' \u{200B}\u{3164} ',
            ].map(function guarded(problem,) {
              return isAbsoluteNaturalnessReviewWire({
                ...VALID_REVIEW,
                findings: [{ paragraph: 2, problem, },],
              },);
            },),).toEqual([
              false,
              false,
              false,
              false,
            ],);
          },
        },),

        it({
          name: 'REFUSES A REPLY MISSING assessment',
          fn: async () => {
            expect(isAbsoluteNaturalnessReviewWire({
              isAcceptable: false,
              findings: [{ paragraph: 2, problem: 'Avoid literal word-for-word order.', },],
            },),).toBe(false,);
          },
        },),

        it({
          name: 'REFUSES A NON-STRING assessment field',
          fn: async () => {
            expect(isAbsoluteNaturalnessReviewWire({ ...VALID_REVIEW, assessment: 7, },),)
              .toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: buildAbsoluteNaturalnessReviewMessages.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS ABSOLUTE WHOLE-CANDIDATE QUALITY instead of comparative improvement',
          fn: async () => {
            const messages = buildAbsoluteNaturalnessReviewMessages({
              subject: {
                lineStructured: false,
                sourceText: '猫猫在窗台上睡觉。',
                candidateText: 'The cat sleeps on the windowsill.',
                paragraphs: ['The cat sleeps on the windowsill.',],
                identityContext: 'Mimi (@mimi_cat)',
              },
            },);
            /**
             Complete reviewer sheet across system and user messages.
             */
            const sheet = messages.map(function text(message,): string {
              return messageText({ message, },);
            },)
              .join('\n',);
            expect(sheet,).toContain('absolute naturalness floor',);
            expect(sheet,).toContain('not against another candidate',);
            expect(sheet,).toContain('Judge the ENTIRE English candidate',);
            expect(sheet,).toContain('stacked time or aspect adverbs',);
            expect(sheet,).toContain('Perform two independent scans before deciding',);
            expect(sheet,).toContain('return the union of material defects',);
            expect(sheet,).toContain('soft breaks that render as spaces',);
            expect(sheet,).toContain('after replacing each soft break with a space',);
            expect(sheet,).toContain('Mimi (@mimi_cat)',);
          },
        },),

        it({
          name: 'GIVES CONFIRMATION SUBSTANTIVELY DISTINCT CHALLENGE RESPONSIBILITY',
          fn: async () => {
            /** Shared exact candidate subject for both responsibilities. */
            const subject = {
              lineStructured: false,
              sourceText: '猫猫在窗台上睡觉。',
              candidateText: 'The cat sleeps on the windowsill.',
              paragraphs: ['The cat sleeps on the windowsill.',],
            };
            const discovery = buildAbsoluteNaturalnessReviewMessages({
              subject,
              perspective: 'defect-discovery',
            },);
            const challenge = buildAbsoluteNaturalnessReviewMessages({
              subject,
              perspective: 'acceptance-challenge',
            },);
            expect(JSON.stringify(challenge,),).not.toBe(JSON.stringify(discovery,),);
            expect(messageText({ message: nonNullishOrThrow(discovery[0],), },),).toContain(
              'without relying on any prior verdict',
            );
            expect(messageText({ message: nonNullishOrThrow(challenge[0],), },),).toContain(
              'A prior editor accepted this exact candidate',
            );
            expect(messageText({ message: nonNullishOrThrow(challenge[0],), },),).toContain(
              'work backward through each paragraph and sentence',
            );
          },
        },),
      ],
    },),

    describe({
      name: `${buildAbsoluteNaturalnessReviewMessages.name} notes between translators`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CUTS A NOTE ENDING A SENTENCE from the candidate and from its paragraph, in both responsibilities',
          fn: async () => {
            /** Candidate whose first sentence ends in a note. */
            const candidate = `The cat slept. ${SENTENCE_NOTE}\nIt purred.`;
            for (
              const user of userSheetsOf({
                subject: {
                  lineStructured: false,
                  sourceText: '猫睡了。猫呼噜了。',
                  candidateText: candidate,
                  paragraphs: [candidate,],
                },
              },)
            ) {
              expect(user,).not.toContain('前面这句',);
              expect(user,).not.toContain('<!--',);
              // The candidate and the one paragraph, each folded to the page's line.
              expect(occurrencesOf({ text: user, needle: 'The cat slept. It purred.', },),).toBe(2,);
            }
          },
        },),

        it({
          name: 'CUTS A NOTE ALONE ON A LINE, and leaves its paragraph out of the sheet while the others keep their '
            + 'numbers',
          fn: async () => {
            /** Candidate with a note for a paragraph of its own. */
            const candidate = `The cat slept.\n\n${LINE_NOTE}\n\nIt purred.`;
            for (
              const user of userSheetsOf({
                subject: {
                  lineStructured: false,
                  sourceText: '猫睡了。猫呼噜了。',
                  candidateText: candidate,
                  paragraphs: [
                    'The cat slept.',
                    LINE_NOTE,
                    'It purred.',
                  ],
                },
              },)
            ) {
              expect(user,).not.toContain('猫窝在哪里',);
              expect(user,).not.toContain('{/*',);
              expect(user,).toContain('PARAGRAPH 1\n',);
              expect(user,).not.toContain('PARAGRAPH 2\n',);
              expect(user,).toContain('PARAGRAPH 3\n',);
            }
          },
        },),

        it({
          name: 'CUTS A NOTE ON A QUOTED LINE with its line, leaving no bare quote marker line, on a line-structured '
            + 'passage',
          fn: async () => {
            /** Quoted lines with a note between two of them. */
            const candidate = `> The cat slept.\n> ${QUOTED_NOTE}\n> It purred.\n`;
            for (
              const user of userSheetsOf({
                subject: {
                  lineStructured: true,
                  sourceText: '> 猫睡了。\n> 猫呼噜了。\n',
                  candidateText: candidate,
                  paragraphs: [candidate,],
                },
              },)
            ) {
              expect(user,).not.toContain('猫 > 窝',);
              expect(user,).not.toContain('>\n',);
              expect(occurrencesOf({ text: user, needle: '\n> The cat slept.\n> It purred.\n', },),).toBe(2,);
            }
          },
        },),

        it({
          name: 'KEEPS A COMMENT THE ORIGINAL CARRIES and a comment in English, while cutting the note beside them',
          fn: async () => {
            /** Quoted lines with the original's own comment, an English one and a note. */
            const candidate =
              `> The cat slept.\n> ${QUOTED_NOTE}\n> ${ENGLISH_COMMENT}\n> It purred. ${SENTENCE_NOTE}\n`;
            for (
              const user of userSheetsOf({
                subject: {
                  lineStructured: true,
                  sourceText: `> 猫睡了。\n> ${QUOTED_NOTE}\n> 猫呼噜了。\n`,
                  candidateText: candidate,
                  paragraphs: [candidate,],
                },
              },)
            ) {
              // Shown in the candidate and its paragraph, and the original's own line is the third.
              expect(occurrencesOf({ text: user, needle: QUOTED_NOTE, },),).toBe(3,);
              expect(occurrencesOf({ text: user, needle: ENGLISH_COMMENT, },),).toBe(2,);
              expect(user,).not.toContain('前面这句',);
            }
          },
        },),
      ],
    },),
  ],
},);
