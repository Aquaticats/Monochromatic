import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  parseMarkdownMathBody,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { footnoteBodyStart, } from './source-ships-footnote.ts';
import {
  charactersAt,
  endOfEdit,
} from './source-ships-nested-text.ts';
import { withoutContainerAndIndentation, } from './source-ships-footnote-container.ts';
import {
  bodyStartsOnLabelLine,
  withoutLabelLineTable,
} from './source-ships-label-line-table.ts';
import { lazyUnderlinesBlanked, } from './source-ships-lazy.ts';
import {
  isIndentedFourOrMore,
  PARAGRAPH_LEAD,
  spansWithoutLead,
} from './source-ships-paragraph-lead.ts';
import { asTheParseReadsIt, } from './source-ships-readable.ts';
import {
  indicesOf,
  originalStretches,
  type SourceSpans,
  type TextEdit,
  type TextRange,
} from './source-ships-ranges.ts';
import { separatingEdits, } from './source-ships-separation.ts';

//region Source ships nested
// A BLOCK NESTED IN ANOTHER, READ AS A TEXT OF ITS OWN: the body of a footnote definition and the blocks indented under
// one are parsed from the characters that make them up, and what the parse says is mapped back to the offsets of the
// whole text (`source-ships-spans.ts` says why).

/**
 What reads the spans of a text, handed to the nested reading so that the two modules do not import each other.
 */
export type SpansReader = (
  arguments_: {
    readonly source: string;
    readonly readable: string;
    readonly root: TreeNode;
  },
) => SourceSpans;

/**
 Reads a block nested in another as a text of its own: the body of a footnote definition, or a block indented under one.

 @param source - whole text

 @param readable - whole text with its comments blanked, the parse's own input

 @param indices - indices of the whole text that make up the block, in order

 @param readSpans - what reads a block's spans, handed in so this module and the reader do not import each other

 @returns Spans of the block at the offsets of the whole text

 @throws {@link Error} when a node of the block has no position, which the reading used never builds

 @example
 ```ts
 const spans = nestedSpans({ source, readable, indices: [6, 7, 8,], readSpans: spansOf, });
 ```
 */
export function nestedSpans(
  {
    source,
    readable,
    indices,
    readSpans,
    asParagraph = false,
    onLabelLine = false,
  }: {
    readonly source: string;
    readonly readable: string;
    readonly indices: readonly number[];
    readonly readSpans: SpansReader;
    readonly asParagraph?: boolean;
    readonly onLabelLine?: boolean;
  },
): SourceSpans {
  /**
   What is put before the block so that the parse reads the paragraph it stood in.
   */
  const lead = asParagraph ? PARAGRAPH_LEAD : '';
  /**
   The block as a text of its own.
   */
  const body = lead + charactersAt({
    text: source,
    indices,
  },);
  /**
   The same with its comments blanked, and a letter in place of a byte order mark that opens it: the mark is an ordinary
   character in the middle of the page, where a parser drops it from the start of its input and stands every offset one
   character off.
   */
  const readableBody = lead + asTheParseReadsIt({
    text: charactersAt({
      text: readable,
      indices,
    },),
  },);
  /**
   The same for the parse, with no table started on the label's line, where the page reads none.
   */
  const bodyReadable = onLabelLine ? withoutLabelLineTable({ body: readableBody, },) : readableBody;
  /**
   What the parse of the block says of it, at the offsets of the block without the lead.
   */
  const inner = spansWithoutLead({
    spans: readSpans({
      source: body,
      readable: bodyReadable,
      root: parseMarkdownMathBody({ body: bodyReadable, },),
    },),
    by: lead.length,
  },);
  return {
    text: inner
      .text
      .flatMap(function original(range,): readonly TextRange[] {
        return originalStretches({
          range,
          indices,
        },);
      },),
    literal: inner
      .literal
      .flatMap(function original(range,): readonly TextRange[] {
        return originalStretches({
          range,
          indices,
        },);
      },),
    edits: inner
      .edits
      .map(function original(edit,): TextEdit {
        return {
          ...edit,
          start: nonNullishOrThrow(indices[edit.start],),
          end: endOfEdit({
            source,
            indices,
            edit,
          },),
        };
      },),
    insideLinks: inner
      .insideLinks
      .flatMap(function original(range,): readonly TextRange[] {
        return originalStretches({
          range,
          indices,
        },);
      },),
    streamStarts: inner
      .streamStarts
      .map(function original(start,): number {
        return nonNullishOrThrow(indices[start],);
      },),
  };
}

