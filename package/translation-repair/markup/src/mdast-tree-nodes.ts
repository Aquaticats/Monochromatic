import type { Nodes, } from 'mdast';

//region Tree nodes
// Every node of a parsed tree, walked with a work stack rather than by
// recursion, since a tree is as deep as the nesting a page writes. Moved here
// from `translate-formula.ts` when the italic-title pass came to read emphasis
// off the parse too (ledger B72).

/**
 Every node of a tree, in no promised order.

 @param root - tree's top node

 @returns The tree's nodes

 @example
 ```ts
 const nodes = treeNodes({ root: parseMdxBody({ body: 'Naps.', },), },);
 ```
 */
export function treeNodes({ root, }: { readonly root: Nodes; },): readonly Nodes[] {
  /**
   Nodes met so far, which is what the function returns.
   */
  const met: Nodes[] = [];
  /**
   Nodes still to visit.
   */
  const pending: Nodes[] = [root,];
  // Each node in turn, until the stack is empty.
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    met.push(node,);
    if ('children' in node)
      pending.push(...(node.children as readonly Nodes[]),);
  }
  return met;
}

//endregion Tree nodes
