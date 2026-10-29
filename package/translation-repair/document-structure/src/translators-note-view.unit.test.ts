/**
 Tests for the map from the text a judge is shown, without the notes between
 translators, back to the stored text: the same shown text as
 `withoutTranslatorsNotes`, a span across a cut coming back as the stored words
 on each side of the note, and the edges where a span meets a cut.
 Fixtures are cat-themed invention only, in the shapes the pinned archives carry.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type ShownTranslation,
  shownTranslation,
  storedStretchesOf,
  withoutTranslatorsNotes,
} from '../dist/final/node/index.mjs';

/**
 A note ending a sentence.
 */
const TRAILING = '<!-- 前面这句猫话懂什么意思，先这么翻译，大家看一下 -->';

/**
 A short JSX note.
 */
const SHORT_NOTE = '{/* 注 */}';

/**
 Texts in the shapes a pinned archive carries a note in, each with a note the original lacks.
 */
const SHAPES: readonly string[] = [
  `The cat slept. ${TRAILING}`,
  `The cat slept. ${TRAILING} \n\nIt purred.\n`,
  `The cat slept.${SHORT_NOTE}\nThen it purred.\n`,
  `The cat slept.\n${SHORT_NOTE}\nThen it purred.\n`,
  `The cat slept.\n\n${SHORT_NOTE}\n\nThen it purred.\n`,
  `> The cat slept.\n>   > ${SHORT_NOTE}\n>   > Then it purred.\n`,
  `The ${TRAILING} cat slept. ${SHORT_NOTE} Then it purred.`,
  `A ${SHORT_NOTE} ${SHORT_NOTE} cat slept.\nThen it purred.\n`,
  `The cat slept.${SHORT_NOTE}\r\nThen it purred.\r\n`,
  `${SHORT_NOTE}\nThe cat slept.\n`,
  `The cat slept.\n${SHORT_NOTE}`,
  'The cat slept.\nThen it purred.\n',
  '',
];

/**
 Every stretch of the stored text a span of the shown one comes back as, joined.

 @param shown - shown text with its runs

 @param text - stored text

 @param start - first offset of the span in the shown text

 @param end - exclusive offset where it ends

 @returns Stored words of each stretch, in order

 @example
 ```ts
 const words = storedWordsOf({ shown, text, start: 0, end: 5, },);
 ```
 */
function storedWordsOf(
  {
    shown,
    text,
    start,
    end,
  }: {
    readonly shown: ShownTranslation;
    readonly text: string;
    readonly start: number;
    readonly end: number;
  },
): readonly string[] {
  return storedStretchesOf({
    shown,
    start,
    end,
  },)
    .map(function wordsOf(stretch,): string {
      return text.slice(
        stretch.from,
        stretch.to,
      );
    },);
}

/**
 A text without its spaces, tabs and line breaks, which a cut may give up beside a note.

 @param text - text to read

 @returns Text with those characters taken out

 @example
 ```ts
 const bare = withoutSpace({ text: 'a b\n', },); // 'ab'
 ```
 */
function withoutSpace({ text, }: { readonly text: string; },): string {
  return Array.from(text,)
    .filter(function isKept(character,): boolean {
      return !' \t\n\r'.includes(character,);
    },)
    .join('',);
}

