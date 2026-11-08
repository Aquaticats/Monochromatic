/**
 Checked-in NeuralWatt listing snapshot data, one segment. @module
 */

import type { NeuralWattListingEntry, } from './listing.ts';

//region Public API

/**
 Listing entries 21 to 24 of the checked-in snapshot, in endpoint
 order. Split into segments so each module stays within the repository
 file size limit; `neuralWattListingSnapshot` joins them.
 */
export const neuralWattListingSnapshotChunk6:
  readonly NeuralWattListingEntry[] = [
    {
      id: 'deepseek-v4-flash',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'DeepSeek V4 Flash',
        capabilities: {
          reasoning: true,
          vision: false,
        },
        reasoning: {
          supported_efforts:
            [
              'max',
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
            xhigh: 'max',
            medium: 'high',
            low: 'high',
            minimal: 'high',
          },
        },
        limits: {
          max_context_length: 1_048_560,
          max_output_tokens: 393_216,
        },
      },
    },
    {
      id: 'deepseek-v4-flash-flex',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'DeepSeek V4 Flash (flex)',
        capabilities: {
          reasoning: true,
          vision: false,
        },
        reasoning: {
          supported_efforts:
            [
              'max',
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
            xhigh: 'max',
            medium: 'high',
            low: 'high',
            minimal: 'high',
          },
        },
        limits: {
          max_context_length: 1_048_560,
          max_output_tokens: 393_216,
        },
      },
    },
    {
      id: 'deepseek-v4-flash-speed',
      max_model_len: 1_048_560,
      metadata: {
        display_name: 'DeepSeek V4 Flash (Speed)',
        capabilities: {
          reasoning: true,
          vision: false,
        },
        reasoning: {
          supported_efforts:
            [
              'max',
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
            xhigh: 'max',
            medium: 'high',
            low: 'high',
            minimal: 'high',
          },
        },
        limits: {
          max_context_length: 1_048_560,
          max_output_tokens: 393_216,
        },
      },
    },
    {
      id: 'gemma-4-31b',
      max_model_len: 262_128,
      metadata: {
        display_name: 'Gemma 4 31B',
        capabilities: {
          reasoning: true,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
              'max',
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
            xhigh: 'max',
            high: 'max',
            medium: 'max',
            low: 'max',
            minimal: 'max',
          },
        },
        limits: {
          max_context_length: 262_128,
          max_output_tokens: 16_384,
        },
        pricing: {
          input_per_million: 0.144,
          output_per_million: 0.42,
          cached_input_per_million: 0.0144,
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
