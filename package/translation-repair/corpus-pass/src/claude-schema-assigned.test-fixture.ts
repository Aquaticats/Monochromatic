import type { TreeNode, } from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import { namesBoundBy, } from './claude-schema-blind-positions.test-fixture.ts';

//region The names a file assigns again
// A name that holds a function (`let g = gatherThroughCache;`) stands for that function only while nothing else is
// assigned to it. The names a file assigns anywhere, in any scope: by an assignment, by a pattern an assignment takes
// apart, and by the head of a `for...in` or `for...of` loop that declares nothing. A name that a declaration binds and
// that is never assigned is not among them, whatever the declaration's keyword.

/**
 The kinds of loop whose head can assign a name instead of declaring one.
 */
const ASSIGNING_LOOP_KINDS: ReadonlySet<string> = new Set([
  'ForInStatement',
  'ForOfStatement',
],);

/**
 The names a node assigns to, when it is an assignment or a loop whose head assigns.

 @param node - node read, of any kind

 @returns Names in order, none for any other node

 @example
 ```ts
 const names = namesAssignedBy({ node, },);
 ```
 */
function namesAssignedBy({ node, }: { readonly node: TreeNode; },): readonly string[] {
  if (node.type === 'AssignmentExpression')
    return namesBoundBy({ node: node.left, },);
  return ASSIGNING_LOOP_KINDS.has(node.type,)
    ? namesBoundBy({ node: node.left, },)
    : [];
}

/**
 The names a file assigns anywhere.

 @param nodes - every node of the file

 @returns Names, each once

 @example
 ```ts
 const assigned = assignedNamesIn({ nodes: nodesUnder({ root: program, },), },);
 ```
 */
export function assignedNamesIn({ nodes, }: { readonly nodes: readonly TreeNode[]; },): ReadonlySet<string> {
  return new Set(nodes.flatMap(function assignedByNode(node,): readonly string[] {
    return namesAssignedBy({ node, },);
  },),);
}

//endregion The names a file assigns again