/**
 Joins the spans of two stretches of a text.

 @param first - spans of the earlier stretch

 @param second - spans of the later stretch

 @returns Spans of both, the earlier first

 @example
 ```ts
 const both = joinedSpans({ first: bodySpans, second: tailSpans, },);
 ```
 */
export function joinedSpans(
  {
    first,
    second,
  }: {
    readonly first: SourceSpans;
    readonly second: SourceSpans;
  },
): SourceSpans {
  return {
    text: [
      ...first.text,
      ...second.text,
    ],
    literal: [
      ...first.literal,
      ...second.literal,
    ],
    edits: [
      ...first.edits,
      ...second.edits,
    ],
    insideLinks: [
      ...first.insideLinks,
      ...second.insideLinks,
    ],
    streamStarts: [
      ...first.streamStarts,
      ...second.streamStarts,
    ],
  };
}

/**
 Spans that hold edits and nothing else.

 @param edits - edits to carry

 @returns Spans with no text, literal, link or stream ranges

 @example
 ```ts
 const spans = editsOnly({ edits: [], },);
 ```
 */
export function editsOnly({ edits, }: { readonly edits: readonly TextEdit[]; },): SourceSpans {
  return {
    text: [],
    literal: [],
    edits,
    insideLinks: [],
    streamStarts: [],
  };
}

/**
 Reads the blocks of a footnote definition as a text of their own, each line giving up the four columns of indentation
 the footnote takes, and the lines after the end of the footnote as a text of their own beside it.

 @param source - whole text

 @param readable - whole text with its comments blanked, the parse's own input

 @param readSpans - what reads a block's spans, handed in so this module and the reader do not import each other

 @param footnotes - where the strict grammar reads a footnote definition in the whole text, which ends a footnote at a
 line that is not indented once a block of the footnote has closed its paragraph, and at a line of raw HTML

 @param pageFootnotes - where the page ends a footnote definition, which a line of raw HTML does not end; the footnotes
 when absent

 @param range - where the body stands, from the start of its first line or the first character after the label

 @param asParagraph - whether the body is a paragraph that goes on in the paragraph right before it, which its first line,
 however it is indented, is a line of

 @param prefix - what the container the body stands in puts before its first line, which every later line carries the marks of
 and gives up with the four columns the footnote takes; empty at the top

 @returns Spans of the body at the offsets of the whole text

 @throws {@link Error} when a node of the body has no position, which the reading used never builds

 @example
 ```ts
 const spans = footnoteBodySpans({ source, readable, readSpans: spansOf, footnotes: [{ start: 0, end: 12, },],
   range: { start: 6, end: 12, }, },);
 ```
 */
