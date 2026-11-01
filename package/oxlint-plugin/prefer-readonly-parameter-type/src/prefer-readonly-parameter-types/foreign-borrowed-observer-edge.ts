/**
 Position-aware foreign ownership inbounds for default-library collection observers.
 
 @module
 */

import type {
  CallExpression,
  Node,
} from 'typescript/ast';
import {
  isCallExpression,
  isIdentifier,
  isInterfaceDeclaration,
  isMethodSignatureDeclaration,
} from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import { callableDeclaration, } from './effect-call-resolution.ts';
import { expressionElementOrigins, } from './effect-element-origin.ts';
import {
  memberCallReceiver,
  NO_MEMBER_RECEIVER,
} from './effect-member-call-receiver.ts';
import { parameterSlotTable, } from './effect-parameter-slots.ts';
import { parametersOfSlots, } from './effect-slot-projection.ts';
import {
  type EffectSlot,
  asParameterIndex,
  type ParameterIndex,
} from './effect-slot-identity.ts';
import {
  callableKey,
  callSiteKey,
  type EffectCallableDeclaration,
  type MutableEffectSummary,
  OWNED_CALLABLE_UNAVAILABLE,
} from './effect-summary-model.ts';
import { isForeignBorrowedType, } from './foreign-borrowed-identity.ts';

/**
 Sentinel when collection call does not prove observer ownership positions.
 */
const FOREIGN_OBSERVER_EDGE_UNAVAILABLE: unique symbol = Symbol(
  'foreign collection observer edge unavailable',
);

/**
 Sentinel when declaration or reference is not a direct collection observer argument.
 */
export const FOREIGN_OBSERVER_CALL_UNAVAILABLE: unique symbol = Symbol(
  'foreign collection observer call unavailable',
);

/**
 Default-library interfaces whose callback position contracts are modeled.
 */
const ARRAY_OBSERVER_OWNERS: ReadonlySet<string> = new Set([
  'Array',
  'ReadonlyArray',
]);

/**
 Array members whose callbacks receive element at position zero and receiver at position two.
 */
const ELEMENT_OBSERVER_MEMBERS: ReadonlySet<string> = new Set([
  'every',
  'filter',
  'find',
  'findLast',
  'flatMap',
  'forEach',
  'map',
  'some',
]);

/**
 Array fold members whose callback positions depend on seed presence.
 */
const FOLD_OBSERVER_MEMBERS: ReadonlySet<string> = new Set([
  'reduce',
  'reduceRight',
]);

/**
 Observer position receiving whole array receiver.
 */
const ELEMENT_OBSERVER_RECEIVER_POSITION = 2;

/**
 Fold observer position receiving whole array receiver.
 */
const FOLD_OBSERVER_RECEIVER_POSITION = 3;

/**
 Resolves selected default-library observer method name.
 
 @param project - Project resolving signature declaration.
 
 @param call - Collection call selecting observer member.
 
 @returns supported member name or unavailable sentinel.
 */
function observerMethodName({
  project,
  call,
}: {
  readonly project: Project;
  readonly call: CallExpression;
}): string | typeof FOREIGN_OBSERVER_EDGE_UNAVAILABLE {
  /**
   Selected method declaration from instantiated call signature.
   */
  const declaration = project.checker
    .getResolvedSignature(call,)
    ?.declaration
    ?.resolve(project,);
  if ((declaration === undefined)
    || (!isMethodSignatureDeclaration(declaration,))
    || (!isIdentifier(declaration.name,))
    || (!project.program
      .isSourceFileDefaultLibrary(declaration.getSourceFile(),)))
    return FOREIGN_OBSERVER_EDGE_UNAVAILABLE;
  /**
   Interface owning selected default-library method.
   */
  const owner = declaration.parent;
  if ((!isInterfaceDeclaration(owner,))
    || (!isIdentifier(owner.name,)))
    return FOREIGN_OBSERVER_EDGE_UNAVAILABLE;
  /**
   Default-library interface name owning selected member.
   */
  const ownerName = owner.name
    .text;
  if (!ARRAY_OBSERVER_OWNERS.has(ownerName,))
    return FOREIGN_OBSERVER_EDGE_UNAVAILABLE;
  /**
   Static member name after identifier narrowing.
   */
  const { text: memberName, } = declaration.name;
  return memberName;
}

/**
 Determines observer formal positions receiving receiver-owned state.
 
 @param memberName - Supported default-library array member.
 
 @param call - Exact call deciding whether fold has independent seed.
 
 @returns receiver-derived positions or unavailable sentinel.
 */
