/**
 Tests for what the word test at the edge of a located span reads as shown, hidden or a boundary (n160 findings Y1, Y2 and Y4).

 A hidden stretch (a link's destination and title, a tag's attributes, a comment) hides every unit in it, whether the unit is one
 character, a reference, an escape or an astral character of two UTF-16 units. Markup that separates or annotates (a line break
 with or without a slash, a block tag, a table cell pipe and its padding, an image, the base and the annotation of a ruby) is a
 word boundary, while a tag that only wraps text stays transparent. The content of inline code is shown as the code word it is.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { cutsShownWord, } from '../dist/final/node/index.mjs';

/**
 Whether the span the quote names in a text cuts a word a reader sees.

 @param text - English as written
 @param quote - written characters of the span, found once in the text
 @returns Whether an edge of the span stands inside a shown word

 @example
 ```ts
 spanCuts({ text: 'See k&#105;ttens now', quote: 'ttens', },); // true
 ```
 */
function spanCuts({ text, quote, }: { readonly text: string; readonly quote: string; },): boolean {
  /**
   Where the span starts.
   */
  const start = text.indexOf(quote,);
  /**
   Whether the quote stands a second time after the first.
   */
  const repeats = text.includes(
    quote,
    start + 1,
  );
  if ((!text.includes(quote,)) || repeats)
    throw new Error(`the quote ${JSON.stringify(quote,)} must stand exactly once in ${JSON.stringify(text,)}`,);
  return cutsShownWord({
    text,
    start,
    end: start + quote.length,
    needle: quote,
  },);
}

/**
 A row: the English, the words quoted, and whether the span cuts a shown word.
 */
type Row = readonly [text: string, quote: string, cuts: boolean,];

/**
 Rows that Y1 names: the units of a hidden stretch are all hidden.
 */
const HIDDEN_STRETCH_ROWS: readonly Row[] = [
  ['foo [kit](u "a&#116;b") ate', 'kit', false,],
  ['foo <a href="x&#116;y">kit</a> ate', 'kit', false,],
  [String.raw`foo [kit](u\_x) ate`, 'kit', false,],
  ['The [kit](u "𝓠") ate', 'kit', false,],
  ['The [kit](https://x.example/𝓠𝓾𝓪𝓷) ate', 'kit', false,],
  ['The <a title="𝓠">kit</a> ate', 'kit', false,],
  // The emoji of a hidden title or comment shows a reader nothing, so the word runs on through it.
  ['The [kit](u "🐱")tens ate', 'kit', true,],
  ['The kit<!-- 🐱 -->tens ate', 'kit', true,],
  // Controls that already held: ASCII in the same places.
  ['The [kit](u "x")tens ate', 'kit', true,],
  ['The [kit](https://x.example/Q) ate', 'kit', false,],
];

/**
 Rows that Y2 names: separating and annotating markup is a word boundary.
 */
const BOUNDARY_ROWS: readonly Row[] = [
  ['cats<br/>dogs', 'dogs', false,],
  ['cats<br/>dogs', 'cats', false,],
  ['cats<br />dogs', 'cats', false,],
  ['cats<br>dogs', 'cats', false,],
  ['cats![pic](u)dogs', 'cats', false,],
  ['cats![pic](u)dogs', 'dogs', false,],
  ['<ruby>kan<rt>ji</rt></ruby>', 'ji', false,],
  ['<ruby>kan<rt>ji</rt></ruby>', 'kan', false,],
  ['<ruby>reload a save file<rt>OVERLOAD</rt></ruby>', 'OVERLOAD', false,],
  ['<ruby>reload a save file<rt>OVERLOAD</rt></ruby>', 'file', false,],
  ['<ruby>kan<rp>(</rp><rt>ji</rt><rp>)</rp></ruby>', 'kan', false,],
  ['| cats | dogs |\n| --- | --- |\n| fish | birds |', 'dogs', false,],
  ['| cats | dogs |\n| --- | --- |\n| fish | birds |', 'cats', false,],
  ['<p>The kit</p><p>tens ate</p>', 'kit', false,],
  ['<p>The kit</p><p>tens ate</p>', 'tens', false,],
  ['<div>cats</div><div>dogs</div>', 'dogs', false,],
  ['<ul><li>cats</li><li>dogs</li></ul>', 'dogs', false,],
  ['<ol><li>cats</li><li>dogs</li></ol>', 'cats', false,],
  ['<table><tr><td>cats</td><td>dogs</td></tr></table>', 'cats', false,],
  ['<table><tr><th>cats</th><th>dogs</th></tr></table>', 'dogs', false,],
  ['<details><summary>The kit</summary>tens ate</details>', 'kit', false,],
  ['<h1>cats</h1><h2>dogs</h2>', 'dogs', false,],
  ['<blockquote>cats</blockquote>dogs', 'dogs', false,],
];

