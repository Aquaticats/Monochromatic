/**
 Caller effects for a callable handed to a callee, decided by what the callee does with it.
 
 Two of the three things a callee can do live here: keep it, which the caller takes as opacity,
 and write through what invoking it produced, which the caller takes as a mutation. The third,
 handing back what invoking it produced, is a returned origin and belongs with the substitution
 walk in `effect-captured-result-origins.ts`.
 
 The admission gate for keeping is the callee's own uncertainty about the formal, not the reason
 for it.
 Gating on retention provenance was considered and rejected as unsound: absent retention
 provenance means call-caused or unknown rather than proven non-retaining, so a callee
 forwarding its callback to something this analysis cannot resolve would pass the gate while
 the unresolved callee kept the callable and invoked it later.
 
 ```ts
 function relay(callback: () => Row,): void {
   unresolved(callback,);
 }
 ```
 
 `relay` records call-caused opacity for its formal and no retention, and a caller handing it
 a capturing closure has to withhold all the same.
 
 The provenance the caller receives is the callee's own, copied unchanged, which is what
 makes the two cases read differently at the boundary without a second decision here. A
 capture reaching a callee that stored it arrives with store provenance and stays silent,
 because a reader cannot act on it. One reaching a callee that could not inspect its own
 callee arrives with that call named, which is exactly what the reader can act on.
 
 @module
 */

import { addUncertaintyProvenance, } from './effect-uncertainty-provenance.ts';
import {
  addEffectSlot,
  type CallEdge,
  type MutableEffectSummary,
} from './effect-summary-model.ts';
import type { EffectSlot, } from './effect-slot-identity.ts';

//region Shared empty values
// Provenance facts and origins carried by nothing, shared so an absent entry
// allocates no set.

/**
 Provenance facts carried by nothing, shared so an absent entry allocates no set.
 */
const NO_PROVENANCE_FACTS: ReadonlySet<string> = new Set<string>();

/**
 Origins carried by nothing, shared so a formal packaging no callable allocates none.
 */
const NO_CAPTURED_ORIGINS: readonly EffectSlot[] = [];

//endregion Shared empty values

//region Uncertain formals
// Marking caller origins opaque when a callable they were captured by reached
// a formal whose handling is unknown.

/**
 Marks caller origins opaque when a callable they were captured by reached an uncertain formal.
 
 @param summary - Caller summary receiving opacity.
 
 @param calleeSummary - Callee summary supplying uncertainty and its provenance.
 
 @param edge - Owned call edge carrying captures per formal.
 
 @mutates summary - Adds opaque slots and copies callee provenance onto them.
 
 @returns whether any caller fact grew.
 
 @example
 ```ts
 propagateCapturedCapability({ summary, calleeSummary, edge });
 ```
 */
export function propagateCapturedCapability({
  summary,
  calleeSummary,
  edge,
}: {
  readonly summary: MutableEffectSummary;
  readonly calleeSummary: MutableEffectSummary;
  readonly edge: CallEdge;
},): boolean {
  /**
   Whether any caller opacity, mutation or provenance fact was added.
   */
  const growth: { any: boolean; } = { any: false, };
  calleeSummary.opaque
    .forEach(function propagateSlot(calleeSlot,): void {
      /**
       Origins the callable filling this slot's formal captured.
       */
      const captured = capturesOfCalleeSlot({
        calleeSummary,
        edge,
        calleeSlot,
      },);
      if (captured.length === 0)
        return;
      /**
       Provenance the callee attached to its own uncertainty about this slot.
       */
      const provenanceFacts = calleeSummary.opaqueProvenanceBySlot
        .get(calleeSlot,)
        ?? NO_PROVENANCE_FACTS;
      captured.forEach(function markCaptured(origin,): void {
        if (addEffectSlot({
          target: summary.opaque,
          value: origin,
        },))
          growth.any = true;
        if (addUncertaintyProvenance({
          target: summary.opaqueProvenanceBySlot,
          affectedSlot: origin,
          provenanceFacts,
        },))
          growth.any = true;
      },);
    },);
  /* And the third thing a callee can do with a callable, beside keeping it and handing back what
   * it produced: write through what it produced. That reaches the caller's value exactly as the
   * other two do, and it was answered by nothing. Measured:
   *
   * ```ts
   * function writeThroughSupplied(written: () => Row,): void {
   *   written().label = 'written';
   * }
   *
   * function handInlineToWriter(inlineWritten: Config,): void {
   *   writeThroughSupplied((): Row => inlineWritten.row,);
   * }
   * ```
   *
   * `writeThroughSupplied` records `mutated=[0]` for its formal, so the callee had already said
   * what it does, and `handInlineToWriter` recorded nothing at all and was offered. Falsified: the
   * closure only reads, so the applied annotation type-checks, the callee's write is on the
   * declared `Row`, and the caller's row changes.
   *
   * Spoken as a mutation rather than as opacity, because that is what it is. A reader is told the
   * parameter is written rather than told an implementation could not be inspected, and #55
   * settled that direction for stores by the same argument. */
  calleeSummary.mutated
    .forEach(function propagateWrittenSlot(calleeSlot,): void {
      capturesOfCalleeSlot({
        calleeSummary,
        edge,
        calleeSlot,
      },)
        .forEach(function markWritten(origin,): void {
          if (addEffectSlot({
            target: summary.mutated,
            value: origin,
          },))
            growth.any = true;
        },);
    },);
  return growth.any;
}

