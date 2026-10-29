import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import { parseSliceBody, } from './parse-slice-body.ts';

//region Markdown blocks
// TOP-LEVEL BLOCKS OF A PASSAGE, for the target-only run, which splices the
// archive's own text and so needs each block's exact bytes, kind and place.
//
// READ OFF THE PARSE (ledger B68), not split on blank lines. A split is not
// what the parser reads: a blockquote may open on the line after a
// paragraph's with no blank line between, and a fence carrying a blank line
// between its own lines is one block to the parser and two to a split. The
// quote guard met the same shape first and counts off the parse the floor
// reads (ledger B42, `quote-preservation.ts`).
//
// NO CARRIAGE-RETURN FOLD IS NEEDED. A split looking for `\n\n` found no
// boundary in a CRLF file (`people/gqt/page.md` is the one such file in the
// pinned corpus) and read the whole page as one block. The slice grammar reads
// CRLF line endings, and its offsets index the text as written, `\r` included.

/**
 One top-level block as the parse reads it: its exact source bytes, its node
 kind, and the offsets those bytes occupy in the text it was read from.

 @example
 ```ts
 const block: ParsedBlock = { text: 'One.', kind: 'paragraph', startOffset: 0, endOffset: 4, };
 ```
 */
export type ParsedBlock = {
  /**
   Exact source bytes of this block, untrimmed.
   */
  readonly text: string;

  /**
   mdast node type (`paragraph`, `blockquote`, `mdxJsxFlowElement` and the
   rest), kept as a plain string because remark plugins extend the
   vocabulary, or `containerTag` for a container's opening or closing tag
   standing alone in the passage, which the grammar holds aside rather than
   parsing.
   */
  readonly kind: string;

  /**
   Start offset within the text this block was read from.
   */
  readonly startOffset: number;

  /**
   Exclusive end offset within the text this block was read from.
   */
  readonly endOffset: number;
};

/**
 Splits a passage into its top-level blocks, each with its own exact bytes,
 kind and offsets.

 A CONTAINER TAG STANDING ALONE IS A BLOCK HERE. The slice grammar masks a
 lone `<details>` or `</details>` so the rest of the passage parses, and a
 passage cut inside a container ends on one; the target-only run anchors on
 the source's last block, so leaving the tag out would anchor one block too
 early and carry the tag into the protected run as well as the lane's own
 rendering.

 @param text - passage to read

 @returns Its blocks, in order, untrimmed

 @throws {@link import('./parse-mdx.ts').MdxParseError} when the shared slice grammar refuses this text

 @example
 ```ts
 const blocks = parsedTopLevelBlocks({ text: 'One.\n\nTwo.', },);
 ```
 */
export function parsedTopLevelBlocks({ text, }: { readonly text: string; },): readonly ParsedBlock[] {
  /**
   The grammar's reading and the container tags it held aside.
   */
  const {
    root,
    tags,
  } = parseSliceBody({ text, },);
  return [
    ...root
      .children
      .map(function toBlock(node,): ParsedBlock {
        /**
         This node's start offset.
         */
        const startOffset = nonNullishOrThrow(node.position
          ?.start
          .offset,);
        /**
         This node's exclusive end offset.
         */
        const endOffset = nonNullishOrThrow(node.position
          ?.end
          .offset,);
        return {
          text: text.slice(
            startOffset,
            endOffset,
          ),
          kind: node.type,
          startOffset,
          endOffset,
        };
      },),
    ...tags.map(function toTagBlock(tag,): ParsedBlock {
      return {
        text: tag.text,
        kind: 'containerTag',
        startOffset: tag.startOffset,
        endOffset: tag.endOffset,
      };
    },),
  ].toSorted(function inPassageOrder(
    left,
    right,
  ): number {
    return left.startOffset - right.startOffset;
  },);
}

//endregion Markdown blocks
