/**
 Tests that the page the run ships never holds two single marks where the archive held one: a gain is not carried beside a kept
 mark that stands past a footnote reference, a tag, a reference link, a link's destination and title, a line break element, a
 comment, a character reference, a backslash escape, white space of any width, a character that renders nothing, a single line
 break inside the paragraph (a line feed, a carriage return, the next-line control, the line separator or the paragraph
 separator) or the markers of a quotation or a list item at the start of the next line (the refusal run holds every character
 the prose view does not read as words, a backslash escape, horizontal white space, every default-ignorable code point, a
 single line break and the markers of its containers), nor is any splice carried on a page that holds a character reference
 that is itself a single or a double quotation mark, wherever it stands, in any slice of the page; and a plain gain is still
 carried, on a page whose references are no mark, and onto words whose edge stands at a character reference the parser decodes
 or a backslash escape, which is kept or cut whole at the edge as the character it shows would be, so that the shipped page
 holds exactly the text the cases name (no half of a unit written, no stop doubled or lost, no semicolon taken from a word that
 merely looks like a reference). An English the grammar refuses keeps the words of its receiving places, with or without a mark
 beside them, so the pass carries no mark there.

 Each case runs the whole pass over a page of four places whose first place holds `dogs’`, builds the page the publishing ships
 and reads the receiving second place and the sentence around it.

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

import { shippedPageOf, } from './consistency-shipped-page.test-fixture.ts';

/**
 One page to run: the English around the receiving second place, which the receiving place's words 'kittens' stand in, and
 the text after the paragraph.
 */
type Case = {
  /**
   What the case is about.
   */
  readonly name: string;

  /**
   The sentence that holds the receiving place, with the place marked.
   */
  readonly second: string;

  /**
   The sentence that holds the chosen first place, `The dogs’ bowl is full.` unless the case needs another, with the place
   marked, and any other text the case needs in the paragraph or in paragraphs of its own.
   */
  readonly first?: string;

  /**
   How many single marks the receiving sentence holds, one unless the case is about a page whose reference stands elsewhere.
   */
  readonly marks?: number;

  /**
   Whether the page holds a character reference that is a quotation mark, so no gain is carried anywhere on it and the
   shipped page holds as many single marks as the archive wrote.
   */
  readonly everywhere?: boolean;

  /**
   What follows the paragraph, a footnote definition or a link definition where the case needs one.
   */
  readonly tail?: string;

  /**
   The sentence as the page ships it where the typography restoration curls the kept straight mark, the sentence as written
   otherwise.
   */
  readonly shippedAs?: string;

  /**
   Whether the writers throw, for the spaces the writer sheets show changed, where a writer that returns the paragraph as it
   stands would return a paragraph that differs from the page.
   */
  readonly silentWriters?: boolean;

  /**
   The outcome of the receiving place, `kept-current` unless the case is about words the pass cannot locate: the combining
   grapheme joiner directly beside the words joins the letter beside it, so the words stand at no word boundary.
   */
  readonly outcome?: string;
};

/**
 The pages whose kept mark stands past what the refusal run now holds, where the gain used to be carried and wrote a second mark.
 */
const DOUBLED: readonly Case[] = [
  { name: 'R01 footnote reference', second: 'The ⟦2kittens2⟧[^1]’ bowl is empty.', tail: '\n\n[^1]: A note.', },
  {
    name: 'S15 footnote reference, straight mark',
    second: 'The ⟦2kittens2⟧[^1]\' bowl is empty.',
    tail: '\n\n[^1]: A note.',
    shippedAs: 'The kittens[^1]’ bowl is empty.',
  },
  { name: 'S16 emphasis and footnote reference', second: 'The *⟦2kittens2⟧*[^1]’ bowl is empty.', tail: '\n\n[^1]: A note.', },
  { name: 'R02 bold tag', second: 'The <b>⟦2kittens2⟧</b>’ bowl is empty.', },
  { name: 'S09 italic tag', second: 'The <i>⟦2kittens2⟧</i>’ bowl is empty.', },
  { name: 'S14 span tag with an attribute', second: 'The <span class="x">⟦2kittens2⟧</span>’ bowl is empty.', },
  { name: 'S07 reference link', second: 'The [⟦2kittens2⟧][1]’ bowl is empty.', tail: '\n\n[1]: https://cats.example/k', },
  { name: 'S10 titled link', second: 'The [⟦2kittens2⟧](https://cats.example/k "the cats")’ bowl is empty.', },
  { name: 'S08 line break element', second: 'The ⟦2kittens2⟧<br/>’ bowl is empty.', },
  { name: 'S04 no-break space', second: 'The ⟦2kittens2⟧\u{00A0}’ bowl is empty.', },
  { name: 'S05 no-break space entity', second: 'The ⟦2kittens2⟧&nbsp;’ bowl is empty.', },
  { name: 'A06 a spaced mark', second: 'The ⟦2kittens2⟧ ’ bowl is empty.', },
];

/**
 The pages whose gain is refused already: a mark directly beside the run the refusal had before.
 */
const REFUSED: readonly Case[] = [
  { name: 'B01 emphasis', second: 'The *⟦2kittens2⟧*’ bowl is empty.', },
  { name: 'B02 strong emphasis', second: 'The **⟦2kittens2⟧**’ bowl is empty.', },
  { name: 'B03 underscores', second: 'The _⟦2kittens2⟧_’ bowl is empty.', },
  { name: 'B04 strikethrough', second: 'The ~~⟦2kittens2⟧~~’ bowl is empty.', },
  { name: 'B08 untitled link', second: 'The [⟦2kittens2⟧](https://cats.example/x)’ bowl is empty.', },
  { name: 'B10 round bracket', second: 'The (⟦2kittens2⟧)’ bowl is empty.', },
  { name: 'B11 straight double mark', second: 'The "⟦2kittens2⟧"’ bowl is empty.', },
  { name: 'B12 curly double mark', second: 'The “⟦2kittens2⟧”’ bowl is empty.', },
  { name: 'S06 comment', second: 'The ⟦2kittens2⟧<!-- c -->’ bowl is empty.', },
  {
    name: 'ZK the NUL character before the mark, which a page shows as the replacement character: held as before, the run itself '
      + 'ending at it (pinned at the unit of the run)',
    second: 'The ⟦2kittens2⟧\u{0000}’ bowl is empty.',
    silentWriters: true,
  },
];

/**
 The character references the re-review put beside the receiving words, each of which a reader sees as a single mark.
 */
const MARK_REFERENCES: readonly string[] = [
  '&#39;',
  '&apos;',
  '&rsquo;',
  '&lsquo;',
  '&#8217;',
  '&#x2019;',
  '&#x27;',
  '&#039;',
];

/**
 The pages whose kept mark is written as a character reference: directly after the words, after a space, directly before them
 and before them past a space. The first place carries the trailing mark or a leading `’tis`, and the page holds the one
 mark the archive held, which a reader sees whichever way it is written.
 */
const REFERENCES: readonly Case[] = MARK_REFERENCES.flatMap(function positionsOf(
  reference,
  index,
): readonly Case[] {
  /**
   The name the re-review gave the reference.
   */
  const id = `E${String(index + 1,)}`;
  /**
   The sentence that holds the chosen words of the leading cases.
   */
  const first = 'He ate ⟦1’tis the cat1⟧ now.';
  return [
    {
      name: `${id}a ${reference} directly after the words`,
      second: `The ⟦2kittens2⟧${reference} bowl is empty.`,
      everywhere: true,
    },
    { name: `${id}b ${reference} after a space`, second: `The ⟦2kittens2⟧ ${reference} bowl is empty.`, everywhere: true, },
    {
      name: `${id}c ${reference} directly before the words`,
      first,
      second: `He ate ${reference}⟦2the kitten2⟧ now.`,
      everywhere: true,
    },
    {
      name: `${id}d ${reference} before the words past a space`,
      first,
      second: `He ate ${reference} ⟦2the kitten2⟧ now.`,
      everywhere: true,
    },
  ];
},);

