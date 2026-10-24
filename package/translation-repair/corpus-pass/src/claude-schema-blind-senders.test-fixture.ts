import type { TreeNode, } from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import {
  type BlindContext,
  outermostValueOf,
} from './claude-schema-blind-positions.test-fixture.ts';
import {
  chainOf,
  COMPUTED_MEMBER,
  NO_CHAIN,
} from './claude-schema-blind-standing.test-fixture.ts';
import { resolvePath, } from './claude-schema-resolve.test-fixture.ts';
import {
  isNoNode,
  keyNameOf,
  NO_NODE,
  nodeAt,
  nodesAt,
} from './claude-schema-tree.test-fixture.ts';

//region Where a stage hands the schema it makes to the bench
// The senders of the third total of the check behind the schema search (`claude-schema-blind-spots.test-fixture.ts`):
// the value of a builder's call may end in an argument of a call only when the call is one of these functions and the
// value is the property of the one object argument that the entry names. A sender is named by the module that declares
// it, so another function of the same name, or a free name, is no sender; any other call that takes the value is
// reported, since it may keep it, copy it or send it where the search cannot read.

/**
 One function where a stage hands the schema it makes to the bench.
 */
export type Sender = {
  /**
   Path of the module that declares the function, under `package/translation-repair`.
   */
  readonly file: string;

  /**
   The function's name.
   */
  readonly name: string;

  /**
   The property of the one object argument that holds the schema.
   */
  readonly parameter: string;

  /**
   Why a value handed here is sent and not kept.
   */
  readonly reason: string;
};

/**
 The senders of the family: the three consistency asks of `corpus-pass` hand their schema to the bench through the
 first, and the stages that share the contest builder through the second.
 */
export const SENDERS: readonly Sender[] = [
  {
    file: 'corpus-pass/src/corpus-run/pass-consistency-gather.ts',
    name: 'gatherThroughCache',
    parameter: 'responseFormat',
    reason: 'it is the schema every seat of the bench is asked with; the function passes it to `gatherStageVoices` '
      + 'and keeps nothing else of it',
  },
  {
    file: 'stage/src/stage-windowed-rounds.ts',
    name: 'runWindowedRounds',
    parameter: 'responseFormat',
    reason: 'it is the schema every reply of the windowed rounds must fit; the function keeps it in the request each '
      + 'round asks with',
  },
];

/**
 The bindings and exports already followed when a callee is resolved from the start.
 */
const NOTHING_FOLLOWED: ReadonlySet<string> = new Set<string>();

/**
 The call that takes an object literal as one of its arguments.

 @param context - what the file's names are read against

 @param object - object literal

 @returns Call, `NO_NODE` when the object is the argument of none

 @example
 ```ts
 const call = callTaking({ context, object, },);
 ```
 */
function callTaking(
  {
    context,
    object,
  }: {
    readonly context: BlindContext;
    readonly object: TreeNode;
  },
): TreeNode {
  /**
   The object under the marks that leave it the same value.
   */
  const top = outermostValueOf({
    node: object,
    parents: context.parents,
  },);
  /**
   The node that holds the object.
   */
  const holder = context.parents
    .get(top,);
  if (holder?.type !== 'CallExpression')
    return NO_NODE;
  return nodesAt({
    node: holder,
    field: 'arguments',
  },)
    .includes(top,)
    ? holder
    : NO_NODE;
}

/**
 Whether a call is a call of a sender, written as the file writes it.

 @param context - what the file's names are read against

 @param call - call

 @param parameter - property of the object argument that holds the value

 @returns Whether the callee resolves to a function of the list whose parameter is the property

 @example
 ```ts
 const isSender = callsSender({ context, call, parameter: 'responseFormat', },);
 ```
 */
function callsSender(
  {
    context,
    call,
    parameter,
  }: {
    readonly context: BlindContext;
    readonly call: TreeNode;
    readonly parameter: string;
  },
): boolean {
  /**
   The names the callee is written with.
   */
  const chain = chainOf({ node: call.callee, },);
  if (chain === NO_CHAIN)
    return false;
  if (chain.members
    .includes(COMPUTED_MEMBER,))
    return false;
  /**
   What the callee stands for in this file.
   */
  const callee = resolvePath({
    family: context.family,
    file: context.file,
    path: [
      chain.head,
      ...chain.members,
    ],
    seen: NOTHING_FOLLOWED,
  },);
  return (callee.kind === 'function') && SENDERS.some(function isCallee(sender,): boolean {
    return (sender.file === callee.file)
      && (sender.name === callee.name)
      && (sender.parameter === parameter);
  },);
}

/**
 Whether a property of an object literal is the parameter of a sender: the object is the argument of a call whose callee
 is that sender, and the key is the entry's parameter.

 @param context - what the file's names are read against

 @param property - property that holds the value

 @param object - object literal that holds the property

 @returns Whether the value of the property is sent

 @example
 ```ts
 const isSent = isSenderProperty({ context, property, object, },);
 ```
 */
export function isSenderProperty(
  {
    context,
    property,
    object,
  }: {
    readonly context: BlindContext;
    readonly property: TreeNode;
    readonly object: TreeNode;
  },
): boolean {
  if (property.computed === true)
    return false;
  /**
   The name of the property, empty for a key that is no name.
   */
  const key = keyNameOf({
    key: nodeAt({
      node: property,
      field: 'key',
    },),
  },);
  /**
   The call that takes the object.
   */
  const call = callTaking({
    context,
    object,
  },);
  return (key !== '')
    && (!isNoNode({ node: call, },))
    && callsSender({
      context,
      call,
      parameter: key,
    },);
}

//endregion Where a stage hands the schema it makes to the bench
