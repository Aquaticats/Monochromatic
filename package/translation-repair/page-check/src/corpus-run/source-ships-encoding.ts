import {
  commentRegionsOf,
  MarkdownParseError,
  parseMarkdownMathBody,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  ESCAPABLE,
  type SourceSpans,
  spansOf,
  type TextEdit,
  type TextRange,
} from './source-ships-spans.ts';

//region Source ships encoding
// THE TEXT OF A SLICE THAT SHIPS ITS ORIGINAL, AS LITERAL TEXT FOR THE STRICT MDX GRAMMAR. The page carries the
// original as the artifact records it, and writes it through here only when the page with the original as recorded
// does not parse, or when the original holds a comment that never closes (`source-ships-compose.ts` says when).
//
// WHAT IS WRITTEN, AND WHERE IT IS READ FROM. The grammar the package parses pages with is `remark-mdx`, which reads a
// `{` as the start of an expression (acorn, through `micromark-extension-mdxjs`, parses it; the `package.json` of
// `remark-mdx` lists the extension), a `<` as the start of a tag, and a line starting `import ` or `export ` as an
// ES module. The text is parsed once as CommonMark with the site's math and nothing else
// (`document-structure/src/parse-markdown-math.ts`), which never refuses a text, and the parse says where those
// characters stand in text a reader sees: a backslash goes before each `{`, `}` and `<` there, a paragraph that MDX
// would read as an ES module has the first letter of its first word written as a character reference
// (`&#105;mport` shows as `import`), and an angle autolink, which MDX reads as a tag, is written as a link with the
// same text and target. Everything else is copied, which is the point of reading the structure from a parse and not
// from lines: a fence is found wherever it stands (in a list item, in a block quote), as are code spans across lines,
// math, link destinations and titles, bare addresses and escapes the original wrote, none of which a backslash
// may touch.
//
// - `>` stays. It is syntax in MDX only inside a tag, and a `<` written as `\<` opens none; at the start of a line it
//   is a block quote of the original, which encoding would turn into a line starting with a greater-than sign.
// - Footnote references `[^n]`, definitions `[^n]:`, headings and every Chinese character stay as they are.
// - A backslash that already escapes an ASCII punctuation character, `\{` or `\\` among them, is stepped over as the
//   pair, so an escape the original wrote is not doubled into a backslash and an open brace.
// - An HTML comment is masked before the grammar reads the page (`document-structure/src/mask-html-comments.ts`), so a
//   comment that closes is copied. One that never closes would hide the rest of the page from the package's own
//   parser, and no backslash stops that (the mask looks for the four characters and not for an escape), so its
//   opener is written as `&lt;!--`, which shows as the same characters and masks nothing.
//
// WHAT IS NOT ENCODED, AND WHY. A source run that parses as written ships as written, `<Foo />` and a balanced
// `{他说}` among it: the original is a page of an upstream MDX site that compiles, so what parses there renders there as
// it does upstream, and encoding it would show the reader a tag the site would have rendered. Only a run the grammar
// refuses (or that holds an unclosed comment) comes through here, and there every brace and every less-than sign that
// the parse reads as text is escaped, a component and a balanced expression included, since a run refused as a whole
// cannot be told apart piece by piece.
//
// LIMITS, kept because the composition in `source-ships-compose.ts` ends in shipping the page without the original
// for a text this does not make parse. The text of a node is escaped whole, so a backslash the original wrote before
// a character in indented code (which MDX reads as a paragraph, escapes included) is read as an escape and the reader
// sees the character without it. A raw HTML block that holds Markdown-looking lines is escaped as text and its lines
// are then read as Markdown. The reach of a bare address is taken as up to the next space or less-than sign, as the
// GFM autolink-literal pattern does. A text nested past the bound the plain reading keeps cannot be parsed, and is
// escaped as text throughout; the strict grammar refuses such a text all the same. An opener of a comment that
// never closes is written as a character reference wherever it stands, inside code too, where it shows as written.