export function footnoteBodySpans(
  {
    source,
    readable,
    readSpans,
    footnotes,
    pageFootnotes = footnotes,
    range,
    prefix = '',
    asParagraph = false,
  }: {
    readonly source: string;
    readonly readable: string;
    readonly readSpans: SpansReader;
    readonly footnotes: readonly TextRange[];
    readonly pageFootnotes?: readonly TextRange[];
    readonly range: TextRange;
    readonly prefix?: string;
    readonly asParagraph?: boolean;
  },
): SourceSpans {
  /**
   The footnote the body belongs to, absent where the strict reading found none there.
   */
  const footnote = footnotes.find(function holds({
    start,
    end,
  },): boolean {
    return (start <= range.start) && (range.start < end);
  },);
  /**
   Where the footnote's body ends: the end of the footnote, when the plain reading goes on past it.
   */
  const bodyEnd = (footnote === undefined) ? range.end : Math.min(
    range.end,
    footnote.end,
  );
  /**
   Spans of the footnote's own lines.
   */
  const body = nestedSpans({
    source,
    readable: lazyUnderlinesBlanked({
      readable,
      range: {
        start: range.start,
        end: bodyEnd,
      },
    },),
    readSpans,
    indices: withoutContainerAndIndentation({
      source,
      range: {
        start: range.start,
        end: bodyEnd,
      },
      prefix,
    },),
    asParagraph,
    onLabelLine: bodyStartsOnLabelLine({
      source,
      start: range.start,
    },),
  },);
  if ((footnote === undefined) || (footnote.end > range.end))
    return body;
  /**
   Whether the page goes on with the footnote's paragraph past where a reader of the source ends the footnote, at a line
   of raw HTML: the line after is then a lazy line on the page, where it starts a block of its own to that reader.
   */
  const pageGoesOn = pageFootnotes.some(function goesOn({
    start,
    end,
  },): boolean {
    return (start === footnote.start) && (end > bodyEnd);
  },);
  if (bodyEnd >= range.end) {
    // The block ends where the footnote does, and the blocks after it are read on their own.
    return joinedSpans({
      first: body,
      second: editsOnly({
        edits: separatingEdits({
          source,
          readable,
          range: {
            start: bodyEnd,
            end: readable.length,
          },
          pageGoesOn,
        },),
      },),
    },);
  }
  /**
   The lines after the end of the footnote.
   */
  const after: TextRange = {
    start: bodyEnd,
    end: range.end,
  };
  return joinedSpans({
    first: body,
    second: joinedSpans({
      first: nestedSpans({
        source,
        readable,
        readSpans,
        indices: indicesOf({ range: after, },),
      },),
      second: editsOnly({
        edits: separatingEdits({
          source,
          readable,
          range: after,
          pageGoesOn,
        },),
      },),
    },),
  },);
}

/**
 Reads a paragraph of the plain reading in which a footnote definition starts on a later line: the lines before the
 definition as a text of their own, and the definition and what follows it as the footnote.

 @param source - whole text

 @param readable - whole text with its comments blanked, the parse's own input

 @param readSpans - what reads a block's spans, handed in so this module and the reader do not import each other

 @param footnotes - where the strict grammar reads a footnote definition in the whole text

 @param pageFootnotes - where the page ends a footnote definition; the footnotes when absent

 @param range - where the paragraph stands

 @param split - where the footnote definition starts in the paragraph

 @param end - where the lines of the footnote end, the end of the paragraph or beyond it

 @returns Spans of both parts at the offsets of the whole text

 @throws {@link Error} when a node of either part has no position, which the reading used never builds

 @example
 ```ts
 const spans = footnoteInParagraphSpans({ source, readable, readSpans: spansOf, footnotes, range,
   split: 4, end: 12, },);
 ```
 */
export function footnoteInParagraphSpans(
  {
    source,
    readable,
    readSpans,
    footnotes,
    pageFootnotes = footnotes,
    range,
    split,
    end,
  }: {
    readonly source: string;
    readonly readable: string;
    readonly readSpans: SpansReader;
    readonly footnotes: readonly TextRange[];
    readonly pageFootnotes?: readonly TextRange[];
    readonly range: TextRange;
    readonly split: number;
    readonly end: number;
  },
): SourceSpans {
  return joinedSpans({
    first: nestedSpans({
      source,
      readable,
      readSpans,
      indices: indicesOf({
        range: {
          start: range.start,
          end: split,
        },
      },),
      asParagraph: isIndentedFourOrMore({
        source,
        at: range.start,
      },),
    },),
    second: footnoteBodySpans({
      source,
      readable,
      readSpans,
      footnotes,
      pageFootnotes,
      range: {
        start: footnoteBodyStart({
          source,
          range: {
            start: split,
            end,
          },
        },),
        end,
      },
    },),
  },);
}

//endregion Source ships nested
