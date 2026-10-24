import {
  commentRegionsOf,
  parseMarkdownMathBody,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { firstNestingExcess, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  addressAsLink,
  bareAddresses,
} from './source-ships-addresses.ts';
import { indentedFenceMarkEnd, } from './source-ships-fence.ts';
import { withModuleLinesWritten, } from './source-ships-module-line.ts';
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
// `remark-mdx` lists the extension), a `<` as the start of a tag, and a line starting `import ` or `export ` as an ES
// module. The text is parsed once as CommonMark with the site's math and nothing else
// (`document-structure/src/parse-markdown-math.ts`), which never refuses a text, and the parse says where those
// characters stand in text a reader sees: a backslash goes before each `{`, `}` and `<` there, and an angle autolink,
// which MDX reads as a tag, is written as a link with the same text and target. Then the strict grammar's own tokenizer
// reads the text so written, and each line it starts an ES module at has its first letter written as a character
// reference (`&#105;mport` shows as `import`; `source-ships-module-line.ts`). Everything else is copied, which is the
// point of reading the structure from a parse and not from lines: a fence is found wherever it stands (in a list item,
// in a block quote), as are code spans across lines, math, link destinations and titles, bare addresses and escapes the
// original wrote, none of which a backslash may touch.
//
// WHERE THE PLAIN PARSE AND THE STRICT GRAMMAR READ A TEXT DIFFERENTLY (`source-ships-spans.ts` says how each is
// met). GFM is not on in the plain parse, so `[^1]: {` and `[^1]: <https://example.net/x>`, which CommonMark
// takes for link reference definitions, are footnotes to the strict grammar with a body to encode, and a block
// indented under a footnote definition is a block of the footnote and no indented code, so a paragraph that opens
// a footnote definition at the top, and a paragraph on the line right after one, are read as the body of the
// footnote is, each line giving up the four columns of indentation the footnote takes: a fence mark indented
// under a footnote is a fence of it, which is copied, and the footnote ends where the strict grammar ends it
// (`document-structure/src/gfm-block-ranges.ts`), so the lines after an unclosed fence that are not indented are
// text of the page and are written as text. A bare address that a less-than sign ends is written as a link with
// the same text and target (`source-ships-addresses.ts`), since a backslash written before the sign is read as
// part of the address and an entity written for it is read as part of the address too. An angle autolink inside
// the text of a link is written as its address alone, since a link cannot hold a link.
//
// - `>` stays. It is syntax in MDX only inside a tag, and a `<` written as `\<` opens none; at the start of a line it
//   is a block quote of the original, which encoding would turn into a line starting with a greater-than sign.
// - Footnote references `[^n]`, definitions `[^n]:`, headings and every Chinese character stay as they are.
// - A backslash that already escapes an ASCII punctuation character, `\{` or `\\` among them, is stepped over as the
//   pair, so an escape the original wrote is not doubled into a backslash and an open brace.
// - An HTML comment is masked before the grammar reads the page (`document-structure/src/mask-html-comments.ts`), so a
//   comment that closes is copied. One that never closes would hide the rest of the page from the package's own
//   parser, and no backslash stops that (the mask looks for the four characters and not for an escape), so its
//   opener, and each later `<!--` inside it, is written as `&lt;!--`, which shows as the same characters and masks
//   nothing (a second opener left in place opens a comment of its own, and the English after the run is hidden).
//
// WHAT IS NOT ENCODED, AND WHY. A source run that parses as written ships as written, `<Foo />` and a balanced
// `{他说}` among it: the original is a page of an upstream MDX site that compiles, so what parses there renders there
// as it does upstream, and encoding it would show the reader a tag the site would have rendered. Only a run the
// grammar refuses (or that holds an unclosed comment) comes through here, and there every brace and every less-than
// sign that the parse reads as text is escaped, a component and a balanced expression included, since a run refused
// as a whole cannot be told apart piece by piece.
//
// SHAPES THE PLAIN PARSE HIDES, and how each is met. The plain parse reads a node as one text and the strict grammar
// reads more in it, or reads the lines of a block as a paragraph. A raw HTML block and indented code are paragraphs to
// MDX, so every character Markdown reads as syntax in them is escaped and the reader sees the characters of the source
// (`source-ships-lines.ts`, `contentStretches`), and no address is read in them after an open bracket. A line
// CommonMark reads as a link reference definition right after a footnote definition, or right after a block that ends
// in a raw HTML block or indented code, is a lazy line of a paragraph to the strict grammar and is written as text
// (`source-ships-footnote.ts`, `source-ships-literal-end.ts`). A bare address after a bracket nothing closes is no
// address to the tokenizer, so its brace is escaped. A code span, inline math, or the title of a link across lines, a
// later one of which starts with `{`, `<` or a fence mark indented four spaces or more, is cut by the strict grammar,
// so the line ending and the marks before that line are written as one space. A fence mark indented four spaces or more
// is a fence to MDX and text to the plain parse, so it is escaped in text, and a fenced block holding one is written
// with a longer fence (`source-ships-fence.ts`, `source-ships-fenced.ts`). A code span that starts with a run of three
// backticks indented four spaces or more is joined to the line before it, as a later line that starts so is
// (`source-ships-lines.ts`). Two runs that declare one name, or a run holding an indented fence mark, are taken by the
// composition at its first step (`source-ships-company.ts`).
//
// THE MODULE LINES ARE NOT READ FROM THE PLAIN PARSE. They were, and each shape the plain parse and the strict
// grammar read differently (a line of invisible characters, a list marker alone after a footnote definition, a
// line after a setext underline that follows a restarted block) needed a rule of its own, and the next shape was
// always missing. The tokenizer says where a line opens a block, so the lines are taken from it, on the text with
// every other rewrite made (`document-structure/src/module-line-starts.ts`).
//
// LIMITS, kept because the composition in `source-ships-compose.ts` ends in shipping the page without the original for
// a text this does not make parse. The step search
// (`doc/audit-ledger-source-text-follow-ups-parser-gaps-and-step-searches.md` records its heads and counts) builds
// forty thousand pairs of runs from a pool of fragments; run with the seeds 31415926 and 27182818 it took step 3 for
// 4923 and 5115 pairs. A run nested past the bound the plain reading keeps (the 300-deep case in
// `source-shipped-page.unit.test.ts` pins it, and the pool holds it on purpose) is in 4876 and 5070 of them. The rest,
// 47 and 45 pairs, are not nested, and for 44 and 40 of them the cause is one limit, which is structural: a `$$` flow
// or a fence that one run opens and no line of the run closes, and a later run closes, with the English between them
// falling inside it and a tag after the closing line (each run of those pairs parses alone). The grammar leaves such a
// flow open to the end of the page (`micromark-extension-math@3.1.0`, `lib/math-flow.js`, `beforeContentChunk`: at the
// end of the input the flow ends with the document; `micromark-core-commonmark@2.0.4`, `lib/code-fenced.js`, the same
// for a fence), where a reader of the run alone sees it end with the run, and closing it would add a line the original
// does not hold. The fence case in `source-shipped-page.unit.test.ts` pins the behaviour. The other structural limit is
// nesting past the bound of 256 (`NESTING_BOUND` in `markup/src/nesting-bound.ts`, which refuses such a text before any
// parser reads it, and `withMdxExhaustionRefused` in `document-structure/src/parse-mdx.ts`, which answers a parse that
// ran out of stack as the same refusal); no writing of the text makes the grammar accept what it refuses by depth. Step
// 2 was taken for 383 and 412 pairs, all but 6 and 2 of each a nested one, and made the page parse for none. The pairs
// the classification of the search leaves unexplained are 3 and 5, and each of them is one of two limits named here: a
// tag alone on a line after a footnote's paragraph (3 and 3), and a byte order mark before a fence mark (0 and 2; the
// same pairs with the marks removed take step 1 alone). These counts are what two seeds measured on one pool; they say
// nothing of a shape the pool does not hold.
//
// A TAG ALONE ON A LINE BETWEEN A FOOTNOTE'S PARAGRAPH AND A DEFINITION LINE IS A LIMIT. The parse of the original
// reads such a tag as a raw HTML block, which ends the footnote's paragraph, so a line after it that reads as a link
// reference definition is a definition (and the encoder writes its braces as it writes any definition's). The page
// writes the tag as text, so that line is a lazy line of the footnote's paragraph, and its braces stay live: the page
// parses and holds an expression the source reader hides (`source-ships-encoding.unit.test.ts` pins the text). A parse
// of the original with each tag read as a letter gave the page's structure for this shape and moved other footnote
// texts the other way (96 of the generated footnote texts went from equal to unequal), so the reading is not changed.
//
// Smaller limits. A footnote definition indented four spaces or more inside a paragraph is a definition to MDX and a
// line of the paragraph to a reader of CommonMark with GFM (the indented code that would cap it is off in MDX): the
// page parses and the encoder writes the module line after a marker there, but the reader of the page and of the source
// differ in the marker (`source-ships-encoding.unit.test.ts` pins the text). The rest of a run after an opener of a
// comment that never closes is read on the page as Markdown, where the package's own mask would hide it, so the shown
// cases compare such a run up to the opener. The reach of a bare address is taken as up to the next space or less-than
// sign, as the GFM autolink-literal pattern does. An opener of a comment that never closes is written as a character
// reference wherever it stands, inside code too, where it shows as written. A fence indented under a footnote
// definition is copied only for a definition at the top. In a list item or a block quote the lines the nested reading
// takes carry the prefix of the container, which it does not strip, so the marks of such a fence are written with a
// backslash and the page shows them (`source-ships-encoding.unit.test.ts` pins both cases). A byte order mark before a
// fence mark makes the plain parse read the run as text and leaves a later fence of the same run open to the strict
// grammar. An angle autolink inside the alternative text of an image is written with a backslash before its less-than
// sign, as any text is, and the page shows the brackets there: the alternative text of an image is a string to the
// plain parse, which holds no autolink node to write as its address (`source-ships-encoding.unit.test.ts` pins the
// case). A table cuts a cell at every pipe, so an angle autolink that holds one is written as text there, as the source
// reads it; a bare address that a pipe ends, and an angle autolink holding a pipe written with a backslash, are not cut
// the same way, and the page shows one link across the cells (`source-ships-encoding.unit.test.ts` pins both).

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
 What the parse is given in place of a byte order mark that opens a text. The page carries the text after a note and a
 blank line, so the mark stands in the middle of the page: an ordinary character, after which the first line of the text
 starts no block, where the parser drops a mark that opens its input and reads the line as the start of the document.
 */
const LEADING_MARK_STAND_IN = 'x';

/**
 What opens an HTML comment.
 */
const COMMENT_OPEN = '<!--';

/**
 What the parse is given in place of an opener of a comment that never closes: the page carries `&lt;!--`, text that
 starts with an ampersand, which a line starting with it makes a paragraph and which keeps the class of the less-than
 sign among emphasis marks. Four spaces would make the line a blank one, or indented code, and a definition on the next
 line would stay a definition to the parse where it is a line of the paragraph to the strict grammar.
 */
const COMMENT_OPENER_STAND_IN = '&!--';

/**
 Characters a backslash goes before when the strict grammar reads them as syntax.
 */
const SYNTAX = '{}<';

/**
 Characters a backslash goes before in a raw HTML block or indented code, where the strict grammar reads a paragraph:
 those that start inline syntax or a block (a backslash, a code span, emphasis, a link, an image, an entity, a tag, a
 table, strikethrough, math, a heading, a list item, a quote, a setext underline) and the braces.
 */
const LITERAL_SYNTAX = '\\`*_{}[]#+-.!<>&|~$=)';

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
 Writes the stand-in of an opener over each place an opener stands.

 @param text - text

 @param starts - index of the `<` of each opener, in text order

 @returns Text of the same length

 @example
 ```ts
 const readable = standingIn({ text: 'a <!-- b', starts: [2,], },);
 ```
 */
function standingIn(
  {
    text,
    starts,
  }: {
    readonly text: string;
    readonly starts: readonly number[];
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
  for (const start of starts) {
    pieces.push(
      text.slice(
        copied.upTo,
        start,
      ),
      COMMENT_OPENER_STAND_IN,
    );
    copied.upTo = start + COMMENT_OPEN.length;
  }
  pieces.push(text.slice(copied.upTo,),);
  return pieces.join('',);
}

/**
 Where each `<!--` of a text stands from an index on.

 @param text - text

 @param from - index to look from

 @returns Index of the `<` of each, in text order

 @example
 ```ts
 const starts = openerStarts({ text: '<!-- a <!-- b', from: 0, },);
 ```
 */
function openerStarts(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): readonly number[] {
  /**
   Indices found.
   */
  const starts: number[] = [];
  for (
    let at = text.indexOf(
      COMMENT_OPEN,
      from,
    );
    at !== (-1);
    at = text.indexOf(
      COMMENT_OPEN,
      at + COMMENT_OPEN.length,
    )
  )
    starts.push(at,);
  return starts;
}

/**
 Gives a text to the parse as the page holds it: a byte order mark that opens it is an ordinary character there.

 @param text - text

 @returns Text of the same length, with a letter in place of a mark that opens it

 @example
 ```ts
 const given = asTheParseReadsIt({ text: '\uFEFF- a', },);
 ```
 */
function asTheParseReadsIt({ text, }: { readonly text: string; },): string {
  return text.startsWith(BYTE_ORDER_MARK,) ? `${LEADING_MARK_STAND_IN}${text.slice(1,)}` : text;
}

/**
 Reads what a text holds, with its comments out of the way.

 @param text - text

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
   Openers of comments that never close, which the page would hide the rest of itself behind: the one that starts such a
   comment and each later one inside it, since the mask opens a comment of its own at every one that stays.
   */
  const openers = regions
    .filter(function open({ terminated, },): boolean {
      return !terminated;
    },)
    .flatMap(function openersIn({ startOffset, },): readonly number[] {
      return openerStarts({
        text,
        from: startOffset,
      },);
    },)
    .map(function toOpener(startOffset,): TextEdit {
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
   The text as the parse reads it: comments that close blanked, so what stands inside them is not parsed, and the
   openers of comments that never close written as text.
   */
  const readable = asTheParseReadsIt({
    text: standingIn({
      text: blanking({
        text,
        ranges: closedComments,
      },),
      starts: openers.map(function toStart({ end, },): number {
        return end - 1;
      },),
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
      literal: [],
      edits: openers,
      insideLinks: [],
      streamStarts: [],
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
    literal: spans.literal,
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
    streamStarts: spans.streamStarts,
    closedComments,
  };
}

/**
 Whether a backslash at an index escapes the character after it, both being characters the encoder may touch.

 @param text - text

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
    streamStarts: reading.streamStarts,
    literal: reading.literal,
  },);
  /**
   Characters a backslash may go before: the stretches read as text, less
   the comments that close and the bare addresses.
   */
  const touchable = new Uint8Array(text.length,);
  /**
   One mark per character, set where every character Markdown reads as syntax is escaped.
   */
  const literal = new Uint8Array(text.length,);
  for (const range of reading.text) {
    touchable.fill(
      1,
      range.start,
      range.end,
    );
  }
  for (const range of reading.literal) {
    touchable.fill(
      1,
      range.start,
      range.end,
    );
    literal.fill(
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
    // IN A RAW HTML BLOCK OR INDENTED CODE a backslash is a character of the source, and is escaped as any other.
    if (literal[at] === 1) {
      pieces.push(LITERAL_SYNTAX.includes(character,) ? `\\${character}` : character,);
      continue;
    }
    // A FENCE MARK INDENTED FOUR SPACES OR MORE opens a code block in the page and is text in the source, so each of
    // its characters is escaped.
    /**
     Index after the indented fence mark that starts here, `at` when none does.
     */
    const markEnd = indentedFenceMarkEnd({
      source: text,
      at,
    },);
    if ((markEnd > at) && (touchable[markEnd - 1] === 1)) {
      /**
       Characters of the mark.
       */
      const marks = text.slice(
        at,
        markEnd,
      );
      for (const mark of marks)
        pieces.push(`\\${mark}`,);
      at = markEnd - 1;
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
  return withModuleLinesWritten({ text: pieces.join('',), },);
}

//endregion Source ships encoding
