/**
 Tests for where a located span begins and ends once a character reference the parser decodes or a backslash escape is kept whole:
 a judge who quotes the words up to the semicolon of a reference, up to the backslash that escapes a full stop, or from the `amp;` of
 a reference or the sign an escape escapes, names a span whose edge stands inside the reference or the escape, and the edge is taken
 to the end of that unit (`wholeEndOf`, `wholeStartOf`), which is then kept or cut at the edge as the character it shows would be
 (`wholeSpanOf`).

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
  wholeEndOf,
  wholeSpanOf,
  wholeStartOf,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: wholeEndOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TAKES THE SEMICOLON THAT ENDS A CHARACTER REFERENCE THE END FALLS INSIDE into the span',
          fn: async () => {
            expect([
              wholeEndOf({ text: 'The dogs &amp; bowl', end: 13, },),
              wholeEndOf({ text: 'The dogs&#39; bowl', end: 12, },),
              wholeEndOf({ text: 'The dogs&rsquo; bowl', end: 14, },),
              wholeEndOf({ text: '𝓬ats &#39; x', end: 10, },),
            ],).toEqual([14, 13, 15, 11,],);
          },
        },),
        it({
          name: 'TAKES THE PUNCTUATION A BACKSLASH ESCAPES into the span, when an odd number of backslashes stand before it',
          fn: async () => {
            expect([
              wholeEndOf({ text: String.raw`The cats\. bowl`, end: 9, },),
              wholeEndOf({ text: String.raw`The cats\? bowl`, end: 9, },),
              wholeEndOf({ text: String.raw`The cats\\\; bowl`, end: 11, },),
            ],).toEqual([10, 10, 12,],);
          },
        },),
        it({
          name: 'LEAVES THE END WHERE IT IS when it falls between two units: the next unit is a semicolon that ends no reference the '
            + 'parser decodes, a stop after an escaped backslash, a reference whose ampersand a backslash escapes, or the text ends',
          fn: async () => {
            expect([
              wholeEndOf({ text: 'The cats; bowl', end: 8, },),
              wholeEndOf({ text: 'The cats &; bowl', end: 10, },),
              wholeEndOf({ text: String.raw`The cats\\. bowl`, end: 10, },),
              wholeEndOf({ text: String.raw`The \&amp; bowl`, end: 9, },),
              wholeEndOf({ text: 'The cats', end: 8, },),
              wholeEndOf({ text: 'The cats. bowl', end: 8, },),
              wholeEndOf({ text: 'See Q&A; then', end: 7, },),
              wholeEndOf({ text: 'See a&zzzz; then', end: 10, },),
            ],).toEqual([8, 10, 10, 9, 8, 8, 7, 10,],);
          },
        },),
        it({
          name: 'TAKES THE END PAST THE REST OF A UNIT IT FALLS INSIDE, also when the unit began before the span: a reference the parser '
            + 'decodes, an escape of any ASCII punctuation, a reference to a character beyond the first plane',
          fn: async () => {
            expect([
              wholeEndOf({ text: 'The &amp; bowl', end: 8, },),
              wholeEndOf({ text: String.raw`The cats\) bowl`, end: 9, },),
              wholeEndOf({ text: String.raw`The cats\*x bowl`, end: 9, },),
              wholeEndOf({ text: 'The a&#128049; bowl', end: 9, },),
              wholeEndOf({ text: 'The a&period; bowl', end: 7, },),
            ],).toEqual([9, 10, 10, 14, 13,],);
          },
        },),
      ],
    },),
    describe({
      name: wholeStartOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TAKES THE START BACK TO THE BEGINNING OF A UNIT IT FALLS INSIDE: a quote that begins at the character an escape '
            + 'escapes, or inside a reference the parser decodes, names the whole unit',
          fn: async () => {
            expect([
              wholeStartOf({ text: String.raw`He said \*kittens* now`, start: 9, },),
              wholeStartOf({ text: 'See dogs&amp; cats now', start: 9, },),
              wholeStartOf({ text: 'See dogs&#39; cats now', start: 10, },),
              wholeStartOf({ text: String.raw`He said \\\*kittens`, start: 11, },),
            ],).toEqual([8, 8, 8, 10,],);
          },
        },),
        it({
          name: 'LEAVES THE START WHERE IT IS when it falls between two units: after a reference or an escape, before one, after a '
            + 'backslash that escapes nothing or a backslash that is itself escaped',
          fn: async () => {
            expect([
              wholeStartOf({ text: 'See dogs&amp; cats now', start: 13, },),
              wholeStartOf({ text: 'See dogs&amp; cats now', start: 8, },),
              wholeStartOf({ text: String.raw`He said \cats now`, start: 9, },),
              wholeStartOf({ text: String.raw`He said \\*cats now`, start: 10, },),
              wholeStartOf({ text: 'See Q&A; cats', start: 7, },),
              wholeStartOf({ text: 'See cats', start: 0, },),
            ],).toEqual([13, 8, 9, 10, 7, 0,],);
          },
        },),
      ],
    },),
    describe({
      name: wholeSpanOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS A SPAN WHOLE THAT NO EDGE CUTS IN TWO: whole words are the same span',
          fn: async () => {
            expect([
              wholeSpanOf({ text: 'The cats bowl', start: 4, end: 8, },),
              wholeSpanOf({ text: 'See Q&A; then', start: 4, end: 7, },),
              wholeSpanOf({ text: String.raw`He said (the cats\) was here`, start: 9, end: 17, },),
            ],).toEqual([
              { start: 4, end: 8, },
              { start: 4, end: 7, },
              { start: 9, end: 17, },
            ],);
          },
        },),
        it({
          name: 'CUTS A UNIT THE END FALLS INSIDE AS THE CHARACTER IT SHOWS WOULD BE CUT: an escaped stop or bracket goes with its '
            + 'backslash, so the span ends where the quote\'s own words end',
          fn: async () => {
            expect([
              wholeSpanOf({ text: String.raw`See cats\. Then`, start: 4, end: 9, },),
              wholeSpanOf({ text: String.raw`He said (the cats\) was here`, start: 9, end: 18, },),
              wholeSpanOf({ text: 'See cats&nbsp;then', start: 4, end: 11, },),
            ],).toEqual([
              { start: 4, end: 8, },
              { start: 9, end: 17, },
              { start: 4, end: 8, },
            ],);
          },
        },),
        it({
          name: 'KEEPS A UNIT THE EDGE FALLS INSIDE WHOLE WHEN THE CHARACTER IT SHOWS STAYS: a reference to an ampersand, an escaped '
            + 'asterisk',
          fn: async () => {
            expect([
              wholeSpanOf({ text: 'See dogs&amp; cats now', start: 9, end: 18, },),
              wholeSpanOf({ text: String.raw`He said \*kittens* now`, start: 9, end: 18, },),
              wholeSpanOf({ text: 'He said dogs&amp; cats now', start: 8, end: 16, },),
            ],).toEqual([
              { start: 8, end: 18, },
              { start: 8, end: 18, },
              { start: 8, end: 17, },
            ],);
          },
        },),
        it({
          name: 'CUTS A WHOLE REFERENCE TO A SINGLE MARK OFF THE START like the mark, and the space after it',
          fn: async () => {
            expect(wholeSpanOf({ text: 'See dogs&#39; cats now', start: 10, end: 18, },),).toEqual({ start: 14, end: 18, },);
          },
        },),
      ],
    },),
  ],
},);
