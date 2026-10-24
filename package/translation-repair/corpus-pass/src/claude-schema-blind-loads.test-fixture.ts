import {
  parentsOf,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import type { FoundShape, } from './claude-schema-blind-spots.test-fixture.ts';
import {
  type ModuleLoad,
  moduleLoadOf,
} from './claude-schema-tree.test-fixture.ts';

//region Loads the schema search cannot follow
// The two shapes of a dynamic `import()` or a `require` that the search leaves unread: a path that is no literal, and a
// load that is not in one of the two forms a binding follows (a settled load held by a constant, and a settled load a
// member is read off in place).

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
 The outermost wrapper around a load, and whether an `await` is among the wrappers.

 @param node - the load, or a wrapper around it

 @param parents - each node's parent

 @returns The outermost wrapper, the node itself when it has none, and whether an `await` waits for the load

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
 Whether the search follows a load, given where it sits: it yields the module itself, and a constant that takes a name or
 members from it holds it, or a member is read off it.

 @param node - the load

 @param parents - each node's parent

 @param isSettled - whether the load yields the module itself without `await`

 @returns Whether a binding or a chain of names carries the module on

 @example
 ```ts
 const followed = isFollowedLoad({ node, parents, isSettled: true, },);
 ```
 */
function isFollowedLoad(
  {
    node,
    parents,
    isSettled,
  }: {
    readonly node: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
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
    parents,
  },);
  /**
   The node that holds the outermost wrapper.
   */
  const holder = parents.get(top,);
  if ((!(isSettled || isAwaited)) || (holder === undefined))
    return false;
  if (holder.type === 'MemberExpression')
    return holder.object === top;
  /**
   What the constant takes the load into.
   */
  const target = (holder.type === 'VariableDeclarator') && (holder.init === top) ? holder.id : undefined;
  return ((typeof target) === 'object') && (target !== null)
    && ('type' in target)
    && ((target.type === 'Identifier') || (target.type === 'ObjectPattern'));
}

/**
 The loads of a file that the search does not follow.

 @param program - the file's program node

 @param nodes - every node of the file

 @returns One entry per load, with the shape it holds

 @example
 ```ts
 const found = loadShapesIn({ program, nodes: nodesUnder({ root: program, },), },);
 ```
 */
export function loadShapesIn(
  {
    program,
    nodes,
  }: {
    readonly program: TreeNode;
    readonly nodes: readonly TreeNode[];
  },
): readonly FoundShape[] {
  /**
   The loads of the file, with what each is.
   */
  const loads = nodes.flatMap(function loadOf(node,): readonly {
    readonly node: TreeNode;
    readonly load: ModuleLoad;
  }[] {
    /**
     The load the node is, none for a call that is no `require`.
     */
    const load = ((node.type === 'ImportExpression') || (node.type === 'CallExpression'))
      ? moduleLoadOf({ node, },)
      : undefined;
    return (load?.isLoad === true) ? [{
      node,
      load,
    },] : [];
  },);
  if (loads.length === 0)
    return [];
  /**
   Each node's parent.
   */
  const parents = parentsOf({ program, },);
  return loads.flatMap(function shapeOfLoad({
    node,
    load,
  },): readonly FoundShape[] {
    if (load.specifier === '') {
      return [
        {
          shape: 'computed-load-path',
          node,
        },
      ];
    }
    return isFollowedLoad({
      node,
      parents,
      isSettled: load.isSettled,
    },)
      ? []
      : [
        {
          shape: 'load-in-unfollowed-form',
          node,
        },
      ];
  },);
}

//endregion Loads the schema search cannot follow
