/**
 Which actual argument positions each formal parameter of a call can receive.
 
 A call edge is read by formal parameter index, while a call's arguments are a syntactic
 list. Those two agree only for a plain positional call against a callee whose formals
 are all ordinary value parameters, and three shapes break the correspondence: an
 explicit `this` formal takes an index while receiving nothing, a rest formal receives
 every remaining actual, and a spread actual feeds several formals from one position.
 Every one of them was measured producing a `readonly` offer for a written parameter;
 `doc/planning/prefer-readonly-call-edge-shapes.md` records the cases.
 
 The mapping is deliberately many-to-many and over-approximating. A formal that could
 receive any of several actuals lists all of them, because missing an origin is what
 makes the rule offer readonly for state something writes, while an extra origin only
 withholds an offer.
 
 @module
 */

import type {
  CallExpression,
  Node,
} from 'typescript/ast';
import {
  isIdentifier,
  isSpreadElement,
} from 'typescript/ast/is';

import type { EffectCallableDeclaration, } from './effect-summary-model.ts';

/**
 Actual argument positions reachable by each formal, indexed by formal position.
 */
export type FormalActualPositions = readonly (readonly number[])[];

/**
 Maps every formal parameter of one call to the actual positions it can receive.
 
 @param callee - Callable whose formals are being filled.
 
 @param call - Call supplying actual arguments.
 
 @returns actual positions per formal, empty for a formal receiving nothing.
 
 @example
 ```ts
 formalActualPositions({ callee, call });
 ```
 */
export function formalActualPositions({
  callee,
  call,
}: {
  readonly callee: EffectCallableDeclaration;
  readonly call: CallExpression;
},): FormalActualPositions {
  /**
   Formals that receive no argument because they precede the first value parameter.
   */
  const skippedFormals = calleeHasThisParameter({ callee, },) ? 1 : 0;
  /**
   Count of actual arguments written at the call.
   */
  const actualCount = call.arguments
    .length;
  /**
   First spread position, or the actual count when the call spreads nothing.
   
   Past a spread the positional correspondence is gone: one syntactic argument supplies
   an unknown number of formals, so every formal from there on may receive anything from
   the spread onward.
   */
  const firstSpread = call.arguments
    .findIndex(function spreadArgument(argument,): boolean {
      return isSpreadElement(argument,);
    },);
  /**
   Spread boundary normalized so absence compares as past every position.
   */
  const spreadBoundary = firstSpread === (-1) ? actualCount : firstSpread;
  return callee.parameters
    .map(function positionsForFormal(
      parameter,
      formalIndex,
    ): readonly number[] {
      if (formalIndex < skippedFormals)
        return [];
      /**
       Position this formal would occupy in a plain positional call.
       */
      const actualIndex = formalIndex - skippedFormals;
      if (parameter.dotDotDotToken !== undefined)
        /* A rest formal collects everything from its own position onward. */
        return actualPositionsFrom({
          start: actualIndex,
          actualCount,
        },);
      if (actualIndex >= spreadBoundary)
        /* At or past the spread, so this formal may be filled by the spread or by any
         * actual after it. */
        return actualPositionsFrom({
          start: spreadBoundary,
          actualCount,
        },);
      if (actualIndex >= actualCount)
        /* Omitted at this call site. A default initializer may still give this formal a
         * value, which is tracked separately from the argument mapping. */
        return [];
      return [actualIndex,];
    },);
}

/**
 Lists every actual position from one index onward.
 
 @param start - First actual position included.
 
 @param actualCount - Count of actual arguments at the call.
 
 @returns ascending actual positions.
 
 @example
 ```ts
 actualPositionsFrom({ start: 1, actualCount: 3 });
 ```
 */
function actualPositionsFrom({
  start,
  actualCount,
}: {
  readonly start: number;
  readonly actualCount: number;
},): readonly number[] {
  if (start >= actualCount)
    return [];
  return Array.from(
    { length: actualCount - start, },
    function positionAt(
      _unused,
      offset,
    ): number {
      return start + offset;
    },
  );
}

/**
 Tests whether a callee declares an explicit `this` parameter.
 
 @param callee - Callable declaration whose formals are inspected.
 
 @returns whether formal index zero receives no argument.
 
 @example
 ```ts
 calleeHasThisParameter({ callee });
 ```
 */
export function calleeHasThisParameter({
  callee,
}: {
  readonly callee: EffectCallableDeclaration;
},): boolean {
  /**
   First declared formal, absent for a callable taking nothing.
   */
  const first = callee.parameters[0]
    ?.name;
  if (first === undefined)
    return false;
  return isIdentifier(first,)
    && (first.getText() === 'this');
}
