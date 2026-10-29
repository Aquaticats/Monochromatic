/**
 Tests for how the present-tense rules read a sentence: its words with their
 offsets, the whitespace between them, and whether a declared name or an -s
 word sits where a rule looks.

 Every case asserts the whole value read. Fixtures are cat-themed invention;
 no corpus text appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isBlankSpan,
  isSEndingWord,
  isWhitespace,
  namesAt,
  wordAfterSpace,
  wordsOf,
} from '../dist/final/node/index.mjs';

/**
 A no-break space, written as its code point so no invisible character sits in the source.
 */
const NO_BREAK_SPACE = String.fromCodePoint(0xA0,);

/**
 A cat name in a script with no letter case.
 */
const CASELESS_CAT_NAME = '猫猫';

/**
 A cat name in a cased script whose letters sit beyond the first plane (Deseret), built from code points so the
 source holds no such character.
 */
const ASTRAL_CAT_NAME = String.fromCodePoint(
  0x1_04_28,
  0x1_04_29,
  0x1_04_2A,
);

/**
 A lone first half of a surrogate pair, which is no character.
 */
const LONE_HIGH_SURROGATE = String.fromCodePoint(0xD8_00,);

//region Present-tense words tests

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: wordsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS words with their offsets, keeping a hyphen and an apostrophe inside a word, and folding the typographic apostrophe',
          fn: async () => {
            expect(wordsOf({ sentence: 'She’s a well-known cat, 3 times.', },),).toEqual([
              {
                text: 'She’s',
                folded: 'she\'s',
                start: 0,
                end: 5,
              },
              {
                text: 'a',
                folded: 'a',
                start: 6,
                end: 7,
              },
              {
                text: 'well-known',
                folded: 'well-known',
                start: 8,
                end: 18,
              },
              {
                text: 'cat',
                folded: 'cat',
                start: 19,
                end: 22,
              },
              {
                text: 'times',
                folded: 'times',
                start: 26,
                end: 31,
              },
            ],);
          },
        },),

        it({
          name: 'READS no word from an empty sentence or one with only punctuation and digits',
          fn: async () => {
            expect(wordsOf({ sentence: '', },),).toEqual([],);
            expect(wordsOf({ sentence: '... 42 !?', },),).toEqual([],);
          },
        },),

        it({
          name: 'READS a word in a script without letter case and one beyond the first plane as one word each, '
            + 'with offsets in UTF-16 units, and no word from a lone surrogate half',
          fn: async () => {
            expect(wordsOf({
              sentence: `${CASELESS_CAT_NAME} ${ASTRAL_CAT_NAME} ${LONE_HIGH_SURROGATE} naps`,
            },),).toEqual([
              {
                text: CASELESS_CAT_NAME,
                folded: CASELESS_CAT_NAME,
                start: 0,
                end: 2,
              },
              {
                text: ASTRAL_CAT_NAME,
                folded: ASTRAL_CAT_NAME,
                start: 3,
                end: 9,
              },
              {
                text: 'naps',
                folded: 'naps',
                start: 12,
                end: 16,
              },
            ],);
          },
        },),
      ],
    },),

    describe({
      name: isWhitespace.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TAKES a space, a tab, a line break and a no-break space for whitespace, and a letter or the end of the text for none',
          fn: async () => {
            expect([
              ' ',
              '\t',
              '\n',
              NO_BREAK_SPACE,
              'a',
              '',
            ].map(function asked(character,): boolean {
              return isWhitespace({ character, },);
            },),).toEqual([
              true,
              true,
              true,
              true,
              false,
              false,
            ],);
          },
        },),
      ],
    },),

    describe({
      name: isBlankSpan.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SEES a run of spaces as blank and a run holding a letter as not',
          fn: async () => {
            expect(isBlankSpan({
              sentence: 'did  she',
              start: 3,
              end: 5,
            },),).toBe(true,);
            expect(isBlankSpan({
              sentence: 'did a she',
              start: 3,
              end: 6,
            },),).toBe(false,);
          },
        },),

        it({
          name: 'REFUSES an empty range, which the word reader never produces',
          fn: async () => {
            const refusal = caught(function emptyRange() {
              return isBlankSpan({
                sentence: 'did she',
                start: 3,
                end: 3,
              },);
            },);
            expect(refusal,).toBeInstanceOf(Error,);
            expect(String(refusal,),).toBe(
              'Error: unreachable: the two words either side of a span are separated by at least one non-word '
                + 'character, so the span is never empty',
            );
          },
        },),
      ],
    },),

    describe({
      name: wordAfterSpace.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FINDS the word past one run of whitespace, and none where no whitespace follows or no word starts after it',
          fn: async () => {
            /**
             Sentence whose words sit apart by a double space and a tilde.
             */
            const sentence = 'Mittens  naps ~ here';
            /**
             Words of that sentence by start offset.
             */
            const wordsByStart = new Map(wordsOf({ sentence, },).map(function entryOf(word,) {
              return [
                word.start,
                word,
              ] as const;
            },),);
            expect(wordAfterSpace({
              sentence,
              wordsByStart,
              end: 7,
            },),).toEqual([
              {
                text: 'naps',
                folded: 'naps',
                start: 9,
                end: 13,
              },
            ],);
            expect(wordAfterSpace({
              sentence,
              wordsByStart,
              end: 9,
            },),).toEqual([],);
            expect(wordAfterSpace({
              sentence,
              wordsByStart,
              end: 13,
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: namesAt.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'MATCHES a declared name folded, and refuses a longer word that begins with it and an empty name',
          fn: async () => {
            expect(namesAt({
              sentence: 'Mittens naps',
              start: 0,
              name: 'mittens',
            },),).toBe(true,);
            expect(namesAt({
              sentence: 'Mittenss naps',
              start: 0,
              name: 'Mittens',
            },),).toBe(false,);
            expect(namesAt({
              sentence: 'Mittens naps',
              start: 0,
              name: '',
            },),).toBe(false,);
            expect(namesAt({
              sentence: 'Tabby naps',
              start: 0,
              name: 'Mittens',
            },),).toBe(false,);
          },
        },),

        it({
          name: 'MATCHES a declared name in a script without letter case and one beyond the first plane, as it matches a Latin one',
          fn: async () => {
            expect(namesAt({
              sentence: `${CASELESS_CAT_NAME} naps`,
              start: 0,
              name: CASELESS_CAT_NAME,
            },),).toBe(true,);
            expect(namesAt({
              sentence: `${ASTRAL_CAT_NAME} naps`,
              start: 0,
              name: ASTRAL_CAT_NAME,
            },),).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES a name followed by a letter in a script without letter case or beyond the first plane, '
            + 'as it refuses one followed by a Latin letter',
          fn: async () => {
            expect(namesAt({
              sentence: `Mittens${CASELESS_CAT_NAME} naps`,
              start: 0,
              name: 'Mittens',
            },),).toBe(false,);
            expect(namesAt({
              sentence: `Mittens${ASTRAL_CAT_NAME} naps`,
              start: 0,
              name: 'Mittens',
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: isSEndingWord.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TAKES a letters-only word of two letters or more ending in s for an -s ending, and a single s, an apostrophe word or a word ending otherwise for none',
          fn: async () => {
            expect([
              'naps',
              'is',
              's',
              'it\'s',
              'nap',
              'cat1s',
            ].map(function asked(word,): boolean {
              return isSEndingWord({ word, },);
            },),).toEqual([
              true,
              true,
              false,
              false,
              false,
              false,
            ],);
          },
        },),

        it({
          name: 'TAKES the letters of a script without letter case and of one beyond the first plane for letters, and a digit for none',
          fn: async () => {
            expect([
              `${CASELESS_CAT_NAME}s`,
              `${ASTRAL_CAT_NAME}s`,
              `${CASELESS_CAT_NAME}1s`,
              `${ASTRAL_CAT_NAME}${LONE_HIGH_SURROGATE}s`,
            ].map(function asked(word,): boolean {
              return isSEndingWord({ word, },);
            },),).toEqual([
              true,
              true,
              false,
              false,
            ],);
          },
        },),
      ],
    },),
  ],
},);

//endregion Present-tense words tests
