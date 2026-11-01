/**
 Owned call-edge construction for effect propagation.
 
 Every argument contributes the origins of everything it packages, with no filter
 derived from what the callee's authored `@mutates` blocks happen to name. An earlier
 revision narrowed an object-literal argument to the contract-named property names
 whenever the callee's parameter was a destructuring pattern, which `ST9` makes the
 normal shape here. That let an authored comment delete a recorded mutation: a callee
 writing through a property its contract omitted had that write attributed to nothing,
 and the caller's parameter was then offered as readonly. `directRestrictedRowEffect`
 in the result-provenance fixture is the measured case, and
 `doc/decision/prefer-readonly-contract-name-narrowing.md` records why the precise
 version has to measure the callee instead of reading its contract.
 
 @module
 */

import type { CallExpression, } from 'typescript/ast';
import type { Project, } from 'typescript/sync';

import {
  argumentCapturedOrigins,
  capturedOriginsByFormal,
} from './effect-captured-argument-origins.ts';
import { expressionContainsForeignBorrowed, } from './foreign-borrowed-classifier.ts';

import {
  callableKey,
  callSiteKey,
  type EffectCallableDeclaration,
  type MutableEffectSummary,
  OWNED_CALLABLE_UNAVAILABLE,
} from './effect-summary-model.ts';
import {
  ARGUMENT_NOT_DECOMPOSABLE,
  type ArgumentPropertyView,
  argumentPropertyView,
  originsOfPropertyKey,
} from './effect-argument-properties.ts';
import {
  callableDeclaration,
  parameterIndexes,
} from './effect-call-resolution.ts';
import {
  memberCallReceiver,
  NO_MEMBER_RECEIVER,
} from './effect-member-call-receiver.ts';
import {
  parameterSlotTable,
  type ParameterSlotTable,
} from './effect-parameter-slots.ts';
import type {
  EffectSlot,
  ParameterIndex,
} from './effect-slot-identity.ts';
import { parametersOfSlots, } from './effect-slot-projection.ts';

import {
  calleeHasThisParameter,
  formalActualPositions,
} from './effect-formal-actual-mapping.ts';

/**
 Sentinel marking a formal that no single actual argument fills.
 */
const NO_SOLE_POSITION = -1;

/**
 Sentinel marking a slot that stands for a whole parameter rather than one of its properties.
 */
const SLOT_IS_WHOLE_PARAMETER: unique symbol = Symbol(
  'effect slot stands for a whole parameter and names no property',
);

/**
 Property key one slot names, or the sentinel for a whole-parameter slot.
 */
type SlotPropertyKey = string | typeof SLOT_IS_WHOLE_PARAMETER;

/**
 Adds one owned call edge with caller-relative parameter roots.
 
 @param project - TypeScript project resolving callbacks and provenance.
 
 @param call - Owned call expression.
 
 @param callee - Exact owned callable declaration.
 
 @param allArgumentIndexes - Caller roots packaged by each argument.
 
 @param summary - Caller summary receiving edge.
 
 @param foreignInbound - Whether call belongs directly to caller summary.
 
 @param analysisRoot - Optional external implementation root.
 
 @mutates summary - Appends exact owned call edge.
 
 @example
 ```ts
 addOwnedCallEdge({ project, call, callee, allArgumentIndexes, summary, foreignInbound });
 ```
 */
