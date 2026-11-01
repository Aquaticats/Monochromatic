/**
 Complete persistent effect-summary payload validation.
 
 @module
 */

import { isRetentionProvenance, } from './effect-retention-provenance.ts';
import {
  isBoundedIndexes,
  isCacheString,
  isIndex,
  isRecord,
  MAX_CALLABLE_ARITY,
} from './effect-summary-cache-primitive.ts';
import type {
  SerializedCallEdge,
  SerializedCallbackKey,
  SerializedEffectSummaries,
  SerializedEffectSummary,
} from './effect-summary-serialization.ts';

/**
 Tests caller-side roots recorded per callee position.
 
 The outer length is a count of the callee's positions, which this validator cannot know,
 so it is bounded by arity alone. The inner values belong to the caller and are bounded by
 whatever the caller has: slots for effect origins, parameters for foreign ownership.
 
 @param value - Parsed JSON value.
 
 @param upperBound - Exclusive caller index limit.
 
 @returns whether nested argument roots are bounded and valid.
 */
function isArgumentRoots({
  value,
  upperBound,
}: {
  readonly value: unknown;
  readonly upperBound: number;
}): boolean {
  return Array.isArray(value,)
    && (value.length <= MAX_CALLABLE_ARITY)
    && value.every(function validRoots(roots,): boolean {
      return isBoundedIndexes({
        value: roots,
        upperBound,
      },);
    },);
}

/**
 Tests serialized callback identity.
 
 @param value - Parsed JSON value.
 
 @returns whether callback identity is unavailable or exact owned key.
 */
function isCallbackKey(value: unknown,): value is SerializedCallbackKey {
  return isRecord(value,)
    && ((value.kind === 'unavailable')
      || ((value.kind === 'owned') && isCacheString(value.key,)));
}

/**
 Tests one serialized owned call edge.
 
 @param value - Parsed JSON value.
 
 @param parameterCount - Exclusive caller parameter-index limit.
 
 @param slotCount - Exclusive caller slot limit.
 
 @returns whether every edge field and relation is valid.
 */
function isCallEdge({
  value,
  parameterCount,
  slotCount,
}: {
  readonly value: unknown;
  readonly parameterCount: number;
  readonly slotCount: number;
}): boolean {
  if ((!isRecord(value,))
    /* Checked because a deferred result use finds its edge by this key and by nothing
     * else. An edge accepted with a malformed one matches no application, so
     * `propagateResultApplications` skips a use it should have resolved, and a skipped use
     * is a missing effect rather than a loud failure: the parameter keeps an offer it
     * should not have. Every other identity on this edge was validated while the one that
     * decides whether an effect arrives was not. */
    || (!isCacheString(value.callSiteKey,))
    || (!isCacheString(value.calleeKey,))
    || (!isCacheString(value.calleeFileName,))
    || (!Array.isArray(value.originsByCalleeSlot,))
    || (!isArgumentRoots({
      value: value.originsByCalleeSlot,
      upperBound: slotCount,
    },))
    /* Bounded by the caller's slot count rather than its parameter count, unlike the
     * foreign list beside it. Both are indexed by the callee's formals, and they differ in
     * what they hold: foreign ownership collapses to caller parameters at the point of
     * record, while a capture stays a caller slot so a destructured caller keeps the
     * property that was captured. */
    || (!Array.isArray(value.capturedOriginsByFormal,))
    || (!isArgumentRoots({
      value: value.capturedOriginsByFormal,
      upperBound: slotCount,
    },))
    || (!Array.isArray(value.foreignOriginsByFormal,))
    || (!isArgumentRoots({
      value: value.foreignOriginsByFormal,
      upperBound: parameterCount,
    },))
    || (!Array.isArray(value.directForeignByFormal,))
    || (!Array.isArray(value.callbackKeysByCalleeSlot,))
    || (!Array.isArray(value.callbackFileNamesByCalleeSlot,))
    || ((typeof value.foreignInbound) !== 'boolean'))
    return false;
  /* Two lengths, not one. Slots split the edge: origins and callback identities are indexed
   * by the callee's slots, while foreign ownership is indexed by its formals, and a callee
   * with a destructured parameter has strictly more of the first. The counts belong to the
   * callee, which this validator cannot see, so only their agreement is checked. */
  /**
   Callee slot count shared by every slot-indexed edge field.
   */
  const slotArity = value.originsByCalleeSlot
    .length;
  /**
   Callee formal count shared by every formal-indexed edge field.
   */
  const formalArity = value.foreignOriginsByFormal
    .length;
  return (formalArity <= slotArity)
    && (value.capturedOriginsByFormal
      .length
      === formalArity)
    && (value.directForeignByFormal
      .length
      === formalArity)
    && (value.callbackKeysByCalleeSlot
      .length
      === slotArity)
    && (value.callbackFileNamesByCalleeSlot
      .length
      === slotArity)
    && value.directForeignByFormal
      .every(function booleanFlag(flag,): boolean {
        return (typeof flag) === 'boolean';
      },)
    && value.callbackKeysByCalleeSlot
      .every(isCallbackKey,)
    && value.callbackFileNamesByCalleeSlot
      .every(isCallbackKey,);
}

