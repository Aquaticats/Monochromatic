/**
 Checked-in NeuralWatt listing snapshot data, one segment. @module
 */

import type { NeuralWattListingEntry, } from './listing.ts';

//region Public API

/**
 Listing entries 1 to 4 of the checked-in snapshot, in endpoint
 order. Split into segments so each module stays within the repository
 file size limit; `neuralWattListingSnapshot` joins them.
 */
export const neuralWattListingSnapshotChunk1:
  readonly NeuralWattListingEntry[] = [
    {
      id: 'qwen-3.8-27b',
      max_model_len: 262_128,
      metadata: {
        display_name: 'Qwen 3.8 27B',
        capabilities: {
          reasoning: true,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'xhigh',
              'medium',
              'low',
              'none',
            ],
          accepted_efforts:
            [
              'max',
              'xhigh',
              'high',
              'medium',
              'low',
              'minimal',
              'none',
            ],
          effort_aliases: {
            max: 'xhigh',
            high: 'xhigh',
            minimal: 'low',
          },
        },
        limits: {
          max_context_length: 262_128,
          max_output_tokens: 131_072,
        },
        pricing: {
          input_per_million: 0.45,
          output_per_million: 3.2,
          cached_input_per_million: 0.25,
          cached_output_per_million: null,
          currency: 'USD',
          pricing_tbd: false,
          service_tier: 'standard',
          flex_discount_multiplier: null,
        },
      },
    },
    {
      id: 'qwen-3.8-27b-flex',
      max_model_len: 262_128,
      metadata: {
        display_name: 'Qwen 3.8 27B (flex)',
        capabilities: {
          reasoning: true,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'xhigh',
              'medium',
              'low',
              'none',
            ],
          accepted_efforts:
            [
              'max',
              'xhigh',
              'high',
              'medium',
              'low',
              'minimal',
              'none',
            ],
          effort_aliases: {
            max: 'xhigh',
            high: 'xhigh',
            minimal: 'low',
          },
        },
        limits: {
          max_context_length: 262_128,
          max_output_tokens: 131_072,
        },
        pricing: {
          input_per_million: 0.2925,
          output_per_million: 2.08,
          cached_input_per_million: 0.1625,
          cached_output_per_million: null,
          currency: 'USD',
          pricing_tbd: false,
          service_tier: 'flex',
          flex_discount_multiplier: 0.65,
        },
      },
    },
    {
      id: 'mimo-v2.6-pro',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'MiMo-V2.6-Pro',
        capabilities: {
          reasoning: true,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'high',
              'none',
            ],
          accepted_efforts:
            [
              'max',
              'xhigh',
              'high',
              'medium',
              'low',
              'minimal',
              'none',
            ],
          effort_aliases: {
            max: 'high',
            xhigh: 'high',
            medium: 'high',
            low: 'high',
            minimal: 'none',
          },
        },
        limits: {
          max_context_length: 1_048_560,
          max_output_tokens: 393_216,
        },
        pricing: {
          input_per_million: 0.435,
          output_per_million: 0.87,
          cached_input_per_million: 0.0036,
          cached_output_per_million: null,
          currency: 'USD',
          pricing_tbd: false,
          service_tier: 'standard',
          flex_discount_multiplier: null,
        },
      },
    },
    {
      id: 'mimo-v2.6-pro-flex',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'MiMo-V2.6-Pro (flex)',
        capabilities: {
          reasoning: true,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'high',
              'none',
            ],
          accepted_efforts:
            [
              'max',
              'xhigh',
              'high',
              'medium',
              'low',
              'minimal',
              'none',
            ],
          effort_aliases: {
            max: 'high',
            xhigh: 'high',
            medium: 'high',
            low: 'high',
            minimal: 'none',
          },
        },
        limits: {
          max_context_length: 1_048_560,
          max_output_tokens: 393_216,
        },
        pricing: {
          input_per_million: 0.28275,
          output_per_million: 0.5655,
          cached_input_per_million: 0.00234,
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
