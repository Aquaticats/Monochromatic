/**
 Tests for the character references of an English that decode to a quotation-like character: the single marks the pass counts
 (the straight apostrophe, the opening and the closing curly single mark, the modifier letter apostrophe, the low and the
 reversed single mark, both single angle quotation marks), the double marks, both double angle quotation marks and the
 backtick. A page that holds one shows a reader a mark that no reader of the raw text sees.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { quoteReferencesIn, } from '../dist/final/node/index.mjs';

/**
 The characters a reference may decode to for the pass to refuse a gain beside it: the eight single marks, the five double
 marks, both double angle quotation marks and the backtick.
 */
const QUOTE_LIKE: readonly string[] = [
  '\'',
  '‘',
  '’',
  '\u{02BC}',
  '\u{201A}',
  '\u{201B}',
  '‹',
  '›',
  '"',
  '“',
  '”',
  '\u{201E}',
  '\u{201F}',
  '«',
  '»',
  '`',
];

/**
 The names of the HTML named references the Markdown parser decodes to one of those characters, read from the parser's own
 table (`decode-named-character-reference` over `character-entities`), which holds no other.
 */
const QUOTE_NAMES: readonly string[] = [
  'apos',
  'bdquo',
  'CloseCurlyDoubleQuote',
  'CloseCurlyQuote',
  'DiacriticalGrave',
  'grave',
  'laquo',
  'ldquo',
  'ldquor',
  'lsaquo',
  'lsquo',
  'lsquor',
  'OpenCurlyDoubleQuote',
  'OpenCurlyQuote',
  'quot',
  'QUOT',
  'raquo',
  'rdquo',
  'rdquor',
  'rsaquo',
  'rsquo',
  'rsquor',
  'sbquo',
];

/**
 The references of the English that stands around an entry, which the entry is put into.

 @param entry - what stands between two words of a sentence

 @returns References the pass names in that sentence

 @example
 ```ts
 const seen = referencesAround({ entry: '&#39;', },); // ['&#39;']
 ```
 */
function referencesAround({ entry, }: { readonly entry: string; },): readonly string[] {
  return quoteReferencesIn({ english: `The kittens${entry} bowl is empty.`, },);
}

