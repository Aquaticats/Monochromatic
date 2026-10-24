import {
  identifierName,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import {
  type BlindContext,
  enclosingFunctionOf,
  namesBoundBy,
  outermostValueOf,
} from './claude-schema-blind-positions.test-fixture.ts';
import {
  isNoNode,
  NO_NODE,
  nodesAt,
} from './claude-schema-tree.test-fixture.ts';

//region Where the result of a builder's call goes
// What the check behind the schema search (`claude-schema-blind-spots.test-fixture.ts`) asks of a builder's call:
// whether its value leaves the function that holds it through a `return`, an arrow function's body or a `yield`, in a
// function the search does not know as a builder. The value is followed through the objects, arrays, spreads,
// conditionals and logical operands it is put into, through the members read off it and the methods called on it, and
// through the constants and the parameters with a default that a function binds it to. A call that takes it as an
// argument ends the walk, since it is taken as the place the schema is sent; a statement, a template and an operator
// end it too.

/**
 The kinds of node that put the value they hold into themselves, so the value is read on from the node that holds them.
 */
const CONTAINER_KINDS: ReadonlySet<string> = new Set([
  'ArrayExpression',
  'AwaitExpression',
  'LogicalExpression',
  'ObjectExpression',
  'SpreadElement',
],);

/**
 Whether a node is the last of a sequence, or any other kind of node.

 @param holder - node that holds the value

 @param node - value

 @returns Whether the value is the one the sequence gives

 @example
 ```ts
 const gives = isGivenBy({ holder, node, });
 ```
 */
function isGivenBy(
  {
    holder,
    node,
  }: {
    readonly holder: TreeNode;
    readonly node: TreeNode;
  },
): boolean {
  return (holder.type !== 'SequenceExpression') || (nodesAt({
    node: holder,
    field: 'expressions',
  },)
    .at(-1,)
    === node);
}

/**
 The node a value is read on from, and the names it is bound to, one step on.

 @param context - what the file's names are read against

 @param node - value

 @returns The nodes that hold the value after this step, the names it is bound to and whether they are read anywhere in
 the file, and the function it leaves by when it leaves, `NO_NODE` when it does not

 @example
 ```ts
 const step = stepOf({ context, node, });
 ```
 */
function stepOf(
  {
    context,
    node,
  }: {
    readonly context: BlindContext;
    readonly node: TreeNode;
  },
): {
  readonly onward: readonly TreeNode[];
  readonly names: readonly string[];
  readonly isFileWide: boolean;
  readonly leaves: TreeNode;
} {
  /**
   The value under the marks that leave it the same.
   */
  const top = outermostValueOf({
    node,
    parents: context.parents,
  },);
  /**
   The node that holds the value.
   */
  const holder = context.parents
    .get(top,);
  /**
   A step that takes the value nowhere.
   */
  const nothing = {
    onward: [],
    names: [],
    isFileWide: false,
    leaves: NO_NODE,
  };
  if (holder === undefined)
    return nothing;
  if ((holder.type === 'ReturnStatement') || (holder.type === 'YieldExpression')) {
    return {
      ...nothing,
      leaves: enclosingFunctionOf({
        node: holder,
        parents: context.parents,
      },),
    };
  }
  if (holder.type === 'ArrowFunctionExpression') {
    return {
      ...nothing,
      leaves: (holder.body === top) ? holder : NO_NODE,
    };
  }
  if (holder.type === 'Property') {
    return {
      ...nothing,
      onward: (holder.value === top) ? [context.parents
        .get(holder,)
        ?? holder,] : [],
    };
  }
  if (holder.type === 'ConditionalExpression') {
    return {
      ...nothing,
      onward: (holder.test === top) ? [] : [holder,],
    };
  }
  if (holder.type === 'MemberExpression') {
    return {
      ...nothing,
      onward: (holder.object === top) ? [holder,] : [],
    };
  }
  if (holder.type === 'CallExpression') {
    return {
      ...nothing,
      onward: (holder.callee === top) ? [holder,] : [],
    };
  }
  if (CONTAINER_KINDS.has(holder.type,) || (holder.type === 'SequenceExpression')) {
    return {
      ...nothing,
      onward: isGivenBy({
        holder,
        node: top,
      },) ? [holder,] : [],
    };
  }
  if ((holder.type === 'VariableDeclarator') && (holder.init === top)) {
    return {
      ...nothing,
      names: namesBoundBy({ node: holder.id, },),
    };
  }
  if ((holder.type === 'AssignmentExpression') && (holder.right === top)) {
    /**
     The name the assignment stores the value in, empty when it stores it in a member or a pattern.
     */
    const stored = identifierName({ node: holder.left, },);
    if (stored !== '') {
      return {
        ...nothing,
        names: [stored,],
        isFileWide: true,
      };
    }
  }
  return ((holder.type === 'AssignmentPattern') && (holder.right === top))
    ? {
      ...nothing,
      names: namesBoundBy({ node: holder.left, },),
    }
    : nothing;
}

/**
 The references to a name inside a function, other than the place that binds it.

 @param context - what the file's names are read against

 @param name - name bound

 @param scope - function that holds the binding

 @returns The nodes that read the name

 @example
 ```ts
 const reads = readsOfName({ context, name: 'format', scope, });
 ```
 */
function readsOfName(
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
    .map(function nodeOfVisit({ node, },): TreeNode {
      return node;
    },)
    .filter(function readsName(node,): boolean {
      return (node.type === 'Identifier') && (identifierName({ node, },) === name)
        && (scope.start <= node.start)
        && (node.end <= scope.end);
    },);
}

/**
 The function a builder's call returns its value from, when the search does not know that function as a builder.

 @param context - what the file's names are read against

 @param call - call of a builder, or the node that holds its value

 @returns The first such function found, `NO_NODE` when the value stays in the function or goes into a call

 @example
 ```ts
 const leaving = leavingFunctionOf({ context, call, });
 ```
 */
export function leavingFunctionOf(
  {
    context,
    call,
  }: {
    readonly context: BlindContext;
    readonly call: TreeNode;
  },
): TreeNode {
  /**
   Names already followed, so a name that leads back to itself ends.
   */
  const followed = new Set<string>();
  /**
   Nodes whose value is still to follow.
   */
  const pending: TreeNode[] = [call,];
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    /**
     Where the value goes next.
     */
    const {
      onward,
      names,
      isFileWide,
      leaves,
    } = stepOf({
      context,
      node,
    },);
    if ((!isNoNode({ node: leaves, },)) && (!context.recognized
      .has(leaves,)))
      return leaves;
    pending.push(...onward,);
    /**
     Where the names are read: the function that binds them, absent at the top level, where a constant is the place a
     schema is declared; the whole file for a name that an assignment stores the value in.
     */
    const scope = isFileWide
      ? context.facts
        .program
      : enclosingFunctionOf({
        node,
        parents: context.parents,
      },);
    if (!isNoNode({ node: scope, },)) {
      for (const name of names) {
        if (!followed.has(`${String(scope.start,)}:${name}`,)) {
          followed.add(`${String(scope.start,)}:${name}`,);
          pending.push(...readsOfName({
            context,
            name,
            scope,
          },),);
        }
      }
    }
  }
  return NO_NODE;
}

//endregion Where the result of a builder's call goes
