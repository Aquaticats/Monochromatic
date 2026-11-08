/**
 Checked-in NeuralWatt listing snapshot data, one segment. @module
 */

import type { NeuralWattListingEntry, } from './listing.ts';

//region Public API

/**
 Listing entries 17 to 20 of the checked-in snapshot, in endpoint
 order. Split into segments so each module stays within the repository
 file size limit; `neuralWattListingSnapshot` joins them.
 */
export const neuralWattListingSnapshotChunk5:
  readonly NeuralWattListingEntry[] = [
    {
      id: 'qwen3.6-35b-flex',
      max_model_len: 262_128,
      metadata: {
        display_name: 'Qwen3.6 35B (flex)',
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
            minimal: 'high',
          },
        },
        limits: {
          max_context_length: 262_128,
        },
        pricing: {
          input_per_million: 0.1885,
          output_per_million: 0.7475,
          cached_input_per_million: 0.01885,
          cached_output_per_million: null,
          currency: 'USD',
          pricing_tbd: false,
          service_tier: 'flex',
          flex_discount_multiplier: 0.65,
        },
      },
    },
    {
      id: 'deepseek-v4.1-flash',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'DeepSeek V4.1 Flash',
        capabilities: {
          reasoning: true,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'max',
              'xhigh',
              'high',
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
            medium: 'high',
            minimal: 'low',
          },
        },
        limits: {
          max_context_length: 1_048_560,
          max_output_tokens: 393_216,
        },
        pricing: {
          input_per_million: 0.15,
          output_per_million: 0.6,
          cached_input_per_million: 0.015,
          cached_output_per_million: null,
          currency: 'USD',
          pricing_tbd: false,
          service_tier: 'standard',
          flex_discount_multiplier: null,
        },
      },
    },
    {
      id: 'deepseek-v4.1-flash-flex',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'DeepSeek V4.1 Flash (flex)',
        capabilities: {
          reasoning: true,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'max',
              'xhigh',
              'high',
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
            medium: 'high',
            minimal: 'low',
          },
        },
        limits: {
          max_context_length: 1_048_560,
          max_output_tokens: 393_216,
        },
        pricing: {
          input_per_million: 0.0975,
          output_per_million: 0.39,
          cached_input_per_million: 0.00975,
          cached_output_per_million: null,
          currency: 'USD',
          pricing_tbd: false,
          service_tier: 'flex',
          flex_discount_multiplier: 0.65,
        },
      },
    },
    {
      id: 'deepseek-v4.1-flash-speed',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'DeepSeek V4.1 Flash (Speed)',
        capabilities: {
          reasoning: true,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'max',
              'xhigh',
              'high',
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
            medium: 'high',
            minimal: 'low',
          },
        },
        limits: {
          max_context_length: 1_048_560,
          max_output_tokens: 393_216,
        },
        pricing: {
          input_per_million: 0.15,
          output_per_million: 0.6,
          cached_input_per_million: 0.015,
          cached_output_per_million: null,
          currency: 'USD',
          pricing_tbd: false,
          service_tier: 'standard',
          flex_discount_multiplier: null,
        },
      },
    },
  ];

//endregion Public API
