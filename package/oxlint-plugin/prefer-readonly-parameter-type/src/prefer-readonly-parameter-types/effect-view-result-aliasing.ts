/**
 Whether a read-only view call's result is receiver state or something newly built.
 
 Split out of `effect-readonly-view-application.ts` for the code-line budget. It answers one
 type-level question and captures nothing, which is what let it move without threading state
 through a new parameter list.
 
 @module
 */

import type {
  Checker,
  Type,
} from 'typescript/sync';

import { typeCanCarryMutableState, } from './effect-primitive-origin.ts';

/**
 Tests whether a call result is receiver state rather than something newly built.
 
 Matches by `Type` identity against the receiver's instantiated element types,
 because the member's signature is instantiated with those arguments, so receiver
 state appearing in a result is the identical instance rather than an equivalent
 one. Only state-carrying matches count, since sharing a primitive element type
 exposes nothing: `readonly string[]` filtered to `string[]` copies primitives.
 
 Receiver identity is deliberately not matched. A member returning the receiver
 itself, `sort` and `reverse`, is a mutator, so the structural claim has already
 recorded the mutation and nothing reachable through the result is new. Matching it
 here regressed `mutableSortObserverEffect` from a derived mutation to a mutation
 plus an opaque boundary, which is the rule saying it cannot tell about a call it
 fully understands.
 
 @param checker - TypeScript checker resolving result type arguments.
 
 @param resultType - Instantiated result type of one call.
 
 @param elementTypes - Types the receiver collection is instantiated over.
 
 @returns whether result may alias caller-owned receiver state.
 
 @example
 ```ts
 resultAliasesReceiverState({ checker, resultType, elementTypes, });
 ```
 */
export function resultAliasesReceiverState({
  checker,
  resultType,
  elementTypes,
}: {
  readonly checker: Checker;
  readonly resultType: Type;
  readonly elementTypes: readonly Type[];
},): boolean {
  /**
   Result itself plus anything a constructed result holds.
   */
  const resultCandidates = [
    resultType,
    ...resultType.isTypeReference()
      ? checker.getTypeArguments(resultType,)
      : [],
  ];
  return resultCandidates
    .some(function aliasesState(candidate,): boolean {
      return elementTypes.includes(candidate,)
        && typeCanCarryMutableState({
          checker,
          type: candidate,
        },);
    },);
}
