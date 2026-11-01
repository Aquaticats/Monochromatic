/**
 Element-flow derivation for default-library read-only view calls.
 
 @module
 */

import type {
  CallExpression,
  Expression,
  Node,
} from 'typescript/ast';
import {
  type Checker,
  type Project,
  SignatureKind,
  type Type,
} from 'typescript/sync';

import { callableDeclaration, } from './effect-call-resolution.ts';
import { expressionOrigins, } from './effect-binding-origins.ts';
import { expressionElementOrigins, } from './effect-element-origin.ts';
import { typeDefinitelyCallable, } from './effect-definitely-callable.ts';
import { viewResultUnaccounted, } from './effect-view-result-gate.ts';
import {
  expressionCanCarryMutableState,
  typeCanCarryMutableState,
} from './effect-primitive-origin.ts';
import {
  asParameterIndex,
  type EffectSlot,
  type ParameterIndex,
} from './effect-slot-identity.ts';
import {
  callableKey,
  type EffectCallableDeclaration,
  type ElementApplication,
  type MutableEffectSummary,
  NO_SLOT_ORIGIN,
  OWNED_CALLABLE_UNAVAILABLE,
  type SlotOrigins,
} from './effect-summary-model.ts';

/**
 Sentinel when a read-only view call leaves reachable user code unproven.
 */
export const READONLY_VIEW_UNDISCHARGED: unique symbol = Symbol(
  'read-only view call has unanalyzable reachable user code',
);

/**
 One call argument whose semantic type resolved, kept at its own position.
 */
type TypedArgument = {
  readonly argument: Expression;
  readonly argumentType: Type;
  readonly argumentIndex: number;
};

/**
 One observer argument backed by an owned implementation.
 */
type OwnedObserver = TypedArgument & {
  readonly declaration: EffectCallableDeclaration;
};

/**
 Sentinel when the member does not describe what an argument position receives.
 */
const OBSERVED_POSITIONS_UNAVAILABLE: unique symbol = Symbol(
  'member signature does not describe observer parameter positions',
);

/**
 Collects callback parameter positions carrying receiver-reachable state.
 
 Read from the member's own instantiated signature, never from how the observer
 annotates itself. TypeScript instantiates the member with the receiver's type
 arguments, so inside that signature the types exposing receiver state are the
 identical `Type` instances rather than merely equivalent ones. Both those
 arguments and the receiver type count: `forEach` hands the whole array to a
 third parameter, reaching the same state without any element parameter.
 
 The observer's own annotations cannot be used for this. A pre-declared
 function passed by reference, `states.map(operations.render)`, annotates its
 parameter independently, producing a structurally identical but distinct type
 that matches nothing. Matching against it silently found no position, and
 discharging on that emptiness dropped a real mutation.
 
 @param checker - TypeScript checker resolving parameter types.
 
 @param call - Read-only view call whose member signature is authoritative.
 
 @param argumentIndex - Position of observer among call arguments.
 
 @param elementTypes - Types the receiver view is instantiated over.
 
 @param receiverType - Receiver collection type.
 
 @returns positions exposing receiver state, or sentinel when undescribed.
 
 @example
 ```ts
 observedParameterIndexes({ checker, call, argumentIndex, elementTypes, receiverType });
 ```
 */
