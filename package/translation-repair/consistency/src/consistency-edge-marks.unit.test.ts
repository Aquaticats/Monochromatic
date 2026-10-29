/**
 Tests for taking the marks and punctuation at the two edges of a quote off its
 words: which edge characters are not the words, which stay because they pair
 with one inside, and that the reading reaches a fixed point.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { edgeFreeWordsOf, } from '../dist/final/node/index.mjs';
import { longerRuns, } from '@monochromatic-dev/translation-repair-test-fixture/ts/runs-of-pieces.test-fixture.ts';

/**
 The words left of a text once its edge characters are taken off.

 @param text - quote, already in the fold

 @returns Words as one text

 @example
 ```ts
 const words = wordsLeftOf({ text: '"cat naps".', },); // 'cat naps'
 ```
 */
function wordsLeftOf({ text, }: { readonly text: string; },): string {
  return edgeFreeWordsOf({ points: Array.from(text,), },)
    .join('',);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: edgeFreeWordsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TAKES SENTENCE PUNCTUATION AND SPACE OFF BOTH EDGES, and nothing inside',
          fn: async () => {
            expect([
              ' The cat, then naps?! ',
              '...',
              '',
            ].map(function leftOf(text,): string {
              return wordsLeftOf({ text, },);
            },),).toEqual(['The cat, then naps', '', '',],);
          },
        },),
        it({
          name: 'TAKES A MARK OFF THAT ENCLOSES ALL THE WORDS, one pair inside another included, once it is exposed',
          fn: async () => {
            expect([
              '"cat naps"',
              '(\'cat naps\')',
              '("cat naps").',
              '[(cat naps)]',
              '""',
            ].map(function leftOf(text,): string {
              return wordsLeftOf({ text, },);
            },),).toEqual(['cat naps', 'cat naps', 'cat naps', 'cat naps', '',],);
          },
        },),
        it({
          name: 'KEEPS A MARK AT AN EDGE THAT PAIRS WITH ONE INSIDE, a bracket or a quotation mark',
          fn: async () => {
            expect([
              '"cat," she said',
              'she said "cat"',
              '(cat) naps (here)',
              'cat naps (here)',
            ].map(function leftOf(text,): string {
              return wordsLeftOf({ text, },);
            },),).toEqual(['"cat," she said', 'she said "cat"', '(cat) naps (here)', 'cat naps (here)',],);
          },
        },),
        it({
          name: 'TAKES A MARK OFF AN EDGE THAT PAIRS WITH NOTHING, and a bracket that can only close or only open there',
          fn: async () => {
            expect([
              '"cat naps',
              'cat naps"',
              ')cat naps(',
              '(cat naps',
              'cat naps)',
            ].map(function leftOf(text,): string {
              return wordsLeftOf({ text, },);
            },),).toEqual(['cat naps', 'cat naps', 'cat naps', 'cat naps', 'cat naps',],);
          },
        },),
        it({
          name: 'KEEPS AN APOSTROPHE INSIDE A WORD OUT OF THE PAIRING, so a quote that opens beside one still closes',
          fn: async () => {
            expect([
              'wasn\'t kind',
              '\'wasn\'t kind\'',
              'the cats\' bowl',
            ].map(function leftOf(text,): string {
              return wordsLeftOf({ text, },);
            },),).toEqual(['wasn\'t kind', 'wasn\'t kind', 'the cats\' bowl',],);
          },
        },),
        it({
          name: 'TAKES A SINGLE MARK OFF AN EDGE THAT PAIRS WITH NOTHING IN THE QUOTE, WHATEVER IT IS: the apostrophe of a '
            + 'dropped opening, of a dropped final letter and of a plural, since the quote alone cannot tell it from a '
            + 'quotation mark',
          fn: async () => {
            expect([
              '\'Tis the season',
              'we were goin\'',
              'the cats\'',
              'the bowl of the cats\'',
              '\'90s cats',
            ].map(function leftOf(text,): string {
              return wordsLeftOf({ text, },);
            },),).toEqual([
              'Tis the season',
              'we were goin',
              'the cats',
              'the bowl of the cats',
              '90s cats',
            ],);
          },
        },),
        it({
          name: 'KEEPS A DOUBLE MARK BETWEEN TWO COMPATIBILITY LETTERS OUT OF THE PAIRING, as the neighbour test of the '
            + 'consistency pass reads it: a mark between mathematical script letters (`𝓠"𝓾`), fullwidth letters or a ligature and a '
            + 'letter is inside a word like one between ASCII letters, so the pair around the words is still taken off',
          fn: async () => {
            expect([
              '"cat s"u naps"',
              // U+1D4E0 MATHEMATICAL BOLD SCRIPT CAPITAL Q and U+1D4FE MATHEMATICAL BOLD SCRIPT SMALL U.
              '"cat \u{1D4E0}"\u{1D4FE} naps"',
              // U+FF21 FULLWIDTH LATIN CAPITAL LETTER A and U+FF53 FULLWIDTH LATIN SMALL LETTER S.
              '"cat \u{FF21}"\u{FF53} naps"',
              // U+FB01 LATIN SMALL LIGATURE FI before an ASCII letter.
              '"cat o\u{FB01}"s naps"',
            ].map(function leftOf(text,): string {
              return wordsLeftOf({ text, },);
            },),).toEqual([
              'cat s"u naps',
              'cat \u{1D4E0}"\u{1D4FE} naps',
              'cat \u{FF21}"\u{FF53} naps',
              'cat o\u{FB01}"s naps',
            ],);
          },
        },),
        it({
          name: 'TAKES A QUOTATION MARK OFF THAT A JUDGE COPIED WITHOUT ITS PAIR, before a letter or after one, with the '
            + 'punctuation beside it',
          fn: async () => {
            expect([
              '\'The cat naps',
              'Cats nap\'',
              '\'The cat naps.',
              'The cat naps,\'',
            ].map(function leftOf(text,): string {
              return wordsLeftOf({ text, },);
            },),).toEqual([
              'The cat naps',
              'Cats nap',
              'The cat naps',
              'The cat naps',
            ],);
          },
        },),
        it({
          name: 'TAKES BOTH MARKS OFF WHEN EACH EDGE HOLDS THE APOSTROPHE OF ITS OWN WORD, and leaves no mark of one '
            + 'behind',
          fn: async () => {
            expect([
              '\'Tis the cats\'',
              '\'90s cats\'',
              '\'Tis the season for the cats\'',
            ].map(function leftOf(text,): string {
              return wordsLeftOf({ text, },);
            },),).toEqual([
              'Tis the cats',
              '90s cats',
              'Tis the season for the cats',
            ],);
          },
        },),
        it({
          name: 'TAKES THE PAIR OFF THAT ENCLOSES THE WHOLE QUOTE while an apostrophe inside it stays, and keeps a pair '
            + 'that closes inside the quote',
          fn: async () => {
            expect([
              '\'cats\' bowl\'',
              '\'the cats\' bowl is full\'',
              '"He said \'hi\' to me"',
              '[a (b] c)',
            ].map(function leftOf(text,): string {
              return wordsLeftOf({ text, },);
            },),).toEqual([
              'cats\' bowl',
              'the cats\' bowl is full',
              'He said \'hi\' to me',
              '[a (b] c)',
            ],);
          },
        },),
        it({
          name: 'TAKES EVERY SINGLE MARK OFF AN EDGE WHETHER OR NOT IT PAIRS WITH A MARK INSIDE THE QUOTE, so no guess about '
            + 'which mark pairs decides what is inside the words',
          fn: async () => {
            expect([
              '\'the cats\' bowl',
              'the cats\' bowl\'',
              '\'the cats\' bowl is full',
              '\'hi\' there',
              'she said \'hi\'',
              '\'\'hi\'\' there',
            ].map(function leftOf(text,): string {
              return wordsLeftOf({ text, },);
            },),).toEqual([
              'the cats\' bowl',
              'the cats\' bowl',
              'the cats\' bowl is full',
              'hi\' there',
              'she said \'hi',
              'hi\'\' there',
            ],);
          },
        },),
        it({
          name: 'TAKES THE MODIFIER LETTER APOSTROPHE OFF EITHER EDGE like the straight mark, and keeps it inside a word',
          fn: async () => {
            expect([
              '\u{02BC}cat naps',
              'cat naps\u{02BC}',
              '\u{02BC}cat naps\u{02BC}',
              '\u{02BC}Tis the season',
              'the cats\u{02BC}',
              '\'\u{02BC}cat naps',
              'dog\u{02BC}s bowl',
            ].map(function leftOf(text,): string {
              return wordsLeftOf({ text, },);
            },),).toEqual([
              'cat naps',
              'cat naps',
              'cat naps',
              'Tis the season',
              'the cats',
              'cat naps',
              'dog\u{02BC}s bowl',
            ],);
          },
        },),
        it({
          name: 'TAKES A SINGLE QUOTATION MARK OFF AN EDGE THAT TOUCHES NO LETTER AND PAIRS WITH NOTHING',
          fn: async () => {
            expect([
              '\' cat naps',
              'cat naps \'',
              '\'\'',
            ].map(function leftOf(text,): string {
              return wordsLeftOf({ text, },);
            },),).toEqual(['cat naps', 'cat naps', '',],);
          },
        },),
        it({
          name: 'LEAVES BRACKETS THAT DO NOT BALANCE IN ORDER where they are, since taking one off would strand another',
          fn: async () => {
            expect(wordsLeftOf({ text: '(cat (naps)', },),).toBe('(cat (naps)',);
          },
        },),
        it({
          name: 'READS ITS OWN RESULT AGAIN AS THE SAME WORDS over every run of up to six of a letter, an s, a space, a '
            + 'full stop, the two quotation marks and a pair of brackets',
          fn: async () => {
            const pieces = ['a', 's', ' ', '.', '\'', '"', '(', ')',];
            const ones = longerRuns({ runs: [''], pieces, },);
            const twos = longerRuns({ runs: ones, pieces, },);
            const threes = longerRuns({ runs: twos, pieces, },);
            const fours = longerRuns({ runs: threes, pieces, },);
            const fives = longerRuns({ runs: fours, pieces, },);
            const changed = [...ones, ...twos, ...threes, ...fours, ...fives, ...longerRuns({ runs: fives, pieces, },),]
              .filter(function readsDifferently(text,): boolean {
                const once = wordsLeftOf({ text, },);
                return wordsLeftOf({ text: once, },) !== once;
              },);
            expect(changed,).toEqual([],);
          },
        },),
        it({
          name: 'READS ITS OWN RESULT AGAIN AS THE SAME WORDS when marks are exposed one beneath another',
          fn: async () => {
            const once = wordsLeftOf({ text: '(("cat naps")).', },);
            expect([once, wordsLeftOf({ text: once, },),],).toEqual(['cat naps', 'cat naps',],);
          },
        },),
        it({
          name: 'CUTS OR KEEPS A CHARACTER REFERENCE AT THE END AS THE CHARACTER IT SHOWS WOULD BE, as one unit: a reference that '
            + 'shows a letter or a sign stays whole, one that shows closing punctuation, space or a single mark goes whole, and one '
            + 'that shows a bracket pairs with a plain bracket',
          fn: async () => {
            expect([
              'cats &amp;',
              'cats&nbsp;',
              'dogs&#39;',
              'cats &amp;.',
              'cats &#x27;;',
              'cats&period;',
              'dogs&lt;',
              '(cats&rpar;',
              'cats &amp;&nbsp;',
            ].map(function wordsOf(text,): string {
              return wordsLeftOf({ text, },);
            },),).toEqual([
              'cats &amp;',
              'cats',
              'dogs',
              'cats &amp;',
              'cats',
              'cats',
              'dogs&lt;',
              'cats',
              'cats &amp;',
            ],);
          },
        },),
        it({
          name: 'TAKES OFF A SEMICOLON THAT ENDS NO REFERENCE THE PARSER DECODES, and one whose ampersand a backslash escapes',
          fn: async () => {
            expect([
              'cats;',
              'cats & dogs;',
              'cats &amp ;',
              String.raw`cats \&amp;`,
              'cats &;',
              'cats &ampersandandmore1234567890123456789012345;',
              'See Q&A;',
              'See R&D;',
              'Call a&b;',
              'Call a&zzzz;',
              'Call a&#00000000039;',
            ].map(function wordsOf(text,): string {
              return wordsLeftOf({ text, },);
            },),).toEqual([
              'cats',
              'cats & dogs',
              'cats &amp',
              String.raw`cats \&amp`,
              'cats &',
              'cats &ampersandandmore1234567890123456789012345',
              'See Q&A',
              'See R&D',
              'Call a&b',
              'Call a&zzzz',
              'Call a&#00000000039',
            ],);
          },
        },),
        it({
          name: 'CUTS A BACKSLASH ESCAPE AT THE END AS THE CHARACTER IT SHOWS WOULD BE, as one unit, never between the backslash and '
            + 'its character: an escaped stop, bracket or quotation mark goes whole, an escaped sign stays whole',
          fn: async () => {
            expect([
              String.raw`cats\.`,
              String.raw`cats\;`,
              String.raw`cats\\.`,
              String.raw`cats\.\.`,
              String.raw`cats\..`,
              String.raw`cats\\\!`,
              String.raw`the cats\)`,
              String.raw`the cats\]`,
              String.raw`the cats\"`,
              String.raw`the cats\'`,
              String.raw`(the cats\)`,
              String.raw`cats\*`,
              String.raw`cats\\`,
              'cats\\',
            ].map(function wordsOf(text,): string {
              return wordsLeftOf({ text, },);
            },),).toEqual([
              'cats',
              'cats',
              String.raw`cats\\`,
              'cats',
              'cats',
              String.raw`cats\\`,
              'the cats',
              'the cats',
              'the cats',
              'the cats',
              'the cats',
              String.raw`cats\*`,
              String.raw`cats\\`,
              'cats\\',
            ],);
          },
        },),
        it({
          name: 'CUTS A UNIT AT THE START AS THE CHARACTER IT SHOWS WOULD BE, a reference to a zero-width space whole, an escaped '
            + 'sign never between its backslash and the sign',
          fn: async () => {
            expect([
              '&ZeroWidthSpace;cats',
              '&#8203;cats',
              String.raw`\*kittens*`,
              String.raw`\(cats)`,
              String.raw`\cats`,
            ].map(function wordsOf(text,): string {
              return wordsLeftOf({ text, },);
            },),).toEqual([
              'cats',
              'cats',
              String.raw`\*kittens*`,
              'cats',
              String.raw`\cats`,
            ],);
          },
        },),
        it({
          name: 'READS A BACKSLASH ESCAPE FROM THE CHARACTERS THE JUDGE WROTE, not from the fold: a backslash before a curly mark the '
            + 'fold turned straight escapes nothing, so only the mark goes and the backslash stays',
          fn: async () => {
            expect({
              curly: edgeFreeWordsOf({ points: Array.from(String.raw`kittens\'`,), originals: Array.from(String.raw`kittens\’`,), },)
                .join('',),
              straight: edgeFreeWordsOf({ points: Array.from(String.raw`kittens\'`,), originals: Array.from(String.raw`kittens\'`,), },)
                .join('',),
            },).toEqual({ curly: 'kittens\\', straight: 'kittens', },);
          },
        },),
      ],
    },),
  ],
},);
