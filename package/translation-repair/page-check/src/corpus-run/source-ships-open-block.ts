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
 Whether a text holds nothing but white space.

 @param text - text

 @returns Whether every character is white space; true for an empty text

 @example
 ```ts
 const blank = isBlank({ text: ' \t', },); // true
 ```
 */
function isBlank({ text, }: { readonly text: string; },): boolean {
  return text.trim() === '';
}

/**
 Characters that stand before the label of a footnote definition on its line: the marks of block quotes, the indentation and
 the marker of a list item.
 */
const BEFORE_LABEL = ' \t>-*+0123456789.)';

/**
 Characters that cannot stand in a footnote label.
 */
const LABEL_BREAKS: ReadonlySet<string> = new Set([
  ']',
  ' ',
  '\t',
],);

/**
 Whether an index follows the label of a footnote definition on its line, with nothing between but white space.

 @param text - text

 @param at - index to look before

 @returns Whether the characters before the index on its line are the marks of containers, a label `[^name]:` and white
 space

 @example
 ```ts
 const after = followsFootnoteLabel({ text: '> [^1]: <!-- c', at: 9, },); // true
 ```
 */
function followsFootnoteLabel(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  /**
   The line before the index, up to the index.
   */
  const line = text.slice(
    text.lastIndexOf(
      '\n',
      at - 1,
    ) + 1,
    at,
  );
  /**
   The line less the white space before the index.
   */
  const head = line.trimEnd();
  if (!head.endsWith(']:',))
    return false;
  /**
   Where the label opens.
   */
  const open = head.lastIndexOf('[^',);
  if (open === (-1))
    return false;
  /**
   What stands in the label.
   */
  const label = head.slice(
    open + 2,
    -2,
  );
  if (Array.from(label,).some(function endsLabel(character,): boolean {
    return LABEL_BREAKS.has(character,);
  },))
    return false;
  return Array.from(head.slice(
    0,
    open,
  ),)
    .every(function isMark(character,): boolean {
      return BEFORE_LABEL.includes(character,);
    },);
}

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
     Where the block starts, as a position.
     */
    const { start, } = node.position;
    /**
     Where the block starts in the text.
     */
    const from = start.offset ?? 0;
    /**
     Where the first opener at or after the start stands: the block starts at the opener or at white space before it, and a
     tab that a list item takes in part is read as spaces the text does not hold, so the opener is looked for in the text and
     not counted off the characters of the block's value.
     */
    const opener = body.indexOf(
      COMMENT_OPEN,
      from,
    );
    if (opener === (-1))
      continue;
    /**
     What stands between the start of the block and the opener.
     */
    const gap = body.slice(
      from,
      opener,
    );
    if (isBlank({ text: gap, },) && starts.includes(opener,))
      found.push(opener,);
  }
  // An opener right after the label of a footnote definition starts the block of raw HTML the definition opens with, which
  // the plain reading, with no footnotes, does not see.
  for (const start of starts) {
    if ((!found.includes(start,)) && followsFootnoteLabel({
      text,
      at: start,
    },))
      found.push(start,);
  }
  return found.toSorted(function ascending(
    left,
    right,
  ): number {
    return left - right;
  },);
}

//endregion Source ships open block