export function addOwnedCallEdge({
  project,
  call,
  callee,
  allArgumentIndexes,
  summary,
  foreignInbound,
  analysisRoot,
}: {
  readonly project: Project;
  readonly call: CallExpression;
  readonly callee: EffectCallableDeclaration;
  readonly allArgumentIndexes: readonly (readonly EffectSlot[])[];
  readonly summary: MutableEffectSummary;
  readonly foreignInbound: boolean;
  readonly analysisRoot?: string;
}): void {
  /**
   Actual positions each formal can receive, covering `this`, rest and spread.
   */
  const positionsByFormal = formalActualPositions({
    callee,
    call,
  },);
  /**
   Caller origins packaged into each formal, unioned over the positions it can receive.
   */
  const originsByFormal = positionsByFormal
    .map(function originsForFormal(positions,): readonly EffectSlot[] {
      /**
       Distinct caller slots reaching this formal.
       */
      const origins = new Set<EffectSlot>();
      positions.forEach(function collectPosition(position,): void {
        (allArgumentIndexes[position] ?? [])
          .forEach(function collectOrigin(origin,): void {
            origins.add(origin,);
          },);
      },);
      return [...origins,];
    },);
  /* A method declaring an explicit `this` formal writes through its receiver, and the receiver
   * is the value before the dot rather than an argument, so `formalActualPositions` has no
   * position for it and left that formal with nothing. `explicitThisReceiver` in the
   * slot-narrowing fixture measured no effect for `row.write()` where `write` assigns
   * `this.label`, which offers a row the method mutates.
   *
   * This is the one place holding both the callee declaration and the call expression, so it is
   * where the receiver can fill that formal. */
  /**
   Caller origins reaching each formal, with the receiver filling an explicit `this`.
   */
  const originsWithReceiver = calleeHasThisParameter({ callee, },)
    ? originsByFormal.map(function fillReceiver(
      origins,
      formalIndex,
    ): readonly EffectSlot[] {
      if (formalIndex !== 0)
        return origins;
      /**
       Value before the dot, absent when the call names no member.
       */
      const receiver = memberCallReceiver({ call, },);
      return receiver === NO_MEMBER_RECEIVER
        ? origins
        : parameterIndexes({
          project,
          bindingOriginBySymbolId: summary.bindingOriginBySymbolId,
          node: receiver,
        },);
    },)
    : originsByFormal;
  /**
   Owned callback declarations paired with actual argument positions.
   */
  const callbacks = call.arguments
    .map(function callbackDeclaration(argument,) {
      return callableDeclaration({
        project,
        node: argument,
        ...(analysisRoot === undefined) ? {} : { analysisRoot, },
      },);
    },);
  /**
   Sole actual position filling each formal, absent when several or none can.
   
   A callback identity names one declaration, so it only means anything for a formal
   fed by exactly one actual. A rest formal or one past a spread reports no callback,
   which makes `propagateInvokedCapabilities` treat its invocation as unresolved rather
   than assume an owned body it cannot name.
   */
  const soleByFormal = positionsByFormal
    .map(function soleForFormal(positions,): number {
      return positions.length === 1 ? positions[0] ?? NO_SOLE_POSITION : NO_SOLE_POSITION;
    },);
  /**
   Slots the callee owns, allocated from the callee declaration exactly as its own summary
   allocated them.
   */
  const calleeSlots = parameterSlotTable({ declaration: callee, },);
  /**
   Property key each callee slot names, absent for a whole-parameter slot.
   */
  const keyOfSlot = slotPropertyKeys({ calleeSlots, },);
  /**
   Authored property structure of the actuals filling each formal.
   
   Sentinel for a formal fed by anything this cannot read, which is the ordinary case: an
   identifier, a call result, a conditional, or a position past a spread. A formal fed by
   several actuals decomposes only when every one of them does, because a property of the
   formal could come from any of them.
   */
  const viewsByFormal = positionsByFormal
    .map(function viewsForFormal(
      positions,
      formalIndex,
    ): readonly ArgumentPropertyView[] | typeof ARGUMENT_NOT_DECOMPOSABLE {
      /* A rest formal is a synthesized array rather than an actual, so its property keys name
       * array indexes. In `function callee(...{ 0: box })` the key `0` is the whole first
       * actual, and resolving that key against a caller's `{ named: owned }` would find
       * nothing and lose every write through `box`. */
      /**
       Formal this position describes, absent when the mapping outruns the declaration.
       */
      const formal = callee.parameters[formalIndex];
      if ((formal === undefined) || (formal.dotDotDotToken !== undefined))
        return ARGUMENT_NOT_DECOMPOSABLE;
      /**
       Decomposition of every actual that can fill this formal.
       */
      const views = positions
        .map(function viewForPosition(
          position,
        ): ArgumentPropertyView | typeof ARGUMENT_NOT_DECOMPOSABLE {
          /**
           Actual at this position, absent when the mapping names one the call lacks.
           */
          const argument = call.arguments[position];
          return argument === undefined
            ? ARGUMENT_NOT_DECOMPOSABLE
            : argumentPropertyView({
              project,
              bindingOriginBySymbolId: summary.bindingOriginBySymbolId,
              node: argument,
            },);
        },);
      return views.every(function decomposed(view,): view is ArgumentPropertyView {
        return view !== ARGUMENT_NOT_DECOMPOSABLE;
      },)
        ? views
        : ARGUMENT_NOT_DECOMPOSABLE;
    },);
  /**
   Caller origins per callee slot, narrowed to the property filling it where possible.
   
   A whole-parameter slot always takes every origin its formal packages. A property slot takes
   only what the caller's authored literal puts under that key, and falls back to the formal's
   full origins whenever the actual exposes no readable structure. That fallback is what keeps
   this sound: propagation looks the edge up by whichever slot the callee recorded its effect
   against and never consults the whole-parameter slot, so a property slot filled from nothing
   would discard a write the callee really performs.
   */
  const originsByCalleeSlot = calleeSlots.parameterOfSlot
    .map(function originsForSlot(
      owner,
      slot,
    ): readonly EffectSlot[] {
      /* A formal-indexed array read with a parameter position. Those coincide by
       * construction, and no brand checks it: a branded number indexes anything. */
      /**
       Every origin this formal packages, which a whole slot takes unnarrowed.
       */
      const wholeOrigins = originsWithReceiver[owner] ?? [];
      /**
       Key this slot names, sentinel when the slot is the whole parameter.
       */
      const key = keyOfSlot[slot] ?? SLOT_IS_WHOLE_PARAMETER;
      /**
       Decomposed actuals filling this formal, sentinel when any resists decomposition.
       */
      const views = viewsByFormal[owner];
      if ((key === SLOT_IS_WHOLE_PARAMETER)
        || (views === undefined)
        || (views === ARGUMENT_NOT_DECOMPOSABLE))
        return wholeOrigins;
      /**
       Origins reaching this key across every actual that can fill the formal.
       */
      const narrowed = new Set<EffectSlot>();
      views.forEach(function narrowView(view,): void {
        originsOfPropertyKey({
          view,
          key,
        },)
          .forEach(function collectNarrowed(origin,): void {
            narrowed.add(origin,);
          },);
      },);
      return [...narrowed,];
    },);
  summary.calls
    .push({
      callSiteKey: callSiteKey(call,),
      calleeKey: callableKey(callee,),
      calleeFileName: callee.getSourceFile()
        .fileName,
      originsByCalleeSlot,
    /* Driven by `callbacks`, the declarations already resolved above, rather than by argument
     * syntax. That is what makes `retain(producer,)` behave like the inline form: the
     * resolver follows a local bound to a function expression where a syntax test would see
     * an identifier and stop. */
      capturedOriginsByFormal: capturedOriginsByFormal({
        positionsByFormal,
        argumentCaptures: argumentCapturedOrigins({
          project,
          bindingOriginBySymbolId: summary.bindingOriginBySymbolId,
          callables: callbacks,
          actuals: call.arguments,
        },),
      },),
    /* Foreign ownership is a marker on a whole parameter, and its consumer compares against
     * caller parameters, so caller slots collapse to the parameters that own them here rather
     * than at the point of use. */
      foreignOriginsByFormal: originsWithReceiver
        .map(function foreignOriginsForFormal(origins,): readonly ParameterIndex[] {
          return [
            ...parametersOfSlots({
              ownership: summary.slots,
              slots: new Set(origins,),
            },),
          ];
        },),
    /* Every covered actual must carry the marker for the formal to count as foreign,
     * because a foreign formal suppresses the readonly offer. Claiming foreignness for a
     * formal that might receive an ordinary argument would suppress an offer this
     * analysis never proved safe, so an unfilled or multiply-filled formal claims
     * nothing. */
      directForeignByFormal: positionsByFormal
        .map(function foreignForFormal(positions,): boolean {
          return (positions.length > 0)
            && positions.every(function positionIsForeign(position,): boolean {
            /**
             Actual argument at this position, absent when the call supplies none.
             */
              const argument = call.arguments[position];
              return (argument !== undefined)
                && expressionContainsForeignBorrowed({
                  project,
                  node: argument,
                },);
            },);
        },),
      foreignInbound,
    /* Callback identities are read with an index drawn from the callee's own invoked set,
     * which names slots, so they are indexed by slot too. A property slot repeats its
     * formal's identity for now: naming the callback a caller packaged into one property
     * needs the packaged-callable scan to report which property it came from, which task
     * #27 covers. */
      callbackKeysByCalleeSlot: calleeSlots.parameterOfSlot
        .map(function callbackKeyForSlot(owner,) {
        /**
         Resolved callback at the sole filling position, when there is one.
         */
          const candidate = soleCallback({
            soleByFormal,
            callbacks,
            formalIndex: owner,
          },);
          return candidate === OWNED_CALLABLE_UNAVAILABLE
            ? OWNED_CALLABLE_UNAVAILABLE
            : callableKey(candidate,);
        },),
      callbackFileNamesByCalleeSlot: calleeSlots.parameterOfSlot
        .map(function callbackFileNameForSlot(owner,) {
        /**
         Resolved callback at the sole filling position, when there is one.
         */
          const candidate = soleCallback({
            soleByFormal,
            callbacks,
            formalIndex: owner,
          },);
          return candidate === OWNED_CALLABLE_UNAVAILABLE
            ? OWNED_CALLABLE_UNAVAILABLE
            : candidate.getSourceFile()
              .fileName;
        },),
    },);
}

