import {
  ancestorsOf,
  identifierName,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import type { BlindContext, } from './claude-schema-blind-positions.test-fixture.ts';
import type { FoundShape, } from './claude-schema-blind-spots.test-fixture.ts';
import { isTypeWithoutValue, } from './claude-schema-reference.test-fixture.ts';
import {
  moduleLoadOf,
  nodeAt,
  nodesAt,
  stringValueOf,
} from './claude-schema-tree.test-fixture.ts';

//region Code made from text by the vm module and by the constructor of a function
// Besides `eval` and `Function` (`claude-schema-blind-unreadable.test-fixture.ts`), code is made from text by the `vm`
// module and by the constructor of a function, which is the `Function` constructor. Both can be told by a read of the
// text. The module is told by its specifier, `vm` or `node:vm`, in an import, an export from, a call of `require`, an
// `import()` or the module of an import-equals declaration. The constructor is told by a member named `constructor`
// written with a dot or with a string literal in brackets, also through `?.`, and by `Reflect.get` with the string
// `constructor` as its key. A key that is no literal and a pattern that takes `constructor` out of a value are not told,
// and are limits of the search that the ledger states, as are the keys of `Reflect.get` that are no literal.

/**
 What a report says of a place that loads the `vm` module.
 */
const LOADS_VM = 'vm runs text the search cannot read';

/**
 What a report says of a place that reads the constructor of a value.
 */
const READS_CONSTRUCTOR = 'constructor of a function builds code from text the search cannot read';

/**
 The specifiers that name the `vm` module.
 */
const VM_SPECIFIERS: ReadonlySet<string> = new Set([
  'node:vm',
  'vm',
],);

/**
 The kinds of node that carry the path of a module in their `source` field.
 */
const SOURCE_KINDS: ReadonlySet<string> = new Set([
  'ExportAllDeclaration',
  'ExportNamedDeclaration',
  'ImportDeclaration',
],);

/**
 The module a node names, when the node is an import, an export from, a load or the module of an import-equals
 declaration.

 @param node - node read, of any kind

 @returns Specifier as written, empty for any other node and for a path that is no single literal

 @example
 ```ts
 const specifier = moduleNamedBy({ node, },);
 ```
 */
function moduleNamedBy({ node, }: { readonly node: TreeNode; },): string {
  if (SOURCE_KINDS.has(node.type,))
    return stringValueOf({ node: node.source, },);
  if (node.type === 'TSExternalModuleReference')
    return stringValueOf({ node: node.expression, },);
  if ((node.type !== 'ImportExpression') && (node.type !== 'CallExpression'))
    return '';
  /**
   The load the node is, if it is one.
   */
  const load = moduleLoadOf({ node, },);
  return load.isLoad ? load.specifier : '';
}

/**
 Whether a member is read by the name `constructor`, with a dot or with a string literal.

 @param node - member expression

 @returns Whether the key is `constructor`

 @example
 ```ts
 const reads = readsConstructor({ node: member, },);
 ```
 */
function readsConstructor({ node, }: { readonly node: TreeNode; },): boolean {
  /**
   The key of the member.
   */
  const key = nodeAt({
    node,
    field: 'property',
  },);
  return (node.computed === true)
    ? (stringValueOf({ node: key, },) === 'constructor')
    : (identifierName({ node: key, },) === 'constructor');
}

/**
 Whether a call is `Reflect.get` with the string `constructor` as its key.

 @param node - call expression

 @returns Whether the callee is `Reflect.get` and the second argument is that string

 @example
 ```ts
 const reads = reflectsConstructor({ node: call, },);
 ```
 */
function reflectsConstructor({ node, }: { readonly node: TreeNode; },): boolean {
  /**
   The callee of the call.
   */
  const callee = nodeAt({
    node,
    field: 'callee',
  },);
  /**
   The second argument of the call, absent when it has none.
   */
  const key = nodesAt({
    node,
    field: 'arguments',
  },)
    .at(1,);
  if ((callee.type !== 'MemberExpression') || (node.type !== 'CallExpression'))
    return false;
  if (callee.computed === true)
    return false;
  /**
   Whether the callee is `Reflect.get`.
   */
  const isReflectGet = (identifierName({ node: callee.object, },) === 'Reflect')
    && (identifierName({ node: callee.property, },) === 'get');
  if (!isReflectGet)
    return false;
  return (key !== undefined) && (stringValueOf({ node: key, },) === 'constructor');
}

/**
 What a node says about code made from text.

 @param node - node read, of any kind

 @returns Report text for a node that loads `vm` or reads a constructor, empty for any other node

 @example
 ```ts
 const position = textCodePositionOf({ node, },);
 ```
 */
function textCodePositionOf({ node, }: { readonly node: TreeNode; },): string {
  if (VM_SPECIFIERS.has(moduleNamedBy({ node, },),))
    return LOADS_VM;
  if ((node.type === 'MemberExpression') && readsConstructor({ node, },))
    return READS_CONSTRUCTOR;
  return ((node.type === 'CallExpression') && reflectsConstructor({ node, },)) ? READS_CONSTRUCTOR : '';
}

/**
 Whether a node loads `vm` or reads a constructor, wherever it sits.

 @param node - node read, of any kind

 @returns Whether the node is such a place

 @example
 ```ts
 const names = namesTextCode({ node, },);
 ```
 */
export function namesTextCode({ node, }: { readonly node: TreeNode; },): boolean {
  return textCodePositionOf({ node, },) !== '';
}

/**
 The places in a file that load `vm` or read the constructor of a value, outside types.

 @param context - what the file's names are read against

 @param nodes - every node of the file

 @returns One entry per place

 @example
 ```ts
 const found = textCodeShapesIn({ context, nodes: nodesUnder({ root: program, },), },);
 ```
 */
export function textCodeShapesIn(
  {
    context,
    nodes,
  }: {
    readonly context: BlindContext;
    readonly nodes: readonly TreeNode[];
  },
): readonly FoundShape[] {
  return nodes.flatMap(function shapesOfNode(node,): readonly FoundShape[] {
    /**
     What the node says about code made from text, empty when nothing.
     */
    const position = textCodePositionOf({ node, },);
    /**
     Whether the node sits inside a type.
     */
    const isInType = ancestorsOf({
      node,
      parents: context.parents,
    },)
      .some(function isType(ancestor,): boolean {
        return isTypeWithoutValue({ node: ancestor, },);
      },);
    return ((position === '') || isInType)
      ? []
      : [
        {
          shape: 'code-the-search-cannot-read',
          node,
          position,
        },
      ];
  },);
}

//endregion Code made from text by the vm module and by the constructor of a function
