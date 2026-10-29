/**
 Tests that the typography restoration reads the pairing context of a mark as a reader sees it (n159, X7 and X4) and pairs a
 quoted two-digit number only when that leaves fewer marks unpaired (n159).

 A unit the prose view does not read as words (an emphasis, strike or code delimiter, a link bracket with its destination and
 title, a tag, a comment, an expression) is transparent for what stands beside a mark: `<b>'hi'</b>` opens at the start of the
 text and closes at its end, and the `'n'` of `rock *'n'* roll` is the elision pair it is without the asterisks. A fragment an
 editor rewrote takes its prose from the slice it lands in with the fragment in place, never from the fragment alone.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { restoreTypography, } from '../dist/final/node/index.mjs';

/**
 A page that writes curly apostrophes and curly double quotes.
 */
const CURLY_PAGE = 'The cat\u{2019}s \u{201C}bowl\u{201D} was full. The dogs\u{2019} bowl too.';

/**
 A text and the text the restoration writes for it on a curly page.
 */
type Row = {
  /**
   What the row is about.
   */
  readonly name: string;

  /**
   The replacement the editor wrote.
   */
  readonly input: string;

  /**
   The replacement as the restoration writes it.
   */
  readonly expected: string;
};

/**
 What the restoration writes for a replacement on the curly page.

 @param replacement - text the editor wrote

 @returns Restored text

 @example
 ```ts
 const out = curled({ replacement: 'rock \'n\' roll', },);
 ```
 */
function curled({ replacement, }: { readonly replacement: string; },): string {
  return restoreTypography({
    replacement,
    replaced: '',
    convention: CURLY_PAGE,
  },);
}

/**
 Texts where markup that shows nothing stands beside a straight mark, and the marks as a reader sees them.
 */
const BESIDE_MARKUP: readonly Row[] = [
  {
    name: 'a quotation between a tag and its closing tag',
    input: '<b>\'hi\'</b>',
    expected: '<b>\u{2018}hi\u{2019}</b>',
  },
  {
    name: 'an elision pair between emphasis delimiters',
    input: 'rock *\'n\'* roll',
    expected: 'rock *\u{2019}n\u{2019}* roll',
  },
  {
    name: 'an elision pair between strong delimiters',
    input: 'rock **\'n\'** roll',
    expected: 'rock **\u{2019}n\u{2019}** roll',
  },
  {
    name: 'a quotation between a link label bracket and its destination',
    input: 'say [\'hi\'](https://x.example/h) now',
    expected: 'say [\u{2018}hi\u{2019}](https://x.example/h) now',
  },
];

/**
 Texts whose marks curl today and must keep curling: the quotations and apostrophes beside each delimiter.
 */
const CONTROLS: readonly Row[] = [
  {
    name: 'a quotation inside emphasis',
    input: '*\'hi\'*',
    expected: '*\u{2018}hi\u{2019}*',
  },
  {
    name: 'a plain elision pair',
    input: 'rock \'n\' roll',
    expected: 'rock \u{2019}n\u{2019} roll',
  },
  {
    name: 'a plain quotation',
    input: 'He said \'hi\' to me',
    expected: 'He said \u{2018}hi\u{2019} to me',
  },
  {
    name: 'a possessive after emphasis',
    input: 'the *cat*\'s bowl',
    expected: 'the *cat*\u{2019}s bowl',
  },
  {
    name: 'a possessive after a link',
    input: '[Peter](https://x.example/p)\'s nap',
    expected: '[Peter](https://x.example/p)\u{2019}s nap',
  },
  {
    name: 'a possessive after a tag',
    input: 'the <b>cat</b>\'s bowl',
    expected: 'the <b>cat</b>\u{2019}s bowl',
  },
];

/**
 Lines where a quoted number can be read two ways, as an elision or as a quotation, and the reading that leaves fewer marks
 unpaired.
 */