/**
 The pages whose reference stands away from the receiving words: earlier in the paragraph, in an earlier paragraph, in a later
 one, and a double mark, so every gain on a page that holds one is refused wherever the reference is.
 */
const PAGE_WIDE: readonly Case[] = [
  {
    name: 'P1 reference earlier in the paragraph',
    first: 'A &lsquo;word is here. The ⟦1dogs’1⟧ bowl is full.',
    second: 'The ⟦2kittens2⟧ bowl is empty.',
    marks: 0,
    everywhere: true,
  },
  {
    name: 'P2 reference in an earlier paragraph',
    first: 'A &rsquo;word is here.\n\nThe ⟦1dogs’1⟧ bowl is full.',
    second: 'The ⟦2kittens2⟧ bowl is empty.',
    marks: 0,
    everywhere: true,
  },
  {
    name: 'P3 reference in a later paragraph',
    second: 'The ⟦2kittens2⟧ bowl is empty.',
    tail: '\n\nA &#8217;word is here.',
    marks: 0,
    everywhere: true,
  },
  {
    name: 'P4 reference to a double mark',
    second: 'The ⟦2kittens2⟧ bowl is &quot;empty&quot;.',
    marks: 0,
    everywhere: true,
  },
];

/**
 The pages whose references are no mark: an ampersand and a no-break space, which the pass does not refuse, so the plain gain is
 still carried there.
 */
const PLAIN_REFERENCES: readonly Case[] = [
  { name: 'ampersand', second: 'The ⟦2kittens2⟧ bowl is empty &amp; full.', },
  { name: 'no-break space', second: 'The ⟦2kittens2⟧ bowl is&nbsp;empty.', },
];

/**
 The pages whose kept mark stands past a backslash escape: of an emphasis character, a bracket, a backslash and a tilde, and
 the escaped emphasis on both sides of the words.
 */
const ESCAPES: readonly Case[] = [
  { name: 'B1 escaped star', second: String.raw`The ⟦2kittens2⟧\*’ bowl is empty.`, },
  { name: 'B2 escaped underscore', second: String.raw`The ⟦2kittens2⟧\_’ bowl is empty.`, },
  { name: 'B3 escaped bracket', second: String.raw`The ⟦2kittens2⟧\]’ bowl is empty.`, },
  { name: 'B4 escaped backslash', second: String.raw`The ⟦2kittens2⟧\\’ bowl is empty.`, },
  { name: 'B5 escaped tilde', second: String.raw`The ⟦2kittens2⟧\~’ bowl is empty.`, },
  { name: 'B06 escaped stars on both sides', second: String.raw`The \*⟦2kittens2⟧\*’ bowl is empty.`, },
  { name: 'B6 leading escaped star', first: 'He ate ⟦1’tis the cat1⟧ now.', second: String.raw`He ate ’\*⟦2the kitten2⟧ now.`, },
  { name: 'B7 leading escaped bracket', first: 'He ate ⟦1’tis the cat1⟧ now.', second: String.raw`He ate ’\[⟦2the kitten2⟧ now.`, },
];

/**
 The code points of the horizontal spaces the re-review put between the words and the kept mark, none of them the space, the
 tab or the no-break space the run stepped over before.
 */
const SPACE_CODES: readonly (readonly [string, number,])[] = [
  ['W1', 0x20_09],
  ['W2', 0x20_03],
  ['W3', 0x20_2F],
  ['W4', 0x20_5F],
  ['W5', 0x30_00],
  ['W6', 0x16_80],
  ['W7', 0x20_02],
  ['W8', 0x20_0A],
  ['W9', 0x20_07],
  ['WA', 0xFE_FF],
];

/**
 The pages whose kept mark stands past one of those spaces, after the words and before them.
 */
const SPACES: readonly Case[] = SPACE_CODES.flatMap(function spacedOf([id, code,],): readonly Case[] {
  /**
   The space as a character.
   */
  const space = String.fromCodePoint(code,);
  /**
   Whether the writer sheets show the space changed, which `invisible-variants.ts` does for U+202F and U+FEFF.
   */
  const silentWriters = (code === 0x20_2F) || (code === 0xFE_FF);
  return [
    {
      name: `${id} U+${code.toString(16,).toUpperCase()} after the words`,
      second: `The ⟦2kittens2⟧${space}’ bowl is empty.`,
      silentWriters,
    },
    {
      name: `${id}b U+${code.toString(16,).toUpperCase()} before the words`,
      first: 'He ate ⟦1’tis the cat1⟧ now.',
      second: `He ate ’${space}⟦2the kitten2⟧ now.`,
      silentWriters,
    },
  ];
},);

/**
 The pages whose kept mark stands on the next line of the paragraph: a soft break, two spaces and a break, a backslash and a
 break, a break and a space, and a line that holds only a character the parse does not call blank or only line separators, where
 the page grammar keeps one paragraph (`maskInvisibleLines`, `maskHtmlComments`, `maskLoneContainerTags`, `parseMdxBody`) and the
 two marks stand in it.
 */
const BREAKS: readonly Case[] = [
  { name: 'K1 soft break', second: 'The ⟦2kittens2⟧\n’ bowl is empty.', },
  { name: 'K2 two spaces and a break', second: 'The ⟦2kittens2⟧  \n’ bowl is empty.', },
  { name: 'K3 backslash and a break', second: 'The ⟦2kittens2⟧\\\n’ bowl is empty.', },
  { name: 'K4 soft break and a space', second: 'The ⟦2kittens2⟧\n ’ bowl is empty.', },
  { name: 'B1 line of an en space between two line feeds', second: 'The ⟦2kittens2⟧\n\u{2002}\n’ bowl is empty.', },
  { name: 'B2 line of a line separator between two line feeds', second: 'The ⟦2kittens2⟧\n\u{2028}\n’ bowl is empty.', },
  { name: 'B3 two paragraph separators', second: 'The ⟦2kittens2⟧\u{2029}\u{2029}’ bowl is empty.', },
  { name: 'B3b a line separator and a paragraph separator', second: 'The ⟦2kittens2⟧\u{2028}\u{2029}’ bowl is empty.', },
  { name: 'B4 line of a no-break space between a line feed and a carriage return', second: 'The ⟦2kittens2⟧\n\u{00A0}\r’ bowl is empty.', },
  { name: 'B5 line of a byte order mark between two carriage returns', second: 'The ⟦2kittens2⟧\r\u{FEFF}\r’ bowl is empty.', },
  { name: 'B6 line of a next-line control between two line feeds', second: 'The ⟦2kittens2⟧\n\u{0085}\n’ bowl is empty.', },
  { name: 'B7 a single line separator', second: 'The ⟦2kittens2⟧\u{2028}’ bowl is empty.', },
  // A line of one invisible or non-ASCII space character is no blank line to CommonMark, which the site renders, so the two
  // marks stand in one paragraph and the gain is refused (n156, W9).
  { name: 'B8 line of an ideographic space between two line feeds', second: 'The ⟦2kittens2⟧\n\u{3000}\n’ bowl is empty.', },
  {
    name: 'B9 line of a zero-width space between two line feeds',
    second: 'The ⟦2kittens2⟧\n\u{200B}\n’ bowl is empty.',
    silentWriters: true,
  },
  { name: 'BA line of a no-break space between two line feeds', second: 'The ⟦2kittens2⟧\n\u{00A0}\n’ bowl is empty.', },
  {
    name: 'BB line of a byte order mark between two line feeds',
    second: 'The ⟦2kittens2⟧\n\u{FEFF}\n’ bowl is empty.',
    silentWriters: true,
  },
  {
    name: 'BC line of a word joiner between two line feeds',
    second: 'The ⟦2kittens2⟧\n\u{2060}\n’ bowl is empty.',
    silentWriters: true,
  },
  { name: 'BD line of a narrow no-break space between two line feeds', second: 'The ⟦2kittens2⟧\n\u{202F}\n’ bowl is empty.', },
  { name: 'BE line of a figure space between two line feeds', second: 'The ⟦2kittens2⟧\n\u{2007}\n’ bowl is empty.', },
  { name: 'BF line of a zero-width non-joiner between two line feeds', second: 'The ⟦2kittens2⟧\n\u{200C}\n’ bowl is empty.', },
  // A lone container tag line closed by a no-break space is text, not a tag line to blank: the English holds a tag that never
  // closes, which the grammar refuses, so the receiving words stay (n156, W5).
  {
    name: 'W5 lone container tag line followed by a no-break space',
    second: 'The ⟦2kittens2⟧\n<Note>\u{00A0}\r\n\u{00AD}’ bowl is empty.',
    silentWriters: true,
  },
];

