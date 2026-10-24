import type { TreeNode, } from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import type { BlindContext, } from './claude-schema-blind-positions.test-fixture.ts';
import { isNoNode, } from './claude-schema-tree.test-fixture.ts';

//region Nodes whose value the walk of a builder's call is still to follow
// What `claude-schema-blind-flow.test-fixture.ts` keeps while it walks: each node with what the names of its file are
// read against, since the walk goes into other files for a constant a file exports.

/**
 A node whose value is still to follow, with what the names of its file are read against.
 */
export type Pending = {
  /**
   What the names of the file that holds the node are read against.
   */
  readonly context: BlindContext;

  /**
   The node.
   */
  readonly node: TreeNode;
};

/**
 What the other files read of a name: the nodes to follow and the places reported at once.
 */
type AcrossFiles = {
  /**
   Nodes to follow in the other files.
   */
  readonly pending: readonly Pending[];

  /**
   The places reported at once.
   */
  readonly strays: readonly string[];
};

/**
 The nodes of one file as pending nodes of that file.

 @param context - what the names of the file are read against

 @param nodes - nodes that hold the value

 @returns One pending node for each

 @example
 ```ts
 const pending = pendingIn({ context, nodes: [call,], });
 ```
 */
export function pendingIn(
  {
    context,
    nodes,
  }: {
    readonly context: BlindContext;
    readonly nodes: readonly TreeNode[];
  },
): readonly Pending[] {
  return nodes.map(function heldIn(node,): Pending {
    return {
      context,
      node,
    };
  },);
}

/**
 Where the other files of the sources read a name that the top level of a file binds, and the places reported at once
 for what they read of it.

 @param context - what the names of the file are read against

 @param name - name bound at the top level

 @returns Nodes to follow in the other files, with the context of each, and the places reported at once

 @example
 ```ts
 const { pending, strays, } = acrossFilesOf({ context, name: 'FORMAT', });
 ```
 */
export function acrossFilesOf(
  {
    context,
    name,
  }: {
    readonly context: BlindContext;
    readonly name: string;
  },
): AcrossFiles {
  /**
   The places in the other files that read the name.
   */
  const readers = context.constants({
    file: context.file,
    name,
  },);
  return {
    pending: readers
      .filter(function holdsNode(reader,): boolean {
        return !isNoNode({ node: reader.node, },);
      },)
      .map(function heldIn(reader,): Pending {
        return {
          context: reader.context,
          node: reader.node,
        };
      },),
    strays: readers
      .map(function strayOf({ stray, },): string {
        return stray;
      },)
      .filter(function isReported(stray,): boolean {
        return stray !== '';
      },),
  };
}

//endregion Nodes whose value the walk of a builder's call is still to follow
