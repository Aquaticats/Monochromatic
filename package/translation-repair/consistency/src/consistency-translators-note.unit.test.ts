/**
 Guards the page consistency judges against a note between translators (ledger
 B698, B702): a judge asked which rendering of a repeated wording the page
 should use, a judge settling a split among the finalists, and a judge choosing
 among the texts a carry might leave a paragraph with all read English, and a
 reader never sees an HTML comment, so a note in it may be neither the
 difference between two renderings nor a word of the one chosen. The original's
 own comments stay in view. The judges' quotes are located in the stored English
 by the code that counts them, so only what is shown is cut.

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
  buildCarryChoiceMessages,
  buildConsistencyMessages,
  buildConsistencyRunOffMessages,
  type CarryChoiceSubject,
  type ConsistencyOccurrenceView,
  type ConsistencySubject,
} from '../dist/final/node/index.mjs';

/**
 Note the translators left each other beside the sentence it is about.
 */
const NOTE = '<!-- 前面这句猫话懂什么意思，先这么翻译，大家看一下 -->';

/**
 Words of the note, as a sheet that shows it prints them.
 */
const NOTE_WORDS = '大家看一下';

/**
 English of the first place's slice, ending with the note.
 */
const FIRST_ENGLISH = `The sunny window put the kitten to sleep. So this cat must be a chubby little treasure.\n\n${NOTE}`;

/**
 English of the second place's slice, ending with the note.
 */
const SECOND_ENGLISH = `In the evening, the sunlight on the sill lulled the kitten to sleep.\n\n${NOTE}`;

/**
 Two places of a repeated sentence, in two slices.
 */
const PLACES: readonly ConsistencyOccurrenceView[] = [
  {
    sliceIndex: 0,
    paragraph: '窗台上的阳光让小猫睡着了。所以这只猫一定是胖胖的小宝贝。',
    markStart: 0,
    markEnd: 12,
    english: { kind: 'present', text: FIRST_ENGLISH, },
  },
  {
    sliceIndex: 2,
    paragraph: '傍晚的时候，窗台上的阳光让小猫睡着了，小鱼干也笑了。',
    markStart: 6,
    markEnd: 18,
    english: { kind: 'present', text: SECOND_ENGLISH, },
  },
];

/**
 What the judge sheets show for a repeated sentence.
 */
const SUBJECT: ConsistencySubject = {
  repeat: { kind: 'maximal', wording: '窗台上的阳光让小猫睡着了', },
  occurrences: PLACES,
};

/**
 What the choice sheet shows, with the note in the chosen place's paragraph and in two of the three candidates.
 */
const CHOICE_SUBJECT: CarryChoiceSubject = {
  repeat: SUBJECT.repeat,
  receiving: [{
    view: PLACES[1] as ConsistencyOccurrenceView,
    span: 'the sunlight on the sill lulled the kitten to sleep',
  },],
  chosen: {
    view: PLACES[0] as ConsistencyOccurrenceView,
    span: 'The sunny window put the kitten to sleep',
    paragraph: FIRST_ENGLISH,
  },
  keeping: [],
  candidates: [
    { label: 'CANDIDATE 1', current: true, text: SECOND_ENGLISH, },
    { label: 'CANDIDATE 2', current: false, text: `The sunny window put the kitten to sleep.\n\n${NOTE}`, },
    { label: 'CANDIDATE 3', current: false, text: 'The sunny window put the kitten to sleep.', },
  ],
};

await describe({
  name: 'a note between translators stays out of the consistency judges',
  children: [
    describe({
      name: buildConsistencyMessages.name,
      children: [
        it({
          name: 'SHOWS NO NOTE in the English of any place, and keeps the English words',
          fn: async () => {
            /**
             What a judge of the repeated wording reads.
             */
            const sheet = userText({ messages: buildConsistencyMessages({ subject: SUBJECT, },), },);
            expect(sheet,).not.toContain(NOTE_WORDS,);
            expect(sheet,).toContain('chubby little treasure',);
            expect(sheet,).toContain('lulled the kitten to sleep',);
          },
        },),
      ],
    },),
    describe({
      name: buildConsistencyRunOffMessages.name,
      children: [
        it({
          name: 'SHOWS NO NOTE in the English of the finalists, and keeps the English words',
          fn: async () => {
            /**
             What a judge settling the split reads.
             */
            const sheet = userText({
              messages: buildConsistencyRunOffMessages({
                subject: SUBJECT,
                finalists: ['OCCURRENCE 1', 'OCCURRENCE 2',],
                renderings: [],
              },),
            },);
            expect(sheet,).not.toContain(NOTE_WORDS,);
            expect(sheet,).toContain('chubby little treasure',);
            expect(sheet,).toContain('lulled the kitten to sleep',);
          },
        },),
      ],
    },),
    describe({
      name: buildCarryChoiceMessages.name,
      children: [
        it({
          name: 'SHOWS NO NOTE in the chosen place\'s paragraph or in any candidate, in either round',
          fn: async () => {
            for (const runOff of [false, true,]) {
              /**
               What a judge choosing among the candidates reads.
               */
              const sheet = userText({ messages: buildCarryChoiceMessages({ subject: CHOICE_SUBJECT, runOff, },), },);
              expect(sheet,).not.toContain(NOTE_WORDS,);
              expect(sheet,).toContain('chubby little treasure',);
              expect(sheet,).toContain('CANDIDATE 3',);
            }
          },
        },),
      ],
    },),
  ],
},);
