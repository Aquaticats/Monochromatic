/**
 Projection of shared Qwen Token Plan Individual listing facts onto the pi
 model shape `@earendil-works/pi-ai` 1.5.2 registers.

 The projection reads whatever the resolved facts carry and never re-decides
 listing policy:
 every value here is a straight copy of the shared core's resolved facts
 plus the two pi row fields those facts leave open (`type` and `inputLimits`),
 both pinned to the builtin catalog's own rows.
 A change to the shared mapping or to the owner's facts decisions lands
 without rework here.

 @module
 */

import {
  QWEN_TOKEN_PLAN_INDIVIDUAL_API,
  QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE,
  expandQwenTokenPlanThinkingLevelMap,
  type QwenTokenPlanEffortsDict,
  type QwenTokenPlanResolvedModelFacts,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/ts';
import type {
  Model,
  ModelCost,
  ModelInputLimits,
} from '@earendil-works/pi-ai';
import {
  PI_SCHEMA_DEFAULT_CONTEXT_WINDOW,
  PI_SCHEMA_DEFAULT_MAX_TOKENS,
  QWEN_TOKEN_PLAN_COST_RATE_ZERO,
  QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL,
  QWEN_TOKEN_PLAN_INDIVIDUAL_PROVIDER_NAME,
} from './constants.ts';

//region Types

/**
 pi chat model one listing entry earns,
 pinned to the openai-completions api the route serves.
 */
export type QwenTokenPlanIndividualPiModel =
  Model<typeof QWEN_TOKEN_PLAN_INDIVIDUAL_API>;

//endregion Types

//region Input limits

/**
 Image input limits every image-input row pins.
 Every image-input row of the builtin catalog
 (`@earendil-works/pi-ai` 1.5.2 `dist/providers/data/qwen-token-plan-individual.json`)
 and of its sibling `qwen-token-plan.json` carries exactly this block,
 so the projection copies it wherever the resolved facts name `image` input
 and omits `inputLimits` where they do not,
 matching the text-only rows byte for byte.
 */
export const QWEN_TOKEN_PLAN_INDIVIDUAL_IMAGE_INPUT_LIMITS: ModelInputLimits = {
  images: {
    resize: {
      maxWidth: 2_000,
      maxHeight: 2_000,
      maxBytes: 4_718_592,
      jpegQuality: 80,
    },
  },
};

//endregion Input limits

//region Efforts narrowing

/**
 Whether the resolved facts carry the offered efforts dict.
 A reasoning model with no dict projects as reasoning without a
 `thinkingLevelMap` (the builtin's own shape for that state),
 so this read decides which of the two reasoning shapes applies.

 @param facts - resolved model facts one listing entry earned

 @returns `true` only when the facts carry an efforts dict

 @example
 ```typescript
 if (hasResolvedEfforts(facts)) {
   // facts.efforts is the offered dict here
 }
 ```
 */
export function hasResolvedEfforts(
  facts: QwenTokenPlanResolvedModelFacts,
): facts is QwenTokenPlanResolvedModelFacts & {
  readonly efforts: QwenTokenPlanEffortsDict;
} {
  return facts.efforts !== undefined;
}

//endregion Efforts narrowing

//region Cost mapping

/**
 Price one model in pi's `ModelCost` shape.
 The facts table carries no pricing and every builtin row prices all four
 rates at zero,
 so the projection serves the zero rate named in `constants.ts`.

 @returns the four cost rates pi's model shape requires

 @example
 ```typescript
 const cost = mapQwenTokenPlanIndividualModelCost();
 ```
 */
export function mapQwenTokenPlanIndividualModelCost(): ModelCost {
  return {
    input: QWEN_TOKEN_PLAN_COST_RATE_ZERO,
    output: QWEN_TOKEN_PLAN_COST_RATE_ZERO,
    cacheRead: QWEN_TOKEN_PLAN_COST_RATE_ZERO,
    cacheWrite: QWEN_TOKEN_PLAN_COST_RATE_ZERO,
  };
}

//endregion Cost mapping

//region Model projection

/**
 Project one resolved facts entry onto the pi model shape.
 The three reasoning states project exactly as the builtin rows do:
 non-reasoning serves `reasoning: false`,
 reasoning with efforts serves the expanded `thinkingLevelMap`,
 and reasoning without efforts serves `reasoning: true` and no map.

 @param facts - resolved model facts one listing entry earned

 @returns the pi chat model the provider serves

 @example
 ```typescript
 const model = projectQwenTokenPlanIndividualModel({ facts });
 ```
 */
export function projectQwenTokenPlanIndividualModel({
  facts,
}: {
  readonly facts: QwenTokenPlanResolvedModelFacts;
},): QwenTokenPlanIndividualPiModel {
  /**
   Fields every projected model carries, independent of the reasoning state;
   typed so the literal wire protocol survives object widening.
   */
  const base: Omit<QwenTokenPlanIndividualPiModel, 'reasoning'> = {
    id: facts.id,
    name: facts.name,
    api: QWEN_TOKEN_PLAN_INDIVIDUAL_API,
    provider: QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE,
    baseUrl: QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL,
    input: [...facts.input],
    cost: mapQwenTokenPlanIndividualModelCost(),
    contextWindow: facts.contextWindow ?? PI_SCHEMA_DEFAULT_CONTEXT_WINDOW,
    maxTokens: facts.maxTokens ?? PI_SCHEMA_DEFAULT_MAX_TOKENS,
    compat: facts.compat,
    ...(facts.input
      .includes('image')
      ? { inputLimits: QWEN_TOKEN_PLAN_INDIVIDUAL_IMAGE_INPUT_LIMITS, }
      : {}),
    type: 'chat',
  };
  if (hasResolvedEfforts(facts))
    return {
      ...base,
      reasoning: true,
      thinkingLevelMap: expandQwenTokenPlanThinkingLevelMap({
        efforts: facts.efforts,
      },),
    };
  if (facts.reasoning)
    return {
      ...base,
      reasoning: true,
    };
  return {
    ...base,
    reasoning: false,
  };
}

/**
 Project one resolved facts list onto the pi catalog the provider serves.

 @param facts - resolved model facts the listing mapping produced

 @returns pi chat models in listing order

 @example
 ```typescript
 const models = projectQwenTokenPlanIndividualModels({
   facts: mapQwenTokenPlanIndividualModels({ listing }),
 });
 ```
 */
export function projectQwenTokenPlanIndividualModels({
  facts,
}: {
  readonly facts: readonly QwenTokenPlanResolvedModelFacts[];
},): QwenTokenPlanIndividualPiModel[] {
  return facts.map(function toPiModel(one,) {
    return projectQwenTokenPlanIndividualModel({ facts: one, },);
  },);
}

//endregion Model projection
