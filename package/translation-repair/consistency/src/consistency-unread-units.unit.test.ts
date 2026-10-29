/**
 Tests for the units of an English that no reader reads as words: everything the prose view blanks, and the units of an HTML
 comment and of a character reference, which the prose keeps as the units of a text node. White space is never one of them.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { maskHtmlComments, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  commentUnitsOf,
  unreadUnitsOf,
} from '../dist/final/node/index.mjs';

/**
 The units of the comments of an English as the page parser's own mask names them (`maskHtmlComments`, the mask the parser blanks
 the comments with before it reads the page): the stretch from each opener to the first closer that starts after the opener's
 four characters, scanned without overlap, so an opener inside a comment, or the dashes an opener shares with the closer
 before it, start no comment of their own. A comment that never closes is the mask's too, and its tail is blanked by the prose
 view, so it is left out here. White space is never one of the units.

 @param english - English

 @returns Offsets of the units of the stretches that are no white space, in ascending order

 @example
 ```ts
 const units = stretchesOf({ english: 'a<!-- b -->c', },); // [1, 2, 3, 4, 6, 8, 9, 10]
 ```
 */
function stretchesOf({ english, }: { readonly english: string; },): readonly number[] {
  /**
   The offsets, in the order they were found.
   */
  const found: number[] = [];
  for (const region of maskHtmlComments({ text: english, },).regions) {
    if (!region.terminated)
      continue;
    for (let at = region.startOffset; at < region.endOffset; at += 1) {
      if (![' ', '\t', '\n', '\r',].includes(english.charAt(at,),))
        found.push(at,);
    }
  }
  return found;
}

/**
 The units of the comments of an English as the code names them, in ascending order.

 @param english - English

 @returns Offsets

 @example
 ```ts
 const units = commentsOf({ english: 'a<!-- b -->c', },); // [1, 2, 3, 4, 6, 8, 9, 10]
 ```
 */
function commentsOf({ english, }: { readonly english: string; },): readonly number[] {
  return [...commentUnitsOf({ english, },),].toSorted(function ascending(left, right,): number {
    return left - right;
  },);
}

/**
 An English with each unit a reader does not read as words shown as a middle dot, so a case reads as the text it is about.
 White space is never shown that way: a line break ends the stretches a run is read over.

 @param english - English of a slice

 @returns English with those units replaced

 @example
 ```ts
 const shown = unreadShown({ english: 'The <b>kittens</b>’ bowl.', },); // 'The ···kittens····’ bowl.'
 ```
 */
function unreadShown({ english, }: { readonly english: string; },): string {
  /**
   The offsets of the units not read as words.
   */
  const unread = unreadUnitsOf({ english, },);
  return Array.from(
    { length: english.length, },
    function unitShown(_, at,): string {
      return unread.has(at,) ? '·' : english.charAt(at,);
    },
  )
    .join('',);
}

/**
 The suite of the units the prose blanks.
 */
