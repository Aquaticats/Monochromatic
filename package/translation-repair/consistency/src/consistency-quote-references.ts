import { rememberedViewOf, } from './consistency-prose-text.ts';
import {
  commentUnitsOf,
  isEscapedAt,
  referenceEndOf,
} from './consistency-unread-units.ts';

//region Consistency quote references
// A CHARACTER REFERENCE THAT IS A QUOTATION MARK (`&#39;`, `&apos;`, `&rsquo;`, `&lsquo;`, `&#8217;`, `&#x2019;`) shows a reader
// a mark that no reader of the raw text sees. Every other reader of the consistency pass reads the characters of the raw
// English, so a page that holds one has a mark the proof, the counter and the refusal run cannot place: the refusal run
// steps over the reference as an unread unit (`consistency-unread-units.ts`), and the mark it names beyond is then a second
// mark for the eye (`dogs’&#39;`). The pass therefore refuses every splice on such a page, whether or not it brings an
// apostrophe (`planConsistencyEdit`, `consistency-splice.ts`; recorded as `edge-reference-mark`,
// `pass-consistency-edge-refusal.ts`), wherever the reference stands on it, in the slice of the splice or in any other slice
// of the page.
//
// WHICH REFERENCES. A reference counts when it decodes to a quotation-like character: the eight single marks the pass counts
// (the straight apostrophe, the opening and the closing curly single mark, the modifier letter apostrophe, the low and the
// reversed single mark, both single angle quotation marks), the straight and the four curly or low double marks, both double
// angle quotation marks and the backtick. A decimal reference holds one to seven digits and a hexadecimal one one to six, as
// the Markdown parser reads them; a named reference is one of the names its table decodes to such a character, case
// sensitive and with its semicolon. A reference counts only where the page shows it as text: one in inline code, a code block,
// a tag's attribute, a link's destination or title, an HTML comment (the stretch the page parser masks, one definition,
// `commentRegionsOf`, `mask-html-comments.ts`) is shown as it is written or not at all, and so is an ampersand that a backslash
// escapes (`\&#39;`), which starts no reference.

/**
 The characters a reference may decode to for the pass to refuse every splice on the page: the eight single marks, the five double
 marks, both double angle quotation marks and the backtick.
 */
const QUOTE_LIKE_CHARACTERS: ReadonlySet<string> = new Set([
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
],);

/**
 The HTML named references that decode to one of those characters, with the character, as the table of the Markdown parser
 (`decode-named-character-reference`, over `character-entities`) holds them: every name of the table that decodes to one.
 */
const QUOTE_REFERENCE_NAMES: ReadonlyMap<string, string> = new Map(Object.entries({
  apos: '\'',
  bdquo: '\u{201E}',
  CloseCurlyDoubleQuote: '”',
  CloseCurlyQuote: '’',
  DiacriticalGrave: '`',
  grave: '`',
  laquo: '«',
  ldquo: '“',
  ldquor: '\u{201E}',
  lsaquo: '‹',
  lsquo: '‘',
  lsquor: '\u{201A}',
  OpenCurlyDoubleQuote: '“',
  OpenCurlyQuote: '‘',
  quot: '"',
  QUOT: '"',
  raquo: '»',
  rdquo: '”',
  rdquor: '”',
  rsaquo: '›',
  rsquo: '’',
  rsquor: '’',
  sbquo: '\u{201A}',
},),);

/**
 How many digits a decimal reference holds at most, as the Markdown parser reads one.
 */
const DECIMAL_DIGITS_LONGEST = 7;

/**
 How many digits a hexadecimal reference holds at most, as the Markdown parser reads one.
 */
const HEXADECIMAL_DIGITS_LONGEST = 6;

/**
 The largest code point.
 */
const CODE_POINT_LARGEST = 0x10_FF_FF;

/**
 The character a reference decodes to, when it decodes to a quotation-like one.

 @param reference - the reference as written, from its `&` to its `;`

 @returns The character as a list of one, none when the reference decodes to anything else, names no character or holds more
 digits than the parser reads

 @example
 ```ts
 const [character,] = quoteCharacterOf({ reference: '&#x2019;', },); // '’'
 ```
 */
function quoteCharacterOf({ reference, }: { readonly reference: string; },): readonly string[] {
  /**
   What stands between the `&` and the `;`.
   */
  const body = reference.slice(
    1,
    -1,
  );
  /**
   The character the reference decodes to, empty when it decodes to none that counts.
   */
  const decoded = (function decode(): string {
    if (!body.startsWith('#',))
      return QUOTE_REFERENCE_NAMES.get(body,) ?? '';
    /**
     Whether the number is written in hexadecimal.
     */
    const isHexadecimal = (body.charAt(1,) === 'x') || (body.charAt(1,) === 'X');
    /**
     The digits of the number.
     */
    const digits = body.slice(isHexadecimal ? 2 : 1,);
    if (digits.length > (isHexadecimal ? HEXADECIMAL_DIGITS_LONGEST : DECIMAL_DIGITS_LONGEST))
      return '';
    /**
     The code point the number names.
     */
    const code = Number(isHexadecimal ? `0x${digits}` : digits,);
    return (code > CODE_POINT_LARGEST) ? '' : String.fromCodePoint(code,);
  })();
  return QUOTE_LIKE_CHARACTERS.has(decoded,) ? [decoded,] : [];
}

/**
 The character references of an English that decode to a quotation-like character and that the page shows as text, in the order
 they stand, as the English writes them. A page that has one shows a reader a mark that no reader of the raw text sees.

 @param english - the English of a slice, or the English of a whole page

 @returns The references, each from its `&` to its `;`, none for a page that holds none

 @example
 ```ts
 const references = quoteReferencesIn({ english: 'The kittens&#39; bowl and the dogs&amp; cats.', },); // ['&#39;']
 ```
 */
export function quoteReferencesIn({ english, }: { readonly english: string; },): readonly string[] {
  /**
   The English with every unit the parse does not call text blanked, so a reference in code or in markup is none.
   */
  const { prose, } = rememberedViewOf({ english, },);
  /**
   The offsets of the units of the HTML comments, which a page does not show.
   */
  const comments = commentUnitsOf({ english, },);
  /**
   The references found, in order.
   */
  const found: string[] = [];
  for (
    let amp = english.indexOf('&',);
    amp !== (-1);
    amp = english.indexOf(
      '&',
      amp + 1,
    )
  ) {
    // An ampersand a backslash escapes is the text `&` and starts no reference; one after an escaped backslash does.
    if ((prose.charAt(amp,) !== '&') || comments.has(amp,))
      continue;
    if (isEscapedAt({
      text: english,
      at: amp,
    },))
      continue;
    /**
     Where the reference that starts here ends, none when none does.
     */
    const [end,] = referenceEndOf({
      english,
      at: amp,
    },);
    if (end === undefined)
      continue;
    /**
     The reference as written.
     */
    const reference = english.slice(
      amp,
      end + 1,
    );
    /**
     The quotation-like character the reference decodes to, none when it decodes to anything else.
     */
    const characters = quoteCharacterOf({ reference, },);
    if (characters.length > 0)
      found.push(reference,);
  }
  return found;
}

//endregion Consistency quote references
