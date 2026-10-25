import type { TreeNode, } from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import { outermostValueOf, } from './claude-schema-blind-positions.test-fixture.ts';
import { pathOf, } from './claude-schema-tree.test-fixture.ts';

//region The loader of modules and the binding of Node read where they are no direct call
// `process.getBuiltinModule` loads any module of Node by the name it is called with, and `process.binding` reaches the
// binding of Node that makes scripts, so each is read only as the callee of a call: the call is told by its text
// (`claude-schema-blind-text-code.test-fixture.ts`) when the string names `vm` or `contextify`, and accepted when it
// names another module or binding. Any other read of either function (a member of it like `call`, `apply` and `bind`,
// an operand of a comma, an arm of a conditional, a value kept by a constant, an argument of a call) can call it
// with a name the search cannot read, so it is reported where it sits.

/**
 What a report says of a pattern that takes `getBuiltinModule` out of a value, or a read of `process.getBuiltinModule`
 where it is no direct call, since the function it names can load `vm` under any name.
 */
export const TAKES_LOADER = 'getBuiltinModule taken out of a value loads modules the search cannot read';

/**
 What a report says of a read of `process.binding` where it is no direct call, since the binding it names makes scripts
 when it is called with the string `contextify`.
 */
const TAKES_BINDING = 'binding of Node taken out of a call runs text the search cannot read';

/**
 The members of `process` that are read only as the callee of a call, each with what a report says of a read elsewhere.
 */
const PROCESS_FUNCTIONS: ReadonlyMap<string, string> = new Map([
  [
    'binding',
    TAKES_BINDING,
  ],
  [
    'getBuiltinModule',
    TAKES_LOADER,
  ],
]);

/**
 What a report says of a member of `process` that is read only as a callee, empty for any other node.

 @param node - node read, of any kind

 @returns Report text when the node is `process.getBuiltinModule` or `process.binding`, whether `process` is read off
 the global object or not and whatever marks it is written under, empty for any other node

 @example
 ```ts
 const text = loaderReadText({ node, },);
 ```
 */
function loaderReadText({ node, }: { readonly node: TreeNode; },): string {
  if (node.type !== 'MemberExpression')
    return '';
  /**
   The names the member is read through.
   */
  const path = pathOf({ node, },);
  return (path.at(-2,) === 'process') ? (PROCESS_FUNCTIONS.get(path.at(-1,) ?? '',) ?? '') : '';
}

/**
 Whether a node is `process.getBuiltinModule` or `process.binding`, wherever it sits.

 @param node - node read, of any kind

 @returns Whether the node is such a member, which the file must be read further for

 @example
 ```ts
 const names = namesLoaderMember({ node, },);
 ```
 */
export function namesLoaderMember({ node, }: { readonly node: TreeNode; },): boolean {
  return loaderReadText({ node, },) !== '';
}

/**
 What a report says of a read of `process.getBuiltinModule` or `process.binding` where it is no direct call.

 @param node - node read, of any kind

 @param parents - each node's parent

 @returns Report text, empty for any other node and for a member that is the callee of a call, under the marks that
 leave it the same

 @example
 ```ts
 const position = loaderReadPositionOf({ node, parents, },);
 ```
 */
export function loaderReadPositionOf(
  {
    node,
    parents,
  }: {
    readonly node: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
  },
): string {
  /**
   What a report says of the member, empty when the node is no such member.
   */
  const text = loaderReadText({ node, },);
  if (text === '')
    return '';
  /**
   The expression under the marks that leave it the same value.
   */
  const top = outermostValueOf({
    node,
    parents,
  },);
  /**
   The node that holds the member.
   */
  const holder = parents.get(top,);
  return ((holder?.type === 'CallExpression') && (holder.callee === top)) ? '' : text;
}

//endregion The loader of modules and the binding of Node read where they are no direct call
