/**
 The sheets of the page-level consistency pass's carry for a refrain whose two
 receiving places share one English paragraph, rendered for
 `rendered-sheets.test-fixture.ts` (ledger X17): the writer asked for the
 paragraph and the judge asked to choose among its candidates, each showing the
 chosen place's English slot, the original blocks around the paragraph and the
 words of other places to keep.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  buildCarryChoiceMessages,
  buildCarryWriterMessages,
  type CarryChosenPlace,
  type ConsistencyOccurrenceView,
} from '@monochromatic-dev/translation-repair-consistency';

import {
  IDENTITY,
  joined,
  REFERENCES,
  type RenderedSheet,
  TRANSLATORS_NOTE,
} from '@monochromatic-dev/translation-repair-repair/ts/rendered-sheets-texts.test-fixture.ts';

//region Refrain carry sheets

/**
 The refrain whose frame the sheets show.
 */
const REFRAIN = {
  kind: 'refrain',
  prefix: '所以这只猫',
  suffix: '的小宝贝',
} as const;

/**
 The English paragraph that holds both receiving places.
 */
const SHARED_ENGLISH = 'So this cat is a gentle little treasure, and so this cat is a dependable little treasure.';

/**
 The first receiving place of the refrain as the consistency sheet shows it.
 */
const FIRST_RECEIVING_VIEW: ConsistencyOccurrenceView = {
  sliceIndex: 4,
  paragraph: '所以这只猫是个温柔的小宝贝，所以这只猫是个可靠的小宝贝。',
  markStart: 0,
  markEnd: 12,
  slotSource: '温柔',
  english: {
    kind: 'present',
    text: SHARED_ENGLISH,
  },
};

/**
 The second receiving place of the refrain as the consistency sheet shows it.
 */
const SECOND_RECEIVING_VIEW: ConsistencyOccurrenceView = {
  sliceIndex: 4,
  paragraph: '所以这只猫是个温柔的小宝贝，所以这只猫是个可靠的小宝贝。',
  markStart: 15,
  markEnd: 27,
  slotSource: '可靠',
  english: {
    kind: 'present',
    text: SHARED_ENGLISH,
  },
};

/**
 The chosen place of the refrain, whose English slot was located.
 */
const CHOSEN_PLACE: CarryChosenPlace = {
  view: {
    sliceIndex: 0,
    paragraph: '所以这只猫是个懒洋洋的小宝贝。',
    markStart: 0,
    markEnd: 12,
    slotSource: '懒洋洋',
    english: {
      kind: 'present',
      text: `So this cat must be a lazy little treasure. ${TRANSLATORS_NOTE}`,
    },
  },
  span: 'So this cat must be a lazy little treasure',
  paragraph: `So this cat must be a lazy little treasure. ${TRANSLATORS_NOTE}`,
  slot: 'lazy',
};

/**
 The sheets of a refrain carried to two places of one paragraph.

 @returns One entry for the writer's sheet and one for the choice sheet

 @example
 ```ts
 const sheets = refrainCarrySheets();
 ```
 */
export function refrainCarrySheets(): readonly RenderedSheet[] {
  return [
    {
      name: 'page consistency carry writer, refrain, two places in one paragraph, words to keep',
      text: joined({
        messages: buildCarryWriterMessages({
          subject: {
            repeat: REFRAIN,
            chosen: CHOSEN_PLACE,
            receiving: [
              {
                view: FIRST_RECEIVING_VIEW,
                span: 'So this cat is a gentle little treasure',
              },
              {
                view: SECOND_RECEIVING_VIEW,
                span: 'so this cat is a dependable little treasure',
              },
            ],
            paragraph: SHARED_ENGLISH,
            keeping: ['and so this cat',],
            before: '傍晚的阳光很暖。',
            after: '小鱼干也笑了。',
            lineStructured: false,
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
          },
        },),
      },),
    },
    {
      name: 'page consistency carry choice, refrain, two places in one paragraph, words to keep',
      text: joined({
        messages: buildCarryChoiceMessages({
          subject: {
            repeat: REFRAIN,
            receiving: [
              {
                view: FIRST_RECEIVING_VIEW,
                span: 'So this cat is a gentle little treasure',
              },
              {
                view: SECOND_RECEIVING_VIEW,
                span: 'so this cat is a dependable little treasure',
              },
            ],
            chosen: CHOSEN_PLACE,
            keeping: ['and so this cat',],
            before: '傍晚的阳光很暖。',
            after: '小鱼干也笑了。',
            candidates: [
              {
                label: 'CANDIDATE 1',
                current: true,
                text: SHARED_ENGLISH,
              },
              {
                label: 'CANDIDATE 2',
                current: false,
                text: `So this cat must be a lazy little treasure, and so this cat must be a lazy little treasure. ${TRANSLATORS_NOTE}`,
              },
            ],
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
          },
        },),
      },),
    },
  ];
}

//endregion Refrain carry sheets