/**
 The pages whose kept mark stands on the next line of a paragraph inside a quotation or a list item, past the markers the next
 line starts with, which render nothing: a quotation marker with its space or without, markers of nested quotations, a list item
 in a quotation and a quotation in a list item, the indentation of a list item and of a lazy line, and the leading side.
 */
const CONTAINERS: readonly Case[] = [
  { name: 'C1 quotation marker', first: '> The ⟦1dogs’1⟧ bowl is full.', second: 'The ⟦2kittens2⟧\n> ’ bowl is empty.', },
  { name: 'C2 quotation marker without a space', first: '> The ⟦1dogs’1⟧ bowl is full.', second: 'The ⟦2kittens2⟧\n>’ bowl is empty.', },
  {
    name: 'C3 nested quotation markers',
    first: '> > The ⟦1dogs’1⟧ bowl is full.',
    second: 'The ⟦2kittens2⟧\n> > ’ bowl is empty.',
  },
  {
    name: 'C4 list item in a quotation',
    first: '> - The ⟦1dogs’1⟧ bowl is full.',
    second: 'The ⟦2kittens2⟧\n>   ’ bowl is empty.',
  },
  {
    name: 'C5 quotation in a list item',
    first: '- > The ⟦1dogs’1⟧ bowl is full.',
    second: 'The ⟦2kittens2⟧\n  > ’ bowl is empty.',
  },
  { name: 'C6 list item continuation', first: '- The ⟦1dogs’1⟧ bowl is full.', second: 'The ⟦2kittens2⟧\n  ’ bowl is empty.', },
  { name: 'C7 lazy line of a quotation', first: '> The ⟦1dogs’1⟧ bowl is full.', second: 'The ⟦2kittens2⟧\n’ bowl is empty.', },
  {
    name: 'C8 quotation marker on the leading side',
    first: '> He ate ⟦1’tis the cat1⟧ now.',
    second: 'He ate ’\n> ⟦2the kitten2⟧ now.',
  },
  {
    name: 'C9 nested quotation markers on the leading side',
    first: '> > He ate ⟦1’tis the cat1⟧ now.',
    second: 'He ate ’\n> > ⟦2the kitten2⟧ now.',
  },
];

/**
 The code points of the characters that render nothing and are no white space, which the writer lane leaves alone, and the
 line and paragraph separators and the next-line control, each a single line break to the run.
 */
const INVISIBLE_CODES: readonly (readonly [string, number,])[] = [
  ['Z1', 0x20_0C],
  ['Z2', 0x20_0D],
  ['Z3', 0x20_0E],
  ['Z4', 0x03_4F],
  ['Z5', 0x18_0E],
  ['Z6', 0xFE_0F],
  ['Z7', 0xE_01_00],
  ['Z8', 0x20_28],
  ['Z9', 0x20_29],
  ['ZA', 0x85],
  ['ZB', 0x20_0B],
  ['ZC', 0x20_60],
  ['ZD', 0xAD],
  ['ZE', 0x01],
  ['ZF', 0x08],
  ['ZG', 0x0E],
  ['ZH', 0x1F],
  ['ZI', 0x7F],
  ['ZJ', 0x28_00],
  ['ZP', 0x80],
  ['ZQ', 0x84],
  ['ZR', 0x86],
  ['ZS', 0x9F],
];

/**
 The pages whose kept mark stands past one of those characters, directly after the words, past a space on each side and before
 the words. The sheet the writers see folds the zero-width space, the word joiner and the soft hyphen, so those cases run with
 silent writers; so do the cases of the controls and of the Braille blank, which show a blank gap or nothing though no
 default-ignorable code point (U+0001 to U+0008, U+000E to U+001F, U+007F, U+0080 to U+0084, U+0086 to U+009F, U+2800).
 */
const INVISIBLES: readonly Case[] = INVISIBLE_CODES.flatMap(function invisibleOf([id, code,],): readonly Case[] {
  /**
   The character.
   */
  const invisible = String.fromCodePoint(code,);
  /**
   Whether the writer sheets show the character changed, which `invisible-variants.ts` does for these.
   */
  const silentWriters = (code === 0x20_0B) || (code === 0x20_60) || (code === 0xAD) || (code <= 0x7F) || (code === 0x28_00)
    || ((code >= 0x80) && (code <= 0x9F) && (code !== 0x85));
  /**
   The name of the code point.
   */
  const label = `U+${code.toString(16,).toUpperCase()}`;
  /**
   The outcome the pass records where the character is a combining one that joins the letter beside it.
   */
  const joined = (code === 0x03_4F) ? { outcome: 'unlocated', } : {};
  return [
    {
      name: `${id} ${label} after the words`,
      second: `The ⟦2kittens2⟧${invisible}’ bowl is empty.`,
      silentWriters,
      ...joined,
    },
    {
      name: `${id}b ${label} between two spaces after the words`,
      second: `The ⟦2kittens2⟧ ${invisible} ’ bowl is empty.`,
      silentWriters,
    },
    {
      name: `${id}c ${label} before the words`,
      first: 'He ate ⟦1’tis the cat1⟧ now.',
      second: `He ate ’${invisible}⟦2the kitten2⟧ now.`,
      silentWriters,
      ...joined,
    },
  ];
},);

/**
 The pages whose gain is refused because a reference that is a quotation mark is the chosen words' own edge or stands beside
 them, though the chosen words bring no apostrophe to the receiving words: a splice of any kind is refused on such a page.
 */
const NON_GAINS: readonly Case[] = [
  {
    name: 'T2 kept reference before the receiving words, the chosen words start with a reference',
    first: 'He ate ⟦1&#39;tis the cat1⟧ now.',
    second: 'He ate &#39;⟦2the kitten2⟧ now.',
    everywhere: true,
  },
  {
    name: 'T4 kept mark before the receiving words, the chosen words start with a reference',
    first: 'He ate ⟦1&#39;tis the cat1⟧ now.',
    second: 'He ate ’ ⟦2the kitten2⟧ now.',
    everywhere: true,
  },
  {
    name: 'T5 chosen words end in a reference',
    first: 'The ⟦1dogs&rsquo;1⟧ bowl is full.',
    second: 'The ⟦2kittens2⟧ bowl is empty.',
    marks: 0,
    everywhere: true,
  },
  {
    name: 'T6 chosen words hold a reference inside',
    first: 'The ⟦1cat&#39;s bowl1⟧ is full.',
    second: 'The ⟦2kitten bowl2⟧ is empty.',
    marks: 0,
    everywhere: true,
  },
  {
    name: 'T7 receiving words end in a reference',
    first: 'The ⟦1cats1⟧ bowl is full.',
    second: 'The ⟦2dogs&#39;2⟧ bowl is empty.',
    everywhere: true,
  },
];

/**
 The pages with a comment that shares its dashes with a second opener, so that the reference after them is page text, and the
 page whose reference is escaped by a backslash and so is no reference.
 */
