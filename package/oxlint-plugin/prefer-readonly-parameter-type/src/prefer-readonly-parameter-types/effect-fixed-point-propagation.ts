/**
 Fixed-point propagation over direct effect summaries.
 
 @module
 */

import { addOpaqueEffect, } from './effect-call-resolution.ts';
import { propagateCallbackRelations, } from './effect-callback-relation.ts';
import { EffectPropagationError, } from './effect-propagation-error.ts';
import { propagateElementApplications, } from './effect-element-application.ts';
import { propagateInvokedCapabilities, } from './effect-invoked-capability.ts';
import { propagateCalleeIndexes, } from './effect-propagation-indexes.ts';
import {
  propagateResultApplications,
  seedReturnedSlots,
} from './effect-result-substitution.ts';
import {
  addEffectSlot,
  type MutableEffectSummary,
} from './effect-summary-model.ts';
import {
  markCapturedCapabilityUnresolved,
  propagateCapturedCapability,
} from './effect-captured-capability.ts';
import { propagateUncertaintyProvenance, } from './effect-uncertainty-provenance.ts';

/**
 Mutable effect dimensions propagated per parameter.
 
 Four rather than three since result substitution landed: the propagated `returned` set
 grows per slot exactly as `mutated`, `invoked` and `opaque` do. The count bounds the
 loop and the loop THROWS at the bound, so adding a dimension without raising this turns
 a slow convergence into a thrown `EffectPropagationError`.
 */
const EFFECT_DIMENSION_COUNT = 4;

/**
 Propagates direct, transitive, recursive, and higher-order effects to fixed point.
 
 @param summaries - Mutable summaries keyed by declaration.
 
 @mutates summaries - Propagates call effects to fixed point.
 
 @example
 ```ts
 propagateEffects(summaries);
 ```
 */
export function propagateEffects(
  summaries: ReadonlyMap<string, MutableEffectSummary>,
): void {
  /* The bound counts the mutable effect bits, and those are not the only thing a pass can
   * report as progress: callback relations, element applications and uncertainty
   * provenance also set `changed` while contributing no counted bit. So the bound is a
   * runaway guard rather than a proof of convergence, and exhausting it means a summary
   * was still growing when the loop stopped. That summary is missing effects, which is
   * the direction that produces an offer for written state, so it is thrown rather than
   * returned. */
  /**
   Total effect bits that can change before fixed point.
   */
  const effectBitCount = [...summaries.values(),].reduce(
    function total(
      totalCount,
      summary,
    ): number {
      return totalCount + (summary.slots
        .slotCount
        * EFFECT_DIMENSION_COUNT);
    },
    0,
  );
  /**
   Mutable convergence state shared across each propagation pass.
   */
  const state = {
    changed: true,
    pass: 0,
  };
  while (state.changed && (state.pass <= effectBitCount)) {
    state.changed = false;
    state.pass++;
    summaries.forEach(
      /**
       Propagates every owned call edge from one caller summary.
       
       @param summary - Caller summary receiving transitive effects.
       
       @mutates summary - Adds callee and callback effects.
       */
      function propagateSummary(summary,): void {
        // Read-only view members carry no summary, so their element flow
        // propagates per summary instead of along a call edge.
        state.changed = propagateElementApplications({
          summaries,
          summary,
        },) || state.changed;
        /* Returned state is seeded before it is substituted, so a callable that returns
         * its own parameter is available to a caller returning that call's result on the
         * same pass rather than the next one. Both run inside the loop rather than once
         * ahead of it, because a callee's returned set can still grow through its own
         * substitutions. */
        state.changed = seedReturnedSlots({ summary, },) || state.changed;
        state.changed = propagateResultApplications({
          summaries,
          summary,
        },) || state.changed;
        summary.calls
          .forEach(function propagateCall(edge,): void {
            /**
             Summary for owned callee edge.
             */
            const calleeSummary = summaries.get(edge.calleeKey,);
            if (calleeSummary === undefined) {
              /* An owned edge naming a callee with no summary used to return silently,
               * which turned an unresolved callee into a no-effect one: the strongest
               * possible claim from the weakest possible evidence. Every origin the edge
               * packages takes opacity instead. A summary goes missing when its callable
               * could not be built at all, which `effect-demand-index.ts` warns about and
               * omits so one failure does not cost a whole file. */
              edge.originsByCalleeSlot
                .forEach(function markUnresolvedSlot(origins,): void {
                  origins.forEach(function markOrigin(origin,): void {
                    state.changed = addEffectSlot({
                      target: summary.opaque,
                      value: origin,
                    },) || state.changed;
                    addOpaqueEffect({
                      summary,
                      affectedSlot: origin,
                      provenance: `callable without an effect summary ${edge.calleeKey}`,
                    },);
                  },);
                },);
              /* A capture is in the same position as an ordinary origin here and was not
               * covered by the loop above, which reads `originsByCalleeSlot` alone. Without
               * this, an owned edge whose callee summary failed to build kept exactly the
               * false offer this channel exists to remove. */
              state.changed = markCapturedCapabilityUnresolved({
                summary,
                edge,
              },) || state.changed;
              return;
            }
            state.changed = propagateInvokedCapabilities({
              summaries,
              summary,
              calleeSummary,
              edge,
            },) || state.changed;
            /* Mutation propagates for every affected callee index, including one the
             * callee also invokes. Subtracting the invoked set here cancelled real
             * writes, because one destructured object parameter gives every binding it
             * introduces the same index, so a callee calling one property and writing
             * another carried both facts on index zero. Order mattered too: whichever
             * of the two passes ran first decided the answer. */
            state.changed = propagateCalleeIndexes({
              target: summary.mutated,
              edge,
              calleeOwnership: calleeSummary.slots,
              calleeIndexes: calleeSummary.mutated,
            },) || state.changed;
            state.changed = propagateCalleeIndexes({
              target: summary.opaque,
              edge,
              calleeOwnership: calleeSummary.slots,
              calleeIndexes: calleeSummary.opaque,
            },) || state.changed;
            state.changed = propagateUncertaintyProvenance({
              summary,
              calleeSummary,
              edge,
              calleeIndexes: calleeSummary.opaque,
            },) || state.changed;
            /* Runs beside the ordinary opacity propagation rather than inside it, because it
             * reads a different edge fact and answers a different question: not what the
             * callee did to a value it received, but what a callable the caller wrote can
             * reach once the callee could not account for it. */
            state.changed = propagateCapturedCapability({
              summary,
              calleeSummary,
              edge,
            },) || state.changed;
            state.changed = propagateCallbackRelations({
              summaries,
              summary,
              calleeSummary,
              edge,
            },) || state.changed;
          },);
      },);
  }
  if (state.changed)
    throw new EffectPropagationError({
      passCount: state.pass,
      effectBitCount,
      summaryCount: summaries.size,
    },);
}