/**
 Inverts the callee's per-parameter key maps into one key per slot.
 
 @param calleeSlots - Slot table of the callee this edge names.
 
 @returns key each slot names, absent at every whole-parameter slot.
 
 @example
 ```ts
 slotPropertyKeys({ calleeSlots });
 ```
 */
function slotPropertyKeys(
  { calleeSlots, }: { readonly calleeSlots: ParameterSlotTable; },
): readonly SlotPropertyKey[] {
  /**
   Keys accumulated per slot, standing for the whole parameters that come first.
   */
  const keys: SlotPropertyKey[] = calleeSlots.parameterOfSlot
    .map(function noKey(): typeof SLOT_IS_WHOLE_PARAMETER {
      return SLOT_IS_WHOLE_PARAMETER;
    },);
  calleeSlots.propertySlotsByParameter
    .forEach(function readParameter(propertySlots,): void {
      propertySlots.forEach(function readKey(
        slot,
        key,
      ): void {
        keys[slot] = key;
      },);
    },);
  return keys;
}

/**
 Resolves the callback filling one formal, when exactly one actual fills it.
 
 @param soleByFormal - Sole filling actual position per formal.
 
 @param callbacks - Callables resolved at each actual position.
 
 @param formalIndex - Formal whose callback is wanted.
 
 @returns owned callback declaration, or sentinel when none is named.
 
 @example
 ```ts
 soleCallback({ soleByFormal, callbacks, formalIndex: 0 });
 ```
 */
function soleCallback({
  soleByFormal,
  callbacks,
  formalIndex,
}: {
  readonly soleByFormal: readonly number[];
  readonly callbacks: readonly (
    EffectCallableDeclaration | typeof OWNED_CALLABLE_UNAVAILABLE
  )[];
  readonly formalIndex: number;
},): EffectCallableDeclaration | typeof OWNED_CALLABLE_UNAVAILABLE {
  /**
   Actual position solely filling this formal, absent when several or none can.
   */
  const position = soleByFormal[formalIndex] ?? NO_SOLE_POSITION;
  if (position === NO_SOLE_POSITION)
    return OWNED_CALLABLE_UNAVAILABLE;
  return callbacks[position] ?? OWNED_CALLABLE_UNAVAILABLE;
}
