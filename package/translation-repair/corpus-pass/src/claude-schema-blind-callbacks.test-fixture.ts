import {
  identifierName,
  isTreeNode,
  type TreeNode,
  unwrapped,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import {
  type BlindContext,
  namesBoundBy,
  outermostValueOf,
} from './claude-schema-blind-positions.test-fixture.ts';
import {
  isBoundAround,
  isDeclaredAtTop,
} from './claude-schema-blind-scopes.test-fixture.ts';
import {
  isFunctionNode,
  NO_NODE,
  nodeAt,
  nodesAt,
  stringValueOf,
} from './claude-schema-tree.test-fixture.ts';

//region The callback a method hands a value to
// What the walk of a builder's value (`claude-schema-blind-flow.test-fixture.ts`) asks when the value is the object a
// method is read off and the member is called: whether the method calls a function it is given with the value, and
// which parameters of that function the value arrives in. `forEach`, `map`, `filter`, `flatMap`, `find`, `some` and
// `every` call it with the element, the index and the array; `reduce` with the accumulator, the element, the index and
// the array. The callback is the first argument. A function written there is read; so is the name of a function the file
// declares, and `Boolean`, `String` and `Number` that no scope binds, which keep nothing of what they are given. Any
// other callback is code the search cannot read. So is a function written as an argument of any other method called
// on the value, `sort`, `findLast` and a method of the object that holds the value included.

/**
 The position of the array in the parameters of the callback of `reduce`, which has the accumulator, the element and the
 index before it.
 */
const FOURTH_PARAMETER = 3;

/**
 The positions of the parameters of a callback that the value arrives in when the method calls it with the element: the
 element and the array. The index is no parameter the value arrives in.
 */
const ELEMENT_POSITIONS: readonly number[] = [
  0,
  2,
];

/**
 The positions of the parameters of a callback that the value arrives in when the method is `reduce`: the accumulator,
 the element and the array.
 */
const REDUCE_POSITIONS: readonly number[] = [
  0,
  1,
  FOURTH_PARAMETER,
];

/**
 The positions of the parameters the value arrives in, by the methods that call a callback with it.
 */
const VALUE_PARAMETERS: ReadonlyMap<string, readonly number[]> = new Map(Object.entries({
  every: ELEMENT_POSITIONS,
  filter: ELEMENT_POSITIONS,
  find: ELEMENT_POSITIONS,
  flatMap: ELEMENT_POSITIONS,
  forEach: ELEMENT_POSITIONS,
  map: ELEMENT_POSITIONS,
  reduce: REDUCE_POSITIONS,
  some: ELEMENT_POSITIONS,
},),);

/**
 The names of the global conversion functions that keep nothing of what they are given.
 */
const KEEPING_NOTHING: ReadonlySet<string> = new Set([
  'Boolean',
  'Number',
  'String',
],);

/**
 The callback a method hands the value to.
 */
export type Callback = {
  /**
   The function whose parameters the value arrives in, `NO_NODE` when the method hands it to none.
   */
  readonly scope: TreeNode;

  /**
   The names the parameters bind.
   */
  readonly names: readonly string[];

  /**
   What a report says of the method when its callback cannot be read, empty when it can.
   */
  readonly unreadable: string;
};

/**
 A method that hands the value to no callback.
 */
export const NO_CALLBACK: Callback = {
  scope: NO_NODE,
  names: [],
  unreadable: '',
};

/**
 The function a callback argument stands for, `NO_NODE` when it is none the file declares.

 @param context - what the file's names are read against

 @param callback - callback argument

 @returns Function written there, or declared by the file under that name when no scope around the callback binds the
 name again

 @example
 ```ts
 const fn = functionOf({ context, callback: argument, },);
 ```
 */
function functionOf(
  {
    context,
    callback,
  }: {
    readonly context: BlindContext;
    readonly callback: TreeNode;
  },
): TreeNode {
  if (isFunctionNode({ node: callback, },))
    return callback;
  /**
   The name the callback is written with, empty for any other expression.
   */
  const name = identifierName({ node: callback, },);
  /**
   The function the file declares under that name.
   */
  const declared = context.facts
    .declared
    .find(function isNamed(candidate,): boolean {
      return candidate.name === name;
    },);
  /**
   Whether a scope around the callback binds the name again, which makes it another thing than the declared function.
   */
  const isBoundAgain = isBoundAround({
    node: callback,
    name,
    parents: context.parents,
  },);
  /**
   Whether the callback names a function of the file that no scope around it hides.
   */
  const isDeclaredName = (name !== '') && (declared !== undefined);
  return (isDeclaredName && (!isBoundAgain)) ? declared.fn : NO_NODE;
}

/**
 The parameters of a function that the value arrives in, as the names they bind.

 @param fn - callback function

 @param indexes - positions of the parameters the value arrives in

 @returns Names in order; a rest parameter binds the value for every position from its own on

 @example
 ```ts
 const names = valueNamesOf({ fn, indexes: [0, 2,], },);
 ```
 */
function valueNamesOf(
  {
    fn,
    indexes,
  }: {
    readonly fn: TreeNode;
    readonly indexes: readonly number[];
  },
): readonly string[] {
  return nodesAt({
    node: fn,
    field: 'params',
  },)
    .flatMap(function boundByParameter(
      parameter,
      position,
    ): readonly string[] {
      return indexes.some(function arrivesIn(index,): boolean {
        return (index === position) || ((parameter.type === 'RestElement') && (index > position));
      },)
        ? namesBoundBy({ node: parameter, },)
        : [];
    },);
}

/**
 Whether a call has a function written as an argument, under the marks that leave it the same.

 @param call - call

 @returns Whether some argument is an arrow function, a function expression or a function under a mark

 @example
 ```ts
 const holds = holdsFunctionArgument({ call, },);
 ```
 */
function holdsFunctionArgument({ call, }: { readonly call: TreeNode; },): boolean {
  return nodesAt({
    node: call,
    field: 'arguments',
  },)
    .some(function isFunctionWritten(argument,): boolean {
      /**
       The argument once the marks around it are taken off.
       */
      const { inner, } = unwrapped({ node: argument, },);
      return isTreeNode(inner,) && isFunctionNode({ node: inner, },);
    },);
}

/**
 The callback a member hands the value to, when the member is a method of the list called with a callback. A method
 outside the list that is called with a function written as an argument hands the value to code the search cannot read.

 @param context - what the file's names are read against

 @param member - member the value is the object of

 @returns Function and the names its parameters bind, or the report for a callback the search cannot read; no callback
 when the member is not called or names no method of the list

 @example
 ```ts
 const callback = callbackOf({ context, member: holder, },);
 ```
 */
export function callbackOf(
  {
    context,
    member,
  }: {
    readonly context: BlindContext;
    readonly member: TreeNode;
  },
): Callback {
  /**
   The member under the marks that leave it the same, the node a call has as its callee.
   */
  const top = outermostValueOf({
    node: member,
    parents: context.parents,
  },);
  /**
   The node that holds the member.
   */
  const call = context.parents
    .get(top,);
  /**
   The name of the method, empty when it is computed from anything but a string.
   */
  const method = (member.computed === true)
    ? stringValueOf({ node: nodeAt({
      node: member,
      field: 'property',
    },), },)
    : identifierName({ node: member.property, },);
  /**
   The positions of the parameters the value arrives in, absent for a method that calls no callback with it.
   */
  const indexes = VALUE_PARAMETERS.get(method,);
  if (call === undefined)
    return NO_CALLBACK;
  if ((call.type !== 'CallExpression') || (call.callee !== top))
    return NO_CALLBACK;
  if (indexes === undefined) {
    return holdsFunctionArgument({ call, },)
      ? {
        ...NO_CALLBACK,
        unreadable: `callback of ${method} cannot be read`,
      }
      : NO_CALLBACK;
  }
  /**
   The first argument of the call, absent when the call has none, and the arguments after it.
   */
  const [
    first,
    ...more
  ] = nodesAt({
    node: call,
    field: 'arguments',
  },);
  /**
   The positions the value arrives in: a start value given to `reduce` is what the accumulator holds at first, so the
   value reaches it only in what the callback returns, which the walk follows as it follows any return.
   */
  const positions = ((method === 'reduce') && (more.length > 0))
    ? indexes.filter(function isAfterAccumulator(index,): boolean {
      return index > 0;
    },)
    : indexes;
  /**
   The first argument once the marks around it are taken off.
   */
  const { inner, } = unwrapped({ node: first, },);
  /**
   The callback as a node, `NO_NODE` when the call has none or it is a spread.
   */
  const callback = (isTreeNode(inner,) && (inner.type !== 'SpreadElement')) ? inner : NO_NODE;
  /**
   The function the callback stands for.
   */
  const fn = functionOf({
    context,
    callback,
  },);
  if (isFunctionNode({ node: fn, },)) {
    return {
      ...NO_CALLBACK,
      scope: fn,
      names: valueNamesOf({
        fn,
        indexes: positions,
      },),
    };
  }
  /**
   The name a conversion function is written with, empty for any other callback.
   */
  const name = identifierName({ node: callback, },);
  /**
   Whether a scope around the callback binds the name again.
   */
  const isBoundAgain = isBoundAround({
    node: callback,
    name,
    parents: context.parents,
  },);
  /**
   The program of the file.
   */
  const { program, } = context.facts;
  /**
   Whether the file binds the name as an import, a function or an object.
   */
  const isBoundAsBinding = context.facts
    .bindings
    .has(name,);
  /**
   Whether the file binds the name at its top level, by any declaration.
   */
  const isDeclared = isDeclaredAtTop({
    program,
    name,
  },);
  /**
   Whether the file binds the name at its top level.
   */
  const isBoundAtTop = isBoundAsBinding || isDeclared;
  /**
   Whether the callback is a conversion function nothing binds again.
   */
  const isConversion = KEEPING_NOTHING.has(name,);
  /**
   Whether neither a scope around the callback nor the top level of the file binds the name.
   */
  const isFree = !(isBoundAgain || isBoundAtTop);
  return (isConversion && isFree)
    ? NO_CALLBACK
    : {
      ...NO_CALLBACK,
      unreadable: `callback of ${method} cannot be read`,
    };
}

//endregion The callback a method hands a value to
