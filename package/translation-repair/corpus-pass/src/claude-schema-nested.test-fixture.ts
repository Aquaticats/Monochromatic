import type { TreeNode, } from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import { gatherDeclaration, } from './claude-schema-declare.test-fixture.ts';
import type { Binding, } from './claude-schema-facts.test-fixture.ts';
import {
  moduleHeadOf,
  nodeAt,
  nodesAt,
  settledSpecifierOf,
} from './claude-schema-tree.test-fixture.ts';

//region Bindings a file makes below its top level, and the loads it reads in place
// What `claude-schema-module.test-fixture.ts` adds to the bindings of a file beyond its top-level statements: a name
// that a variable declaration inside a function or a block binds to a loaded module or to another name, and the module
// that a `(await import(path)).name` or a `require(path).name` reads in place. Both are read as the file's own.

/**
 A set of bound names, each with what the name stands for.
 */
type NamedBindings = readonly (readonly [
  string,
  Binding,
])[];

/**
 The kinds of node that can be a load: an `import()`, a `require` call, and an `await` of either.
 */
const LOAD_KINDS: ReadonlySet<string> = new Set([
  'AwaitExpression',
  'CallExpression',
  'ImportExpression',
],);

/**
 The bindings of the loads a file reads in place, each under the name a chain of names starts with.

 @param nodes - every node of the file

 @returns One entry per module a settled `import()` or `require` of a literal path loads, anywhere in the file

 @example
 ```ts
 const loads = loadBindingsIn({ nodes: nodesUnder({ root: program, },), },);
 ```
 */
export function loadBindingsIn({ nodes, }: { readonly nodes: readonly TreeNode[]; },): NamedBindings {
  return nodes
    .flatMap(function bindingOfLoad(node,): NamedBindings {
      /**
       The literal path the node loads, empty when it loads none.
       */
      const specifier = LOAD_KINDS.has(node.type,) ? settledSpecifierOf({ node, },) : '';
      return (specifier === '')
        ? []
        : [
          [
            moduleHeadOf({ specifier, },),
            {
              kind: 'module',
              specifier,
              members: [],
            },
          ],
        ];
    },);
}

/**
 The bindings that variable declarations make inside functions and blocks, which the file reads as its own: the name
 of a constant that holds a loaded module or another name, or takes members out of one. The first binding of a name,
 in the order of the text, is the one kept.

 @param program - file's program node

 @param nodes - every node of the file

 @returns One entry per name, in the order of the text

 @example
 ```ts
 const nested = nestedBindingsIn({ program, nodes: nodesUnder({ root: program, },), },);
 ```
 */
export function nestedBindingsIn(
  {
    program,
    nodes,
  }: {
    readonly program: TreeNode;
    readonly nodes: readonly TreeNode[];
  },
): ReadonlyMap<string, Binding> {
  /**
   The declarations of the top level, which the file's own statements gather.
   */
  const topLevel = new Set(nodesAt({
    node: program,
    field: 'body',
  },)
    .flatMap(function declarationsOf(statement,): readonly TreeNode[] {
      return [
        statement,
        nodeAt({
          node: statement,
          field: 'declaration',
        },),
      ];
    },),);
  /**
   The nested declarations, in the order of the text.
   */
  const declarations = nodes
    .filter(function isNestedDeclaration(node,): boolean {
      return (node.type === 'VariableDeclaration') && (!topLevel.has(node,));
    },)
    .toSorted(function inTextOrder(
      first,
      second,
    ): number {
      return first.start - second.start;
    },);
  return new Map(declarations
    .flatMap(function bindingsOf(declaration,): NamedBindings {
      /**
       Everything the declaration binds.
       */
      const { bindings, } = gatherDeclaration({ declaration, },);
      return bindings.filter(function isFollowed([, binding,],): boolean {
        return (binding.kind === 'module') || (binding.kind === 'path');
      },);
    },)
    .toReversed(),);
}

//endregion Bindings a file makes below its top level, and the loads it reads in place
