/**
 Tests for the English the consistency judges read: each place's English
 without the notes the archive's translators left each other, with the
 comments the slice's original holds kept wherever the English repeats them.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  buildConsistencyMessages,
  type ConsistencyOccurrenceView,
  englishWithoutNotes,
  originalOfViews,
  viewsWithoutNotes,
} from '../dist/final/node/index.mjs';

/**
 Note the translators left each other beside the sentence it is about.
 */
const NOTE = '<!-- 前面这句猫话懂什么意思，先这么翻译，大家看一下 -->';

/**
 Comment the original itself carries, in a paragraph the sheet does not show.
 */
const ORIGINAL_COMMENT = '<!-- 猫窝：Cat Bed -->';

/**
 A place whose slice's original holds a comment in another paragraph, repeated in its English beside a note.
 */
const VIEW: ConsistencyOccurrenceView = {
  sliceIndex: 0,
  paragraph: '窗台上的阳光让小猫睡着了。',
  markStart: 0,
  markEnd: 12,
  sliceSource: `猫窝在这里。${ORIGINAL_COMMENT}\n\n窗台上的阳光让小猫睡着了。`,
  english: {
    kind: 'present',
    text: `The cat bed is here. ${ORIGINAL_COMMENT}\n\nThe sunny window put the kitten to sleep.\n\n${NOTE}`,
  },
};

await describe({
  name: 'the English the consistency judges read',
  children: [
    describe({
      name: originalOfViews.name,
      children: [
        it({
          name: 'JOINS THE SLICE ORIGINAL WHERE A PLACE HAS ONE and the shown paragraph where it has none',
          fn: async () => {
            expect(originalOfViews({
              views: [
                VIEW,
                {
                  sliceIndex: 1,
                  paragraph: '小猫醒了。',
                  markStart: 0,
                  markEnd: 5,
                  english: { kind: 'absent', },
                },
              ],
            },),).toBe(`${VIEW.sliceSource ?? ''}\n小猫醒了。`,);
          },
        },),
      ],
    },),
    describe({
      name: englishWithoutNotes.name,
      children: [
        it({
          name: 'LEAVES ENGLISH THAT IS ABSENT AS IT IS',
          fn: async () => {
            expect(englishWithoutNotes({ english: { kind: 'absent', }, original: '猫', },),).toEqual({ kind: 'absent', },);
          },
        },),
        it({
          name: 'CUTS THE NOTE and keeps a comment the original holds',
          fn: async () => {
            expect(englishWithoutNotes({ english: VIEW.english, original: VIEW.sliceSource ?? '', },),).toEqual({
              kind: 'present',
              // The cut takes the note and leaves the blank line it stood after.
              text: `The cat bed is here. ${ORIGINAL_COMMENT}\n\nThe sunny window put the kitten to sleep.\n\n`,
            },);
          },
        },),
      ],
    },),
    describe({
      name: viewsWithoutNotes.name,
      children: [
        it({
          name: 'CUTS THE ENGLISH OF EVERY PLACE and changes nothing else about it',
          fn: async () => {
            /**
             The places as the judges read them.
             */
            const [shown,] = viewsWithoutNotes({ views: [VIEW,], },);
            expect(shown?.paragraph,).toBe(VIEW.paragraph,);
            expect(shown?.sliceSource,).toBe(VIEW.sliceSource,);
            expect(shown?.english.kind === 'present' ? shown.english.text.includes('大家看一下',) : true,).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: buildConsistencyMessages.name,
      children: [
        it({
          name: 'KEEPS A COMMENT THE SLICE ORIGINAL HOLDS even when the shown paragraph does not',
          fn: async () => {
            /**
             What the judge reads.
             */
            const sheet = buildConsistencyMessages({
              subject: {
                repeat: { kind: 'maximal', wording: '窗台上的阳光让小猫睡着了', },
                occurrences: [VIEW,],
              },
            },)
              .map(function contentOf(message,): string {
                return message.content;
              },)
              .join('\n',);
            expect(sheet,).toContain(ORIGINAL_COMMENT,);
            expect(sheet,).not.toContain('大家看一下',);
          },
        },),
      ],
    },),
  ],
},);