/**
 What a comment that never closes is written as, in place of its opening `<`.
 */
const COMMENT_OPENER = '&lt;';

/**
 The code point of the byte order mark.
 */
const BYTE_ORDER_MARK_CODE_POINT = 0xFE_FF;

/**
 The character a text may start with that the parser drops from its input.
 */
const BYTE_ORDER_MARK = String.fromCodePoint(BYTE_ORDER_MARK_CODE_POINT,);

/**
 What opens an HTML comment.
 */
const COMMENT_OPEN = '<!--';

/**
 Characters a backslash goes before when the strict grammar reads them as syntax.
 */
const SYNTAX = '{}<';

/**
 What GFM reads as the start of an address with a protocol.
 */
const PROTOCOLS: readonly string[] = [
  'http://',
  'https://',
];

/**
 What GFM reads as the start of an address without one.
 */
const WWW = 'www.';

/**
 Characters that may stand before `www.` for GFM to read an address.
 */
const BEFORE_WWW = ' \t\n\r(*_[]~';

/**
 What a text holds that the encoder acts on.
 */
type TextReading = SourceSpans & {
  /**
   Comments that close, which the page's mask blanks and the encoder leaves as they are.
   */
  readonly closedComments: readonly TextRange[];
};

/**
 Whether a character is an ASCII letter.

 @param character - one character, or none

 @returns Whether it is `a` to `z` or `A` to `Z`

 @example
 ```ts
 const letter = isAsciiLetter({ character: 'a', },);
 ```
 */
function isAsciiLetter({ character, }: { readonly character: string; },): boolean {
  /**
   Whether it is a lower-case letter.
   */
  const lower = (character >= 'a') && (character <= 'z');
  /**
   Whether it is an upper-case letter.
   */
  const upper = (character >= 'A') && (character <= 'Z');
  return lower || upper;
}

/**
 Whether GFM reads an address starting at an index.

 @param text - the text

 @param at - index to look at

 @returns Whether a protocol or `www.` starts there after a character GFM accepts before it

 @example
 ```ts
 const starts = startsAddress({ text: '见 https://example.net', at: 2, },);
 ```
 */
function startsAddress(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  if (PROTOCOLS.some(function startsHere(protocol,): boolean {
    return text.startsWith(
      protocol,
      at,
    );
  },))
    return !isAsciiLetter({ character: text.charAt(at - 1,), },);
  return text.startsWith(
    WWW,
    at,
  ) && ((at === 0) || BEFORE_WWW.includes(text.charAt(at - 1,),));
}

/**
 Where an address ends: at the next space or less-than sign, as the GFM pattern reads it.

 @param text - the text

 @param from - index the address starts at

 @param limit - index the address cannot run past

 @returns Index of the first character that is not part of it, or the limit

 @example
 ```ts
 const stop = addressEnd({ text: 'https://example.net/a b', from: 0, limit: 23, },);
 ```
 */
function addressEnd(
  {
    text,
    from,
    limit,
  }: {
    readonly text: string;
    readonly from: number;
    readonly limit: number;
  },
): number {
  for (let at = from; at < limit; at += 1) {
    /**
     Character read.
     */
    const character = text.charAt(at,);
    if ((character === '<') || (character.trim() === ''))
      return at;
  }
  return limit;
}

/**
 Finds the bare addresses inside the stretches a reader sees as text.

 @param text - the text

 @param ranges - stretches read as text

 @returns Where each address stands, from its protocol to the next space or less-than sign

 @example
 ```ts
 const addresses = bareAddresses({ text: '见 https://example.net/a', ranges: [{ start: 0, end: 22, },], },);
 ```
 */
