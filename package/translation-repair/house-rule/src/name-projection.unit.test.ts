/**
 Tests the name projection the declared-name guard and the link-name floor
 share (ledger B23): the key and the offsets each unit keeps into the text as
 composed, and a key carried only where each end meets a word edge, a small
 letter running on being no edge and a case change or a digit inside a
 handle being one. Cat-themed invention throughout.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  carriesName,
  nameOccurrences,
  nameProjection,
  projectName,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: projectName.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS LETTERS AND DIGITS, lowercased, through escapes, spaces and punctuation',
          fn: async () => {
            expect(projectName({ text: String.raw`Mittens\_the\_Cat, 2024!`, },).key,).toBe('mittensthecat2024',);
            expect(nameProjection({ text: 'Mittens the Cat', },),).toBe('mittensthecat',);
          },
        },),
        it({
          name: 'INDEXES THE TEXT AS COMPOSED, so a decomposed accent is one character and its offsets name it',
          fn: async () => {
            /**
             Projection of a name whose accent is written as a combining mark.
             */
            const projection = projectName({ text: 'A Mitte\u{0308}ns', },);
            expect(projection.composed,).toBe('A Mitt\u{00EB}ns',);
            expect(projection.key,).toBe('amitt\u{00EB}ns',);
            expect(projection.starts,).toEqual([0, 2, 3, 4, 5, 6, 7, 8,],);
            expect(projection.ends,).toEqual([1, 3, 4, 5, 6, 7, 8, 9,],);
          },
        },),
        it({
          name: 'KEEPS A LETTER BEYOND THE FIRST PLANE WHOLE, both of its units naming the one character',
          fn: async () => {
            /**
             Projection of a script capital followed by a Latin letter.
             */
            const projection = projectName({ text: '\u{1D4DC}a', },);
            expect(projection.key,).toBe('\u{1D4DC}a',);
            expect(projection.starts,).toEqual([0, 0, 2,],);
            expect(projection.ends,).toEqual([2, 2, 3,],);
          },
        },),
      ],
    },),

    describe({
      name: carriesName.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FINDS A KEY STANDING APART, whatever spacing or punctuation the projection dropped inside it',
          fn: async () => {
            for (const [text, form,] of [
              ['Ann naps.', 'Ann',],
              ['(Ann)', 'Ann',],
              ['Mittens Cat naps.', 'MittensCat',],
              [String.raw`Mittens\_Cat naps.`, 'Mittens Cat',],
            ] as const) {
              expect(carriesName({
                projection: projectName({ text, },),
                key: nameProjection({ text: form, },),
              },),).toBe(true,);
            }
          },
        },),
        it({
          name: 'FINDS NO KEY THAT ONLY RUNS ON INSIDE A WORD, in either case',
          fn: async () => {
            for (const text of [
              'The cat cannot nap.',
              'THE CAT CANNOT NAP.',
              'Annabel naps.',
            ]) {
              expect(carriesName({
                projection: projectName({ text, },),
                key: 'ann',
              },),).toBe(false,);
            }
            expect(carriesName({
              projection: projectName({ text: 'The cherryblossom naps.', },),
              key: 'blossom',
            },),).toBe(false,);
          },
        },),
        it({
          name: 'READS A SMALL LETTER BEFORE A CAPITAL, OR A LETTER BESIDE A DIGIT, AS AN EDGE, which is how a handle '
            + 'joins a name',
          fn: async () => {
            for (const text of [
              'MittensBlossom naps.',
              'Blossom2024 naps.',
              '2024Blossom naps.',
            ]) {
              expect(carriesName({
                projection: projectName({ text, },),
                key: 'blossom',
              },),).toBe(true,);
            }
          },
        },),
        it({
          name: 'READS A COMBINING MARK AFTER THE KEY AS THE WORD RUNNING ON',
          fn: async () => {
            expect(carriesName({
              projection: projectName({ text: 'Ann\u{0301} naps.', },),
              key: 'ann',
            },),).toBe(false,);
          },
        },),
        it({
          name: 'NEEDS NO EDGE BESIDE A HAN KEY, since Chinese writes no spaces',
          fn: async () => {
            expect(carriesName({
              projection: projectName({ text: 'Mittens\u{732B}\u{5C0F}\u{59D0}naps', },),
              key: '\u{732B}\u{5C0F}\u{59D0}',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'TRIES EVERY OCCURRENCE, so a key running on at first and standing apart later is carried',
          fn: async () => {
            expect(carriesName({
              projection: projectName({ text: 'The cat cannot nap, said Ann.', },),
              key: 'ann',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'CARRIES NO EMPTY KEY rather than one at every offset',
          fn: async () => {
            expect(carriesName({
              projection: projectName({ text: 'Ann naps.', },),
              key: '',
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: nameOccurrences.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LISTS EVERY OCCURRENCE STANDING APART with where it starts and ends in the composed text',
          fn: async () => {
            expect(nameOccurrences({
              projection: projectName({ text: 'Ann naps. Cannot Ann wake? Ann.', },),
              key: 'ann',
            },),).toEqual([
              {
                start: 0,
                end: 3,
              },
              {
                start: 17,
                end: 20,
              },
              {
                start: 27,
                end: 30,
              },
            ],);
          },
        },),
        it({
          name: 'LISTS NOTHING FOR AN EMPTY KEY or for a key that only runs on inside a word',
          fn: async () => {
            expect(nameOccurrences({
              projection: projectName({ text: 'Ann naps.', },),
              key: '',
            },),).toEqual([],);
            expect(nameOccurrences({
              projection: projectName({ text: 'The cat cannot nap.', },),
              key: 'ann',
            },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