const COMMENT_SHAPES: readonly Case[] = [
  {
    name: 'V5a reference after a comment that a second opener ends early',
    second: 'The ⟦2kittens2⟧ bowl is empty.',
    tail: '\n\nx <!-- y <!--> &#39; z --> w.',
    marks: 0,
    everywhere: true,
  },
  {
    name: 'V5c reference after an escaped backslash is a reference',
    second: String.raw`The ⟦2kittens2⟧ bowl is empty \\&#39; x.`,
    everywhere: true,
  },
];

/**
 The pages that a plain gain is still carried onto, with the text the receiving place carries: references that are no mark and
 are not read as one, and words at whose edge stands a reference that ends in a semicolon, which the cut of the words must keep.
 */
const CARRIED: readonly (Case & { readonly carries: string; })[] = [
  {
    name: 'V5b escaped ampersand is no reference',
    second: String.raw`The ⟦2kittens2⟧ bowl is empty \&#39; x.`,
    carries: String.raw`The dogs’ bowl is empty \&#39; x.`,
  },
  {
    name: 'S0 chosen words end in a reference of a plain character',
    first: 'The ⟦1cats &amp;1⟧ bowl is full.',
    second: 'The ⟦2kittens2⟧ bowl is empty.',
    carries: 'The cats &amp; bowl is empty.',
  },
  {
    name: 'S1 chosen words end in a no-break space reference, which is cut whole like the space it shows',
    first: 'The ⟦1cats&nbsp;1⟧ bowl is full.',
    second: 'The ⟦2kittens2⟧ bowl is empty.',
    carries: 'The cats bowl is empty.',
  },
  {
    name: 'S2 receiving words end in a reference of a plain character',
    first: 'The ⟦1cats1⟧ bowl is full.',
    second: 'The ⟦2dogs &amp;2⟧ bowl is empty.',
    carries: 'The cats bowl is empty.',
  },
  {
    name: 'S3 chosen words end in a literal semicolon, which ends no reference and is taken off as before',
    first: 'The ⟦1cats;1⟧ bowl is full.',
    second: 'The ⟦2kittens2⟧ bowl is empty.',
    carries: 'The cats bowl is empty.',
  },
  {
    name: 'S4 chosen words end in an escaped full stop, which is cut whole like a plain one',
    first: String.raw`The ⟦1cats\.1⟧ bowl is full.`,
    second: 'The ⟦2kittens2⟧ bowl is empty.',
    carries: 'The cats bowl is empty.',
  },
  {
    name: 'S5 judges quote the chosen words up to the semicolon of the reference that ends them',
    first: 'The ⟦1cats &amp1⟧; bowl is full.',
    second: 'The ⟦2kittens2⟧ bowl is empty.',
    carries: 'The cats &amp; bowl is empty.',
  },
  {
    name: 'S6 judges quote the receiving words up to the semicolon of the reference that ends them',
    first: 'The ⟦1cats1⟧ bowl is full.',
    second: 'The ⟦2dogs &amp2⟧; bowl is empty.',
    carries: 'The cats bowl is empty.',
  },
  {
    name: 'S7 judges quote the chosen words up to the backslash that escapes the full stop ending them, which is cut whole',
    first: String.raw`The ⟦1cats\1⟧. bowl is full.`,
    second: 'The ⟦2kittens2⟧ bowl is empty.',
    carries: 'The cats bowl is empty.',
  },
  {
    name: 'S8 judges quote the receiving words up to the backslash that escapes the full stop ending them, which stays beside them',
    first: 'The ⟦1cats1⟧ bowl is full.',
    second: String.raw`The ⟦2dogs\2⟧. bowl is empty.`,
    carries: String.raw`The cats\. bowl is empty.`,
  },
  {
    name: 'K8 quotation marker that is text, in the middle of a line',
    first: '> The ⟦1dogs’1⟧ bowl is full.',
    second: 'The ⟦2kittens2⟧ > ’ bowl is empty.',
    carries: 'The dogs’ > ’ bowl is empty.',
  },
  {
    name: 'K9 blank quotation line between the marks',
    first: '> The ⟦1dogs’1⟧ bowl is full.',
    second: 'The ⟦2kittens2⟧\n>\n> ’ bowl is empty.',
    carries: 'The dogs’\n>\n> ’ bowl is empty.',
  },
  {
    name: 'KA marker outside every quotation starts a quotation',
    second: 'The ⟦2kittens2⟧\n> ’ bowl is empty.',
    carries: 'The dogs’\n> ’ bowl is empty.',
  },
  {
    name: 'B0 a line of a space between two line feeds is a blank line, so the mark stands in another paragraph',
    second: 'The ⟦2kittens2⟧\n \n’ bowl is empty.',
    carries: 'The dogs’\n \n’ bowl is empty.',
  },
];

/**
 One page whose words a judge quotes with a backslash escape, a character reference or a quote edge at a unit, and the first
 section of the page as the publishing ships it.
 */
type EdgePage = {
  /**
   What the case is about.
   */
  readonly name: string;

  /**
   The English of the first section with the markers of its places.
   */
  readonly template: string;

  /**
   The first section as the page ships it, the heading and the paragraph.
   */
  readonly shipped: string;

  /**
   The outcome of each of the four places, in order.
   */
  readonly outcomes: readonly string[];
};

/**
 The outcomes of a page whose first place is chosen and whose other three places take its words.
 */
const ALL_CARRIED: readonly string[] = ['chosen', 'replaced', 'replaced', 'replaced',];

/**
 The outcomes of a page whose chosen first place gives its words to the third and fourth places while the second place's quote
 stands nowhere, because an edge of it stands inside a word.
 */
const PLACE_TWO_REFUSED: readonly string[] = ['chosen', 'unlocated', 'replaced', 'replaced',];

/**
 The outcomes of a page whose chosen quote stands nowhere, so the pass touches no place.
 */
const NONE_TOUCHED: readonly string[] = ['untouched', 'untouched', 'untouched', 'untouched',];

/**
 One receiving second place whose quote has an edge inside a word that a reader sees, through a unit or a character the page
 shows nothing for, with the first section as the page ships it (n156, W2): the second place keeps its words and the third and
 fourth are unchanged by it.

 @param name - what the case is about

 @param before - English around the second place without the markers

 @param marked - English around the second place with the markers

 @returns Page case

 @example
 ```ts
 const entry = insideWord({ name: 'M3', before: 'See kit&shy;tens now.', marked: 'See kit&shy;⟦2tens2⟧ now.', },);
 ```
 */
function insideWord(
  {
    name,
    before,
    marked,
  }: {
    readonly name: string;
    readonly before: string;
    readonly marked: string;
  },
): EdgePage {
  return {
    name,
    template: `The ⟦1dogs’1⟧ bowl. ${marked} A ⟦3cat3⟧ naps.`,
    shipped: `## One\n\nThe dogs’ bowl. ${before} A dogs’ naps.`,
    outcomes: PLACE_TWO_REFUSED,
  };
}

/**
 The pages whose span edge stands at a backslash escape or a character reference: the unit is cut or kept whole as the character it
 shows would be, the stop, bracket or space goes beside the words like a plain one, a semicolon that ends no reference the parser
 decodes is a plain semicolon, and a quote that begins or ends inside a unit names the whole unit. Each ships exactly this text.
 */
