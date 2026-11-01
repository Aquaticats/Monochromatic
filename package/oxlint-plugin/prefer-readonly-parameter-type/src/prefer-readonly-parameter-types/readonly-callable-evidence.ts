import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { EffectCallableDeclaration, } from './effect-summary-model.ts';
import type { CallableEffectSummary, } from './effect-summaries.ts';
import type { ParameterIndex, } from './effect-slot-identity.ts';
import {
  MUTATION_CONTRACT_UNAVAILABLE,
  mutationContractsForDeclaration,
  mutationTargetIndexes,
} from './mutation-contract-query.ts';
import type { classifyReadonlyType, } from './readonly-classifier.ts';
import {
  factsNeedForeignProof,
  readonlyParameterFacts,
  type ReadonlyParameterFacts,
} from './readonly-parameter-facts.ts';

/**
 Category-neutral evidence for one implemented callable.
 
 @example
 ```ts
 evidence.parameterFacts.forEach(useFacts);
 ```
 */
export type ReadonlyCallableEvidence = {
  readonly declaration: EffectCallableDeclaration;
  readonly project: Parameters<typeof classifyReadonlyType>[0]['project'];
  readonly contracts: ReturnType<typeof mutationContractsForDeclaration>;
  readonly targetIndexes: ReturnType<typeof mutationTargetIndexes>;
  readonly parameterFacts: readonly ReadonlyParameterFacts[];
  readonly foreignBorrowedParameters: ReadonlySet<ParameterIndex>;
};

/**
 Computes every category-neutral fact for one implemented callable.
 
 @param declaration - Callable declaration owning parameter evidence.
 
 @param effectSummary - Whole-project effects reaching callable parameters.
 
 @param project - TypeScript project used by readonly classification.
 
 @param proveForeignBorrowed - Deferred complete ownership proof for callable.
 
 @returns immutable evidence consumed by split rule reporters.
 
 @example
 ```ts
 readonlyCallableEvidence({ declaration, effectSummary, project, proveForeignBorrowed });
 ```
 */
export function readonlyCallableEvidence({
  declaration,
  effectSummary,
  project,
  proveForeignBorrowed,
}: ForeignBorrowed<{
  readonly declaration: EffectCallableDeclaration;
  readonly effectSummary: CallableEffectSummary;
  readonly project: Parameters<typeof classifyReadonlyType>[0]['project'];
  readonly proveForeignBorrowed: () => ReadonlySet<ParameterIndex>;
}>,): ReadonlyCallableEvidence {
  /**
   Source file owning callable and authored comments.
   */
  const sourceFile = declaration.getSourceFile();
  /**
   Attached mutation contracts, when callable has TSDoc.
   */
  const contracts = mutationContractsForDeclaration({
    declaration,
    sourceFile,
  },);
  /**
   Valid contract targets mapped to parameter indexes.
   */
  const targetIndexes = mutationTargetIndexes({
    declaration,
    sourceFile,
  },);
  /**
   Parsed mutation blocks, absent when callable has no TSDoc.
   */
  const blocks = contracts === MUTATION_CONTRACT_UNAVAILABLE ? [] : contracts.blocks;
  /**
   Mutation blocks grouped by semantic parameter index.
   */
  const blocksByParameter = new Map<number, typeof blocks>();
  blocks.forEach(function groupBlock(block,): void {
    /**
     Parameter index matching authored target.
     */
    const parameterIndex = targetIndexes.get(block.parameterName,);
    if (parameterIndex === undefined)
      return;
    blocksByParameter.set(
      parameterIndex,
      [
        ...blocksByParameter.get(parameterIndex,) ?? [],
        block,
      ],
    );
  },);
  /**
   Everything each split policy reads before foreign ownership.
   */
  const parameterFacts = readonlyParameterFacts({
    declaration,
    effectSummary,
    project,
    targetIndexes,
    blocksByParameter,
  },);
  /**
   Parameters whose exact marker provenance reaches foreign ownership.
   */
  const foreignBorrowedParameters = parameterFacts
    .some(function verdictReadsForeign(facts: ReadonlyParameterFacts,): boolean {
      return factsNeedForeignProof(facts,);
    },)
    ? proveForeignBorrowed()
    : new Set<ParameterIndex>();

  return {
    declaration,
    project,
    contracts,
    targetIndexes,
    parameterFacts,
    foreignBorrowedParameters,
  };
}
