/**
 Owned callback-argument effect propagation.
 
 @module
 */

import { addOpaqueEffect, } from './effect-call-resolution.ts';
import { asParameterIndex, } from './effect-slot-identity.ts';
import {
  calleeSlotOrigins,
  parameterCarriesSlot,
  provenanceOfParameter,
} from './effect-slot-projection.ts';
import {
  addEffectSlot,
  type CallEdge,
  type MutableEffectSummary,
  OWNED_CALLABLE_UNAVAILABLE,
} from './effect-summary-model.ts';
import { addUncertaintyProvenance, } from './effect-uncertainty-provenance.ts';

/**
 Propagates callback argument effects through one owned call edge.
 
 @param summaries - Owned callable summaries by declaration key.
 
 @param summary - Caller summary receiving callback effects.
 
 @param calleeSummary - Callee summary defining callback relations.
 
 @param edge - Caller-to-callee argument edge.
 
 @returns whether caller summary changed.
 
 @mutates summary - Adds callback mutation and uncertainty effects.
 
 @example
 ```ts
 propagateCallbackRelations({ summaries, summary, calleeSummary, edge });
 ```
 */
export function propagateCallbackRelations({
  summaries,
  summary,
  calleeSummary,
  edge,
}: {
  readonly summaries: ReadonlyMap<string, MutableEffectSummary>;
  readonly summary: MutableEffectSummary;
  readonly calleeSummary: MutableEffectSummary;
  readonly edge: CallEdge;
},): boolean {
  /**
   Whether any callback effect changed caller summary.
   */
  let changed = false;
  for (const relation of calleeSummary.relations) {
    /**
     Caller parameters packaged as callback source value.
     */
    const sourceCallerIndexes = calleeSlotOrigins({
      edge,
      ownership: calleeSummary.slots,
      slot: relation.sourceSlot,
    },);
    /**
     Callback declaration key passed to callback parameter.
     */
    const callbackKey = edge.callbackKeysByCalleeSlot[relation.callbackSlot];
    /**
     Summary for passed callback declaration, absent when none is named or built.
     */
    const callbackSummary = ((callbackKey === undefined)
      || (callbackKey === OWNED_CALLABLE_UNAVAILABLE))
      ? undefined
      : summaries.get(callbackKey,);
    if (callbackSummary === undefined) {
      /* The callee proved it invokes this parameter with that source, and the edge cannot say
       * which body runs. Skipping used to answer "no effect", which is the strongest possible
       * claim from the weakest possible evidence, and it is exactly what a caller packaging a
       * callback and its argument into one destructured parameter hits: the actual at that
       * position is an object literal, so resolving a callable there finds none.
       * `packagedCallbackInvocation` in the slot-narrowing fixture measured the result as no
       * written parameter at all while the packaged callback writes what it is handed.
       *
       * Every origin the source packages takes opacity instead, which is what the rule says
       * everywhere else it cannot inspect a call. */
      for (const sourceCallerIndex of sourceCallerIndexes) {
        changed = addEffectSlot({
          target: summary.opaque,
          value: sourceCallerIndex,
        },) || changed;
        addOpaqueEffect({
          summary,
          affectedSlot: sourceCallerIndex,
          provenance: `callback supplied to ${edge.calleeKey} that this rule cannot name`,
        },);
      }
      continue;
    }
    /* The relation names a syntactic argument position of the inner invocation, which is a
     * parameter position of the callback, not a slot of it. A callback that destructures
     * that parameter records its writes against property slots, so asking its slot sets
     * directly would answer `false` and drop the write. Projecting to parameters first keeps
     * the answer sound; carrying property precision through a callback would need the
     * relation to name a slot on both sides, which it does not yet. */
    /**
     Callback parameter this relation's argument position fills.
     */
    const callbackParameter = asParameterIndex(relation.callbackArgumentPosition,);
    /**
     Whether callback argument carries proven mutation.
     */
    const callbackArgumentMutated = parameterCarriesSlot({
      ownership: callbackSummary.slots,
      slots: callbackSummary.mutated,
      parameterIndex: callbackParameter,
    },);
    /**
     Whether callback argument carries unresolved uncertainty.
     */
    const callbackArgumentOpaque = parameterCarriesSlot({
      ownership: callbackSummary.slots,
      slots: callbackSummary.opaque,
      parameterIndex: callbackParameter,
    },);
    for (const sourceCallerIndex of sourceCallerIndexes) {
      /**
       Whether mutation propagation changed caller summary.
       */
      const mutationChanged = callbackArgumentMutated
        && addEffectSlot({
          target: summary.mutated,
          value: sourceCallerIndex,
        },);
      /**
       Whether opaque propagation changed caller summary.
       */
      const opaqueChanged = callbackArgumentOpaque
        && addEffectSlot({
          target: summary.opaque,
          value: sourceCallerIndex,
        },);
      /**
       Whether callback uncertainty provenance changed caller summary.
       */
      const provenanceChanged = callbackArgumentOpaque
        && addUncertaintyProvenance({
          target: summary.opaqueProvenanceBySlot,
          affectedSlot: sourceCallerIndex,
          provenanceFacts: provenanceOfParameter({
            ownership: callbackSummary.slots,
            provenanceBySlot: callbackSummary.opaqueProvenanceBySlot,
            parameterIndex: callbackParameter,
          },),
        },);
      changed = mutationChanged
        || opaqueChanged
        || provenanceChanged
        || changed;
    }
  }
  return changed;
}