const SPAN_EDGES: readonly EdgePage[] = [
  {
    name: 'E1 chosen quote ends in an escaped closing parenthesis',
    template: String.raw`He said (⟦1the cats\)1⟧ was here. He said the ⟦2kittens2⟧ was there. A ⟦3cat3⟧ naps.`,
    shipped: '## One\n\nHe said (the cats\\) was here. He said the the cats was there. A the cats naps.',
    outcomes: ALL_CARRIED,
  },
  {
    name: 'E1b the same, the receiving words in a sentence of their own',
    template: String.raw`He said (⟦1the cats\)1⟧ was here. He said ⟦2the kittens2⟧ was there. A ⟦3cat3⟧ naps.`,
    shipped: '## One\n\nHe said (the cats\\) was here. He said the cats was there. A the cats naps.',
    outcomes: ALL_CARRIED,
  },
  {
    name: 'E3c chosen quote ends in an escaped closing bracket',
    template: String.raw`He said [⟦1the cats\]1⟧ was here. He said ⟦2the kittens2⟧ was there. A ⟦3cat3⟧ naps.`,
    shipped: '## One\n\nHe said [the cats\\] was here. He said the cats was there. A the cats naps.',
    outcomes: ALL_CARRIED,
  },
  {
    name: 'F0 control: a plain full stop beside the receiving words stays where the English has it',
    template: 'The ⟦1dogs’1⟧ bowl is full. See ⟦2kittens2⟧. Then more. A ⟦3cat3⟧ naps.',
    shipped: '## One\n\nThe dogs’ bowl is full. See dogs’. Then more. A dogs’ naps.',
    outcomes: ALL_CARRIED,
  },
  {
    name: 'F2 receiving quote ends in an escaped full stop, which stays beside the words like a plain one',
    template: String.raw`The ⟦1dogs’1⟧ bowl is full. See ⟦2kittens\.2⟧ Then more. A ⟦3cat3⟧ naps.`,
    shipped: '## One\n\nThe dogs’ bowl is full. See dogs’\\. Then more. A dogs’ naps.',
    outcomes: ALL_CARRIED,
  },
  {
    name: 'F2b control: the receiving quote stops before the escaped full stop',
    template: String.raw`The ⟦1dogs’1⟧ bowl is full. See ⟦2kittens2⟧\. Then more. A ⟦3cat3⟧ naps.`,
    shipped: '## One\n\nThe dogs’ bowl is full. See dogs’\\. Then more. A dogs’ naps.',
    outcomes: ALL_CARRIED,
  },
  {
    name: 'F3 chosen quote ends in an escaped full stop, which no place carries',
    template: String.raw`The ⟦1dogs\.1⟧ Bowl full. The ⟦2kittens2⟧. A ⟦3cat3⟧ naps.`,
    shipped: '## One\n\nThe dogs\\. Bowl full. The dogs. A dogs naps.',
    outcomes: ALL_CARRIED,
  },
  {
    name: 'F3b chosen and receiving quotes both end in an escaped full stop',
    template: String.raw`The ⟦1dogs\.1⟧ Bowl full. The ⟦2kittens\.2⟧ Then. A ⟦3cat3⟧ naps.`,
    shipped: '## One\n\nThe dogs\\. Bowl full. The dogs\\. Then. A dogs naps.',
    outcomes: ALL_CARRIED,
  },
  {
    name: 'F4 receiving quote begins at the character an escape escapes, so the whole escape is replaced',
    template: String.raw`The ⟦1dogs’1⟧ bowl is full. He said \⟦2*kittens*2⟧ now. A ⟦3cat3⟧ naps.`,
    shipped: '## One\n\nThe dogs’ bowl is full. He said dogs’ now. A dogs’ naps.',
    outcomes: ALL_CARRIED,
  },
  {
    name: 'F4b control: an escaped parenthesis whose quote leaves the escape whole',
    template: String.raw`The ⟦1dogs’1⟧ bowl is full. He said \⟦2(kittens)2⟧ now. A ⟦3cat3⟧ naps.`,
    shipped: '## One\n\nThe dogs’ bowl is full. He said \\(dogs’) now. A dogs’ naps.',
    outcomes: ALL_CARRIED,
  },
  {
    name: 'E4 receiving quote begins inside a reference that follows a letter, so the quote stands inside a word and is refused',
    template: 'The ⟦1dogs’1⟧ bowl is full. See dogs&⟦2amp; cats2⟧ now. A ⟦3cat3⟧ naps.',
    shipped: '## One\n\nThe dogs’ bowl is full. See dogs&amp; cats now. A dogs’ naps.',
    outcomes: PLACE_TWO_REFUSED,
  },
  {
    name: 'E4b chosen quote begins inside a reference that follows a letter, so the chosen words stand nowhere and no place changes',
    template: 'See dogs&⟦1amp; cats1⟧ now. The ⟦2kittens2⟧ bowl is full. A ⟦3cat3⟧ naps.',
    shipped: '## One\n\nSee dogs&amp; cats now. The kittens bowl is full. A cat naps.',
    outcomes: NONE_TOUCHED,
  },
  {
    name: 'F1 receiving words end before a semicolon that a word like Q&A merely has',
    template: 'The ⟦1dogs’1⟧ bowl is full. See ⟦2Q&A2⟧; then more. A ⟦3cat3⟧ naps.',
    shipped: '## One\n\nThe dogs’ bowl is full. See dogs’; then more. A dogs’ naps.',
    outcomes: ALL_CARRIED,
  },
  {
    name: 'F1b chosen words end before a semicolon that a word like R&D merely has',
    template: 'See ⟦1R&D1⟧; then more. The ⟦2kittens2⟧ bowl. A ⟦3cat3⟧ naps.',
    shipped: '## One\n\nSee R&D; then more. The R&D bowl. A R&D naps.',
    outcomes: ALL_CARRIED,
  },
  {
    name: 'F1c control: the same sentence with the words spelled out',
    template: 'The ⟦1dogs’1⟧ bowl is full. See ⟦2Q and A2⟧; then more. A ⟦3cat3⟧ naps.',
    shipped: '## One\n\nThe dogs’ bowl is full. See dogs’; then more. A dogs’ naps.',
    outcomes: ALL_CARRIED,
  },
  {
    name: 'F6 receiving words end in a no-break space reference, which stays beside them',
    template: 'The ⟦1dogs’1⟧ bowl is full. See ⟦2kittens&nbsp;2⟧more. A ⟦3cat3⟧ naps.',
    shipped: '## One\n\nThe dogs’ bowl is full. See dogs’&nbsp;more. A dogs’ naps.',
    outcomes: ALL_CARRIED,
  },
  // The word test reads the text a reader sees (n156, W2): an edge with a letter that a unit shows, a default-ignorable character
  // or an ampersand between letters beside it stands inside a word, so the quote is refused whichever way it was widened.
  insideWord({ name: 'M3 a soft hyphen reference before the receiving quote', before: 'See kit&shy;tens now.', marked: 'See kit&shy;⟦2tens2⟧ now.', },),
  insideWord({ name: 'M4 a numeric reference of a letter before the receiving quote', before: 'See k&#105;ttens now.', marked: 'See k&#105;⟦2ttens2⟧ now.', },),
  insideWord({ name: 'M5 a soft hyphen reference after the receiving quote', before: 'See kitt&shy;ens now.', marked: 'See ⟦2kitt2⟧&shy;ens now.', },),
  insideWord({ name: 'M8 receiving quote begins inside a reference after a letter', before: 'See Tom&amp; Jerry now.', marked: 'See Tom&⟦2amp; Jerry2⟧ now.', },),
  insideWord({ name: 'M13 a reference of a letter after the receiving quote', before: 'See kitten&#115; now.', marked: 'See ⟦2kitten2⟧&#115; now.', },),
  insideWord({ name: 'M15 a zero-width space before the receiving quote', before: 'See kit\u{200B}tens now.', marked: 'See kit\u{200B}⟦2tens2⟧ now.', },),
  insideWord({ name: 'N1 a reference ampersand between letters after the receiving quote', before: 'Call AT&amp;T now.', marked: 'Call ⟦2AT2⟧&amp;T now.', },),
  insideWord({ name: 'N3 receiving quote begins inside the reference ampersand between letters', before: 'Call AT&amp;T now.', marked: 'Call AT&⟦2amp;T now2⟧.', },),
  insideWord({ name: 'N4 a reference ampersand between letters before the receiving quote', before: 'Call AT&amp;T now.', marked: 'Call AT&amp;⟦2T now2⟧.', },),
  insideWord({ name: 'N5 a plain ampersand between letters after the receiving quote', before: 'Our R&D team.', marked: 'Our ⟦2R2⟧&D team.', },),
  insideWord({ name: 'N6 a plain ampersand between letters before the receiving quote', before: 'Our R&D team.', marked: 'Our R&⟦2D team2⟧.', },),
  // The markup that shows nothing between two pieces of one word (n159, X1): the word a reader sees is cut there as at a letter.
  insideWord({ name: 'X1a emphasis after the receiving quote', before: 'The kit*tens* ate.', marked: 'The ⟦2kit2⟧*tens* ate.', },),
  insideWord({ name: 'X9a an underscore joins the receiving word to the word after it', before: 'The kit_tens ate.', marked: 'The ⟦2kit2⟧_tens ate.', },),
  insideWord({ name: 'X9b a Greek letter beside the receiving quote is a word character', before: 'The kittenω naps.', marked: 'The ⟦2kitten2⟧ω naps.', },),
  insideWord({ name: 'X1b emphasis around the receiving quote', before: 'The kit*tens* ate.', marked: 'The kit*⟦2tens2⟧* ate.', },),
  insideWord({ name: 'X1c strong emphasis before the receiving word', before: 'The **kit**tens ate.', marked: 'The **⟦2kit2⟧**tens ate.', },),
  insideWord({ name: 'X1d a link label then a word', before: 'The [kit](u)tens ate.', marked: 'The [⟦2kit2⟧](u)tens ate.', },),
  insideWord({ name: 'X1e a tag then a word', before: 'The <b>kit</b>tens ate.', marked: 'The <b>⟦2kit2⟧</b>tens ate.', },),
  {
    name: 'X1f the chosen place is half a word, so the chosen words stand nowhere',
    template: 'The ⟦1dogs1⟧*s* bowl. The ⟦2kittens2⟧ ate. A ⟦3cat3⟧ naps.',
    shipped: '## One\n\nThe dogs*s* bowl. The kittens ate. A cat naps.',
    outcomes: NONE_TOUCHED,
  },
  // Controls: a whole word beside each delimiter still carries.
  {
    name: 'X1g control: a whole emphasised receiving word carries',
    template: 'The ⟦1dogs’1⟧ bowl. The *⟦2kittens2⟧* ate. A ⟦3cat3⟧ naps.',
    shipped: '## One\n\nThe dogs’ bowl. The *dogs’* ate. A dogs’ naps.',
    outcomes: ALL_CARRIED,
  },
  {
    name: 'X1h control: a whole receiving word in a link label carries',
    template: 'The ⟦1dogs’1⟧ bowl. The [⟦2kittens2⟧](u) ate. A ⟦3cat3⟧ naps.',
    shipped: '## One\n\nThe dogs’ bowl. The [dogs’](u) ate. A dogs’ naps.',
    outcomes: ALL_CARRIED,
  },
  {
    name: 'M11 chosen quote begins after a numeric reference of a letter, so the chosen words stand nowhere',
    template: 'See k&#105;⟦1ttens bowl1⟧ now. The ⟦2kittens2⟧ ate. A ⟦3cat3⟧ naps.',
    shipped: '## One\n\nSee k&#105;ttens bowl now. The kittens ate. A cat naps.',
    outcomes: NONE_TOUCHED,
  },
  {
    name: 'M12 chosen quote begins inside a reference after a letter, so the chosen words stand nowhere',
    template: 'See Tom&⟦1amp; Jerry1⟧ now. The ⟦2kittens2⟧ ate. A ⟦3cat3⟧ naps.',
    shipped: '## One\n\nSee Tom&amp; Jerry now. The kittens ate. A cat naps.',
    outcomes: NONE_TOUCHED,
  },
  // Controls: the quotes refused today stay refused, and whole words carry, with the reference or the ampersand inside them.
  {
    name: 'M0 control: a whole receiving word carries',
    template: 'The ⟦1dogs’1⟧ bowl. See ⟦2kittens2⟧ now. A ⟦3cat3⟧ naps.',
    shipped: '## One\n\nThe dogs’ bowl. See dogs’ now. A dogs’ naps.',
    outcomes: ALL_CARRIED,
  },
  insideWord({ name: 'M1 control: no unit, the quote begins inside a word', before: 'See kittens bowl now.', marked: 'See k⟦2ittens2⟧ bowl now.', },),
  insideWord({ name: 'M2 control: no unit, the quote ends inside a word', before: 'See kittens now.', marked: 'See ⟦2kitt2⟧ens now.', },),
  insideWord({ name: 'M7 control: the quote begins at the character an escape escapes', before: String.raw`See x\*y now.`, marked: String.raw`See x\⟦2*y2⟧ now.`, },),
  insideWord({ name: 'M10 control: the quote ends at the backslash of an escape', before: String.raw`See kittens\* now.`, marked: String.raw`See ⟦2kittens\2⟧* now.`, },),
  insideWord({ name: 'N7 control: the quote ends before a hyphen between letters', before: 'The so-called team.', marked: 'The ⟦2so2⟧-called team.', },),
  {
    name: 'N2 the quote ends inside the reference of AT&amp;T, whose whole reference then stands before the T of the word',
    template: 'The ⟦1dogs’1⟧ bowl. Call ⟦2AT&amp2⟧;T now. A ⟦3cat3⟧ naps.',
    shipped: '## One\n\nThe dogs’ bowl. Call AT&amp;T now. A dogs’ naps.',
    outcomes: ['chosen', 'unlocated', 'replaced', 'replaced',],
  },
  {
    name: 'W8a the chosen words end in the reference &#39;, so the page-wide reference guard (consistency-splice.ts) keeps every place',
    template: 'The ⟦1dogs&#39;1⟧ bowl is full. See ⟦2kittens2⟧ now. A ⟦3cat3⟧ naps.',
    shipped: '## One\n\nThe dogs&#39; bowl is full. See kittens now. A cat naps.',
    outcomes: ['chosen', 'kept-current', 'kept-current', 'kept-current',],
  },
  {
    name: 'W8b the reference &#39; stands far from every place, which the same guard keeps alike',
    template: 'The ⟦1dogs’1⟧ bowl is full. See ⟦2kittens2⟧ now. A ⟦3cat3⟧ naps. Say &#39;hi&#39; twice.',
    shipped: '## One\n\nThe dogs’ bowl is full. See kittens now. A cat naps. Say &#39;hi&#39; twice.',
    outcomes: ['chosen', 'kept-current', 'kept-current', 'kept-current',],
  },
  {
    name: 'W8c control: a reference that is no quotation mark (&amp;) far from every place keeps nothing',
    template: 'The ⟦1dogs’1⟧ bowl is full. See ⟦2kittens2⟧ now. A ⟦3cat3⟧ naps. Say Tom &amp; Jerry twice.',
    shipped: '## One\n\nThe dogs’ bowl is full. See dogs’ now. A dogs’ naps. Say Tom &amp; Jerry twice.',
    outcomes: ALL_CARRIED,
  },
  {
    name: 'M9 control: the quote ends with the reference ampersand beside a space',
    template: 'The ⟦1dogs’1⟧ bowl. See ⟦2Tom &2⟧amp; Jerry now. A ⟦3cat3⟧ naps.',
    shipped: '## One\n\nThe dogs’ bowl. See dogs’ Jerry now. A dogs’ naps.',
    outcomes: ALL_CARRIED,
  },
  {
    name: 'N8 control: an ampersand word carries whole',
    template: 'The ⟦1dogs’1⟧ bowl. Call ⟦2AT&amp;T2⟧ now. A ⟦3cat3⟧ naps.',
    shipped: '## One\n\nThe dogs’ bowl. Call dogs’ now. A dogs’ naps.',
    outcomes: ALL_CARRIED,
  },
  {
    name: 'N9 control: a chosen ampersand word carries whole into plain places',
    template: 'Call ⟦1AT&amp;T1⟧ now. The ⟦2kittens2⟧ ate. A ⟦3cat3⟧ naps.',
    shipped: '## One\n\nCall AT&amp;T now. The AT&amp;T ate. A AT&amp;T naps.',
    outcomes: ALL_CARRIED,
  },
];

