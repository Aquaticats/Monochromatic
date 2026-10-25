import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  parseMarkdownMathBody,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  footnoteBodyStart,
  withoutIndentation,
  withoutQuoteMarks,
} from './source-ships-footnote.ts';
import { lazyUnderlinesBlanked, } from './source-ships-lazy.ts';
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
 The characters of a text at the indices.

 @param text - text

 @param indices - indices of the characters, in order

 @returns Characters joined

 @example
 ```ts
 const body = charactersAt({ text: 'abc', indices: [0, 2,], },);
 ```
 */
function charactersAt(
  {
    text,
    indices,
  }: {
    readonly text: string;
    readonly indices: readonly number[];
  },
): string {
  return indices
    .map(function characterAt(at,): string {
      return text.charAt(at,);
    },)
    .join('',);
}

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
  }: {
    readonly source: string;
    readonly readable: string;
    readonly indices: readonly number[];
    readonly readSpans: SpansReader;
  },
): SourceSpans {
  /**
   The block as a text of its own.
   */
  const body = charactersAt({
    text: source,
    indices,
  },);
  /**
   The same with its comments blanked.
   */
  const bodyReadable = charactersAt({
    text: readable,
    indices,
  },);
  /**
   What the parse of the block says of it.
   */
  const inner = readSpans({
    source: body,
    readable: bodyReadable,
    root: parseMarkdownMathBody({ body: bodyReadable, },),
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
          end: nonNullishOrThrow(indices[edit.end - 1],) + 1,
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
function joinedSpans(
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
function editsOnly({ edits, }: { readonly edits: readonly TextEdit[]; },): SourceSpans {
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
 line that is not indented once a block of the footnote has closed its paragraph

 @param range - where the body stands, from the start of its first line or the first character after the label

 @param inContainer - whether the footnote stands in a block quote or a list item, whose quote marks each later line
 gives up before its indentation

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
    range,
    inContainer = false,
  }: {
    readonly source: string;
    readonly readable: string;
    readonly readSpans: SpansReader;
    readonly footnotes: readonly TextRange[];
    readonly range: TextRange;
    readonly inContainer?: boolean;
  },
): SourceSpans {
  /**
   The footnote the body belongs to, absent where the strict reading found none there and for a footnote in a container,
   whose paragraph the plain reading keeps whole as the strict grammar does: it ends where the paragraph does, the
   lines after it are lines of the paragraph and not a block that starts there.
   */
  const footnote = inContainer
    ? undefined
    : footnotes.find(function holds({
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
    indices: withoutIndentation({
      source,
      range: {
        start: range.start,
        end: bodyEnd,
      },
      marks: inContainer,
    },),
  },);
  if ((footnote === undefined) || (footnote.end > range.end))
    return body;
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
        indices: inContainer
          ? withoutQuoteMarks({
            source,
            range: after,
          },)
          : indicesOf({ range: after, },),
      },),
      second: editsOnly({
        edits: separatingEdits({
          source,
          readable,
          range: after,
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

 @param range - where the paragraph stands

 @param split - where the footnote definition starts in the paragraph

 @param end - where the lines of the footnote end, the end of the paragraph or beyond it

 @param inContainer - whether the paragraph stands in a block quote or a list item: the lines before the definition lose
 their quote marks, the line of the definition belongs to no part before its label, and each later line of the
 footnote loses them before its indentation

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
    range,
    split,
    end,
    inContainer = false,
  }: {
    readonly source: string;
    readonly readable: string;
    readonly readSpans: SpansReader;
    readonly footnotes: readonly TextRange[];
    readonly range: TextRange;
    readonly split: number;
    readonly end: number;
    readonly inContainer?: boolean;
  },
): SourceSpans {
  return joinedSpans({
    first: nestedSpans({
      source,
      readable,
      readSpans,
      indices: inContainer
        ? withoutQuoteMarks({
          source,
          range: {
            start: range.start,
            end: source.lastIndexOf(
              '\n',
              split - 1,
            ) + 1,
          },
        },)
        : indicesOf({
          range: {
            start: range.start,
            end: split,
          },
        },),
    },),
    second: footnoteBodySpans({
      source,
      readable,
      readSpans,
      footnotes,
      inContainer,
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
