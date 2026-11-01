/**
 Tests for the tree walker the formula floor and the italic-title pass share
 (ledger B72): every node once, the root included, and a tree deeper than a
 recursive walk survives. Fixtures are cat-themed invention only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { treeNodes, } from '@monochromatic-dev/translation-repair-markup';
import { parseMdxBody, } from '../dist/final/node/index.mjs';

/**
 A node as the built package types it, which bundles its own copy of the
 mdast types.
 */
type TreeNode = Parameters<typeof treeNodes>[0]['root'];

/**
 A node that may stand inside a paragraph.
 */
type PhrasingNode = Extract<TreeNode, { readonly type: 'emphasis'; }>['children'][number];

/**
 Levels of emphasis the deep case nests, far past the frames a recursive walk
 of one call per level gets from Node's default stack.
 */
const DEEP_LEVELS = 100_000;

await describe({
  name: treeNodes.name,
  children: [
    it({
      name: 'RETURNS every node of a parsed tree once, the root included',
      fn: async () => {
        expect(
          treeNodes({ root: parseMdxBody({ body: 'The cat *naps* in **sun**.', },), },)
            .map(function typeOf(node,): string {
              return node.type;
            },)
            .toSorted(),
        ).toEqual([
          'emphasis',
          'paragraph',
          'root',
          'strong',
          'text',
          'text',
          'text',
          'text',
          'text',
        ],);
      },
    },),
    it({
      name: 'RETURNS a node with no children as the only node',
      fn: async () => {
        /**
         A text node standing alone.
         */
        const leaf: TreeNode = { type: 'text', value: 'nap', };
        expect(treeNodes({ root: leaf, },),).toEqual([leaf,],);
      },
    },),
    it({
      name: 'WALKS a tree nested deeper than a recursive walk survives, every level counted',
      fn: async () => {
        /**
         Emphasis nested level on level around one word.
         */
        const root = Array
          .from({ length: DEEP_LEVELS, },)
          .reduce<PhrasingNode>(
          function wrap(inner,): PhrasingNode {
            return { type: 'emphasis', children: [inner,], };
          },
          { type: 'text', value: 'nap', },
          );
        expect(treeNodes({ root, },),).toHaveLength(DEEP_LEVELS + 1,);
      },
    },),
  ],
},);
