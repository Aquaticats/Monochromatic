import type { ArchiveOriginalSpan, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { wholeOpening, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import {
  blockquoteParagraphs,
  type LineRange,
  paragraphWords,
  splitQuotedLine,
} from './blockquote-paragraphs.ts';
import {
  rewriteEverySlice,
  type TextRewrite,
} from './page-slice-rewrite.ts';

//region Blockquote quote unify
// CLASS ONE HUNDRED SIXTY-EIGHT (TianqiChen6667, 2026-09-26): the original
// quotes each of her messages with 「」 and the archive sets every one as a
// bare blockquote, yet the page wrapped one blockquote's two paragraphs in
// “…” and left the other nine bare, so one page quoted its messages two ways.
// The blockquote already marks the words as hers, which is why the archive
// drops the marks. This page-assembly pass reads the archive's blockquote
// paragraphs and, where bare ones outnumber quoted ones, unwraps each page
// blockquote paragraph that is one quotation from its first mark to its last.
// A paragraph holding any other mark of the pair (a quotation inside it, or
// two quotations) stays as written, since which marks pair is then not the
// paragraph's edge. Only the unwrapping direction is taken: adding marks
// would need to know which blockquotes are speech.

/**
 One pair of quotation marks a blockquote paragraph may be wrapped in.
 */
type QuotePair = {
  readonly open: string;
  readonly close: string;
};

/**
 Pairs read as wrapping: curly doubles, and straight doubles for a page the
 quote-style pass left straight.
 */
const QUOTE_PAIRS: readonly QuotePair[] = [
  {
    open: '“',
    close: '”',
  },
  {
    open: '"',
    close: '"',
  },
];

/**
 Marks a lone quotation in straight doubles carries, one at each edge.
 */
const STRAIGHT_LONE_MARKS = 2;

/**
 Most UTF-16 units of a paragraph's opening a finding quotes, ending on a
 whole character (`wholeOpening`). Named as units, not words: the name
 `NOTE_WORDS` promised words and the cut counted units (ledger B21).
 */
const NOTE_OPENING_UNITS = 40;

/**
 Whether a paragraph's words open with a pair's opening mark and close with
 its closing mark.

 @param words - paragraph words

 @param pair - quotation marks to test

 @returns Whether the pair wraps the words

 @example
 ```ts
 wraps({ words: '“Purr.”', pair: QUOTE_PAIRS[0], },); // true
 ```
 */
function wraps(
  {
    words,
    pair,
  }: {
    readonly words: string;
    readonly pair: QuotePair;
  },
): boolean {
  /**
   Words without their edge spaces.
   */
  const trimmed = words.trim();
  return (trimmed.length > 1)
    && trimmed.startsWith(pair.open,)
    && trimmed.endsWith(pair.close,);
}

/**
 Counts one mark in a text.

 @param text - text to scan

 @param mark - one code unit

 @returns How often the mark stands in the text

 @example
 ```ts
 countMark({ text: '“a” “b”', mark: '“', },); // 2
 ```
 */
function countMark(
  {
    text,
    mark,
  }: {
    readonly text: string;
    readonly mark: string;
  },
): number {
  /**
   Text cut at every mark.
   */
  const pieces = text.split(mark,);
  return pieces.length - 1;
}

/**
 Whether a paragraph's words are one quotation in a pair: wrapped by it, and
 carrying no other mark of it.

 @param words - paragraph words

 @param pair - quotation marks to test

 @returns Whether unwrapping the pair leaves no mark of it behind

 @example
 ```ts
 loneQuotation({ words: '“Purr.”', pair: QUOTE_PAIRS[0], },); // true
 ```
 */
function loneQuotation(
  {
    words,
    pair,
  }: {
    readonly words: string;
    readonly pair: QuotePair;
  },
): boolean {
  if (!wraps({
    words,
    pair,
  },))
    return false;
  /**
   Opening marks of the pair.
   */
  const opens = countMark({
    text: words,
    mark: pair.open,
  },);
  /**
   Closing marks of the pair.
   */
  const closes = countMark({
    text: words,
    mark: pair.close,
  },);
  return pair.open === pair.close ? (opens === STRAIGHT_LONE_MARKS) : ((opens === 1) && (closes === 1));
}

/**
 Whether the archive sets its blockquote paragraphs bare more often than
 quoted.

 @param slices - prepared pairs, whose target text is the archive's

 @returns Whether bare paragraphs outnumber quoted ones

 @example
 ```ts
 archiveQuotesBare({ slices, },); // true
 ```
 */
function archiveQuotesBare({ slices, }: { readonly slices: readonly ChunkPair[]; },): boolean {
  /**
   Whether each archive blockquote paragraph is quoted.
   */
  const quoted = slices
    .filter(function isBody(slice,): boolean {
      return slice.syntax !== 'front-matter';
    },)
    .flatMap(function readSlice(slice,): readonly boolean[] {
      /**
       Archive lines of this slice.
       */
      const lines = slice.target
        .text
        .split('\n',);
      /**
       Blockquote paragraphs of this slice.
       */
      const ranges = blockquoteParagraphs({ lines, },);
      return ranges.map(function isQuoted(range,): boolean {
        /**
         Words of this paragraph.
         */
        const words = paragraphWords({
          lines,
          range,
        },);
        return QUOTE_PAIRS.some(function wrapsWords(pair,): boolean {
          return wraps({
            words,
            pair,
          },);
        },);
      },);
    },);
  /**
   Quoted paragraphs in the archive.
   */
  const quotedCount = quoted
    .filter(Boolean,)
    .length;
  /**
   Bare paragraphs in the archive.
   */
  const bareCount = quoted.length - quotedCount;
  return bareCount > quotedCount;
}

/**
 Removes a pair's opening mark from a paragraph's first line and its closing
 mark from the paragraph's last.

 @param lines - text split at line feeds

 @param range - lines of the paragraph

 @param pair - quotation marks wrapping it

 @returns Lines with the paragraph unwrapped

 @example
 ```ts
 unwrapWith({ lines: ['> “Purr.”',], range: { first: 0, last: 0, }, pair: QUOTE_PAIRS[0], },); // ['> Purr.']
 ```
 */
function unwrapWith(
  {
    lines,
    range,
    pair,
  }: {
    readonly lines: readonly string[];
    readonly range: LineRange;
    readonly pair: QuotePair;
  },
): readonly string[] {
  return lines.map(function unwrapLine(
    line,
    index,
  ): string {
    if ((index < range.first) || (index > range.last))
      return line;
    /**
     This line's markers and words.
     */
    const {
      prefix,
      content,
    } = splitQuotedLine({ line, },);
    /**
     Where the closing mark stands on the last line.
     */
    const closeAt = content.lastIndexOf(pair.close,);
    /**
     Words with the closing mark cut from the last line; cut first so a
     straight pair's opening mark is still the first of its kind.
     */
    const closed = index === range.last
      ? `${content.slice(
        0,
        closeAt,
      )}${content.slice(closeAt + 1,)}`
      : content;
    /**
     Words with the opening mark cut from the first line.
     */
    const opened = index === range.first
      ? closed.replace(
        pair.open,
        '',
      )
      : closed;
    return `${prefix}${opened}`;
  },);
}

/**
 Unwraps one paragraph where it is one quotation in a pair.

 @param lines - text split at line feeds

 @param range - lines of the paragraph

 @returns Lines with the paragraph unwrapped, or the same lines

 @example
 ```ts
 unwrapParagraph({ lines: ['> “Purr.”',], range: { first: 0, last: 0, }, },); // ['> Purr.']
 ```
 */
function unwrapParagraph(
  {
    lines,
    range,
  }: {
    readonly lines: readonly string[];
    readonly range: LineRange;
  },
): readonly string[] {
  /**
   Paragraph words joined.
   */
  const words = paragraphWords({
    lines,
    range,
  },);
  // At most one pair fits, since the paragraph opens with one character.
  return QUOTE_PAIRS
    .filter(function fits(pair,): boolean {
      return loneQuotation({
        words,
        pair,
      },);
    },)
    .reduce<readonly string[]>(
    function unwrap(
      current,
      pair,
    ): readonly string[] {
      return unwrapWith({
        lines: current,
        range,
        pair,
      },);
    },
    lines,
    );
}

/**
 Unwraps each blockquote paragraph of a slice that is one quotation.

 @param text - one slice's text as the page carries it

 @returns Text with those paragraphs unwrapped and a note per paragraph

 @example
 ```ts
 unwrapSlice({ text: '> “Purr.”', },).text; // '> Purr.'
 ```
 */
function unwrapSlice({ text, }: { readonly text: string; },): TextRewrite {
  /**
   Slice lines.
   */
  const lines = text.split('\n',);
  /**
   Paragraphs to read; unwrapping keeps every line, so the ranges hold.
   */
  const ranges = blockquoteParagraphs({ lines, },);
  /**
   Lines after every paragraph is read.
   */
  const unwrapped = ranges.reduce<readonly string[]>(
    function unwrapEach(
      current,
      range,
    ): readonly string[] {
      return unwrapParagraph({
        lines: current,
        range,
      },);
    },
    lines,
  );
  /**
   Paragraphs the pass unwrapped, as notes.
   */
  const changed = ranges
    .map(function toWords(range,): readonly [
      string,
      string,
    ] {
      return [
        paragraphWords({
          lines,
          range,
        },),
        paragraphWords({
          lines: unwrapped,
          range,
        },),
      ];
    },)
    .filter(function differs([
      before,
      after,
    ],): boolean {
      return before !== after;
    },)
    .map(function note([before,],): string {
      /**
       Paragraph on one line, so the finding stays one log line.
       */
      const flat = before.replaceAll(
        '\n',
        ' / ',
      );
      /**
       Opening the note quotes.
       */
      const opening = wholeOpening({
        text: flat,
        units: NOTE_OPENING_UNITS,
      },);
      return `${opening}… unwrapped to the archive's bare blockquote style`;
    },);
  return {
    text: unwrapped.join('\n',),
    changed,
  };
}

/**
 Unwraps the page's quoted blockquote paragraphs where the archive sets its
 blockquotes bare, the slices no lane replaced included.

 @param slices - prepared pairs, whose archive text stands where no row replaces it

 @param replacements - what the page would write per slice

 @param archiveOriginalSpans - spans sealed as the English original

 @returns Replacements with the paragraphs unwrapped (a row added for an
 archive slice the pass changed), the rewritten rows alone, and one finding
 per slice changed

 @example
 ```ts
 const page = unwrapBlockquoteQuotes({ slices, replacements, archiveOriginalSpans: [], },);
 ```
 */
export function unwrapBlockquoteQuotes(
  {
    slices,
    replacements,
    archiveOriginalSpans,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
    readonly archiveOriginalSpans: readonly ArchiveOriginalSpan[];
  },
): {
  readonly replacements: readonly SliceReplacement[];
  readonly restored: readonly SliceReplacement[];
  readonly findings: readonly string[];
} {
  if (!archiveQuotesBare({ slices, },)) {
    return {
      replacements,
      restored: [],
      findings: [],
    };
  }
  return rewriteEverySlice({
    slices,
    replacements,
    archiveOriginalSpans,
    rewrite: unwrapSlice,
    findingName: 'blockquote-quotes-unwrapped',
  },);
}

//endregion Blockquote quote unify