const NUMBERS: readonly Row[] = [
  {
    name: 'a tie keeps the elision: the opener outside pairs with the mark after the number',
    input: '\'the \'90s\'',
    expected: '\u{2018}the \u{2019}90s\u{2019}',
  },
  {
    name: 'a tie keeps the elision for a plain number',
    input: '\'a \'42\' b',
    expected: '\u{2018}a \u{2019}42\u{2019} b',
  },
  {
    name: 'the quoted number wins when it leaves the closing mark of the outer quotation a partner',
    input: '\'the \'90s\' kids\'',
    expected: '\u{2018}the \u{2018}90s\u{2019} kids\u{2019}',
  },
  {
    name: 'the quoted number wins inside a quotation that a later mark closes',
    input: '\'he is \'42\' years old\'',
    expected: '\u{2018}he is \u{2018}42\u{2019} years old\u{2019}',
  },
  {
    name: 'a quoted number alone is a quotation',
    input: 'he is \'42\' years old',
    expected: 'he is \u{2018}42\u{2019} years old',
  },
];

/**
 A fragment, the slice it lands in, and the marks as the restoration writes them there.
 */
type Landing = {
  /**
   What the row is about.
   */
  readonly name: string;

  /**
   The slice the fragment lands in, with the fragment as the editor's region held it.
   */
  readonly slice: string;

  /**
   The region the editor rewrote, found once in the slice.
   */
  readonly region: string;

  /**
   The fragment as restored.
   */
  readonly expected: string;
};

/**
 Fragments that parse differently alone from the way they parse in the slice they land in.
 */
const LANDINGS: readonly Landing[] = [
  {
    name: 'an unclosed tag at the start of the region',
    slice: 'He said <b>It\'s here</b> today.',
    region: '<b>It\'s here',
    expected: '<b>It\u{2019}s here',
  },
  {
    name: 'a closing tag in the region',
    slice: 'A <b>It\'s here</b> and \'x\'.',
    region: 'It\'s here</b> and \'x\'',
    expected: 'It\u{2019}s here</b> and \u{2018}x\u{2019}',
  },
  {
    name: 'a component opened before the region',
    slice: '<Note>It\'s here</Note> too.',
    region: 'It\'s here',
    expected: 'It\u{2019}s here',
  },
  {
    name: 'the tail of a tag with an attribute',
    slice: '<Note {...a} b="it\'s" /> It\'s here.',
    region: '{...a} b="it\'s" /> It\'s',
    expected: '{...a} b="it\'s" /> It\u{2019}s',
  },
];

await describe({
  name: restoreTypography.name,
  children: [
    ...BESIDE_MARKUP.map(function besideCase(row,) {
      return it({
        name: `READS THE CONTEXT OF A MARK AS SHOWN TEXT: ${row.name}`,
        fn: async () => {
          expect(curled({ replacement: row.input, },),).toBe(row.expected,);
        },
      },);
    },),
    ...CONTROLS.map(function controlCase(row,) {
      return it({
        name: `STILL CURLS THE MARKS BESIDE MARKUP THAT CURLED BEFORE: ${row.name}`,
        fn: async () => {
          expect(curled({ replacement: row.input, },),).toBe(row.expected,);
        },
      },);
    },),
    ...NUMBERS.map(function numberCase(row,) {
      return it({
        name: `PAIRS A QUOTED NUMBER BY THE READING THAT LEAVES FEWER MARKS UNPAIRED: ${row.name}`,
        fn: async () => {
          expect(curled({ replacement: row.input, },),).toBe(row.expected,);
        },
      },);
    },),
    ...LANDINGS.map(function landingCase(row,) {
      return it({
        name: `TAKES THE PROSE OF A FRAGMENT FROM THE SLICE IT LANDS IN: ${row.name}`,
        fn: async () => {
          /**
           Where the region starts in the slice.
           */
          const start = row.slice.indexOf(row.region,);
          expect(restoreTypography({
            replacement: row.region,
            replaced: row.region,
            convention: CURLY_PAGE,
            landing: {
              slice: row.slice,
              start,
              end: start + row.region.length,
            },
          },),).toBe(row.expected,);
        },
      },);
    },),
    it({
      name: 'LEAVES A FRAGMENT AS WRITTEN WHEN THE SLICE ITS REGION LANDS IN IS REFUSED BY THE GRAMMAR, since no parse shows its prose',
      fn: async () => {
        /**
         A slice with a brace that never closes.
         */
        const slice = '{oops A <b>It\'s here</b> today.';
        /**
         The region the editor rewrote.
         */
        const region = 'It\'s here';
        /**
         Where the region starts.
         */
        const start = slice.indexOf(region,);
        expect(restoreTypography({
          replacement: region,
          replaced: region,
          convention: CURLY_PAGE,
          landing: {
            slice,
            start,
            end: start + region.length,
          },
        },),).toBe(region,);
      },
    },),
  ],
},);
