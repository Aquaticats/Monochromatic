/**
 Caller origins a returned callable can reach once whoever received it invokes.
 
 Returning parameter-reachable state is permitted by `doc/decision/prefer-readonly-result-provenance.md`
 on one stated condition: that callers keep tracking the value through the recorded returned
 origins. `return config.row` satisfies it. `return (): Row => config.row` does not, because
 `expressionOrigins` has no provenance successors for a function expression, so nothing is
 recorded and no caller can substitute through it. The precondition fails rather than the
 policy applying, which is what makes this a false offer instead of a permitted return.
 
 Opacity rather than a returned origin. A returned origin asserts that a caller can reach
 these parameters through this result; what a returned callable carries is the capability to
 reach them by invoking it. `packagedCallableOrigins` over-approximates, scanning the whole
 subtree including nested callable bodies, and an over-approximation is safe on a channel
 that withholds and unsafe on one that claims. Nothing today discharges on the strength of a
 returned set, so the reuse would not break yet; it would state the wrong relation and break
 as soon as something did.
 
 Silent, like every other retention. There is no call to name and no boundary a reader could
 inspect, so the provenance goes through the retention vocabulary and the offer is simply
 withheld.
 
 @module
 */

import type { Node, } from 'typescript/ast';
import type { Project, } from 'typescript/sync';

import {
  addOpaqueEffect,
  callableDeclaration,
} from './effect-call-resolution.ts';
import { effectOriginLocation, } from './effect-origin-location.ts';
import { transitiveCallableOrigins, } from './effect-callable-capture-closure.ts';
import { reachableValueSources, } from './effect-result-reach.ts';
import { returnedCallableProvenance, } from './effect-retention-provenance.ts';
import {
  type EffectCallableDeclaration,
  type MutableEffectSummary,
  OWNED_CALLABLE_UNAVAILABLE,
  type SlotOrigins,
} from './effect-summary-model.ts';

/**
 Records opacity for every caller origin a returned callable captured.
 
 @param project - TypeScript project resolving the returned expression.
 
 @param bindingOriginBySymbolId - Parameter and alias origins of the returning callable.
 
 @param summary - Summary receiving opacity.
 
 @param returned - Expression handed back by one return statement.
 
 @mutates summary - Adds an opaque slot and retention provenance per captured origin.
 
 @example
 ```ts
 recordReturnedCallableCapture({ project, bindingOriginBySymbolId, summary, returned });
 ```
 */
export function recordReturnedCallableCapture({
  project,
  bindingOriginBySymbolId,
  summary,
  returned,
}: {
  readonly project: Project;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly summary: MutableEffectSummary;
  readonly returned: Node;
},): void {
  /**
   Callable the returned expression resolves to, absent when a value is returned instead.
   
   Resolved rather than tested syntactically, so returning a closure by name behaves like
   returning it inline, which is the same resolution the store path and the call edge use.
   */
  /* Asked of every source the returned value can have come from, not of the expression alone.
   * `return { next: () => ({ value: config.row, }), }` hands back a callable inside a literal, and
   * resolving the literal answers with no callable, so the capture went unrecorded and a caller
   * driving the iterator changed the caller's row. Falsified.
   *
   * `reachableValueSources` already descends an authored aggregate for the result-site walk, and
   * this is the same descent asking a different question of each answer. */
  /**
   Callables the returned value can be, or can be reached through.
   */
  const callables = reachableValueSources({
    project,
    node: returned,
  },)
    .flatMap(function resolveSource(source,): readonly EffectCallableDeclaration[] {
      /**
       Callable this source resolves to, absent when it is not one.
       */
      const candidate = callableDeclaration({
        project,
        node: source,
      },);
      return candidate === OWNED_CALLABLE_UNAVAILABLE ? [] : [candidate,];
    },);
  if (callables.length === 0)
    return;
  /**
   Where the return sits, so the fact points at the escape rather than at the callable.
   */
  const location = effectOriginLocation({ node: returned, },);
  /**
   Provenance naming the return as the escape, silent like every other retention.
   */
  const provenance = returnedCallableProvenance({ location, },);
  callables.forEach(function recordCallable(callable,): void {
    transitiveCallableOrigins({
      project,
      bindingOriginBySymbolId,
      packaged: callable,
    },)
      .forEach(function recordCapturedSlot(affectedSlot,): void {
        addOpaqueEffect({
          summary,
          affectedSlot,
          provenance,
        },);
      },);
  },);
}
