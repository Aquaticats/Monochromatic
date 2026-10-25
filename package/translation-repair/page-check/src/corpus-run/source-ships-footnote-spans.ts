import type {
  BlockRange,
  TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { footnoteInContainerSpans, } from './source-ships-container.ts';
import {
  isLazyText,
  opensFootnoteBody,
} from './source-ships-footnote-container.ts';
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
import { footnoteStartsInSpans, } from './source-ships-lines.ts';
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
// spaces. The lines before it are read as a text of their own and the footnote with what follows it as another, with
// the quote marks of each line taken off when the footnote line stays in the container
// (`source-ships-container.ts`); what follows the paragraph in the container is read on its own.

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

 @param footnotes - where the strict grammar reads a footnote definition in the whole text, a line of raw HTML ending it

 @param pageFootnotes - where the page ends a footnote definition, which the container reading follows; the footnotes
 when absent

 @param pageOnlyStarts - where the page starts a footnote definition and a reader of the source does not, none when the
 caller reads none; one inside a code span, inline math, emphasis, a link or an image is no place to cut the paragraph,
 since the line is joined to the one before it (`source-ships-lines.ts`)

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
    pageFootnotes = footnotes,
    pageOnlyStarts = [],
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
    readonly pageFootnotes?: readonly BlockRange[];
    readonly pageOnlyStarts?: readonly number[];
    readonly pending: Pending[];
    readonly node: TreeNode;
    readonly range: TextRange;
    readonly isRootChild: boolean;
    readonly continuesFootnote: boolean;
    readonly state: FootnoteState;
  },
): FootnoteRead {
  /**
   Where the footnote definitions stand for this block: at the top they end where a GFM reader of the source ends them,
   a line of raw HTML among the places; in a container they end where the page ends them, since a code span the plain
   reading keeps across a footnote line is cut there.
   */
  const ranges = isRootChild ? footnotes : pageFootnotes;
  /**
   The starts only the page reads that stand inside a span of the block, which no footnote cuts.
   */
  const joined = ((node.type === 'paragraph') || (node.type === 'heading'))
    ? footnoteStartsInSpans({
      source,
      node,
      starts: pageOnlyStarts,
    },)
    : [];
  /**
   Where a footnote definition starts inside the paragraph, or the heading made of its lines and a line of equals signs
   or hyphens, when one does on a line after its first, less the starts that join a span.
   */
  const split = ((node.type === 'paragraph') || (node.type === 'heading'))
    ? footnoteStartInside({
      footnotes: ranges.filter(function cuts({ start, },): boolean {
        return !joined.includes(start,);
      },),
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
      footnotes: ranges,
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
      spans: isRootChild
        ? footnoteInParagraphSpans({
          source,
          readable,
          readSpans,
          footnotes,
          pageFootnotes,
          range,
          split,
          end,
        },)
        : footnoteInContainerSpans({
          source,
          readable,
          readSpans,
          footnotes: pageFootnotes,
          range,
          split,
          end,
        },),
    };
  }
  /**
   Whether the block opens a footnote definition that is read as the body of a footnote: a paragraph at the top, a link
   reference definition that the strict grammar reads as one, or a paragraph in a container.
   */
  const opensFootnoteHere = opensFootnoteBody({
    source,
    node,
    range,
    isRootChild,
  },);
  if (!(opensFootnoteHere || continuesFootnote))
    return NOT_IN_A_FOOTNOTE;
  /**
   Whether the block is a paragraph that goes on in the definition's paragraph as a lazy line without the marks of its
   container, so that its first line is text, whatever its indentation, and not the start of a block of the footnote.
   */
  const goesOnAsText = isLazyText({
    source,
    node,
    range,
    state,
  },);
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
      pageFootnotes,
      range: opensFootnoteHere
        ? {
          start: footnoteBodyStart({
            source,
            range,
          },),
          end: reach.end,
        }
        : range,
      prefix: isRootChild ? '' : state.prefix,
      asParagraph: goesOnAsText,
    },),
  };
}

//endregion Source ships footnote spans
