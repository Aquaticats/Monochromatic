import {
  parseMdxBody,
  requireMdxRefusal,
  withMdxExhaustionRefused,
} from '../dist/final/node/index.mjs';

//region Tag name spellings
// TEST SUPPORT, NOT PACKAGE SOURCE. The spellings of a tag name the readers of
// a tag are held to the strict grammar over, and the grammar's verdict on a
// document, shared by every case that asks both: each reader of a tag name
// (`mdx-tag-name.ts`, `mask-container-tags.ts`, `corpus-run/tag-attributes.ts`)
// is read over the same set, so none is held to a smaller one.

/**
 One character of every class the grammar treats differently inside a tag
 name, none of them whitespace to it (those are the separators' own cases):
 an ASCII letter, a digit, `$`, `_`, `-`, the member and prefix separators,
 characters no name holds, letters of other scripts, a letter past U+FFFF
 (two units to the compiler), a combining mark, a mark that is no identifier
 character, a digit of another script, the middle dot, a character only the
 old identifier rules start a name with, and the zero-width and bidirectional
 controls with the soft hyphen, the word joiner and the variation selectors.
 */
const NAME_CHARACTERS: readonly string[] = [
  'b',
  '3',
  '$',
  '_',
  '-',
  '.',
  ':',
  '@',
  '#',
  '猫',
  'é',
  'ж',
  '\u{FF21}',
  '\u{20000}',
  '\u{1D49C}',
  '\u{1F431}',
  '\u{0301}',
  '\u{20DD}',
  '\u{0663}',
  '\u{00B7}',
  '\u{2118}',
  '\u{200B}',
  '\u{200C}',
  '\u{200D}',
  '\u{200E}',
  '\u{200F}',
  '\u{202A}',
  '\u{202E}',
  '\u{2066}',
  '\u{2069}',
  '\u{00AD}',
  '\u{2060}',
  '\u{3164}',
  '\u{180E}',
  '\u{0085}',
  '\u{034F}',
  '\u{FE0F}',
  '\u{E0041}',
];

/**
 Names written with whitespace around a separator or with a part the grammar
 refuses, which no single character shows.
 */
const SEPARATED_NAMES: readonly string[] = [
  'a . b',
  'a : b',
  'a.b.c',
  'a . b . c',
  'a:b.c',
  'a.b:c',
  'a:b:c',
  'a:',
  'a.',
  'a..b',
  '.a',
  ':a',
  'a::b',
  'a-b-',
  'a.-b',
  'a.3',
  'a:3',
];

/**
 Every name spelling the differential asks: each character in the places a
 name is told apart (first, inside, last, and after each separator), and the
 separated names.

 @returns Spellings in a fixed order

 @example
 ```ts
 const names = tagNameSpellings();
 ```
 */
export function tagNameSpellings(): readonly string[] {
  return [
    ...NAME_CHARACTERS.flatMap(function placed(character,): readonly string[] {
      return [
        `${character}b`,
        `a${character}b`,
        `ab${character}`,
        `a.${character}b`,
        `a.b${character}`,
        `a:${character}b`,
        `a:b${character}`,
      ];
    },),
    ...SEPARATED_NAMES,
  ];
}

/**
 Whether the strict grammar reads a document whole.

 @param document - document under the grammar

 @returns True when the grammar accepts it

 @example
 ```ts
 const accepted = grammarAcceptsDocument({ document: '<details>\n\nA cat naps.\n\n</details>\n', },);
 ```
 */
export function grammarAcceptsDocument({ document, }: { readonly document: string; },): boolean {
  try {
    withMdxExhaustionRefused({
      parse: parseMdxBody,
      input: { body: document, },
    },);
    return true;
  }
  catch (error) {
    // Only the grammar's own refusal reads as a document it does not accept.
    requireMdxRefusal({ error, },);
    return false;
  }
}

//endregion Tag name spellings