function receiverDerivedObserverPositions({
  memberName,
  call,
}: {
  readonly memberName: string;
  readonly call: CallExpression;
}): readonly ParameterIndex[] | typeof FOREIGN_OBSERVER_EDGE_UNAVAILABLE {
  if (ELEMENT_OBSERVER_MEMBERS.has(memberName,)) {
    return [
      asParameterIndex(0,),
      asParameterIndex(ELEMENT_OBSERVER_RECEIVER_POSITION,),
    ];
  }
  if (!FOLD_OBSERVER_MEMBERS.has(memberName,))
    return FOREIGN_OBSERVER_EDGE_UNAVAILABLE;
  /**
   Exact call arguments used to distinguish fold overload.
   */
  const { arguments: callArguments, } = call;
  /**
   Whether call supplies independent accumulator seed after observer.
   */
  const seeded = callArguments.length > 1;
  return seeded
    ? [
      asParameterIndex(1,),
      asParameterIndex(FOLD_OBSERVER_RECEIVER_POSITION,),
    ]
    : [
      asParameterIndex(0,),
      asParameterIndex(1,),
      asParameterIndex(FOLD_OBSERVER_RECEIVER_POSITION,),
    ];
}

/**
 Tests whether observer argument resolves to exact demanded declaration.
 
 @param project - Project resolving callback expression.
 
 @param call - Collection call carrying observer.
 
 @param observerDeclaration - Demanded observer declaration.
 
 @returns whether first call argument names exact observer.
 */
function callUsesObserver({
  project,
  call,
  observerDeclaration,
}: {
  readonly project: Project;
  readonly call: CallExpression;
  readonly observerDeclaration: EffectCallableDeclaration;
}): boolean {
  /**
   First argument position reserved for every supported observer member.
   */
  const [observerArgument,] = call.arguments;
  if (observerArgument === undefined)
    return false;
  /**
   Owned callback declaration selected from argument expression.
   */
  const resolved = callableDeclaration({
    project,
    node: observerArgument,
  },);
  if (resolved === OWNED_CALLABLE_UNAVAILABLE)
    return false;
  return callableKey(resolved,) === callableKey(observerDeclaration,);
}

/**
 Finds collection call directly containing observer declaration or reference.
 
 @param node - Observer declaration or identifier reference.
 
 @returns parent collection call or unavailable sentinel.
 
 @example
 ```ts
 foreignObserverCall({ node: observerDeclaration });
 ```
 */
export function foreignObserverCall({
  node,
}: {
  readonly node: Node;
}): CallExpression | typeof FOREIGN_OBSERVER_CALL_UNAVAILABLE {
  /**
   Direct parent candidate containing node as one argument.
   */
  const { parent, } = node;
  if ((parent === undefined) || (!isCallExpression(parent,)))
    return FOREIGN_OBSERVER_CALL_UNAVAILABLE;
  return parent.arguments
    .some(function exactArgument(argument,): boolean {
      return argument === node;
    },)
    ? parent
    : FOREIGN_OBSERVER_CALL_UNAVAILABLE;
}

/**
 Adds exact synthetic inbound edge from collection receiver to owned observer.
 
 @param project - Project resolving call,
 receiver,
 and observer identities.
 
 @param call - Signature usage call carrying demanded observer.
 
 @param observerDeclaration - Observer whose foreign formals are being proven.
 
 @param callerSummary - Enclosing caller ownership summary receiving synthetic edge.
 
 @returns whether call was a supported exact observer inbound.
 
 @mutates callerSummary - Appends one position-aware foreign inbound edge.
 
 @example
 ```ts
 addForeignObserverInbound({ project, call, observerDeclaration, callerSummary });
 ```
 */
