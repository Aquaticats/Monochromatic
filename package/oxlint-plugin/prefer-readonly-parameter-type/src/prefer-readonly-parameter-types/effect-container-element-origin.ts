/**
 Which receiver a fresh container's elements came from, through any local hops.
 
 @module
 */

import type {
  Expression,
  Node,
} from 'typescript/ast';
import { isCallExpression, } from 'typescript/ast/is';
import type {
  Checker,
  Project,
} from 'typescript/sync';

import {
  bindingDeclarationInitializer,
  NO_BINDING_INITIALIZER,
} from './effect-binding-initializer.ts';
import {
  callResultElementReceiver,
  RESULT_NOT_RECEIVER_STATE,
} from './effect-member-result-relation.ts';

/**
 Sentinel when nothing proves the value is a container of receiver elements.
 */
export const NOT_A_RECEIVER_CONTAINER: unique symbol = Symbol(
  'value is not a verified container of receiver elements',
);

/**
 Resolves the receiver whose elements an expression's container value holds.
 
 The container relation is recorded on a call, and code rarely writes through the call
 expression itself: `rows.filter(kept)[0].label = 'x'` is legal and unusual, while
 `const kept = rows.filter(keeps); kept[0].label = 'x'` is the shape that appears. So the
 question has to survive local hops, and this follows a binding to its declaration's
 initializer until it reaches a call or runs out.
 
 Only a declaration initializer is followed, never a later assignment. A reassigned local
 therefore keeps answering for the container it was declared with, which over-attributes
 rather than under-attributes: the write is credited to a receiver whose elements the
 local may no longer hold, which costs precision and never an offer.
 
 @param project - TypeScript project resolving declarations and default-library ownership.
 
 @param checker - TypeScript checker resolving signatures and types.
 
 @param node - Expression whose value may be a container of receiver elements.
 
 @returns receiver expression, or sentinel when nothing proves the relation.
 
 @example
 ```ts
 containerElementReceiver({ project, checker, node: elementAccess.expression });
 ```
 */
export function containerElementReceiver({
  project,
  checker,
  node,
}: {
  readonly project: Project;
  readonly checker: Checker;
  readonly node: Node;
},): Expression | typeof NOT_A_RECEIVER_CONTAINER {
  /**
   Declarations already visited, and the only thing that ends this walk.
   
   Each step either answers or moves to a declaration initializer, a file holds finitely
   many of those, and no node is examined twice, so the loop terminates on the set alone.
   
   A hop count sat beside it and has been removed. Its stated reason was cost, saving a
   declaration resolution per hop on chains it said do not appear in the corpus, and its
   stated consequence was that stopping early "only withholds an origin". That is backwards
   for this rule. Withholding an origin withholds a charge, every consumer of these origins
   only ever adds one, and a parameter with no charge is a parameter offered read-only.
   
   Measured at the count's threshold, chaining a container call through local aliases and
   writing through an element of the last: at three aliases the write is attributed to the
   parameter, and from seven onward the parameter comes back with no mutation and no
   opacity at all. Not an unattributed write but a clean parameter, which is the state a
   read-only offer is minted from, on a callable that rewrites a row the caller owns.
   */
  const visited = new Set<Node>();
  /**
   Next declaration to examine, holding one node at a time.
   
   A one-element queue rather than a cursor, matching `expressionElementOrigins`, because
   the loop is now ended by the visited set rather than by a counter and a queue says that
   in the condition. This walk follows a single chain, so the queue never holds two.
   */
  const pending: Node[] = [node,];
  while (pending.length > 0) {
    /**
     Declaration being examined for a container relation.
     */
    const current = pending.pop();
    if ((current === undefined) || visited.has(current,))
      return NOT_A_RECEIVER_CONTAINER;
    visited.add(current,);
    if (isCallExpression(current,)) {
      /**
       Receiver this call's container result may hold the elements of.
       */
      const receiver = callResultElementReceiver({
        project,
        checker,
        call: current,
      },);
      return (receiver === RESULT_NOT_RECEIVER_STATE)
        ? NOT_A_RECEIVER_CONTAINER
        : receiver;
    }
    /**
     Value this name was declared with, when it names one local declaration.
     */
    const initializer = bindingDeclarationInitializer({
      project,
      checker,
      node: current,
    },);
    if (initializer === NO_BINDING_INITIALIZER)
      return NOT_A_RECEIVER_CONTAINER;
    pending.push(initializer,);
  }
  return NOT_A_RECEIVER_CONTAINER;
}
