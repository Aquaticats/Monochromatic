/**
 JSON representation for persistent direct effect summaries.
 
 @module
 */

import { slotsByParameterFrom, } from './effect-parameter-slots.ts';
import {
  asEffectSlot,
  asParameterIndex,
  type EffectSlot,
  type ParameterIndex,
} from './effect-slot-identity.ts';
import {
  type CallbackRelation,
  type CallEdge,
  type ElementApplication,
  type ResultApplication,
  type MutableEffectSummary,
  OWNED_CALLABLE_UNAVAILABLE,
  type SlotOwnership,
} from './effect-summary-model.ts';

/**
 Serialized unavailable callback identity.
 */
const UNAVAILABLE_CALLBACK = 'unavailable';

/**
 Serialized owned callback identity.
 */
const OWNED_CALLBACK = 'owned';

/**
 JSON-safe callback identity.
 */
export type SerializedCallbackKey =
  | {
    readonly kind: typeof UNAVAILABLE_CALLBACK;
  }
    | {
      readonly kind: typeof OWNED_CALLBACK;
      readonly key: string;
    };

/**
 JSON-safe call edge.
 */
export type SerializedCallEdge =
  & Omit<
    CallEdge,
    | 'callbackKeysByCalleeSlot'
      | 'callbackFileNamesByCalleeSlot'
      | 'originsByCalleeSlot'
      | 'capturedOriginsByFormal'
      | 'foreignOriginsByFormal'
    >
    & {
      readonly originsByCalleeSlot: readonly (readonly number[])[];
      readonly capturedOriginsByFormal: readonly (readonly number[])[];
      readonly foreignOriginsByFormal: readonly (readonly number[])[];
      readonly callbackKeysByCalleeSlot: readonly SerializedCallbackKey[];
      readonly callbackFileNamesByCalleeSlot: readonly SerializedCallbackKey[];
    };

/**
 JSON-safe direct summary.
 */
export type SerializedEffectSummary = {
  readonly parameterCount: number;
  readonly parameterOfSlot: readonly number[];
  readonly directMutated: readonly number[];
  readonly directInvoked: readonly number[];
  readonly directOpaque: readonly number[];
  readonly opaqueProvenanceBySlot: readonly (readonly [
    number,
    readonly string[]
  ])[];
  readonly mutated: readonly number[];
  readonly invoked: readonly number[];
  readonly opaque: readonly number[];
  readonly directForeignBorrowed: readonly number[];
  readonly directReturned: readonly number[];
  readonly returned: readonly number[];
  readonly relations: readonly CallbackRelation[];
  readonly elementApplications: readonly ElementApplication[];
  readonly resultApplications: readonly ResultApplication[];
  readonly calls: readonly SerializedCallEdge[];
};

/**
 JSON-safe summaries keyed by stable callable identity.
 */
export type SerializedEffectSummaries = readonly (
  readonly [
    string,
    SerializedEffectSummary
  ]
)[];

/**
 Rebuilds slot ownership from a restored payload.
 
 @param summary - Restored summary carrying its persisted ownership.
 
 @returns ownership usable for projection back to parameters.
 
 @example
 ```ts
 restoredOwnership(summary);
 ```
 */
function restoredOwnership(summary: SerializedEffectSummary,): SlotOwnership {
  /**
   Owning parameter of every slot, as branded positions.
   */
  const parameterOfSlot = summary.parameterOfSlot
    .map(function brandOwner(owner,): ParameterIndex {
      return asParameterIndex(owner,);
    },);
  return {
    parameterCount: summary.parameterCount,
    slotCount: parameterOfSlot.length,
    parameterOfSlot,
    slotsByParameter: slotsByParameterFrom({ parameterOfSlot, },),
  };
}

/**
 Rebrands a restored list of slot numbers.
 
 @param slots - Plain slot numbers read from the payload.
 
 @returns same slots, branded.
 
 @example
 ```ts
 restoredSlotList([0, 1]);
 ```
 */
function restoredSlotList(slots: readonly number[],): readonly EffectSlot[] {
  return slots.map(function brandSlot(slot,): EffectSlot {
    return asEffectSlot(slot,);
  },);
}