const unreadUnitsSuite = describe({
  name: unreadUnitsOf.name,
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'NAMES THE UNITS THE PROSE BLANKS, a footnote reference and its definition marker, a tag, a reference link\'s label, '
        + 'a link\'s destination with its title, a line break element, the delimiters of emphasis and code',
      fn: async () => {
        expect([
          unreadShown({ english: 'The kittens[^1]’ bowl.\n\n[^1]: A note.', },),
          unreadShown({ english: 'The <b>kittens</b>’ bowl.', },),
          unreadShown({ english: 'The [kittens][1]’ bowl.\n\n[1]: https://cats.example/k', },),
          unreadShown({ english: 'The [kittens](https://cats.example/k "the cats")’ bowl.', },),
          unreadShown({ english: 'The kittens<br/>’ bowl.', },),
          unreadShown({ english: 'Fill *kittens* and `it’s` now.', },),
        ],).toEqual([
          'The kittens····’ bowl.\n\n····· A note.',
          'The ···kittens····’ bowl.',
          `The ·kittens····’ bowl.\n\n···· ${'·'.repeat(22,)}`,
          `The ·kittens${'·'.repeat(24,)} ${'·'.repeat(4,)} ${'·'.repeat(6,)}’ bowl.`,
          'The kittens·····’ bowl.',
          'Fill ·kittens· and ······ now.',
        ],);
      },
    },),
    it({
      name: 'NAMES AN HTML COMMENT AND A CHARACTER REFERENCE, which the prose keeps as the units of a text node, and a '
        + 'comment that runs over a line break, whose line break is no unread unit',
      fn: async () => {
        expect([
          unreadShown({ english: 'The kittens<!-- c -->’ bowl.', },),
          unreadShown({ english: 'The kittens&nbsp;’ and &#160;&amp; bowl.', },),
          unreadShown({ english: 'a<!-- x\ny -->b', },),
          unreadShown({ english: 'The {kittens&nbsp;’ bowl.', },),
          unreadShown({ english: 'The kittens&#xA0;’ and &#XA0; bowl.', },),
        ],).toEqual([
          'The kittens···· · ···’ bowl.',
          `The kittens${'·'.repeat(6,)}’ and ${'·'.repeat(11,)} bowl.`,
          'a···· ·\n· ···b',
          `The {kittens${'·'.repeat(6,)}’ bowl.`,
          `The kittens${'·'.repeat(6,)}’ and ${'·'.repeat(6,)} bowl.`,
        ],);
      },
    },),
    it({
      name: 'NAMES NOTHING IN PLAIN PROSE, an ampersand with no reference after it, or the white space between words, so a '
        + 'unit is unread only when the page does not show it as a word',
      fn: async () => {
        expect([
          unreadShown({ english: 'The cats’ bowl, said Mio.', },),
          unreadShown({ english: 'Tom & Jerry and &nope bowl; ok.', },),
          unreadShown({ english: '', },),
        ],).toEqual(['The cats’ bowl, said Mio.', 'Tom & Jerry and &nope bowl; ok.', '',],);
      },
    },),
  ],
},);

/**
 The suite of the units of the HTML comments.
 */
const commentUnitsSuite = describe({
  name: commentUnitsOf.name,
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'NAMES THE STRETCHES OF THE PARSER\'S OWN MASK, the units of many openers that share one closer once, and no stretch '
        + 'for the dashes an opener shares with the closer before it (`<!-->`, `<!--->`)',
      fn: async () => {
        /**
         The Englishes asked about: many openers and one closer, with white space and a line break inside, a closer that comes
         before the opener it would overlap, the openers `<!-->` and `<!--->` that share dashes with a closer, an opener with
         no closer, and none.
         */
        const englishes = [
          `${'<!--'.repeat(50,)}-->`,
          `a${'<!--'.repeat(30,)} b -->c`,
          'a<!-- b <!-- c --> d <!-- e',
          '<!-- a --> x <!-- b --> y <!-- c',
          '<!---->x<!--->y-->',
          '<!--<!-->-->',
          'a<!-- x\ny <!-- z\n -->b',
          '<!-- a --><!-- b --><!-- c -->',
          '<!-- a <!-- b --> c <!-- d <!-- e --> f -->',
          '-->a<!--',
          '<!--',
          'plain',
          '',
          'x <!-- y <!--> &#39; z --> w.',
          '<!-- a <!--> b -->c<!--->d-->e',
          'a<!--><!-- b -->',
        ];
        expect(englishes.map(function commentsFor(english,): readonly number[] {
          return commentsOf({ english, },);
        },),).toEqual(englishes.map(function stretchesFor(english,): readonly number[] {
          return stretchesOf({ english, },);
        },),);
        expect(commentsOf({ english: englishes[0] ?? '', },).length,).toBe(203,);
      },
    },),
  ],
},);

await describe({
  name: '',
  concurrency: 1,
  children: [
    unreadUnitsSuite,
    commentUnitsSuite,
  ],
},);