function observedParameterIndexes({
  checker,
  call,
  argumentIndex,
  elementTypes,
  receiverType,
}: {
  readonly checker: Checker;
  readonly call: CallExpression;
  readonly argumentIndex: number;
  readonly elementTypes: readonly Type[];
  readonly receiverType: Type;
},): readonly ParameterIndex[] | typeof OBSERVED_POSITIONS_UNAVAILABLE {
  /**
   Member parameter symbol receiving observer at this position.
   */
  const memberParameter = checker.getResolvedSignature(call,)
    ?.getParameters()[argumentIndex];
  if (memberParameter === undefined)
    return OBSERVED_POSITIONS_UNAVAILABLE;
  /**
   Callback type the member declares for this position.
   */
  const declaredObserverType = checker.getTypeOfSymbolAtLocation(
    memberParameter,
    call.expression,
  );
  if (declaredObserverType === undefined)
    return OBSERVED_POSITIONS_UNAVAILABLE;
  // An optional member parameter, `toSorted(compareFn?)`, types as a union with
  // `undefined`, which exposes no call signature at all. Strip it first, or every
  // optional-observer member would look undescribed and stay opaque.
  /**
   Declared callback type with any optionality removed.
   */
  const presentObserverType = checker.getNonNullableType(
    declaredObserverType,
  );
  if (presentObserverType === undefined)
    return OBSERVED_POSITIONS_UNAVAILABLE;
  /**
   Call signature the member declares for this position.
   */
  const [declaredSignature,] = checker.getSignaturesOfType(
    presentObserverType,
    SignatureKind.Call,
  );
  if (declaredSignature === undefined)
    return OBSERVED_POSITIONS_UNAVAILABLE;
  return declaredSignature
    .getParameters()
    .flatMap(function reachableParameter(
      parameterSymbol,
      observerParameterIndex,
    ): readonly ParameterIndex[] {
      /**
       Instantiated member callback parameter type at this call.
       */
      const parameterType = checker.getTypeOfSymbolAtLocation(
        parameterSymbol,
        call.expression,
      );
      // An unresolved parameter type counts as reachable: failing to prove a
      // position safe must widen what propagates, never narrow it.
      /**
       Whether this position exposes state reachable from the receiver.
       */
      const reachable = (parameterType === undefined)
        || (parameterType === receiverType)
        || elementTypes.includes(parameterType,);
      return reachable
        ? [asParameterIndex(observerParameterIndex,),]
        : [];
    },);
}

/**
 Derives element-flow relations for one default-library read-only view call.
 
 Answers only the reachable-user-code question; the caller has already proven
 that the member cannot restructure the receiver. Every path that cannot be
 derived returns the undischarged sentinel so the receiver stays opaque:
 a member observing elements with no caller-supplied observer, such as `join`
 or a bare `toSorted()`, supplies nothing to analyze, and an observer whose
 implementation is not owned source cannot be analyzed at all.
 
 @param project - TypeScript project resolving observer declarations.
 
 @param checker - TypeScript checker resolving receiver and parameter types.
 
 @param bindingOriginBySymbolId - Current callable parameter and alias origins.
 
 @param call - Read-only view call expression.
 
 @param receiver - Receiver expression rooted at a caller parameter.
 
 @param receiverSlot - Caller slot owning receiver.
 
 @param analysisRoot - Optional external implementation root.
 
 @returns derived relations, or sentinel when reachable code stays unproven.
 
 @example
 ```ts
 readonlyViewElementApplications({ project, checker, call, receiver, receiverSlot });
 ```
 */
