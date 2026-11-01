/**
 The receiver of a member call, however the member was named.
 
 One definition shared by every consumer, because three of them independently tested
 for a property-access callee and a call named any other way fell through all three
 at once. `values['push']('appended')` reached neither the collection handling nor
 the opaque boundary, so nothing recorded its mutation and the parameter was offered
 `readonly`; applying that suggestion failed with
 `error TS7015: Element implicitly has an 'any' type because index expression is not
 of type 'number'`.
 
 @module
 */

import type {
  CallExpression,
  Expression,
  Node,
} from 'typescript/ast';
import {
  isElementAccessExpression,
  isNonNullExpression,
  isParenthesizedExpression,
  isPropertyAccessExpression,
} from 'typescript/ast/is';

/**
 Sentinel when a call names no receiver, so there is no member call at all.
 
 A bare `identifier()` or an immediately invoked expression has no receiver, which is
 a different fact from a receiver this cannot resolve.
 */
export const NO_MEMBER_RECEIVER: unique symbol = Symbol(
  'call names no member receiver',
);

/**
 Strips callee wrappers that change nothing about which member is called.
 
 `(facts.get)(key)` and `facts.get!(key)` select the same member as `facts.get(key)`,
 so a callee test that rejects them rejects calls the engine treats identically.
 
 @param callee - Called expression, possibly wrapped.
 
 @returns innermost callee expression.
 
 @example
 ```ts
 unwrappedCallee({ callee: call.expression });
 ```
 */
function unwrappedCallee({ callee, }: { readonly callee: Node; },): Node {
  /**
   Cursor descending through runtime-transparent callee wrappers.
   */
  const cursor: { current: Node; } = { current: callee, };
  while (isParenthesizedExpression(cursor.current,)
    || isNonNullExpression(cursor.current,)) {
    cursor.current = cursor.current
      .expression;
  }
  return cursor.current;
}

/**
 Resolves the expression a member call was made on.
 
 Accepts property and element access alike. Which of the two the author wrote is a
 syntax choice with no runtime difference in what receives the call, so making it
 decide whether an effect is recorded is a defect rather than a policy.
 
 This deliberately does not check what the member is or whether it resolves to
 anything: callers still have to run overload resolution and consult their own
 authority. Its only job is to answer which expression the call was made on.
 
 @param call - Call expression whose receiver is wanted.
 
 @returns receiver expression, or sentinel when the call names none.
 
 @example
 ```ts
 memberCallReceiver({ call });
 ```
 */
export function memberCallReceiver(
  { call, }: { readonly call: CallExpression; },
): Expression | typeof NO_MEMBER_RECEIVER {
  /**
   Callee after runtime-transparent wrappers are removed.
   */
  const callee = unwrappedCallee({ callee: call.expression, },);
  return isPropertyAccessExpression(callee,) || isElementAccessExpression(callee,)
    ? callee.expression
    : NO_MEMBER_RECEIVER;
}