/**
 Rebrands a restored set of slot numbers.
 
 @param slots - Plain slot numbers read from the payload.
 
 @returns mutable branded slot set.
 
 @example
 ```ts
 restoredSlots([0, 1]);
 ```
 */
function restoredSlots(slots: readonly number[],): Set<EffectSlot> {
  return new Set(restoredSlotList(slots,),);
}

/**
 Serializes callback identity without process-local symbols.
 
 @param key - Owned callable key or unavailable sentinel.
 
 @returns JSON-safe callback identity.
 
 @example
 ```ts
 serializeCallbackKey(OWNED_CALLABLE_UNAVAILABLE);
 ```
 */
function serializeCallbackKey(
  key: string | typeof OWNED_CALLABLE_UNAVAILABLE,
): SerializedCallbackKey {
  return key === OWNED_CALLABLE_UNAVAILABLE
    ? { kind: UNAVAILABLE_CALLBACK, }
    : {
      kind: OWNED_CALLBACK,
      key,
    };
}

/**
 Restores callback identity from JSON-safe representation.
 
 @param key - Serialized callback identity.
 
 @returns owned callable key or unavailable sentinel.
 
 @example
 ```ts
 deserializeCallbackKey({ kind: 'unavailable' });
 ```
 */
function deserializeCallbackKey(
  key: SerializedCallbackKey,
): string | typeof OWNED_CALLABLE_UNAVAILABLE {
  return key.kind === UNAVAILABLE_CALLBACK
    ? OWNED_CALLABLE_UNAVAILABLE
    : key.key;
}

/**
 Serializes direct summaries for persistent cache storage.
 
 @param summaries - Direct summaries tied to one exact source.
 
 @returns deterministic JSON-safe entries.
 
 @example
 ```ts
 serializeEffectSummaries(summaries);
 ```
 */
export function serializeEffectSummaries(
  summaries: ReadonlyMap<string, MutableEffectSummary>,
): SerializedEffectSummaries {
  return [...summaries.entries(),]
    .toSorted(function sortEntries(
      [left,],
      [right,],
    ): number {
      return left.localeCompare(right,);
    },)
    .map(function serializeEntry([key, summary,],): readonly [
      string,
      SerializedEffectSummary,
    ] {
      return [
        key,
        {
          parameterCount: summary.slots
            .parameterCount,
          /* Ownership is persisted because a restored summary has no declaration to derive
           * it from, and every effect set it carries is meaningless without it: the numbers
           * are slots, and only this says which parameter each one answers for. */
          parameterOfSlot: [...summary.slots
            .parameterOfSlot,],
          directMutated: [...summary.directMutated,],
          directInvoked: [...summary.directInvoked,],
          directOpaque: [...summary.directOpaque,],
          opaqueProvenanceBySlot: [...summary.opaqueProvenanceBySlot
            .entries(),]
            .map(function serializeProvenance([index, facts,],): readonly [
              number,
              readonly string[]
            ] {
              return [
                index,
                [...facts,].toSorted(),
              ];
            },),
          mutated: [...summary.mutated,],
          invoked: [...summary.invoked,],
          opaque: [...summary.opaque,],
          directForeignBorrowed: [...summary.directForeignBorrowed,],
          directReturned: [...summary.directReturned,],
          returned: [...summary.returned,],
          relations: summary.relations
            .map(function copyRelation(relation,) {
              return { ...relation, };
            },),
          elementApplications: summary.elementApplications
            .map(function copyApplication(application,): ElementApplication {
              return {
                ...application,
                observerParameterIndexes: [
                  ...application.observerParameterIndexes,
                ],
              };
            },),
          resultApplications: summary.resultApplications
            .map(function copyResultApplication(application,): ResultApplication {
              return { ...application, };
            },),
          calls: summary.calls
            .map(function serializeCall(edge,): SerializedCallEdge {
              return {
                ...edge,
                originsByCalleeSlot: edge.originsByCalleeSlot
                  .map(function plainOrigins(origins,): readonly number[] {
                    return [...origins,];
                  },),
                capturedOriginsByFormal: edge.capturedOriginsByFormal
                  .map(function plainCaptured(origins,): readonly number[] {
                    return [...origins,];
                  },),
                foreignOriginsByFormal: edge.foreignOriginsByFormal
                  .map(function plainForeign(origins,): readonly number[] {
                    return [...origins,];
                  },),
                callbackKeysByCalleeSlot: edge.callbackKeysByCalleeSlot
                  .map(serializeCallbackKey,),
                callbackFileNamesByCalleeSlot: edge.callbackFileNamesByCalleeSlot
                  .map(serializeCallbackKey,),
              };
            },),
        },
      ];
    },);
}

