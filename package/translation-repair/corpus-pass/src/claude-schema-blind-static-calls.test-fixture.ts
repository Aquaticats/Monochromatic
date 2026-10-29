import {
  isTreeNode,
  type TreeNode,
  unwrapped,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import { builtinNameOf, } from './claude-schema-blind-global-values.test-fixture.ts';
import {
  memberNameOf,
  NO_NODE,
  nodeAt,
  nodesAt,
} from './claude-schema-tree.test-fixture.ts';

//region A call of a method of a built-in object, however the call is written
// `Object.assign(holder, value)` is one call of one method with two arguments, and so are
// `Object.assign.call(null, holder, value)`, `Object.assign.apply(null, [holder, value])`,
// `Reflect.apply(Object.assign, null, [holder, value])`, `Object.assign(...[holder, value])` and
// `(0, Object.assign)(holder, value)`, also with the object read off a chain of global objects or written under a type
// mark. The check behind the schema search reads a few methods for what they do with their arguments (the methods that
// store in their first argument, the methods that read a member by a key), so each of them reads the call through this
// one reading. A call whose arguments are spread from anything but an array written in place, and a method that is
// bound and called after, are not read.

/**
 A call of a method of a built-in object, with the arguments the method receives.
 */
type StaticCall = {
  /**
   The name of the built-in object the method is read off, empty when the call is no call of such a method.
   */
  readonly object: string;

  /**
   The name of the method, empty when it is computed from anything but a string.
   */
  readonly method: string;

  /**
   The arguments the method receives, in order, the arguments spread from an array written in place expanded.
   */
  readonly received: readonly TreeNode[];
};

/**
 What a call that is no call of a method of a built-in object reads as.
 */
const NO_STATIC_CALL: StaticCall = {
  object: '',
  method: '',
  received: [],
};

/**
 The expression a call is made on, without the type marks and parentheses around it, and the last operand of a comma.

 @param node - callee, of any kind or none

 @returns Expression, `NO_NODE` for anything that is no node

 @example
 ```ts
 const value = calleeValueOf({ node: call.callee, },); // `Object.assign` for `(0, Object.assign)`
 ```
 */
function calleeValueOf({ node, }: { readonly node: unknown; },): TreeNode {
  /**
   The expression without the type marks and parentheses around it.
   */
  const { inner, } = unwrapped({ node, },);
  if (!isTreeNode(inner,))
    return NO_NODE;
  if (inner.type !== 'SequenceExpression')
    return inner;
  /**
   The last operand of the comma, absent when it has none.
   */
  const last = nodesAt({
    node: inner,
    field: 'expressions',
  },)
    .at(-1,);
  return (last === undefined)
    ? NO_NODE
    : calleeValueOf({ node: last, },);
}

/**
 The object and the method a member expression reads, the object by the name of the built-in object it stands for.

 @param node - member expression, or any node

 @returns Name of the built-in object and name of the method, both empty for a node that is no member

 @example
 ```ts
 const { object, method, } = methodRead({ node: callee, },);
 // => 'Object' and 'assign' for `globalThis.Object.assign`
 ```
 */
function methodRead(
  { node, }: { readonly node: TreeNode; },
): Pick<StaticCall, 'method' | 'object'> {
  if (node.type !== 'MemberExpression') {
    return {
      object: '',
      method: '',
    };
  }
  return {
    object: builtinNameOf({ node: node.object, },),
    method: memberNameOf({ node, },),
  };
}

/**
 The elements of an array written in place, under the type marks that leave it the same value.

 @param node - expression, of any kind or none

 @returns Elements in order, none for anything that is no array written in place

 @example
 ```ts
 const elements = arrayElementsOf({ node: call.arguments.at(1), },);
 ```
 */
function arrayElementsOf({ node, }: { readonly node: unknown; },): readonly TreeNode[] {
  /**
   The expression without the type marks and parentheses around it.
   */
  const { inner, } = unwrapped({ node, },);
  return (isTreeNode(inner,) && (inner.type === 'ArrayExpression'))
    ? nodesAt({
      node: inner,
      field: 'elements',
    },)
    : [];
}

/**
 The arguments of a call with a leading spread of an array written in place expanded into its elements.

 @param args - arguments as the call writes them

 @returns Elements of the array and arguments after the spread, or arguments unchanged when the first is no
 spread of an array written in place

 @example
 ```ts
 const received = spreadOut({ args: nodesAt({ node: call, field: 'arguments', }), },);
 ```
 */
function spreadOut({ args, }: { readonly args: readonly TreeNode[]; },): readonly TreeNode[] {
  /**
   The first argument and the others.
   */
  const [first, ...rest] = args;
  if ((first === undefined) || (first.type !== 'SpreadElement'))
    return args;
  /**
   The elements of the array the spread takes its arguments from, none when it is no array written in place.
   */
  const elements = arrayElementsOf({ node: first.argument, },);
  return (elements.length === 0)
    ? args
    : [
      ...elements,
      ...rest,
    ];
}

/**
 The call of a method of a built-in object that a node is, whichever way it is written: directly, through `call` or
 `apply`, through `Reflect.apply`, with a leading spread of an array written in place, or on the last operand of a
 comma.

 @param call - node read, of any kind

 @returns Name of the object, name of the method and arguments the method receives; the name of the object
 is empty for a node that is no call, and the names are whatever the callee reads for a call of anything else

 @example
 ```ts
 const { object, method, received, } = staticCallOf({ call, },);
 // => 'Object', 'assign' and [holder, value] for `Object.assign.call(null, holder, value)`
 ```
 */
export function staticCallOf({ call, }: { readonly call: TreeNode; },): StaticCall {
  if (call.type !== 'CallExpression')
    return NO_STATIC_CALL;
  /**
   The arguments of the call as it writes them.
   */
  const args = nodesAt({
    node: call,
    field: 'arguments',
  },);
  /**
   The method the call is made on, as the callee reads it.
   */
  const callee = calleeValueOf({ node: nodeAt({
    node: call,
    field: 'callee',
  },), },);
  if (callee.type !== 'MemberExpression')
    return NO_STATIC_CALL;
  /**
   The object and the method the callee reads.
   */
  const read = methodRead({ node: callee, },);
  if ((read.object === 'Reflect') && (read.method === 'apply')) {
    return {
      ...methodRead({ node: calleeValueOf({ node: args.at(0,), },), },),
      received: arrayElementsOf({ node: args.at(2,), },),
    };
  }
  /**
   The method `call` or `apply` is read off, `NO_NODE` when the callee is read off anything else.
   */
  const target = ((read.method === 'call') || (read.method === 'apply'))
    ? calleeValueOf({ node: callee.object, },)
    : NO_NODE;
  if (target.type === 'MemberExpression') {
    return {
      ...methodRead({ node: target, },),
      received: (read.method === 'call')
        ? spreadOut({ args: args.slice(1,), },)
        : arrayElementsOf({ node: args.at(1,), },),
    };
  }
  return {
    ...read,
    received: spreadOut({ args, },),
  };
}

//endregion A call of a method of a built-in object, however the call is written
