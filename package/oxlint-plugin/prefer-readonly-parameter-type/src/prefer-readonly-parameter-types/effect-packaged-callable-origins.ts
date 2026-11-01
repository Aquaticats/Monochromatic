/**
 Caller origins a callable packaged inside a call-argument literal can hand over.
 
 The argument walk reads the value each property holds. When that value is a callable,
 or when the property is an accessor or a method and has no readable value at all, the
 parameter it reaches lives in a body the callee runs, not in anything the walk can see.
 Three measured forms in the result-provenance fixture produced the same defect, each
 recording no origin while the callee wrote through what came back:
 `accessorPackagedEffect` uses `get unnamed() { return row; }`,
 `methodReturnPackagedEffect` uses a method, and `arrowReturnPackagedEffect` uses an
 ordinary property holding an arrow. Each was offered `row` as readonly.
 
 This covers callables nested inside a packaged literal, which is exactly where the
 callback relation has no reach: `callbackKeys` is recorded per argument position, so a
 callable handed over directly is resolved and analyzed as a callback, while one wrapped
 in a literal is invisible to it.
 
 The body is scanned rather than evaluated, and every binding it names contributes,
 whatever position it appears in. That over-approximates, which is the direction that
 costs a withheld offer rather than a wrong one, and it needs no claim about which of
 several returns runs.
 
 @module
 */

import type { Node, } from 'typescript/ast';
import {
  isIdentifier,
  isShorthandPropertyAssignment,
  isTypeNode,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import { expressionCanCarryMutableState, } from './effect-primitive-origin.ts';
import type { EffectSlot, } from './effect-slot-identity.ts';
import {
  collectAstNodes,
  NO_SLOT_ORIGIN,
  type SlotOrigins,
} from './effect-summary-model.ts';

/**
 Collects caller parameter origins any binding named inside a packaged callable carries.
 
 @param project - TypeScript project resolving binding symbols.
 
 @param bindingOriginBySymbolId - Current callable parameter and alias origins.
 
 @param packaged - Callable, accessor or method authored inside a call-argument literal.
 
 @returns parameter indexes reachable through that body.
 
 @example
 ```ts
 packagedCallableOrigins({ project, bindingOriginBySymbolId, packaged });
 ```
 */
export function packagedCallableOrigins({
  project,
  bindingOriginBySymbolId,
  packaged,
}: {
  readonly project: Project;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly packaged: Node;
},): ReadonlySet<EffectSlot> {
  /**
   Checker resolving each named binding to its declaring symbol.
   */
  const { checker, } = project;
  /**
   Origins any binding inside this accessor can carry.
   */
  const origins = new Set<EffectSlot>();
  collectAstNodes(packaged,)
    .forEach(function collectNamed(node,): void {
      if (!isIdentifier(node,))
        return;
      /* A name in type position packages no value, so asking the checker about it is
       * both pointless and a route into paths this scan has no business on: a workspace
       * sweep over an earlier revision that did ask ended in
       * `panic: interface conversion: checker.TypeData is *checker.TypeReference, not
       * *checker.TupleType`, which aborts the whole run rather than one file. */
      if (enclosedByTypeNode({
        node,
        packaged,
      },))
        return;
      /**
       Symbol this occurrence resolves to.
       
       A shorthand property's name resolves to the property, not to the local it reads,
       so the value symbol has to be asked for separately. `parameterIndexes` already
       does this for shorthand properties it walks directly; without it here, a
       parameter named only as `{ row }` inside an accessor body contributes nothing.
       `accessorShorthandEffect` in the call-edge fixture measured that.
       */
      const symbol = isShorthandPropertyAssignment(node.parent,)
        && (node.parent
          .name
          === node)
        ? checker.getShorthandAssignmentValueSymbol(node.parent,)
        : checker.getSymbolAtLocation(node,);
      if (symbol === undefined)
        return;
      /**
       Caller parameters this binding can hold.
       */
      const named = bindingOriginBySymbolId.get(symbol.id,) ?? NO_SLOT_ORIGIN;
      if (named.size === 0)
        return;
      /* A binding that cannot carry mutable state grants the callee nothing, and
       * including it would name primitives in the resulting diagnostic. */
      if (!expressionCanCarryMutableState({
        checker,
        node,
      },))
        return;
      named.forEach(function collectOrigin(origin,): void {
        origins.add(origin,);
      },);
    },);
  return origins;
}

/**
 Tests whether a name sits inside a type annotation rather than in value position.
 
 @param node - Name being classified.
 
 @param packaged - Outer declaration bounding the ascent.
 
 @returns whether a type node encloses this name.
 
 @example
 ```ts
 enclosedByTypeNode({ node, packaged });
 ```
 */
function enclosedByTypeNode({
  node,
  packaged,
}: {
  readonly node: Node;
  readonly packaged: Node;
},): boolean {
  /**
   Cursor ascending toward the packaged declaration.
   */
  const cursor: { current: Node; } = { current: node, };
  while (cursor.current !== packaged) {
    if (isTypeNode(cursor.current,))
      return true;
    if (cursor.current
      .parent
      === cursor.current)
      return false;
    cursor.current = cursor.current
      .parent;
  }
  return false;
}
