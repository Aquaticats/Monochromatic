/**
 External callable effect mapping to caller parameter origins.
 
 @module
 */

import type {
  CallExpression,
  Node,
} from 'typescript/ast';

import { addOpaqueEffect, } from './effect-call-resolution.ts';
import { formalActualPositions, } from './effect-formal-actual-mapping.ts';
import type { EffectSlot, } from './effect-slot-identity.ts';
import {
  addEffectSlot,
  isEffectCallableDeclaration,
  type MutableEffectSummary,
} from './effect-summary-model.ts';
import type { ExternalCallableEffect, } from './external-callable-effect.ts';

/**
 Applies inferred external callable effects to caller summary.
 
 @param externalEffect - Proven external implementation effect.
 
 @param declaration - Declaration the consumer resolved, whose formals order the arguments.
 
 @param call - Call whose arguments feed those formals.
 
 @param allArgumentIndexes - Caller parameter origins by call **argument** position.
 
 @param summary - Caller summary receiving mapped effects.
 
 @mutates summary - Adds proven mutation,
 invocation,
 and unresolved transitive effects.
 
 @example
 ```ts
 applyExternalEffect({ externalEffect, declaration, call, allArgumentIndexes, summary });
 ```
 */
export function applyExternalEffect({
  externalEffect,
  declaration,
  call,
  allArgumentIndexes,
  summary,
}: {
  readonly externalEffect: ExternalCallableEffect;
  readonly declaration: Node;
  readonly call: CallExpression;
  readonly allArgumentIndexes: readonly (readonly EffectSlot[])[];
  readonly summary: MutableEffectSummary;
}): void {
  /* Mapped once, here, so no lookup below can index an actual-position array with a formal
   * position. That was the previous shape of this function and it dropped facts rather than
   * inventing them: `external(...tuple,)` left a proven mutation of a later formal reading an index
   * that did not exist, and a rest formal charged the first actual only. Both directions kept an
   * offer the external analyzer had already disproved.
   *
   * External summaries stay parameter-level deliberately: a slot number means nothing across a
   * project boundary, where the declaration a consumer resolves need not be the one the external
   * analyzer inspected. That is exactly why the mapping is built from the declaration the consumer
   * resolved, since that declaration is what ordered the arguments at this call. */
  /**
   Caller parameter origins by external formal position, empty when no formal list could be read.
   */
  const argumentIndexes = formalArgumentIndexes({
    declaration,
    call,
    allArgumentIndexes,
  },);
  /**
   Origins charged for a formal this mapping could not place, which is every argument's.
   */
  const unplacedOrigins = distinctSlots(allArgumentIndexes.flat(),);
  externalEffect.summary
    .referentMutatedParameterIndexes
    .forEach(function externalMutation(parameter,): void {
      (argumentIndexes[parameter] ?? unplacedOrigins)
        .forEach(function callerMutation(index,): void {
          addEffectSlot({
            target: summary.directMutated,
            value: index,
          },);
        },);
    },);
  externalEffect.summary
    .invokedParameterIndexes
    .forEach(function externalInvocation(parameter,): void {
      (argumentIndexes[parameter] ?? unplacedOrigins)
        .forEach(function callerInvocation(index,): void {
          addEffectSlot({
            target: summary.directInvoked,
            value: index,
          },);
        },);
    },);
  externalEffect.summary
    .callbackRelations
    .forEach(function externalCallbackRelation(relation,): void {
      (argumentIndexes[relation.sourceParameterIndex] ?? unplacedOrigins)
        .forEach(function callbackSourceUncertainty(index,): void {
          addOpaqueEffect({
            summary,
            affectedSlot: index,
            provenance: `${externalEffect.provenance} callback argument ${String(relation.callbackArgumentPosition,)}`,
          },);
        },);
    },);
  externalEffect.summary
    .opaqueParameterIndexes
    .forEach(function externalUncertainty(parameter,): void {
      (argumentIndexes[parameter] ?? unplacedOrigins)
        .forEach(function callerUncertainty(index,): void {
          addOpaqueEffect({
            summary,
            affectedSlot: index,
            provenance: externalEffect.provenance,
          },);
        },);
    },);
}

/**
 Maps caller origins from argument positions onto external formal positions.
 
 The owned call edge has had this since rest and spread were mapped for it, and the external path
 never did. `formalActualPositions` answers it for a readable formal list, covering a `this`
 parameter, a rest formal collecting everything from its own position onward, and every formal at or
 past a spread, where positional correspondence is gone.
 
 Answers empty when the declaration has no readable formals, and leaves a formal past the end of
 its answer for the caller to charge with every argument's origins. Both cases arise for real: the
 analysed implementation need not be the declaration the consumer resolved, so a summary can name a
 formal this declaration does not declare.
 
 Over-approximating is the safe direction. Charging an argument that was not affected costs an
 offer; failing to charge one that was affected keeps an offer that is false.
 
 Exported for its own test. No shape in the fixture corpus reaches this through a diagnostic,
 because doing so needs an installed package with a locked version whose shipped implementation
 provably mutates a formal, invoked with a spread. A mutant restoring the actual-position indexing
 survived the whole suite, which is what exporting this answers.
 
 @param declaration - Declaration the consumer resolved.
 
 @param call - Call whose arguments feed the formals.
 
 @param allArgumentIndexes - Caller parameter origins by argument position.
 
 @returns caller origins by formal position.
 
 @example
 ```ts
 formalArgumentIndexes({ declaration, call, allArgumentIndexes });
 ```
 */
export function formalArgumentIndexes({
  declaration,
  call,
  allArgumentIndexes,
}: {
  readonly declaration: Node;
  readonly call: CallExpression;
  readonly allArgumentIndexes: readonly (readonly EffectSlot[])[];
},): readonly (readonly EffectSlot[])[] {
  if (!isEffectCallableDeclaration(declaration,))
    return [];
  return formalActualPositions({
    callee: declaration,
    call,
  },)
    .map(function originsForFormal(positions,): readonly EffectSlot[] {
      return distinctSlots(positions.flatMap(
        function originsAtPosition(position,): readonly EffectSlot[] {
          return allArgumentIndexes[position] ?? [];
        },
      ),);
    },);
}

/**
 Removes repeated slots while keeping first-seen order.
 
 @param slots - Slots that may repeat, since one formal can receive several actuals.
 
 @returns slots without repeats.
 
 @example
 ```ts
 distinctSlots(slots);
 ```
 */
function distinctSlots(slots: readonly EffectSlot[],): readonly EffectSlot[] {
  return [...new Set(slots,),];
}
