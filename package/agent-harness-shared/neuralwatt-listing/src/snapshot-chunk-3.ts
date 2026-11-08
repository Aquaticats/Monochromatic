/**
 Checked-in NeuralWatt listing snapshot data, one segment. @module
 */

import type { NeuralWattListingEntry, } from './listing.ts';

//region Public API

/**
 Listing entries 9 to 12 of the checked-in snapshot, in endpoint
 order. Split into segments so each module stays within the repository
 file size limit; `neuralWattListingSnapshot` joins them.
 */
export const neuralWattListingSnapshotChunk3:
  readonly NeuralWattListingEntry[] = [
    {
      id: 'glm-5.3',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'GLM 5.3',
        capabilities: {
          reasoning: true,
          vision: false,
        },
        reasoning: {
          supported_efforts:
            [
              'max',
              'high',
              'low',
            ],
          accepted_efforts:
            [
              'max',
              'xhigh',
              'high',
              'medium',
              'low',
              'minimal',
            ],
          effort_aliases: {
            xhigh: 'max',
            medium: 'high',
            minimal: 'low',
          },
        },
        limits: {
          max_context_length: 1_048_560,
        },
        pricing: {
          input_per_million: 1.45,
          output_per_million: 4.5,
          cached_input_per_million: 0.145,
          cached_output_per_million: null,
          currency: 'USD',
          pricing_tbd: false,
          service_tier: 'standard',
          flex_discount_multiplier: null,
        },
      },
    },
    {
      id: 'glm-5.3-flex',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'GLM 5.3 (flex)',
        capabilities: {
          reasoning: true,
          vision: false,
        },
        reasoning: {
          supported_efforts:
            [
              'max',
              'high',
              'low',
            ],
          accepted_efforts:
            [
              'max',
              'xhigh',
              'high',
              'medium',
              'low',
              'minimal',
            ],
          effort_aliases: {
            xhigh: 'max',
            medium: 'high',
            minimal: 'low',
          },
        },
        limits: {
          max_context_length: 1_048_560,
        },
        pricing: {
          input_per_million: 0.9425,
          output_per_million: 2.925,
          cached_input_per_million: 0.09425,
          cached_output_per_million: null,
          currency: 'USD',
          pricing_tbd: false,
          service_tier: 'flex',
          flex_discount_multiplier: 0.65,
        },
      },
    },
    {
      id: 'glm-5.3-flash',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'GLM-5.3 Flash',
        capabilities: {
          reasoning: true,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'max',
              'high',
              'low',
            ],
          accepted_efforts:
            [
              'max',
              'xhigh',
              'high',
              'medium',
              'low',
              'minimal',
            ],
          effort_aliases: {
            xhigh: 'max',
            medium: 'high',
            minimal: 'low',
          },
        },
        limits: {
          max_context_length: 1_048_560,
        },
        pricing: {
          input_per_million: 0.15,
          output_per_million: 0.5,
          cached_input_per_million: 0.03,
          cached_output_per_million: null,
          currency: 'USD',
          pricing_tbd: false,
          service_tier: 'standard',
          flex_discount_multiplier: null,
        },
      },
    },
    {
      id: 'glm-5.3-flash-flex',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'GLM-5.3 Flash (flex)',
        capabilities: {
          reasoning: true,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'max',
              'high',
              'low',
            ],
          accepted_efforts:
            [
              'max',
              'xhigh',
              'high',
              'medium',
              'low',
              'minimal',
            ],
          effort_aliases: {
            xhigh: 'max',
            medium: 'high',
            minimal: 'low',
          },
        },
        limits: {
          max_context_length: 1_048_560,
        },
        pricing: {
          input_per_million: 0.0975,
          output_per_million: 0.325,
          cached_input_per_million: 0.0195,
          cached_output_per_million: null,
          currency: 'USD',
          pricing_tbd: false,
          service_tier: 'flex',
          flex_discount_multiplier: 0.65,
        },
      },
    },
  ];

//endregion Public API
