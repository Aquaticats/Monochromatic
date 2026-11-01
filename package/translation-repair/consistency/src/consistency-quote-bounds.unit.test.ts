/**
 Tests for where a quote's words begin and end: the space and invisible
 characters around them and the punctuation that closes them left out, a
 character beyond the first plane never cut in halves.

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
  quoteBounds,
  quoteCore,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: quoteCore.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TAKES THE CLOSING PUNCTUATION AND SPACE OFF A QUOTE\'S END and nothing else',
          fn: async () => {
            expect({
              full: quoteCore({ quote: 'the cat naps.', },),
              many: quoteCore({ quote: 'the cat naps...!? ', },),
              wide: quoteCore({ quote: '小猫在睡觉。', },),
              inner: quoteCore({ quote: 'the cat, napping', },),
              quoted: quoteCore({ quote: 'she said "hi"', },),
              none: quoteCore({ quote: '.!', },),
            },).toEqual({
              full: 'the cat naps',
              many: 'the cat naps',
              wide: '小猫在睡觉',
              inner: 'the cat, napping',
              quoted: 'she said "hi"',
              none: '',
            },);
          },
        },),
        it({
          name: 'CUTS OR KEEPS A CHARACTER REFERENCE AT THE END OF A QUOTE AS THE CHARACTER IT SHOWS WOULD BE, as one unit: a reference '
            + 'that shows a letter or a mark the quote does not cut stays whole, one that shows closing punctuation or space goes whole',
          fn: async () => {
            expect({
              quote: quoteCore({ quote: 'dogs&#39;', },),
              curly: quoteCore({ quote: 'dogs&rsquo;', },),
              ampersand: quoteCore({ quote: 'cats &amp;', },),
              space: quoteCore({ quote: 'cats&nbsp;', },),
              hexadecimal: quoteCore({ quote: 'dogs&#x2019;', },),
              stop: quoteCore({ quote: 'dogs&#39;.', },),
              twice: quoteCore({ quote: 'dogs&#39;;', },),
              literal: quoteCore({ quote: 'cats;', },),
              noName: quoteCore({ quote: 'cats &;', },),
              notClosed: quoteCore({ quote: 'cats &amp bowl;', },),
              inner: quoteCore({ quote: 'cat&#39;s bowl.', },),
              shownStop: quoteCore({ quote: 'cats&period;', },),
              shownStopNumeric: quoteCore({ quote: 'cats&#46;', },),
              shownComma: quoteCore({ quote: 'cats&comma;.', },),
              shownSpaceNumeric: quoteCore({ quote: 'cats&#160;', },),
              shownLetter: quoteCore({ quote: 'cats&eacute;', },),
            },).toEqual({
              quote: 'dogs&#39;',
              curly: 'dogs&rsquo;',
              ampersand: 'cats &amp;',
              space: 'cats',
              hexadecimal: 'dogs&#x2019;',
              stop: 'dogs&#39;',
              twice: 'dogs&#39;',
              literal: 'cats',
              noName: 'cats &',
              notClosed: 'cats &amp bowl',
              inner: 'cat&#39;s bowl',
              shownStop: 'cats',
              shownStopNumeric: 'cats',
              shownComma: 'cats',
              shownSpaceNumeric: 'cats',
              shownLetter: 'cats&eacute;',
            },);
          },
        },),
        it({
          name: 'READS ONLY WHAT THE PARSER DECODES AS A REFERENCE AT THE END OF A QUOTE: a word that merely looks like a reference '
            + '(`Q&A;`, `R&D;`, `a&b;`, `a&zzzz;`, a numeric reference past the parser\'s digits) ends in a semicolon that is closing '
            + 'punctuation, as after any other word',
          fn: async () => {
            expect({
              questionAnswer: quoteCore({ quote: 'See Q&A;', },),
              research: quoteCore({ quote: 'See R&D;', },),
              telephone: quoteCore({ quote: 'AT&T;', },),
              short: quoteCore({ quote: 'Call a&b;', },),
              unknownName: quoteCore({ quote: 'Call a&zzzz;', },),
              elevenDigits: quoteCore({ quote: 'Call a&#00000000039;', },),
              sevenDigits: quoteCore({ quote: 'Call a&#0000039;', },),
              control: quoteCore({ quote: 'AT&amp;', },),
            },).toEqual({
              questionAnswer: 'See Q&A',
              research: 'See R&D',
              telephone: 'AT&T',
              short: 'Call a&b',
              unknownName: 'Call a&zzzz',
              elevenDigits: 'Call a&#00000000039',
              sevenDigits: 'Call a&#0000039;',
              control: 'AT&amp;',
            },);
          },
        },),
        it({
          name: 'CUTS A BACKSLASH ESCAPE AT THE END OF A QUOTE AS THE CHARACTER IT SHOWS WOULD BE, as one unit: an escaped stop, '
            + 'bracket or quotation mark goes with its backslash like a plain one, and an escaped letter-like or markup character stays',
          fn: async () => {
            expect({
              stop: quoteCore({ quote: String.raw`cats\.`, },),
              semicolon: quoteCore({ quote: String.raw`cats\;`, },),
              exclamation: quoteCore({ quote: String.raw`cats\!`, },),
              escapedBackslash: quoteCore({ quote: String.raw`cats\\.`, },),
              threeBackslashes: quoteCore({ quote: String.raw`cats\\\.`, },),
              lone: quoteCore({ quote: 'cats\\', },),
              letter: quoteCore({ quote: String.raw`cats\a.`, },),
              ampersand: quoteCore({ quote: String.raw`cats\&amp;`, },),
              escapedStopThenStop: quoteCore({ quote: String.raw`cats\..`, },),
              reference: quoteCore({ quote: String.raw`cats&amp;\.`, },),
              star: quoteCore({ quote: String.raw`cats\*`, },),
              hash: quoteCore({ quote: String.raw`cats\#`, },),
              backslashesOnly: quoteCore({ quote: String.raw`cats\\`, },),
            },).toEqual({
              stop: 'cats',
              semicolon: 'cats',
              exclamation: 'cats',
              escapedBackslash: String.raw`cats\\`,
              threeBackslashes: String.raw`cats\\`,
              lone: 'cats\\',
              letter: String.raw`cats\a`,
              ampersand: String.raw`cats\&amp`,
              escapedStopThenStop: 'cats',
              reference: 'cats&amp;',
              star: String.raw`cats\*`,
              hash: String.raw`cats\#`,
              backslashesOnly: String.raw`cats\\`,
            },);
          },
        },),
        it({
          name: 'CUTS A UNIT THAT SHOWS NOTHING OFF THE START OF A QUOTE WHOLE: a reference to a zero-width space goes with all its '
            + 'characters, a backslash before a letter is a word character and stays',
          fn: async () => {
            expect({
              named: quoteCore({ quote: '&ZeroWidthSpace;cats', },),
              numeric: quoteCore({ quote: '&#8203;&#x200B;cats', },),
              space: quoteCore({ quote: '&nbsp;cats', },),
              escapedLetter: quoteCore({ quote: String.raw`\cats`, },),
              escapedStar: quoteCore({ quote: String.raw`\*cats`, },),
            },).toEqual({
              named: 'cats',
              numeric: 'cats',
              space: 'cats',
              escapedLetter: String.raw`\cats`,
              escapedStar: String.raw`\*cats`,
            },);
          },
        },),
        it({
          name: 'KEEPS A VARIATION SELECTOR WITH THE CHARACTER IT FOLLOWS, taking only the punctuation after it off a '
            + 'quote\'s end',
          fn: async () => {
            const heart = String.fromCodePoint(0x27_64, 0xFE_0F,);
            expect({
              emoji: quoteCore({ quote: `I love you ${heart}.`, },),
              astral: quoteCore({ quote: `Meow ${String.fromCodePoint(0x1_F6_3A,)}...`, },),
              astralKept: quoteCore({ quote: `Meow ${String.fromCodePoint(0x1_F6_3A,)}`, },),
            },).toEqual({
              emoji: `I love you ${heart}`,
              astral: `Meow ${String.fromCodePoint(0x1_F6_3A,)}`,
              astralKept: `Meow ${String.fromCodePoint(0x1_F6_3A,)}`,
            },);
          },
        },),
      ],
    },),

    describe({
      name: quoteBounds.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES WHERE THE WORDS BEGIN AND END IN UTF-16 UNITS, after a leading space and before the closing punctuation',
          fn: async () => {
            const cat = String.fromCodePoint(0x1_F6_3A,);
            expect([
              quoteBounds({ quote: ' the cat naps.', },),
              quoteBounds({ quote: `${cat} naps ${cat}!`, },),
              quoteBounds({ quote: '. ', },),
              quoteBounds({ quote: '', },),
            ],).toEqual([
              { start: 1, end: 13, },
              { start: 0, end: 10, },
              { start: 0, end: 0, },
              { start: 0, end: 0, },
            ],);
          },
        },),
        it({
          name: 'TAKES THE SPACE OFF A QUOTE\'S START as well as its end',
          fn: async () => {
            expect(quoteCore({ quote: '  \n the cat naps \n', },),).toBe('the cat naps',);
          },
        },),
      ],
    },),
  ],
},);
