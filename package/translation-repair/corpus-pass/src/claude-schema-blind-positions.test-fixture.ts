import {
  identifierName,
  isTreeNode,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import { gatherDeclaration, } from './claude-schema-declare.test-fixture.ts';
import type { ModuleFacts, } from './claude-schema-facts.test-fixture.ts';
import type { Family, } from './claude-schema-resolve.test-fixture.ts';
import {
  isFunctionNode,
  NO_NODE,
  nodesAt,
  type Visit,
} from './claude-schema-tree.test-fixture.ts';
import type { KnownFunctions, } from './claude-schema-usage.test-fixture.ts';

//region Where a node sits in a file
// What the check behind the schema search (`claude-schema-blind-spots.test-fixture.ts`) asks of the tree around a
// node: what a file's names are read against, the node that holds another and the field it sits in, the wrappers around
// an expression that leave it the same value, the function around a node, and whether a declaration binds its names as
// the search reads it.

/**
 What a file's names are read against: the modules of the sources, the file, the builders and the modules that carry
 one.
 */
export type BlindContext = {
  /**
   The modules of the sources.
   */
  readonly family: Family;

  /**
   Path of the file read.
   */
  readonly file: string;

  /**
   What the file declares, binds and refers to.
   */
  readonly facts: ModuleFacts;

  /**
   The builders and wrappers of the sources.
   */
  readonly builders: KnownFunctions;

  /**
   Paths of the modules that carry a builder.
   */
  readonly bearing: ReadonlySet<string>;

  /**
   Keys of the objects that hold a builder as a member.
   */
  readonly carriers: ReadonlySet<string>;

  /**
   The functions of the file that the search knows as builders or wrappers.
   */
  readonly recognized: ReadonlySet<TreeNode>;

  /**
   Each node's parent.
   */
  readonly parents: ReadonlyMap<TreeNode, TreeNode>;

  /**
   The chains of names the file's code writes where a value sits, with the node that holds each.
   */
  readonly references: readonly Visit[];
};

/**
 The kinds of node that put a mark around an expression and leave the expression the value it was.
 */
const TRANSPARENT_KINDS: ReadonlySet<string> = new Set([
  'ChainExpression',
  'ParenthesizedExpression',
  'TSAsExpression',
  'TSInstantiationExpression',
  'TSNonNullExpression',
  'TSSatisfiesExpression',
  'TSTypeAssertion',
],);

/**
 The kinds of node that hold values the search does not read: a namespace, an enum, a parameter property, an export
 assignment and an import-equals declaration.
 */
const UNREAD_CONSTRUCT_KINDS: ReadonlySet<string> = new Set([
  'TSEnumDeclaration',
  'TSExportAssignment',
  'TSImportEqualsDeclaration',
  'TSModuleDeclaration',
  'TSParameterProperty',
],);

/**
 Where a node sits in the node that holds it, as the holder's kind and the field.

 @param holder - node that holds the other

 @param child - node held

 @returns `Kind.field`, the kind alone when no field holds the node

 @example
 ```ts
 const position = positionOf({ holder: property, child: value, });
 // => 'Property.value'
 ```
 */
export function positionOf(
  {
    holder,
    child,
  }: {
    readonly holder: TreeNode;
    readonly child: TreeNode;
  },
): string {
  /**
   The field of the holder that holds the node, absent when none does.
   */
  const entry = Object.entries(holder,)
    .find(function holdsChild([, value,],): boolean {
      return (value === child) || (Array.isArray(value,) && value.includes(child,));
    },);
  return (entry === undefined) ? holder.type : `${holder.type}.${entry[0]}`;
}

/**
 The expression under the marks that leave it the same value: parentheses, a type mark and an optional chain.

 @param node - expression read

 @param parents - each node's parent

 @returns Outermost node that stands for the same value

 @example
 ```ts
 const top = outermostValueOf({ node: callee, parents, });
 ```
 */
export function outermostValueOf(
  {
    node,
    parents,
  }: {
    readonly node: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
  },
): TreeNode {
  /**
   The node that holds this one.
   */
  const parent = parents.get(node,);
  return ((parent !== undefined) && TRANSPARENT_KINDS.has(parent.type,)
    && (parent.expression === node))
    ? outermostValueOf({
      node: parent,
      parents,
    },)
    : node;
}

/**
 The function that holds a node, the node itself when it is one.

 @param node - node read

 @param parents - each node's parent

 @returns Nearest function, `NO_NODE` at the top level

 @example
 ```ts
 const fn = enclosingFunctionOf({ node, parents, });
 ```
 */
export function enclosingFunctionOf(
  {
    node,
    parents,
  }: {
    readonly node: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
  },
): TreeNode {
  if (isFunctionNode({ node, },))
    return node;
  /**
   The node that holds this one.
   */
  const parent = parents.get(node,);
  return (parent === undefined)
    ? NO_NODE
    : enclosingFunctionOf({
      node: parent,
      parents,
    },);
}

/**
 The kind of a namespace, an enum, a parameter property, an export assignment or an import-equals declaration that holds
 a node.

 @param node - node read

 @param parents - each node's parent

 @returns Kind of the nearest such node, empty when none holds the node

 @example
 ```ts
 const construct = unreadConstructAround({ node, parents, });
 ```
 */
export function unreadConstructAround(
  {
    node,
    parents,
  }: {
    readonly node: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
  },
): string {
  if (UNREAD_CONSTRUCT_KINDS.has(node.type,))
    return node.type;
  /**
   The node that holds this one.
   */
  const parent = parents.get(node,);
  return (parent === undefined)
    ? ''
    : unreadConstructAround({
      node: parent,
      parents,
    },);
}

/**
 Every name a pattern or a plain name binds, the targets inside nested patterns, rests and defaults included.

 @param node - id of a declaration, of any kind or none

 @returns Names in order

 @example
 ```ts
 const names = namesBoundBy({ node: declarator.id, });
 ```
 */
export function namesBoundBy({ node, }: { readonly node: unknown; },): readonly string[] {
  if (!isTreeNode(node,))
    return [];
  if (node.type === 'Identifier')
    return [identifierName({ node, },),];
  if (node.type === 'ObjectPattern') {
    return nodesAt({
      node,
      field: 'properties',
    },)
      .flatMap(function boundByProperty(property,): readonly string[] {
        return namesBoundBy({ node: (property.type === 'Property') ? property.value : property, },);
      },);
  }
  if (node.type === 'ArrayPattern') {
    return nodesAt({
      node,
      field: 'elements',
    },)
      .flatMap(function boundByElement(element,): readonly string[] {
        return namesBoundBy({ node: element, },);
      },);
  }
  if (node.type === 'RestElement')
    return namesBoundBy({ node: node.argument, },);
  return (node.type === 'AssignmentPattern') ? namesBoundBy({ node: node.left, },) : [];
}

/**
 Whether a declarator binds every name it names to what the search reads it as: the binding the file records for each
 name is the one the declaration makes, so a name bound twice, a rest, a computed key, a nested pattern and a value the
 search does not follow all fail.

 @param declarator - declarator read

 @param parents - each node's parent

 @param facts - what the file declares, binds and refers to

 @returns Whether each name the declarator binds has the binding the declaration makes

 @example
 ```ts
 const binds = bindsAsReadBy({ declarator, parents, facts, });
 ```
 */
export function bindsAsReadBy(
  {
    declarator,
    parents,
    facts,
  }: {
    readonly declarator: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
    readonly facts: ModuleFacts;
  },
): boolean {
  /**
   The declaration that holds the declarator.
   */
  const declaration = parents.get(declarator,);
  if (declaration === undefined)
    return false;
  /**
   What the declaration binds, by name.
   */
  const made = new Map(gatherDeclaration({ declaration, },)
    .bindings,);
  /**
   The names the declarator binds.
   */
  const names = namesBoundBy({ node: declarator.id, },);
  return (names.length > 0) && names.every(function isBoundAsMade(name,): boolean {
    return (made.has(name,))
      && (JSON.stringify(made.get(name,),) === JSON.stringify(facts.bindings
        .get(name,),));
  },);
}

//endregion Where a node sits in a file
