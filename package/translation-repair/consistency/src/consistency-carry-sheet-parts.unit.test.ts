/**
 Tests for the parts the writer's sheet and the choice sheet both show: how a
 receiving place is labelled when a block holds one and when it holds several,
 the blocks of a place's ORIGINAL, the blocks around the one that holds the
 places, the chosen place's English and the words to keep.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  aroundBlocksOf,
  type CarryChosenPlace,
  chosenEnglishBlocksOf,
  type ConsistencyOccurrenceView,
  keepingBlockOf,
  originalBlocksOf,
  receivingLabel,
  replaceBlocksOf,
} from '../dist/final/node/index.mjs';

/**
 The fence the cases shown blocks with.
 */
const FENCE = '=====';

/**
 The marks the cases mark the repeated wording with.
 */
const MARKS = { open: '⟦', close: '⟧', } as const;

/**
 A place as the consistency sheet shows it, with the repeated wording at the start.
 */
const VIEW: ConsistencyOccurrenceView = {
  sliceIndex: 3,
  paragraph: '小猫打盹。',
  markStart: 0,
  markEnd: 4,
  english: { kind: 'present', text: 'The kitten naps.', },
};

/**
 The chosen place of the cases.
 */
const CHOSEN: CarryChosenPlace = {
  view: VIEW,
  span: 'The cat naps',
  paragraph: 'The cat naps. It is warm.',
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: receivingLabel.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE ONLY RECEIVING PLACE BARE AND NUMBERS SEVERAL FROM ONE, in the order the block holds them',
          fn: async () => {
            expect([
              receivingLabel({ name: 'THIS PLACE', position: 0, count: 1, },),
              receivingLabel({ name: 'THIS PLACE', position: 0, count: 2, },),
              receivingLabel({ name: 'THIS PLACE', position: 1, count: 2, },),
              receivingLabel({ name: 'THE WORDS TO REPLACE', position: 2, count: 3, },),
            ],).toEqual([
              'THIS PLACE',
              'THIS PLACE 1',
              'THIS PLACE 2',
              'THE WORDS TO REPLACE 3',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: originalBlocksOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHOWS THE ORIGINAL WITH THE REPEATED WORDING MARKED, and the slot in the original too where the place has '
            + 'one, each block ending in a line break',
          fn: async () => {
            expect([
              originalBlocksOf({ view: VIEW, name: 'THIS PLACE', fence: FENCE, marks: MARKS, },),
              originalBlocksOf({
                view: { ...VIEW, slotSource: '温柔', },
                name: 'THIS PLACE 2',
                fence: FENCE,
                marks: MARKS,
              },),
            ],).toEqual([
              '===== THIS PLACE: ORIGINAL =====\n⟦小猫打盹⟧。\n',
              '===== THIS PLACE 2: ORIGINAL =====\n⟦小猫打盹⟧。\n===== THIS PLACE 2: SLOT IN THE ORIGINAL =====\n温柔\n',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: replaceBlocksOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHOWS THE WORDS EACH RECEIVING PLACE REPLACES, bare for the only place and numbered from one for several, '
            + 'each block ending in a line break, and nothing when no place receives',
          fn: async () => {
            expect([
              replaceBlocksOf({ receiving: [{ view: VIEW, span: 'The kitten naps', },], fence: FENCE, },),
              replaceBlocksOf({
                receiving: [{ view: VIEW, span: 'The kitten naps', }, { view: VIEW, span: 'the kitten purrs', },],
                fence: FENCE,
              },),
              replaceBlocksOf({ receiving: [], fence: FENCE, },),
            ],).toEqual([
              '===== THE WORDS TO REPLACE =====\nThe kitten naps\n',
              '===== THE WORDS TO REPLACE 1 =====\nThe kitten naps\n===== THE WORDS TO REPLACE 2 =====\nthe kitten purrs\n',
              '',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: aroundBlocksOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHOWS THE BLOCK BEFORE AND THE BLOCK AFTER IN THAT ORDER, and nothing for a block the slice lacks',
          fn: async () => {
            expect([
              aroundBlocksOf({ before: '前一段', after: '后一段', fence: FENCE, },),
              aroundBlocksOf({ after: '后一段', fence: FENCE, },),
              aroundBlocksOf({ before: '前一段', fence: FENCE, },),
              aroundBlocksOf({ fence: FENCE, },),
            ],).toEqual([
              '===== ORIGINAL BEFORE THIS PLACE =====\n前一段\n===== ORIGINAL AFTER THIS PLACE =====\n后一段\n',
              '===== ORIGINAL AFTER THIS PLACE =====\n后一段\n',
              '===== ORIGINAL BEFORE THIS PLACE =====\n前一段\n',
              '',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: chosenEnglishBlocksOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHOWS THE CHOSEN RENDERING AND THE CHOSEN PLACE\'S ENGLISH PARAGRAPH, and its English slot only when it has one '
            + 'located',
          fn: async () => {
            expect([
              chosenEnglishBlocksOf({ chosen: CHOSEN, fence: FENCE, },),
              chosenEnglishBlocksOf({ chosen: { ...CHOSEN, slot: 'warm', }, fence: FENCE, },),
            ],).toEqual([
              '===== CHOSEN RENDERING =====\nThe cat naps\n===== CHOSEN PLACE: ENGLISH PARAGRAPH =====\nThe cat naps. It is warm.\n',
              '===== CHOSEN RENDERING =====\nThe cat naps\n===== CHOSEN PLACE: ENGLISH PARAGRAPH =====\nThe cat naps. It is warm.\n'
                + '===== CHOSEN PLACE: ENGLISH SLOT =====\nwarm\n',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: keepingBlockOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LISTS THE WORDS OF OTHER PLACES TO KEEP, one to a line, and shows nothing when there are none',
          fn: async () => {
            expect([
              keepingBlockOf({ keeping: ['The cat naps', 'The cat dozes',], fence: FENCE, },),
              keepingBlockOf({ keeping: [], fence: FENCE, },),
            ],).toEqual([
              '===== WORDS OF OTHER PLACES TO KEEP EXACTLY =====\nThe cat naps\nThe cat dozes\n',
              '',
            ],);
          },
        },),
        it({
          name: 'LISTS A STRETCH OF A PLACE THAT RUNS OVER AN EDGE AFTER THE WORDS TO KEEP, saying in plain words that it '
            + 'stays the first or the last words of the paragraph and why, with no section of its own',
          fn: async () => {
            expect([
              keepingBlockOf({
                keeping: ['The cat naps',],
                crossing: [{ edge: 'start', words: 'to sleep', },],
                fence: FENCE,
              },),
              keepingBlockOf({ keeping: [], crossing: [{ edge: 'end', words: 'put the kitten', },], fence: FENCE, },),
            ],).toEqual([
              '===== WORDS OF OTHER PLACES TO KEEP EXACTLY =====\nThe cat naps\nto sleep\n(these are the FIRST words of the '
                + 'paragraph: the place they belong to runs on from the previous paragraph, so they stay the first words of '
                + 'your paragraph, exactly as they are)\n',
              '===== WORDS OF OTHER PLACES TO KEEP EXACTLY =====\nput the kitten\n(these are the LAST words of the '
                + 'paragraph: the place they belong to runs on into the next paragraph, so they stay the last words of '
                + 'your paragraph, exactly as they are)\n',
            ],);
          },
        },),
      ],
    },),
  ],
},);
