import type { TreeNode, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  FOOTNOTE_INDENT,
  indentationOf,
  isIndentedCode,
  linesWithoutPrefix,
  TAB_COLUMNS,
} from './source-ships-footnote.ts';
import type { FootnoteState, } from './source-ships-pending.ts';
import type { TextRange, } from './source-ships-ranges.ts';

//region Source ships footnote container
// THE BLOCKS UNDER A FOOTNOTE DEFINITION IN A CONTAINER. The strict grammar disables indented code, so a block indented four
// columns beyond the marks of a block quote or the indentation of a list item, under a footnote definition there, is a block of
// the footnote, as at the top (`source-ships-footnote.ts`). The plain reading takes it for indented code, or for a lazy line of
// the paragraph above, and each line of it carries the marks of the container before the four columns the footnote takes.

/**
 Characters a line of a container puts before its content: the marks of a block quote and white space.
 */
const CONTAINER_LINE_MARKS = ' \t>';

/**
 How many characters of a container's marks and white space a line starts with, up to as many as the container puts
 before its content.

 @param source - text

 @param at - index of the first character of the line

 @param most - how many characters the container puts before its content on the footnote's first line

 @returns Count of characters

 @example
 ```ts
 const count = containerCharactersAt({ source: '>     a', at: 0, most: 2, },); // 2
 ```
 */
export function containerCharactersAt(
  {
    source,
    at,
    most,
  }: {
    readonly source: string;
    readonly at: number;
    readonly most: number;
  },
): number {
  /**
   Characters counted.
   */
  const counted = { count: 0, };
  while (counted.count < most) {
    /**
     The character after those counted.
     */
    const character = source.charAt(at + counted.count,);
    if (!CONTAINER_LINE_MARKS.includes(character,))
      break;
    counted.count += 1;
  }
  return counted.count;
}

/**
 The indices of the characters of a block indented under a footnote definition in a container that remain once each line
 gives up the marks and the indentation of the container and then the four columns the footnote takes.

 @param source - text

 @param range - where the block stands, from the first character after the marks of its container on its first line

 @param prefixLength - how many characters of marks and white space the container puts before the block on its first line,
 which every later line carries as many of

 @returns Indices in text order

 @example
 ```ts
 const kept = withoutContainerAndIndentation({ source, range, prefixLength: 2, },);
 ```
 */
export function withoutContainerAndIndentation(
  {
    source,
    range,
    prefixLength,
  }: {
    readonly source: string;
    readonly range: TextRange;
    readonly prefixLength: number;
  },
): readonly number[] {
  return linesWithoutPrefix({
    source,
    range,
    prefixLength: function containerAndIndentation(line,): number {
      /**
       How many characters of the container's marks and white space the line starts with, none for the first line, which
       starts after them.
       */
      const container = containerCharactersAt({
        source,
        at: line.start,
        most: (line.start === range.start) ? 0 : prefixLength,
      },);
      return container + indentationOf({
        source,
        line: {
          start: line.start + container,
          end: line.end,
        },
      },);
    },
  },);
}

/**
 Whether a node is a paragraph the strict grammar reads as a block of the footnote definition right before it: it starts a
 line that is indented four columns or more beyond the container's marks, which the plain reading takes for a lazy line
 of a paragraph where the page, with no indented code, reads whatever block the line starts.

 @param source - text

 @param node - node of the parse

 @param range - where the node stands, from its first character after the indentation

 @param prefixLength - how many characters the container puts before the label of the footnote on its line

 @returns Whether the node is a paragraph of the footnote

 @example
 ```ts
 const continues = continuesAsParagraph({ source, node, range, prefixLength: 2, },);
 ```
 */
export function continuesAsParagraph(
  {
    source,
    node,
    range,
    prefixLength,
  }: {
    readonly source: string;
    readonly node: TreeNode;
    readonly range: TextRange;
    readonly prefixLength: number;
  },
): boolean {
  if (node.type !== 'paragraph')
    return false;
  /**
   Where the paragraph's line starts.
   */
  const lineStart = source.lastIndexOf(
    '\n',
    range.start - 1,
  ) + 1;
  /**
   Where the container's marks and white space end on the line, as far as the footnote's own line has them.
   */
  const containerEnd = lineStart + containerCharactersAt({
    source,
    at: lineStart,
    most: Math.min(
      prefixLength,
      range.start - lineStart,
    ),
  },);
  /**
   What stands between the container's marks and the paragraph.
   */
  const between = source.slice(
    containerEnd,
    range.start,
  );
  /**
   Columns of white space between them, none counted for a character that is no white space.
   */
  const columns = { count: 0, };
  for (const character of between) {
    if (character === ' ')
      columns.count += 1;
    else if (character === '\t')
      columns.count += TAB_COLUMNS;
    else
      return false;
  }
  return columns.count >= FOOTNOTE_INDENT;
}

/**
 Whether a block continues the footnote definition that the block read before it in its container opened.

 @param source - text

 @param node - block

 @param range - where the block stands

 @param state - what the block read before it was: whether it is part of a footnote definition, and how far the container
 indents the definition

 @param isRootChild - whether the block stands at the top of the text, where a paragraph after a definition is read with it
 by the footnote's own range

 @returns Whether the block is indented code under the footnote, or a paragraph in a container indented four columns beyond
 the container's marks

 @example
 ```ts
 const continues = continuesTheFootnote({ source, node, range, state, isRootChild: false, },);
 ```
 */
export function continuesTheFootnote(
  {
    source,
    node,
    range,
    state,
    isRootChild,
  }: {
    readonly source: string;
    readonly node: TreeNode;
    readonly range: TextRange;
    readonly state: FootnoteState;
    readonly isRootChild: boolean;
  },
): boolean {
  if (!state.open)
    return false;
  if (isIndentedCode({
    source,
    node,
    range,
  },))
    return true;
  return (!isRootChild) && continuesAsParagraph({
    source,
    node,
    range,
    prefixLength: state.prefixLength,
  },);
}

//endregion Source ships footnote container
