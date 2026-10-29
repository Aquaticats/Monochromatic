/**
 Tests for reading the form of the apostrophes a page writes outside the words
 of a place: the one form they share, or none when they share none or the page
 holds none. The page is the whole page when the caller hands it (the slices
 beside the place's own count), less the stretches the caller names, and the
 place's own slice alone when it hands none.

 Fixtures are cat-themed invention; no corpus content appears here.

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
  ConsistencyInvariantError,
  inReceivingForm,
  pageApostropheFormOf,
} from '../dist/final/node/index.mjs';

/**
 The span of the last `the family` of an English, the words the form is asked for.

 @param english - English, which holds the words

 @returns Start and end of the words

 @example
 ```ts
 const span = spanOfFamily({ english: 'Fill the family bowl.', },); // { start: 5, end: 15 }
 ```
 */
function spanOfFamily({ english, }: { readonly english: string; },): { readonly start: number; readonly end: number; } {
  /**
   Where the words start.
   */
  const start = english.lastIndexOf('the family',);
  return {
    start,
    end: start + 'the family'.length,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: pageApostropheFormOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GIVES THE ONE FORM THE APOSTROPHES OUTSIDE THE SPAN SHARE, those inside a word and a proven one at the end of '
            + 'a word, straight, curly or the modifier letter',
          fn: async () => {
            expect([
              pageApostropheFormOf({ english: 'Fill the family bowl. The cat’s dish.', start: 5, end: 15, },),
              pageApostropheFormOf({ english: 'Fill the family bowl. The cat\'s dish.', start: 5, end: 15, },),
              pageApostropheFormOf({ english: 'Fill the family bowl. The cat\u{02BC}s dish.', start: 5, end: 15, },),
              pageApostropheFormOf({ english: 'The parents’ bowl and the family dish.', start: 22, end: 32, },),
            ],).toEqual([['’',], ['\'',], ['\u{02BC}',], ['’',],],);
          },
        },),
        it({
          name: 'GIVES NONE WHEN THE APOSTROPHES OUTSIDE THE SPAN ARE OF MORE THAN ONE FORM OR THE PAGE HOLDS NONE, and counts '
            + 'no quotation mark, no apostrophe inside the span and none inside code',
          fn: async () => {
            expect([
              pageApostropheFormOf({ english: 'Fill the family bowl. The cat’s dish and the pup\'s bed.', start: 5, end: 15, },),
              pageApostropheFormOf({ english: 'Fill the family bowl.', start: 5, end: 15, },),
              pageApostropheFormOf({ english: '‘Mio’ said, fill the family bowl.', start: 17, end: 27, },),
              pageApostropheFormOf({ english: 'Fill the cat’s dish and bowl.', start: 5, end: 14, },),
              pageApostropheFormOf({ english: 'Say `it\'s` and fill the family bowl.', start: 20, end: 30, },),
            ],).toEqual([[], [], [], [], [],],);
          },
        },),
        it({
          name: 'READS THE APOSTROPHES OF THE WHOLE PAGE, in the slices beside the slice of the span, so a slice with none of its '
            + 'own takes the form its page writes, and the page\'s other forms count however many slices they stand in',
          fn: async () => {
            /**
             The slice of the span, and where it starts in each page of the cases.
             */
            const english = 'Fill the family bowl.';
            expect([
              pageApostropheFormOf({
                english,
                start: 5,
                end: 15,
                page: { text: `The cat’s dish.\n\nThe pup’s bed.\n\n${english}`, offset: 33, },
              },),
              pageApostropheFormOf({
                english,
                start: 5,
                end: 15,
                page: { text: `${english}\n\nThe cat's dish.\n\nThe pup's bed.`, offset: 0, },
              },),
              pageApostropheFormOf({
                english,
                start: 5,
                end: 15,
                page: { text: `The cat’s dish.\n\n${english}\n\nThe pup's bed.`, offset: 17, },
              },),
              pageApostropheFormOf({
                english,
                start: 5,
                end: 15,
                page: { text: `Say \`it's\`.\n\n${english}\n\nThe cat’s dish.`, offset: 13, },
              },),
              pageApostropheFormOf({ english, start: 5, end: 15, page: { text: `Say \`it's\`.\n\n${english}`, offset: 13, }, },),
            ],).toEqual([['’',], ['\'',], [], ['’',], [],],);
          },
        },),
        it({
          name: 'LEAVES OUT THE STRETCHES OF THE PAGE IT IS TOLD TO, the chosen words that arrive, so the page\'s own form is the '
            + 'one read and the arriving mark does not mix it',
          fn: async () => {
            /**
             The slice of the span, and the page: a curly slice, the slice of the span, and the slice of the chosen words.
             */
            const english = 'Fill the family bowl.';
            const text = `The cat’s dish.\n\n${english}\n\nThen the dogs' nap.`;
            expect([
              pageApostropheFormOf({ english, start: 5, end: 15, page: { text, offset: 17, }, apart: [{ start: 45, end: 54, },], },),
              pageApostropheFormOf({ english, start: 5, end: 15, page: { text, offset: 17, }, },),
            ],).toEqual([['’',], [],],);
          },
        },),
        it({
          name: 'REFUSES A PAGE THAT DOES NOT HOLD THE SLICE AT THE OFFSET IT IS GIVEN, since the span would then name other '
            + 'words than the slice\'s',
          fn: async () => {
            expect(
              caught(function readsMisplaced(): unknown {
                return pageApostropheFormOf({
                  english: 'Fill the family bowl.',
                  start: 5,
                  end: 15,
                  page: { text: 'The cat’s dish.', offset: 10, },
                },);
              },),
            ).toBeInstanceOf(ConsistencyInvariantError,);
          },
        },),
      ],
    },),
    describe({
      name: `${pageApostropheFormOf.name} after markup and by the neighbour test of the consistency pass`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS THE APOSTROPHES THE PROOF READS AS JOINED AFTER CLOSING MARKUP, after emphasis, a link, a bracket and a '
            + 'double quotation mark, so a page that writes only those has a form',
          fn: async () => {
            expect([
              pageApostropheFormOf({ english: 'Fill *Mio*’s dish. Fill the family bowl.', start: 22, end: 32, },),
              pageApostropheFormOf({ english: 'Fill [Mio](https://cats.example/m)’s dish. Fill the family bowl.', start: 46, end: 56, },),
              pageApostropheFormOf({ english: 'Fill (Mio)\'s dish. Fill the family bowl.', start: 19, end: 29, },),
              pageApostropheFormOf({ english: 'Fill "Mio"ʼs dish. Fill the family bowl.', start: 19, end: 29, },),
            ],).toEqual([['’',], ['’',], ['\'',], ['\u{02BC}',],],);
          },
        },),
        it({
          name: 'COUNTS AN APOSTROPHE BETWEEN COMPATIBILITY LETTERS AS INSIDE A WORD, by the neighbour test of the consistency '
            + 'pass: one written in mathematical script letters (`𝓠𝓾𝓪𝓷’s`) or fullwidth ones is counted though a quotation '
            + 'could close with a later mark, and one beside a micro sign (NFKC Greek) is a mark that could open and is not',
          fn: async () => {
            /**
             The English whose first mark stands between mathematical script letters and a letter, with a quotation after it
             whose closing mark could have been the partner of a leading mark.
             */
            const script = 'Fill \u{1D4E0}\u{1D4FE}\u{1D4EA}\u{1D4F7}’s dish and ‘Mio’ said. Fill the family bowl.';
            /**
             The English whose first mark stands between two fullwidth letters, with the same quotation after it.
             */
            const fullwidth = 'Fill \u{FF21}’\u{FF53} dish and ‘Mio’ said. Fill the family bowl.';
            /**
             The English whose first mark stands between a micro sign and a letter, which is Greek under NFKC.
             */
            const micro = 'Fill \u{00B5}’s dish and ‘Mio’ said. Fill the family bowl.';
            expect([
              pageApostropheFormOf({ english: script, ...spanOfFamily({ english: script, },), },),
              pageApostropheFormOf({ english: fullwidth, ...spanOfFamily({ english: fullwidth, },), },),
              pageApostropheFormOf({ english: micro, ...spanOfFamily({ english: micro, },), },),
            ],).toEqual([['’',], ['’',], [],],);
          },
        },),
        it({
          name: 'GIVES NONE WHEN A JOINED APOSTROPHE AND ANOTHER OF A WORD DIFFER IN FORM, and counts no mark between two '
            + 'letters of another script, which stands beside no word of Latin script',
          fn: async () => {
            expect([
              pageApostropheFormOf({ english: 'Fill Mio’s dish and *Kiki*\'s dish. Fill the family bowl.', start: 40, end: 50, },),
              pageApostropheFormOf({ english: 'Fill 猫’猫 dish. Fill the family bowl.', start: 17, end: 27, },),
              pageApostropheFormOf({ english: 'Fill αβ’γδ dish. Fill the family bowl.', start: 17, end: 27, },),
            ],).toEqual([[], [], [],],);
          },
        },),
      ],
    },),
    describe({
      name: inReceivingForm.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WRITES THE APOSTROPHES OF A WORD IN THE FORM THE RECEIVING WORDS SHARE, whatever form the page has, and the '
            + 'modifier letter apostrophe is a form of its own',
          fn: async () => {
            expect([
              inReceivingForm({ text: 'the dogs\'', receiving: 'the kittens’', page: () => ['\'',], },),
              inReceivingForm({ text: 'the dog’s bowl', receiving: 'the kitten\'s bowl', },),
              inReceivingForm({ text: 'the dog’s bowl', receiving: 'the kittenʼs bowl', },),
              inReceivingForm({ text: 'the dog\'s and the cat\'s', receiving: 'the kitten’s and the pup’s', },),
            ],).toEqual([
              'the dogs’',
              'the dog\'s bowl',
              'the dogʼs bowl',
              'the dog’s and the cat’s',
            ],);
          },
        },),
        it({
          name: 'FOLLOWS THE PAGE ONLY WHEN THE RECEIVING WORDS HOLD NO ONE FORM, and keeps the chosen place\'s form when the '
            + 'page names none, the receiving words mix forms or the words hold no apostrophe of a word',
          fn: async () => {
            expect([
              inReceivingForm({ text: 'the dogs\'', receiving: 'the family', page: () => ['’',], },),
              inReceivingForm({ text: 'the dogs\'', receiving: 'the family', page: () => [], },),
              inReceivingForm({ text: 'the dogs\'', receiving: 'the family', },),
              inReceivingForm({ text: 'the dogs\'', receiving: 'the cat’s and the pup\'s', page: () => ['’',], },),
              inReceivingForm({ text: 'the dogs', receiving: 'the kittens’', page: () => ['\'',], },),
              inReceivingForm({ text: '“the dogs”', receiving: 'the kittens’', },),
            ],).toEqual([
              'the dogs’',
              'the dogs\'',
              'the dogs\'',
              'the dogs’',
              'the dogs',
              '“the dogs”',
            ],);
          },
        },),
      ],
    },),
  ],
},);