function bareAddresses(
  {
    text,
    ranges,
  }: {
    readonly text: string;
    readonly ranges: readonly TextRange[];
  },
): readonly TextRange[] {
  /**
   Addresses found.
   */
  const found: TextRange[] = [];
  for (const range of ranges) {
    for (let at = range.start; at < range.end; at += 1) {
      if (!startsAddress({
        text,
        at,
      },))
        continue;
      /**
       Index where the address ends.
       */
      const stop = addressEnd({
        text,
        from: at,
        limit: range.end,
      },);
      found.push({
        start: at,
        end: stop,
      },);
      at = stop - 1;
    }
  }
  return found;
}

/**
 Replaces every character of a text but its line endings with a space.

 @param text - text to blank

 @returns Text of the same length

 @example
 ```ts
 const spaces = spacesLike({ text: '<!-- a\n-->', },);
 ```
 */
function spacesLike({ text, }: { readonly text: string; },): string {
  return text
    .split('\n',)
    .map(function spaces(line,): string {
      return ' '.repeat(line.length,);
    },)
    .join('\n',);
}

/**
 Replaces the characters of each stretch with spaces and keeps the line endings.

 @param text - text to blank

 @param ranges - stretches to blank, in text order

 @returns Text of the same length

 @example
 ```ts
 const blanked = blanking({ text: 'a<!-- b -->c', ranges: [{ start: 1, end: 11, },], },);
 ```
 */
function blanking(
  {
    text,
    ranges,
  }: {
    readonly text: string;
    readonly ranges: readonly TextRange[];
  },
): string {
  /**
   Pieces of the result, joined at the end.
   */
  const pieces: string[] = [];
  /**
   Where the text not yet copied starts.
   */
  const copied = { upTo: 0, };
  for (const range of ranges) {
    pieces.push(
      text.slice(
        copied.upTo,
        range.start,
      ),
      spacesLike({
        text: text.slice(
          range.start,
          range.end,
        ),
      },),
    );
    copied.upTo = range.end;
  }
  pieces.push(text.slice(copied.upTo,),);
  return pieces.join('',);
}

/**
 Reads what a text holds, with its comments out of the way.

 @param text - text without a leading byte order mark

 @returns Stretches a reader sees as text, rewrites, comments that close, and the openers of comments that never close

 @example
 ```ts
 const reading = readText({ text: '她说 {', },);
 ```
 */
function readText({ text, }: { readonly text: string; },): TextReading {
  /**
   Comments of the text, as the package's mask finds them.
   */
  const regions = commentRegionsOf({ text, },);
  /**
   Comments that close, which the mask blanks and the page then does not hold.
   */
  const closedComments = regions
    .filter(function closes({ terminated, },): boolean {
      return terminated;
    },)
    .map(function toRange(region,): TextRange {
      return {
        start: region.startOffset,
        end: region.endOffset,
      };
    },);
  /**
   Openers of comments that never close, which the page would hide the rest of itself behind.
   */
  const openers = regions
    .filter(function open({ terminated, },): boolean {
      return !terminated;
    },)
    .map(function toOpener({ startOffset, },): TextEdit {
      // A backslash the original wrote before the opener goes with it, since `\&lt;` would show the reference.
      /**
       Whether the opener's `<` is preceded by a backslash that is not itself escaped.
       */
      const escaped = (text.charAt(startOffset - 1,) === '\\')
        && ((startOffset < 2) || (text.charAt(startOffset - 2,) !== '\\'));
      return {
        start: escaped ? (startOffset - 1) : startOffset,
        end: startOffset + 1,
        replacement: COMMENT_OPENER,
      };
    },);
  /**
   The text as the parse reads it: comments and the openers blanked, so what stands inside them is not parsed.
   */
  const readable = blanking({
    text,
    ranges: [
      ...closedComments,
      ...openers.map(function toBlank({ end, },): TextRange {
        return {
          start: end - 1,
          end: (end - 1) + COMMENT_OPEN.length,
        };
      },),
    ].toSorted(function byStart(
      left,
      right,
    ): number {
      return left.start - right.start;
    },),
  },);
  try {
    /**
     What the parse says of the readable text.
     */
    const spans = spansOf({
      source: text,
      root: parseMarkdownMathBody({ body: readable, },),
    },);
    return {
      text: spans.text,
      edits: [
        ...spans.edits,
        ...openers,
      ].toSorted(function byStart(
        left,
        right,
      ): number {
        return left.start - right.start;
      },),
      closedComments,
    };
  }
  catch (error) {
    // A text nested past the bound cannot be parsed; the strict grammar refuses it too, so it is escaped as text.
    if (!(error instanceof MarkdownParseError))
      throw error;
    return {
      text: [
        {
          start: 0,
          end: text.length,
        },
      ],
      edits: openers,
      closedComments,
    };
  }
}

