/**
 Projection of shared NeuralWatt listing facts onto the pi model shape
 `@earendil-works/pi-ai` 1.5.0 registers.

 The projection reads whatever the facts carry and never re-decides listing
 policy:
 every fallback here covers a pi type requirement the listing facts leave
 open,
 so a change to the shared mapping or to the owner's snapshot decisions lands
 without rework here.

 @module
 */

import {
  NEURALWATT_API,
  NEURALWATT_BASE_URL,
  NEURALWATT_ROUTE,
  expandThinkingLevelMap,
  type NeuralWattEffortsDict,
  type NeuralWattModelFacts,
  type NeuralWattModelPricing,
  type NeuralWattReasoningEfforts,
} from '@monochromatic-dev/agent-harness-shared-neuralwatt-listing/ts';
import type {
  Model,
  ModelCost,
  OpenAICompletionsCompat,
} from '@earendil-works/pi-ai';
import {
  NEURALWATT_COST_RATE_ZERO,
  PI_SCHEMA_DEFAULT_CONTEXT_WINDOW,
  PI_SCHEMA_DEFAULT_MAX_TOKENS,
} from './constants.ts';

//region Types

/**
 pi chat model one listing entry earns,
 pinned to the openai-completions api the route serves.
 */
export type NeuralWattPiModel = Model<typeof NEURALWATT_API>;

//endregion Types

//region Compat switches

/**
 Compat switches the route pins on every projected model.
 `supportsDeveloperRole: false` keeps the system prompt on role `system`
 because `mimo-v2.6-pro` answers `400 unsupported_message_role` to
 `developer`
 (`doc/troubleshooting/neuralwatt-rejects-developer-role.md`);
 the effort pins keep `reasoning_effort` on its documented top-level parameter
 (`package/dsh-plugin/neuralwatt/README.md`, "Thinking level dispatch").
 */
export const NEURALWATT_OPENAI_COMPLETIONS_COMPAT = {
  supportsDeveloperRole: false,
  supportsReasoningEffort: true,
  thinkingFormat: 'openai',
} as const satisfies OpenAICompletionsCompat;

//endregion Compat switches

//region Efforts narrowing

/**
 The offered efforts dict one facts entry may carry,
 or absence when the entry declares nothing.

 @param reasoningEfforts - reasoning declaration read from the facts

 @returns the dict when the facts declare one,
 `undefined` for the non-reasoning declaration and for a silent entry
 */
export function effortsDictOf({
  reasoningEfforts,
}: {
  readonly reasoningEfforts: NeuralWattReasoningEfforts | undefined;
},): NeuralWattEffortsDict | undefined {
  if (reasoningEfforts === undefined)
    return undefined;
  if (typeof reasoningEfforts === 'boolean')
    return undefined;
  return reasoningEfforts;
}

//endregion Efforts narrowing

//region Cost mapping

/**
 Price one model serves in pi's `ModelCost` shape.
 Every rate the listing leaves null becomes {@link NEURALWATT_COST_RATE_ZERO},
 the recorded acceptance for a schema with no unknown marker.

 @param pricing - per-million pricing the facts carry

 @returns the four cost rates pi's model shape requires
 */
export function mapNeuralWattModelCost({
  pricing,
}: {
  readonly pricing: NeuralWattModelPricing | undefined;
},): ModelCost {
  return {
    input: pricing?.inputPerMillion ?? NEURALWATT_COST_RATE_ZERO,
    output: pricing?.outputPerMillion ?? NEURALWATT_COST_RATE_ZERO,
    cacheRead: pricing?.cachedInputPerMillion ?? NEURALWATT_COST_RATE_ZERO,
    cacheWrite: pricing?.cachedOutputPerMillion ?? NEURALWATT_COST_RATE_ZERO,
  };
}

//endregion Cost mapping

//region Model projection

/**
 Project one facts entry onto the pi model shape.
 Limits the listing omits fall back to pi's documented schema defaults,
 and the thinking map is the shared expansion so DSH and pi dispatch
 identically.

 @param facts - serviceable model facts one listing entry earned

 @returns the pi chat model the provider serves
 */
export function projectNeuralWattModel({
  facts,
}: {
  readonly facts: NeuralWattModelFacts;
},): NeuralWattPiModel {
  /**
   Offered efforts dict, or absence for a non-reasoning or silent entry.
   */
  const efforts = effortsDictOf({ reasoningEfforts: facts.reasoningEfforts, },);
  return {
    id: facts.id,
    name: facts.name,
    api: NEURALWATT_API,
    provider: NEURALWATT_ROUTE,
    baseUrl: NEURALWATT_BASE_URL,
    input: [...facts.input],
    cost: mapNeuralWattModelCost({ pricing: facts.pricing, },),
    reasoning: efforts !== undefined,
    contextWindow: facts.contextWindow ?? PI_SCHEMA_DEFAULT_CONTEXT_WINDOW,
    maxTokens: facts.maxTokens ?? PI_SCHEMA_DEFAULT_MAX_TOKENS,
    ...(efforts === undefined ? {} : {
      thinkingLevelMap: expandThinkingLevelMap({ efforts, },),
    },),
    compat: NEURALWATT_OPENAI_COMPLETIONS_COMPAT,
  };
}

/**
 Project one facts list onto the pi catalog the provider serves.

 @param facts - serviceable model facts the listing mapping produced

 @returns pi chat models in listing order
 */
export function projectNeuralWattModels({
  facts,
}: {
  readonly facts: readonly NeuralWattModelFacts[];
},): NeuralWattPiModel[] {
  return facts.map(function toPiModel(one,) {
    return projectNeuralWattModel({ facts: one, },);
  },);
}

//endregion Model projection