await describe({
  name: quoteReferencesIn.name,
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'NAMES EVERY NAMED REFERENCE THE PARSER DECODES TO A QUOTATION-LIKE CHARACTER, as the English writes it',
      fn: async () => {
        expect(QUOTE_NAMES.map(function namedIn(name,): readonly string[] {
          return referencesAround({ entry: `&${name};`, },);
        },),).toEqual(QUOTE_NAMES.map(function written(name,): readonly string[] {
          return [`&${name};`,];
        },),);
      },
    },),
    it({
      name: 'NAMES A DECIMAL OR HEXADECIMAL REFERENCE TO EACH QUOTATION-LIKE CHARACTER, in lower or upper case, with leading zeros',
      fn: async () => {
        /**
         The spellings of the references to one character.
         */
        const spellings = QUOTE_LIKE.map(function spellingsOf(character,): readonly string[] {
          /**
           The code point of the character.
           */
          const code = nonNullishOrThrow(character.codePointAt(0,),);
          /**
           The code point in hexadecimal.
           */
          const hexadecimal = code.toString(16,);
          return [
            `&#${String(code,)};`,
            `&#x${hexadecimal};`,
            `&#X${hexadecimal.toUpperCase()};`,
            `&#${String(code,).padStart(7, '0',)};`,
            `&#x${hexadecimal.padStart(6, '0',)};`,
          ];
        },);
        expect(spellings.map(function seenIn(entries,): readonly (readonly string[])[] {
          return entries.map(function seenFor(entry,): readonly string[] {
            return referencesAround({ entry, },);
          },);
        },),).toEqual(spellings.map(function expectedFor(entries,): readonly (readonly string[])[] {
          return entries.map(function expectedOne(entry,): readonly string[] {
            return [entry,];
          },);
        },),);
      },
    },),
    it({
      name: 'NAMES NOTHING FOR A REFERENCE TO ANOTHER CHARACTER, an ampersand, a no-break space, a copyright sign, an ellipsis, '
        + 'a dash, an angle bracket',
      fn: async () => {
        expect([
          '&amp;',
          '&nbsp;',
          '&#160;',
          '&#xA0;',
          '&#38;',
          '&#x26;',
          '&copy;',
          '&hellip;',
          '&#8230;',
          '&mdash;',
          '&lt;',
          '&gt;',
        ].map(function seenFor(entry,): readonly string[] {
          return referencesAround({ entry, },);
        },),).toEqual(Array.from(
          { length: 12, },
          function none(): readonly string[] {
            return [];
          },
        ),);
      },
    },),
    it({
      name: 'NAMES NOTHING FOR TEXT THAT IS NO REFERENCE, a name with no semicolon, a name of another case, a number with no digits, '
        + 'more digits than the parser reads, a space inside, and an ampersand beside a mark',
      fn: async () => {
        expect([
          'The kittens&apos bowl.',
          'The kittens&#39 bowl.',
          'The kittens&Rsquo; bowl.',
          'The kittens&RSQUO; bowl.',
          'The kittens&#; bowl.',
          'The kittens&#x; bowl.',
          'The kittens&#xZ; bowl.',
          'The kittens&; bowl.',
          'The kittens&#00000039; bowl.',
          'The kittens&#x0000027; bowl.',
          'The kittens& rsquo; bowl.',
          'The kittens&&rsquo bowl.',
          'Tom & Jerry’s bowl.',
        ].map(function seenFor(english,): readonly string[] {
          return quoteReferencesIn({ english, },);
        },),).toEqual(Array.from(
          { length: 13, },
          function none(): readonly string[] {
            return [];
          },
        ),);
      },
    },),
    it({
      name: 'NAMES EVERY REFERENCE IN ORDER, with the one that is repeated, and the one among the rest that is a mark',
      fn: async () => {
        expect([
          quoteReferencesIn({ english: 'A &#39;cat&#39; &amp; a &rsquo;dog&nbsp;&lsquo;.', },),
          quoteReferencesIn({ english: '', },),
        ],).toEqual([['&#39;', '&#39;', '&rsquo;', '&lsquo;',], [],],);
      },
    },),
    it({
      name: 'NAMES A REFERENCE THE PAGE SHOWS AS TEXT, in a paragraph, an emphasis, a link\'s label and a heading, and across the '
        + 'slices of a page',
      fn: async () => {
        expect([
          quoteReferencesIn({ english: 'The *kittens&#39;* bowl.', },),
          quoteReferencesIn({ english: 'The [kittens&rsquo;](https://cats.example/k) bowl.', },),
          quoteReferencesIn({ english: '## The kittens&lsquo;\n\nThe bowl.', },),
          quoteReferencesIn({ english: 'The bowl is full.\n\nThe kittens&#x2019; nap.', },),
        ],).toEqual([['&#39;',], ['&rsquo;',], ['&lsquo;',], ['&#x2019;',],],);
      },
    },),
    it({
      name: 'NAMES NO REFERENCE THE PAGE DOES NOT SHOW AS TEXT, one in inline code, a code block, a tag\'s attribute, a link\'s '
        + 'destination and title, and a comment',
      fn: async () => {
        expect([
          quoteReferencesIn({ english: 'The `kittens&#39;` bowl.', },),
          quoteReferencesIn({ english: 'The bowl.\n\n```\nkittens&#39;\n```\n', },),
          quoteReferencesIn({ english: 'The <a title="&quot;">kittens</a> bowl.', },),
          quoteReferencesIn({ english: 'The [kittens](https://cats.example/k?q=&quot;) bowl.', },),
          quoteReferencesIn({ english: 'The [kittens](https://cats.example/k "&quot;x") bowl.', },),
          quoteReferencesIn({ english: 'The kittens<!-- &#39; --> bowl.', },),
        ],).toEqual([[], [], [], [], [], [],],);
      },
    },),
    it({
      name: 'NAMES NO REFERENCE WHOSE AMPERSAND A BACKSLASH ESCAPES, which the page shows as the text it is written, and names the '
        + 'reference after an escaped backslash, which escapes nothing',
      fn: async () => {
        expect([
          quoteReferencesIn({ english: String.raw`kittens\&#39; x`, },),
          quoteReferencesIn({ english: String.raw`kittens\&rsquo; x`, },),
          quoteReferencesIn({ english: String.raw`kittens\\&#39; x`, },),
          quoteReferencesIn({ english: String.raw`kittens\\\&#39; x`, },),
          quoteReferencesIn({ english: String.raw`kittens \&#39; and &#39; x`, },),
          quoteReferencesIn({ english: String.raw`kittens\ &#39; x`, },),
        ],).toEqual([[], [], ['&#39;',], [], ['&#39;',], ['&#39;',],],);
      },
    },),
    it({
      name: 'NAMES THE REFERENCE AFTER A COMMENT THAT A SECOND OPENER ENDS EARLY, because the page parser\'s mask ends the comment at '
        + 'the dashes the opener shares with its closer, and names none inside a comment that closes',
      fn: async () => {
        expect([
          quoteReferencesIn({ english: 'x <!-- y <!--> &#39; z --> w.', },),
          quoteReferencesIn({ english: 'x <!-- y <!--> z --> &#39; w.', },),
          quoteReferencesIn({ english: 'x <!-- y &#39; z --> w.', },),
          quoteReferencesIn({ english: 'x <!-- y <!-- &#39; --> w &#39;.', },),
        ],).toEqual([['&#39;',], ['&#39;',], [], ['&#39;',],],);
      },
    },),
  ],
},);
