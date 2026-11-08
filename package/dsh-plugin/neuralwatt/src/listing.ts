/**
 Model catalog mapping for the NeuralWatt provider: listing entries the
 service publishes to provider profiles the shipped pi-ai adapter serves.
 The listing-to-facts mapping lives in the shared core; this module assembles
 the provider dict and re-exports the shared entry types. @module
 */

import type {
  PiAiProviderProfile,
} from '@deepseek-ai/dsh-llm-pi-ai';
import {
  mapNeuralWattListing,
  type NeuralWattListingEntry,
} from '@monochromatic-dev/agent-harness-shared-neuralwatt-listing/ts';
import {
  NEURALWATT_API,
  NEURALWATT_API_KEY_ENV,
  NEURALWATT_BASE_URL,
  NEURALWATT_ROUTE,
} from './constants.ts';
import {
  factsToPiAiModelProfile,
} from './model-profile.ts';

export {
  expandThinkingLevelMap,
  isRecord,
  type NeuralWattEffortsDict,
  type NeuralWattListingEntry,
  type NeuralWattThinkingLevelMap,
} from '@monochromatic-dev/agent-harness-shared-neuralwatt-listing/ts';

//region Public API

/**
 Provider dict serving every serviceable listing entry on one route: the
 neuralwatt route, pinning credential, protocol, endpoint, and effort
 dispatch so nothing else has to.

  @param listing - listing entries read from the service endpoint

  @returns provider dict keyed by the route constant

  @example
  ```ts
  const providers = mapNeuralWattProviders({
    listing: neuralWattListingSnapshot,
  });
  ```
  */
export function mapNeuralWattProviders({
  listing,
}: {
  readonly listing: readonly NeuralWattListingEntry[];
}): Record<string, PiAiProviderProfile> {
  return {
    [NEURALWATT_ROUTE]: {
      apiKeyEnv: NEURALWATT_API_KEY_ENV,
      api: NEURALWATT_API,
      baseURL: NEURALWATT_BASE_URL,
      // NeuralWatt rejects the developer role (unsupported_message_role on
      // mimo-v2.6-pro), and pi-ai URL auto-detection enables it for reasoning
      // models, so pin the classic system role for every served model.
      compat: {
        supportsReasoningEffort: true,
        supportsDeveloperRole: false
      },
      models: mapNeuralWattListing({ listing, },)
        .map(function byFacts(facts) {
          return factsToPiAiModelProfile({ facts, },);
        },),
    },
  };
}

//endregion Public API