export function addForeignObserverInbound({
  project,
  call,
  observerDeclaration,
  callerSummary,
}: {
  readonly project: Project;
  readonly call: CallExpression;
  readonly observerDeclaration: EffectCallableDeclaration;
  readonly callerSummary: MutableEffectSummary;
}): boolean {
  /**
   Supported default-library observer method selected at this call.
   */
  const memberName = observerMethodName({
    project,
    call,
  },);
  if (memberName === FOREIGN_OBSERVER_EDGE_UNAVAILABLE)
    return false;
  /**
   Whether call's observer argument resolves to demanded declaration.
   */
  const usesObserver = callUsesObserver({
    project,
    call,
    observerDeclaration,
  },);
  if (!usesObserver)
    return false;
  /**
   Receiver-derived observer positions from member contract.
   */
  /**
   Receiver-derived callback positions for selected supported member.
   */
  const derivedPositions = receiverDerivedObserverPositions({
    memberName,
    call,
  },);
  if (derivedPositions === FOREIGN_OBSERVER_EDGE_UNAVAILABLE)
    return false;
  /**
   Collection receiver expression whose elements enter observer.
   */
  const receiver = memberCallReceiver({ call, },);
  if (receiver === NO_MEMBER_RECEIVER)
    return false;
  /**
   Caller slots whose elements receiver exposes.
   */
  const receiverSlots = expressionElementOrigins({
    project,
    bindingOriginBySymbolId: callerSummary.bindingOriginBySymbolId,
    node: receiver,
  },);
  /**
   Caller parameters owning receiver-derived elements.
   */
  const receiverParameters = [...parametersOfSlots({
    ownership: callerSummary.slots,
    slots: receiverSlots,
  },),];
  /**
   Exact receiver marker covers local marker boundaries without caller parameter origin.
   */
  const receiverType = project.checker
    .getTypeAtLocation(receiver,);
  /**
   Whether exact receiver type carries explicit foreign boundary marker.
   */
  const receiverDirectlyForeign = receiverType === undefined
    ? false
    : isForeignBorrowedType({
      project,
      type: receiverType,
    },);
  /**
   Observer parameters including optional explicit TypeScript `this` declaration.
   */
  const { parameters: observerParameters, } = observerDeclaration;
  /**
   Runtime formal positions begin after explicit `this` declaration when present.
   */
  const [firstObserverParameter,] = observerParameters;
  /**
   First formal name,
   absent only for zero-parameter callback.
   */
  const firstObserverName = firstObserverParameter?.name;
  /**
   Whether declaration begins with TypeScript-only `this` parameter.
   */
  const hasExplicitThis = (firstObserverName !== undefined)
    && isIdentifier(firstObserverName,)
    && (firstObserverName.text === 'this');
  /**
   Declaration offset translating runtime callback positions.
   */
  const runtimePositionOffset = hasExplicitThis ? 1 : 0;
  /**
   Observer slot ownership required by synthetic call edge arrays.
   */
  const observerSlots = parameterSlotTable({ declaration: observerDeclaration, },);
  /**
   Caller edges receiving synthetic observer inbound.
   */
  const { calls, } = callerSummary;
  calls.push({
    callSiteKey: callSiteKey(call,),
    calleeKey: callableKey(observerDeclaration,),
    calleeFileName: observerDeclaration.getSourceFile()
      .fileName,
    originsByCalleeSlot: observerSlots.parameterOfSlot
      .map(function noEffectOrigins(): readonly EffectSlot[] {
        return [];
      },),
    capturedOriginsByFormal: observerParameters
      .map(function noCapturedOrigins(): readonly EffectSlot[] {
        return [];
      },),
    foreignOriginsByFormal: observerParameters
      .map(function observerForeignOrigins(
        _parameter,
        parameterIndex,
      ): readonly ParameterIndex[] {
        /**
         Runtime callback position excluding explicit `this` declaration.
         */
        const runtimePosition = parameterIndex - runtimePositionOffset;
        /**
         Whether declaration formal receives collection receiver state.
         */
        const receiverDerived = (runtimePosition >= 0)
          && derivedPositions.includes(asParameterIndex(runtimePosition,),);
        return receiverDerived ? receiverParameters : [];
      },),
    directForeignByFormal: observerParameters
      .map(function observerDirectForeign(
        _parameter,
        parameterIndex,
      ): boolean {
        /**
         Runtime callback position excluding explicit `this` declaration.
         */
        const runtimePosition = parameterIndex - runtimePositionOffset;
        if (!receiverDirectlyForeign)
          return false;
        if (runtimePosition < 0)
          return false;
        return derivedPositions.includes(asParameterIndex(runtimePosition,),);
      },),
    foreignInbound: true,
    callbackKeysByCalleeSlot: observerSlots.parameterOfSlot
      .map(function unavailableCallback() {
        return OWNED_CALLABLE_UNAVAILABLE;
      },),
    callbackFileNamesByCalleeSlot: observerSlots.parameterOfSlot
      .map(function unavailableCallbackFile() {
        return OWNED_CALLABLE_UNAVAILABLE;
      },),
  },);
  return true;
}