/**
 What an English the grammar refuses holds beside the first place: an unclosed brace, a tag with a character no name holds, an
 unclosed tag and a closing tag that closes nothing. The pass reads such an English as text.
 */
const REFUSALS: readonly (readonly [string, string,])[] = [
  ['an unclosed brace', '{',],
  ['a tag with a character no name holds', '<b.',],
  ['an unclosed tag', '<div',],
  ['a closing tag that closes nothing', '</div>',],
];

/**
 What stands between the receiving words and the rest of the sentence: nothing, a kept mark on the next line of the paragraph,
 and a kept mark in a paragraph of its own.
 */
const BETWEEN_SPELLINGS: readonly (readonly [string, string,])[] = [
  ['no mark', ' ',],
  ['a kept mark on the next line', '\n’ ',],
  ['a kept mark in the next paragraph', '\n\n’ ',],
];

/**
 The pages of an English the grammar refuses, at the pass: the receiving places keep their words whatever stands beside them,
 even where no mark does, so the pass never carries a mark there. The same English parsed (a control) carries the words where
 no mark stands in the way, and keeps them where a mark on the next line of the paragraph does.
 */
const REFUSED_ENGLISH: readonly EdgePage[] = [
  ...REFUSALS.flatMap(function refusedOf([refusal, bad,],): readonly EdgePage[] {
    return BETWEEN_SPELLINGS.map(function betweenOf([between, text,],): EdgePage {
      return {
        name: `${refusal}, ${between}`,
        template: `The ⟦1dogs’1⟧ bowl is full ${bad}. The ⟦2kittens2⟧${text}bowl is empty. A ⟦3cat3⟧ naps.`,
        shipped: `## One\n\nThe dogs’ bowl is full ${bad}. The kittens${text}bowl is empty. A cat naps.`,
        outcomes: ['chosen', 'kept-current', 'kept-current', (text === ' ') ? 'replaced' : 'kept-current',],
      };
    },);
  },),
  {
    name: 'control: the same English parsed, no mark',
    template: 'The ⟦1dogs’1⟧ bowl is full fine. The ⟦2kittens2⟧ bowl is empty. A ⟦3cat3⟧ naps.',
    shipped: '## One\n\nThe dogs’ bowl is full fine. The dogs’ bowl is empty. A dogs’ naps.',
    outcomes: ALL_CARRIED,
  },
  {
    name: 'control: the same English parsed, a kept mark on the next line',
    template: 'The ⟦1dogs’1⟧ bowl is full fine. The ⟦2kittens2⟧\n’ bowl is empty. A ⟦3cat3⟧ naps.',
    shipped: '## One\n\nThe dogs’ bowl is full fine. The kittens\n’ bowl is empty. A cat naps.',
    outcomes: ['chosen', 'kept-current', 'kept-current', 'kept-current',],
  },
];

