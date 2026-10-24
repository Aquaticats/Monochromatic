import type { TreeNode, } from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import { leavingFunctionOf, } from './claude-schema-blind-flow.test-fixture.ts';
import {
  bindsAsReadBy,
  type BlindContext,
  outermostValueOf,
  positionOf,
  unreadConstructAround,
} from './claude-schema-blind-positions.test-fixture.ts';
import type { FoundShape, } from './claude-schema-blind-spots.test-fixture.ts';
import {
  type Chain,
  chainOf,
  COMPUTED_MEMBER,
  isImportOfNothing,
  NO_CHAIN,
  standingAfter,
} from './claude-schema-blind-standing.test-fixture.ts';
import { isNoNode, } from './claude-schema-tree.test-fixture.ts';

//region Where a chain of names that stands for a builder or for a module that carries one sits
// The second total of the check behind the schema search: every chain of names that stands for a builder, and every
// chain that stands for a module that carries one, must sit where the search reads it. What a chain stands for is read
// by `claude-schema-blind-standing.test-fixture.ts`. A builder is read as the callee of a call, of `new` or of a tagged
// template, or as the value of a constant that the search binds to it; a module is read as the object of a member, or
// as the value of a constant or a pattern that the search binds to it. Any other place that holds one is reported.

/**
 The members of a function that call it: `build.call(...)`, `build.apply(...)` and `build.bind(...)` are `build`.
 */
const CALL_METHODS: ReadonlySet<string> = new Set([
  'apply',
  'bind',
  'call',
],);

/**
 The shape a builder holds where it stands, by the node that holds it.

 @param context - what the file's names are read against

 @param node - chain that stands for the builder

 @param isBound - whether the chain ends in `.bind`, so what the call of it gives is the builder again

 @returns The shape, none where the search reads the place

 @example
 ```ts
 const found = builderShapesAt({ context, node, isBound: false, },);
 ```
 */
function builderShapesAt(
  {
    context,
    node,
    isBound,
  }: {
    readonly context: BlindContext;
    readonly node: TreeNode;
    readonly isBound: boolean;
  },
): readonly FoundShape[] {
  /**
   The expression under the marks that leave it the same value.
   */
  const top = outermostValueOf({
    node,
    parents: context.parents,
  },);
  /**
   The node that holds the builder.
   */
  const holder = context.parents
    .get(top,);
  if (holder === undefined)
    return [];
  if (((holder.type === 'CallExpression') || (holder.type === 'NewExpression')) && (holder.callee === top)) {
    return (isBound && (holder.type === 'CallExpression'))
      ? builderShapesAt({
        context,
        node: holder,
        isBound: false,
      },)
      : callShapesAt({
        context,
        call: holder,
      },);
  }
  if ((holder.type === 'TaggedTemplateExpression') && (holder.tag === top)) {
    return callShapesAt({
      context,
      call: holder,
    },);
  }
  if ((holder.type === 'VariableDeclarator') && (holder.init === top)
    && bindsAsReadBy({
      declarator: holder,
      parents: context.parents,
      facts: context.facts,
    },))
    return [];
  return [
    {
      shape: 'builder-outside-a-read-position',
      node,
      position: positionOf({
        holder,
        child: top,
      },),
    },
  ];
}

/**
 The shape a call of a builder holds: its value goes out of a function that the search does not know as a builder.

 @param context - what the file's names are read against

 @param call - call, `new` or tagged template of a builder

 @returns The shape, none where the value stays or goes into a call

 @example
 ```ts
 const found = callShapesAt({ context, call, },);
 ```
 */
function callShapesAt(
  {
    context,
    call,
  }: {
    readonly context: BlindContext;
    readonly call: TreeNode;
  },
): readonly FoundShape[] {
  /**
   The function the value is returned from, `NO_NODE` when it is not.
   */
  const leaving = leavingFunctionOf({
    context,
    call,
  },);
  return isNoNode({ node: leaving, },)
    ? []
    : [
      {
        shape: 'builder-result-returned-outside-a-wrapper',
        node: call,
        position: `${leaving.type} returns it`,
      },
    ];
}

/**
 The shape a module that carries a builder holds where it stands as a whole.

 @param context - what the file's names are read against

 @param node - chain that stands for the module or the object

 @returns The shape, none where the search binds the names a constant or a pattern takes from it

 @example
 ```ts
 const found = carrierShapesAt({ context, node, },);
 ```
 */
function carrierShapesAt(
  {
    context,
    node,
  }: {
    readonly context: BlindContext;
    readonly node: TreeNode;
  },
): readonly FoundShape[] {
  /**
   The expression under the marks that leave it the same value.
   */
  const top = outermostValueOf({
    node,
    parents: context.parents,
  },);
  /**
   The node that holds the module.
   */
  const holder = context.parents
    .get(top,);
  if (holder === undefined)
    return [];
  if ((holder.type === 'VariableDeclarator') && (holder.init === top)
    && bindsAsReadBy({
      declarator: holder,
      parents: context.parents,
      facts: context.facts,
    },))
    return [];
  return [
    {
      shape: 'module-binding-outside-a-read-position',
      node,
      position: positionOf({
        holder,
        child: top,
      },),
    },
  ];
}

