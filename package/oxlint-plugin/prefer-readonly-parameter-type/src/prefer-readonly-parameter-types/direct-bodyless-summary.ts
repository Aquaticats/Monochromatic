/**
 Direct effects of a callable that declares a signature and no body.
 
 Split out of `direct-effect-summary.ts` for the code-line budget. It is a self-contained
 branch: a bodyless declaration has nothing to walk, so everything it can say comes from its
 parameter types and its authored contracts.
 
 @module
 */

import type { Checker, } from 'typescript/sync';

import { addOpaqueEffect, } from './effect-call-resolution.ts';
import { expressionCanCarryMutableState, } from './effect-primitive-origin.ts';
import { asEffectSlot, } from './effect-slot-identity.ts';
import {
  addEffectSlot,
  callableKey,
  type EffectCallableDeclaration,
  EFFECT_SLOT_UNAVAILABLE,
  type MutableEffectSummary,
} from './effect-summary-model.ts';
import {
  MUTATION_CONTRACT_UNAVAILABLE,
  mutationContractsForDeclaration,
  mutationTargetIndexes,
} from './mutation-contract-query.ts';

/**
 Records everything a bodyless callable can be held to.
 
 Every parameter that can carry mutable state takes opacity, because no body proves what the
 implementation does with it. Authored contracts add known effects on top and never remove
 that opacity, which is the policy `direct-effect-summary.ts` states for contracts generally.
 
 @param checker - TypeScript checker classifying parameter types.
 
 @param declaration - Bodyless callable being summarized.
 
 @param summary - Summary receiving opacity and contract effects.
 
 @mutates summary - Adds bodyless opacity and authored contract mutations.
 
 @example
 ```ts
 recordBodylessEffects({ checker, declaration, summary });
 ```
 */
export function recordBodylessEffects({
  checker,
  declaration,
  summary,
}: {
  readonly checker: Checker;
  readonly declaration: EffectCallableDeclaration;
  readonly summary: MutableEffectSummary;
},): void {
  declaration.parameters
    .forEach(function rejectBodylessParameter(
      parameter,
      parameterIndex,
    ): void {
      if (!expressionCanCarryMutableState({
        checker,
        node: parameter.name,
      },))
        return;
      addOpaqueEffect({
        summary,
        affectedSlot: asEffectSlot(parameterIndex,),
        provenance: `bodyless callable ${callableKey(declaration,)}`,
      },);
    },);
  /**
   Authored bodyless mutation contracts remain documentation of known effects.
   They never remove unresolved implementation opacity.
   */
  const contracts = mutationContractsForDeclaration({
    declaration,
    sourceFile: declaration.getSourceFile(),
  },);
  if (contracts !== MUTATION_CONTRACT_UNAVAILABLE) {
    /**
     Contract target names mapped to source parameter indexes.
     */
    const targetIndexes = mutationTargetIndexes({
      declaration,
      sourceFile: declaration.getSourceFile(),
    },);
    /* An authored name resolves to a whole parameter, so it seeds that parameter's own slot
     * rather than any property slot beneath it. The whole slot is the wider claim of the two,
     * and a caller fills it with every origin the actual packages, so a contract naming a
     * destructured property still reaches every value that property could hold. Checking such
     * a name against the measured property facts is what per-property attribution finally
     * makes possible, and is deliberately not done here. */
    contracts.blocks
      .forEach(function seedContract(block,): void {
        /**
         Parameter this contract names, absent when the name matches none.
         */
        const named = targetIndexes.get(block.parameterName,);
        addEffectSlot({
          target: summary.mutated,
          value: named === undefined
            ? EFFECT_SLOT_UNAVAILABLE
            : asEffectSlot(named,),
        },);
      },);
  }
  summary.directOpaque
    .forEach(function seedBodylessOpacity(slot,): void {
      summary.opaque
        .add(slot,);
    },);
}
