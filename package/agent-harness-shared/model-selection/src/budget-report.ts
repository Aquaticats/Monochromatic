/**
 Fast judge-model candidate reporting and failure error type.
 
 @module
 */

import { scoreModelSpeed, } from './speed-signals.ts';
import type {
  BudgetModelCandidate,
  ModelIdentity,
  ModelPricing,
} from './types.ts';

//region Error class

/**
 Error thrown when no suitable judge model can be found.
 
 Includes the best candidates from the active provider and across all
 providers for custom fallback logic.
 
 @example
 ```typescript
 throw new NoBudgetModelError('no active model set');
 ```
 */
export class NoBudgetModelError extends Error {
  /**
   Why no budget model was found.
   */
  readonly reason: string;
  /**
   Best candidate from same provider, when one was found.
   */
  readonly sameProvider?: BudgetModelCandidate;
  /**
   Fastest candidate across all providers, when one was found.
   */
  readonly fastestOverall?: BudgetModelCandidate;

  /**
   Construct a NoBudgetModelError.
   
   @param reason - why no budget model was found
   
   @param candidates - optional context about candidates found
   */
  constructor(
    reason: string,
    candidates: {
      readonly sameProvider?: BudgetModelCandidate;
      readonly fastestOverall?: BudgetModelCandidate;
    } = {},
  ) {
    /**
     Per-line accumulator for multi-line error message.
     */
    const lines = [
      "Tried to auto-detect a fast judge model for a background task, but couldn't find one.",
      `Reason: ${reason}`,
    ];
    if (candidates.sameProvider
      !== undefined) {
      /**
       Local alias so template strings stay readable.
       */
      const candidate = candidates.sameProvider;
      /**
       Candidate line describing the best same-provider option.
       */
      const sameProviderLine = [
        `Best same-provider option: ${candidate.provider}/${candidate.modelId}`,
        `(speed ${candidate.speedScore};`,
        `$${candidate.costInput}/$${candidate.costOutput} per M tokens)`,
      ]
        .join(' ',);
      lines.push(sameProviderLine,);
    }
    /**
     Fastest-overall candidate, surfaced only when present with an API key.
     */
    const fastest = candidates.fastestOverall;
    if (fastest?.hasApiKey
      === true) {
      /**
       Candidate line describing the fastest authenticated option.
       */
      const fastestLine = [
        `Fastest with API key: ${fastest.provider}/${fastest.modelId}`,
        `(speed ${fastest.speedScore};`,
        `$${fastest.costInput}/$${fastest.costOutput} per M tokens)`,
      ]
        .join(' ',);
      lines.push(fastestLine,);
    }
    lines.push(
      [
        'To fix: configure a model explicitly in the extension settings,',
        'or switch to a provider with faster models.',
      ].join(' ',),
    );

    super(lines.join('\n',),);
    this.name = 'NoBudgetModelError';
    this.reason = reason;
    if (candidates.sameProvider
      !== undefined)
      this.sameProvider = candidates.sameProvider;
    if (candidates.fastestOverall
      !== undefined)
      this.fastestOverall = candidates.fastestOverall;
  }
}

//endregion Error class

//region Candidate helpers

/**
 Build budget-model candidate metadata from a model.
 
 @param model - model to report
 
 @param hasConfiguredAuth - whether host registry reports usable auth
 
 @returns model candidate metadata
 
 @example
 ```typescript
 toBudgetModelCandidate({ model, hasConfiguredAuth: true });
 ```
 */
export function toBudgetModelCandidate(
  {
    model,
    hasConfiguredAuth,
  }: {
    readonly model: ModelPricing;
    readonly hasConfiguredAuth: boolean;
  },
): BudgetModelCandidate {
  return {
    provider: model.provider,
    modelId: model.id,
    speedScore: scoreModelSpeed(model,),
    costInput: model.cost
      .input,
    costOutput: model.cost
      .output,
    hasApiKey: hasConfiguredAuth,
  };
}

/**
 Return canonical provider/model slug for a model identity.
 
 @param model - model identity
 
 @returns provider/model slug
 
 @example
 ```typescript
 budgetModelSlug(model);
 ```
 */
export function budgetModelSlug(
  model: ModelIdentity,
): string {
  return `${model.provider}/${model.id}`;
}

//endregion Candidate helpers