/**
 The spellings of a single mark a reader sees: the four characters the pass counts as a mark of a word and the references
 to single marks.
 */
const MARK_SPELLINGS: readonly string[] = [
  '\'',
  '‘',
  '’',
  '\u{02BC}',
  ...MARK_REFERENCES,
];

/**
 How many single marks a text shows a reader, whichever way each is written.

 @param text - text

 @returns Count

 @example
 ```ts
 const count = marksIn({ text: 'The dogs’ bowl&#39;', },); // 2
 ```
 */
function marksIn({ text, }: { readonly text: string; },): number {
  return MARK_SPELLINGS.reduce(function counted(sum, spelling,): number {
    return sum + (text.split(spelling,).length - 1);
  }, 0,);
}

/**
 A text with the markers of the places taken out, as the archive wrote it.

 @param text - text with the markers

 @returns Text without them

 @example
 ```ts
 const written = unmarked({ text: 'The ⟦2kittens2⟧ bowl.', },); // 'The kittens bowl.'
 ```
 */
function unmarked({ text, }: { readonly text: string; },): string {
  return ['1', '2', '3',].reduce(function stripped(rest, digit,): string {
    return rest.replaceAll(
      `⟦${digit}`,
      '',
    )
      .replaceAll(
      `${digit}⟧`,
      '',
      );
  }, text,);
}

/**
 Runs one case.

 @param name - what the case is about

 @param second - sentence that holds the receiving place

 @param tail - what follows the paragraph

 @returns Outcome of the receiving place, the sentence around it as the page ships it, and sentence it stood in

 @example
 ```ts
 const seen = await seenOf({ name: 'R01', second: 'The ⟦2kittens2⟧[^1]’ bowl is empty.', tail: '', },);
 ```
 */
async function seenOf(
  {
    first = 'The ⟦1dogs’1⟧ bowl is full.',
    second,
    tail = '',
    shippedAs,
    silentWriters = false,
  }: Case,
): Promise<{
  readonly outcome: string;
  readonly kept: boolean;
  readonly sentenceMarks: number;
  readonly marks: number;
  readonly marksWritten: number;
  readonly shipped: string;
}> {
  /**
   The sentence with the markers taken out, as the archive wrote it.
   */
  const written = unmarked({ text: second, },);
  /**
   The English of the first section with the markers of its places.
   */
  const template = `${first} ${second} A ⟦3cat3⟧ naps.${tail}`;
  /**
   The page run through the pass and shipped.
   */
  const page = await shippedPageOf({ template, silentWriters, },);
  return {
    outcome: nonNullishOrThrow(page.outcomes[1],),
    kept: page.shipped.includes(shippedAs ?? written,),
    sentenceMarks: marksIn({ text: shippedAs ?? written, },),
    marks: marksIn({ text: page.shipped, },),
    marksWritten: marksIn({ text: unmarked({ text: template, },), },),
    shipped: page.shipped,
  };
}

/**
 What stands between the receiving word and its kept mark, a reference whose definition is in the second section: a name for the
 case, the text, the second section and the outcome of the place. A link definition stands alone in its section because the
 definitions are joined by a line break, as the refinement gate joins them (`collectDefinitions`), and a line break after a
 footnote definition continues its paragraph. The text of a reference link is a word the span would join (`kittens` and `x`), so
 the pass names no span there; a footnote reference shows nothing, so the span is found and its kept mark is kept.
 */
const DEFINED_ELSEWHERE: readonly (readonly [string, string, string, string,])[] = [
  ['a footnote reference', '[^1]', 'The kittens nap.\n\n[^1]: A note.\n', 'kept-current'],
  ['a footnote reference and a space', '[^1] ', 'The kittens nap.\n\n[^1]: A note.\n', 'kept-current'],
  ['two footnote references', '[^1][^1]', 'The kittens nap.\n\n[^1]: A note.\n', 'kept-current'],
  ['a reference link', '[x][r]', 'The kittens nap.\n\n[r]: https://x.example/r\n', 'unlocated'],
];

