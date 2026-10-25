import type {
  BlockRange,
  TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  rangeOf,
  type TextEdit,
} from './source-ships-ranges.ts';

//region Source ships opener interrupt
// A COMMENT THAT NEVER CLOSES, OPENED FIRST ON A LINE RIGHT AFTER A PARAGRAPH. A reader of the source takes the opener for
// the start of a block of raw HTML to the end of the text, which ends the paragraph before it even with no blank line
// between them, so a backtick or a dollar sign in the paragraph that nothing closes opens no code span or math there. The
// page has no raw HTML: the opener is written as a character reference (`source-ships-encoding.ts`) and the lines after
// it as the text of a paragraph, so the paragraph goes on, and the backtick pairs with one in the lines after the opener,
// which the page then shows as a code span the source does not have. A blank line before the line of the opener ends the
// paragraph on the page where it ends for the reader of the source. A paragraph that a footnote definition holds is left
// as it is: the page is arranged to go on with a footnote over a line of raw HTML
// (`document-structure/src/gfm-block-ranges.ts`).

/**
 Finds the line ending that a blank line is put after, when the opener starts a block of raw HTML that cuts a paragraph.

 @param text - whole text, or the text the parse was given, of the same length

 @param root - parse of the text by the plain reading with the site's math, with the tails of its footnotes read on their
 own

 @param start - index of the `<` of the opener that starts a block of raw HTML to the end of the text, -1 when none does

 @param footnotes - where a footnote definition stands, for a reader of the source and for the page

 @returns The edit that doubles the line ending before the opener's line; none when no paragraph stands right before it,
 when a footnote definition holds that paragraph, or when the opener starts no raw HTML block of the top of the text

 @example
 ```ts
 const edits = interruptionEdits({ text: 'a `x\n<!-- y', root, start: 5, footnotes: [], },);
 ```
 */
export function interruptionEdits(
  {
    text,
    root,
    start,
    footnotes,
  }: {
    readonly text: string;
    readonly root: TreeNode;
    readonly start: number;
    readonly footnotes: readonly BlockRange[];
  },
): readonly TextEdit[] {
  if ((start === (-1)) || (!('children' in root)))
    return [];
  /**
   The index of the raw HTML block that holds the opener among the blocks of the top of the text.
   */
  const at = root.children.findIndex(function holdsOpener(block,): boolean {
    if ((block.type !== 'html') || (block.position === undefined))
      return false;
    /**
     Where the block stands.
     */
    const range = rangeOf({ node: block, },);
    return (range.start <= start) && (start < range.end);
  },);
  /**
   The raw HTML block and the block right before it.
   */
  const block = root.children[at];
  const before = root.children[at - 1];
  if ((block?.position === undefined) || (before?.position === undefined) || (before.type !== 'paragraph'))
    return [];
  if ((before.position.end.line + 1) !== block.position.start.line)
    return [];
  /**
   Where the paragraph stands.
   */
  const paragraph = rangeOf({ node: before, },);
  if (footnotes.some(function holdsParagraph({
    start: from,
    end,
  },): boolean {
    return (from < paragraph.end) && (paragraph.start < end);
  },))
    return [];
  /**
   Where the line of the opener starts, after the line ending that ends the line before it.
   */
  const lineStart = text.lastIndexOf(
    '\n',
    start - 1,
  ) + 1;
  /**
   The line ending, with its carriage return when the text has one.
   */
  const ending = text.startsWith(
    '\r\n',
    lineStart - 2,
  )
    ? '\r\n'
    : '\n';
  return [
    {
      start: lineStart - ending.length,
      end: lineStart,
      replacement: `${ending}${ending}`,
    },
  ];
}

//endregion Source ships opener interrupt
