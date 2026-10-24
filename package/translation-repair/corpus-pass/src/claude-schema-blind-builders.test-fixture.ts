import {
  identifierName,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import { chainShapesAt, } from './claude-schema-blind-chains.test-fixture.ts';
import type { BlindContext, } from './claude-schema-blind-positions.test-fixture.ts';
import type { FoundShape, } from './claude-schema-blind-spots.test-fixture.ts';
import { mayLeadToBuilder, } from './claude-schema-blind-standing.test-fixture.ts';
import {
  isPureTypeVisit,
  referenceVisitsUnder,
} from './claude-schema-reference.test-fixture.ts';
import {
  holdsFormatValue,
  isFunctionNode,
  type Visit,
} from './claude-schema-tree.test-fixture.ts';

//region Builder uses the schema search cannot follow
// The part of the check behind the search (`claude-schema-blind-spots.test-fixture.ts`) that needs to know which names
// are builders and which modules carry one: every chain of names in the file that stands for either, read where the
// search reads it or reported, and a function that holds a format and that no declared function holds.

/**
 The chains of names a file writes where a value sits, with the node that holds each; the namespaces, enums and
 parameter properties the search leaves out are read.

 @param program - file's program node

 @returns One visit per chain

 @example
 ```ts
 const references = chainVisitsIn({ program, },);
 ```
 */
export function chainVisitsIn({ program, }: { readonly program: TreeNode; },): readonly Visit[] {
  return referenceVisitsUnder({
    root: program,
    skips: isPureTypeVisit,
  },);
}

/**
 The places a function holds a format that no declared function holds.

 @param nodes - every node of the file

 @param declared - nodes that hold the functions the file declares

 @returns One entry per outermost such function

 @example
 ```ts
 const found = undeclaredFormatShapesIn({ nodes, declared, },);
 ```
 */
function undeclaredFormatShapesIn(
  {
    nodes,
    declared,
  }: {
    readonly nodes: readonly TreeNode[];
    readonly declared: readonly TreeNode[];
  },
): readonly FoundShape[] {
  /**
   The functions that hold a format and sit in no declared function.
   */
  const undeclared = nodes.filter(function isUndeclared(node,): boolean {
    return isFunctionNode({ node, },) && holdsFormatValue({ root: node, },)
      && (!declared.some(function holds(holder,): boolean {
        return (holder.start <= node.start) && (node.end <= holder.end);
      },));
  },);
  return undeclared
    .filter(function isOutermost(node,): boolean {
      return !undeclared.some(function holdsNode(other,): boolean {
        return (other !== node) && (other.start <= node.start)
          && (node.end <= other.end);
      },);
    },)
    .map(function shapeOfFunction(node,): FoundShape {
      return {
        shape: 'format-in-undeclared-function',
        node,
        position: node.type,
      };
    },);
}

/**
 The first name of the module reference of an import-equals declaration, `contest` in `import make = contest.build`.

 @param node - module reference, a name or a qualified name, of any kind

 @returns The name, empty for a reference to a module path

 @example
 ```ts
 const head = leftmostNameOf({ node: declaration.moduleReference, },);
 ```
 */
function leftmostNameOf({ node, }: { readonly node: unknown; },): string {
  return ((typeof node) === 'object') && (node !== null)
    && ('left' in node)
    ? leftmostNameOf({ node: node.left, },)
    : identifierName({ node, },);
}

/**
 The import-equals declarations of a file that bring in a builder or a module that carries one by name, which the search
 does not read.

 @param context - what the file's names are read against

 @param nodes - every node of the file

 @returns One entry per declaration

 @example
 ```ts
 const found = importEqualsShapesIn({ context, nodes, },);
 ```
 */
function importEqualsShapesIn(
  {
    context,
    nodes,
  }: {
    readonly context: BlindContext;
    readonly nodes: readonly TreeNode[];
  },
): readonly FoundShape[] {
  return nodes
    .filter(function bringsInByName(node,): boolean {
      return (node.type === 'TSImportEqualsDeclaration')
        && mayLeadToBuilder({
          context,
          name: leftmostNameOf({ node: node.moduleReference, },),
        },);
    },)
    .map(function shapeOfDeclaration(node,): FoundShape {
      return {
        shape: 'builder-in-typescript-value-construct',
        node,
        position: node.type,
      };
    },);
}

/**
 The builder uses of a file that the search does not follow.

 @param context - what the file's names are read against

 @param nodes - every node of the file

 @param declared - nodes that hold the functions the file declares

 @param checksUndeclared - whether to look for a function that holds a format and that no declared function holds

 @returns One entry per place, with the shape it holds

 @example
 ```ts
 const found = builderShapesIn({ context, nodes, declared: [], checksUndeclared: false, },);
 ```
 */
export function builderShapesIn(
  {
    context,
    nodes,
    declared,
    checksUndeclared,
  }: {
    readonly context: BlindContext;
    readonly nodes: readonly TreeNode[];
    readonly declared: readonly TreeNode[];
    readonly checksUndeclared: boolean;
  },
): readonly FoundShape[] {
  return [
    ...context.references
      .flatMap(function shapesOfChain({ node, },): readonly FoundShape[] {
        return chainShapesAt({
          context,
          node,
        },);
      },),
    ...importEqualsShapesIn({
      context,
      nodes,
    },),
    ...(checksUndeclared
      ? undeclaredFormatShapesIn({
        nodes,
        declared,
      },)
      : []),
  ];
}

//endregion Builder uses the schema search cannot follow