/**
 Deserializes direct summaries after cache envelope validation.
 
 @param summaries - JSON-safe summary entries written by this analyzer.
 
 @returns mutable direct summaries safe for fixed-point propagation.
 
 @throws TypeError when structurally corrupt payload cannot be rehydrated.
 
 @example
 ```ts
 deserializeEffectSummaries(serialized);
 ```
 */
export function deserializeEffectSummaries(
  summaries: SerializedEffectSummaries,
): ReadonlyMap<string, MutableEffectSummary> {
  return new Map(summaries.map(function deserializeEntry([key, summary,],): [
    string,
    MutableEffectSummary,
  ] {
    return [
      key,
      {
        slots: restoredOwnership(summary,),
        bindingOriginBySymbolId: new Map(),

        directMutated: restoredSlots(summary.directMutated,),
        directInvoked: restoredSlots(summary.directInvoked,),
        directOpaque: restoredSlots(summary.directOpaque,),
        opaqueProvenanceBySlot: new Map(summary.opaqueProvenanceBySlot
          .map(function deserializeProvenance([slot, facts,],): [
            EffectSlot,
            Set<string>
          ] {
            return [
              asEffectSlot(slot,),
              new Set(facts,),
            ];
          },),),
        /* Reseeded from the direct sets rather than trusted as stored. A serializer always
         * writes a summary whose propagated sets already contain its direct ones, because
         * `directEffectSummary` seeds them before returning, and propagation only ever adds.
         * Nothing downstream reseeds, though, so a payload that lost the relation would drop
         * a write that the same payload still records as direct. The union costs nothing and
         * removes the dependence on validation being exhaustive. */
        mutated: restoredSlots([
          ...summary.mutated,
          ...summary.directMutated,
        ],),
        invoked: restoredSlots([
          ...summary.invoked,
          ...summary.directInvoked,
        ],),
        opaque: restoredSlots([
          ...summary.opaque,
          ...summary.directOpaque,
        ],),
        directForeignBorrowed: new Set(summary.directForeignBorrowed
          .map(function brandForeign(owner,): ParameterIndex {
            return asParameterIndex(owner,);
          },),),
        directReturned: restoredSlots(summary.directReturned,),
        returned: restoredSlots(summary.returned,),
        relations: summary.relations
          .map(function copyRelation(relation,) {
            return { ...relation, };
          },),
        resultApplications: summary.resultApplications
          .map(function restoreResultApplication(application,): ResultApplication {
            return { ...application, };
          },),
        elementApplications: summary.elementApplications
          .map(function copyApplication(application,): ElementApplication {
            return {
              ...application,
              observerParameterIndexes: [
                ...application.observerParameterIndexes,
              ],
            };
          },),
        calls: summary.calls
          .map(function deserializeCall(edge,): CallEdge {
            return {
              ...edge,
              originsByCalleeSlot: edge.originsByCalleeSlot
                .map(restoredSlotList,),
              capturedOriginsByFormal: edge.capturedOriginsByFormal
                .map(restoredSlotList,),
              foreignOriginsByFormal: edge.foreignOriginsByFormal
                .map(function restoreForeign(origins,): readonly ParameterIndex[] {
                  return origins.map(function brandForeignOrigin(owner,): ParameterIndex {
                    return asParameterIndex(owner,);
                  },);
                },),
              callbackKeysByCalleeSlot: edge.callbackKeysByCalleeSlot
                .map(deserializeCallbackKey,),
              callbackFileNamesByCalleeSlot: edge.callbackFileNamesByCalleeSlot
                .map(deserializeCallbackKey,),
            };
          },),
      },
    ];
  },),);
}
