import type { TreeNode, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  FOOTNOTE_INDENT,
  indentationOf,
  isIndentedCode,
  opensFootnote,
  linesWithoutPrefix,
  startLineOf,
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
 What a container puts before a line, and whether the line carries it.
 */
export type ContainerPrefix = {
  /**
   How many characters of marks and white space the line gives up to the container.
   */
  readonly length: number;

  /**
   Whether the line carries every mark of a block quote the container has; a line without one is a lazy line, text of the
   paragraph above it that the container does not hold.
   */
  readonly marked: boolean;
};

/**
 The prefix of a line that carries no mark of the container: a lazy line, which gives up nothing.
 */
const UNMARKED: ContainerPrefix = {
  length: 0,
  marked: false,
};

/**
 What a line gives up to the container that put a pattern before the footnote's first line.

 Each mark of a block quote in the pattern must stand on the line, in its place, and each other character of the pattern
 (white space, the marker of a list item) is matched by a space or a tab when the line has one there.

 @param source - text

 @param at - index of the first character of the line

 @param pattern - what the container put before the footnote's label on its line, such as `> ` or `- `

 @returns How many characters the container takes of the line, and whether the line carries its marks

 @example
 ```ts
 const prefix = containerPrefixOf({ source: '>     a', at: 0, pattern: '> ', },); // { length: 2, marked: true }
 ```
 */
export function containerPrefixOf(
  {
    source,
    at,
    pattern,
  }: {
    readonly source: string;
    readonly at: number;
    readonly pattern: string;
  },
): ContainerPrefix {
  /**
   Characters taken.
   */
  const taken = { count: 0, };
  for (let place = 0; place < pattern.length; place += 1) {
    /**
     The character of the line the next place of the pattern is matched against.
     */
    const character = source.charAt(at + taken.count,);
    if (pattern.charAt(place,) === '>') {
      if (character !== '>')
        return UNMARKED;
      taken.count += 1;
    }
    else if ((character === ' ') || (character === '\t')) {
      taken.count += 1;
    }
  }
  return {
    length: taken.count,
    marked: true,
  };
}

/**
 The indices of the characters of a block indented under a footnote definition in a container that remain once each line
 gives up the marks and the indentation of the container and then the four columns the footnote takes. A lazy line, which
 carries no mark of a block quote, gives up nothing.

 @param source - text

 @param range - where the block stands, from the first character after the marks of its container on its first line

 @param prefix - what the container puts before the footnote's label on its line, which every later line carries the
 marks of

 @returns Indices in text order

 @example
 ```ts
 const kept = withoutContainerAndIndentation({ source, range, prefix: '> ', },);
 ```
 */
export function withoutContainerAndIndentation(
  {
    source,
    range,
    prefix,
  }: {
    readonly source: string;
    readonly range: TextRange;
    readonly prefix: string;
  },
): readonly number[] {
  return linesWithoutPrefix({
    source,
    range,
    prefixLength: function containerAndIndentation(line,): number {
      if (line.start === range.start) {
        // The first line starts after the marks of its container.
        return indentationOf({
          source,
          line,
        },);
      }
      /**
       What the container takes of the line.
       */
      const container = containerPrefixOf({
        source,
        at: line.start,
        pattern: prefix,
      },);
      if (!container.marked)
        return 0;
      return container.length + indentationOf({
        source,
        line: {
          start: line.start + container.length,
          end: line.end,
        },
      },);
    },
  },);
}

/**
 Whether a stretch of the text holds a line, after its first, that carries the marks of the container and is indented by the
 four columns a footnote takes: a line that starts a block of the footnote where the plain reading sees a lazy line.

 @param source - text

 @param range - where the stretch stands, from its first line

 @param prefix - what the container puts before the label of the footnote on its line

 @returns Whether such a line stands in the stretch

 @example
 ```ts
 const holds = holdsIndentedLine({ source: '> [^1]: x\n>     a\n', range: { start: 2, end: 17, }, prefix: '> ', },);
 ```
 */
export function holdsIndentedLine(
  {
    source,
    range,
    prefix,
  }: {
    readonly source: string;
    readonly range: TextRange;
    readonly prefix: string;
  },
): boolean {
  for (
    let feed = source.indexOf(
      '\n',
      range.start,
    );
    (feed !== (-1)) && ((feed + 1) < range.end);
    feed = source.indexOf(
      '\n',
      feed + 1,
    )
  ) {
    /**
     Where the line after the feed starts.
     */
    const lineStart = feed + 1;
    /**
     What the container takes of the line.
     */
    const container = containerPrefixOf({
      source,
      at: lineStart,
      pattern: prefix,
    },);
    if (!container.marked)
      continue;
    /**
     Where the indentation of the footnote would start on the line.
     */
    const indentAt = lineStart + container.length;
    /**
     Whether the line is indented by four spaces.
     */
    const spaced = source.startsWith(
      '    ',
      indentAt,
    );
    /**
     Whether the line is indented by a tab.
     */
    const tabbed = source.startsWith(
      '\t',
      indentAt,
    );
    if (spaced || tabbed)
      return true;
  }
  return false;
}

/**
 Whether a node is a paragraph the strict grammar reads as a block of the footnote definition right before it: it starts a
 line that is indented four columns or more beyond the container's marks, which the plain reading takes for a lazy line
 of a paragraph where the page, with no indented code, reads whatever block the line starts.

 @param source - text

 @param node - node of the parse

 @param range - where the node stands, from its first character after the indentation

 @param prefix - what the container puts before the label of the footnote on its line

 @returns Whether the node is a paragraph of the footnote

 @example
 ```ts
 const continues = continuesAsParagraph({ source, node, range, prefix: '> ', },);
 ```
 */
export function continuesAsParagraph(
  {
    source,
    node,
    range,
    prefix,
  }: {
    readonly source: string;
    readonly node: TreeNode;
    readonly range: TextRange;
    readonly prefix: string;
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
   What the container takes of the paragraph's line.
   */
  const container = containerPrefixOf({
    source,
    at: lineStart,
    pattern: prefix,
  },);
  if (!container.marked)
    return false;
  /**
   Where the container's marks and white space end on the line, as far as the paragraph's own line goes.
   */
  const containerEnd = Math.min(
    lineStart + container.length,
    range.start,
  );
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
 Whether a block opens a footnote definition that is read as the body of a footnote.

 A paragraph at the top that opens one is, and so is a link reference definition that the strict grammar reads as one. A
 paragraph in a container is when it holds a line, after its first, that starts a block of the footnote: the plain reading
 sees a lazy line there, and any other paragraph in a container is read as it stands.

 @param source - text

 @param node - block

 @param range - where the block stands

 @param state - what the block is once read, whose prefix is the one the block's own line has when the block opens one

 @param isRootChild - whether the block stands at the top of the text

 @returns Whether the block opens a footnote definition read as a body

 @example
 ```ts
 const opens = opensFootnoteBody({ source, node, range, state, isRootChild: false, },);
 ```
 */
export function opensFootnoteBody(
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
  if (!opensFootnote({
    source,
    node,
    range,
  },))
    return false;
  if (isRootChild || (node.type === 'definition'))
    return true;
  if (node.type !== 'paragraph')
    return false;
  return holdsIndentedLine({
    source,
    range,
    prefix: state.prefix,
  },);
}

/**
 Whether a block goes on in the paragraph of the footnote definition that the block read before it in its container opened,
 as a lazy line: a paragraph or a line of raw HTML that starts on the line right after the definition itself, and opens no
 definition of its own.

 The footnote stays open after such a block, so that a block indented four columns after a blank line still belongs to it.

 @param source - text

 @param node - block

 @param range - where the block stands

 @param state - what the block read before it was

 @param isRootChild - whether the block stands at the top of the text, where the footnote's own range says what it holds

 @returns Whether the block is a lazy line of the definition's paragraph

 @example
 ```ts
 const lazy = goesOnAsLazyText({ source, node, range, state, isRootChild: false, },);
 ```
 */
export function goesOnAsLazyText(
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
  if (isRootChild)
    return false;
  if (!state.opensFootnote)
    return false;
  /**
   The line the block starts on.
   */
  const startLine = startLineOf({
    node,
    fallback: 0,
  },);
  /**
   The line the definition ends on.
   */
  const { endLine, } = state;
  /**
   Whether the block starts on the line right after the definition, with no blank line between.
   */
  const rightAfter = startLine <= (endLine + 1);
  /**
   Whether the block is of a kind that goes on in a paragraph as text.
   */
  const isText = (node.type === 'paragraph') || (node.type === 'html');
  if (!(rightAfter && isText))
    return false;
  return !opensFootnote({
    source,
    node,
    range,
  },);
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

 @returns Whether the block is indented code under the footnote, a paragraph in a container indented four columns beyond
 the container's marks, or a paragraph right after the definition that holds a line indented so

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
  // THE LINES RIGHT AFTER A DEFINITION THAT OPENS WITH RAW HTML ARE THAT BLOCK'S, in a container: no blank line stands
  // between it and the block, which a reader of the source takes for text. At the top the footnote's own range says so.
  /**
   The line the block read before ends on, and whether it opens a definition that starts with raw HTML.
   */
  const {
    endLine,
    opensRawHtml,
  } = state;
  /**
   The line the block starts on.
   */
  const startLine = startLineOf({
    node,
    fallback: 0,
  },);
  /**
   Whether the block starts on the line right after the definition, with no blank line between.
   */
  const rightAfter = startLine <= (endLine + 1);
  /**
   Whether the block is a line of the raw HTML block the definition opens with.
   */
  const inRawHtml = opensRawHtml && rightAfter;
  if ((!isRootChild) && inRawHtml)
    return false;
  /**
   Whether the block is a lazy line of the definition's paragraph that holds a line indented four columns, which is a
   block of the footnote where the plain reading sees a lazy line.
   */
  const lazy = goesOnAsLazyText({
    source,
    node,
    range,
    state,
    isRootChild,
  },);
  /**
   Whether it holds such a line.
   */
  const indented = holdsIndentedLine({
    source,
    range,
    prefix: state.prefix,
  },);
  if (lazy && indented)
    return true;
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
    prefix: state.prefix,
  },);
}

//endregion Source ships footnote container
