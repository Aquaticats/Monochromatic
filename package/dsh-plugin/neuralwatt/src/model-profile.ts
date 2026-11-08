/**
 Model profile projection for the NeuralWatt provider: shared listing facts
 onto the `PiAiModelProfile` shape the shipped pi-ai adapter serves. Pricing
 is deliberately absent here: this plugin serves models, it does not estimate
 cost, and a pi-side projection owns the cost fields. @module
 */

import type {
  PiAiModelProfile,
} from '@deepseek-ai/dsh-llm-pi-ai';
import type {
  NeuralWattModelFacts,
} from '@monochromatic-dev/agent-harness-shared-neuralwatt-listing/ts';

//region Public API

/**
 Project one set of shared listing facts onto the served model profile:
 every field the facts disclose and none they do not, so an unsized capacity
 stays unset instead of gaining a guess. The projection is inert toward
 pricing, keeping the written route byte-identical to the pre-extraction
 mapping whatever the snapshot's pricing block carries.

  @param facts - shared listing facts one entry mapped to

  @returns model profile ready for the provider dict

  @example
  ```ts
  const profile = factsToPiAiModelProfile({ facts: mapNeuralWattListing({
    listing: neuralWattListingSnapshot,
  })[0]! });
  ```
  */
export function factsToPiAiModelProfile({
  facts,
}: {
  readonly facts: NeuralWattModelFacts;
}): PiAiModelProfile {
  return {
    id: facts.id,
    name: facts.name,
    ...(facts.contextWindow === undefined ? {} : { contextWindow: facts.contextWindow }),
    ...(facts.maxTokens === undefined ? {} : { maxTokens: facts.maxTokens }),
    input: [
      ...facts.input,
    ],
    ...(facts.reasoningEfforts === undefined
      ? {}
      : { reasoningEfforts: facts.reasoningEfforts }),
  };
}

//endregion Public API
