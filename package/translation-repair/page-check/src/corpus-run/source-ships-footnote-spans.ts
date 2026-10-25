import type {
  BlockRange,
  TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  footnoteBodyStart,
  footnoteStartInside,
  NO_FOOTNOTE_INSIDE,
  opensFootnote,
} from './source-ships-footnote.ts';
import {
  footnoteBodySpans,
  footnoteInParagraphSpans,
  type SpansReader,
} from './source-ships-nested.ts';
import {
  type FootnoteState,
  type Pending,
  takeFootnote,
} from './source-ships-pending.ts';
import type {
  SourceSpans,
  TextRange,
} from './source-ships-ranges.ts';

//region Source ships footnote spans
// HOW A BLOCK THAT BELONGS TO A FOOTNOTE DEFINITION IS READ, which the spans reader asks of each block it walks: a
// paragraph that opens a footnote definition at the top, a link reference definition that is one to the strict grammar,
// blocks indented under a definition, and a paragraph in which a definition starts on a later line are read as the body
// of the footnote is (`source-ships-nested.ts`), each line giving up the four columns the footnote takes, and the
// blocks the footnote reaches into are taken off the nodes still to read.
//
// A FOOTNOTE LINE IN A BLOCK QUOTE OR A LIST ITEM is cut where the strict grammar starts the footnote, whatever stands
// before it on its line: the marks of the quote, the hanging indent of the item, or a lazy line indented up to three
// spaces. The lines before it are read as a text of their own and the footnote as the body, to the end of the
// paragraph, with the quote marks of each line taken off before its indentation. The paragraph is the plain
// reading's, which is one to the strict grammar too, so a definition line after the footnote is a line of it and not a
// definition, and what follows the paragraph in the container is read on its own.

/**
 The answer for a block that is no part of a footnote definition.
 */
type NotInAFootnote = {
  readonly kind: 'none';
};

/**
 The answer for a block that is part of a footnote definition: what the reading found in it and the state of the top
 once it is read.
 */
type InAFootnote = {
  readonly kind: 'read';
  readonly spans: SourceSpans;
  readonly state: FootnoteState;
};

/**
 What reading a block as part of a footnote answers.
 */
export type FootnoteRead = InAFootnote | NotInAFootnote;

/**
 The answer for a block that is no part of a footnote definition.
 */
const NOT_IN_A_FOOTNOTE: FootnoteRead = { kind: 'none', };

/**
 Reads a block as part of a footnote definition when it is one.

 @param source - whole text

 @param readable - whole text with its comments blanked, the parse's own input

 @param readSpans - what reads a block's spans, handed in so this module and the reader do not import each other

 @param footnotes - where the strict grammar reads a footnote definition in the whole text

 @param pending - nodes still to read, the next one on top; the blocks the footnote reaches into are removed from it

 @param node - block

 @param range - where the block stands

 @param isRootChild - whether the block is at the top of the text

 @param continuesFootnote - whether the block is indented code under a footnote definition

 @param state - state of the block's container once the block is read

 @returns Spans of the block and of what the footnote reaches into, with the state of the top, or that the block is
 no part of a footnote

 @throws {@link Error} when a node of the footnote has no position, which the reading used never builds

 @example
 ```ts
 const read = footnoteRead({ source, readable, readSpans: spansOf, footnotes, pending, node, range, isRootChild: true,
   continuesFootnote: false, state, },);
 ```
 */
export function footnoteRead(
  {
    source,
    readable,
    readSpans,
    footnotes,
    pending,
    node,
    range,
    isRootChild,
    continuesFootnote,
    state,
  }: {
    readonly source: string;
    readonly readable: string;
    readonly readSpans: SpansReader;
    readonly footnotes: readonly BlockRange[];
    readonly pending: Pending[];
    readonly node: TreeNode;
    readonly range: TextRange;
    readonly isRootChild: boolean;
    readonly continuesFootnote: boolean;
    readonly state: FootnoteState;
  },
): FootnoteRead {
  /**
   Where a footnote definition starts inside the paragraph, when one does on a line after its first.
   */
  const split = (node.type === 'paragraph')
    ? footnoteStartInside({
      footnotes,
      range,
    },)
    : NO_FOOTNOTE_INSIDE;
  if (split !== NO_FOOTNOTE_INSIDE) {
    /**
     How many blocks are still to read, which falls when the footnote reaches blocks of the top.
     */
    const stillToRead = pending.length;
    /**
     How far the footnote reaches.
     */
    const reach = takeFootnote({
      pending,
      footnotes,
      source,
      node,
      range: {
        start: split,
        end: range.end,
      },
      state,
    },);
    /**
     Where the lines of the footnote end: a footnote in a container ends with the paragraph, unless it took blocks of the
     top, since the blocks after the paragraph in the container are read on their own.
     */
    const end = (isRootChild || (pending.length < stillToRead)) ? reach.end : range.end;
    return {
      kind: 'read',
      state: reach.state,
      spans: footnoteInParagraphSpans({
        source,
        readable,
        readSpans,
        footnotes,
        range,
        split,
        end,
        inContainer: !isRootChild,
      },),
    };
  }
  /**
   Whether the block is a paragraph at the top whose first line opens a footnote definition, or a link reference
   definition that the strict grammar reads as one.
   */
  const opensFootnoteHere = opensFootnote({
    source,
    node,
    range,
  },) && (isRootChild || (node.type === 'definition'));
  if (!(opensFootnoteHere || continuesFootnote))
    return NOT_IN_A_FOOTNOTE;
  /**
   How far the footnote reaches, when the block opens one at the top.
   */
  const reach = (opensFootnoteHere && isRootChild)
    ? takeFootnote({
      pending,
      footnotes,
      source,
      node,
      range,
      state,
    },)
    : {
      end: range.end,
      state,
    };
  return {
    kind: 'read',
    state: reach.state,
    spans: footnoteBodySpans({
      source,
      readable,
      readSpans,
      footnotes,
      range: opensFootnoteHere
        ? {
          start: footnoteBodyStart({
            source,
            range,
          },),
          end: reach.end,
        }
        : range,
    },),
  };
}

//endregion Source ships footnote spans
