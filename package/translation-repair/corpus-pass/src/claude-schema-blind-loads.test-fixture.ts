import type { TreeNode, } from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import {
  bindsAsReadBy,
  type BlindContext,
  positionOf,
} from './claude-schema-blind-positions.test-fixture.ts';
import type { FoundShape, } from './claude-schema-blind-spots.test-fixture.ts';
import { isTypeWithoutValue, } from './claude-schema-reference.test-fixture.ts';
import { targetOf, } from './claude-schema-target.test-fixture.ts';
import {
  type ModuleLoad,
  moduleLoadOf,
  nodesAt,
  stringValueOf,
} from './claude-schema-tree.test-fixture.ts';

//region Loads and path literals the schema search cannot follow
// The first total of the check behind the schema search: every string that names a module which carries a builder, and
// every `import()` or `require`, must sit where the search follows it. A path literal that is the source of an import
// or of an export from is followed; so is the path of a load that is awaited or required and is held by a constant,
// taken apart by a pattern whose names the search binds, or read off in place. A load whose path is no single literal
// cannot be told from one that reaches a builder, and is reported with the rest.

/**
 The kinds of node that a load can sit inside without changing what it is: parentheses, a type mark, an optional chain
 and `await`.
 */
const LOAD_WRAPPER_KINDS: ReadonlySet<string> = new Set([
  'AwaitExpression',
  'ChainExpression',
  'ParenthesizedExpression',
  'TSAsExpression',
  'TSNonNullExpression',
  'TSSatisfiesExpression',
  'TSTypeAssertion',
],);

/**
 The kinds of declaration whose `source` is a path that the search follows: an import and the two forms of export from.
 */
const SOURCE_DECLARATION_KINDS: ReadonlySet<string> = new Set([
  'ExportAllDeclaration',
  'ExportNamedDeclaration',
  'ImportDeclaration',
],);

/**
 The outermost wrapper around a load, and whether an `await` is among the wrappers.

 @param node - load, or a wrapper around it

 @param parents - each node's parent

 @returns Outermost wrapper, the node itself when it has none, and whether an `await` waits for the load

 @example
 ```ts
 const { top, isAwaited, } = outermostWrapperOf({ node, parents, },);
 ```
 */
function outermostWrapperOf(
  {
    node,
    parents,
  }: {
    readonly node: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
  },
): {
  readonly top: TreeNode;
  readonly isAwaited: boolean;
} {
  /**
   The node that holds this one.
   */
  const parent = parents.get(node,);
  if ((parent === undefined) || (!LOAD_WRAPPER_KINDS.has(parent.type,))) {
    return {
      top: node,
      isAwaited: false,
    };
  }
  /**
   The outermost wrapper around the parent.
   */
  const outer = outermostWrapperOf({
    node: parent,
    parents,
  },);
  return {
    top: outer.top,
    isAwaited: outer.isAwaited || (parent.type === 'AwaitExpression'),
  };
}

/**
 Whether the search follows a load, given where it sits: it yields the module itself, and a constant that takes a name
 or members from it holds it, or a member is read off it.

 @param context - what the file's names are read against

 @param node - load

 @param isSettled - whether the load yields the module itself without `await`

 @returns Whether a binding the search reads or a chain of names carries the module on

 @example
 ```ts
 const followed = isFollowedLoad({ context, node, isSettled: true, },);
 ```
 */
function isFollowedLoad(
  {
    context,
    node,
    isSettled,
  }: {
    readonly context: BlindContext;
    readonly node: TreeNode;
    readonly isSettled: boolean;
  },
): boolean {
  /**
   The outermost wrapper around the load, and whether an `await` waits for it.
   */
  const {
    top,
    isAwaited,
  } = outermostWrapperOf({
    node,
    parents: context.parents,
  },);
  /**
   The node that holds the outermost wrapper.
   */
  const holder = context.parents
    .get(top,);
  if ((!(isSettled || isAwaited)) || (holder === undefined))
    return false;
  if (holder.type === 'MemberExpression')
    return holder.object === top;
  return (holder.type === 'VariableDeclarator') && (holder.init === top)
    && bindsAsReadBy({
      declarator: holder,
      parents: context.parents,
      facts: context.facts,
    },);
}

/**
 Whether a node sits inside a type that holds no value.

 @param node - node read

 @param parents - each node's parent

 @returns Whether the node or some node around it is a type

 @example
 ```ts
 const inType = isInsideType({ node, parents, },);
 ```
 */
function isInsideType(
  {
    node,
    parents,
  }: {
    readonly node: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
  },
): boolean {
  if (isTypeWithoutValue({ node, },))
    return true;
  /**
   The node that holds this one.
   */
  const parent = parents.get(node,);
  return (parent !== undefined) && isInsideType({
    node: parent,
    parents,
  },);
}