/**
 Tests one callback relation.
 
 @param value - Parsed JSON value.
 
 @param slotCount - Exclusive callable slot limit.
 
 @returns whether relation indexes are valid.
 */
function isCallbackRelation({
  value,
  slotCount,
}: {
  readonly value: unknown;
  readonly slotCount: number;
}): boolean {
  return isRecord(value,)
    && isIndex({
      value: value.callbackSlot,
      upperBound: slotCount,
    },)
    && isIndex({
      value: value.sourceSlot,
      upperBound: slotCount,
    },)
    && isIndex({
      value: value.callbackArgumentPosition,
      upperBound: MAX_CALLABLE_ARITY,
    },);
}

/**
 Tests one serialized element-flow relation.
 
 @param value - Parsed JSON value.
 
 @param slotCount - Exclusive callable slot limit.
 
 @returns whether relation identity and indexes are valid.
 */
function isElementApplication({
  value,
  slotCount,
}: {
  readonly value: unknown;
  readonly slotCount: number;
}): boolean {
  return isRecord(value,)
    && isIndex({
      value: value.receiverSlot,
      upperBound: slotCount,
    },)
    && isCacheString(value.callbackKey,)
    && isBoundedIndexes({
      value: value.observerParameterIndexes,
      upperBound: MAX_CALLABLE_ARITY,
    },);
}

/**
 Kinds a persisted deferred result use may name.
 
 Held here rather than imported from the model so the validator states the payload
 shape it accepts in its own terms. A kind added to the model and not to this set is
 rejected as corrupt, which is the safe direction: a summary missing one deferred use
 withholds offers, while one carrying an uninterpretable use would reach propagation.
 */
const SERIALIZED_RESULT_APPLICATION_KINDS: ReadonlySet<string> = new Set([
  'mutated',
  'retained',
  'returned',
],);

/**
 Tests one serialized deferred result use.
 
 This field went unvalidated while every sibling was checked, and the gap was harmless
 only by accident: an unrecognised payload contributed a call-site key that matched no
 edge and was skipped. The retaining kind ended that, because it carries provenance the
 propagation requires, and a payload naming that kind without it now reaches a throw
 rather than a skip. Validating here is what keeps `rejects corrupt nested persistent
 payloads` the behaviour for corrupt input instead of a crash inside the fixed point.
 
 @param value - Parsed JSON value.
 
 @returns whether the deferred use names a known kind and carries what that kind needs.
 */
