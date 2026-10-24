import type { TreeNode, } from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import {
  type BlindContext,
  outermostValueOf,
} from './claude-schema-blind-positions.test-fixture.ts';
import { isBoundAround, } from './claude-schema-blind-scopes.test-fixture.ts';
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
// value is the property of the one object argument that the entry names, at the argument the sender reads: the first,
// or the second through `.call`, and none through `.apply` or `.bind`. A sender is named by the module that declares
// it, so another function of the same name, a free name, or a name that a scope around the call binds again (a
// parameter, a constant, a nested function or class, a loop variable, a `catch` binding) is no sender; any other call
// that takes the value is reported, since it may keep it, copy it or send it where the search cannot read.

/**
 One function where a stage hands the schema it makes to the bench.
 */
type Sender = {
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
const SENDERS: readonly Sender[] = [
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
 The call that holds an object literal. The object is one of its arguments: a call whose callee is the object names no
 function, and a callee that is no name is no call of a sender.

 @param context - what the file's names are read against

 @param object - object literal

 @returns Call, `NO_NODE` when no call holds the object

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
  return (holder?.type === 'CallExpression') ? holder : NO_NODE;
}

/**
 The place of an argument among the arguments of a call that no sender reads: the call is a `.apply` or a `.bind`. It
 differs from -1, which is what the index of an object that is no argument of the call is.
 */
const NO_ARGUMENT = -2;

/**
 The place among the arguments of a call where the function the callee names reads its object. A function called
 directly reads its first argument; through `.call` the first argument is the `this` it is called with, so it reads the
 second; through `.apply` the arguments are a list, and through `.bind` they are a partial application, so none of the
 arguments written there is read as the object.

 @param members - names the callee is written with after its head

 @returns Index among the arguments, `NO_ARGUMENT` when the call writes none that is read

 @example
 ```ts
 const read = argumentReadBy({ members: ['call',], },); // 1
 ```
 */
function argumentReadBy({ members, }: { readonly members: readonly string[]; },): number {
  /**
   The last name of the callee.
   */
  const last = members.at(-1,);
  if (last === 'call')
    return 1;
  return ((last === 'apply') || (last === 'bind')) ? NO_ARGUMENT : 0;
}

/**
 Whether a call is a call of a sender, written as the file writes it.

 @param context - what the file's names are read against

 @param call - call

 @param parameter - property of the object argument that holds the value

 @param argument - index of the object among the arguments of the call

 @returns Whether the callee resolves to a function of the list whose parameter is the property, and reads the object
 at that argument

 @example
 ```ts
 const isSender = callsSender({ context, call, parameter: 'responseFormat', argument: 0, },);
 ```
 */
function callsSender(
  {
    context,
    call,
    parameter,
    argument,
  }: {
    readonly context: BlindContext;
    readonly call: TreeNode;
    readonly parameter: string;
    readonly argument: number;
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
  if (argumentReadBy({ members: chain.members, },) !== argument)
    return false;
  if (isBoundAround({
    node: call,
    name: chain.head,
    parents: context.parents,
  },))
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
  if ((key === '') || isNoNode({ node: call, },))
    return false;
  return callsSender({
    context,
    call,
    parameter: key,
    argument: nodesAt({
      node: call,
      field: 'arguments',
    },)
      .indexOf(outermostValueOf({
        node: object,
        parents: context.parents,
      },),),
  },);
}

//endregion Where a stage hands the schema it makes to the bench
