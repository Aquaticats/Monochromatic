import {
  identifierName,
  isTreeNode,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import type { FoundShape, } from './claude-schema-blind-spots.test-fixture.ts';
import { typeScriptValueReferencesIn, } from './claude-schema-reference.test-fixture.ts';
import type { Family, } from './claude-schema-resolve.test-fixture.ts';
import {
  holdsFormatValue,
  isFunctionNode,
  isFunctionVisit,
  nodeAt,
  nodesAt,
  pathOf,
  visitsUnder,
  withoutAwait,
} from './claude-schema-tree.test-fixture.ts';
import {
  type KnownFunctions,
  standsForKnown,
} from './claude-schema-usage.test-fixture.ts';

//region Builder uses the schema search cannot follow
// The shapes of the check behind the search (`claude-schema-blind-spots.test-fixture.ts`) that need to know which names
// are builders: a builder stored as the value of an object, a builder handed to a call that a constant holds, a
// builder's result returned through a local, a builder called inside a namespace, an enum or a parameter property, and
// a function that holds a format and that no declared function holds.

/**
 What a file's names are read against: the modules of the sources, the file, and the builders found.
 */
type BuilderContext = {
  /**
   The modules of the sources.
   */
  readonly family: Family;

  /**
   Path of the file read.
   */
  readonly file: string;

  /**
   The builders and wrappers of the sources.
   */
  readonly builders: KnownFunctions;
};

/**
 Whether an expression is a name or a chain of names that stands for a builder.

 @param context - what the names are read against

 @param node - expression read, of any kind or none

 @returns Whether it stands for a builder or a wrapper

 @example
 ```ts
 const isBuilder = isBuilderExpression({ context, node: property.value, },);
 ```
 */
function isBuilderExpression(
  {
    context,
    node,
  }: {
    readonly context: BuilderContext;
    readonly node: unknown;
  },
): boolean {
  /**
   The names the expression is written with.
   */
  const path = pathOf({ node, },);
  return (path.length > 0) && standsForKnown({
    family: context.family,
    file: context.file,
    path,
    known: context.builders,
  },);
}

/**
 Whether an expression is the call of a builder, once `await` and type marks are taken off.

 @param context - what the names are read against

 @param node - expression read, of any kind or none

 @returns Whether it is a call whose callee stands for a builder or a wrapper

 @example
 ```ts
 const isCall = isBuilderCall({ context, node: declarator.init, },);
 ```
 */
function isBuilderCall(
  {
    context,
    node,
  }: {
    readonly context: BuilderContext;
    readonly node: unknown;
  },
): boolean {
  /**
   The expression the call sits in.
   */
  const call = isTreeNode(node,) ? withoutAwait({ node, },) : undefined;
  return isTreeNode(call,) && (call.type === 'CallExpression')
    && isBuilderExpression({
      context,
      node: call.callee,
    },);
}

/**
 The nodes that store a builder as the value of an object, or hand it to a call that a constant holds.

 @param context - what the names are read against

 @param nodes - every node of the file

 @returns One entry per place

 @example
 ```ts
 const found = storedBuilderShapesIn({ context, nodes, },);
 ```
 */
function storedBuilderShapesIn(
  {
    context,
    nodes,
  }: {
    readonly context: BuilderContext;
    readonly nodes: readonly TreeNode[];
  },
): readonly FoundShape[] {
  /**
   The properties of patterns that take a value apart, which name where a member goes and store nothing.
   */
  const patternProperties = new Set(nodes
    .filter(function isPattern(node,): boolean {
      return node.type === 'ObjectPattern';
    },)
    .flatMap(function propertiesOf(pattern,): readonly TreeNode[] {
      return nodesAt({
        node: pattern,
        field: 'properties',
      },);
    },),);
  return nodes.flatMap(function shapeOfStore(node,): readonly FoundShape[] {
    if ((node.type === 'Property') && (!patternProperties.has(node,))
      && (isBuilderExpression({
        context,
        node: node.value,
      },))
      && (node.kind === 'init')) {
      return [
        {
          shape: 'builder-stored-in-object',
          node,
        },
      ];
    }
    if (node.type !== 'VariableDeclarator')
      return [];
    /**
     The call the constant holds, once `await` and type marks are taken off.
     */
    const call = withoutAwait({ node: nodeAt({
      node,
      field: 'init',
    },), },);
    return (isTreeNode(call,)
      && (call.type === 'CallExpression')
      && nodesAt({
        node: call,
        field: 'arguments',
      },)
        .some(function isBuilderArgument(argument,): boolean {
          return isBuilderExpression({
            context,
            node: argument,
          },);
        },))
      ? [
        {
          shape: 'builder-handed-to-a-call-held-by-a-constant',
          node,
        },
      ]
      : [];
  },);
}

/**
 The returns of a function that give back a local constant that holds a builder's result.

 @param context - what the names are read against

 @param fn - function read

 @returns One entry per such `return`

 @example
 ```ts
 const found = returnedLocalShapesIn({ context, fn, },);
 ```
 */
function returnedLocalShapesIn(
  {
    context,
    fn,
  }: {
    readonly context: BuilderContext;
    readonly fn: TreeNode;
  },
): readonly FoundShape[] {
  /**
   The statements of the function's own body, not those of a function inside it.
   */
  const own = visitsUnder({
    root: nodeAt({
      node: fn,
      field: 'body',
    },),
    skips: isFunctionVisit,
  },)
    .map(function nodeOf({ node, },): TreeNode {
      return node;
    },);
  /**
   Names of the local constants that hold a builder's result.
   */
  const locals = new Set(own.flatMap(function nameOfLocal(node,): readonly string[] {
    return ((node.type === 'VariableDeclarator') && isBuilderCall({
      context,
      node: node.init,
    },)
      && (identifierName({ node: node.id, },) !== ''))
      ? [identifierName({ node: node.id, },),]
      : [];
  },),);
  return own.flatMap(function shapeOfReturn(node,): readonly FoundShape[] {
    return ((node.type === 'ReturnStatement') && locals.has(identifierName({ node: withoutAwait({ node: nodeAt({
      node,
      field: 'argument',
    },), }), },),))
      ? [
        {
          shape: 'result-returned-through-a-local',
          node,
        },
      ]
      : [];
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
      };
    },);
}

