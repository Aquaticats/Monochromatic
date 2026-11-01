import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { Root, } from 'mdast';

import {
  NO_NODE_BOUNDS,
  nodeBounds,
} from './footnote-unpositioned-runs.ts';
import { flattenContainers, } from './unwrap-container.ts';

//region Footnote definition starts
// WHERE THE FOOTNOTE GRAPH READS A DEFINITION OFF ONE PARSE: a top-level block
// of the tree, containers dissolved, which the graph reads as a definition
// (`footnote-graph.ts`). A definition nested in a list item, a block quote or
// another definition is none to it, and the mention scan
// (`footnote-mentions.ts`) asks this to give a label the role the graph gives
// it.

/**
 Offsets of the `[` of every footnote definition one parse reads as a
 top-level block, containers dissolved.

 @param root - tree one grammar read a text into

 @returns Offsets in the text the parser read

 @throws {@link Error} when a parsed footnote definition carries no offset,
 though the parser sets one on every node it builds

 @example
 ```ts
 const starts = definitionStartsOf({ root: parseMarkdownBody({ body: '[^1]: note', },), },);
 ```
 */
export function definitionStartsOf(
  { root, }: { readonly root: ForeignBorrowed<Root>; },
): ReadonlySet<number> {
  /**
   Offsets found so far.
   */
  const starts = new Set<number>();
  for (const block of flattenContainers({ children: root.children, },)
    .blocks) {
    if (block.type !== 'footnoteDefinition')
      continue;
    /**
     The definition's own span, absent on a node the transform rebuilt.
     */
    const bounds = nodeBounds(block,);
    if (bounds === NO_NODE_BOUNDS)
      throw new Error(
        'unreachable: a parsed footnote definition carries no position, though the parser sets one on every node it '
          + 'builds',
      );
    starts.add(bounds.start,);
  }
  return starts;
}

//endregion Footnote definition starts
