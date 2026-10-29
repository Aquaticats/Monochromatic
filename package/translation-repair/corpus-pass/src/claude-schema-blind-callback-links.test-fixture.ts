import {
  isTreeNode,
  type TreeNode,
  unwrapped,
} from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import { namesBoundBy, } from './claude-schema-blind-positions.test-fixture.ts';
import type { Link, } from './claude-schema-blind-stores.test-fixture.ts';
import {
  nodeAt,
  nodesAt,
} from './claude-schema-tree.test-fixture.ts';

//region Names a function written as an argument binds from what its call reads
// A function written inside a call, as an argument, takes its parameters from the call: the callback of `forEach`,
// `map` and `then` gets the elements of the value the method is called on, the callback of `reduce` also gets the
// start value, and any other call may hand its other arguments to the function. So the parameters of such a function
// are names made from a parameter of the enclosing function when the callee or another argument of the call reads one,
// and a value stored through them (`args.forEach((value) => { holder.a = value; })`) is stored from a parameter all the
// same (`claude-schema-blind-stores.test-fixture.ts`). The call is read widely, as the call of a function that is not
// known is read everywhere else: the callee and every argument that is no function count, whatever the method is. A
// function that is declared apart and handed over by its name (`args.forEach(put)`) is not read: its parameters are
// bound where it is declared, and what is handed to it is a flow between two functions. A function written as the
// callee (`((value) => { holder.a = value; })(args)`) takes the value of each argument of its call.

/**
 The kinds of node that are a function written as a value.
 */
const FUNCTION_VALUES: ReadonlySet<string> = new Set([
  'ArrowFunctionExpression',
  'FunctionExpression',
]);

/**
 Whether an argument is a function written in place, whatever type marks it is written under.

 @param node - argument of a call

 @returns Whether it is an arrow function or a function expression

 @example
 ```ts
 const isCallback = isFunctionValue({ node: argument, },);
 ```
 */
function isFunctionValue({ node, }: { readonly node: TreeNode; },): boolean {
  /**
   The argument without the type marks and parentheses around it.
   */
  const { inner, } = unwrapped({ node, },);
  return isTreeNode(inner,) && FUNCTION_VALUES.has(inner.type,);
}

/**
 The names the parameters of a function written in place bind.

 @param node - argument of a call that is a function written in place

 @returns Names in order

 @example
 ```ts
 const names = parameterNamesOf({ node: callback, },);
 ```
 */
function parameterNamesOf({ node, }: { readonly node: TreeNode; },): readonly string[] {
  /**
   The function without the type marks and parentheses around it.
   */
  const { inner, } = unwrapped({ node, },);
  return isTreeNode(inner,)
    ? nodesAt({
      node: inner,
      field: 'params',
    },)
      .flatMap(function boundByParameter(parameter,): readonly string[] {
        return namesBoundBy({ node: parameter, },);
      },)
    : [];
}

/**
 The links a call makes for the function written as its callee: its parameters take the value of each argument.

 @param node - call or `new` expression

 @returns One link for each argument when the callee is an arrow function or a function expression, whatever type marks
 and parentheses it is written under, none for any other callee

 @example
 ```ts
 const links = calledFunctionLinksOf({ node, },);
 ```
 */
function calledFunctionLinksOf({ node, }: { readonly node: TreeNode; },): readonly Link[] {
  /**
   The callee, which `isFunctionValue` and `parameterNamesOf` read without the type marks and parentheses around it.
   */
  const callee = nodeAt({
    node,
    field: 'callee',
  },);
  if (!isFunctionValue({ node: callee, },))
    return [];
  /**
   The names the parameters of the called function bind.
   */
  const bound = parameterNamesOf({ node: callee, },);
  return nodesAt({
    node,
    field: 'arguments',
  },)
    .map(function linkOfArgument(read,): Link {
      return {
        read,
        bound,
      };
    },);
}

/**
 The links a call makes for the functions written as its arguments: their parameters take the value of the callee and
 of each argument that is no function written in place.

 @param node - node read

 @returns One link for the callee and one for each other argument when the call has such a function, none for any other
 node

 @example
 ```ts
 const links = argumentLinksOf({ node, },);
 ```
 */
function argumentLinksOf({ node, }: { readonly node: TreeNode; },): readonly Link[] {
  if ((node.type !== 'CallExpression') && (node.type !== 'NewExpression'))
    return [];
  /**
   The arguments of the call that are functions written in place.
   */
  const callbacks = nodesAt({
    node,
    field: 'arguments',
  },)
    .filter(function isCallback(argument,): boolean {
      return isFunctionValue({ node: argument, },);
    },);
  /**
   The names the parameters of those functions bind.
   */
  const bound = callbacks.flatMap(function boundByFunction(written,): readonly string[] {
    return parameterNamesOf({ node: written, },);
  },);
  if (bound.length === 0)
    return [];
  return [
    nodeAt({
      node,
      field: 'callee',
    },),
    ...nodesAt({
      node,
      field: 'arguments',
    },)
      .filter(function isValue(argument,): boolean {
        return !callbacks.includes(argument,);
      },),
  ].map(function linkOfRead(read,): Link {
    return {
      read,
      bound,
    };
  },);
}

/**
 The links a call makes for the functions written in it: the functions written as its arguments take the value of the
 callee and of each argument that is no function written in place, and a function written as the callee takes the value
 of each argument.

 @param node - node read

 @returns Links of the arguments, then the links of the callee, none for a node that is no call or `new`

 @example
 ```ts
 const links = callbackLinksOf({ node, },);
 ```
 */
export function callbackLinksOf({ node, }: { readonly node: TreeNode; },): readonly Link[] {
  return [
    ...argumentLinksOf({ node, },),
    ...(((node.type === 'CallExpression') || (node.type === 'NewExpression'))
      ? calledFunctionLinksOf({ node, },)
      : []),
  ];
}

//endregion Names a function written as an argument binds from what its call reads