/**
 The builder uses of a file that the search does not follow.

 @param context - what the names are read against

 @param nodes - every node of the file

 @param refersToBuilder - whether the file refers to a builder anywhere in code, which the places that store a builder,
 hand it to a call or return its result through a local all need

 @param declared - nodes that hold the functions the file declares

 @param checksUndeclared - whether to look for a function that holds a format and that no declared function holds

 @returns One entry per place, with the shape it holds

 @example
 ```ts
 const found = builderShapesIn({ context, nodes, refersToBuilder: true, declared: [], checksUndeclared: false, },);
 ```
 */
export function builderShapesIn(
  {
    context,
    nodes,
    refersToBuilder,
    declared,
    checksUndeclared,
  }: {
    readonly context: BuilderContext;
    readonly nodes: readonly TreeNode[];
    readonly refersToBuilder: boolean;
    readonly declared: readonly TreeNode[];
    readonly checksUndeclared: boolean;
  },
): readonly FoundShape[] {
  return [
    ...(refersToBuilder
      ? [
        ...storedBuilderShapesIn({
          context,
          nodes,
        },),
        ...nodes.filter(function isFunction(node,): boolean {
          return isFunctionNode({ node, },);
        },)
          .flatMap(function shapesOfFunction(fn,): readonly FoundShape[] {
            return returnedLocalShapesIn({
              context,
              fn,
            },);
          },),
      ]
      : []),
    ...nodes.filter(function holdsValues(node,): boolean {
      return (node.type === 'TSModuleDeclaration') || (node.type === 'TSEnumDeclaration')
        || (node.type === 'TSParameterProperty');
    },)
      .filter(function readsBuilder(node,): boolean {
        return typeScriptValueReferencesIn({ root: node, },)
          .some(function isBuilderPath(path,): boolean {
            return standsForKnown({
              family: context.family,
              file: context.file,
              path,
              known: context.builders,
            },);
          },);
      },)
      .map(function shapeOfConstruct(node,): FoundShape {
        return {
          shape: 'builder-in-namespace-enum-or-parameter-property',
          node,
        };
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