/**
 The shapes a chain of names holds when it stands for a builder or for a module that carries one.

 @param context - what the file's names are read against

 @param node - chain, as the node that starts it

 @returns One entry for the first place the chain is read in a way the search does not, none for any chain that stands
 for neither or is read where the search reads it

 @example
 ```ts
 const found = chainShapesAt({ context, node, },);
 ```
 */
export function chainShapesAt(
  {
    context,
    node,
  }: {
    readonly context: BlindContext;
    readonly node: TreeNode;
  },
): readonly FoundShape[] {
  /**
   The names the chain is written with.
   */
  const chain = chainOf({ node, },);
  if (chain === NO_CHAIN)
    return [];
  if ((chain.members
    .length
    > 0) && isImportOfNothing({
      context,
      head: chain.head,
    },)) {
    return [
      {
        shape: 'module-binding-outside-a-read-position',
        node,
        position: 'member of a default the module does not export',
      },
    ];
  }
  /**
   How many members at the start of the chain can be read.
   */
  const readable = chain.members
    .includes(COMPUTED_MEMBER,)
    ? chain.members
      .indexOf(COMPUTED_MEMBER,)
    : chain.members
      .length;
  return shapesAlong({
    context,
    chain,
    node,
    readable,
    count: 0,
  },);
}

/**
 Walks along a chain, one member at a time, until it stands for a builder, a module that carries one, or neither.

 @param context - what the file's names are read against

 @param chain - chain read

 @param node - chain, as the node that starts it

 @param readable - how many members at the start of the chain can be read

 @param count - members read so far

 @returns The shape, none for a chain that stands for neither or is read where the search reads it

 @example
 ```ts
 const found = shapesAlong({ context, chain, node, readable: 1, count: 0, },);
 ```
 */
function shapesAlong(
  {
    context,
    chain,
    node,
    readable,
    count,
  }: {
    readonly context: BlindContext;
    readonly chain: Chain;
    readonly node: TreeNode;
    readonly readable: number;
    readonly count: number;
  },
): readonly FoundShape[] {
  /**
   What the chain stands for after the members read so far.
   */
  const {
    standing,
    isFree,
  } = standingAfter({
    context,
    chain,
    count,
  },);
  if (standing === 'builder') {
    return builderShapesAlong({
      context,
      chain,
      node,
      count,
    },);
  }
  if (standing === 'other') {
    return (isFree && (count < readable))
      ? shapesAlong({
        context,
        chain,
        node,
        readable,
        count: count + 1,
      },)
      : [];
  }
  /**
   The kind of construct the search does not read that holds the module, empty when none does.
   */
  const construct = unreadConstructAround({
    node,
    parents: context.parents,
  },);
  if (construct !== '') {
    return [
      {
        shape: 'builder-in-typescript-value-construct',
        node,
        position: construct,
      },
    ];
  }
  if (count
    === chain.members
      .length) {
    return carrierShapesAt({
      context,
      node,
    },);
  }
  return (count >= readable)
    ? [
      {
        shape: 'module-binding-outside-a-read-position',
        node,
        position: 'member computed from a name',
      },
    ]
    : shapesAlong({
      context,
      chain,
      node,
      readable,
      count: count + 1,
    },);
}

/**
 The shapes a chain holds once it stands for a builder after `count` members: the members left must be `call`, `apply`
 or `bind`, and the place it sits in must be one the search reads.

 @param context - what the file's names are read against

 @param chain - chain read

 @param node - chain, as the node that starts it

 @param count - members that lead to the builder

 @returns The shape, none for a chain read where the search reads it

 @example
 ```ts
 const found = builderShapesAlong({ context, chain, node, count: 1, },);
 ```
 */
function builderShapesAlong(
  {
    context,
    chain,
    node,
    count,
  }: {
    readonly context: BlindContext;
    readonly chain: Chain;
    readonly node: TreeNode;
    readonly count: number;
  },
): readonly FoundShape[] {
  /**
   The members after the builder.
   */
  const rest = chain.members
    .slice(count,);
  /**
   The first member after the builder that is no call method, absent when there is none; a member computed from
   anything but a string is one.
   */
  const stray = rest.find(function isStray(member,): boolean {
    return (member === COMPUTED_MEMBER) || (!CALL_METHODS.has(member,));
  },);
  if (stray !== undefined) {
    return [
      {
        shape: 'builder-outside-a-read-position',
        node,
        position: (stray === COMPUTED_MEMBER) ? 'member computed from a name' : `member ${stray} of a builder`,
      },
    ];
  }
  /**
   Where the construct the search does not read holds the chain, empty when none does.
   */
  const construct = unreadConstructAround({
    node,
    parents: context.parents,
  },);
  if (construct !== '') {
    return [
      {
        shape: 'builder-in-typescript-value-construct',
        node,
        position: construct,
      },
    ];
  }
  return builderShapesAt({
    context,
    node,
    isBound: rest.at(-1,) === 'bind',
  },);
}

//endregion Where a chain of names that stands for a builder or for a module that carries one sits
