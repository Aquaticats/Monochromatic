import {
  identifierName,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import {
  type Callback,
  callbackOf,
  NO_CALLBACK,
} from './claude-schema-blind-callbacks.test-fixture.ts';
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

//region One step of the walk of a builder's value
// What `claude-schema-blind-flow.test-fixture.ts` asks of one node that holds the value: the nodes that hold it next,
// the names it is bound to, the function it leaves by, the place it goes when that place is reported, and the callback a
// method called on it hands it to. The places are told apart by the node that holds the value and the field it sits in.

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
 an operator, a substitution in a plain template (a tagged one is told apart before this set is asked), a statement
 that drops it.
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
 The tagged template that takes a template literal as its substitutions: the tag function is a call whose arguments
 are the substitutions, so a value put in one goes to a call, unlike a value put in a plain template.

 @param node - node that holds a value, of any kind

 @param parents - each node's parent

 @returns Tagged template, `NO_NODE` when the node is no template literal or a plain one

 @example
 ```ts
 const tagged = taggedTemplateOf({ node: holder, parents, },);
 ```
 */
function taggedTemplateOf(
  {
    node,
    parents,
  }: {
    readonly node: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
  },
): TreeNode {
  /**
   The node that holds the template.
   */
  const parent = parents.get(node,);
  // The quasi of a tagged template is always a template literal, so the kind of the node needs no test.
  return ((parent?.type === 'TaggedTemplateExpression') && (parent.quasi === node)) ? parent : NO_NODE;
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

  /**
   The callback a method called on the value hands it to, none when the step is no such method.
   */
  readonly callback: Callback;
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
export function stepOf(
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
    callback: NO_CALLBACK,
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
      leaves: holder,
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
      callback: (holder.object === top)
        ? callbackOf({
          context,
          member: holder,
        },)
        : NO_CALLBACK,
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
      names: namesBoundBy({ node: holder.left, },),
    };
  }
  /**
   The tagged template whose substitutions the holder is, `NO_NODE` when a plain template or another node holds the
   value.
   */
  const tagged = taggedTemplateOf({
    parents: context.parents,
    node: holder,
  },);
  if (!isNoNode({ node: tagged, },)) {
    return {
      ...nothing,
      stray: positionOf({
        holder: tagged,
        child: holder,
      },),
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

//endregion One step of the walk of a builder's value
