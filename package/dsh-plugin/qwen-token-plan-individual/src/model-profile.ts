/**
 Model profile projection for the Qwen Token Plan Individual provider: shared
 resolved listing facts onto the `PiAiModelProfile` shape the shipped pi-ai
 adapter serves. Pricing is deliberately absent here: this plugin serves
 models, it does not estimate cost. @module
 */

import type {
  PiAiModelProfile,
} from '@deepseek-ai/dsh-llm-pi-ai';
import type {
  QwenTokenPlanResolvedModelFacts,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/ts';

//region Public API

/**
 Project one set of resolved shared facts onto the served model profile: a
 straight copy of every field the facts decided, so the shared core's fallback
 walk (facts row, then older siblings, then defaults) is the only place model
 knowledge lives. The three reasoning states project exactly as the profile
 vocabulary states them: `false` for a non-reasoning model, the declared
 efforts dict for a model with selectable efforts, and an omitted field for a
 model that reasons without a selector, which then inherits the installed
 catalog entry's capability (and, for an id no catalog row covers, reads as
 "does not reason" in this harness).

  @param facts - resolved shared facts one listing entry mapped to

  @returns model profile ready for the provider dict

  @example
  ```ts
  const profile = factsToPiAiModelProfile({ facts: mapQwenTokenPlanIndividualModels({
    listing: qwenTokenPlanIndividualListingSnapshot.entries,
  })[0]! });
  ```
  */
export function factsToPiAiModelProfile({
  facts,
}: {
  readonly facts: QwenTokenPlanResolvedModelFacts;
}): PiAiModelProfile {
  return {
    id: facts.id,
    name: facts.name,
    contextWindow: facts.contextWindow,
    maxTokens: facts.maxTokens,
    input: [
      ...facts.input,
    ],
    ...(facts.reasoning
      ? (facts.efforts === undefined
        ? {}
        : { reasoningEfforts: facts.efforts, })
      : { reasoningEfforts: false as const, }),
    compat: {
      ...facts.compat,
    },
  };
}

//endregion Public API
