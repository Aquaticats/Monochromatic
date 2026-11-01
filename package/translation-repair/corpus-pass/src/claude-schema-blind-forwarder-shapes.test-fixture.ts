import type { TreeNode, } from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import {
  type BlindContext,
  outermostValueOf,
  positionOf,
} from './claude-schema-blind-positions.test-fixture.ts';
import type { FoundShape, } from './claude-schema-blind-spots.test-fixture.ts';
import {
  type Chain,
  COMPUTED_MEMBER,
} from './claude-schema-blind-standing.test-fixture.ts';

//region Where a forwarder sits
// The part of the second total of the check behind the schema search (`claude-schema-blind-chains.test-fixture.ts`)
// that reads a chain of names standing for a forwarder, a function that hands a sender a builder call made from its own
// parameters. A forwarder is read as the callee of a call, of `new` or of a tagged template, as the whole value of an
// export assignment, and through the members `call`, `apply` and `bind`; the members `name` and `length` hand out no
// function. Any other place that holds one is reported, since the files that get the function there can call it where
// the listing of callers does not look.

/**
 The members of a function that call it: `build.call(...)`, `build.apply(...)` and `build.bind(...)` are `build`.
 */
export const CALL_METHODS: ReadonlySet<string> = new Set([
  'apply',
  'bind',
  'call',
],);

/**
 The members of a function that read a string or a number off it and hand out no function: the logger tag of the repo is
 `fn.name`.
 */
const NON_CALLING_MEMBERS: ReadonlySet<string> = new Set([
  'length',
  'name',
],);

/**
 The shape a forwarder holds where it stands, by the node that holds it: a call of it is read, since the files that call
 it are listed, and so is the whole value of an export assignment, which is read as the module. Any other place is
 reported, since a file that gets the function there can call it unseen.

 @param context - what the file's names are read against

 @param node - chain that stands for the forwarder

 @param isBound - whether the chain ends in `.bind`, so what the call of it gives is the forwarder again

 @param held - name of the forwarder, written into the shape

 @returns Shape, none where the forwarder is called

 @example
 ```ts
 const found = forwarderShapesAt({ context, node, isBound: false, held: 'build', },);
 ```
 */
function forwarderShapesAt(
  {
    context,
    node,
    isBound,
    held,
  }: {
    readonly context: BlindContext;
    readonly node: TreeNode;
    readonly isBound: boolean;
    readonly held: string;
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
   The node that holds the forwarder.
   */
  const holder = context.parents
    .get(top,);
  if (holder === undefined)
    return [];
  /**
   Whether the holder is a call or `new`.
   */
  const isCallOrNew = (holder.type === 'CallExpression') || (holder.type === 'NewExpression');
  /**
   Whether the forwarder is the tag of a tagged template.
   */
  const isTag = (holder.type === 'TaggedTemplateExpression') && (holder.tag === top);
  /**
   Whether the forwarder is the callee of a call, of `new` or of a tagged template, which the listing of callers reads.
   */
  const isCalled = (isCallOrNew && (holder.callee === top)) || isTag;
  if (isCalled && isBound) {
    return forwarderShapesAt({
      context,
      node: holder,
      isBound: false,
      held,
    },);
  }
  /**
   Whether the forwarder is the whole value of an export assignment, which the listing of callers reads as the module
   itself.
   */
  const isExported = holder.type === 'TSExportAssignment';
  return (isCalled || isExported)
    ? []
    : [
      {
        shape: 'forwarder-outside-a-read-position',
        node,
        position: positionOf({
          holder,
          child: top,
        },),
        held,
      },
    ];
}

/**
 The shapes a chain holds once it stands for a forwarder after `count` members: the members left must be `call`, `apply`
 or `bind`, and the place it sits in must be a call.

 @param context - what the file's names are read against

 @param chain - chain read

 @param node - chain, as the node that starts it

 @param count - members that lead to the forwarder

 @param held - name of the forwarder, written into the shape

 @returns Shape, none for a chain that is called

 @example
 ```ts
 const found = forwarderShapesAlong({ context, chain, node, count: 1, held: 'build', },);
 ```
 */
export function forwarderShapesAlong(
  {
    context,
    chain,
    node,
    count,
    held,
  }: {
    readonly context: BlindContext;
    readonly chain: Chain;
    readonly node: TreeNode;
    readonly count: number;
    readonly held: string;
  },
): readonly FoundShape[] {
  /**
   The members after the forwarder.
   */
  const rest = chain.members
    .slice(count,);
  if (NON_CALLING_MEMBERS.has(rest.at(0,) ?? '',))
    return [];
  /**
   The first member after the forwarder that is no call method, absent when there is none.
   */
  const stray = rest.find(function isStray(member,): boolean {
    return !CALL_METHODS.has(member,);
  },);
  if (stray !== undefined) {
    return [
      {
        shape: 'forwarder-outside-a-read-position',
        node,
        position: (stray === COMPUTED_MEMBER) ? 'member computed from a name' : `member ${stray} of a forwarder`,
        held,
      },
    ];
  }
  return forwarderShapesAt({
    context,
    node,
    isBound: rest.at(-1,) === 'bind',
    held,
  },);
}

//endregion Where a forwarder sits
