import {
  parseMarkdownMathBody,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { firstNestingExcess, } from '@monochromatic-dev/translation-repair-markup/ts';
import { COMMENT_OPEN, } from './source-ships-open-comment.ts';
import { asTheParseReadsIt, } from './source-ships-readable.ts';

//region Source ships open block
// THE OPENERS OF A COMMENT THAT NEVER CLOSES THAT START A BLOCK OF RAW HTML to a reader of the source, wherever they stand.
// A line that starts with such an opener after the marks of its containers and at most three spaces is the start of a block of
// raw HTML that runs to the end of its container (to the end of the text, at the top), whose lines are text to the
// reader: a list marker, a heading mark, a brace and a backtick in them are what they look like, and a backtick before the
// block pairs with nothing after it, since the block ends the paragraph it follows. The page carries each opener as a
// character reference and has no raw HTML, so the encoder gives the parse the openers that start blocks as they stand and
// every other opener as the text the page carries (`source-ships-readable.ts`), and the parse reads the blocks. Which
// openers start a block is asked of the parse itself, since a block quote, a list item and a fence each decide it by
// their own rules; the text is read once with every opener as it stands, and the openers at which a block of raw HTML
// starts are the answer.

/**
 Finds the openers of a comment that never closes at which a reader of the source reads a block of raw HTML start.

 @param text - text with its comments that close blanked, of the length of the text the openers stand in

 @param starts - index of the `<` of each opener of a comment that never closes, in text order

 @returns Those of the openers that start a block of raw HTML, in text order; none for a text nested past what the plain
 reading admits

 @example
 ```ts
 const blocks = blockOpenerStarts({ text: '> a\n> <!-- c\n', starts: [6,], },); // [6]
 ```
 */
export function blockOpenerStarts(
  {
    text,
    starts,
  }: {
    readonly text: string;
    readonly starts: readonly number[];
  },
): readonly number[] {
  if (starts.length === 0)
    return [];
  /**
   The text as the parse reads it, with every opener as it stands.
   */
  const body = asTheParseReadsIt({ text, },);
  if (firstNestingExcess({
    body,
    grammar: 'markdown',
  },)
    .kind
    === 'beyond') {
    return [];
  }
  /**
   Openers found.
   */
  const found: number[] = [];
  /**
   Nodes still to read, a work stack so that a deep tree is no deep recursion.
   */
  const pending: TreeNode[] = [parseMarkdownMathBody({ body, },),];
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    if ('children' in node)
      pending.push(...node.children,);
    if ((node.type !== 'html') || (node.position === undefined))
      continue;
    /**
     The block's text and where it starts, its first line's white space included.
     */
    const {
      value,
      position: { start, },
    } = node;
    /**
     The block's first line without the white space it starts with.
     */
    const trimmed = value.trimStart();
    /**
     How many characters of white space that was.
     */
    const lead = value.length - trimmed.length;
    if (!trimmed.startsWith(COMMENT_OPEN,))
      continue;
    /**
     Where the opener stands.
     */
    const opener = (start.offset ?? 0) + lead;
    if (starts.includes(opener,))
      found.push(opener,);
  }
  return found.toSorted(function ascending(
    left,
    right,
  ): number {
    return left - right;
  },);
}

//endregion Source ships open block
