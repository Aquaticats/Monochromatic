/**
 Tests for the word test at the edge of a located span (n156, W2). The test reads the text a reader sees: a backslash escape or a
 character reference is its shown character, a default-ignorable character is transparent, and an ampersand between two word
 characters joins them as a hyphen does. It runs on the final edges, after a quote that began inside a unit was widened to the
 whole unit.

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

 @param needle - words the judge quoted, whose first and last characters say which edges are word edges, the quote itself where absent

 @returns Whether an edge of the span stands inside a shown word

 @example
 ```ts
 spanCuts({ text: 'See k&#105;ttens now', quote: 'ttens', },); // true
 ```
 */
function spanCuts(
  {
    text,
    quote,
    needle = quote,
  }: {
    readonly text: string;
    readonly quote: string;
    readonly needle?: string;
  },
): boolean {
  /**
   Where the span starts.
   */
  const start = text.indexOf(quote,);
  return cutsShownWord({
    text,
    start,
    end: start + quote.length,
    needle,
  },);
}

await describe({
  name: cutsShownWord.name,
  children: [
    it({
      name: 'CUTS A WORD WHEN THE NEIGHBOUR IS A UNIT THAT SHOWS A LETTER: a numeric reference before the start and after the end',
      fn: async () => {
        expect([
          spanCuts({ text: 'See k&#105;ttens now', quote: 'ttens', },),
          spanCuts({ text: 'See kitten&#115; now', quote: 'kitten', },),
          spanCuts({ text: 'See kitten&#115; now', quote: 'kitten&#115;', },),
        ],).toEqual([true, true, false,],);
      },
    },),
    it({
      name: 'READS A DEFAULT-IGNORABLE CHARACTER AS NOT THERE: a soft hyphen reference or a zero-width space beside the span lets the '
        + 'letter beyond it join the word',
      fn: async () => {
        expect([
          spanCuts({ text: 'See kit&shy;tens now', quote: 'tens', },),
          spanCuts({ text: 'See kitt&shy;ens now', quote: 'kitt', },),
          spanCuts({ text: 'See kit\u{200B}tens now', quote: 'tens', },),
          spanCuts({ text: 'See kitt\u{200B}ens now', quote: 'kitt', },),
        ],).toEqual([true, true, true, true,],);
      },
    },),
    it({
      name: 'JOINS TWO WORD CHARACTERS THROUGH AN AMPERSAND, a plain one and a reference that shows one, as a hyphen does',
      fn: async () => {
        expect([
          spanCuts({ text: 'Call AT&amp;T now', quote: 'AT', },),
          spanCuts({ text: 'Call AT&amp;T now', quote: 'T now', },),
          spanCuts({ text: 'Our R&D team', quote: 'R', },),
          spanCuts({ text: 'Our R&D team', quote: 'D team', },),
        ],).toEqual([true, true, true, true,],);
      },
    },),
    it({
      name: 'CUTS A WORD WHEN A QUOTE THAT BEGAN INSIDE A REFERENCE WAS WIDENED TO IT, since its first word character was a letter '
        + 'and the shown character before the whole reference is a letter',
      fn: async () => {
        expect([
          spanCuts({ text: 'See dogs&amp; cats now', quote: '&amp; cats', needle: 'amp; cats', },),
          spanCuts({ text: 'See Tom&amp; Jerry now', quote: '&amp; Jerry', needle: 'amp; Jerry', },),
          spanCuts({ text: 'Call AT&amp;T now', quote: '&amp;T now', needle: 'amp;T now', },),
        ],).toEqual([true, true, true,],);
      },
    },),
    it({
      name: 'CUTS NO WORD AT WHOLE WORDS: a plain word, a word with an ampersand inside, a span that ends in the reference of an '
        + 'ampersand beside a space, a span that ends before a no-break space, and words beside punctuation or Han text',
      fn: async () => {
        expect([
          spanCuts({ text: 'See kittens now', quote: 'kittens', },),
          spanCuts({ text: 'Call AT&amp;T now', quote: 'AT&amp;T', },),
          spanCuts({ text: 'See Tom &amp; Jerry now', quote: 'Tom &amp;', needle: 'Tom &', },),
          spanCuts({ text: 'See kittens&nbsp;more', quote: 'kittens', },),
          spanCuts({ text: 'See (kittens), now', quote: 'kittens', },),
          spanCuts({ text: '看kittens好', quote: 'kittens', },),
        ],).toEqual([false, false, false, false, false, false,],);
      },
    },),
    it({
      name: 'CUTS NO WORD AT AN EDGE THAT IS NO LATIN WORD EDGE OF THE QUOTE, so a quote that ends in punctuation or begins with a '
        + 'mark is not tested there',
      fn: async () => {
        expect([
          spanCuts({ text: 'See dogs&amp; now', quote: '&amp;', needle: '&', },),
          spanCuts({ text: 'See kittens,&#115; now', quote: 'kittens,', },),
        ],).toEqual([false, false,],);
      },
    },),
    it({
      name: 'READS THE MARKUP THAT SHOWS NOTHING AS NOT THERE (n159, X1): emphasis, strong, a link label, a tag and a code span between '
        + 'two pieces of one word let the letters on both sides join',
      fn: async () => {
        expect([
          spanCuts({ text: 'The kit*tens* ate.', quote: 'kit', },),
          spanCuts({ text: 'The kit*tens* ate.', quote: 'tens', },),
          spanCuts({ text: 'The **kit**tens ate.', quote: 'kit', },),
          spanCuts({ text: 'The [kit](u)tens ate.', quote: 'kit', },),
          spanCuts({ text: 'The <b>kit</b>tens ate.', quote: 'kit', },),
          spanCuts({ text: 'The dogs*s* bowl.', quote: 'dogs', },),
          spanCuts({ text: 'The kit<b class="x">tens</b> ate.', quote: 'kit', },),
          spanCuts({ text: 'The kit<!-- note -->tens ate.', quote: 'kit', },),
          spanCuts({ text: 'The *kitten*\'s bowl.', quote: 'kitten', },),
        ],).toEqual([true, true, true, true, true, true, true, true, true,],);
      },
    },),
    it({
      name: 'CUTS NO WORD AT A WHOLE WORD BESIDE EACH DELIMITER (n159, X1): the word stands apart from what is beside the markup',
      fn: async () => {
        expect([
          spanCuts({ text: 'The *kittens* ate.', quote: 'kittens', },),
          spanCuts({ text: 'The **kittens** ate.', quote: 'kittens', },),
          spanCuts({ text: 'The [kittens](u) ate.', quote: 'kittens', },),
          spanCuts({ text: 'The <b>kittens</b> ate.', quote: 'kittens', },),
          spanCuts({ text: 'A *dogs*, kittens ate.', quote: 'dogs', },),
          spanCuts({ text: 'The kit\n\ntens ate.', quote: 'kit', },),
          spanCuts({ text: 'The kit\n- tens ate.', quote: 'kit', },),
        ],).toEqual([false, false, false, false, false, false, false,],);
      },
    },),
    it({
      name: 'READS A LETTER, MARK OR DIGIT OF ANY SCRIPT AS A WORD CHARACTER AT AN EDGE (n159, X9), and an underscore between two of '
        + 'them joins them',
      fn: async () => {
        expect([
          spanCuts({ text: 'Привет мир', quote: 'Прив', },),
          spanCuts({ text: 'Γειά σου', quote: 'Γει', },),
          spanCuts({ text: 'dogs_cats ate', quote: 'dogs', },),
          spanCuts({ text: 'snake_case ate', quote: 'case', },),
          spanCuts({ text: 'שלום עולם', quote: 'של', },),
          spanCuts({ text: 'x١٢٣ y', quote: 'x', },),
        ],).toEqual([true, true, true, true, true, true,],);
      },
    },),
    it({
      name: 'CUTS NO WORD AT WHOLE WORDS OF OTHER SCRIPTS OR AT A LONE UNDERSCORE (n159, X9)',
      fn: async () => {
        expect([
          spanCuts({ text: 'Привет мир', quote: 'Привет', },),
          spanCuts({ text: 'Γειά σου', quote: 'Γειά', },),
          spanCuts({ text: 'dogs _ cats ate', quote: 'dogs', },),
          spanCuts({ text: 'See _dogs_ ate', quote: 'dogs', },),
          spanCuts({ text: 'See dogs_ ate', quote: 'dogs', },),
          spanCuts({ text: 'x² y', quote: 'x', },),
        ],).toEqual([false, false, false, false, false, false,],);
      },
    },),
    it({
      name: 'READS AN IMAGE WITH NO ALT TEXT AS NOTHING, so the word before it and the apostrophe after it are one word as in the text '
        + 'with no image there, and an image with an alt of only spaces or a word as a boundary as before (n169)',
      fn: async () => {
        /**
         Whether the first `kit` of a text is cut at its end, with the page's definitions given.

         @param text - English
         @param definitions - definitions of the page
         @returns Whether the stand cuts a shown word
         */
        function cutAtFirst(
          { text, definitions = '', }: { readonly text: string; readonly definitions?: string; },
        ): boolean {
          return cutsShownWord({
            text,
            start: 0,
            end: 3,
            needle: 'kit',
            definitions,
          },);
        }
        expect([
          cutAtFirst({ text: 'kit\'kit', },),
          cutAtFirst({ text: 'kit![](u)\'kit', },),
          cutAtFirst({ text: 'kit![][r]\'kit\n\n[r]: u', },),
          cutAtFirst({ text: 'kit![](u)![](v)\'kit', },),
          cutAtFirst({ text: 'kit![ ](u)\'kit', },),
          cutAtFirst({ text: 'kit![alt](u)\'kit', },),
          cutAtFirst({ text: 'kit![alt][r]\'kit\n\n[r]: u', },),
          cutAtFirst({ text: 'kit \'kit', },),
          cutAtFirst({ text: 'kit![](u) kit', },),
        ],).toEqual([true, true, true, true, false, false, false, false, false,],);
      },
    },),
    it({
      name: 'READS A LINK WHOSE DESTINATION HOLDS A LINE BREAK AS THE GRAMMAR DOES, the break inside the link hidden with the rest of '
        + 'its destination, and a line break between words as a break (n169)',
      fn: async () => {
        /**
         Whether a span of a text cuts a shown word, the span found by the quote.
         */
        function cutAt(
          { text, quote, }: { readonly text: string; readonly quote: string; },
        ): boolean {
          return spanCuts({
            text,
            quote,
          },);
        }
        expect([
          cutAt({ text: 'kit\'kit', quote: 'kit', },),
          cutAt({ text: 'kit[](u\n)\'kit', quote: 'kit', },),
          cutAt({ text: 'kit[](u\nv)\'kit', quote: 'kit', },),
          cutAt({ text: 'kit![](u\n)\'kit', quote: 'kit', },),
          cutAt({ text: '[kit](u\n)tens', quote: 'kit', },),
          cutAt({ text: '[kit](u\n "t\nt")tens', quote: 'kit', },),
          cutAt({ text: 'kit [](u\n) kit', quote: 'kit', },),
          cutAt({ text: 'kit\n\'kit', quote: 'kit', },),
          cutAt({ text: 'kit\n\ntens', quote: 'kit', },),
          cutAt({ text: '[kit](u)\n\ntens', quote: 'kit', },),
        ],).toEqual([true, true, false, true, true, true, false, false, false, false,],);
      },
    },),
  ],
},);
