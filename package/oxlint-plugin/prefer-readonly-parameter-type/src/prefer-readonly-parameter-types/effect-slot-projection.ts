/**
 Projection between slot-keyed effect facts and parameter-keyed ones.
 
 Effects are recorded against slots, while several consumers ask parameter-level questions:
 the rule reports on a parameter, foreign ownership marks a parameter, overload agreement
 compares two declarations whose slot tables have nothing to do with each other, and an
 external package's summary crosses a project boundary where only parameter positions mean
 the same thing on both sides.
 
 Projecting loses precision and never soundness, because a parameter answers for every slot
 it owns. The reverse direction, broadcasting a parameter-level fact to all of its slots, is
 what a caller does when it cannot decompose an actual, and that one is load-bearing:
 propagation reads an edge by the callee's effect slot and never consults the whole
 parameter, so a fact left only on the whole slot is a fact no callee lookup will find.
 
 @module
 */

import type {
  EffectSlot,
  ParameterIndex,
} from './effect-slot-identity.ts';
import type {
  CallEdge,
  SlotOwnership,
} from './effect-summary-model.ts';

//region Projecting origins onto slots
// The caller origins an edge supplies for one callee slot, never answering
// short.

/**
 Reads the caller origins an edge supplies for one callee slot, never answering short.
 
 The broadcast in `addOwnedCallEdge` fills every callee slot, so a freshly built edge always
 has an entry. A restored one need not: the persisted arrays are validated for internal
 agreement but nothing compares their length against the callee they name, and an edge
 truncated to its formals would answer nothing for a property slot. Answering nothing there
 discards a write the callee performs, which is the direction that offers `readonly` for
 mutated state, so a missing entry falls back to the owning parameter's origins instead.
 That is the answer this analysis gave before slots existed, so the failure mode is lost
 precision rather than lost soundness.
 
 @param edge - Call edge supplying caller origins.
 
 @param ownership - Slot ownership of the callee the edge names.
 
 @param slot - Callee slot carrying some effect.
 
 @returns caller origins reaching that slot, empty only when the formal has none.
 
 @example
 ```ts
 calleeSlotOrigins({ edge, ownership: calleeSummary.slots, slot });
 ```
 */
export function calleeSlotOrigins({
  edge,
  ownership,
  slot,
}: {
  readonly edge: CallEdge;
  readonly ownership: SlotOwnership;
  readonly slot: EffectSlot;
},): readonly EffectSlot[] {
  /* Absent and empty mean different things here, and conflating them undoes every narrowing at
   * once with nothing failing. An absent entry is an edge that never answered for this slot. An
   * empty one is a decomposed caller literal that fills this key nowhere, which is the answer
   * per-property attribution exists to produce. Widening the test to `supplied?.length` would
   * turn each of those into the owning parameter's full origins. */
  /**
   Origins recorded against this exact slot, absent for a truncated edge.
   */
  const supplied = edge.originsByCalleeSlot[slot];
  if (supplied !== undefined)
    return supplied;
  /**
   Parameter owning this slot, absent when the slot is outside the callee.
   */
  const owner = ownership.parameterOfSlot[slot];
  if (owner === undefined)
    return [];
  return edge.originsByCalleeSlot[owner] ?? [];
}

//endregion Projecting origins onto slots

//region Slot ownership queries
// The parameters owning a set of slots, and the slots a set of parameters owns.

/**
 Collects the parameters owning a set of slots.
 
 @param ownership - Slot ownership of the callable those slots belong to.
 
 @param slots - Slots carrying some effect.
 
 @returns parameters owning at least one of those slots.
 
 @example
 ```ts
 parametersOfSlots({ ownership: summary.slots, slots: summary.mutated });
 ```
 */
export function parametersOfSlots({
  ownership,
  slots,
}: {
  readonly ownership: SlotOwnership;
  readonly slots: ReadonlySet<EffectSlot>;
},): ReadonlySet<ParameterIndex> {
  /**
   Owning parameters accumulated across every affected slot.
   */
  const owners = new Set<ParameterIndex>();
  slots.forEach(function owner(slot,): void {
    /**
     Parameter owning this slot, absent when the slot is outside this callable.
     */
    const owned = ownership.parameterOfSlot[slot];
    if (owned !== undefined)
      owners.add(owned,);
  },);
  return owners;
}

/**
 Collects every slot owned by a set of parameters.
 
 @param ownership - Slot ownership of the callable those parameters belong to.
 
 @param parameters - Parameters whose slots are wanted.
 
 @returns whole-parameter slots and every property slot beneath them.
 
 @example
 ```ts
 slotsOfParameters({ ownership: summary.slots, parameters: foreign });
 ```
 */
export function slotsOfParameters({
  ownership,
  parameters,
}: {
  readonly ownership: SlotOwnership;
  readonly parameters: Iterable<ParameterIndex>;
},): ReadonlySet<EffectSlot> {
  /**
   Slots accumulated across every named parameter.
   */
  const owned = new Set<EffectSlot>();
  for (const parameterIndex of parameters)
    (ownership.slotsByParameter[parameterIndex] ?? [])
      .forEach(function collect(slot,): void {
        owned.add(slot,);
      },);
  return owned;
}

//endregion Slot ownership queries

//region Uncertainty and effect queries
// The uncertainty provenance recorded against every slot one parameter owns,
// and whether any of those slots carries an effect.

/**
 Unions the uncertainty provenance recorded against every slot one parameter owns.
 
 @param ownership - Slot ownership of the callable holding that provenance.
 
 @param provenanceBySlot - Uncertainty facts recorded per slot.
 
 @param parameterIndex - Parameter whose facts are wanted.
 
 @returns every fact recorded against that parameter or a property of it.
 
 @example
 ```ts
 provenanceOfParameter({ ownership, provenanceBySlot, parameterIndex });
 ```
 */
export function provenanceOfParameter({
  ownership,
  provenanceBySlot,
  parameterIndex,
}: {
  readonly ownership: SlotOwnership;
  readonly provenanceBySlot: ReadonlyMap<EffectSlot, ReadonlySet<string>>;
  readonly parameterIndex: ParameterIndex;
},): ReadonlySet<string> {
  /**
   Facts accumulated across every slot the parameter owns.
   */
  const facts = new Set<string>();
  (ownership.slotsByParameter[parameterIndex] ?? [])
    .forEach(function collect(slot,): void {
      (provenanceBySlot.get(slot,) ?? [])
        .forEach(function add(fact,): void {
          facts.add(fact,);
        },);
    },);
  return facts;
}

/**
 Tests whether any slot owned by one parameter carries an effect.
 
 @param ownership - Slot ownership of the callable being asked about.
 
 @param slots - Slots carrying some effect.
 
 @param parameterIndex - Parameter under test.
 
 @returns whether that parameter owns an affected slot.
 
 @example
 ```ts
 parameterCarriesSlot({ ownership, slots: summary.mutated, parameterIndex });
 ```
 */
export function parameterCarriesSlot({
  ownership,
  slots,
  parameterIndex,
}: {
  readonly ownership: SlotOwnership;
  readonly slots: ReadonlySet<EffectSlot>;
  readonly parameterIndex: ParameterIndex;
},): boolean {
  return (ownership.slotsByParameter[parameterIndex] ?? [])
    .some(function affected(slot,): boolean {
      return slots.has(slot,);
    },);
}
//endregion Uncertainty and effect queries
