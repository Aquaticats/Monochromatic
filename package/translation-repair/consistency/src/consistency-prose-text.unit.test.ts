/**
 Tests for the prose of an English: the text the reader of a page reads as
 words, with every other stretch (code, markup of a component, a comment, the
 destination of a link) blanked and no offset moved.

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
  blankedOf,
  emphasisDelimitersOf,
  proseViewOf,
} from '../dist/final/node/index.mjs';
import { withExhaustionFromParse, } from '@monochromatic-dev/translation-repair-test-fixture/ts/parse-exhaustion.test-fixture.ts';

/**
 The prose of an English: the view of one parse, without its delimiters. Production reads the prose and the delimiters
 together (`proseViewOf`), so this reader exists for the cases that pin the prose alone.

 @param english - English of a slice

 @returns Prose, as long as the English

 @example
 ```ts
 const prose = proseTextOf({ english: 'Fill `it\'s` now.', },); // 'Fill        now.'
 ```
 */
function proseTextOf({ english, }: { readonly english: string; },): string {
  /**
   What one parse of the English gives.
   */
  const { prose, } = proseViewOf({ english, },);
  return prose;
}

/**
 A run of blanks as long as a stretch of the English the prose leaves out.

 @param length - how many UTF-16 units the stretch holds

 @returns That many spaces

 @example
 ```ts
 const blank = blanks({ length: 3, },); // '   '
 ```
 */
function blanks({ length, }: { readonly length: number; },): string {
  return ' '.repeat(length,);
}

/**
 An English that raises a failure the first time the reading of a slice body reads it, which is in the masks before the
 parse, so a case meets there the failure it wants to meet and no stack to spend.

 @param error - what the first read of the English raises

 @returns Stand-in for an English, whose every read raises the error
 */