function isResultApplication(value: unknown,): boolean {
  if ((!isRecord(value,))
    || (!isCacheString(value.callSiteKey,))
    || ((typeof value.kind) !== 'string')
    || (!SERIALIZED_RESULT_APPLICATION_KINDS.has(value.kind,)))
    return false;
  /* Provenance is required for exactly the retaining kind and forbidden on the other two,
   * so both directions are checked: a retention without it reaches the diagnostic as an
   * unexplained opaque slot, and a mutation carrying one would mean the payload was
   * written by something whose model does not match this one.
   *
   * The prefix is checked too, not merely the type. A retained use whose provenance is an
   * arbitrary string passes every structural test and is then read back as call-caused
   * opacity, producing an unresolved-effect report naming a call that does not exist. */
  return value.kind === 'retained'
    ? isCacheString(value.provenance,) && isRetentionProvenance(value.provenance,)
    : value.provenance === undefined;
}

/**
 Tests opaque provenance entries.
 
 @param value - Parsed JSON value.
 
 @param slotCount - Exclusive callable slot limit.
 
 @returns whether provenance keys and facts are bounded.
 */
function isOpaqueProvenance({
  value,
  slotCount,
}: {
  readonly value: unknown;
  readonly slotCount: number;
}): boolean {
  if (!Array.isArray(value,))
    return false;
  /**
   Parsed provenance entries narrowed from JSON array.
   */
  const entries: readonly unknown[] = value;
  /* Rehydration builds a `Map` from these pairs, so a repeated slot keeps only the last
   * one and every fact recorded against the earlier entries is dropped silently. The
   * serializer cannot produce a repeat, because it writes from a `Map`, which makes a
   * repeat proof the payload was not written by this code. Losing facts rather than
   * failing is the outcome worth refusing: a slot that arrives with its retention facts
   * missing is reported as an unresolved effect instead of being withheld quietly. */
  if (new Set(entries.map(function slotOf(entry,): unknown {
    return Array.isArray(entry,) ? entry[0] : entry;
  },),).size !== entries.length)
    return false;
  return entries.every(function validEntry(entry,): boolean {
    if ((!Array.isArray(entry,)) || (entry.length !== 2))
      return false;
    /**
     Parsed tuple fields retained as unknown until individual validation.
     */
    const fields: readonly unknown[] = entry;
    /**
     Parsed provenance parameter index and fact list.
     */
    const [index, facts,] = fields;
    if ((!isIndex({
      value: index,
      upperBound: slotCount,
    },))
      || (!Array.isArray(facts,))
      || (facts.length > MAX_CALLABLE_ARITY))
      return false;
    /**
     Parsed provenance facts narrowed from JSON array.
     */
    const factValues: readonly unknown[] = facts;
    return factValues.every(function validFact(fact,): boolean {
      return isCacheString(fact,);
    },);
  },);
}

/**
 Tests complete serialized direct summary.
 
 @param value - Parsed JSON value.
 
 @returns whether all summary fields satisfy runtime schema.
 */
