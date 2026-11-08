/**
 Checked-in NeuralWatt listing snapshot data, one segment. @module
 */

import type { NeuralWattListingEntry, } from './listing.ts';

//region Public API

/**
 Listing entries 13 to 16 of the checked-in snapshot, in endpoint
 order. Split into segments so each module stays within the repository
 file size limit; `neuralWattListingSnapshot` joins them.
 */
export const neuralWattListingSnapshotChunk4:
  readonly NeuralWattListingEntry[] = [
    {
      id: 'nw-small',
      max_model_len: 262_128,
      metadata: {
        display_name: 'Neuralwatt Small',
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
      id: 'nw-small-flex',
      max_model_len: 262_128,
      metadata: {
        display_name: 'Neuralwatt Small (flex)',
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
      id: 'qwen3.6-35b',
      max_model_len: 262_128,
      metadata: {
        display_name: 'Qwen3.6 35B',
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
          input_per_million: 0.29,
          output_per_million: 1.15,
          cached_input_per_million: 0.029,
          cached_output_per_million: null,
          currency: 'USD',
          pricing_tbd: false,
          service_tier: 'standard',
          flex_discount_multiplier: null,
        },
      },
    },
    {
      id: 'qwen3.6-35b-fast',
      max_model_len: 262_128,
      metadata: {
        display_name: 'Qwen3.6 35B Fast',
        capabilities: {
          reasoning: false,
          vision: true,
        },
        reasoning: {
          supported_efforts:
            [
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
            max: 'none',
            xhigh: 'none',
            high: 'none',
            medium: 'none',
            low: 'none',
            minimal: 'none',
          },
        },
        limits: {
          max_context_length: 262_128,
        },
        pricing: {
          input_per_million: 0.29,
          output_per_million: 1.15,
          cached_input_per_million: 0.029,
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