/**
 Whether a path literal is the path of a load, which the load's own shape answers.

 @param node - literal read

 @param holder - node that holds the literal

 @returns Whether the holder is an `import()` or a `require` whose path is the literal

 @example
 ```ts
 const isPath = isLoadPath({ node, holder, },);
 ```
 */
function isLoadPath(
  {
    node,
    holder,
  }: {
    readonly node: TreeNode;
    readonly holder: TreeNode;
  },
): boolean {
  return ((holder.type === 'ImportExpression') && (holder.source === node))
    || ((holder.type === 'CallExpression')
      && moduleLoadOf({ node: holder, },)
        .isLoad
      && nodesAt({
        node: holder,
        field: 'arguments',
      },)
        .includes(node,));
}

/**
 The shapes a load holds: a path that is no literal, or a path that names a module carrying a builder in a form the
 search does not follow.

 @param context - what the file's names are read against

 @param node - load

 @param load - what the load is

 @returns Shape, none for a load the search follows or whose path names no module that carries a builder

 @example
 ```ts
 const found = shapesOfLoad({ context, node, load, },);
 ```
 */
function shapesOfLoad(
  {
    context,
    node,
    load,
  }: {
    readonly context: BlindContext;
    readonly node: TreeNode;
    readonly load: ModuleLoad;
  },
): readonly FoundShape[] {
  if (load.specifier === '') {
    return [
      {
        shape: 'computed-load-path',
        node,
        position: 'path is not one literal',
      },
    ];
  }
  if (!context.bearing
    .has(targetOf({
      family: context.family,
      from: context.file,
      specifier: load.specifier,
    },),)) {
    return [];
  }
  /**
   The outermost wrapper around the load.
   */
  const { top, } = outermostWrapperOf({
    node,
    parents: context.parents,
  },);
  return isFollowedLoad({
    context,
    node,
    isSettled: load.isSettled,
  },)
    ? []
    : [
      {
        shape: 'load-in-unfollowed-form',
        node,
        position: positionOf({
          holder: context.parents
            .get(top,)
            ?? top,
          child: top,
        },),
      },
    ];
}

/**
 Whether a string or a template with no substitution names a module that carries a builder.

 @param context - what the file's names are read against

 @param node - node read, of any kind

 @returns Whether the node is a string whose text, read from this file, names such a module

 @example
 ```ts
 const names = namesBearingModule({ context, node, },);
 ```
 */
export function namesBearingModule(
  {
    context,
    node,
  }: {
    readonly context: Pick<BlindContext, 'bearing' | 'family' | 'file'>;
    readonly node: TreeNode;
  },
): boolean {
  /**
   The text of the literal, empty for any node that is no string.
   */
  const text = stringValueOf({ node, },);
  return (text !== '')
    && context.bearing
      .has(targetOf({
        family: context.family,
        from: context.file,
        specifier: text,
      },),);
}

/**
 The shape a path literal holds when it names a module that carries a builder and sits outside an import, an export from
 and a load.

 @param context - what the file's names are read against

 @param node - string or template read

 @returns Shape, none for any other node

 @example
 ```ts
 const found = shapesOfPathLiteral({ context, node, },);
 ```
 */
function shapesOfPathLiteral(
  {
    context,
    node,
  }: {
    readonly context: BlindContext;
    readonly node: TreeNode;
  },
): readonly FoundShape[] {
  /**
   The node that holds the literal.
   */
  const holder = context.parents
    .get(node,);
  if ((holder === undefined) || (!namesBearingModule({
    context,
    node,
  },)))
    return [];
  return ((SOURCE_DECLARATION_KINDS.has(holder.type,) && (holder.source === node))
    || isLoadPath({
      node,
      holder,
    },)
    || isInsideType({
      node,
      parents: context.parents,
    },))
    ? []
    : [
      {
        shape: 'module-path-outside-a-load',
        node,
        position: positionOf({
          holder,
          child: node,
        },),
      },
    ];
}

/**
 The loads and path literals of a file that the search does not follow.

 @param context - what the file's names are read against

 @param nodes - every node of the file

 @returns One entry per place, with the shape it holds

 @example
 ```ts
 const found = loadShapesIn({ context, nodes: nodesUnder({ root: program, },), },);
 ```
 */
export function loadShapesIn(
  {
    context,
    nodes,
  }: {
    readonly context: BlindContext;
    readonly nodes: readonly TreeNode[];
  },
): readonly FoundShape[] {
  return nodes.flatMap(function shapesOfNode(node,): readonly FoundShape[] {
    if ((node.type === 'Literal') || (node.type === 'TemplateLiteral')) {
      return shapesOfPathLiteral({
        context,
        node,
      },);
    }
    /**
     The load the node is, none for a node that is no load.
     */
    const load = ((node.type === 'ImportExpression') || (node.type === 'CallExpression'))
      ? moduleLoadOf({ node, },)
      : undefined;
    return (load?.isLoad === true)
      ? shapesOfLoad({
        context,
        node,
        load,
      },)
      : [];
  },);
}

//endregion Loads and path literals the schema search cannot follow