await describe({
  name: '',
  children: [
    describe({
      name: shownTranslation.name,
      children: [
        it({
          name: 'SHOWS the same text as withoutTranslatorsNotes does for the same arguments, in every shape',
          fn: async () => {
            for (const text of SHAPES) {
              expect(shownTranslation({ text, },).text,).toBe(withoutTranslatorsNotes({ text, },),);
            }
          },
        },),
        it({
          name: 'KEEPS a note the original also writes, as withoutTranslatorsNotes does, so the whole text is one run',
          fn: async () => {
            /** Text holding a note the original repeats. */
            const text = `The cat slept. ${SHORT_NOTE}\nIt purred.\n`;
            const shown = shownTranslation({
              text,
              original: `猫睡了。${SHORT_NOTE}\n`,
            },);
            expect(shown.text,).toBe(text,);
            expect(shown.kept,).toEqual([{ shownStart: 0, storedStart: 0, length: text.length, },],);
          },
        },),
        it({
          name: 'PLACES each run at the offset it has in both texts',
          fn: async () => {
            /** Text whose note ends the first line. */
            const text = `The cat slept.${SHORT_NOTE}\nThen it purred.\n`;
            const shown = shownTranslation({ text, },);
            expect(shown.kept,).toEqual([
              { shownStart: 0, storedStart: 0, length: 14, },
              { shownStart: 14, storedStart: 23, length: 17, },
            ],);
            expect(shown.storedLength,).toBe(text.length,);
          },
        },),
      ],
    },),

    describe({
      name: storedStretchesOf.name,
      children: [
        it({
          name: 'RETURNS a span inside one run at the offsets it has there',
          fn: async () => {
            /** Text whose note ends the first line. */
            const text = `The cat slept.${SHORT_NOTE}\nThen it purred.\n`;
            const shown = shownTranslation({ text, },);
            expect(storedWordsOf({ shown, text, start: 0, end: 7, },),).toEqual(['The cat',],);
            expect(storedWordsOf({ shown, text, start: 15, end: 22, },),).toEqual(['Then it',],);
            expect(storedStretchesOf({ shown, start: 15, end: 22, },),).toEqual([{ from: 24, to: 31, },],);
          },
        },),
        it({
          name: 'SPLITS a span across a note that ends a line into the words on each side, the line break given up',
          fn: async () => {
            /** Text whose note ends the first line. */
            const text = `The cat slept.${SHORT_NOTE}\nThen it purred.\n`;
            const shown = shownTranslation({ text, },);
            expect(storedWordsOf({ shown, text, start: 4, end: 22, },),).toEqual(['cat slept.', 'Then it',],);
          },
        },),
        it({
          name: 'LEAVES THE LINE of a note alone on it out of every stretch, at one level of quoting and at two',
          fn: async () => {
            /** Text whose note stands alone on a line. */
            const alone = `The cat slept.\n${SHORT_NOTE}\nThen it purred.\n`;
            /** Quoted text whose note stands alone on a doubly quoted line. */
            const quoted = `> The cat slept.\n>   > ${SHORT_NOTE}\n>   > Then it purred.\n`;
            for (const text of [alone, quoted,]) {
              const shown = shownTranslation({ text, },);
              /** The stored words the whole of the shown text comes back as. */
              const words = storedWordsOf({
                shown,
                text,
                start: 0,
                end: shown.text.length,
              },);
              expect(words.join('',).includes('注',),).toBe(false,);
              expect(words.length,).toBe(2,);
              expect(words[0]?.endsWith('The cat slept.',),).toBe(true,);
              expect(words[1]?.endsWith('Then it purred.\n',),).toBe(true,);
            }
          },
        },),
        it({
          name: 'ENDS a span that stops where a cut begins at the cut and STARTS one that begins where it ends past it',
          fn: async () => {
            /** Text whose note, with the space after it, is cut between two words. */
            const text = `The cat slept. ${SHORT_NOTE} Then it purred.`;
            const shown = shownTranslation({ text, },);
            /** Where the shown text has its first sentence end. */
            const stop = 'The cat slept.'.length;
            expect(storedStretchesOf({ shown, start: 0, end: stop, },),).toEqual([{ from: 0, to: stop, },],);
            /** Where the second sentence starts in the shown text. */
            const resume = shown.text.indexOf('Then',);
            expect(storedStretchesOf({ shown, start: resume, end: shown.text.length, },),).toEqual([
              { from: text.indexOf('Then',), to: text.length, },
            ],);
          },
        },),
        it({
          name: 'GIVES UP THE SPACE a cut left on the edge it touches, so a span across a note between words is two stretches of words',
          fn: async () => {
            /** Text whose note is cut with the space after it. */
            const text = `The cat ${SHORT_NOTE} slept.`;
            const shown = shownTranslation({ text, },);
            expect(shown.text,).toBe('The cat slept.',);
            expect(storedWordsOf({ shown, text, start: 0, end: shown.text.length, },),).toEqual(['The cat', 'slept.',],);
          },
        },),
        it({
          name: 'GIVES UP THE LINE BREAK of a CRLF line a note ended, the carriage return with it',
          fn: async () => {
            /** CRLF text whose note ends the first line. */
            const text = `The cat slept.${SHORT_NOTE}\r\nThen it purred.\r\n`;
            const shown = shownTranslation({ text, },);
            expect(storedWordsOf({ shown, text, start: 0, end: shown.text.length - 2, },),).toEqual([
              'The cat slept.',
              'Then it purred.',
            ],);
          },
        },),
        it({
          name: 'TREATS two notes side by side as one cut, so a span across them is two stretches',
          fn: async () => {
            /** Text with two notes on one line. */
            const text = `A ${SHORT_NOTE} ${SHORT_NOTE} cat slept.`;
            const shown = shownTranslation({ text, },);
            expect(shown.kept.length,).toBe(2,);
            expect(storedWordsOf({ shown, text, start: 0, end: shown.text.length, },),).toEqual(['A', 'cat slept.',],);
          },
        },),
        it({
          name: 'RETURNS nothing for a span that is only the whitespace a cut left, and for a span that reaches no run',
          fn: async () => {
            /** Text whose note ends the first line. */
            const text = `The cat slept.${SHORT_NOTE}\nThen it purred.\n`;
            const shown = shownTranslation({ text, },);
            expect(storedStretchesOf({ shown, start: 14, end: 15, },),).toEqual([],);
            expect(storedStretchesOf({ shown, start: 40, end: 50, },),).toEqual([],);
          },
        },),
        it({
          name: 'KEEPS a text with no note as one run that gives nothing up, so no stretch moves',
          fn: async () => {
            /** Text with no note, edged by whitespace. */
            const text = ' The cat slept.\n';
            const shown = shownTranslation({ text, },);
            expect(storedStretchesOf({ shown, start: 0, end: text.length, },),).toEqual([{ from: 0, to: text.length, },],);
          },
        },),
        it({
          name: 'COMES BACK, for every span of every shape, as the shown words without a note in them',
          fn: async () => {
            for (const text of SHAPES) {
              const shown = shownTranslation({ text, },);
              for (let start = 0; start <= shown.text.length; start += 1) {
                for (let end = start; end <= shown.text.length; end += 1) {
                  /** The words the span comes back as, in the stored text. */
                  const words = storedWordsOf({
                    shown,
                    text,
                    start,
                    end,
                  },);
                  expect(withoutSpace({ text: words.join('',), },),).toBe(withoutSpace({
                    text: shown.text
                      .slice(
                      start,
                      end,
                      ),
                  },),);
                  expect(words.join('',).includes('注',),).toBe(false,);
                  expect(words.join('',).includes('猫话',),).toBe(false,);
                }
              }
            }
          },
        },),
      ],
    },),
  ],
},);
