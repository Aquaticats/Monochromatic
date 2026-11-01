/**
 Diagnostic comparison for readonly parameter types and mutation contracts.
 
 @module
 */

import type { ParsedMutationContractBlock, } from '@monochromatic-dev/oxlint-plugin-shared/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  Fixer,
} from '@oxlint/plugins';
import { MUTATION_CONTRACT_UNAVAILABLE, } from './mutation-contract-query.ts';
import { opaqueEffectReport, } from './opaque-effect-diagnostic.ts';
import type { ReadonlyCallableEvidence, } from './readonly-callable-evidence.ts';
import type { ReadonlyRuleCategory, } from './readonly-rule-category.ts';
import { reportRedundantForeignBorrowed, } from './redundant-marker-report.ts';
import { readonlyMutableReason, } from './readonly-classification-model.ts';
import { readonlyPreferenceGuidance, } from './readonly-preference-guidance.ts';
import { readonlyParameterSuggestions, } from './readonly-suggestions.ts';

import {
  oxlintOffset,
  semanticLocation,
} from './semantic-location.ts';

/**
 Reports stale mutation block with verified removal suggestion.
 
 @param context - Rule context receiving diagnostic.
 
 @param block - Shared parsed mutation block.
 
 @param parameterSubject - Stable one-line subject for targeted parameter.
 
 @param commentBodyStartOffset - Absolute source start of comment body.
 
 @mutates context - Emits Oxlint diagnostics through foreign rule context.
 */
function reportStaleContract({
  context,
  block,
  parameterSubject,
  commentBodyStartOffset,
}: ForeignBorrowed<{
  readonly context: Context;
  readonly block: ParsedMutationContractBlock;
  readonly parameterSubject: string;
  readonly commentBodyStartOffset: number;
}>,): void {
  /**
   Absolute Oxlint range for complete stale block.
   */
  const range: [
    number,
    number
  ] = [
    oxlintOffset({
      offset: commentBodyStartOffset + block.blockStartOffset,
      hasBOM: context.sourceCode
        .hasBOM,
    },),
    oxlintOffset({
      offset: commentBodyStartOffset + block.blockEndOffset,
      hasBOM: context.sourceCode
        .hasBOM,
    },),
  ];
  /**
   Replacement preserving closing-comment indentation for final block.
   */
  const replacement = context.sourceCode
    .text
    .startsWith(
    '*/',
    range[1],
    )
    ? ' '
    : '';
  context.report({
    node: context.sourceCode
      .ast,
    loc: semanticLocation({
      context,
      start: commentBodyStartOffset + block.blockStartOffset,
      end: commentBodyStartOffset + block.blockEndOffset,
    },),
    messageId: 'staleMutatesTag',
    data: { parameterSubject, },
    suggest: [
      {
        desc: `Remove stale @mutates ${block.parameterName} block.`,
        fix(fixer: ForeignBorrowed<Fixer>,): ReturnType<Fixer['replaceTextRange']> {
          return fixer.replaceTextRange(
            range,
            replacement,
          );
        },
      },
    ],
  },);
}

/**
 Reports one policy category from shared callable evidence.
 
 @param context - Rule context receiving selected diagnostics.
 
 @param evidence - Category-neutral facts for one callable.
 
 @param category - Public rule policy selecting applicable facts.
 
 @example
 ```ts
 reportReadonlyCallableEvidence({ context, evidence, category: 'preference' });
 ```
 
 @mutates context - Emits selected diagnostics through foreign rule context.
 */
export function reportReadonlyCallableEvidence({
  context,
  evidence,
  category,
}: ForeignBorrowed<{
  readonly context: Context;
  readonly evidence: ReadonlyCallableEvidence;
  readonly category: ReadonlyRuleCategory;
}>,): void {
  /**
   Shared semantic values used by every category reporter.
   */
  const {
    declaration,
    project,
    contracts,
    targetIndexes,
    parameterFacts,
    foreignBorrowedParameters,
  } = evidence;
  /**
   Source file owning callable and authored parameter ranges.
   */
  const sourceFile = declaration.getSourceFile();

  parameterFacts.forEach(function reportParameter(facts,): void {
    /**
     Parameter-level facts consumed by category branches.
     */
    const {
      parameter,
      parameterIndex,
      parameterSubject,
      inputSubject,
      affectedNames,
      parameterType,
      classification,
      originEvidence,
      parameterBlocks,
      opaque,
      retained,
      acceptedHostOpacity,
      affected,
      provedMutation,
      mutated,
      uncertainty,
      foreignHostCapability,
      redundantMarkerPossible,
    } = facts;
    /**
     Whether exact ownership-marker provenance reaches current parameter.
     */
    const foreignBorrowed = foreignBorrowedParameters.has(parameterIndex,);
    /**
     Report location spanning parameter binding.
     */
    const loc = semanticLocation({
      context,
      start: parameter.name
        .getStart(sourceFile,),
      end: parameter.name
        .end,
    },);

    if (category === 'preference') {
      if (((!opaque) || acceptedHostOpacity)
        && (!mutated)
        && (!retained)
        && (!foreignBorrowed)
        && (classification.kind === 'mutable')) {
        /**
         Verified semantic type suggestions available for current syntax.
         */
        const suggestions = readonlyParameterSuggestions({
          context,
          parameter,
          project,
        },);
        context.report({
          node: context.sourceCode
            .ast,
          loc,
          messageId: 'shouldBeReadonly',
          data: {
            parameterSubject,
            reason: readonlyMutableReason(classification,),
            guidance: readonlyPreferenceGuidance({
              suggestions,
              originEvidence,
              classification,
            },),
          },
          ...suggestions.length === 0 ? {} : { suggest: suggestions, },
        },);
      }
      return;
    }

    if (category === 'mutation') {
      if (provedMutation
        && (!foreignBorrowed)
        && ((classification.kind === 'deep-readonly')
          || (classification.kind === 'projected-readonly-capability'))) {
        context.report({
          loc,
          messageId: 'readonlyParameterMutation',
          data: {
            parameterSubject,
            reason: classification.kind === 'projected-readonly-capability'
              ? `{classification.reason}; caller-reachable state is mutated`
              : 'caller-reachable state is mutated',
          },
        },);
      }
      return;
    }

    if (category === 'opaque-effect') {
      if (opaque && (!acceptedHostOpacity)) {
        context.report(opaqueEffectReport({
          loc,
          inputSubject,
          targetIndexes,
          parameterIndex,
          uncertainty,
          ...(affectedNames === undefined) ? {} : { affectedNames, },
        },),);
      }
      else if (classification.kind === 'projected-readonly-capability') {
        context.report({
          loc,
          messageId: 'projectedCallableCapability',
          data: {
            parameterSubject,
            reason: classification.reason,
          },
        },);
      }
      return;
    }

    if (opaque
      && foreignHostCapability
      && (parameterBlocks.length === 0)) {
      context.report({
        loc,
        messageId: 'hostCapabilityContractRequired',
        data: { parameterSubject, },
      },);
    }
    if ((!opaque)
      && (!affected)
      && (contracts !== MUTATION_CONTRACT_UNAVAILABLE)) {
      parameterBlocks.forEach(function stale(block,): void {
        reportStaleContract({
          context,
          block,
          parameterSubject,
          commentBodyStartOffset: contracts.commentBodyStartOffset,
        },);
      },);
    }
    if (redundantMarkerPossible && foreignBorrowed) {
      reportRedundantForeignBorrowed({
        context,
        project,
        parameterType,
        parameterSubject,
        loc,
      },);
    }
  },);
}