//endregion Uncertain formals

//region Captured origins
// The caller origins captured by the callable filling one callee slot's formal.

/**
 Names the caller origins captured by the callable filling one callee slot's formal.
 
 A capture fills a whole formal rather than a property of one, so every slot a formal owns
 reports the same captures. Reading it per slot is what lets the callee's own per-slot facts
 decide which captures matter.
 
 @param calleeSummary - Callee summary naming which formal owns each slot.
 
 @param edge - Owned call edge carrying captures per formal.
 
 @param calleeSlot - Slot the callee recorded a fact against.
 
 @returns captures reaching that slot's formal.
 
 @example
 ```ts
 capturesOfCalleeSlot({ calleeSummary, edge, calleeSlot });
 ```
 */
function capturesOfCalleeSlot({
  calleeSummary,
  edge,
  calleeSlot,
}: {
  readonly calleeSummary: MutableEffectSummary;
  readonly edge: CallEdge;
  readonly calleeSlot: EffectSlot;
},): readonly EffectSlot[] {
  /**
   Formal owning this callee slot, since a capture fills a whole formal.
   */
  const owner = calleeSummary.slots
    .parameterOfSlot[calleeSlot];
  if (owner === undefined)
    return NO_CAPTURED_ORIGINS;
  return edge.capturedOriginsByFormal[owner] ?? NO_CAPTURED_ORIGINS;
}

//endregion Captured origins

//region Unresolved callees
// Every capture on an edge stays opaque when the callee has no summary at all.

/**
 Marks every capture on an edge opaque when the callee has no summary at all.
 
 The unresolved-callee branch already takes opacity for every ordinary origin an edge
 packages, on the reasoning that a callable which could not be built proves nothing. A
 capture is in exactly that position and was not covered, so an owned edge whose summary
 failed to build kept the same false offer this work removed everywhere else.
 
 @param summary - Caller summary receiving opacity.
 
 @param edge - Owned call edge whose callee has no summary.
 
 @mutates summary - Adds opaque slots and names the missing summary as their cause.
 
 @returns whether any caller fact grew.
 
 @example
 ```ts
 markCapturedCapabilityUnresolved({ summary, edge });
 ```
 */
export function markCapturedCapabilityUnresolved({
  summary,
  edge,
}: {
  readonly summary: MutableEffectSummary;
  readonly edge: CallEdge;
},): boolean {
  /**
   Whether any caller opacity or provenance fact was added.
   */
  const growth: { any: boolean; } = { any: false, };
  /**
   Cause named for every capture handed to a callable with no summary.
   */
  const provenanceFacts: ReadonlySet<string> = new Set([
    `callable without an effect summary ${edge.calleeKey}`,
  ],);
  edge.capturedOriginsByFormal
    .forEach(function markFormal(captured,): void {
      captured.forEach(function markCaptured(origin,): void {
        if (addEffectSlot({
          target: summary.opaque,
          value: origin,
        },))
          growth.any = true;
        if (addUncertaintyProvenance({
          target: summary.opaqueProvenanceBySlot,
          affectedSlot: origin,
          provenanceFacts,
        },))
          growth.any = true;
      },);
    },);
  return growth.any;
}

//endregion Unresolved callees
