import type { Root, } from 'mdast';
import {
  NO_NODE_BOUNDS,
  nodeBounds,
  type TreeNode,
} from './footnote-unpositioned-runs.ts';
import { noteHeadingFloor, } from './heading-floor-store.ts';

//region Heading floors
// WHERE A SETEXT HEADING'S OWN LINES START, when the parser starts the heading earlier. A setext heading is the
// content of a paragraph that an underline closes, and micromark resolves the link reference definitions at the start
// of that content before it reads the rest as the heading, so the heading node's position begins at the first
// definition and the definition nodes before it overlap it (`[a]: u\n[ x\n---` gives a definition at 0 to 6 and a
// heading at 0 to the end). The footnote reading takes the raw text of a heading as the text beside the markup of the
// heading, which holds no square bracket, and a definition holds two, so it needs the offset after the last definition.

/**
 Notes, for each setext heading of a tree that follows link reference definitions, the offset its own lines start at.

 @param root - tree the parser has just built, with its positions final; nothing is changed in it

 @example
 ```ts
 noteHeadingFloors({ root: unified().use(remarkParse,).parse('[a]: u\n[ x\n---\n',), },);
 ```
 */
export function noteHeadingFloors({ root, }: { readonly root: Root; }): void {
  /**
   Nodes still to visit, so a deep tree is walked without recursion.
   */
  const pending: TreeNode[] = [root,];
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    if (!('children' in node))
      continue;
    /**
     The node's children, in source order.
     */
    const { children, } = node;
    children.forEach(function noteFloor(
      child,
      index,
    ): void {
      /**
       The block before this one, which a definition can be.
       */
      const before = children[index - 1];
      if (child.type !== 'heading')
        return;
      if (before === undefined)
        return;
      if (before.type !== 'definition')
        return;
      /**
       Where the definition stands.
       */
      const definition = nodeBounds(before,);
      /**
       Where the heading stands.
       */
      const heading = nodeBounds(child,);
      if ((definition === NO_NODE_BOUNDS) || (heading === NO_NODE_BOUNDS))
        return;
      if (heading.start < definition.end) {
        noteHeadingFloor({
          heading: child,
          floor: definition.end,
        },);
      }
    },);
    pending.push(...children,);
  }
}

//endregion Heading floors