function throwingEnglish({ error, }: { readonly error: Error; },): string {
  return new Proxy(
    {},
    {
      get(): never {
        throw error;
      },
    },
  ) as unknown as string;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: proseTextOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LEAVES PLAIN PROSE AS IT IS, quotation marks, apostrophes and emphasis included, and moves no offset',
          fn: async () => {
            /**
             English made of text alone.
             */
            const english = 'The cats’ bowl, said Mio, was \'full\'.\n\nThen ‘the kittens’ ate.';
            expect(proseTextOf({ english, },),).toBe(english,);
          },
        },),
        it({
          name: 'BLANKS THE MARKERS OF EMPHASIS AND KEEPS THE EMPHASIZED WORDS, with the line breaks where they were',
          fn: async () => {
            expect(proseTextOf({ english: 'The *kittens*’ bowl.\n\nSaid **Mio**.', },),).toBe(
              'The  kittens ’ bowl.\n\nSaid   Mio  .',
            );
          },
        },),
        it({
          name: 'BLANKS INLINE CODE AND A FENCED CODE BLOCK, so a mark inside code is no mark of the prose, and keeps a '
            + 'backtick that opens no code',
          fn: async () => {
            expect([
              proseTextOf({ english: 'Fill `it\'s` now.', },),
              proseTextOf({ english: 'Fill\n\n```\nit\'s\n```\n\nthe cats’ bowl.', },),
              proseTextOf({ english: 'Fill the ` bowl.', },),
            ],).toEqual([
              `Fill ${blanks({ length: 6, },)} now.`,
              `Fill\n\n${blanks({ length: 3, },)}\n${blanks({ length: 4, },)}\n${blanks({ length: 3, },)}\n\nthe cats’ bowl.`,
              'Fill the ` bowl.',
            ],);
          },
        },),
        it({
          name: 'BLANKS A COMPONENT\'S MARKUP AND ITS STRING ATTRIBUTES, and an HTML comment, but keeps the words inside a '
            + 'component',
          fn: async () => {
            /**
             A component line whose strings hold straight single marks, like the photo lists of a page.
             */
            const component = '<PhotoScroll photos={[\'a.webp\', \'b.webp\']} />';
            expect([
              proseTextOf({ english: `${component}\n\nThe parents’ bowl.`, },),
              proseTextOf({ english: '<!-- it\'s a note -->\n\nThe cats’ bowl.', },),
              proseTextOf({ english: 'The <b>cats’</b> bowl.', },),
            ],).toEqual([
              `${blanks({ length: component.length, },)}\n\nThe parents’ bowl.`,
              `${blanks({ length: 20, },)}\n\nThe cats’ bowl.`,
              `The ${blanks({ length: 3, },)}cats’${blanks({ length: 4, },)} bowl.`,
            ],);
          },
        },),
        it({
          name: 'BLANKS THE DESTINATION OF A LINK AND KEEPS ITS TEXT, so a straight mark in an address is no mark of the prose',
          fn: async () => {
            expect(proseTextOf({ english: 'See [the cats](https://cats.example/it\'s) now.', },),).toBe(
              `See ${blanks({ length: 1, },)}the cats${blanks({ length: 28, },)} now.`,
            );
          },
        },),
        it({
          name: 'KEEPS THE WHOLE ENGLISH WHEN THE GRAMMAR REFUSES IT, so a page that cannot be read is read as text, where '
            + 'every mark is a possible partner',
          fn: async () => {
            /**
             An opening brace the grammar cannot close.
             */
            const english = 'Fill {the cats’ bowl, said \'Mio.';
            expect(proseTextOf({ english, },),).toBe(english,);
          },
        },),
        it({
          name: 'KEEPS THE WHOLE ENGLISH WHEN THE ENGINE REPORTS ITS STACK EXHAUSTED INSIDE THE PARSE, the same refusal a '
            + 'grammar refusal is, built where it is caught',
          fn: async () => {
            /**
             An English the grammar accepts, read while the parse it makes raises what the engine raises when its stack is spent.
             */
            const english = 'Fill the cats’ bowl, said \'Mio.\'';
            expect(withExhaustionFromParse({
              run: function read(): string {
                return proseTextOf({ english, },);
              },
            },),).toBe(english,);
          },
        },),
        it({
          name: 'PROPAGATES A STACK EXHAUSTION RAISED BEFORE THE PARSE, in one of the masks of a slice body, since a mask is no '
            + 'parse and only the parse call reads an exhaustion as the grammar\'s refusal',
          fn: async () => {
            /**
             The failure the first read of the English raises, which the masks make before any parse.
             */
            const spent = new RangeError('Maximum call stack size exceeded',);
            /**
             An English whose first read raises it.
             */
            const english = throwingEnglish({ error: spent, },);

            expect(caught(function readSpent(): void {
              proseTextOf({ english, },);
            },),).toBe(spent,);
          },
        },),
        it({
          name: 'PROPAGATES A RangeError WITH ANOTHER MESSAGE, since only a stack exhaustion is a refusal of the page',
          fn: async () => {
            /**
             The failure the parse raises, which is no stack exhaustion.
             */
            const stray = new RangeError('Invalid array length',);
            /**
             An English whose first read raises it.
             */
            const english = throwingEnglish({ error: stray, },);

            expect(caught(function readStray(): void {
              proseTextOf({ english, },);
            },),).toBe(stray,);          },
        },),
      ],
    },),
    describe({
      name: emphasisDelimitersOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE DELIMITERS THE PROSE BLANKS, the closing ones apart: one for emphasis, two for strong, one or two for '
            + 'strikethrough, and three for emphasis around strong',
          fn: async () => {
            /**
             The offsets of both kinds of delimiter, for an English.
             */
            function delimitersOf({ english, }: { readonly english: string; },): {
              readonly offsets: readonly number[];
              readonly closing: readonly number[];
            } {
              const { offsets, closing, } = emphasisDelimitersOf({ english, },);
              return {
                offsets: [...offsets,].toSorted(function ascending(left, right,): number {
                  return left - right;
                },),
                closing: [...closing,].toSorted(function ascending(left, right,): number {
                  return left - right;
                },),
              };
            }
            expect([
              delimitersOf({ english: 'a *b* c', },),
              delimitersOf({ english: 'a **b** c', },),
              delimitersOf({ english: 'a ~~b~~ c', },),
              delimitersOf({ english: 'a ~b~ c', },),
              delimitersOf({ english: 'a _b_ c', },),
              delimitersOf({ english: 'a ***b*** c', },),
            ],).toEqual([
              { offsets: [2, 4,], closing: [4,], },
              { offsets: [2, 3, 5, 6,], closing: [5, 6,], },
              { offsets: [2, 3, 5, 6,], closing: [5, 6,], },
              { offsets: [2, 4,], closing: [4,], },
              { offsets: [2, 4,], closing: [4,], },
              { offsets: [2, 3, 4, 6, 7, 8,], closing: [6, 7, 8,], },
            ],);
          },
        },),
        it({
          name: 'NAMES NO DELIMITER FOR A LONE ASTERISK, AN INTRAWORD UNDERSCORE, MARKS IN CODE OR A PAGE THE GRAMMAR REFUSES',
          fn: async () => {
            expect([
              [...emphasisDelimitersOf({ english: 'The cat*\'s bowl.', },).offsets,],
              [...emphasisDelimitersOf({ english: 'snake_case_name', },).offsets,],
              [...emphasisDelimitersOf({ english: 'Fill `*it*` now.', },).offsets,],
              [...emphasisDelimitersOf({ english: 'Fill {the *cats* bowl', },).offsets,],
              [...emphasisDelimitersOf({ english: '', },).offsets,],
            ],).toEqual([[], [], [], [], [],],);
          },
        },),
      ],
    },),
    describe({
      name: proseViewOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GIVES THE PROSE AND THE DELIMITERS OF ONE PARSE, the prose equal to the prose of the English and the delimiters '
            + 'those of the English',
          fn: async () => {
            /**
             An English with emphasis and a lone asterisk.
             */
            const english = 'The *kittens*’ bowl, the cat*\'s dish.';
            const view = proseViewOf({ english, },);
            expect({
              prose: view.prose,
              offsets: [...view.emphasis.offsets,],
              closing: [...view.emphasis.closing,],
            },).toEqual({
              prose: proseTextOf({ english, },),
              offsets: [...emphasisDelimitersOf({ english, },).offsets,],
              closing: [...emphasisDelimitersOf({ english, },).closing,],
            },);
          },
        },),
      ],
    },),
    describe({
      name: blankedOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TURNS EVERY UNIT OF A STRETCH INTO A SPACE AND KEEPS EVERY LINE BREAK, so the lines stay where they were',
          fn: async () => {
            expect([
              blankedOf({ text: 'it\'s', },),
              blankedOf({ text: '--\nthe cats’\n--', },),
              blankedOf({ text: '', },),
            ],).toEqual(['    ', '  \n         \n  ', '',],);
          },
        },),
      ],
    },),
  ],
},);
