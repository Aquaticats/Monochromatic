/**
 Tests for the apostrophes of an English that the proof reads as joined to the word after them across closing marks: the
 offsets in the English as it was written, the compatibility letters that join, the marks that open or reach no run of
 closing marks, the closing delimiters of emphasis by the parse, and the links read as a reader sees them (the same twin of the
 English that the cut of a word reads).

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { joinedMarksOf, } from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    describe({
      name: joinedMarksOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE APOSTROPHES AFTER CLOSING MARKS BY THEIR OFFSET IN THE ENGLISH AS WRITTEN, whatever the fold did to the '
            + 'white space before them and whatever form the mark is written in',
          fn: async () => {
            /**
             The English: a line break and a run of spaces before the marks, which the fold folds to one unit each.
             */
            const english = 'Fill\n\n   the **cat**\'s bowl and the "dog"’s and (the pup)ʼs and [a cat](b)\'s.';
            expect([...joinedMarksOf({ english, },),].toSorted(function ascending(
              left,
              right,
            ): number {
              return left - right;
            },),).toEqual([
              english.indexOf('**\'s',) + 2,
              english.indexOf('"’s',) + 1,
              english.indexOf(')ʼs',) + 1,
              english.indexOf('(b)\'s',) + 3,
            ],);
          },
        },),
        it({
          name: 'NAMES THE APOSTROPHE BEFORE A COMPATIBILITY LETTER, as the neighbour test of the consistency pass reads it: '
            + '`the **cat**’𝓠𝓾𝓪𝓷` and `the **cat**’Ａ` name it, `the **cat**’µ` names none',
          fn: async () => {
            /**
             The English: one apostrophe after closing emphasis, then a mathematical script letter.
             */
            const script = 'the **cat**\'\u{1D4E0}\u{1D4FE}\u{1D4EA}\u{1D4F7} bowl';
            /**
             The English: one apostrophe after closing emphasis, then a fullwidth letter.
             */
            const fullwidth = 'the **cat**\'\u{FF21} bowl';
            /**
             The English: one apostrophe after closing emphasis, then a micro sign, which is Greek under NFKC.
             */
            const micro = 'the **cat**\'\u{00B5} bowl';
            expect([
              [...joinedMarksOf({ english: script, },),],
              [...joinedMarksOf({ english: fullwidth, },),],
              [...joinedMarksOf({ english: micro, },),],
            ],).toEqual([
              [script.indexOf('**\'',) + 2,],
              [fullwidth.indexOf('**\'',) + 2,],
              [],
            ],);
          },
        },),
        it({
          name: 'NAMES NO MARK THAT NO RUN OF CLOSING MARKS REACHES, or that no letter or digit follows, or that opens: after '
            + 'a space, after opening emphasis, before a space, and the opening curly single mark',
          fn: async () => {
            expect([
              'the cat \'s bowl',
              '*\'Tis* well',
              'the **cats**\' bowl',
              'the **cat**‘s bowl',
              'the cat\'s bowl',
              'the cat bowl',
              '',
            ].map(function joinedIn(english,): readonly number[] {
              return [...joinedMarksOf({ english, },),];
            },),).toEqual([[], [], [], [], [], [], [],],);
          },
        },),
      ],
    },),
    describe({
      name: `${joinedMarksOf.name} by the parse`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES NO APOSTROPHE AFTER A LONE ASTERISK, which is a character, and still names one after a closed stretch',
          fn: async () => {
            expect([
              [...joinedMarksOf({ english: 'The cat*\'s bowl.', },),],
              [...joinedMarksOf({ english: 'The *cat*\'s bowl.', },),],
            ],).toEqual([[], [9,],],);
          },
        },),
      ],
    },),
    describe({
      name: `${joinedMarksOf.name} with links`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A LINK AS A READER SEES IT, as the cut of a word does: a link that shows nothing is not there, so the '
            + 'apostrophe after it opens a word and joins none, a link that shows closing marks and nothing else closes as those '
            + 'marks do, and a link whose label ends in a closing mark keeps that mark',
          fn: async () => {
            /**
             English with one link before an apostrophe and a word, and the offsets of the apostrophes that join.
             */
            const rows: readonly (readonly [string, readonly number[]])[] = [
              ['see [](u)\'kit here', [],],
              [String.raw`see [\]](u)'kit here`, [11,],],
              ['see [kit)](u)\'s bowl', [13,],],
              ['see [kit!](u)\'s bowl', [],],
              ['see [Kit.](u)\'s bowl', [],],
              ['see [kit ](u)\'s bowl', [],],
              ['see [kit](u)\'s bowl', [12,],],
            ];
            expect(rows.map(function joinedIn([english,],): readonly number[] {
              return [...joinedMarksOf({ english, },),];
            },),).toEqual(rows.map(function expectedOf([, joined,],): readonly number[] {
              return joined;
            },),);
          },
        },),
      ],
    },),
  ],
},);
