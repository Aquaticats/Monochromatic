/**
 Model catalog mapping for the Qwen Token Plan Individual provider: listing
 entries the endpoint publishes to the provider profile the shipped pi-ai
 adapter serves. The listing-to-facts mapping (facts table, sibling fallback,
 media skip) lives in the shared core; this module assembles the provider dict
 and re-exports the shared entry types. @module
 */

import type {
  PiAiProviderProfile,
} from '@deepseek-ai/dsh-llm-pi-ai';
import {
  mapQwenTokenPlanIndividualModels,
  type QwenTokenPlanListingEntry,
  type QwenTokenPlanModelFacts,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/ts';
import {
  QWEN_TOKEN_PLAN_INDIVIDUAL_API,
  QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL,
  QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME,
  QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE,
} from './constants.ts';
import {
  factsToPiAiModelProfile,
} from './model-profile.ts';

export {
  expandQwenTokenPlanThinkingLevelMap,
  mapQwenTokenPlanIndividualModels,
  qwenTokenPlanIndividualFacts,
  type QwenTokenPlanCompatPins,
  type QwenTokenPlanEffortsDict,
  type QwenTokenPlanListingEntry,
  type QwenTokenPlanModelFacts,
  type QwenTokenPlanModelKind,
  type QwenTokenPlanResolvedModelFacts,
  type QwenTokenPlanThinkingLevelMap,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/ts';

//region Public API

/**
 Provider dict serving every serviceable listing entry on one route: the
 qwen-token-plan-individual route, pinning credential, protocol, endpoint, and
 the wire-compat pins each resolved model carries, so nothing else has to.

  @param listing - listing entries read from the endpoint or the snapshot

  @param facts - facts table the mapping resolves against, defaulted to the
  shared one

  @returns provider dict keyed by the route constant

  @example
  ```ts
  const providers = mapQwenTokenPlanIndividualProviders({
    listing: qwenTokenPlanIndividualListingSnapshot.entries,
  });
  ```
  */
export function mapQwenTokenPlanIndividualProviders({
  listing,
  facts,
}: {
  readonly listing: readonly QwenTokenPlanListingEntry[];
  readonly facts?: readonly QwenTokenPlanModelFacts[];
}): Record<string, PiAiProviderProfile> {
  /**
   Resolved facts every serviceable listing entry earns from the shared
   mapping, before the projection onto the served profile shape.
   */
  const resolvedModels = facts === undefined
    ? mapQwenTokenPlanIndividualModels({ listing, })
    : mapQwenTokenPlanIndividualModels({
      listing,
      facts,
    },);
  return {
    [QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE]: {
      apiKeyEnv: QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME,
      api: QWEN_TOKEN_PLAN_INDIVIDUAL_API,
      baseURL: QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL,
      models: resolvedModels
        .map(function byFacts(resolved) {
          return factsToPiAiModelProfile({
            facts: resolved,
          },);
        },),
    },
  };
}

//endregion Public API
