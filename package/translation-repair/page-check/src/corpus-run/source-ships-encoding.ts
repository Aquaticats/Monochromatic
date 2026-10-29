import {
  commentRegionsOf,
  footnoteDefinitionRangesOf,
  pageFootnoteDefinitionRangesOf,
  parseMarkdownMathBody,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { firstNestingExcess, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  addressAsLink,
  bareAddresses,
} from './source-ships-addresses.ts';
import { indentedFenceMarkEnd, } from './source-ships-fence.ts';
import { blockOpenerStarts, } from './source-ships-open-block.ts';
import {
  openerStarts,
  rangesWithoutOpeners,
} from './source-ships-open-comment.ts';
import { rootWithFreshTails, } from './source-ships-fresh-tails.ts';
import { withModuleLinesWritten, } from './source-ships-module-line.ts';
import {
  closedWith,
  closerOfWritten,
  NO_MARK,
} from './source-ships-open-flow.ts';
import { editsWithOpeners, } from './source-ships-opener-in-code.ts';
import { htmlInterruptionEdits, } from './source-ships-html-interrupt.ts';
import {
  asTheParseReadsIt,
  blanking,
  foldLoneCarriageReturns,
  standingIn,
} from './source-ships-readable.ts';
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
// met). Footnotes are not on in the plain parse (it has the tables and the strikethrough of GFM and no other part
// of it), so `[^1]: {` and `[^1]: <https://example.net/x>`, which CommonMark
// takes for link reference definitions, are footnotes to the strict grammar with a body to encode, and a block
// indented under a footnote definition is a block of the footnote and no indented code, so a paragraph that opens
// a footnote definition at the top, and a paragraph on the line right after one, are read as the body of the
// footnote is, each line giving up the four columns of indentation the footnote takes: a fence mark indented
// under a footnote is a fence of it, which is copied, and the footnote ends where the strict grammar ends it
// (`document-structure/src/gfm-block-ranges.ts`), so the lines after an unclosed fence that are not indented are
// text of the page and are written as text. The last block of a footnote that is a raw HTML block, or indented code,
// is a paragraph to the strict grammar, so a link reference definition that follows the footnote on the next line
// would be a lazy line of that paragraph and its braces an expression: the line ending that ends the footnote is
// written twice, and the definition is one on the page as it is in the original (`source-ships-nested.ts`). A bare
// address that a less-than sign ends is written as a link with
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
// A LONE CARRIAGE RETURN IS FOLDED TO A LINE FEED BEFORE ANYTHING IS READ (`source-ships-readable.ts`,
// `foldLoneCarriageReturns`). The grammar and the plain parse read it as a line ending and every reader here splits
// lines on the line feed, so a text of such endings was one line to them: a footnote's separator and the closing line
// of a flow were missed, an expression leaked on the page and a closed flow was closed twice. Folding at the one entry
// keeps the length, so every offset stands, and leaves Windows line endings as they are. The text a run that parses as
// written ships is not encoded and keeps whatever endings it has; the corpus reader folds Windows endings and leaves a
// lone carriage return alone (`slice-reading/src/line-endings.ts`).
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
// forty thousand pairs of runs from a pool of fragments, and its record names the pairs that reach the third step.
// The limit that remains is structural: nesting past the bound of 256 (`NESTING_BOUND` in
// `markup/src/nesting-bound.ts`, which refuses such a text before any parser reads it, and `withMdxExhaustionRefused`
// in `document-structure/src/parse-mdx.ts`, which answers a parse that ran out of stack as the same refusal); no
// writing of the text makes the grammar accept what it refuses by depth, and the 300-deep case in
// `source-shipped-page.unit.test.ts` pins it (the pool holds it on purpose).
//
// A FENCE OR A `$$` FLOW THAT A RUN OPENS AND NO LINE OF THE RUN CLOSES was a structural limit and is closed. The
// grammar leaves such a flow open to the end of the page (`micromark-extension-math@3.1.0`, `lib/math-flow.js`,
// `beforeContentChunk`: at the end of the input the flow ends with the document; `micromark-core-commonmark@2.0.4`,
// `lib/code-fenced.js`, the same for a fence), so the English between two runs fell inside it, and a line of the later
// run that looks like its closing line ended it and left the text after that line outside, where a brace or a tag is
// code. A reader of the run alone sees the flow end with the run, so a run written as literal text ends with the
// closing line of its flow (`source-ships-open-flow.ts`), and the composition takes a run that ends so at its first
// step (`source-ships-company.ts`). The closing line shows nothing: a fence and a display math read the same with it.
//
// THE WHOLE-PAGE CHECK reads the page the composition ships and finds every expression, ES module and element the
// strict grammar builds in the text of a slice written as literal text (`source-live-constructs.test-fixture.ts`);
// the original, read by the package's own grammar with GFM and the site's math, builds none, so there must be none.
// It reads the cases of `source-ships-encoding.unit.test.ts` and, once for each state of the code, the generated
// texts and the pairs of the step search.
//
// Smaller limits. A footnote definition indented four spaces or more inside a paragraph is a definition to MDX and a
// line of the paragraph to a reader of CommonMark with GFM (the indented code that would cap it is off in MDX): the
// page parses and the encoder writes the module line after a marker there, but the reader of the page and of the source
// differ in the marker (`source-ships-encoding.unit.test.ts` pins the text). The rest of a run after an opener of a
// comment that never closes is read on the page as Markdown, where the package's own mask would hide it, so the shown
// cases compare such a run up to the opener. The reach of a bare address is taken as up to the next space or less-than
// sign, as the GFM autolink-literal pattern does. An opener of a comment that never closes is written as a character
// reference, and where it stands in a fence, a display math, a code span or inline math what holds it is taken out of the
// code first (`source-ships-opener-in-code.ts`), since the reference shows as written in code; one in a code span that only
// the reading of a footnote in a block quote pairs is not found, and the comparison reports the page. A fence indented
// under a footnote definition is copied, at the top and in a block quote or a list item: the lines the nested reading
// takes carry the prefix of the container, which it strips with the four columns the footnote takes
// (`source-ships-footnote-container.ts`; `source-ships-encoding.unit.test.ts` pins both cases). A byte order mark that
// opens a run is an ordinary character on the page, where a fence mark after it opens no fence, so the encoder reads
// the run with a letter in its place (`source-ships-readable.ts`), and a fence that this leaves open at the end of the
// run is closed like any other. An angle autolink inside the alternative text of an image is written as its address
// alone, and a code span in it is copied as it stands, since the alternative text of an image is a string to the plain
// parse, which holds the address of an autolink and the content of a code span in it
// (`source-ships-image-alt.ts`; an alternative text that is not one paragraph on its own, a line of a block quote
// without its mark that reads as an underline on its own among them, is written as text). A table cuts a cell at every pipe, so the plain reading, which
// reads tables, cuts an address there as the source does, and a pipe written with a backslash in an address written as a
// link is written with a backslash in its text and its target (`source-ships-links.ts`). A footnote label that holds a brace or a less-than sign is
// written with a backslash in the reference, as any text is, and left as it stands in the definition, where the page
// holds no expression for it: the two no longer name one footnote, and the page shows the
// reference as text (`source-ships-encoding.unit.test.ts` pins three labels). Writing the same backslash into both
// labels would make them agree again, and has not been done.

/**
 What a comment that never closes is written as, in place of its opening `<`.
 */
const COMMENT_OPENER = '&lt;';

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
 What a backtick is written as where it is text a reader of the source sees and the page's grammar must not read as the
 delimiter of a code span. A backslash does not stop that: a code span is searched for in the raw text, so a lone backtick
 on a line before a run of backticks written with a backslash before each pairs with the first and makes a code span of
 the lines between, which shows the backslash, and a backslash before the first only leaves a run that pairs with another
 of its size. A character reference holds no backtick in the text.
 */
const BACKTICK_REFERENCE = '&#96;';

/**
 Writes a character of a raw HTML block, of indented code or of an indented fence mark so that the page shows it as text.

 @param character - one character of the text

 @returns Character with a backslash before it where Markdown reads it as syntax, a backtick as a character reference

 @example
 ```ts
 const written = writtenLiteral({ character: '*', },); // `\*`
 ```
 */
function writtenLiteral({ character, }: { readonly character: string; },): string {
  if (character === '`')
    return BACKTICK_REFERENCE;
  return LITERAL_SYNTAX.includes(character,) ? `\\${character}` : character;
}

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
   Index of the `<` of each opener of a comment that never closes, in text order.
   */
  const openerStartsInText = openers.map(function toStart({ end, },): number {
    return end - 1;
  },);
  /**
   The text with its comments that close blanked, so what stands inside them is not parsed.
   */
  const blanked = blanking({
    text,
    ranges: closedComments,
  },);
  /**
   The openers that start a block of raw HTML to a reader of the source, which the parse is given as they stand.
   */
  const blockStarts = blockOpenerStarts({
    text: blanked,
    starts: openerStartsInText,
  },);
  /**
   The text as the parse reads it: comments that close blanked, and the openers of comments that never close written as
   text, except those that start a block of raw HTML.
   */
  const readable = asTheParseReadsIt({
    text: standingIn({
      text: blanked,
      starts: openerStartsInText.filter(function notRawHtml(start,): boolean {
        return !blockStarts.includes(start,);
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
   The parse of the readable text, with the tails of the footnotes read on their own.
   */
  const root = rootWithFreshTails({
    readable,
    root: parseMarkdownMathBody({ body: readable, },),
  },);
  /**
   What the parse says of the readable text.
   */
  const spans = spansOf({
    source: text,
    readable,
    root,
  },);
  /**
   The blocks of raw HTML that cut a paragraph, which the page cuts too.
   */
  const interruption = htmlInterruptionEdits({
    readable,
    root,
  },);
  /**
   Where a footnote definition stands for a reader of the source and for the page, which the openers in code are written
   around; none where the text holds no opener.
   */
  const footnotes = (openerStartsInText.length === 0)
    ? []
    : [
      ...footnoteDefinitionRangesOf({ body: readable, },),
      ...pageFootnoteDefinitionRangesOf({ body: readable, },),
    ];
  return {
    text: spans.text,
    literal: rangesWithoutOpeners({
      ranges: spans.literal,
      starts: openerStartsInText,
    },),
    edits: editsWithOpeners({
      source: text,
      root,
      starts: openerStartsInText,
      edits: [
        ...spans.edits,
        ...interruption.edits,
      ],
      openers,
      footnotes,
      closed: closedComments,
    },),
    insideLinks: spans.insideLinks,
    streamStarts: [
      ...spans.streamStarts,
      ...interruption.starts,
    ],
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
 Drops an edit that repeats the one before it in a list in text order.

 A block the readers reach twice, as the paragraph a footnote opens in is, puts its edit twice, and the second would
 stop every edit after it from being made, as the edits must not overlap.

 @param edits - edits in text order

 @returns Edits with each run of identical ones cut to its first

 @example
 ```ts
 const once = withoutRepeatedEdits({ edits: [{ start: 9, end: 10, replacement: '\n\n', }, { start: 9, end: 10, replacement: '\n\n', },], },);
 ```
 */
function withoutRepeatedEdits({ edits, }: { readonly edits: readonly TextEdit[]; },): readonly TextEdit[] {
  /**
   The edits kept.
   */
  const kept: TextEdit[] = [];
  /**
   Start, end and replacement of the edit kept last, joined, which no two different edits share since the start and the end are numbers.
   */
  const lastKey = { value: '', };
  for (const edit of edits) {
    /**
     Start, end and replacement of this edit, joined.
     */
    const key = `${edit.start}:${edit.end}:${edit.replacement}`;
    if ((kept.length > 0) && (key === lastKey.value))
      continue;
    lastKey.value = key;
    kept.push(edit,);
  }
  return kept;
}

/**
 Writes a backslash before each `{`, `}` and `<` of the text that a reader sees as text, and the other rewrites that
 are made in place.

 @param text - slice text as the artifact records it, a lone carriage return among its line endings included

 @returns Text with those characters escaped and every lone carriage return written as a line feed, equal to the text
 when it holds none

 @example
 ```ts
 const escaped = escapedText({ text: '她说 { 然后走了。', },);
 ```
 */
function escapedText({ text: recorded, }: { readonly text: string; },): string {
  // THE ONE PLACE A LONE CARRIAGE RETURN IS READ: the readers of this module split lines on the line feed and the
  // grammar reads a carriage return as a line ending, so a text of such endings was read as a single line, its
  // footnote separator and the closing line of its flow were missed, and the page held an expression the original
  // lacked.
  /**
   The text with every lone carriage return written as a line feed, of the same length.
   */
  const text = foldLoneCarriageReturns({ text: recorded, },);
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
   The edits, with the links that bare addresses a less-than sign ends become, in text order.
   */
  const allEdits = [
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
   The same edits, each made once.
   */
  const edits = withoutRepeatedEdits({ edits: allEdits, },);
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
      pieces.push(writtenLiteral({ character, },),);
      continue;
    }
    // A FENCE MARK INDENTED FOUR SPACES OR MORE opens a code block in the page and is text in the source, so each of
    // its characters is written as text (a backtick as a character reference, see `BACKTICK_REFERENCE`).
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
        pieces.push(writtenLiteral({ character: mark, },),);
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
  return pieces.join('',);
}

/**
 Whether a text, once written as literal text, ends inside a fence or display math, which the page does not end where
 the text does: the opening line of the next run closes it.

 @param text - text as the artifact records it

 @returns Whether encoding the text closes a fence or display math

 @example
 ```ts
 const open = leavesFlowOpen({ text: '$$\nx', },);
 ```
 */
export function leavesFlowOpen({ text, }: { readonly text: string; },): boolean {
  return closerOfWritten({ text: escapedText({ text, },), },) !== NO_MARK;
}

/**
 Writes the text of a slice so that the strict MDX grammar reads it as literal text.

 A backslash goes before each `{`, `}` and `<` that a reader sees as text, a fence or display math the text leaves open
 at its end is closed, and the rewrites the module comment lists are made; nothing else changes.

 @param text - slice text as the artifact records it

 @returns Text with those characters escaped, equal to the text when it holds none

 @example
 ```ts
 const literal = encodeAsLiteralMdx({ text: '她说 { 然后走了。', },);
 ```
 */
export function encodeAsLiteralMdx({ text, }: { readonly text: string; },): string {
  /**
   The text with its escapes made.
   */
  const escaped = escapedText({ text, },);
  return withModuleLinesWritten({
    text: closedWith({
      text: escaped,
      closer: closerOfWritten({ text: escaped, },),
    },),
  },);
}

//endregion Source ships encoding
