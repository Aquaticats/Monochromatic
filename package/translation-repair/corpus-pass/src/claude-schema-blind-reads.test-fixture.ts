import {
  identifierName,
  isTreeNode,
  type TreeNode,
  unwrapped,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import type { BlindContext, } from './claude-schema-blind-positions.test-fixture.ts';
import {
  isNoNode,
  NO_NODE,
} from './claude-schema-tree.test-fixture.ts';

//region Where a name bound in a file is read
// What the walk of a builder's value (`claude-schema-blind-flow.test-fixture.ts`) and the walk across files
// (`claude-schema-blind-constants.test-fixture.ts`) both ask of a file: the places its code reads a name, inside a
// function or in the whole file.

/**
 The identifier a chain of members starts with, `a` in `a.b.c`.

 @param node - chain, or a plain name, of any kind or none

 @returns Identifier, `NO_NODE` when the chain starts with anything else

 @example
 ```ts
 const head = headIdentifierOf({ node: chain, },);
 ```
 */
function headIdentifierOf({ node, }: { readonly node: unknown; },): TreeNode {
  /**
   The chain without the marks around it.
   */
  const { inner, } = unwrapped({ node, },);
  if (!isTreeNode(inner,))
    return NO_NODE;
  if (inner.type === 'Identifier')
    return inner;
  return (inner.type === 'MemberExpression')
    ? headIdentifierOf({ node: inner.object, },)
    : NO_NODE;
}

/**
 The references to a name inside a function, other than the place that binds it, also where a member is read off the
 name.

 @param context - what the file's names are read against

 @param name - name bound

 @param scope - function that holds the binding, or the program for the whole file

 @returns Identifiers that read the name

 @example
 ```ts
 const reads = readsOfName({ context, name: 'format', scope, });
 ```
 */
export function readsOfName(
  {
    context,
    name,
    scope,
  }: {
    readonly context: BlindContext;
    readonly name: string;
    readonly scope: TreeNode;
  },
): readonly TreeNode[] {
  return context.references
    .map(function headOfVisit({ node, },): TreeNode {
      return headIdentifierOf({ node, },);
    },)
    .filter(function readsName(head,): boolean {
      return (!isNoNode({ node: head, },)) && (identifierName({ node: head, },) === name)
        && (scope.start <= head.start)
        && (head.end <= scope.end);
    },);
}

//endregion Where a name bound in a file is read
