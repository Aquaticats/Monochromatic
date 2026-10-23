import {
  commentRegionsOf,
  parseMarkdownMathBody,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { firstNestingExcess, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  addressAsLink,
  bareAddresses,
} from './source-ships-addresses.ts';
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
// characters stand in text a reader sees: a backslash goes before each `{`, `}` and `<` there, a paragraph or setext
// heading that MDX would read as an ES module has the first letter of its first word written as a character reference
// (`&#105;mport` shows as `import`), and an angle autolink, which MDX reads as a tag, is written as a link with the
// same text and target. Everything else is copied, which is the point of reading the structure from a parse and not
// from lines: a fence is found wherever it stands (in a list item, in a block quote), as are code spans across lines,
// math, link destinations and titles, bare addresses and escapes the original wrote, none of which a backslash may
// touch.
//
// WHERE THE PLAIN PARSE AND THE STRICT GRAMMAR READ A TEXT DIFFERENTLY (`source-ships-spans.ts` says how each is met).
// GFM is not on in the plain parse, so `[^1]: {` and `[^1]: <https://example.net/x>`, which CommonMark takes for link
// reference definitions, are footnotes to the strict grammar with a body to encode, and a block indented under a
// footnote definition is a block of the footnote and no indented code. A bare address that a less-than sign ends is
// written as a link with the same text and target (`source-ships-addresses.ts`), since a backslash written before the
// sign is read as part of the address and an entity written for it is read as part of the address too. An angle
// autolink inside the text of a link is written as text, since a link cannot hold a link.
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
// `{他说}` among it: the original is a page of an upstream MDX site that compiles, so what parses there renders there
// as it does upstream, and encoding it would show the reader a tag the site would have rendered. Only a run the
// grammar refuses (or that holds an unclosed comment) comes through here, and there every brace and every less-than
// sign that the parse reads as text is escaped, a component and a balanced expression included, since a run refused
// as a whole cannot be told apart piece by piece.
//
// LIMITS, kept because the composition in `source-ships-compose.ts` ends in shipping the page without the original for
// a text this does not make parse. A seeded search of forty thousand pairs of runs built from a pool of fragments took
// step 3 for 6624 pairs; its causes were a run nested past the bound the plain reading keeps (the 300-deep case in
// `source-shipped-page.unit.test.ts` pins it), a `$$` flow one run opens and a later run closes with the English
// between them falling inside it and a tag after the closing line (a fence does the same, which the fence case in that
// file pins), a raw HTML block that runs to a closing marker (`<script>`, a comment never closed), a fence mark
// indented four spaces or more (MDX reads a fence there, the plain parse a paragraph or indented code), and the shapes
// under SHAPES THE PLAIN PARSE HIDES. Step 2 was taken for 838 pairs and made the page parse for 32 of them (a fence
// mark indented in both runs; one pair of module lines with a bare address); in the rest it was taken and did not, a
// run nested past the bound beside a run holding a character step 2 escapes among them.
//
// SHAPES THE PLAIN PARSE HIDES. These are texts the encoder does not make parse: the plain parse reads a node as one
// text and the strict grammar reads more in it. A raw HTML block or indented code holding
// `[https://example.net/a{ b](u)` has its address taken for a bare one. A line CommonMark reads as a link reference
// definition right after a footnote definition is a line of the footnote to the strict grammar. A bare address after a
// bracket nothing closes is not read as an address by the strict grammar. A code span or inline math across lines, a
// later one of which starts with `{` or `<`, is cut by the strict grammar. A byte order mark line before a setext
// heading starting `import` hides the module line from the plain parse.
//
// Smaller limits. The text of a node is escaped whole, so a backslash the original wrote before a character in indented
// code (which MDX reads as a paragraph, escapes included) is read as an escape and the reader sees the character
// without it. A raw HTML block that holds Markdown-looking lines is escaped as text and its lines are then read as
// Markdown. The reach of a bare address is taken as up to the next space or less-than sign, as the GFM autolink-literal
// pattern does. An opener of a comment that never closes is written as a character reference wherever it stands, inside
// code too, where it shows as written.

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
 What a text holds that the encoder acts on.
 */
type TextReading = SourceSpans & {
  /**
   Comments that close, which the page's mask blanks and the encoder leaves as they are.
   */
  readonly closedComments: readonly TextRange[];
};

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
  // A TEXT NESTED PAST THE BOUND OF THE PLAIN READING IS NOT PARSED, since the reading refuses it before it reads it;
  // the strict grammar refuses such a text too, so every stretch of it outside a comment is escaped as text. This is a
  // branch on the reading's own predicate and not a caught error, since the callers (the judgement of a page on disk
  // among them) hold no logger and a text nested that deep is an expected input, not a fault.
  if (firstNestingExcess({
    body: readable,
    grammar: 'markdown',
  },)
    .kind
    === 'beyond') {
    return {
      text: [
        {
          start: 0,
          end: text.length,
        },
      ],
      edits: openers,
      insideLinks: [],
      closedComments,
    };
  }
  /**
   What the parse says of the readable text.
   */
  const spans = spansOf({
    source: text,
    readable,
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
    insideLinks: spans.insideLinks,
    closedComments,
  };
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
   Bare addresses of the text.
   */
  const addresses = bareAddresses({
    text,
    ranges: reading.text,
    insideLinks: reading.insideLinks,
  },);
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
    ...addresses,
  ]) {
    touchable.fill(
      0,
      range.start,
      range.end,
    );
  }
  /**
   The edits, with the links that bare addresses a less-than sign ends become.
   */
  const edits = [
    ...reading.edits,
    ...addresses
      .filter(function endsAtTag({ endsAtTag: ends, },): boolean {
        return ends;
      },)
      .map(function asLink(address,): TextEdit {
        return addressAsLink({
          text,
          address,
        },);
      },),
  ].toSorted(function byStart(
    left,
    right,
  ): number {
    return left.start - right.start;
  },);
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
    const edit = edits[upcoming.edit];
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