/**
 Whether a backslash at an index escapes the character after it, both being characters the encoder may touch.

 @param text - the text

 @param at - index of the character read

 @param touchable - one mark per character, set where a backslash may stand

 @returns Whether the pair is an escape the original wrote

 @example
 ```ts
 const pair = escapesNext({ text: '\\{', at: 0, touchable: new Uint8Array([1, 1,],), },);
 ```
 */
function escapesNext(
  {
    text,
    at,
    touchable,
  }: {
    readonly text: string;
    readonly at: number;
    readonly touchable: Uint8Array;
  },
): boolean {
  if (text.charAt(at,) !== '\\')
    return false;
  if (touchable[at + 1] !== 1)
    return false;
  return ESCAPABLE.includes(text.charAt(at + 1,),);
}

/**
 Writes the text of a slice so that the strict MDX grammar reads it as literal text.

 A backslash goes before each `{`, `}` and `<` that a reader sees as text, and the rewrites the module comment lists
 are made; nothing else changes.

 @param text - slice text as the artifact records it

 @returns Text with those characters escaped, equal to the text when it holds none

 @example
 ```ts
 const literal = encodeAsLiteralMdx({ text: '她说 { 然后走了。', },);
 ```
 */
export function encodeAsLiteralMdx({ text, }: { readonly text: string; },): string {
  // The parser drops a leading byte order mark and every position then stands one short, so the mark is kept apart.
  if (text.startsWith(BYTE_ORDER_MARK,))
    return `${BYTE_ORDER_MARK}${encodeAsLiteralMdx({ text: text.slice(1,), },)}`;
  /**
   What the text holds.
   */
  const reading = readText({ text, },);
  /**
   Characters a backslash may go before: the stretches read as text, less the comments that close and the bare
   addresses.
   */
  const touchable = new Uint8Array(text.length,);
  for (const range of reading.text) {
    touchable.fill(
      1,
      range.start,
      range.end,
    );
  }
  for (const range of [
    ...reading.closedComments,
    ...bareAddresses({
      text,
      ranges: reading.text,
    },),
  ]) {
    touchable.fill(
      0,
      range.start,
      range.end,
    );
  }
  /**
   Index of the next edit to make, the edits being in text order and not overlapping.
   */
  const upcoming = { edit: 0, };
  /**
   Pieces of the encoded text, joined at the end.
   */
  const pieces: string[] = [];
  for (let at = 0; at < text.length; at += 1) {
    /**
     The edit starting here, when one does.
     */
    const edit = reading.edits[upcoming.edit];
    if ((edit !== undefined) && (edit.start === at)) {
      pieces.push(edit.replacement,);
      at = edit.end - 1;
      upcoming.edit += 1;
      continue;
    }
    /**
     Character read.
     */
    const character = text.charAt(at,);
    if (touchable[at] !== 1) {
      pieces.push(character,);
      continue;
    }
    // A BACKSLASH BEFORE PUNCTUATION IS AN ESCAPE ALREADY, and is stepped over as the pair it is.
    if (escapesNext({
      text,
      at,
      touchable,
    },)) {
      pieces.push(
        character,
        text.charAt(at + 1,),
      );
      at += 1;
      continue;
    }
    pieces.push(SYNTAX.includes(character,) ? `\\${character}` : character,);
  }
  return pieces.join('',);
}

//endregion Source ships encoding