export function readonlyViewElementApplications({
  project,
  checker,
  bindingOriginBySymbolId,
  call,
  receiver,
  receiverSlot,
  analysisRoot,
  body,
}: {
  readonly project: Project;
  readonly checker: Checker;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly call: CallExpression;
  readonly receiver: Expression;
  readonly receiverSlot: EffectSlot;
  readonly analysisRoot?: string;
  readonly body?: Node;
},): readonly ElementApplication[] | typeof READONLY_VIEW_UNDISCHARGED {
  /**
   Receiver collection type carrying the element type argument.
   */
  const receiverType = checker.getTypeAtLocation(receiver,);
  if ((receiverType === undefined) || (!receiverType.isTypeReference()))
    return READONLY_VIEW_UNDISCHARGED;
  // Every type argument counts, not just the first. `ReadonlyArray<T>` puts its
  // element at 0, but `ReadonlyMap<K, V>` puts the key there and the value that
  // callbacks receive at 1, so reading only position 0 misses map values.
  /**
   Types the read-only view is instantiated over.
   */
  const elementTypes = checker.getTypeArguments(receiverType,);
  if (elementTypes.length === 0)
    return READONLY_VIEW_UNDISCHARGED;
  // A member that builds a collection result routes it through
  // `ArraySpeciesCreate`, which reads `constructor[Symbol.species]` and calls
  // whatever it returns. That user code then receives everything the result
  // holds, which for `filter`, `slice`, `concat`, `flat` and an identity `map`
  // is the receiver's own elements. Measured in
  // `doc/decision/prefer-readonly-effect-model-split.md`.
  //
  // Whether a member consults species is not derivable: `toReversed`, `with`
  // and `toSpliced` build new arrays without it. So the derivable and
  // conservative test is what the result could carry. A result holding only
  // primitives keeps receiver state out of the channel however it is built,
  // which is what rescues `map` with a primitive-returning observer.
  //
  // Species is not the only way state leaves through the result. A member can
  // simply hand an element back, `find` and `findLast` returning `T | undefined`,
  // and that element is the receiver's own. Both routes are the same question,
  // what the result exposes, so both go through one predicate.
  //
  // This over-restricts members that return the receiver itself, `sort`, and
  // members that never construct, `reduce` accumulating into an array, since
  // neither is distinguishable here. Both directions fail closed.
  /**
   Instantiated result type of this call.
   */
  const resultType = checker.getTypeAtLocation(call,);
  if (resultType === undefined)
    return READONLY_VIEW_UNDISCHARGED;
  if (viewResultUnaccounted({
    project,
    checker,
    call,
    resultType,
    elementTypes,
    ...(body === undefined) ? {} : { body, },
  },))
    return READONLY_VIEW_UNDISCHARGED;
  /**
   Every argument paired with its resolved type.
   */
  const typedArguments = [...call.arguments,]
    .flatMap(function typedArgument(
      argument,
      argumentIndex,
    ): readonly TypedArgument[] {
      /**
       Argument type, absent when the checker cannot resolve it.
       */
      const argumentType = checker.getTypeAtLocation(argument,);
      return (argumentType === undefined)
        ? []
        : [{
          argument,
          argumentType,
          argumentIndex,
        },];
    },);
  /**
   Argument count at the call site.
   */
  const argumentCount = call.arguments
    .length;
  if (typedArguments.length !== argumentCount)
    return READONLY_VIEW_UNDISCHARGED;
  /**
   Arguments that are definitely callable at runtime.
   */
  const observers = typedArguments
    .filter(function callableArgument({ argumentType, },): boolean {
      return typeDefinitelyCallable({
        checker,
        type: argumentType,
      },);
    },);
  if (observers.length === 0)
    return READONLY_VIEW_UNDISCHARGED;
  // Anything else passed alongside the observers reaches the member by a route
  // the element-flow derivation does not describe, `map`'s `thisArg` being the
  // standing example, so state arriving that way leaves the call underived.
  /* "Arriving" is the whole of it. Asking whether the argument's type can carry state asked
   * something wider: a fold's `[]` seed is an object, so the type test said yes, while the
   * value holds nothing at the moment of the call and nothing the receiver owns can arrive
   * through it. What the observer puts in afterwards is the observer's effect. */
  if (typedArguments
    .some(function unobservedArgument({ argument, },): boolean {
      if (observers
        .some(function isObserver(observer,): boolean {
          return observer.argument === argument;
        },))
        return false;
      if (!expressionCanCarryMutableState({
        checker,
        node: argument,
      },))
        return false;
      /**
       Caller parameters this argument already holds when the call is made.
       */
      const arriving = expressionOrigins({
        project,
        bindingOriginBySymbolId,
        node: argument,
      },);
      return arriving.size > 0;
    },))
    return READONLY_VIEW_UNDISCHARGED;
  /**
   Owned declarations behind every observer argument.
   */
  const ownedObservers = observers
    .flatMap(function ownedObserver({
      argument,
      argumentType,
      argumentIndex,
    },): readonly OwnedObserver[] {
      /**
       Observer implementation, absent when it is not owned source.
       */
      const declaration = callableDeclaration({
        project,
        node: argument,
        ...(analysisRoot === undefined) ? {} : { analysisRoot, },
      },);
      return (declaration === OWNED_CALLABLE_UNAVAILABLE)
        ? []
        : [{
          argument,
          argumentType,
          argumentIndex,
          declaration,
        },];
    },);
  if (ownedObservers.length !== observers.length)
    return READONLY_VIEW_UNDISCHARGED;
  /**
   Positions the member hands receiver state to, per observer.
   */
  const observedPositions = ownedObservers
    .map(function positionsFor({ argumentIndex, },) {
      return observedParameterIndexes({
        checker,
        call,
        argumentIndex,
        elementTypes,
        receiverType,
      },);
    },);
  if (observedPositions
    .some(function undescribed(positions,): boolean {
      return positions === OBSERVED_POSITIONS_UNAVAILABLE;
    },))
    return READONLY_VIEW_UNDISCHARGED;
  return ownedObservers
    .map(function application(
      { declaration, },
      observerIndex,
    ): ElementApplication {
      /**
       Positions for this observer, already proven described above.
       */
      const positions = observedPositions[observerIndex];
      return {
        receiverSlot,
        callbackKey: callableKey(declaration,),
        observerParameterIndexes: (positions === undefined)
          || (positions === OBSERVED_POSITIONS_UNAVAILABLE)
          ? []
          : positions,
      };
    },);
}

