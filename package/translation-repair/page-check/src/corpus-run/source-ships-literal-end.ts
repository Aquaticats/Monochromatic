import type { TreeNode, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { isIndentedCode, } from './source-ships-footnote.ts';
import { rangeOf, } from './source-ships-ranges.ts';

//region Source ships literal end
// A BLOCK THAT ENDS IN A RAW HTML BLOCK OR IN INDENTED CODE. The strict grammar has neither, and reads their lines as a
// paragraph, so a line right after one that the plain reading takes for a definition is a lazy line of that paragraph
// and a text, as it is after a footnote definition (`source-ships-footnote.ts`). The block that ends in one is the
// raw HTML block or the indented code itself, or a block quote or a list whose last block is one.

/**
 Node types whose last child is the end of the block.
 */
const CONTAINERS: ReadonlySet<string> = new Set([
  'blockquote',
  'list',
  'listItem',
],);

/**
 Whether a block ends in a raw HTML block or in indented code.

 @param source - the text

 @param node - block directly under the root

 @returns Whether the last block in it, through block quotes and lists, is a raw HTML block or indented code

 @example
 ```ts
 const ends = endsInLiteral({ source, node, },);
 ```
 */
export function endsInLiteral(
  {
    source,
    node,
  }: {
    readonly source: string;
    readonly node: TreeNode;
  },
): boolean {
  if (node.type === 'html')
    return true;
  if (CONTAINERS.has(node.type,) && ('children' in node)) {
    /**
     The children of the block.
     */
    const { children, } = node;
    /**
     The last of them.
     */
    const last = children.at(-1,);
    return (last !== undefined) && endsInLiteral({
      source,
      node: last,
    },);
  }
  return isIndentedCode({
    source,
    node,
    range: rangeOf({ node, },),
  },);
}

//endregion Source ships literal end