await describe({
  name: shippedPageOf.name,
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    ...[
      ...DOUBLED.map(function doubledCase(entry,) {
        return {
          entry,
          name: `KEEPS THE RECEIVING WORDS WHEN A KEPT MARK STANDS PAST WHAT THE PROSE VIEW DOES NOT READ (${entry.name}), so the `
            + 'shipped page holds exactly one mark there',
        };
      },),
      ...REFERENCES.map(function referenceCase(entry,) {
        return {
          entry,
          name: `KEEPS THE RECEIVING WORDS WHEN THE PAGE HOLDS A CHARACTER REFERENCE THAT IS A SINGLE MARK (${entry.name}), so the `
            + 'shipped page holds exactly one mark there',
        };
      },),
      ...PAGE_WIDE.map(function pageWideCase(entry,) {
        return {
          entry,
          name: `KEEPS THE RECEIVING WORDS WHEN A CHARACTER REFERENCE THAT IS A QUOTATION MARK STANDS ELSEWHERE ON THE PAGE (${
            entry.name
          }), so no gain is carried onto the page`,
        };
      },),
      ...ESCAPES.map(function escapeCase(entry,) {
        return {
          entry,
          name: `KEEPS THE RECEIVING WORDS WHEN A KEPT MARK STANDS PAST A BACKSLASH ESCAPE (${entry.name}), so the shipped page `
            + 'holds exactly one mark there',
        };
      },),
      ...SPACES.map(function spaceCase(entry,) {
        return {
          entry,
          name: `KEEPS THE RECEIVING WORDS WHEN A KEPT MARK STANDS PAST A HORIZONTAL SPACE OF ANY WIDTH (${entry.name}), so the `
            + 'shipped page holds exactly one mark there',
        };
      },),
      ...BREAKS.map(function breakCase(entry,) {
        return {
          entry,
          name: `KEEPS THE RECEIVING WORDS WHEN A KEPT MARK STANDS ON THE NEXT LINE OF THE PARAGRAPH (${entry.name}), so the `
            + 'shipped page holds exactly one mark there',
        };
      },),
      ...CONTAINERS.map(function containerCase(entry,) {
        return {
          entry,
          name: `KEEPS THE RECEIVING WORDS WHEN A KEPT MARK STANDS PAST THE MARKERS OF A QUOTATION OR A LIST ITEM AT THE START OF THE NEXT LINE (${
            entry.name
          }), so the shipped page holds exactly one mark there`,
        };
      },),
      ...INVISIBLES.map(function invisibleCase(entry,) {
        return {
          entry,
          name: `KEEPS THE RECEIVING WORDS WHEN A KEPT MARK STANDS PAST A CHARACTER THAT RENDERS NOTHING OR A SINGLE LINE BREAK (${
            entry.name
          }), so the shipped page holds exactly one mark there`,
        };
      },),
      ...NON_GAINS.map(function nonGainCase(entry,) {
        return {
          entry,
          name: `KEEPS THE RECEIVING WORDS WHEN THE PAGE HOLDS A REFERENCE THAT IS A QUOTATION MARK, THOUGH THE SPLICE BRINGS NO GAIN (${
            entry.name
          }), so the shipped page holds exactly the marks the archive held`,
        };
      },),
      ...COMMENT_SHAPES.map(function commentCase(entry,) {
        return {
          entry,
          name: `KEEPS THE RECEIVING WORDS WHEN THE REFERENCE THE PAGE SHOWS STANDS BESIDE A COMMENT OR AN ESCAPE (${
            entry.name
          }), so the shipped page holds exactly the marks the archive held`,
        };
      },),
      ...REFUSED.map(function refusedCase(entry,) {
        return {
          entry,
          name: `STILL KEEPS THE RECEIVING WORDS WHEN A MARK STANDS PAST THE MARKUP THE REFUSAL ALREADY HELD (${entry.name})`,
        };
      },),
    ].map(function keptCase({ entry, name, },) {
      return it({
        name,
        fn: async () => {
          const seen = await seenOf(entry,);
          expect({
            outcome: seen.outcome,
            kept: seen.kept,
            sentenceMarks: seen.sentenceMarks,
            ...((entry.everywhere === true) ? { marks: seen.marks, } : {}),
          },).toEqual({
            outcome: entry.outcome ?? 'kept-current',
            kept: true,
            sentenceMarks: entry.marks ?? 1,
            ...((entry.everywhere === true) ? { marks: seen.marksWritten, } : {}),
          },);
        },
      },);
    },),
    it({
      name: 'STILL CARRIES A PLAIN GAIN: the receiving words become `dogs’` and the page holds that one mark',
      fn: async () => {
        const seen = await seenOf({ name: 'plain', second: 'The ⟦2kittens2⟧ bowl is empty.', },);
        expect({
          outcome: seen.outcome,
          carried: seen.shipped.includes('The dogs’ bowl is empty.',),
        },).toEqual({ outcome: 'replaced', carried: true, },);
      },
    },),
    ...PLAIN_REFERENCES.map(function plainCase(entry,) {
      return it({
        name: `STILL CARRIES A PLAIN GAIN WHEN THE PAGE HOLDS A REFERENCE THAT IS NO QUOTATION MARK (${entry.name})`,
        fn: async () => {
          const seen = await seenOf(entry,);
          expect({
            outcome: seen.outcome,
            carried: seen.shipped.includes(unmarked({ text: entry.second, },).replace(
              'kittens',
              'dogs’',
            ),),
          },).toEqual({ outcome: 'replaced', carried: true, },);
        },
      },);
    },),
    ...CARRIED.map(function carriedCase(entry,) {
      return it({
        name: `STILL CARRIES THE CHOSEN WORDS WHOLE WHERE NO MARK OR REFERENCE STANDS IN THE WAY (${entry.name}), so the receiving `
          + 'place reads as the chosen words do',
        fn: async () => {
          const seen = await seenOf(entry,);
          expect({
            outcome: seen.outcome,
            carried: seen.shipped.includes(entry.carries,),
          },).toEqual({ outcome: 'replaced', carried: true, },);
        },
      },);
    },),
    ...[
      ...SPAN_EDGES.map(function spanEdgeTitle(entry,) {
        return {
          title: `SHIPS EXACTLY THIS PAGE WHEN A SPAN EDGE STANDS AT A BACKSLASH ESCAPE OR A CHARACTER REFERENCE (${entry.name}), `
            + 'with no half of a unit written and no stop doubled or lost',
          entry,
        };
      },),
      ...REFUSED_ENGLISH.map(function refusedEnglishTitle(entry,) {
        return {
          title: `KEEPS THE RECEIVING WORDS IN AN ENGLISH THE GRAMMAR REFUSES, so the pass carries no mark there (${entry.name})`,
          entry,
        };
      },),
    ].map(function edgePageCase({
      title,
      entry,
    },) {
      return it({
        name: title,
        fn: async () => {
          /**
           The page run through the pass and shipped.
           */
          const page = await shippedPageOf({
            template: entry.template,
            silentWriters: true,
          },);
          expect({
            shipped: nonNullishOrThrow(page.shipped.split('\n\n## Two',)[0],),
            outcomes: page.outcomes,
          },).toEqual({
            shipped: entry.shipped,
            outcomes: entry.outcomes,
          },);
        },
      },);
    },),
    ...DEFINED_ELSEWHERE.map(function definedElsewhereCase([name, between, definitions, outcome,],) {
      return it({
        name: `KEEPS THE RECEIVING WORDS WHEN THE DEFINITION OF THE REFERENCE BEFORE THE KEPT MARK STANDS IN ANOTHER SLICE (n159, X2): ${name}`,
        fn: async () => {
          /**
           The page run through the pass, the definitions in the second section.
           */
          const page = await shippedPageOf({
            template: `The ⟦1dogs’1⟧ bowl is full. The ⟦2kittens2⟧${between}’ bowl is empty. A ⟦3cat3⟧ naps.`,
            silentWriters: true,
            secondSection: definitions,
          },);
          expect({
            shipped: nonNullishOrThrow(page.shipped.split('\n\n## Two',)[0],),
            outcome: page.outcomes[1],
          },).toEqual({
            shipped: `## One\n\nThe dogs’ bowl is full. The kittens${between}’ bowl is empty. A cat naps.`,
            outcome,
          },);
        },
      },);
    },),
  ],
},);