/**
 Records element-flow relations for a read-only view call on a parameter.
 
 @param project - TypeScript project resolving observer declarations.
 
 @param checker - TypeScript checker resolving receiver and parameter types.
 
 @param bindingOriginBySymbolId - Current callable parameter and alias origins.
 
 @param call - Read-only view call expression.
 
 @param receiver - Receiver expression whose element origins are required.
 
 @param summary - Caller summary receiving derived relations.
 
 @param analysisRoot - Optional external implementation root.
 
 @returns whether call was fully derived and needs no opaque fallback.
 
 @mutates summary - Appends derived element-flow relations.
 
 @example
 ```ts
 recordReadonlyViewApplications({ project, checker, bindingOriginBySymbolId, call, receiver, summary });
 ```
 */
export function recordReadonlyViewApplications({
  project,
  checker,
  bindingOriginBySymbolId,
  call,
  receiver,
  summary,
  analysisRoot,
  body,
}: {
  readonly project: Project;
  readonly checker: Checker;
  readonly bindingOriginBySymbolId: ReadonlyMap<number, SlotOrigins>;
  readonly call: CallExpression;
  readonly receiver: Expression;
  readonly summary: MutableEffectSummary;
  readonly analysisRoot?: string;
  readonly body?: Node;
},): boolean {
  // The element question rather than the value question, and the two differ exactly
  // where this derivation is needed most. A receiver that is a container another
  // member built, `rows.filter(keep).reduce(fold, 0)` in either its chained or its
  // bound spelling, is a value no parameter holds, so asking which parameter owns it
  // answers nothing and this returns before deriving anything. The call then falls to
  // the receiver claim, which cannot answer for a member carrying an observer, and the
  // parameter stayed opaque for a fold that reads a length. Asking where the receiver's
  // elements came from answers the parameter, which is what the observer derivation is
  // about: the observer receives elements, never the container.
  //
  // Widening only, never narrowing: for a receiver that is a parameter the element walk
  // finds no declaration initializer to follow and falls back to exactly the value
  // origins, which is why every container fixture reads identically across the change.
  /**
   Caller parameters owning receiver elements, when receiver can carry mutable state.
   */
  const receiverOrigins = expressionCanCarryMutableState({
    checker,
    node: receiver,
  },)
    ? expressionElementOrigins({
      project,
      bindingOriginBySymbolId,
      node: receiver,
    },)
    : NO_SLOT_ORIGIN;
  if (receiverOrigins.size === 0)
    return false;
  /**
   Relations derived for every parameter the receiver can hold.
   */
  const derived: ElementApplication[] = [];
  /* One derivation per origin, accumulated before anything is recorded. Discharge
   * depends on the call and the receiver's type rather than on which parameter the
   * receiver came from, so these agree in practice; a disagreement must still leave
   * the whole call to the opaque boundary instead of recording a partial answer for
   * the origins that happened to derive. */
  for (const receiverSlot of receiverOrigins) {
    /**
     Derived relations for one origin, or sentinel when user code stays unproven.
     */
    const applications = readonlyViewElementApplications({
      project,
      checker,
      bindingOriginBySymbolId,
      call,
      receiver,
      receiverSlot,
      ...(body === undefined) ? {} : { body, },
      ...(analysisRoot === undefined) ? {} : { analysisRoot, },
    },);
    if (applications === READONLY_VIEW_UNDISCHARGED)
      return false;
    applications.forEach(function collect(application,): void {
      derived.push(application,);
    },);
  }
  derived.forEach(function record(application,): void {
    summary.elementApplications
      .push(application,);
  },);
  return true;
}
