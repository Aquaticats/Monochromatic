import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  footnoteDefinitionRangesOf,
  parseMarkdownMathBody,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { rangeOf, } from './source-ships-ranges.ts';

//region Source ships fresh tails
// THE TEXT AFTER A FOOTNOTE THAT OPENS WITH A TAG ALONE IS READ ON ITS OWN. The plain parse has no footnotes: it reads
// `[^1]: <b>` as a link reference definition whose destination is `<b>`, and a line right after a definition is, in the
// content the definition was peeled from, a line of the paragraph that remains, so a tag alone on it (which cannot
// interrupt a paragraph) is text there and a code span, a heading mark or display math after it is syntax. A reader of the
// source in GFM ends the footnote at its first line, since its raw HTML block is not a lazy line, and reads the tag alone
// on the next line as raw HTML that holds the lines up to the blank line. The lines after a footnote's end are therefore
// parsed from there, with the lines before them blank so that every offset and line number stands.

/**
 Offset of the first character of the line after the one an offset stands at the end of.

 @param text - whole text

 @param end - offset of the end of a line, before its line ending

 @returns Offset of the next line, -1 when the text ends at the offset

 @example
 ```ts
 const start = lineAfter({ text: 'a\nb', end: 1, },);
 ```
 */
function lineAfter(
  {
    text,
    end,
  }: {
    readonly text: string;
    readonly end: number;
  },
): number {
  if (text.startsWith(
    '\r\n',
    end,
  ))
    return end + 2;
  if (text.startsWith(
    '\n',
    end,
  ))
    return end + 1;
  return -1;
}

/**
 The text with every character before an offset replaced by a space, the line endings kept.

 @param text - whole text

 @param until - offset the blanking stops at

 @returns Text of the same length whose lines before the offset are blank

 @example
 ```ts
 const blank = blankedBefore({ text: 'a\nb', until: 2, },);
 ```
 */
function blankedBefore(
  {
    text,
    until,
  }: {
    readonly text: string;
    readonly until: number;
  },
): string {
  /**
   The text before the offset.
   */
  const before = text.slice(
    0,
    until,
  );
  /**
   Its characters, each kept when it ends a line and blanked otherwise.
   */
  const blanked = before
    .split('',)
    .map(function blank(character,): string {
      return ((character === '\n') || (character === '\r')) ? character : ' ';
    },);
  return blanked.join('',) + text.slice(until,);
}

/**
 The root of a plain parse, with the children typed read-only as the rest of the reading has them.
 */
type RootNode = Extract<TreeNode, { readonly type: 'root'; }>;

/**
 Whether a block quote or a list holds an offset.

 @param block - block of the root

 @param offset - offset in the whole text

 @returns Whether the block is a container whose stretch holds the offset

 @example
 ```ts
 const held = holdsOffset({ block, offset: 12, },);
 ```
 */
function holdsOffset(
  {
    block,
    offset,
  }: {
    readonly block: TreeNode;
    readonly offset: number;
  },
): boolean {
  if ((block.type !== 'blockquote') && (block.type !== 'list'))
    return false;
  /**
   Where the container starts and ends.
   */
  const {
    start,
    end,
  } = rangeOf({ node: block, },);
  return (start <= offset) && (offset < end);
}

/**
 Reads the blocks after each top-level footnote on their own where a paragraph of the plain parse starts right there.

 @param readable - text as the parse reads it, with its comments blanked

 @param root - plain parse of the text

 @returns The root, its blocks from each such place replaced by the parse of the text from there

 @example
 ```ts
 const fresh = rootWithFreshTails({ readable, root: parseMarkdownMathBody({ body: readable, },), },);
 ```
 */
export function rootWithFreshTails(
  {
    readable,
    root,
  }: {
    readonly readable: string;
    readonly root: RootNode;
  },
): RootNode {
  /**
   The blocks with the tails read so far.
   */
  const children = [...root.children,];
  for (const range of footnoteDefinitionRangesOf({ body: readable, },)) {
    /**
     Where the text after the footnote starts.
     */
    const tail = lineAfter({
      text: readable,
      end: range.end,
    },);
    if (tail < 0)
      continue;
    /**
     Index of the block that starts at the tail or later.
     */
    const at = children.findIndex(function startsThere(child,): boolean {
      /**
       Where the block starts.
       */
      const { start, } = rangeOf({ node: child, },);
      return start >= tail;
    },);
    if (at === (-1))
      continue;
    /**
     The block there.
     */
    const first = nonNullishOrThrow(children[at],);
    /**
     Where it starts.
     */
    const { start: firstStart, } = rangeOf({ node: first, },);
    // A FOOTNOTE IN A BLOCK QUOTE OR A LIST ITEM IS READ AS THE PAGE READS IT, so its tail is left to the plain parse.
    /**
     Whether a block quote or a list holds the footnote.
     */
    const inContainer = children.some(function holds(child,): boolean {
      return holdsOffset({
        block: child,
        offset: range.start,
      },);
    },);
    if (inContainer)
      continue;
    if (first.type !== 'paragraph')
      continue;
    if (firstStart !== tail)
      continue;
    /**
     The text from the tail, with the lines before it blank.
     */
    const fresh = parseMarkdownMathBody({
      body: blankedBefore({
        text: readable,
        until: tail,
      },),
    },);
    children.splice(
      at,
      children.length - at,
      ...fresh.children,
    );
  }
  return {
    ...root,
    children,
  };
}

//endregion Source ships fresh tails
