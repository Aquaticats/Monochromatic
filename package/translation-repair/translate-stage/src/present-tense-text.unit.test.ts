/**
 Tests for how a paragraph is read for the present-tense rules: the words a
 page shows, the speech blanked, and the sentence offsets.

 Every case asserts the whole value read. Fixtures are cat-themed invention;
 no corpus text appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  maskQuotedSpeech,
  sentenceSpans,
  visibleWords,
} from '../dist/final/node/index.mjs';

//region Present-tense text tests

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: visibleWords.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS a paragraph as one line of words: marks dropped, a link reduced to its label, a bare address dropped, whitespace collapsed',
          fn: async () => {
            expect(visibleWords({
              text: '  [Mittens](https://cats.example/m)   *naps*\n see https://cats.example now `ok`  ',
            },),).toBe('Mittens naps see now ok',);
          },
        },),

        it({
          name: 'KEEPS a bracket that opens no complete link, whether its label never closes or its target never opens or closes',
          fn: async () => {
            expect([
              '[Mittens naps',
              '[Mittens] naps',
              '[Mittens](https://cats.example/m naps',
            ].map(function read(text,): string {
              return visibleWords({ text, },);
            },),).toEqual([
              '[Mittens naps',
              '[Mittens] naps',
              '[Mittens](https://cats.example/m naps',
            ],);
          },
        },),

        it({
          name: 'READS nothing from an empty paragraph',
          fn: async () => {
            expect(visibleWords({ text: '', },),).toBe('',);
          },
        },),
      ],
    },),

    describe({
      name: maskQuotedSpeech.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'BLANKS speech between straight quotes, curly quotes and guillemets, quotation marks included, and keeps the text outside, at the same length',
          fn: async () => {
            /**
             Text with one quotation of each kind.
             */
            const text = 'The vet said, "She naps." Then “Hi” and «yo» ended.';
            expect(maskQuotedSpeech({ text, },),).toBe(
              `The vet said, ${' '.repeat(11,)} Then ${' '.repeat(4,)} and ${' '.repeat(4,)} ended.`,
            );
            expect(maskQuotedSpeech({ text, },).length,).toBe(text.length,);
          },
        },),

        it({
          name: 'BLANKS an opening quotation that never closes through the end of the text',
          fn: async () => {
            expect(maskQuotedSpeech({ text: 'She said, “I nap', },),).toBe(`She said, ${' '.repeat(6,)}`,);
          },
        },),

        it({
          name: 'BLANKS speech between single curly marks, marks included, and keeps the text outside, at the same length (n166, Z10)',
          fn: async () => {
            /**
             Text with one quotation in single curly marks.
             */
            const text = 'The vet said, ‘She naps.’ Then she left.';
            expect(maskQuotedSpeech({ text, },),).toBe(`The vet said, ${' '.repeat('‘She naps.’'.length,)} Then she left.`,);
            expect(maskQuotedSpeech({ text, },).length,).toBe(text.length,);
          },
        },),

        it({
          name: 'BLANKS the whole quotation in single curly marks that holds an apostrophe inside a word, and a quotation nested in it '
            + '(n166, Z10)',
          fn: async () => {
            expect(maskQuotedSpeech({ text: 'The vet said, ‘the cat’s bowl is full.’ Then she left.', },),).toBe(
              `The vet said, ${' '.repeat('‘the cat’s bowl is full.’'.length,)} Then she left.`,
            );
            expect(maskQuotedSpeech({ text: 'The vet said, ‘she said ‘hi’ to me.’ Then she left.', },),).toBe(
              `The vet said, ${' '.repeat('‘she said ‘hi’ to me.’'.length,)} Then she left.`,
            );
          },
        },),

        it({
          name: 'KEEPS the text of an apostrophe inside a word and of an opening single mark that no mark closes, since the closing mark '
            + 'and the apostrophe are one character (n166, Z10)',
          fn: async () => {
            expect(maskQuotedSpeech({ text: 'The vet called it ‘the cat’s bowl and left.', },),).toBe('The vet called it ‘the cat’s bowl and left.',);
            expect(maskQuotedSpeech({ text: 'The cat’s bowl is full, and the cats’ bowls too.', },),).toBe(
              'The cat’s bowl is full, and the cats’ bowls too.',
            );
          },
        },),

        it({
          name: 'KEEPS the text between marks of two different pairs: an opening curly mark closed by a straight one, and a straight '
            + 'one closed by a curly one, are no curly pair and blank nothing, narration between them included',
          fn: async () => {
            /**
             Two quotations each opened and closed by marks of two kinds, with narration between them.
             */
            const text = '‘I nap.\' She naps. \'I sleep.’';
            expect(maskQuotedSpeech({ text, },),).toBe(text,);
          },
        },),
      ],
    },),

    describe({
      name: sentenceSpans.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SPLITS at a run of sentence endings and keeps the run with its sentence, and a last sentence without an ending',
          fn: async () => {
            expect(sentenceSpans({ text: 'Mittens naps... Did she? Yes', },),).toEqual([
              {
                start: 0,
                end: 15,
              },
              {
                start: 15,
                end: 24,
              },
              {
                start: 24,
                end: 28,
              },
            ],);
          },
        },),

        it({
          name: 'SPLITS nothing from an empty text',
          fn: async () => {
            expect(sentenceSpans({ text: '', },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);

//endregion Present-tense text tests