function isEffectSummary(value: unknown,): value is SerializedEffectSummary {
  if ((!isRecord(value,))
    || ((typeof value.parameterCount) !== 'number')
    || (!isIndex({
      value: value.parameterCount,
      upperBound: MAX_CALLABLE_ARITY,
    })))
    return false;
  /**
   Callable parameter count reused by every parameter-relative field.
   */
  const { parameterCount, } = value;
  /* Ownership is what makes the rest of the payload meaningful, so it is checked before
   * anything that depends on it. Every slot must name a parameter this callable has, and the
   * whole parameters must come first and in order, because that is the numbering the
   * allocator produces and a caller's cached edge indexes into. */
  /**
   Persisted slot ownership, still unvalidated.
   */
  const { parameterOfSlot, } = value;
  if (!Array.isArray(parameterOfSlot,))
    return false;
  /**
   Distinct owners named by the persisted ownership.
   */
  const distinctOwners = [...new Set(parameterOfSlot,),];
  if ((parameterOfSlot.length < parameterCount)
    || (parameterOfSlot.length > MAX_CALLABLE_ARITY)
    || (!isBoundedIndexes({
      value: distinctOwners,
      upperBound: parameterCount,
    },))
    || parameterOfSlot
      .slice(
      0,
      parameterCount,
      )
      .some(function outOfOrderWholeSlot(
        owner,
        slot,
      ): boolean {
        return owner !== slot;
      },))
    return false;
  /**
   Slot count bounding every slot-relative field.
   */
  const slotCount = parameterOfSlot.length;
  /**
   Set-backed effect arrays requiring bounded unique slots.
   */
  const slotArrays = [
    value.directMutated,
    value.directInvoked,
    value.directOpaque,
    value.mutated,
    value.invoked,
    value.opaque,
    value.directReturned,
    /* `returned` was serialized, deserialized through `restoredSlots`, and never checked
     * here. A payload missing it or holding a non-array passed validation and crashed
     * inside rehydration, which is the one outcome `rejects corrupt nested persistent
     * payloads` says must not happen. It sits beside `directReturned` because propagation
     * seeds one from the other and both index the same slots. */
    value.returned,
  ];
  return slotArrays.every(function validSlots(slots,): boolean {
    return isBoundedIndexes({
      value: slots,
      upperBound: slotCount,
    },);
  },)
    && isBoundedIndexes({
      value: value.directForeignBorrowed,
      upperBound: parameterCount,
    },)
    && isOpaqueProvenance({
      value: value.opaqueProvenanceBySlot,
      slotCount,
    },)
    && Array.isArray(value.relations,)
    && (value.relations
      .length
      <= MAX_CALLABLE_ARITY)
    && value.relations
      .every(function validRelation(relation,): boolean {
        return isCallbackRelation({
          value: relation,
          slotCount,
        },);
      },)
    && Array.isArray(value.elementApplications,)
    && (value.elementApplications
      .length
      <= MAX_CALLABLE_ARITY)
    && value.elementApplications
      .every(function validApplication(application,): boolean {
        return isElementApplication({
          value: application,
          slotCount,
        },);
      },)
    && Array.isArray(value.resultApplications,)
    /* Bounded like `calls` and `elementApplications` beside it. The cap is a sanity limit
     * rather than a real arity, and one body can defer more result uses than a callable
     * has parameters, but accepting an unbounded array from a file on disk is the part
     * worth refusing. */
    && (value.resultApplications
      .length
      <= MAX_CALLABLE_ARITY)
    && value.resultApplications
      .every(function validResultApplication(application,): boolean {
        return isResultApplication(application,);
      },)
    && Array.isArray(value.calls,)
    && (value.calls
      .length
      <= MAX_CALLABLE_ARITY)
    && value.calls
      .every(function validCall(call,): boolean {
        return isCallEdge({
          value: call,
          parameterCount,
          slotCount,
        },);
      },);
}

/**
 Tests full persistent effect-summary payload schema.
 
 @param value - Parsed JSON payload.
 
 @returns whether value can be safely rehydrated.
 
 @example
 ```ts
 if (isSerializedEffectSummaries(value)) deserializeEffectSummaries(value);
 ```
 */
export function isSerializedEffectSummaries(
  value: unknown,
): value is SerializedEffectSummaries {
  if ((!Array.isArray(value,)) || (value.length > MAX_CALLABLE_ARITY))
    return false;
  /**
   Parsed summary entries narrowed from JSON array.
   */
  const entries: readonly unknown[] = value;
  /**
   Callable keys rejecting duplicate cache entries.
   */
  const keys = new Set<string>();
  for (const entry of entries) {
    if ((!Array.isArray(entry,)) || (entry.length !== 2))
      return false;
    /**
     Parsed tuple fields retained as unknown until individual validation.
     */
    const fields: readonly unknown[] = entry;
    /**
     Parsed callable key and direct summary.
     */
    const [key, summary,] = fields;
    if ((!isCacheString(key,)) || keys.has(key,)
      || (!isEffectSummary(summary,)))
      return false;
    keys.add(key,);
  }
  return true;
}