/**
 Rows that Y2 says stay transparent: a tag that only wraps text.
 */
const WRAPPING_ROWS: readonly Row[] = [
  ['<b>kit</b>tens', 'kit', true,],
  ['<i>kit</i>tens', 'kit', true,],
  ['<em>kit</em>tens', 'kit', true,],
  ['<strong>kit</strong>tens', 'kit', true,],
  ['<span>kit</span>tens', 'kit', true,],
  ['<a href="u">kit</a>tens', 'kit', true,],
  ['<u>kit</u>tens', 'kit', true,],
  ['<s>kit</s>tens', 'kit', true,],
  ['<sub>kit</sub>tens', 'kit', true,],
  ['<sup>kit</sup>tens', 'kit', true,],
  ['kit*tens*', 'kit', true,],
  ['**kit**tens', 'kit', true,],
  ['[kit](u)tens', 'kit', true,],
];

/**
 Rows that Y4 names: inline code beside a text word reads as the code word it shows.
 */
const INLINE_CODE_ROWS: readonly Row[] = [
  ['The `kit`tens ate.', 'tens', true,],
  ['The kit`tens` ate.', 'kit', true,],
  ['The `kit` ate.', 'kit', false,],
  ['The `kit` tens ate.', 'tens', false,],
  ['The kit `tens` ate.', 'kit', false,],
];

/**
 Rows that Z6 names: inline code padded by one space on each side (content not all spaces) shows the code word without the padding,
 as CommonMark strips it, so the code word runs on into the text beside it. A space on one side only, a space beside spaces only and
 a second space on each side are shown, and stand between the words.
 */
const PADDED_CODE_ROWS: readonly Row[] = [
  ['The ` kit `tens ate.', 'tens', true,],
  ['The kit` tens ` ate.', 'kit', true,],
  ['The `` kit ``tens ate.', 'tens', true,],
  ['The kit`` tens `` ate.', 'kit', true,],
  // Controls: the padding is kept where CommonMark keeps it.
  ['The ` kit`tens ate.', 'tens', true,],
  ['The kit` tens` ate.', 'kit', false,],
  ['The `kit `tens ate.', 'tens', false,],
  ['The `  `tens ate.', 'tens', false,],
  ['The `  kit  `tens ate.', 'tens', false,],
];

/**
 Rows that Z7 names: the four raw elements (`pre`, `textarea`, `script`, `style`) end a word at each of their tags.
 */
const RAW_ELEMENT_ROWS: readonly Row[] = [
  ['cats<pre>dogs</pre>', 'cats', false,],
  ['cats<pre>dogs</pre>', 'dogs', false,],
  ['cats<textarea></textarea>dogs', 'cats', false,],
  ['cats<textarea></textarea>dogs', 'dogs', false,],
  ['cats<script></script>dogs', 'cats', false,],
  ['cats<script></script>dogs', 'dogs', false,],
  ['cats<style></style>dogs', 'cats', false,],
  ['cats<style></style>dogs', 'dogs', false,],
  ['cats<PRE>dogs</PRE>', 'cats', false,],
  ['cats<pre class="x">dogs</pre>', 'dogs', false,],
];

await describe({
  name: 'the shown-text word test, hidden stretches, boundaries and inline code',
  concurrency: 1,
  children: ([
    ['Y1: every unit of a hidden stretch is hidden', HIDDEN_STRETCH_ROWS,],
    ['Y2: separating and annotating markup is a word boundary', BOUNDARY_ROWS,],
    ['Y2: a tag that wraps text stays transparent', WRAPPING_ROWS,],
    ['Y4: the content of inline code is shown', INLINE_CODE_ROWS,],
    ['Z6: the one padding space of inline code is not shown', PADDED_CODE_ROWS,],
    ['Z7: the tags of a raw element are a word boundary', RAW_ELEMENT_ROWS,],
  ] as const).map(function groupOf([name, rows,],) {
    return describe({
      name,
      children: rows.map(function testOf([text, quote, cuts,],) {
        return it({
          name: `${cuts ? 'CUTS' : 'DOES NOT CUT'} the word at ${JSON.stringify(quote,)} in ${JSON.stringify(text,)}`,
          fn: async () => {
            expect(spanCuts({ text, quote, },),).toBe(cuts,);
          },
        },);
      },),
    },);
  },),
},);
