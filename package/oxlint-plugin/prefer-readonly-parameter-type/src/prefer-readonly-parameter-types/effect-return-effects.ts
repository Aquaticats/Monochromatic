/**
 What one return statement records, and the policy that decides it.
 
 Returning parameter-reachable state is not itself an effect: the caller already holds the
 parameter, so handing back a piece of it grants no capability the caller lacked. Recording
 which parameters a result can carry is what lets a caller keep tracking that value, and that
 tracking is the condition under which treating a return as benign stays sound. Until callers
 substitute through this fact, no receiver opacity may be discharged on the strength of it.
 `doc/decision/prefer-readonly-result-provenance.md` records the policy.
 
 Split out of `direct-effect-summary.ts` for its line budget, which the returned-callable
 capture pushed over. Nothing about the behaviour moved with it.
 
 @module
 */

import type { Node, } from 'typescript/ast';
import type {
  Checker,
  Project,
} from 'typescript/sync';

import { expressionOrigins, } from './effect-binding-origins.ts';
import { expressionElementOrigins, } from './effect-element-origin.ts';
import { expressionCanCarryMutableState, } from './effect-primitive-origin.ts';
import { targetResultSites, } from './effect-result-binding.ts';
import { recordResultApplicationSites, } from './effect-result-substitution.ts';
import { recordReturnedCallableCapture, } from './effect-returned-callable.ts';
import {
  addEffectSlots,
  isEffectCallableDeclaration,
  type MutableEffectSummary,
  type SlotOrigins,
} from './effect-summary-model.ts';

/**
 Records everything one returned expression contributes.
 
 @param project - TypeScript project resolving origins.
 
 @param checker - Checker deciding whether a returned value can carry state.
 
 @param bindingOriginBySymbolId - Parameter and alias origins of this callable.
 
 @param resultSitesBySymbolId - Call sites each local binding can hold a result of.
 
 @param summary - Summary receiving returned origins, captures and deferred uses.
 
 @param returned - Expression handed back by one return statement.
 
 @mutates summary - Adds returned origins, captured opacity and a deferred result use.
 
 @example
 ```ts
 recordReturnEffects({ project, checker, bindingOriginBySymbolId, summary, returned });
 ```
 */
export function recordReturnEffects({
  project,
  checker,
  bindingOriginBySymbolId,
  resultSitesBySymbolId,
  summary,
  returned,
}: {
  readonly project: Project;
  readonly checker: Checker;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly resultSitesBySymbolId: ReadonlyMap<number, ReadonlySet<string>>;
  readonly summary: MutableEffectSummary;
  readonly returned: Node;
},): void {
      /* Returning a callable is not returning a value the caller can track. The accepted
   * decision permits returning parameter-reachable state on one stated condition, that
   * callers keep tracking it through the recorded returned origins, and
   * `expressionOrigins` has no provenance successors for a function expression, so
   * `return (): Row => config.row` records no returned origin and no caller can
   * substitute through it. The condition fails rather than the policy applying.
   *
   * Falsified rather than argued: `ReadonlyDeep` applied, type-checked clean beside a
   * control whose direct write was rejected, and the driver changed the caller's row
   * through the returned closure.
   *
   * Opacity rather than a returned origin, which was the tempting reuse. A returned
   * origin asserts a caller can reach these parameters through this result, and what a
   * returned closure carries is the capability to reach them by invoking it.
   * `packagedCallableOrigins` over-approximates, scanning nested callable bodies, and
   * an over-approximation is safe on a channel that withholds and unsafe on one that
   * claims. Nothing today discharges on a returned set, so this would not break yet;
   * it would state the wrong relation and break whenever something did. */
  recordReturnedCallableCapture({
    project,
    bindingOriginBySymbolId,
    summary,
    returned,
  },);
  /* Only a returned value that can carry state records anything. A returned
   * primitive derived from a parameter grants the caller nothing: measured on
   * `readOnlyLookupEffect`, which returns `(facts.get(key) ?? new Set()).size`,
   * where the resolver correctly reaches `facts` through the property access and
   * the `??`, and recording that as a returned origin would claim the caller can
   * reach the map through a `number`. */
  if (expressionCanCarryMutableState({
    checker,
    node: returned,
  },)) {
    addEffectSlots({
      target: summary.directReturned,
      values: expressionOrigins({
        project,
        bindingOriginBySymbolId,
        node: returned,
      },),
    },);
    /* And the same question asked of the result's elements, because a returned container of
     * receiver elements carries the caller's own values while its own identity is fresh.
     * `expressionOrigins` answers for the value and correctly finds nothing on
     * `return rows.slice(0,)`: the array handed back is not `rows`. What the caller can reach
     * through it is every element `rows` held.
     *
     * Measured before this existed: `pickElement`, returning `rows.at(0,)`, recorded
     * `returned=[0]` and a caller writing through the result attributed the write, while
     * `pickContainer`, returning `rows.slice(0,)`, recorded nothing and the identical write
     * through the identical elements attributed nothing. One relation was consulted and its
     * sibling was not.
     *
     * This adds attribution rather than removing it, which is the safe direction here, and it
     * is the precondition the deferred discharge in
     * `doc/planning/prefer-readonly-return-substitution.md` actually needs: a returned
     * container has to be a fact a caller can propagate before any opacity is discharged on
     * the strength of it. */
    addEffectSlots({
      target: summary.directReturned,
      values: expressionElementOrigins({
        project,
        bindingOriginBySymbolId,
        node: returned,
      },),
    },);
    /* Returning another callable's result carries whatever that result carries, and
     * the resolver above cannot see it: a callee's summary does not exist while its
     * callers are scanned. Without this, `b` returning `a(x,)` records no returned
     * origin at all, so no caller of `b` can substitute through it either. */
    /* Asked of the binding record rather than of the expression alone, which every write and
     * store site has done since the deferred relation existed while this one had not. So
     * `return [firstRow(config,),][0] as Row` named no site: a call underlies neither the
     * element access nor the array literal, it underlies a member of the literal, and only the
     * widened walk looks there. Falsified. */
    recordResultApplicationSites({
      summary,
      sites: targetResultSites({
        project,
        resultSitesBySymbolId,
        node: returned,
      },),
      kind: 'returned',
    },);
  }
}
/**
 Tests whether one return statement belongs to the callable being summarised.
 
 A nested callable's body is scanned inline, so every return inside an active nested callable
 reaches the same branch as the enclosing callable's own returns. Without this, a callable
 returning nothing claimed a returned origin from a nested body, which is a positive capability
 claim about a result that does not exist.
 
 @param node - Return statement found in the selected body nodes.
 
 @param body - Body of the callable being summarised.
 
 @returns whether no nested callable sits between the return and that body.
 
 @example
 ```ts
 returnBelongsToCallable({ node, body });
 ```
 */
export function returnBelongsToCallable({
  node,
  body,
}: {
  readonly node: Node;
  readonly body: Node;
},): boolean {
  /**
   Cursor ascending from the return toward the body.
   */
  const cursor: { current: Node; } = { current: node, };
  while (cursor.current !== body) {
    if (isEffectCallableDeclaration(cursor.current,))
      return false;
    if (cursor.current
      .parent
      === cursor.current)
      return true;
    cursor.current = cursor.current
      .parent;
  }
  return true;
}
