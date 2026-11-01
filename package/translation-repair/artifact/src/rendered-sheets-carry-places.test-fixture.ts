/**
 The places of the repeated sentence, and the sentence, that the carry sheets of `rendered-sheets.test-fixture.ts` show (ledger
 X17): the three places as the consistency sheet shows them, the repeated sentence and the chosen place as the carry sheets show it.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import type {
  CarryChosenPlace,
  ConsistencyOccurrenceView,
} from '@monochromatic-dev/translation-repair-consistency';
import { TRANSLATORS_NOTE, } from '@monochromatic-dev/translation-repair-repair/ts/rendered-sheets-texts.test-fixture.ts';

//region Carry places

/**
 The first place of the repeated sentence as the consistency sheet shows it.
 */
export const FIRST_VIEW: ConsistencyOccurrenceView = {
  sliceIndex: 0,
  paragraph: '窗台上的阳光让小猫睡着了。',
  markStart: 0,
  markEnd: 12,
  english: {
    kind: 'present',
    text: `The sunny window put the kitten to sleep. ${TRANSLATORS_NOTE}`,
  },
};

/**
 The second place of the repeated sentence as the consistency sheet shows it.
 */
export const SECOND_VIEW: ConsistencyOccurrenceView = {
  sliceIndex: 2,
  paragraph: '傍晚的时候，窗台上的阳光让小猫睡着了，小鱼干也笑了。',
  markStart: 6,
  markEnd: 18,
  english: {
    kind: 'present',
    text: `In the evening, the sunlight on the sill lulled the kitten to sleep, and the fish snacks smiled too. ${TRANSLATORS_NOTE}`,
  },
};

/**
 The third place of the repeated sentence as the consistency sheet shows it.
 */
export const THIRD_VIEW: ConsistencyOccurrenceView = {
  sliceIndex: 4,
  paragraph: '夜里，窗台上的阳光让小猫睡着了。',
  markStart: 3,
  markEnd: 15,
  english: {
    kind: 'present',
    text: `At night the sunny window put the kitten to sleep. ${TRANSLATORS_NOTE}`,
  },
};

/**
 The repeated sentence.
 */
export const SENTENCE = {
  kind: 'maximal',
  wording: '窗台上的阳光让小猫睡着了',
} as const;

/**
 The chosen place of the repeated sentence as the carry sheets show it: its words and the paragraph that holds them.
 */
export const CHOSEN_PLACE: CarryChosenPlace = {
  view: FIRST_VIEW,
  span: 'The sunny window put the kitten to sleep',
  paragraph: `The sunny window put the kitten to sleep. ${TRANSLATORS_NOTE}`,
} as const;

//endregion Carry places
