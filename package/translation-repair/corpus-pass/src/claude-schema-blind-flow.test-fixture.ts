import {
  identifierName,
  isTreeNode,
  type TreeNode,
  unwrapped,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import { isSenderProperty, } from './claude-schema-blind-senders.test-fixture.ts';
import {
  type BlindContext,
  enclosingFunctionOf,
  namesBoundBy,
  outermostValueOf,
  positionOf,
} from './claude-schema-blind-positions.test-fixture.ts';
import {
  isNoNode,
  NO_NODE,
  nodesAt,
} from './claude-schema-tree.test-fixture.ts';

//region Where the result of a builder's call goes
// What the check behind the schema search (`claude-schema-blind-spots.test-fixture.ts`) asks of a builder's call: where
// its value goes. The value is followed through the objects, arrays, spreads, conditionals and logical operands it is
// put into, through the members read off it and the methods called on it, and through the constants and the parameters
// with a default that a function or the file binds it to. The walk ends in one of these places, and every other place
// is reported:
//
// - a `return`, an arrow function's body or a `yield`, in a function the search knows as a wrapper (in any other
//   function the value leaves it, and the function is reported);
// - the property of a sender's object argument, which `claude-schema-blind-senders.test-fixture.ts` lists by module,
//   function and parameter;
// - a constant, a pattern, a default or a name assigned, whose every reference is followed in turn.
//
// A place that takes the value and gives nothing the schema could travel in ends the walk without a report: the test
// of a conditional or a loop, a computed key, an operand of an operator, a substitution in a template, a statement
// that drops it, an element of a sequence that is not the last, and the target of an assignment. Any other place is
// reported with its position: an argument of a call or of `new`, a store into a member or a field, a pattern an
// assignment takes the value apart into, a class that extends it, a thrown value, a loop that iterates it.

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
 The kinds of node that read the value they hold and give back nothing the schema could travel in: a test, an operand of
 an operator, a substitution in a template, a statement that drops it.
 */
const DROPPING_KINDS: ReadonlySet<string> = new Set([
  'BinaryExpression',
  'DoWhileStatement',
  'ExpressionStatement',
  'IfStatement',
  'SwitchCase',
  'SwitchStatement',
  'TemplateLiteral',
  'UnaryExpression',
  'UpdateExpression',
  'WhileStatement',
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
 One step of the walk.
 */
type Step = {
  /**
   The nodes that hold the value after this step.
   */
  readonly onward: readonly TreeNode[];

  /**
   The names the value is bound to.
   */
  readonly names: readonly string[];

  /**
   Whether the names are read anywhere in the file.
   */
  readonly isFileWide: boolean;

  /**
   The function the value leaves by, `NO_NODE` when it does not.
   */
  readonly leaves: TreeNode;

  /**
   Where the value goes when the place is reported, empty when it is not.
   */
  readonly stray: string;
};

/**
 The node a value is read on from, and the names it is bound to, one step on.

 @param context - what the file's names are read against

 @param node - value

 @returns Nodes that hold the value after this step, the names it is bound to and whether they are read anywhere in the
 file, the function it leaves by when it leaves, `NO_NODE` when it does not, and the place it goes when that place is
 reported

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
): Step {
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
    stray: '',
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
    /**
     The object literal that holds the property.
     */
    const object = context.parents
      .get(holder,)
      ?? holder;
    return {
      ...nothing,
      onward: ((holder.value === top) && (!isSenderProperty({
        context,
        property: holder,
        object,
      },)))
        ? [object,]
        : [],
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
  if ((holder.type === 'CallExpression') && (holder.callee === top)) {
    return {
      ...nothing,
      onward: [holder,],
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
  if (holder.type === 'AssignmentExpression') {
    /**
     The name the assignment stores the value in, empty when it stores it in a member or a pattern.
     */
    const stored = identifierName({ node: holder.left, },);
    if (holder.right !== top)
      return nothing;
    return (stored === '')
      ? {
        ...nothing,
        stray: positionOf({
          holder,
          child: top,
        },),
      }
      : {
        ...nothing,
        names: [stored,],
        isFileWide: true,
      };
  }
  if (holder.type === 'AssignmentPattern') {
    return {
      ...nothing,
      names: (holder.right === top) ? namesBoundBy({ node: holder.left, },) : [],
    };
  }
  return DROPPING_KINDS.has(holder.type,)
    ? nothing
    : {
      ...nothing,
      stray: positionOf({
        holder,
        child: top,
      },),
    };
}

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

 @param scope - function that holds the binding

 @returns Identifiers that read the name

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
    .map(function headOfVisit({ node, },): TreeNode {
      return headIdentifierOf({ node, },);
    },)
    .filter(function readsName(head,): boolean {
      return (!isNoNode({ node: head, },)) && (identifierName({ node: head, },) === name)
        && (scope.start <= head.start)
        && (head.end <= scope.end);
    },);
}

/**
 Where the value of a builder's call goes.
 */
export type Destinations = {
  /**
   The first function the value is returned from that the search does not know as a wrapper, `NO_NODE` when the value
   leaves by no such function.
   */
  readonly leaving: TreeNode;

  /**
   The places the value goes that the check reports, as the kind of node and the field, each once.
   */
  readonly strays: readonly string[];
};

/**
 Where a builder's call sends its value: the function it is returned from when the search does not know that function
 as a builder, and every place that takes it that no sender, wrapper or constant answers for.

 @param context - what the file's names are read against

 @param call - call of a builder, or the node that holds its value

 @returns First such function found, `NO_NODE` when the value stays in the function, and the places reported

 @example
 ```ts
 const { leaving, strays, } = destinationsOf({ context, call, });
 ```
 */
export function destinationsOf(
  {
    context,
    call,
  }: {
    readonly context: BlindContext;
    readonly call: TreeNode;
  },
): Destinations {
  /**
   Names already followed, so a name that leads back to itself ends.
   */
  const followed = new Set<string>();
  /**
   The places reported so far.
   */
  const strays = new Set<string>();
  /**
   Nodes whose value is still to follow.
   */
  const pending: TreeNode[] = [call,];
  /**
   The functions left by that the search does not know, in the order they were met.
   */
  const leavers: TreeNode[] = [];
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    /**
     Where the value goes next.
     */
    const {
      onward,
      names,
      isFileWide,
      leaves,
      stray,
    } = stepOf({
      context,
      node,
    },);
    if ((!isNoNode({ node: leaves, },)) && (!context.recognized
      .has(leaves,)))
      leavers.push(leaves,);
    if (stray !== '')
      strays.add(stray,);
    pending.push(...onward,);
    /**
     The function that binds the names, `NO_NODE` at the top level.
     */
    const binding = enclosingFunctionOf({
      node,
      parents: context.parents,
    },);
    /**
     Where the names are read: the function that binds them, the whole file at the top level and for a name that an
     assignment stores the value in.
     */
    const scope = (isFileWide || isNoNode({ node: binding, },))
      ? context.facts
        .program
      : binding;
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
  return {
    leaving: leavers[0] ?? NO_NODE,
    strays: [...strays,],
  };
}

//endregion Where the result of a builder's call goes
