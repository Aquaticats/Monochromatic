import type { Nodes, } from 'mdast';

import type { DeepReadonlyData, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Typography shown view
// THE TEXT A READER SEES AROUND A MARK, which the pairing of single quotes reads. A unit the prose view does not read as words
// shows a reader nothing between two pieces of text: the delimiters of emphasis, strike and code, the brackets of a link with its
// destination and title, a tag, a comment, an expression. A mark beside one of them stands beside the text on its other side
// (`<b>'hi'</b>` opens at the start of the text and closes at its end, and the `'n'` of `rock *'n'* roll` is the elision pair it
// is without the asterisks). The content of inline code, math and a code block is shown, though no rule writes in it, so it
// stays between the marks as the characters it is (`` `code`'s ``). A line break is kept whatever it belongs to, since no
// quotation pair crosses one. The view is the text without the units that show nothing; `writtenBack` puts the units a rule wrote
// into the text as written.

/**
 A node of the tree the slice grammar reads, which it hands over read-only.
 */
type TreeNode = DeepReadonlyData<Nodes>;

/**
 The offsets of a stretch of the body.
 */
export type ShownRange = {
  /**
   Offset where the stretch starts.
   */
  readonly start: number;

  /**
   Exclusive offset where it ends.
   */
  readonly end: number;
};

/**
 The node types whose whole stretch is shown to a reader though it is no prose: math and code blocks. Inline code shows its content
 and not the backticks around it.
 */
const SHOWN_NODE_TYPES: ReadonlySet<string> = new Set([
  'code',
  'inlineMath',
  'math',
],);

/**
 How many backticks open an inline code span, counted from its start.

 @param masked - the text the parser read

 @param start - offset where the span starts

 @param end - exclusive offset where the span ends

 @returns The length of the run of backticks at the start, at most half the span

 @example
 ```ts
 const width = backtickRunAt({ masked: '``a``', start: 0, end: 5, },); // 2
 ```
 */
function backtickRunAt(
  {
    masked,
    start,
    end,
  }: {
    readonly masked: string;
    readonly start: number;
    readonly end: number;
  },
): number {
  /**
   Where the run has reached, mutated only inside this function.
   */
  const run = { width: 0, };
  while (((start + run.width) < end) && (masked.charAt(start + run.width,) === '`'))
    run.width += 1;
  return Math.min(
    run.width,
    Math.floor((end - start) / 2),
  );
}

/**
 The stretches of a tree a reader sees although no rule writes in them: the content of inline code, inline math, math blocks and
 code blocks. Walked with a work stack rather than by recursion, since a tree is as deep as the nesting a page writes.

 @param root - the tree's top node

 @param masked - the text the parser read

 @returns The stretches, in no promised order

 @example
 ```ts
 const shown = shownRangesOf({ root, masked: 'a `b` c', },); // one stretch, offset 3 to 4
 ```
 */
export function shownRangesOf(
  {
    root,
    masked,
  }: {
    readonly root: TreeNode;
    readonly masked: string;
  },
): readonly ShownRange[] {
  /**
   Stretches met so far, mutated only inside this function.
   */
  const ranges: ShownRange[] = [];
  /**
   Nodes still to visit.
   */
  const pending: TreeNode[] = [root,];
  // Each node in turn, until the stack is empty.
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    /**
     The node's type, read as text since the extensions add types the tree's own union does not list.
     */
    const kind: string = node.type;
    /**
     Where the parser put the node.
     */
    const start = node.position
      ?.start
      .offset;
    /**
     Where the parser ended the node.
     */
    const end = node.position
      ?.end
      .offset;
    if ((start !== undefined) && (end !== undefined)) {
      if (SHOWN_NODE_TYPES.has(kind,)) {
        ranges.push({
          start,
          end,
        },);
      }
      if (kind === 'inlineCode') {
        /**
         How many backticks open and close the span.
         */
        const width = backtickRunAt({
          masked,
          start,
          end,
        },);
        ranges.push({
          start: start + width,
          end: end - width,
        },);
      }
    }
    if ('children' in node)
      pending.push(...(node.children as readonly TreeNode[]),);
  }
  return ranges;
}

/**
 The text a reader sees: the units a reader sees and every line break.

 @param text - the text as written

 @param visible - one flag per unit, true where a reader sees the unit

 @returns The view's text, and for each of its units the offset it has in the text

 @example
 ```ts
 const view = shownViewOf({ text: '*a*', visible: [false, true, false,], },); // text 'a', offsets [1]
 ```
 */
export function shownViewOf(
  {
    text,
    visible,
  }: {
    readonly text: string;
    readonly visible: readonly boolean[];
  },
): {
  readonly text: string;
  readonly offsets: readonly number[];
} {
  /**
   Offsets of the units kept, mutated only inside this function.
   */
  const offsets: number[] = [];
  for (let index = 0; index < text.length; index += 1) {
    if ((visible[index] === true) || (text.charAt(index,) === '\n'))
      offsets.push(index,);
  }
  return {
    text: offsets
      .map(function unitAt(offset,): string {
        return text.charAt(offset,);
      },)
      .join('',),
    offsets,
  };
}

/**
 Puts the units a rule wrote in the view back into the text as written.

 @param text - the text as written

 @param offsets - for each unit of the view, its offset in the text (`shownViewOf`)

 @param units - the view's units after the rule, one for each of the view's

 @returns The text with each view unit replaced by what the rule wrote for it

 @example
 ```ts
 const back = writtenBack({ text: '*a*', offsets: [1,], units: ['b',], },); // '*b*'
 ```
 */
export function writtenBack(
  {
    text,
    offsets,
    units,
  }: {
    readonly text: string;
    readonly offsets: readonly number[];
    readonly units: readonly string[];
  },
): string {
  /**
   The text by unit, mutated only inside this function.
   */
  const written = Array.from(
    { length: text.length, },
    function unitOf(
      _unused,
      index,
    ): string {
      return text.charAt(index,);
    },
  );
  for (let at = 0; at < offsets.length; at += 1) {
    /**
     Where this view unit stands in the text.
     */
    const offset = offsets[at];
    if (offset !== undefined)
      written[offset] = units[at] ?? text.charAt(offset,);
  }
  return written.join('',);
}

//endregion Typography shown view
